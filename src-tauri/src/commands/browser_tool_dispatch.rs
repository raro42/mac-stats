//! Browser tool dispatch handlers for the agent router tool loop.
//!
//! Extracted from `commands/ollama.rs` to keep modules small and cohesive.

use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;
use tracing::info;

use crate::commands::browser_helpers::{
    append_latest_browser_state_guidance, extract_browser_navigation_target,
    should_use_http_fallback_after_browser_action_error,
};
use crate::config::Config;

pub(crate) struct BrowserScreenshotResult {
    pub message: String,
    pub attachment_path: Option<PathBuf>,
}

#[derive(Debug)]
enum BrowserClickTarget {
    Index(u32),
    Coordinates { x: f64, y: f64 },
}

fn browser_click_coord_x_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)coordinate_x\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)").expect("coord_x regex")
    })
}

fn browser_click_coord_y_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)coordinate_y\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)").expect("coord_y regex")
    })
}

fn browser_search_css_scope_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bcss_scope\s*[:=]\s*(.+)$").expect("css_scope regex")
    })
}

/// Text pattern and optional subtree selector for `BROWSER_SEARCH_PAGE`.
fn parse_browser_search_page_arg(arg: &str) -> (String, Option<String>) {
    let trimmed = arg.trim();
    if let Some(caps) = browser_search_css_scope_re().captures(trimmed) {
        let scope = caps.get(1).and_then(|m| {
            let t = m.as_str().trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        });
        if let Some(m0) = caps.get(0) {
            let pattern_part = trimmed[..m0.start()].trim().to_string();
            return (pattern_part, scope);
        }
    }
    (trimmed.to_string(), None)
}

fn browser_query_attrs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\s+attrs\s*[:=]\s*(.+)$").expect("attrs regex"))
}

/// CSS selector and optional comma-separated attribute names for `BROWSER_QUERY`.
fn parse_browser_query_arg(arg: &str) -> (String, Option<String>) {
    let trimmed = arg.trim();
    if let Some(caps) = browser_query_attrs_re().captures(trimmed) {
        let attrs = caps.get(1).and_then(|m| {
            let t = m.as_str().trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        });
        if let Some(m0) = caps.get(0) {
            let sel = trimmed[..m0.start()].trim().to_string();
            return (sel, attrs);
        }
    }
    (trimmed.to_string(), None)
}

fn parse_browser_click_arg(arg: &str) -> Result<BrowserClickTarget, String> {
    let s = arg.trim();
    let x_m = browser_click_coord_x_re().captures(s);
    let y_m = browser_click_coord_y_re().captures(s);
    if let (Some(xc), Some(yc)) = (x_m, y_m) {
        let x = xc
            .get(1)
            .ok_or_else(|| "coordinate_x value missing".to_string())?
            .as_str()
            .parse::<f64>()
            .map_err(|_| "coordinate_x must be a number".to_string())?;
        let y = yc
            .get(1)
            .ok_or_else(|| "coordinate_y value missing".to_string())?
            .as_str()
            .parse::<f64>()
            .map_err(|_| "coordinate_y must be a number".to_string())?;
        return Ok(BrowserClickTarget::Coordinates { x, y });
    }
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() >= 3 && tokens[0].eq_ignore_ascii_case("coords") {
        let x = tokens[1]
            .parse::<f64>()
            .map_err(|_| "coords: invalid x".to_string())?;
        let y = tokens[2]
            .parse::<f64>()
            .map_err(|_| "coords: invalid y".to_string())?;
        return Ok(BrowserClickTarget::Coordinates { x, y });
    }
    let first = tokens.first().copied().unwrap_or("");
    let idx = first.parse::<u32>().map_err(|_| {
        "BROWSER_CLICK requires a 1-based element index (e.g. BROWSER_CLICK: 3) or pixel coordinates relative to the screenshot shown to the vision model (e.g. BROWSER_CLICK: coordinate_x=120 coordinate_y=340, or BROWSER_CLICK: coords 120 340).".to_string()
    })?;
    if idx == 0 {
        return Err("BROWSER_CLICK index must be >= 1.".to_string());
    }
    Ok(BrowserClickTarget::Index(idx))
}

fn send_status(tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>, msg: &str) {
    if let Some(tx) = tx {
        let _ = tx.send(msg.to_string());
    }
}

fn browser_tools_disabled_guard() -> Option<String> {
    if Config::browser_tools_enabled() {
        return None;
    }

    static LOGGED: OnceLock<()> = OnceLock::new();
    LOGGED.get_or_init(|| {
        // Avoid leaking URLs/targets; this message is stable and policy-friendly.
        crate::mac_stats_info!(
            "browser/tools_disabled",
            "Browser tools disabled in config (browserToolsEnabled=false)"
        );
    });

    Some("Browser tools disabled in config".to_string())
}

fn nav_url_changed_hint_if_navigation_timeout(err: &str) -> Option<bool> {
    // Error returned by `browser_agent::navigate_and_get_state_inner`.
    if err.contains("Navigation failed: timeout after") {
        crate::browser_agent::take_last_nav_timeout_url_changed_hint()
    } else {
        None
    }
}

fn append_browser_readiness_context(
    base: String,
    cdp_used: bool,
    nav_url_changed: Option<bool>,
) -> String {
    if let Some(ctx) = crate::browser_agent::format_last_browser_error_context(
        cdp_used,
        nav_url_changed,
    ) {
        crate::mac_stats_debug!(
            "browser/tools",
            "Browser tool error context: {}",
            crate::logging::ellipse(&ctx, 200)
        );
        format!("{}\n{}", base, ctx)
    } else {
        base
    }
}

pub(crate) async fn handle_browser_screenshot(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> BrowserScreenshotResult {
    if let Some(msg) = browser_tools_disabled_guard() {
        return BrowserScreenshotResult {
            message: msg,
            attachment_path: None,
        };
    }

    let url_arg = arg.trim().to_string();
    let is_current = url_arg.is_empty() || url_arg.eq_ignore_ascii_case("current");
    if !is_current {
        info!(
            "Agent router: rejecting BROWSER_SCREENSHOT: {} — use NAVIGATE first, then SCREENSHOT: current",
            crate::logging::ellipse(&url_arg, 60)
        );
        BrowserScreenshotResult {
            message: format!(
                "BROWSER_SCREENSHOT only works on the current page. Use BROWSER_NAVIGATE: {} first, then BROWSER_SCREENSHOT: current. Never use BROWSER_SCREENSHOT: <url>.",
                url_arg
            ),
            attachment_path: None,
        }
    } else {
        send_status(status_tx, "📸 Taking screenshot of current page");
        match tokio::task::spawn_blocking(crate::browser_agent::take_screenshot_current_page).await
        {
            Ok(Ok(path)) => {
                BrowserScreenshotResult {
                    message: format!(
                        "Screenshot of current page saved to: {}.\n\nTell the user the screenshot was taken; the app will attach it in Discord.",
                        path.display()
                    ),
                    attachment_path: Some(path),
                }
            }
            Ok(Err(e)) => {
                info!(
                    "Agent router [{}]: BROWSER_SCREENSHOT (current) failed: {}",
                    request_id,
                    crate::logging::ellipse(&e, 200)
                );
                let base_msg = format!(
                    "Screenshot of current page failed: {}. (Use BROWSER_NAVIGATE and BROWSER_CLICK first with CDP; then BROWSER_SCREENSHOT: current. Chromium may need to be listening on the configured CDP port, default 9222 — see browserCdpPort in ~/.mac-stats/config.json.)",
                    e
                );
                let msg = append_browser_readiness_context(base_msg, true, None);
                BrowserScreenshotResult {
                    message: msg,
                    attachment_path: None,
                }
            }
            Err(e) => BrowserScreenshotResult {
                message: format!("Screenshot task error: {}", e),
                attachment_path: None,
            },
        }
    }
}

pub(crate) async fn handle_browser_save_pdf(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> BrowserScreenshotResult {
    if let Some(msg) = browser_tools_disabled_guard() {
        return BrowserScreenshotResult {
            message: msg,
            attachment_path: None,
        };
    }

    let url_arg = arg.trim().to_string();
    let is_current = url_arg.is_empty() || url_arg.eq_ignore_ascii_case("current");
    if !is_current {
        info!(
            "Agent router: rejecting BROWSER_SAVE_PDF: {} — use NAVIGATE first, then BROWSER_SAVE_PDF: current",
            crate::logging::ellipse(&url_arg, 60)
        );
        BrowserScreenshotResult {
            message: format!(
                "BROWSER_SAVE_PDF only exports the **current** CDP tab (same safety model as BROWSER_SCREENSHOT). Use BROWSER_NAVIGATE: {} first, then BROWSER_SAVE_PDF: current. Do not pass a raw URL to BROWSER_SAVE_PDF — that would bypass the same navigate-time checks.",
                url_arg
            ),
            attachment_path: None,
        }
    } else {
        send_status(status_tx, "📄 Saving current page as PDF…");
        match tokio::task::spawn_blocking(crate::browser_agent::save_print_pdf_current_page).await {
            Ok(Ok(path)) => BrowserScreenshotResult {
                message: format!(
                    "PDF of current page saved to: {}.\n\nTell the user the PDF was written; the app may attach it for Discord when the channel allows PDFs. Path is always returned here even if the channel cannot attach PDF.",
                    path.display()
                ),
                attachment_path: Some(path),
            },
            Ok(Err(e)) => {
                info!(
                    "Agent router [{}]: BROWSER_SAVE_PDF (current) failed: {}",
                    request_id,
                    crate::logging::ellipse(&e, 200)
                );
                let base_msg = format!(
                    "PDF export of current page failed: {}. (Use BROWSER_NAVIGATE (CDP) first; HTTP-only fallback cannot print. Then BROWSER_SAVE_PDF: current. Chromium must listen on the configured CDP port — see browserCdpPort in ~/.mac-stats/config.json.)",
                    e
                );
                let msg = append_browser_readiness_context(base_msg, true, None);
                BrowserScreenshotResult {
                    message: msg,
                    attachment_path: None,
                }
            }
            Err(e) => BrowserScreenshotResult {
                message: format!("PDF export task error: {}", e),
                attachment_path: None,
            },
        }
    }
}

pub(crate) async fn handle_browser_navigate(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> (String, Vec<PathBuf>) {
    if let Some(msg) = browser_tools_disabled_guard() {
        return (msg, vec![]);
    }

    let raw_arg = arg.trim().to_string();
    if raw_arg.is_empty() {
        return (
            "BROWSER_NAVIGATE requires a URL (e.g. BROWSER_NAVIGATE: https://www.example.com). Please try again with a URL.".to_string(),
            vec![],
        );
    }
    if let Some(url_arg) = extract_browser_navigation_target(&raw_arg) {
        let new_tab = raw_arg
            .split_whitespace()
            .any(|w| w.eq_ignore_ascii_case("new_tab"));
        send_status(
            status_tx,
            &format!(
                "🧭 Navigating to {}…{}",
                url_arg,
                if new_tab { " (new tab)" } else { "" }
            ),
        );
        info!(
            "Agent router [{}]: BROWSER_NAVIGATE: URL sent to CDP: {} new_tab={}",
            request_id, url_arg, new_tab
        );
        match tokio::task::spawn_blocking({
            let u = url_arg.clone();
            move || crate::browser_agent::navigate_and_get_state_with_options_and_downloads(&u, new_tab)
        })
        .await
        {
            Ok(Ok((state_str, dls))) => (state_str, dls),
            Ok(Err(cdp_err)) => {
                let cdp_port = crate::config::Config::browser_cdp_port();
                info!(
                    "BROWSER_NAVIGATE CDP failed, ensuring Chromium on port {} and retrying: {}",
                    cdp_port,
                    crate::logging::ellipse(&cdp_err, 120)
                );
                tokio::task::spawn_blocking(move || {
                    crate::browser_agent::ensure_chrome_on_port(cdp_port)
                })
                    .await
                    .ok();
                match tokio::task::spawn_blocking({
                    let u = url_arg.clone();
                    move || crate::browser_agent::navigate_and_get_state_with_options_and_downloads(&u, new_tab)
                })
                .await
                {
                    Ok(Ok((state_str, dls))) => (state_str, dls),
                    Ok(Err(cdp_err2)) => {
                        info!(
                            "BROWSER_NAVIGATE CDP retry failed, trying HTTP fallback: {}",
                            crate::logging::ellipse(&cdp_err2, 120)
                        );
                        match tokio::task::spawn_blocking(move || {
                            crate::browser_agent::navigate_http(&url_arg)
                        })
                        .await
                        {
                            Ok(Ok(state_str)) => (state_str, vec![]),
                            Ok(Err(http_err)) => {
                                let nav_url_changed =
                                    nav_url_changed_hint_if_navigation_timeout(&cdp_err2);
                                let base = format!(
                                    "BROWSER_NAVIGATE failed (CDP: {}). HTTP fallback also failed: {}",
                                    crate::logging::ellipse(&cdp_err2, 80),
                                    http_err
                                );
                                (
                                    append_browser_readiness_context(base, false, nav_url_changed),
                                    vec![],
                                )
                            }
                            Err(e) => (
                                format!("BROWSER_NAVIGATE HTTP fallback task error: {}", e),
                                vec![],
                            ),
                        }
                    }
                    Err(e) => (format!("BROWSER_NAVIGATE CDP retry task error: {}", e), vec![]),
                }
            }
            Err(e) => (format!("BROWSER_NAVIGATE task error: {}", e), vec![]),
        }
    } else {
        (
            append_latest_browser_state_guidance(&format!(
                "BROWSER_NAVIGATE requires a concrete URL. The step {:?} was not executed because it did not contain a grounded browser target. This was an agent planning/parsing issue, not evidence about the site.",
                raw_arg
            )),
            vec![],
        )
    }
}

pub(crate) async fn handle_browser_go_back(
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    send_status(status_tx, "🔙 Going back…");
    info!("Agent router [{}]: BROWSER_GO_BACK", request_id);
    match tokio::task::spawn_blocking(crate::browser_agent::go_back).await {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            let base = format!("BROWSER_GO_BACK failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            append_latest_browser_state_guidance(&base)
        }
        Err(e) => format!("BROWSER_GO_BACK task error: {}", e),
    }
}

pub(crate) async fn handle_browser_go_forward(
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    send_status(status_tx, "⏩ Going forward…");
    info!("Agent router [{}]: BROWSER_GO_FORWARD", request_id);
    match tokio::task::spawn_blocking(crate::browser_agent::go_forward).await {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            let base = format!("BROWSER_GO_FORWARD failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            append_latest_browser_state_guidance(&base)
        }
        Err(e) => format!("BROWSER_GO_FORWARD task error: {}", e),
    }
}

/// Optional arg: `nocache` or `hard` (case-insensitive) for cache-bypass reload; empty for normal refresh.
pub(crate) async fn handle_browser_reload(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let tok = arg.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    let ignore_cache = matches!(tok.as_str(), "nocache" | "hard" | "bypass");
    let status_msg = if ignore_cache {
        "🔄 Reloading page (cache bypass)…"
    } else {
        "🔄 Reloading page…"
    };
    send_status(status_tx, status_msg);
    info!(
        "Agent router [{}]: BROWSER_RELOAD ignore_cache={}",
        request_id, ignore_cache
    );
    match tokio::task::spawn_blocking(move || {
        crate::browser_agent::reload_current_tab(ignore_cache)
    })
    .await
    {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            let base = format!("BROWSER_RELOAD failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            append_latest_browser_state_guidance(&base)
        }
        Err(e) => format!("BROWSER_RELOAD task error: {}", e),
    }
}

pub(crate) async fn handle_browser_clear_cookies(
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    send_status(status_tx, "🍪 Clearing persisted and in-browser cookies…");
    info!("Agent router [{}]: BROWSER_CLEAR_COOKIES", request_id);
    match tokio::task::spawn_blocking(crate::browser_agent::clear_browser_auth_storage).await {
        Ok(Ok(msg)) => msg,
        Ok(Err(e)) => {
            let base = format!("BROWSER_CLEAR_COOKIES failed: {}", e);
            append_browser_readiness_context(base, true, None)
        }
        Err(e) => format!("BROWSER_CLEAR_COOKIES task error: {}", e),
    }
}

pub(crate) async fn handle_browser_switch_tab(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let trimmed = arg.trim();
    let index = match trimmed.parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            return append_latest_browser_state_guidance(
                "BROWSER_SWITCH_TAB requires a 0-based tab index matching the Tabs line in the latest browser state (e.g. BROWSER_SWITCH_TAB: 0).",
            );
        }
    };
    send_status(status_tx, &format!("🗂️ Switching to tab {}…", index));
    info!(
        "Agent router [{}]: BROWSER_SWITCH_TAB index={}",
        request_id, index
    );
    match tokio::task::spawn_blocking(move || crate::browser_agent::switch_tab_to_index(index))
        .await
    {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            let base = format!("BROWSER_SWITCH_TAB failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            append_latest_browser_state_guidance(&base)
        }
        Err(e) => format!("BROWSER_SWITCH_TAB task error: {}", e),
    }
}

pub(crate) async fn handle_browser_close_tab(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let trimmed = arg.trim();
    let index = match trimmed.parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            return append_latest_browser_state_guidance(
                "BROWSER_CLOSE_TAB requires a 0-based tab index matching the Tabs line (e.g. BROWSER_CLOSE_TAB: 1). Cannot close the last remaining tab.",
            );
        }
    };
    send_status(status_tx, &format!("🗑️ Closing tab {}…", index));
    info!(
        "Agent router [{}]: BROWSER_CLOSE_TAB index={}",
        request_id, index
    );
    match tokio::task::spawn_blocking(move || crate::browser_agent::close_tab_at_index(index))
        .await
    {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            let base = format!("BROWSER_CLOSE_TAB failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            append_latest_browser_state_guidance(&base)
        }
        Err(e) => format!("BROWSER_CLOSE_TAB task error: {}", e),
    }
}

fn parse_browser_hover_arg(arg: &str) -> Result<u32, String> {
    let first = arg.trim().split_whitespace().next().unwrap_or("");
    let idx = first.parse::<u32>().map_err(|_| {
        "BROWSER_HOVER requires a 1-based element index (e.g. BROWSER_HOVER: 4).".to_string()
    })?;
    if idx == 0 {
        return Err("BROWSER_HOVER index must be >= 1.".to_string());
    }
    Ok(idx)
}

fn parse_browser_drag_arg(arg: &str) -> Result<(u32, u32), String> {
    let mut it = arg.trim().split_whitespace();
    let a = it.next().ok_or_else(|| {
        "BROWSER_DRAG requires two 1-based indices: BROWSER_DRAG: <from_index> <to_index>."
            .to_string()
    })?;
    let b = it.next().ok_or_else(|| {
        "BROWSER_DRAG requires two 1-based indices: BROWSER_DRAG: <from_index> <to_index>."
            .to_string()
    })?;
    let from = a
        .parse::<u32>()
        .map_err(|_| "BROWSER_DRAG: from_index must be a positive integer.".to_string())?;
    let to = b
        .parse::<u32>()
        .map_err(|_| "BROWSER_DRAG: to_index must be a positive integer.".to_string())?;
    if from == 0 || to == 0 {
        return Err("BROWSER_DRAG indices must be >= 1.".to_string());
    }
    Ok((from, to))
}

pub(crate) async fn handle_browser_click(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> (String, Vec<PathBuf>) {
    if let Some(msg) = browser_tools_disabled_guard() {
        return (msg, vec![]);
    }

    let target = match parse_browser_click_arg(arg) {
        Ok(t) => t,
        Err(e) => return (append_latest_browser_state_guidance(&e), vec![]),
    };

    match target {
        BrowserClickTarget::Coordinates { x, y } => {
            let (sx, sy) = crate::browser_agent::scale_click_coords_from_llm_screenshot_space(x, y);
            send_status(
                status_tx,
                &format!("🖱️ Clicking at ({:.0}, {:.0}) in viewport…", sx, sy),
            );
            info!(
                "Agent router [{}]: BROWSER_CLICK coordinates model=({:.1},{:.1}) viewport=({:.1},{:.1})",
                request_id, x, y, sx, sy
            );
            match tokio::task::spawn_blocking(move || {
                crate::browser_agent::click_at_viewport_coordinates_with_downloads(sx, sy)
            })
            .await
            {
                Ok(Ok((state_str, dls))) => (state_str, dls),
                Ok(Err(cdp_err)) => {
                    let base = format!("BROWSER_CLICK (coordinates) failed: {}", cdp_err);
                    let base = append_browser_readiness_context(base, true, None);
                    (append_latest_browser_state_guidance(&base), vec![])
                }
                Err(e) => (
                    append_latest_browser_state_guidance(&format!(
                        "BROWSER_CLICK task error: {}",
                        e
                    )),
                    vec![],
                ),
            }
        }
        BrowserClickTarget::Index(idx) => {
            let status_msg = {
                let label = crate::browser_agent::get_last_element_label(idx);
                if let Some(l) = label {
                    format!(
                        "🖱️ Clicking element {} ({})",
                        idx,
                        crate::logging::ellipse(&l, 30)
                    )
                } else {
                    format!("🖱️ Clicking element {}", idx)
                }
            };
            send_status(status_tx, &status_msg);
            info!("Agent router [{}]: BROWSER_CLICK index {}", request_id, idx);
            match tokio::task::spawn_blocking(move || {
                crate::browser_agent::click_by_index_inner_with_downloads(idx)
            })
            .await
            {
                Ok(Ok((state_str, dls))) => (state_str, dls),
                Ok(Err(cdp_err)) => {
                    if should_use_http_fallback_after_browser_action_error(
                        "BROWSER_CLICK",
                        &cdp_err,
                    ) {
                        match tokio::task::spawn_blocking(move || {
                            crate::browser_agent::click_http(idx)
                        })
                        .await
                        {
                            Ok(Ok(state_str)) => (state_str, vec![]),
                            Ok(Err(e)) => {
                                let base = format!("BROWSER_CLICK failed: {}", e);
                                let base = append_browser_readiness_context(base, false, None);
                                (append_latest_browser_state_guidance(&base), vec![])
                            }
                            Err(e) => (format!("BROWSER_CLICK task error: {}", e), vec![]),
                        }
                    } else {
                        let base = format!("BROWSER_CLICK failed: {}", cdp_err);
                        let base = append_browser_readiness_context(base, true, None);
                        (append_latest_browser_state_guidance(&base), vec![])
                    }
                }
                Err(e) => (
                    append_latest_browser_state_guidance(&format!(
                        "BROWSER_CLICK task error: {}",
                        e
                    )),
                    vec![],
                ),
            }
        }
    }
}

pub(crate) async fn handle_browser_hover(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> (String, Vec<PathBuf>) {
    if let Some(msg) = browser_tools_disabled_guard() {
        return (msg, vec![]);
    }

    let idx = match parse_browser_hover_arg(arg) {
        Ok(i) => i,
        Err(e) => return (append_latest_browser_state_guidance(&e), vec![]),
    };

    let status_msg = {
        let label = crate::browser_agent::get_last_element_label(idx);
        if let Some(l) = label {
            format!(
                "👆 Hovering element {} ({})",
                idx,
                crate::logging::ellipse(&l, 30)
            )
        } else {
            format!("👆 Hovering element {}", idx)
        }
    };
    send_status(status_tx, &status_msg);
    info!("Agent router [{}]: BROWSER_HOVER index {}", request_id, idx);
    match tokio::task::spawn_blocking(move || {
        crate::browser_agent::hover_by_index_inner_with_downloads(idx)
    })
    .await
    {
        Ok(Ok((state_str, dls))) => (state_str, dls),
        Ok(Err(e)) => {
            let base = format!("BROWSER_HOVER failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            (append_latest_browser_state_guidance(&base), vec![])
        }
        Err(e) => (
            append_latest_browser_state_guidance(&format!(
                "BROWSER_HOVER task error: {}",
                e
            )),
            vec![],
        ),
    }
}

pub(crate) async fn handle_browser_drag(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> (String, Vec<PathBuf>) {
    if let Some(msg) = browser_tools_disabled_guard() {
        return (msg, vec![]);
    }

    let (from_i, to_i) = match parse_browser_drag_arg(arg) {
        Ok(p) => p,
        Err(e) => return (append_latest_browser_state_guidance(&e), vec![]),
    };

    let status_msg = {
        let lf = crate::browser_agent::get_last_element_label(from_i);
        let lt = crate::browser_agent::get_last_element_label(to_i);
        match (lf, lt) {
            (Some(a), Some(b)) => format!(
                "↔️ Drag {} ({}) → {} ({})",
                from_i,
                crate::logging::ellipse(&a, 24),
                to_i,
                crate::logging::ellipse(&b, 24)
            ),
            _ => format!("↔️ Drag {} → {}", from_i, to_i),
        }
    };
    send_status(status_tx, &status_msg);
    info!(
        "Agent router [{}]: BROWSER_DRAG from {} to {}",
        request_id, from_i, to_i
    );
    match tokio::task::spawn_blocking(move || {
        crate::browser_agent::drag_by_indices_inner_with_downloads(from_i, to_i)
    })
    .await
    {
        Ok(Ok((state_str, dls))) => (state_str, dls),
        Ok(Err(e)) => {
            let base = format!("BROWSER_DRAG failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            (append_latest_browser_state_guidance(&base), vec![])
        }
        Err(e) => (
            append_latest_browser_state_guidance(&format!(
                "BROWSER_DRAG task error: {}",
                e
            )),
            vec![],
        ),
    }
}

pub(crate) async fn handle_browser_download(
    arg: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> (String, Vec<PathBuf>) {
    if let Some(msg) = browser_tools_disabled_guard() {
        return (msg, vec![]);
    }
    let trimmed = arg.trim();
    let secs = if trimmed.is_empty() {
        30u64
    } else {
        trimmed.parse::<u64>().unwrap_or(30)
    }
    .clamp(1, 120);
    send_status(
        status_tx,
        &format!("⬇️ Waiting up to {}s for browser download…", secs),
    );
    info!(
        "Agent router [{}]: BROWSER_DOWNLOAD timeout_secs={}",
        request_id, secs
    );
    match tokio::task::spawn_blocking(move || crate::browser_agent::wait_for_browser_download(secs))
        .await
    {
        Ok(Ok((msg, paths))) => (msg, paths),
        Ok(Err(e)) => (format!("BROWSER_DOWNLOAD failed: {}", e), vec![]),
        Err(e) => (format!("BROWSER_DOWNLOAD task error: {}", e), vec![]),
    }
}

pub(crate) async fn handle_browser_input(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let mut parts = arg.trim().splitn(2, |c: char| c.is_whitespace());
    let index_arg = parts.next().unwrap_or("").trim();
    let index_for_status = index_arg.parse::<u32>().ok();
    let status_msg = match index_for_status {
        Some(idx) => {
            let label = crate::browser_agent::get_last_element_label(idx);
            if let Some(l) = label {
                format!(
                    "✍️ Typing into element {} ({})…",
                    idx,
                    crate::logging::ellipse(&l, 30)
                )
            } else {
                format!("✍️ Typing into element {}…", idx)
            }
        }
        None => format!(
            "✍️ Typing into element {}…",
            if index_arg.is_empty() { "?" } else { index_arg }
        ),
    };
    send_status(status_tx, &status_msg);
    let text = parts.next().unwrap_or("").trim().to_string();
    let index = index_arg.parse::<u32>().ok();
    match index {
        Some(idx) => {
            info!("BROWSER_INPUT: index {} ({} chars)", idx, text.len());
            let text_clone = text.clone();
            match tokio::task::spawn_blocking(move || {
                crate::browser_agent::input_by_index(idx, &text_clone)
            })
            .await
            {
                Ok(Ok(state_str)) => state_str,
                Ok(Err(cdp_err)) => {
                    if should_use_http_fallback_after_browser_action_error(
                        "BROWSER_INPUT",
                        &cdp_err,
                    ) {
                        match tokio::task::spawn_blocking(move || {
                            crate::browser_agent::input_http(idx, &text)
                        })
                        .await
                        {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(e)) => {
                                let base = format!("BROWSER_INPUT failed: {}", e);
                                let base = append_browser_readiness_context(base, false, None);
                                append_latest_browser_state_guidance(&base)
                            }
                            Err(e) => format!("BROWSER_INPUT task error: {}", e),
                        }
                    } else {
                        let base = format!("BROWSER_INPUT failed: {}", cdp_err);
                        let base = append_browser_readiness_context(base, true, None);
                        append_latest_browser_state_guidance(&base)
                    }
                }
                Err(e) => append_latest_browser_state_guidance(&format!(
                    "BROWSER_INPUT task error: {}",
                    e
                )),
            }
        }
        None => append_latest_browser_state_guidance(
            "BROWSER_INPUT requires a numeric index and text (e.g. BROWSER_INPUT: 4 search query). Use the index from the Current page Elements list.",
        ),
    }
}

pub(crate) async fn handle_browser_upload(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let mut parts = arg.trim().splitn(2, |c: char| c.is_whitespace());
    let index_arg = parts.next().unwrap_or("").trim();
    let path_arg = parts.next().unwrap_or("").trim();
    let index = match index_arg.parse::<u32>() {
        Ok(n) if n >= 1 => n,
        _ => {
            return append_latest_browser_state_guidance(
                "BROWSER_UPLOAD requires a 1-based element index and a file path (e.g. BROWSER_UPLOAD: 3 ~/mac-stats/uploads/doc.pdf). The file must exist, be non-empty, and live under ~/.mac-stats/uploads/ or ~/.mac-stats/screenshots/; relative paths are resolved under uploads/.",
            );
        }
    };
    let path_display: String = if path_arg.len() > 60 {
        format!("{}…", path_arg.chars().take(60).collect::<String>())
    } else {
        path_arg.to_string()
    };
    send_status(
        status_tx,
        &format!(
            "📎 Uploading file for element {} ({})…",
            index,
            crate::logging::ellipse(&path_display, 45)
        ),
    );
    let path_owned = path_arg.to_string();
    match tokio::task::spawn_blocking(move || {
        let canon = crate::browser_agent::resolve_browser_upload_source_path(&path_owned)?;
        crate::browser_agent::upload_file_by_index(index, &canon)
    })
    .await
    {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            let base = format!("BROWSER_UPLOAD failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            append_latest_browser_state_guidance(&base)
        }
        Err(e) => append_latest_browser_state_guidance(&format!(
            "BROWSER_UPLOAD task error: {}",
            e
        )),
    }
}

pub(crate) async fn handle_browser_keys(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let chord = arg.trim().to_string();
    if chord.is_empty() {
        return append_latest_browser_state_guidance(
            "BROWSER_KEYS requires a chord: use + between parts, no spaces inside (e.g. BROWSER_KEYS: Escape, BROWSER_KEYS: Meta+f, BROWSER_KEYS: Ctrl+Enter). Allowlisted keys: Enter, Escape, Tab, Backspace, F5, or one letter a–z with at least one of Meta, Ctrl, Alt, Shift. Sends keys to the **page** only (not Chrome chrome UI). CDP required — no HTTP fallback.",
        );
    }
    send_status(
        status_tx,
        &format!("⌨️ Keys: {}…", crate::logging::ellipse(&chord, 40)),
    );
    info!("BROWSER_KEYS: {}", crate::logging::ellipse(&chord, 80));
    let chord_clone = chord.clone();
    match tokio::task::spawn_blocking(move || {
        crate::browser_agent::dispatch_browser_keys(&chord_clone)
    })
    .await
    {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            let base = format!("BROWSER_KEYS failed: {}", e);
            let base = append_browser_readiness_context(base, true, None);
            append_latest_browser_state_guidance(&base)
        }
        Err(e) => append_latest_browser_state_guidance(&format!("BROWSER_KEYS task error: {}", e)),
    }
}

pub(crate) async fn handle_browser_scroll(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let scroll_arg = if arg.trim().is_empty() {
        "down".to_string()
    } else {
        arg.trim().to_string()
    };
    send_status(
        status_tx,
        &format!("📜 Scrolling {}…", crate::logging::ellipse(&scroll_arg, 20)),
    );
    match tokio::task::spawn_blocking(move || crate::browser_agent::scroll_page(&scroll_arg)).await
    {
        Ok(Ok(state_str)) => state_str,
        Ok(Err(e)) => {
            info!(
                "BROWSER_SCROLL failed: {}",
                crate::logging::ellipse(&e, 200)
            );
            let base = format!("BROWSER_SCROLL failed: {}", e);
            append_browser_readiness_context(base, true, None)
        }
        Err(e) => format!("BROWSER_SCROLL task error: {}", e),
    }
}

fn browser_extract_include_images(arg: &str) -> bool {
    let a = arg.trim().to_lowercase();
    if a.is_empty() {
        return true;
    }
    match a.as_str() {
        "no_images" | "without_images" | "skip_images" | "no-images" | "text_only" | "textonly" => {
            false
        }
        "images" | "with_images" | "with-images" | "markdown" | "default" => true,
        _ => {
            info!(
                "BROWSER_EXTRACT: unrecognized arg {:?}; defaulting include_images=true",
                crate::logging::ellipse(&a, 80)
            );
            true
        }
    }
}

pub(crate) async fn handle_browser_extract(raw_arg: &str) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let include_images = browser_extract_include_images(raw_arg);
    if !include_images {
        info!("BROWSER_EXTRACT: include_images=false (no ![alt](url) lines)");
    }

    match tokio::task::spawn_blocking(move || {
        crate::browser_agent::extract_page_text_with_options(include_images)
    })
    .await
    {
        Ok(Ok(text)) => {
            crate::commands::suspicious_patterns::log_untrusted_suspicious_scan("browser-extract", &text);
            crate::commands::untrusted_content::wrap_untrusted_content("browser-extract", &text)
        }
        Ok(Err(_cdp_err)) => {
            match tokio::task::spawn_blocking(crate::browser_agent::extract_http).await {
                Ok(Ok(text)) => {
                    crate::commands::suspicious_patterns::log_untrusted_suspicious_scan(
                        "browser-extract",
                        &text,
                    );
                    crate::commands::untrusted_content::wrap_untrusted_content("browser-extract", &text)
                }
                Ok(Err(e)) => append_browser_readiness_context(
                    format!(
                        "BROWSER_EXTRACT failed: {}. (Navigate to a page first with BROWSER_NAVIGATE.)",
                        e
                    ),
                    false,
                    None,
                ),
                Err(e) => format!("BROWSER_EXTRACT task error: {}", e),
            }
        }
        Err(e) => format!("BROWSER_EXTRACT task error: {}", e),
    }
}

pub(crate) async fn handle_browser_search_page(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let (pattern, css_scope) = parse_browser_search_page_arg(arg);
    if pattern.is_empty() {
        return "BROWSER_SEARCH_PAGE requires a search pattern (e.g. BROWSER_SEARCH_PAGE: Ralf Röber). Optional: css_scope=<CSS selector> to search only inside that subtree (e.g. BROWSER_SEARCH_PAGE: price css_scope=main).".to_string();
    }
    let scope_note = css_scope
        .as_ref()
        .map(|s| format!(" (scoped: {})", crate::logging::ellipse(s, 40)))
        .unwrap_or_default();
    send_status(
        status_tx,
        &format!(
            "🔍 Searching page for \"{}\"{}…",
            crate::logging::ellipse(&pattern, 30),
            scope_note
        ),
    );
    info!(
        "BROWSER_SEARCH_PAGE: pattern_len={} css_scope={}",
        pattern.len(),
        css_scope
            .as_ref()
            .map(|s| crate::logging::ellipse(s, 80))
            .unwrap_or_else(|| "none".into())
    );
    let pattern_clone = pattern.clone();
    let scope_clone = css_scope.clone();
    match tokio::task::spawn_blocking(move || {
        crate::browser_agent::search_page_text(
            &pattern_clone,
            scope_clone.as_deref(),
        )
    })
    .await
    {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            info!(
                "BROWSER_SEARCH_PAGE failed: {}",
                crate::logging::ellipse(&e, 200)
            );
            let base = format!(
                "BROWSER_SEARCH_PAGE failed: {}. (Navigate to a page first with BROWSER_NAVIGATE.)",
                e
            );
            append_browser_readiness_context(base, true, None)
        }
        Err(e) => format!("BROWSER_SEARCH_PAGE task error: {}", e),
    }
}

pub(crate) async fn handle_browser_query(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if let Some(msg) = browser_tools_disabled_guard() {
        return msg;
    }

    let (selector, attrs) = parse_browser_query_arg(arg);
    if selector.is_empty() {
        return "BROWSER_QUERY requires a CSS selector (e.g. BROWSER_QUERY: nav a). Optional: attrs=href,id,class (comma-separated).".to_string();
    }
    send_status(
        status_tx,
        &format!(
            "🧩 CSS query {}…",
            crate::logging::ellipse(&selector, 40)
        ),
    );
    info!(
        "BROWSER_QUERY: selector={} attrs={}",
        crate::logging::ellipse(&selector, 80),
        attrs
            .as_ref()
            .map(|a| crate::logging::ellipse(a, 80))
            .unwrap_or_else(|| "none".into())
    );
    let sel = selector.clone();
    let attrs_clone = attrs.clone();
    match tokio::task::spawn_blocking(move || {
        crate::browser_agent::browser_query(&sel, attrs_clone.as_deref())
    })
    .await
    {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            info!(
                "BROWSER_QUERY failed: {}",
                crate::logging::ellipse(&e, 200)
            );
            let base = format!(
                "BROWSER_QUERY failed: {}. (Navigate to a page first with BROWSER_NAVIGATE.)",
                e
            );
            append_browser_readiness_context(base, true, None)
        }
        Err(e) => format!("BROWSER_QUERY task error: {}", e),
    }
}

#[cfg(test)]
mod browser_arg_parse_tests {
    use super::{
        parse_browser_click_arg, parse_browser_query_arg, parse_browser_search_page_arg,
        BrowserClickTarget,
    };

    #[test]
    fn parses_index() {
        match parse_browser_click_arg("12").unwrap() {
            BrowserClickTarget::Index(12) => {}
            _ => panic!("expected index"),
        }
    }

    #[test]
    fn parses_coordinate_keys() {
        match parse_browser_click_arg("coordinate_x=10 coordinate_y=20").unwrap() {
            BrowserClickTarget::Coordinates { x, y } => {
                assert!((x - 10.0).abs() < f64::EPSILON);
                assert!((y - 20.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected coords"),
        }
    }

    #[test]
    fn parses_coords_keyword() {
        match parse_browser_click_arg("coords 100 200").unwrap() {
            BrowserClickTarget::Coordinates { x, y } => {
                assert!((x - 100.0).abs() < f64::EPSILON);
                assert!((y - 200.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected coords"),
        }
    }

    #[test]
    fn parses_search_page_css_scope() {
        let (p, s) = parse_browser_search_page_arg("price css_scope=.product");
        assert_eq!(p, "price");
        assert_eq!(s.as_deref(), Some(".product"));
    }

    #[test]
    fn parses_browser_query_attrs() {
        let (sel, a) = parse_browser_query_arg("nav.main a attrs=href, id");
        assert_eq!(sel, "nav.main a");
        assert_eq!(a.as_deref(), Some("href, id"));
    }
}
