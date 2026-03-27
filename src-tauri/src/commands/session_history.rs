//! Session history preparation: cap, compact, and apply new-topic logic.
//!
//! Extracted from `ollama.rs` to reduce the size of `answer_with_ollama_and_fetch`.

use crate::commands::compaction::{
    compact_conversation_history, COMPACTION_THRESHOLD, MIN_CONVERSATIONAL_FOR_COMPACTION,
};
use crate::commands::compaction_hooks::{
    emit_mac_stats_compaction_event, run_after_compaction_fire_and_forget,
    run_before_compaction_fire_and_forget,
};
use crate::commands::reply_helpers::{append_to_file, looks_like_discord_401_confusion};

pub use crate::commands::compaction_hooks::CompactionLifecycleContext;
use crate::ollama::ChatMessage;

pub(crate) const CONVERSATION_HISTORY_CAP: usize = 20;

/// Prior-turn cap for Discord `having_fun` **idle thought** only (smaller than [`CONVERSATION_HISTORY_CAP`]).
pub(crate) const HAVING_FUN_IDLE_HISTORY_CAP: usize = 10;

const _: () = assert!(HAVING_FUN_IDLE_HISTORY_CAP < CONVERSATION_HISTORY_CAP);

/// Build agent-router **execution** messages: system prompt, then prior turns, then the current user message.
///
/// Order matches `docs/022_feature_review_plan.md` §F2 (history after system, before the current question).
pub(crate) fn build_execution_message_stack(
    system: ChatMessage,
    history: &[ChatMessage],
    user: ChatMessage,
) -> Vec<ChatMessage> {
    let mut msgs = Vec::with_capacity(1 + history.len() + 1);
    msgs.push(system);
    msgs.extend(history.iter().cloned());
    msgs.push(user);
    msgs
}

/// Keep the last `cap` items in chronological order (oldest first).
///
/// Same effect as `items.into_iter().rev().take(cap).rev().collect()` but skips work when
/// `items.len() <= cap`. Used for conversation history caps (agent router, Discord having_fun).
pub(crate) fn cap_tail_chronological<T>(items: Vec<T>, cap: usize) -> Vec<T> {
    if cap == 0 {
        return Vec::new();
    }
    if items.len() <= cap {
        return items;
    }
    items.into_iter().rev().take(cap).rev().collect()
}

fn annotate_discord_401(mut msg: ChatMessage) -> ChatMessage {
    msg.content =
        crate::commands::directive_tags::strip_inline_directive_tags_for_display(&msg.content);
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
    lifecycle: CompactionLifecycleContext,
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

        let hook_src = lifecycle.hook_source.as_str();
        let hook_sid = lifecycle.hook_session_id;
        let msg_count_before = raw_history.len();

        run_before_compaction_fire_and_forget(hook_src, hook_sid, &pairs, request_id);

        if lifecycle.emit_cpu_compaction_ui {
            emit_mac_stats_compaction_event("start", false, request_id, None);
        }

        match compact_conversation_history(&raw_history, question, discord_reply_channel_id).await {
            Ok((context, lessons)) => {
                if lifecycle.emit_cpu_compaction_ui {
                    emit_mac_stats_compaction_event("end", false, request_id, Some(true));
                }

                let lessons_written = lessons
                    .as_ref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                let skip_after_for_having_fun = discord_reply_channel_id
                    .is_some_and(|ch| crate::discord::is_discord_channel_having_fun(ch));
                if !skip_after_for_having_fun {
                    run_after_compaction_fire_and_forget(
                        hook_src,
                        hook_sid,
                        msg_count_before,
                        lessons_written,
                        request_id,
                    );
                } else {
                    tracing::debug!(
                        target: "mac_stats::compaction",
                        "after_compaction hook skipped (Discord having_fun fixed context)"
                    );
                }

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
                crate::commands::ori_lifecycle::maybe_capture_compaction_fire_and_forget(
                    lessons.as_deref(),
                    hook_src,
                    hook_sid,
                    request_id,
                    discord_reply_channel_id,
                );
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
                if lifecycle.emit_cpu_compaction_ui {
                    emit_mac_stats_compaction_event("end", false, request_id, Some(false));
                }

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

#[cfg(test)]
mod tests {
    use super::{
        build_execution_message_stack, cap_tail_chronological, prepare_conversation_history,
        CompactionLifecycleContext, CONVERSATION_HISTORY_CAP, HAVING_FUN_IDLE_HISTORY_CAP,
    };
    use crate::commands::compaction::COMPACTION_THRESHOLD;
    use crate::ollama::ChatMessage;

    /// Below-threshold `prepare_conversation_history` tests use 1–2 turns; compaction must not run.
    const _: () = assert!(COMPACTION_THRESHOLD > 2);

    const SYSTEM_CORRECTION_401: &str =
        "[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]";

    #[test]
    fn execution_stack_order_system_history_user() {
        let system = ChatMessage {
            role: "system".to_string(),
            content: "sys".to_string(),
            images: None,
        };
        let h1 = ChatMessage {
            role: "user".to_string(),
            content: "u1".to_string(),
            images: None,
        };
        let h2 = ChatMessage {
            role: "assistant".to_string(),
            content: "a1".to_string(),
            images: None,
        };
        let user = ChatMessage {
            role: "user".to_string(),
            content: "current".to_string(),
            images: None,
        };
        let out = build_execution_message_stack(system, &[h1.clone(), h2.clone()], user.clone());
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].role, "system");
        assert_eq!(out[0].content, "sys");
        assert_eq!(out[1].role, h1.role);
        assert_eq!(out[1].content, h1.content);
        assert_eq!(out[2].role, h2.role);
        assert_eq!(out[2].content, h2.content);
        assert_eq!(out[3].role, user.role);
        assert_eq!(out[3].content, user.content);
    }

    #[test]
    fn execution_stack_empty_history() {
        let system = ChatMessage {
            role: "system".to_string(),
            content: "s".to_string(),
            images: None,
        };
        let user = ChatMessage {
            role: "user".to_string(),
            content: "q".to_string(),
            images: None,
        };
        let out = build_execution_message_stack(system, &[], user.clone());
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].role, user.role);
        assert_eq!(out[1].content, user.content);
    }

    #[test]
    fn cap_tail_keeps_last_n_in_chronological_order() {
        // docs/022_feature_review_plan.md §F1: same cap as `answer_with_ollama_and_fetch` + Discord reply path.
        let cap = CONVERSATION_HISTORY_CAP;
        let total = cap + 5;
        let v: Vec<i32> = (1..=total as i32).collect();
        let out = cap_tail_chronological(v, cap);
        assert_eq!(out.len(), cap);
        let start = (total - cap + 1) as i32;
        assert_eq!(out, (start..=total as i32).collect::<Vec<_>>());
    }

    #[test]
    fn cap_tail_unchanged_when_vec_shorter_than_cap() {
        let v = vec!["a", "b"];
        let out = cap_tail_chronological(v, 20);
        assert_eq!(out, vec!["a", "b"]);
    }

    #[test]
    fn cap_tail_zero_returns_empty() {
        let v = vec![1, 2, 3];
        assert!(cap_tail_chronological(v, 0).is_empty());
    }

    #[test]
    fn cap_tail_exact_cap_preserves_all() {
        let v = vec![1, 2, 3, 4, 5];
        let out = cap_tail_chronological(v, 5);
        assert_eq!(out, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn conversation_history_caps_match_discord_contract() {
        // docs/022_feature_review_plan.md §F1: router and Discord having_fun reply share CONVERSATION_HISTORY_CAP.
        // Ordering idle < conversation is enforced at compile time (const assert above).
        assert_eq!(CONVERSATION_HISTORY_CAP, 20);
        assert_eq!(HAVING_FUN_IDLE_HISTORY_CAP, 10);
    }

    #[tokio::test]
    async fn prepare_history_new_topic_below_compaction_threshold_clears() {
        let raw = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "earlier".to_string(),
                images: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "old reply".to_string(),
                images: None,
            },
        ];
        let out = prepare_conversation_history(
            raw,
            "fresh question",
            true,
            None,
            "test-req-new-topic",
            CompactionLifecycleContext::default(),
        )
        .await;
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn prepare_history_below_threshold_appends_discord_401_correction() {
        let confused = "Got 401 Unauthorized: bad bearer token calling Discord API.";
        let raw = vec![ChatMessage {
            role: "assistant".to_string(),
            content: confused.to_string(),
            images: None,
        }];
        let out = prepare_conversation_history(
            raw,
            "follow-up",
            false,
            None,
            "test-req-401-annotate",
            CompactionLifecycleContext::default(),
        )
        .await;
        assert_eq!(out.len(), 1);
        assert!(out[0].content.contains(confused));
        assert!(out[0].content.contains(SYSTEM_CORRECTION_401));
    }
}
