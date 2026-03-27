//! Periodic agent heartbeat (OpenClaw-style): one Ollama turn on an interval with a checklist
//! and HEARTBEAT_OK silent-ack handling.

use crate::config::{Config, HeartbeatSettings};
use crate::mac_stats_info;
use crate::mac_stats_warn;
use chrono::Utc;
use std::path::PathBuf;
use std::time::Duration;
use tracing::error;

const HEARTBEAT_TOKEN: &str = "HEARTBEAT_OK";

const DEFAULT_CHECKLIST: &str = "- Glance at anything the user asked you to watch (email, calendar, tasks, system health) if you have tools or files for it.\n- If there is nothing worth interrupting the user for, reply with HEARTBEAT_OK only.";

const HEARTBEAT_SYSTEM_APPEND: &str = "This turn is an **automated heartbeat** from mac-stats. There is **no prior chat** for this turn — do not assume continuity from earlier conversations.\n\n\
**When to reach out:** Important email or messages, a calendar event starting within about two hours, actionable reminders, security or system issues, or anything the user would reasonably want interrupted for.\n\n\
**When to stay quiet:** Reply with HEARTBEAT_OK if nothing is new, the user is likely busy, it is very late unless urgent, you would be repeating a check you already made recently, or the signal is low-value noise.\n\n\
**Rotation:** Over successive heartbeats you may rotate through different checks (email, calendar, notifications, disk space, etc.) so each run stays fast.\n\n\
**Memory (optional):** If the user keeps daily notes or MEMORY.md under ~/.mac-stats, you may occasionally use a beat to consolidate into long-term memory — keep it light.";

fn expand_path_with_tilde(raw: &str) -> PathBuf {
    let p = raw.trim();
    if let Some(rest) = p.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    if p == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(p)
}

fn load_checklist_body(settings: &HeartbeatSettings) -> String {
    if let Some(ref path_str) = settings.checklist_path {
        let path = expand_path_with_tilde(path_str);
        if path.is_file() {
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    mac_stats_info!(
                        "scheduler/heartbeat",
                        "Using checklist file {}",
                        path.display()
                    );
                    return text;
                }
                Err(e) => {
                    mac_stats_warn!(
                        "scheduler/heartbeat",
                        "Failed to read checklist file {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        } else {
            mac_stats_warn!(
                "scheduler/heartbeat",
                "Checklist path not a file: {}",
                path.display()
            );
        }
    }
    if let Some(ref inline) = settings.checklist_prompt {
        mac_stats_info!(
            "scheduler/heartbeat",
            "Using inline checklistPrompt from config"
        );
        return inline.clone();
    }
    mac_stats_info!("scheduler/heartbeat", "Using built-in default checklist");
    DEFAULT_CHECKLIST.to_string()
}

fn build_user_message(checklist: &str) -> String {
    let checklist_trim = checklist.trim();
    crate::commands::suspicious_patterns::log_untrusted_suspicious_scan(
        "heartbeat-checklist",
        checklist_trim,
    );
    let wrapped = crate::commands::untrusted_content::wrap_untrusted_content(
        "heartbeat-checklist",
        checklist_trim,
    );
    format!(
        "This is a scheduled **heartbeat** check (not a live chat message).\n\n\
## Checklist\n\n{}\n\n\
## Response contract\n\
- If nothing needs the user's attention, respond with **HEARTBEAT_OK** only (you may add insignificant whitespace before/after).\n\
- If something needs attention, write a normal message and **do not** include the token HEARTBEAT_OK.\n",
        wrapped
    )
}

/// True when the reply is a silent ack (starts or ends with HEARTBEAT_OK and the remainder is short).
pub(crate) fn is_heartbeat_ack(reply: &str, ack_max_chars: usize) -> bool {
    let s = reply.trim();
    if s.is_empty() {
        return false;
    }
    let t = HEARTBEAT_TOKEN;
    let rest = if s == t {
        ""
    } else if let Some(after) = s.strip_prefix(t) {
        if after.is_empty() || after.starts_with(char::is_whitespace) {
            after.trim_start()
        } else {
            return false;
        }
    } else if let Some(before) = s.strip_suffix(t) {
        if before.is_empty() || before.ends_with(char::is_whitespace) {
            before.trim_end()
        } else {
            return false;
        }
    } else {
        return false;
    };
    rest.chars().count() <= ack_max_chars
}

async fn run_one_beat(settings: &HeartbeatSettings) {
    let checklist = load_checklist_body(settings);
    let question = build_user_message(&checklist);
    let timeout_secs = Config::scheduler_task_timeout_secs();
    let timeout_dur = Duration::from_secs(timeout_secs);
    mac_stats_info!(
        "scheduler/heartbeat",
        "Running heartbeat (timeout={}s, ack_max_chars={})",
        timeout_secs,
        settings.ack_max_chars
    );
    let partial = crate::commands::partial_progress::PartialProgressCapture::new();
    let partial_outer = partial.clone();
    let reply_ch = settings
        .reply_to_channel_id
        .as_ref()
        .and_then(|s| s.parse::<u64>().ok());
    let session_key = reply_ch
        .map(|id| format!("discord:{}", id))
        .unwrap_or_else(|| "heartbeat".to_string());
    let ollama_k = session_key.clone();
    let hb_tick_ts = Utc::now();
    let hb_mid = format!("heartbeat:{}", hb_tick_ts.timestamp_millis());
    let result = crate::keyed_queue::run_serial(session_key, async move {
        tokio::time::timeout(
            timeout_dur,
            crate::commands::ollama::answer_with_ollama_and_fetch(
                crate::commands::ollama::OllamaRequest {
                    question,
                    retry_on_verification_no: true,
                    from_remote: true,
                    allow_schedule: false,
                    conversation_history: None,
                    discord_reply_channel_id: reply_ch,
                    inbound_stale_guard: Some(crate::commands::abort_cutoff::InboundStaleGuard {
                        message_id: hb_mid,
                        timestamp_utc: hb_tick_ts,
                    }),
                    heartbeat_system_append: Some(HEARTBEAT_SYSTEM_APPEND.to_string()),
                    compaction_hook_source: Some("heartbeat".to_string()),
                    partial_progress_capture: Some(partial),
                    ollama_queue_key: Some(ollama_k),
                    ..Default::default()
                },
            ),
        )
        .await
    })
    .await;
    let reply = match result {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            if matches!(
                &e,
                crate::commands::ollama_run_error::OllamaRunError::StaleInboundAfterAbort
            ) {
                mac_stats_info!(
                    "scheduler/heartbeat",
                    "Heartbeat: skipped (stale vs abort cutoff for delivery channel)"
                );
                return;
            }
            error!("Heartbeat: Ollama run failed: {}", e);
            return;
        }
        Err(_) => {
            mac_stats_warn!(
                "scheduler/heartbeat",
                "Heartbeat: Ollama run timed out after {}s (scheduler_task_timeout_secs)",
                timeout_secs
            );
            error!("Heartbeat: Ollama run timed out after {}s", timeout_secs);
            if let Some(summary) = partial_outer.format_user_summary() {
                mac_stats_info!(
                    "scheduler/heartbeat",
                    "Heartbeat timeout partial progress:\n{}",
                    summary
                );
            }
            return;
        }
    };
    if is_heartbeat_ack(&reply.text, settings.ack_max_chars) {
        mac_stats_info!(
            "scheduler/heartbeat",
            "Heartbeat ack — not delivering (HEARTBEAT_OK contract, {} chars)",
            reply.text.chars().count()
        );
        return;
    }
    crate::commands::judge::run_judge_if_enabled(
        "heartbeat",
        &reply.text,
        &reply.attachment_paths,
        None,
    )
    .await;
    let Some(ref ch_str) = settings.reply_to_channel_id else {
        mac_stats_info!(
            "scheduler/heartbeat",
            "Heartbeat: reply not delivered (no replyToChannelId); {} chars, {} attachments",
            reply.text.chars().count(),
            reply.attachment_paths.len()
        );
        return;
    };
    let Ok(channel_id) = ch_str.parse::<u64>() else {
        error!(
            "Heartbeat: invalid replyToChannelId (expected numeric snowflake): {}",
            ch_str
        );
        return;
    };
    let message = format!("**[Heartbeat]**\n\n{}", reply.text.trim());
    if reply.attachment_paths.is_empty() {
        if let Err(e) = crate::discord::send_message_to_channel(channel_id, &message).await {
            error!("Heartbeat: Discord send failed: {}", e);
        } else {
            mac_stats_info!(
                "scheduler/heartbeat",
                "Heartbeat: delivered to Discord channel {}",
                channel_id
            );
        }
    } else if let Err(e) = crate::discord::send_message_to_channel_with_attachments(
        channel_id,
        &message,
        &reply.attachment_paths,
    )
    .await
    {
        error!("Heartbeat: Discord send with attachments failed: {}", e);
    } else {
        mac_stats_info!(
            "scheduler/heartbeat",
            "Heartbeat: delivered (with attachments) to Discord channel {}",
            channel_id
        );
    }
}

async fn heartbeat_loop() {
    loop {
        let settings = Config::heartbeat_settings();
        if !settings.enabled {
            tokio::time::sleep(Duration::from_secs(60)).await;
            continue;
        }
        let wait = Duration::from_secs(settings.interval_secs.max(60));
        mac_stats_info!(
            "scheduler/heartbeat",
            "Heartbeat enabled — next run in {}s",
            wait.as_secs()
        );
        tokio::time::sleep(wait).await;
        let settings = Config::heartbeat_settings();
        if !settings.enabled {
            continue;
        }
        run_one_beat(&settings).await;
    }
}

/// Background thread: when `config.json` → `heartbeat.enabled` is true, runs checklist turns on `intervalSecs`.
pub fn spawn_heartbeat_thread() {
    std::thread::spawn(|| {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                error!("Heartbeat: failed to create tokio runtime: {}", e);
                return;
            }
        };
        mac_stats_info!("scheduler/heartbeat", "Heartbeat thread spawned");
        rt.block_on(heartbeat_loop());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ack_exact_token() {
        assert!(is_heartbeat_ack("HEARTBEAT_OK", 300));
        assert!(is_heartbeat_ack("  HEARTBEAT_OK  ", 300));
    }

    #[test]
    fn ack_prefix_suffix_whitespace() {
        assert!(is_heartbeat_ack("HEARTBEAT_OK\n", 300));
        assert!(is_heartbeat_ack("\nHEARTBEAT_OK", 300));
    }

    #[test]
    fn ack_small_rest() {
        assert!(is_heartbeat_ack("HEARTBEAT_OK\n\nok", 10));
        assert!(!is_heartbeat_ack("HEARTBEAT_OK\n\nok longer text", 5));
    }

    #[test]
    fn not_ack_when_token_in_middle_or_glued() {
        assert!(!is_heartbeat_ack("say HEARTBEAT_OK please", 300));
        assert!(!is_heartbeat_ack("HEARTBEAT_OKX", 300));
    }

    #[test]
    fn not_ack_substantial() {
        let long = format!("HEARTBEAT_OK\n\n{}", "x".repeat(400));
        assert!(!is_heartbeat_ack(&long, 300));
    }
}
