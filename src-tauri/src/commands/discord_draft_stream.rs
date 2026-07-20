//! Throttled in-place Discord message edits for long-running Ollama tool loops.
//!
//! Discord path sends a placeholder message, then edits it periodically while tools run,
//! then flushes the final reply into the same message (first chunk only; extra chunks use new messages).

use std::time::{Duration, Instant};

use serenity::builder::EditMessage;
use serenity::client::Context;
use serenity::model::channel::Message;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, warn};

use crate::commands::outbound_pipeline::DISCORD_CONTENT_MAX_CHARS;

#[derive(Debug)]
enum Cmd {
    Update(String),
    Flush {
        text: String,
        reply: oneshot::Sender<bool>,
    },
    /// Delete the placeholder message (silent-expected empty final; no `(No reply text.)`).
    AbandonSilent {
        reply: oneshot::Sender<()>,
    },
    Stop,
}

/// Handle to queue draft edits from the tool loop (clone per use; shares one background task).
#[derive(Clone)]
pub struct DiscordDraftHandle {
    tx: mpsc::UnboundedSender<Cmd>,
}

impl DiscordDraftHandle {
    /// Queue the latest draft text. Coalesced with other updates inside the worker (latest wins per throttle window).
    pub fn update(&self, text: impl Into<String>) {
        let text = text.into();
        if self.tx.send(Cmd::Update(text)).is_err() {
            debug!(target: "discord/draft", "draft update dropped (editor stopped)");
        }
    }

    /// Replace the message with `text` immediately after any in-flight work, ignoring throttle.
    /// Returns `true` when Discord accepted the edit (or text was empty); `false` on drop/API failure.
    pub async fn flush(&self, text: &str) -> bool {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .tx
            .send(Cmd::Flush {
                text: text.to_string(),
                reply: reply_tx,
            })
            .is_err()
        {
            debug!(target: "discord/draft", "draft flush dropped (editor stopped)");
            return false;
        }
        reply_rx.await.unwrap_or(false)
    }

    pub fn stop(&self) {
        let _ = self.tx.send(Cmd::Stop);
    }

    /// Delete the Discord placeholder message (used when `silent_user_output` and the final text is empty).
    pub async fn abandon_silent(&self) {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .tx
            .send(Cmd::AbandonSilent { reply: reply_tx })
            .is_err()
        {
            debug!(target: "discord/draft", "silent abandon dropped (editor stopped)");
            return;
        }
        let _ = reply_rx.await;
    }
}

/// Truncate to Discord's per-message limit at a char boundary.
pub(crate) fn clamp_discord_content(s: &str) -> String {
    let n = s.chars().count();
    if n <= DISCORD_CONTENT_MAX_CHARS {
        return s.to_string();
    }
    let mut out = String::new();
    for ch in s.chars().take(DISCORD_CONTENT_MAX_CHARS.saturating_sub(1)) {
        out.push(ch);
    }
    out.push('…');
    out
}

async fn apply_edit(ctx: &Context, message: &mut Message, content: &str, label: &str) -> bool {
    let clipped = clamp_discord_content(content);
    if clipped.is_empty() {
        return true;
    }
    match message
        .edit(ctx, EditMessage::new().content(&clipped))
        .await
    {
        Ok(()) => {
            info!(
                target: "discord/draft",
                "{} ({} chars)",
                label,
                clipped.chars().count()
            );
            true
        }
        Err(e) => {
            warn!(target: "discord/draft", "{} failed: {}", label, e);
            false
        }
    }
}

/// Spawn the editor task. `throttle` is the minimum time between non-flush edits.
pub fn spawn_discord_draft_editor(
    ctx: Context,
    mut message: Message,
    throttle: Duration,
) -> DiscordDraftHandle {
    let (tx, mut rx) = mpsc::unbounded_channel::<Cmd>();

    tokio::spawn(async move {
        let mut last_sent = String::new();
        let mut next_allowed = Instant::now();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Cmd::Stop => break,
                Cmd::AbandonSilent { reply } => {
                    match message.delete(&ctx.http).await {
                        Ok(()) => info!(
                            target: "discord/draft",
                            "silent abandon: placeholder message deleted"
                        ),
                        Err(e) => warn!(
                            target: "discord/draft",
                            "silent abandon: delete placeholder failed: {}",
                            e
                        ),
                    }
                    let _ = reply.send(());
                    return;
                }
                Cmd::Flush { text, reply } => {
                    let ok = apply_edit(&ctx, &mut message, &text, "draft flush").await;
                    let _ = reply.send(ok);
                    break;
                }
                Cmd::Update(mut latest) => {
                    // Coalesce: while more commands are already queued, keep only the newest Update or handle Flush.
                    loop {
                        match rx.try_recv() {
                            Ok(Cmd::Update(t)) => latest = t,
                            Ok(Cmd::AbandonSilent { reply }) => {
                                if clamp_discord_content(&latest) != last_sent {
                                    let wait =
                                        next_allowed.saturating_duration_since(Instant::now());
                                    if !wait.is_zero() {
                                        tokio::time::sleep(wait).await;
                                    }
                                    let _ = apply_edit(&ctx, &mut message, &latest, "draft update")
                                        .await;
                                }
                                match message.delete(&ctx.http).await {
                                    Ok(()) => info!(
                                        target: "discord/draft",
                                        "silent abandon: placeholder message deleted (coalesced)"
                                    ),
                                    Err(e) => warn!(
                                        target: "discord/draft",
                                        "silent abandon: delete failed (coalesced): {}",
                                        e
                                    ),
                                }
                                let _ = reply.send(());
                                return;
                            }
                            Ok(Cmd::Flush { text, reply }) => {
                                // Skip intermediate update: flush is authoritative and must win.
                                let ok =
                                    apply_edit(&ctx, &mut message, &text, "draft flush").await;
                                let _ = reply.send(ok);
                                return;
                            }
                            Ok(Cmd::Stop) => return,
                            Err(mpsc::error::TryRecvError::Empty) => break,
                            Err(mpsc::error::TryRecvError::Disconnected) => return,
                        }
                    }

                    if clamp_discord_content(&latest) == last_sent {
                        continue;
                    }

                    let wait = next_allowed.saturating_duration_since(Instant::now());
                    if !wait.is_zero() {
                        tokio::time::sleep(wait).await;
                    }

                    if apply_edit(&ctx, &mut message, &latest, "draft update").await {
                        last_sent = clamp_discord_content(&latest);
                    }
                    next_allowed = Instant::now() + throttle;
                }
            }
        }
    });

    DiscordDraftHandle { tx }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_under_limit_unchanged() {
        let s = "hello";
        assert_eq!(clamp_discord_content(s), s);
    }

    #[test]
    fn clamp_truncates_with_ellipsis() {
        let s: String = "x".repeat(3000);
        let out = clamp_discord_content(&s);
        assert_eq!(out.chars().count(), DISCORD_CONTENT_MAX_CHARS);
        assert!(out.ends_with('…'));
    }
}
