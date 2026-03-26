//! General tool loop guard — detects repeated tool invocations and cycles within
//! a single agent-router request to prevent the model from getting stuck.
//!
//! **Legacy mode (default):** Complements per-tool dedup with cross-tool pattern
//! detection (cycles like A→B→A→B) and blocks the same (tool, arg) after 3 calls.
//!
//! **Optional mode** (`toolLoopDetection` in config): Hash-based repeat detection
//! inspired by OpenClaw — configurable history window, warning appended to tool
//! results, critical threshold stops the run with a user-facing message. When
//! disabled, legacy behaviour is unchanged.

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use sha2::{Digest, Sha256};

use crate::config::ToolLoopDetectionConfig;

/// Maximum number of times the exact same (tool, arg) can be invoked (legacy mode).
const MAX_SAME_CALL: u32 = 3;
/// Maximum cycle length to detect (e.g. 4 means patterns up to ABCD→ABCD).
const MAX_CYCLE_LEN: usize = 4;
/// Minimum history length before cycle detection kicks in.
const MIN_HISTORY_FOR_CYCLE: usize = 4;

const LOOP_WARN_SUFFIX: &str = "\n\n[SYSTEM] Same tool and arguments were repeated several times in this run; consider a different approach or call DONE.";

const LOOP_CRITICAL_USER_MSG: &str = "Limit: tool-loop repeat detection — Same action was repeated too many times in this session. Try rephrasing your request, adjusting `toolLoopDetection` thresholds, or starting a new topic.";

/// After a tool runs, optional loop detection may ask to append a warning or stop the run.
pub(crate) enum ToolLoopAfterResult {
    None,
    Warning(String),
    Critical(String),
}

/// Tracks tool invocations within a single request.
pub(crate) struct ToolLoopGuard {
    inner: ToolLoopGuardInner,
}

enum ToolLoopGuardInner {
    Legacy(LegacyState),
    Optional(OptionalState),
}

struct LegacyState {
    history: Vec<(String, u64)>,
    counts: HashMap<(String, u64), u32>,
}

struct OptionalState {
    config: ToolLoopDetectionConfig,
    history: VecDeque<OptionalHistoryEntry>,
}

#[derive(Clone, PartialEq, Eq)]
struct OptionalHistoryEntry {
    tool: String,
    arg_hash: [u8; 32],
}

impl ToolLoopGuard {
    pub fn new(loop_detection: Option<ToolLoopDetectionConfig>) -> Self {
        Self {
            inner: match loop_detection {
                Some(cfg) => ToolLoopGuardInner::Optional(OptionalState {
                    config: cfg,
                    history: VecDeque::new(),
                }),
                None => ToolLoopGuardInner::Legacy(LegacyState {
                    history: Vec::new(),
                    counts: HashMap::new(),
                }),
            },
        }
    }

    /// Before executing a tool: legacy — block on repeat/cycle; optional — cycle check only.
    pub fn before_tool_execute(&mut self, tool: &str, arg: &str) -> Option<String> {
        match &mut self.inner {
            ToolLoopGuardInner::Legacy(s) => s.record_and_check(tool, arg),
            ToolLoopGuardInner::Optional(s) => s.begin_tool(tool, arg),
        }
    }

    /// After the tool result is known (optional mode only).
    pub fn after_tool_result(&mut self, tool: &str, arg: &str, result: &str) -> ToolLoopAfterResult {
        match &mut self.inner {
            ToolLoopGuardInner::Legacy(_) => ToolLoopAfterResult::None,
            ToolLoopGuardInner::Optional(s) => s.after_result(tool, arg, result),
        }
    }
}

impl LegacyState {
    fn record_and_check(&mut self, tool: &str, arg: &str) -> Option<String> {
        let arg_hash = hash_arg_legacy(arg);
        let key = (tool.to_string(), arg_hash);

        let count = self.counts.entry(key.clone()).or_insert(0);
        *count += 1;

        self.history.push(key.clone());

        if *count > MAX_SAME_CALL {
            return Some(format!(
                "Loop detected: {} has been called {} times with the same argument. \
                 Reply with your answer or call DONE.",
                tool, count
            ));
        }

        if let Some(cycle_len) = detect_cycle(&self.history) {
            return Some(format!(
                "Cycle detected: the last {} tool calls repeat a previous pattern. \
                 Break the cycle — try a different approach or reply with DONE.",
                cycle_len
            ));
        }

        None
    }
}

impl OptionalState {
    fn begin_tool(&mut self, tool: &str, arg: &str) -> Option<String> {
        let arg_hash = stable_arg_hash(tool, arg);
        let entry = OptionalHistoryEntry {
            tool: tool.to_string(),
            arg_hash,
        };
        self.history.push_back(entry);
        while self.history.len() > self.config.history_size {
            self.history.pop_front();
        }

        let keys: Vec<(String, [u8; 32])> = self
            .history
            .iter()
            .map(|e| (e.tool.clone(), e.arg_hash))
            .collect();
        if let Some(cycle_len) = detect_cycle_on_keys(&keys) {
            self.history.pop_back();
            return Some(format!(
                "Cycle detected: the last {} tool calls repeat a previous pattern. \
                 Break the cycle — try a different approach or reply with DONE.",
                cycle_len
            ));
        }

        None
    }

    fn after_result(&mut self, tool: &str, arg: &str, _result: &str) -> ToolLoopAfterResult {
        let expected = stable_arg_hash(tool, arg);
        if let Some(last) = self.history.back() {
            if last.tool != tool || last.arg_hash != expected {
                tracing::warn!(
                    "tool loop detection: history tail mismatch for {} (optional guard)",
                    tool
                );
                return ToolLoopAfterResult::None;
            }
        } else {
            return ToolLoopAfterResult::None;
        }

        let same_count = self
            .history
            .iter()
            .filter(|e| e.tool == tool && e.arg_hash == expected)
            .count() as u32;

        let w = self.config.warning_threshold;
        let c = self.config.critical_threshold;

        if same_count >= c {
            tracing::info!(
                "Agent router: tool-loop detection critical — tool={} same_signature_count={} (threshold {})",
                tool,
                same_count,
                c
            );
            return ToolLoopAfterResult::Critical(LOOP_CRITICAL_USER_MSG.to_string());
        }
        if same_count >= w {
            tracing::info!(
                "Agent router: tool-loop detection warning — tool={} same_signature_count={} (threshold {})",
                tool,
                same_count,
                w
            );
            return ToolLoopAfterResult::Warning(LOOP_WARN_SUFFIX.to_string());
        }

        ToolLoopAfterResult::None
    }
}

/// Stable SHA-256 of normalized arguments for optional loop detection.
pub(crate) fn stable_arg_hash(tool: &str, arg: &str) -> [u8; 32] {
    let normalized = normalize_arg_for_loop_detection(tool, arg);
    let mut hasher = Sha256::new();
    hasher.update(tool.as_bytes());
    hasher.update(b"\0");
    hasher.update(normalized.as_bytes());
    let out = hasher.finalize();
    out.into()
}

fn normalize_arg_for_loop_detection(tool: &str, arg: &str) -> String {
    let s = arg.trim();
    let lower_tool = tool.to_ascii_uppercase();
    let urlish_tool = matches!(
        lower_tool.as_str(),
        "FETCH_URL"
            | "BROWSER_NAVIGATE"
            | "BROWSER_SCREENSHOT"
            | "BROWSER_RELOAD"
            | "REDMINE_API"
            | "DISCORD_API"
    );
    if urlish_tool {
        if let Ok(u) = url::Url::parse(s) {
            let scheme = u.scheme().to_lowercase();
            if scheme == "http" || scheme == "https" {
                let host = u
                    .host_str()
                    .map(|h| h.to_lowercase())
                    .unwrap_or_default();
                let mut path = u.path().to_string();
                while path.len() > 1 && path.ends_with('/') {
                    path.pop();
                }
                if path.is_empty() {
                    path.push('/');
                }
                let port = u.port().map(|p| format!(":{}", p)).unwrap_or_default();
                let query = u.query().map(|q| format!("?{}", q)).unwrap_or_default();
                let fragment = u.fragment().map(|f| format!("#{}", f)).unwrap_or_default();
                return format!("{scheme}://{host}{port}{path}{query}{fragment}");
            }
        }
    }
    s.to_string()
}

fn hash_arg_legacy(arg: &str) -> u64 {
    let normalized = arg.trim();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    normalized.hash(&mut hasher);
    hasher.finish()
}

fn detect_cycle(history: &[(String, u64)]) -> Option<usize> {
    let len = history.len();
    if len < MIN_HISTORY_FOR_CYCLE {
        return None;
    }
    for cycle_len in 2..=MAX_CYCLE_LEN.min(len / 2) {
        let tail_start = len - cycle_len;
        let prev_start = tail_start - cycle_len;
        if history[prev_start..tail_start] == history[tail_start..len] {
            return Some(cycle_len);
        }
    }
    None
}

fn detect_cycle_on_keys(history: &[(String, [u8; 32])]) -> Option<usize> {
    let len = history.len();
    if len < MIN_HISTORY_FOR_CYCLE {
        return None;
    }
    for cycle_len in 2..=MAX_CYCLE_LEN.min(len / 2) {
        let tail_start = len - cycle_len;
        let prev_start = tail_start - cycle_len;
        if history[prev_start..tail_start] == history[tail_start..len] {
            let window = &history[prev_start..len];
            let distinct: HashSet<(&str, &[u8; 32])> =
                window.iter().map(|(t, h)| (t.as_str(), h)).collect();
            // Pure AAAA… repeats are handled by the repeat counter, not ping-pong detection.
            if distinct.len() < 2 {
                continue;
            }
            return Some(cycle_len);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy() -> ToolLoopGuard {
        ToolLoopGuard::new(None)
    }

    fn optional_cfg(w: u32, c: u32) -> ToolLoopDetectionConfig {
        ToolLoopDetectionConfig {
            history_size: 25,
            warning_threshold: w,
            critical_threshold: c,
        }
    }

    #[test]
    fn allows_first_call() {
        let mut guard = legacy();
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://example.com")
            .is_none());
    }

    #[test]
    fn allows_up_to_max_same_call() {
        let mut guard = legacy();
        for _ in 0..MAX_SAME_CALL {
            assert!(guard
                .before_tool_execute("FETCH_URL", "https://example.com")
                .is_none());
        }
        let result = guard.before_tool_execute("FETCH_URL", "https://example.com");
        assert!(result.is_some());
        assert!(result.unwrap().contains("Loop detected"));
    }

    #[test]
    fn different_args_are_independent() {
        let mut guard = legacy();
        for i in 0..10 {
            assert!(guard
                .before_tool_execute("FETCH_URL", &format!("https://example.com/{}", i))
                .is_none());
        }
    }

    #[test]
    fn different_tools_same_arg_have_independent_counts() {
        let mut guard = legacy();
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://example.com")
            .is_none());
        assert!(guard.before_tool_execute("RUN_CMD", "uptime").is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://example.com")
            .is_none());
        assert!(guard.before_tool_execute("BRAVE_SEARCH", "test").is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://example.com")
            .is_none());
        assert!(guard
            .before_tool_execute("BROWSER_NAVIGATE", "https://example.com")
            .is_none());
        assert!(guard
            .before_tool_execute("BROWSER_NAVIGATE", "https://example.com")
            .is_none());
    }

    #[test]
    fn detects_ab_ab_cycle() {
        let mut guard = legacy();
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
        assert!(guard.before_tool_execute("BRAVE_SEARCH", "test").is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
        let result = guard.before_tool_execute("BRAVE_SEARCH", "test");
        assert!(result.is_some());
        assert!(result.unwrap().contains("Cycle detected"));
    }

    #[test]
    fn detects_abc_abc_cycle() {
        let mut guard = legacy();
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
        assert!(guard.before_tool_execute("BRAVE_SEARCH", "query").is_none());
        assert!(guard
            .before_tool_execute("BROWSER_NAVIGATE", "https://b.com")
            .is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
        assert!(guard.before_tool_execute("BRAVE_SEARCH", "query").is_none());
        let result = guard.before_tool_execute("BROWSER_NAVIGATE", "https://b.com");
        assert!(result.is_some());
        assert!(result.unwrap().contains("Cycle detected"));
    }

    #[test]
    fn no_false_positive_on_short_history() {
        let mut guard = legacy();
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
    }

    #[test]
    fn no_false_positive_on_varied_pattern() {
        let mut guard = legacy();
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
        assert!(guard.before_tool_execute("BRAVE_SEARCH", "q1").is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://b.com")
            .is_none());
        assert!(guard.before_tool_execute("BRAVE_SEARCH", "q2").is_none());
    }

    #[test]
    fn whitespace_normalization_legacy() {
        let mut guard = legacy();
        assert!(guard
            .before_tool_execute("FETCH_URL", "  https://a.com  ")
            .is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "https://a.com")
            .is_none());
        assert!(guard
            .before_tool_execute("FETCH_URL", "  https://a.com  ")
            .is_none());
        let result = guard.before_tool_execute("FETCH_URL", "https://a.com");
        assert!(result.is_some());
    }

    #[test]
    fn optional_url_host_case_folds() {
        let h1 = stable_arg_hash("FETCH_URL", "https://EXAMPLE.com/path/");
        let h2 = stable_arg_hash("FETCH_URL", "https://example.com/path");
        assert_eq!(h1, h2);
    }

    #[test]
    fn optional_warning_then_critical() {
        let mut guard = ToolLoopGuard::new(Some(optional_cfg(3, 5)));
        for i in 1..=3 {
            assert!(guard.before_tool_execute("RUN_CMD", "echo hi").is_none());
            let r = guard.after_tool_result("RUN_CMD", "echo hi", &format!("out{}", i));
            if i < 3 {
                assert!(matches!(r, ToolLoopAfterResult::None));
            } else {
                assert!(matches!(r, ToolLoopAfterResult::Warning(_)));
            }
        }
        assert!(guard.before_tool_execute("RUN_CMD", "echo hi").is_none());
        let r = guard.after_tool_result("RUN_CMD", "echo hi", "out4");
        assert!(matches!(r, ToolLoopAfterResult::Warning(_)));
        assert!(guard.before_tool_execute("RUN_CMD", "echo hi").is_none());
        let r = guard.after_tool_result("RUN_CMD", "echo hi", "out5");
        assert!(matches!(r, ToolLoopAfterResult::Critical(_)));
    }

    #[test]
    fn optional_cycle_still_blocks() {
        let mut guard = ToolLoopGuard::new(Some(optional_cfg(10, 20)));
        assert!(guard.before_tool_execute("FETCH_URL", "https://a.com").is_none());
        assert!(guard.before_tool_execute("BRAVE_SEARCH", "q").is_none());
        assert!(guard.before_tool_execute("FETCH_URL", "https://a.com").is_none());
        let r = guard.before_tool_execute("BRAVE_SEARCH", "q");
        assert!(r.is_some());
        assert!(r.unwrap().contains("Cycle detected"));
    }
}
