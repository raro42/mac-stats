//! Per-turn wall-clock coordination for the agent router: active request id per coordination
//! key (Discord channel or non-Discord slot), timeout cleanup that does not clobber a newer turn,
//! and output gating (see `TurnOutputGate`).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use crate::commands::discord_draft_stream::{clamp_discord_content, DiscordDraftHandle};
use crate::commands::partial_progress::PartialProgressCapture;
use crate::commands::verification::OllamaReply;
use crate::{mac_stats_info, mac_stats_warn};

static ACTIVE_TURN_BY_KEY: OnceLock<Mutex<HashMap<u64, String>>> = OnceLock::new();

fn active_map() -> &'static Mutex<HashMap<u64, String>> {
    ACTIVE_TURN_BY_KEY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Key for "this router invocation": one slot per Discord channel; single shared slot for all
/// non-Discord runs (CPU window, scheduler, task runner, heartbeat) so parallel local runs are
/// coordinated — avoid resetting the shared browser for a stale timeout if a newer run claimed the slot.
pub fn coordination_key(discord_reply_channel_id: Option<u64>) -> u64 {
    discord_reply_channel_id.unwrap_or(1)
}

pub fn resolve_turn_budget_secs(
    discord_reply_channel_id: Option<u64>,
    from_remote: bool,
    turn_timeout_secs: Option<u64>,
) -> u64 {
    if let Some(s) = turn_timeout_secs {
        return s.clamp(60, crate::config::AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS);
    }
    if discord_reply_channel_id.is_some() {
        crate::config::Config::agent_router_turn_timeout_secs_discord()
    } else if !from_remote {
        crate::config::Config::agent_router_turn_timeout_secs_ui()
    } else {
        crate::config::Config::agent_router_turn_timeout_secs_remote()
    }
}

pub fn register(coord_key: u64, request_id: &str) {
    let mut g = active_map().lock().unwrap_or_else(|e| e.into_inner());
    g.insert(coord_key, request_id.to_string());
    mac_stats_info!(
        "ollama/chat",
        "Agent router turn registered (coord_key={}, request_id={})",
        coord_key,
        request_id
    );
}

pub fn unregister_if_matches(coord_key: u64, request_id: &str) {
    let mut g = active_map().lock().unwrap_or_else(|e| e.into_inner());
    if g.get(&coord_key).map(|s| s.as_str()) == Some(request_id) {
        g.remove(&coord_key);
        mac_stats_info!(
            "ollama/chat",
            "Agent router turn cleared (coord_key={}, request_id={})",
            coord_key,
            request_id
        );
    }
}

fn peek_matches(coord_key: u64, request_id: &str) -> bool {
    let g = active_map().lock().unwrap_or_else(|e| e.into_inner());
    g.get(&coord_key).map(|s| s.as_str()) == Some(request_id)
}

/// Shared flag: while `true`, status lines and similar may be forwarded to the user.
pub type TurnOutputGate = Arc<AtomicBool>;

pub fn new_output_gate_open() -> TurnOutputGate {
    Arc::new(AtomicBool::new(true))
}

pub fn gate_allows_send(gate: &TurnOutputGate) -> bool {
    gate.load(Ordering::Acquire)
}

pub fn gate_close(gate: &TurnOutputGate) {
    gate.store(false, Ordering::Release);
}

/// After a wall-clock turn timeout: optional browser reset (only if this request still owns the
/// slot), bounded grace, then unregister. Returns the reply to send to the user.
pub async fn finalize_turn_timeout(
    coord_key: u64,
    request_id: &str,
    budget_secs: u64,
    discord_draft: Option<&DiscordDraftHandle>,
    partial: Option<&PartialProgressCapture>,
) -> Result<OllamaReply, String> {
    mac_stats_warn!(
        "ollama/chat",
        "Agent router [{}]: limit=agent_router_session_wall_clock — turn wall-clock timeout (budget {}s) — closing output gate and running cleanup",
        request_id,
        budget_secs
    );

    crate::commands::abort_cutoff::record_cutoff(
        coord_key,
        request_id.to_string(),
        chrono::Utc::now(),
    );

    let grace =
        Duration::from_secs(crate::config::Config::agent_router_turn_timeout_cleanup_grace_secs());
    let still_owner = peek_matches(coord_key, request_id);
    if still_owner {
        let coord_key_copy = coord_key;
        let rid = request_id.to_string();
        let cleanup = tokio::task::spawn_blocking(move || {
            if !peek_matches(coord_key_copy, &rid) {
                mac_stats_info!(
                    "ollama/chat",
                    "Agent router turn-timeout: browser cleanup skipped (coord_key={}, newer turn)",
                    coord_key_copy
                );
                return;
            }
            match crate::browser_agent::navigate_and_get_state("about:blank") {
                Ok(_) => mac_stats_info!(
                    "ollama/chat",
                    "Agent router turn-timeout: navigated active tab to about:blank"
                ),
                Err(e) => mac_stats_warn!(
                    "ollama/chat",
                    "Agent router turn-timeout: about:blank reset failed (non-fatal): {}",
                    e
                ),
            }
        });
        match tokio::time::timeout(grace, async { cleanup.await }).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => mac_stats_warn!(
                "ollama/chat",
                "Agent router turn-timeout: cleanup task join error: {}",
                e
            ),
            Err(_) => mac_stats_warn!(
                "ollama/chat",
                "Agent router turn-timeout: cleanup grace {}s elapsed (detached)",
                grace.as_secs()
            ),
        }
    } else {
        mac_stats_info!(
            "ollama/chat",
            "Agent router turn-timeout: cleanup skipped — coord_key={} not owned by request_id={}",
            coord_key,
            request_id
        );
    }

    unregister_if_matches(coord_key, request_id);

    let mut text = format!(
        "**Limit: agent router session wall-clock** — The full agent run exceeded its wall-clock budget (**{}s**). \
         This is separate from a single Ollama HTTP call timeout (`ollamaChatTimeoutSecs`).\n\n\
         The run was stopped so the channel or UI does not hang. For long unattended jobs you may raise \
         `agentRouterTurnTimeoutSecsDiscord`, `agentRouterTurnTimeoutSecsUi`, or `agentRouterTurnTimeoutSecsRemote` in `~/.mac-stats/config.json` (up to 48h). Defaults stay short for menu-bar responsiveness.\n\n\
         If this keeps happening, try a narrower question, a faster model, or widen the matching `agentRouterTurnTimeoutSecs*` value.",
        budget_secs
    );
    if let Some(p) = partial {
        if let Some(extra) = p.format_user_summary() {
            text.push_str("\n\n");
            text.push_str(&extra);
        }
    }

    if let Some(d) = discord_draft {
        d.flush(&clamp_discord_content(&text)).await;
    }

    Ok(OllamaReply {
        text,
        attachment_paths: vec![],
        ..Default::default()
    })
}
