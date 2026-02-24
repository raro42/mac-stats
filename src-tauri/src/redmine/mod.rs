//! Redmine REST API client for the agent router.
//!
//! Provides GET, POST (create issue), and PUT (update issue / add notes) access to a Redmine instance.
//! Auth via API key (`X-Redmine-API-Key` header). Config read from env or `.config.env`:
//!   REDMINE_URL=https://redmine.example.com
//!   REDMINE_API_KEY=<key>
//!
//! When building agent descriptions, we fetch projects, trackers, statuses, and priorities
//! and inject them as context so the LLM can resolve names (e.g. "Create in AMVARA") to IDs.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

const MAX_RESPONSE_CHARS: usize = 12_000;
const TIMEOUT_SECS: u64 = 20;
const CONTEXT_CACHE_TTL_SECS: u64 = 300; // 5 minutes

/// Cache for Redmine create context (projects, trackers, statuses, priorities).
static REDMINE_CONTEXT_CACHE: Mutex<Option<(String, Instant)>> = Mutex::new(None);

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
    let path_no_query = path.split('?').next().unwrap_or(path);
    if let Some(rest) = path_no_query.strip_prefix("issues/") {
        return rest.ends_with(".json") || rest.ends_with(".xml");
    }
    false
}

/// POST paths that are allowed. Currently: creating issues only.
fn is_allowed_post_path(path: &str) -> bool {
    let path = path.trim().trim_start_matches('/');
    let path_no_query = path.split('?').next().unwrap_or(path);
    path_no_query == "issues.json" || path_no_query == "issues.xml"
}

/// Internal: perform a GET request to Redmine, return response body or error.
async fn redmine_get(path: &str) -> Result<String, String> {
    let base_url = get_redmine_url()
        .ok_or_else(|| "Redmine not configured".to_string())?;
    let api_key = get_redmine_api_key()
        .ok_or_else(|| "Redmine API key missing".to_string())?;
    let path = path.trim_start_matches('/');
    let url = format!("{}/{}", base_url.trim_end_matches('/'), path);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let resp = client
        .get(&url)
        .header("X-Redmine-API-Key", &api_key)
        .header("User-Agent", format!("mac-stats/{}", crate::config::Config::version()))
        .send()
        .await
        .map_err(|e| format!("Redmine GET failed: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Redmine {} {}: {}", status, path, crate::logging::ellipse(&body, 200)));
    }
    resp.text().await.map_err(|e| format!("Redmine response: {}", e))
}

/// Parse projects from GET /projects.json response. Returns "id=name (identifier), ...".
fn parse_projects(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("projects")?.as_array()?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|p| {
            let id = p.get("id")?.as_i64()?;
            let name = p.get("name")?.as_str()?;
            let ident = p.get("identifier").and_then(|i| i.as_str()).unwrap_or("");
            if ident.is_empty() {
                Some(format!("{}={}", id, name))
            } else {
                Some(format!("{}={} ({})", id, name, ident))
            }
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Projects: {}.", parts.join(", ")))
}

/// Parse trackers from GET /trackers.json response.
fn parse_trackers(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("trackers")?.as_array()?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|t| {
            let id = t.get("id")?.as_i64()?;
            let name = t.get("name")?.as_str()?;
            Some(format!("{}={}", id, name))
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Trackers: {}.", parts.join(", ")))
}

/// Parse issue statuses from GET /issue_statuses.json response.
fn parse_issue_statuses(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("issue_statuses")
        .and_then(|a| a.as_array())
        .or_else(|| v.as_array());
    let arr = arr?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|s| {
            let id = s.get("id")?.as_i64()?;
            let name = s.get("name")?.as_str()?;
            Some(format!("{}={}", id, name))
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Statuses: {}.", parts.join(", ")))
}

/// Parse issue priorities from GET /enumerations/issue_priorities.json response.
fn parse_priorities(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("issue_priorities")?.as_array()?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|p| {
            let id = p.get("id")?.as_i64()?;
            let name = p.get("name")?.as_str()?;
            Some(format!("{}={}", id, name))
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Priorities: {}.", parts.join(", ")))
}

/// Fetch and format Redmine create context (projects, trackers, statuses, priorities).
/// Cached for CONTEXT_CACHE_TTL_SECS. Used so the LLM can resolve "Create in AMVARA" to project_id.
pub async fn get_redmine_create_context() -> Option<String> {
    {
        let guard = REDMINE_CONTEXT_CACHE.lock().ok()?;
        if let Some((ref ctx, instant)) = *guard {
            if instant.elapsed() < Duration::from_secs(CONTEXT_CACHE_TTL_SECS) && !ctx.is_empty() {
                debug!("Redmine create context: using cache ({} chars)", ctx.len());
                return Some(ctx.clone());
            }
        }
    }

    let base = "Current Redmine values (use when creating issues; match project/tracker/status/priority by name or id): ";
    let mut parts = Vec::new();

    if let Ok(body) = redmine_get("projects.json").await {
        if let Some(s) = parse_projects(&body) {
            parts.push(s);
        }
    } else {
        debug!("Redmine context: projects fetch failed");
    }
    if let Ok(body) = redmine_get("trackers.json").await {
        if let Some(s) = parse_trackers(&body) {
            parts.push(s);
        }
    }
    if let Ok(body) = redmine_get("issue_statuses.json").await {
        if let Some(s) = parse_issue_statuses(&body) {
            parts.push(s);
        }
    }
    if let Ok(body) = redmine_get("enumerations/issue_priorities.json").await {
        if let Some(s) = parse_priorities(&body) {
            parts.push(s);
        }
    }

    if parts.is_empty() {
        warn!("Redmine create context: no data fetched (all endpoints failed or empty)");
        return None;
    }

    let context = format!("{}{}", base, parts.join(" "));
    info!("Redmine create context: {} chars (projects, trackers, statuses, priorities)", context.len());

    {
        let mut guard = REDMINE_CONTEXT_CACHE.lock().ok()?;
        *guard = Some((context.clone(), Instant::now()));
    }
    Some(context)
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
        "POST" => is_allowed_post_path(&path),
        _ => false,
    };
    if !allowed {
        return Err(format!(
            "Redmine API: method {} not allowed for path {} (GET anything, POST /issues.json to create, PUT only issues/{{id}}.json)",
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

    if method_upper == "PUT" || method_upper == "POST" {
        let body_str = body.unwrap_or("{}").trim();
        debug!("Redmine API {} body: {}", method_upper, body_str);
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

    info!("Redmine API {} {} → {} ({} chars)", method_upper, path, status, body_text.len());

    if body_text.chars().count() > MAX_RESPONSE_CHARS {
        Ok(crate::logging::ellipse(&body_text, MAX_RESPONSE_CHARS))
    } else {
        Ok(body_text)
    }
}
