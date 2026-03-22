//! HTTP-only browser fallback when CDP/Chrome is not available.
//! Fetches HTML via reqwest, parses links/forms with scraper, presents numbered list to LLM.
//! BROWSER_NAVIGATE = fetch + parse; BROWSER_CLICK = follow link or submit form; BROWSER_INPUT = fill form field.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use scraper::{Html, Selector};
use crate::mac_stats_info;
use url::Url;

use crate::commands::browser::fetch_page_content;

/// One interactive element for the LLM (1-based index). Same idea as CDP Interactable.
#[derive(Debug, Clone)]
pub struct HttpInteractable {
    pub index: u32,
    pub tag: String,
    pub text: String,
    pub href: Option<String>,
    pub name: Option<String>,
    #[allow(dead_code)]
    pub input_type: Option<String>,
    pub is_submit: bool,
}

/// State for the HTTP-only "browser": current page URL, parsed elements, body text, and filled form values.
#[derive(Debug, Default)]
struct HttpBrowserState {
    current_url: String,
    interactables: Vec<HttpInteractable>,
    body_text: String,
    form_values: HashMap<String, String>,
}

fn http_state() -> &'static Mutex<HttpBrowserState> {
    static STATE: OnceLock<Mutex<HttpBrowserState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(HttpBrowserState::default()))
}

/// Resolve href against base URL; return None if invalid or non-http(s).
fn resolve_href(base: &str, href: &str) -> Option<String> {
    let base_url = Url::parse(base).ok()?;
    let resolved = base_url.join(href).ok()?;
    match resolved.scheme() {
        "http" | "https" => Some(resolved.to_string()),
        _ => None,
    }
}

/// Parse HTML into interactables (links, inputs, buttons) and body text. Base URL for resolving relative hrefs.
fn parse_html(html: &str, base_url: &str) -> (Vec<HttpInteractable>, String) {
    let document = Html::parse_document(html);
    let mut interactables = Vec::new();
    let mut index = 1u32;

    // Links: a[href]
    let a_sel = Selector::parse("a[href]").unwrap();
    for el in document.select(&a_sel) {
        let href = el.value().attr("href").map(str::to_string);
        let text = el.text().collect::<String>().trim().to_string();
        let resolved = href
            .as_ref()
            .and_then(|h| resolve_href(base_url, h))
            .or(href);
        if let Some(h) = resolved {
            if h.starts_with("http://") || h.starts_with("https://") {
                interactables.push(HttpInteractable {
                    index,
                    tag: "a".to_string(),
                    text: text.chars().take(80).collect(),
                    href: Some(h),
                    name: None,
                    input_type: None,
                    is_submit: false,
                });
                index += 1;
            }
        }
    }

    // Form inputs and buttons (inside form or standalone)
    let input_sel = Selector::parse("input, textarea, button").unwrap();
    for el in document.select(&input_sel) {
        let tag = el.value().name().to_lowercase();
        let name = el.value().attr("name").map(str::to_string);
        let input_type = el
            .value()
            .attr("type")
            .map(|t| t.to_lowercase())
            .unwrap_or_else(|| "text".to_string());
        let is_submit = tag == "button" || input_type == "submit" || input_type == "image";
        let text = if tag == "button" {
            el.text().collect::<String>().trim().to_string()
        } else {
            el.value().attr("placeholder").unwrap_or("").to_string()
        };
        let text = text.chars().take(80).collect::<String>();
        // Skip hidden for display (we can still use them for submit)
        if input_type == "hidden" {
            continue;
        }
        interactables.push(HttpInteractable {
            index,
            tag,
            text,
            href: None,
            name,
            input_type: Some(input_type),
            is_submit,
        });
        index += 1;
    }

    // Body text: strip script/style, take body inner text (simplified)
    let body_text = document
        .select(&Selector::parse("body").unwrap())
        .next()
        .map(|body| body.text().collect::<String>())
        .unwrap_or_else(|| document.root_element().text().collect::<String>());
    let body_text = body_text.split_whitespace().collect::<Vec<_>>().join(" ");
    let body_text = body_text.chars().take(30_000).collect::<String>();

    (interactables, body_text)
}

fn http_interactable_label(i: &HttpInteractable) -> String {
    let s = if i.text.trim().is_empty() {
        i.name.as_deref().unwrap_or("")
    } else {
        i.text.trim()
    };
    let s = if s.is_empty() {
        i.href.as_deref().unwrap_or("link")
    } else {
        s
    };
    let max = 50;
    if s.len() > max {
        format!("{}…", &s[..max.saturating_sub(1)])
    } else {
        s.to_string()
    }
}

fn format_http_state_for_llm(url: &str, interactables: &[HttpInteractable]) -> String {
    let mut s = format!("Current page (HTTP): {}\n", url);
    s.push_str("Elements:\n");
    for i in interactables {
        let kind = if i.tag == "a" {
            "link"
        } else if i.is_submit {
            "button"
        } else {
            "input"
        };
        let label = if !i.text.is_empty() {
            i.text.as_str()
        } else if let Some(ref n) = i.name {
            n.as_str()
        } else if let Some(ref h) = i.href {
            h.as_str()
        } else {
            "(no label)"
        };
        let label_escaped = label
            .replace('\n', " ")
            .chars()
            .take(80)
            .collect::<String>();
        s.push_str(&format!("[{}] {} '{}'\n", i.index, kind, label_escaped));
    }
    if interactables.is_empty() {
        s.push_str("(no interactive elements found)\n");
    }
    s
}

/// Navigate to URL via HTTP fetch, parse HTML, store state. Returns formatted "Current page" + elements for LLM.
pub fn navigate_http(url: &str) -> Result<String, String> {
    let url = url.trim();
    let url = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    };
    mac_stats_info!(
        "browser/http_fallback",
        "Browser agent [HTTP fallback]: BROWSER_NAVIGATE: {}",
        url
    );
    let html = fetch_page_content(&url)?;
    let (interactables, body_text) = parse_html(&html, &url);
    let mut state = http_state().lock().map_err(|e| e.to_string())?;
    state.current_url = url.clone();
    state.interactables = interactables.clone();
    state.body_text = body_text;
    state.form_values.clear();
    drop(state);
    super::set_last_element_labels(
        interactables
            .iter()
            .map(|i| (i.index, http_interactable_label(i)))
            .collect(),
    );
    Ok(format_http_state_for_llm(&url, &interactables))
}

/// Click by index: if link, navigate to href; if submit button, submit form (POST/GET) and update state.
pub fn click_http(index: u32) -> Result<String, String> {
    if index == 0 {
        return Err("BROWSER_CLICK index must be >= 1".to_string());
    }
    let state = http_state().lock().map_err(|e| e.to_string())?;
    let interactables = state.interactables.clone();
    let current_url = state.current_url.clone();
    drop(state);

    let el = interactables
        .iter()
        .find(|e| e.index == index)
        .ok_or_else(|| format!("BROWSER_CLICK: no element at index {}", index))?;

    if let Some(ref href) = el.href {
        return navigate_http(href);
    }

    if el.is_submit {
        return submit_http_form(&interactables, &current_url);
    }

    Err(format!(
        "BROWSER_CLICK: element [{}] is not a link or submit button",
        index
    ))
}

/// Build form action URL and submit (GET or POST) with current form_values. Reuses first form on page for simplicity.
fn submit_http_form(
    _interactables: &[HttpInteractable],
    _current_url: &str,
) -> Result<String, String> {
    let state = http_state().lock().map_err(|e| e.to_string())?;
    let form_values = state.form_values.clone();
    let base_url = state.current_url.clone();
    drop(state);

    let action_url = base_url;
    let url_parsed = Url::parse(&action_url).map_err(|e| format!("URL parse: {}", e))?;
    let mut url_with_params = url_parsed.clone();
    for (k, v) in &form_values {
        url_with_params.query_pairs_mut().append_pair(k, v);
    }
    let target = url_with_params.to_string();
    mac_stats_info!(
        "browser/http_fallback",
        "Browser agent [HTTP fallback]: form submit GET {}",
        target
    );
    let html = fetch_page_content(&target)?;
    let (interactables, body_text) = parse_html(&html, &target);
    let mut state = http_state().lock().map_err(|e| e.to_string())?;
    state.current_url = target.clone();
    state.interactables = interactables.clone();
    state.body_text = body_text;
    state.form_values.clear();
    drop(state);
    super::set_last_element_labels(
        interactables
            .iter()
            .map(|i| (i.index, http_interactable_label(i)))
            .collect(),
    );
    Ok(format_http_state_for_llm(&target, &interactables))
}

/// Fill form field by index (1-based). Updates in-memory form_values; no fetch.
pub fn input_http(index: u32, text: &str) -> Result<String, String> {
    if index == 0 {
        return Err("BROWSER_INPUT index must be >= 1".to_string());
    }
    let mut state = http_state().lock().map_err(|e| e.to_string())?;
    let el = state
        .interactables
        .iter()
        .find(|e| e.index == index)
        .ok_or_else(|| format!("BROWSER_INPUT: no element at index {}", index))?;
    let name = el.name.clone().ok_or_else(|| {
        format!(
            "BROWSER_INPUT: element [{}] has no name (not an input)",
            index
        )
    })?;
    if el.is_submit {
        return Err("BROWSER_INPUT: use BROWSER_CLICK to submit; element is a button".to_string());
    }
    state.form_values.insert(name, text.to_string());
    let interactables = state.interactables.clone();
    let url = state.current_url.clone();
    drop(state);
    Ok(format_http_state_for_llm(&url, &interactables))
}

/// Return visible body text from last HTTP-fetched page.
pub fn extract_http() -> Result<String, String> {
    let state = http_state().lock().map_err(|e| e.to_string())?;
    if state.body_text.is_empty() && state.current_url.is_empty() {
        return Err("BROWSER_EXTRACT: no page loaded. Use BROWSER_NAVIGATE first.".to_string());
    }
    Ok(state.body_text.clone())
}
