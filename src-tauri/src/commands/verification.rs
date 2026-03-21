//! Verification pipeline: success criteria extraction, completion checking,
//! topic detection, response summarization helpers, and retry hint building.

use base64::Engine;
use std::path::{Path, PathBuf};

use crate::commands::ollama::send_ollama_chat_messages;
use crate::commands::perplexity_helpers::is_news_query;
use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply, is_grounded_redmine_time_entries_blocked_reply,
    is_redmine_time_entries_request, is_redmine_review_or_summarize_only,
    redmine_time_entries_range,
};
use crate::commands::browser_helpers::{
    browser_retry_grounding_prompt, explicit_no_playable_video_finding, is_browser_task_request,
    is_video_review_request,
};

/// Reply from the agent: text plus optional attachment paths (e.g. screenshots) for Discord.
#[derive(Debug, Clone)]
pub struct OllamaReply {
    pub text: String,
    pub attachment_paths: Vec<PathBuf>,
}

/// Request-local execution context for a single Discord/Ollama run (task-008 Phase 1).
/// Holds only state belonging to this request so verification retries cannot inherit
/// stale criteria, tool payloads, or task context from prior requests.
#[derive(Clone)]
pub struct RequestRunContext {
    pub request_id: String,
    pub retry_count: u32,
    pub original_user_question: String,
    pub discord_channel_id: Option<u64>,
    pub discord_user_id: Option<u64>,
    pub discord_user_name: Option<String>,
}

pub(crate) fn user_explicitly_asked_for_screenshot(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("screenshot")
        || q.contains("take a screenshot")
        || q.contains("create a screenshot")
        || (q.contains("capture") && (q.contains("page") || q.contains("browser")))
}

pub(crate) fn truncate_text_on_line_boundaries(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = String::new();
    let mut used = 0usize;
    for line in text.lines() {
        let line_len = line.chars().count();
        let extra = if out.is_empty() {
            line_len
        } else {
            line_len + 1
        };
        if used + extra > max_chars {
            break;
        }
        if !out.is_empty() {
            out.push('\n');
            used += 1;
        }
        out.push_str(line);
        used += line_len;
    }
    if out.trim().is_empty() {
        let take = max_chars.saturating_sub(12);
        let mut truncated: String = text.chars().take(take).collect();
        truncated.push_str("\n[truncated]");
        return truncated;
    }
    out.push_str("\n[truncated]");
    out
}

pub(crate) fn summarize_response_for_verification(
    question: &str,
    response_content: &str,
    attachment_count: usize,
) -> String {
    if response_content.trim().is_empty() && attachment_count > 0 {
        return format!("{} attachment(s) were sent to the user.", attachment_count);
    }
    let preferred = if is_redmine_time_entries_request(question) {
        extract_redmine_time_entries_summary_for_reply(response_content)
            .unwrap_or_else(|| response_content.to_string())
    } else {
        response_content.to_string()
    };
    let max_chars = if is_redmine_time_entries_request(question) {
        4000
    } else {
        1500
    };
    truncate_text_on_line_boundaries(&preferred, max_chars)
}

/// Read the first image file (png, jpg, jpeg, webp) from paths and return its base64 encoding.
pub(crate) fn first_image_as_base64(paths: &[PathBuf]) -> Option<String> {
    let ext_ok = |p: &Path| {
        p.extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp"))
            .unwrap_or(false)
    };
    for path in paths {
        if ext_ok(path) {
            if let Ok(bytes) = std::fs::read(path) {
                return Some(base64::engine::general_purpose::STANDARD.encode(&bytes));
            }
        }
    }
    None
}

/// Returns the verification-prompt block when the last news search had only hub/landing pages
/// and the request is news-like; otherwise "".
pub(crate) fn verification_news_hub_only_block(news_search_was_hub_only: Option<bool>, question: &str) -> &'static str {
    const HUB_ONLY_BLOCK: &str = "The search results given to the assistant for this news request were only hub/landing/tag/standings pages (no concrete article links). If the assistant's answer presents them as complete recent news articles, reply NO and state that article-grade sources were not found.\n\n";
    if news_search_was_hub_only == Some(true) && is_news_query(question) {
        HUB_ONLY_BLOCK
    } else {
        ""
    }
}

/// For news/current-events requests: tell verifier to accept concise/bullet answers
/// and only require 2+ sources and dates when available.
pub(crate) fn verification_news_format_note(question: &str) -> &'static str {
    const NOTE: &str = "For this news/current-events request: reply YES if the answer has at least 2 named sources and includes dates when available in the results; do not reply NO only because the answer is short or in bullet form.\n\n";
    if is_news_query(question) {
        NOTE
    } else {
        ""
    }
}

pub(crate) fn original_request_for_retry(
    question: &str,
    conversation_history: Option<&[crate::ollama::ChatMessage]>,
    is_verification_retry: bool,
) -> String {
    if !is_verification_retry {
        return question.to_string();
    }
    conversation_history
        .and_then(|history| {
            history
                .iter()
                .rev()
                .find(|msg| msg.role == "user" && !msg.content.trim().is_empty())
        })
        .map(|msg| msg.content.trim().to_string())
        .unwrap_or_else(|| question.to_string())
}

pub(crate) fn sanitize_success_criteria(question: &str, criteria: Vec<String>) -> Vec<String> {
    let q = question.to_lowercase();
    let explicit_last_30_days = q.contains("last 30 days")
        || q.contains("past 30 days")
        || q.contains("30-day")
        || q.contains("30 day")
        || q.contains("this month")
        || q.contains("last month");
    let explicit_last_week = q.contains("last week")
        || q.contains("past week")
        || q.contains("this week")
        || q.contains("last 7 days")
        || q.contains("past 7 days");
    let generic_news_request = q.contains("news");
    let explicit_football_request = q.contains("football")
        || q.contains("soccer")
        || q.contains("fc barcelona")
        || q.contains("barcelona fc")
        || q.contains("barça")
        || q.contains("barca")
        || q.contains("club")
        || q.contains("match")
        || q.contains("transfer")
        || q.contains("la liga");
    let explicit_named_sources = q.contains("bbc")
        || q.contains("cnn")
        || q.contains("reuters")
        || q.contains("ap ")
        || q.contains("associated press");
    let review_videos_request =
        q.contains("video") && (q.contains("review") || q.contains("check"));
    let explicit_playback_request = q.contains("play video")
        || q.contains("play the video")
        || q.contains("playable")
        || q.contains("watch the video")
        || q.contains("start the video");

    let mut sanitized = Vec::new();
    for criterion in criteria {
        let trimmed = criterion.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.contains("last 30 days") && !explicit_last_30_days {
            if generic_news_request {
                sanitized.push("recent news items were summarized".to_string());
            }
            continue;
        }
        if (lower.contains("last week")
            || lower.contains("past week")
            || lower.contains("last 7 days")
            || lower.contains("past 7 days"))
            && !explicit_last_week
        {
            if generic_news_request {
                let replacement = "information includes dates and relevant details".to_string();
                if !sanitized.iter().any(|existing| existing == &replacement) {
                    sanitized.push(replacement);
                }
            }
            continue;
        }
        if generic_news_request && !explicit_football_request
            && (lower.contains("football club")
                || lower.contains("sports website")
                || lower.contains("related to the team"))
            {
                let replacement = if lower.contains("sports website") {
                    "credible named sources cited".to_string()
                } else if lower.contains("related to the team") {
                    "major recent developments involving Barcelona were covered".to_string()
                } else {
                    "recent news items involving Barcelona were summarized".to_string()
                };
                if !sanitized.iter().any(|existing| existing == &replacement) {
                    sanitized.push(replacement);
                }
                continue;
            }
        if generic_news_request
            && !explicit_named_sources
            && (lower.contains("bbc") || lower.contains("cnn") || lower.contains("reuters"))
        {
            if !sanitized
                .iter()
                .any(|existing| existing == "credible named sources cited")
            {
                sanitized.push("credible named sources cited".to_string());
            }
            continue;
        }
        if review_videos_request
            && !explicit_playback_request
            && (lower.contains("video") || lower.contains("playable"))
        {
            let replacement = "video availability or playability was checked".to_string();
            if !sanitized.iter().any(|existing| existing == &replacement) {
                sanitized.push(replacement);
            }
            continue;
        }
        if !sanitized.iter().any(|existing| existing == trimmed) {
            sanitized.push(trimmed.to_string());
        }
    }
    sanitized
}

/// Build a short summary of the last N turns (user/assistant pairs) for the new-topic check.
/// Each message content is truncated to avoid blowing context.
pub(crate) fn summarize_last_turns(messages: &[crate::ollama::ChatMessage], max_turns: usize) -> String {
    const PER_MSG: usize = 120;
    let take = (max_turns * 2).min(messages.len());
    let start = messages.len().saturating_sub(take);
    messages[start..]
        .iter()
        .map(|m| {
            let c: String = m.content.chars().take(PER_MSG).collect();
            let suffix = if m.content.chars().count() > PER_MSG {
                "…"
            } else {
                ""
            };
            format!("{}: {}{}", m.role, c, suffix)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// One short Ollama call to extract 1–3 concrete success criteria from the user request.
/// Returns None on error or empty so the run is not blocked.
pub(crate) async fn extract_success_criteria(
    question: &str,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Option<Vec<String>> {
    let q: String = question.chars().take(800).collect();
    let system = "You are an assistant that extracts success criteria. Reply with 1 to 3 concrete success criteria, one per line (e.g. 'screenshot of the page attached', 'page content fetched'). No numbering, no extra text.";
    let user = format!(
        "User request:\n\n{}\n\nList 1–3 concrete success criteria (one per line). Reply with only the criteria.",
        q
    );
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user,
            images: None,
        },
    ];
    let response = match send_ollama_chat_messages(messages, model_override, options_override).await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("Agent router: success criteria extraction failed: {}", e);
            return None;
        }
    };
    let text = response.message.content.trim();
    let criteria: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() > 2)
        .take(3)
        .map(String::from)
        .collect();
    if criteria.is_empty() {
        tracing::debug!(
            "Agent router: success criteria extraction returned no criteria (empty or parse)"
        );
        None
    } else {
        Some(criteria)
    }
}

/// One short Ollama call to detect if the user's current message is a new topic vs same thread.
/// Returns Ok(true) for NEW_TOPIC, Ok(false) for SAME_TOPIC. On error returns Err (caller may keep history).
pub(crate) async fn detect_new_topic(
    question: &str,
    last_turns_summary: &str,
    model: &str,
) -> Result<bool, String> {
    let system = "You are a classifier. Answer only with exactly one of: NEW_TOPIC or SAME_TOPIC.";
    let user = format!(
        "Given the user's current message and a short summary of the last turns, is the user starting a **new topic** (reply NEW_TOPIC) or continuing the **same thread** (SAME_TOPIC)?\n\nCurrent message: {}\n\nLast turns:\n{}",
        question.chars().take(500).collect::<String>(),
        last_turns_summary
    );
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user,
            images: None,
        },
    ];
    let response = send_ollama_chat_messages(messages, Some(model.to_string()), None).await?;
    let text = response.message.content.trim().to_uppercase();
    if text.contains("NEW_TOPIC") {
        Ok(true)
    } else if text.contains("SAME_TOPIC") {
        Ok(false)
    } else {
        Err(format!(
            "Unexpected response (expected NEW_TOPIC or SAME_TOPIC): {}…",
            text.chars().take(80).collect::<String>()
        ))
    }
}

/// Returns (satisfied, optional reason when not satisfied).
/// When we have image attachment(s) and a local vision model, sends the first image and asks
/// "Does this image satisfy the request?"; otherwise text-only verification.
/// When page_content_from_browser is Some (e.g. from BROWSER_EXTRACT), it is included so the
/// verifier can check requested text against JS-rendered content (SPAs).
/// When news_search_was_hub_only is Some(true), the verifier must not accept an answer that
/// presents hub/landing/tag/standings pages as complete recent news.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn verify_completion(
    question: &str,
    response_content: &str,
    attachment_paths: &[PathBuf],
    success_criteria: Option<&[String]>,
    page_content_from_browser: Option<&str>,
    news_search_was_hub_only: Option<bool>,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<(bool, Option<String>), String> {
    use crate::ollama::models::{get_vision_model_for_verification, is_vision_capable};

    let has_attachments = !attachment_paths.is_empty();
    let screenshot_requested = user_explicitly_asked_for_screenshot(question);
    let response_summary =
        summarize_response_for_verification(question, response_content, attachment_paths.len());
    if is_grounded_redmine_time_entries_blocked_reply(question, &response_summary) {
        tracing::info!("Agent router: verification accepted grounded Redmine blocked-state answer");
        return Ok((true, None));
    }
    if is_video_review_request(question) && explicit_no_playable_video_finding(&response_summary) {
        tracing::info!("Agent router: verification accepted explicit no-playable-video finding");
        return Ok((true, None));
    }
    let system = "You are a completion checker. Answer only with YES or NO, and if NO add one short sentence after a newline explaining what's missing.";
    let criteria_block = success_criteria
        .filter(|c| !c.is_empty())
        .map(|c| {
            format!(
                "Success criteria (from user request):\n{}\n\n",
                c.join("\n")
            )
        })
        .unwrap_or_default();
    let browser_content_block = page_content_from_browser
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let truncated = s.chars().take(8000).collect::<String>();
            format!(
                "Rendered page text from browser (JS-rendered; use this to check if requested text appears on the page):\n\n{}\n\n",
                truncated
            )
        })
        .unwrap_or_default();
    let attachment_block = if screenshot_requested || has_attachments {
        format!(
            "Attachments sent: {}.\n\n",
            if has_attachments { "yes" } else { "no" }
        )
    } else {
        String::new()
    };
    let news_hub_only_block =
        verification_news_hub_only_block(news_search_was_hub_only, question);
    let news_format_note = verification_news_format_note(question);
    let verification_tail = if screenshot_requested {
        "Did we fully satisfy the request (including any requested screenshot/file attachment)? Reply YES or NO. If NO, on the next line add one sentence: what's missing."
    } else {
        "Did we fully satisfy the request? Reply YES or NO. If NO, on the next line add one sentence: what's missing."
    };
    let user_text = format!(
        "Original request: {}\n\n{}{}{}What we did (summary): {}\n\n{}{}{}",
        question.chars().take(500).collect::<String>(),
        criteria_block,
        news_hub_only_block,
        news_format_note,
        response_summary,
        browser_content_block,
        attachment_block,
        verification_tail
    );

    let image_b64 = first_image_as_base64(attachment_paths);
    let vision_model = model_override
        .as_ref()
        .filter(|m| is_vision_capable(m))
        .cloned()
        .or_else(get_vision_model_for_verification);
    let tried_vision = image_b64.is_some() && vision_model.is_some();

    let (messages, verification_model) = if let (Some(b64), Some(vm)) = (image_b64, vision_model) {
        tracing::debug!("Agent router: verification using vision model {}", vm);
        (
            vec![
                crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                    images: None,
                },
                crate::ollama::ChatMessage {
                    role: "user".to_string(),
                    content: user_text.clone(),
                    images: Some(vec![b64]),
                },
            ],
            Some(vm),
        )
    } else {
        (
            vec![
                crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                    images: None,
                },
                crate::ollama::ChatMessage {
                    role: "user".to_string(),
                    content: user_text,
                    images: None,
                },
            ],
            model_override.clone(),
        )
    };

    let response = match send_ollama_chat_messages(
        messages,
        verification_model.clone(),
        options_override.clone(),
    )
    .await
    {
        Ok(r) => r.message.content,
        Err(e) => {
            if tried_vision {
                tracing::debug!(
                    "Agent router: vision verification failed ({}), falling back to text-only",
                    e
                );
                let messages_text = vec![
                    crate::ollama::ChatMessage {
                        role: "system".to_string(),
                        content: system.to_string(),
                        images: None,
                    },
                    crate::ollama::ChatMessage {
                        role: "user".to_string(),
                        content: format!(
                            "Original request: {}\n\n{}{}{}What we did (summary): {}\n\n{}Did we fully satisfy the request? Reply YES or NO. If NO, on the next line add one sentence: what's missing.",
                            question.chars().take(500).collect::<String>(),
                            criteria_block,
                            news_hub_only_block,
                            news_format_note,
                            response_summary,
                            if screenshot_requested {
                                "Attachments sent: yes.\n\n"
                            } else {
                                ""
                            }
                        ),
                        images: None,
                    },
                ];
                match send_ollama_chat_messages(
                    messages_text,
                    model_override,
                    options_override.clone(),
                )
                .await
                {
                    Ok(r) => r.message.content,
                    Err(e2) => {
                        tracing::warn!("Completion verification (text fallback) failed: {}", e2);
                        return Ok((true, None));
                    }
                }
            } else {
                tracing::warn!("Completion verification call failed: {}", e);
                return Ok((true, None));
            }
        }
    };
    let response_upper = response.trim().to_uppercase();
    let satisfied = response_upper.starts_with("YES");
    tracing::debug!(
        "Agent router: verification result: {} (response: {}...)",
        if satisfied { "YES" } else { "NO" },
        response.trim().chars().take(80).collect::<String>()
    );
    let reason = if !satisfied {
        let first_line = response.lines().next().unwrap_or("").trim();
        let rest = response
            .lines()
            .skip(1)
            .map(str::trim)
            .find(|s| !s.is_empty())
            .or_else(|| response.lines().nth(1).map(str::trim))
            .filter(|s| !s.is_empty());
        rest.or(if first_line.len() > 3 {
            Some(first_line)
        } else {
            None
        })
        .map(|s| s.to_string())
    } else {
        None
    };
    Ok((satisfied, reason))
}

/// Build a domain-specific retry hint when verification says the request wasn't satisfied.
///
/// Examines the verification reason and original request to produce targeted retry guidance
/// (e.g. Redmine-specific, news-specific, screenshot-specific, browser-grounding) so the
/// model doesn't wander off into unrelated tool chains on retry.
pub(crate) fn build_verification_retry_hint(
    request_for_verification: &str,
    reason: Option<&str>,
    retry_base: &str,
    response_content: &str,
    attachment_paths: &[PathBuf],
    last_browser_extract: Option<&str>,
) -> String {
    let reason_lower = reason
        .map(|r| r.to_lowercase())
        .unwrap_or_default();

    let reason_about_attachments = reason_lower.contains("screenshot")
        || reason_lower.contains("attachment")
        || (reason_lower.contains("missing")
            && (reason_lower.contains("upload") || reason_lower.contains("sent")));

    let reason_about_time_or_data = reason_lower.contains("time")
        || reason_lower.contains("actual data")
        || reason_lower.contains("spent time")
        || reason_lower.contains("project parameter")
        || (reason_lower.contains("missing")
            && (reason_lower.contains("data") || reason_lower.contains("parameter")));

    let reason_about_json_format = reason_lower.contains("json format")
        || reason_lower.contains("not in json")
        || (reason_lower.contains("response") && reason_lower.contains("json"));

    let reason_about_news_sourcing = reason_lower.contains("source")
        || reason_lower.contains("date")
        || reason_lower.contains("publication")
        || reason_lower.contains("credible")
        || reason_lower.contains("article");

    let response_has_ticket_summary = response_content.len() > 150
        && (response_content.contains("Subject")
            || response_content.contains("Description")
            || response_content.contains("Status")
            || response_content.contains("Redmine"));

    if is_redmine_time_entries_request(request_for_verification) {
        let (from, to) = redmine_time_entries_range(request_for_verification);
        format!(
            "This is a Redmine time-entry list/report request. Stay in that domain only. Do not use BROWSER_*, FETCH_URL, RUN_CMD, TASK_*, or single-issue endpoints like /issues/{{id}}.json unless the user explicitly asks for them. Base the answer only on the actual Redmine time-entry data already fetched, or re-fetch the same period with REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100 if needed. If the result is empty, say no time entries or worked tickets were found for that period. Do not mention screenshots or attachments. Do not return raw tool directives as the final user answer.\n\n{}",
            from, to, retry_base
        )
    } else if is_redmine_review_or_summarize_only(request_for_verification)
        && response_has_ticket_summary
    {
        "The request was only to review/summarize. A summary was already provided. Reply with a brief confirmation and DONE: success; do not update or close the ticket.".to_string()
    } else if reason_about_json_format {
        format!(
            "Success criteria require a response in JSON format. Reply with **valid JSON only** (e.g. total hours, project breakdown, user contributions); do not reply with prose or markdown lists.\n\n{}",
            retry_base
        )
    } else if is_news_query(request_for_verification) && reason_about_news_sourcing {
        tracing::info!(
            "Agent router: verification NO for news (article-grade/sourcing); retrying with PERPLEXITY_SEARCH hint"
        );
        format!(
            "This is a news-summary request. Stay in search-and-summary mode only. Re-run PERPLEXITY_SEARCH with a refined query if needed, but do **not** browse generic homepages, do **not** open BBC/CNN/NYTimes landing pages, and do **not** use screenshots or attachments. Reply with 3 concise bullet points. Each bullet must include: headline/topic, source name, publication date, and one-sentence factual summary. Prefer article-like results; if only hub/landing pages are available, say that clearly.\n\n{}",
            retry_base
        )
    } else if reason_about_time_or_data
        && (request_for_verification.to_lowercase().contains("redmine")
            || request_for_verification.to_lowercase().contains("time")
            || request_for_verification.to_lowercase().contains("spent"))
    {
        format!(
            "Use the correct Redmine API for time entries: REDMINE_API: GET /time_entries.json with from= and to= for the requested period (for example 2026-03-01..2026-03-31 for this month, or the same day for today) and include limit=100 so the results are not truncated. Omit optional filters like user_id or project_id unless the user explicitly asked for them. Do not use /search.json for time entries. Then reply with the data or a clear summary.\n\n{}",
            retry_base
        )
    } else if !attachment_paths.is_empty() && reason_about_attachments {
        let n = attachment_paths.len();
        format!(
            "The app already attached {} file(s) to this reply. If the only missing item was screenshots/attachments, \
             reply with a brief summary of what was done and end with **DONE: success**. \
             Do not invoke AGENT: orchestrator and do not create new tasks.\n\n{}",
            n, retry_base
        )
    } else if attachment_paths.is_empty()
        && user_explicitly_asked_for_screenshot(request_for_verification)
        && reason_lower.contains("screenshot")
    {
        format!(
            "Screenshots could not be attached to this reply. Reply with a brief summary of what was done (e.g. screenshot taken and saved), state that the app could not attach it to Discord, and end with **DONE: no**. \
             Do not invoke AGENT: orchestrator and do not create new tasks.\n\n{}",
            retry_base
        )
    } else if reason_lower.contains("cookie")
        || (reason_lower.contains("consent") && reason_lower.contains("banner"))
    {
        format!(
            "Original request: \"{}\". Verification said the cookie consent banner was not addressed. A screenshot may already have been taken. Complete the remaining step: dismiss the cookie banner (BROWSER_CLICK on the consent button using the Elements list) then BROWSER_SCREENSHOT: current if needed, or reply with a brief summary and **DONE: no** if the browser session is no longer available.\n\n{}",
            request_for_verification.trim(),
            retry_base
        )
    } else if is_browser_task_request(request_for_verification)
        && (last_browser_extract.is_some()
            || response_content.contains("Current page:")
            || reason_lower.contains("browser")
            || reason_lower.contains("click")
            || reason_lower.contains("video")
            || reason_lower.contains("screenshot"))
    {
        browser_retry_grounding_prompt(request_for_verification, retry_base)
    } else {
        retry_base.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_text_on_line_boundaries_does_not_cut_mid_line() {
        let text = "line one\nline two is longer\nline three";
        assert_eq!(
            truncate_text_on_line_boundaries(text, 12),
            "line one\n[truncated]"
        );
    }

    #[test]
    fn summarize_response_for_verification_prefers_structured_redmine_summary() {
        let response = "Redmine API result:\n\nDerived Redmine time-entry summary\nRange: 2026-03-06..2026-03-06\nFetched 1 time entry (total available: 1). Total hours: 2.00\n\nTickets worked:\n- #7209 Fix login — 2.00h across 1 entry (project: Core; users: Ralf; activities: Development)\n\nEntry details:\n- 2026-03-06 | entry 1 | #7209 Fix login | 2.00h | Development | project: Core | user: Ralf\n\nUse this data to answer the user's question.";
        let summary = summarize_response_for_verification(
            "Provide me the list of redmine tickets work on today.",
            response,
            0,
        );
        assert!(summary.contains("Derived Redmine time-entry summary"));
        assert!(summary.contains("#7209 Fix login"));
        assert!(!summary.contains("Entry details:"));
    }

    #[test]
    fn original_request_for_retry_uses_last_real_user_message() {
        let history = vec![
            crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: "Original request".to_string(),
                images: None,
            },
            crate::ollama::ChatMessage {
                role: "assistant".to_string(),
                content: "Intermediate reply".to_string(),
                images: None,
            },
        ];
        let retry_prompt = "Verification said we didn't fully complete the task.";
        assert_eq!(
            original_request_for_retry(retry_prompt, Some(&history), true),
            "Original request".to_string()
        );
    }

    #[test]
    fn original_request_for_retry_ignores_history_when_not_retrying() {
        let history = vec![crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: "Older request".to_string(),
            images: None,
        }];
        assert_eq!(
            original_request_for_retry("Current request", Some(&history), false),
            "Current request".to_string()
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_last_30_days_window() {
        let criteria = vec![
            "reliable news sources cited".to_string(),
            "recent articles (last 30 days)".to_string(),
            "source name and publication date included".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona? Mention sources and dates.",
                criteria
            ),
            vec![
                "reliable news sources cited".to_string(),
                "recent news items were summarized".to_string(),
                "source name and publication date included".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_named_source_examples() {
        let criteria = vec![
            "articles from credible sources like BBC or CNN".to_string(),
            "dates of the news articles".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona? Mention sources and dates.",
                criteria
            ),
            vec![
                "credible named sources cited".to_string(),
                "dates of the news articles".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_relaxes_video_playability_for_review_requests() {
        let criteria = vec![
            "homepage loaded successfully".to_string(),
            "\"about\" section navigable".to_string(),
            "videos playable without errors".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Use browser to review www.amvara.de, click on about and review videos.",
                criteria
            ),
            vec![
                "homepage loaded successfully".to_string(),
                "\"about\" section navigable".to_string(),
                "video availability or playability was checked".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_football_focus_for_generic_barcelona_news() {
        let criteria = vec![
            "recent news articles about Barcelona football club".to_string(),
            "verified sources such as reputable sports websites".to_string(),
            "coverage of major events or updates related to the team".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona?",
                criteria
            ),
            vec![
                "recent news items involving Barcelona were summarized".to_string(),
                "credible named sources cited".to_string(),
                "major recent developments involving Barcelona were covered".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_last_week_for_generic_news() {
        let criteria = vec![
            "recent news articles about Barcelona from credible sources".to_string(),
            "information includes dates and relevant details".to_string(),
            "articles are from the last week".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona?",
                criteria
            ),
            vec![
                "recent news articles about Barcelona from credible sources".to_string(),
                "information includes dates and relevant details".to_string(),
            ]
        );
    }

    #[test]
    fn verification_news_hub_only_block_included_when_hub_only_and_news_query() {
        let block = verification_news_hub_only_block(Some(true), "what's the latest news on Barcelona?");
        assert!(!block.is_empty());
        assert!(block.contains("hub/landing/tag/standings"));
        assert!(block.contains("article-grade sources were not found"));
    }

    #[test]
    fn verification_news_hub_only_block_empty_when_not_news_query() {
        assert_eq!(
            verification_news_hub_only_block(Some(true), "what is 2+2?"),
            ""
        );
    }

    #[test]
    fn verification_news_hub_only_block_empty_when_not_hub_only() {
        assert_eq!(
            verification_news_hub_only_block(Some(false), "latest headlines"),
            ""
        );
        assert_eq!(
            verification_news_hub_only_block(None, "latest headlines"),
            ""
        );
    }
}
