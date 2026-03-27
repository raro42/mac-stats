//! CDP (Chrome DevTools Protocol) browser agent.
//!
//! To use the browser, either:
//! 1. Start a Chromium-based browser yourself with remote debugging (default loopback port **9222**, or `browserCdpPort` in `config.json`).
//! 2. Or let mac-stats launch the configured Chromium binary on that port when nothing is listening (default: Google Chrome on macOS).
//!
//! Supports BROWSER_NAVIGATE / BROWSER_GO_BACK / BROWSER_GO_FORWARD / BROWSER_RELOAD / BROWSER_CLEAR_COOKIES / BROWSER_SWITCH_TAB / BROWSER_CLOSE_TAB / BROWSER_CLICK / BROWSER_HOVER / BROWSER_DRAG / BROWSER_INPUT / BROWSER_UPLOAD / BROWSER_KEYS / BROWSER_SCROLL / BROWSER_EXTRACT / BROWSER_SEARCH_PAGE / BROWSER_QUERY / BROWSER_SAVE_PDF (index-based state). Optional config-driven CDP **`Emulation.setDeviceMetricsOverride`** / **`setGeolocationOverride`** (see **`browserCdpEmulateViewport*`** / **`browserCdpEmulateGeolocation*`** in `config.json`). Session is kept
//! until idle longer than Config::browser_idle_timeout_secs() (default 5 minutes; configurable).
//! On a **new** CDP attach (not session reuse), the agent polls until tab targets are enumerable (~8s max, ~200ms interval) so the WebSocket being up does not imply automation-ready.
//! When CDP is unavailable, HTTP fallback (fetch + scraper) provides NAVIGATE/CLICK/INPUT/EXTRACT without Chrome.
//!
//! **Visible Chrome mac-stats does not spawn:** When you attach to an already-running browser on the CDP port,
//! there is no child PID owned by mac-stats, so process-exit invalidation does not apply until the next
//! mac-stats-owned launch.

mod artifact_atomic;
pub mod artifact_limits;
pub(crate) mod cdp_downloads;
mod cdp_fetch_proxy_auth;
mod cdp_grant_permissions;
mod cdp_target_crash_listener;
mod cdp_trace_archive;
mod cdp_url;
mod cookie_storage;
mod credentials;
mod dom_snapshot;
mod http_fallback;
mod screenshot_annotate;
pub mod url_filter;

pub use http_fallback::{click_http, extract_http, input_http, navigate_http};

use std::collections::{HashMap, HashSet, VecDeque};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::{mac_stats_debug, mac_stats_info, mac_stats_warn};
use headless_chrome::browser::tab::point::Point;
use headless_chrome::browser::tab::ModifierKey;
use headless_chrome::protocol::cdp::types::Event;
use headless_chrome::protocol::cdp::Accessibility;
use headless_chrome::protocol::cdp::DOMDebugger;
use headless_chrome::protocol::cdp::Emulation;
use headless_chrome::protocol::cdp::Input;
use headless_chrome::protocol::cdp::Network;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Page::DialogType;
use headless_chrome::protocol::cdp::Runtime;
use headless_chrome::protocol::cdp::DOM;
use headless_chrome::types::Bounds;
use headless_chrome::Browser;
use headless_chrome::Element;
use headless_chrome::LaunchOptions;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};
use tokio::sync::oneshot;
use url::Url;

// ---------------------------------------------------------------------------
// Browser state for BROWSER_NAVIGATE / history / reload / BROWSER_CLICK / BROWSER_INPUT
// ---------------------------------------------------------------------------

/// One interactive element (link, button, input) with 1-based index for the LLM.
#[derive(Debug, Clone)]
pub struct Interactable {
    pub index: u32,
    /// Lowercase HTML tag name (e.g. `input`, `select`, `textarea`).
    pub tag: String,
    pub text: String,
    pub href: Option<String>,
    pub placeholder: Option<String>,
    pub input_type: Option<String>,
    /// True when `isContentEditable` (rich text / role=textbox regions).
    pub contenteditable: bool,
    /// Heuristic: class / `data-provide` suggests a datepicker widget on a text field.
    pub datepicker_like: bool,
    /// From CDP Accessibility tree (`Accessibility.getFullAXTree`), keyed by `backendDOMNodeId`.
    pub accessible_name: Option<String>,
    /// ARIA / AX role string when available (compact).
    pub ax_role: Option<String>,
    /// DOM backend node id for stale-index detection (CDP `DOM.describeNode` on the element object).
    pub backend_dom_node_id: Option<DOM::BackendNodeId>,
    /// HTML `name` attribute when present (helps identity remapping).
    pub dom_name: Option<String>,
    /// HTML `aria-label` when present.
    pub aria_label: Option<String>,
    /// Viewport-relative bounds from `DOMSnapshot.captureSnapshot` (CSS px), when available.
    pub bounds_css: Option<(f64, f64, f64, f64)>,
    /// `getBoundingClientRect()` in CSS px from page JS; preferred for screenshot overlays (not overwritten by snapshot).
    pub annot_bounds_css: Option<(f64, f64, f64, f64)>,
    /// True when collected from a cross-origin iframe (viewport coords do not match top-level screenshot).
    pub from_subframe: bool,
    /// True when a higher paint-order layer intersects this box but does not fully cover it.
    pub covered: bool,
}

/// Layout metrics for LLM-facing snapshots (layout viewport + document scroll size + scroll offset).
/// **Viewport** uses `window.innerWidth` / `window.innerHeight` (layout viewport CSS px).
/// **Document** uses `max(documentElement scroll/client, body scroll/client)` for full scrollable size.
/// **Scroll** uses `window.scrollX` / `window.scrollY`.
#[derive(Debug, Clone, Copy)]
pub struct BrowserLayoutMetrics {
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub document_width: u32,
    pub document_height: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserLayoutMetricsJs {
    scroll_x: f64,
    scroll_y: f64,
    viewport_width: f64,
    viewport_height: f64,
    document_width: f64,
    document_height: f64,
}

/// Current page state: URL, title, and numbered list of interactables.
#[derive(Debug, Clone)]
pub struct BrowserState {
    pub current_url: String,
    pub page_title: Option<String>,
    pub interactables: Vec<Interactable>,
    /// Count of `PerformanceResourceTiming` entries (`performance.getEntriesByType('resource')`), when available.
    pub resource_timing_entry_count: Option<u32>,
    /// CDP `Runtime.evaluate` geometry for the focused frame; absent if evaluation failed.
    pub layout_metrics: Option<BrowserLayoutMetrics>,
}

// ---------------------------------------------------------------------------
// Bounded page diagnostics for tool state (opt-in)
// ---------------------------------------------------------------------------
const DIAG_MAX_CONSOLE_LINES: usize = 10;
const DIAG_MAX_CONSOLE_CHARS_PER_LINE: usize = 200;
const DIAG_MAX_UNCAUGHT_EXC_MESSAGES: usize = 2;
const DIAG_MAX_UNCAUGHT_EXC_CHARS_PER_MESSAGE: usize = 200;

/// Recent auto-dismissed JS dialogs shown in CDP browser snapshots (bounded for context size).
const CDP_JS_DIALOG_HISTORY_MAX: usize = 3;
const CDP_JS_DIALOG_MESSAGE_MAX_CHARS: usize = 200;

fn normalize_diagnostic_text(s: &str, max_chars: usize) -> String {
    // Keep it compact for tool results: collapse whitespace and hard-truncate.
    let compact = s.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(max_chars).collect::<String>()
}

fn push_bounded_dedup(queue: &mut VecDeque<String>, item: String, max_len: usize) {
    if queue.iter().any(|x| *x == item) {
        return;
    }
    queue.push_back(item);
    while queue.len() > max_len {
        queue.pop_front();
    }
}

/// CDP `Log` / `Runtime` events → bounded diagnostic queues (shared by navigate + extract).
fn push_cdp_diagnostic_event(
    event: &Event,
    console_buf: &Arc<Mutex<VecDeque<String>>>,
    exc_buf: &Arc<Mutex<VecDeque<String>>>,
) {
    match event {
        Event::LogEntryAdded(ev) => {
            let level = &ev.params.entry.level;
            let prefix = match level {
                headless_chrome::protocol::cdp::Log::LogEntryLevel::Error => Some("error"),
                headless_chrome::protocol::cdp::Log::LogEntryLevel::Warning => Some("warning"),
                _ => None,
            };
            let Some(prefix) = prefix else {
                return;
            };
            let raw_text = ev.params.entry.text.trim();
            if raw_text.is_empty() {
                return;
            }
            let normalized = normalize_diagnostic_text(raw_text, DIAG_MAX_CONSOLE_CHARS_PER_LINE);
            if normalized.is_empty() {
                return;
            }
            let line = format!("[{}] {}", prefix, normalized);
            if let Ok(mut q) = console_buf.lock() {
                push_bounded_dedup(&mut q, line, DIAG_MAX_CONSOLE_LINES);
            }
        }
        Event::RuntimeExceptionThrown(ev) => {
            let details = &ev.params.exception_details;
            let raw_text = if !details.text.trim().is_empty() {
                details.text.as_str()
            } else {
                details
                    .stack_trace
                    .as_ref()
                    .and_then(|st| st.description.as_deref())
                    .unwrap_or("")
            };
            let raw_text = raw_text.trim();
            if raw_text.is_empty() {
                return;
            }
            let normalized = normalize_diagnostic_text(
                raw_text,
                DIAG_MAX_UNCAUGHT_EXC_CHARS_PER_MESSAGE,
            );
            if normalized.is_empty() {
                return;
            }
            let msg = format!("Uncaught exception: {}", normalized);
            if let Ok(mut q) = exc_buf.lock() {
                push_bounded_dedup(&mut q, msg, DIAG_MAX_UNCAUGHT_EXC_MESSAGES);
            }
        }
        _ => {}
    }
}

type CdpDiagListenerWeak =
    std::sync::Weak<dyn headless_chrome::browser::tab::EventListener<Event> + Send + Sync>;

/// Enable CDP Log + Runtime, register bounded diagnostic listener. Caller must remove listener and disable domains when done.
fn try_attach_bounded_cdp_page_diagnostics(
    tab: &headless_chrome::Tab,
) -> Option<(
    Arc<Mutex<VecDeque<String>>>,
    Arc<Mutex<VecDeque<String>>>,
    CdpDiagListenerWeak,
)> {
    let console_buf: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let exc_buf: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let c = Arc::clone(&console_buf);
    let e = Arc::clone(&exc_buf);
    let listener = Arc::new(move |event: &Event| push_cdp_diagnostic_event(event, &c, &e));

    if let Err(e) = tab.enable_log().and_then(|t| t.enable_runtime()) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: failed to enable diagnostics log/runtime: {} (continuing)",
            e
        );
        return None;
    }

    match tab.add_event_listener(listener) {
        Ok(w) => Some((console_buf, exc_buf, w)),
        Err(e) => {
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: failed to register diagnostics event listener: {} (continuing)",
                e
            );
            let _ = tab.disable_runtime();
            let _ = tab.disable_log();
            None
        }
    }
}

fn detach_bounded_cdp_page_diagnostics(tab: &headless_chrome::Tab, weak: &CdpDiagListenerWeak) {
    let _ = tab.remove_event_listener(weak);
    let _ = tab.disable_runtime();
    let _ = tab.disable_log();
}

/// `## Page diagnostics` block for tool results, or empty when there is nothing to show.
fn format_bounded_page_diagnostics_tool_section(
    console_buf: &Arc<Mutex<VecDeque<String>>>,
    exc_buf: &Arc<Mutex<VecDeque<String>>>,
) -> String {
    let console_vec = console_buf
        .lock()
        .map(|q| q.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let exc_vec = exc_buf
        .lock()
        .map(|q| q.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    if console_vec.is_empty() && exc_vec.is_empty() {
        return String::new();
    }
    let mut s = String::from("\n## Page diagnostics\n");
    if !console_vec.is_empty() {
        s.push_str("Console (error/warning):\n");
        for l in console_vec {
            s.push_str(&format!("- {}\n", l));
        }
    }
    if !exc_vec.is_empty() {
        s.push_str("Uncaught exceptions:\n");
        for m in exc_vec {
            s.push_str(&format!("- {}\n", m));
        }
    }
    s
}

/// Short tail wait so `console.*` fired right after `Runtime.evaluate` can reach the CDP listener.
const DIAG_EXTRACT_TAIL_WAIT: Duration = Duration::from_millis(100);

/// RAII: bounded CDP diagnostics during **BROWSER_EXTRACT** (same caps as navigate; fresh buffers per extract).
struct TabExtractDiagnosticsSession {
    tab: Arc<headless_chrome::Tab>,
    listener: Option<CdpDiagListenerWeak>,
    console: Arc<Mutex<VecDeque<String>>>,
    exc: Arc<Mutex<VecDeque<String>>>,
}

impl TabExtractDiagnosticsSession {
    fn try_start(tab: Arc<headless_chrome::Tab>) -> Option<Self> {
        let (console, exc, w) = try_attach_bounded_cdp_page_diagnostics(tab.as_ref())?;
        Some(Self {
            tab,
            listener: Some(w),
            console,
            exc,
        })
    }

    fn section(&self) -> String {
        format_bounded_page_diagnostics_tool_section(&self.console, &self.exc)
    }
}

impl Drop for TabExtractDiagnosticsSession {
    fn drop(&mut self) {
        if let Some(w) = self.listener.take() {
            detach_bounded_cdp_page_diagnostics(self.tab.as_ref(), &w);
        }
    }
}

// ---------------------------------------------------------------------------
// Recent JS dialogs (CDP auto-dismiss transcript for the LLM)
// ---------------------------------------------------------------------------
//
// Bounded FIFO for the current CDP browser session: each snapshot includes the same last
// N entries until pushed off or the session is replaced (idle timeout, errors, shutdown).

static CDP_JS_DIALOG_HISTORY: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn cdp_js_dialog_history() -> &'static Mutex<VecDeque<String>> {
    CDP_JS_DIALOG_HISTORY.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn clear_cdp_js_dialog_history() {
    if let Ok(mut q) = cdp_js_dialog_history().lock() {
        q.clear();
    }
}

fn dialog_type_short(t: &DialogType) -> &'static str {
    match t {
        DialogType::Alert => "alert",
        DialogType::Confirm => "confirm",
        DialogType::Prompt => "prompt",
        DialogType::Beforeunload => "beforeunload",
    }
}

fn record_cdp_js_dialog_dismissed(dialog_type: &DialogType, message: &str) {
    let msg = normalize_diagnostic_text(message, CDP_JS_DIALOG_MESSAGE_MAX_CHARS);
    let line = format!("[{}] {}", dialog_type_short(dialog_type), msg);
    if let Ok(mut q) = cdp_js_dialog_history().lock() {
        q.push_back(line);
        while q.len() > CDP_JS_DIALOG_HISTORY_MAX {
            q.pop_front();
        }
    }
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: recorded JS dialog for LLM snapshot (type={:?})",
        dialog_type
    );
}

fn recent_js_dialogs_section_for_cdp_llm() -> String {
    let lines: Vec<String> = cdp_js_dialog_history()
        .lock()
        .map(|q| q.iter().cloned().collect())
        .unwrap_or_default();
    let mut s = String::from("Recent JS dialogs:\n");
    if lines.is_empty() {
        s.push_str("None\n");
    } else {
        for line in lines {
            s.push_str(&line);
            s.push('\n');
        }
    }
    s
}

/// Raw row returned from JS get_interactables snippet (before assigning index).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct InteractableRow {
    /// From page script (`tag_name` in JSON); legacy `tag` accepted if present.
    #[serde(rename = "tag_name", alias = "tag")]
    tag: String,
    text: String,
    #[serde(default)]
    href: Option<String>,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(default)]
    input_type: Option<String>,
    #[serde(default)]
    contenteditable: bool,
    #[serde(default)]
    datepicker_like: bool,
    #[serde(default)]
    dom_name: Option<String>,
    #[serde(default)]
    aria_label: Option<String>,
    #[serde(default)]
    bounds_css: Option<(f64, f64, f64, f64)>,
    #[serde(default)]
    covered: bool,
    /// Top-level viewport CSS bounds from `getBoundingClientRect()` (JSON key `annot_bounds`).
    #[serde(default, rename = "annot_bounds")]
    annot_bounds: Option<[f64; 4]>,
    /// Set for cross-origin iframe rows; screenshot annotation skips these.
    #[serde(skip, default)]
    pub(crate) from_subframe: bool,
}

/// JSON from `SPA_READINESS_JS` (`Runtime.evaluate` string result).
#[derive(Debug, Deserialize)]
struct SpaReadinessRow {
    element_count: u64,
    text_length: u64,
    has_body: bool,
}

#[derive(Debug, Clone)]
struct SpaReadinessSnapshot {
    element_count: usize,
    text_length: usize,
    has_body: bool,
}

/// How many times we hit `/json/version` before deciding nothing is listening (transient false negatives on loopback).
const CDP_DISCOVERY_ATTEMPTS: u32 = 4;
/// Pause between discovery attempts (~1s total backoff across retries).
const CDP_DISCOVERY_RETRY_SLEEP_MS: u64 = 225;
/// Host used for `http://…/json/version` discovery today (loopback). Advertised `ws://0.0.0.0/…` URLs are rewritten to this.
const CDP_DISCOVERY_HTTP_HOST: &str = "127.0.0.1";

/// Resolve per-request HTTP timeout for CDP `GET /json/version` (and equivalent probes). Today all discovery targets
/// loopback; if mac-stats later adds a configurable non-loopback CDP host, apply longer minimum floors here for remote
/// profiles (OpenClaw-style longer floors vs pure localhost).
fn resolve_cdp_http_timeout_for_discovery_host(host: &str) -> Duration {
    let _ = host;
    Duration::from_secs(crate::config::Config::browser_cdp_http_timeout_secs())
}

/// HTTP client timeout for each `/json/version` probe (config: `browserCdpHttpTimeoutSecs`).
fn cdp_discovery_http_timeout_duration() -> Duration {
    resolve_cdp_http_timeout_for_discovery_host(CDP_DISCOVERY_HTTP_HOST)
}

/// `User-Agent` for CDP HTTP discovery (`/json/version`); identifiable in traces and DevTools.
fn cdp_discovery_user_agent() -> String {
    format!("mac-stats/{}", env!("CARGO_PKG_VERSION"))
}

/// Fetch WebSocket debugger URL from Chrome running with --remote-debugging-port.
fn get_ws_url_with_timeout(port: u16, timeout: Duration) -> Result<String, String> {
    let discovery_seed = format!("http://{}:{}", CDP_DISCOVERY_HTTP_HOST, port);
    let url = cdp_url::cdp_http_base_from_endpoint(&discovery_seed)
        .ok()
        .and_then(|base| base.join("json/version").ok())
        .map(|u| u.to_string())
        .unwrap_or_else(|| cdp_url::json_version_probe_url(CDP_DISCOVERY_HTTP_HOST, port));
    let url_for_logs = cdp_url::redact_cdp_url(&url);
    let res = (|| {
        let user_agent = cdp_discovery_user_agent();
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: discovery GET {} with User-Agent {}",
            url_for_logs,
            user_agent
        );
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .no_proxy()
            .user_agent(user_agent.as_str())
            .build()
            .map_err(|e| format!("HTTP client: {}", e))?;
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| format!("Request to {}: {}", url_for_logs, e))?;
        if !resp.status().is_success() {
            return Err(format!("{} returned {}", url_for_logs, resp.status()));
        }
        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Parse JSON from {}: {}", url_for_logs, e))?;
        let ws_raw = json
            .get("webSocketDebuggerUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "webSocketDebuggerUrl not found in /json/version".to_string())?
            .to_string();
        let ws = cdp_url::rewrite_ws_debugger_host_for_discovery(&ws_raw, CDP_DISCOVERY_HTTP_HOST);
        if ws != ws_raw {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: adjusted webSocketDebuggerUrl bind-all host → {}",
                cdp_url::redact_cdp_url(&ws)
            );
        }
        Ok(ws)
    })();

    match &res {
        Ok(_) => record_cdp_http_probe_result(port, true, None),
        Err(e) => record_cdp_http_probe_result(port, false, Some(e.clone())),
    }

    res
}

fn get_ws_url(port: u16) -> Result<String, String> {
    get_ws_url_with_timeout(port, cdp_discovery_http_timeout_duration())
}

/// Probe loopback CDP with bounded retries before treating the port as free (avoids duplicate Chrome launch on flaky first probe).
fn try_discover_cdp_ws_url(port: u16) -> Result<String, String> {
    let mut last_err = String::new();
    for attempt in 0..CDP_DISCOVERY_ATTEMPTS {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(CDP_DISCOVERY_RETRY_SLEEP_MS));
        }
        match get_ws_url_with_timeout(port, cdp_discovery_http_timeout_duration()) {
            Ok(ws) => {
                if attempt > 0 {
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: CDP discovery succeeded on retry (attempt {}/{})",
                        attempt + 1,
                        CDP_DISCOVERY_ATTEMPTS
                    );
                }
                return Ok(ws);
            }
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

/// After visible `launch_chrome_on_port`, poll `GET /json/version` until `webSocketDebuggerUrl` is available or the deadline elapses.
/// Per attempt uses the configured HTTP discovery timeout (`browserCdpHttpTimeoutSecs`); overall wait is bounded by config (default ~15s).
#[allow(unused_assignments)] // `last_err` seed is unused if the first probe succeeds immediately
fn wait_for_cdp_http_after_visible_launch(port: u16) -> Result<(), String> {
    let max_wait =
        Duration::from_secs(crate::config::Config::browser_cdp_post_launch_max_wait_secs());
    let poll_interval =
        Duration::from_millis(crate::config::Config::browser_cdp_post_launch_poll_interval_ms());
    let deadline = Instant::now() + max_wait;
    let poll_started = Instant::now();
    let mut last_err = String::new();
    let mut attempt: u32 = 0;

    loop {
        attempt += 1;
        match get_ws_url_with_timeout(port, cdp_discovery_http_timeout_duration()) {
            Ok(_) => {
                if attempt > 1 {
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: post-launch CDP HTTP ready on port {} after {} attempts in {:?}",
                        port,
                        attempt,
                        poll_started.elapsed()
                    );
                }
                return Ok(());
            }
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: post-launch readiness probe {} on port {}: {}",
                    attempt,
                    port,
                    &e
                );
                last_err = e;
            }
        }

        let err_tail = if last_err.is_empty() {
            "(none)"
        } else {
            last_err.as_str()
        };
        if Instant::now() >= deadline {
            return Err(format!(
                "Chrome did not expose CDP on port {} within {:?} after launch. Last error: {}. Check that Chromium is installed, port {} is not in use by another app, or start the browser manually with --remote-debugging-port={}.",
                port, max_wait, err_tail, port, port
            ));
        }

        let remaining = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or_default();
        let sleep_for = poll_interval.min(remaining);
        if sleep_for.is_zero() {
            return Err(format!(
                "Chrome did not expose CDP on port {} within {:?} after launch. Last error: {}. Check that Chromium is installed, port {} is not in use by another app, or start the browser manually with --remote-debugging-port={}.",
                port, max_wait, err_tail, port, port
            ));
        }
        std::thread::sleep(sleep_for);
    }
}

/// After the CDP WebSocket handshake, wait until targets are enumerable (macOS user Chrome can expose the port before tabs are ready).
const CDP_ATTACH_READY_TIMEOUT: Duration = Duration::from_secs(15);
const CDP_ATTACH_READY_POLL: Duration = Duration::from_millis(200);

// ---------------------------------------------------------------------------
// Browser agent UX / navigation timing (tune in one place)
// ---------------------------------------------------------------------------
/// Poll interval while waiting for `Page.lifecycleEvent` network-idle-class signals after navigation.
const BROWSER_NAV_LIFECYCLE_POLL: Duration = Duration::from_millis(50);
/// Max time to wait for `networkIdle` / `networkAlmostIdle` after same-host navigation.
const BROWSER_NAV_NETWORK_IDLE_TIMEOUT_SAME_DOMAIN: Duration = Duration::from_secs(4);
/// Max time to wait for `networkIdle` / `networkAlmostIdle` after cross-host navigation.
const BROWSER_NAV_NETWORK_IDLE_TIMEOUT_CROSS_DOMAIN: Duration = Duration::from_secs(10);
/// Default post-navigate minimum dwell is **0.25s** via `Config::browser_post_navigate_min_dwell_secs()`
/// (replaces this former fixed constant).
/// After BROWSER_CLICK / coordinate click when the page may be settling.
const BROWSER_AFTER_CLICK_OR_COORD_SETTLE: Duration = Duration::from_millis(800);
/// After dismissing a cookie banner via click.
const BROWSER_COOKIE_BANNER_CLICK_SETTLE: Duration = Duration::from_millis(700);
/// After `history.back` / `history.forward` before waiting for load.
const BROWSER_HISTORY_SCRIPT_THEN_WAIT: Duration = Duration::from_millis(300);
/// After back/forward/reload navigation completes.
const BROWSER_AFTER_HISTORY_NAV_SETTLE: Duration = Duration::from_millis(500);
/// When `wait_until_navigated` fails without timing out (SPA/hash), pause before continuing.
const BROWSER_WAIT_UNTIL_NAVIGATED_FALLBACK: Duration = Duration::from_secs(2);
/// After resetting tab to `about:blank` for policy enforcement.
const BROWSER_POLICY_RESET_SETTLE: Duration = Duration::from_millis(400);
/// Stabilize before screenshotting the current tab.
const BROWSER_SCREENSHOT_STABILIZE_CURRENT: Duration = Duration::from_secs(1);
/// Stabilize after navigating for a URL screenshot.
const BROWSER_SCREENSHOT_STABILIZE_AFTER_URL_NAV: Duration = Duration::from_secs(2);
/// SPA blank-page retry: first extra wait before re-checking readiness.
const BROWSER_SPA_BLANK_RETRY_FIRST_WAIT: Duration = Duration::from_secs(3);
/// SPA blank-page retry: pause after reload before `wait_until_navigated`.
const BROWSER_SPA_BLANK_RETRY_RELOAD_PAUSE: Duration = Duration::from_millis(300);
/// SPA blank-page retry: final settle before last readiness snapshot.
const BROWSER_SPA_BLANK_RETRY_FINAL_WAIT: Duration = Duration::from_secs(5);
/// After BROWSER_INPUT when the DOM may be updating.
const BROWSER_POST_INPUT_SETTLE: Duration = Duration::from_millis(300);
/// After BROWSER_HOVER when hover-only UI may be animating in.
const BROWSER_AFTER_HOVER_SETTLE: Duration = Duration::from_millis(400);
/// Interpolated steps for BROWSER_DRAG between element centers (excluding press/release endpoints).
const BROWSER_DRAG_MOVE_STEPS: u32 = 8;
/// Re-fetch browser state after a UI interaction (cookie banner, etc.).
const BROWSER_STATE_REFRESH_AFTER_UI_MS: Duration = Duration::from_millis(500);
/// After file upload or scroll evaluate, allow DOM/layout to settle.
const BROWSER_POST_UPLOAD_SETTLE: Duration = Duration::from_millis(400);
const BROWSER_POST_SCROLL_SETTLE: Duration = Duration::from_millis(400);
/// Phone-extract flow: pause after navigation sync before scrolling.
const BROWSER_PHONE_EXTRACT_POST_NAV: Duration = Duration::from_secs(2);
/// Phone-extract flow: pause after scroll-to-bottom before reading DOM.
const BROWSER_PHONE_EXTRACT_POST_SCROLL: Duration = Duration::from_secs(2);

/// Poll until `get_tabs` lock succeeds and targets are usable, or timeout. Only for fresh CDP attach (not cached session reuse).
///
/// Chrome may report **zero** page tabs when the user closed every tab or briefly during startup. In that case we still treat
/// attach as ready so [`get_current_tab`] can bootstrap `about:blank` (OpenClaw-style). After one successful empty read we do a
/// short follow-up poll so transient startup empties can populate before we accept zero tabs.
fn wait_for_cdp_targets_ready_after_attach(browser: &Browser) -> Result<(), String> {
    let deadline = Instant::now() + CDP_ATTACH_READY_TIMEOUT;
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        if Instant::now() >= deadline {
            break;
        }
        match browser.get_tabs().lock() {
            Ok(tabs) => {
                if !tabs.is_empty() {
                    mac_stats_debug!(
                        "browser/cdp",
                        "Browser agent [CDP]: attach readiness OK after {} probe(s), {} tab(s)",
                        attempt,
                        tabs.len()
                    );
                    return Ok(());
                }
                drop(tabs);
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: attach readiness probe {} — tab list empty (follow-up probe for transient startup)",
                    attempt
                );
                let remaining = deadline
                    .checked_duration_since(Instant::now())
                    .unwrap_or_default();
                let sleep_for = CDP_ATTACH_READY_POLL.min(remaining);
                if !sleep_for.is_zero() {
                    std::thread::sleep(sleep_for);
                }
                match browser.get_tabs().lock() {
                    Ok(tabs2) => {
                        if !tabs2.is_empty() {
                            mac_stats_debug!(
                                "browser/cdp",
                                "Browser agent [CDP]: attach readiness OK after {} probe(s), {} tab(s) (after empty-list follow-up)",
                                attempt,
                                tabs2.len()
                            );
                            return Ok(());
                        }
                        mac_stats_info!(
                            "browser/cdp",
                            "Browser agent [CDP]: attach readiness OK with 0 page tabs (get_current_tab will bootstrap about:blank if needed)"
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        let short = if msg.len() > 160 {
                            format!("{}…", &msg[..160])
                        } else {
                            msg
                        };
                        mac_stats_debug!(
                            "browser/cdp",
                            "Browser agent [CDP]: attach readiness follow-up probe failed: {}",
                            short
                        );
                        mac_stats_info!(
                            "browser/cdp",
                            "Browser agent [CDP]: attach readiness OK with 0 page tabs (follow-up lock failed; get_current_tab will bootstrap about:blank if needed)"
                        );
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                let msg = e.to_string();
                let short = if msg.len() > 160 {
                    format!("{}…", &msg[..160])
                } else {
                    msg
                };
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: attach readiness probe {} failed: {}",
                    attempt,
                    short
                );
            }
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or_default();
        let sleep_for = CDP_ATTACH_READY_POLL.min(remaining);
        if sleep_for.is_zero() {
            break;
        }
        std::thread::sleep(sleep_for);
    }
    Err(
        "Chrome debugging port accepted a connection, but page targets were not ready in time. \
If Chrome showed a prompt to allow remote debugging, approve it; keep Chrome running and retry."
            .to_string(),
    )
}

/// Open a CDP session from a WebSocket URL and wait until tab enumeration is reliable (attach path only).
/// Zero page tabs are allowed; [`get_current_tab`] bootstraps `about:blank` when needed.
fn connect_browser_to_ws_url(ws_url: &str) -> Result<Browser, String> {
    let inferred_port = Url::parse(ws_url)
        .ok()
        .and_then(|u| u.port())
        .unwrap_or_else(|| crate::config::Config::browser_cdp_port());
    let ws_redacted = cdp_url::redact_cdp_url(ws_url);

    let ws_connect_timeout =
        Duration::from_secs(crate::config::Config::browser_cdp_ws_connect_timeout_secs());
    let res: Result<Browser, String> = (|| {
        let browser = Browser::connect_with_timeout(ws_url.to_string(), ws_connect_timeout)
            .map_err(|e| format!("CDP connect (ws={}): {}", ws_redacted, e))?;
        wait_for_cdp_targets_ready_after_attach(&browser)?;
        Ok(browser)
    })();

    match &res {
        Ok(_) => record_cdp_attach_result(inferred_port, true, None),
        Err(e) => record_cdp_attach_result(inferred_port, false, Some(e.clone())),
    }

    res
}

/// Connect to Chrome at the given debugging port.
pub fn connect_cdp(port: u16) -> Result<Browser, String> {
    let ws_url = get_ws_url(port)?;
    mac_stats_info!(
        "browser",
        "Browser agent: connecting to CDP at port {}",
        port
    );
    connect_browser_to_ws_url(&ws_url)
}

/// Ensure Chrome is listening on port (launch if not). Call before retrying CDP when it failed.
/// When headless was requested for this run, we do not launch visible Chrome — retry will use headless launcher instead.
pub fn ensure_chrome_on_port(port: u16) {
    if prefer_headless_for_run() {
        return;
    }
    if try_discover_cdp_ws_url(port).is_ok() {
        return;
    }
    if let Ok(child) = launch_chrome_on_port(port) {
        spawn_visible_chrome_child_reaper(child);
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: launched Chromium on port {} (polling CDP readiness)",
            port
        );
        match wait_for_cdp_http_after_visible_launch(port) {
            Ok(()) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: post-launch CDP ready on port {} (ensure_chrome_on_port)",
                    port
                );
            }
            Err(e) => {
                mac_stats_warn!("browser/cdp", "Browser agent [CDP]: {}", e);
            }
        }
    }
}

/// Viewport size (width, height) from config.json (browserViewportWidth, browserViewportHeight); defaults 1800x2400.
fn viewport_width() -> u32 {
    crate::config::Config::browser_viewport_width()
}
fn viewport_height() -> u32 {
    crate::config::Config::browser_viewport_height()
}

/// Chromium flags that keep painting, timers, and CDP IPC responsive when the automation
/// Chrome window is not the foreground app. mac-stats is a menu-bar app, so the debugging
/// Chrome instance is often fully occluded; without these, Chromium may throttle renderers,
/// pause occluded windows, or delay IPC — producing stale screenshots or slow navigations.
/// Also drops the `AutomationControlled` blink feature so fewer sites gate on `navigator.webdriver`.
/// (Aligned with common browser-use launch sets; distinct from UA/proxy identity.)
const CHROME_MENU_BAR_AUTOMATION_FLAGS: &[&str] = &[
    "--disable-renderer-backgrounding",
    "--disable-backgrounding-occluded-windows",
    "--disable-ipc-flooding-protection",
    "--disable-features=CalculateNativeWinOcclusion",
    "--disable-blink-features=AutomationControlled",
];

fn validate_browser_chromium_user_data_dir(dir: &Path) -> Result<(), String> {
    if dir.exists() {
        if !dir.is_dir() {
            return Err(format!(
                "browserChromiumUserDataDir exists but is not a directory ({})",
                dir.display()
            ));
        }
        std::fs::read_dir(dir).map_err(|e| {
            format!(
                "browserChromiumUserDataDir is not readable ({}): {}",
                dir.display(),
                e
            )
        })?;
        return Ok(());
    }
    let Some(parent) = dir.parent() else {
        return Err(format!(
            "browserChromiumUserDataDir has no parent ({})",
            dir.display()
        ));
    };
    if !parent.exists() {
        return Err(format!(
            "parent directory of browserChromiumUserDataDir does not exist ({})",
            parent.display()
        ));
    }
    Ok(())
}

fn validate_visible_launch_chromium_executable(path: &Path) -> Result<(), String> {
    let configured = crate::config::Config::browser_chromium_executable_configured();
    if configured {
        if !path.exists() {
            return Err(format!(
                "browserChromiumExecutable not found at {} (check ~/.mac-stats/config.json or MAC_STATS_BROWSER_CHROMIUM_EXECUTABLE)",
                path.display()
            ));
        }
        if !path.is_file() {
            return Err(format!(
                "browserChromiumExecutable is not a regular file ({})",
                path.display()
            ));
        }
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        if !path.exists() {
            return Err(format!(
                "Google Chrome not found at default path {}. Install Chrome, set browserChromiumExecutable to Brave/Edge/Chromium, or start your browser manually with --remote-debugging-port",
                path.display()
            ));
        }
        if !path.is_file() {
            return Err(format!(
                "Default Chrome path is not a file ({})",
                path.display()
            ));
        }
    }
    Ok(())
}

/// Launch Chromium with `--remote-debugging-port` so mac-stats can connect. The process keeps running independently.
/// Returns the spawned [`Child`] so the caller can reap it (via a background [`Child::wait`]) and avoid zombies.
/// On success, poll HTTP discovery until CDP answers (`wait_for_cdp_http_after_visible_launch`) then `connect_cdp(port)`.
fn launch_chrome_on_port(port: u16) -> Result<Child, String> {
    let chrome_path = crate::config::Config::browser_chromium_executable_path();
    if let Some(ref udd) = crate::config::Config::browser_chromium_user_data_dir() {
        validate_browser_chromium_user_data_dir(udd)?;
    }
    validate_visible_launch_chromium_executable(&chrome_path)?;

    let mut cmd = Command::new(&chrome_path);
    cmd.arg(format!("--remote-debugging-port={}", port))
        .arg(format!(
            "--window-size={},{}",
            viewport_width(),
            viewport_height()
        ))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-extensions")
        .arg("--disable-background-networking")
        .arg("--disable-sync")
        .arg("--disable-default-apps")
        .arg("--disable-background-timer-throttling");
    if let Some(udd) = crate::config::Config::browser_chromium_user_data_dir() {
        cmd.arg(format!("--user-data-dir={}", udd.display()));
    }
    for flag in CHROME_MENU_BAR_AUTOMATION_FLAGS {
        cmd.arg(flag);
    }
    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            format!(
                "Launch Chromium ({}): {} — check browserChromiumExecutable or start the browser manually with --remote-debugging-port={}",
                chrome_path.display(),
                e,
                port
            )
        })?;
    let exe_hint = chrome_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("chromium");
    let pid = child.id();
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: launched {} on port {} (PID {}; user_data_dir={})",
        exe_hint,
        port,
        pid,
        crate::config::Config::browser_chromium_user_data_dir()
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "default".to_string())
    );
    Ok(child)
}

/// Reap a visible Chrome [`Child`] in the background (no session invalidation). Use after launch when we do not
/// adopt the process into the CDP session (e.g. `ensure_chrome_on_port`, or CDP connect failed after spawn).
fn spawn_visible_chrome_child_reaper(mut child: Child) {
    let pid = child.id();
    let spawn_result = std::thread::Builder::new()
        .name("mac-stats-chrome-reap".into())
        .spawn(move || {
            match child.wait() {
                Ok(status) => {
                    mac_stats_debug!(
                        "browser/cdp",
                        "Browser agent [CDP]: reaped mac-stats-spawned visible Chrome PID {} (status={:?})",
                        pid,
                        status.code()
                    );
                }
                Err(e) => {
                    mac_stats_debug!(
                        "browser/cdp",
                        "Browser agent [CDP]: wait on spawned visible Chrome PID {} failed: {}",
                        pid,
                        e
                    );
                }
            }
        });
    if let Err(e) = spawn_result {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: could not spawn reaper thread for Chrome PID {} ({}); child may become a zombie until app exit",
            pid,
            e
        );
    }
}

/// PID of the visible Chrome process mac-stats launched for the current CDP session (`0` = none / headless / user-started Chrome).
static OWNED_VISIBLE_CHROME_CHILD_PID: AtomicU32 = AtomicU32::new(0);

fn clear_owned_visible_chrome_child_pid() {
    OWNED_VISIBLE_CHROME_CHILD_PID.store(0, Ordering::SeqCst);
}

/// When the owned visible Chrome exits, invalidate the cached session (same teardown as other error paths).
/// Waits on [`Child::wait`] in a background thread so we never hold [`browser_session`] while blocking on the child.
fn spawn_owned_visible_chrome_exit_watcher(mut child: Child, pid: u32) {
    // Use `thread::spawn` (not `Builder`) so we never drop a `Child` inside a failed `spawn` closure (would zombie).
    let _ = std::thread::spawn(move || {
        let wait_outcome = child.wait();
        if OWNED_VISIBLE_CHROME_CHILD_PID.load(Ordering::SeqCst) != pid {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: visible Chrome PID {} exited but session no longer tracks this launch (wait={:?}); skipping session clear",
                pid,
                wait_outcome.as_ref().ok().and_then(|s| s.code())
            );
            return;
        }
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: mac-stats-launched visible Chrome exited (PID {}); invalidating CDP session",
            pid
        );
        clear_cached_browser_session(&format!("visible Chrome process exited (PID {})", pid));
    });
}

/// Launch Chrome via headless_chrome crate (fallback when we cannot launch on a fixed port).
fn launch_via_headless_chrome() -> Result<Browser, String> {
    let extra_args: Vec<&std::ffi::OsStr> = CHROME_MENU_BAR_AUTOMATION_FLAGS
        .iter()
        .map(|s| std::ffi::OsStr::new(*s))
        .chain([
            std::ffi::OsStr::new("--disable-software-rasterizer"),
            std::ffi::OsStr::new("--mute-audio"),
        ])
        .collect();
    let mut binding = LaunchOptions::default_builder();
    let mut builder = binding
        .headless(true)
        .window_size(Some((viewport_width(), viewport_height())))
        .args(extra_args);
    if crate::config::Config::browser_chromium_executable_configured() {
        let p = crate::config::Config::browser_chromium_executable_path();
        if !p.exists() || !p.is_file() {
            return Err(format!(
                "browserChromiumExecutable not found or not a file: {}",
                p.display()
            ));
        }
        builder = builder.path(Some(p));
    }
    if let Some(udd) = crate::config::Config::browser_chromium_user_data_dir() {
        validate_browser_chromium_user_data_dir(&udd)
            .map_err(|e| format!("Invalid browserChromiumUserDataDir: {}", e))?;
        builder = builder.user_data_dir(Some(udd));
    }
    let opts = builder
        .build()
        .map_err(|e| format!("Launch options: {}", e))?;
    let b = Browser::new(opts).map_err(|e| format!("Launch Chrome: {}", e))?;
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
    cdp_fetch_proxy_auth::ensure_fetch_proxy_auth_on_tab(&tab);
    apply_cdp_emulation_to_tab(tab.as_ref());
    let prev_url = tab.get_url();
    let nav_timeout_secs = crate::config::Config::browser_navigation_timeout_secs();
    tab.set_default_timeout(Duration::from_secs(nav_timeout_secs));
    {
        let (post_nav_net_flight, _post_nav_net_guard) =
            prepare_post_nav_network_idle_tracking(tab.as_ref());
        let redirect_rws_buf = Arc::new(Mutex::new(VecDeque::new()));
        cdp_enable_network_for_redirect_chain_capture(&tab);
        let redirect_rws_weak =
            cdp_attach_redirect_chain_rws_listener(&tab, Arc::clone(&redirect_rws_buf));
        let _redirect_rws_guard = CdpRedirectRwsListenerGuard {
            tab: tab.as_ref(),
            weak: redirect_rws_weak,
        };
        with_lifecycle_event_buffer(&tab, |buf_opt| {
            if let Some(b) = buf_opt {
                if let Ok(mut q) = b.lock() {
                    q.clear();
                }
            }
            if let Ok(mut q) = redirect_rws_buf.lock() {
                q.clear();
            }
            let nav_start = Instant::now();
            tab.navigate_to(url).map_err(|e| {
                let msg = e.to_string();
                let detail = navigate_failed_detail_from_display(&msg);
                log_navigation_cdp_failure(url, &detail);
                navigation_tool_result_for_failed_navigate(url, &detail)
            })?;
            synchronize_tab_after_cdp_navigation(
                &tab,
                prev_url.as_str(),
                url,
                buf_opt,
                nav_start,
                Duration::from_secs(nav_timeout_secs),
                nav_timeout_secs,
                None,
                post_nav_net_flight.as_ref(),
            )
        })?;
        cdp_validate_redirect_chain_from_rws_buffer(&redirect_rws_buf, url)?;
    }
    let final_u = tab.get_url();
    if let Some(msg) = post_navigate_load_failure_message(url, final_u.as_str(), Some(tab.as_ref()))
    {
        return Err(msg);
    }
    assert_final_document_url_ssrf_post_check(final_u.as_str(), Some(url))?;
    mac_stats_info!("browser", "Browser agent: navigated to {}", url);
    Ok(tab)
}

/// Get visible text of the page (after JS has run), including open shadow roots and same-origin iframes.
pub fn get_page_text(tab: &headless_chrome::Tab) -> Result<String, String> {
    let result = tab
        .evaluate(PAGE_TEXT_SHADOW_IFRAME_JS, false)
        .map_err(|e| format!("Evaluate page text: {}", e))?;
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

/// Shared JS: returns ordered visible interactable DOM nodes (must stay in sync with BROWSER_CLICK / BROWSER_INPUT / BROWSER_UPLOAD).
/// Recurses into open shadow roots and same-origin iframes (depth/size limited); cross-origin frames are merged from Rust via CDP.
const INTERACTABLE_NODES_BUILDER: &str = r#"
function __macStatsInteractableNodes() {
  var IFRAME_MAX_DEPTH = 3;
  var MIN_IFRAME_W = 50;
  var MIN_IFRAME_H = 50;
  function visible(el) {
    if (!el || el.nodeType !== 1) return false;
    var r = el.getBoundingClientRect();
    if (r.width < 1 || r.height < 1) return false;
    var st = window.getComputedStyle(el);
    if (st.visibility === 'hidden' || st.display === 'none' || st.opacity === '0') return false;
    return true;
  }
  var seen = new Set();
  var ordered = [];
  function add(el) {
    if (!el || el.nodeType !== 1) return;
    if (seen.has(el)) return;
    if (!visible(el)) return;
    seen.add(el);
    ordered.push(el);
  }
  var sel = [
    'a',
    'button',
    'input',
    'textarea',
    'select',
    '[onclick]',
    '[type="submit"]',
    '[role="button"]',
    '[role="link"]',
    '[role="tab"]',
    '[role="menuitem"]',
    '[role="option"]',
    '[role="switch"]',
    '[role="checkbox"]',
    '[role="radio"]',
    '[role="combobox"]',
    '[role="textbox"]',
    '[role="searchbox"]',
    '[aria-expanded]',
    '[aria-haspopup]',
    '[tabindex]:not([tabindex="-1"])'
  ].join(', ');
  var goodRole = { button: 1, link: 1, tab: 1, menuitem: 1, option: 1, switch: 1, checkbox: 1, radio: 1,
    combobox: 1, textbox: 1, searchbox: 1, treeitem: 1, slider: 1, spinbutton: 1 };

  function collectInteractablesInRoot(root, iframeDepth) {
    if (!root || !root.querySelectorAll) return;
    var base = root.querySelectorAll(sel);
    for (var i = 0; i < base.length; i++) add(base[i]);

    var roleNodes = root.querySelectorAll('[role]');
    for (var j = 0; j < roleNodes.length; j++) {
      var elr = roleNodes[j];
      var rr = elr.getAttribute('role');
      if (rr) {
        rr = String(rr).trim().toLowerCase();
        if (goodRole[rr]) add(elr);
      }
    }

    var scope = root.querySelectorAll('*');
    var maxScan = Math.min(scope.length, 8000);
    var k;
    for (k = 0; k < maxScan; k++) {
      var e = scope[k];
      if (seen.has(e)) continue;
      if (!visible(e)) continue;
      try {
        if (window.getComputedStyle(e).cursor === 'pointer') add(e);
      } catch (err) {}
    }

    for (k = 0; k < scope.length; k++) {
      var node = scope[k];
      if (node.shadowRoot) {
        try {
          collectInteractablesInRoot(node.shadowRoot, iframeDepth);
        } catch (errSr) {}
      }
      if (node.tagName === 'IFRAME' && iframeDepth < IFRAME_MAX_DEPTH) {
        try {
          var rect = node.getBoundingClientRect();
          if (rect.width >= MIN_IFRAME_W && rect.height >= MIN_IFRAME_H) {
            var doc2 = node.contentDocument;
            if (doc2 && doc2.documentElement) {
              collectInteractablesInRoot(doc2, iframeDepth + 1);
            }
          }
        } catch (errIf) {}
      }
    }
  }

  collectInteractablesInRoot(document, 0);
  return ordered;
}
"#;

/// `Runtime.callFunctionOn` with `this` = the interactables node array: returns the same JSON envelope as
/// [`get_interactables_eval_js`] so row metadata matches element `objectId`s resolved from that array.
const INTERACTABLE_NODES_METADATA_BATCH_FN: &str = r#"
function(){
  var nodes = this;
  var out = [];
  for (var i = 0; i < nodes.length; i++) {
    var el = nodes[i];
    var tag = el.tagName.toLowerCase();
    var inputType = el.type ? String(el.type).toLowerCase() : null;
    var text;
    if (tag === 'input' && inputType === 'password') {
      text = (el.placeholder ? el.placeholder : '').trim().slice(0, 200);
    } else {
      text = (el.innerText || el.textContent || el.value || el.placeholder || '').trim().slice(0, 200);
    }
    var href = el.href ? el.href : null;
    var placeholder = el.placeholder ? el.placeholder : null;
    var domName = null;
    try { if (el.name) domName = String(el.name); } catch (e0) {}
    var ariaLabel = null;
    try { ariaLabel = el.getAttribute('aria-label'); } catch (e1) {}
    var ce = !!el.isContentEditable;
    var clsRaw = el.className ? String(el.className) : '';
    var cls = clsRaw.toLowerCase();
    var dpAttr = null;
    try { dpAttr = el.getAttribute('data-provide'); } catch (e2) {}
    var dpa = dpAttr ? String(dpAttr).toLowerCase() : '';
    var datepickerLike = cls.indexOf('datepicker') >= 0 || cls.indexOf('date-picker') >= 0 || dpa.indexOf('datepicker') >= 0;
    var br = el.getBoundingClientRect();
    var annot_bounds = [br.left, br.top, br.width, br.height];
    out.push({ tag_name: tag, text: text, href: href, placeholder: placeholder, input_type: inputType, contenteditable: ce, datepicker_like: datepickerLike, dom_name: domName, aria_label: ariaLabel, annot_bounds: annot_bounds });
  }
  var dpr = (typeof window.devicePixelRatio === 'number' && isFinite(window.devicePixelRatio) && window.devicePixelRatio > 0) ? window.devicePixelRatio : 1;
  return JSON.stringify({ dpr: dpr, nodes: out });
}
"#;

/// Max iframe depth (main = 0) for interactable / text traversal (matches JS `IFRAME_MAX_DEPTH`).
const BROWSER_IFRAME_TRAVERSAL_MAX_DEPTH: u32 = 3;
/// Skip tiny iframes (tracking pixels); matches JS `MIN_IFRAME_W/H`.
const BROWSER_IFRAME_MIN_CSS_PX: f64 = 50.0;
/// Cap extra interactables from cross-origin CDP frame passes (per full snapshot).
const BROWSER_CROSS_ORIGIN_INTERACTABLES_CAP: usize = 250;

static MAC_STATS_ISOLATED_WORLD_SEQ: AtomicU64 = AtomicU64::new(0);

/// Visible text: main document, open shadow roots, and same-origin iframes (depth/size limits match interactables).
const PAGE_TEXT_SHADOW_IFRAME_JS: &str = r#"
(function() {
  var IFRAME_MAX_DEPTH = 3;
  var MIN_IFRAME_W = 50;
  var MIN_IFRAME_H = 50;
  var parts = [];
  function addRootText(root) {
    if (!root) return;
    try {
      if (root.nodeType === 9) {
        var b = root.body || root.documentElement;
        if (b) {
          var t = String(b.innerText || '').trim();
          if (t) parts.push(t);
        }
      } else if (root.nodeType === 11) {
        var ts = String(root.innerText || root.textContent || '').trim();
        if (ts) parts.push(ts);
      }
    } catch (e) {}
  }
  function walk(root, iframeDepth) {
    if (!root || !root.querySelectorAll) return;
    addRootText(root);
    var scope = root.querySelectorAll('*');
    var i;
    for (i = 0; i < scope.length; i++) {
      var el = scope[i];
      if (el.shadowRoot) {
        try { walk(el.shadowRoot, iframeDepth); } catch (e1) {}
      }
      if (el.tagName === 'IFRAME' && iframeDepth < IFRAME_MAX_DEPTH) {
        try {
          var r = el.getBoundingClientRect();
          if (r.width >= MIN_IFRAME_W && r.height >= MIN_IFRAME_H) {
            var d = el.contentDocument;
            if (d && d.documentElement) walk(d, iframeDepth + 1);
          }
        } catch (e2) {}
      }
    }
  }
  walk(document, 0);
  return parts.filter(Boolean).join('\n\n');
})()
"#;

/// DOM → markdown (CDP `Runtime.evaluate`). `__INCLUDE_IMAGES__` replaced with `true` / `false` before eval.
const PAGE_MARKDOWN_EXTRACT_JS: &str = r##"(function() {
  var INCLUDE_IMAGES = __INCLUDE_IMAGES__;
  var IFRAME_MAX_DEPTH = 3;
  var MIN_IFRAME_W = 50;
  var MIN_IFRAME_H = 50;

  function visible(el) {
    if (!el || el.nodeType !== 1) return false;
    try {
      var r = el.getBoundingClientRect();
      if (r.width < 1 || r.height < 1) return false;
      var st = window.getComputedStyle(el);
      if (st.visibility === 'hidden' || st.display === 'none' || st.opacity === '0') return false;
    } catch (e) { return false; }
    return true;
  }

  function countAllElements() {
    var n = 0;
    function walk(node) {
      if (!node) return;
      if (node.nodeType === 1) {
        n++;
        if (node.shadowRoot) {
          try { walk(node.shadowRoot); } catch (e1) {}
        }
        var c = node.firstChild;
        while (c) {
          walk(c);
          c = c.nextSibling;
        }
        if (node.tagName === 'IFRAME') {
          try {
            var rect = node.getBoundingClientRect();
            if (rect.width >= MIN_IFRAME_W && rect.height >= MIN_IFRAME_H) {
              var d = node.contentDocument;
              if (d && d.documentElement) walk(d.documentElement);
            }
          } catch (e2) {}
        }
      } else {
        var ch = node.firstChild;
        while (ch) {
          walk(ch);
          ch = ch.nextSibling;
        }
      }
    }
    if (document.documentElement) walk(document.documentElement);
    return n;
  }

  function textCharCount() {
    try {
      var b = document.body;
      if (!b) return 0;
      return (String(b.innerText || '').replace(/\s+/g, ' ').trim()).length;
    } catch (e) { return 0; }
  }

  function escapeCell(s) {
    return String(s || '').replace(/\|/g, '\\|').replace(/\r?\n/g, ' ').trim();
  }

  function escapeLinkText(s) {
    return String(s || '').replace(/\]/g, '');
  }

  function inlineContent(node, depth) {
    if (!node || depth > 40) return '';
    if (node.nodeType === 3) {
      return String(node.nodeValue || '').replace(/\s+/g, ' ').trim();
    }
    if (node.nodeType !== 1) return '';
    var el = node;
    if (!visible(el)) return '';
    var tag = el.tagName;
    if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'NOSCRIPT' || tag === 'TEMPLATE') return '';
    if (tag === 'BR') return ' ';
    if (tag === 'WBR') return '';
    if (tag === 'A') {
      var href = el.getAttribute('href') || '';
      try { href = new URL(href, document.baseURI).href; } catch (e0) {}
      var inner = '';
      for (var c = el.firstChild; c; c = c.nextSibling) {
        inner += inlineContent(c, depth + 1);
      }
      inner = inner.trim();
      if (!inner) inner = href.slice(0, 200);
      if (!href) return inner;
      return '[' + escapeLinkText(inner).slice(0, 800) + '](' + href + ')';
    }
    if (tag === 'IMG') {
      if (!INCLUDE_IMAGES) return '';
      var src = el.getAttribute('src') || '';
      try { src = new URL(src, document.baseURI).href; } catch (e1) {}
      var alt = el.getAttribute('alt') || '';
      if (!src) return '';
      return '![' + escapeLinkText(alt).slice(0, 200) + '](' + src + ')';
    }
    if (tag === 'STRONG' || tag === 'B') {
      var t = '';
      for (var c2 = el.firstChild; c2; c2 = c2.nextSibling) t += inlineContent(c2, depth + 1);
      var x = t.trim();
      return x ? '**' + x + '**' : '';
    }
    if (tag === 'EM' || tag === 'I') {
      var t2 = '';
      for (var c3 = el.firstChild; c3; c3 = c3.nextSibling) t2 += inlineContent(c3, depth + 1);
      var y = t2.trim();
      return y ? '*' + y + '*' : '';
    }
    if (tag === 'CODE') {
      var t3 = '';
      for (var c4 = el.firstChild; c4; c4 = c4.nextSibling) {
        t3 += c4.nodeType === 3 ? String(c4.nodeValue || '') : inlineContent(c4, depth + 1);
      }
      var z = t3.replace(/`/g, "'").trim();
      return z ? '`' + z.slice(0, 2000) + '`' : '';
    }
    var acc = '';
    for (var c5 = el.firstChild; c5; c5 = c5.nextSibling) {
      acc += inlineContent(c5, depth + 1);
    }
    return acc;
  }

  function emitList(listEl, ordered, iframeDepth) {
    var lines = [];
    var kids = listEl.children;
    for (var i = 0; i < kids.length; i++) {
      if (kids[i].tagName !== 'LI') continue;
      var bullet = ordered ? (lines.length + 1) + '. ' : '- ';
      var inner = '';
      for (var c = kids[i].firstChild; c; c = c.nextSibling) {
        inner += emitBlock(c, iframeDepth);
      }
      inner = inner.replace(/\n\n\n+/g, '\n\n').trim();
      lines.push(bullet + inner.replace(/\n/g, '\n  '));
    }
    return lines.join('\n') + '\n\n';
  }

  function emitTable(tableEl) {
    var trs = tableEl.querySelectorAll('tr');
    var rows = [];
    var maxCols = 0;
    for (var ti = 0; ti < trs.length; ti++) {
      var cells = trs[ti].querySelectorAll('th,td');
      var row = [];
      for (var ci = 0; ci < cells.length; ci++) {
        var txt = inlineContent(cells[ci], 0);
        row.push(escapeCell(txt));
      }
      if (row.length) {
        maxCols = Math.max(maxCols, row.length);
        rows.push(row);
      }
    }
    if (!rows.length) return '';
    var out = '';
    for (var ri = 0; ri < rows.length; ri++) {
      while (rows[ri].length < maxCols) rows[ri].push('');
      out += '| ' + rows[ri].join(' | ') + ' |\n';
      if (ri === 0) {
        var sep = [];
        for (var s = 0; s < maxCols; s++) sep.push('---');
        out += '| ' + sep.join(' | ') + ' |\n';
      }
    }
    return out + '\n';
  }

  function emitBlock(node, iframeDepth) {
    if (!node) return '';
    if (node.nodeType === 3) {
      var tx = String(node.nodeValue || '').replace(/\s+/g, ' ').trim();
      return tx ? tx + '\n\n' : '';
    }
    if (node.nodeType !== 1) return '';
    var el = node;
    if (!visible(el)) return '';
    var tag = el.tagName;
    if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'NOSCRIPT' || tag === 'TEMPLATE') return '';

    var shadowOut = '';
    if (el.shadowRoot) {
      try {
        var sr = el.shadowRoot;
        var c0 = sr.firstChild;
        while (c0) {
          shadowOut += emitBlock(c0, iframeDepth);
          c0 = c0.nextSibling;
        }
      } catch (es) {}
    }

    if (tag === 'IFRAME' && iframeDepth < IFRAME_MAX_DEPTH) {
      try {
        var rect = el.getBoundingClientRect();
        if (rect.width >= MIN_IFRAME_W && rect.height >= MIN_IFRAME_H) {
          var doc2 = el.contentDocument;
          if (doc2 && doc2.body) {
            return shadowOut + emitBlock(doc2.body, iframeDepth + 1);
          }
        }
      } catch (eif) {}
      return shadowOut;
    }

    if (tag === 'UL') return shadowOut + emitList(el, false, iframeDepth);
    if (tag === 'OL') return shadowOut + emitList(el, true, iframeDepth);
    if (tag === 'TABLE') return shadowOut + emitTable(el);

    if (tag === 'PRE') {
      try {
        var preT = String(el.innerText || '').trim();
        return shadowOut + '```\n' + preT.slice(0, 50000) + '\n```\n\n';
      } catch (ep) { return shadowOut; }
    }

    if (tag === 'HR') return shadowOut + '---\n\n';

    if (tag === 'H1' || tag === 'H2' || tag === 'H3' || tag === 'H4' || tag === 'H5' || tag === 'H6') {
      var lev = parseInt(tag.charAt(1), 10) || 1;
      var hashes = '';
      for (var h = 0; h < lev; h++) hashes += '#';
      var ht = inlineContent(el, 0).trim();
      return shadowOut + hashes + ' ' + ht + '\n\n';
    }

    if (tag === 'P') {
      var pt = inlineContent(el, 0).trim();
      return shadowOut + (pt ? pt + '\n\n' : '');
    }

    if (tag === 'BLOCKQUOTE') {
      var bq = '';
      for (var bc = el.firstChild; bc; bc = bc.nextSibling) {
        bq += emitBlock(bc, iframeDepth);
      }
      var lines = bq.split(/\n+/);
      var prefixed = [];
      for (var bi = 0; bi < lines.length; bi++) {
        var L = lines[bi].trim();
        if (L) prefixed.push('> ' + L);
      }
      return shadowOut + (prefixed.length ? prefixed.join('\n') + '\n\n' : '');
    }

    var rest = '';
    for (var fc = el.firstChild; fc; fc = fc.nextSibling) {
      rest += emitBlock(fc, iframeDepth);
    }
    return shadowOut + rest;
  }

  try {
    var body = document.body;
    if (!body) {
      return JSON.stringify({ ok: true, md: '', elements: countAllElements(), text_chars: textCharCount() });
    }
    var md = '';
    var ch = body.firstChild;
    while (ch) {
      md += emitBlock(ch, 0);
      ch = ch.nextSibling;
    }
    md = md.replace(/\n\n\n+/g, '\n\n').trim();
    return JSON.stringify({
      ok: true,
      md: md,
      elements: countAllElements(),
      text_chars: textCharCount()
    });
  } catch (err) {
    var elems = 0;
    var tchars = 0;
    try { elems = countAllElements(); tchars = textCharCount(); } catch (e2) {}
    return JSON.stringify({
      ok: false,
      error: String(err && err.message ? err.message : err),
      md: '',
      elements: elems,
      text_chars: tchars
    });
  }
})()"##;

fn build_page_markdown_extract_js(include_images: bool) -> String {
    PAGE_MARKDOWN_EXTRACT_JS.replace(
        "__INCLUDE_IMAGES__",
        if include_images { "true" } else { "false" },
    )
}

/// Cap markdown at `max_chars`, preferring paragraph/table boundaries (`\n\n` splits).
fn truncate_markdown_at_blocks(markdown: &str, max_chars: usize) -> String {
    let total = markdown.chars().count();
    if total <= max_chars {
        return markdown.to_string();
    }
    let blocks: Vec<&str> = markdown.split("\n\n").collect();
    let mut acc = String::new();
    for block in blocks {
        let block_trim = block.trim_end();
        if block_trim.is_empty() {
            continue;
        }
        let candidate = if acc.is_empty() {
            block_trim.to_string()
        } else {
            format!("{acc}\n\n{block_trim}")
        };
        if candidate.chars().count() > max_chars {
            break;
        }
        acc = candidate;
    }
    if acc.is_empty() {
        acc = markdown.chars().take(max_chars).collect::<String>();
    }
    format!(
        "{acc}\n\n[Truncated: {total} chars total; output capped at ~{} chars near a markdown block boundary.]",
        acc.chars().count()
    )
}

fn default_interactables_js_dpr() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
struct InteractablesJsEnvelope {
    #[serde(default = "default_interactables_js_dpr")]
    #[allow(dead_code)]
    dpr: f64,
    nodes: Vec<InteractableRow>,
}

fn parse_interactables_json_envelope(json_str: &str) -> Result<Vec<InteractableRow>, String> {
    let trimmed = json_str.trim_start();
    if trimmed.starts_with('[') {
        return serde_json::from_str::<Vec<InteractableRow>>(json_str)
            .map_err(|e| format!("Parse interactables JSON (legacy array): {}", e));
    }
    let env: InteractablesJsEnvelope =
        serde_json::from_str(json_str).map_err(|e| format!("Parse interactables JSON: {}", e))?;
    Ok(env.nodes)
}

/// Full `Runtime.evaluate` expression for interactable metadata JSON (built from [`INTERACTABLE_NODES_BUILDER`]).
fn get_interactables_eval_js() -> String {
    let mut s = String::from("(function() { ");
    s.push_str(INTERACTABLE_NODES_BUILDER);
    s.push_str(
        r#"
  var nodes = __macStatsInteractableNodes();
  var out = [];
  for (var i = 0; i < nodes.length; i++) {
    var el = nodes[i];
    var tag = el.tagName.toLowerCase();
    var inputType = el.type ? String(el.type).toLowerCase() : null;
    var text;
    if (tag === 'input' && inputType === 'password') {
      text = (el.placeholder ? el.placeholder : '').trim().slice(0, 200);
    } else {
      text = (el.innerText || el.textContent || el.value || el.placeholder || '').trim().slice(0, 200);
    }
    var href = el.href ? el.href : null;
    var placeholder = el.placeholder ? el.placeholder : null;
    var domName = null;
    try { if (el.name) domName = String(el.name); } catch (e0) {}
    var ariaLabel = null;
    try { ariaLabel = el.getAttribute('aria-label'); } catch (e1) {}
    var ce = !!el.isContentEditable;
    var clsRaw = el.className ? String(el.className) : '';
    var cls = clsRaw.toLowerCase();
    var dpAttr = null;
    try { dpAttr = el.getAttribute('data-provide'); } catch (e2) {}
    var dpa = dpAttr ? String(dpAttr).toLowerCase() : '';
    var datepickerLike = cls.indexOf('datepicker') >= 0 || cls.indexOf('date-picker') >= 0 || dpa.indexOf('datepicker') >= 0;
    var br = el.getBoundingClientRect();
    var annot_bounds = [br.left, br.top, br.width, br.height];
    out.push({ tag_name: tag, text: text, href: href, placeholder: placeholder, input_type: inputType, contenteditable: ce, datepicker_like: datepickerLike, dom_name: domName, aria_label: ariaLabel, annot_bounds: annot_bounds });
  }
  var dpr = (typeof window.devicePixelRatio === 'number' && isFinite(window.devicePixelRatio) && window.devicePixelRatio > 0) ? window.devicePixelRatio : 1;
  return JSON.stringify({ dpr: dpr, nodes: out });
})()
"#,
    );
    s
}

/// Returns JSON `{ element_count, text_length, has_body }` for SPA skeleton / blank detection after navigate.
const SPA_READINESS_JS: &str = r#"
(function() {
  var body = document.body;
  var hasBody = !!body;
  var n = document.querySelectorAll('*').length;
  var t = body ? String(body.innerText || '').trim().length : 0;
  return JSON.stringify({ element_count: n, text_length: t, has_body: hasBody });
})()
"#;

fn spa_readiness_snapshot(tab: &headless_chrome::Tab) -> Result<SpaReadinessSnapshot, String> {
    let result = tab
        .evaluate(SPA_READINESS_JS, false)
        .map_err(|e| format!("SPA readiness evaluate: {}", e))?;
    let json_str = result
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .ok_or_else(|| "SPA readiness JS did not return a string".to_string())?;
    let row: SpaReadinessRow =
        serde_json::from_str(json_str).map_err(|e| format!("Parse SPA readiness JSON: {}", e))?;
    Ok(SpaReadinessSnapshot {
        element_count: row.element_count as usize,
        text_length: row.text_length as usize,
        has_body: row.has_body,
    })
}

/// `Some(reason)` when the page should be treated as blank / skeleton and warrants retry waits.
fn spa_page_appears_blank_reason(snap: &SpaReadinessSnapshot) -> Option<&'static str> {
    if !snap.has_body || snap.element_count == 0 {
        if !snap.has_body {
            Some("no_document_body")
        } else {
            Some("zero_dom_elements")
        }
    } else if snap.element_count > 20 && snap.text_length < snap.element_count.saturating_mul(5) {
        Some("skeleton_sparse_text")
    } else {
        None
    }
}

/// After the baseline post-navigate sleep: optionally wait, reload, and re-check so SPAs can hydrate.
/// Errors only when `document.body` is still missing after the retry cascade (unloadable page).
fn run_spa_blank_page_retry_if_needed(
    tab: &headless_chrome::Tab,
    nav_timeout_secs: u64,
    url_label: &str,
) -> Result<(), String> {
    if !crate::config::Config::browser_spa_retry_enabled() {
        return Ok(());
    }

    let mut snap = spa_readiness_snapshot(tab)?;
    let Some(initial_reason) = spa_page_appears_blank_reason(&snap) else {
        return Ok(());
    };

    crate::debug2!(
        "Browser agent [CDP] SPA readiness: blank page after initial wait url={} phase=initial reason={} element_count={} text_length={} has_body={}",
        url_label,
        initial_reason,
        snap.element_count,
        snap.text_length,
        snap.has_body
    );

    std::thread::sleep(BROWSER_SPA_BLANK_RETRY_FIRST_WAIT);
    snap = spa_readiness_snapshot(tab)?;
    if spa_page_appears_blank_reason(&snap).is_none() {
        crate::debug2!(
            "Browser agent [CDP] SPA readiness: recovered after 3s wait url={} element_count={} text_length={}",
            url_label,
            snap.element_count,
            snap.text_length
        );
        return Ok(());
    }

    let r = spa_page_appears_blank_reason(&snap).unwrap_or("unknown");
    crate::debug2!(
        "Browser agent [CDP] SPA readiness: still blank after 3s url={} reason={} element_count={} text_length={} — Page.reload",
        url_label,
        r,
        snap.element_count,
        snap.text_length
    );

    let (post_nav_net_flight, _post_nav_net_guard) = prepare_post_nav_network_idle_tracking(tab);
    tab.set_default_timeout(Duration::from_secs(nav_timeout_secs));
    tab.reload(false, None)
        .map_err(|e| format!("SPA blank-page retry: reload failed: {}", e))?;
    std::thread::sleep(BROWSER_SPA_BLANK_RETRY_RELOAD_PAUSE);
    match tab.wait_until_navigated() {
        Ok(_) => {}
        Err(e) => {
            let err_str = e.to_string();
            if err_str.to_lowercase().contains("timeout") {
                return Err(format!(
                    "SPA blank-page retry: reload navigation timed out after {}s",
                    nav_timeout_secs
                ));
            }
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: SPA retry reload wait_until_navigated: {} — continuing after delay",
                e
            );
            std::thread::sleep(BROWSER_WAIT_UNTIL_NAVIGATED_FALLBACK);
        }
    }

    apply_configured_post_nav_stabilization(post_nav_net_flight.as_ref(), "spa_blank_reload");
    std::thread::sleep(BROWSER_SPA_BLANK_RETRY_FINAL_WAIT);
    snap = spa_readiness_snapshot(tab)?;
    if let Some(final_reason) = spa_page_appears_blank_reason(&snap) {
        crate::debug2!(
            "Browser agent [CDP] SPA readiness: final check url={} reason={} element_count={} text_length={} has_body={}",
            url_label,
            final_reason,
            snap.element_count,
            snap.text_length,
            snap.has_body
        );
        if !snap.has_body {
            return Err(format!(
                "Page appears unloadable after navigation (no document body after SPA readiness retries). Target URL: {}",
                url_label
            ));
        }
        crate::debug2!(
            "Browser agent [CDP] SPA readiness: accepting page with body despite blank/skeleton heuristic url={}",
            url_label
        );
    }

    Ok(())
}

fn interactables_count_js() -> String {
    let mut s = String::from("(function(){ ");
    s.push_str(INTERACTABLE_NODES_BUILDER);
    s.push_str(" return __macStatsInteractableNodes().length; })()");
    s
}

fn interactable_element_eval_for_index(index: u32) -> String {
    let mut s = String::from("(function(idx){ ");
    s.push_str(INTERACTABLE_NODES_BUILDER);
    s.push_str(" var nodes = __macStatsInteractableNodes(); if (idx < 1 || idx > nodes.length) return null; return nodes[idx-1]; })(");
    s.push_str(&index.to_string());
    s.push(')');
    s
}

fn cdp_runtime_call_function_on(
    tab: &headless_chrome::Tab,
    object_id: &str,
    function_declaration: &str,
    arguments: Vec<serde_json::Value>,
    return_by_value: bool,
    await_promise: bool,
) -> Result<Runtime::RemoteObject, String> {
    let args = if arguments.is_empty() {
        None
    } else {
        Some(
            arguments
                .into_iter()
                .map(|v| Runtime::CallArgument {
                    value: Some(v),
                    unserializable_value: None,
                    object_id: None,
                })
                .collect(),
        )
    };
    let res = tab
        .call_method(Runtime::CallFunctionOn {
            function_declaration: function_declaration.to_string(),
            object_id: Some(object_id.to_string()),
            arguments: args,
            return_by_value: Some(return_by_value),
            generate_preview: Some(false),
            silent: Some(false),
            await_promise: Some(await_promise),
            user_gesture: None,
            execution_context_id: None,
            object_group: None,
            throw_on_side_effect: None,
            serialization_options: None,
            unique_context_id: None,
        })
        .map_err(|e| format!("Runtime.callFunctionOn: {}", e))?;
    if res.exception_details.is_some() {
        return Err("Runtime.callFunctionOn: exception".to_string());
    }
    Ok(res.result)
}

/// Main-document interactables: one `Runtime.evaluate` retains the node array; metadata and each `objectId`
/// are derived from that same handle so indices cannot drift between JSON and CDP on busy pages.
fn collect_main_world_interactable_rows_and_object_ids(
    tab: &headless_chrome::Tab,
) -> Result<(Vec<InteractableRow>, Vec<String>), String> {
    let nodes_expr = format!(
        "(function() {{ {} return __macStatsInteractableNodes(); }})()",
        INTERACTABLE_NODES_BUILDER
    );
    let arr_ro = tab
        .evaluate(&nodes_expr, false)
        .map_err(|e| format!("interactables nodes array evaluate: {}", e))?;
    let array_oid = arr_ro.object_id.ok_or_else(|| {
        "interactables nodes array: DevTools returned no object handle (null or non-object result)"
            .to_string()
    })?;

    let meta_ro = cdp_runtime_call_function_on(
        tab,
        &array_oid,
        INTERACTABLE_NODES_METADATA_BATCH_FN,
        vec![],
        true,
        false,
    )
    .map_err(|e| format!("interactables metadata batch: {}", e))?;
    let meta_str = meta_ro
        .value
        .as_ref()
        .and_then(|v| v.as_str().map(std::string::ToString::to_string))
        .ok_or_else(|| "interactables metadata: expected JSON string value".to_string())?;
    let rows = parse_interactables_json_envelope(&meta_str)?;

    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: interactables main-world snapshot rows={} (atomic node array + batch metadata)",
        rows.len()
    );

    let mut object_ids = Vec::with_capacity(rows.len());
    for i in 0..rows.len() {
        let el_ro = cdp_runtime_call_function_on(
            tab,
            &array_oid,
            "function(i){ return this[Number(i)]; }",
            vec![serde_json::json!(i)],
            false,
            false,
        )
        .map_err(|e| format!("interactable object id [{}]: {}", i + 1, e))?;
        let oid = el_ro.object_id.ok_or_else(|| {
            format!(
                "interactable object id [{}]: DevTools returned no object handle",
                i + 1
            )
        })?;
        object_ids.push(oid);
    }

    Ok((rows, object_ids))
}

fn dom_suggests_datepicker(class_attr: Option<&str>, data_provide: Option<&str>) -> bool {
    let c = class_attr.unwrap_or("").to_lowercase();
    if c.contains("datepicker") || c.contains("date-picker") {
        return true;
    }
    data_provide
        .map(|s| s.to_lowercase().contains("datepicker"))
        .unwrap_or(false)
}

fn interactable_row_from_element(el: &Element<'_>) -> Result<InteractableRow, String> {
    let tag = el.tag_name.to_lowercase();
    let text = el
        .get_inner_text()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(200)
        .collect::<String>();
    let href = if tag == "a" {
        el.call_js_fn(
            "function(){ return this.href ? String(this.href) : null; }",
            vec![],
            true,
        )
        .ok()
        .and_then(|r| r.value)
        .and_then(|v| match v {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) => Some(s),
            _ => None,
        })
    } else {
        None
    };
    let placeholder = el.get_attribute_value("placeholder").ok().flatten();
    let input_type = el.get_attribute_value("type").ok().flatten();
    let dom_name = el.get_attribute_value("name").ok().flatten();
    let aria_label = el.get_attribute_value("aria-label").ok().flatten();
    let class_attr = el.get_attribute_value("class").ok().flatten();
    let data_provide = el.get_attribute_value("data-provide").ok().flatten();
    let datepicker_like = dom_suggests_datepicker(class_attr.as_deref(), data_provide.as_deref());
    let contenteditable = el
        .call_js_fn(
            "function(){ return !!this.isContentEditable; }",
            vec![],
            true,
        )
        .ok()
        .and_then(|r| r.value)
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(InteractableRow {
        tag,
        text,
        href,
        placeholder,
        input_type,
        contenteditable,
        datepicker_like,
        dom_name,
        aria_label,
        bounds_css: None,
        covered: false,
        annot_bounds: None,
        from_subframe: false,
    })
}

fn compact_ax_label(s: &str) -> String {
    let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
    t.chars().take(200).collect()
}

/// Extract label/role text from a CDP `AXValue` object without deserializing into generated structs
/// (Chrome may add fields or enum values that break strict parsing).
fn ax_label_from_ax_json(axv: &JsonValue) -> Option<String> {
    let inner = axv.get("value")?;
    let raw = if let Some(s) = inner.as_str() {
        s.to_string()
    } else {
        inner
            .as_object()
            .and_then(|o| o.get("value"))
            .and_then(|x| x.as_str())
            .map(std::string::ToString::to_string)?
    };
    let t = compact_ax_label(&raw);
    (!t.is_empty()).then_some(t)
}

fn merge_ax_nodes_json_into_map(
    root: &JsonValue,
    out: &mut HashMap<DOM::BackendNodeId, (Option<String>, Option<String>)>,
) {
    let Some(nodes) = root.get("nodes").and_then(|n| n.as_array()) else {
        return;
    };
    for n in nodes {
        let raw_bid = match n.get("backendDOMNodeId") {
            Some(v) => v,
            None => continue,
        };
        let u = raw_bid
            .as_u64()
            .or_else(|| raw_bid.as_i64().map(|i| i as u64));
        let Some(u) = u else {
            continue;
        };
        let Ok(bid) = DOM::BackendNodeId::try_from(u) else {
            continue;
        };
        let name = n.get("name").and_then(ax_label_from_ax_json);
        let role = n.get("role").and_then(ax_label_from_ax_json);
        if name.is_none() && role.is_none() {
            continue;
        }
        let e = out.entry(bid).or_insert((None, None));
        if e.0.is_none() {
            e.0 = name.clone();
        }
        if e.1.is_none() {
            e.1 = role.clone();
        }
    }
}

/// `Accessibility.getFullAXTree` for the focused page (main frame + subframes). Best-effort; returns empty on failure.
fn fetch_merged_ax_backend_map(
    tab: &headless_chrome::Tab,
) -> HashMap<DOM::BackendNodeId, (Option<String>, Option<String>)> {
    let mut map: HashMap<DOM::BackendNodeId, (Option<String>, Option<String>)> = HashMap::new();
    match tab.call_method_json(Accessibility::GetFullAXTree {
        depth: None,
        frame_id: None,
    }) {
        Ok(r) => merge_ax_nodes_json_into_map(&r, &mut map),
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: Accessibility.getFullAXTree (main) failed: {} — continuing without AX names",
                e
            );
            return map;
        }
    }
    let frame_tree = match tab.call_method(Page::GetFrameTree(None)) {
        Ok(r) => r.frame_tree,
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: getFrameTree for AX merge failed: {}",
                e
            );
            return map;
        }
    };
    let mut flat: Vec<(Page::Frame, u32)> = Vec::new();
    walk_frame_tree_depth(&frame_tree, 0, &mut flat);
    for (frame, depth) in flat {
        if depth == 0 {
            continue;
        }
        match tab.call_method_json(Accessibility::GetFullAXTree {
            depth: None,
            frame_id: Some(frame.id.clone()),
        }) {
            Ok(r) => merge_ax_nodes_json_into_map(&r, &mut map),
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: Accessibility.getFullAXTree frame skip: {}",
                    e
                );
            }
        }
    }
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: AX backend map entries={}",
        map.len()
    );
    map
}

pub(crate) fn backend_id_for_object_id(
    tab: &headless_chrome::Tab,
    object_id: &str,
) -> Result<DOM::BackendNodeId, String> {
    let node = tab
        .call_method(DOM::DescribeNode {
            node_id: None,
            backend_node_id: None,
            object_id: Some(object_id.to_string()),
            depth: None,
            pierce: None,
        })
        .map_err(|e| format!("DOM.describeNode: {}", e))?
        .node;
    Ok(node.backend_node_id)
}

fn normalize_identity_token(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn interactable_identity_tokens(i: &Interactable) -> HashSet<String> {
    let mut set = HashSet::new();
    for src in [
        i.accessible_name.as_deref(),
        Some(i.text.as_str()),
        i.placeholder.as_deref(),
        i.href.as_deref(),
        i.dom_name.as_deref(),
        i.aria_label.as_deref(),
    ] {
        let Some(t) = src else { continue };
        let n = normalize_identity_token(t);
        if !n.is_empty() {
            set.insert(n);
        }
    }
    set
}

fn interactables_identity_match(expected: &Interactable, candidate: &Interactable) -> bool {
    if expected.tag.to_lowercase() != candidate.tag.to_lowercase() {
        return false;
    }
    match (&expected.accessible_name, &candidate.accessible_name) {
        (Some(a), Some(b)) if !a.is_empty() && !b.is_empty() => {
            if normalize_identity_token(a) != normalize_identity_token(b) {
                return false;
            }
        }
        _ => {}
    }
    let e = interactable_identity_tokens(expected);
    let c = interactable_identity_tokens(candidate);
    !e.is_disjoint(&c)
}

/// After a DOM reorder, find the unique new 1-based index for `expected` (from the last snapshot). Used by BROWSER_CLICK / BROWSER_INPUT remapping.
fn find_unique_identity_match(
    expected: &Interactable,
    fresh: &[Interactable],
) -> Result<u32, String> {
    let matches: Vec<&Interactable> = fresh
        .iter()
        .filter(|c| interactables_identity_match(expected, c))
        .collect();
    match matches.len() {
        0 => Err(format!(
            "no element matched tag=<{}> with overlapping accessible name / text / attributes (expected hints: {:?})",
            expected.tag,
            interactable_identity_tokens(expected)
                .into_iter()
                .take(6)
                .collect::<Vec<_>>()
        )),
        1 => Ok(matches[0].index),
        _ => {
            let cand = matches
                .iter()
                .map(|m| {
                    format!(
                        "[{}] tag={} ax={:?} text={:?}",
                        m.index,
                        m.tag,
                        m.accessible_name.as_deref().unwrap_or(""),
                        m.text.chars().take(40).collect::<String>()
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            Err(format!("ambiguous identity match ({} candidates: {})", matches.len(), cand))
        }
    }
}

fn runtime_evaluate_in_context(
    tab: &headless_chrome::Tab,
    context_id: Runtime::ExecutionContextId,
    expression: &str,
    return_by_value: bool,
    await_promise: bool,
) -> Result<Runtime::RemoteObject, String> {
    let ret = tab
        .call_method(Runtime::Evaluate {
            expression: expression.to_string(),
            return_by_value: Some(return_by_value),
            generate_preview: Some(false),
            silent: Some(true),
            await_promise: Some(await_promise),
            include_command_line_api: Some(false),
            user_gesture: Some(false),
            object_group: None,
            context_id: Some(context_id),
            throw_on_side_effect: None,
            timeout: None,
            disable_breaks: None,
            repl_mode: None,
            allow_unsafe_eval_blocked_by_csp: None,
            unique_context_id: None,
            serialization_options: None,
        })
        .map_err(|e| format!("Runtime.evaluate (frame context): {}", e))?;
    if ret.exception_details.is_some() {
        return Err("Runtime.evaluate (frame context): exception".to_string());
    }
    Ok(ret.result)
}

fn tuple_security_origin_equals(a: &str, b: &str) -> bool {
    let ua = match Url::parse(a) {
        Ok(u) => u,
        Err(_) => return false,
    };
    let ub = match Url::parse(b) {
        Ok(u) => u,
        Err(_) => return false,
    };
    if !ua.origin().is_tuple() || !ub.origin().is_tuple() {
        return false;
    }
    ua.origin() == ub.origin()
}

/// `about:blank` / `about:srcdoc` inherit parent origin; parent pass (main JS or prior frame eval) already covered them.
fn should_skip_cdp_frame_collect(child_url: &str, parent_url: &str) -> bool {
    let c = child_url.trim();
    if c == "about:blank" || c.starts_with("about:srcdoc") {
        return true;
    }
    tuple_security_origin_equals(child_url, parent_url)
}

fn index_frame_tree(ft: &Page::FrameTree, m: &mut HashMap<String, Page::Frame>) {
    m.insert(ft.frame.id.clone(), ft.frame.clone());
    if let Some(ref kids) = ft.child_frames {
        for c in kids {
            index_frame_tree(c, m);
        }
    }
}

fn walk_frame_tree_depth(ft: &Page::FrameTree, depth: u32, acc: &mut Vec<(Page::Frame, u32)>) {
    acc.push((ft.frame.clone(), depth));
    if let Some(ref kids) = ft.child_frames {
        for c in kids {
            walk_frame_tree_depth(c, depth + 1, acc);
        }
    }
}

/// Cross-origin (or otherwise JS-opaque) subframes: `Page.getFrameTree` + `createIsolatedWorld` + same interactable JS as the main world.
fn collect_cross_origin_frame_interactables(
    tab: &headless_chrome::Tab,
) -> Result<Vec<(InteractableRow, String)>, String> {
    let frame_tree = tab
        .call_method(Page::GetFrameTree(None))
        .map_err(|e| format!("getFrameTree: {}", e))?
        .frame_tree;

    let mut by_id: HashMap<String, Page::Frame> = HashMap::new();
    index_frame_tree(&frame_tree, &mut by_id);

    let mut flat: Vec<(Page::Frame, u32)> = Vec::new();
    walk_frame_tree_depth(&frame_tree, 0, &mut flat);

    let eval_interactables = get_interactables_eval_js();
    let mut out: Vec<(InteractableRow, String)> = Vec::new();

    for (frame, depth) in flat {
        if depth == 0 || depth > BROWSER_IFRAME_TRAVERSAL_MAX_DEPTH {
            continue;
        }
        let Some(ref pid) = frame.parent_id else {
            continue;
        };
        let Some(parent_fr) = by_id.get(pid) else {
            continue;
        };
        if should_skip_cdp_frame_collect(&frame.url, &parent_fr.url) {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: skip cross-origin frame scan (same inherited/origin as parent) depth={} child_url={}",
                depth,
                frame.url.chars().take(96).collect::<String>()
            );
            continue;
        }

        let world_n = MAC_STATS_ISOLATED_WORLD_SEQ.fetch_add(1, Ordering::Relaxed);
        let ctx = match tab.call_method(Page::CreateIsolatedWorld {
            frame_id: frame.id.clone(),
            world_name: Some(format!("__macStatsIw{}", world_n)),
            grant_univeral_access: Some(true),
        }) {
            Ok(r) => r.execution_context_id,
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: CreateIsolatedWorld failed (skip frame): {}",
                    e
                );
                continue;
            }
        };

        let size_json = match runtime_evaluate_in_context(
            tab,
            ctx,
            r#"(function(){try{var w=Math.max(document.documentElement.clientWidth||0,window.innerWidth||0);var h=Math.max(document.documentElement.clientHeight||0,window.innerHeight||0);return JSON.stringify({w:w,h:h});}catch(e){return "{\"w\":0,\"h\":0}";}})()"#,
            true,
            false,
        ) {
            Ok(ro) => ro
                .value
                .and_then(|v| v.as_str().map(std::string::ToString::to_string))
                .unwrap_or_else(|| "{\"w\":0,\"h\":0}".to_string()),
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: frame size evaluate failed (skip): {}",
                    e
                );
                continue;
            }
        };
        let size_v: serde_json::Value =
            serde_json::from_str(&size_json).unwrap_or(serde_json::json!({}));
        let fw = size_v["w"].as_f64().unwrap_or(0.0);
        let fh = size_v["h"].as_f64().unwrap_or(0.0);
        if fw < BROWSER_IFRAME_MIN_CSS_PX || fh < BROWSER_IFRAME_MIN_CSS_PX {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: skip cross-origin frame ({}x{} < {}px) id={}",
                fw,
                fh,
                BROWSER_IFRAME_MIN_CSS_PX as i32,
                frame.id.chars().take(12).collect::<String>()
            );
            continue;
        }

        let json_ro = match runtime_evaluate_in_context(tab, ctx, &eval_interactables, true, false)
        {
            Ok(ro) => ro,
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: frame interactables evaluate failed: {}",
                    e
                );
                continue;
            }
        };
        let json_str = match json_ro.value.as_ref().and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: frame interactables: missing JSON string"
                );
                continue;
            }
        };
        let mut frame_rows: Vec<InteractableRow> = match parse_interactables_json_envelope(json_str)
        {
            Ok(v) => v,
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: frame interactables JSON parse: {}",
                    e
                );
                continue;
            }
        };
        if frame_rows.is_empty() {
            continue;
        }
        let room = BROWSER_CROSS_ORIGIN_INTERACTABLES_CAP.saturating_sub(out.len());
        if room == 0 {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: cross-origin frame interactables cap {} reached",
                BROWSER_CROSS_ORIGIN_INTERACTABLES_CAP
            );
            break;
        }
        if frame_rows.len() > room {
            frame_rows.truncate(room);
        }
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: cross-origin frame +{} interactables depth={}",
            frame_rows.len(),
            depth
        );
        let start_len = out.len();
        for i in 1u32..=frame_rows.len() as u32 {
            let expr = interactable_element_eval_for_index(i);
            let el_ro = match runtime_evaluate_in_context(tab, ctx, &expr, false, false) {
                Ok(ro) => ro,
                Err(e) => {
                    mac_stats_debug!(
                        "browser/cdp",
                        "Browser agent [CDP]: frame interactable object id [{}]: {}",
                        i,
                        e
                    );
                    out.truncate(start_len);
                    break;
                }
            };
            let Some(oid) = el_ro.object_id else {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: frame interactable object id [{}]: no handle",
                    i
                );
                out.truncate(start_len);
                break;
            };
            let mut row = frame_rows[(i - 1) as usize].clone();
            row.from_subframe = true;
            out.push((row, oid));
        }
    }

    Ok(out)
}

fn augment_interactables_with_cdp_listeners(
    tab: &headless_chrome::Tab,
    rows: &mut Vec<InteractableRow>,
    object_ids: &mut Vec<String>,
) -> Result<(), String> {
    let root = tab
        .call_method(DOM::GetDocument {
            depth: Some(0),
            pierce: Some(true),
        })
        .map_err(|e| format!("getDocument: {}", e))?
        .root
        .node_id;
    let node_ids: Vec<_> = tab
        .call_method(DOM::QuerySelectorAll {
            node_id: root,
            selector: "div, span, label, li, p, section, article, header, nav, td, th".to_string(),
        })
        .map_err(|e| format!("listener probe QuerySelectorAll: {}", e))?
        .node_ids
        .into_iter()
        .take(220)
        .collect();

    let mut added: usize = 0;
    const CAP_ADD: usize = 48;

    for nid in node_ids {
        if added >= CAP_ADD {
            break;
        }
        if nid == 0 {
            continue;
        }
        let el = match Element::new(tab, nid) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let tag = el.tag_name.to_lowercase();
        if matches!(
            tag.as_str(),
            "button" | "a" | "input" | "textarea" | "select"
        ) {
            continue;
        }
        if !el.is_visible().unwrap_or(false) {
            continue;
        }
        let listeners = match tab.call_method(DOMDebugger::GetEventListeners {
            object_id: el.remote_object_id.clone(),
            depth: None,
            pierce: Some(true),
        }) {
            Ok(l) => l.listeners,
            Err(_) => continue,
        };
        let hit = listeners.iter().any(|listener| {
            let t = listener.Type.to_lowercase();
            t == "click" || t == "mousedown" || t == "pointerdown"
        });
        if !hit {
            continue;
        }
        let row = interactable_row_from_element(&el).map_err(|e| e.to_string())?;
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: interactables +1 via DOMDebugger.getEventListeners (tag={})",
            row.tag
        );
        object_ids.push(el.remote_object_id.clone());
        rows.push(row);
        added += 1;
    }
    Ok(())
}

fn cdp_scroll_metrics(tab: &headless_chrome::Tab) -> Result<(f64, f64, f64, f64), String> {
    let remote = tab
        .evaluate(
            "JSON.stringify({sx:window.scrollX,sy:window.scrollY,vw:window.innerWidth,vh:window.innerHeight})",
            false,
        )
        .map_err(|e| e.to_string())?;
    let s = remote
        .value
        .and_then(|v| v.as_str().map(std::string::ToString::to_string))
        .ok_or_else(|| "viewport metrics: no JSON string".to_string())?;
    let v: serde_json::Value =
        serde_json::from_str(&s).map_err(|e| format!("viewport metrics parse: {}", e))?;
    Ok((
        v["sx"].as_f64().unwrap_or(0.0),
        v["sy"].as_f64().unwrap_or(0.0),
        v["vw"].as_f64().unwrap_or(1280.0),
        v["vh"].as_f64().unwrap_or(720.0),
    ))
}

fn quad_doc_center(q: &[f64]) -> Option<(f64, f64)> {
    if q.len() < 8 {
        return None;
    }
    Some((
        (q[0] + q[2] + q[4] + q[6]) / 4.0,
        (q[1] + q[3] + q[5] + q[7]) / 4.0,
    ))
}

fn quad_area(q: &[f64]) -> f64 {
    if q.len() < 8 {
        return 0.0;
    }
    let mut a = 0.0;
    for k in 0..4 {
        let i = k * 2;
        let j = ((k + 1) % 4) * 2;
        a += q[i] * q[j + 1] - q[i + 1] * q[j];
    }
    0.5 * a.abs()
}

fn pick_viewport_click_point_from_content_quads(
    quads: &[Vec<f64>],
    sx: f64,
    sy: f64,
    vw: f64,
    vh: f64,
) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64, f64)> = None;
    for q in quads {
        let (cx, cy) = quad_doc_center(q)?;
        let vx = cx - sx;
        let vy = cy - sy;
        let area = quad_area(q);
        if area <= 1.0 {
            continue;
        }
        if vx >= -2.0 && vy >= -2.0 && vx <= vw + 2.0 && vy <= vh + 2.0 {
            let replace = best
                .as_ref()
                .map(|(best_area, _, _)| area > *best_area)
                .unwrap_or(true);
            if replace {
                best = Some((area, vx, vy));
            }
        }
    }
    if let Some((_, vx, vy)) = best {
        return Some((vx, vy));
    }
    let mut best2: Option<(f64, f64, f64)> = None;
    for q in quads {
        let (cx, cy) = quad_doc_center(q)?;
        let vx = (cx - sx).clamp(0.0, vw);
        let vy = (cy - sy).clamp(0.0, vh);
        let area = quad_area(q);
        if area <= 1.0 {
            continue;
        }
        let replace = best2
            .as_ref()
            .map(|(best_area, _, _)| area > *best_area)
            .unwrap_or(true);
        if replace {
            best2 = Some((area, vx, vy));
        }
    }
    best2.map(|(_, vx, vy)| (vx, vy))
}

fn viewport_midpoint_via_js(
    tab: &headless_chrome::Tab,
    object_id: &str,
) -> Result<(f64, f64), String> {
    let res = tab
        .call_method(Runtime::CallFunctionOn {
            function_declaration: "function(){ var r=this.getBoundingClientRect(); return {x:r.left+r.width*0.5,y:r.top+r.height*0.5,w:r.width,h:r.height}; }".to_string(),
            object_id: Some(object_id.to_string()),
            arguments: None,
            return_by_value: Some(true),
            await_promise: Some(false),
            user_gesture: None,
            silent: Some(false),
            generate_preview: Some(false),
            execution_context_id: None,
            object_group: None,
            throw_on_side_effect: None,
            serialization_options: None,
            unique_context_id: None,
        })
        .map_err(|e| e.to_string())?;
    if res.exception_details.is_some() {
        return Err("getBoundingClientRect: exception".to_string());
    }
    let v = res
        .result
        .value
        .ok_or_else(|| "getBoundingClientRect: no value".to_string())?;
    let x = v["x"].as_f64().ok_or_else(|| "bbox x".to_string())?;
    let y = v["y"].as_f64().ok_or_else(|| "bbox y".to_string())?;
    let w = v["w"].as_f64().unwrap_or(0.0);
    let h = v["h"].as_f64().unwrap_or(0.0);
    if w < 1.0 || h < 1.0 {
        return Err("zero-size bounding box".to_string());
    }
    Ok((x, y))
}

fn cdp_element_from_point_covers_target(
    tab: &headless_chrome::Tab,
    object_id: &str,
    x: f64,
    y: f64,
) -> Result<bool, String> {
    let res = tab
        .call_method(Runtime::CallFunctionOn {
            function_declaration: "function(px,py){ var t=document.elementFromPoint(px,py); return !!(t && (t===this || this.contains(t))); }".to_string(),
            object_id: Some(object_id.to_string()),
            arguments: Some(vec![
                Runtime::CallArgument {
                    value: Some(serde_json::json!(x)),
                    unserializable_value: None,
                    object_id: None,
                },
                Runtime::CallArgument {
                    value: Some(serde_json::json!(y)),
                    unserializable_value: None,
                    object_id: None,
                },
            ]),
            return_by_value: Some(true),
            await_promise: Some(false),
            user_gesture: None,
            silent: Some(false),
            generate_preview: Some(false),
            execution_context_id: None,
            object_group: None,
            throw_on_side_effect: None,
            serialization_options: None,
            unique_context_id: None,
        })
        .map_err(|e| e.to_string())?;
    if res.exception_details.is_some() {
        return Ok(false);
    }
    Ok(res.result.value.and_then(|v| v.as_bool()).unwrap_or(false))
}

fn cdp_js_click_element(tab: &headless_chrome::Tab, object_id: &str) -> Result<(), String> {
    let res = tab
        .call_method(Runtime::CallFunctionOn {
            function_declaration: "function(){ this.click(); return 'clicked'; }".to_string(),
            object_id: Some(object_id.to_string()),
            arguments: None,
            return_by_value: Some(true),
            await_promise: Some(false),
            user_gesture: Some(true),
            silent: Some(false),
            generate_preview: Some(false),
            execution_context_id: None,
            object_group: None,
            throw_on_side_effect: None,
            serialization_options: None,
            unique_context_id: None,
        })
        .map_err(|e| e.to_string())?;
    if res.exception_details.is_some() {
        return Err("JS click: exceptionDetails".to_string());
    }
    let ok = res.result.value.as_ref().and_then(|v| v.as_str()) == Some("clicked");
    if !ok {
        return Err(format!(
            "JS click: unexpected result {:?}",
            res.result.value
        ));
    }
    Ok(())
}

fn cdp_scroll_into_view_if_needed_warn(tab: &headless_chrome::Tab, object_id: &str, context: &str) {
    if let Err(e) = tab.call_method(DOM::ScrollIntoViewIfNeeded {
        node_id: None,
        backend_node_id: None,
        object_id: Some(object_id.to_string()),
        rect: None,
    }) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: DOM.scrollIntoViewIfNeeded ({}): {} — continuing",
            context,
            e
        );
    }
}

/// Viewport CSS coordinates for pointer actions (same geometry as BROWSER_CLICK).
/// When `scroll_first` is true, runs `DOM.scrollIntoViewIfNeeded` for the node first.
fn cdp_viewport_point_for_object_id(
    tab: &headless_chrome::Tab,
    object_id: &str,
    scroll_first: bool,
) -> Result<(f64, f64), String> {
    if scroll_first {
        cdp_scroll_into_view_if_needed_warn(tab, object_id, "pointer geometry");
    }

    let (sx, sy, vw, vh) = cdp_scroll_metrics(tab)?;

    let (vx, vy) = match tab.call_method(DOM::GetContentQuads {
        node_id: None,
        backend_node_id: None,
        object_id: Some(object_id.to_string()),
    }) {
        Ok(o) if !o.quads.is_empty() => {
            let quads: Vec<Vec<f64>> = o
                .quads
                .iter()
                .map(|q| q.iter().map(|x| *x as f64).collect())
                .collect();
            pick_viewport_click_point_from_content_quads(&quads, sx, sy, vw, vh)
                .or_else(|| viewport_midpoint_via_js(tab, object_id).ok())
        }
        _ => viewport_midpoint_via_js(tab, object_id).ok(),
    }
    .ok_or_else(|| "no click point from geometry".to_string())?;
    Ok((vx, vy))
}

fn cdp_click_by_object_id(tab: &headless_chrome::Tab, object_id: &str) -> Result<(), String> {
    let (vx, vy) = cdp_viewport_point_for_object_id(tab, object_id, true)?;

    let use_coords = match cdp_element_from_point_covers_target(tab, object_id, vx, vy) {
        Ok(true) => true,
        Ok(false) => {
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: BROWSER_CLICK target occluded at ({:.1},{:.1}) — using JS element.click()",
                vx,
                vy
            );
            false
        }
        Err(_) => false,
    };

    if use_coords {
        match tab.click_point(Point { x: vx, y: vy }) {
            Ok(_) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: BROWSER_CLICK dispatched Input.dispatchMouseEvent at ({:.1},{:.1})",
                    vx,
                    vy
                );
                return Ok(());
            }
            Err(e) => {
                mac_stats_warn!(
                    "browser/cdp",
                    "Browser agent [CDP]: coordinate click failed: {} — JS element.click()",
                    e
                );
            }
        }
    }

    cdp_js_click_element(tab, object_id)?;
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_CLICK used JS element.click() fallback"
    );
    Ok(())
}

fn cdp_hover_by_object_id(tab: &headless_chrome::Tab, object_id: &str) -> Result<(), String> {
    let (vx, vy) = cdp_viewport_point_for_object_id(tab, object_id, true)?;
    if !cdp_element_from_point_covers_target(tab, object_id, vx, vy).unwrap_or(false) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: BROWSER_HOVER target may be occluded at ({:.1},{:.1}) — moving pointer anyway",
            vx,
            vy
        );
    }
    tab.move_mouse_to_point(Point { x: vx, y: vy })
        .map_err(|e| format!("hover move: {}", e))?;
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_HOVER dispatched Input.dispatchMouseEvent at ({:.1},{:.1})",
        vx,
        vy
    );
    Ok(())
}

fn cdp_dispatch_mouse_pressed(tab: &headless_chrome::Tab, x: f64, y: f64) -> Result<(), String> {
    tab.call_method(Input::DispatchMouseEvent {
        Type: Input::DispatchMouseEventTypeOption::MousePressed,
        x,
        y,
        button: Some(Input::MouseButton::Left),
        click_count: Some(1),
        modifiers: None,
        timestamp: None,
        buttons: None,
        force: None,
        tangential_pressure: None,
        tilt_x: None,
        tilt_y: None,
        twist: None,
        delta_x: None,
        delta_y: None,
        pointer_Type: None,
    })
    .map_err(|e| format!("mousePressed: {}", e))?;
    Ok(())
}

fn cdp_dispatch_mouse_released(tab: &headless_chrome::Tab, x: f64, y: f64) -> Result<(), String> {
    tab.call_method(Input::DispatchMouseEvent {
        Type: Input::DispatchMouseEventTypeOption::MouseReleased,
        x,
        y,
        button: Some(Input::MouseButton::Left),
        click_count: Some(1),
        modifiers: None,
        timestamp: None,
        buttons: None,
        force: None,
        tangential_pressure: None,
        tilt_x: None,
        tilt_y: None,
        twist: None,
        delta_x: None,
        delta_y: None,
        pointer_Type: None,
    })
    .map_err(|e| format!("mouseReleased: {}", e))?;
    Ok(())
}

fn cdp_dispatch_mouse_moved(tab: &headless_chrome::Tab, x: f64, y: f64) -> Result<(), String> {
    tab.call_method(Input::DispatchMouseEvent {
        Type: Input::DispatchMouseEventTypeOption::MouseMoved,
        x,
        y,
        modifiers: None,
        timestamp: None,
        button: None,
        buttons: None,
        click_count: None,
        force: None,
        tangential_pressure: None,
        tilt_x: None,
        tilt_y: None,
        twist: None,
        delta_x: None,
        delta_y: None,
        pointer_Type: None,
    })
    .map_err(|e| format!("mouseMoved: {}", e))?;
    Ok(())
}

/// Drag left button from `from_id` center to `to_id` center: scroll both into view, then
/// move → press → interpolated moves → release at end.
fn cdp_drag_between_object_ids(
    tab: &headless_chrome::Tab,
    from_id: &str,
    to_id: &str,
) -> Result<(), String> {
    cdp_scroll_into_view_if_needed_warn(tab, from_id, "BROWSER_DRAG start");
    cdp_scroll_into_view_if_needed_warn(tab, to_id, "BROWSER_DRAG end");
    let (fx, fy) = cdp_viewport_point_for_object_id(tab, from_id, false)?;
    let (tx, ty) = cdp_viewport_point_for_object_id(tab, to_id, false)?;
    tab.move_mouse_to_point(Point { x: fx, y: fy })
        .map_err(|e| format!("drag move to start: {}", e))?;
    cdp_dispatch_mouse_pressed(tab, fx, fy)?;
    let n = BROWSER_DRAG_MOVE_STEPS.max(1);
    for s in 1..=n {
        let t = s as f64 / n as f64;
        let x = fx + (tx - fx) * t;
        let y = fy + (ty - fy) * t;
        cdp_dispatch_mouse_moved(tab, x, y)?;
    }
    cdp_dispatch_mouse_released(tab, tx, ty)?;
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_DRAG from ({:.1},{:.1}) to ({:.1},{:.1}) steps={}",
        fx,
        fy,
        tx,
        ty,
        n
    );
    Ok(())
}

fn resolve_interactable_object_id(
    tab: &headless_chrome::Tab,
    index: u32,
) -> Result<String, String> {
    if let Ok(g) = last_interactable_object_ids().lock() {
        if let Some(id) = g.get(index as usize - 1) {
            return Ok(id.clone());
        }
    }
    let count_js = interactables_count_js();
    let n_ro = tab
        .evaluate(&count_js, false)
        .map_err(|e| format!("interactables count: {}", e))?;
    let n = n_ro
        .value
        .as_ref()
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "interactables count: not a number".to_string())? as u32;
    if index < 1 || index > n {
        return Err(format!("index out of range (max {})", n));
    }
    let expr = interactable_element_eval_for_index(index);
    let el_ro = tab
        .evaluate(&expr, false)
        .map_err(|e| format!("resolve interactable element: {}", e))?;
    el_ro
        .object_id
        .ok_or_else(|| "element handle missing (stale index or detached node)".to_string())
}

fn resolve_interactable_for_action(
    tab: &headless_chrome::Tab,
    index: u32,
    tool: &str,
) -> Result<String, String> {
    let expected_entry = last_interactables_snapshot().lock().ok().and_then(|g| {
        g.as_ref()
            .and_then(|v| v.get(index.saturating_sub(1) as usize).cloned())
    });

    match resolve_interactable_object_id(tab, index) {
        Ok(oid) => {
            if let Some(ref exp) = expected_entry {
                if let Some(stored) = exp.backend_dom_node_id {
                    match backend_id_for_object_id(tab, &oid) {
                        Ok(cur) if cur == stored => return Ok(oid),
                        Ok(_) => {
                            mac_stats_info!(
                                "browser/cdp",
                                "Browser agent [CDP]: {} index={} stale (backendNodeId drift); remapping by identity",
                                tool,
                                index
                            );
                            return remap_interactable_action(tab, tool, index, exp);
                        }
                        Err(e) => {
                            mac_stats_debug!(
                                "browser/cdp",
                                "Browser agent [CDP]: {} describeNode for stale-check failed: {} — using resolved object",
                                tool,
                                e
                            );
                            return Ok(oid);
                        }
                    }
                } else {
                    return Ok(oid);
                }
            } else {
                Ok(oid)
            }
        }
        Err(e) => {
            if let Some(ref exp) = expected_entry {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: {} index={} resolve failed ({}); trying identity remap",
                    tool,
                    index,
                    e
                );
                remap_interactable_action(tab, tool, index, exp)
            } else {
                Err(format!("{}: {}", tool, e))
            }
        }
    }
}

fn remap_interactable_action(
    tab: &headless_chrome::Tab,
    tool: &str,
    requested_index: u32,
    expected: &Interactable,
) -> Result<String, String> {
    let fresh = get_interactables(tab)?;
    let new_idx = find_unique_identity_match(expected, fresh.as_slice()).map_err(|msg| {
        format!(
            "{}: element [{}] is no longer valid — {}",
            tool, requested_index, msg
        )
    })?;
    if new_idx != requested_index {
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: {} identity remapped {} -> {}",
            tool,
            requested_index,
            new_idx
        );
    }
    resolve_interactable_object_id(tab, new_idx).map_err(|e| format!("{}: {}", tool, e))
}

/// Get visible interactive elements from the page via JS. Returns 1-based indices. Used for BROWSER_NAVIGATE state.
pub fn get_interactables(tab: &headless_chrome::Tab) -> Result<Vec<Interactable>, String> {
    let (mut rows, mut object_ids) = collect_main_world_interactable_rows_and_object_ids(tab)?;

    let extra_rows = collect_cross_origin_frame_interactables(tab).unwrap_or_else(|e| {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: cross-origin frame interactables not merged: {}",
            e
        );
        Vec::new()
    });
    if !extra_rows.is_empty() {
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: interactables +{} from cross-origin iframes (CDP isolated world)",
            extra_rows.len()
        );
        for (row, oid) in extra_rows {
            rows.push(row);
            object_ids.push(oid);
        }
    }

    augment_interactables_with_cdp_listeners(tab, &mut rows, &mut object_ids)?;
    dom_snapshot::apply_snapshot_paint_filter(tab, &mut rows, &mut object_ids);
    set_interactable_object_ids(object_ids.clone());

    let ax_map = fetch_merged_ax_backend_map(tab);
    let mut interactables: Vec<Interactable> = Vec::with_capacity(rows.len());
    for (i, (row, oid)) in rows.into_iter().zip(object_ids.into_iter()).enumerate() {
        let backend = backend_id_for_object_id(tab, &oid).ok();
        let (ax_name, ax_role) = backend
            .and_then(|b| ax_map.get(&b).cloned())
            .unwrap_or((None, None));
        interactables.push(Interactable {
            index: (i + 1) as u32,
            tag: row.tag,
            text: row.text,
            href: row.href,
            placeholder: row.placeholder,
            input_type: row.input_type,
            contenteditable: row.contenteditable,
            datepicker_like: row.datepicker_like,
            accessible_name: ax_name,
            ax_role,
            backend_dom_node_id: backend,
            dom_name: row.dom_name,
            aria_label: row.aria_label,
            bounds_css: row.bounds_css,
            annot_bounds_css: row.annot_bounds.map(|[x, y, w, h]| (x, y, w, h)),
            from_subframe: row.from_subframe,
            covered: row.covered,
        });
    }
    Ok(interactables)
}

/// Try to find and click a cookie-reject / only-necessary button on the current page. Called automatically after BROWSER_NAVIGATE.
/// Patterns are loaded from ~/.mac-stats/agents/cookie_reject_patterns.md (user-editable for translation or extra sites).
/// Returns true if a matching element was clicked, false otherwise. Logs at INFO (and DEBUG for details) so it is visible in logs and verbose mode.
fn try_dismiss_cookie_banner(tab: &headless_chrome::Tab) -> Result<bool, String> {
    if is_chrome_internal_error_document_url(tab.get_url().as_str()) {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: skip cookie banner on chrome-error document"
        );
        return Ok(false);
    }
    let patterns = crate::config::Config::load_cookie_reject_patterns();
    let interactables = get_interactables(tab)?;
    let label_lower = |i: &Interactable| -> String {
        let s = if !i.text.is_empty() {
            i.text.as_str()
        } else if let Some(ref p) = i.placeholder {
            p.as_str()
        } else {
            ""
        };
        s.to_lowercase()
    };
    for i in &interactables {
        let lower = label_lower(i);
        let lower_trim = lower.trim();
        if lower_trim.is_empty() {
            continue;
        }
        let matched = patterns.iter().any(|pat| {
            let p = pat.trim().to_lowercase();
            !p.is_empty() && (lower_trim.contains(&p) || lower_trim == p)
        });
        if matched {
            mac_stats_info!("browser/cdp",
                "Browser agent [CDP]: attempting to dismiss cookie banner (clicking element [{}] '{}')",
                i.index,
                i.text.chars().take(50).collect::<String>()
            );
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: cookie banner — matched pattern, index {} text {:?}",
                i.index,
                i.text
            );
            let oid = match resolve_interactable_object_id(tab, i.index) {
                Ok(id) => id,
                Err(e) => {
                    mac_stats_warn!(
                        "browser/cdp",
                        "Browser agent [CDP]: cookie banner resolve index {}: {}",
                        i.index,
                        e
                    );
                    return Ok(false);
                }
            };
            if let Err(e) = cdp_click_by_object_id(tab, &oid) {
                mac_stats_warn!(
                    "browser/cdp",
                    "Browser agent [CDP]: cookie banner click failed: {}",
                    e
                );
                return Ok(false);
            }
            std::thread::sleep(BROWSER_COOKIE_BANNER_CLICK_SETTLE);
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: cookie banner dismissed (clicked element [{}])",
                i.index
            );
            return Ok(true);
        }
    }
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: cookie banner — no consent control found (checked {} elements)",
        interactables.len()
    );
    Ok(false)
}

/// Single `Runtime.evaluate` returning JSON string with layout viewport, document size, and scroll offsets.
const BROWSER_LAYOUT_METRICS_JS: &str = r#"(function(){
  var de = document.documentElement;
  var body = document.body;
  var docW = de.scrollWidth || de.clientWidth;
  var docH = de.scrollHeight || de.clientHeight;
  if (body) {
    docW = Math.max(docW, body.scrollWidth, body.clientWidth);
    docH = Math.max(docH, body.scrollHeight, body.clientHeight);
  }
  return JSON.stringify({
    scrollX: Math.floor(window.scrollX || window.pageXOffset || 0),
    scrollY: Math.floor(window.scrollY || window.pageYOffset || 0),
    viewportWidth: Math.floor(window.innerWidth || 0),
    viewportHeight: Math.floor(window.innerHeight || 0),
    documentWidth: Math.floor(docW),
    documentHeight: Math.floor(docH)
  });
})()"#;

fn try_browser_layout_metrics(tab: &headless_chrome::Tab) -> Option<BrowserLayoutMetrics> {
    let result = match tab.evaluate(BROWSER_LAYOUT_METRICS_JS, false) {
        Ok(r) => r,
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: layout metrics evaluate failed: {}",
                e
            );
            return None;
        }
    };
    let json_str = result.value.as_ref().and_then(|v| v.as_str())?;
    let parsed: BrowserLayoutMetricsJs = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: layout metrics JSON parse failed: {} (raw len {})",
                e,
                json_str.len()
            );
            return None;
        }
    };
    Some(BrowserLayoutMetrics {
        scroll_x: parsed.scroll_x.floor() as i32,
        scroll_y: parsed.scroll_y.floor() as i32,
        viewport_width: parsed.viewport_width.floor().max(0.0) as u32,
        viewport_height: parsed.viewport_height.floor().max(0.0) as u32,
        document_width: parsed.document_width.floor().max(0.0) as u32,
        document_height: parsed.document_height.floor().max(0.0) as u32,
    })
}

/// Get current browser state (URL, title, interactables). Call after navigate or after click/input.
pub fn get_browser_state(tab: &headless_chrome::Tab) -> Result<BrowserState, String> {
    let current_url = tab.get_url();
    let page_title = tab
        .evaluate("document.title", false)
        .ok()
        .and_then(|r| r.value.as_ref().and_then(|v| v.as_str().map(String::from)));
    let interactables = get_interactables(tab)?;
    let resource_timing_entry_count = resource_timing_resource_count(tab);
    let layout_metrics = try_browser_layout_metrics(tab);
    Ok(BrowserState {
        current_url,
        page_title,
        interactables,
        resource_timing_entry_count,
        layout_metrics,
    })
}

fn truncate_for_llm_tab_field(s: &str, max_chars: usize) -> String {
    s.replace('\n', " ")
        .replace('|', "/")
        .chars()
        .take(max_chars)
        .collect()
}

/// One-line summary of open tabs for the LLM (`Tabs: [0] url "title" (active) | ...`).
fn format_tabs_line_for_llm(
    browser: &Browser,
    current_idx: usize,
    focused_page_title: Option<&str>,
) -> Result<String, String> {
    let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    if tabs.is_empty() {
        return Ok(String::new());
    }
    let mut parts = Vec::new();
    for (i, tab) in tabs.iter().enumerate() {
        let url = truncate_for_llm_tab_field(&tab.get_url(), 120);
        let title_raw = if i == current_idx {
            focused_page_title
                .filter(|t| !t.trim().is_empty())
                .map(std::string::ToString::to_string)
                .unwrap_or_else(|| {
                    tab.get_target_info()
                        .map(|info| info.title)
                        .unwrap_or_default()
                })
        } else {
            tab.get_target_info()
                .map(|info| info.title)
                .unwrap_or_default()
        };
        let title = truncate_for_llm_tab_field(&title_raw, 60);
        let active = if i == current_idx { " (active)" } else { "" };
        parts.push(format!("[{}] {} \"{}\"{}", i, url, title, active));
    }
    Ok(format!("Tabs: {}", parts.join(" | ")))
}

fn format_browser_state_for_llm_impl(
    state: &BrowserState,
    tabs_ctx: Option<(&Browser, usize)>,
) -> String {
    record_viewport_css_for_llm_coord_scaling(viewport_width(), viewport_height());
    let mut s = String::new();
    if url_or_title_suggests_certificate_interstitial(
        state.current_url.as_str(),
        state.page_title.as_deref(),
    ) {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: LLM snapshot prepended chrome-error/TLS hint (url or title)"
        );
        s.push_str(
            "Warning: Chrome error or TLS interstitial; numbered elements below are not a real loaded site.\n",
        );
    } else if is_new_tab_page_url(state.current_url.as_str()) {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: LLM snapshot prepended new-tab/blank hint"
        );
        s.push_str(
            "Note: tab is on a browser new-tab or blank page, not a website. Use BROWSER_NAVIGATE with a real https URL when you need page content.\n",
        );
    }
    if let Some((browser, cur_idx)) = tabs_ctx {
        match format_tabs_line_for_llm(browser, cur_idx, state.page_title.as_deref()) {
            Ok(line) if !line.is_empty() => {
                s.push_str(&line);
                s.push('\n');
            }
            Ok(_) => {}
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: tab list for LLM skipped: {}",
                    e
                );
            }
        }
    }
    if let Some(m) = state.layout_metrics {
        s.push_str(&format!(
            "Viewport: {}x{}\nDocument: {}x{}\nScroll: ({}, {})\n",
            m.viewport_width,
            m.viewport_height,
            m.document_width,
            m.document_height,
            m.scroll_x,
            m.scroll_y
        ));
    }
    s.push_str(&format!("Current page: {}\n", state.current_url));
    if let Some(ref t) = state.page_title {
        s.push_str(&format!("Title: {}\n", t));
    }
    if let Some(n) = state.resource_timing_entry_count {
        s.push_str(&format!(
            "Performance resource entries (getEntriesByType('resource')): {}\n",
            n
        ));
    }
    s.push_str("Elements:\n");
    for i in &state.interactables {
        let kind = if i.tag == "a" {
            "link".to_string()
        } else if i.tag == "select" {
            "select".to_string()
        } else if i.contenteditable {
            "contenteditable".to_string()
        } else if i.tag == "input" {
            let mut s = format!("input[{}]", i.input_type.as_deref().unwrap_or("text"));
            if i.datepicker_like {
                s.push_str("[datepicker]");
            }
            s
        } else if i.tag == "textarea" {
            "input[textarea]".to_string()
        } else {
            "button".to_string()
        };
        let label = if let Some(a) = i.accessible_name.as_ref().filter(|s| !s.is_empty()) {
            a.as_str()
        } else if !i.text.is_empty() {
            i.text.as_str()
        } else if let Some(ref p) = i.placeholder {
            p.as_str()
        } else if let Some(ref h) = i.href {
            h.as_str()
        } else {
            "(no label)"
        };
        let label_escaped = label
            .replace('\n', " ")
            .replace('"', "'")
            .chars()
            .take(80)
            .collect::<String>();
        let role_shown = i
            .ax_role
            .as_deref()
            .filter(|r| !r.is_empty())
            .unwrap_or("-");
        let bbox = i
            .bounds_css
            .map(|(x, y, w, h)| format!("{:.0},{:.0},{:.0},{:.0}", x, y, w, h))
            .unwrap_or_else(|| "-".to_string());
        let covered = if i.covered { "yes" } else { "no" };
        s.push_str(&format!(
            "[{}] {} tag={} role={} bbox={} covered={} \"{}\"\n",
            i.index, kind, i.tag, role_shown, bbox, covered, label_escaped
        ));
    }
    if state.interactables.is_empty() {
        s.push_str("(no interactive elements found)\n");
    }
    s.push_str(&recent_js_dialogs_section_for_cdp_llm());
    credentials::append_available_credentials_hint(&state.current_url, &mut s);
    s
}

/// Format BrowserState as a string for the LLM (Current page: URL, Elements: [1] ...).
/// Omits the open-tabs line (used for tests and contexts without a [`Browser`] handle).
pub fn format_browser_state_for_llm(state: &BrowserState) -> String {
    format_browser_state_for_llm_impl(state, None)
}

/// Full LLM snapshot including `Tabs:` line when CDP session is available.
fn format_browser_state_snapshot(browser: &Browser, state: &BrowserState) -> String {
    let cur = current_tab_index().lock().map(|g| *g).unwrap_or(0);
    format_browser_state_for_llm_impl(state, Some((browser, cur)))
}

fn new_tabs_opened_notice(browser: &Browser, previous_len: usize) -> Option<String> {
    let tabs = browser.get_tabs().lock().ok()?;
    if tabs.len() <= previous_len {
        return None;
    }
    let mut parts = Vec::new();
    for i in previous_len..tabs.len() {
        let url = truncate_for_llm_tab_field(&tabs[i].get_url(), 120);
        let title = tabs[i]
            .get_target_info()
            .map(|info| truncate_for_llm_tab_field(&info.title, 50))
            .unwrap_or_default();
        parts.push(format!("[{}] {} \"{}\"", i, url, title));
    }
    Some(format!("Note: A new tab opened: {}\n", parts.join(" | ")))
}

fn new_focus_index_after_close(current: usize, close_idx: usize, new_len: usize) -> usize {
    if new_len == 0 {
        return 0;
    }
    let idx = if close_idx < current {
        current - 1
    } else if close_idx > current {
        current
    } else if close_idx > 0 {
        close_idx - 1
    } else {
        0
    };
    idx.min(new_len.saturating_sub(1))
}

/// When tab-limit enforcement cannot run safely, log at **warn** once per process (fail-open hygiene).
static BROWSER_TAB_LIMIT_ENFORCE_WARNED: AtomicBool = AtomicBool::new(false);

fn warn_browser_tab_limit_enforcement_skipped(reason: &str) {
    if BROWSER_TAB_LIMIT_ENFORCE_WARNED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: managed tab cap: skipping further enforcement attempts ({})",
            reason
        );
    }
}

/// Best-effort: if open page tabs exceed [`crate::config::Config::browser_max_page_tabs`], close **other**
/// tabs in **ascending index order** (oldest in Chrome’s tab list first) until within the cap. Never closes `keep`.
/// Updates [`CURRENT_TAB_INDEX`] like [`close_tab_at_index_inner`]. No-op when cap is 0 or already within limit.
fn try_enforce_browser_tab_limit(browser: &Browser, keep: &Arc<headless_chrome::Tab>) {
    let limit = crate::config::Config::browser_max_page_tabs();
    if limit == 0 {
        return;
    }
    let keep_id = keep.get_target_id().clone();
    let keep_id_str = keep_id.as_str();
    let before = match browser.get_tabs().lock() {
        Ok(t) => t.len(),
        Err(e) => {
            warn_browser_tab_limit_enforcement_skipped(&format!("get_tabs lock poisoned: {}", e));
            return;
        }
    };
    if before <= limit {
        return;
    }
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: managed tab cap: before_count={} limit={} keep_target_id={}",
        before,
        limit,
        truncate_target_id_for_log(keep_id_str)
    );
    let mut closed_total: u32 = 0;
    loop {
        let tabs_guard = match browser.get_tabs().lock() {
            Ok(t) => t,
            Err(e) => {
                warn_browser_tab_limit_enforcement_skipped(&format!(
                    "get_tabs lock poisoned: {}",
                    e
                ));
                break;
            }
        };
        let n = tabs_guard.len();
        if n <= limit {
            drop(tabs_guard);
            break;
        }
        let keep_idx = tabs_guard
            .iter()
            .position(|t| t.get_target_id().as_str() == keep_id_str);
        let Some(keep_idx) = keep_idx else {
            drop(tabs_guard);
            warn_browser_tab_limit_enforcement_skipped(
                "focused tab target_id not found among open tabs",
            );
            break;
        };
        // Close the lowest-index tab that is not the kept tab (deterministic "oldest other" order).
        let Some(close_idx) = (0..n).find(|&i| i != keep_idx) else {
            drop(tabs_guard);
            break;
        };
        let to_close = tabs_guard.get(close_idx).cloned();
        drop(tabs_guard);
        let Some(tclose) = to_close else {
            warn_browser_tab_limit_enforcement_skipped("tab disappeared before close");
            break;
        };
        if let Err(e) = tclose.close(false) {
            warn_browser_tab_limit_enforcement_skipped(&e.to_string());
            break;
        }
        closed_total = closed_total.saturating_add(1);
        let new_len = browser
            .get_tabs()
            .lock()
            .map(|t| t.len())
            .unwrap_or(n.saturating_sub(1));
        if let Ok(mut g) = current_tab_index().lock() {
            *g = new_focus_index_after_close(*g, close_idx, new_len);
        }
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: managed tab cap: closed tab index={} ({} tab(s) remain)",
            close_idx,
            new_len
        );
    }
    let after = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(before);
    if closed_total > 0 {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: managed tab cap: after_count={} closed={}",
            after,
            closed_total
        );
        let _ = keep.bring_to_front();
    }
}

/// Switch focused tab by **0-based** index (same numbering as the `Tabs:` line). Updates CDP focus and the cached current-tab index.
pub fn switch_tab_to_index(index: usize) -> Result<String, String> {
    with_connection_retry(|| switch_tab_to_index_inner(index))
}

fn switch_tab_to_index_inner(index: usize) -> Result<String, String> {
    let browser = get_or_create_browser(cdp_debug_port())
        .inspect_err(|e| clear_browser_session_on_error(e))?;
    let n = {
        let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        let n = tabs.len();
        if n == 0 {
            return Err("BROWSER_SWITCH_TAB: no open tabs.".to_string());
        }
        if index >= n {
            return Err(format!(
                "BROWSER_SWITCH_TAB: index {} is out of range ({} tab(s), valid indices 0..={})",
                index,
                n,
                n.saturating_sub(1)
            ));
        }
        n
    };
    if let Ok(mut g) = current_tab_index().lock() {
        *g = index;
    }
    let tab = {
        let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        tabs.get(index)
            .cloned()
            .ok_or_else(|| "BROWSER_SWITCH_TAB: tab disappeared".to_string())?
    };
    tab.bring_to_front()
        .map_err(|e| format!("BROWSER_SWITCH_TAB: bring_to_front: {}", e))?;
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_SWITCH_TAB index={} (tab_count={})",
        index,
        n
    );
    register_dialog_auto_dismiss(&tab);
    let bounds = Bounds::Normal {
        left: None,
        top: None,
        width: Some(viewport_width() as f64),
        height: Some(viewport_height() as f64),
    };
    if let Err(e) = tab.set_bounds(bounds) {
        mac_stats_warn!(
            "browser",
            "Browser agent: BROWSER_SWITCH_TAB set_bounds failed: {} (continuing)",
            e
        );
    }
    cookie_storage::apply_pending_cookie_restore(&tab);
    check_browser_alive(&browser, &tab)?;
    record_active_automation_target(&tab);
    assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None)?;
    let state = get_browser_state(&tab).inspect_err(|e| clear_browser_session_on_error(e))?;
    let snapshot = format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    Ok(snapshot)
}

/// Close tab at **0-based** index. Cannot close the last remaining tab. Updates focus per browser-use-style rules.
pub fn close_tab_at_index(index: usize) -> Result<String, String> {
    with_connection_retry(|| close_tab_at_index_inner(index))
}

fn close_tab_at_index_inner(index: usize) -> Result<String, String> {
    let browser = get_or_create_browser(cdp_debug_port())
        .inspect_err(|e| clear_browser_session_on_error(e))?;
    let current = current_tab_index()
        .lock()
        .map_err(|e| e.to_string())
        .map(|g| *g)?;
    let n = {
        let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        let n = tabs.len();
        if n == 0 {
            return Err("BROWSER_CLOSE_TAB: no open tabs.".to_string());
        }
        if n == 1 {
            return Err(
                "BROWSER_CLOSE_TAB: cannot close the last tab; open another tab first or use BROWSER_NAVIGATE with new_tab."
                    .to_string(),
            );
        }
        if index >= n {
            return Err(format!(
                "BROWSER_CLOSE_TAB: index {} is out of range ({} tab(s), valid indices 0..={})",
                index,
                n,
                n.saturating_sub(1)
            ));
        }
        n
    };
    let tab = {
        let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        tabs.get(index)
            .cloned()
            .ok_or_else(|| "BROWSER_CLOSE_TAB: tab disappeared".to_string())?
    };
    tab.close(false).map_err(|e| {
        let s = format!("BROWSER_CLOSE_TAB: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_CLOSE_TAB closed index={} (had {} tabs)",
        index,
        n
    );
    let new_len = browser.get_tabs().lock().map_err(|e| e.to_string())?.len();
    let new_idx = new_focus_index_after_close(current, index, new_len);
    if let Ok(mut g) = current_tab_index().lock() {
        *g = new_idx;
    }
    let (_, tab) = get_current_tab().inspect_err(|e| clear_browser_session_on_error(e))?;
    assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None)?;
    let state = get_browser_state(&tab).inspect_err(|e| clear_browser_session_on_error(e))?;
    let body = format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    let snapshot = format!("BROWSER_CLOSE_TAB: closed tab [{}].\n{}", index, body);
    set_last_browser_state_snapshot(snapshot.clone());
    Ok(snapshot)
}

/// Extract telephone numbers from text. German-style: +49..., 0..., etc.
pub fn extract_telephone_numbers(text: &str) -> Vec<String> {
    let re = Regex::new(
        r#"(?x)
        \+49\s*[\d\s\/\-\(\)]{6,}|
        0049\s*[\d\s\/\-\(\)]{6,}|
        \+[1-9]\d{0,2}\s*[\d\s\/\-\(\)]{6,}|
        0\d{2,4}[\s\/\-]?\d[\d\s\/\-]{4,}
        "#,
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
    mac_stats_info!(
        "browser",
        "Browser agent: launching Chrome via headless_chrome"
    );
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

fn fetch_page_and_extract_phones_with_browser(
    browser: &Browser,
    url: &str,
) -> Result<Vec<String>, String> {
    let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    let tab = tabs.first().cloned().ok_or_else(|| "No tab".to_string())?;
    drop(tabs);
    cdp_fetch_proxy_auth::ensure_fetch_proxy_auth_on_tab(&tab);
    apply_cdp_emulation_to_tab(tab.as_ref());
    check_browser_alive(browser, &tab)?;
    mac_stats_info!("browser", "Browser agent: navigating to {}", url);
    let prev_url = tab.get_url();
    let nav_timeout_secs = crate::config::Config::browser_navigation_timeout_secs();
    tab.set_default_timeout(Duration::from_secs(nav_timeout_secs));
    let (post_nav_net_flight, _post_nav_net_guard) =
        prepare_post_nav_network_idle_tracking(tab.as_ref());
    let redirect_rws_buf = Arc::new(Mutex::new(VecDeque::new()));
    cdp_enable_network_for_redirect_chain_capture(&tab);
    let redirect_rws_weak =
        cdp_attach_redirect_chain_rws_listener(&tab, Arc::clone(&redirect_rws_buf));
    let _redirect_rws_guard = CdpRedirectRwsListenerGuard {
        tab: tab.as_ref(),
        weak: redirect_rws_weak,
    };
    with_lifecycle_event_buffer(&tab, |buf_opt| {
        if let Some(b) = buf_opt {
            if let Ok(mut q) = b.lock() {
                q.clear();
            }
        }
        if let Ok(mut q) = redirect_rws_buf.lock() {
            q.clear();
        }
        let nav_start = Instant::now();
        tab.navigate_to(url).map_err(|e| {
            let msg = e.to_string();
            let detail = navigate_failed_detail_from_display(&msg);
            log_navigation_cdp_failure(url, &detail);
            navigation_tool_result_for_failed_navigate(url, &detail)
        })?;
        synchronize_tab_after_cdp_navigation(
            &tab,
            prev_url.as_str(),
            url,
            buf_opt,
            nav_start,
            Duration::from_secs(nav_timeout_secs),
            nav_timeout_secs,
            None,
            post_nav_net_flight.as_ref(),
        )
    })?;
    cdp_validate_redirect_chain_from_rws_buffer(&redirect_rws_buf, url)?;
    let final_u = tab.get_url();
    if let Some(msg) = post_navigate_load_failure_message(url, final_u.as_str(), Some(tab.as_ref()))
    {
        return Err(msg);
    }
    assert_final_document_url_ssrf_post_check(final_u.as_str(), Some(url))?;
    std::thread::sleep(BROWSER_PHONE_EXTRACT_POST_NAV);
    // Scroll to bottom to trigger footer/lazy content
    let _ = tab.evaluate("window.scrollTo(0, document.body.scrollHeight)", false);
    std::thread::sleep(BROWSER_PHONE_EXTRACT_POST_SCROLL);
    let text = get_page_text(&tab)?;
    let mut phones = extract_telephone_numbers(&text);
    if phones.is_empty() {
        if let Ok(html) = get_page_html(&tab) {
            mac_stats_info!(
                "browser",
                "Browser agent: page HTML length {} chars",
                html.len()
            );
            let tel_links = extract_tel_from_html(&html);
            if !tel_links.is_empty() {
                mac_stats_info!(
                    "browser",
                    "Browser agent: found tel: links in HTML: {:?}",
                    tel_links
                );
            }
            phones = tel_links;
        }
    }
    if phones.is_empty() {
        mac_stats_warn!("browser",
            "Browser agent: no telephone numbers found in page text ({} chars). First 800 chars: {}",
            text.len(),
            text.chars().take(800).collect::<String>()
        );
    } else {
        for p in &phones {
            mac_stats_info!("browser", "Browser agent: telephone number found: {}", p);
        }
    }
    Ok(phones)
}

/// Cached browser session: (Browser, created_at, last_used, was_headless). Dropped when idle longer than browser_idle_timeout_secs.
type BrowserSessionCell = Mutex<Option<(Browser, Instant, Instant, bool)>>;
static BROWSER_SESSION: OnceLock<BrowserSessionCell> = OnceLock::new();

/// Index of the current tab for BROWSER_* actions (0 = first tab). Updated when BROWSER_NAVIGATE is used with new_tab.
static CURRENT_TAB_INDEX: OnceLock<Mutex<usize>> = OnceLock::new();

/// Mutex held for the entire "create new browser" path so only one thread can launch at a time (avoids multiple Chrome PIDs from races).
static LAUNCH_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

/// When true, one-shot CDP session clear + operation retry is suppressed (idle/preference teardown or app shutdown).
static BROWSER_INTENTIONAL_STOP: AtomicBool = AtomicBool::new(false);

/// CDP `TargetID` of the tab we last treated as the automation focus (see `CURRENT_TAB_INDEX`).
static ACTIVE_TAB_TARGET_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();

/// Host only (for logs) of the automation tab URL; paired with [`ACTIVE_TAB_TARGET_ID`] for crash diagnostics.
static ACTIVE_AUTOMATION_TAB_URL_HOST: OnceLock<Mutex<Option<String>>> = OnceLock::new();

/// When set, the next `get_current_tab` / `get_or_create_browser` returns a stable renderer-crash error once (then clears).
static PENDING_RENDERER_CRASH_TOOL_ERROR: OnceLock<Mutex<bool>> = OnceLock::new();

fn active_tab_target_id_store() -> &'static Mutex<Option<String>> {
    ACTIVE_TAB_TARGET_ID.get_or_init(|| Mutex::new(None))
}

fn active_automation_tab_url_host_store() -> &'static Mutex<Option<String>> {
    ACTIVE_AUTOMATION_TAB_URL_HOST.get_or_init(|| Mutex::new(None))
}

fn pending_renderer_crash_tool_error_flag() -> &'static Mutex<bool> {
    PENDING_RENDERER_CRASH_TOOL_ERROR.get_or_init(|| Mutex::new(false))
}

fn set_pending_renderer_crash_tool_error() {
    if let Ok(mut g) = pending_renderer_crash_tool_error_flag().lock() {
        *g = true;
    }
}

fn take_pending_renderer_crash_tool_error() -> Option<String> {
    let Ok(mut g) = pending_renderer_crash_tool_error_flag().lock() else {
        return None;
    };
    if *g {
        *g = false;
        Some("Browser tab renderer crashed; session reset".to_string())
    } else {
        None
    }
}

/// CDP `Target.targetCrashed` from the side WebSocket (see `cdp_target_crash_listener`).
pub(super) fn notify_target_renderer_crashed_side(crashed_target_id: &str) {
    let is_active = {
        let Ok(g) = active_tab_target_id_store().lock() else {
            return;
        };
        match g.as_deref() {
            Some(s) if s == crashed_target_id => true,
            _ => false,
        }
    };
    if !is_active {
        return;
    }
    let host = active_automation_tab_url_host_store()
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| "(unknown)".to_string());
    let tid_short = truncate_target_id_for_log(crashed_target_id);
    mac_stats_warn!(
        "browser/cdp",
        "Browser agent [CDP]: Target.targetCrashed for active automation tab target_id={} host={}",
        tid_short,
        host
    );
    set_pending_renderer_crash_tool_error();
    clear_cached_browser_session("target renderer crashed (CDP Target.targetCrashed)");
}

fn truncate_target_id_for_log(id: &str) -> String {
    let t = id.trim();
    const N: usize = 12;
    let count = t.chars().count();
    if count <= N {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(N).collect::<String>())
    }
}

fn launch_mutex() -> &'static Mutex<()> {
    LAUNCH_MUTEX.get_or_init(|| Mutex::new(()))
}

/// Command-line signature for headless Chrome launched by the headless_chrome crate (temp profile dir).
const HEADLESS_CHROME_PROFILE_SIGNATURE: &str = "rust-headless-chrome-profile";

/// Kill Chrome processes that were spawned by mac-stats (headless_chrome crate) and are no longer tracked.
/// Such orphans can appear after races or crashes. Call before launching a new headless Chrome so we don't accumulate processes.
pub fn kill_orphaned_browser_processes() {
    #[cfg(unix)]
    {
        // pgrep -f matches full command line; find Chrome with our temp profile path.
        let output = match Command::new("pgrep")
            .args(["-f", HEADLESS_CHROME_PROFILE_SIGNATURE])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                mac_stats_debug!(
                    "browser",
                    "Browser agent: pgrep for orphaned Chrome failed ({}), skipping cleanup",
                    e
                );
                return;
            }
        };
        if !output.status.success() {
            // No matching processes (pgrep exits 1 when no match)
            return;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pids: Vec<&str> = stdout.split_whitespace().collect();
        for pid_str in pids {
            if let Ok(pid) = pid_str.parse::<i32>() {
                if pid > 0 {
                    let _ = Command::new("kill").arg(pid.to_string()).status();
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: killed orphaned headless Chrome PID {}",
                        pid
                    );
                }
            }
        }
    }
}

/// Drop the cached CDP session on app shutdown so Chrome we launched does not outlive the process.
///
/// - **Headless** (`was_headless`): drop the [`Browser`] handle, then send SIGTERM to the child PID
///   when known (safety net if drop does not tear down the process).
/// - **Visible / user Chrome** (`was_headless == false`): only drop the handle (closes the WebSocket);
///   never kills the browser process.
pub fn close_browser_session() {
    struct SyncDebugLog;
    impl Drop for SyncDebugLog {
        fn drop(&mut self) {
            crate::logging::sync_debug_log_best_effort();
        }
    }
    let _sync_debug_log = SyncDebugLog;

    BROWSER_INTENTIONAL_STOP.store(true, Ordering::SeqCst);
    clear_owned_visible_chrome_child_pid();
    cdp_target_crash_listener::invalidate_listener_generation();
    let ws = cdp_downloads::peek_cdp_ws_url();
    cdp_trace_archive::stop_and_persist_best_effort(ws.as_deref(), "app shutdown");
    cdp_downloads::clear_stored_cdp_ws_url();
    let taken = match browser_session().lock() {
        Ok(mut g) => g.take(),
        Err(e) => {
            mac_stats_warn!(
                "browser",
                "Browser agent: could not lock session for shutdown (poisoned mutex: {}); skipping browser close",
                e
            );
            return;
        }
    };
    let Some((browser, _created_at, _last_used, was_headless)) = taken else {
        mac_stats_debug!(
            "browser",
            "Browser agent: no cached browser session to close on shutdown"
        );
        return;
    };
    clear_cdp_js_dialog_history();

    #[cfg(unix)]
    let headless_pid = if was_headless {
        browser.get_process_id()
    } else {
        None
    };

    drop(browser);

    #[cfg(unix)]
    if was_headless {
        if let Some(pid) = headless_pid.filter(|p| *p > 0) {
            match Command::new("kill").arg(pid.to_string()).status() {
                Ok(status) if status.success() => {
                    mac_stats_debug!(
                        "browser",
                        "Browser agent: sent SIGTERM to headless Chrome PID {} (shutdown safety net)",
                        pid
                    );
                }
                Ok(status) => {
                    mac_stats_debug!(
                        "browser",
                        "Browser agent: shutdown safety-net kill for PID {} exited with status {:?}",
                        pid,
                        status.code()
                    );
                }
                Err(e) => {
                    mac_stats_debug!(
                        "browser",
                        "Browser agent: shutdown safety-net kill for PID {} failed: {}",
                        pid,
                        e
                    );
                }
            }
        }
    }

    mac_stats_info!("browser", "Browser session closed on shutdown");
}

/// Last page's element list (index → label) for status messages. Set after each navigate/click/input so "Clicking element 7 (Accept all)…" can show the label.
/// HashMap for O(1) lookup when resolving labels for BROWSER_CLICK/BROWSER_INPUT status messages.
static LAST_ELEMENT_LABELS: OnceLock<Mutex<Option<HashMap<u32, String>>>> = OnceLock::new();
/// Last browser state text shown to the model. Used to re-ground retries after browser-step failures.
static LAST_BROWSER_STATE_SNAPSHOT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
/// CDP `Runtime` object ids for interactables in order (1-based index = position). Filled in `get_interactables` so
/// `BROWSER_CLICK` / `BROWSER_INPUT` match the augmented list (JS + `DOMDebugger.getEventListeners` extras).
static LAST_INTERACTABLE_OBJECT_IDS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
/// Full interactables from the last CDP snapshot (for identity remapping when indices drift).
static LAST_INTERACTABLES_SNAPSHOT: OnceLock<Mutex<Option<Vec<Interactable>>>> = OnceLock::new();

/// Internal snapshot used to enrich returned `BROWSER_*` tool errors with compact readiness context.
///
/// Kept in-process and intentionally best-effort: it describes the *last known* CDP/discovery/session state
/// so operators can distinguish "port not listening" vs "WebSocket up but automation not ready" quickly.
struct BrowserHealthSnapshot {
    /// Configured CDP port (even if current transport is headless/HTTP-only).
    cdp_port: u16,
    /// Result of the most recent `/json/version` probe (get_ws_url attempt).
    cdp_http_ok: Option<bool>, // None => not used (e.g. headless transport)
    cdp_http_err_summary: Option<String>, // truncated summary
    /// Result of the most recent CDP attach attempt (connect + ready poll).
    ws_ok: Option<bool>, // None => not used (e.g. headless transport)
    ws_err_summary: Option<String>,       // truncated summary
    /// Whether the currently cached session was reused from idle cache vs created for this tool call.
    session_created_this_turn: Option<bool>,
    /// Elapsed time since session creation (if known).
    session_created_at: Option<Instant>,
}

static LAST_BROWSER_HEALTH: OnceLock<Mutex<Option<BrowserHealthSnapshot>>> = OnceLock::new();

/// Captured on navigation timeouts so the dispatcher can include `navchg=<0|1>` in error context.
static LAST_NAV_TIMEOUT_URL_CHANGED_HINT: OnceLock<Mutex<Option<bool>>> = OnceLock::new();

fn last_element_labels() -> &'static Mutex<Option<HashMap<u32, String>>> {
    LAST_ELEMENT_LABELS.get_or_init(|| Mutex::new(None))
}

fn last_browser_state_snapshot() -> &'static Mutex<Option<String>> {
    LAST_BROWSER_STATE_SNAPSHOT.get_or_init(|| Mutex::new(None))
}

fn last_interactable_object_ids() -> &'static Mutex<Vec<String>> {
    LAST_INTERACTABLE_OBJECT_IDS.get_or_init(|| Mutex::new(Vec::new()))
}

fn last_interactables_snapshot() -> &'static Mutex<Option<Vec<Interactable>>> {
    LAST_INTERACTABLES_SNAPSHOT.get_or_init(|| Mutex::new(None))
}

fn set_interactable_object_ids(ids: Vec<String>) {
    if let Ok(mut g) = last_interactable_object_ids().lock() {
        *g = ids;
    }
}

fn clear_interactable_object_ids() {
    if let Ok(mut g) = last_interactable_object_ids().lock() {
        g.clear();
    }
    if let Ok(mut g) = last_interactables_snapshot().lock() {
        *g = None;
    }
}

fn last_browser_health() -> &'static Mutex<Option<BrowserHealthSnapshot>> {
    LAST_BROWSER_HEALTH.get_or_init(|| Mutex::new(None))
}

fn last_nav_timeout_url_changed_hint() -> &'static Mutex<Option<bool>> {
    LAST_NAV_TIMEOUT_URL_CHANGED_HINT.get_or_init(|| Mutex::new(None))
}

/// Configured viewport CSS size recorded whenever browser state is formatted for the LLM.
static LAST_VIEWPORT_CSS_FOR_LLM_COORDS: OnceLock<Mutex<(u32, u32)>> = OnceLock::new();
/// Pixel size of the screenshot image last sent to a vision LLM when LLM resize was applied (completion verification).
static LAST_LLM_SCREENSHOT_PIXEL_DIMS: OnceLock<Mutex<Option<(u32, u32)>>> = OnceLock::new();

fn last_viewport_css_for_llm_coords() -> &'static Mutex<(u32, u32)> {
    LAST_VIEWPORT_CSS_FOR_LLM_COORDS.get_or_init(|| Mutex::new((0, 0)))
}

fn last_llm_screenshot_pixel_dims() -> &'static Mutex<Option<(u32, u32)>> {
    LAST_LLM_SCREENSHOT_PIXEL_DIMS.get_or_init(|| Mutex::new(None))
}

/// Record configured viewport CSS size whenever browser state is produced for the model.
pub fn record_viewport_css_for_llm_coord_scaling(w: u32, h: u32) {
    if let Ok(mut g) = last_viewport_css_for_llm_coords().lock() {
        *g = (w, h);
    }
}

/// Set (or clear) dimensions of the last vision message image when resize-for-LLM was used.
pub fn set_last_llm_screenshot_pixel_dims_for_coord_scaling(dims: Option<(u32, u32)>) {
    if let Ok(mut g) = last_llm_screenshot_pixel_dims().lock() {
        *g = dims;
    }
}

/// Map (x, y) from resized screenshot space into viewport CSS pixels when resize was used for the last vision call.
pub fn scale_click_coords_from_llm_screenshot_space(x: f64, y: f64) -> (f64, f64) {
    let Ok(llm_lock) = last_llm_screenshot_pixel_dims().lock() else {
        return (x, y);
    };
    let Some((rw, rh)) = *llm_lock else {
        return (x, y);
    };
    let Ok(vp_lock) = last_viewport_css_for_llm_coords().lock() else {
        return (x, y);
    };
    let (vw, vh) = *vp_lock;
    if vw == 0 || vh == 0 || rw == 0 || rh == 0 {
        return (x, y);
    }
    let vx = x * vw as f64 / rw as f64;
    let vy = y * vh as f64 / rh as f64;
    mac_stats_info!(
        "browser/llm_screenshot",
        "Scaled click coords from LLM screenshot space {}x{} to viewport {}x{}: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
        rw,
        rh,
        vw,
        vh,
        x,
        y,
        vx,
        vy
    );
    (vx, vy)
}

fn truncate_compact(s: &str, max_chars: usize) -> String {
    let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut it = t.chars().take(max_chars);
    let out: String = it.by_ref().collect();
    if t.chars().count() > max_chars {
        format!("{}…", out)
    } else {
        out
    }
}

fn with_last_browser_health_mut<F>(cdp_port: u16, f: F)
where
    F: FnOnce(&mut BrowserHealthSnapshot),
{
    if let Ok(mut guard) = last_browser_health().lock() {
        if guard.is_none() {
            *guard = Some(BrowserHealthSnapshot {
                cdp_port,
                cdp_http_ok: None,
                cdp_http_err_summary: None,
                ws_ok: None,
                ws_err_summary: None,
                session_created_this_turn: None,
                session_created_at: None,
            });
        }
        if let Some(ref mut h) = guard.as_mut() {
            f(h)
        }
    }
}

fn record_cdp_http_probe_result(cdp_port: u16, ok: bool, err_summary: Option<String>) {
    with_last_browser_health_mut(cdp_port, |h| {
        h.cdp_port = cdp_port;
        h.cdp_http_ok = Some(ok);
        h.cdp_http_err_summary = err_summary;
    })
}

fn record_cdp_attach_result(cdp_port: u16, ok: bool, err_summary: Option<String>) {
    with_last_browser_health_mut(cdp_port, |h| {
        h.cdp_port = cdp_port;
        h.ws_ok = Some(ok);
        h.ws_err_summary = err_summary;
    })
}

fn record_session_created_this_turn(cdp_port: u16, reused: bool, created_at: Instant) {
    with_last_browser_health_mut(cdp_port, |h| {
        h.cdp_port = cdp_port;
        h.session_created_this_turn = Some(!reused);
        h.session_created_at = Some(created_at);
    })
}

pub(crate) fn take_last_nav_timeout_url_changed_hint() -> Option<bool> {
    let mut guard = last_nav_timeout_url_changed_hint().lock().ok()?;
    guard.take()
}

fn normalize_origin_path_for_nav_timeout_hint(s: &str) -> Option<String> {
    let u = Url::parse(s).ok()?;
    let scheme = u.scheme().to_ascii_lowercase();
    let host = u.host_str()?.to_ascii_lowercase();
    let mut path = u.path().trim_end_matches('/').to_string();
    if path.is_empty() {
        path = "/".to_string();
    }
    Some(format!("{}://{}{}", scheme, host, path))
}

fn record_nav_timeout_url_changed_hint(requested_url: &str, tab: &headless_chrome::Tab) {
    let final_url = tab.get_url();
    let nav_changed = match (
        normalize_origin_path_for_nav_timeout_hint(requested_url),
        normalize_origin_path_for_nav_timeout_hint(final_url.as_str()),
    ) {
        (Some(a), Some(b)) => a != b,
        _ => final_url.as_str() != requested_url,
    };
    if let Ok(mut g) = last_nav_timeout_url_changed_hint().lock() {
        *g = Some(nav_changed);
    }
}

fn format_context_suffix_from_health(
    health: &BrowserHealthSnapshot,
    nav_url_changed: Option<bool>,
    max_chars: usize,
) -> String {
    let cdp_http = match health.cdp_http_ok {
        Some(true) => "ok".to_string(),
        Some(false) => {
            let summary = health
                .cdp_http_err_summary
                .as_deref()
                .map(|s| truncate_compact(s, 40))
                .unwrap_or_else(|| "err".to_string());
            format!("fail({})", summary)
        }
        None => "not_used".to_string(),
    };
    let ws = match health.ws_ok {
        Some(true) => "ok".to_string(),
        Some(false) => {
            let summary = health
                .ws_err_summary
                .as_deref()
                .map(|s| truncate_compact(s, 40))
                .unwrap_or_else(|| "err".to_string());
            format!("fail({})", summary)
        }
        None => "not_used".to_string(),
    };
    let sess = match health.session_created_this_turn {
        Some(true) => "new",
        Some(false) => "reused",
        None => "?",
    };
    let age = health
        .session_created_at
        .map(|t| {
            let secs = Instant::now().duration_since(t).as_secs_f64();
            // Keep compact: 1 decimal is enough for operator debugging.
            format!("{:.1}s", secs)
        })
        .unwrap_or_else(|| "na".to_string());
    let mut s = format!(
        "context: cdp_http={} ws={} port={} sess={} age={}",
        cdp_http, ws, health.cdp_port, sess, age
    );
    if let Some(changed) = nav_url_changed {
        s.push_str(&format!(" navchg={}", if changed { 1 } else { 0 }));
    }
    if s.chars().count() > max_chars {
        s.chars().take(max_chars - 1).collect::<String>() + "…"
    } else {
        s
    }
}

pub(crate) fn format_last_browser_error_context(
    cdp_used: bool,
    nav_url_changed: Option<bool>,
) -> Option<String> {
    if !cdp_used {
        let mut s = "context: cdp=not_used".to_string();
        if let Some(changed) = nav_url_changed {
            s.push_str(&format!(" navchg={}", if changed { 1 } else { 0 }));
        }
        return Some(s);
    }
    let guard = last_browser_health().lock().ok()?;
    guard
        .as_ref()
        .map(|h| format_context_suffix_from_health(h, nav_url_changed, 190))
}

fn element_label_for_status(i: &Interactable) -> String {
    let label = if let Some(a) = i.accessible_name.as_ref().filter(|s| !s.trim().is_empty()) {
        a.as_str()
    } else if !i.text.is_empty() {
        i.text.as_str()
    } else if let Some(ref p) = i.placeholder {
        p.as_str()
    } else if let Some(ref h) = i.href {
        h.as_str()
    } else {
        "(no label)"
    };
    label
        .replace('\n', " ")
        .chars()
        .take(40)
        .collect::<String>()
}

fn cache_cdp_interactable_tool_state(interactables: &[Interactable]) {
    set_last_element_labels(
        interactables
            .iter()
            .map(|i| (i.index, element_label_for_status(i)))
            .collect(),
    );
    if let Ok(mut g) = last_interactables_snapshot().lock() {
        *g = Some(interactables.to_vec());
    }
}

/// Set the last page's element labels (called after navigate/click/input). Used so status can show "Clicking element 7 (Accept all)…".
/// Duplicate indices: last entry wins. Callers (CDP/HTTP paths) supply unique indices from page state.
pub(crate) fn set_last_element_labels(labels: Vec<(u32, String)>) {
    if let Ok(mut g) = last_element_labels().lock() {
        *g = Some(labels.into_iter().collect());
    }
}

pub(crate) fn set_last_browser_state_snapshot(snapshot: String) {
    http_fallback::mark_browser_session_cdp();
    if let Ok(mut g) = last_browser_state_snapshot().lock() {
        *g = Some(snapshot);
    }
}

/// Get the label for element at 1-based index from the last cached page state. Used for status message context.
/// Edge cases: returns None if lock is poisoned, cache is empty, or index not in last state (e.g. first action in run before any navigate).
pub fn get_last_element_label(index: u32) -> Option<String> {
    last_element_labels()
        .lock()
        .ok()
        .and_then(|g| g.as_ref().and_then(|labels| labels.get(&index).cloned()))
}

pub fn get_last_browser_state_snapshot() -> Option<String> {
    last_browser_state_snapshot()
        .lock()
        .ok()
        .and_then(|g| g.clone())
}

/// User said "headless" → true (no visible window). User said "browser" or default → false (visible desktop app).
static PREFER_HEADLESS: AtomicBool = AtomicBool::new(false);

/// Set headless preference for this request. Call at start of tool loop from question.
/// "headless" in question → true. Otherwise → false (visible Chrome).
pub fn set_prefer_headless_for_run(prefer: bool) {
    PREFER_HEADLESS.store(prefer, Ordering::Relaxed);
}

/// Current headless preference. Used so we never launch visible Chrome when headless was requested (e.g. on BROWSER_NAVIGATE retry).
pub fn prefer_headless_for_run() -> bool {
    PREFER_HEADLESS.load(Ordering::Relaxed)
}

/// Tracks tabs where the JS dialog auto-dismiss handler has been registered (by Arc raw pointer).
static DIALOG_REGISTERED_TABS: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();

fn dialog_registered_tabs() -> &'static Mutex<HashSet<usize>> {
    DIALOG_REGISTERED_TABS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Register a JS dialog (alert/confirm/prompt/beforeunload) auto-dismiss handler on a tab.
/// Idempotent: skips if the handler was already registered on this exact tab instance.
fn register_dialog_auto_dismiss(tab: &Arc<headless_chrome::Tab>) {
    let ptr = Arc::as_ptr(tab) as usize;
    if let Ok(mut set) = dialog_registered_tabs().lock() {
        if !set.insert(ptr) {
            return;
        }
    }
    let tab_weak = Arc::downgrade(tab);
    let listener = Arc::new(move |event: &Event| {
        if let Event::PageJavascriptDialogOpening(ev) = event {
            let dialog_type = &ev.params.Type;
            let message = &ev.params.message;
            let accept = !matches!(dialog_type, DialogType::Prompt);
            mac_stats_debug!("browser/cdp",
                "Browser agent [CDP]: JS dialog opened (type={:?}, message={:?}) — auto-dismissing (accept={})",
                dialog_type, message, accept
            );
            if let Some(tab) = tab_weak.upgrade() {
                let dialog = tab.get_dialog();
                let result = if accept {
                    dialog.accept(None)
                } else {
                    dialog.dismiss()
                };
                if let Err(e) = result {
                    mac_stats_warn!(
                        "browser/cdp",
                        "Browser agent [CDP]: failed to auto-dismiss JS dialog: {}",
                        e
                    );
                } else {
                    record_cdp_js_dialog_dismissed(dialog_type, message);
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: auto-dismissed JS {:?} dialog: {:?}",
                        dialog_type,
                        message.chars().take(100).collect::<String>()
                    );
                }
            }
        }
    });
    if let Err(e) = tab.add_event_listener(listener) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: failed to register dialog auto-dismiss handler: {}",
            e
        );
    } else {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: registered JS dialog auto-dismiss handler on tab"
        );
    }
}

fn browser_session() -> &'static Mutex<Option<(Browser, Instant, Instant, bool)>> {
    BROWSER_SESSION.get_or_init(|| Mutex::new(None))
}

fn is_connection_error(err_msg: &str) -> bool {
    err_msg.contains("connection is closed")
        || err_msg.contains("underlying connection")
        || err_msg.contains("timeout while listening")
        || err_msg.contains("Transport loop")
        || err_msg.contains("Unable to make method calls")
}

/// True if the current tab is a new-tab or blank page (e.g. after session reset). Caller should ask user to BROWSER_NAVIGATE first.
/// Aligns with [`is_new_tab_page_url`] so `chrome://new-tab-page` is treated like `chrome://newtab`.
fn is_new_tab_or_blank(url: &str) -> bool {
    let u = url.trim();
    u.is_empty() || is_new_tab_page_url(u)
}

/// Same URL set as browser-use `is_new_tab_page`: `about:blank`, `chrome://new-tab-page` /
/// `chrome://newtab` with optional trailing slash. Only the `chrome://` scheme prefix is
/// ASCII case-insensitive; the host/path segment matches browser-use literally.
fn is_new_tab_page_url(url: &str) -> bool {
    let url = url.trim();
    if url == "about:blank" {
        return true;
    }
    const PREFIX: &[u8] = b"chrome://";
    let b = url.as_bytes();
    if b.len() < PREFIX.len() || !b[..PREFIX.len()].eq_ignore_ascii_case(PREFIX) {
        return false;
    }
    let rest = &url[PREFIX.len()..];
    matches!(
        rest,
        "new-tab-page" | "new-tab-page/" | "newtab" | "newtab/"
    )
}

/// Chrome internal error document (load failure). Cookie-banner heuristics must not run here.
fn is_chrome_internal_error_document_url(url: &str) -> bool {
    url.trim()
        .to_ascii_lowercase()
        .starts_with("chrome-error://")
}

/// Classify Chrome `errorText` / displayed detail into a short operator-facing TLS class for logs.
fn tls_certificate_error_class_from_chrome_detail(detail: &str) -> Option<&'static str> {
    let lower = detail.to_ascii_lowercase();
    if lower.contains("net::err_cert_authority_invalid") {
        return Some("cert_authority");
    }
    if lower.contains("net::err_cert_date_invalid") {
        return Some("cert_date");
    }
    if lower.contains("net::err_cert_common_name_invalid") {
        return Some("cert_common_name");
    }
    if lower.contains("net::err_cert_weak_signature_algorithm") {
        return Some("cert_weak_signature");
    }
    if lower.contains("net::err_cert_invalid") {
        return Some("cert_invalid");
    }
    if lower.contains("net::err_cert_") {
        return Some("cert_other");
    }
    if lower.contains("net::err_ssl_") || lower.contains("err_ssl_") {
        return Some("ssl_error");
    }
    if lower.contains("certificate")
        && (lower.contains("invalid")
            || lower.contains("untrusted")
            || lower.contains("privacy")
            || lower.contains("not private"))
    {
        return Some("certificate_page");
    }
    None
}

fn page_title_suggests_certificate_error(title: &str) -> bool {
    let t = title.to_ascii_lowercase();
    t.contains("certificate")
        || t.contains("privacy error")
        || t.contains("not private")
        || t.contains("net::err_cert")
        || (t.contains("ssl") && t.contains("error"))
}

/// True when the tab URL is Chrome's error document or the title looks like a TLS interstitial.
fn url_or_title_suggests_certificate_interstitial(url: &str, title_opt: Option<&str>) -> bool {
    if is_chrome_internal_error_document_url(url) {
        return true;
    }
    title_opt
        .map(page_title_suggests_certificate_error)
        .unwrap_or(false)
}

fn tab_document_title_best_effort(tab: &headless_chrome::Tab) -> Option<String> {
    tab.evaluate("document.title", false).ok().and_then(|r| {
        r.value
            .as_ref()
            .and_then(|v| v.as_str().map(std::string::ToString::to_string))
    })
}

/// Host only for logs (no path/query) so tokens in URLs are not written to debug.log.
fn host_for_navigation_log(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(std::string::ToString::to_string))
        .unwrap_or_else(|| "(no-host)".to_string())
}

/// First token / short prefix of Chrome `errorText` for compact debug lines (e.g. `net::ERR_…`).
fn navigation_error_class(error_text: &str) -> String {
    let t = error_text.trim();
    if t.is_empty() {
        return "(empty)".to_string();
    }
    if let Some(pos) = t.find(|c: char| c.is_whitespace()) {
        t[..pos].chars().take(80).collect()
    } else {
        t.chars().take(80).collect()
    }
}

/// LLM-facing navigation error: trim, cap length, redact obvious local path segments.
fn sanitize_navigation_error_for_llm(error_text: &str) -> String {
    let re_pathy =
        Regex::new(r"(?i)(/Users/[^\s]+|/home/[^\s]+|file://[^\s]+)").expect("static regex");
    let mut s = error_text.trim().to_string();
    s = re_pathy.replace_all(&s, "[path]").to_string();
    const MAX: usize = 400;
    if s.chars().count() > MAX {
        return s.chars().take(MAX).collect::<String>() + "…";
    }
    s
}

fn log_navigation_cdp_failure(requested_url: &str, raw_error_text: &str) {
    let host = host_for_navigation_log(requested_url);
    let tls_class = tls_certificate_error_class_from_chrome_detail(raw_error_text);
    let class = tls_class
        .map(|s| s.to_string())
        .unwrap_or_else(|| navigation_error_class(raw_error_text));
    if tls_class.is_some() {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: navigation TLS/cert failure host={} error_class={}",
            host,
            class
        );
    } else {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: navigation failure host={} error_class={}",
            host,
            class
        );
    }
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: navigation failure full errorText={}",
        raw_error_text
    );
}

/// Tool result when CDP `Page.navigate` reported failure (`errorText`) or equivalent.
fn format_navigation_failed_for_tool(requested_url: &str, chrome_detail: &str) -> String {
    let sanitized = sanitize_navigation_error_for_llm(chrome_detail);
    format!(
        "Navigation failed: the target page did not load. Chrome reported: {}\nDo not treat the tab as having loaded the requested site. Requested URL: {}",
        sanitized, requested_url
    )
}

/// Dedicated TLS/certificate navigation result (operator-actionable; no blind automation on interstitials).
fn format_tls_certificate_navigation_failed_for_tool(
    requested_url: &str,
    chrome_detail: &str,
    tls_class_label: &'static str,
) -> String {
    let sanitized = sanitize_navigation_error_for_llm(chrome_detail);
    format!(
        "Navigation failed: TLS/certificate problem ({}). Chrome reported: {}\n\
Automation cannot bypass the security interstitial. Open the same URL once in the visible Chrome session attached to this app and proceed if you accept the risk. \
For read-only static HTML (no script execution), FETCH_URL may work where your host policy allows; it does not bypass SSRF or allowlist rules.\n\
Requested URL: {}",
        tls_class_label, sanitized, requested_url
    )
}

/// Picks TLS-specific vs generic navigation tool text using [`tls_certificate_error_class_from_chrome_detail`].
fn navigation_tool_result_for_failed_navigate(requested_url: &str, chrome_detail: &str) -> String {
    if let Some(tls_class) = tls_certificate_error_class_from_chrome_detail(chrome_detail) {
        format_tls_certificate_navigation_failed_for_tool(requested_url, chrome_detail, tls_class)
    } else {
        format_navigation_failed_for_tool(requested_url, chrome_detail)
    }
}

fn navigation_timeout_error_with_proxy_hint(timeout_secs_label: u64) -> String {
    let base = format!("Navigation failed: timeout after {}s", timeout_secs_label);
    if crate::config::Config::browser_cdp_proxy_credentials_active() {
        format!(
            "{}. If traffic goes through an authenticating HTTP(S) proxy, verify browserCdpProxyUsername and browserCdpProxyPassword; otherwise the destination may be slow or blocked.",
            base
        )
    } else {
        format!(
            "{}. If Chrome uses a proxy that requires login (typical on corporate networks), set browserCdpProxyUsername and browserCdpProxyPassword in config.json for the CDP-driven browser only (not used for FETCH_URL).",
            base
        )
    }
}

fn navigate_failed_detail_from_display(err_msg: &str) -> String {
    const PREFIX: &str = "Navigate failed: ";
    err_msg.strip_prefix(PREFIX).unwrap_or(err_msg).to_string()
}

/// After `navigate_to` returns Ok, Chrome may still show an error document without `errorText` on the navigate response.
fn post_navigate_load_failure_message(
    requested_url: &str,
    final_url: &str,
    tab: Option<&headless_chrome::Tab>,
) -> Option<String> {
    if is_chrome_internal_error_document_url(final_url) {
        let title_opt = tab.and_then(tab_document_title_best_effort);
        let tls_from_title = title_opt
            .as_ref()
            .map(|t| page_title_suggests_certificate_error(t))
            .unwrap_or(false);
        let host = host_for_navigation_log(requested_url);
        let log_class = if tls_from_title {
            "cert_interstitial_page"
        } else {
            "chrome_error"
        };
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: navigation TLS/cert or chrome-error host={} error_class={}",
            host,
            log_class
        );
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: navigation failure full errorText=chrome-error document (no errorText on navigate)"
        );
        if tls_from_title {
            return Some(format_tls_certificate_navigation_failed_for_tool(
                requested_url,
                "internal chrome-error page (TLS/certificate interstitial likely)",
                "cert_interstitial_page",
            ));
        }
        return Some(format_navigation_failed_for_tool(
            requested_url,
            "internal error page (chrome-error://); the network request did not succeed",
        ));
    }
    let requested_http = Url::parse(requested_url)
        .ok()
        .is_some_and(|u| matches!(u.scheme(), "http" | "https"));
    if requested_http && is_new_tab_or_blank(final_url) {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: heuristic navigation failure host={} final_url=blank-after-http",
            host_for_navigation_log(requested_url)
        );
        return Some(
            "Navigation may have failed: an HTTP(S) URL was requested but the tab URL is still blank; Chrome did not return errorText on Page.navigate. Do not assume the page loaded."
                .to_string(),
        );
    }
    None
}

const SESSION_RESET_MSG: &str = "Browser session was reset; current page is a new tab. Use BROWSER_NAVIGATE: <your target URL> first to reopen the page, then retry.";

/// Max wait for `Runtime.evaluate("1+1")` during CDP health checks (unresponsive browser / hung tab).
const BROWSER_CDP_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);

/// Max wait for `Runtime.evaluate("document.readyState")` before screenshot capture.
const BROWSER_DOCUMENT_READY_LIVENESS_TIMEOUT: Duration = Duration::from_secs(3);

/// Drop cached CDP session before a one-shot router retry (`OllamaRunError::BrowserSessionLost`).
pub fn invalidate_cached_browser_session_for_retry(reason: &str) {
    clear_cached_browser_session(reason);
}

/// Drop cached session so the next tool call reconnects or relaunchs Chrome.
fn clear_cached_browser_session(reason: &str) {
    let ws = cdp_downloads::peek_cdp_ws_url();
    cdp_trace_archive::stop_and_persist_best_effort(ws.as_deref(), reason);
    clear_owned_visible_chrome_child_pid();
    cdp_target_crash_listener::invalidate_listener_generation();
    cdp_fetch_proxy_auth::clear_proxy_auth_setup_targets();
    cdp_downloads::clear_stored_cdp_ws_url();
    clear_cdp_js_dialog_history();
    clear_interactable_object_ids();
    if let Ok(mut g) = active_tab_target_id_store().lock() {
        *g = None;
    }
    if let Ok(mut h) = active_automation_tab_url_host_store().lock() {
        *h = None;
    }
    if let Ok(mut guard) = browser_session().lock() {
        if guard.is_some() {
            *guard = None;
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: cleared session — {} (next use will reconnect or relaunch)",
                reason
            );
        }
    }
}

/// Clear the cached browser session so the next use will reconnect or relaunch.
/// Used for CDP transport errors and for explicit **Browser unresponsive** results from `check_browser_alive`.
fn clear_browser_session_on_error(err_msg: &str) {
    if is_connection_error(err_msg) {
        clear_cached_browser_session("connection error");
    } else if err_msg.contains("Browser unresponsive") {
        clear_cached_browser_session("browser unresponsive (health check)");
    }
}

/// `Runtime.evaluate("1+1")` on a worker thread; result delivered via oneshot for `tokio::time::timeout`.
async fn evaluate_one_plus_one_oneshot(tab: Arc<headless_chrome::Tab>) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let r = tab
            .evaluate("1+1", false)
            .map(|_| ())
            .map_err(|e| e.to_string());
        let _ = tx.send(r);
    });
    match rx.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!(
            "Browser unresponsive (JavaScript engine not responding): {}",
            e
        )),
        Err(_) => Err("Browser unresponsive: health check channel closed".to_string()),
    }
}

/// Same as [`evaluate_one_plus_one_oneshot`] when no Tokio runtime is available (sync callers).
fn evaluate_one_plus_one_blocking_timeout(tab: Arc<headless_chrome::Tab>) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let r = tab
            .evaluate("1+1", false)
            .map(|_| ())
            .map_err(|e| e.to_string());
        let _ = tx.send(r);
    });
    match rx.recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!(
            "Browser unresponsive (JavaScript engine not responding): {}",
            e
        )),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            Err("Browser unresponsive: health check timed out after 2s".to_string())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err("Browser unresponsive: health check thread ended unexpectedly".to_string())
        }
    }
}

/// Lightweight page liveness before screenshot: `document.readyState` under a short cap.
fn tab_document_ready_liveness(tab: &Arc<headless_chrome::Tab>) -> Result<(), String> {
    let tab = Arc::clone(tab);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let r = tab
            .evaluate("document.readyState", false)
            .map(|_| ())
            .map_err(|e| e.to_string());
        let _ = tx.send(r);
    });
    match rx.recv_timeout(BROWSER_DOCUMENT_READY_LIVENESS_TIMEOUT) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => {
            clear_cached_browser_session("tab liveness (document.readyState evaluate failed)");
            Err(format!("Tab liveness failed: {}", e))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            clear_cached_browser_session("tab liveness (document.readyState timed out)");
            Err("Tab liveness failed: document.readyState timed out after 3s".to_string())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            clear_cached_browser_session("tab liveness (evaluate thread ended)");
            Err("Tab liveness failed: evaluate thread ended unexpectedly".to_string())
        }
    }
}

/// Fast fail when Chrome is hung or dead but the WebSocket is still open: optional `kill -0` on our child PID,
/// then `Runtime.evaluate("1+1")` under a 2-second cap (`tokio::time::timeout` when a runtime handle exists).
fn check_browser_alive(browser: &Browser, tab: &Arc<headless_chrome::Tab>) -> Result<(), String> {
    #[cfg(unix)]
    if let Some(pid) = browser.get_process_id() {
        // SAFETY: `kill(pid, 0)` only tests process existence; no signal is delivered to the target.
        if unsafe { libc::kill(pid as i32, 0) } != 0 {
            let msg = format!(
                "Browser unresponsive: Chrome child process {} is no longer running",
                pid
            );
            mac_stats_warn!("browser/cdp", "Browser agent [CDP]: {}", msg);
            clear_browser_session_on_error(&msg);
            return Err(msg);
        }
    }

    let tab = Arc::clone(tab);
    let eval_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        // `block_on` on a Tokio worker thread deadlocks that worker (nested scheduling).
        // Heartbeat / scheduler threads run `answer_with_ollama_and_fetch` on their own runtime;
        // CDP health checks must not stall the executor. `block_in_place` hands off this worker.
        tokio::task::block_in_place(|| {
            match handle.block_on(tokio::time::timeout(
                BROWSER_CDP_HEALTH_CHECK_TIMEOUT,
                evaluate_one_plus_one_oneshot(tab),
            )) {
                Ok(inner) => inner,
                Err(_) => Err("Browser unresponsive: health check timed out after 2s".to_string()),
            }
        })
    } else {
        evaluate_one_plus_one_blocking_timeout(tab)
    };

    if let Err(ref e) = eval_result {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: {} — clearing cached browser session",
            e
        );
        clear_browser_session_on_error(e);
    }
    eval_result
}

fn should_retry_cdp_after_clearing_session(err_msg: &str) -> bool {
    // `check_browser_alive` already clears the session on "Browser unresponsive"; a second
    // `with_connection_retry` pass would reconnect to a fresh/blank tab and surface
    // SESSION_RESET_MSG instead of the health-check error (slow and confusing for the LLM).
    is_connection_error(err_msg)
        || err_msg.contains("Tab liveness failed")
        || err_msg.contains("Browser tab renderer crashed")
}

fn cdp_retry_reason_tag(err_msg: &str) -> &'static str {
    if err_msg.contains("Tab liveness failed") {
        "liveness"
    } else if err_msg.contains("Browser tab renderer crashed") {
        "target_crashed"
    } else if err_msg.contains("Browser unresponsive") {
        "browser_unresponsive"
    } else {
        "websocket_or_transport"
    }
}

/// Run `f()`. On CDP transport / unresponsive / liveness failure, clear session and retry once (unless [`BROWSER_INTENTIONAL_STOP`]).
fn with_connection_retry<F, T>(f: F) -> Result<T, String>
where
    F: Fn() -> Result<T, String>,
{
    match f() {
        Ok(v) => Ok(v),
        Err(e) => {
            if !should_retry_cdp_after_clearing_session(&e) {
                if e.contains("Browser unresponsive") {
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: no CDP reconnect retry after health failure (session already cleared; failing fast)"
                    );
                }
                return Err(e);
            }
            if BROWSER_INTENTIONAL_STOP.load(Ordering::SeqCst) {
                mac_stats_info!(
                    "browser/cdp",
                    "Browser agent [CDP]: CDP reconnect skipped (intentional browser stop)"
                );
                return Err(e);
            }
            let t0 = Instant::now();
            clear_browser_session_on_error(&e);
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: CDP reconnect attempt reason={} (session cleared, single retry)",
                cdp_retry_reason_tag(&e)
            );
            let r = f();
            let elapsed = t0.elapsed();
            match &r {
                Ok(_) => {
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: CDP reconnect succeeded in {:.2}s",
                        elapsed.as_secs_f64()
                    );
                }
                Err(e2) => {
                    mac_stats_warn!(
                        "browser/cdp",
                        "Browser agent [CDP]: CDP reconnect retry failed after {:.2}s: {}",
                        elapsed.as_secs_f64(),
                        truncate_compact(e2, 120)
                    );
                }
            }
            r
        }
    }
}

/// Get or create a browser; reuse if last use was within idle timeout and preference matches, else close and create new.
/// Only one thread can be in the "create new browser" path at a time (LAUNCH_MUTEX) to avoid multiple Chrome PIDs from races.
fn get_or_create_browser(port: u16) -> Result<Browser, String> {
    // New acquisition after `close_browser_session` must allow CDP reconnect-on-error again.
    BROWSER_INTENTIONAL_STOP.store(false, Ordering::SeqCst);
    if let Some(msg) = take_pending_renderer_crash_tool_error() {
        return Err(msg);
    }
    let timeout_secs = crate::config::Config::browser_idle_timeout_secs();
    let prefer_headless = PREFER_HEADLESS.load(Ordering::Relaxed);
    let custom_chromium = crate::config::Config::browser_chromium_executable_configured();
    let mut guard = browser_session().lock().map_err(|e| e.to_string())?;
    let now = Instant::now();
    if let Some((ref browser, created_at, last_used, was_headless)) = guard.as_ref() {
        if now.duration_since(*last_used).as_secs() < timeout_secs
            && *was_headless == prefer_headless
        {
            let b = browser.clone();
            let created_at_val = *created_at;
            *guard = Some((b.clone(), created_at_val, now, prefer_headless));
            record_session_created_this_turn(port, true, created_at_val);
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: reusing existing session (idle timeout {}s, headless={})",
                timeout_secs,
                prefer_headless
            );
            return Ok(b);
        }
        cookie_storage::try_save_cookies_from_browser(browser);
        if *was_headless != prefer_headless {
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: preference changed (headless {} → {}), creating new session",
                was_headless,
                prefer_headless
            );
        } else {
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: session idle > {}s, closing browser",
                timeout_secs
            );
        }
    }
    if guard.is_some() {
        cdp_target_crash_listener::invalidate_listener_generation();
    }
    if guard.is_some() {
        let ws = cdp_downloads::peek_cdp_ws_url();
        cdp_trace_archive::stop_and_persist_best_effort(
            ws.as_deref(),
            "browser idle timeout / session replacement",
        );
    }
    clear_owned_visible_chrome_child_pid();
    *guard = None;
    clear_cdp_js_dialog_history();
    drop(guard);

    if let Ok(mut set) = dialog_registered_tabs().lock() {
        set.clear();
    }

    // Serialize creation so only one thread launches at a time; kill orphaned headless Chrome before launching.
    let _launch_guard = launch_mutex().lock().map_err(|e| e.to_string())?;
    // Re-check session after acquiring launch lock (another thread may have created it).
    {
        let guard = browser_session().lock().map_err(|e| e.to_string())?;
        let now = Instant::now();
        if let Some((ref browser, created_at, last_used, was_headless)) = guard.as_ref() {
            if now.duration_since(*last_used).as_secs() < timeout_secs
                && *was_headless == prefer_headless
            {
                mac_stats_info!("browser/cdp",
                    "Browser agent [CDP]: reusing session after launch lock (another thread created it)"
                );
                record_session_created_this_turn(port, true, *created_at);
                return Ok(browser.clone());
            }
        }
    }

    if !prefer_headless {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: effective client timeouts for new visible session — HTTP /json/version={:?}, WebSocket connect={:?} (discovery host {})",
            cdp_discovery_http_timeout_duration(),
            Duration::from_secs(crate::config::Config::browser_cdp_ws_connect_timeout_secs()),
            CDP_DISCOVERY_HTTP_HOST,
        );
    }
    if prefer_headless {
        // Headless transport means we do not probe/connect CDP via `/json/version` and a WebSocket.
        // Clear CDP probe/attach markers so error context does not imply a WS existed.
        with_last_browser_health_mut(port, |h| {
            h.cdp_http_ok = None;
            h.cdp_http_err_summary = None;
            h.ws_ok = None;
            h.ws_err_summary = None;
        });
        kill_orphaned_browser_processes();
    }
    let (browser, stored_ws_url) = if prefer_headless {
        mac_stats_info!("browser/cdp",
            "Browser agent [CDP]: user requested headless — launching headless Chrome (no visible window)"
        );
        let b = launch_via_headless_chrome()?;
        let w = b.get_ws_url();
        let u = if w.contains("not running") {
            None
        } else {
            Some(w)
        };
        (b, u)
    } else if let Ok(ws_url) = try_discover_cdp_ws_url(port) {
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: connecting to Chromium on port {} (visible)",
            port
        );
        let w = ws_url.clone();
        let b = connect_browser_to_ws_url(&ws_url)?;
        (b, Some(w))
    } else {
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: nothing listening on CDP port {}, launching visible Chromium on {}",
            port,
            port
        );
        match launch_chrome_on_port(port) {
            Ok(child) => {
                let pid = child.id();
                match wait_for_cdp_http_after_visible_launch(port) {
                    Ok(()) => {
                        mac_stats_info!(
                            "browser/cdp",
                            "Browser agent [CDP]: connecting to Chromium on port {} (after launch, visible)",
                            port
                        );
                        let b = connect_cdp(port)?;
                        let u = get_ws_url(port).ok();
                        OWNED_VISIBLE_CHROME_CHILD_PID.store(pid, Ordering::SeqCst);
                        spawn_owned_visible_chrome_exit_watcher(child, pid);
                        (b, u)
                    }
                    Err(msg) => {
                        spawn_visible_chrome_child_reaper(child);
                        if custom_chromium {
                            return Err(format!(
                                "{} No fallback to another browser is attempted when browserChromiumExecutable is set.",
                                msg
                            ));
                        }
                        mac_stats_warn!(
                            "browser/cdp",
                            "Browser agent [CDP]: {} — falling back to headless_chrome launcher",
                            msg
                        );
                        let b = launch_via_headless_chrome()?;
                        let w = b.get_ws_url();
                        let u = if w.contains("not running") {
                            None
                        } else {
                            Some(w)
                        };
                        (b, u)
                    }
                }
            }
            Err(e) => {
                if custom_chromium {
                    return Err(format!(
                        "{} No fallback to another browser is attempted when browserChromiumExecutable is set.",
                        e
                    ));
                }
                mac_stats_info!(
                    "browser/cdp",
                    "Browser agent [CDP]: could not launch visible Chromium on port {} — using headless_chrome launcher ({})",
                    port,
                    truncate_compact(&e, 80)
                );
                let b = launch_via_headless_chrome()?;
                let w = b.get_ws_url();
                let u = if w.contains("not running") {
                    None
                } else {
                    Some(w)
                };
                (b, u)
            }
        }
    };
    cdp_downloads::store_cdp_ws_url(stored_ws_url.clone());
    let grants = crate::config::Config::browser_cdp_grant_permissions();
    if !grants.is_empty() {
        if let Some(ref u) = stored_ws_url {
            cdp_grant_permissions::apply_browser_grant_permissions_best_effort(u.as_str(), &grants);
        } else {
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: browserCdpGrantPermissions is set but no CDP WebSocket URL is tracked for this session; Browser.grantPermissions skipped"
            );
        }
    }
    *browser_session().lock().map_err(|e| e.to_string())? =
        Some((browser.clone(), now, now, prefer_headless));
    if let Some(ref u) = stored_ws_url {
        cdp_target_crash_listener::spawn_target_crash_side_listener(u);
    }
    cookie_storage::mark_cookie_restore_pending();
    record_session_created_this_turn(port, false, now);
    Ok(browser)
}

/// Normalize URL for display/filename (add https if no scheme).
fn normalize_url_for_screenshot(url: &str) -> String {
    let u = url.trim();
    if u.is_empty() {
        return "page".to_string();
    }
    if u.len() >= 7 && u[..7].eq_ignore_ascii_case("file://") {
        return u.to_string();
    }
    if !u.starts_with("http://") && !u.starts_with("https://") {
        format!("https://{}", u)
    } else {
        u.to_string()
    }
}

/// Ellipsis long URLs for debug logs (query strings can be noisy or sensitive).
fn ellipse_url_for_ssrf_debug_log(url: &str) -> String {
    const MAX: usize = 200;
    if url.chars().count() <= MAX {
        return url.to_string();
    }
    crate::logging::ellipse(url, MAX)
}

// ── CDP redirect-chain SSRF (Network.requestWillBeSent, type Document) ───────
//
// headless_chrome's `Tab::navigate_to` does not expose `Page.navigate`'s loaderId, so we cannot
// filter hops using the navigation handle alone. We correlate document `requestWillBeSent` events
// by shared `loader_id` and a first-hop URL match to the requested navigation URL. If correlation
// fails, intermediate hops are not validated on this stack beyond what CDP exposes; the mandatory
// post-navigate final URL SSRF check (`assert_final_document_url_ssrf_post_check`) remains.

const CDP_RWS_REDIRECT_CHAIN_BUF_CAP: usize = 128;

type CdpRwsRedirectEntry = (String, String);

/// Best-effort `Network.enable` so redirect hops appear in `Network.requestWillBeSent`.
fn cdp_enable_network_for_redirect_chain_capture(tab: &headless_chrome::Tab) {
    if let Err(e) = tab.call_method(Network::Enable {
        max_total_buffer_size: None,
        max_resource_buffer_size: None,
        max_post_data_size: None,
        report_direct_socket_traffic: None,
        enable_durable_messages: None,
    }) {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: Network.enable (redirect-chain SSRF) failed: {} — final-URL SSRF only for hop chain",
            e
        );
    }
}

fn cdp_attach_redirect_chain_rws_listener(
    tab: &headless_chrome::Tab,
    buf: Arc<Mutex<VecDeque<CdpRwsRedirectEntry>>>,
) -> Option<std::sync::Weak<dyn headless_chrome::browser::tab::EventListener<Event> + Send + Sync>>
{
    let b = Arc::clone(&buf);
    let listener = Arc::new(move |event: &Event| {
        let Event::NetworkRequestWillBeSent(ev) = event else {
            return;
        };
        let p = &ev.params;
        if !matches!(p.Type, Some(Network::ResourceType::Document)) {
            return;
        }
        let lid = p.loader_id.clone();
        let u = p.request.url.clone();
        if let Ok(mut g) = b.lock() {
            while g.len() >= CDP_RWS_REDIRECT_CHAIN_BUF_CAP {
                g.pop_front();
            }
            g.push_back((lid, u));
        }
    });
    match tab.add_event_listener(listener) {
        Ok(w) => Some(w),
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: redirect-chain RWS listener attach failed: {} — final-URL SSRF only for hop chain",
                e
            );
            None
        }
    }
}

struct CdpRedirectRwsListenerGuard<'a> {
    tab: &'a headless_chrome::Tab,
    weak: Option<
        std::sync::Weak<dyn headless_chrome::browser::tab::EventListener<Event> + Send + Sync>,
    >,
}

impl Drop for CdpRedirectRwsListenerGuard<'_> {
    fn drop(&mut self) {
        if let Some(w) = self.weak.take() {
            let _ = self.tab.remove_event_listener(&w);
        }
    }
}

/// Tracks `Network.requestWillBeSent` / `loadingFinished` / `loadingFailed` for optional post-navigate idle wait.
fn cdp_attach_network_in_flight_listener(
    tab: &headless_chrome::Tab,
    in_flight: Arc<Mutex<HashSet<String>>>,
) -> Option<std::sync::Weak<dyn headless_chrome::browser::tab::EventListener<Event> + Send + Sync>>
{
    let b = Arc::clone(&in_flight);
    let listener = Arc::new(move |event: &Event| match event {
        Event::NetworkRequestWillBeSent(ev) => {
            let id = ev.params.request_id.clone();
            if let Ok(mut g) = b.lock() {
                g.insert(id);
            }
        }
        Event::NetworkLoadingFinished(ev) => {
            let id = ev.params.request_id.clone();
            if let Ok(mut g) = b.lock() {
                g.remove(&id);
            }
        }
        Event::NetworkLoadingFailed(ev) => {
            let id = ev.params.request_id.clone();
            if let Ok(mut g) = b.lock() {
                g.remove(&id);
            }
        }
        _ => {}
    });
    match tab.add_event_listener(listener) {
        Ok(w) => Some(w),
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: Network in-flight listener attach failed: {} — skipping configurable network-idle wait",
                e
            );
            None
        }
    }
}

struct CdpNetworkIdleInFlightGuard<'a> {
    tab: &'a headless_chrome::Tab,
    weak: Option<
        std::sync::Weak<dyn headless_chrome::browser::tab::EventListener<Event> + Send + Sync>,
    >,
}

impl Drop for CdpNetworkIdleInFlightGuard<'_> {
    fn drop(&mut self) {
        if let Some(w) = self.weak.take() {
            let _ = self.tab.remove_event_listener(&w);
        }
    }
}

/// When network-idle stabilization is enabled, attach **before** `navigate_to` / reload / history
/// so `requestWillBeSent` events are not missed. Guard removes the listener when dropped.
fn prepare_post_nav_network_idle_tracking(
    tab: &headless_chrome::Tab,
) -> (
    Option<Arc<Mutex<HashSet<String>>>>,
    CdpNetworkIdleInFlightGuard<'_>,
) {
    if !crate::config::Config::browser_post_navigate_network_idle_enabled() {
        return (None, CdpNetworkIdleInFlightGuard { tab, weak: None });
    }
    cdp_enable_network_for_redirect_chain_capture(tab);
    let flight = Arc::new(Mutex::new(HashSet::new()));
    let weak = cdp_attach_network_in_flight_listener(tab, Arc::clone(&flight));
    let flight_opt = if weak.is_some() { Some(flight) } else { None };
    (flight_opt, CdpNetworkIdleInFlightGuard { tab, weak })
}

/// After `wait_until_navigated` (and any non-timeout SPA/hash fallback sleep), apply configurable
/// minimum dwell and optional network in-flight quiet wait (browser-use style).
fn apply_configured_post_nav_stabilization(
    in_flight: Option<&Arc<Mutex<HashSet<String>>>>,
    context: &'static str,
) {
    let min_s = crate::config::Config::browser_post_navigate_min_dwell_secs();
    let min_dwell = Duration::from_secs_f64(min_s);
    if !min_dwell.is_zero() {
        let t0 = Instant::now();
        std::thread::sleep(min_dwell);
        let slept_ms = t0.elapsed().as_millis();
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: post_nav_stabilization {} min_dwell_applied_ms≈{} (configured {:.3}s)",
            context,
            slept_ms,
            min_s
        );
    }

    if !crate::config::Config::browser_post_navigate_network_idle_enabled() {
        return;
    }
    let max_extra_s = crate::config::Config::browser_post_navigate_network_idle_max_extra_secs();
    if max_extra_s <= 0.0 || !max_extra_s.is_finite() {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: post_nav_stabilization {} network_idle skipped (max_extra<=0)",
            context
        );
        return;
    }
    let Some(flight_arc) = in_flight else {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: post_nav_stabilization {} network_idle skipped (no in-flight tracker — listener attach failed?)",
            context
        );
        return;
    };

    let quiet = Duration::from_secs_f64(
        crate::config::Config::browser_post_navigate_network_idle_quiet_secs(),
    );
    let max_extra = Duration::from_secs_f64(max_extra_s);
    let deadline = Instant::now() + max_extra;
    let wait_phase_start = Instant::now();
    let mut quiet_start: Option<Instant> = None;

    loop {
        if Instant::now() >= deadline {
            let inflight_n = flight_arc.lock().map(|g| g.len()).unwrap_or(0);
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: post_nav_stabilization {} network_idle hit max_extra_secs cap (≈{:.3}s) in_flight_remaining≈{}",
                context,
                max_extra_s,
                inflight_n
            );
            break;
        }

        let inflight_n = flight_arc.lock().map(|g| g.len()).unwrap_or(0);
        if inflight_n == 0 {
            match quiet_start {
                None => quiet_start = Some(Instant::now()),
                Some(qs) => {
                    if qs.elapsed() >= quiet {
                        let extra_ms = wait_phase_start.elapsed().as_millis();
                        mac_stats_debug!(
                            "browser/cdp",
                            "Browser agent [CDP]: post_nav_stabilization {} network_idle_quiet_met extra_wait_ms≈{} quiet_window_s≈{:.3} in_flight_was_empty_for≈{:.3}s",
                            context,
                            extra_ms,
                            quiet.as_secs_f64(),
                            qs.elapsed().as_secs_f64()
                        );
                        break;
                    }
                }
            }
        } else {
            quiet_start = None;
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

/// True when `first_hop` is the same navigation target as `requested` (scheme may differ for http↔https).
fn cdp_redirect_chain_first_hop_matches_request(first_hop: &str, requested: &str) -> bool {
    let t1 = first_hop.trim();
    let t2 = requested.trim();
    if t1 == t2 {
        return true;
    }
    let Ok(u1) = Url::parse(t1) else {
        return false;
    };
    let Ok(u2) = Url::parse(t2) else {
        return false;
    };
    let h1 = u1.host_str().unwrap_or("").to_ascii_lowercase();
    let h2 = u2.host_str().unwrap_or("").to_ascii_lowercase();
    if h1 != h2 {
        return false;
    }
    let s1 = u1.scheme().to_ascii_lowercase();
    let s2 = u2.scheme().to_ascii_lowercase();
    let p1 = u1.port_or_known_default();
    let p2 = u2.port_or_known_default();
    let ports_align = p1 == p2
        || (matches!(s1.as_str(), "http" | "https") && matches!(s2.as_str(), "http" | "https")
            && matches!(
                (p1, p2),
                (Some(80), Some(443)) | (Some(443), Some(80))
            ));
    if !ports_align {
        return false;
    }
    let p1 = if u1.path().is_empty() { "/" } else { u1.path() };
    let p2 = if u2.path().is_empty() { "/" } else { u2.path() };
    p1 == p2
}

/// Longest document redirect chain in buffer whose first hop matches the requested URL (same loader_id run).
fn cdp_extract_document_redirect_chain_from_rws_buffer(
    buf: &[(String, String)],
    requested_url: &str,
) -> Option<Vec<String>> {
    let mut best: Option<Vec<String>> = None;
    let mut i = 0;
    while i < buf.len() {
        if !cdp_redirect_chain_first_hop_matches_request(&buf[i].1, requested_url) {
            i += 1;
            continue;
        }
        let loader = &buf[i].0;
        let mut chain = vec![buf[i].1.clone()];
        let mut j = i + 1;
        while j < buf.len() && buf[j].0 == *loader {
            chain.push(buf[j].1.clone());
            j += 1;
        }
        let take = best.as_ref().map(|b| chain.len() > b.len()).unwrap_or(true);
        if take {
            best = Some(chain);
        }
        i = j;
    }
    best
}

fn ordered_distinct_redirect_hop_urls(urls: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for u in urls {
        let t = u.trim().to_string();
        if t.is_empty() {
            continue;
        }
        if seen.insert(t.clone()) {
            out.push(t);
        }
    }
    out
}

/// Run [`crate::commands::browser::validate_url_no_ssrf`] on each HTTP(S) hop (same rules / vocabulary as FETCH_URL).
fn validate_http_redirect_hops_ssrf(hops: &[String]) -> Result<(), String> {
    let allowed = crate::config::Config::ssrf_allowed_hosts();
    for url_str in hops {
        let t = url_str.trim();
        if t.is_empty() {
            continue;
        }
        let Ok(parsed) = Url::parse(t) else {
            return Err(format!(
                "Navigation redirect chain included a URL that could not be parsed (SSRF guard, same rules as FETCH_URL): {}",
                ellipse_url_for_ssrf_debug_log(t)
            ));
        };
        match parsed.scheme().to_ascii_lowercase().as_str() {
            "http" | "https" => {
                crate::commands::browser::validate_url_no_ssrf(&parsed, &allowed).map_err(|e| {
                    mac_stats_warn!(
                        "browser/security",
                        "Browser agent: redirect-chain SSRF block url={} detail={}",
                        ellipse_url_for_ssrf_debug_log(t),
                        e
                    );
                    format!(
                        "A redirect hop in the navigation was blocked to prevent SSRF (blocked host, allowlist bypass, or DNS failure — same rules as FETCH_URL): {}",
                        e
                    )
                })?;
            }
            other => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: redirect-chain hop skipped SSRF IP check (non-http scheme): {}",
                    other
                );
            }
        }
    }
    Ok(())
}

fn cdp_validate_redirect_chain_from_rws_buffer(
    buf: &Mutex<VecDeque<CdpRwsRedirectEntry>>,
    requested_url: &str,
) -> Result<(), String> {
    let snapshot: Vec<(String, String)> = buf
        .lock()
        .map(|q| q.iter().cloned().collect())
        .unwrap_or_default();
    let Some(chain) = cdp_extract_document_redirect_chain_from_rws_buffer(&snapshot, requested_url)
    else {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: could not correlate CDP document redirect hops to this navigation (intermediate SSRF may be missed); final-URL SSRF check still applies"
        );
        return Ok(());
    };
    let distinct = ordered_distinct_redirect_hop_urls(chain);
    let hosts: Vec<String> = distinct
        .iter()
        .filter_map(|u| {
            Url::parse(u)
                .ok()
                .and_then(|p| p.host_str().map(|h| h.to_ascii_lowercase()))
        })
        .collect();
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: redirect-chain SSRF distinct_hops={} hosts_redacted=[{}]",
        distinct.len(),
        hosts.join(", ")
    );
    validate_http_redirect_hops_ssrf(&distinct)
}

/// Post-CDP navigation SSRF guard on the **effective** document URL (after HTTP redirects, SPA navigations, etc.).
///
/// - **http / https:** Same resolution-based check as pre-navigation — [`crate::commands::browser::validate_url_no_ssrf`]
///   with `ssrfAllowedHosts` / config allowlist.
/// - **Explicitly allowed non-network documents:** `about:blank` and `about:srcdoc` only (including trivial fragments).
/// - **`file:`** — allowed only when it passes the same local `.html`/`.htm` checks as pre-navigation
///   ([`crate::commands::browser::validate_file_url_for_browser_navigation`]).
///   Do not extend casually to `javascript:` or other opaque schemes.
/// - **Browser-internal (no IP resolution):** `chrome-error:`, `devtools:`, `chrome:`, `edge:`, `brave:` — same class as
///   OpenClaw’s post-navigate handling; the LLM snapshot already adds hints for chrome-error / new-tab pages.
/// - **blob:** / **data:** — skipped here (no DNS); `javascript:` is rejected.
fn assert_final_document_url_ssrf_post_check(
    final_url: &str,
    requested_url_for_log: Option<&str>,
) -> Result<(), String> {
    let final_trim = final_url.trim();
    if final_trim.is_empty() {
        return Ok(());
    }
    let Ok(parsed) = Url::parse(final_trim) else {
        return Err(format!(
            "After navigation, the tab URL could not be parsed (SSRF guard): {}",
            ellipse_url_for_ssrf_debug_log(final_trim)
        ));
    };
    let scheme = parsed.scheme().to_ascii_lowercase();
    match scheme.as_str() {
        "http" | "https" => {
            if let Some(req) = requested_url_for_log
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: post-navigate SSRF recheck requested_url={} final_url={}",
                    ellipse_url_for_ssrf_debug_log(req),
                    ellipse_url_for_ssrf_debug_log(final_trim),
                );
            } else {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: post-navigate SSRF recheck final_url={}",
                    ellipse_url_for_ssrf_debug_log(final_trim),
                );
            }
            let allowed = crate::config::Config::ssrf_allowed_hosts();
            crate::commands::browser::validate_url_no_ssrf(&parsed, &allowed).map_err(|e| {
                mac_stats_warn!(
                    "browser/security",
                    "Browser agent: post-navigate SSRF block final_url={} detail={}",
                    ellipse_url_for_ssrf_debug_log(final_trim),
                    e
                );
                format!(
                    "After navigation, the active page URL was blocked to prevent SSRF (a redirect or in-page navigation may have changed the target): {}",
                    e
                )
            })
        }
        "about" => {
            if is_allowed_about_document_for_ssrf(final_trim, &parsed) {
                Ok(())
            } else {
                Err(format!(
                    "After navigation, `about:` URL is not in the allowed set (only about:blank and about:srcdoc): {}",
                    ellipse_url_for_ssrf_debug_log(final_trim)
                ))
            }
        }
        "chrome-error" | "devtools" | "chrome" | "edge" | "brave" => Ok(()),
        "blob" | "data" => Ok(()),
        "file" => crate::commands::browser::validate_file_url_for_browser_navigation(&parsed)
            .map(|_| ())
            .map_err(|e| {
                format!(
                    "After navigation, the tab `file:` URL failed local safety checks: {}",
                    e
                )
            }),
        "javascript" => Err(format!(
            "After navigation, the tab URL uses disallowed scheme `{}:` (SSRF guard)",
            scheme
        )),
        _ => Ok(()),
    }
}

/// Allowed `about:` documents for post-nav SSRF (no IP check). Keep in sync with OpenClaw-style explicit allow-sets.
fn is_allowed_about_document_for_ssrf(raw: &str, parsed: &Url) -> bool {
    let tl = raw.trim().to_ascii_lowercase();
    if tl == "about:blank"
        || tl.starts_with("about:blank#")
        || tl.starts_with("about:blank?")
        || tl == "about:srcdoc"
        || tl.starts_with("about:srcdoc#")
        || tl.starts_with("about:srcdoc?")
    {
        return true;
    }
    if !parsed.scheme().eq_ignore_ascii_case("about") {
        return false;
    }
    let p = parsed.path().to_ascii_lowercase();
    p == "blank" || p == "srcdoc"
}

/// After navigation (redirect, click, history), if the tab landed on a blocked http(s) URL, reset to `about:blank`.
/// Returns a prefix line for the tool result when a reset was performed.
fn enforce_tab_url_navigation_policy(
    tab: &headless_chrome::browser::tab::Tab,
    context: &str,
) -> Result<Option<String>, String> {
    let url = tab.get_url();
    let url_str = url.as_str();
    if !url_filter::url_scheme_subject_to_domain_policy(url_str) {
        return Ok(None);
    }
    if url_filter::is_navigation_allowed(url_str) {
        return Ok(None);
    }
    let host = url_filter::host_label_for_policy_message(url_str);
    mac_stats_warn!(
        "browser/security",
        "Browser security watchdog: after {}, URL blocked (host={}); resetting tab to about:blank",
        context,
        host
    );
    tab.navigate_to("about:blank").map_err(|e| {
        format!(
            "Page landed on a URL blocked by security policy (host: {}). Failed to reset tab to about:blank: {}",
            host, e
        )
    })?;
    std::thread::sleep(BROWSER_POLICY_RESET_SETTLE);
    let _ = tab.wait_until_navigated();
    Ok(Some(format!(
        "**Security warning:** After {}, the active tab URL matched a blocked host (`{}`). The tab was reset to `about:blank`.\n\n",
        context, host
    )))
}

/// Returns true if both URLs are valid HTTP(S) and have the same host (domain). Used to apply a shorter wait timeout for same-domain navigations. Invalid or non-http(s) URLs return false.
fn is_same_domain(current_url: &str, target_url: &str) -> bool {
    let current = match Url::parse(current_url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    let target = match Url::parse(target_url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    let scheme_ok = |u: &Url| u.scheme() == "http" || u.scheme() == "https";
    if !scheme_ok(&current) || !scheme_ok(&target) {
        return false;
    }
    let host_current = match current.host_str() {
        Some(h) => h.to_lowercase(),
        None => return false,
    };
    let host_target = match target.host_str() {
        Some(h) => h.to_lowercase(),
        None => return false,
    };
    host_current == host_target
}

/// headless_chrome enables lifecycle events on tab attach; repeat so reattached / edge sessions still emit `Page.lifecycleEvent`.
fn cdp_ensure_lifecycle_events_enabled(tab: &headless_chrome::Tab) {
    let _ = tab.call_method(Page::SetLifecycleEventsEnabled { enabled: true });
}

fn resource_timing_resource_count(tab: &headless_chrome::Tab) -> Option<u32> {
    let res = tab
        .evaluate(
            "(function(){ try { return performance.getEntriesByType('resource').length; } catch(e) { return -1; } })()",
            false,
        )
        .ok()?;
    let n = res.value?.as_f64()? as i64;
    if n < 0 {
        None
    } else {
        Some(n as u32)
    }
}

/// After CDP navigation starts (`Page.navigate`, reload, or history), wait for network-idle lifecycle signals (adaptive window), then align with headless_chrome's internal navigation flag.
///
/// `post_nav_network_in_flight`: when `Some`, caller must have attached the listener **before**
/// starting navigation (see [`prepare_post_nav_network_idle_tracking`]).
fn synchronize_tab_after_cdp_navigation(
    tab: &headless_chrome::Tab,
    prev_url: &str,
    target_url: &str,
    lifecycle_buf: Option<&Arc<Mutex<VecDeque<String>>>>,
    nav_started_at: Instant,
    overall_timeout: Duration,
    timeout_secs_label: u64,
    on_nav_wait_timeout_record_hint_for: Option<&str>,
    post_nav_network_in_flight: Option<&Arc<Mutex<HashSet<String>>>>,
) -> Result<(), String> {
    let overall_deadline = nav_started_at
        .checked_add(overall_timeout)
        .unwrap_or_else(Instant::now);
    let idle_cap = if is_same_domain(prev_url, target_url) {
        BROWSER_NAV_NETWORK_IDLE_TIMEOUT_SAME_DOMAIN
    } else {
        BROWSER_NAV_NETWORK_IDLE_TIMEOUT_CROSS_DOMAIN
    };
    let idle_deadline = nav_started_at
        .checked_add(idle_cap)
        .map(|t| t.min(overall_deadline))
        .unwrap_or(overall_deadline);

    if let Some(buf) = lifecycle_buf {
        let mut saw_network_idle_class = false;
        while Instant::now() < idle_deadline {
            let seen = buf
                .lock()
                .map(|q| {
                    q.iter()
                        .any(|n| n == "networkIdle" || n == "networkAlmostIdle")
                })
                .unwrap_or(false);
            if seen {
                saw_network_idle_class = true;
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: Page.lifecycleEvent network idle class (networkIdle or networkAlmostIdle) same_domain_hint={}",
                    is_same_domain(prev_url, target_url)
                );
                break;
            }
            std::thread::sleep(BROWSER_NAV_LIFECYCLE_POLL);
        }
        if !saw_network_idle_class {
            let rs = tab
                .evaluate("document.readyState", false)
                .ok()
                .and_then(|r| r.value.as_ref().and_then(|v| v.as_str().map(String::from)))
                .unwrap_or_else(|| "(unknown)".to_string());
            if rs == "complete" {
                mac_stats_warn!(
                    "browser/cdp",
                    "Browser agent [CDP]: no networkIdle/networkAlmostIdle within {:?}; document.readyState=complete — proceeding",
                    idle_cap
                );
            } else {
                mac_stats_warn!(
                    "browser/cdp",
                    "Browser agent [CDP]: no networkIdle/networkAlmostIdle within {:?}; readyState={} — proceeding",
                    idle_cap,
                    rs
                );
            }
        }
    }

    let rem = overall_deadline.saturating_duration_since(Instant::now());
    if rem.is_zero() {
        if let Some(req) = on_nav_wait_timeout_record_hint_for {
            record_nav_timeout_url_changed_hint(req, tab);
        }
        return Err(navigation_timeout_error_with_proxy_hint(timeout_secs_label));
    }
    tab.set_default_timeout(rem.max(Duration::from_millis(200)));
    match tab.wait_until_navigated() {
        Ok(_) => {}
        Err(e) => {
            let err_str = e.to_string();
            if err_str.to_lowercase().contains("timeout") {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: navigation wait timed out after {}s",
                    timeout_secs_label
                );
                if let Some(req) = on_nav_wait_timeout_record_hint_for {
                    record_nav_timeout_url_changed_hint(req, tab);
                }
                return Err(navigation_timeout_error_with_proxy_hint(timeout_secs_label));
            }
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: wait_until_navigated failed (SPA/hash?): {} — continuing after delay",
                e
            );
            std::thread::sleep(BROWSER_WAIT_UNTIL_NAVIGATED_FALLBACK);
        }
    }
    apply_configured_post_nav_stabilization(post_nav_network_in_flight, "cdp_navigation_sync");
    Ok(())
}

fn with_lifecycle_event_buffer<R>(
    tab: &headless_chrome::Tab,
    run: impl FnOnce(Option<&Arc<Mutex<VecDeque<String>>>>) -> Result<R, String>,
) -> Result<R, String> {
    cdp_ensure_lifecycle_events_enabled(tab);
    let buf = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let bc = Arc::clone(&buf);
    let listener = Arc::new(move |event: &Event| {
        if let Event::PageLifecycleEvent(ev) = event {
            let n = ev.params.name.to_string();
            if let Ok(mut q) = bc.lock() {
                q.push_back(n);
                while q.len() > 80 {
                    q.pop_front();
                }
            }
        }
    });
    match tab.add_event_listener(listener) {
        Ok(wk) => {
            let r = run(Some(&buf));
            let _ = tab.remove_event_listener(&wk);
            r
        }
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: lifecycle listener attach failed: {} — using wait_until_navigated only",
                e
            );
            run(None)
        }
    }
}

fn cdp_debug_port() -> u16 {
    crate::config::Config::browser_cdp_port()
}

fn current_tab_index() -> &'static Mutex<usize> {
    CURRENT_TAB_INDEX.get_or_init(|| Mutex::new(0))
}

fn record_active_automation_target(tab: &Arc<headless_chrome::Tab>) {
    if let Ok(mut g) = active_tab_target_id_store().lock() {
        *g = Some(tab.get_target_id().clone());
    }
    if let Ok(mut h) = active_automation_tab_url_host_store().lock() {
        *h = Some(host_for_navigation_log(&tab.get_url()));
    }
}

fn stop_download_aux_listener(holder: &Arc<Mutex<Option<Arc<AtomicBool>>>>) {
    if let Ok(mut g) = holder.lock() {
        if let Some(st) = g.take() {
            st.store(true, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(450));
        }
    }
}

/// Page + Browser-domain hints so Chrome writes downloads under `~/.mac-stats/browser-downloads/`.
fn ensure_chrome_download_artifacts_on_tab(tab: &headless_chrome::Tab) -> Result<(), String> {
    let dir = crate::config::Config::browser_downloads_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("browser downloads dir: {}", e))?;
    let path_str = dir.to_string_lossy().to_string();
    if let Err(e) = tab.call_method(Page::SetDownloadBehavior {
        behavior: Page::SetDownloadBehaviorBehaviorOption::Allow,
        download_path: Some(path_str),
    }) {
        mac_stats_debug!(
            "browser/cdp",
            "Page.setDownloadBehavior: {} (continuing)",
            e
        );
    }
    if let Some(ws) = cdp_downloads::peek_cdp_ws_url() {
        cdp_downloads::apply_browser_download_behavior_best_effort(&ws, &dir);
    }
    Ok(())
}

fn url_looks_like_http_pdf(url: &str) -> bool {
    let t = url.trim();
    let tl = t.to_ascii_lowercase();
    let base = tl
        .split('?')
        .next()
        .unwrap_or("")
        .split('#')
        .next()
        .unwrap_or("");
    (tl.starts_with("http://") || tl.starts_with("https://")) && base.ends_with(".pdf")
}

/// CDP `Page.printToPDF` → raw PDF bytes; enforces [`artifact_limits::ensure_buffer_within_browser_artifact_cap`].
fn print_tab_to_pdf_bytes(
    tab: &headless_chrome::Tab,
    print_background: bool,
    artifact_kind: &str,
) -> Result<Vec<u8>, String> {
    use base64::Engine;
    let pdf = tab
        .call_method(Page::PrintToPDF {
            landscape: None,
            display_header_footer: None,
            print_background: Some(print_background),
            scale: None,
            paper_width: None,
            paper_height: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            page_ranges: None,
            header_template: None,
            footer_template: None,
            prefer_css_page_size: None,
            transfer_mode: None,
            generate_tagged_pdf: None,
            generate_document_outline: None,
        })
        .map_err(|e| format!("Page.printToPDF: {}", e))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(pdf.data.trim())
        .map_err(|e| format!("Page.printToPDF base64 decode: {}", e))?;
    if bytes.is_empty() {
        return Err("Page.printToPDF returned empty PDF".to_string());
    }
    artifact_limits::ensure_buffer_within_browser_artifact_cap(bytes.len(), artifact_kind)?;
    Ok(bytes)
}

fn try_save_printed_pdf(tab: &headless_chrome::Tab, download_dir: &Path) -> Option<PathBuf> {
    let bytes = print_tab_to_pdf_bytes(tab, true, "printToPDF (navigate .pdf tab)").ok()?;
    let name = format!(
        "{}_nav_print.pdf",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let path = artifact_atomic::write_bytes_atomic_same_dir(download_dir, &name, &bytes).ok()?;
    mac_stats_info!("browser/cdp", "Page.printToPDF saved {}", path.display());
    Some(path)
}

/// Prefer the tab matching the last recorded CDP `TargetID` when the focused index drifted (e.g. user switched tabs in Chrome).
fn reconcile_active_automation_tab(
    browser: &Browser,
    tab: Arc<headless_chrome::Tab>,
) -> Result<Arc<headless_chrome::Tab>, String> {
    let cur = tab.get_target_id().clone();
    let stored = match active_tab_target_id_store().lock() {
        Ok(g) => g.clone(),
        Err(_) => return Ok(tab),
    };
    match stored {
        None => Ok(tab),
        Some(ref s) if s == &cur => Ok(tab),
        Some(stored) => {
            let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
            if let Some((idx, t)) = tabs
                .iter()
                .enumerate()
                .find(|(_, x)| x.get_target_id().as_str() == stored.as_str())
            {
                let out = t.clone();
                drop(tabs);
                if let Ok(mut g) = current_tab_index().lock() {
                    if *g != idx {
                        mac_stats_warn!(
                            "browser/cdp",
                            "Browser agent [CDP]: current_tab_index pointed at a different page than stored target_id — refocusing tab index {} (automation target)",
                            idx
                        );
                    }
                    *g = idx;
                }
                out.bring_to_front()
                    .map_err(|e| format!("bring_to_front (target reconciliation): {}", e))?;
                return Ok(out);
            }
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: stored automation target_id no longer exists among open tabs; adopting current tab"
            );
            if let Ok(mut g) = active_tab_target_id_store().lock() {
                *g = Some(cur);
            }
            Ok(tab)
        }
    }
}

/// Strip fragment and trim for comparing "same page" across tabs (OpenClaw-style URL alignment).
fn canonical_url_for_tab_focus_match(url: &str) -> String {
    let t = url.trim();
    match Url::parse(t) {
        Ok(mut u) => {
            u.set_fragment(None);
            u.to_string()
        }
        Err(_) => t.to_string(),
    }
}

const MAX_TAB_URLS_IN_FOCUS_LOST_ERROR: usize = 8;
const MAX_TAB_URL_CHARS_IN_FOCUS_LOST_ERROR: usize = 120;

fn truncate_url_for_focus_error(s: &str) -> String {
    let t = s.trim();
    let count = t.chars().count();
    if count <= MAX_TAB_URL_CHARS_IN_FOCUS_LOST_ERROR {
        t.to_string()
    } else {
        format!(
            "{}…",
            t.chars()
                .take(MAX_TAB_URL_CHARS_IN_FOCUS_LOST_ERROR.saturating_sub(1))
                .collect::<String>()
        )
    }
}

/// OpenClaw-style post-navigate `TargetID` / tab-list reconciliation: after `Page.navigate`, history moves, or reload,
/// re-enumerate CDP tabs and align [`CURRENT_TAB_INDEX`] with the page we navigated so the automation `Vec` index
/// does not drift from Chromium’s ordering. **Do not remove** during refactors without restoring equivalent behaviour
/// for multi-tab sessions (see OpenClaw `resolveTargetIdAfterNavigate`).
///
/// Runs **before** [`get_browser_state`], cookie-banner handling, and screenshot prep on those paths.
fn reconcile_post_navigate_tab_focus(
    browser: &Browser,
    navigated_tab: &Arc<headless_chrome::Tab>,
    pre_navigate_target_id: &str,
    final_url: &str,
) -> Result<Arc<headless_chrome::Tab>, String> {
    let final_canon = canonical_url_for_tab_focus_match(final_url);
    let old_idx = current_tab_index()
        .lock()
        .map_err(|e| e.to_string())
        .map(|g| *g)?;

    let tabs_guard = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    if let Some((idx, t)) = tabs_guard
        .iter()
        .enumerate()
        .find(|(_, t)| t.get_target_id().as_str() == pre_navigate_target_id)
    {
        let t = t.clone();
        drop(tabs_guard);
        if let Ok(mut g) = current_tab_index().lock() {
            if *g != idx {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: post-navigate tab reconciliation old_index={} new_index={} reason=navigate_target_id_sync",
                    *g,
                    idx
                );
                *g = idx;
            }
        }
        let _ = t.bring_to_front();
        record_active_automation_target(&t);
        return Ok(t);
    }
    drop(tabs_guard);

    // `pre_navigate_target_id` no longer appears in CDP tab list — recover by URL or safe fallback.
    let tabs_guard = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    let n = tabs_guard.len();
    let mut matching: Vec<(usize, Arc<headless_chrome::Tab>)> = Vec::new();
    for (i, t) in tabs_guard.iter().enumerate() {
        let u = t.get_url();
        if canonical_url_for_tab_focus_match(u.as_str()) == final_canon {
            matching.push((i, t.clone()));
        }
    }
    drop(tabs_guard);

    if n == 0 {
        return Err(
            "Browser has no open tabs after navigation; use BROWSER_NAVIGATE to open a page."
                .to_string(),
        );
    }

    if n == 1 {
        let tabs_guard = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        let t = tabs_guard
            .get(0)
            .cloned()
            .ok_or_else(|| "No tab in browser".to_string())?;
        drop(tabs_guard);
        if let Ok(mut g) = current_tab_index().lock() {
            if *g != 0 {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: post-navigate tab reconciliation old_index={} new_index={} reason=single_tab_fallback",
                    *g,
                    0
                );
                *g = 0;
            }
        }
        let _ = t.bring_to_front();
        record_active_automation_target(&t);
        return Ok(t);
    }

    if matching.len() == 1 {
        let (idx, t) = matching.pop().expect("matching.len() == 1");
        if let Ok(mut g) = current_tab_index().lock() {
            if *g != idx {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: post-navigate tab reconciliation old_index={} new_index={} reason=unique_url_match",
                    *g,
                    idx
                );
                *g = idx;
            }
        }
        let _ = t.bring_to_front();
        record_active_automation_target(&t);
        return Ok(t);
    }

    if matching.len() > 1 {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: post-navigate tab reconciliation ambiguous same-URL: {} tab(s) match final_url={}; keeping prior index={} (navigate target id missing from CDP list)",
            matching.len(),
            truncate_url_for_focus_error(final_url),
            old_idx
        );
        return Ok(navigated_tab.clone());
    }

    let tabs_guard = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    let mut candidates: Vec<String> = Vec::new();
    for t in tabs_guard.iter().take(MAX_TAB_URLS_IN_FOCUS_LOST_ERROR) {
        candidates.push(truncate_url_for_focus_error(t.get_url().as_str()));
    }
    drop(tabs_guard);
    Err(format!(
        "Browser session lost the focused tab (CDP target no longer listed among open tabs). {} open tab(s). Sample URLs: {}. Use BROWSER_NAVIGATE or BROWSER_SWITCH_TAB to pick a tab.",
        n,
        candidates.join("; ")
    ))
}

/// CDP `Emulation.setDeviceMetricsOverride` / `setGeolocationOverride` (or clears) per current config.
/// Call after attach and before navigation on a tab; also before screenshot/extract paths that use [`get_current_tab`].
/// HTTP fetch fallback is unaffected. Overrides apply to **all** CDP automation for this process until the session is replaced or config changes on the next apply.
fn apply_cdp_emulation_to_tab(tab: &headless_chrome::Tab) {
    let dims = crate::config::Config::browser_cdp_emulate_viewport_dimensions();
    if let Some((w, h)) = dims {
        let dsf = crate::config::Config::browser_cdp_emulate_device_scale_factor();
        let mobile = crate::config::Config::browser_cdp_emulate_mobile();
        match tab.call_method(Emulation::SetDeviceMetricsOverride {
            width: w,
            height: h,
            device_scale_factor: dsf,
            mobile,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        }) {
            Ok(_) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: Emulation.setDeviceMetricsOverride {}x{} dsf={:.3} mobile={}",
                    w,
                    h,
                    dsf,
                    mobile
                );
            }
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: Emulation.setDeviceMetricsOverride failed: {} (continuing)",
                    e
                );
            }
        }
    } else if let Err(e) = tab.call_method(Emulation::ClearDeviceMetricsOverride(None)) {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: Emulation.clearDeviceMetricsOverride failed: {} (continuing)",
            e
        );
    }

    if let Some((lat, lon, accuracy)) = crate::config::Config::browser_cdp_emulate_geolocation() {
        match tab.call_method(Emulation::SetGeolocationOverride {
            latitude: Some(lat),
            longitude: Some(lon),
            accuracy,
            altitude: None,
            altitude_accuracy: None,
            heading: None,
            speed: None,
        }) {
            Ok(_) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: Emulation.setGeolocationOverride lat={:.5} lon={:.5} accuracy={:?}",
                    lat,
                    lon,
                    accuracy
                );
            }
            Err(e) => {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: Emulation.setGeolocationOverride failed: {} (continuing)",
                    e
                );
            }
        }
    } else if let Err(e) = tab.call_method(Emulation::ClearGeolocationOverride(None)) {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: Emulation.clearGeolocationOverride failed: {} (continuing)",
            e
        );
    }
}

/// When [`CURRENT_TAB_INDEX`] points at `about:blank` or a Chrome new-tab surface but another tab has navigable content,
/// refocus automation (stored TargetID, then matching host, then first non-ephemeral tab). Reduces false
/// "Browser session was reset" errors after CDP reconnect or tab-order drift while a real page stays open.
fn maybe_recover_focus_from_ephemeral_blank_tab(
    browser: &Browser,
    tab: Arc<headless_chrome::Tab>,
) -> Result<Arc<headless_chrome::Tab>, String> {
    let url = tab.get_url();
    if !is_new_tab_or_blank(url.as_str()) {
        return Ok(tab);
    }
    let tabs_guard = browser.get_tabs().lock().map_err(|e| e.to_string())?;
    if tabs_guard.len() <= 1 {
        drop(tabs_guard);
        return Ok(tab);
    }
    if let Ok(g) = active_tab_target_id_store().lock() {
        if let Some(ref sid) = *g {
            if let Some((idx, t)) = tabs_guard
                .iter()
                .enumerate()
                .find(|(_, x)| x.get_target_id().as_str() == sid.as_str())
            {
                if !is_new_tab_or_blank(t.get_url().as_str()) {
                    let t = t.clone();
                    drop(tabs_guard);
                    if let Ok(mut g) = current_tab_index().lock() {
                        *g = idx;
                    }
                    t.bring_to_front()
                        .map_err(|e| format!("bring_to_front (blank-tab recovery): {}", e))?;
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: focused tab was blank/new-tab; refocused tab index {} (stored automation TargetID)",
                        idx
                    );
                    return Ok(t);
                }
            }
        }
    }
    if let Ok(hg) = active_automation_tab_url_host_store().lock() {
        if let Some(ref want_host) = *hg {
            if want_host != "(no-host)" && want_host != "(unknown)" {
                for (idx, t) in tabs_guard.iter().enumerate() {
                    let u = t.get_url();
                    if is_new_tab_or_blank(u.as_str()) {
                        continue;
                    }
                    if host_for_navigation_log(u.as_str()) == *want_host {
                        let t = t.clone();
                        drop(tabs_guard);
                        if let Ok(mut g) = current_tab_index().lock() {
                            *g = idx;
                        }
                        t.bring_to_front()
                            .map_err(|e| format!("bring_to_front (blank-tab recovery): {}", e))?;
                        mac_stats_info!(
                            "browser/cdp",
                            "Browser agent [CDP]: focused tab was blank/new-tab; refocused tab index {} (matched automation host {})",
                            idx,
                            want_host
                        );
                        return Ok(t);
                    }
                }
            }
        }
    }
    for (idx, t) in tabs_guard.iter().enumerate() {
        let u = t.get_url();
        if !is_new_tab_or_blank(u.as_str()) {
            let t = t.clone();
            drop(tabs_guard);
            if let Ok(mut g) = current_tab_index().lock() {
                *g = idx;
            }
            t.bring_to_front()
                .map_err(|e| format!("bring_to_front (blank-tab recovery): {}", e))?;
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: focused tab was blank/new-tab; refocused tab index {} (first non-blank tab)",
                idx
            );
            return Ok(t);
        }
    }
    drop(tabs_guard);
    Ok(tab)
}

/// Get the current tab from BROWSER_SESSION. Uses CURRENT_TAB_INDEX when set (e.g. after new-tab navigate).
/// If Chrome has no page tabs (e.g. user closed every tab), creates one `about:blank` tab once per call (OpenClaw-style bootstrap).
/// Ensures tab window bounds are at least VIEWPORT_WIDTH x VIEWPORT_HEIGHT (e.g. when connecting to existing Chrome).
fn get_current_tab() -> Result<(Browser, Arc<headless_chrome::Tab>), String> {
    if let Some(msg) = take_pending_renderer_crash_tool_error() {
        return Err(msg);
    }
    let browser = get_or_create_browser(cdp_debug_port())?;
    let tab = {
        let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        let len = tabs.len();
        if len == 0 {
            drop(tabs);
            mac_stats_info!(
                "browser/cdp",
                "Browser agent [CDP]: no open tabs; bootstrapping about:blank page tab"
            );
            let new_tab = browser.new_tab().map_err(|e| {
                let e_str = e.to_string();
                mac_stats_warn!(
                    "browser/cdp",
                    "Browser agent [CDP]: empty-browser bootstrap new_tab failed: {}",
                    e_str
                );
                let s = format!(
                    "Chrome has no open tabs and automatic tab creation failed: {}. Open a tab manually or use BROWSER_NAVIGATE with new_tab.",
                    e_str
                );
                clear_browser_session_on_error(&s);
                s
            })?;
            let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
            let idx = tabs
                .iter()
                .position(|t| std::sync::Arc::ptr_eq(t, &new_tab))
                .unwrap_or(0);
            drop(tabs);
            if let Ok(mut guard) = current_tab_index().lock() {
                *guard = idx;
            }
            new_tab.bring_to_front().ok();
            new_tab
        } else {
            let idx = *current_tab_index().lock().map_err(|e| e.to_string())?;
            let idx = idx.min(len.saturating_sub(1));
            tabs.get(idx)
                .cloned()
                .ok_or_else(|| "No tab in browser".to_string())?
        }
    };
    let tab = maybe_recover_focus_from_ephemeral_blank_tab(&browser, tab)?;
    let tab = reconcile_active_automation_tab(&browser, tab)?;
    let bounds = Bounds::Normal {
        left: None,
        top: None,
        width: Some(viewport_width() as f64),
        height: Some(viewport_height() as f64),
    };
    cookie_storage::apply_pending_cookie_restore(&tab);
    if let Err(e) = tab.set_bounds(bounds) {
        mac_stats_warn!(
            "browser",
            "Browser agent: set_bounds {}x{} failed: {} (continuing)",
            viewport_width(),
            viewport_height(),
            e
        );
    }
    register_dialog_auto_dismiss(&tab);
    check_browser_alive(&browser, &tab)?;
    record_active_automation_target(&tab);
    let _ = ensure_chrome_download_artifacts_on_tab(&tab);
    cdp_fetch_proxy_auth::ensure_fetch_proxy_auth_on_tab(&tab);
    apply_cdp_emulation_to_tab(tab.as_ref());
    cdp_trace_archive::maybe_start_recording_after_cdp_session_ready();
    Ok((browser, tab))
}

/// Navigate to URL and return formatted browser state for the LLM. Used by BROWSER_NAVIGATE.
pub fn navigate_and_get_state(url: &str) -> Result<String, String> {
    navigate_and_get_state_with_options(url, false)
}

/// Navigate to URL, optionally in a new tab. When `new_tab` is true, opens URL in a new tab and switches focus to it.
pub fn navigate_and_get_state_with_options(url: &str, new_tab: bool) -> Result<String, String> {
    with_connection_retry(|| navigate_and_get_state_inner(url, new_tab).map(|(s, _)| s))
}

/// Same as [`navigate_and_get_state_with_options`] but also returns completed download paths for Discord attachments.
pub(crate) fn navigate_and_get_state_with_options_and_downloads(
    url: &str,
    new_tab: bool,
) -> Result<(String, Vec<PathBuf>), String> {
    with_connection_retry(|| navigate_and_get_state_inner(url, new_tab))
}

fn navigate_and_get_state_inner(
    url: &str,
    new_tab: bool,
) -> Result<(String, Vec<PathBuf>), String> {
    let navigate_target = crate::commands::browser::normalize_and_validate_cdp_navigation_url(url)?;
    if Url::parse(&navigate_target)
        .ok()
        .is_some_and(|u| u.scheme().eq_ignore_ascii_case("file"))
    {
        mac_stats_debug!(
            "browser/security",
            "Browser agent [CDP]: BROWSER_NAVIGATE file:// precheck passed (local html)"
        );
    }
    if let Some(msg) = url_filter::navigation_precheck_error(&navigate_target) {
        return Err(msg);
    }
    // After SSRF + domain precheck so we do not log strict/warn when the URL was already rejected.
    let proxy_state_prefix =
        crate::commands::browser::ssrf_proxy_env_notice_for_tool("BROWSER_NAVIGATE")?;

    let nav_timeout_secs = crate::config::Config::browser_navigation_timeout_secs();
    let same_domain_timeout_secs =
        crate::config::Config::browser_same_domain_navigation_timeout_secs();
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_NAVIGATE: {} (new_tab={})",
        navigate_target,
        new_tab
    );
    let (browser, tab) = if new_tab {
        let browser = get_or_create_browser(cdp_debug_port())
            .inspect_err(|e| clear_browser_session_on_error(e))?;
        let new_tab = browser.new_tab().map_err(|e| {
            let s = format!("New tab failed: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
        let tabs = browser.get_tabs().lock().map_err(|e| e.to_string())?;
        let idx = tabs
            .iter()
            .position(|t| std::sync::Arc::ptr_eq(t, &new_tab))
            .unwrap_or(tabs.len().saturating_sub(1));
        drop(tabs);
        if let Ok(mut guard) = current_tab_index().lock() {
            *guard = idx;
        }
        new_tab.bring_to_front().ok();
        register_dialog_auto_dismiss(&new_tab);
        check_browser_alive(&browser, &new_tab)?;
        cookie_storage::apply_pending_cookie_restore(&new_tab);
        let _ = ensure_chrome_download_artifacts_on_tab(&new_tab);
        cdp_fetch_proxy_auth::ensure_fetch_proxy_auth_on_tab(&new_tab);
        apply_cdp_emulation_to_tab(new_tab.as_ref());
        cdp_trace_archive::maybe_start_recording_after_cdp_session_ready();
        (browser, new_tab)
    } else {
        get_current_tab().inspect_err(|e| clear_browser_session_on_error(e))?
    };
    let redirect_rws_buf = Arc::new(Mutex::new(VecDeque::new()));
    cdp_enable_network_for_redirect_chain_capture(tab.as_ref());
    let redirect_rws_weak =
        cdp_attach_redirect_chain_rws_listener(tab.as_ref(), Arc::clone(&redirect_rws_buf));
    let _redirect_rws_guard = CdpRedirectRwsListenerGuard {
        tab: tab.as_ref(),
        weak: redirect_rws_weak,
    };
    let current_url = tab.get_url();
    let actual_timeout_secs = if let Some(same_secs) = same_domain_timeout_secs {
        if is_same_domain(current_url.as_str(), &navigate_target) {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: same-domain navigation, using {}s timeout",
                same_secs
            );
            same_secs
        } else {
            nav_timeout_secs
        }
    } else {
        nav_timeout_secs
    };

    // Optional bounded diagnostic capture: collect a small number of console error/warning lines
    // and uncaught exceptions during/after navigation so blank SPA failures become interpretable.
    let include_diag = crate::config::Config::browser_include_diagnostics_in_state();
    let mut diag_console_lines: Option<Arc<Mutex<VecDeque<String>>>> = None;
    let mut diag_uncaught_excs: Option<Arc<Mutex<VecDeque<String>>>> = None;
    let mut diag_listener_weak: Option<CdpDiagListenerWeak> = None;
    if include_diag {
        if let Some((console_buf, exc_buf, w)) = try_attach_bounded_cdp_page_diagnostics(tab.as_ref())
        {
            diag_listener_weak = Some(w);
            diag_console_lines = Some(console_buf);
            diag_uncaught_excs = Some(exc_buf);
        }
    }

    let mut lifecycle_listener_weak = None;
    cdp_ensure_lifecycle_events_enabled(&tab);
    let lifecycle_buf = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let lb = Arc::clone(&lifecycle_buf);
    let lifecycle_listener = Arc::new(move |event: &Event| {
        if let Event::PageLifecycleEvent(ev) = event {
            let n = ev.params.name.to_string();
            if let Ok(mut q) = lb.lock() {
                q.push_back(n);
                while q.len() > 80 {
                    q.pop_front();
                }
            }
        }
    });
    let lifecycle_listener_attached = match tab.add_event_listener(lifecycle_listener) {
        Ok(w) => {
            lifecycle_listener_weak = Some(w);
            true
        }
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: lifecycle listener not attached: {} (sync uses wait_until_navigated only)",
                e
            );
            false
        }
    };
    let lifecycle_buf_for_sync = if lifecycle_listener_attached {
        Some(&lifecycle_buf)
    } else {
        None
    };

    let download_dir = crate::config::Config::browser_downloads_dir();
    let _ = std::fs::create_dir_all(&download_dir);
    let _ = ensure_chrome_download_artifacts_on_tab(&tab);
    let pre_dl = cdp_downloads::download_dir_file_snapshot(&download_dir);
    let aux_holder: Arc<Mutex<Option<Arc<AtomicBool>>>> = Arc::new(Mutex::new(None));
    let aux_out: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));

    // Stop diagnostics + lifecycle listeners before every return (avoid leaks between tool calls).
    let mut cleanup_nav = || {
        if let Some(w) = lifecycle_listener_weak.take() {
            let _ = tab.remove_event_listener(&w);
        }
        if let Some(w) = diag_listener_weak.take() {
            detach_bounded_cdp_page_diagnostics(tab.as_ref(), &w);
        }
    };

    tab.set_default_timeout(Duration::from_secs(actual_timeout_secs));
    if let Ok(mut q) = lifecycle_buf.lock() {
        q.clear();
    }
    if let Ok(mut q) = redirect_rws_buf.lock() {
        q.clear();
    }
    if let Some(ws) = cdp_downloads::peek_cdp_ws_url() {
        let s = Arc::new(AtomicBool::new(false));
        cdp_downloads::spawn_download_aux_listener(
            ws,
            download_dir.clone(),
            Arc::clone(&s),
            Arc::clone(&aux_out),
        );
        if let Ok(mut g) = aux_holder.lock() {
            *g = Some(s);
        }
        std::thread::sleep(Duration::from_millis(80));
    }
    let (post_nav_net_flight, post_nav_net_guard) =
        prepare_post_nav_network_idle_tracking(tab.as_ref());
    let nav_start = Instant::now();
    let pre_navigate_target_id = tab.get_target_id().clone();
    let nav_res = tab.navigate_to(&navigate_target).map_err(|e| {
        let msg = e.to_string();
        let detail = navigate_failed_detail_from_display(&msg);
        log_navigation_cdp_failure(&navigate_target, &detail);
        let s = navigation_tool_result_for_failed_navigate(&navigate_target, &detail);
        clear_browser_session_on_error(&s);
        s
    });
    if let Err(e) = nav_res {
        drop(post_nav_net_guard);
        stop_download_aux_listener(&aux_holder);
        cleanup_nav();
        return Err(e);
    }
    let sync_res = synchronize_tab_after_cdp_navigation(
        &tab,
        current_url.as_str(),
        navigate_target.as_str(),
        lifecycle_buf_for_sync,
        nav_start,
        Duration::from_secs(actual_timeout_secs),
        actual_timeout_secs,
        Some(navigate_target.as_str()),
        post_nav_net_flight.as_ref(),
    );
    drop(post_nav_net_guard);
    if let Err(e) = sync_res {
        stop_download_aux_listener(&aux_holder);
        cleanup_nav();
        return Err(e);
    }
    if let Err(e) =
        cdp_validate_redirect_chain_from_rws_buffer(&redirect_rws_buf, navigate_target.as_str())
    {
        stop_download_aux_listener(&aux_holder);
        cleanup_nav();
        return Err(e);
    }
    let final_after_wait = tab.get_url();
    if let Some(msg) = post_navigate_load_failure_message(
        navigate_target.as_str(),
        final_after_wait.as_str(),
        Some(tab.as_ref()),
    ) {
        stop_download_aux_listener(&aux_holder);
        cleanup_nav();
        return Err(msg);
    }
    if let Err(e) =
        run_spa_blank_page_retry_if_needed(&tab, actual_timeout_secs, navigate_target.as_str())
    {
        stop_download_aux_listener(&aux_holder);
        cleanup_nav();
        return Err(e);
    }
    if let Err(e) = assert_final_document_url_ssrf_post_check(
        tab.get_url().as_str(),
        Some(navigate_target.as_str()),
    ) {
        stop_download_aux_listener(&aux_holder);
        cleanup_nav();
        return Err(e);
    }
    let final_url_for_reconcile = tab.get_url();
    let tab = match reconcile_post_navigate_tab_focus(
        &browser,
        &tab,
        pre_navigate_target_id.as_str(),
        final_url_for_reconcile.as_str(),
    ) {
        Ok(t) => t,
        Err(e) => {
            stop_download_aux_listener(&aux_holder);
            cleanup_nav();
            return Err(e);
        }
    };
    let post_nav_warn =
        match enforce_tab_url_navigation_policy(&tab, "BROWSER_NAVIGATE (redirect or final URL)") {
            Ok(w) => w.unwrap_or_default(),
            Err(e) => {
                stop_download_aux_listener(&aux_holder);
                cleanup_nav();
                return Err(e);
            }
        };
    let mut state = match get_browser_state(&tab) {
        Ok(s) => s,
        Err(e) => {
            clear_browser_session_on_error(&e);
            stop_download_aux_listener(&aux_holder);
            cleanup_nav();
            return Err(e);
        }
    };
    // Auto-dismiss cookie banner so the user doesn't have to ask. Visible in logs and verbose mode.
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: checking for cookie banner after navigate to {}",
        navigate_target
    );
    if let Ok(true) = try_dismiss_cookie_banner(&tab) {
        std::thread::sleep(BROWSER_STATE_REFRESH_AFTER_UI_MS);
        if let Ok(refreshed) = get_browser_state(&tab) {
            state = refreshed;
        }
    }

    let diagnostics_section =
        if include_diag {
            if let (Some(c), Some(e)) = (&diag_console_lines, &diag_uncaught_excs) {
                let n_c = c.lock().map(|q| q.len()).unwrap_or(0);
                let n_e = e.lock().map(|q| q.len()).unwrap_or(0);
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: captured diagnostics console_lines={} uncaught_exceptions={}",
                    n_c,
                    n_e
                );
                format_bounded_page_diagnostics_tool_section(c, e)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

    cleanup_nav();

    record_active_automation_target(&tab);
    try_enforce_browser_tab_limit(&browser, &tab);
    let mut snapshot = proxy_state_prefix.unwrap_or_default()
        + &post_nav_warn
        + &format_browser_state_snapshot(&browser, &state);
    snapshot.push_str(&diagnostics_section);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    if let Err(e) = cookie_storage::save_cookies_from_tab(&tab) {
        mac_stats_debug!(
            "browser/cookies",
            "Browser agent [cookies]: save after navigate skipped: {}",
            e
        );
    }
    std::thread::sleep(cdp_downloads::POST_ACTION_DOWNLOAD_WAIT);
    stop_download_aux_listener(&aux_holder);
    let cdp_paths = aux_out.lock().map(|g| g.clone()).unwrap_or_default();
    let mut dl_paths = cdp_downloads::merge_with_directory_diff(&download_dir, &pre_dl, &cdp_paths);
    if url_looks_like_http_pdf(&state.current_url) {
        if let Some(p) = try_save_printed_pdf(&tab, &download_dir) {
            dl_paths.push(p);
        }
    }
    dl_paths.sort();
    dl_paths.dedup();
    snapshot.push_str(&cdp_downloads::format_download_attachment_note(&dl_paths));
    Ok((snapshot, dl_paths))
}

/// Clear persisted CDP cookie jar file and in-browser cookies (`Network.clearBrowserCookies`).
/// The JSON file is removed first so a CDP failure still drops persisted state.
pub fn clear_browser_auth_storage() -> Result<String, String> {
    let removed_file = cookie_storage::remove_storage_file_if_present()?;
    match get_current_tab() {
        Ok((_, tab)) => {
            cookie_storage::clear_browser_cookie_jar(&tab)?;
            Ok(format!(
                "BROWSER_CLEAR_COOKIES: {} and cleared in-browser cookie jar via CDP.",
                if removed_file {
                    "removed browser_storage_state.json"
                } else {
                    "no persisted cookie file was present"
                }
            ))
        }
        Err(e) => {
            if removed_file {
                Ok(format!(
                    "BROWSER_CLEAR_COOKIES: removed persisted cookie file; could not run Network.clearBrowserCookies ({}).",
                    e
                ))
            } else {
                Err(e)
            }
        }
    }
}

/// Go back one step in the current tab's history and return the new page state. Use when the agent should return to the previous page without re-entering the URL.
pub fn go_back() -> Result<String, String> {
    with_connection_retry(go_back_inner)
}

fn go_back_inner() -> Result<String, String> {
    let (browser, tab) = get_current_tab().inspect_err(|e| clear_browser_session_on_error(e))?;
    let url = tab.get_url();
    if is_new_tab_or_blank(url.as_str()) {
        return Err("No page to go back from. Use BROWSER_NAVIGATE first.".to_string());
    }
    let nav_timeout_secs = crate::config::Config::browser_navigation_timeout_secs();
    tab.set_default_timeout(Duration::from_secs(nav_timeout_secs));
    mac_stats_info!("browser/cdp", "Browser agent [CDP]: BROWSER_GO_BACK");
    let pre_navigate_target_id = tab.get_target_id().clone();
    with_lifecycle_event_buffer(&tab, |buf_opt| {
        let (post_nav_net_flight, _post_nav_net_guard) =
            prepare_post_nav_network_idle_tracking(&tab);
        let prev_url = tab.get_url();
        let nav_start = Instant::now();
        tab.evaluate("window.history.back()", false)
            .map_err(|e| format!("Go back failed: {}", e))?;
        std::thread::sleep(BROWSER_HISTORY_SCRIPT_THEN_WAIT);
        let after_url = tab.get_url();
        synchronize_tab_after_cdp_navigation(
            &tab,
            prev_url.as_str(),
            after_url.as_str(),
            buf_opt,
            nav_start,
            Duration::from_secs(nav_timeout_secs),
            nav_timeout_secs,
            None,
            post_nav_net_flight.as_ref(),
        )
    })?;
    std::thread::sleep(BROWSER_AFTER_HISTORY_NAV_SETTLE);
    assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None)?;
    let final_url = tab.get_url();
    let tab = reconcile_post_navigate_tab_focus(
        &browser,
        &tab,
        pre_navigate_target_id.as_str(),
        final_url.as_str(),
    )?;
    let prefix = enforce_tab_url_navigation_policy(&tab, "BROWSER_GO_BACK")?.unwrap_or_default();
    let state = get_browser_state(&tab).inspect_err(|e| clear_browser_session_on_error(e))?;
    let snapshot = prefix + &format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: went back to {}",
        state.current_url
    );
    Ok(snapshot)
}

/// Go forward one step in the current tab's history and return the new page state.
pub fn go_forward() -> Result<String, String> {
    with_connection_retry(go_forward_inner)
}

fn go_forward_inner() -> Result<String, String> {
    let (browser, tab) = get_current_tab().inspect_err(|e| clear_browser_session_on_error(e))?;
    let url = tab.get_url();
    if is_new_tab_or_blank(url.as_str()) {
        return Err("No page to go forward from. Use BROWSER_NAVIGATE first.".to_string());
    }
    let nav_timeout_secs = crate::config::Config::browser_navigation_timeout_secs();
    tab.set_default_timeout(Duration::from_secs(nav_timeout_secs));
    mac_stats_info!("browser/cdp", "Browser agent [CDP]: BROWSER_GO_FORWARD");
    let pre_navigate_target_id = tab.get_target_id().clone();
    with_lifecycle_event_buffer(&tab, |buf_opt| {
        let (post_nav_net_flight, _post_nav_net_guard) =
            prepare_post_nav_network_idle_tracking(&tab);
        let prev_url = tab.get_url();
        let nav_start = Instant::now();
        tab.evaluate("window.history.forward()", false)
            .map_err(|e| format!("Go forward failed: {}", e))?;
        std::thread::sleep(BROWSER_HISTORY_SCRIPT_THEN_WAIT);
        let after_url = tab.get_url();
        synchronize_tab_after_cdp_navigation(
            &tab,
            prev_url.as_str(),
            after_url.as_str(),
            buf_opt,
            nav_start,
            Duration::from_secs(nav_timeout_secs),
            nav_timeout_secs,
            None,
            post_nav_net_flight.as_ref(),
        )
    })?;
    std::thread::sleep(BROWSER_AFTER_HISTORY_NAV_SETTLE);
    assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None)?;
    let final_url = tab.get_url();
    let tab = reconcile_post_navigate_tab_focus(
        &browser,
        &tab,
        pre_navigate_target_id.as_str(),
        final_url.as_str(),
    )?;
    let prefix = enforce_tab_url_navigation_policy(&tab, "BROWSER_GO_FORWARD")?.unwrap_or_default();
    let state = get_browser_state(&tab).inspect_err(|e| clear_browser_session_on_error(e))?;
    let snapshot = prefix + &format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: went forward to {}",
        state.current_url
    );
    Ok(snapshot)
}

/// Reload the current tab via CDP `Page.reload`. When `ignore_cache` is true, bypass cache (Shift+reload style).
pub fn reload_current_tab(ignore_cache: bool) -> Result<String, String> {
    with_connection_retry(|| reload_current_tab_inner(ignore_cache))
}

fn reload_current_tab_inner(ignore_cache: bool) -> Result<String, String> {
    let (browser, tab) = get_current_tab().inspect_err(|e| clear_browser_session_on_error(e))?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    let nav_timeout_secs = crate::config::Config::browser_navigation_timeout_secs();
    tab.set_default_timeout(Duration::from_secs(nav_timeout_secs));
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_RELOAD (ignore_cache={})",
        ignore_cache
    );
    let pre_navigate_target_id = tab.get_target_id().clone();
    with_lifecycle_event_buffer(&tab, |buf_opt| {
        let (post_nav_net_flight, _post_nav_net_guard) =
            prepare_post_nav_network_idle_tracking(&tab);
        let prev_url = tab.get_url();
        let nav_start = Instant::now();
        tab.reload(ignore_cache, None)
            .map_err(|e| format!("Reload failed: {}", e))?;
        synchronize_tab_after_cdp_navigation(
            &tab,
            prev_url.as_str(),
            prev_url.as_str(),
            buf_opt,
            nav_start,
            Duration::from_secs(nav_timeout_secs),
            nav_timeout_secs,
            None,
            post_nav_net_flight.as_ref(),
        )
    })?;
    std::thread::sleep(BROWSER_AFTER_HISTORY_NAV_SETTLE);
    assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None)?;
    let final_url = tab.get_url();
    let tab = reconcile_post_navigate_tab_focus(
        &browser,
        &tab,
        pre_navigate_target_id.as_str(),
        final_url.as_str(),
    )?;
    let prefix = enforce_tab_url_navigation_policy(&tab, "BROWSER_RELOAD")?.unwrap_or_default();
    let state = get_browser_state(&tab).inspect_err(|e| clear_browser_session_on_error(e))?;
    let snapshot = prefix + &format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: reloaded, current URL {}",
        state.current_url
    );
    Ok(snapshot)
}

/// Click the Nth interactive element (1-based index). Returns updated browser state string.
pub fn click_by_index(index: u32) -> Result<String, String> {
    with_connection_retry(|| click_by_index_inner(index).map(|(s, _)| s))
}

/// Move the pointer over the Nth interactive element (1-based index). Returns updated browser state string.
pub fn hover_by_index(index: u32) -> Result<String, String> {
    with_connection_retry(|| hover_by_index_inner(index).map(|(s, _)| s))
}

/// Drag from element `from_index` to `to_index` (1-based indices from the Elements list). Returns updated browser state string.
pub fn drag_by_indices(from_index: u32, to_index: u32) -> Result<String, String> {
    with_connection_retry(|| drag_by_indices_inner(from_index, to_index).map(|(s, _)| s))
}

pub(crate) fn click_by_index_inner_with_downloads(
    index: u32,
) -> Result<(String, Vec<PathBuf>), String> {
    with_connection_retry(|| click_by_index_inner(index))
}

fn click_by_index_inner(index: u32) -> Result<(String, Vec<PathBuf>), String> {
    if index == 0 {
        return Err("BROWSER_CLICK index must be >= 1".to_string());
    }
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let n_before = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_CLICK index={}",
        index
    );
    let object_id = resolve_interactable_for_action(&tab, index, "BROWSER_CLICK").map_err(|e| {
        let s = format!("BROWSER_CLICK: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;

    let download_dir = crate::config::Config::browser_downloads_dir();
    let _ = std::fs::create_dir_all(&download_dir);
    let _ = ensure_chrome_download_artifacts_on_tab(&tab);
    let pre_dl = cdp_downloads::download_dir_file_snapshot(&download_dir);
    let aux_holder: Arc<Mutex<Option<Arc<AtomicBool>>>> = Arc::new(Mutex::new(None));
    let aux_out: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    if let Some(ws) = cdp_downloads::peek_cdp_ws_url() {
        let s = Arc::new(AtomicBool::new(false));
        cdp_downloads::spawn_download_aux_listener(
            ws,
            download_dir.clone(),
            Arc::clone(&s),
            Arc::clone(&aux_out),
        );
        if let Ok(mut g) = aux_holder.lock() {
            *g = Some(s);
        }
        std::thread::sleep(Duration::from_millis(80));
    }

    if let Err(e) = cdp_click_by_object_id(&tab, &object_id) {
        stop_download_aux_listener(&aux_holder);
        let s = format!("BROWSER_CLICK: {}", e);
        clear_browser_session_on_error(&s);
        return Err(s);
    }
    std::thread::sleep(BROWSER_AFTER_CLICK_OR_COORD_SETTLE);
    let n_after = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if n_after > n_before {
        if let Ok(mut g) = current_tab_index().lock() {
            *g = n_after.saturating_sub(1);
        }
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: BROWSER_CLICK tab count {} -> {}; focusing newest tab",
            n_before,
            n_after
        );
    }
    let (browser, tab) = match get_current_tab() {
        Ok(x) => x,
        Err(e) => {
            stop_download_aux_listener(&aux_holder);
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let new_tab_note = new_tabs_opened_notice(&browser, n_before);
    if let Err(e) = assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None) {
        stop_download_aux_listener(&aux_holder);
        return Err(e);
    }
    let mut state = match get_browser_state(&tab) {
        Ok(s) => s,
        Err(e) => {
            stop_download_aux_listener(&aux_holder);
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let mut prefix = String::new();
    if url_filter::url_scheme_subject_to_domain_policy(state.current_url.as_str())
        && !url_filter::is_navigation_allowed(state.current_url.as_str())
    {
        prefix = match enforce_tab_url_navigation_policy(
            &tab,
            "BROWSER_CLICK (e.g. new tab or redirect)",
        ) {
            Ok(w) => w.unwrap_or_default(),
            Err(e) => {
                stop_download_aux_listener(&aux_holder);
                clear_browser_session_on_error(&e);
                return Err(e);
            }
        };
        if !prefix.is_empty() {
            state = match get_browser_state(&tab) {
                Ok(s) => s,
                Err(e) => {
                    stop_download_aux_listener(&aux_holder);
                    clear_browser_session_on_error(&e);
                    return Err(e);
                }
            };
        }
    }
    try_enforce_browser_tab_limit(&browser, &tab);
    let mut snapshot = prefix;
    if let Some(note) = new_tab_note {
        snapshot.push_str(&note);
    }
    snapshot.push_str(&format_browser_state_snapshot(&browser, &state));
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    std::thread::sleep(cdp_downloads::POST_ACTION_DOWNLOAD_WAIT);
    stop_download_aux_listener(&aux_holder);
    let cdp_paths = aux_out.lock().map(|g| g.clone()).unwrap_or_default();
    let mut dl_paths = cdp_downloads::merge_with_directory_diff(&download_dir, &pre_dl, &cdp_paths);
    if url_looks_like_http_pdf(&state.current_url) {
        if let Some(p) = try_save_printed_pdf(&tab, &download_dir) {
            dl_paths.push(p);
        }
    }
    dl_paths.sort();
    dl_paths.dedup();
    snapshot.push_str(&cdp_downloads::format_download_attachment_note(&dl_paths));
    Ok((snapshot, dl_paths))
}

pub(crate) fn hover_by_index_inner_with_downloads(
    index: u32,
) -> Result<(String, Vec<PathBuf>), String> {
    with_connection_retry(|| hover_by_index_inner(index))
}

fn hover_by_index_inner(index: u32) -> Result<(String, Vec<PathBuf>), String> {
    if index == 0 {
        return Err("BROWSER_HOVER index must be >= 1".to_string());
    }
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let n_before = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_HOVER index={}",
        index
    );
    let object_id = resolve_interactable_for_action(&tab, index, "BROWSER_HOVER").map_err(|e| {
        let s = format!("BROWSER_HOVER: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;

    if let Err(e) = cdp_hover_by_object_id(&tab, &object_id) {
        let s = format!("BROWSER_HOVER: {}", e);
        clear_browser_session_on_error(&s);
        return Err(s);
    }
    std::thread::sleep(BROWSER_AFTER_HOVER_SETTLE);
    let n_after = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if n_after > n_before {
        if let Ok(mut g) = current_tab_index().lock() {
            *g = n_after.saturating_sub(1);
        }
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: BROWSER_HOVER tab count {} -> {}; focusing newest tab",
            n_before,
            n_after
        );
    }
    let (browser, tab) = match get_current_tab() {
        Ok(x) => x,
        Err(e) => {
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let new_tab_note = new_tabs_opened_notice(&browser, n_before);
    if let Err(e) = assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None) {
        return Err(e);
    }
    let mut state = match get_browser_state(&tab) {
        Ok(s) => s,
        Err(e) => {
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let mut prefix = String::new();
    if url_filter::url_scheme_subject_to_domain_policy(state.current_url.as_str())
        && !url_filter::is_navigation_allowed(state.current_url.as_str())
    {
        prefix = match enforce_tab_url_navigation_policy(&tab, "BROWSER_HOVER") {
            Ok(w) => w.unwrap_or_default(),
            Err(e) => {
                clear_browser_session_on_error(&e);
                return Err(e);
            }
        };
        if !prefix.is_empty() {
            state = match get_browser_state(&tab) {
                Ok(s) => s,
                Err(e) => {
                    clear_browser_session_on_error(&e);
                    return Err(e);
                }
            };
        }
    }
    try_enforce_browser_tab_limit(&browser, &tab);
    let mut snapshot = prefix;
    if let Some(note) = new_tab_note {
        snapshot.push_str(&note);
    }
    snapshot.push_str(&format_browser_state_snapshot(&browser, &state));
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    Ok((snapshot, vec![]))
}

pub(crate) fn drag_by_indices_inner_with_downloads(
    from_index: u32,
    to_index: u32,
) -> Result<(String, Vec<PathBuf>), String> {
    with_connection_retry(|| drag_by_indices_inner(from_index, to_index))
}

fn drag_by_indices_inner(from_index: u32, to_index: u32) -> Result<(String, Vec<PathBuf>), String> {
    if from_index == 0 || to_index == 0 {
        return Err("BROWSER_DRAG: each index must be >= 1 (from the Elements list).".to_string());
    }
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let n_before = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_DRAG from_index={} to_index={}",
        from_index,
        to_index
    );
    let from_id =
        resolve_interactable_for_action(&tab, from_index, "BROWSER_DRAG").map_err(|e| {
            let s = format!("BROWSER_DRAG: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    let to_id = resolve_interactable_for_action(&tab, to_index, "BROWSER_DRAG").map_err(|e| {
        let s = format!("BROWSER_DRAG: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;

    let download_dir = crate::config::Config::browser_downloads_dir();
    let _ = std::fs::create_dir_all(&download_dir);
    let _ = ensure_chrome_download_artifacts_on_tab(&tab);
    let pre_dl = cdp_downloads::download_dir_file_snapshot(&download_dir);
    let aux_holder: Arc<Mutex<Option<Arc<AtomicBool>>>> = Arc::new(Mutex::new(None));
    let aux_out: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    if let Some(ws) = cdp_downloads::peek_cdp_ws_url() {
        let s = Arc::new(AtomicBool::new(false));
        cdp_downloads::spawn_download_aux_listener(
            ws,
            download_dir.clone(),
            Arc::clone(&s),
            Arc::clone(&aux_out),
        );
        if let Ok(mut g) = aux_holder.lock() {
            *g = Some(s);
        }
        std::thread::sleep(Duration::from_millis(80));
    }

    if let Err(e) = cdp_drag_between_object_ids(&tab, &from_id, &to_id) {
        stop_download_aux_listener(&aux_holder);
        let s = format!("BROWSER_DRAG: {}", e);
        clear_browser_session_on_error(&s);
        return Err(s);
    }
    std::thread::sleep(BROWSER_AFTER_CLICK_OR_COORD_SETTLE);
    let n_after = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if n_after > n_before {
        if let Ok(mut g) = current_tab_index().lock() {
            *g = n_after.saturating_sub(1);
        }
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: BROWSER_DRAG tab count {} -> {}; focusing newest tab",
            n_before,
            n_after
        );
    }
    let (browser, tab) = match get_current_tab() {
        Ok(x) => x,
        Err(e) => {
            stop_download_aux_listener(&aux_holder);
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let new_tab_note = new_tabs_opened_notice(&browser, n_before);
    if let Err(e) = assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None) {
        stop_download_aux_listener(&aux_holder);
        return Err(e);
    }
    let mut state = match get_browser_state(&tab) {
        Ok(s) => s,
        Err(e) => {
            stop_download_aux_listener(&aux_holder);
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let mut prefix = String::new();
    if url_filter::url_scheme_subject_to_domain_policy(state.current_url.as_str())
        && !url_filter::is_navigation_allowed(state.current_url.as_str())
    {
        prefix = match enforce_tab_url_navigation_policy(&tab, "BROWSER_DRAG") {
            Ok(w) => w.unwrap_or_default(),
            Err(e) => {
                stop_download_aux_listener(&aux_holder);
                clear_browser_session_on_error(&e);
                return Err(e);
            }
        };
        if !prefix.is_empty() {
            state = match get_browser_state(&tab) {
                Ok(s) => s,
                Err(e) => {
                    stop_download_aux_listener(&aux_holder);
                    clear_browser_session_on_error(&e);
                    return Err(e);
                }
            };
        }
    }
    try_enforce_browser_tab_limit(&browser, &tab);
    let mut snapshot = prefix;
    if let Some(note) = new_tab_note {
        snapshot.push_str(&note);
    }
    snapshot.push_str(&format_browser_state_snapshot(&browser, &state));
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    std::thread::sleep(cdp_downloads::POST_ACTION_DOWNLOAD_WAIT);
    stop_download_aux_listener(&aux_holder);
    let cdp_paths = aux_out.lock().map(|g| g.clone()).unwrap_or_default();
    let mut dl_paths = cdp_downloads::merge_with_directory_diff(&download_dir, &pre_dl, &cdp_paths);
    if url_looks_like_http_pdf(&state.current_url) {
        if let Some(p) = try_save_printed_pdf(&tab, &download_dir) {
            dl_paths.push(p);
        }
    }
    dl_paths.sort();
    dl_paths.dedup();
    snapshot.push_str(&cdp_downloads::format_download_attachment_note(&dl_paths));
    Ok((snapshot, dl_paths))
}

/// Click at viewport CSS coordinates (e.g. from a vision model). Uses CDP only; no HTTP fallback.
pub fn click_at_viewport_coordinates(x: f64, y: f64) -> Result<String, String> {
    with_connection_retry(|| click_at_viewport_coordinates_inner(x, y).map(|(s, _)| s))
}

pub(crate) fn click_at_viewport_coordinates_with_downloads(
    x: f64,
    y: f64,
) -> Result<(String, Vec<PathBuf>), String> {
    with_connection_retry(|| click_at_viewport_coordinates_inner(x, y))
}

fn click_at_viewport_coordinates_inner(x: f64, y: f64) -> Result<(String, Vec<PathBuf>), String> {
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let n_before = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: click_at_viewport_coordinates ({:.1}, {:.1})",
        x,
        y
    );

    let download_dir = crate::config::Config::browser_downloads_dir();
    let _ = std::fs::create_dir_all(&download_dir);
    let _ = ensure_chrome_download_artifacts_on_tab(&tab);
    let pre_dl = cdp_downloads::download_dir_file_snapshot(&download_dir);
    let aux_holder: Arc<Mutex<Option<Arc<AtomicBool>>>> = Arc::new(Mutex::new(None));
    let aux_out: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    if let Some(ws) = cdp_downloads::peek_cdp_ws_url() {
        let s = Arc::new(AtomicBool::new(false));
        cdp_downloads::spawn_download_aux_listener(
            ws,
            download_dir.clone(),
            Arc::clone(&s),
            Arc::clone(&aux_out),
        );
        if let Ok(mut g) = aux_holder.lock() {
            *g = Some(s);
        }
        std::thread::sleep(Duration::from_millis(80));
    }

    if let Err(e) = tab.click_point(Point { x, y }) {
        stop_download_aux_listener(&aux_holder);
        let s = format!("Click at point: {}", e);
        clear_browser_session_on_error(&s);
        return Err(s);
    }
    std::thread::sleep(BROWSER_AFTER_CLICK_OR_COORD_SETTLE);
    let n_after = browser.get_tabs().lock().map(|t| t.len()).unwrap_or(0);
    if n_after > n_before {
        if let Ok(mut g) = current_tab_index().lock() {
            *g = n_after.saturating_sub(1);
        }
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: coordinate click tab count {} -> {}; focusing newest tab",
            n_before,
            n_after
        );
    }
    let (browser, tab) = match get_current_tab() {
        Ok(x) => x,
        Err(e) => {
            stop_download_aux_listener(&aux_holder);
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let new_tab_note = new_tabs_opened_notice(&browser, n_before);
    if let Err(e) = assert_final_document_url_ssrf_post_check(tab.get_url().as_str(), None) {
        stop_download_aux_listener(&aux_holder);
        return Err(e);
    }
    let mut state = match get_browser_state(&tab) {
        Ok(s) => s,
        Err(e) => {
            stop_download_aux_listener(&aux_holder);
            clear_browser_session_on_error(&e);
            return Err(e);
        }
    };
    let mut prefix = String::new();
    if url_filter::url_scheme_subject_to_domain_policy(state.current_url.as_str())
        && !url_filter::is_navigation_allowed(state.current_url.as_str())
    {
        prefix = match enforce_tab_url_navigation_policy(&tab, "BROWSER_CLICK coordinates") {
            Ok(w) => w.unwrap_or_default(),
            Err(e) => {
                stop_download_aux_listener(&aux_holder);
                clear_browser_session_on_error(&e);
                return Err(e);
            }
        };
        if !prefix.is_empty() {
            state = match get_browser_state(&tab) {
                Ok(s) => s,
                Err(e) => {
                    stop_download_aux_listener(&aux_holder);
                    clear_browser_session_on_error(&e);
                    return Err(e);
                }
            };
        }
    }
    try_enforce_browser_tab_limit(&browser, &tab);
    let mut snapshot = prefix;
    if let Some(note) = new_tab_note {
        snapshot.push_str(&note);
    }
    snapshot.push_str(&format_browser_state_snapshot(&browser, &state));
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    std::thread::sleep(cdp_downloads::POST_ACTION_DOWNLOAD_WAIT);
    stop_download_aux_listener(&aux_holder);
    let cdp_paths = aux_out.lock().map(|g| g.clone()).unwrap_or_default();
    let mut dl_paths = cdp_downloads::merge_with_directory_diff(&download_dir, &pre_dl, &cdp_paths);
    if url_looks_like_http_pdf(&state.current_url) {
        if let Some(p) = try_save_printed_pdf(&tab, &download_dir) {
            dl_paths.push(p);
        }
    }
    dl_paths.sort();
    dl_paths.dedup();
    snapshot.push_str(&cdp_downloads::format_download_attachment_note(&dl_paths));
    Ok((snapshot, dl_paths))
}

/// Wait for a download to finish (CDP events + `browser-downloads/` directory diff), up to `timeout_secs` (clamped 1–120).
pub fn wait_for_browser_download(timeout_secs: u64) -> Result<(String, Vec<PathBuf>), String> {
    with_connection_retry(|| wait_for_browser_download_inner(timeout_secs))
}

fn wait_for_browser_download_inner(timeout_secs: u64) -> Result<(String, Vec<PathBuf>), String> {
    let (_, tab) = get_current_tab().inspect_err(|e| clear_browser_session_on_error(e))?;
    let download_dir = crate::config::Config::browser_downloads_dir();
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    ensure_chrome_download_artifacts_on_tab(&tab)?;
    let pre_dl = cdp_downloads::download_dir_file_snapshot(&download_dir);
    let aux_holder: Arc<Mutex<Option<Arc<AtomicBool>>>> = Arc::new(Mutex::new(None));
    let aux_out: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    if let Some(ws) = cdp_downloads::peek_cdp_ws_url() {
        let s = Arc::new(AtomicBool::new(false));
        cdp_downloads::spawn_download_aux_listener(
            ws,
            download_dir.clone(),
            Arc::clone(&s),
            Arc::clone(&aux_out),
        );
        if let Ok(mut g) = aux_holder.lock() {
            *g = Some(s);
        }
        std::thread::sleep(Duration::from_millis(80));
    }
    let cap = timeout_secs.clamp(1, 120);
    let deadline = Instant::now() + Duration::from_secs(cap);
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(200));
        let cdp_paths = aux_out.lock().map(|g| g.clone()).unwrap_or_default();
        let got = cdp_downloads::merge_with_directory_diff(&download_dir, &pre_dl, &cdp_paths);
        if !got.is_empty() {
            break;
        }
    }
    stop_download_aux_listener(&aux_holder);
    let cdp_paths = aux_out.lock().map(|g| g.clone()).unwrap_or_default();
    let found = cdp_downloads::merge_with_directory_diff(&download_dir, &pre_dl, &cdp_paths);
    let msg = if found.is_empty() {
        mac_stats_info!(
            "browser/cdp",
            "BROWSER_DOWNLOAD: timeout after {}s (no new completed file under {})",
            cap,
            download_dir.display()
        );
        format!(
            "BROWSER_DOWNLOAD: no completed download within {}s (watching {}).",
            cap,
            download_dir.display()
        )
    } else {
        mac_stats_info!(
            "browser/cdp",
            "BROWSER_DOWNLOAD: {} completed file(s) for attachment pipeline",
            found.len()
        );
        format!(
            "BROWSER_DOWNLOAD: {} file(s) ready.{}",
            found.len(),
            cdp_downloads::format_download_attachment_note(&found)
        )
    };
    Ok((msg, found))
}

/// CDP `Runtime.callFunctionOn`: select / HTML5 value inputs / datepicker-like text / contenteditable;
/// returns `use_key_dispatch` for ordinary text fields (handled with `Input.dispatchKeyEvent` per character).
const MAC_STATS_BROWSER_INPUT_APPLY_JS: &str = r#"function(t) {
  var el = this;
  if (!el || el.nodeType !== 1) return "bad_element";
  try { el.scrollIntoView({block:"center", inline:"nearest"}); } catch (e0) {}
  var tag = el.tagName;
  var text = String(t);
  if (tag === "SELECT") {
    el.focus();
    var want = text.trim();
    var opts = el.options;
    var matched = null;
    for (var i = 0; i < opts.length; i++) {
      var o = opts[i];
      if (o.value === want) { matched = o; break; }
      var ot = String(o.text).trim();
      if (ot === want || ot.toLowerCase() === want.toLowerCase()) { matched = o; break; }
    }
    if (!matched) return "select_no_match";
    el.value = matched.value;
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
    return "ok_select";
  }
  if (el.isContentEditable) {
    el.focus();
    el.textContent = "";
    try { document.execCommand("insertText", false, text); }
    catch (e1) { el.textContent = text; }
    return "ok_contenteditable";
  }
  if (tag === "INPUT" || tag === "TEXTAREA") {
    var typ = String(el.type || "text").toLowerCase();
    var skipKey = {checkbox:1,radio:1,file:1,button:1,submit:1,reset:1,image:1,hidden:1};
    var clsRaw = el.className ? String(el.className) : "";
    var cls = clsRaw.toLowerCase();
    var dpAttr = el.getAttribute("data-provide");
    var dpa = dpAttr ? String(dpAttr).toLowerCase() : "";
    var datepickerLike = cls.indexOf("datepicker") >= 0 || cls.indexOf("date-picker") >= 0 || dpa.indexOf("datepicker") >= 0;
    var nativeTypes = {date:1,time:1,"datetime-local":1,month:1,week:1,color:1,range:1};
    if (nativeTypes[typ] || (typ === "text" && datepickerLike)) {
      el.focus();
      var proto = (tag === "TEXTAREA") ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype;
      var desc = Object.getOwnPropertyDescriptor(proto, "value");
      if (desc && desc.set) desc.set.call(el, text);
      else el.value = text;
      el.dispatchEvent(new Event("input", { bubbles: true }));
      el.dispatchEvent(new Event("change", { bubbles: true }));
      try { el.dispatchEvent(new Event("blur", { bubbles: true })); } catch (e2) {}
      if (datepickerLike && typ === "text" && window.jQuery && window.jQuery(el).datepicker) {
        try { window.jQuery(el).datepicker("update"); } catch (e3) {}
      }
      return datepickerLike ? "ok_datepicker" : "ok_native";
    }
    if (skipKey[typ]) return "unsupported_type";
    return "use_key_dispatch";
  }
  return "not_text_input";
}"#;

/// Type text into the Nth interactive element (1-based index). Returns updated browser state string.
pub fn input_by_index(index: u32, text: &str) -> Result<String, String> {
    with_connection_retry(|| input_by_index_inner(index, text))
}

fn input_by_index_inner(index: u32, text: &str) -> Result<String, String> {
    if index == 0 {
        return Err("BROWSER_INPUT index must be >= 1".to_string());
    }
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    let page_url = tab.get_url();
    let text = credentials::substitute_secret_tags_in_input(&page_url, text)?;
    if let Ok(g) = last_interactables_snapshot().lock() {
        if let Some(list) = g.as_ref() {
            if let Some(m) = list.get(index.saturating_sub(1) as usize) {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: BROWSER_INPUT index={} route_hint tag={} input_type={:?} contenteditable={} datepicker_like={}",
                    index,
                    m.tag,
                    m.input_type,
                    m.contenteditable,
                    m.datepicker_like
                );
            }
        }
    }
    let object_id = resolve_interactable_for_action(&tab, index, "BROWSER_INPUT").map_err(|e| {
        let s = format!("BROWSER_INPUT: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    if let Err(e) = tab.call_method(DOM::ScrollIntoViewIfNeeded {
        node_id: None,
        backend_node_id: None,
        object_id: Some(object_id.clone()),
        rect: None,
    }) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: BROWSER_INPUT DOM.scrollIntoViewIfNeeded: {} — continuing",
            e
        );
    }
    let res = tab
        .call_method(Runtime::CallFunctionOn {
            function_declaration: MAC_STATS_BROWSER_INPUT_APPLY_JS.to_string(),
            object_id: Some(object_id.clone()),
            arguments: Some(vec![Runtime::CallArgument {
                value: Some(serde_json::Value::String(text.to_string())),
                unserializable_value: None,
                object_id: None,
            }]),
            return_by_value: Some(true),
            await_promise: Some(false),
            user_gesture: None,
            silent: Some(false),
            generate_preview: Some(false),
            execution_context_id: None,
            object_group: None,
            throw_on_side_effect: None,
            serialization_options: None,
            unique_context_id: None,
        })
        .map_err(|e| {
            let s = format!("BROWSER_INPUT: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    if res.exception_details.is_some() {
        let s = "BROWSER_INPUT: script exception while applying value".to_string();
        clear_browser_session_on_error(&s);
        return Err(s);
    }
    let msg = res
        .result
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("?")
        .to_string();
    match msg.as_str() {
        "use_key_dispatch" => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: BROWSER_INPUT index={} path=key_dispatch (CDP key events)",
                index
            );
            tab.call_method(Runtime::CallFunctionOn {
                function_declaration: "function(){ this.focus(); return \"ok\"; }".to_string(),
                object_id: Some(object_id),
                arguments: None,
                return_by_value: Some(true),
                await_promise: Some(false),
                user_gesture: None,
                silent: Some(false),
                generate_preview: Some(false),
                execution_context_id: None,
                object_group: None,
                throw_on_side_effect: None,
                serialization_options: None,
                unique_context_id: None,
            })
            .map_err(|e| {
                let s = format!("BROWSER_INPUT: focus: {}", e);
                clear_browser_session_on_error(&s);
                s
            })?;
            tab.type_str(text.as_str()).map_err(|e| {
                let s = format!("BROWSER_INPUT: {}", e);
                clear_browser_session_on_error(&s);
                s
            })?;
        }
        "ok_select" | "ok_native" | "ok_datepicker" | "ok_contenteditable" => {
            if msg == "ok_datepicker" {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: BROWSER_INPUT index={} path=datepicker_heuristic (direct value + events)",
                    index
                );
            } else {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: BROWSER_INPUT index={} path={}",
                    index,
                    msg
                );
            }
        }
        "select_no_match" => {
            return Err(format!(
                "BROWSER_INPUT: <select> has no option matching {:?} (use option value or visible label).",
                text
            ));
        }
        "unsupported_type" => {
            return Err(
                "BROWSER_INPUT: this control does not accept typed text (e.g. checkbox, file, button)."
                    .to_string(),
            );
        }
        "not_text_input" => {
            return Err(
                "BROWSER_INPUT: element is not a text field, textarea, select, or contenteditable region."
                    .to_string(),
            );
        }
        "bad_element" => {
            return Err("BROWSER_INPUT: invalid element handle.".to_string());
        }
        other => {
            return Err(format!(
                "BROWSER_INPUT: unexpected script result {:?}",
                other
            ));
        }
    }
    std::thread::sleep(BROWSER_POST_INPUT_SETTLE);
    let state = get_browser_state(&tab).inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let snapshot = format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    Ok(snapshot)
}

const FIND_FILE_INPUT_NEAR_INDEX_JS: &str = r#"function(){
  const isFile = (el) => el && el.nodeType === 1 && el.tagName === 'INPUT' && String(el.type).toLowerCase() === 'file';
  let node = this;
  let depth = 0;
  const maxDepth = 5;
  while (node && depth <= maxDepth) {
    if (isFile(node)) return node;
    if (node.children) {
      for (let i = 0; i < node.children.length; i++) {
        const c = node.children[i];
        if (isFile(c)) return c;
      }
    }
    node = node.parentElement;
    depth++;
  }
  return null;
}"#;

fn expand_path_with_tilde(path_arg: &str) -> PathBuf {
    let t = path_arg.trim();
    if let Some(rest) = t.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(t)
}

/// Resolve a user/model path for [`upload_file_by_index`]. Relative paths are rooted at `browser_uploads_dir()`.
/// The result must be a non-empty regular file under `~/.mac-stats/uploads/` or `~/.mac-stats/screenshots/` (canonical prefix check).
pub fn resolve_browser_upload_source_path(path_arg: &str) -> Result<PathBuf, String> {
    let trimmed = path_arg.trim();
    if trimmed.is_empty() {
        return Err("missing file path after element index".to_string());
    }
    let raw = expand_path_with_tilde(trimmed);
    let path = if raw.is_absolute() {
        raw
    } else {
        crate::config::Config::browser_uploads_dir().join(raw)
    };
    let canon = path
        .canonicalize()
        .map_err(|e| format!("cannot access file (does it exist?): {}", e))?;
    let meta =
        std::fs::metadata(&canon).map_err(|e| format!("cannot read file metadata: {}", e))?;
    if !meta.is_file() {
        return Err("path is not a regular file".to_string());
    }
    if meta.len() == 0 {
        return Err("file is empty (0 bytes)".to_string());
    }
    let _ = std::fs::create_dir_all(crate::config::Config::browser_uploads_dir());
    let _ = std::fs::create_dir_all(crate::config::Config::screenshots_dir());
    let uploads_root = crate::config::Config::browser_uploads_dir();
    let shots_root = crate::config::Config::screenshots_dir();
    let uploads_canon = uploads_root
        .canonicalize()
        .map_err(|e| format!("uploads directory: {}", e))?;
    let shots_canon = shots_root
        .canonicalize()
        .map_err(|e| format!("screenshots directory: {}", e))?;
    if !canon.starts_with(&uploads_canon) && !canon.starts_with(&shots_canon) {
        return Err(
            "file must be under ~/.mac-stats/uploads/ or ~/.mac-stats/screenshots/ (no other paths)"
                .to_string(),
        );
    }
    Ok(canon)
}

fn file_input_indices_hint(tab: &headless_chrome::Tab) -> String {
    match get_interactables(tab) {
        Ok(list) => {
            let mut idxs: Vec<u32> = list
                .into_iter()
                .filter(|i| {
                    i.input_type
                        .as_deref()
                        .map(|t| t.eq_ignore_ascii_case("file"))
                        .unwrap_or(false)
                })
                .map(|i| i.index)
                .collect();
            if idxs.is_empty() {
                "No file inputs appear in the current Elements list.".to_string()
            } else {
                idxs.sort_unstable();
                format!(
                    "Try BROWSER_UPLOAD using one of these file-input indices from the Elements list: {}.",
                    idxs
                        .iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        Err(e) => format!("(Could not list file-input indices: {})", e),
    }
}

fn find_file_input_object_id(
    tab: &headless_chrome::Tab,
    start_object_id: &str,
) -> Result<String, String> {
    let res = tab
        .call_method(Runtime::CallFunctionOn {
            function_declaration: FIND_FILE_INPUT_NEAR_INDEX_JS.to_string(),
            object_id: Some(start_object_id.to_string()),
            arguments: None,
            return_by_value: Some(false),
            await_promise: Some(false),
            user_gesture: None,
            silent: Some(false),
            generate_preview: Some(false),
            execution_context_id: None,
            object_group: None,
            throw_on_side_effect: None,
            serialization_options: None,
            unique_context_id: None,
        })
        .map_err(|e| format!("find file input: {}", e))?;
    if res.exception_details.is_some() {
        return Err("find file input: JavaScript exception".to_string());
    }
    if let Some(id) = res.result.object_id.clone() {
        return Ok(id);
    }
    Err(String::new())
}

/// Set files on the nearest `<input type=file>` from the interactable at `index` via CDP `DOM.setFileInputFiles`.
/// `source_path` is validated by [`resolve_browser_upload_source_path`].
pub fn upload_file_by_index(index: u32, source_path: &Path) -> Result<String, String> {
    with_connection_retry(|| upload_file_by_index_inner(index, source_path))
}

fn upload_file_by_index_inner(index: u32, source_path: &Path) -> Result<String, String> {
    if index == 0 {
        return Err("BROWSER_UPLOAD index must be >= 1".to_string());
    }
    let abs = source_path
        .to_str()
        .ok_or_else(|| "BROWSER_UPLOAD: path is not valid UTF-8".to_string())?
        .to_string();
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_UPLOAD index={} file_len={}",
        index,
        abs.len()
    );
    let start_oid = resolve_interactable_object_id(&tab, index).map_err(|e| {
        let s = format!("BROWSER_UPLOAD: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    let file_oid = match find_file_input_object_id(&tab, &start_oid) {
        Ok(oid) => oid,
        Err(e) if e.is_empty() => {
            let hint = file_input_indices_hint(&tab);
            return Err(format!(
                "BROWSER_UPLOAD: no <input type=file> within 5 ancestor levels of element {} (checked self and immediate children at each level). {}",
                index, hint
            ));
        }
        Err(e) => {
            let s = format!("BROWSER_UPLOAD: {}", e);
            clear_browser_session_on_error(&s);
            return Err(s);
        }
    };
    tab.call_method(DOM::ScrollIntoViewIfNeeded {
        node_id: None,
        backend_node_id: None,
        object_id: Some(file_oid.clone()),
        rect: None,
    })
    .map_err(|e| {
        let s = format!("BROWSER_UPLOAD: scroll into view: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    tab.call_method(DOM::SetFileInputFiles {
        files: vec![abs],
        node_id: None,
        backend_node_id: None,
        object_id: Some(file_oid),
    })
    .map_err(|e| {
        let s = format!("BROWSER_UPLOAD: setFileInputFiles: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_UPLOAD setFileInputFiles OK index={}",
        index
    );
    std::thread::sleep(BROWSER_POST_UPLOAD_SETTLE);
    let state = get_browser_state(&tab).inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let snapshot = format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    Ok(snapshot)
}

/// Parse `BROWSER_KEYS` chord: `+`-separated tokens, modifiers first. Allowlisted only (no arbitrary strings).
fn parse_browser_keys_chord(arg: &str) -> Result<(Vec<ModifierKey>, String), String> {
    let raw = arg.trim();
    if raw.is_empty() {
        return Err(
            "BROWSER_KEYS requires a chord (e.g. BROWSER_KEYS: Escape or BROWSER_KEYS: Meta+f)."
                .to_string(),
        );
    }
    let parts: Vec<&str> = raw
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return Err("BROWSER_KEYS: empty chord.".to_string());
    }
    let n = parts.len();
    let mut mod_mask: u32 = 0;
    for part in parts.iter().take(n.saturating_sub(1)) {
        let bit = match part.to_ascii_lowercase().as_str() {
            "meta" => ModifierKey::Meta as u32,
            "ctrl" | "control" => ModifierKey::Ctrl as u32,
            "alt" => ModifierKey::Alt as u32,
            "shift" => ModifierKey::Shift as u32,
            other => {
                return Err(format!(
                    "BROWSER_KEYS: unknown modifier '{}'. Use Meta, Ctrl, Alt, or Shift (join with +, no spaces inside the chord).",
                    other
                ));
            }
        };
        mod_mask |= bit;
    }
    let mut modifiers = Vec::new();
    if mod_mask & (ModifierKey::Alt as u32) != 0 {
        modifiers.push(ModifierKey::Alt);
    }
    if mod_mask & (ModifierKey::Ctrl as u32) != 0 {
        modifiers.push(ModifierKey::Ctrl);
    }
    if mod_mask & (ModifierKey::Meta as u32) != 0 {
        modifiers.push(ModifierKey::Meta);
    }
    if mod_mask & (ModifierKey::Shift as u32) != 0 {
        modifiers.push(ModifierKey::Shift);
    }

    let key_raw = parts[n - 1];
    let key_lower = key_raw.to_ascii_lowercase();
    let main_key = match key_lower.as_str() {
        "enter" => "Enter".to_string(),
        "escape" | "esc" => "Escape".to_string(),
        "tab" => "Tab".to_string(),
        "backspace" => "Backspace".to_string(),
        "f5" => "F5".to_string(),
        _ => {
            if key_raw.len() == 1 {
                let c = key_raw.chars().next().unwrap();
                if c.is_ascii_alphabetic() {
                    if modifiers.is_empty() {
                        return Err(
                            "BROWSER_KEYS: a single letter requires at least one modifier (e.g. Meta+f) or use a named key (Enter, Escape, Tab, Backspace, F5)."
                                .to_string(),
                        );
                    }
                    c.to_ascii_lowercase().to_string()
                } else {
                    return Err(format!(
                        "BROWSER_KEYS: key '{}' is not allowlisted.",
                        key_raw
                    ));
                }
            } else {
                return Err(format!(
                    "BROWSER_KEYS: unknown key '{}'. Allowed: Enter, Escape, Tab, Backspace, F5, or one letter a–z with Meta/Ctrl/Alt/Shift.",
                    key_raw
                ));
            }
        }
    };
    Ok((modifiers, main_key))
}

/// Dispatch a keyboard chord on the **focused page** via CDP (no HTTP fallback). Returns formatted browser state.
pub fn dispatch_browser_keys(chord_spec: &str) -> Result<String, String> {
    with_connection_retry(|| dispatch_browser_keys_inner(chord_spec))
}

fn dispatch_browser_keys_inner(chord_spec: &str) -> Result<String, String> {
    let (modifiers, main_key) = parse_browser_keys_chord(chord_spec)?;
    let (browser, tab) = get_current_tab().inspect_err(|e| clear_browser_session_on_error(e))?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: BROWSER_KEYS main_key={} modifier_count={}",
        main_key,
        modifiers.len()
    );
    let mod_slice = if modifiers.is_empty() {
        None
    } else {
        Some(modifiers.as_slice())
    };
    tab.press_key_with_modifiers(main_key.as_str(), mod_slice)
        .map_err(|e| {
            let s = format!("BROWSER_KEYS failed: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    std::thread::sleep(Duration::from_millis(250));
    let state = get_browser_state(&tab).inspect_err(|e| clear_browser_session_on_error(e))?;
    let snapshot = format_browser_state_snapshot(&browser, &state);
    cache_cdp_interactable_tool_state(&state.interactables);
    set_last_browser_state_snapshot(snapshot.clone());
    Ok(snapshot)
}

/// One "page" step for BROWSER_SCROLL `down` / `up`: fraction of layout viewport height (see `BROWSER_LAYOUT_METRICS_JS`).
const BROWSER_SCROLL_VIEWPORT_FRACTION: f64 = 0.9;
const BROWSER_SCROLL_STEP_MIN_PX: i32 = 200;
const BROWSER_SCROLL_STEP_MAX_PX: i32 = 2000;
const BROWSER_SCROLL_METRICS_FALLBACK_PX: i32 = 500;

fn browser_scroll_step_px_from_layout_metrics(tab: &headless_chrome::Tab) -> i32 {
    match try_browser_layout_metrics(tab) {
        Some(m) if m.viewport_height > 0 => {
            let raw = (m.viewport_height as f64 * BROWSER_SCROLL_VIEWPORT_FRACTION).round() as i32;
            let step = raw.clamp(BROWSER_SCROLL_STEP_MIN_PX, BROWSER_SCROLL_STEP_MAX_PX);
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: BROWSER_SCROLL step from viewport height {} -> {}px",
                m.viewport_height,
                step
            );
            step
        }
        _ => {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: BROWSER_SCROLL layout metrics unavailable; using fallback {}px",
                BROWSER_SCROLL_METRICS_FALLBACK_PX
            );
            BROWSER_SCROLL_METRICS_FALLBACK_PX
        }
    }
}

/// Scroll the current page. Arg: "down", "up", "bottom", "top", or pixels (e.g. "500"). Returns updated browser state.
pub fn scroll_page(arg: &str) -> Result<String, String> {
    with_connection_retry(|| scroll_page_inner(arg))
}

fn scroll_page_inner(arg: &str) -> Result<String, String> {
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    let arg = arg.trim().to_lowercase();
    let scroll_js = if arg == "bottom" || arg == "end" {
        "window.scrollTo(0, document.body.scrollHeight); 'scrolled to bottom'".to_string()
    } else if arg == "top" || arg == "start" {
        "window.scrollTo(0, 0); 'scrolled to top'".to_string()
    } else if arg == "down" {
        let step = browser_scroll_step_px_from_layout_metrics(tab.as_ref());
        format!("window.scrollBy(0, {}); 'scrolled down {}px'", step, step)
    } else if arg == "up" {
        let step = browser_scroll_step_px_from_layout_metrics(tab.as_ref());
        format!("window.scrollBy(0, -{}); 'scrolled up {}px'", step, step)
    } else if let Ok(pixels) = arg.parse::<i32>() {
        let px = pixels.clamp(-10000, 10000);
        if px >= 0 {
            format!("window.scrollBy(0, {}); 'scrolled down {}px'", px, px)
        } else {
            format!("window.scrollBy(0, {}); 'scrolled up {}px'", px, -px)
        }
    } else {
        let step = browser_scroll_step_px_from_layout_metrics(tab.as_ref());
        format!("window.scrollBy(0, {}); 'scrolled down {}px'", step, step)
    };
    tab.evaluate(&scroll_js, false).map_err(|e| {
        let s = format!("Scroll evaluate: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    std::thread::sleep(BROWSER_POST_SCROLL_SETTLE);
    let state = get_browser_state(&tab).inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let snapshot = format_browser_state_snapshot(&browser, &state);
    set_last_browser_state_snapshot(snapshot.clone());
    Ok(snapshot)
}

/// Search current page for a text pattern (like grep). Returns matches with surrounding context and DOM paths. Zero LLM cost.
/// Use to find specific text, verify content exists, or locate data. Use after BROWSER_NAVIGATE/CLICK.
/// When `css_scope` is set, only text under the first element matching that selector is searched.
pub fn search_page_text(pattern: &str, css_scope: Option<&str>) -> Result<String, String> {
    with_connection_retry(|| search_page_text_inner(pattern, css_scope))
}

fn truncate_context_window(text: &str, start: usize, end: usize) -> String {
    let mut start_byte = start.min(text.len());
    while start_byte > 0 && !text.is_char_boundary(start_byte) {
        start_byte -= 1;
    }
    let mut end_byte = end.min(text.len());
    while end_byte < text.len() && !text.is_char_boundary(end_byte) {
        end_byte += 1;
    }
    text[start_byte..end_byte].replace(char::is_whitespace, " ")
}

fn search_page_text_from_plain_text(pattern: &str, text: &str) -> Result<String, String> {
    let normalized = text.trim();
    if normalized.is_empty() {
        return Ok(format!("No matches found for \"{}\" on page.", pattern));
    }
    let regex = regex::RegexBuilder::new(&regex::escape(pattern))
        .case_insensitive(true)
        .build()
        .map_err(|e| format!("Search regex build: {}", e))?;
    let matches: Vec<_> = regex.find_iter(normalized).collect();
    if matches.is_empty() {
        return Ok(format!("No matches found for \"{}\" on page.", pattern));
    }
    let mut lines = vec![format!(
        "Found {} match{} for \"{}\" on page:",
        matches.len(),
        if matches.len() == 1 { "" } else { "es" },
        pattern
    )];
    lines.push(String::new());
    for (i, m) in matches.iter().take(20).enumerate() {
        let context = truncate_context_window(
            normalized,
            m.start().saturating_sub(80),
            (m.end() + 80).min(normalized.len()),
        );
        lines.push(format!(
            "[{}] {}{}{}",
            i + 1,
            if m.start() > 0 { "..." } else { "" },
            context.trim(),
            if m.end() < normalized.len() {
                "..."
            } else {
                ""
            }
        ));
    }
    if matches.len() > 20 {
        lines.push(format!(
            "\n... showing {} of {} total matches.",
            20,
            matches.len()
        ));
    }
    Ok(lines.join("\n"))
}

fn search_page_text_inner(pattern: &str, css_scope: Option<&str>) -> Result<String, String> {
    let (_, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    if let Some(scope_sel) = css_scope.filter(|s| !s.trim().is_empty()) {
        mac_stats_debug!(
            "browser/cdp",
            "BROWSER_SEARCH_PAGE: css_scope={}",
            crate::logging::ellipse(scope_sel, 80)
        );
    }
    // Escape for JS string: backslash and quotes
    let pattern_escaped = pattern
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace(['\n', '\r'], " ");
    const CONTEXT_CHARS: i32 = 80;
    const MAX_RESULTS: i32 = 20;
    let scope_init = if let Some(sel) = css_scope.map(str::trim).filter(|s| !s.is_empty()) {
        let sel_json =
            serde_json::to_string(sel).map_err(|e| format!("css_scope encode: {}", e))?;
        format!(
            r#"var scopeRoot = document.querySelector({});
  if (!scopeRoot) return {{ error: 'css_scope matched no element', matches: [], total: 0 }};"#,
            sel_json
        )
    } else {
        "var scopeRoot = document.body;\n  if (!scopeRoot) return { error: 'no body', matches: [], total: 0 };"
            .to_string()
    };
    // Use indexOf for literal substring search (no regex escaping issues)
    let js = format!(
        r#"
(function() {{
  {}
  var walker = document.createTreeWalker(scopeRoot, NodeFilter.SHOW_TEXT, null);
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
        if (elemPath) elemPath = 'body > ' + elemPath;
        else elemPath = 'body';
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
        scope_init, pattern_escaped, MAX_RESULTS, CONTEXT_CHARS, CONTEXT_CHARS, MAX_RESULTS
    );
    let result = tab.evaluate(&js, false).map_err(|e| {
        let s = format!("Search page evaluate: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    let value = if let Some(value) = result.value.as_ref() {
        value
    } else {
        if css_scope.is_some_and(|s| !s.trim().is_empty()) {
            return Err(
                "BROWSER_SEARCH_PAGE: CDP returned no structured search result; cannot apply css_scope. Retry without css_scope or fix the selector."
                    .to_string(),
            );
        }
        let text = get_page_text(&tab).inspect_err(|e| {
            clear_browser_session_on_error(e);
        })?;
        return search_page_text_from_plain_text(pattern, &text);
    };
    let obj = value
        .as_object()
        .ok_or("search_page did not return object")?;
    if let Some(err) = obj.get("error").and_then(|v| v.as_str()) {
        return Err(format!("search_page error: {}", err));
    }
    let total = obj.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let empty: &[serde_json::Value] = &[];
    let matches = obj
        .get("matches")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(empty);
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
        let ctx = m.get("context").and_then(|v| v.as_str()).unwrap_or("?");
        let path = m.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let loc = if path.is_empty() {
            String::new()
        } else {
            format!(" [path: {}]", path)
        };
        lines.push(format!("[{}] {}{}", i + 1, ctx, loc));
    }
    let has_more = obj
        .get("has_more")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if has_more {
        lines.push(format!(
            "\n... showing {} of {} total matches.",
            matches.len(),
            total
        ));
    }
    Ok(lines.join("\n"))
}

const BROWSER_QUERY_MAX_RESULTS: i32 = 25;

/// Run `document.querySelectorAll` on the current page; returns tag, truncated text, child count, and requested attributes (`href`/`src` resolved to absolute URLs). Capped at [`BROWSER_QUERY_MAX_RESULTS`]. Zero LLM cost.
pub fn browser_query(selector: &str, attrs_csv: Option<&str>) -> Result<String, String> {
    with_connection_retry(|| browser_query_inner(selector, attrs_csv))
}

fn browser_query_inner(selector: &str, attrs_csv: Option<&str>) -> Result<String, String> {
    let sel = selector.trim();
    if sel.is_empty() {
        return Err("BROWSER_QUERY requires a non-empty CSS selector.".to_string());
    }
    let mut attr_names: Vec<String> = attrs_csv
        .map(|s| {
            s.split(',')
                .map(|a| a.trim().to_string())
                .filter(|a| !a.is_empty())
                .collect()
        })
        .unwrap_or_default();
    attr_names.truncate(16);
    let (_, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_debug!(
        "browser/cdp",
        "BROWSER_QUERY: selector={} attrs={}",
        crate::logging::ellipse(sel, 80),
        attr_names.len()
    );
    let sel_json = serde_json::to_string(sel).map_err(|e| format!("selector encode: {}", e))?;
    let attrs_json =
        serde_json::to_string(&attr_names).map_err(|e| format!("attrs encode: {}", e))?;
    let js = format!(
        r#"
(function() {{
  try {{
    var sel = {sel_json};
    var attrNames = {attrs_json};
    var max = {max};
    var nodes = document.querySelectorAll(sel);
    var out = [];
    for (var i = 0; i < nodes.length && out.length < max; i++) {{
      var el = nodes[i];
      var tag = el.tagName ? el.tagName.toLowerCase() : '';
      var text = (el.innerText || el.textContent || '').replace(/\s+/g, ' ').trim();
      if (text.length > 300) text = text.slice(0, 300) + '…';
      var childCount = el.children ? el.children.length : 0;
      var attrs = {{}};
      for (var j = 0; j < attrNames.length; j++) {{
        var an = attrNames[j];
        var v = el.getAttribute(an);
        if (v !== null && v !== '') {{
          if ((an === 'href' || an === 'src') && v) {{
            try {{ attrs[an] = new URL(v, document.baseURI).href; }} catch (e) {{ attrs[an] = v; }}
          }} else {{
            attrs[an] = v;
          }}
        }}
      }}
      out.push({{ tag: tag, text: text, attrs: attrs, children: childCount }});
    }}
    return {{ ok: true, elements: out, total: nodes.length }};
  }} catch (e) {{
    return {{ ok: false, error: String(e.message || e) }};
  }}
}})()
"#,
        sel_json = sel_json,
        attrs_json = attrs_json,
        max = BROWSER_QUERY_MAX_RESULTS
    );
    let result = tab.evaluate(&js, false).map_err(|e| {
        let s = format!("BROWSER_QUERY evaluate: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    let value = result
        .value
        .as_ref()
        .ok_or("BROWSER_QUERY: no result from page")?;
    let obj = value
        .as_object()
        .ok_or("BROWSER_QUERY: unexpected result shape")?;
    if !obj.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let err = obj
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(format!("BROWSER_QUERY failed: {}", err));
    }
    let total = obj.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let empty: &[serde_json::Value] = &[];
    let elements = obj
        .get("elements")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(empty);
    if elements.is_empty() {
        return Ok(format!(
            "BROWSER_QUERY: no elements matched selector {:?} (document has {} total matches).",
            crate::logging::ellipse(sel, 120),
            total
        ));
    }
    let mut lines = vec![
        format!(
            "BROWSER_QUERY: {} match(es) for selector {:?} (showing up to {}):",
            total,
            crate::logging::ellipse(sel, 120),
            BROWSER_QUERY_MAX_RESULTS
        ),
        String::new(),
    ];
    for (i, el) in elements.iter().enumerate() {
        let tag = el.get("tag").and_then(|v| v.as_str()).unwrap_or("?");
        let text = el.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let children = el.get("children").and_then(|v| v.as_i64()).unwrap_or(0);
        let attrs = el
            .get("attrs")
            .and_then(|v| v.as_object())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| {
                        format!(
                            "{}={}",
                            k,
                            v.as_str()
                                .map(|s| crate::logging::ellipse(s, 200))
                                .unwrap_or_else(|| "?".into())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let attr_part = if attrs.is_empty() {
            String::new()
        } else {
            format!(" | {}", attrs)
        };
        lines.push(format!(
            "[{}] <{}> children={}{} — {}",
            i + 1,
            tag,
            children,
            attr_part,
            text
        ));
    }
    if total > elements.len() as i64 {
        lines.push(format!(
            "\n... showing {} of {} elements.",
            elements.len(),
            total
        ));
    }
    Ok(lines.join("\n"))
}

/// Extract page content for **BROWSER_EXTRACT**: markdown from the live DOM (links, tables, optional images), with innerText fallback.
pub fn extract_page_text() -> Result<String, String> {
    extract_page_text_with_options(true)
}

/// Same as [`extract_page_text`] with control over `![alt](url)` lines (`no_images` tool arg).
pub fn extract_page_text_with_options(include_images: bool) -> Result<String, String> {
    with_connection_retry(|| extract_page_text_inner(include_images))
}

fn extract_page_text_inner(include_images: bool) -> Result<String, String> {
    let (_, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    if is_new_tab_or_blank(tab.get_url().as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    let extract_warn_prefix = {
        let u = tab.get_url();
        let title = tab_document_title_best_effort(tab.as_ref());
        if url_or_title_suggests_certificate_interstitial(u.as_str(), title.as_deref()) {
            mac_stats_warn!(
                "browser/cdp",
                "BROWSER_EXTRACT: page is chrome-error or TLS interstitial (host={})",
                host_for_navigation_log(u.as_str())
            );
            Some(
                "Warning: Chrome error or TLS interstitial; extracted text is not the real destination site.\n\n",
            )
        } else {
            None
        }
    };
    let extract_diag = if crate::config::Config::browser_include_diagnostics_in_state() {
        TabExtractDiagnosticsSession::try_start(Arc::clone(&tab))
    } else {
        None
    };
    const MAX_EXTRACT_CHARS: usize = 30_000;
    let js = build_page_markdown_extract_js(include_images);
    let eval_result = tab.evaluate(&js, false).map_err(|e| {
        let s = format!("Evaluate page markdown: {}", e);
        clear_browser_session_on_error(&s);
        s
    })?;
    let raw_json = eval_result
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let (md_ok, md, elements, text_chars, js_err) =
        match serde_json::from_str::<serde_json::Value>(raw_json) {
            Ok(v) => {
                let ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
                let md = v
                    .get("md")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let elements = v.get("elements").and_then(|x| x.as_u64()).unwrap_or(0);
                let text_chars = v.get("text_chars").and_then(|x| x.as_u64()).unwrap_or(0);
                let err = v
                    .get("error")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                (ok, md, elements, text_chars, err)
            }
            Err(e) => {
                mac_stats_warn!(
                    "browser/cdp",
                    "BROWSER_EXTRACT: markdown JSON parse failed ({}); falling back to innerText",
                    e
                );
                (false, String::new(), 0, 0, String::new())
            }
        };

    let mut text = if md_ok && !md.trim().is_empty() {
        mac_stats_info!(
            "browser/cdp",
            "BROWSER_EXTRACT: markdown ok elements={} text_chars={} include_images={} md_chars={}",
            elements,
            text_chars,
            include_images,
            md.chars().count()
        );
        md
    } else {
        if md_ok {
            mac_stats_warn!(
                "browser/cdp",
                "BROWSER_EXTRACT: markdown empty; falling back to innerText"
            );
        } else if !js_err.is_empty() {
            mac_stats_warn!(
                "browser/cdp",
                "BROWSER_EXTRACT: markdown JS error ({}); falling back to innerText",
                crate::logging::ellipse(&js_err, 200)
            );
        } else {
            mac_stats_warn!(
                "browser/cdp",
                "BROWSER_EXTRACT: markdown not used; falling back to innerText"
            );
        }
        get_page_text(&tab).inspect_err(|e| {
            clear_browser_session_on_error(e);
        })?
    };

    text = truncate_markdown_at_blocks(&text, MAX_EXTRACT_CHARS);

    if elements > 20 && text_chars < elements.saturating_mul(5) {
        text.push_str(
            "\n\nNote: this page may still be loading (many elements, very little text).",
        );
    }

    if extract_diag.is_some() {
        std::thread::sleep(DIAG_EXTRACT_TAIL_WAIT);
    }
    let diag_section = extract_diag
        .as_ref()
        .map(|d| d.section())
        .unwrap_or_default();
    if let Some(ref d) = extract_diag {
        let n_c = d.console.lock().map(|q| q.len()).unwrap_or(0);
        let n_e = d.exc.lock().map(|q| q.len()).unwrap_or(0);
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: BROWSER_EXTRACT diagnostics console_lines={} uncaught_exceptions={}",
            n_c,
            n_e
        );
    }

    let mut out = String::new();
    if let Some(p) = extract_warn_prefix {
        out.push_str(p);
    }
    out.push_str(&crate::commands::text_normalize::apply_untrusted_homoglyph_normalization(text));
    out.push_str(&diag_section);
    Ok(out)
}

fn tab_inner_width_css(tab: &headless_chrome::Tab) -> Option<f64> {
    let ro = tab
        .evaluate(
            "(function(){return Math.max(window.innerWidth||0,document.documentElement.clientWidth||0)})()",
            false,
        )
        .ok()?;
    ro.value
        .as_ref()
        .and_then(|v| v.as_f64())
        .filter(|x| x.is_finite() && *x > 1.0)
}

/// Take a screenshot of the current CDP tab (no navigation). Use after BROWSER_NAVIGATE + BROWSER_CLICK.
/// Saves the raw PNG to `~/.mac-stats/screenshots/<timestamp>_current.png`, writes
/// `<timestamp>_current-annotated.png` with interactable overlays, and returns the **annotated** path
/// (tool attachment / `screenshot:saved` event).
pub fn take_screenshot_current_page() -> Result<PathBuf, String> {
    with_connection_retry(take_screenshot_current_page_inner)
}

fn take_screenshot_current_page_inner() -> Result<PathBuf, String> {
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: take_screenshot_current_page (no navigation)"
    );
    let (_, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let final_url = tab.get_url();
    if is_new_tab_or_blank(final_url.as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: screenshotting current page: {}",
        final_url
    );
    tab_document_ready_liveness(&tab)?;
    std::thread::sleep(BROWSER_SCREENSHOT_STABILIZE_CURRENT);
    let png_data = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .map_err(|e| {
            let s = format!("Capture screenshot: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    artifact_limits::ensure_buffer_within_browser_artifact_cap(
        png_data.len(),
        "BROWSER_SCREENSHOT PNG",
    )?;
    let dir = crate::config::Config::screenshots_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create screenshots dir: {}", e))?;
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_current.png", ts);
    let path = artifact_atomic::write_bytes_atomic_same_dir(&dir, &filename, &png_data)
        .map_err(|e| format!("Write screenshot: {}", e))?;

    let dpr = dom_snapshot::tab_device_pixel_ratio(&tab);
    let vw = tab_inner_width_css(&tab);
    let interactables = get_interactables(&tab).unwrap_or_else(|e| {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: get_interactables after screenshot failed (annotation skipped): {}",
            e
        );
        Vec::new()
    });
    let viewport_w = vw.unwrap_or_else(|| {
        image::load_from_memory(&png_data)
            .map(|img| (img.width() as f64 / dpr).max(1.0))
            .unwrap_or(1024.0)
    });
    let out_path = screenshot_annotate::try_annotate_screenshot(
        &path,
        &png_data,
        &interactables,
        dpr,
        viewport_w,
    );

    crate::events::emit(
        "screenshot:saved",
        crate::events::EventPayload::ScreenshotSaved {
            path: out_path.clone(),
        },
    );
    Ok(out_path)
}

/// Export the **current** CDP tab to PDF via `Page.printToPDF` (after **BROWSER_NAVIGATE**, same session as screenshot).
/// Writes `~/.mac-stats/pdfs/<timestamp>_current.pdf`. Refuses in HTTP fetch fallback mode and on blank/new-tab pages.
pub fn save_print_pdf_current_page() -> Result<PathBuf, String> {
    with_connection_retry(save_print_pdf_current_page_inner)
}

fn save_print_pdf_current_page_inner() -> Result<PathBuf, String> {
    if http_fallback::browser_session_was_http_fallback() {
        mac_stats_info!(
            "browser/http_fallback",
            "BROWSER_SAVE_PDF refused: session is HTTP fallback (no print layout fidelity)"
        );
        return Err(
            "BROWSER_SAVE_PDF is not supported in HTTP fallback browser mode (no live Chrome print layout). Use BROWSER_NAVIGATE in CDP first so the page loads in Chrome, or use BROWSER_EXTRACT / BROWSER_SCREENSHOT instead. Do not assume a PDF was produced."
                .to_string(),
        );
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: save_print_pdf_current_page (no navigation)"
    );
    let (_, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let final_url = tab.get_url();
    if is_new_tab_or_blank(final_url.as_str()) {
        return Err(SESSION_RESET_MSG.to_string());
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: printToPDF current page: {}",
        final_url
    );
    tab_document_ready_liveness(&tab)?;
    std::thread::sleep(BROWSER_SCREENSHOT_STABILIZE_CURRENT);
    let print_bg = crate::config::Config::browser_print_pdf_background();
    let bytes = print_tab_to_pdf_bytes(&tab, print_bg, "BROWSER_SAVE_PDF").map_err(|e| {
        if !e.contains("browserArtifactMaxBytes") {
            clear_browser_session_on_error(&e);
        }
        if e.contains("browserArtifactMaxBytes") {
            format!(
                "{} For PDFs, try shorter pages (scroll + capture sections with BROWSER_SCREENSHOT), or raise browserArtifactMaxBytes in ~/.mac-stats/config.json.",
                e
            )
        } else {
            e
        }
    })?;
    let dir = crate::config::Config::pdfs_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create pdfs dir: {}", e))?;
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_current.pdf", ts);
    let path = artifact_atomic::write_bytes_atomic_same_dir(&dir, &filename, &bytes)
        .map_err(|e| format!("Write PDF: {}", e))?;
    mac_stats_info!(
        "browser/cdp",
        "BROWSER_SAVE_PDF wrote {} ({} bytes, print_background={})",
        path.display(),
        bytes.len(),
        print_bg
    );
    Ok(path)
}

/// Take a screenshot of the given URL using CDP (reuses session if within idle timeout, else connects or launches).
/// When url is empty or "current", screenshots the current tab (use after BROWSER_NAVIGATE + BROWSER_CLICK).
/// Saves the raw PNG to `~/.mac-stats/screenshots/<timestamp>_<domain>.png`, writes a matching
/// `-annotated.png` file, and returns the **annotated** path.
/// Browser session is kept until unused for Config::browser_idle_timeout_secs() (default 5 min; configurable).
pub fn take_screenshot(url: &str) -> Result<PathBuf, String> {
    with_connection_retry(|| take_screenshot_inner(url))
}

fn take_screenshot_inner(url: &str) -> Result<PathBuf, String> {
    let url_trimmed = url.trim();
    if url_trimmed.is_empty() || url_trimmed.eq_ignore_ascii_case("current") {
        return take_screenshot_current_page_inner();
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: take_screenshot called with url (raw): {:?}",
        url
    );
    let url_normalized =
        crate::commands::browser::normalize_and_validate_cdp_navigation_url(url_trimmed)?;
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: normalized URL: {}",
        url_normalized
    );

    crate::commands::browser::ssrf_proxy_env_notice_for_tool("BROWSER_SCREENSHOT (URL)")?; // after URL precheck

    // URL screenshots must respect CURRENT_TAB_INDEX (same tab as get_current_tab / BROWSER_NAVIGATE),
    // not tabs.first(), so multi-tab sessions navigate and capture the focused tab.
    let (browser, tab) = get_current_tab().inspect_err(|e| {
        clear_browser_session_on_error(e);
    })?;
    let focused_tab_index = current_tab_index()
        .lock()
        .map(|g| *g)
        .unwrap_or(0);
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: take_screenshot URL path: using focused tab index {} (get_current_tab / CURRENT_TAB_INDEX)",
        focused_tab_index
    );
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: navigating to: {}",
        url_normalized
    );
    let prev_url = tab.get_url();
    let nav_timeout_secs = crate::config::Config::browser_navigation_timeout_secs();
    tab.set_default_timeout(Duration::from_secs(nav_timeout_secs));
    let redirect_rws_buf = Arc::new(Mutex::new(VecDeque::new()));
    cdp_enable_network_for_redirect_chain_capture(tab.as_ref());
    let redirect_rws_weak =
        cdp_attach_redirect_chain_rws_listener(tab.as_ref(), Arc::clone(&redirect_rws_buf));
    let _redirect_rws_guard = CdpRedirectRwsListenerGuard {
        tab: tab.as_ref(),
        weak: redirect_rws_weak,
    };
    let (post_nav_net_flight, _post_nav_net_guard) =
        prepare_post_nav_network_idle_tracking(tab.as_ref());
    with_lifecycle_event_buffer(&tab, |buf_opt| {
        if let Some(b) = buf_opt {
            if let Ok(mut q) = b.lock() {
                q.clear();
            }
        }
        if let Ok(mut q) = redirect_rws_buf.lock() {
            q.clear();
        }
        let nav_start = Instant::now();
        tab.navigate_to(&url_normalized).map_err(|e| {
            let msg = e.to_string();
            let detail = navigate_failed_detail_from_display(&msg);
            log_navigation_cdp_failure(&url_normalized, &detail);
            let s = navigation_tool_result_for_failed_navigate(&url_normalized, &detail);
            clear_browser_session_on_error(&s);
            s
        })?;
        synchronize_tab_after_cdp_navigation(
            &tab,
            prev_url.as_str(),
            url_normalized.as_str(),
            buf_opt,
            nav_start,
            Duration::from_secs(nav_timeout_secs),
            nav_timeout_secs,
            None,
            post_nav_net_flight.as_ref(),
        )
    })?;
    if let Err(e) =
        cdp_validate_redirect_chain_from_rws_buffer(&redirect_rws_buf, url_normalized.as_str())
    {
        clear_browser_session_on_error(&e);
        return Err(e);
    }
    let final_url = tab.get_url();
    assert_final_document_url_ssrf_post_check(final_url.as_str(), Some(url_normalized.as_str()))?;
    if let Some(msg) = post_navigate_load_failure_message(
        url_normalized.as_str(),
        final_url.as_str(),
        Some(tab.as_ref()),
    ) {
        return Err(msg);
    }
    try_enforce_browser_tab_limit(&browser, &tab);
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: navigated; final tab URL: {}",
        final_url
    );
    if let Ok(title) = tab.evaluate("document.title", false) {
        let title_str = title
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("(none)");
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: page title: {}",
            title_str
        );
        if title_str.to_lowercase().contains("404")
            || title_str.to_lowercase().contains("not found")
        {
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: page appears to be 404 or not found"
            );
        }
    }
    std::thread::sleep(BROWSER_SCREENSHOT_STABILIZE_AFTER_URL_NAV);
    tab_document_ready_liveness(&tab)?;
    let png_data = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .map_err(|e| {
            let s = format!("Capture screenshot: {}", e);
            clear_browser_session_on_error(&s);
            s
        })?;
    artifact_limits::ensure_buffer_within_browser_artifact_cap(
        png_data.len(),
        "BROWSER_SCREENSHOT PNG",
    )?;
    let dir = crate::config::Config::screenshots_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create screenshots dir: {}", e))?;
    let host_segment = url_normalized
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("page");
    let domain = artifact_atomic::sanitize_untrusted_basename(host_segment, "page");
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.png", ts, domain);
    let path = artifact_atomic::write_bytes_atomic_same_dir(&dir, &filename, &png_data)
        .map_err(|e| format!("Write screenshot: {}", e))?;

    let dpr = dom_snapshot::tab_device_pixel_ratio(&tab);
    let vw = tab_inner_width_css(&tab);
    let interactables = get_interactables(&tab).unwrap_or_else(|e| {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: get_interactables after screenshot failed (annotation skipped): {}",
            e
        );
        Vec::new()
    });
    let viewport_w = vw.unwrap_or_else(|| {
        image::load_from_memory(&png_data)
            .map(|img| (img.width() as f64 / dpr).max(1.0))
            .unwrap_or(1024.0)
    });
    let out_path = screenshot_annotate::try_annotate_screenshot(
        &path,
        &png_data,
        &interactables,
        dpr,
        viewport_w,
    );

    crate::events::emit(
        "screenshot:saved",
        crate::events::EventPayload::ScreenshotSaved {
            path: out_path.clone(),
        },
    );
    Ok(out_path)
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

    #[test]
    fn search_page_text_from_plain_text_returns_no_matches_cleanly() {
        let result = search_page_text_from_plain_text("videos", "About us and services").unwrap();
        assert_eq!(result, "No matches found for \"videos\" on page.");
    }

    #[test]
    fn search_page_text_from_plain_text_returns_context_matches() {
        let result = search_page_text_from_plain_text(
            "videos",
            "About. Amvara's videos are featured on the about page.",
        )
        .unwrap();
        assert!(result.contains("Found 1 match"));
        assert!(result.contains("Amvara's videos"));
    }

    #[test]
    fn navigate_failed_detail_strips_headless_chrome_prefix() {
        assert_eq!(
            navigate_failed_detail_from_display("Navigate failed: net::ERR_NAME_NOT_RESOLVED"),
            "net::ERR_NAME_NOT_RESOLVED"
        );
    }

    #[test]
    fn chrome_error_url_detected_case_insensitive() {
        assert!(is_chrome_internal_error_document_url(
            "chrome-error://chromewebdata/"
        ));
    }

    #[test]
    fn post_navigate_ssrf_allows_chrome_error_and_about_blank() {
        assert!(assert_final_document_url_ssrf_post_check(
            "chrome-error://chromewebdata/",
            Some("https://example.com")
        )
        .is_ok());
        assert!(assert_final_document_url_ssrf_post_check("about:blank", None).is_ok());
        assert!(assert_final_document_url_ssrf_post_check("about:srcdoc", None).is_ok());
    }

    #[test]
    fn post_navigate_ssrf_rejects_loopback_http() {
        let r = assert_final_document_url_ssrf_post_check(
            "http://127.0.0.1/",
            Some("https://example.com"),
        );
        assert!(r.is_err(), "{:?}", r);
    }

    #[test]
    fn post_navigate_ssrf_rejects_file_and_javascript() {
        assert!(assert_final_document_url_ssrf_post_check("file:///etc/passwd", None).is_err());
        assert!(assert_final_document_url_ssrf_post_check("javascript:alert(1)", None).is_err());
    }

    #[test]
    fn post_navigate_ssrf_rejects_disallowed_about() {
        assert!(assert_final_document_url_ssrf_post_check("about:config", None).is_err());
    }

    #[test]
    fn redirect_chain_first_hop_matches_http_https_equivalent() {
        assert!(cdp_redirect_chain_first_hop_matches_request(
            "https://example.com/start",
            "http://example.com/start"
        ));
        assert!(!cdp_redirect_chain_first_hop_matches_request(
            "https://example.com/other",
            "http://example.com/start"
        ));
    }

    #[test]
    fn redirect_chain_extract_groups_same_loader_id() {
        let buf = vec![
            ("L1".to_string(), "https://a.example/start".to_string()),
            ("L1".to_string(), "https://b.example/next".to_string()),
            ("L2".to_string(), "https://c.example/".to_string()),
        ];
        let chain =
            cdp_extract_document_redirect_chain_from_rws_buffer(&buf, "http://a.example/start");
        assert_eq!(chain.as_ref().map(|c| c.len()), Some(2));
        assert!(chain.unwrap()[1].contains("b.example"));
    }

    #[test]
    fn redirect_chain_validate_blocks_private_hop_after_public_first() {
        let buf = Mutex::new(VecDeque::new());
        {
            let mut q = buf.lock().unwrap();
            q.push_back(("L9".to_string(), "https://example.com/start".to_string()));
            q.push_back(("L9".to_string(), "http://127.0.0.1/secret".to_string()));
        }
        let r = cdp_validate_redirect_chain_from_rws_buffer(&buf, "https://example.com/start");
        let err = r.expect_err("loopback hop must fail SSRF");
        assert!(
            err.contains("redirect hop") && err.contains("SSRF"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn redirect_chain_uncorrelated_buffer_skips_hop_validation() {
        let buf = Mutex::new(VecDeque::new());
        {
            let mut q = buf.lock().unwrap();
            q.push_back(("L1".to_string(), "https://other.example/".to_string()));
        }
        assert!(cdp_validate_redirect_chain_from_rws_buffer(&buf, "https://nope.example/")
            .is_ok());
    }

    #[test]
    fn new_tab_page_url_matches_browser_use_set() {
        assert!(is_new_tab_page_url("about:blank"));
        assert!(is_new_tab_page_url("chrome://new-tab-page"));
        assert!(is_new_tab_page_url("chrome://new-tab-page/"));
        assert!(is_new_tab_page_url("chrome://newtab"));
        assert!(is_new_tab_page_url("chrome://newtab/"));
        assert!(is_new_tab_page_url("Chrome://newtab"));
        assert!(is_new_tab_page_url("  chrome://new-tab-page/  "));
        assert!(!is_new_tab_page_url("https://example.com"));
        assert!(!is_new_tab_page_url("chrome://settings"));
        assert!(!is_new_tab_page_url(""));
        assert!(!is_new_tab_page_url("About:blank"));
    }

    #[test]
    fn format_browser_state_prepends_new_tab_hint() {
        let state = BrowserState {
            current_url: "chrome://new-tab-page/".to_string(),
            page_title: Some("New Tab".to_string()),
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: None,
        };
        let out = format_browser_state_for_llm(&state);
        assert!(out.starts_with("Note: tab is on a browser new-tab"));
        assert!(out.contains("Current page: chrome://new-tab-page/"));
    }

    #[test]
    fn format_browser_state_prepends_chrome_error_hint() {
        let state = BrowserState {
            current_url: "chrome-error://chromewebdata/".to_string(),
            page_title: None,
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: None,
        };
        let out = format_browser_state_for_llm(&state);
        assert!(out.starts_with("Warning: Chrome error or TLS interstitial"));
        assert!(out.contains("Current page: chrome-error://"));
    }

    #[test]
    fn scale_click_coords_scales_from_llm_image_to_viewport() {
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(None);
        record_viewport_css_for_llm_coord_scaling(1920, 1080);
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(Some((1400, 850)));
        let (vx, vy) = scale_click_coords_from_llm_screenshot_space(700.0, 425.0);
        assert!((vx - 960.0).abs() < 0.01, "vx={vx}");
        assert!((vy - 540.0).abs() < 0.01, "vy={vy}");
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(None);
        record_viewport_css_for_llm_coord_scaling(0, 0);
    }

    #[test]
    fn scale_click_coords_pass_through_when_no_llm_resize_dims() {
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(None);
        record_viewport_css_for_llm_coord_scaling(1920, 1080);
        let (x, y) = scale_click_coords_from_llm_screenshot_space(123.0, 456.0);
        assert_eq!((x, y), (123.0, 456.0));
    }

    #[test]
    fn format_browser_state_prepends_viewport_document_scroll_before_current_page() {
        let state = BrowserState {
            current_url: "https://example.com/long".to_string(),
            page_title: None,
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: Some(BrowserLayoutMetrics {
                scroll_x: 0,
                scroll_y: 120,
                viewport_width: 1280,
                viewport_height: 720,
                document_width: 1280,
                document_height: 2400,
            }),
        };
        let out = format_browser_state_for_llm(&state);
        let idx_vp = out.find("Viewport: 1280x720\n").expect("viewport line");
        let idx_doc = out.find("Document: 1280x2400\n").expect("document line");
        let idx_scroll = out.find("Scroll: (0, 120)\n").expect("scroll line");
        let idx_cur = out.find("Current page:").expect("current page");
        assert!(idx_vp < idx_doc && idx_doc < idx_scroll && idx_scroll < idx_cur);
    }

    #[test]
    fn format_browser_state_includes_recent_js_dialogs_section() {
        clear_cdp_js_dialog_history();
        let state = BrowserState {
            current_url: "https://example.com".to_string(),
            page_title: None,
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: None,
        };
        let out_empty = format_browser_state_for_llm(&state);
        assert!(out_empty.contains("Recent JS dialogs:\nNone\n"));

        record_cdp_js_dialog_dismissed(&DialogType::Alert, "token-xyz");
        let out = format_browser_state_for_llm(&state);
        assert!(out.contains("Recent JS dialogs:\n"));
        assert!(out.contains("[alert] token-xyz"));
        clear_cdp_js_dialog_history();
    }

    #[test]
    fn post_navigate_detects_chrome_error_document() {
        let m = post_navigate_load_failure_message(
            "https://example.com",
            "chrome-error://chromewebdata/",
            None,
        );
        assert!(m.is_some());
        assert!(m.unwrap().contains("Navigation failed"));
    }

    #[test]
    fn tls_cert_error_text_maps_to_tls_tool_message() {
        let detail = "net::ERR_CERT_AUTHORITY_INVALID";
        assert_eq!(
            tls_certificate_error_class_from_chrome_detail(detail),
            Some("cert_authority")
        );
        let msg = navigation_tool_result_for_failed_navigate("https://bad.example/", detail);
        assert!(msg.contains("TLS/certificate"));
        assert!(msg.contains("FETCH_URL"));
    }

    #[test]
    fn sanitize_navigation_error_redacts_user_path() {
        let s = sanitize_navigation_error_for_llm("net::ERR_FAILED /Users/alice/secret/file");
        assert!(!s.contains("/Users/alice"));
        assert!(s.contains("[path]"));
    }

    #[test]
    fn parse_browser_keys_accepts_escape_and_meta_f() {
        let (m, k) = parse_browser_keys_chord("Escape").unwrap();
        assert!(m.is_empty());
        assert_eq!(k, "Escape");
        let (m, k) = parse_browser_keys_chord("Meta+f").unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(k, "f");
        let (m, k) = parse_browser_keys_chord("Ctrl+Shift+Enter").unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(k, "Enter");
    }

    #[test]
    fn parse_browser_keys_rejects_plain_letter_and_unknown() {
        assert!(parse_browser_keys_chord("f").is_err());
        assert!(parse_browser_keys_chord("Meta+F12").is_err());
        assert!(parse_browser_keys_chord("Win+f").is_err());
    }

    #[test]
    fn truncate_markdown_at_blocks_unchanged_when_under_limit() {
        let s = "one\n\ntwo";
        assert_eq!(truncate_markdown_at_blocks(s, 100), s);
    }

    #[test]
    fn truncate_markdown_at_blocks_drops_tail_blocks() {
        let s = "a\n\nbbbb\n\ncccccccccc";
        let out = truncate_markdown_at_blocks(s, 8);
        assert!(out.starts_with("a\n\nbbbb"));
        assert!(out.contains("[Truncated:"));
        assert!(!out.contains("cccccccccc"));
    }

    fn sample_interactable(
        index: u32,
        tag: &str,
        accessible_name: Option<&str>,
        text: &str,
    ) -> Interactable {
        Interactable {
            index,
            tag: tag.to_string(),
            text: text.to_string(),
            href: None,
            placeholder: None,
            input_type: None,
            contenteditable: false,
            datepicker_like: false,
            accessible_name: accessible_name.map(String::from),
            ax_role: None,
            backend_dom_node_id: None,
            dom_name: None,
            aria_label: None,
            bounds_css: None,
            annot_bounds_css: None,
            from_subframe: false,
            covered: false,
        }
    }

    #[test]
    fn interactable_identity_remap_after_order_swap() {
        let stale_target = sample_interactable(2, "button", Some("Keep"), "Keep");
        let fresh = vec![
            sample_interactable(1, "button", Some("Keep"), "Keep"),
            sample_interactable(2, "button", Some("Transient"), "Transient"),
        ];
        assert_eq!(
            super::find_unique_identity_match(&stale_target, &fresh).unwrap(),
            1
        );
    }

    #[test]
    fn interactable_identity_ambiguous_returns_error() {
        let stale = sample_interactable(1, "button", Some("Go"), "Go");
        let fresh = vec![
            sample_interactable(1, "button", Some("Go"), "Go"),
            sample_interactable(2, "button", Some("Go"), "Go"),
        ];
        assert!(super::find_unique_identity_match(&stale, &fresh).is_err());
    }
}
