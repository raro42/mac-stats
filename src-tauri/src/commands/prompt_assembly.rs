//! System prompt assembly for the execution step.
//!
//! Composable section order and static-prefix invariants live in [`crate::prompts`].

use crate::prompts::build_execution_system_prompt;

/// Assembled execution system content ready for the Ollama messages array.
pub(crate) struct ExecutionSystemPrompt {
    pub content: String,
    /// Byte length of the stable prefix (soul/tools/platform/model before memory/metrics/plan).
    #[allow(dead_code)] // Exposed for diagnostics; logged once when built in this module.
    pub static_prefix_len: usize,
}

/// Build the execution system prompt content. Called by both the direct-tool fast path
/// (no plan suffix) and the normal LLM execution path (with plan suffix).
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_execution_system_content(
    router_soul: &str,
    memory_block: &str,
    discord_user_context: &str,
    skill_content: Option<&str>,
    execution_prompt: &str,
    metrics_for_system: &str,
    discord_screenshot_reminder: &str,
    redmine_howto_reminder: &str,
    news_format_reminder: &str,
    discord_platform_formatting: &str,
    model_identity: &str,
    plan_suffix: Option<&str>,
    ori_briefing_section: &str,
    ori_prefetch_section: &str,
) -> ExecutionSystemPrompt {
    let built = build_execution_system_prompt(
        router_soul,
        memory_block,
        discord_user_context,
        skill_content,
        execution_prompt,
        metrics_for_system,
        discord_screenshot_reminder,
        redmine_howto_reminder,
        news_format_reminder,
        discord_platform_formatting,
        model_identity,
        plan_suffix,
        ori_briefing_section,
        ori_prefetch_section,
    );
    crate::mac_stats_debug!(
        "ollama/chat",
        "Execution system prompt assembled: {} bytes total, {}-byte static prefix (composable sections)",
        built.content.len(),
        built.static_prefix_len
    );
    ExecutionSystemPrompt {
        content: built.content,
        static_prefix_len: built.static_prefix_len,
    }
}

/// Append optional heartbeat instructions to the execution system prompt (periodic check runs).
pub(crate) fn append_heartbeat_section(
    prompt: &mut ExecutionSystemPrompt,
    extra: Option<&str>,
) {
    if let Some(h) = extra.map(str::trim).filter(|s| !s.is_empty()) {
        prompt.content.push_str("\n\n## Heartbeat\n\n");
        prompt.content.push_str(h);
    }
}
