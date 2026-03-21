//! Session history preparation: cap, compact, and apply new-topic logic.
//!
//! Extracted from `ollama.rs` to reduce the size of `answer_with_ollama_and_fetch`.

use crate::commands::compaction::{
    compact_conversation_history, COMPACTION_THRESHOLD, MIN_CONVERSATIONAL_FOR_COMPACTION,
};
use crate::commands::reply_helpers::{append_to_file, looks_like_discord_401_confusion};
use crate::ollama::ChatMessage;

pub(crate) const CONVERSATION_HISTORY_CAP: usize = 20;

fn annotate_discord_401(mut msg: ChatMessage) -> ChatMessage {
    if msg.role == "assistant" && looks_like_discord_401_confusion(&msg.content) {
        msg.content.push_str(
            "\n\n[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]",
        );
    }
    msg
}

/// Cap, compact (if above threshold), and apply new-topic clearing to the raw conversation history.
/// Returns the prepared history ready for use in planning/execution prompts.
pub(crate) async fn prepare_conversation_history(
    raw_history: Vec<ChatMessage>,
    question: &str,
    is_new_topic: bool,
    discord_reply_channel_id: Option<u64>,
    request_id: &str,
) -> Vec<ChatMessage> {
    use tracing::info;

    if raw_history.len() >= COMPACTION_THRESHOLD {
        let pairs: Vec<(String, String)> = raw_history
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect();
        let conversational = crate::session_memory::count_conversational_messages(&pairs);

        if conversational < MIN_CONVERSATIONAL_FOR_COMPACTION {
            info!(
                "Session compaction [{}]: skipped (no real conversational value: {} conversational messages, need at least {}); keeping full history ({} messages)",
                request_id, conversational, MIN_CONVERSATIONAL_FOR_COMPACTION, raw_history.len()
            );
            return raw_history.into_iter().map(annotate_discord_401).collect();
        }

        info!(
            "Session compaction [{}]: {} messages exceed threshold ({}), compacting",
            request_id,
            raw_history.len(),
            COMPACTION_THRESHOLD
        );
        match compact_conversation_history(&raw_history, question, discord_reply_channel_id).await {
            Ok((context, lessons)) => {
                info!(
                    "Session compaction [{}]: produced context ({} chars), lessons: {}",
                    request_id,
                    context.len(),
                    lessons.as_ref().map(|_| "present").unwrap_or("none")
                );
                if let Some(ref lesson_text) = lessons {
                    let memory_path = discord_reply_channel_id
                        .map(crate::config::Config::memory_file_path_for_discord_channel)
                        .unwrap_or_else(crate::config::Config::memory_file_path);
                    for line in lesson_text.lines() {
                        let line = line.trim().trim_start_matches("- ").trim();
                        if !line.is_empty() && line.len() > 5 {
                            let entry = format!("- {}\n", line);
                            let _ = append_to_file(&memory_path, &entry);
                        }
                    }
                    info!(
                        "Session compaction [{}]: wrote lessons to {:?}",
                        request_id, memory_path
                    );
                }
                let context_lower = context.to_lowercase();
                let not_needed = context_lower.contains("not needed for this request")
                    || context_lower.contains("covered different topics");
                if let Some(channel_id) = discord_reply_channel_id {
                    let compacted = vec![
                        ("system".to_string(), context.clone()),
                        ("user".to_string(), question.to_string()),
                    ];
                    crate::session_memory::replace_session("discord", channel_id, compacted);
                }
                if is_new_topic || not_needed {
                    info!(
                        "Session compaction: prior context not relevant to current question (new topic), using no prior context"
                    );
                    vec![]
                } else {
                    info!(
                        "Session compaction: replaced {} messages with summary ({} chars)",
                        raw_history.len(),
                        context.len()
                    );
                    vec![ChatMessage {
                        role: "system".to_string(),
                        content: format!(
                            "Previous session context (compacted from {} messages):\n\n{}",
                            raw_history.len(),
                            context
                        ),
                        images: None,
                    }]
                }
            }
            Err(e) => {
                let n = raw_history.len();
                let err_s = e.to_string();
                let is_skip = err_s.contains("no real conversational value");
                if is_skip {
                    info!(
                        "Session compaction skipped: {}; keeping full history ({} messages)",
                        err_s, n
                    );
                } else {
                    let msg = if err_s.to_lowercase().contains("unauthorized")
                        || err_s.contains("401")
                    {
                        format!(
                            "Session compaction failed: {} (use a local model for compaction; cloud models may require auth). Keeping full history ({} messages) for this request.",
                            e, n
                        )
                    } else {
                        format!(
                            "Session compaction failed: {}; keeping full history ({} messages) for this request.",
                            e, n
                        )
                    };
                    tracing::warn!("{}", msg);
                }
                raw_history.into_iter().map(annotate_discord_401).collect()
            }
        }
    } else if is_new_topic {
        vec![]
    } else {
        raw_history.into_iter().map(annotate_discord_401).collect()
    }
}
