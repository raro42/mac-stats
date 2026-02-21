//! Redmine REST API client for the agent router.
//!
//! Provides GET and PUT access to a Redmine instance. Auth via API key
//! (`X-Redmine-API-Key` header). Config read from env or `.config.env`:
//!   REDMINE_URL=https://redmine.example.com
//!   REDMINE_API_KEY=<key>
//!
//! PUT is restricted to adding notes on issues (no destructive writes).

use std::path::Path;
use tracing::{debug, info};

const MAX_RESPONSE_CHARS: usize = 12_000;
const TIMEOUT_SECS: u64 = 20;

/// Read a key from `.config.env` style files (KEY=value, one per line).
fn read_key_from_file(path: &Path, key_names: &[&str]) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let t = line.trim();
        for &name in key_names {
            if let Some(val) = t.strip_prefix(name).and_then(|r| r.strip_prefix('=')) {
                let val = val.trim().trim_matches('\'').trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }
    None
}

/// Search env vars, then `.config.env` / `.env.config` in cwd, src-tauri, ~/.mac-stats.
fn read_config(env_name: &str, file_keys: &[&str]) -> Option<String> {
    if let Ok(v) = std::env::var(env_name) {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    let candidates: Vec<std::path::PathBuf> = [
        std::env::current_dir().ok().map(|d| d.join(".config.env")),
        std::env::current_dir().ok().map(|d| d.join(".env.config")),
        std::env::current_dir().ok().map(|d| d.join("..").join(".env.config")),
        std::env::var("HOME").ok().map(|h| Path::new(&h).join(".mac-stats").join(".config.env")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if let Some(val) = read_key_from_file(path, file_keys) {
            return Some(val);
        }
    }
    None
}

pub fn get_redmine_url() -> Option<String> {
    read_config("REDMINE_URL", &["REDMINE_URL", "REDMINE-URL"])
}

pub fn get_redmine_api_key() -> Option<String> {
    read_config("REDMINE_API_KEY", &["REDMINE_API_KEY", "REDMINE-API-KEY"])
}

/// Whether Redmine is configured (URL + API key both present).
pub fn is_configured() -> bool {
    get_redmine_url().is_some() && get_redmine_api_key().is_some()
}

/// PUT paths that are allowed. Currently: updating issues (adding notes).
fn is_allowed_put_path(path: &str) -> bool {
    let path = path.trim().trim_start_matches('/');
    // Allow: issues/{id}.json
    if let Some(rest) = path.strip_prefix("issues/") {
        return rest.ends_with(".json") || rest.ends_with(".xml");
    }
    false
}

/// Perform a Redmine API request.
///
/// - method: GET or PUT (PUT only for allowed paths).
/// - path: relative to base URL (e.g. `/issues/1234.json?include=journals`).
/// - body: optional JSON body for PUT.
pub async fn redmine_api_request(
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let base_url = get_redmine_url()
        .ok_or_else(|| "Redmine not configured (REDMINE_URL missing from env or .config.env)".to_string())?;
    let api_key = get_redmine_api_key()
        .ok_or_else(|| "Redmine not configured (REDMINE_API_KEY missing from env or .config.env)".to_string())?;

    let path = path.trim();
    if path.is_empty() {
        return Err("Redmine API path must not be empty".to_string());
    }
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    let method_upper = method.to_uppercase();
    let allowed = match method_upper.as_str() {
        "GET" => true,
        "PUT" => is_allowed_put_path(&path),
        _ => false,
    };
    if !allowed {
        return Err(format!(
            "Redmine API: method {} not allowed for path {} (GET anything, PUT only issues)",
            method_upper, path
        ));
    }

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    info!("Redmine API: {} {}", method_upper, url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let mut req = client
        .request(
            method_upper.parse().map_err(|e| format!("Invalid method: {}", e))?,
            &url,
        )
        .header("X-Redmine-API-Key", &api_key)
        .header("User-Agent", format!("mac-stats/{}", crate::config::Config::version()));

    if method_upper == "PUT" {
        let body_str = body.unwrap_or("{}").trim();
        debug!("Redmine API PUT body: {}", body_str);
        let body_json: serde_json::Value = serde_json::from_str(body_str)
            .map_err(|e| format!("Invalid JSON body: {}", e))?;
        req = req.header("Content-Type", "application/json").json(&body_json);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Redmine request failed: {}", e))?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        info!("Redmine API {} {} → {}", method_upper, path, status);
        return Err(format!(
            "Redmine API {}: {}",
            status,
            crate::logging::ellipse(&body_text, 500)
        ));
    }

    info!("Redmine API {} {} → 200 ({} chars)", method_upper, path, body_text.len());

    if body_text.chars().count() > MAX_RESPONSE_CHARS {
        Ok(crate::logging::ellipse(&body_text, MAX_RESPONSE_CHARS))
    } else {
        Ok(body_text)
    }
}
