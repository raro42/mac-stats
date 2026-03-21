//! System prompt assembly for the execution step.
//!
//! Consolidates the duplicated prompt building logic that was in `ollama.rs`
//! (direct-tool path and LLM-execution path).

/// Assembled execution system content ready for the Ollama messages array.
pub(crate) struct ExecutionSystemPrompt {
    pub content: String,
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
) -> ExecutionSystemPrompt {
    let plan_block = plan_suffix
        .map(|p| format!("\n\nYour plan: {}", p))
        .unwrap_or_default();

    let content = match skill_content {
        Some(skill) => format!(
            "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}{}{}{}{}{}{}{}",
            discord_user_context,
            skill,
            execution_prompt,
            metrics_for_system,
            discord_screenshot_reminder,
            redmine_howto_reminder,
            news_format_reminder,
            discord_platform_formatting,
            plan_block,
            model_identity,
        ),
        None => format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            router_soul,
            memory_block,
            discord_user_context,
            execution_prompt,
            metrics_for_system,
            discord_screenshot_reminder,
            redmine_howto_reminder,
            news_format_reminder,
            discord_platform_formatting,
            plan_block,
            model_identity,
        ),
    };

    ExecutionSystemPrompt { content }
}
