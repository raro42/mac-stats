//! Tauri commands for skills (agent prompt overlays in ~/.mac-stats/agents/skills/).

use crate::skills;

/// List all loaded skills for the Settings UI (number, topic, path).
#[tauri::command]
pub fn list_skills() -> Result<Vec<skills::SkillForUi>, String> {
    Ok(skills::list_skills_for_ui())
}
