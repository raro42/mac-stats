//! Content reduction (truncation/summarization) and skill/JS execution helpers.
//!
//! Extracted from `ollama.rs` to keep the orchestrator focused.

use std::io::Write;
use std::process::Command;

use crate::commands::ollama_chat::send_ollama_chat_messages;

/// Heuristic: chars to tokens (conservative).
pub(crate) const CHARS_PER_TOKEN: usize = 4;

/// Reserve tokens for model reply and wrapper text.
const RESERVE_TOKENS: u32 = 512;

/// When over limit by at most 1/this fraction, truncate only (no summarization) to avoid extra Ollama call.
const TRUNCATE_ONLY_THRESHOLD_DENOM: u32 = 4;

/// Truncate at last newline or space before max_chars so we don't cut mid-word. O(max_chars).
pub(crate) fn truncate_at_boundary(body: &str, max_chars: usize) -> String {
    let mut last_break = max_chars;
    let mut broke_early = false;
    for (i, c) in body.chars().enumerate() {
        if i >= max_chars {
            broke_early = true;
            break;
        }
        if c == '\n' || c == ' ' {
            last_break = i + 1;
        }
    }
    if !broke_early {
        return body.to_string();
    }
    body.chars().take(last_break).collect()
}

/// Reduce fetched page content to fit the model context: summarize via Ollama if needed, else truncate.
/// Uses byte-length heuristic for fast path and "slightly over" path to avoid full char count; only
/// when summarization is needed do we count chars for logging.
pub(crate) async fn reduce_fetched_content_to_fit(
    body: &str,
    context_size_tokens: u32,
    estimated_used_tokens: u32,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<String, String> {
    use tracing::info;

    let max_tokens_for_body = context_size_tokens
        .saturating_sub(RESERVE_TOKENS)
        .saturating_sub(estimated_used_tokens);
    let max_chars = (max_tokens_for_body as usize).saturating_mul(CHARS_PER_TOKEN);

    // Fast path: cheap byte heuristic (len/4 >= char_count/4 for UTF-8). Avoids char count when body fits.
    let body_tokens_upper = body.len() / CHARS_PER_TOKEN;
    if body_tokens_upper <= max_tokens_for_body as usize {
        return Ok(body.to_string());
    }

    // Slightly over: within 25% of limit → truncate only, no summarization (saves one Ollama round-trip).
    let threshold = max_tokens_for_body + (max_tokens_for_body / TRUNCATE_ONLY_THRESHOLD_DENOM);
    if body_tokens_upper <= threshold as usize {
        let truncated = truncate_at_boundary(body, max_chars);
        return Ok(format!(
            "{} (content truncated due to context limit)",
            truncated.trim_end()
        ));
    }

    // Way over: summarization path. Compute exact token estimate only for logging.
    let body_tokens_est = body.chars().count() / CHARS_PER_TOKEN;
    info!(
        "Agent router: page content too large (est. {} tokens), max {} tokens; reducing",
        body_tokens_est, max_tokens_for_body
    );

    let body_truncated_for_request = truncate_at_boundary(body, max_chars);
    let summary_tokens = (max_tokens_for_body / 2).max(256);
    let summarization_messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Summarize the following web page content in under {} tokens, keeping the most relevant information for answering questions. Output only the summary, no preamble.",
                summary_tokens
            ),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: body_truncated_for_request,
            images: None,
        },
    ];

    match send_ollama_chat_messages(summarization_messages, model_override, options_override).await
    {
        Ok(resp) => {
            let summary = resp.message.content.trim().to_string();
            if summary.is_empty() {
                let fallback = truncate_at_boundary(body, max_chars);
                Ok(format!(
                    "{} (content truncated due to context limit)",
                    fallback.trim_end()
                ))
            } else {
                Ok(summary)
            }
        }
        Err(e) => {
            info!("Agent router: summarization failed ({}), truncating", e);
            let fallback = truncate_at_boundary(body, max_chars);
            Ok(format!(
                "{} (content truncated due to context limit)",
                fallback.trim_end()
            ))
        }
    }
}

/// True if `needle` appears in `haystack` with no ASCII alnum / `_` immediately before or after.
/// Avoids treating `context_length_exceeded` as present inside `old_context_length_exceeded`.
fn contains_bounded_token(haystack: &str, needle: &str) -> bool {
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    for (i, _) in haystack.match_indices(needle) {
        let before_ok = haystack[..i]
            .chars()
            .last()
            .is_none_or(|c| !ident_continue(c));
        let after_ok = haystack[i + needle.len()..]
            .chars()
            .next()
            .is_none_or(|c| !ident_continue(c));
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Check whether an Ollama error string indicates a context-window overflow.
pub(crate) fn is_context_overflow_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("context overflow")
        || lower.contains("prompt too long")
        || lower.contains("prompt is too long")
        || lower.contains("context length exceeded")
        || lower.contains("maximum context length")
        || lower.contains("exceeds the model's context window")
        || lower.contains("exceeds the model context")
        || lower.contains("exceeded the model context")
        || lower.contains("exceed the model context")
        || lower.contains("model context exceeded")
        || lower.contains("model context limit exceeded")
        || lower.contains("exceeds the model's maximum context")
        || lower.contains("exceeded the model's maximum context")
        || lower.contains("exceed the model's maximum context")
        || lower.contains("exceeds the context window")
        || lower.contains("context window exceeded")
        || lower.contains("exceeded the context limit")
        || lower.contains("exceeds the context limit")
        || lower.contains("exceed the context limit")
        || lower.contains("requested more tokens than")
        || lower.contains("total prompt tokens exceed")
        || lower.contains("fit in the context")
        || lower.contains("larger than the context")
        || lower.contains("outside the context window")
        || lower.contains("outside of the context window")
        || lower.contains("beyond the context window")
        || lower.contains("over the context window")
        || lower.contains("ran out of context")
        || lower.contains("running out of context")
        || (lower.contains("context budget")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("full")))
        || (lower.contains("conversation")
            && lower.contains("too long")
            && lower.contains("context"))
        || lower.contains("context size exceeded")
        || lower.contains("exceeded context size")
        || lower.contains("prompt has more tokens than")
        || lower.contains("exceeds available context")
        || lower.contains("context limit exceeded")
        || lower.contains("exceeds context length")
        || lower.contains("requested tokens exceed")
        || (lower.contains("too long") && lower.contains("context"))
        || (lower.contains("too large") && lower.contains("context"))
        || (lower.contains("cannot fit") && lower.contains("context"))
        || (lower.contains("does not fit") && lower.contains("context"))
        || (lower.contains("doesn't fit") && lower.contains("context"))
        || (lower.contains("unable to fit") && lower.contains("context"))
        || (lower.contains("won't fit") && lower.contains("context"))
        || (lower.contains("longer than") && lower.contains("context"))
        || lower.contains("exceeds maximum context")
        || lower.contains("maximum context exceeded")
        || lower.contains("insufficient context")
        || lower.contains("prompt exceeds the context")
        || (lower.contains("greater than") && lower.contains("context"))
        || (contains_bounded_token(&lower, "n_ctx")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too small")))
        || (contains_bounded_token(&lower, "num_ctx")
            && (lower.contains("exceed")
                || lower.contains("larger")
                || lower.contains("greater")
                || lower.contains("longer")))
        || lower.contains("exceeds maximum sequence length")
        || lower.contains("maximum sequence length exceeded")
        || lower.contains("sequence length exceeds")
        || lower.contains("input sequence is too long")
        || (lower.contains("total tokens exceed")
            && (lower.contains("context")
                || lower.contains("model")
                || lower.contains("maximum")
                || lower.contains("sequence")
                || lower.contains("n_ctx")))
        || lower.contains("prompt length exceeds")
        || (lower.contains("too many tokens") && lower.contains("context"))
        || lower.contains("max context exceeded")
        || lower.contains("exceeds max context")
        || lower.contains("beyond the context")
        || lower.contains("not enough context")
        || (lower.contains("context buffer")
            && (lower.contains("overflow")
                || lower.contains("exceed")
                || lower.contains("full")))
        || ((lower.contains("prompt tokens exceed") || lower.contains("input tokens exceed"))
            && (lower.contains("context")
                || lower.contains("model")
                || lower.contains("maximum")
                || lower.contains("sequence")
                || lower.contains("n_ctx")
                || lower.contains("window")))
        || lower.contains("exceeds the configured context")
        || (lower.contains("configured context")
            && (lower.contains("overflow")
                || lower.contains("exceed")
                || lower.contains("full")))
        || (lower.contains("would exceed") && lower.contains("context"))
        || lower.contains("past the context")
        || lower.contains("reached the context limit")
        || lower.contains("hit the context limit")
        || lower.contains("over the context limit")
        || (lower.contains("truncated")
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("max context")))
        || lower.contains("context exhausted")
        || (lower.contains("context")
            && (lower.contains("fully exhausted") || lower.contains("completely exhausted")))
        || lower.contains("insufficient remaining context")
        || ((lower.contains("kv cache") || lower.contains("kv-cache"))
            && lower.contains("is full")
            && lower.contains("context"))
        || lower.contains("exceeds the context size")
        || lower.contains("context size exceeds")
        || lower.contains("exceeded the context size")
        || (lower.contains("context window") && lower.contains("too small"))
        || (lower.contains("context capacity")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("full")))
        || (lower.contains("allocated context")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("full")))
        || (lower.contains("prefill")
            && lower.contains("context")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")))
        || (contains_bounded_token(&lower, "max_context")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")))
        || (contains_bounded_token(&lower, "context_length")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")))
        // camelCase JSON / gateway errors (to_lowercase → "maxcontext" / "contextlength")
        || (contains_bounded_token(&lower, "maxcontext")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")))
        || (contains_bounded_token(&lower, "contextlength")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")))
        || (contains_bounded_token(&lower, "n_ctx_per_seq")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")
                || lower.contains("too small")))
        // snake_case JSON / config (distinct from prose "context window")
        || (contains_bounded_token(&lower, "context_window")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")
                || lower.contains("too small")))
        || (contains_bounded_token(&lower, "context_limit")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")))
        || (contains_bounded_token(&lower, "contextlimit")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")))
        || lower.contains("exceed the maximum context")
        || lower.contains("exceeded maximum context")
        || lower.contains("maximum context is exceeded")
        || lower.contains("context token limit exceeded")
        || lower.contains("exceeds the context token limit")
        || lower.contains("exceeded the context token limit")
        // Word order / filler between "exceed" and "maximum … context" (e.g. "model", "allowed")
        || (lower.contains("maximum context")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")))
        || (lower.contains("max context")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")))
        // "allowed" between max/imum and context breaks contiguous `maximum context` / `max context` substrings.
        || (lower.contains("maximum allowed context")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")))
        || (lower.contains("max allowed context")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")))
        || (lower.contains("context length limit")
            && (lower.contains("exceed")
                || lower.contains("overflow")
                || lower.contains("too long")
                || lower.contains("too large")
                || lower.contains("larger")
                || lower.contains("greater")))
        // OpenAI-style / JSON `code` values and similar (bounded so `old_context_length_exceeded` etc. do not match)
        || contains_bounded_token(&lower, "context_budget_exceeded")
        || contains_bounded_token(&lower, "context_length_exceeded")
        || contains_bounded_token(&lower, "max_context_length_exceeded")
        || contains_bounded_token(&lower, "context_window_exceeded")
        || contains_bounded_token(&lower, "max_context_exceeded")
        // Demonstrative phrasing (distinct from `the model's` already covered above)
        || lower.contains("exceeds this model's context")
        || lower.contains("exceeded this model's context")
        || lower.contains("exceed this model's context")
        // Chat-completions style: "messages exceed …" without "total tokens" wording.
        // Require explicit context-slot phrases (not bare `model`) so lines like
        // "status messages exceed limits (no model context)" do not match.
        // Possessive `model's context` is a slot (e.g. "too long for this model's context") and
        // does not substring-match bare `model context` in "(no model context configured)".
        || ((lower.contains("messages exceed") || lower.contains("messages exceeded"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Singular "message exceed(s/ed)" (parallel to plural `messages exceed`, FEAT-D298).
        || ((lower.contains("message exceeds") || lower.contains("message exceeded"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural "inputs exceed …" (present / past). Wording like `inputs exceed the context window`
        // does not contain the substring `exceeds the context window` (bare `exceed` before the slot).
        // Same context-slot guard as `messages exceed` (FEAT-D295).
        || ((lower.contains("inputs exceed") || lower.contains("inputs exceeded"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Singular "input exceed(s/ed)" (parallel to plural `inputs exceed`, FEAT-D300).
        // `input exceed` matches present/past via `exceed` prefix of `exceeds` / `exceeded`.
        // Does not substring-match `inputs exceed` (letter `s` between `input` and `exceed`).
        || (lower.contains("input exceed")
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // "message/input(s) … too long" (distinct from `prompt too long` already handled above).
        // Same context-slot guard as `messages exceed` (FEAT-D295) so incidental `model context`
        // copy does not match non-slot errors. Plural `inputs are/were` (FEAT-D302) parallels
        // `messages are/were` and does not substring-match singular `input is/was`.
        || ((lower.contains("message is too long")
            || lower.contains("messages are too long")
            || lower.contains("message was too long")
            || lower.contains("messages were too long")
            || lower.contains("input is too long")
            || lower.contains("input was too long")
            || lower.contains("inputs are too long")
            || lower.contains("inputs were too long"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
}

/// Check whether an Ollama error indicates message role/ordering conflict.
fn is_role_ordering_error(lower: &str) -> bool {
    (lower.contains("role") && (lower.contains("alternate") || lower.contains("ordering")))
        || lower.contains("incorrect role")
        || lower.contains("roles must alternate")
        || lower.contains("expected role")
        || (lower.contains("invalid") && lower.contains("role"))
}

/// Check whether an Ollama error indicates corrupted session / missing tool input.
fn is_corrupted_session_error(lower: &str) -> bool {
    (lower.contains("tool") && lower.contains("missing"))
        || lower.contains("invalid message")
        || lower.contains("malformed")
        || (lower.contains("tool_calls") && lower.contains("expected"))
}

/// Rewrite a raw Ollama/pipeline error into a short, user-friendly message.
///
/// Maps known error categories to actionable text that suggests starting a
/// new topic (matching the wording users already know from session reset).
/// Returns `None` when the error does not match any known pattern, so callers
/// can fall back to their existing formatting.
pub(crate) fn sanitize_ollama_error_for_user(raw: &str) -> Option<String> {
    let lower = raw.to_lowercase();

    let friendly = if is_context_overflow_error(raw) {
        Some(
            "The conversation got too long for the model's context window. \
             Try starting a new topic or using a model with a larger context."
                .to_string(),
        )
    } else if is_role_ordering_error(&lower) {
        Some(
            "Message ordering conflict — please try again. \
             If this keeps happening, start a new topic to reset the conversation."
                .to_string(),
        )
    } else if is_corrupted_session_error(&lower) {
        Some(
            "The conversation history looks corrupted. \
             Start a new topic to begin a fresh session."
                .to_string(),
        )
    } else {
        None
    };

    if friendly.is_some() {
        tracing::debug!("Sanitized Ollama error for user — raw: {}", raw);
    }

    friendly
}

/// Truncate oversized tool-result messages in the conversation to `max_chars_per_result`.
///
/// Only truncates assistant/user/system messages whose content exceeds `max_chars_per_result`
/// and that look like tool results (heuristic: not the very first system prompt, and not
/// messages that are the user's original question).
///
/// Returns the number of messages that were truncated.
pub(crate) fn truncate_oversized_tool_results(
    messages: &mut [crate::ollama::ChatMessage],
    max_chars_per_result: usize,
) -> usize {
    let mut truncated_count = 0usize;
    for (i, msg) in messages.iter_mut().enumerate() {
        // Skip the first message (system prompt) — it contains the agent instructions.
        if i == 0 && msg.role == "system" {
            continue;
        }
        let char_count = msg.content.chars().count();
        if char_count <= max_chars_per_result {
            continue;
        }
        let truncated_body = truncate_at_boundary(&msg.content, max_chars_per_result);
        msg.content = format!(
            "{}\n\n[truncated from {} to {} chars due to context limit]",
            truncated_body.trim_end(),
            char_count,
            max_chars_per_result
        );
        truncated_count += 1;
    }
    truncated_count
}

/// Run a single Ollama request in a new session (no conversation history). Used for SKILL agent.
/// System message = skill content, user message = task. Returns the assistant reply or error string.
pub(crate) async fn run_skill_ollama_session(
    skill_content: &str,
    user_message: &str,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<String, String> {
    use tracing::info;
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: skill_content.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            images: None,
        },
    ];
    info!(
        "Agent router: SKILL session request (user message {} chars)",
        user_message.chars().count()
    );
    let response = send_ollama_chat_messages(messages, model_override, options_override).await?;
    Ok(response.message.content.trim().to_string())
}

/// Run JavaScript via Node.js (if available). Used for RUN_JS in Discord/agent context.
/// Writes code to a temp file and runs `node -e "..."` to eval and print the result.
///
/// **Security:** RUN_JS is agent-triggered and runs with process privileges. Agent or prompt
/// compromise can lead to arbitrary code execution. Treat agent output as untrusted code.
pub(crate) fn run_js_via_node(code: &str) -> Result<String, String> {
    let tmp_dir = crate::config::Config::tmp_js_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let _ = std::fs::create_dir_all(&tmp_dir);
    let path = tmp_dir.join(format!("mac_stats_js_{}_{}.js", std::process::id(), stamp));
    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid temp path".to_string())?;

    let mut f = std::fs::File::create(&path).map_err(|e| format!("Create temp file: {}", e))?;
    f.write_all(code.as_bytes())
        .map_err(|e| format!("Write temp file: {}", e))?;
    f.flush().map_err(|e| format!("Flush: {}", e))?;
    drop(f);

    // Node -e script: read file, eval code, print result (no user code in -e, so no escaping).
    let eval_script = r#"const fs=require('fs');const p=process.argv[1];const c=fs.readFileSync(p,'utf8');try{const r=eval(c);console.log(r!==undefined?String(r):'undefined');}catch(e){console.error(e.message);process.exit(1);}"#;
    let out = Command::new("node")
        .arg("-e")
        .arg(eval_script)
        .arg(path_str)
        .output()
        .map_err(|e| format!("Node not available or failed: {}", e))?;

    let _ = std::fs::remove_file(&path);

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(stderr.trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_at_boundary_returns_full_string_when_short() {
        let body = "hello";
        assert_eq!(truncate_at_boundary(body, 100), "hello");
    }

    #[test]
    fn truncate_at_boundary_exact_length_returns_full_string() {
        let body = "hello";
        assert_eq!(truncate_at_boundary(body, 5), "hello");
    }

    #[test]
    fn truncate_at_boundary_truncates_at_last_word_boundary() {
        let body = "hello world this is a test";
        let result = truncate_at_boundary(body, 11);
        // Last space within first 11 chars is at index 5 → takes "hello " (6 chars)
        assert_eq!(result, "hello ");
    }

    #[test]
    fn truncate_at_boundary_breaks_at_last_space_before_limit() {
        let body = "abcde fghij klmno";
        let result = truncate_at_boundary(body, 10);
        // Last space within first 10 chars is at index 5 → takes "abcde " (6 chars)
        assert_eq!(result, "abcde ");
    }

    #[test]
    fn truncate_at_boundary_uses_later_boundary_when_available() {
        let body = "ab cd ef gh ij kl mn";
        let result = truncate_at_boundary(body, 10);
        // Last space within first 10 chars is at index 8 (before 'g') → takes "ab cd ef " (9 chars)
        assert_eq!(result, "ab cd ef ");
    }

    #[test]
    fn truncate_at_boundary_no_break_point_uses_max() {
        let body = "abcdefghijklmno";
        let result = truncate_at_boundary(body, 5);
        assert_eq!(result, "abcde");
    }

    #[test]
    fn detects_context_overflow_errors() {
        assert!(is_context_overflow_error("Ollama error: context overflow"));
        assert!(is_context_overflow_error(
            "Ollama error: prompt too long for context"
        ));
        assert!(is_context_overflow_error("context length exceeded"));
        assert!(is_context_overflow_error(
            "maximum context length is 4096 tokens"
        ));
        assert!(is_context_overflow_error(
            "exceeds the model's context window"
        ));
        assert!(is_context_overflow_error("request too long for context"));
        assert!(is_context_overflow_error(
            "llama runner: requested more tokens than fit in the context window"
        ));
        assert!(is_context_overflow_error(
            "error: total prompt tokens exceed context size"
        ));
        assert!(is_context_overflow_error("context window exceeded"));
        assert!(is_context_overflow_error("exceeded the context limit"));
        assert!(is_context_overflow_error(
            "error: exceeds the context limit (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: cannot exceed the context limit for this model"
        ));
        assert!(is_context_overflow_error("input exceeds the context window"));
        assert!(is_context_overflow_error(
            "error: prompt does not fit in the context window"
        ));
        assert!(is_context_overflow_error(
            "llama runner: prompt is larger than the context size"
        ));
        assert!(is_context_overflow_error(
            "input falls outside the context window"
        ));
        assert!(is_context_overflow_error(
            "model ran out of context during generation"
        ));
        assert!(is_context_overflow_error(
            "error: context size exceeded (n_ctx=4096)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: exceeded context size for prompt"
        ));
        assert!(is_context_overflow_error(
            "prompt has more tokens than allowed by the model context"
        ));
        assert!(is_context_overflow_error(
            "input exceeds available context; truncate or increase num_ctx"
        ));
        assert!(is_context_overflow_error(
            "the encoded prompt is longer than the context length"
        ));
        assert!(is_context_overflow_error(
            "llama runner: context limit exceeded (max 8192)"
        ));
        assert!(is_context_overflow_error(
            "error: input exceeds context length for this model"
        ));
        assert!(is_context_overflow_error(
            "requested tokens exceed the maximum context window"
        ));
        assert!(is_context_overflow_error(
            "the prompt is too large for the allocated context"
        ));
        assert!(is_context_overflow_error(
            "error: batch cannot fit in context; reduce prompt size"
        ));
        assert!(is_context_overflow_error(
            "model error: prompt does not fit — context is full"
        ));
        assert!(is_context_overflow_error(
            "error: the prompt is too long for this model"
        ));
        assert!(is_context_overflow_error(
            "llama runner: input exceeds maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "maximum context exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "insufficient context: increase n_ctx or shorten the prompt"
        ));
        assert!(is_context_overflow_error(
            "error: prompt exceeds the context window size"
        ));
        assert!(is_context_overflow_error(
            "batch unable to fit in context; try a smaller prompt"
        ));
        assert!(is_context_overflow_error(
            "the prompt doesn't fit in the allocated context buffer"
        ));
        assert!(is_context_overflow_error(
            "generation won't fit in remaining context"
        ));
        assert!(is_context_overflow_error(
            "llama runner: encoded prompt is greater than the context length"
        ));
        assert!(is_context_overflow_error(
            "error: n_ctx exceeded — prompt requires more tokens than allocated"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: n_ctx overflow while processing batch"
        ));
        assert!(is_context_overflow_error(
            "model options: num_ctx too small; prompt is longer than num_ctx"
        ));
        assert!(is_context_overflow_error(
            "error: requested num_ctx is larger than the model allows"
        ));
        assert!(is_context_overflow_error(
            "error: exceeds maximum sequence length for this model"
        ));
        assert!(is_context_overflow_error(
            "maximum sequence length exceeded (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: sequence length exceeds n_ctx"
        ));
        assert!(is_context_overflow_error(
            "input sequence is too long for the configured context"
        ));
        assert!(is_context_overflow_error(
            "error: total tokens exceed model limit"
        ));
        assert!(is_context_overflow_error(
            "prompt length exceeds maximum allowed tokens"
        ));
        assert!(is_context_overflow_error(
            "too many tokens in prompt for the context window"
        ));
        assert!(is_context_overflow_error(
            "error: max context exceeded (model limit 4096)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: input exceeds max context for this model"
        ));
        assert!(is_context_overflow_error(
            "generation stopped: prompt extends beyond the context window"
        ));
        assert!(is_context_overflow_error(
            "error: not enough context remaining for completion"
        ));
        assert!(is_context_overflow_error(
            "kv cache error: context buffer overflow during prefill"
        ));
        assert!(is_context_overflow_error(
            "prompt tokens exceed the model's maximum context window"
        ));
        assert!(is_context_overflow_error(
            "input tokens exceed n_ctx; reduce prompt or raise num_ctx"
        ));
        assert!(is_context_overflow_error(
            "error: input exceeds the configured context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: configured context full; shorten the prompt"
        ));
        assert!(is_context_overflow_error(
            "generation would exceed remaining context; aborting"
        ));
        assert!(is_context_overflow_error(
            "error: prompt extends past the context boundary"
        ));
        assert!(is_context_overflow_error(
            "model hit the context limit during prefill"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: reached the context limit (n_ctx)"
        ));
        assert!(is_context_overflow_error(
            "error: encoded batch went over the context limit"
        ));
        assert!(is_context_overflow_error(
            "warning: prompt truncated to fit max context"
        ));
        assert!(is_context_overflow_error(
            "server truncated input due to context length constraints"
        ));
        assert!(is_context_overflow_error(
            "llama runner: context exhausted during prefill"
        ));
        assert!(is_context_overflow_error(
            "error: the model context is fully exhausted; shorten the prompt"
        ));
        assert!(is_context_overflow_error(
            "decode failed: KV cache for this context slot is full"
        ));
        assert!(is_context_overflow_error(
            "insufficient remaining context for the completion request"
        ));
        assert!(is_context_overflow_error(
            "error: encoded prompt exceeds the context size (n_ctx=4096)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: context size exceeds allocated n_ctx"
        ));
        assert!(is_context_overflow_error(
            "error: exceeded the context size for this request"
        ));
        assert!(is_context_overflow_error(
            "model error: the context window is too small for this prompt"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: context capacity exceeded during batch decode"
        ));
        assert!(is_context_overflow_error(
            "error: prompt overflows allocated context buffer"
        ));
        assert!(is_context_overflow_error(
            "prefill failed: input exceeds context slot (n_ctx)"
        ));
        assert!(is_context_overflow_error(
            "error: max_context exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "llama runner: prompt is larger than max_context allows"
        ));
        assert!(is_context_overflow_error(
            "validation failed: context_length exceeds the configured maximum"
        ));
        assert!(is_context_overflow_error(
            "json error: context_length too large for model slot"
        ));
        assert!(is_context_overflow_error(
            "API error: maxContext exceeded for this completion request"
        ));
        assert!(is_context_overflow_error(
            "validation: contextLength exceeds server maximum (8192)"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: n_ctx_per_seq too small for encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "error: exceeds the model context limit (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "API error: exceeded the model context for this request"
        ));
        assert!(is_context_overflow_error(
            "validation: model context exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "json: context_window exceeds the configured maximum"
        ));
        assert!(is_context_overflow_error(
            "options: context_window too small for prompt"
        ));
        assert!(is_context_overflow_error(
            "validation: context_limit exceeds the configured maximum"
        ));
        assert!(is_context_overflow_error(
            "json error: contextlimit overflow for this request"
        ));
        assert!(is_context_overflow_error(
            "error: cannot exceed the maximum context for this model"
        ));
        assert!(is_context_overflow_error(
            "llama runner: exceeded maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "model error: maximum context is exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "API error: context token limit exceeded (8192)"
        ));
        assert!(is_context_overflow_error(
            "error: encoded prompt exceeds the context token limit"
        ));
        assert!(is_context_overflow_error(
            "server: exceeded the context token limit for slot 0"
        ));
        assert!(is_context_overflow_error(
            "error: encoded prompt exceeds the model's maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: input exceeded the model's maximum context for this model"
        ));
        assert!(is_context_overflow_error(
            "validation: batch would exceed the model's maximum context"
        ));
        assert!(is_context_overflow_error(
            "error: prompt extends beyond the context window"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: generation ran over the context window boundary"
        ));
        assert!(is_context_overflow_error(
            "error: tokens fall outside of the context window"
        ));
        assert!(is_context_overflow_error(
            "error: input exceeds model maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: cannot exceed the allowed maximum context for this slot"
        ));
        assert!(is_context_overflow_error(
            "API error: encoded prompt is too large for max context"
        ));
        assert!(is_context_overflow_error(
            "llama runner: running out of context during decode"
        ));
        assert!(is_context_overflow_error(
            "error: context budget exceeded for this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: conversation is too long for the model context window"
        ));
        assert!(is_context_overflow_error(
            "openai error: invalid_request_error (context_length_exceeded)"
        ));
        assert!(is_context_overflow_error(
            "{\"error\":{\"code\":\"max_context_length_exceeded\",\"message\":\"...\"}}"
        ));
        assert!(is_context_overflow_error(
            "gateway: type invalid_request_error code context_window_exceeded"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: inference failed: max_context_exceeded"
        ));
        assert!(is_context_overflow_error(
            "error: exceeds this model's context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "API: exceeded this model's context window during prefill"
        ));
        assert!(is_context_overflow_error(
            "validation: cannot exceed this model's context limit"
        ));
        assert!(is_context_overflow_error(
            "API error: messages exceed the model's context window"
        ));
        assert!(is_context_overflow_error(
            "openai: messages exceeded maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: messages exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "API error: message exceeds the model's context window"
        ));
        assert!(is_context_overflow_error(
            "validation: your message exceeded maximum context for this model"
        ));
        assert!(is_context_overflow_error(
            "API: the message is too long for the model's context window"
        ));
        assert!(is_context_overflow_error(
            "error: messages are too long for maximum context on this model"
        ));
        assert!(is_context_overflow_error(
            "batch: inputs are too long for the model's context window"
        ));
        assert!(is_context_overflow_error(
            "API: inputs were too long; exceeded available context on this request"
        ));
        assert!(is_context_overflow_error(
            "validation: input is too long for context length 8192"
        ));
        assert!(is_context_overflow_error(
            "request failed: message was too long; exceeds available context"
        ));
        assert!(is_context_overflow_error(
            "error: request exceeds your maximum allowed context for this model"
        ));
        assert!(is_context_overflow_error(
            "validation: cannot exceed max allowed context on this endpoint"
        ));
        assert!(is_context_overflow_error(
            "gateway: context length limit exceeded for the chat completion"
        ));
        assert!(is_context_overflow_error(
            "openai error: invalid_request_error (context_budget_exceeded)"
        ));
        assert!(is_context_overflow_error(
            "API error: inputs exceed the model's context window"
        ));
        assert!(is_context_overflow_error(
            "gateway: inputs exceeded available context on this request"
        ));
        assert!(is_context_overflow_error(
            "validation: inputs exceed context length for this model"
        ));
        assert!(is_context_overflow_error(
            "API: input exceeded available context on this request"
        ));
        assert!(is_context_overflow_error(
            "validation: input exceed the model's context window"
        ));
        assert!(is_context_overflow_error(
            "your message is too long for this model's context"
        ));
        assert!(is_context_overflow_error(
            "chat: messages exceed the model's context on this turn"
        ));
        assert!(is_context_overflow_error(
            "batch: inputs exceeded the model's context"
        ));
    }

    #[test]
    fn does_not_match_unrelated_errors() {
        assert!(!is_context_overflow_error(
            "Ollama HTTP 503: service unavailable"
        ));
        assert!(!is_context_overflow_error("connection refused"));
        assert!(!is_context_overflow_error("rate limit exceeded"));
        assert!(!is_context_overflow_error("timeout"));
        assert!(!is_context_overflow_error(
            "cannot fit model weights into GPU memory"
        ));
        assert!(!is_context_overflow_error("request too large"));
        assert!(!is_context_overflow_error("unable to fit in GPU memory"));
        assert!(!is_context_overflow_error("the file won't fit on disk"));
        assert!(!is_context_overflow_error(
            "diagnostic: n_ctx=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "hint: set num_ctx in Modelfile to match the model card"
        ));
        assert!(!is_context_overflow_error(
            "too many tokens in request (rate limited)"
        ));
        assert!(!is_context_overflow_error(
            "billing: total tokens exceed your monthly quota"
        ));
        assert!(!is_context_overflow_error(
            "API error: prompt tokens exceed per-minute billing cap"
        ));
        assert!(!is_context_overflow_error(
            "network buffer full; retry later"
        ));
        assert!(!is_context_overflow_error(
            "response truncated for display (unrelated to model context)"
        ));
        assert!(!is_context_overflow_error(
            "log truncated; see full trace for context"
        ));
        assert!(!is_context_overflow_error(
            "this feature is fully supported in the application context"
        ));
        assert!(!is_context_overflow_error(
            "gpu kv cache is full (tensor allocation failed)"
        ));
        assert!(!is_context_overflow_error(
            "prefill completed; context tensors initialized"
        ));
        assert!(!is_context_overflow_error(
            "Modelfile: allocated context is 8192 tokens (default)"
        ));
        assert!(!is_context_overflow_error(
            "options: max_context=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "request fields: context_length, temperature, stream"
        ));
        assert!(!is_context_overflow_error(
            "config: maxContext=8192 num_ctx=8192 (startup ok)"
        ));
        assert!(!is_context_overflow_error(
            "docs: contextLength optional in request schema"
        ));
        assert!(!is_context_overflow_error(
            "diagnostic: n_ctx_per_seq=4096 batch ok"
        ));
        assert!(!is_context_overflow_error(
            "defaults: context_window=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "docs: context_window optional in request schema"
        ));
        assert!(!is_context_overflow_error(
            "note: tune model context for your workload (no error)"
        ));
        assert!(!is_context_overflow_error(
            "options: context_limit=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "request fields: context_limit, temperature, stream"
        ));
        assert!(!is_context_overflow_error(
            "config: contextLimit=8192 (startup ok)"
        ));
        assert!(!is_context_overflow_error(
            "API token limit exceeded: upgrade your plan"
        ));
        assert!(!is_context_overflow_error(
            "docs: default maximum context is 8192 tokens (informational)"
        ));
        assert!(!is_context_overflow_error(
            "hint: tune max context in Modelfile for longer threads (no error)"
        ));
        assert!(!is_context_overflow_error(
            "running out of patience while waiting for the API"
        ));
        assert!(!is_context_overflow_error(
            "note: context budget is 1M tokens on the enterprise plan (informational)"
        ));
        assert!(!is_context_overflow_error(
            "UI: conversation is too long to display; click to expand"
        ));
        assert!(!is_context_overflow_error(
            "migration: dropped column old_context_length_exceeded from analytics"
        ));
        assert!(!is_context_overflow_error(
            "refactor: rename old_max_context_exceeded flag to overflow_seen"
        ));
        assert!(!is_context_overflow_error(
            "status messages exceed rate limits (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "UI: error message exceeds 80 characters (display limit)"
        ));
        assert!(!is_context_overflow_error(
            "discord: message is too long (max 2000 characters)"
        ));
        assert!(!is_context_overflow_error(
            "form validation: inputs are too long (max 10 text fields)"
        ));
        assert!(!is_context_overflow_error(
            "form error: the input is too long (max 500 chars)"
        ));
        assert!(!is_context_overflow_error(
            "API: inputs exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "validation: inputs exceeded the maximum attachment size"
        ));
        assert!(!is_context_overflow_error(
            "API: input exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "validation: input exceeded the maximum attachment size"
        ));
        assert!(!is_context_overflow_error(
            "docs: maximum allowed context is 128000 tokens (informational)"
        ));
        assert!(!is_context_overflow_error(
            "config: context length limit is 8192 (startup ok)"
        ));
        assert!(!is_context_overflow_error(
            "migration: dropped column old_context_budget_exceeded from events"
        ));
    }

    fn make_msg(role: &str, content: &str) -> crate::ollama::ChatMessage {
        crate::ollama::ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            images: None,
        }
    }

    #[test]
    fn truncate_tool_results_skips_system_prompt() {
        let big = "x".repeat(10_000);
        let mut msgs = vec![make_msg("system", &big), make_msg("user", "hello")];
        let n = truncate_oversized_tool_results(&mut msgs, 500);
        assert_eq!(n, 0, "system prompt at index 0 should not be truncated");
        assert_eq!(msgs[0].content.len(), 10_000);
    }

    #[test]
    fn truncate_tool_results_truncates_large_messages() {
        let big_result = "word ".repeat(2000);
        let mut msgs = vec![
            make_msg("system", "You are an AI."),
            make_msg("user", "fetch https://example.com"),
            make_msg("assistant", "FETCH_URL: https://example.com"),
            make_msg("user", &big_result),
        ];
        let n = truncate_oversized_tool_results(&mut msgs, 500);
        assert_eq!(n, 1);
        assert!(
            msgs[3].content.chars().count() < 600,
            "expected truncated msg under 600 chars, got {}",
            msgs[3].content.chars().count()
        );
        assert!(msgs[3].content.contains("[truncated from"));
    }

    #[test]
    fn truncate_tool_results_leaves_small_messages() {
        let mut msgs = vec![
            make_msg("system", "You are an AI."),
            make_msg("user", "hello"),
            make_msg("assistant", "hi there"),
        ];
        let n = truncate_oversized_tool_results(&mut msgs, 500);
        assert_eq!(n, 0);
    }

    #[test]
    fn truncate_tool_results_handles_multiple() {
        let big1 = "a".repeat(5000);
        let big2 = "b".repeat(8000);
        let mut msgs = vec![
            make_msg("system", "prompt"),
            make_msg("user", &big1),
            make_msg("assistant", "ok"),
            make_msg("user", &big2),
        ];
        let n = truncate_oversized_tool_results(&mut msgs, 1000);
        assert_eq!(n, 2);
        assert!(msgs[1].content.contains("[truncated from 5000 to 1000"));
        assert!(msgs[3].content.contains("[truncated from 8000 to 1000"));
    }

    #[test]
    fn sanitize_context_overflow_suggests_new_topic() {
        let msg = sanitize_ollama_error_for_user("Ollama error: context overflow");
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("new topic"));
        assert!(msg.contains("context window"));
    }

    #[test]
    fn sanitize_prompt_too_long_suggests_new_topic() {
        let msg = sanitize_ollama_error_for_user("Ollama error: prompt too long for context");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("new topic"));
    }

    #[test]
    fn sanitize_maximum_context_length_tokens() {
        let msg = sanitize_ollama_error_for_user("maximum context length is 2048 tokens");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("context window"));
    }

    #[test]
    fn sanitize_context_length_exceeded_phrase() {
        let msg = sanitize_ollama_error_for_user("context length exceeded");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("new topic"));
    }

    #[test]
    fn sanitize_exceeds_models_context_window_phrase() {
        let msg = sanitize_ollama_error_for_user("input exceeds the model's context window");
        let text = msg.expect("overflow phrase should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_requested_more_tokens_than_context_phrase() {
        let msg = sanitize_ollama_error_for_user(
            "llama runner: requested more tokens than fit in the context window",
        );
        let text = msg.expect("runner overflow phrase should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_fit_in_the_context_phrase() {
        let msg = sanitize_ollama_error_for_user(
            "error: prompt does not fit in the context window",
        );
        let text = msg.expect("fit-in-context phrase should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_context_size_exceeded_phrase() {
        let msg = sanitize_ollama_error_for_user(
            "llama runner: context size exceeded (n_ctx=8192)",
        );
        let text = msg.expect("context size exceeded should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_role_ordering_error() {
        let msg =
            sanitize_ollama_error_for_user("Ollama error: roles must alternate user/assistant");
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("ordering"));
        assert!(msg.contains("new topic"));
    }

    #[test]
    fn sanitize_incorrect_role_error() {
        let msg = sanitize_ollama_error_for_user("incorrect role information in message");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("ordering"));
    }

    #[test]
    fn sanitize_corrupted_session_tool_missing() {
        let msg = sanitize_ollama_error_for_user("tool call input missing from history");
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("corrupted"));
        assert!(msg.contains("new topic"));
    }

    #[test]
    fn sanitize_returns_none_for_unknown_errors() {
        assert!(sanitize_ollama_error_for_user("connection refused").is_none());
        assert!(sanitize_ollama_error_for_user("timeout").is_none());
        assert!(sanitize_ollama_error_for_user("Ollama HTTP 503: service unavailable").is_none());
        assert!(sanitize_ollama_error_for_user("Failed to send chat request").is_none());
    }
}
