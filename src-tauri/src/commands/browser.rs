//! Browser / URL fetch support for Ollama and AI tasks.
//!
//! Provides server-side page fetch (no CORS). Used by the Ollama tool protocol
//! (FETCH_URL) and can be invoked from the frontend.

use std::time::Duration;
use tracing::{info, warn};
use url::Url;

/// Max response body size (chars) to avoid huge strings (e.g. 500 KB of text).
const MAX_BODY_CHARS: usize = 500_000;

/// Browser-like User-Agent so servers that block bots/scrapers allow the request (avoids 403).
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Extract the first URL-like token from text (e.g. FETCH_URL arg that may contain extra words).
/// Takes the first substring that starts with http:// or https:// and runs until first whitespace or newline.
/// Returns None if no such substring exists.
pub fn extract_first_url(arg: &str) -> Option<String> {
    let s = arg.trim();
    let start = s.find("https://").or_else(|| s.find("http://"))?;
    let rest = &s[start..];
    let end = rest
        .find(|c: char| c.is_whitespace())
        .unwrap_or(rest.len());
    let url = rest[..end].trim_end_matches(|c: char| c == '.' || c == ',' || c == ';' || c == ':');
    if url.is_empty() {
        return None;
    }
    Some(url.to_string())
}

/// Validate and normalize URL for fetch. Returns clear error for invalid or IDN URLs.
fn validate_fetch_url(url: &str) -> Result<Url, String> {
    let parsed = Url::parse(url).map_err(|e| {
        let err_str = e.to_string();
        if err_str.to_lowercase().contains("international domain name") || err_str.contains("IDN") {
            "Invalid URL: international domain names (IDN) are not supported. Use punycode (e.g. xn--...) or a different URL.".to_string()
        } else {
            format!("Invalid URL for FETCH_URL: {}", e)
        }
    })?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("URL must use http or https".to_string()),
    }
    Ok(parsed)
}

/// Fetch a URL and return the response body as text.
/// Extracts first URL from arg (task-002), validates, uses same timeout/SSL policy as website monitors.
/// Used by Ollama FETCH_URL flow and by the fetch_page Tauri command.
pub fn fetch_page_content(url: &str) -> Result<String, String> {
    let raw = url.trim();
    let url_str = extract_first_url(raw).ok_or_else(|| {
        "Invalid URL for FETCH_URL: no http:// or https:// URL found. Provide a single URL only.".to_string()
    })?;
    let _parsed = validate_fetch_url(&url_str)?;
    let url = _parsed.as_str();

    info!("Fetch page: GET {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let resp = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let code = status.as_u16();
        let reason = status.canonical_reason().unwrap_or("");
        warn!(
            "Fetch page failed: {} {} for URL {}",
            code, reason, url
        );
        return Err(format!("HTTP {}: {}", code, reason));
    }

    let body = resp
        .text()
        .map_err(|e| format!("Read body: {}", e))?;

    let body = if body.chars().count() > MAX_BODY_CHARS {
        crate::logging::ellipse(&body, MAX_BODY_CHARS)
    } else {
        body
    };
    let n = body.chars().count();
    info!("Fetch page: fetched {} chars from {}", n, url);
    Ok(body)
}

/// Tauri command: fetch a URL and return body as text (for frontend or tools).
#[tauri::command]
pub async fn fetch_page(url: String) -> Result<String, String> {
    let url = url.clone();
    tokio::task::spawn_blocking(move || fetch_page_content(&url))
        .await
        .map_err(|e| format!("Task join: {}", e))?
}
