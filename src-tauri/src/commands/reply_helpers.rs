//! Reply-routing helpers, Mastodon integration, and utility functions.
//!
//! Extracted from `ollama.rs` to keep the main command file focused on
//! chat execution and tool orchestration.

use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply, grounded_redmine_time_entries_failure_reply,
    question_explicitly_requests_json,
};

pub(crate) fn strip_tool_result_instructions(tool_result: &str) -> String {
    let mut cleaned = tool_result;
    for marker in [
        "\n\nUse this data to answer the user's question.",
        "\n\nUse only this Redmine data to continue or answer",
        "\n\nUse this data to answer.",
        "\n\nUse this to answer the user's question.",
    ] {
        if let Some(idx) = cleaned.find(marker) {
            cleaned = &cleaned[..idx];
            break;
        }
    }
    for prefix in [
        "Redmine API result:\n\n",
        "Discord API result:\n\n",
        "Here is the command output:\n\n",
        "Here is the page content:\n\n",
        "Search results:\n\n",
    ] {
        if let Some(rest) = cleaned.strip_prefix(prefix) {
            cleaned = rest;
            break;
        }
    }
    cleaned.trim().to_string()
}

pub(crate) fn final_reply_from_tool_results(question: &str, tool_result: &str) -> String {
    if let Some(reply) = grounded_redmine_time_entries_failure_reply(question, tool_result) {
        return reply;
    }
    if !question_explicitly_requests_json(question) {
        if let Some(summary) = extract_redmine_time_entries_summary_for_reply(tool_result) {
            return summary;
        }
    }
    let cleaned = strip_tool_result_instructions(tool_result);
    if cleaned.is_empty() {
        "The requested tool ran, but no final user-facing answer was produced.".to_string()
    } else {
        cleaned
    }
}

/// Resolve Mastodon credentials: instance URL and access token.
/// Checks env vars (MASTODON_INSTANCE_URL, MASTODON_ACCESS_TOKEN), then ~/.mac-stats/.config.env,
/// then Keychain (mastodon_instance_url, mastodon_access_token).
pub(crate) fn get_mastodon_config() -> Option<(String, String)> {
    let resolve = |env_key: &str, file_key: &str, keychain_key: &str| -> Option<String> {
        if let Ok(v) = std::env::var(env_key) {
            let v = v.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
        for base in [
            std::env::current_dir().ok(),
            std::env::var("HOME").ok().map(std::path::PathBuf::from),
        ]
        .into_iter()
        .flatten()
        {
            let paths = [
                base.join(".config.env"),
                base.join(".mac-stats").join(".config.env"),
            ];
            for p in &paths {
                if let Ok(content) = std::fs::read_to_string(p) {
                    for line in content.lines() {
                        if let Some(val) = line.strip_prefix(file_key) {
                            let val = val.trim().trim_matches('"').trim().to_string();
                            if !val.is_empty() {
                                return Some(val);
                            }
                        }
                    }
                }
            }
        }
        if let Ok(Some(v)) = crate::security::get_credential(keychain_key) {
            if !v.is_empty() {
                return Some(v);
            }
        }
        None
    };
    let instance = resolve(
        "MASTODON_INSTANCE_URL",
        "MASTODON_INSTANCE_URL=",
        "mastodon_instance_url",
    )?;
    let token = resolve(
        "MASTODON_ACCESS_TOKEN",
        "MASTODON_ACCESS_TOKEN=",
        "mastodon_access_token",
    )?;
    Some((instance.trim_end_matches('/').to_string(), token))
}

/// Post a status to Mastodon. Visibility: public, unlisted, private, or direct.
pub(crate) async fn mastodon_post(status: &str, visibility: &str) -> Result<String, String> {
    let (instance, token) = get_mastodon_config()
        .ok_or("Mastodon not configured. Set MASTODON_INSTANCE_URL and MASTODON_ACCESS_TOKEN in env or ~/.mac-stats/.config.env")?;
    let url = format!("{}/api/v1/statuses", instance);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let payload = serde_json::json!({
        "status": status,
        "visibility": visibility,
    });
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Mastodon API request failed: {}", e))?;
    let status_code = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status_code.is_success() {
        let url = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()));
        Ok(match url {
            Some(u) => format!("Posted to Mastodon: {}", u),
            None => "Posted to Mastodon successfully.".to_string(),
        })
    } else {
        Err(format!("Mastodon API error {}: {}", status_code, body))
    }
}

/// Append a line to a file, creating it if needed. Returns the path on success.
pub(crate) fn append_to_file(
    path: &std::path::Path,
    content: &str,
) -> Result<std::path::PathBuf, String> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open: {}", e))?;
    f.write_all(content.as_bytes())
        .map_err(|e| format!("write: {}", e))?;
    Ok(path.to_path_buf())
}

/// Detect prior assistant messages that mention 401/token errors about Discord (from FETCH_URL misuse).
pub(crate) fn looks_like_discord_401_confusion(content: &str) -> bool {
    let lower = content.to_lowercase();
    (lower.contains("401") || lower.contains("unauthorized"))
        && (lower.contains("token")
            || lower.contains("credential")
            || lower.contains("authentication"))
        && (lower.contains("discord") || lower.contains("guild") || lower.contains("channel"))
}

/// Extract a URL from the question for pre-routing (e.g. screenshot). Prefers https?:// then www.
/// Strips trailing punctuation from the URL.
pub(crate) fn extract_url_from_question(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for prefix in ["https://", "http://"] {
        if let Some(pos) = lower.find(prefix) {
            let start = text[pos..]
                .char_indices()
                .next()
                .map(|(o, _)| pos + o)
                .unwrap_or(pos);
            let after = &text[start + prefix.len()..];
            let end = after
                .char_indices()
                .find(|(_, c)| {
                    *c == ' '
                        || *c == '\n'
                        || *c == '\t'
                        || *c == ')'
                        || *c == ']'
                        || *c == '"'
                        || *c == '>'
                })
                .map(|(i, _)| i)
                .unwrap_or(after.len());
            let url = format!("{}{}", &text[start..start + prefix.len()], &after[..end]);
            let url = url.trim_end_matches(['.', ',', ';', '!', '?']);
            if !url.is_empty() && url.len() > prefix.len() {
                return Some(url.to_string());
            }
        }
    }
    if let Some(pos) = lower.find("www.") {
        let after = &text[pos..];
        let end = after
            .char_indices()
            .find(|(_, c)| {
                *c == ' '
                    || *c == '\n'
                    || *c == '\t'
                    || *c == ')'
                    || *c == ']'
                    || *c == '"'
                    || *c == '>'
            })
            .map(|(i, _)| i)
            .unwrap_or(after.len());
        let url = after[..end].trim_end_matches(['.', ',', ';', '!', '?']);
        if url.len() > 4 {
            let full = if url.starts_with("http") {
                url.to_string()
            } else {
                format!("https://{}", url)
            };
            return Some(full);
        }
    }
    None
}

/// If the question clearly asks for a screenshot of a URL, return RECOMMEND string for pre-routing.
/// Skip pre-route when user wants multi-step or cookie consent removal — let planner handle those.
pub(crate) fn extract_screenshot_recommendation(question: &str) -> Option<String> {
    let q = question.trim();
    let q_lower = q.to_lowercase();
    let has_multi_step = q_lower.contains("navigate all")
        || q_lower.contains("find ")
        || q_lower.contains("when you found")
        || q_lower.contains("when found")
        || (q_lower.contains("click on")
            || q_lower.contains("and click")
            || q_lower.contains("then click"));
    let asks_cookie_consent = q_lower.contains("cookie")
        && (q_lower.contains("remove")
            || q_lower.contains("dismiss")
            || q_lower.contains("banner")
            || q_lower.contains("consent"));
    if has_multi_step || asks_cookie_consent {
        return None;
    }
    let has_screenshot_intent = q_lower.contains("screenshot")
        || q_lower.contains("take a screenshot")
        || q_lower.contains("create a screenshot")
        || q_lower.contains("capture the page")
        || (q_lower.contains("capture")
            && (q_lower.contains("page") || q_lower.contains("browser")));
    let has_browser_or_url_context = q_lower.contains("browser")
        || q_lower.contains("chrome")
        || q_lower.contains("goto")
        || q_lower.contains("go to")
        || q_lower.contains("visit")
        || q_lower.contains("navigate")
        || q_lower.contains("http")
        || q_lower.contains("www.");
    if has_screenshot_intent && has_browser_or_url_context {
        if let Some(url) = extract_url_from_question(q) {
            let rec = format!("BROWSER_NAVIGATE: {}\nBROWSER_SCREENSHOT: current", url);
            tracing::info!(
                "Agent router: pre-routed to BROWSER_NAVIGATE + BROWSER_SCREENSHOT (browser-use style): {}",
                crate::logging::ellipse(&url, 60)
            );
            return Some(rec);
        }
    }
    None
}

/// Extract the trimmed argument after the last literal tool prefix in the user's message.
pub(crate) fn extract_last_prefixed_argument(text: &str, prefix: &str) -> Option<String> {
    if prefix.is_empty() || text.len() < prefix.len() {
        return None;
    }
    let mut last_match = None;
    for (idx, _) in text.char_indices() {
        if let Some(candidate) = text.get(idx..idx + prefix.len()) {
            if candidate.eq_ignore_ascii_case(prefix) {
                last_match = Some(idx);
            }
        }
    }
    let start = last_match?;
    let arg = text.get(start + prefix.len()..)?.trim();
    if arg.is_empty() {
        None
    } else {
        Some(arg.to_string())
    }
}

/// True when the planner's recommendation is only "DONE: no" or "DONE: success" with no actual tool steps.
pub(crate) fn is_bare_done_plan(s: &str) -> bool {
    let t = s.trim().trim_matches('*').trim();
    t.eq_ignore_ascii_case("DONE: no") || t.eq_ignore_ascii_case("DONE: success")
}

fn normalize_reply_for_compare(s: &str) -> String {
    let mut out = s.trim().to_string();
    for suffix in [
        "\n\nDONE: success",
        "\n\nDONE: no",
        "\nDONE: success",
        "\nDONE: no",
    ] {
        if out.ends_with(suffix) {
            out = out.strip_suffix(suffix).unwrap_or(&out).trim().to_string();
            break;
        }
    }
    out
}

/// True when the final reply is effectively the same as the intermediate.
pub(crate) fn is_final_same_as_intermediate(intermediate: &str, final_answer: &str) -> bool {
    let a = normalize_reply_for_compare(intermediate);
    let b = normalize_reply_for_compare(final_answer);
    if a == b {
        return true;
    }
    if b.len() < 120 && !b.contains('\n') && a.len() > 150 {
        let b_lower = b.to_lowercase();
        if b_lower.contains("same as intermediate")
            || b_lower.contains("confirmed")
            || (b_lower.contains("done") && !b_lower.contains("error"))
            || b_lower == "done."
            || b_lower == "confirmed."
        {
            return true;
        }
    }
    false
}

pub(crate) fn is_agent_unavailable_error(error: &str) -> bool {
    let e = error.to_lowercase();
    e.contains("busy or unavailable")
        || e.contains("timed out")
        || e.contains("timeout")
        || e.contains("503")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_last_prefixed_argument_prefers_last_run_cmd_literal() {
        let question = "Run this command and show me the output: RUN_CMD: cat /etc/hosts";
        assert_eq!(
            extract_last_prefixed_argument(question, "RUN_CMD:"),
            Some("cat /etc/hosts".to_string())
        );
    }

    #[test]
    fn final_reply_from_tool_results_uses_redmine_summary_not_raw_tool_wrapper() {
        let tool_result = "Redmine API result:\n\nDerived Redmine time-entry summary\nRange: 2026-03-06..2026-03-06\nFetched 0 time entries (total available: 0). Total hours: 0.00\n\nTickets worked:\n- None found in these time entries.\n\nUse this data to answer the user's question.";
        let reply = final_reply_from_tool_results(
            "Provide me the list of redmine tickets work on today.",
            tool_result,
        );
        assert!(reply.starts_with("Derived Redmine time-entry summary"));
        assert!(!reply.starts_with("Redmine API result:"));
    }

    #[test]
    fn looks_like_discord_401_confusion_true_when_unauthorized_token_and_discord() {
        assert!(looks_like_discord_401_confusion(
            "Got 401 Unauthorized: bad bearer token calling Discord API."
        ));
    }

    #[test]
    fn looks_like_discord_401_confusion_true_for_guild_channel_credential_wording() {
        assert!(looks_like_discord_401_confusion(
            "UNAUTHORIZED: invalid credentials for this guild channel endpoint."
        ));
    }

    #[test]
    fn looks_like_discord_401_confusion_false_without_discord_context() {
        assert!(!looks_like_discord_401_confusion(
            "401 Unauthorized: missing or invalid API token."
        ));
    }

    #[test]
    fn looks_like_discord_401_confusion_false_without_auth_signal() {
        assert!(!looks_like_discord_401_confusion(
            "Could not list channels in the Discord server."
        ));
    }
}
