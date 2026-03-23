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

/// True if `phrase` appears in `haystack` and is not immediately preceded by ASCII alnum / `_`
/// (so `rows exceed` does not match inside `arrows exceed` / `throws exceed`, `columns exceed`
/// does not match inside `microcolumns exceed`, `tables exceed` does not match inside
/// `constables exceed`, `table exceed` does not match inside `stable exceed`, `blocks exceed`
/// does not match inside `roadblocks exceed`, `block exceed` does not match inside
/// `roadblock exceed` / `sunblock exceed`, `segments exceed` does not match inside
/// `microsegments exceed`, `segment exceed` does not match inside `multisegment exceeds`,
/// `sections exceed` does not match inside `subsections exceed`, and `section exceed` does not
/// match inside `intersection exceed`, `paragraphs exceed` does not match inside
/// `counterparagraphs exceed`, and `paragraph exceed` does not match inside
/// `counterparagraph exceed`, `sentences exceed` does not match inside
/// `microsentences exceed`, and `sentence exceed` does not match inside
/// `microsentence exceed`, `words exceed` does not match inside
/// `buzzwords exceed` / `keywords exceed`, and `word exceed` does not match inside
/// `buzzword exceed`; `characters exceed` does not match inside `megacharacters exceed` /
/// `metacharacters exceed`, and `character exceed` does not match inside `noncharacter exceed`;
/// `bytes exceed` does not match inside `megabytes exceed` / `kilobytes exceed`, and
/// `byte exceed` does not match inside `kilobyte exceed`; `bits exceed` does not match inside
/// `megabits exceed` / `kilobits exceed`, and `bit exceed` does not match inside `kilobit exceed`
/// or as a substring of `rabbit exceed` (left-boundary rejects the inner `bit`); `fields exceed`
/// does not match inside `battlefields exceed` / `cornfields exceed`, and `field exceed` does not
/// match inside `afield exceed` / `subfield exceed`; `values exceed` does not match inside
/// `eigenvalues exceed` / `meanvalues exceed`, and `value exceed` does not match inside
/// `devalue exceed` / `overvalue exceed`; `keys exceed` does not match inside
/// `hotkeys exceed` / `turnkeys exceed`, and `key exceed` does not match inside
/// `monkey exceed` / `passkey exceed`; `properties exceed` does not match inside
/// `microproperties exceed`, and `property exceed` does not match inside
/// `subproperty exceed`; `schemas exceed` does not match inside `microschemas exceed` /
/// `holoschemas exceed`, and `schema exceed` does not match inside `subschema exceed`;
/// `parameters exceed` does not match inside `microparameters exceed` /
/// `metaparameters exceed`, and `parameter exceed` does not match inside
/// `subparameter exceed`; `arguments exceed` does not match inside
/// `microarguments exceed` / `metaarguments exceed`, and `argument exceed` does not match inside
/// `subargument exceed`; `variables exceed` does not match inside `metavariables exceed` /
/// `hypervariables exceed`, and `variable exceed` does not match inside `multivariable exceed` /
/// `subvariable exceed`; `headers exceed` does not match inside `microheaders exceed` /
/// `metaheaders exceed`, and `header exceed` does not match inside `subheader exceed`;
/// `cookies exceed` does not match inside `microcookies exceed` / `metacookies exceed`, and
/// `cookie exceed` does not match inside `subcookie exceed`; `bodies exceed` does not match
/// inside `microbodies exceed` / `metabodies exceed`, and `body exceed` does not match inside
/// `subbody exceed`.
fn contains_phrase_after_ident_boundary(haystack: &str, phrase: &str) -> bool {
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    for (i, _) in haystack.match_indices(phrase) {
        if i == 0 {
            return true;
        }
        if !haystack[..i]
            .chars()
            .next_back()
            .is_some_and(ident_continue)
        {
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
        // Compact wording: no `the` between preposition and `context window` (FEAT-D304;
        // distinct from FEAT-D291's `beyond the` / `over the` / `outside of the` substrings).
        || lower.contains("beyond context window")
        || lower.contains("over context window")
        || lower.contains("outside context window")
        || lower.contains("outside of context window")
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
        // Plural / singular "content(s) exceed(s/ed)" (FEAT-D303). Wording like
        // `contents exceed the context window` does not substring-match `exceeds the context window`.
        // `content exceed` covers singular `exceeds` / `exceeded` without matching `contents exceed`
        // (the `s` after `content` breaks the substring). Same context-slot guard as `inputs exceed`.
        || ((lower.contains("contents exceed") || lower.contains("content exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "output(s) exceed(s/ed)" (FEAT-D305). Parallel to `inputs exceed` /
        // `content exceed`. `output exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match `outputs exceed` (the `s` after `output`).
        || ((lower.contains("outputs exceed")
            || lower.contains("outputs exceeded")
            || lower.contains("output exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "response(s) exceed(s/ed)" (FEAT-D306). Parallel to `outputs exceed` /
        // `inputs exceed`. `response exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match `responses exceed` (the `s` after `response`).
        || ((lower.contains("responses exceed")
            || lower.contains("responses exceeded")
            || lower.contains("response exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "request(s) exceed(s/ed)" (FEAT-D307). Parallel to `responses exceed` /
        // `inputs exceed`. `request exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match `requests exceed` (the `s` after `request`).
        || ((lower.contains("requests exceed")
            || lower.contains("requests exceeded")
            || lower.contains("request exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "quer(y/ies) exceed(s/ed)" (FEAT-D308). Parallel to `requests exceed` /
        // `inputs exceed`. `query exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `queries exceed` (no `query` + ` exceed`
        // inside the word `queries`).
        || ((lower.contains("queries exceed")
            || lower.contains("queries exceeded")
            || lower.contains("query exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "call(s) exceed(s/ed)" (FEAT-D309). Parallel to `queries exceed` /
        // `requests exceed`. `call exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `calls exceed` (the `s` after `call`).
        || ((lower.contains("calls exceed")
            || lower.contains("calls exceeded")
            || lower.contains("call exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "batch(es) exceed(s/ed)" (FEAT-D310). Parallel to `calls exceed` /
        // `queries exceed`. `batch exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `batches exceed` (the `es` after `batch`).
        || ((lower.contains("batches exceed")
            || lower.contains("batches exceeded")
            || lower.contains("batch exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "token(s) exceed(s/ed)" (FEAT-D311). Parallel to `batches exceed` /
        // `batch exceed`. `token exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `tokens exceed` (the `s` after `token`).
        || ((lower.contains("tokens exceed")
            || lower.contains("tokens exceeded")
            || lower.contains("token exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "item(s) exceed(s/ed)" (FEAT-D312). Parallel to `tokens exceed` /
        // `token exceed`. `item exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `items exceed` (the `s` after `item`).
        || ((lower.contains("items exceed")
            || lower.contains("items exceeded")
            || lower.contains("item exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "entr(y/ies) exceed(s/ed)" (FEAT-D313). Parallel to `items exceed` /
        // `item exceed`. `entry exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `entries exceed` (`entr` + `ies` vs `entry`).
        || ((lower.contains("entries exceed")
            || lower.contains("entries exceeded")
            || lower.contains("entry exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "record(s) exceed(s/ed)" (FEAT-D314). Parallel to `entries exceed` /
        // `entry exceed`. `record exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `records exceed` (the `s` after `record`).
        || ((lower.contains("records exceed")
            || lower.contains("records exceeded")
            || lower.contains("record exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "chunk(s) exceed(s/ed)" (FEAT-D315). Parallel to `records exceed` /
        // `record exceed`. `chunk exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `chunks exceed` (the `s` after `chunk`).
        || ((lower.contains("chunks exceed")
            || lower.contains("chunks exceeded")
            || lower.contains("chunk exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "document(s) exceed(s/ed)" (FEAT-D316). Parallel to `chunks exceed` /
        // `chunk exceed`. `document exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `documents exceed` (the `s` after `document`).
        || ((lower.contains("documents exceed")
            || lower.contains("documents exceeded")
            || lower.contains("document exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "file(s) exceed(s/ed)" (FEAT-D317). Parallel to `documents exceed` /
        // `document exceed`. `file exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `files exceed` (the `s` after `file`).
        || ((lower.contains("files exceed")
            || lower.contains("files exceeded")
            || lower.contains("file exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "line(s) exceed(s/ed)" (FEAT-D318). Parallel to `files exceed` /
        // `file exceed`. `line exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `lines exceed` (the `s` after `line`).
        || ((lower.contains("lines exceed")
            || lower.contains("lines exceeded")
            || lower.contains("line exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "cell(s) exceed(s/ed)" (FEAT-D319). Parallel to `lines exceed` /
        // `line exceed`. `cell exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `cells exceed` (the `s` after `cell`).
        || ((lower.contains("cells exceed")
            || lower.contains("cells exceeded")
            || lower.contains("cell exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "row(s) exceed(s/ed)" (FEAT-D320). Parallel to `cells exceed` /
        // `cell exceed`. `row exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `rows exceed` (the `s` after `row`).
        // Ident-boundary on the phrases so `arrows exceed` / `throws exceed` / `arrow exceed`
        // do not match.
        || ((contains_phrase_after_ident_boundary(&lower, "rows exceed")
            || contains_phrase_after_ident_boundary(&lower, "rows exceeded")
            || contains_phrase_after_ident_boundary(&lower, "row exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "column(s) exceed(s/ed)" (FEAT-D321). Parallel to `rows exceed` /
        // `row exceed`. `column exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `columns exceed` (the `s` after `column`).
        // Ident-boundary so `microcolumns exceed` / `multicolumn exceeds` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "columns exceed")
            || contains_phrase_after_ident_boundary(&lower, "columns exceeded")
            || contains_phrase_after_ident_boundary(&lower, "column exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "table(s) exceed(s/ed)" (FEAT-D322). Parallel to `columns exceed` /
        // `column exceed`. `table exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `tables exceed` (the `s` after `table`).
        // Ident-boundary so `constables exceed` / `stable exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "tables exceed")
            || contains_phrase_after_ident_boundary(&lower, "tables exceeded")
            || contains_phrase_after_ident_boundary(&lower, "table exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "block(s) exceed(s/ed)" (FEAT-D323). Parallel to `tables exceed` /
        // `table exceed`. `block exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `blocks exceed` (the `s` after `block`).
        // Ident-boundary so `roadblocks exceed` / `roadblock exceed` / `sunblock exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "blocks exceed")
            || contains_phrase_after_ident_boundary(&lower, "blocks exceeded")
            || contains_phrase_after_ident_boundary(&lower, "block exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "segment(s) exceed(s/ed)" (FEAT-D324). Parallel to `blocks exceed` /
        // `block exceed`. `segment exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `segments exceed` (the `s` after `segment`).
        // Ident-boundary so `microsegments exceed` / `multisegment exceeds` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "segments exceed")
            || contains_phrase_after_ident_boundary(&lower, "segments exceeded")
            || contains_phrase_after_ident_boundary(&lower, "segment exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "section(s) exceed(s/ed)" (FEAT-D325). Parallel to `segments exceed` /
        // `segment exceed`. `section exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `sections exceed` (the `s` after `section`).
        // Ident-boundary so `subsections exceed` / `intersection exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "sections exceed")
            || contains_phrase_after_ident_boundary(&lower, "sections exceeded")
            || contains_phrase_after_ident_boundary(&lower, "section exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "paragraph(s) exceed(s/ed)" (FEAT-D326). Parallel to `sections exceed` /
        // `section exceed`. `paragraph exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `paragraphs exceed` (the `s` after `paragraph`).
        // Ident-boundary so `counterparagraphs exceed` / `counterparagraph exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "paragraphs exceed")
            || contains_phrase_after_ident_boundary(&lower, "paragraphs exceeded")
            || contains_phrase_after_ident_boundary(&lower, "paragraph exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "sentence(s) exceed(s/ed)" (FEAT-D327). Parallel to `paragraphs exceed` /
        // `paragraph exceed`. `sentence exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `sentences exceed` (the `s` after `sentence`).
        // Ident-boundary so `microsentences exceed` / `microsentence exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "sentences exceed")
            || contains_phrase_after_ident_boundary(&lower, "sentences exceeded")
            || contains_phrase_after_ident_boundary(&lower, "sentence exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "word(s) exceed(s/ed)" (FEAT-D328). Parallel to `sentences exceed` /
        // `sentence exceed`. `word exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `words exceed` (the `s` after `word`).
        // Ident-boundary so `buzzwords exceed` / `keywords exceed` / `buzzword exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "words exceed")
            || contains_phrase_after_ident_boundary(&lower, "words exceeded")
            || contains_phrase_after_ident_boundary(&lower, "word exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "character(s) exceed(s/ed)" (FEAT-D329). Parallel to `words exceed` /
        // `word exceed`. `character exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `characters exceed` (the `s` after `character`).
        // Ident-boundary so `megacharacters exceed` / `metacharacters exceed` / `noncharacter exceed`
        // do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "characters exceed")
            || contains_phrase_after_ident_boundary(&lower, "characters exceeded")
            || contains_phrase_after_ident_boundary(&lower, "character exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "byte(s) exceed(s/ed)" (FEAT-D330). Parallel to `characters exceed` /
        // `character exceed`. `byte exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `bytes exceed` (the `s` after `byte`).
        // Ident-boundary so `megabytes exceed` / `kilobytes exceed` / `kilobyte exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "bytes exceed")
            || contains_phrase_after_ident_boundary(&lower, "bytes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "byte exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "bit(s) exceed(s/ed)" (FEAT-D331). Parallel to `bytes exceed` /
        // `byte exceed`. `bit exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `bits exceed` (the `s` after `bit`).
        // Ident-boundary so `megabits exceed` / `kilobits exceed` / `kilobit exceed` and
        // `rabbit exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "bits exceed")
            || contains_phrase_after_ident_boundary(&lower, "bits exceeded")
            || contains_phrase_after_ident_boundary(&lower, "bit exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "field(s) exceed(s/ed)" (FEAT-D332). Parallel to `bits exceed` /
        // `bit exceed`. `field exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `fields exceed` (the `s` after `field`).
        // Ident-boundary so `battlefields exceed` / `cornfields exceed` / `afield exceed` /
        // `subfield exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "fields exceed")
            || contains_phrase_after_ident_boundary(&lower, "fields exceeded")
            || contains_phrase_after_ident_boundary(&lower, "field exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "value(s) exceed(s/ed)" (FEAT-D333). Parallel to `fields exceed` /
        // `field exceed`. `value exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `values exceed` (the `s` after `value`).
        // Ident-boundary so `eigenvalues exceed` / `meanvalues exceed` / `devalue exceed` /
        // `overvalue exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "values exceed")
            || contains_phrase_after_ident_boundary(&lower, "values exceeded")
            || contains_phrase_after_ident_boundary(&lower, "value exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "key(s) exceed(s/ed)" (FEAT-D334). Parallel to `values exceed` /
        // `value exceed`. `key exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `keys exceed` (the `s` after `key`).
        // Ident-boundary so `hotkeys exceed` / `turnkeys exceed` / `monkey exceed` /
        // `passkey exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "keys exceed")
            || contains_phrase_after_ident_boundary(&lower, "keys exceeded")
            || contains_phrase_after_ident_boundary(&lower, "key exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "propert(y/ies) exceed(s/ed)" (FEAT-D335). Parallel to `keys exceed` /
        // `key exceed`. `property exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `properties exceed` (the `s` after
        // `property`). Ident-boundary so `microproperties exceed` / `subproperty exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "properties exceed")
            || contains_phrase_after_ident_boundary(&lower, "properties exceeded")
            || contains_phrase_after_ident_boundary(&lower, "property exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "schema(s) exceed(s/ed)" (FEAT-D336). Parallel to `properties exceed` /
        // `property exceed`. `schema exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `schemas exceed` (the `s` after `schema`).
        // Ident-boundary so `microschemas exceed` / `holoschemas exceed` / `subschema exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "schemas exceed")
            || contains_phrase_after_ident_boundary(&lower, "schemas exceeded")
            || contains_phrase_after_ident_boundary(&lower, "schema exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "parameter(s) exceed(s/ed)" (FEAT-D337). Parallel to `schemas exceed` /
        // `schema exceed`. `parameter exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `parameters exceed` (the `s` after
        // `parameter`). Ident-boundary so `microparameters exceed` / `metaparameters exceed` /
        // `subparameter exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "parameters exceed")
            || contains_phrase_after_ident_boundary(&lower, "parameters exceeded")
            || contains_phrase_after_ident_boundary(&lower, "parameter exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "argument(s) exceed(s/ed)" (FEAT-D338). Parallel to `parameters exceed` /
        // `parameter exceed`. `argument exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `arguments exceed` (the `s` after
        // `argument`). Ident-boundary so `microarguments exceed` / `metaarguments exceed` /
        // `subargument exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "arguments exceed")
            || contains_phrase_after_ident_boundary(&lower, "arguments exceeded")
            || contains_phrase_after_ident_boundary(&lower, "argument exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "variable(s) exceed(s/ed)" (FEAT-D339). Parallel to `arguments exceed` /
        // `argument exceed`. `variable exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `variables exceed` (the `s` after
        // `variable`). Ident-boundary so `metavariables exceed` / `hypervariables exceed` /
        // `multivariable exceed` / `subvariable exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "variables exceed")
            || contains_phrase_after_ident_boundary(&lower, "variables exceeded")
            || contains_phrase_after_ident_boundary(&lower, "variable exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "header(s) exceed(s/ed)" (FEAT-D340). Parallel to `variables exceed` /
        // `variable exceed`. `header exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `headers exceed` (the `s` after `header`).
        // Ident-boundary so `microheaders exceed` / `metaheaders exceed` / `subheader exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "headers exceed")
            || contains_phrase_after_ident_boundary(&lower, "headers exceeded")
            || contains_phrase_after_ident_boundary(&lower, "header exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "cookie(s) exceed(s/ed)" (FEAT-D341). Parallel to `headers exceed` /
        // `header exceed`. `cookie exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `cookies exceed` (the `s` after `cookie`).
        // Ident-boundary so `microcookies exceed` / `metacookies exceed` / `subcookie exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "cookies exceed")
            || contains_phrase_after_ident_boundary(&lower, "cookies exceeded")
            || contains_phrase_after_ident_boundary(&lower, "cookie exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "bod(y/ies) exceed(s/ed)" (FEAT-D342). Parallel to `cookies exceed` /
        // `cookie exceed`. `body exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `bodies exceed` (the `s` after `body`).
        // Ident-boundary so `microbodies exceed` / `metabodies exceed` / `subbody exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "bodies exceed")
            || contains_phrase_after_ident_boundary(&lower, "bodies exceeded")
            || contains_phrase_after_ident_boundary(&lower, "body exceed"))
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
            "llama runner: prompt extends beyond context window (8192)"
        ));
        assert!(is_context_overflow_error(
            "API: encoded span runs over context window for slot 0"
        ));
        assert!(is_context_overflow_error(
            "error: batch indices fall outside context window"
        ));
        assert!(is_context_overflow_error(
            "validator: token range lies outside of context window"
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
        assert!(is_context_overflow_error(
            "API: contents exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: contents exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: content exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "error: content exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: outputs exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: outputs exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: output exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "runner: output exceeded the context window during decode"
        ));
        assert!(is_context_overflow_error(
            "API: responses exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: responses exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: response exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: response exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: requests exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: requests exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: request exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: request exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: queries exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: queries exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: query exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: query exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: calls exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: calls exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: call exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: call exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: batches exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: batches exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: batch exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: batch exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: tokens exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: tokens exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: token exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: token exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: items exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: items exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: item exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: item exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: entries exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: entries exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: entry exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: entry exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: records exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: records exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: record exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: record exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: chunks exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: chunks exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: chunk exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: chunk exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: documents exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: documents exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: document exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: document exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: files exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: files exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: file exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: file exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: lines exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: lines exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: line exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: line exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: cells exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: cells exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: cell exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: cell exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: rows exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: rows exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: row exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: row exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: columns exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: columns exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: column exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: column exceeded the context window"
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
            "validation: contents exceed max attachment size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "batch: contents exceed per-request rate limits"
        ));
        assert!(!is_context_overflow_error(
            "GPU: shader outputs exceed max attachment slots (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "pipeline: outputs exceed per-stage rate limits"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: responses exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: requests exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: queries exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: calls exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: batches exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: tokens exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: items exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: entries exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: records exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: chunks exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: documents exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: files exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: tokens exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: items exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: entries exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: records exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: chunks exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: documents exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: files exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: file exceed max upload size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: lines exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: lines exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "log: line exceed max line length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: cells exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: cells exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "spreadsheet: cell exceed max formula length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: rows exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: rows exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "spreadsheet: row exceed max row height (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: columns exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: columns exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "spreadsheet: column exceed max column width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: tables exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: tables exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: table exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: table exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: tables exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: tables exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "SQL: constables exceed department headcount (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: stable exceed viewport width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: blocks exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: blocks exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: block exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: block exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: blocks exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: blocks exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: block exceed max object size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "traffic: roadblocks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "safety: roadblock exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "skincare: sunblock exceed SPF labeling limits (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: segments exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: segments exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: segment exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: segment exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: segments exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: segments exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "video: segment exceed max GOP duration (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: microsegments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: multisegment exceeds max track width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: sections exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: sections exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: section exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: section exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: sections exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: sections exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: section exceed max heading depth (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "docs: subsections exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "geometry: intersection exceed tolerance (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: paragraphs exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: paragraphs exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: paragraph exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: paragraph exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: paragraphs exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: paragraphs exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: paragraph exceed max width in twips (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "legal: counterparagraphs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "brief: counterparagraph exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: sentences exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: sentences exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: sentence exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: sentence exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: sentences exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: sentences exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "NLP: sentence exceed max tokens per span (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: microsentences exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: microsentence exceed display width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: words exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: words exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: word exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: word exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: words exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: words exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "NLP: word exceed max syllables per token (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "lexicon: buzzwords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "SEO: keywords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "puzzles: crosswords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "RAG: microwords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "glossary: buzzword exceed display width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: characters exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: characters exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: character exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: character exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: characters exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: characters exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "UI: character exceed max field width in pixels (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: megacharacters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "regex: metacharacters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: noncharacter exceed token class limit (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: bytes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bytes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: byte exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: byte exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bytes exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bytes exceeded daily upload cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: byte exceed max object size on this bucket (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: megabytes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "transfer: kilobytes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "codec: kilobyte exceed frame size cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: bits exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bits exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: bit exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: bit exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bits exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bits exceeded daily transfer cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "codec: bit exceed max frame size on this stream (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: megabits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "transfer: kilobits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "codec: kilobit exceed symbol size cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "parser: rabbit exceed max nesting depth (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: fields exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: fields exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: field exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: field exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: fields exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: fields exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: field exceed max string length on this column (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: battlefields exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "geo: cornfields exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: afield exceed display width cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: subfield exceed nesting depth (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: values exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: values exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: value exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: value exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: values exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: values exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: value exceed max numeric range on this column (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "stats: eigenvalues exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "solver: meanvalues exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "finance: devalue exceed policy threshold (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "pricing: overvalue exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: keys exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: keys exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: key exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: key exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: keys exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: keys exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: key exceed max name length on this column (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "UI: hotkeys exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "property: turnkeys exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "zoo: monkey exceed feeding schedule cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "auth: passkey exceed device binding limit (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: properties exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: properties exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: property exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: property exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: properties exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: properties exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: property exceed max nesting depth on this object (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microproperties exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subproperty exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: schemas exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: schemas exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: schema exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: schema exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: schemas exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: schemas exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "registry: schema exceed max $ref depth on this object (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microschemas exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subschema exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: parameters exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: parameters exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: parameter exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: parameter exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: parameters exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: parameters exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "openapi: parameter exceed max in-path segments on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microparameters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaparameters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subparameter exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: arguments exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: arguments exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: argument exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: argument exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: arguments exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: arguments exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "graphql: argument exceed max list depth on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microarguments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaarguments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subargument exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "GraphQL: variables exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: variables exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: variable exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: variable exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: variables exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: variables exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "template: variable exceed max substitution depth on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: metavariables exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: hypervariables exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: multivariable exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: subvariable exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: headers exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: headers exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: header exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: header exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: headers exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: headers exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: header exceed max allowed total size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microheaders exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaheaders exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subheader exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: cookies exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: cookies exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: cookie exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: cookie exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: cookies exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: cookies exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: cookie exceed max allowed total count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcookies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacookies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subcookie exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: bodies exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bodies exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: body exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: body exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bodies exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bodies exceeded max upload size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: body exceed max allowed total size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbodies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabodies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subbody exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "microcolumns exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "multicolumn exceeds max column width (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "arrows exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "throws exceed available context for the completion"
        ));
        assert!(!is_context_overflow_error(
            "arrow exceed per-row display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "catalog: entry exceed max sku length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "audit: record exceed max field length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: document exceed max attachment size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "database: queries exceeded the allowed execution time"
        ));
        assert!(!is_context_overflow_error(
            "survey requests exceed the allowed quota for this form"
        ));
        assert!(!is_context_overflow_error(
            "survey responses exceed the allowed quota for this form"
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
