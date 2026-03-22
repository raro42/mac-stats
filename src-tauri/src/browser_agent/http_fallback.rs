//! HTTP-only browser fallback when CDP/Chrome is not available.
//! Fetches HTML via reqwest, parses links/forms with scraper, presents numbered list to LLM.
//! BROWSER_NAVIGATE = fetch + parse; BROWSER_CLICK = follow link or submit form; BROWSER_INPUT = fill form field.

use std::collections::{HashMap, HashSet};

use std::sync::Mutex;
use std::sync::OnceLock;

use crate::mac_stats_info;
use scraper::{ElementRef, Html, Selector};
use url::Url;

use crate::commands::browser::{fetch_page_content, fetch_page_post_form_urlencoded};

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
    /// Form scope: `0` = controls outside any `<form>` (GET to current document URL).
    pub form_id: usize,
    /// HTML `value` (or textarea text); used when the user did not type into the field.
    default_value: Option<String>,
}

#[derive(Debug, Clone)]
struct FormSpec {
    action_url: String,
    method_is_post: bool,
    multipart: bool,
    hidden: Vec<(String, String)>,
}

/// State for the HTTP-only "browser": current page URL, parsed elements, body text, and filled form values.
#[derive(Debug, Default)]
struct HttpBrowserState {
    current_url: String,
    interactables: Vec<HttpInteractable>,
    forms: Vec<FormSpec>,
    body_text: String,
    /// (form_id, field name) → value from BROWSER_INPUT
    form_values: HashMap<(usize, String), String>,
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

fn resolve_form_action(base_url: &str, action_attr: &str) -> Result<String, String> {
    let base_u = Url::parse(base_url).map_err(|e| format!("Invalid document URL: {}", e))?;
    let trimmed = action_attr.trim();
    let resolved = if trimmed.is_empty() {
        base_u
    } else {
        base_u
            .join(trimmed)
            .map_err(|e| format!("Invalid form action URL: {}", e))?
    };
    match resolved.scheme() {
        "http" | "https" => Ok(resolved.to_string()),
        other => Err(format!(
            "Form action must use http or https (got scheme {:?}). Relative actions like javascript: are not supported in HTTP fallback.",
            other
        )),
    }
}

fn input_type_lowercase(el: ElementRef<'_>) -> String {
    el.attr("type")
        .map(|t: &str| t.to_ascii_lowercase())
        .unwrap_or_else(|| "text".to_string())
}

fn is_submit_control(tag: &str, input_type: &str) -> bool {
    tag == "button" || input_type == "submit" || input_type == "image"
}

/// Returns `forms` vector index (1-based: first `<form>` is `1`, matching [`FormSpec`] index).
fn enclosing_form_index(el: ElementRef<'_>, form_elements: &[ElementRef<'_>]) -> usize {
    let mut cur = Some(el);
    while let Some(e) = cur {
        if e.value().name().eq_ignore_ascii_case("form") {
            for (i, f) in form_elements.iter().enumerate() {
                if *f == e {
                    return i + 1;
                }
            }
            return 0;
        }
        cur = e.parent().and_then(ElementRef::wrap);
    }
    0
}

/// Parse HTML into interactables, per-form metadata, and body text.
fn parse_html(html: &str, base_url: &str) -> Result<(Vec<HttpInteractable>, Vec<FormSpec>, String), String> {
    let document = Html::parse_document(html);
    let mut forms: Vec<FormSpec> = vec![FormSpec {
        action_url: base_url.to_string(),
        method_is_post: false,
        multipart: false,
        hidden: vec![],
    }];
    let form_sel = Selector::parse("form").map_err(|e| format!("form selector: {}", e))?;
    let form_elements: Vec<ElementRef<'_>> = document.select(&form_sel).collect();

    for form_el in &form_elements {
        let action_raw = form_el.value().attr("action").unwrap_or("");
        let action_url = resolve_form_action(base_url, action_raw)?;

        let method_raw = form_el
            .value()
            .attr("method")
            .unwrap_or("get")
            .trim()
            .to_ascii_lowercase();
        let method_is_post = method_raw == "post";

        let enctype_raw = form_el
            .value()
            .attr("enctype")
            .unwrap_or("application/x-www-form-urlencoded")
            .to_ascii_lowercase();
        let multipart = enctype_raw.contains("multipart/form-data");

        let mut hidden = Vec::new();
        let inner_inputs =
            Selector::parse("input[name]").map_err(|e| format!("input selector: {}", e))?;
        for inp in form_el.select(&inner_inputs) {
            let t = input_type_lowercase(inp);
            if t != "hidden" {
                continue;
            }
            let Some(n) = inp.attr("name").map(str::to_string) else {
                continue;
            };
            let v = inp.attr("value").unwrap_or("").to_string();
            hidden.push((n, v));
        }

        forms.push(FormSpec {
            action_url,
            method_is_post,
            multipart,
            hidden,
        });
    }

    let mut interactables = Vec::new();
    let mut index = 1u32;

    let a_sel = Selector::parse("a[href]").map_err(|e| format!("a selector: {}", e))?;
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
                    form_id: 0,
                    default_value: None,
                });
                index += 1;
            }
        }
    }

    let input_sel =
        Selector::parse("input, textarea, button").map_err(|e| format!("controls selector: {}", e))?;
    for el in document.select(&input_sel) {
        let tag = el.value().name().to_lowercase();
        let input_type = if tag == "input" {
            input_type_lowercase(el)
        } else {
            String::new()
        };
        if tag == "input" && input_type == "hidden" {
            continue;
        }

        let form_id = enclosing_form_index(el, &form_elements);
        let name = el.value().attr("name").map(str::to_string);
        let is_submit = is_submit_control(&tag, &input_type);

        let text = if tag == "button" {
            el.text().collect::<String>().trim().to_string()
        } else if tag == "textarea" {
            el.text().collect::<String>().trim().to_string()
        } else {
            el.value().attr("placeholder").unwrap_or("").to_string()
        };
        let text = text.chars().take(80).collect::<String>();

        let default_value = if tag == "textarea" {
            Some(el.text().collect::<String>())
        } else if tag == "input" {
            el.value().attr("value").map(str::to_string)
        } else if tag == "button" {
            el.value().attr("value").map(str::to_string)
        } else {
            None
        };

        let input_type_field = if tag == "input" {
            Some(input_type.clone())
        } else {
            None
        };

        interactables.push(HttpInteractable {
            index,
            tag,
            text,
            href: None,
            name,
            input_type: input_type_field,
            is_submit,
            form_id,
            default_value,
        });
        index += 1;
    }

    let body_text = document
        .select(&Selector::parse("body").unwrap())
        .next()
        .map(|body| body.text().collect::<String>())
        .unwrap_or_else(|| document.root_element().text().collect::<String>());
    let body_text = body_text.split_whitespace().collect::<Vec<_>>().join(" ");
    let body_text = body_text.chars().take(30_000).collect::<String>();
    let body_text =
        crate::commands::text_normalize::apply_untrusted_homoglyph_normalization(body_text);

    Ok((interactables, forms, body_text))
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

fn format_http_state_for_llm(
    url: &str,
    interactables: &[HttpInteractable],
    transition_note: Option<&str>,
) -> String {
    let mut s = format!("Current page (HTTP): {}\n", url);
    if let Some(note) = transition_note {
        s.push_str(note);
        s.push('\n');
    }
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

fn collect_form_submission_pairs(
    forms: &[FormSpec],
    interactables: &[HttpInteractable],
    form_values: &HashMap<(usize, String), String>,
    submit: &HttpInteractable,
) -> Result<Vec<(String, String)>, String> {
    let fid = submit.form_id;
    let spec = forms
        .get(fid)
        .ok_or_else(|| format!("Internal error: form_id {} out of range", fid))?;

    let mut seen_names: HashSet<String> = HashSet::new();
    let mut pairs: Vec<(String, String)> = Vec::new();

    for (k, v) in &spec.hidden {
        pairs.push((k.clone(), v.clone()));
        seen_names.insert(k.clone());
    }

    for el in interactables {
        if el.form_id != fid {
            continue;
        }
        let Some(ref name) = el.name else {
            continue;
        };
        if el.is_submit && el.index != submit.index {
            continue;
        }

        let t = el
            .input_type
            .as_deref()
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        if el.is_submit {
            let v = form_values
                .get(&(fid, name.clone()))
                .cloned()
                .or_else(|| el.default_value.clone())
                .unwrap_or_else(|| "Submit".to_string());
            pairs.push((name.clone(), v));
            continue;
        }

        if t == "checkbox" || t == "radio" {
            if let Some(v) = form_values.get(&(fid, name.clone())) {
                pairs.push((name.clone(), v.clone()));
            }
            continue;
        }

        let v = form_values
            .get(&(fid, name.clone()))
            .cloned()
            .or_else(|| el.default_value.clone())
            .unwrap_or_default();
        if seen_names.contains(name) && spec.hidden.iter().any(|(k, _)| k == name) {
            // User overwrote a hidden field via BROWSER_INPUT (same name as hidden).
            pairs.retain(|(k, _)| k != name);
            seen_names.remove(name);
        }
        pairs.push((name.clone(), v));
    }

    Ok(pairs)
}

fn apply_form_submission(spec: &FormSpec, pairs: &[(String, String)]) -> Result<String, String> {
    let html = if spec.method_is_post {
        mac_stats_info!(
            "browser/http_fallback",
            "Browser agent [HTTP fallback]: form submit POST {} ({} field(s))",
            spec.action_url,
            pairs.len()
        );
        fetch_page_post_form_urlencoded(&spec.action_url, pairs)?
    } else {
        let url_parsed = Url::parse(&spec.action_url)
            .map_err(|e| format!("URL parse for form GET: {}", e))?;
        let mut url_with_params = url_parsed;
        for (k, v) in pairs {
            url_with_params.query_pairs_mut().append_pair(k, v);
        }
        let target = url_with_params.to_string();
        mac_stats_info!(
            "browser/http_fallback",
            "Browser agent [HTTP fallback]: form submit GET {}",
            target
        );
        fetch_page_content(&target)?
    };

    Ok(html)
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
    let (interactables, forms, body_text) = parse_html(&html, &url)?;
    let mut state = http_state().lock().map_err(|e| e.to_string())?;
    state.current_url = url.clone();
    state.interactables = interactables.clone();
    state.forms = forms;
    state.body_text = body_text;
    state.form_values.clear();
    drop(state);
    super::set_last_element_labels(
        interactables
            .iter()
            .map(|i| (i.index, http_interactable_label(i)))
            .collect(),
    );
    Ok(format_http_state_for_llm(&url, &interactables, None))
}

/// Click by index: if link, navigate to href; if submit button, submit form (POST/GET) and update state.
pub fn click_http(index: u32) -> Result<String, String> {
    if index == 0 {
        return Err("BROWSER_CLICK index must be >= 1".to_string());
    }
    let state = http_state().lock().map_err(|e| e.to_string())?;
    let interactables = state.interactables.clone();
    let forms = state.forms.clone();
    let current_url = state.current_url.clone();
    drop(state);

    let el = interactables
        .iter()
        .find(|e| e.index == index)
        .ok_or_else(|| format!("BROWSER_CLICK: no element at index {}", index))?
        .clone();

    if let Some(ref href) = el.href {
        return navigate_http(href);
    }

    if el.is_submit {
        return submit_http_form(&el, &interactables, &forms, &current_url);
    }

    Err(format!(
        "BROWSER_CLICK: element [{}] is not a link or submit button",
        index
    ))
}

fn submit_http_form(
    submit: &HttpInteractable,
    interactables: &[HttpInteractable],
    forms: &[FormSpec],
    _current_url: &str,
) -> Result<String, String> {
    let spec = forms
        .get(submit.form_id)
        .ok_or_else(|| format!("Form submit: invalid form_id {}", submit.form_id))?;

    if spec.multipart {
        return Err(
            "HTTP fallback: form uses enctype multipart/form-data; only application/x-www-form-urlencoded is supported. Use CDP browser for file uploads."
                .to_string(),
        );
    }

    let state = http_state().lock().map_err(|e| e.to_string())?;
    let form_values = state.form_values.clone();
    drop(state);

    let pairs = collect_form_submission_pairs(forms, interactables, &form_values, submit)?;
    let transition_was_post = spec.method_is_post;

    let html = apply_form_submission(spec, &pairs)?;
    let final_url = if spec.method_is_post {
        spec.action_url.clone()
    } else {
        let url_parsed = Url::parse(&spec.action_url).map_err(|e| format!("URL parse: {}", e))?;
        let mut u = url_parsed;
        for (k, v) in &pairs {
            u.query_pairs_mut().append_pair(k, v);
        }
        u.to_string()
    };

    let (interactables_new, forms_new, body_text) = parse_html(&html, &final_url)?;
    let mut state = http_state().lock().map_err(|e| e.to_string())?;
    state.current_url = final_url.clone();
    state.interactables = interactables_new.clone();
    state.forms = forms_new;
    state.body_text = body_text;
    state.form_values.clear();
    drop(state);
    super::set_last_element_labels(
        interactables_new
            .iter()
            .map(|i| (i.index, http_interactable_label(i)))
            .collect(),
    );

    let note = if transition_was_post {
        Some("Last transition: form submitted via POST (application/x-www-form-urlencoded).")
    } else {
        None
    };
    Ok(format_http_state_for_llm(
        &final_url,
        &interactables_new,
        note,
    ))
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
        .ok_or_else(|| format!("BROWSER_INPUT: no element at index {}", index))?
        .clone();
    let name = el.name.clone().ok_or_else(|| {
        format!(
            "BROWSER_INPUT: element [{}] has no name (not an input)",
            index
        )
    })?;
    if el.is_submit {
        return Err("BROWSER_INPUT: use BROWSER_CLICK to submit; element is a button".to_string());
    }
    state
        .form_values
        .insert((el.form_id, name), text.to_string());
    let interactables = state.interactables.clone();
    let url = state.current_url.clone();
    drop(state);
    Ok(format_http_state_for_llm(&url, &interactables, None))
}

/// Return visible body text from last HTTP-fetched page.
pub fn extract_http() -> Result<String, String> {
    let state = http_state().lock().map_err(|e| e.to_string())?;
    if state.body_text.is_empty() && state.current_url.is_empty() {
        return Err("BROWSER_EXTRACT: no page loaded. Use BROWSER_NAVIGATE first.".to_string());
    }
    Ok(state.body_text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_html_for_test(html: &str, base: &str) -> (Vec<HttpInteractable>, Vec<FormSpec>) {
        parse_html(html, base).map(|a| (a.0, a.1)).expect("parse")
    }

    #[test]
    fn post_form_action_login_collects_body_fields() {
        let html = r#"<html><body>
<form method="POST" action="/login" id="f1">
  <input type="hidden" name="csrf" value="tok"/>
  <input type="text" name="user" value=""/>
  <input type="password" name="pass" value=""/>
  <input type="submit" name="go" value="Sign in"/>
</form></body></html>"#;
        let (interactables, forms) =
            parse_html_for_test(html, "https://example.com/app/page");
        let submit = interactables
            .iter()
            .find(|i| i.is_submit)
            .expect("submit");
        assert!(forms[submit.form_id].method_is_post);
        assert_eq!(
            forms[submit.form_id].action_url,
            "https://example.com/login"
        );

        let mut fv = HashMap::new();
        fv.insert((submit.form_id, "user".to_string()), "alice".to_string());
        fv.insert((submit.form_id, "pass".to_string()), "secret".to_string());

        let pairs = collect_form_submission_pairs(&forms, &interactables, &fv, submit).unwrap();
        let mut map: HashMap<&str, &str> = HashMap::new();
        for (k, v) in &pairs {
            map.insert(k.as_str(), v.as_str());
        }
        assert_eq!(map.get("csrf").copied(), Some("tok"));
        assert_eq!(map.get("user").copied(), Some("alice"));
        assert_eq!(map.get("pass").copied(), Some("secret"));
        assert_eq!(map.get("go").copied(), Some("Sign in"));
    }

    #[test]
    fn get_form_uses_query_not_post() {
        let html = r#"<html><body>
<form method="get" action="/search">
  <input type="text" name="q" value=""/>
  <input type="submit" value="Go"/>
</form></body></html>"#;
        let (interactables, forms) = parse_html_for_test(html, "https://shop.example/");
        let submit = interactables.iter().find(|i| i.is_submit).unwrap();
        assert!(!forms[submit.form_id].method_is_post);
        assert_eq!(
            forms[submit.form_id].action_url,
            "https://shop.example/search"
        );
        let mut fv = HashMap::new();
        fv.insert((submit.form_id, "q".to_string()), "shoes".to_string());
        let pairs = collect_form_submission_pairs(&forms, &interactables, &fv, submit).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("q".to_string(), "shoes".to_string()));
    }

    #[test]
    fn malformed_action_non_http_errors_on_parse() {
        let html = r#"<html><body>
<form method="post" action="javascript:void(0)">
  <input type="text" name="x" value=""/>
  <input type="submit" value="Go"/>
</form></body></html>"#;
        let err = parse_html(html, "https://example.com/").unwrap_err();
        assert!(
            err.contains("http") || err.contains("javascript"),
            "err={}",
            err
        );
    }

    #[test]
    fn formless_input_gets_form_id_zero() {
        let html = r#"<html><body>
<input type="text" name="orphan" value=""/>
<button type="submit">Go</button>
</body></html>"#;
        let (interactables, forms) = parse_html_for_test(html, "https://example.com/page");
        let inp = interactables.iter().find(|i| i.name.as_deref() == Some("orphan")).unwrap();
        assert_eq!(inp.form_id, 0);
        assert_eq!(forms[0].action_url, "https://example.com/page");
        assert!(!forms[0].method_is_post);
    }
}
