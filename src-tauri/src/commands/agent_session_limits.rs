//! Agent router **session** limits: how wall-clock time, HTTP timeouts, and tool iteration caps stack.
//!
//! # Limit matrix (typical long Discord or scheduler agent run)
//!
//! Rows are independent knobs; **whichever threshold you hit first** ends or shapes the run.
//!
//! | Limit | What it caps | Typical default | Config / env (see `config::Config`) |
//! |-------|----------------|-----------------|--------------------------------------|
//! | **Ollama per-request HTTP** | One `POST /api/chat` (planning, follow-up, verify, etc.) | 300s | `ollamaChatTimeoutSecs`, `MAC_STATS_OLLAMA_CHAT_TIMEOUT_SECS` |
//! | **Agent router session wall-clock** | Entire `answer_with_ollama_and_fetch` (criteria + plan + tool loop + verification) | Discord 300s, in-app 180s, remote 300s | `agentRouterTurnTimeoutSecsDiscord` / `Ui` / `Remote`, `MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_SECS_*` (max 48h) |
//! | **Max tool iterations** (no per-agent override) | Tool-dispatch rounds when the entry agent is the default router | 15 per entry path | `agentRouterMaxToolIterationsDiscord` / `Ui` / `Remote`, `MAC_STATS_AGENT_ROUTER_MAX_TOOL_ITERATIONS_*` |
//! | **Per-agent max tools** | Overrides the row above for that agent | `agent.json` | `max_tool_iterations` |
//! | **Consecutive tool/LLM failures** | Stops tool loop early with partial text | 3 | `maxConsecutiveToolFailures`, `MAC_STATS_MAX_CONSECUTIVE_TOOL_FAILURES` |
//! | **Tool-loop repeat detection** | Optional warning / critical stop on repeated tool+arg | off (legacy 3-identical guard) | `toolLoopDetection`, `MAC_STATS_TOOL_LOOP_DETECTION_ENABLED` |
//! | **Browser CDP idle** | Closes shared browser after no use | 300s | `browserIdleTimeoutSecs` |
//! | **Scheduler task wall-clock** | One scheduled job execution | see scheduler | `schedulerTaskTimeoutSecs` |
//!
//! **Diagnostics:** Timeouts and caps aim for user-visible text that names the limit (per-request vs session wall-clock vs tool iteration cap) so logs and Discord replies are readable without Rust stack traces.

/// Default `max_tool_iterations` for the main router when `agent_override` is `None`, by entry path.
pub(crate) fn default_max_tool_iterations_for_router(
    discord_reply_channel_id: Option<u64>,
    from_remote: bool,
) -> u32 {
    if discord_reply_channel_id.is_some() {
        crate::config::Config::agent_router_max_tool_iterations_discord()
    } else if !from_remote {
        crate::config::Config::agent_router_max_tool_iterations_ui()
    } else {
        crate::config::Config::agent_router_max_tool_iterations_remote()
    }
}
