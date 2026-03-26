//! Composable **execution** system prompt for the agent router (`answer_with_ollama_and_fetch`).
//!
//! Tool inventory text lives in [`crate::commands::agent_descriptions`]; this module focuses on
//! how soul, execution template, Discord hints, memory, live metrics, and the plan are ordered.
//!
//! ## Deterministic section order
//!
//! **Without skill overlay** (normal router soul from `soul.md`):
//!
//! 1. **Identity / soul** — personality + `You are mac-stats v…`
//! 2. **Discord user context** — who the user is (empty when not from Discord)
//! 3. **Tools + conversation rules** — `execution_prompt.md` after `{{AGENTS}}` substitution
//! 4. **Platform formatting** — Discord table/link rules when applicable
//! 5. **Model identity** — which Ollama model answers
//! 6. *Static prefix ends here* — everything above is stable for fixed soul, user, agents text,
//!    platform mode, and model; it must not embed live metrics, clocks, or session memory.
//! 7. **Memory / context** — global / channel / main-session memory blocks
//! 8. **Ori briefing** — optional vault `self/` + `ops/` excerpts (`MAC_STATS_ORI_*`, see `docs/ori-lifecycle.md`)
//! 9. **Ori prefetch** — optional `ori query similar` results
//! 10. **System metrics** — live CPU/RAM/disk snapshot for the model
//! 11. **Contextual reminders** — question-derived hints (screenshots, Redmine how-to, news format)
//! 12. **Plan** — optional `Your plan: …` from the planner
//!
//! **With skill overlay** (soul omitted per docs/022 §F4): Discord context → skill block →
//! execution template → platform → model → *(static end)* → metrics → reminders → plan.
//! Memory and Ori sections are not injected on this path (same as historical behaviour).
//!
//! Reordering relative to older builds places **stable** material first so Ollama (and tests) can
//! rely on a byte-stable prefix when only metrics/memory/plan/reminders change.

/// Fully assembled execution system message plus the byte length of the static prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionSystemPromptBuilt {
    pub content: String,
    /// UTF-8 byte length of [`Self::content`] prefix that must stay stable when only dynamic
    /// sections (memory, metrics, reminders, plan) change.
    pub static_prefix_len: usize,
}

// --- Section builders (each returns owned `String`; composed in [`build_execution_system_prompt`]). ---

fn section_identity_soul(router_soul: &str) -> String {
    router_soul.to_string()
}

fn section_discord_user_context(ctx: &str) -> String {
    ctx.to_string()
}

/// Tools + tool-usage rules + conversation guidelines from `execution_prompt.md` (with `{{AGENTS}}` already expanded).
fn section_tools_and_conversation_rules(execution_prompt: &str) -> String {
    execution_prompt.to_string()
}

fn section_platform_formatting(discord_platform_formatting: &str) -> String {
    discord_platform_formatting.to_string()
}

fn section_model_identity(model_identity: &str) -> String {
    model_identity.to_string()
}

fn section_memory_context(memory_block: &str) -> String {
    memory_block.to_string()
}

/// Live CPU/RAM/disk (and any clock lines embedded there) — expected to change between turns.
fn section_system_metrics_summary(metrics_for_system: &str) -> String {
    metrics_for_system.to_string()
}

fn section_contextual_reminders(
    discord_screenshot: &str,
    redmine_howto: &str,
    news_format: &str,
) -> String {
    format!("{discord_screenshot}{redmine_howto}{news_format}")
}

fn section_plan_block(plan_suffix: Option<&str>) -> String {
    plan_suffix
        .map(|p| format!("\n\nYour plan: {}", p))
        .unwrap_or_default()
}

fn section_skill_overlay(discord_user_context: &str, skill: &str) -> String {
    format!(
        "{}Additional instructions from skill:\n\n{}\n\n---\n\n",
        discord_user_context, skill
    )
}

fn join_static_without_skill(
    router_soul: &str,
    discord_user_context: &str,
    execution_prompt: &str,
    discord_platform_formatting: &str,
    model_identity: &str,
) -> String {
    format!(
        "{}{}{}{}{}",
        section_identity_soul(router_soul),
        section_discord_user_context(discord_user_context),
        section_tools_and_conversation_rules(execution_prompt),
        section_platform_formatting(discord_platform_formatting),
        section_model_identity(model_identity),
    )
}

fn join_dynamic_without_skill(
    memory_block: &str,
    ori_briefing_section: &str,
    ori_prefetch_section: &str,
    metrics_for_system: &str,
    contextual_reminders: &str,
    plan_block: &str,
) -> String {
    format!(
        "{}{}{}{}{}{}",
        section_memory_context(memory_block),
        ori_briefing_section,
        ori_prefetch_section,
        section_system_metrics_summary(metrics_for_system),
        contextual_reminders,
        plan_block,
    )
}

/// Build the execution-step system prompt from named logical sections.
///
/// `execution_prompt` must already include the substituted `{{AGENTS}}` tool list.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_execution_system_prompt(
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
) -> ExecutionSystemPromptBuilt {
    let plan_block = section_plan_block(plan_suffix);
    let contextual_reminders = section_contextual_reminders(
        discord_screenshot_reminder,
        redmine_howto_reminder,
        news_format_reminder,
    );

    match skill_content {
        Some(skill) => {
            let static_part = format!(
                "{}{}{}{}",
                section_skill_overlay(discord_user_context, skill),
                section_tools_and_conversation_rules(execution_prompt),
                section_platform_formatting(discord_platform_formatting),
                section_model_identity(model_identity),
            );
            let dynamic_part = join_dynamic_without_skill(
                "",
                "",
                "",
                metrics_for_system,
                &contextual_reminders,
                &plan_block,
            );
            let static_prefix_len = static_part.len();
            let content = format!("{static_part}{dynamic_part}");
            ExecutionSystemPromptBuilt {
                content,
                static_prefix_len,
            }
        }
        None => {
            let static_part = join_static_without_skill(
                router_soul,
                discord_user_context,
                execution_prompt,
                discord_platform_formatting,
                model_identity,
            );
            let dynamic_part = join_dynamic_without_skill(
                memory_block,
                ori_briefing_section,
                ori_prefetch_section,
                metrics_for_system,
                &contextual_reminders,
                &plan_block,
            );
            let static_prefix_len = static_part.len();
            let content = format!("{static_part}{dynamic_part}");
            ExecutionSystemPromptBuilt {
                content,
                static_prefix_len,
            }
        }
    }
}

/// Returns the UTF-8 prefix of `full` of length `prefix_len` (caller must ensure `prefix_len` is on
/// a char boundary — it is when taken from [`ExecutionSystemPromptBuilt::static_prefix_len`]).
#[cfg(test)]
fn static_prefix_bytes(full: &str, prefix_len: usize) -> &str {
    &full[..prefix_len.min(full.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_prefix_identical_when_only_metrics_change() {
        let m_a = "\n\nMETRICS_A";
        let m_b = "\n\nMETRICS_BBBBB";

        let built_a = build_execution_system_prompt(
            "SOUL\n\n",
            "\n\nMEMORY",
            "Discord ctx\n\n",
            None,
            "EXEC_PROMPT\n",
            m_a,
            "",
            "",
            "",
            "\n\n**Platform**",
            "\n\nYou are model X.",
            Some("do things"),
            "",
            "",
        );
        let built_b = build_execution_system_prompt(
            "SOUL\n\n",
            "\n\nMEMORY",
            "Discord ctx\n\n",
            None,
            "EXEC_PROMPT\n",
            m_b,
            "",
            "",
            "",
            "\n\n**Platform**",
            "\n\nYou are model X.",
            Some("do things"),
            "",
            "",
        );

        assert_ne!(built_a.content, built_b.content);
        assert_eq!(built_a.static_prefix_len, built_b.static_prefix_len);
        let p_a = static_prefix_bytes(&built_a.content, built_a.static_prefix_len);
        let p_b = static_prefix_bytes(&built_b.content, built_b.static_prefix_len);
        assert_eq!(p_a, p_b);
    }

    #[test]
    fn static_prefix_differs_when_soul_changes() {
        let params = |soul: &str| {
            build_execution_system_prompt(
                soul,
                "\n\nMEMORY",
                "",
                None,
                "EXEC\n",
                "\n\nMETRICS",
                "",
                "",
                "",
                "",
                "\n\nMODEL",
                None,
                "",
                "",
            )
        };
        let a = params("SOUL_A\n\n");
        let b = params("SOUL_B\n\n");
        let p_a = static_prefix_bytes(&a.content, a.static_prefix_len);
        let p_b = static_prefix_bytes(&b.content, b.static_prefix_len);
        assert_ne!(p_a, p_b);
    }

    #[test]
    fn intentional_churn_in_identity_section_breaks_stability() {
        // If someone appends a timestamp to the soul block, the "static" prefix should grow and
        // no longer match a build with a fixed soul (regression signal).
        let fixed = build_execution_system_prompt(
            "FIXED_SOUL\n\n",
            "",
            "",
            None,
            "EXEC\n",
            "\n\nM1",
            "",
            "",
            "",
            "",
            "\n\nMODEL",
            None,
            "",
            "",
        );
        let leaking = build_execution_system_prompt(
            "FIXED_SOUL\n\nTIME: now\n\n",
            "",
            "",
            None,
            "EXEC\n",
            "\n\nM1",
            "",
            "",
            "",
            "",
            "\n\nMODEL",
            None,
            "",
            "",
        );
        assert_ne!(
            static_prefix_bytes(&fixed.content, fixed.static_prefix_len),
            static_prefix_bytes(&leaking.content, leaking.static_prefix_len)
        );
    }

    #[test]
    fn skill_branch_omits_memory_static_prefix_coherent() {
        let a = build_execution_system_prompt(
            "",
            "SHOULD_NOT_APPEAR",
            "DC\n\n",
            Some("skill body"),
            "EXEC\n",
            "\n\nM1",
            "",
            "",
            "",
            "",
            "\n\nMODEL",
            None,
            "",
            "",
        );
        assert!(
            !a.content.contains("SHOULD_NOT_APPEAR"),
            "skill path must not inject memory block"
        );
        let b = build_execution_system_prompt(
            "",
            "SHOULD_NOT_APPEAR",
            "DC\n\n",
            Some("skill body"),
            "EXEC\n",
            "\n\nM2",
            "",
            "",
            "",
            "",
            "\n\nMODEL",
            None,
            "",
            "",
        );
        assert_eq!(
            static_prefix_bytes(&a.content, a.static_prefix_len),
            static_prefix_bytes(&b.content, b.static_prefix_len)
        );
    }
}
