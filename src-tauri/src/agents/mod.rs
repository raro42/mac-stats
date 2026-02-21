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
    } else {
        let list: String = agents
            .iter()
            .map(|a| {
                a.slug
                    .as_deref()
                    .unwrap_or(a.name.as_str())
            })
            .collect::<Vec<_>>()
            .join(", ");
        info!("Agents: loaded {} from {:?}: {}", agents.len(), dir, list);
        let soul_path = Config::soul_file_path();
        let soul_exists = std::fs::read_to_string(&soul_path)
            .ok()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        info!(
            "Agents: shared soul {:?}: {}",
            soul_path,
            if soul_exists { "present" } else { "missing (will write default on first use)" }
        );
    }

    // Auto-resolve model assignments from cached catalog (if available)
    if let Some(catalog) = crate::ollama::models::get_global_catalog() {
        resolve_agent_models(&mut agents, &catalog);
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

fn load_one_agent(dir: &Path, id: &str) -> Option<Agent> {
    let config_path = dir.join("agent.json");
    let content = std::fs::read_to_string(&config_path).map_err(|e| {
        warn!("Agents: could not read {:?}: {}", config_path, e);
    }).ok()?;
    let config: AgentConfig = serde_json::from_str(&content).map_err(|e| {
        warn!("Agents: invalid JSON in {:?}: {}", config_path, e);
    }).ok()?;

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
    let soul = std::fs::read_to_string(&soul_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| {
            debug!("Agents: agent {} using soul from {:?}", id, soul_path);
            s
        })
        .or_else(|| {
            let shared = Config::load_soul_content();
            if shared.is_empty() {
                None
            } else {
                debug!("Agents: agent {} using shared soul from {:?}", id, Config::soul_file_path());
                Some(shared)
            }
        });
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

    let combined_prompt = build_combined_prompt(soul.as_deref(), mood.as_deref(), memory.as_deref(), &skill);

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
        max_tool_iterations,
    })
}

fn build_combined_prompt(soul: Option<&str>, mood: Option<&str>, memory: Option<&str>, skill: &str) -> String {
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
    if let Some(a) = agents.iter().find(|a| {
        a.slug.as_ref().map(|s| s.to_lowercase()) == Some(slug_match.clone())
    }) {
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
        .split(|c| c == ',' || c == '\n')
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
                agent_label,
                model_name,
                agent.model_role
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
            warn!(
                "Model resolution: {} -> no model found for role '{}', leaving unset",
                agent_label, role
            );
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
        let agents = vec![
            Agent {
                id: "001".to_string(),
                name: "General".to_string(),
                slug: Some("generalist".to_string()),
                model: None,
                model_role: None,
                orchestrator: false,
                enabled: true,
                combined_prompt: String::new(),
                max_tool_iterations: 15,
            },
        ];
        assert!(find_agent_by_id_or_name(&agents, "generalist").is_some());
        assert!(find_agent_by_id_or_name(&agents, "Generalist").is_some());
    }

    #[test]
    fn find_agent_by_id() {
        let agents = vec![
            Agent {
                id: "001".to_string(),
                name: "General".to_string(),
                slug: None,
                model: None,
                model_role: None,
                orchestrator: false,
                enabled: true,
                combined_prompt: String::new(),
                max_tool_iterations: 15,
            },
        ];
        assert!(find_agent_by_id_or_name(&agents, "001").is_some());
    }
}
