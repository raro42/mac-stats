//! Session compaction: summarise long conversation histories using a
//! fast model and write lessons to memory files.
//!
//! Extracted from `ollama.rs` for cohesion.

/// Minimum number of messages before session compaction triggers.
pub(crate) const COMPACTION_THRESHOLD: usize = 8;
/// Minimum conversational (non-internal) messages to run compaction; fewer means skip (task-008 Phase 5).
pub(crate) const MIN_CONVERSATIONAL_FOR_COMPACTION: usize = 2;

/// Fixed CONTEXT for casual/having_fun sessions so the compactor never invents task or platform context.
const COMPACTOR_CASUAL_CONTEXT: &str =
    "Casual conversation; no task or verified outcome. Not needed for this request.";

/// Minimum messages to compact in the 30-min periodic pass (lower than on-request 8 so we flush more).
const PERIODIC_COMPACTION_MIN_MESSAGES: usize = 4;
/// Sessions with no activity for this long are considered inactive; after compacting they are cleared.
const INACTIVE_THRESHOLD_MINUTES: i64 = 30;

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

    let response = super::ollama::send_ollama_chat_messages(msgs, model, None).await?;
    let output = response.message.content.trim().to_string();

    let (context, lessons) = parse_compaction_output(&output);
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
        let compact_result = compact_conversation_history(&messages, &actual_question, discord_ch).await;
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
