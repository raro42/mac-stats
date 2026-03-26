//! Session compaction: summarise long conversation histories using a
//! fast model and write lessons to memory files.
//!
//! Extracted from `ollama.rs` for cohesion.

/// Minimum number of messages before session compaction triggers.
pub(crate) const COMPACTION_THRESHOLD: usize = 8;
/// Minimum conversational (non-internal) messages to run compaction; fewer means skip (task-008 Phase 5).
pub(crate) const MIN_CONVERSATIONAL_FOR_COMPACTION: usize = 2;

/// Hard cap on the CONTEXT section produced by the compaction LLM.
/// 12 000 bytes ≈ ~3 000 tokens — leaves headroom for system prompt, tools,
/// user question and model reply in a 32 K–40 K context window.
const MAX_COMPACTION_CONTEXT_BYTES: usize = 12_000;
const TRUNCATION_MARKER: &str = " [summary truncated]";

/// Fixed CONTEXT for casual/having_fun sessions so the compactor never invents task or platform context.
const COMPACTOR_CASUAL_CONTEXT: &str =
    "Casual conversation; no task or verified outcome. Not needed for this request.";

/// Minimum messages to compact in the 30-min periodic pass (lower than on-request 8 so we flush more).
const PERIODIC_COMPACTION_MIN_MESSAGES: usize = 4;
/// Sessions with no activity for this long are considered inactive; after compacting they are cleared.
const INACTIVE_THRESHOLD_MINUTES: i64 = 30;

/// Truncate `text` to at most `MAX_COMPACTION_CONTEXT_BYTES`, cutting at the
/// last sentence boundary (`.` / `!` / `?`) within budget to keep the summary
/// coherent for the model.  Uses `floor_char_boundary` so multi-byte UTF-8
/// content never causes a panic.
fn cap_context(text: &str) -> String {
    let budget = MAX_COMPACTION_CONTEXT_BYTES.saturating_sub(TRUNCATION_MARKER.len());
    if text.len() <= MAX_COMPACTION_CONTEXT_BYTES {
        return text.to_string();
    }
    let safe_budget = text.floor_char_boundary(budget);
    let slice = &text[..safe_budget];
    let cut = slice
        .rfind(['.', '!', '?'])
        .map(|i| i + 1) // keep the punctuation
        .unwrap_or(safe_budget);
    format!("{}{}", &text[..cut], TRUNCATION_MARKER)
}

/// Compact a long conversation history into a concise summary using a fast model.
/// Extracts verified facts, successful outcomes, and user intent; drops failures and hallucinations.
/// Also extracts lessons learned (returned separately for memory.md).
/// For Discord having_fun channels, skips the model and returns minimal CONTEXT so we never invent themes (e.g. "language learning platform").
pub(crate) async fn compact_conversation_history(
    messages: &[crate::ollama::ChatMessage],
    current_question: &str,
    discord_channel_id: Option<u64>,
) -> Result<(String, Option<String>), String> {
    use tracing::info;

    if let Some(channel_id) = discord_channel_id {
        if crate::discord::is_discord_channel_having_fun(channel_id) {
            info!(
                "Session compaction: Discord having_fun channel {} — using fixed minimal context (no LLM)",
                channel_id
            );
            return Ok((COMPACTOR_CASUAL_CONTEXT.to_string(), None));
        }
    }

    let pairs: Vec<(String, String)> = messages
        .iter()
        .map(|m| (m.role.clone(), m.content.clone()))
        .collect();
    let conversational = crate::session_memory::count_conversational_messages(&pairs);
    if conversational < MIN_CONVERSATIONAL_FOR_COMPACTION {
        return Err(format!(
            "session has no real conversational value ({} conversational messages, need at least {})",
            conversational, MIN_CONVERSATIONAL_FOR_COMPACTION
        ));
    }

    let small_model = crate::ollama::models::get_global_catalog()
        .and_then(|c| c.resolve_role("small").map(|m| m.name.clone()));

    let model = small_model.or_else(|| {
        let guard = super::ollama_config::get_ollama_client().lock().ok()?;
        let client = guard.as_ref()?;
        Some(client.config.model.clone())
    });

    let conversation_text: String = messages
        .iter()
        .map(|m| format!("[{}]: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    let system_prompt = r#"You are a session compactor. Given a conversation between a user and an assistant, produce TWO sections:

## CONTEXT
A concise summary (max 300 words) of ONLY verified facts and successful outcomes **relevant to the user's current question**. Rules:
- **PRESERVE in CONTEXT:** (1) The **first system or task-setting instructions** (initial 1–2 messages that set task/context) — include their gist or key points so active work and standing instructions survive. (2) The **most recent assistant reply or tool outcome** (the last substantive answer or result) — include it so the current turn's result is not lost.
- **Purely social/casual:** If the conversation has no tool results, no task IDs, no API confirmations (just chat, jokes, or off-topic banter), output ONLY: "Casual conversation; no task or verified outcome. Not needed for this request." and in LESSONS write "None." Do NOT infer themes (e.g. "language learning", "platform", "digital learning") from casual wording.
- If the conversation spans **multiple unrelated topics**, summarize ONLY what is relevant to the **current question** (see below). If the current question is clearly a **new topic** (unrelated to most of the history), output exactly: "Previous context covered different topics; not needed for this request." and keep CONTEXT to that one sentence.
- KEEP: IDs confirmed by API responses (guild IDs, channel IDs, user IDs), successful API calls and their actual results, user preferences and standing instructions, established context the user built up — but only if relevant to the current question. Preserve open decisions and concrete results so active work survives summarization.
- DROP: Failed attempts (401 errors, wrong tool usage, timeouts), hallucinated or unverified claims (assistant saying something happened without API confirmation), apologies, suggestions that weren't followed, repeated back-and-forth about the same error.
- If the assistant claimed an action succeeded but there's no API result confirming it, mark it as UNVERIFIED.
- Write as a factual briefing, not a conversation recap.

## LESSONS
Bullet points of important lessons learned (if any). Things like:
- Tools that worked vs. tools that failed
- Correct IDs or endpoints discovered
- User corrections about how things should work
- Mistakes to avoid in future

If no lessons, write "None."

Output ONLY these two sections, nothing else."#;

    let user_msg = format!(
        "The user's current question is: \"{}\"\n\nCompact this conversation:\n\n{}",
        current_question, conversation_text
    );

    let msgs = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_msg,
            images: None,
        },
    ];

    info!(
        "Session compaction: sending {} messages ({} chars) to model {:?}",
        messages.len(),
        conversation_text.len(),
        model
    );

    let response = super::ollama::send_ollama_chat_messages(
        msgs,
        model,
        None,
        super::ollama::OllamaHttpQueue::Nested,
    )
    .await?;
    let output = response.message.content.trim().to_string();

    let (raw_context, lessons) = parse_compaction_output(&output);
    let context = if raw_context.len() > MAX_COMPACTION_CONTEXT_BYTES {
        let capped = cap_context(&raw_context);
        tracing::warn!(
            "Session compaction: CONTEXT exceeded cap ({} bytes > {}); truncated to {} bytes",
            raw_context.len(),
            MAX_COMPACTION_CONTEXT_BYTES,
            capped.len()
        );
        capped
    } else {
        raw_context
    };
    info!(
        "Session compaction: produced context ({} chars), lessons: {}",
        context.len(),
        lessons.as_deref().unwrap_or("none")
    );

    Ok((context, lessons))
}

/// Parse the compaction output into context and lessons sections.
pub(crate) fn parse_compaction_output(output: &str) -> (String, Option<String>) {
    let lower = output.to_lowercase();
    let context_header = lower.find("## context");
    let lessons_header = lower.find("## lessons");

    let context_body_start = context_header.map(|i| i + "## context".len());
    let lessons_body_start = lessons_header.map(|i| i + "## lessons".len());

    let context = match (context_body_start, lessons_header) {
        (Some(cs), Some(lh)) => output[cs..lh].trim().to_string(),
        (Some(cs), None) => output[cs..].trim().to_string(),
        _ => output.to_string(),
    };

    let lessons = lessons_body_start
        .map(|ls| output[ls..].trim().to_string())
        .filter(|s| !s.is_empty() && s.to_lowercase() != "none." && s.to_lowercase() != "none");

    (context, lessons)
}

/// Run session compaction for all in-memory sessions that meet the threshold.
/// Writes lessons to global memory; replaces active sessions with summary, clears inactive ones.
/// Call from a 30-minute background loop.
pub async fn run_periodic_session_compaction() {
    use tracing::info;
    let sessions = crate::session_memory::list_sessions();
    let now = chrono::Local::now();
    let inactive_cutoff = now - chrono::Duration::minutes(INACTIVE_THRESHOLD_MINUTES);
    for entry in sessions {
        if entry.message_count < PERIODIC_COMPACTION_MIN_MESSAGES {
            continue;
        }
        let messages: Vec<crate::ollama::ChatMessage> =
            crate::session_memory::get_messages(&entry.source, entry.session_id)
                .into_iter()
                .map(|(role, content)| crate::ollama::ChatMessage {
                    role,
                    content,
                    images: None,
                })
                .collect();
        if messages.len() < PERIODIC_COMPACTION_MIN_MESSAGES {
            continue;
        }
        let pairs: Vec<(String, String)> = messages
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect();
        let conversational = crate::session_memory::count_conversational_messages(&pairs);
        if conversational < MIN_CONVERSATIONAL_FOR_COMPACTION {
            info!(
                "Periodic session compaction: skipped for {} {} (no real conversational value: {} conversational messages, need at least {})",
                entry.source, entry.session_id, conversational, MIN_CONVERSATIONAL_FOR_COMPACTION
            );
            continue;
        }
        info!(
            "Periodic session compaction: {} {} ({} messages, last_activity {:?})",
            entry.source,
            entry.session_id,
            messages.len(),
            entry.last_activity
        );
        let pairs_for_hook: Vec<(String, String)> = messages
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect();
        let periodic_rid = format!("periodic-{}-{}", entry.source, entry.session_id);
        crate::commands::compaction_hooks::run_before_compaction_fire_and_forget(
            &entry.source,
            entry.session_id,
            &pairs_for_hook,
            &periodic_rid,
        );
        let mut actual_question = "Periodic session compaction.".to_string();
        for msg in messages.iter().rev() {
            if msg.role == "user" {
                actual_question = msg.content.clone();
                break;
            }
        }

        let discord_ch = if entry.source == "discord" {
            Some(entry.session_id)
        } else {
            None
        };
        let compact_result =
            compact_conversation_history(&messages, &actual_question, discord_ch).await;
        let compact_result = match compact_result {
            Ok(ok) => Ok(ok),
            Err(e) => {
                let err_s = e.to_string();
                if err_s.contains("no real conversational value") {
                    info!(
                        "Periodic session compaction: skipped for {} {}: {}",
                        entry.source, entry.session_id, err_s
                    );
                    continue;
                }
                tracing::warn!(
                    "Periodic session compaction failed for {} {}: {}, retrying once in 3s",
                    entry.source,
                    entry.session_id,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                compact_conversation_history(&messages, &actual_question, discord_ch).await
            }
        };
        match compact_result {
            Ok((context, lessons)) => {
                let lessons_written = lessons
                    .as_ref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                let skip_after_having_fun = entry.source == "discord"
                    && crate::discord::is_discord_channel_having_fun(entry.session_id);
                if !skip_after_having_fun {
                    crate::commands::compaction_hooks::run_after_compaction_fire_and_forget(
                        &entry.source,
                        entry.session_id,
                        messages.len(),
                        lessons_written,
                        &periodic_rid,
                    );
                } else {
                    tracing::debug!(
                        target: "mac_stats::compaction",
                        "after_compaction hook skipped (periodic, Discord having_fun fixed context)"
                    );
                }
                if let Some(ref lesson_text) = lessons {
                    let memory_path = if entry.source == "discord" {
                        crate::config::Config::memory_file_path_for_discord_channel(
                            entry.session_id,
                        )
                    } else {
                        crate::config::Config::memory_file_path()
                    };
                    for line in lesson_text.lines() {
                        let line = line.trim().trim_start_matches("- ").trim();
                        if !line.is_empty() && line.len() > 5 {
                            let entry_line = format!("- {}\n", line);
                            let _ = super::reply_helpers::append_to_file(&memory_path, &entry_line);
                        }
                    }
                    info!(
                        "Periodic session compaction: wrote lessons to {:?}",
                        memory_path
                    );
                }
                crate::commands::ori_lifecycle::maybe_capture_compaction_fire_and_forget(
                    lessons.as_deref(),
                    &entry.source,
                    entry.session_id,
                    &periodic_rid,
                    discord_ch,
                );
                let inactive = entry.last_activity < inactive_cutoff;
                if inactive {
                    crate::session_memory::clear_session(&entry.source, entry.session_id);
                    info!(
                        "Periodic session compaction: cleared inactive session {} {}",
                        entry.source, entry.session_id
                    );
                } else {
                    let compacted = vec![("system".to_string(), context)];
                    crate::session_memory::replace_session(
                        &entry.source,
                        entry.session_id,
                        compacted,
                    );
                    info!(
                        "Periodic session compaction: replaced active session {} {} with summary",
                        entry.source, entry.session_id
                    );
                }
            }
            Err(e) => {
                let err_s = e.to_string();
                if err_s.contains("no real conversational value") {
                    info!(
                        "Periodic session compaction: skipped for {} {}: {}",
                        entry.source, entry.session_id, err_s
                    );
                } else {
                    tracing::warn!(
                        "Periodic session compaction failed for {} {}: {} (session unchanged; will retry next cycle)",
                        entry.source,
                        entry.session_id,
                        e
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_context_short_text_unchanged() {
        let short = "This is a short summary.";
        assert_eq!(cap_context(short), short);
    }

    #[test]
    fn cap_context_exactly_at_limit() {
        let text = "a".repeat(MAX_COMPACTION_CONTEXT_BYTES);
        assert_eq!(cap_context(&text), text);
    }

    #[test]
    fn cap_context_truncates_at_sentence_boundary() {
        let first = "a".repeat(11_950);
        let text = format!(
            "{}. This part should be cut off because the total is over 12000 bytes.",
            first
        );
        assert!(text.len() > MAX_COMPACTION_CONTEXT_BYTES);

        let capped = cap_context(&text);
        assert!(capped.len() <= MAX_COMPACTION_CONTEXT_BYTES);
        assert!(capped.ends_with(TRUNCATION_MARKER));
        assert!(capped.contains(&first));
    }

    #[test]
    fn cap_context_no_sentence_boundary_cuts_at_budget() {
        let text = "a".repeat(MAX_COMPACTION_CONTEXT_BYTES + 500);
        let capped = cap_context(&text);
        assert!(capped.len() <= MAX_COMPACTION_CONTEXT_BYTES);
        assert!(capped.ends_with(TRUNCATION_MARKER));
    }

    #[test]
    fn cap_context_preserves_sentence_punctuation() {
        let prefix = "a".repeat(11_950);
        let text = format!("{}. And then more text follows that pushes us well over the twelve thousand byte limit easily and without doubt.", prefix);
        assert!(text.len() > MAX_COMPACTION_CONTEXT_BYTES);

        let capped = cap_context(&text);
        assert!(capped.ends_with(TRUNCATION_MARKER));
        let before_marker = &capped[..capped.len() - TRUNCATION_MARKER.len()];
        assert!(
            before_marker.ends_with('.'),
            "Expected cut after sentence-ending dot, got: ...{}",
            &before_marker[before_marker.len().saturating_sub(20)..]
        );
    }

    #[test]
    fn cap_context_multibyte_utf8_near_boundary() {
        let prefix = "a".repeat(11_970);
        // each '—' (em-dash) is 3 bytes; place several near the budget boundary
        let text = format!(
            "{}. Summary—with—em—dashes—that—push—past—the—limit—and—keep—going.",
            prefix
        );
        assert!(text.len() > MAX_COMPACTION_CONTEXT_BYTES);

        let capped = cap_context(&text);
        assert!(capped.len() <= MAX_COMPACTION_CONTEXT_BYTES);
        assert!(capped.ends_with(TRUNCATION_MARKER));
        assert!(
            capped.is_char_boundary(capped.len()),
            "result must be valid UTF-8"
        );
    }

    #[test]
    fn cap_context_emoji_near_boundary() {
        let prefix = "a".repeat(11_975);
        // each emoji is 4 bytes
        let text = format!("{}🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥", prefix);
        assert!(text.len() > MAX_COMPACTION_CONTEXT_BYTES);

        let capped = cap_context(&text);
        assert!(capped.len() <= MAX_COMPACTION_CONTEXT_BYTES);
        assert!(capped.ends_with(TRUNCATION_MARKER));
    }

    #[test]
    fn parse_compaction_output_basic() {
        let input = "## CONTEXT\nSome context here.\n\n## LESSONS\n- Lesson one\n- Lesson two";
        let (ctx, lessons) = parse_compaction_output(input);
        assert_eq!(ctx, "Some context here.");
        assert!(lessons.is_some());
        assert!(lessons.unwrap().contains("Lesson one"));
    }

    #[test]
    fn parse_compaction_output_no_lessons() {
        let input = "## CONTEXT\nJust context, no lessons header.";
        let (ctx, lessons) = parse_compaction_output(input);
        assert_eq!(ctx, "Just context, no lessons header.");
        assert!(lessons.is_none());
    }

    #[test]
    fn parse_compaction_output_lessons_none() {
        let input = "## CONTEXT\nSome context.\n\n## LESSONS\nNone.";
        let (_, lessons) = parse_compaction_output(input);
        assert!(lessons.is_none());
    }

    #[test]
    fn parse_compaction_output_no_headers() {
        let input = "Just raw text with no markdown headers at all.";
        let (ctx, lessons) = parse_compaction_output(input);
        assert_eq!(ctx, input);
        assert!(lessons.is_none());
    }

    #[test]
    fn parse_compaction_output_mixed_case_headers() {
        let input = "## Context\nMixed case context.\n\n## Lessons\n- lesson A";
        let (ctx, lessons) = parse_compaction_output(input);
        assert_eq!(ctx, "Mixed case context.");
        assert!(lessons.is_some());
        assert!(lessons.unwrap().contains("lesson A"));
    }

    #[test]
    fn parse_compaction_output_empty_string() {
        let (ctx, lessons) = parse_compaction_output("");
        assert_eq!(ctx, "");
        assert!(lessons.is_none());
    }
}
