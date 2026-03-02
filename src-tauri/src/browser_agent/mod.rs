//! CDP (Chrome DevTools Protocol) browser agent.
//!
//! Connects to Chrome via WebSocket (user runs Chrome with --remote-debugging-port=9222),
//! navigates, gets page content, extracts data. Phase 1 of light browser-use style automation.
//! The browser session is kept alive and only closed after Config::browser_idle_timeout_secs() of inactivity (default 1 hour).

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use regex::Regex;
use std::path::PathBuf;
use tracing::{info, warn};

/// Fetch WebSocket debugger URL from Chrome running with --remote-debugging-port.
fn get_ws_url(port: u16) -> Result<String, String> {
    let url = format!("http://127.0.0.1:{}/json/version", port);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("Request to {}: {}", url, e))?;
    if !resp.status().is_success() {
        return Err(format!("{} returned {}", url, resp.status()));
    }
    let json: serde_json::Value = resp
        .json()
        .map_err(|e| format!("Parse JSON from {}: {}", url, e))?;
    let ws = json
        .get("webSocketDebuggerUrl")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "webSocketDebuggerUrl not found in /json/version".to_string())?;
    Ok(ws.to_string())
}

/// Connect to Chrome at the given debugging port.
pub fn connect_cdp(port: u16) -> Result<Browser, String> {
    let ws_url = get_ws_url(port)?;
    info!("Browser agent: connecting to CDP at port {}", port);
    Browser::connect_with_timeout(ws_url, Duration::from_secs(60))
        .map_err(|e| format!("CDP connect: {}", e))
}

/// Navigate to URL and return the tab (first/only page tab). Caller must use tab.
pub fn navigate(browser: &Browser, url: &str) -> Result<Arc<headless_chrome::Tab>, String> {
    let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    let tab = tabs
        .first()
        .cloned()
        .ok_or_else(|| "No tab in browser".to_string())?;
    drop(tabs);
    tab.navigate_to(url)
        .map_err(|e| format!("Navigate to {}: {}", url, e))?;
    tab.wait_until_navigated()
        .map_err(|e| format!("Wait navigated: {}", e))?;
    info!("Browser agent: navigated to {}", url);
    Ok(tab)
}

/// Get visible text of the page (after JS has run) via document.body.innerText.
pub fn get_page_text(tab: &headless_chrome::Tab) -> Result<String, String> {
    let script = "document.body ? document.body.innerText : document.documentElement.innerText";
    let result = tab.evaluate(script, false).map_err(|e| format!("Evaluate: {}", e))?;
    let text = result
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(text)
}

/// Get full HTML of the page (for tel: links etc.).
pub fn get_page_html(tab: &headless_chrome::Tab) -> Result<String, String> {
    tab.get_content().map_err(|e| format!("Get content: {}", e))
}

/// Extract telephone numbers from text. German-style: +49..., 0..., etc.
pub fn extract_telephone_numbers(text: &str) -> Vec<String> {
    let re = Regex::new(
        r#"(?x)
        \+49\s*[\d\s\/\-\(\)]{6,}|
        0049\s*[\d\s\/\-\(\)]{6,}|
        \+[1-9]\d{0,2}\s*[\d\s\/\-\(\)]{6,}|
        0\d{2,4}[\s\/\-]?\d[\d\s\/\-]{4,}
        "#
    )
    .unwrap();
    re.find_iter(text)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| s.chars().filter(|c| c.is_ascii_digit()).count() >= 6)
        .collect::<Vec<_>>()
}

/// Extract tel: hrefs from HTML (e.g. <a href="tel:+49...">).
fn extract_tel_from_html(html: &str) -> Vec<String> {
    let re = Regex::new(r#"tel:([^\s"'>]+)"#).unwrap();
    re.captures_iter(html)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| s.chars().any(|c| c.is_ascii_digit()))
        .collect::<Vec<_>>()
}

/// Full flow: connect to CDP at port, navigate to url, get page text, extract phone numbers; log and return.
pub fn fetch_page_and_extract_phones(port: u16, url: &str) -> Result<Vec<String>, String> {
    let browser = connect_cdp(port)?;
    fetch_page_and_extract_phones_with_browser(&browser, url)
}

/// Launch Chrome via headless_chrome (Browser::default()), navigate to url, extract phone numbers. Use when no Chrome is listening on a port.
pub fn launch_browser_and_extract_phones(url: &str) -> Result<Vec<String>, String> {
    info!("Browser agent: launching Chrome via headless_chrome");
    let browser = Browser::default().map_err(|e| format!("Launch Chrome: {}", e))?;
    for _ in 0..30 {
        std::thread::sleep(Duration::from_millis(200));
        let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        if !tabs.is_empty() {
            drop(tabs);
            break;
        }
        drop(tabs);
    }
    fetch_page_and_extract_phones_with_browser(&browser, url)
}

fn fetch_page_and_extract_phones_with_browser(browser: &Browser, url: &str) -> Result<Vec<String>, String> {
    let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    let tab = tabs
        .first()
        .cloned()
        .ok_or_else(|| "No tab".to_string())?;
    drop(tabs);
    info!("Browser agent: navigating to {}", url);
    tab.navigate_to(url)
        .map_err(|e| format!("Navigate: {}", e))?;
    tab.wait_until_navigated()
        .map_err(|e| format!("Wait navigated: {}", e))?;
    std::thread::sleep(Duration::from_secs(2));
    // Scroll to bottom to trigger footer/lazy content
    let _ = tab.evaluate("window.scrollTo(0, document.body.scrollHeight)", false);
    std::thread::sleep(Duration::from_secs(2));
    let text = get_page_text(&tab)?;
    let mut phones = extract_telephone_numbers(&text);
    if phones.is_empty() {
        if let Ok(html) = get_page_html(&tab) {
            info!("Browser agent: page HTML length {} chars", html.len());
            let tel_links = extract_tel_from_html(&html);
            if !tel_links.is_empty() {
                info!("Browser agent: found tel: links in HTML: {:?}", tel_links);
            }
            phones = tel_links;
        }
    }
    if phones.is_empty() {
        warn!(
            "Browser agent: no telephone numbers found in page text ({} chars). First 800 chars: {}",
            text.len(),
            text.chars().take(800).collect::<String>()
        );
    } else {
        for p in &phones {
            info!("Browser agent: telephone number found: {}", p);
        }
    }
    Ok(phones)
}

/// Cached browser session: (Browser, last_used). Dropped when idle longer than browser_idle_timeout_secs.
static BROWSER_SESSION: OnceLock<Mutex<Option<(Browser, Instant)>>> = OnceLock::new();

fn browser_session() -> &'static Mutex<Option<(Browser, Instant)>> {
    BROWSER_SESSION.get_or_init(|| Mutex::new(None))
}

/// Get or create a browser; reuse if last use was within idle timeout, else close and create new. Updates last_used.
fn get_or_create_browser(port: u16) -> Result<Browser, String> {
    let timeout_secs = crate::config::Config::browser_idle_timeout_secs();
    let mut guard = browser_session().lock().map_err(|e| e.to_string())?;
    let now = Instant::now();
    if let Some((ref browser, last_used)) = guard.as_ref() {
        if now.duration_since(*last_used).as_secs() < timeout_secs {
            let b = browser.clone();
            *guard = Some((b.clone(), now));
            info!(
                "Browser agent [CDP]: reusing existing session (idle timeout {}s)",
                timeout_secs
            );
            return Ok(b);
        }
        info!(
            "Browser agent [CDP]: session idle > {}s, closing browser",
            timeout_secs
        );
    }
    *guard = None;
    drop(guard);
    let browser = if get_ws_url(port).is_ok() {
        info!("Browser agent [CDP]: connecting to Chrome on port {}", port);
        connect_cdp(port)?
    } else {
        info!("Browser agent [CDP]: no Chrome on port {}, launching", port);
        let b = Browser::default().map_err(|e| format!("Launch Chrome: {}", e))?;
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(200));
            let tabs = b.get_tabs().lock().map_err(|e| e.to_string())?;
            if !tabs.is_empty() {
                drop(tabs);
                break;
            }
            drop(tabs);
        }
        b
    };
    *browser_session().lock().map_err(|e| e.to_string())? = Some((browser.clone(), Instant::now()));
    Ok(browser)
}

/// Normalize URL for display/filename (add https if no scheme).
fn normalize_url_for_screenshot(url: &str) -> String {
    let u = url.trim();
    if u.is_empty() {
        return "page".to_string();
    }
    if !u.starts_with("http://") && !u.starts_with("https://") {
        format!("https://{}", u)
    } else {
        u.to_string()
    }
}

/// Take a screenshot of the given URL using CDP (reuses session if within idle timeout, else connects or launches).
/// Saves PNG to ~/.mac-stats/screenshots/<timestamp>_<domain>.png and returns the path.
/// Browser session is kept until unused for Config::browser_idle_timeout_secs() (default 1 hour).
pub fn take_screenshot(url: &str) -> Result<PathBuf, String> {
    info!("Browser agent [CDP]: take_screenshot called with url (raw): {:?}", url);
    let url_normalized = normalize_url_for_screenshot(url);
    info!("Browser agent [CDP]: normalized URL: {}", url_normalized);
    let port = 9222u16;
    let browser = get_or_create_browser(port)?;
    let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    let tab = tabs.first().cloned().ok_or_else(|| "No tab".to_string())?;
    drop(tabs);
    info!("Browser agent [CDP]: navigating to: {}", url_normalized);
    tab.navigate_to(&url_normalized)
        .map_err(|e| {
            warn!("Browser agent [CDP]: navigate_to failed: {}", e);
            format!("Navigate: {}", e)
        })?;
    tab.wait_until_navigated()
        .map_err(|e| {
            warn!("Browser agent [CDP]: wait_until_navigated failed: {}", e);
            format!("Wait navigated: {}", e)
        })?;
    let final_url = tab.get_url();
    info!("Browser agent [CDP]: navigated; final tab URL: {}", final_url);
    if let Ok(title) = tab.evaluate("document.title", false) {
        let title_str = title
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("(none)");
        info!("Browser agent [CDP]: page title: {}", title_str);
        if title_str.to_lowercase().contains("404") || title_str.to_lowercase().contains("not found") {
            warn!("Browser agent [CDP]: page appears to be 404 or not found");
        }
    }
    std::thread::sleep(Duration::from_secs(2));
    let png_data = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .map_err(|e| format!("Capture screenshot: {}", e))?;
    let dir = crate::config::Config::screenshots_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create screenshots dir: {}", e))?;
    let domain = url_normalized
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("page")
        .replace(['.', ':', '?', '&', '='], "_");
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.png", ts, domain);
    let path = dir.join(&filename);
    std::fs::write(&path, &png_data).map_err(|e| format!("Write screenshot: {}", e))?;
    info!("Browser agent [CDP]: screenshot saved to {:?}", path);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_phones() {
        let t = "Kontakt: +49 30 12345678 oder 0049 30 87654321";
        let p = extract_telephone_numbers(t);
        assert!(!p.is_empty());
    }
}
