//! Brave Search API integration for Ollama agents.
//!
//! API key from BRAVE_API_KEY env or .config.env (BRAVE_API_KEY=...).
//! See: https://api-dashboard.search.brave.com/documentation/guides/authentication
//! Rate limits: https://api-dashboard.search.brave.com/documentation/guides/rate-limiting

use std::path::Path;
use tracing::{info, warn};

const BRAVE_WEB_SEARCH_URL: &str = "https://api.search.brave.com/res/v1/web/search";

/// Rate limit header names (Brave Search API).
const HDR_LIMIT: &str = "x-ratelimit-limit";
const HDR_REMAINING: &str = "x-ratelimit-remaining";
const HDR_RESET: &str = "x-ratelimit-reset";
const HDR_POLICY: &str = "x-ratelimit-policy";

fn log_brave_rate_limit_headers(resp: &reqwest::Response) {
    let limit = resp
        .headers()
        .get(HDR_LIMIT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    let remaining = resp
        .headers()
        .get(HDR_REMAINING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    let reset = resp
        .headers()
        .get(HDR_RESET)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    let policy = resp
        .headers()
        .get(HDR_POLICY)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    info!(
        "Brave agent: rate limit — limit={} remaining={} reset_sec={} policy={}",
        limit, remaining, reset, policy
    );
}

/// Read Brave API key from a .config.env-style file.
/// Accepts BRAVE_API_KEY= or BRAVE-API-KEY= (hyphens).
fn brave_key_from_config_env_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().find(|l| {
        let t = l.trim();
        t.starts_with("BRAVE_API_KEY=") || t.starts_with("BRAVE-API-KEY=")
    })?;
    let (_, v) = line.split_once('=')?;
    let key = v.trim().to_string();
    if key.is_empty() {
        return None;
    }
    Some(key)
}

/// Get Brave API key: BRAVE_API_KEY env, then .config.env (cwd, cwd/src-tauri, ~/.mac-stats).
/// In .config.env use BRAVE_API_KEY=... or BRAVE-API-KEY=...
pub fn get_brave_api_key() -> Option<String> {
    if let Ok(k) = std::env::var("BRAVE_API_KEY") {
        let k = k.trim().to_string();
        if !k.is_empty() {
            return Some(k);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        // .config.env in current directory (e.g. when run from src-tauri)
        let p = cwd.join(".config.env");
        if p.is_file() {
            if let Some(k) = brave_key_from_config_env_file(&p) {
                return Some(k);
            }
        }
        // src-tauri/.config.env when run from project root
        let p_src = cwd.join("src-tauri").join(".config.env");
        if p_src.is_file() {
            if let Some(k) = brave_key_from_config_env_file(&p_src) {
                return Some(k);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            if let Some(k) = brave_key_from_config_env_file(&p) {
                return Some(k);
            }
        }
    }
    None
}

/// Run a web search via Brave Search API. Returns a formatted string of results for Ollama.
pub async fn brave_web_search(query: &str, api_key: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Brave HTTP client: {}", e))?;

    info!("Brave agent: search query \"{}\"", query);

    let resp = client
        .get(BRAVE_WEB_SEARCH_URL)
        .query(&[("q", query)])
        .header("Accept", "application/json")
        .header("X-Subscription-Token", api_key.trim())
        .send()
        .await
        .map_err(|e| format!("Brave request failed: {}", e))?;

    let status = resp.status();
    log_brave_rate_limit_headers(&resp);

    if !status.is_success() {
        if status.as_u16() == 429 {
            let reset_sec = resp
                .headers()
                .get(HDR_RESET)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.trim().parse::<u64>().ok());
            warn!(
                "Brave agent: rate limited (429). Retry after {:?} seconds (see X-RateLimit-Reset).",
                reset_sec
            );
        }
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Brave API HTTP {}: {}",
            status.as_u16(),
            if body.is_empty() {
                status.canonical_reason().unwrap_or("").to_string()
            } else {
                body.chars().take(200).collect::<String>()
            }
        ));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Brave response parse: {}", e))?;

    let results = json
        .get("web")
        .and_then(|w| w.get("results"))
        .and_then(|r| r.as_array())
        .ok_or_else(|| "Brave API: no web.results in response".to_string())?;

    let mut lines = Vec::with_capacity(results.len().min(10));
    for (i, r) in results.iter().take(10).enumerate() {
        let title = r.get("title").and_then(|t| t.as_str()).unwrap_or("(no title)");
        let url = r.get("url").and_then(|u| u.as_str()).unwrap_or("");
        let desc = r.get("description").and_then(|d| d.as_str()).unwrap_or("");
        lines.push(format!("{}. {} | {}\n   {}", i + 1, title, url, desc));
    }
    let text = if lines.is_empty() {
        "No web results found.".to_string()
    } else {
        format!("Brave Search results for \"{}\":\n\n{}", query, lines.join("\n\n"))
    };
    info!("Brave agent: got {} results for \"{}\"", results.len(), query);
    Ok(text)
}
