//! Network and API tool dispatch handlers for the agent router tool loop.
//!
//! Contains: FETCH_URL, BRAVE_SEARCH, DISCORD_API, REDMINE_API.
//! Extracted from `commands/ollama.rs` to keep modules small and cohesive.

use tracing::info;

use crate::commands::content_reduction::{reduce_fetched_content_to_fit, CHARS_PER_TOKEN};
use crate::commands::redmine_helpers::is_redmine_review_or_summarize_only;
use crate::commands::untrusted_content::wrap_untrusted_content;

fn send_status(tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>, msg: &str) {
    if let Some(tx) = tx {
        let _ = tx.send(msg.to_string());
    }
}

/// Result from `handle_discord_api`, carrying dedup info alongside the message.
pub(crate) struct DiscordApiResult {
    pub message: String,
    /// When the call succeeded, stores `(method, path)` for consecutive-call dedup.
    pub successful_call: Option<(String, String)>,
}

// ── FETCH_URL (discord.com redirect) ─────────────────────────────────────

pub(crate) async fn handle_fetch_url_discord_redirect(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let path = if let Some(pos) = arg.find("/api/v10") {
        arg[pos + "/api/v10".len()..].to_string()
    } else if let Some(pos) = arg.find("/api/") {
        arg[pos + "/api".len()..].to_string()
    } else {
        String::new()
    };
    if !path.is_empty() {
        info!(
            "Agent router: redirecting FETCH_URL discord.com -> DISCORD_API GET {}",
            path
        );
        send_status(status_tx, &format!("Discord API: GET {}", path));
        match crate::discord::api::discord_api_request("GET", &path, None).await {
            Ok(result) => format!(
                "Discord API result (GET {}):\n\n{}\n\nUse this to answer the user's question.",
                path,
                wrap_untrusted_content("discord-api-response", &result)
            ),
            Err(e) => format!(
                "Discord API failed (GET {}): {}. Try DISCORD_API: GET {} or delegate to AGENT: discord-expert.",
                path, e, path
            ),
        }
    } else {
        info!(
            "Agent router: blocked FETCH_URL for discord.com (no API path). Redirecting to discord-expert."
        );
        "Cannot fetch discord.com pages directly. Discord requires authenticated API access. Use AGENT: discord-expert for all Discord tasks, or use DISCORD_API: GET <path> with the correct API endpoint.".to_string()
    }
}

// ── FETCH_URL (regular) ──────────────────────────────────────────────────

pub(crate) async fn handle_fetch_url(
    arg: &str,
    estimated_context_used: usize,
    context_size_tokens: u32,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<String, String> {
    send_status(
        status_tx,
        &format!("🌐 Fetching page at {}…", crate::logging::ellipse(arg, 45)),
    );
    info!("Discord/Ollama: FETCH_URL requested: {}", arg);
    let url = arg.to_string();
    let fetch_result = tokio::task::spawn_blocking(move || {
        crate::commands::browser::fetch_page_content_for_agent(&url)
    })
    .await
    .map_err(|e| format!("Fetch task: {}", e))?
    .map_err(|e| format!("Fetch page failed: {}", e));

    match fetch_result {
        Ok(body) => {
            let original_len = body.len();
            let cleaned = crate::commands::html_cleaning::clean_html(&body);
            let cleaned_len = cleaned.len();
            if original_len > 0 {
                let ratio = (cleaned_len as f64 / original_len as f64 * 100.0) as u32;
                info!(
                    "FETCH_URL: HTML cleaned {} → {} bytes ({}% of original)",
                    original_len, cleaned_len, ratio
                );
            }
            let body_for_llm = if cleaned.trim().is_empty() {
                "Page fetched but no readable text content found (page may require JavaScript rendering). Try BROWSER_NAVIGATE instead.".to_string()
            } else {
                crate::commands::text_normalize::apply_untrusted_homoglyph_normalization(cleaned)
            };
            let body_fit = reduce_fetched_content_to_fit(
                &body_for_llm,
                context_size_tokens,
                (estimated_context_used / CHARS_PER_TOKEN + 50) as u32,
                model_override,
                options_override,
            )
            .await?;
            crate::commands::suspicious_patterns::log_untrusted_suspicious_scan(
                "fetched-page",
                &body_fit,
            );
            Ok(format!(
                "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                wrap_untrusted_content("fetched-page", &body_fit)
            ))
        }
        Err(e) => {
            if e.contains("401") {
                info!("Discord/Ollama: Fetch returned 401 Unauthorized, stopping");
                Ok("That URL returned 401 Unauthorized. Do not try another URL. Answer based on what you know.".to_string())
            } else {
                info!(
                    "Discord/Ollama: FETCH_URL failed: {}",
                    crate::logging::ellipse(&e, 300)
                );
                let url_lower = arg.to_lowercase();
                let redmine_hint =
                    if url_lower.contains("redmine") || url_lower.contains("/issues/") {
                        " For Redmine tickets use REDMINE_API or say \"review ticket <id>\"."
                    } else {
                        ""
                    };
                Ok(format!(
                    "That URL could not be fetched (connection or server error).{redmine_hint} Answer without that page."
                ))
            }
        }
    }
}

// ── BRAVE_SEARCH ─────────────────────────────────────────────────────────

pub(crate) async fn handle_brave_search(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    send_status(
        status_tx,
        &format!(
            "🌐 Searching the web for \"{}\"…",
            crate::logging::ellipse(arg, 35)
        ),
    );
    info!("Discord/Ollama: BRAVE_SEARCH requested: {}", arg);
    match crate::commands::brave::get_brave_api_key() {
        Some(api_key) => match crate::commands::brave::brave_web_search(arg, &api_key).await {
            Ok(results) => format!(
                "Brave Search results:\n\n{}\n\nUse these to answer the user's question.",
                wrap_untrusted_content("brave-search-results", &results)
            ),
            Err(e) => format!(
                "Brave Search failed: {}. Answer without search results.",
                e
            ),
        },
        None => "Brave Search is not configured (no BRAVE_API_KEY in env or .config.env). Answer without search results.".to_string(),
    }
}

// ── DISCORD_API ──────────────────────────────────────────────────────────

pub(crate) async fn handle_discord_api(
    arg: &str,
    last_successful_call: Option<&(String, String)>,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> DiscordApiResult {
    let arg = arg.trim();
    let (method, rest) = match arg.find(' ') {
        Some(i) => (arg[..i].trim().to_string(), arg[i..].trim()),
        None => ("GET".to_string(), arg),
    };
    let (path_raw, body) = if let Some(idx) = rest.find(" {") {
        let (p, b) = rest.split_at(idx);
        (p.trim().to_string(), Some(b.trim().to_string()))
    } else {
        (rest.to_string(), None)
    };
    let path = crate::commands::agent_session::normalize_discord_api_path(&path_raw);

    if path.is_empty() {
        return DiscordApiResult {
            message: "DISCORD_API requires: DISCORD_API: <METHOD> <path> or DISCORD_API: POST <path> {\"content\":\"...\"}."
                .to_string(),
            successful_call: None,
        };
    }
    if last_successful_call.is_some_and(|(m, p)| m == &method && p == &path) {
        return DiscordApiResult {
            message: "You already received the data for this endpoint above. Format it for the user and reply; do not call DISCORD_API again for the same path."
                .to_string(),
            successful_call: None,
        };
    }

    let status_msg = format!("Calling Discord API: {} {}", method, path);
    send_status(status_tx, &status_msg);
    info!("Discord API: {} {}", method, path);

    match crate::discord::api::discord_api_request(&method, &path, body.as_deref()).await {
        Ok(result) => DiscordApiResult {
            message: format!(
                "Discord API result:\n\n{}\n\nUse this to answer the user's question.",
                wrap_untrusted_content("discord-api-response", &result)
            ),
            successful_call: Some((method, path)),
        },
        Err(e) => {
            let msg = crate::discord::api::sanitize_discord_api_error(&e);
            DiscordApiResult {
                message: format!("Discord API failed: {}. Answer without this result.", msg),
                successful_call: None,
            }
        }
    }
}

// ── REDMINE_API ──────────────────────────────────────────────────────────

pub(crate) async fn handle_redmine_api(
    arg: &str,
    question: &str,
    request_id: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let arg = arg.trim();
    let (method, rest) = match arg.find(' ') {
        Some(i) => (arg[..i].trim().to_string(), arg[i..].trim()),
        None => ("GET".to_string(), arg),
    };
    let (path, body) = if let Some(idx) = rest.find(" {") {
        let (p, b) = rest.split_at(idx);
        (p.trim().to_string(), Some(b.trim().to_string()))
    } else {
        (rest.to_string(), None)
    };

    if path.is_empty() {
        return "REDMINE_API requires: REDMINE_API: GET /issues/1234.json?include=journals,attachments"
            .to_string();
    }

    send_status(status_tx, &format!("Querying Redmine: {} {}", method, path));
    info!(
        "Agent router [{}]: REDMINE_API {} {}",
        request_id, method, path
    );

    match crate::redmine::redmine_api_request(&method, &path, body.as_deref()).await {
        Ok(result) => {
            let wrapped = wrap_untrusted_content("redmine-api-response", &result);
            let mut msg = if path.contains("time_entries") {
                format!(
                    "Redmine API result:\n\n{}\n\nUse this data to answer the user's question. The derived summary above already lists the actual tickets worked (if any), totals, users, projects, and entry details. Use that instead of inventing ticket ids or subjects.",
                    wrapped
                )
            } else {
                format!(
                    "Redmine API result:\n\n{}\n\nUse this data to answer the user's question. Summarize the issue clearly: subject, description quality, what's missing, status, assignee, and key comments.",
                    wrapped
                )
            };
            if method.to_uppercase() == "GET" {
                if is_redmine_review_or_summarize_only(question) {
                    msg.push_str(
                        "\n\nThe user asked only to review/summarize. Do NOT update the ticket or add a comment. Reply with your summary and DONE: success.",
                    );
                } else if path.contains("time_entries") {
                    msg.push_str(
                        "\n\nUse this data to answer. If the user asked for tickets worked, list the actual issue ids and subjects from the derived summary. If the user asked for \"this month\", use from/to for the current month. If success criteria require JSON format, reply with valid JSON only (e.g. total hours, ticket list, project breakdown).",
                    );
                } else {
                    let id = path
                        .trim_start_matches('/')
                        .strip_prefix("issues/")
                        .map(|s| s.split(['.', '?']).next().unwrap_or("").to_string())
                        .unwrap_or_default();
                    if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
                        msg.push_str(&format!(
                            "\n\nIf the user asked to **update** this ticket or **add a comment**, your next reply MUST be exactly one line: REDMINE_API: PUT /issues/{}.json {{\"issue\":{{\"notes\":\"<your comment text>\"}}}}. Do not reply with only a summary.",
                            id
                        ));
                    }
                }
            }
            msg
        }
        Err(e) => format!("Redmine API failed: {}. Answer without this result.", e),
    }
}
