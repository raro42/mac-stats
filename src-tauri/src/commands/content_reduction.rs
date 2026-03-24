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
/// `subbody exceed`; `parts exceed` does not match inside `microparts exceed` /
/// `metaparts exceed`, and `part exceed` does not match inside `subpart exceed`;
/// `pieces exceed` does not match inside `micropieces exceed` /
/// `metapieces exceed`, and `piece exceed` does not match inside `subpiece exceed`;
/// `shards exceed` does not match inside `microshards exceed` /
/// `metashards exceed`, and `shard exceed` does not match inside `subshard exceed`
/// (left-boundary rejects `reshard exceed`); `fragments exceed` does not match inside
/// `microfragments exceed` / `metafragments exceed`, and `fragment exceed` does not match inside
/// `subfragment exceed` (left-boundary rejects `refragment exceed`); `packets exceed` does not match
/// inside `micropackets exceed` / `metapackets exceed`, and `packet exceed` does not match inside
/// `subpacket exceed` (left-boundary rejects `repacket exceed`); `frames exceed` does not match
/// inside `microframes exceed` / `metaframes exceed`, and `frame exceed` does not match inside
/// `subframe exceed` (left-boundary rejects `reframe exceed`); `samples exceed` does not match
/// inside `microsamples exceed` / `metasamples exceed`, and `sample exceed` does not match inside
/// `subsample exceed` (left-boundary rejects `resample exceed`); `observations exceed` does not
/// match inside `microobservations exceed` / `metaobservations exceed`, and `observation exceed`
/// does not match inside `subobservation exceed` (left-boundary rejects `preobservation exceed`);
/// `events exceed` does not match inside `microevents exceed` / `metaevents exceed`, and
/// `event exceed` does not match inside `subevent exceed` (left-boundary rejects `preevent exceed`);
/// `traces exceed` does not match inside `microtraces exceed` / `metatraces exceed`, and
/// `trace exceed` does not match inside `subtrace exceed` (left-boundary rejects `pretrace exceed`
/// and `retrace exceed`);
/// `spans exceed` does not match inside `microspans exceed` / `metaspans exceed`, and
/// `span exceed` does not match inside `subspan exceed` (left-boundary rejects `prespan exceed`
/// and `respan exceed`);
/// `attributes exceed` does not match inside `microattributes exceed` / `metaattributes exceed`, and
/// `attribute exceed` does not match inside `subattribute exceed` (left-boundary rejects
/// `preattribute exceed` and `reattribute exceed`);
/// `links exceed` does not match inside `microlinks exceed` / `metalinks exceed`, and
/// `link exceed` does not match inside `sublink exceed` (left-boundary rejects `prelink exceed`
/// and `relink exceed`);
/// `scopes exceed` does not match inside `microscopes exceed` / `metascopes exceed`, and
/// `scope exceed` does not match inside `subscope exceed` (left-boundary rejects `prescope exceed`
/// and `rescope exceed`);
/// `resources exceed` does not match inside `microresources exceed` / `metaresources exceed`, and
/// `resource exceed` does not match inside `subresource exceed` (left-boundary rejects
/// `preresource exceed` and `reresource exceed`);
/// `metrics exceed` does not match inside `micrometrics exceed` / `metametrics exceed`, and
/// `metric exceed` does not match inside `submetric exceed` (left-boundary rejects
/// `premetric exceed` and `remetric exceed`);
/// `dimensions exceed` does not match inside `microdimensions exceed` / `metadimensions exceed`, and
/// `dimension exceed` does not match inside `subdimension exceed` (left-boundary rejects
/// `predimension exceed` and `redimension exceed`);
/// `tensors exceed` does not match inside `microtensors exceed` / `metatensors exceed`, and
/// `tensor exceed` does not match inside `subtensor exceed` (left-boundary rejects `pretensor exceed`
/// and `retensor exceed`);
/// `activations exceed` does not match inside `microactivations exceed` / `metaactivations exceed`, and
/// `activation exceed` does not match inside `subactivation exceed` (left-boundary rejects
/// `preactivation exceed` and `reactivation exceed`);
/// `gradients exceed` does not match inside `microgradients exceed` / `metagradients exceed`, and
/// `gradient exceed` does not match inside `subgradient exceed` (left-boundary rejects
/// `pregradient exceed` and `regradient exceed`);
/// `weights exceed` does not match inside `microweights exceed` / `metaweights exceed`, and
/// `weight exceed` does not match inside `subweight exceed` (left-boundary rejects `preweight exceed`
/// and `reweight exceed`);
/// `biases exceed` does not match inside `microbiases exceed` / `metabiases exceed`, and
/// `bias exceed` does not match inside `subbias exceed` (left-boundary rejects `prebias exceed`
/// and `rebias exceed`);
/// `layers exceed` does not match inside `microlayers exceed` / `metalayers exceed`, and
/// `layer exceed` does not match inside `sublayer exceed` (left-boundary rejects `prelayer exceed`
/// and `relayer exceed`);
/// `heads exceed` does not match inside `microheads exceed` / `metaheads exceed`, and
/// `head exceed` does not match inside `subhead exceed` (left-boundary rejects `prehead exceed`
/// and `rehead exceed`);
/// `positions exceed` does not match inside `micropositions exceed` / `metapositions exceed`, and
/// `position exceed` does not match inside `subposition exceed` (left-boundary rejects
/// `preposition exceed` and `reposition exceed`);
/// `embeddings exceed` does not match inside `microembeddings exceed` / `metaembeddings exceed`, and
/// `embedding exceed` does not match inside `subembedding exceed` (left-boundary rejects
/// `preembedding exceed` and `reembedding exceed`);
/// `logits exceed` does not match inside `micrologits exceed` / `metalogits exceed`, and
/// `logit exceed` does not match inside `sublogit exceed` (left-boundary rejects `prelogit exceed`
/// and `relogit exceed`);
/// `probabilities exceed` does not match inside `microprobabilities exceed` /
/// `metaprobabilities exceed`, and `probability exceed` does not match inside `subprobability exceed`
/// (left-boundary rejects `preprobability exceed` and `reprobability exceed`);
/// `logprobs exceed` does not match inside `micrologprobs exceed` / `metalogprobs exceed`, and
/// `logprob exceed` does not match inside `sublogprob exceed` (left-boundary rejects `prelogprob exceed`
/// and `relogprob exceed`);
/// `messages exceed` does not match inside `micromessages exceed` / `metamessages exceed`, and
/// `message exceed` does not match inside `submessage exceed` (left-boundary rejects `premessage exceed`
/// and `remessage exceed`);
/// `inputs exceed` does not match inside `microinputs exceed` / `metainputs exceed`, and
/// `input exceed` does not match inside `subinput exceed` (left-boundary rejects `preinput exceed`
/// and `reinput exceed`);
/// `outputs exceed` does not match inside `microoutputs exceed` / `metaoutputs exceed`, and
/// `output exceed` does not match inside `suboutput exceed` (left-boundary rejects `preoutput exceed`
/// and `reoutput exceed`);
/// `responses exceed` does not match inside `microresponses exceed` / `metaresponses exceed`, and
/// `response exceed` does not match inside `subresponse exceed` (left-boundary rejects `preresponse exceed`
/// and `reresponse exceed`);
/// `requests exceed` does not match inside `microrequests exceed` / `metarequests exceed`, and
/// `request exceed` does not match inside `subrequest exceed` (left-boundary rejects `prerequest exceed`);
/// `records exceed` does not match inside `microrecords exceed` / `metarecords exceed`, and
/// `record exceed` does not match inside `subrecord exceed` (left-boundary rejects `rerecord exceed`
/// and `prerecord exceed`);
/// `total prompt tokens exceed` does not match inside `micrototal prompt tokens exceed`;
/// `requested tokens exceed` does not match inside `microrequested tokens exceed`;
/// `total tokens exceed` does not match inside `micrototal tokens exceed` / `metatotal tokens exceed`;
/// `prompt tokens exceed` does not match inside `subprompt tokens exceed` / `preprompt tokens exceed`,
/// nor when the preceding ident token is a compound ending in `total` (e.g. `micrototal prompt tokens exceed`);
/// `input tokens exceed` does not match inside `subinput tokens exceed` / `preinput tokens exceed`;
/// plural `tokens exceed` / `tokens exceeded` skip matches whose preceding token is `prompt`, `input`,
/// or `total` (so `micrototal prompt tokens exceed` is not matched on the `tokens exceed` arm).
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

/// ASCII `[a-zA-Z0-9_]+` token immediately before `phrase_start` (skipping whitespace).
fn preceding_ascii_ident_token(haystack: &str, phrase_start: usize) -> Option<&str> {
    if phrase_start == 0 {
        return None;
    }
    let b = haystack.as_bytes();
    let mut end = phrase_start;
    while end > 0 && b[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    if end == 0 {
        return None;
    }
    let mut start = end;
    while start > 0 {
        let c = b[start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' {
            start -= 1;
        } else {
            break;
        }
    }
    if start == end {
        None
    } else {
        Some(&haystack[start..end])
    }
}

/// `prompt tokens exceed` with ident-boundary, skipping matches where the prior token is `*total` (not bare `total`).
fn contains_prompt_tokens_exceed_after_boundary_excluding_compound_total(haystack: &str) -> bool {
    const PHRASE: &str = "prompt tokens exceed";
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    for (i, _) in haystack.match_indices(PHRASE) {
        if i > 0 && haystack[..i].chars().next_back().is_some_and(ident_continue) {
            continue;
        }
        if preceding_ascii_ident_token(haystack, i).is_some_and(|t| t != "total" && t.ends_with("total"))
        {
            continue;
        }
        return true;
    }
    false
}

/// `tokens exceed` / `tokens exceeded` with ident-boundary; skips when the prior token is `prompt` / `input` /
/// `total` / `requested` or a longer ASCII ident ending with one of those roots (e.g. `microrequested`).
fn contains_tokens_exceed_subphrase_at_boundary(haystack: &str, phrase: &str) -> bool {
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    fn skip_prev_token_for_tokens_arm(t: &str) -> bool {
        ["prompt", "input", "total", "requested"]
            .iter()
            .any(|root| t.ends_with(*root))
    }
    for (i, _) in haystack.match_indices(phrase) {
        if i > 0 && haystack[..i].chars().next_back().is_some_and(ident_continue) {
            continue;
        }
        if preceding_ascii_ident_token(haystack, i).is_some_and(skip_prev_token_for_tokens_arm) {
            continue;
        }
        return true;
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
        // Ident-boundary (FEAT-D377): `micrototal prompt tokens exceed` does not embed
        // `total prompt tokens exceed` at a boundary; parallel to `inputs exceed` (FEAT-D375).
        || contains_phrase_after_ident_boundary(&lower, "total prompt tokens exceed")
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
        || contains_phrase_after_ident_boundary(&lower, "requested tokens exceed")
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
        || (contains_phrase_after_ident_boundary(&lower, "total tokens exceed")
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
        || ((contains_prompt_tokens_exceed_after_boundary_excluding_compound_total(&lower)
            || contains_phrase_after_ident_boundary(&lower, "input tokens exceed"))
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
        // Ident-boundary (FEAT-D358): `micromessages exceed` / `metamessages exceed` do not embed
        // `messages exceed` at a boundary; parallel to `requests exceed` (FEAT-D353).
        || ((contains_phrase_after_ident_boundary(&lower, "messages exceed")
            || contains_phrase_after_ident_boundary(&lower, "messages exceeded"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Singular "message exceed(s/ed)" (parallel to plural `messages exceed`, FEAT-D298).
        // `message exceed` matches present/past via `exceed` prefix of `exceeds` / `exceeded`.
        // Does not substring-match plural `messages exceed` (the `s` after `message` breaks
        // `message` + space + `exceed`). Ident-boundary (FEAT-D358): `submessage exceed` /
        // `micromessage exceeds` do not false-positive.
        || (contains_phrase_after_ident_boundary(&lower, "message exceed")
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
        // Ident-boundary (FEAT-D375): `microinputs exceed` / `metainputs exceed` do not embed
        // `inputs exceed` at a boundary; parallel to `messages exceed` (FEAT-D358).
        || ((contains_phrase_after_ident_boundary(&lower, "inputs exceed")
            || contains_phrase_after_ident_boundary(&lower, "inputs exceeded"))
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
        // Ident-boundary (FEAT-D375): `subinput exceed` / `preinput exceed` / `reinput exceed` likewise.
        || (contains_phrase_after_ident_boundary(&lower, "input exceed")
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
        // Plural / singular "output(s) exceed(s/ed)" (FEAT-D305; ident-boundary FEAT-D376). Parallel
        // to `inputs exceed` / `content exceed`. `output exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match `outputs exceed` (the `s`
        // after `output`). Ident-boundary so `microoutputs exceed` / `metaoutputs exceed` /
        // `suboutput exceed` do not false-positive; `preoutput exceed` and `reoutput exceed` are
        // rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "outputs exceed")
            || contains_phrase_after_ident_boundary(&lower, "outputs exceeded")
            || contains_phrase_after_ident_boundary(&lower, "output exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "response(s) exceed(s/ed)" (FEAT-D306; ident-boundary FEAT-D378). Parallel
        // to `outputs exceed` / `inputs exceed`. `response exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match `responses exceed` (the `s`
        // after `response`). Ident-boundary so `microresponses exceed` / `metaresponses exceed` /
        // `subresponse exceed` do not false-positive; `preresponse exceed` and `reresponse exceed` are
        // rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "responses exceed")
            || contains_phrase_after_ident_boundary(&lower, "responses exceeded")
            || contains_phrase_after_ident_boundary(&lower, "response exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "request(s) exceed(s/ed)" (FEAT-D307; ident-boundary FEAT-D353). Parallel
        // to `responses exceed` / `inputs exceed`. `request exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match `requests exceed` (the `s`
        // after `request`). Ident-boundary so `microrequests exceed` / `metarequests exceed` /
        // `subrequest exceed` do not false-positive; `prerequest exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "requests exceed")
            || contains_phrase_after_ident_boundary(&lower, "requests exceeded")
            || contains_phrase_after_ident_boundary(&lower, "request exceed"))
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
        // Plural / singular "token(s) exceed(s/ed)" (FEAT-D311; FEAT-D377: ident-boundary + skip when
        // the prior token is `prompt` / `input` / `total` so `micrototal prompt tokens exceed` is not
        // matched here via embedded `tokens exceed`). Parallel to `batches exceed` / `batch exceed`.
        || ((contains_tokens_exceed_subphrase_at_boundary(&lower, "tokens exceed")
            || contains_tokens_exceed_subphrase_at_boundary(&lower, "tokens exceeded")
            || contains_phrase_after_ident_boundary(&lower, "token exceed"))
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
        // Plural / singular "record(s) exceed(s/ed)" (FEAT-D314; ident-boundary FEAT-D351). Parallel
        // to `entries exceed` / `entry exceed`. `record exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match plural `records exceed`
        // (the `s` after `record`). Ident-boundary so `microrecords exceed` / `metarecords exceed` /
        // `subrecord exceed` do not false-positive; `rerecord exceed` / `prerecord exceed` are
        // rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "records exceed")
            || contains_phrase_after_ident_boundary(&lower, "records exceeded")
            || contains_phrase_after_ident_boundary(&lower, "record exceed"))
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
        // Plural / singular "part(s) exceed(s/ed)" (FEAT-D343). Parallel to `bodies exceed` /
        // `body exceed`. `part exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `parts exceed` (the `s` after `part`).
        // Ident-boundary so `microparts exceed` / `metaparts exceed` / `subpart exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "parts exceed")
            || contains_phrase_after_ident_boundary(&lower, "parts exceeded")
            || contains_phrase_after_ident_boundary(&lower, "part exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "piece(s) exceed(s/ed)" (FEAT-D344). Parallel to `parts exceed` /
        // `part exceed`. `piece exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `pieces exceed` (the `s` after `piece`).
        // Ident-boundary so `micropieces exceed` / `metapieces exceed` / `subpiece exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "pieces exceed")
            || contains_phrase_after_ident_boundary(&lower, "pieces exceeded")
            || contains_phrase_after_ident_boundary(&lower, "piece exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "shard(s) exceed(s/ed)" (FEAT-D345). Parallel to `pieces exceed` /
        // `piece exceed`. `shard exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `shards exceed` (the `s` after `shard`).
        // Ident-boundary so `microshards exceed` / `metashards exceed` / `subshard exceed` do not
        // false-positive; `reshard exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "shards exceed")
            || contains_phrase_after_ident_boundary(&lower, "shards exceeded")
            || contains_phrase_after_ident_boundary(&lower, "shard exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "fragment(s) exceed(s/ed)" (FEAT-D346). Parallel to `shards exceed` /
        // `shard exceed`. `fragment exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `fragments exceed` (the `s` after `fragment`).
        // Ident-boundary so `microfragments exceed` / `metafragments exceed` / `subfragment exceed` do not
        // false-positive; `refragment exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "fragments exceed")
            || contains_phrase_after_ident_boundary(&lower, "fragments exceeded")
            || contains_phrase_after_ident_boundary(&lower, "fragment exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "packet(s) exceed(s/ed)" (FEAT-D347). Parallel to `fragments exceed` /
        // `fragment exceed`. `packet exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `packets exceed` (the `s` after `packet`).
        // Ident-boundary so `micropackets exceed` / `metapackets exceed` / `subpacket exceed` do not
        // false-positive; `repacket exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "packets exceed")
            || contains_phrase_after_ident_boundary(&lower, "packets exceeded")
            || contains_phrase_after_ident_boundary(&lower, "packet exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "frame(s) exceed(s/ed)" (FEAT-D348). Parallel to `packets exceed` /
        // `packet exceed`. `frame exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `frames exceed` (the `s` after `frame`).
        // Ident-boundary so `microframes exceed` / `metaframes exceed` / `subframe exceed` do not
        // false-positive; `reframe exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "frames exceed")
            || contains_phrase_after_ident_boundary(&lower, "frames exceeded")
            || contains_phrase_after_ident_boundary(&lower, "frame exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "sample(s) exceed(s/ed)" (FEAT-D349). Parallel to `frames exceed` /
        // `frame exceed`. `sample exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `samples exceed` (the `s` after `sample`).
        // Ident-boundary so `microsamples exceed` / `metasamples exceed` / `subsample exceed` do not
        // false-positive; `resample exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "samples exceed")
            || contains_phrase_after_ident_boundary(&lower, "samples exceeded")
            || contains_phrase_after_ident_boundary(&lower, "sample exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "observation(s) exceed(s/ed)" (FEAT-D350). Parallel to `samples exceed` /
        // `sample exceed`. `observation exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `observations exceed` (the `s` after `observation`).
        // Ident-boundary so `microobservations exceed` / `metaobservations exceed` / `subobservation exceed` do not
        // false-positive; `preobservation exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "observations exceed")
            || contains_phrase_after_ident_boundary(&lower, "observations exceeded")
            || contains_phrase_after_ident_boundary(&lower, "observation exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "event(s) exceed(s/ed)" (FEAT-D352). Parallel to `observations exceed` /
        // `observation exceed`. `event exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `events exceed` (the `s` after `event`).
        // Ident-boundary so `microevents exceed` / `metaevents exceed` / `subevent exceed` do not
        // false-positive; `preevent exceed` is rejected the same way (and embedded `event exceed` in
        // `prevent exceed` is rejected by the left boundary).
        || ((contains_phrase_after_ident_boundary(&lower, "events exceed")
            || contains_phrase_after_ident_boundary(&lower, "events exceeded")
            || contains_phrase_after_ident_boundary(&lower, "event exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "trace(s) exceed(s/ed)" (FEAT-D354). Parallel to `events exceed` /
        // `event exceed`. `trace exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `traces exceed` (the `s` after `trace`).
        // Ident-boundary so `microtraces exceed` / `metatraces exceed` / `subtrace exceed` do not
        // false-positive; `pretrace exceed` and `retrace exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "traces exceed")
            || contains_phrase_after_ident_boundary(&lower, "traces exceeded")
            || contains_phrase_after_ident_boundary(&lower, "trace exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "span(s) exceed(s/ed)" (FEAT-D355). Parallel to `traces exceed` /
        // `trace exceed`. `span exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `spans exceed` (the `s` after `span`).
        // Ident-boundary so `microspans exceed` / `metaspans exceed` / `subspan exceed` do not
        // false-positive; `prespan exceed` and `respan exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "spans exceed")
            || contains_phrase_after_ident_boundary(&lower, "spans exceeded")
            || contains_phrase_after_ident_boundary(&lower, "span exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "attribute(s) exceed(s/ed)" (FEAT-D356). Parallel to `spans exceed` /
        // `span exceed`. `attribute exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `attributes exceed` (the `s` after
        // `attribute`). Ident-boundary so `microattributes exceed` / `metaattributes exceed` /
        // `subattribute exceed` do not false-positive; `preattribute exceed` and `reattribute exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "attributes exceed")
            || contains_phrase_after_ident_boundary(&lower, "attributes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "attribute exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "link(s) exceed(s/ed)" (FEAT-D357). Parallel to `attributes exceed` /
        // `attribute exceed`. `link exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `links exceed` (the `s` after `link`).
        // Ident-boundary so `microlinks exceed` / `metalinks exceed` / `sublink exceed` do not
        // false-positive; `prelink exceed` and `relink exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "links exceed")
            || contains_phrase_after_ident_boundary(&lower, "links exceeded")
            || contains_phrase_after_ident_boundary(&lower, "link exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "scope(s) exceed(s/ed)" (FEAT-D359). Parallel to `links exceed` /
        // `link exceed`. `scope exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `scopes exceed` (the `s` after `scope`).
        // Ident-boundary so `microscopes exceed` / `metascopes exceed` / `subscope exceed` do not
        // false-positive; `prescope exceed` and `rescope exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "scopes exceed")
            || contains_phrase_after_ident_boundary(&lower, "scopes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "scope exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "resource(s) exceed(s/ed)" (FEAT-D360). Parallel to `scopes exceed` /
        // `scope exceed`. `resource exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `resources exceed` (the `s` after `resource`).
        // Ident-boundary so `microresources exceed` / `metaresources exceed` / `subresource exceed`
        // do not false-positive; `preresource exceed` and `reresource exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "resources exceed")
            || contains_phrase_after_ident_boundary(&lower, "resources exceeded")
            || contains_phrase_after_ident_boundary(&lower, "resource exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "metric(s) exceed(s/ed)" (FEAT-D361). Parallel to `resources exceed` /
        // `resource exceed`. `metric exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `metrics exceed` (the `s` after `metric`).
        // Ident-boundary so `micrometrics exceed` / `metametrics exceed` / `submetric exceed`
        // do not false-positive; `premetric exceed` and `remetric exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "metrics exceed")
            || contains_phrase_after_ident_boundary(&lower, "metrics exceeded")
            || contains_phrase_after_ident_boundary(&lower, "metric exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "dimension(s) exceed(s/ed)" (FEAT-D362). Parallel to `metrics exceed` /
        // `metric exceed`. `dimension exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `dimensions exceed` (the `s` after `dimension`).
        // Ident-boundary so `microdimensions exceed` / `metadimensions exceed` / `subdimension exceed`
        // do not false-positive; `predimension exceed` and `redimension exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "dimensions exceed")
            || contains_phrase_after_ident_boundary(&lower, "dimensions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "dimension exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "tensor(s) exceed(s/ed)" (FEAT-D363). Parallel to `dimensions exceed` /
        // `dimension exceed`. `tensor exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `tensors exceed` (the `s` after `tensor`).
        // Ident-boundary so `microtensors exceed` / `metatensors exceed` / `subtensor exceed`
        // do not false-positive; `pretensor exceed` and `retensor exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "tensors exceed")
            || contains_phrase_after_ident_boundary(&lower, "tensors exceeded")
            || contains_phrase_after_ident_boundary(&lower, "tensor exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "activation(s) exceed(s/ed)" (FEAT-D364). Parallel to `tensors exceed` /
        // `tensor exceed`. `activation exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `activations exceed` (the `s` after `activation`).
        // Ident-boundary so `microactivations exceed` / `metaactivations exceed` / `subactivation exceed`
        // do not false-positive; `preactivation exceed` and `reactivation exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "activations exceed")
            || contains_phrase_after_ident_boundary(&lower, "activations exceeded")
            || contains_phrase_after_ident_boundary(&lower, "activation exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "gradient(s) exceed(s/ed)" (FEAT-D365). Parallel to `activations exceed` /
        // `activation exceed`. `gradient exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `gradients exceed` (the `s` after `gradient`).
        // Ident-boundary so `microgradients exceed` / `metagradients exceed` / `subgradient exceed`
        // do not false-positive; `pregradient exceed` and `regradient exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "gradients exceed")
            || contains_phrase_after_ident_boundary(&lower, "gradients exceeded")
            || contains_phrase_after_ident_boundary(&lower, "gradient exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "weight(s) exceed(s/ed)" (FEAT-D366). Parallel to `gradients exceed` /
        // `gradient exceed`. `weight exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `weights exceed` (the `s` after `weight`).
        // Ident-boundary so `microweights exceed` / `metaweights exceed` / `subweight exceed`
        // do not false-positive; `preweight exceed` and `reweight exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "weights exceed")
            || contains_phrase_after_ident_boundary(&lower, "weights exceeded")
            || contains_phrase_after_ident_boundary(&lower, "weight exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "bias(es) exceed(s/ed)" (FEAT-D367). Parallel to `weights exceed` /
        // `weight exceed`. `bias exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `biases exceed` (the `s` after `bias`).
        // Ident-boundary so `microbiases exceed` / `metabiases exceed` / `subbias exceed`
        // do not false-positive; `prebias exceed` and `rebias exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "biases exceed")
            || contains_phrase_after_ident_boundary(&lower, "biases exceeded")
            || contains_phrase_after_ident_boundary(&lower, "bias exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "layer(s) exceed(s/ed)" (FEAT-D368). Parallel to `biases exceed` /
        // `bias exceed`. `layer exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `layers exceed` (the `s` after `layer`).
        // Ident-boundary so `microlayers exceed` / `metalayers exceed` / `sublayer exceed`
        // do not false-positive; `prelayer exceed` and `relayer exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "layers exceed")
            || contains_phrase_after_ident_boundary(&lower, "layers exceeded")
            || contains_phrase_after_ident_boundary(&lower, "layer exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "head(s) exceed(s/ed)" (FEAT-D369). Parallel to `layers exceed` /
        // `layer exceed`. `head exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `heads exceed` (the `s` after `head`).
        // Ident-boundary so `microheads exceed` / `metaheads exceed` / `subhead exceed`
        // do not false-positive; `prehead exceed` and `rehead exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "heads exceed")
            || contains_phrase_after_ident_boundary(&lower, "heads exceeded")
            || contains_phrase_after_ident_boundary(&lower, "head exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "position(s) exceed(s/ed)" (FEAT-D370). Parallel to `heads exceed` /
        // `head exceed`. `position exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `positions exceed` (the `s` after
        // `position`). Ident-boundary so `micropositions exceed` / `metapositions exceed` /
        // `subposition exceed` do not false-positive; `preposition exceed` and `reposition exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "positions exceed")
            || contains_phrase_after_ident_boundary(&lower, "positions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "position exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "embedding(s) exceed(s/ed)" (FEAT-D371). Parallel to `positions exceed` /
        // `position exceed`. `embedding exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `embeddings exceed` (the `s` after
        // `embedding`). Ident-boundary so `microembeddings exceed` / `metaembeddings exceed` /
        // `subembedding exceed` do not false-positive; `preembedding exceed` and `reembedding exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "embeddings exceed")
            || contains_phrase_after_ident_boundary(&lower, "embeddings exceeded")
            || contains_phrase_after_ident_boundary(&lower, "embedding exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "logit(s) exceed(s/ed)" (FEAT-D372). Parallel to `embeddings exceed` /
        // `embedding exceed`. `logit exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `logits exceed` (the `s` after
        // `logit`). Ident-boundary so `micrologits exceed` / `metalogits exceed` /
        // `sublogit exceed` do not false-positive; `prelogit exceed` and `relogit exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "logits exceed")
            || contains_phrase_after_ident_boundary(&lower, "logits exceeded")
            || contains_phrase_after_ident_boundary(&lower, "logit exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "probabilit(y|ies) exceed(s/ed)" (FEAT-D373). Parallel to `logits exceed` /
        // `logit exceed`. `probability exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `probabilities exceed` (no `probability` + space
        // + `exceed` inside that spelling). Ident-boundary so
        // `microprobabilities exceed` / `metaprobabilities exceed` / `subprobability exceed` do not
        // false-positive; `preprobability exceed` and `reprobability exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "probabilities exceed")
            || contains_phrase_after_ident_boundary(&lower, "probabilities exceeded")
            || contains_phrase_after_ident_boundary(&lower, "probability exceed"))
            && (lower.contains("context window")
                || lower.contains("context length")
                || lower.contains("context limit")
                || lower.contains("context size")
                || lower.contains("max context")
                || lower.contains("maximum context")
                || lower.contains("available context")
                || lower.contains("model's context")))
        // Plural / singular "logprob(s) exceed(s/ed)" (FEAT-D374). Parallel to `probabilities exceed` /
        // `probability exceed`. `logprob exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `logprobs exceed` (the `s` after
        // `logprob`). Ident-boundary so `micrologprobs exceed` / `metalogprobs exceed` /
        // `sublogprob exceed` do not false-positive; `prelogprob exceed` and `relogprob exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "logprobs exceed")
            || contains_phrase_after_ident_boundary(&lower, "logprobs exceeded")
            || contains_phrase_after_ident_boundary(&lower, "logprob exceed"))
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
        assert!(is_context_overflow_error(
            "input exceeds the context window"
        ));
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
            "config: microrequests exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metarequests exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subrequest exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prerequest exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: micromessages exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamessages exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: submessage exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: premessage exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: remessage exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microinputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metainputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subinput exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preinput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: reinput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microoutputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaoutputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: suboutput exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preoutput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: reoutput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microresponses exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaresponses exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subresponse exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preresponse exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: reresponse exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: micrototal prompt tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: microrequested tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: micrototal tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatotal tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subprompt tokens exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preprompt tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subinput tokens exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preinput tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "http: request exceed max allowed size on this route (no model context configured)"
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
        assert!(is_context_overflow_error(
            "gateway: parts exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: parts exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: part exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: part exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: parts exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: parts exceeded max multipart count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: part exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microparts exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaparts exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subpart exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: pieces exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: pieces exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: piece exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: piece exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: pieces exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: pieces exceeded max puzzle count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: piece exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropieces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapieces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subpiece exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: shards exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: shards exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: shard exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: shard exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: shards exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: shards exceeded max replica count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: shard exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microshards exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metashards exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subshard exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: reshard exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: fragments exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: fragments exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: fragment exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: fragment exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: fragments exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: fragments exceeded max packet count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: fragment exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfragments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metafragments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subfragment exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: refragment exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: packets exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: packets exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: packet exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: packet exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: packets exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: packets exceeded max MTU count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: packet exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropackets exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapackets exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subpacket exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: repacket exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: frames exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: frames exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: frame exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: frame exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: frames exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: frames exceeded max GOP size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: frame exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microframes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaframes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subframe exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: reframe exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: samples exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: samples exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: sample exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: sample exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: samples exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: samples exceeded max batch size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: sample exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microsamples exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metasamples exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "dsp: subsample exceed decimation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: resample exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: observations exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: observations exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: observation exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: observation exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: observations exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: observations exceeded max batch size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: observation exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microobservations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaobservations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subobservation exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preobservation exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: events exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: events exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: event exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: event exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: events exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: events exceeded max batch size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: event exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microevents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaevents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subevent exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preevent exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "workflow: prevent exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: traces exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: traces exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: trace exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: trace exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: traces exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: traces exceeded max span count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: trace exceed max allowed depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microtraces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatraces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subtrace exceed span cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: pretrace exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: retrace exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: spans exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: spans exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: span exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: span exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: spans exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: spans exceeded max attribute count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: span exceed max allowed depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microspans exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaspans exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subspan exceed link cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prespan exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: respan exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: attributes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: attributes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: attribute exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: attribute exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: attributes exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: attributes exceeded max key count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: attribute exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microattributes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaattributes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subattribute exceed tag cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preattribute exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reattribute exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: links exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: links exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: link exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: link exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: links exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: links exceeded max per-span cap on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: link exceed max allowed count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microlinks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalinks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: sublink exceed span cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelink exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: relink exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: scopes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: scopes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: scope exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: scope exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: scopes exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: scopes exceeded max instrumentation cap on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: scope exceed max allowed depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microscopes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metascopes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subscope exceed instrumentation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prescope exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: rescope exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: resources exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: resources exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: resource exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: resource exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: resources exceed per-tenant rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: resources exceeded max attribute bytes on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: resource exceed max allowed labels on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microresources exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaresources exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subresource exceed descriptor cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preresource exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reresource exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: metrics exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: metrics exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: metric exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: metric exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: metrics exceed per-tenant rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: metrics exceeded max series cardinality on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: metric exceed max label count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micrometrics exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metametrics exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: submetric exceed descriptor cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: premetric exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: remetric exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: dimensions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: dimensions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: dimension exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: dimension exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: dimensions exceed per-request tensor rank limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: dimensions exceeded max axis count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: dimension exceed max embedding width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microdimensions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metadimensions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subdimension exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: predimension exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: redimension exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: tensors exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: tensors exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: tensor exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: tensor exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: tensors exceed per-request graph node limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: tensors exceeded max operand count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: tensor exceed max rank on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microtensors exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatensors exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subtensor exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: pretensor exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: retensor exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: activations exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: activations exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: activation exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: activation exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: activations exceed per-layer buffer limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: activations exceeded max feature map count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: activation exceed max channel width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microactivations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaactivations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subactivation exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preactivation exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reactivation exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: gradients exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: gradients exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: gradient exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: gradient exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: gradients exceed per-step clip limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: gradients exceeded max backward-pass depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: gradient exceed max Jacobian rows on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microgradients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metagradients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subgradient exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: pregradient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: regradient exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: weights exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: weights exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: weight exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: weight exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: weights exceed per-layer L2 clip limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: weights exceeded max trainable parameter count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: weight exceed max shard rows on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microweights exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaweights exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subweight exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preweight exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reweight exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: biases exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: biases exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: bias exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: bias exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: biases exceed per-layer offset limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: biases exceeded max initializer count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: bias exceed max row width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbiases exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabiases exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subbias exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prebias exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: rebias exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: layers exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: layers exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: layer exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: layer exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: layers exceed per-stack depth limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: layers exceeded max module count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: layer exceed max channel width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microlayers exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalayers exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: sublayer exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelayer exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: relayer exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: heads exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: heads exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: head exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: head exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: heads exceed per-attention fan-in limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: heads exceeded max parallel workers on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: head exceed max replica count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microheads exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaheads exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subhead exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prehead exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: rehead exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: positions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: positions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: position exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: position exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: positions exceed max sequence index for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: positions exceeded allowed KV slots on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: position exceed max slot width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropositions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapositions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "grammar: subposition exceed rule depth (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preposition exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: reposition exceed grid bounds (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: embeddings exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: embeddings exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: embedding exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: embedding exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: embeddings exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: embeddings exceeded max vector dimensions on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: embedding exceed max batch width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microembeddings exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaembeddings exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subembedding exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preembedding exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: reembedding exceed cache budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: logits exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: logits exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: logit exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: logit exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: logits exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: logits exceeded max vocabulary width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: logit exceed max batch dim on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micrologits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalogits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: sublogit exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelogit exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: relogit exceed cache budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: probabilities exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: probabilities exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: probability exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: probability exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: probabilities exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: probabilities exceeded max softmax width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: probability exceed max sampling dim on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microprobabilities exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaprobabilities exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subprobability exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preprobability exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: reprobability exceed cache budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: logprobs exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: logprobs exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: logprob exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: logprob exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: logprobs exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: logprobs exceeded max return-n on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: logprob exceed max top-k on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micrologprobs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalogprobs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: sublogprob exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelogprob exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: relogprob exceed cache budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microrecords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metarecords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subrecord exceed row cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: rerecord exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "pipeline: prerecord exceed the model's context window on this request"
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
        let msg =
            sanitize_ollama_error_for_user("error: prompt does not fit in the context window");
        let text = msg.expect("fit-in-context phrase should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_context_size_exceeded_phrase() {
        let msg =
            sanitize_ollama_error_for_user("llama runner: context size exceeded (n_ctx=8192)");
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
