//! Request-scoped capture of router tool steps for timeout and error surfacing.
//!
//! Filled during `run_tool_loop`; safe to read after `tokio::time::timeout` drops the
//! Ollama future because state lives in an `Arc<Mutex<…>>`.

use std::sync::{Arc, Mutex};

const ASSISTANT_SNIPPET_MAX: usize = 200;
const MAX_TOOL_LINES_IN_SUMMARY: usize = 12;

#[derive(Debug, Default)]
pub struct PartialProgressInner {
    pub tool_runs: Vec<(String, String, bool)>,
    pub last_assistant_snippet: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PartialProgressCapture {
    inner: Arc<Mutex<PartialProgressInner>>,
}

impl PartialProgressCapture {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PartialProgressInner::default())),
        }
    }

    pub(crate) fn record_tool_run(&self, name: &str, input_summary: &str, ok: bool) {
        if let Ok(mut g) = self.inner.lock() {
            g.tool_runs
                .push((name.to_string(), input_summary.to_string(), ok));
        }
    }

    pub(crate) fn set_last_assistant_text(&self, text: &str) {
        let t = text.trim();
        if t.is_empty() {
            return;
        }
        let over = t.chars().count() > ASSISTANT_SNIPPET_MAX;
        let snippet: String = t.chars().take(ASSISTANT_SNIPPET_MAX).collect();
        let suffix = if over { "…" } else { "" };
        if let Ok(mut g) = self.inner.lock() {
            g.last_assistant_snippet = Some(format!("{snippet}{suffix}"));
        }
    }

    /// Human-readable block for Discord / schedule failure lines (no raw tool result bodies).
    /// Uses `try_lock` so a timeout handler never blocks on the cancelled Ollama task.
    pub fn format_user_summary(&self) -> Option<String> {
        let g = self.inner.try_lock().ok()?;
        if g.tool_runs.is_empty() && g.last_assistant_snippet.is_none() {
            return None;
        }
        let mut out = String::new();
        let n = g.tool_runs.len();
        if n > 0 {
            out.push_str(&format!(
                "[Partial progress: {n} tool call(s) before interruption]\n"
            ));
            for (i, (name, summary, ok)) in g.tool_runs.iter().enumerate() {
                if i >= MAX_TOOL_LINES_IN_SUMMARY {
                    let rest = n - MAX_TOOL_LINES_IN_SUMMARY;
                    out.push_str(&format!("- … and {rest} more tool step(s)\n"));
                    break;
                }
                let status = if *ok { "ok" } else { "failed" };
                out.push_str(&format!("- {name} ({status}): {summary}\n"));
            }
        }
        if let Some(ref s) = g.last_assistant_snippet {
            if !s.is_empty() {
                out.push_str("Last assistant text: ");
                out.push_str(s);
                out.push('\n');
            }
        }
        let used_browser = g
            .tool_runs
            .iter()
            .any(|(name, _, _)| name.starts_with("BROWSER_"));
        drop(g);
        if used_browser {
            if let Some(snapshot) = crate::browser_agent::get_last_browser_state_snapshot() {
                if let Some(line) = browser_snapshot_one_liner(&snapshot) {
                    out.push_str("Browser: ");
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }
        let t = out.trim_end().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }
}

fn browser_snapshot_one_liner(snapshot: &str) -> Option<String> {
    let mut current: Option<String> = None;
    let mut title: Option<String> = None;
    for line in snapshot.lines() {
        if let Some(rest) = line.strip_prefix("Current page: ") {
            current = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Title: ") {
            title = Some(rest.trim().to_string());
        }
    }
    match (current, title) {
        (Some(u), Some(t)) => Some(format!("{u} — {t}")),
        (Some(u), None) => Some(u),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_summary_lists_tools_and_snippet() {
        let c = PartialProgressCapture::new();
        c.record_tool_run("FETCH_URL", "https://example.com", true);
        c.record_tool_run("RUN_CMD", "date", false);
        c.set_last_assistant_text("Checking the page next.");
        let s = c.format_user_summary().expect("summary");
        assert!(s.contains("[Partial progress: 2 tool call(s)"));
        assert!(s.contains("FETCH_URL (ok):"));
        assert!(s.contains("RUN_CMD (failed):"));
        assert!(s.contains("Last assistant text:"));
    }
}
