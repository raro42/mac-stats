//! Tauri commands for agent CRUD and listing.
//! Agents live under ~/.mac-stats/agents/agent-<id>/ with agent.json, skill.md, soul.md, mood.md.

use crate::agents::{load_all_agents, get_agent_dir, find_agent_by_id_or_name, AgentConfig};
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Summary for list view (no prompt content).
#[derive(Debug, Clone, Serialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub model: Option<String>,
    pub orchestrator: bool,
    pub enabled: bool,
}

/// Full details for edit view (includes soul, mood, skill text).
#[derive(Debug, Clone, Serialize)]
pub struct AgentDetails {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub model: Option<String>,
    pub orchestrator: bool,
    pub enabled: bool,
    pub skill: String,
    pub soul: Option<String>,
    pub mood: Option<String>,
}

#[tauri::command]
pub fn list_agents() -> Vec<AgentSummary> {
    load_all_agents()
        .into_iter()
        .map(|a| AgentSummary {
            id: a.id,
            name: a.name,
            slug: a.slug,
            model: a.model,
            orchestrator: a.orchestrator,
            enabled: a.enabled,
        })
        .collect()
}

#[tauri::command]
pub fn get_agent_details(selector: String) -> Result<AgentDetails, String> {
    let agents = load_all_agents();
    let agent = find_agent_by_id_or_name(&agents, &selector)
        .ok_or_else(|| format!("Agent not found: {}", selector))?;
    let dir = get_agent_dir(&agent.id).ok_or_else(|| format!("Agent directory missing: {}", agent.id))?;
    let skill = std::fs::read_to_string(dir.join("skill.md")).unwrap_or_default();
    let soul = std::fs::read_to_string(dir.join("soul.md"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let mood = std::fs::read_to_string(dir.join("mood.md"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok(AgentDetails {
        id: agent.id.clone(),
        name: agent.name.clone(),
        slug: agent.slug.clone(),
        model: agent.model.clone(),
        orchestrator: agent.orchestrator,
        enabled: agent.enabled,
        skill,
        soul,
        mood,
    })
}

fn write_agent_file(dir: &Path, filename: &str, content: &str) -> Result<(), String> {
    let path = dir.join(filename);
    std::fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

#[tauri::command]
pub fn update_agent_skill(agent_id: String, content: String) -> Result<(), String> {
    let dir = get_agent_dir(&agent_id).ok_or_else(|| format!("Agent not found: {}", agent_id))?;
    write_agent_file(&dir, "skill.md", &content)
}

#[tauri::command]
pub fn update_agent_soul(agent_id: String, content: String) -> Result<(), String> {
    let dir = get_agent_dir(&agent_id).ok_or_else(|| format!("Agent not found: {}", agent_id))?;
    write_agent_file(&dir, "soul.md", content.trim())
}

#[tauri::command]
pub fn update_agent_mood(agent_id: String, content: String) -> Result<(), String> {
    let dir = get_agent_dir(&agent_id).ok_or_else(|| format!("Agent not found: {}", agent_id))?;
    write_agent_file(&dir, "mood.md", content.trim())
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentConfigPayload {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub model: Option<String>,
    pub orchestrator: Option<bool>,
    pub enabled: Option<bool>,
    pub description: Option<String>,
    pub max_tool_iterations: Option<u32>,
}

#[tauri::command]
pub fn update_agent_config(agent_id: String, payload: UpdateAgentConfigPayload) -> Result<(), String> {
    let dir = get_agent_dir(&agent_id).ok_or_else(|| format!("Agent not found: {}", agent_id))?;
    let path = dir.join("agent.json");
    let current: AgentConfig = {
        let s = std::fs::read_to_string(&path).map_err(|e| format!("Failed to read agent.json: {}", e))?;
        serde_json::from_str(&s).map_err(|e| format!("Invalid agent.json: {}", e))?
    };
    let name = payload.name.unwrap_or(current.name);
    let slug = payload.slug.or(current.slug);
    let model = payload.model.or(current.model);
    let orchestrator = payload.orchestrator.or(current.orchestrator).unwrap_or(false);
    let enabled = payload.enabled.unwrap_or(current.enabled.unwrap_or(true));
    let description = payload.description.or(current.description);
    let max_tool_iterations = payload.max_tool_iterations.or(current.max_tool_iterations);
    let updated = AgentConfig {
        name,
        slug,
        model,
        orchestrator: Some(orchestrator),
        enabled: Some(enabled),
        description,
        max_tool_iterations,
    };
    let json = serde_json::to_string_pretty(&updated).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write agent.json: {}", e))
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentPayload {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub skill_initial: Option<String>,
}

#[tauri::command]
pub fn create_agent(payload: CreateAgentPayload) -> Result<(), String> {
    let id = payload.id.trim();
    if id.is_empty() {
        return Err("Agent id is required".to_string());
    }
    if id.contains(std::path::MAIN_SEPARATOR) || id.contains('/') || id.contains('\\') {
        return Err("Agent id must not contain path separators".to_string());
    }
    let dir = Config::agents_dir().join(format!("agent-{}", id));
    if dir.exists() {
        return Err(format!("Agent already exists: {}", id));
    }
    Config::ensure_agents_directory().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create agent dir: {}", e))?;
    let config = AgentConfig {
        name: payload.name.trim().to_string(),
        slug: payload.slug.filter(|s| !s.trim().is_empty()),
        model: payload.model.filter(|s| !s.trim().is_empty()),
        orchestrator: Some(false),
        enabled: Some(true),
        description: None,
        max_tool_iterations: None, // default 15 when loaded
    };
    let config_json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("agent.json"), config_json).map_err(|e| e.to_string())?;
    let skill = payload
        .skill_initial
        .unwrap_or_else(|| "# Skill\n\nDescribe what this agent does.".to_string());
    std::fs::write(dir.join("skill.md"), skill).map_err(|e| e.to_string())?;
    let testing = "# Testing\n\nOptional test cases (one per line):\n\n- Input: <user input> Expected: <expected>\n";
    let _ = std::fs::write(dir.join("testing.md"), testing);
    Ok(())
}

#[tauri::command]
pub fn delete_agent(agent_id: String) -> Result<(), String> {
    let dir = get_agent_dir(&agent_id).ok_or_else(|| format!("Agent not found: {}", agent_id))?;
    std::fs::remove_dir_all(&dir).map_err(|e| format!("Failed to delete agent: {}", e))
}

fn set_agent_enabled(agent_id: &str, enabled: bool) -> Result<(), String> {
    let dir = get_agent_dir(agent_id).ok_or_else(|| format!("Agent not found: {}", agent_id))?;
    let path = dir.join("agent.json");
    let s = std::fs::read_to_string(&path).map_err(|e| format!("Failed to read agent.json: {}", e))?;
    let mut config: AgentConfig = serde_json::from_str(&s).map_err(|e| format!("Invalid agent.json: {}", e))?;
    config.enabled = Some(enabled);
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write agent.json: {}", e))
}

#[tauri::command]
pub fn disable_agent(agent_id: String) -> Result<(), String> {
    set_agent_enabled(&agent_id, false)
}

#[tauri::command]
pub fn enable_agent(agent_id: String) -> Result<(), String> {
    set_agent_enabled(&agent_id, true)
}

// --- Prompt file management ---

#[derive(Debug, Clone, Serialize)]
pub struct PromptFile {
    pub name: String,
    pub path: String,
    pub content: String,
}

/// List all editable prompt files (soul, planning, execution) with their content.
#[tauri::command]
pub fn list_prompt_files() -> Vec<PromptFile> {
    vec![
        PromptFile {
            name: "soul".to_string(),
            path: Config::soul_file_path().to_string_lossy().to_string(),
            content: Config::load_soul_content(),
        },
        PromptFile {
            name: "planning_prompt".to_string(),
            path: Config::planning_prompt_path().to_string_lossy().to_string(),
            content: Config::load_planning_prompt(),
        },
        PromptFile {
            name: "execution_prompt".to_string(),
            path: Config::execution_prompt_path().to_string_lossy().to_string(),
            content: Config::load_execution_prompt(),
        },
    ]
}

/// Save content to a named prompt file. Name must be one of: soul, planning_prompt, execution_prompt.
#[tauri::command]
pub fn save_prompt_file(name: String, content: String) -> Result<(), String> {
    let path = match name.as_str() {
        "soul" => Config::soul_file_path(),
        "planning_prompt" => Config::planning_prompt_path(),
        "execution_prompt" => Config::execution_prompt_path(),
        _ => return Err(format!("Unknown prompt file: {}. Use: soul, planning_prompt, execution_prompt.", name)),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(&path, content.trim()).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}
