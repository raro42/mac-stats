//! Perplexity Search Tauri commands.
//!
//! API key is resolved (in order) from: PERPLEXITY_API_KEY env, .config.env / .env.config
//! (cwd, src-tauri, ~/.mac-stats), then Keychain (perplexity_api_key).

use crate::perplexity;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Keychain account used for Perplexity API key (same as frontend store_credential).
pub const PERPLEXITY_KEYCHAIN_ACCOUNT: &str = "perplexity_api_key";

/// Read PERPLEXITY_API_KEY from a .config.env / .env.config style file.
fn perplexity_key_from_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().find(|l| {
        let t = l.trim();
        t.starts_with("PERPLEXITY_API_KEY=") && !t.starts_with("#")
    })?;
    let (_, v) = line.split_once('=')?;
    let key = v.trim().to_string();
    if key.is_empty() {
        return None;
    }
    Some(key)
}

/// Get Perplexity API key: PERPLEXITY_API_KEY env, then .config.env / .env.config, then Keychain.
pub fn get_perplexity_api_key() -> Option<String> {
    if let Ok(k) = std::env::var("PERPLEXITY_API_KEY") {
        let k = k.trim().to_string();
        if !k.is_empty() {
            tracing::debug!("Perplexity: API key from PERPLEXITY_API_KEY env");
            return Some(k);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        for name in [".config.env", ".env.config"] {
            let p = cwd.join(name);
            if p.is_file() {
                if let Some(k) = perplexity_key_from_file(&p) {
                    tracing::debug!("Perplexity: API key from {} (cwd)", name);
                    return Some(k);
                }
            }
            let p_src = cwd.join("src-tauri").join(name);
            if p_src.is_file() {
                if let Some(k) = perplexity_key_from_file(&p_src) {
                    tracing::debug!("Perplexity: API key from src-tauri/{}", name);
                    return Some(k);
                }
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let base = Path::new(&home).join(".mac-stats");
        for name in [".config.env", ".env.config"] {
            let p = base.join(name);
            if p.is_file() {
                if let Some(k) = perplexity_key_from_file(&p) {
                    tracing::debug!("Perplexity: API key from ~/.mac-stats/{}", name);
                    return Some(k);
                }
            }
        }
    }
    if let Ok(Some(k)) = crate::security::get_credential(PERPLEXITY_KEYCHAIN_ACCOUNT) {
        if !k.trim().is_empty() {
            tracing::debug!("Perplexity: API key from Keychain");
            return Some(k);
        }
    }
    None
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerplexitySearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerplexitySearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerplexitySearchResponse {
    pub results: Vec<PerplexitySearchResult>,
    pub id: String,
}

/// Run a Perplexity web search. API key from PERPLEXITY_API_KEY env, .config.env / .env.config, or Keychain.
#[tauri::command]
pub async fn perplexity_search(request: PerplexitySearchRequest) -> Result<PerplexitySearchResponse, String> {
    use tracing::debug;

    let api_key = get_perplexity_api_key()
        .ok_or_else(|| "Perplexity API key not set. Set PERPLEXITY_API_KEY in env or .config.env / .env.config, or add it in Settings.")?;

    let query = request.query.trim();
    if query.is_empty() {
        return Err("Search query is empty.".to_string());
    }

    debug!("Perplexity search: query len={}", query.len());

    let response = perplexity::search(api_key.as_str(), query, request.max_results)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("401") || msg.contains("Unauthorized") {
                "Invalid or expired API key. Update it in Settings.".to_string()
            } else {
                msg
            }
        })?;

    let results = response
        .results
        .into_iter()
        .map(|p| PerplexitySearchResult {
            title: p.title,
            url: p.url,
            snippet: p.snippet,
            date: p.date,
            last_updated: p.last_updated,
        })
        .collect();

    Ok(PerplexitySearchResponse {
        results,
        id: response.id,
    })
}

/// Check if Perplexity is configured (API key from env, .config.env / .env.config, or Keychain). Does not validate the key.
#[tauri::command]
pub fn is_perplexity_configured() -> Result<bool, String> {
    Ok(get_perplexity_api_key().is_some())
}
