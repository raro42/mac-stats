//! CDP (Chrome DevTools Protocol) browser agent.
//!
//! To use the browser, either:
//! 1. Start Chrome yourself: `Google Chrome --remote-debugging-port=9222`
//! 2. Or let mac-stats launch Chrome on 9222 when nothing is listening (requires Chrome installed).
//!
//! Supports BROWSER_NAVIGATE / BROWSER_CLICK / BROWSER_INPUT / BROWSER_SCROLL / BROWSER_EXTRACT (index-based state). Session is kept
//! until idle longer than Config::browser_idle_timeout_secs() (default 1 hour).
//! When CDP is unavailable, HTTP fallback (fetch + scraper) provides NAVIGATE/CLICK/INPUT/EXTRACT without Chrome.

mod http_fallback;

pub use http_fallback::{click_http, extract_http, input_http, navigate_http};

use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Browser state for BROWSER_NAVIGATE / BROWSER_CLICK / BROWSER_INPUT
// ---------------------------------------------------------------------------

/// One interactive element (link, button, input) with 1-based index for the LLM.
#[derive(Debug, Clone)]
pub struct Interactable {
    pub index: u32,
    pub tag: String,
    pub text: String,
    pub href: Option<String>,
    pub placeholder: Option<String>,
    pub input_type: Option<String>,
}

/// Current page state: URL, title, and numbered list of interactables.
#[derive(Debug, Clone)]
pub struct BrowserState {
    pub current_url: String,
    pub page_title: Option<String>,
    pub interactables: Vec<Interactable>,
}

/// Raw row returned from JS get_interactables snippet (before assigning index).
#[derive(Debug, Deserialize)]
struct InteractableRow {
    tag: String,
    text: String,
    #[serde(default)]
    href: Option<String>,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(default)]
    input_type: Option<String>,
}

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

/// Ensure Chrome is listening on port (launch if not). Call before retrying CDP when it failed.
pub fn ensure_chrome_on_port(port: u16) {
    if get_ws_url(port).is_ok() {
        return;
    }
    if launch_chrome_on_port(port).is_ok() {
        info!("Browser agent [CDP]: launched Chrome on port {} (caller may retry CDP)", port);
        std::thread::sleep(Duration::from_secs(4));
    }
}

/// Launch Chrome with --remote-debugging-port so mac-stats can connect. Chrome keeps running (process is detached).
/// Returns Ok(()) if spawn succeeded; caller should wait 2–3s then try get_ws_url(port) / connect_cdp(port).
#[cfg(target_os = "macos")]
fn launch_chrome_on_port(port: u16) -> Result<(), String> {
    let chrome_path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
    Command::new(chrome_path)
        .arg(format!("--remote-debugging-port={}", port))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Launch Chrome: {} (is Chrome installed at {}?)", e, chrome_path))?;
    info!("Browser agent [CDP]: launched Chrome on port {} (detached)", port);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn launch_chrome_on_port(port: u16) -> Result<(), String> {
    let chrome_path = "google-chrome";
    Command::new(chrome_path)
        .arg(format!("--remote-debugging-port={}", port))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Launch Chrome: {} (start Chrome manually with --remote-debugging-port={})", e, port))?;
    info!("Browser agent [CDP]: launched Chrome on port {} (detached)", port);
    Ok(())
}

/// Launch Chrome via headless_chrome crate (fallback when we cannot launch on a fixed port).
fn launch_via_headless_chrome() -> Result<Browser, String> {
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
    Ok(b)
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

/// JS snippet that returns JSON array of visible interactive elements (tag, text, href, placeholder, input_type).
const GET_INTERACTABLES_JS: &str = r#"
(function() {
  var sel = 'a, button, input, textarea, [role="button"], [onclick], [type="submit"]';
  var nodes = document.querySelectorAll(sel);
  var out = [];
  for (var i = 0; i < nodes.length; i++) {
    var el = nodes[i];
    var rect = el.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) continue;
    var style = window.getComputedStyle(el);
    if (style.visibility === 'hidden' || style.display === 'none' || style.opacity === '0') continue;
    var tag = el.tagName.toLowerCase();
    var text = (el.innerText || el.textContent || el.value || el.placeholder || '').trim().slice(0, 200);
    var href = el.href ? el.href : null;
    var placeholder = el.placeholder ? el.placeholder : null;
    var inputType = el.type ? el.type : null;
    out.push({ tag: tag, text: text, href: href, placeholder: placeholder, input_type: inputType });
  }
  return JSON.stringify(out);
})()
"#;

/// Get visible interactive elements from the page via JS. Returns 1-based indices. Used for BROWSER_NAVIGATE state.
pub fn get_interactables(tab: &headless_chrome::Tab) -> Result<Vec<Interactable>, String> {
    let result = tab
        .evaluate(GET_INTERACTABLES_JS, false)
        .map_err(|e| format!("Evaluate get_interactables: {}", e))?;
    let json_str = result
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .ok_or_else(|| "get_interactables JS did not return a string".to_string())?;
    let rows: Vec<InteractableRow> =
        serde_json::from_str(json_str).map_err(|e| format!("Parse interactables JSON: {}", e))?;
    let interactables = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| Interactable {
            index: (i + 1) as u32,
            tag: row.tag,
            text: row.text,
            href: row.href,
            placeholder: row.placeholder,
            input_type: row.input_type,
        })
        .collect();
    Ok(interactables)
}

/// Get current browser state (URL, title, interactables). Call after navigate or after click/input.
pub fn get_browser_state(tab: &headless_chrome::Tab) -> Result<BrowserState, String> {
    let current_url = tab.get_url();
    let page_title = tab
        .evaluate("document.title", false)
        .ok()
        .and_then(|r| r.value.as_ref().and_then(|v| v.as_str().map(String::from)));
    let interactables = get_interactables(tab)?;
    Ok(BrowserState {
        current_url,
        page_title,
        interactables,
    })
}

/// Format BrowserState as a string for the LLM (Current page: URL, Elements: [1] ...).
pub fn format_browser_state_for_llm(state: &BrowserState) -> String {
    let mut s = format!("Current page: {}\n", state.current_url);
    if let Some(ref t) = state.page_title {
        s.push_str(&format!("Title: {}\n", t));
    }
    s.push_str("Elements:\n");
    for i in &state.interactables {
        let kind = if i.tag == "a" {
            "link"
        } else if i.tag == "input" || i.tag == "textarea" {
            "input"
        } else {
            "button"
        };
        let label = if !i.text.is_empty() {
            i.text.as_str()
        } else if let Some(ref p) = i.placeholder {
            p.as_str()
        } else if let Some(ref h) = i.href {
            h.as_str()
        } else {
            "(no label)"
        };
        let label_escaped = label.replace('\n', " ").chars().take(80).collect::<String>();
        s.push_str(&format!("[{}] {} '{}'\n", i.index, kind, label_escaped));
    }
    if state.interactables.is_empty() {
        s.push_str("(no interactive elements found)\n");
    }
    s
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

/// Cached browser session: (Browser, last_used, was_headless). Dropped when idle longer than browser_idle_timeout_secs.
static BROWSER_SESSION: OnceLock<Mutex<Option<(Browser, Instant, bool)>>> = OnceLock::new();

/// User said "headless" → true (no visible window). User said "browser" or default → false (visible desktop app).
static PREFER_HEADLESS: AtomicBool = AtomicBool::new(false);

/// Set headless preference for this request. Call at start of tool loop from question.
/// "headless" in question → true. Otherwise → false (visible Chrome).
pub fn set_prefer_headless_for_run(prefer: bool) {
    PREFER_HEADLESS.store(prefer, Ordering::Relaxed);
}

fn browser_session() -> &'static Mutex<Option<(Browser, Instant, bool)>> {
    BROWSER_SESSION.get_or_init(|| Mutex::new(None))
}

fn is_connection_error(err_msg: &str) -> bool {
    err_msg.contains("connection is closed")
        || err_msg.contains("underlying connection")
        || err_msg.contains("timeout while listening")
        || err_msg.contains("Unable to make method calls")
}

/// Clear the cached browser session so the next use will reconnect or relaunch. Call when a connection error is detected.
fn clear_browser_session_on_error(err_msg: &str) {
    if is_connection_error(err_msg) {
        if let Ok(mut guard) = browser_session().lock() {
            if guard.is_some() {
                *guard = None;
                info!("Browser agent [CDP]: cleared session after connection error (next use will reconnect or relaunch)");
            }
        }
    }
}

/// Run f(). On connection error, clear session and retry once for seamless recovery from stale sessions.
fn with_connection_retry<F, T>(f: F) -> Result<T, String>
where
    F: Fn() -> Result<T, String>,
{
    match f() {
        Ok(v) => Ok(v),
        Err(e) => {
            if is_connection_error(&e) {
                clear_browser_session_on_error(&e);
                info!("Browser agent [CDP]: retrying after connection error (session cleared)");
                f()
            } else {
                Err(e)
            }
        }
    }
}

/// Get or create a browser; reuse if last use was within idle timeout and preference matches, else close and create new.
fn get_or_create_browser(port: u16) -> Result<Browser, String> {
    let timeout_secs = crate::config::Config::browser_idle_timeout_secs();
    let prefer_headless = PREFER_HEADLESS.load(Ordering::Relaxed);
    let mut guard = browser_session().lock().map_err(|e| e.to_string())?;
    let now = Instant::now();
    if let Some((ref browser, last_used, was_headless)) = guard.as_ref() {
        if now.duration_since(*last_used).as_secs() < timeout_secs && *was_headless == prefer_headless {
            let b = browser.clone();
            *guard = Some((b.clone(), now, prefer_headless));
            info!(
                "Browser agent [CDP]: reusing existing session (idle timeout {}s, headless={})",
                timeout_secs, prefer_headless
            );
            return Ok(b);
        }
        if *was_headless != prefer_headless {
            info!("Browser agent [CDP]: preference changed (headless {} → {}), creating new session", was_headless, prefer_headless);
        } else {
            info!(
                "Browser agent [CDP]: session idle > {}s, closing browser",
                timeout_secs
            );
        }
    }
    *guard = None;
    drop(guard);
    let browser = if prefer_headless {
        info!("Browser agent [CDP]: user requested headless — launching headless Chrome (no visible window)");
        launch_via_headless_chrome()?
    } else if get_ws_url(port).is_ok() {
        info!("Browser agent [CDP]: connecting to Chrome on port {} (visible)", port);
        connect_cdp(port)?
    } else {
        info!("Browser agent [CDP]: no Chrome on port {}, launching visible Chrome on {}", port, port);
        if launch_chrome_on_port(port).is_ok() {
            std::thread::sleep(Duration::from_secs(3));
            if get_ws_url(port).is_ok() {
                info!("Browser agent [CDP]: connecting to Chrome on port {} (after launch, visible)", port);
                connect_cdp(port)?
            } else {
                warn!("Browser agent [CDP]: Chrome launch may have failed or not ready; falling back to headless_chrome launcher");
                launch_via_headless_chrome()?
            }
        } else {
            info!("Browser agent [CDP]: could not launch Chrome on {}, using headless_chrome launcher", port);
            launch_via_headless_chrome()?
        }
    };
    *browser_session().lock().map_err(|e| e.to_string())? = Some((browser.clone(), Instant::now(), prefer_headless));
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

const PORT: u16 = 9222;

/// Get the current tab from BROWSER_SESSION. Fails if no browser or no tab.
fn get_current_tab() -> Result<(Browser, Arc<headless_chrome::Tab>), String> {
    let browser = get_or_create_browser(PORT)?;
    let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    let tab = tabs
        .first()
        .cloned()
        .ok_or_else(|| "No tab in browser".to_string())?;
    drop(tabs);
    Ok((browser, tab))
}

/// Navigate to URL and return formatted browser state for the LLM. Used by BROWSER_NAVIGATE.
pub fn navigate_and_get_state(url: &str) -> Result<String, String> {
    with_connection_retry(|| navigate_and_get_state_inner(url))
}

fn navigate_and_get_state_inner(url: &str) -> Result<String, String> {
    let url_normalized = normalize_url_for_screenshot(url);
    info!("Browser agent [CDP]: BROWSER_NAVIGATE: {}", url_normalized);
    let (_, tab) = get_current_tab().map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    tab.navigate_to(&url_normalized)
        .map_err(|e| {
            let s = format!("Navigate: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    tab.wait_until_navigated()
        .map_err(|e| {
            let s = format!("Wait navigated: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    std::thread::sleep(Duration::from_millis(1500));
    let state = get_browser_state(&tab).map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    Ok(format_browser_state_for_llm(&state))
}

/// Click the Nth interactive element (1-based index). Returns updated browser state string.
pub fn click_by_index(index: u32) -> Result<String, String> {
    with_connection_retry(|| click_by_index_inner(index))
}

fn click_by_index_inner(index: u32) -> Result<String, String> {
    if index == 0 {
        return Err("BROWSER_CLICK index must be >= 1".to_string());
    }
    let (_, tab) = get_current_tab().map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    let click_js = format!(
        r#"
(function() {{
  var sel = 'a, button, input, textarea, [role="button"], [onclick], [type="submit"]';
  var nodes = document.querySelectorAll(sel);
  var visible = [];
  for (var i = 0; i < nodes.length; i++) {{
    var el = nodes[i];
    var rect = el.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) continue;
    var style = window.getComputedStyle(el);
    if (style.visibility === 'hidden' || style.display === 'none' || style.opacity === '0') continue;
    visible.push(el);
  }}
  var idx = {};
  if (idx >= 1 && idx <= visible.length) {{
    visible[idx - 1].click();
    return 'clicked';
  }}
  return 'index out of range (max ' + visible.length + ')';
}})()
"#,
        index
    );
    let result = tab.evaluate(&click_js, false).map_err(|e| {
        let s = format!("Click evaluate: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    let msg = result
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("?")
        .to_string();
    if msg != "clicked" {
        return Err(format!("BROWSER_CLICK: {}", msg));
    }
    std::thread::sleep(Duration::from_millis(800));
    let state = get_browser_state(&tab).map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    Ok(format_browser_state_for_llm(&state))
}

/// Type text into the Nth interactive element (1-based index). Returns updated browser state string.
pub fn input_by_index(index: u32, text: &str) -> Result<String, String> {
    with_connection_retry(|| input_by_index_inner(index, text))
}

fn input_by_index_inner(index: u32, text: &str) -> Result<String, String> {
    if index == 0 {
        return Err("BROWSER_INPUT index must be >= 1".to_string());
    }
    let (_, tab) = get_current_tab().map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    let input_js = format!(
        r#"
(function() {{
  var sel = 'a, button, input, textarea, [role="button"], [onclick], [type="submit"]';
  var nodes = document.querySelectorAll(sel);
  var visible = [];
  for (var i = 0; i < nodes.length; i++) {{
    var el = nodes[i];
    var rect = el.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) continue;
    var style = window.getComputedStyle(el);
    if (style.visibility === 'hidden' || style.display === 'none' || style.opacity === '0') continue;
    visible.push(el);
  }}
  var idx = {};
  if (idx < 1 || idx > visible.length) return 'index out of range (max ' + visible.length + ')';
  var el = visible[idx - 1];
  if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable) {{
    el.focus();
    if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
      el.value = "{}";
      el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    }} else {{
      el.innerText = "{}";
    }}
    return 'typed';
  }}
  return 'element at index is not an input or textarea';
}})()
"#,
        index,
        escaped,
        escaped
    );
    let result = tab.evaluate(&input_js, false).map_err(|e| {
        let s = format!("Input evaluate: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    let msg = result
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("?")
        .to_string();
    if msg != "typed" {
        return Err(format!("BROWSER_INPUT: {}", msg));
    }
    std::thread::sleep(Duration::from_millis(300));
    let state = get_browser_state(&tab).map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    Ok(format_browser_state_for_llm(&state))
}

/// Scroll the current page. Arg: "down", "up", "bottom", "top", or pixels (e.g. "500"). Returns updated browser state.
pub fn scroll_page(arg: &str) -> Result<String, String> {
    with_connection_retry(|| scroll_page_inner(arg))
}

fn scroll_page_inner(arg: &str) -> Result<String, String> {
    let (_, tab) = get_current_tab().map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    let arg = arg.trim().to_lowercase();
    let scroll_js = if arg == "bottom" || arg == "end" {
        "window.scrollTo(0, document.body.scrollHeight); 'scrolled to bottom'".to_string()
    } else if arg == "top" || arg == "start" {
        "window.scrollTo(0, 0); 'scrolled to top'".to_string()
    } else if arg == "down" {
        "window.scrollBy(0, 500); 'scrolled down 500px'".to_string()
    } else if arg == "up" {
        "window.scrollBy(0, -500); 'scrolled up 500px'".to_string()
    } else if let Ok(pixels) = arg.parse::<i32>() {
        let px = pixels.clamp(-10000, 10000);
        if px >= 0 {
            format!("window.scrollBy(0, {}); 'scrolled down {}px'", px, px)
        } else {
            format!("window.scrollBy(0, {}); 'scrolled up {}px'", px, -px)
        }
    } else {
        "window.scrollBy(0, 500); 'scrolled down 500px'".to_string()
    };
    tab.evaluate(&scroll_js, false).map_err(|e| {
        let s = format!("Scroll evaluate: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    std::thread::sleep(Duration::from_millis(400));
    let state = get_browser_state(&tab).map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    Ok(format_browser_state_for_llm(&state))
}

/// Search current page for a text pattern (like grep). Returns matches with surrounding context. Zero LLM cost.
/// Use to find specific text, verify content exists, or locate data. Use after BROWSER_NAVIGATE/CLICK.
pub fn search_page_text(pattern: &str) -> Result<String, String> {
    with_connection_retry(|| search_page_text_inner(pattern))
}

fn search_page_text_inner(pattern: &str) -> Result<String, String> {
    let (_, tab) = get_current_tab().map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    // Escape for JS string: backslash and quotes
    let pattern_escaped = pattern
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', " ")
        .replace('\r', " ");
    const CONTEXT_CHARS: i32 = 80;
    const MAX_RESULTS: i32 = 20;
    // Use indexOf for literal substring search (no regex escaping issues)
    let js = format!(
        r#"
(function() {{
  var scope = document.body;
  if (!scope) return {{ error: 'no body', matches: [], total: 0 }};
  var walker = document.createTreeWalker(scope, NodeFilter.SHOW_TEXT);
  var fullText = '';
  var nodeOffsets = [];
  while (walker.nextNode()) {{
    var node = walker.currentNode;
    var text = node.textContent;
    if (text && text.trim()) {{
      nodeOffsets.push({{ offset: fullText.length, length: text.length, node: node }});
      fullText += text;
    }}
  }}
  var pat = '{}';
  var patLower = pat.toLowerCase();
  var textLower = fullText.toLowerCase();
  var matches = [];
  var idx = textLower.indexOf(patLower);
  while (idx !== -1 && matches.length < {}) {{
    var start = Math.max(0, idx - {});
    var end = Math.min(fullText.length, idx + pat.length + {});
    var context = fullText.slice(start, end).replace(/\s+/g, ' ').trim();
    var elemPath = '';
    for (var i = 0; i < nodeOffsets.length; i++) {{
      var no = nodeOffsets[i];
      if (no.offset <= idx && no.offset + no.length > idx) {{
        var el = no.node.parentElement;
        while (el && el !== document.body) {{
          elemPath = (el.tagName ? el.tagName.toLowerCase() : '') + (elemPath ? ' > ' + elemPath : '');
          el = el.parentElement;
        }}
        break;
      }}
    }}
    matches.push({{ context: (start > 0 ? '...' : '') + context + (end < fullText.length ? '...' : ''), path: elemPath }});
    idx = textLower.indexOf(patLower, idx + 1);
  }}
  var totalFound = 0;
  var i = textLower.indexOf(patLower);
  while (i !== -1) {{ totalFound++; i = textLower.indexOf(patLower, i + 1); }}
  return {{ matches: matches, total: totalFound, has_more: totalFound > {} }};
}})()
"#,
        pattern_escaped,
        MAX_RESULTS,
        CONTEXT_CHARS,
        CONTEXT_CHARS,
        MAX_RESULTS
    );
    let result = tab.evaluate(&js, false).map_err(|e| {
        let s = format!("Search page evaluate: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    let value = result.value.as_ref().ok_or("search_page returned no value")?;
    let obj = value.as_object().ok_or("search_page did not return object")?;
    if let Some(err) = obj.get("error").and_then(|v| v.as_str()) {
        return Err(format!("search_page error: {}", err));
    }
    let total = obj.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let empty: &[serde_json::Value] = &[];
    let matches = obj.get("matches").and_then(|v| v.as_array()).map(|v| v.as_slice()).unwrap_or(empty);
    if total == 0 {
        return Ok(format!("No matches found for \"{}\" on page.", pattern));
    }
    let mut lines = vec![format!(
        "Found {} match{} for \"{}\" on page:",
        total,
        if total == 1 { "" } else { "es" },
        pattern
    )];
    lines.push(String::new());
    for (i, m) in matches.iter().enumerate() {
        let ctx = m
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let path = m.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let loc = if path.is_empty() {
            String::new()
        } else {
            format!(" (in {})", path)
        };
        lines.push(format!("[{}] {}{}", i + 1, ctx, loc));
    }
    let has_more = obj.get("has_more").and_then(|v| v.as_bool()).unwrap_or(false);
    if has_more {
        lines.push(format!(
            "\n... showing {} of {} total matches.",
            matches.len(),
            total
        ));
    }
    Ok(lines.join("\n"))
}

/// Extract visible text from the current page (body innerText). Use after BROWSER_NAVIGATE/CLICK to get page content for the LLM.
pub fn extract_page_text() -> Result<String, String> {
    with_connection_retry(extract_page_text_inner)
}

fn extract_page_text_inner() -> Result<String, String> {
    let (_, tab) = get_current_tab().map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    let text = get_page_text(&tab).map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    const MAX_EXTRACT_CHARS: usize = 30_000;
    let out = if text.chars().count() > MAX_EXTRACT_CHARS {
        format!(
            "{}\n\n[Truncated: {} chars total; showing first {}.]",
            text.chars().take(MAX_EXTRACT_CHARS).collect::<String>(),
            text.chars().count(),
            MAX_EXTRACT_CHARS
        )
    } else {
        text
    };
    Ok(out)
}

/// Take a screenshot of the current CDP tab (no navigation). Use after BROWSER_NAVIGATE + BROWSER_CLICK.
/// Saves PNG to ~/.mac-stats/screenshots/<timestamp>_current.png.
pub fn take_screenshot_current_page() -> Result<PathBuf, String> {
    with_connection_retry(take_screenshot_current_page_inner)
}

fn take_screenshot_current_page_inner() -> Result<PathBuf, String> {
    info!("Browser agent [CDP]: take_screenshot_current_page (no navigation)");
    let (_, tab) = get_current_tab().map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    let final_url = tab.get_url();
    info!("Browser agent [CDP]: screenshotting current page: {}", final_url);
    std::thread::sleep(Duration::from_secs(1));
    let png_data = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .map_err(|e| {
            let s = format!("Capture screenshot: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    let dir = crate::config::Config::screenshots_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create screenshots dir: {}", e))?;
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_current.png", ts);
    let path = dir.join(&filename);
    std::fs::write(&path, &png_data).map_err(|e| format!("Write screenshot: {}", e))?;
    info!("Browser agent [CDP]: screenshot saved to {:?}", path);
    Ok(path)
}

/// Take a screenshot of the given URL using CDP (reuses session if within idle timeout, else connects or launches).
/// When url is empty or "current", screenshots the current tab (use after BROWSER_NAVIGATE + BROWSER_CLICK).
/// Saves PNG to ~/.mac-stats/screenshots/<timestamp>_<domain>.png and returns the path.
/// Browser session is kept until unused for Config::browser_idle_timeout_secs() (default 1 hour).
pub fn take_screenshot(url: &str) -> Result<PathBuf, String> {
    with_connection_retry(|| take_screenshot_inner(url))
}

fn take_screenshot_inner(url: &str) -> Result<PathBuf, String> {
    let url_trimmed = url.trim();
    if url_trimmed.is_empty() || url_trimmed.eq_ignore_ascii_case("current") {
        return take_screenshot_current_page_inner();
    }
    info!("Browser agent [CDP]: take_screenshot called with url (raw): {:?}", url);
    let url_normalized = normalize_url_for_screenshot(url_trimmed);
    info!("Browser agent [CDP]: normalized URL: {}", url_normalized);
    let port = 9222u16;
    let browser = get_or_create_browser(port).map_err(|e| {
        clear_browser_session_on_error(&e);
        e
    })?;
    let tabs = browser.get_tabs().lock().map_err(|e| {
        let s = e.to_string();
        clear_browser_session_on_error(&s);
        s
    })?;
    let tab = tabs.first().cloned().ok_or_else(|| "No tab".to_string())?;
    drop(tabs);
    info!("Browser agent [CDP]: navigating to: {}", url_normalized);
    tab.navigate_to(&url_normalized)
        .map_err(|e| {
            let s = format!("Navigate: {}", e);
            warn!("Browser agent [CDP]: navigate_to failed: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    tab.wait_until_navigated()
        .map_err(|e| {
            let s = format!("Wait navigated: {}", e);
            warn!("Browser agent [CDP]: wait_until_navigated failed: {}", e);
            clear_browser_session_on_error(&s);
            s
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
        .map_err(|e| {
            let s = format!("Capture screenshot: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
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
