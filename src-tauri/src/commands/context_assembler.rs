//! Shared **context assembly** for Ollama chat: cap history, token-budget trim, and stack order.
//!
//! CPU-window chat and the agent router (`answer_with_ollama_and_fetch`) both build
//! `system → history → current user` stacks; this module centralizes trimming and ordering so
//! behaviour stays aligned (see task FEAT context-assembly-trait).

use crate::commands::content_reduction::CHARS_PER_TOKEN;
use crate::commands::session_history::{build_execution_message_stack, cap_tail_chronological};
use crate::ollama::ChatMessage;
use crate::{mac_stats_debug, mac_stats_info};

/// Tokens reserved below configured `num_ctx` / model context (reply + API overhead).
pub(crate) const CONTEXT_ASSEMBLER_SAFETY_TOKENS: u32 = 512;

/// Shared prompt fragments (metrics, non-agent tool preamble) so CPU chat and the router stay in sync.
pub mod fragments {
    use crate::commands::ollama_memory::load_soul_content;
    use crate::metrics::format_metrics_for_ai_context;

    /// Tool instructions appended to soul for non-agent chat (code execution + FETCH_URL).
    pub const NON_AGENT_TOOL_INSTRUCTIONS: &str = "\n\nYou are a general purpose AI. If you are asked for actual data like day or weather information, or flight information or stock information. Then we need to compile that information using specially crafted clients for doing so. You will put \"[variable-name]\" into the answer to signal that we need to go another step and ask an agent to fulfil the answer.\n\nWhenever asked with \"[variable-name]\", you must provide a javascript snippet to be executed in the browser console to retrieve that information. Mark the answer to be executed as javascript. Do not put any other words around it. Do not insert formatting. Only return the code to be executed. This is needed for the next AI to understand and execute the same. When answering, use the role: code-assistant in the response. When you return executable code:\n- Start the response with: ROLE=code-assistant\n- On the next line, output ONLY executable JavaScript\n- Do not add explanations or formatting\n\nFor web pages: To fetch a page and use its content (e.g. \"navigate to X and get Y\"), reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will fetch the page and give you the text; then answer the user based on that.";

    /// Live CPU/RAM/disk block for prompts (same source for all surfaces).
    #[inline]
    pub fn live_metrics_for_prompt() -> String {
        format_metrics_for_ai_context()
    }

    /// Metrics block formatted for the agent-router execution system prompt (leading newlines).
    #[inline]
    pub fn live_metrics_execution_system_section() -> String {
        format!("\n\n{}", live_metrics_for_prompt())
    }

    /// Default CPU / non-agent system prompt: soul file + shared tool instructions.
    pub fn default_non_agent_system_prompt_text() -> String {
        format!(
            "{}{}{}",
            load_soul_content(),
            NON_AGENT_TOOL_INSTRUCTIONS,
            crate::commands::directive_tags::NON_AGENT_DIRECTIVE_APPEND
        )
    }

    /// User turn text for CPU chat: metrics snapshot + question (shared wording with prior behaviour).
    pub fn cpu_window_user_turn_with_metrics(question: &str) -> String {
        format!(
            "{}\n\nUser question: {}",
            live_metrics_for_prompt(),
            question
        )
    }
}

/// Effective token budget for history + system + current user (model context minus safety margin).
#[inline]
pub(crate) fn context_token_budget(context_size_tokens: u32) -> usize {
    context_size_tokens
        .saturating_sub(CONTEXT_ASSEMBLER_SAFETY_TOKENS)
        .max(256) as usize
}

#[inline]
fn estimate_message_tokens(m: &ChatMessage) -> usize {
    let mut n = m.content.chars().count() / CHARS_PER_TOKEN;
    if let Some(imgs) = m.images.as_ref() {
        for b64 in imgs {
            n = n.saturating_add(b64.len() / CHARS_PER_TOKEN);
        }
    }
    n.saturating_add(4)
}

fn estimate_system_and_user_tokens(system_context: &str, current_user: &ChatMessage) -> usize {
    system_context.chars().count() / CHARS_PER_TOKEN + estimate_message_tokens(current_user)
}

fn total_history_tokens(history: &[ChatMessage]) -> usize {
    history.iter().map(estimate_message_tokens).sum()
}

/// Drop oldest chat messages until the slice fits `max_tokens` (or empty).
pub(crate) fn trim_history_oldest_first_to_token_budget(
    mut history: Vec<ChatMessage>,
    max_tokens: usize,
) -> Vec<ChatMessage> {
    while !history.is_empty() && total_history_tokens(&history) > max_tokens {
        history.remove(0);
    }
    history
}

/// Resolve token budget from the configured Ollama model (CPU chat when no per-request override).
pub(crate) async fn resolve_default_chat_context_token_budget() -> usize {
    use crate::commands::ollama_config::{
        get_ollama_client, read_ollama_api_key_from_env_or_config,
        read_ollama_fast_model_from_env_or_config,
    };
    use crate::security::get_credential;

    let (endpoint, effective, api_key): (String, String, Option<String>) = {
        let guard = match get_ollama_client().lock() {
            Ok(g) => g,
            Err(_) => {
                let b =
                    context_token_budget(crate::ollama::ModelInfo::default().context_size_tokens);
                mac_stats_debug!(
                    "ollama/chat",
                    token_budget = b,
                    reason = "ollama_client_lock_poisoned",
                    "context_assembler: CPU chat token budget fallback (default context)"
                );
                return b;
            }
        };
        let Some(client) = guard.as_ref() else {
            let b = context_token_budget(crate::ollama::ModelInfo::default().context_size_tokens);
            mac_stats_debug!(
                "ollama/chat",
                token_budget = b,
                reason = "no_ollama_client",
                "context_assembler: CPU chat token budget fallback (default context)"
            );
            return b;
        };
        let effective = read_ollama_fast_model_from_env_or_config()
            .unwrap_or_else(|| client.config.model.clone());
        let api_key = client
            .config
            .api_key
            .as_ref()
            .and_then(|acc| get_credential(acc).ok().flatten())
            .or_else(read_ollama_api_key_from_env_or_config);
        (client.config.endpoint.clone(), effective, api_key)
    };

    let (info, ctx_src) =
        crate::ollama::resolve_model_context_budget(&endpoint, &effective, api_key.as_deref())
            .await;
    let b = context_token_budget(info.context_size_tokens);
    mac_stats_info!(
        "ollama/chat",
        model = %effective,
        context_size = info.context_size_tokens,
        context_source = ctx_src.as_str(),
        token_budget = b,
        "context_assembler: resolved CPU chat token budget (effective context)"
    );
    b
}

/// Unifies how mac-stats caps and orders messages for Ollama (OpenClaw-style context engine hook).
pub trait ContextAssembler {
    /// Cap tail message count (same cap as session history contract).
    fn compact(&self, history: Vec<ChatMessage>) -> Vec<ChatMessage>;

    /// `system` + trimmed `history` + `current_user`, oldest history dropped first to respect `token_budget_tokens`.
    fn assemble(
        &self,
        history: &[ChatMessage],
        system_context: &str,
        current_user: ChatMessage,
        token_budget_tokens: usize,
    ) -> Vec<ChatMessage>;
}

/// CPU-window chat: metrics-augmented user turns, streaming-friendly history filter lives in the caller.
pub struct FrontendContextAssembler;

/// Agent router / Discord / scheduler: planner + execution stacks share this assembler.
pub struct AgentContextAssembler;

impl ContextAssembler for FrontendContextAssembler {
    fn compact(&self, history: Vec<ChatMessage>) -> Vec<ChatMessage> {
        cap_tail_chronological(
            history,
            crate::commands::session_history::CONVERSATION_HISTORY_CAP,
        )
    }

    fn assemble(
        &self,
        history: &[ChatMessage],
        system_context: &str,
        current_user: ChatMessage,
        token_budget_tokens: usize,
    ) -> Vec<ChatMessage> {
        assemble_impl(
            "frontend",
            history,
            system_context,
            current_user,
            token_budget_tokens,
        )
    }
}

impl ContextAssembler for AgentContextAssembler {
    fn compact(&self, history: Vec<ChatMessage>) -> Vec<ChatMessage> {
        cap_tail_chronological(
            history,
            crate::commands::session_history::CONVERSATION_HISTORY_CAP,
        )
    }

    fn assemble(
        &self,
        history: &[ChatMessage],
        system_context: &str,
        current_user: ChatMessage,
        token_budget_tokens: usize,
    ) -> Vec<ChatMessage> {
        assemble_impl(
            "agent_router",
            history,
            system_context,
            current_user,
            token_budget_tokens,
        )
    }
}

fn assemble_impl(
    surface: &'static str,
    history: &[ChatMessage],
    system_context: &str,
    current_user: ChatMessage,
    token_budget_tokens: usize,
) -> Vec<ChatMessage> {
    let headroom = estimate_system_and_user_tokens(system_context, &current_user);
    let reserve = headroom.saturating_add(64);
    let max_hist = token_budget_tokens.saturating_sub(reserve);
    let before = history.len();
    let trimmed = trim_history_oldest_first_to_token_budget(history.to_vec(), max_hist);
    if trimmed.len() < before {
        mac_stats_info!(
            "ollama/chat",
            surface,
            before,
            after = trimmed.len(),
            token_budget = token_budget_tokens,
            "context_assembler: trimmed history to token budget (oldest first; see prior log for model context_size and context_source)"
        );
    }
    build_execution_message_stack(
        ChatMessage {
            role: "system".to_string(),
            content: system_context.to_string(),
            images: None,
        },
        &trimmed,
        current_user,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::session_history::CONVERSATION_HISTORY_CAP;

    #[test]
    fn context_token_budget_subtracts_safety() {
        assert_eq!(context_token_budget(4096), 3584);
        assert_eq!(context_token_budget(8192), 7680);
        assert_eq!(context_token_budget(512), 256);
    }

    #[test]
    fn trim_history_drops_oldest_until_under_budget() {
        let mk = |i: usize| ChatMessage {
            role: "user".to_string(),
            content: "x".repeat(i * CHARS_PER_TOKEN * 10),
            images: None,
        };
        let h = vec![mk(1), mk(2), mk(3)];
        let max_tok = estimate_message_tokens(&h[1]) + estimate_message_tokens(&h[2]);
        let out = trim_history_oldest_first_to_token_budget(h, max_tok);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn frontend_compact_matches_tail_cap() {
        let mut v = Vec::new();
        for i in 0..(CONVERSATION_HISTORY_CAP + 3) {
            v.push(ChatMessage {
                role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                content: format!("m{i}"),
                images: None,
            });
        }
        let out = ContextAssembler::compact(&FrontendContextAssembler, v);
        assert_eq!(out.len(), CONVERSATION_HISTORY_CAP);
    }

    #[test]
    fn assemble_preserves_system_history_user_order() {
        let hist = vec![ChatMessage {
            role: "user".to_string(),
            content: "old".to_string(),
            images: None,
        }];
        let user = ChatMessage {
            role: "user".to_string(),
            content: "new".to_string(),
            images: None,
        };
        let out =
            ContextAssembler::assemble(&AgentContextAssembler, &hist, "SYS", user.clone(), 100_000);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].role, "system");
        assert_eq!(out[0].content, "SYS");
        assert_eq!(out[1].content, "old");
        assert_eq!(out[2].content, "new");
    }
}
