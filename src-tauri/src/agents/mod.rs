//! Agents: directory-based LLM agents under ~/.mac-stats/agents/agent-<id>/
//!
//! Each agent has agent.json (name, optional slug, model, orchestrator, enabled),
//! required skill.md, optional soul.md and mood.md. Soul: use agent-<id>/soul.md if present,
//! else ~/.mac-stats/agents/soul.md. Combined prompt order: soul → mood → skill.
//! Used by the Ollama tool loop (AGENT: <selector> [task]) and by the agent-test CLI.

pub mod cli;
pub mod watch;

use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

/// True if shared `soul.md` exists and contains non-whitespace text.
fn shared_soul_file_nonempty() -> bool {
    let soul_path = Config::soul_file_path();
    std::fs::read_to_string(&soul_path)
        .ok()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

fn log_shared_soul_presence() {
    let soul_path = Config::soul_file_path();
    let soul_ok = shared_soul_file_nonempty();
    info!(
        "Agents: shared soul {:?}: {}",
        soul_path,
        if soul_ok {
            "present"
        } else {
            "missing (will write default on first use)"
        }
    );
}

/// Per-agent config from agent.json. Name is required; others optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    /// Explicit model override. When set and available, used as-is.
    #[serde(default)]
    pub model: Option<String>,
    /// Desired model role: "general", "code", or "small".
    /// Used to auto-resolve the best available model when `model` is absent or unavailable.
    #[serde(default)]
    pub model_role: Option<String>,
    #[serde(default)]
    pub orchestrator: Option<bool>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
    /// Max tool iterations per request (AGENT, TASK_*, etc.). Default 15 when missing.
    #[serde(default)]
    pub max_tool_iterations: Option<u32>,
}

/// One loaded agent: id from directory name, config from agent.json, combined prompt from soul+mood+skill.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    /// Effective model: either explicit override from agent.json or resolved from model_role.
    pub model: Option<String>,
    /// Declared model role from agent.json (e.g., "general", "code", "small").
    pub model_role: Option<String>,
    pub orchestrator: bool,
    pub enabled: bool,
    pub combined_prompt: String,
    /// Same as combined_prompt but without long-term memory (soul+mood+skill only). Used in Discord guild / having_fun so personal memory is not loaded.
    pub combined_prompt_without_memory: String,
    /// Max tool iterations per request when this agent is in charge. Default 15.
    pub max_tool_iterations: u32,
}

/// Load all agents from ~/.mac-stats/agents/. Each subdirectory named agent-<id> is one agent.
/// Requires agent.json and skill.md; soul.md and mood.md are optional. Disabled agents are skipped.
/// Logs and skips invalid entries.
pub fn load_agents() -> Vec<Agent> {
    let dir = Config::agents_dir();
    if !dir.is_dir() {
        info!("Agents: directory missing, path={:?}", dir);
        log_shared_soul_presence();
        return Vec::new();
    }

    let read_dir = match std::fs::read_dir(&dir) {
        Ok(r) => r,
        Err(e) => {
            warn!("Agents: could not read directory {:?}: {}", dir, e);
            return Vec::new();
        }
    };

    let mut agents = Vec::new();
    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let id = match dir_name.strip_prefix("agent-") {
            Some(rest) if !rest.is_empty() => rest.to_string(),
            _ => continue,
        };
        if let Some(agent) = load_one_agent(&path, &id) {
            if agent.enabled {
                agents.push(agent);
            } else {
                debug!("Agents: skipping disabled agent {:?}", id);
            }
        }
    }

    agents.sort_by(|a, b| a.id.cmp(&b.id));

    if agents.is_empty() {
        info!("Agents: no enabled agents in {:?}", dir);
        log_shared_soul_presence();
    } else {
        let list: String = agents
            .iter()
            .map(|a| a.slug.as_deref().unwrap_or(a.name.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        info!("Agents: loaded {} from {:?}: {}", agents.len(), dir, list);
        log_shared_soul_presence();
    }

    // Auto-resolve model assignments from cached catalog (if available)
    if let Some(catalog) = crate::ollama::models::get_global_catalog() {
        resolve_agent_models(&mut agents, &catalog);
    } else if !agents.is_empty() {
        info!(
            "Agents: model catalog not yet available, model_role resolution skipped (Ollama may still be starting)"
        );
    }

    agents
}

/// Load all agents (enabled and disabled). Used by UI/CRUD to list and manage agents.
pub fn load_all_agents() -> Vec<Agent> {
    let dir = Config::agents_dir();
    if !dir.is_dir() {
        return Vec::new();
    }
    let read_dir = match std::fs::read_dir(&dir) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut agents: Vec<Agent> = read_dir
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|path| {
            let dir_name = path.file_name().and_then(|s| s.to_str())?;
            let id = dir_name.strip_prefix("agent-").filter(|s| !s.is_empty())?;
            load_one_agent(&path, id)
        })
        .collect();
    agents.sort_by(|a, b| a.id.cmp(&b.id));
    agents
}

/// Return the directory path for an agent id if it exists.
pub fn get_agent_dir(id: &str) -> Option<std::path::PathBuf> {
    let path = Config::agents_dir().join(format!("agent-{}", id));
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

/// Per-agent `soul.md` wins when non-empty; otherwise use shared `~/.mac-stats/agents/soul.md`.
/// Empty per-agent file or missing file → shared fallback. Locks 022 §F3 (no double soul).
fn agent_soul_or_shared(per_agent_trimmed: Option<String>, shared_soul: String) -> Option<String> {
    let fallback = if shared_soul.is_empty() {
        None
    } else {
        Some(shared_soul)
    };
    per_agent_trimmed.filter(|s| !s.is_empty()).or(fallback)
}

fn load_one_agent(dir: &Path, id: &str) -> Option<Agent> {
    let config_path = dir.join("agent.json");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| {
            warn!("Agents: could not read {:?}: {}", config_path, e);
        })
        .ok()?;
    let config: AgentConfig = serde_json::from_str(&content)
        .map_err(|e| {
            warn!("Agents: invalid JSON in {:?}: {}", config_path, e);
        })
        .ok()?;

    let skill_path = dir.join("skill.md");
    let skill = std::fs::read_to_string(&skill_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if skill.is_empty() {
        warn!("Agents: missing or empty skill.md for agent {:?}", id);
        return None;
    }

    // Soul: per-agent soul.md if present, else shared ~/.mac-stats/agents/soul.md. Always include in context when available.
    let soul_path = dir.join("soul.md");
    let trimmed_per_agent = std::fs::read_to_string(&soul_path)
        .ok()
        .map(|s| s.trim().to_string());
    let had_nonempty_per_agent = trimmed_per_agent
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let soul = agent_soul_or_shared(trimmed_per_agent, Config::load_soul_content());
    if soul.is_some() {
        if had_nonempty_per_agent {
            debug!("Agents: agent {} using soul from {:?}", id, soul_path);
        } else {
            debug!(
                "Agents: agent {} using shared soul from {:?}",
                id,
                Config::soul_file_path()
            );
        }
    }
    let mood = std::fs::read_to_string(dir.join("mood.md"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Memory: global memory.md (shared) + per-agent memory.md, concatenated
    let global_memory = std::fs::read_to_string(Config::memory_file_path())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let agent_memory = std::fs::read_to_string(dir.join("memory.md"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let memory = match (global_memory, agent_memory) {
        (Some(g), Some(a)) => {
            debug!("Agents: agent {} using global + agent memory", id);
            Some(format!("{}\n\n{}", g, a))
        }
        (Some(g), None) => {
            debug!("Agents: agent {} using global memory only", id);
            Some(g)
        }
        (None, Some(a)) => {
            debug!("Agents: agent {} using agent memory only", id);
            Some(a)
        }
        (None, None) => None,
    };

    let combined_prompt =
        build_combined_prompt(soul.as_deref(), mood.as_deref(), memory.as_deref(), &skill);
    let combined_prompt_without_memory =
        build_combined_prompt(soul.as_deref(), mood.as_deref(), None, &skill);

    let max_tool_iterations = config.max_tool_iterations.unwrap_or(15);
    Some(Agent {
        id: id.to_string(),
        name: config.name,
        slug: config.slug,
        model: config.model,
        model_role: config.model_role,
        orchestrator: config.orchestrator.unwrap_or(false),
        enabled: config.enabled.unwrap_or(true),
        combined_prompt,
        combined_prompt_without_memory,
        max_tool_iterations,
    })
}

fn build_combined_prompt(
    soul: Option<&str>,
    mood: Option<&str>,
    memory: Option<&str>,
    skill: &str,
) -> String {
    let mut out = String::new();
    if let Some(s) = soul {
        out.push_str(s);
        out.push_str("\n\n");
    }
    if let Some(s) = mood {
        out.push_str(s);
        out.push_str("\n\n");
    }
    if let Some(m) = memory {
        out.push_str("## Memory (lessons learned — follow these)\n\n");
        out.push_str(m);
        out.push_str("\n\n");
    }
    out.push_str(skill);
    out
}

/// Find an agent by slug (case-insensitive), then by name (case-insensitive), then by id (exact or agent-<id>).
pub fn find_agent_by_id_or_name<'a>(agents: &'a [Agent], selector: &str) -> Option<&'a Agent> {
    let selector = selector.trim();
    if selector.is_empty() {
        return None;
    }
    let slug_match = selector.to_lowercase();
    if let Some(a) = agents
        .iter()
        .find(|a| a.slug.as_ref().map(|s| s.to_lowercase()) == Some(slug_match.clone()))
    {
        return Some(a);
    }
    let name_match = selector.to_lowercase();
    if let Some(a) = agents.iter().find(|a| a.name.to_lowercase() == name_match) {
        return Some(a);
    }
    if let Some(a) = agents.iter().find(|a| a.id == selector) {
        return Some(a);
    }
    let id_from_prefix = selector.strip_prefix("agent-").unwrap_or(selector);
    agents.iter().find(|a| a.id == id_from_prefix)
}

/// Read RUN_CMD allowlist from the first enabled orchestrator's skill.md.
/// Looks for a section "## RUN_CMD allowlist" (case-insensitive); content until next "## " or EOF
/// is split by comma and newline, trimmed, lowercased. Returns None if no orchestrator or no section.
pub fn get_run_cmd_allowlist() -> Option<Vec<String>> {
    let agents = load_agents();
    let orchestrator = agents.iter().find(|a| a.orchestrator)?;
    let dir = get_agent_dir(&orchestrator.id)?;
    let skill_path = dir.join("skill.md");
    let content = std::fs::read_to_string(&skill_path).ok()?;
    parse_run_cmd_allowlist_from_md(&content)
}

/// Parse "## RUN_CMD allowlist" section from markdown. Returns None if section empty or missing.
fn parse_run_cmd_allowlist_from_md(content: &str) -> Option<Vec<String>> {
    const HEADER: &str = "## run_cmd allowlist";
    let content_lower = content.to_lowercase();
    let start = content_lower.find(HEADER)?;
    // Slice from original content so we preserve the block text (index matches content_lower).
    let after_header = content[start + HEADER.len()..].trim_start();
    let block = if let Some(next) = after_header.find("\n## ") {
        after_header[..next].trim()
    } else {
        after_header.trim()
    };
    let mut commands: Vec<String> = block
        .split([',', '\n'])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if commands.is_empty() {
        return None;
    }
    // Dedupe preserving order
    commands.sort();
    commands.dedup();
    Some(commands)
}

/// Resolve model assignments for all agents using the given catalog.
///
/// For each agent:
/// 1. If `model` is set and available in catalog -> keep it.
/// 2. If `model` is set but NOT available -> warn, fall through to model_role.
/// 3. If `model_role` is set -> resolve from catalog.
/// 4. Otherwise -> leave model as-is (will use global default at chat time).
pub fn resolve_agent_models(agents: &mut [Agent], catalog: &crate::ollama::models::ModelCatalog) {
    for agent in agents.iter_mut() {
        let agent_label = agent.slug.as_deref().unwrap_or(&agent.name);

        // Step 1: explicit model override
        if let Some(ref model_name) = agent.model {
            if catalog.has_model(model_name) {
                info!(
                    "Model resolution: {} -> {} (explicit override, available)",
                    agent_label, model_name
                );
                continue;
            }
            warn!(
                "Model resolution: {} -> model '{}' not available, falling back to model_role={:?}",
                agent_label, model_name, agent.model_role
            );
        }

        // Step 2: resolve from model_role
        if let Some(ref role) = agent.model_role {
            if let Some(resolved) = catalog.resolve_role(role) {
                info!(
                    "Model resolution: {} -> {} (role={}, {:.1}B)",
                    agent_label, resolved.name, role, resolved.param_billions
                );
                agent.model = Some(resolved.name.clone());
                continue;
            }
            let has_only_cloud = catalog.models.iter().all(|m| m.is_cloud);
            if has_only_cloud {
                warn!(
                    "Model resolution: {} -> no model found for role '{}', leaving unset; cloud default will be used at chat time (no local models available)",
                    agent_label, role
                );
            } else {
                warn!(
                    "Model resolution: {} -> no model found for role '{}', leaving unset",
                    agent_label, role
                );
            }
            agent.model = None;
            continue;
        }

        // Step 3: neither model nor model_role set
        debug!(
            "Model resolution: {} -> no model or model_role, will use global default",
            agent_label
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::sync::Mutex;

    /// Tests override `HOME` for `Config::agents_dir()`; serialize so env does not race.
    static HOME_FOR_AGENTS_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct HomeOverride {
        previous: Option<String>,
    }

    impl HomeOverride {
        fn set(home: &Path) -> Self {
            let previous = std::env::var("HOME").ok();
            std::env::set_var("HOME", home.as_os_str());
            Self { previous }
        }
    }

    impl Drop for HomeOverride {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
    }

    #[test]
    fn shared_soul_file_nonempty_true_when_file_has_text() {
        let _guard = HOME_FOR_AGENTS_TEST_LOCK.lock().expect("home test lock");
        let base = std::env::temp_dir().join(format!("mac-stats-soul-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".mac-stats/agents")).unwrap();
        let _home = HomeOverride::set(&base);
        std::fs::write(Config::soul_file_path(), "be kind\n").unwrap();
        assert!(shared_soul_file_nonempty());
    }

    #[test]
    fn shared_soul_file_nonempty_false_when_empty_or_whitespace() {
        let _guard = HOME_FOR_AGENTS_TEST_LOCK.lock().expect("home test lock");
        let base =
            std::env::temp_dir().join(format!("mac-stats-soul-empty-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".mac-stats/agents")).unwrap();
        let _home = HomeOverride::set(&base);
        std::fs::write(Config::soul_file_path(), "   \n\t").unwrap();
        assert!(!shared_soul_file_nonempty());
    }

    #[test]
    fn shared_soul_file_nonempty_false_when_missing() {
        let _guard = HOME_FOR_AGENTS_TEST_LOCK.lock().expect("home test lock");
        let base = std::env::temp_dir().join(format!(
            "mac-stats-soul-missing-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".mac-stats/agents")).unwrap();
        let _home = HomeOverride::set(&base);
        assert!(!shared_soul_file_nonempty());
    }

    #[test]
    fn build_combined_prompt_order() {
        let s = build_combined_prompt(Some("soul"), Some("mood"), Some("memory"), "skill");
        assert!(s.starts_with("soul"));
        assert!(s.contains("mood"));
        assert!(s.contains("memory"));
        assert!(s.ends_with("skill"));
    }

    #[test]
    fn find_agent_by_slug() {
        let agents = vec![Agent {
            id: "001".to_string(),
            name: "General".to_string(),
            slug: Some("generalist".to_string()),
            model: None,
            model_role: None,
            orchestrator: false,
            enabled: true,
            combined_prompt: String::new(),
            combined_prompt_without_memory: String::new(),
            max_tool_iterations: 15,
        }];
        assert!(find_agent_by_id_or_name(&agents, "generalist").is_some());
        assert!(find_agent_by_id_or_name(&agents, "Generalist").is_some());
    }

    #[test]
    fn find_agent_by_id() {
        let agents = vec![Agent {
            id: "001".to_string(),
            name: "General".to_string(),
            slug: None,
            model: None,
            model_role: None,
            orchestrator: false,
            enabled: true,
            combined_prompt: String::new(),
            combined_prompt_without_memory: String::new(),
            max_tool_iterations: 15,
        }];
        assert!(find_agent_by_id_or_name(&agents, "001").is_some());
    }

    #[test]
    fn agent_soul_or_shared_prefers_per_agent() {
        assert_eq!(
            agent_soul_or_shared(Some("only me".to_string()), "shared soul".to_string()).as_deref(),
            Some("only me")
        );
    }

    #[test]
    fn agent_soul_or_shared_ignores_empty_per_agent() {
        assert_eq!(
            agent_soul_or_shared(Some(String::new()), "shared".to_string()).as_deref(),
            Some("shared")
        );
        assert_eq!(
            agent_soul_or_shared(None, "shared".to_string()).as_deref(),
            Some("shared")
        );
    }

    #[test]
    fn agent_soul_or_shared_none_when_no_soul() {
        assert_eq!(agent_soul_or_shared(None, String::new()), None);
        assert_eq!(
            agent_soul_or_shared(Some(String::new()), String::new()),
            None
        );
    }
}
