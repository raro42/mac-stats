//! Perplexity Search Tauri commands.
//!
//! API key is stored in Keychain under account PERPLEXITY_KEYCHAIN_ACCOUNT.
//! Use Settings in the app to store the key (store_credential).

use crate::perplexity;
use serde::{Deserialize, Serialize};

/// Keychain account used for Perplexity API key (same as frontend store_credential).
pub const PERPLEXITY_KEYCHAIN_ACCOUNT: &str = "perplexity_api_key";

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

/// Run a Perplexity web search. API key is read from Keychain (perplexity_api_key).
#[tauri::command]
pub async fn perplexity_search(request: PerplexitySearchRequest) -> Result<PerplexitySearchResponse, String> {
    use tracing::debug;

    let api_key = crate::security::get_credential(PERPLEXITY_KEYCHAIN_ACCOUNT)
        .map_err(|e| format!("Keychain: {}", e))?
        .ok_or_else(|| "Perplexity API key not set. Add it in Settings.")?;

    if api_key.trim().is_empty() {
        return Err("Perplexity API key is empty. Add it in Settings.".to_string());
    }

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

/// Check if Perplexity is configured (API key present in Keychain). Does not validate the key.
#[tauri::command]
pub fn is_perplexity_configured() -> Result<bool, String> {
    let key = crate::security::get_credential(PERPLEXITY_KEYCHAIN_ACCOUNT).map_err(|e| e.to_string())?;
    Ok(key.map(|k| !k.trim().is_empty()).unwrap_or(false))
}
