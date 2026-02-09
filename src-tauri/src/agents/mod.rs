//! Agents: directory-based LLM agents under ~/.mac-stats/agents/agent-<id>/
//!
//! Each agent has agent.json (name, optional slug, model, orchestrator, enabled),
//! required skill.md, optional soul.md and mood.md. Combined prompt order: soul → mood → skill.
//! Used by the Ollama tool loop (AGENT: <selector> [task]) and by the agent-test CLI.

pub mod cli;

use crate::config::Config;
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, info, warn};

/// Per-agent config from agent.json. Name is required; others optional.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub orchestrator: Option<bool>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

/// One loaded agent: id from directory name, config from agent.json, combined prompt from soul+mood+skill.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub model: Option<String>,
    pub orchestrator: bool,
    pub enabled: bool,
    pub combined_prompt: String,
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
    }

    agents
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

    let soul = std::fs::read_to_string(dir.join("soul.md"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let mood = std::fs::read_to_string(dir.join("mood.md"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let combined_prompt = build_combined_prompt(soul.as_deref(), mood.as_deref(), &skill);

    Some(Agent {
        id: id.to_string(),
        name: config.name,
        slug: config.slug,
        model: config.model,
        orchestrator: config.orchestrator.unwrap_or(false),
        enabled: config.enabled.unwrap_or(true),
        combined_prompt,
    })
}

fn build_combined_prompt(soul: Option<&str>, mood: Option<&str>, skill: &str) -> String {
    let mut out = String::new();
    if let Some(s) = soul {
        out.push_str(s);
        out.push_str("\n\n");
    }
    if let Some(s) = mood {
        out.push_str(s);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_combined_prompt_order() {
        let s = build_combined_prompt(Some("soul"), Some("mood"), "skill");
        assert!(s.starts_with("soul"));
        assert!(s.contains("mood"));
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
                orchestrator: false,
                enabled: true,
                combined_prompt: String::new(),
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
                orchestrator: false,
                enabled: true,
                combined_prompt: String::new(),
            },
        ];
        assert!(find_agent_by_id_or_name(&agents, "001").is_some());
    }
}
