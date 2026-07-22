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
            "gateway: bands exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: orbitals exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: basis functions exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: auxiliary functions exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: primitive gaussians exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: contracted gaussians exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: spherical gaussians exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: cartesian gaussians exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: gaussian shells exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: density matrices exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: molecular orbitals exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: atomic orbitals exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: wave functions exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: slater determinants exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: configuration state functions exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: csf coefficients exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: ci coefficients exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: mo coefficients exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: natural orbitals exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: occupied orbitals exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: canonical orbitals exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: electrons exceed available context on this request"
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
            "gateway: shard overflow relative to maximum context on this model"
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

    /// FEAT-D391: the `message`/`inputs` … `too long` + explicit-slot conjunct uses
    /// `contains_phrase_after_ident_boundary`, not substring `contains`, on each phrase.
    #[test]
    fn message_input_too_long_phrases_use_ident_boundary_on_conjunct() {
        let s1 = "fixture: submessage is too long (pipeline note)";
        let l1 = s1.to_lowercase();
        assert!(l1.contains("message is too long"));
        assert!(!contains_phrase_after_ident_boundary(
            &l1,
            "message is too long"
        ));

        let l2 = "micromessage is too long".to_lowercase();
        assert!(l2.contains("message is too long"));
        assert!(!contains_phrase_after_ident_boundary(
            &l2,
            "message is too long"
        ));

        let l3 = "metainputs are too long".to_lowercase();
        assert!(l3.contains("inputs are too long"));
        assert!(!contains_phrase_after_ident_boundary(
            &l3,
            "inputs are too long"
        ));

        assert!(!is_context_overflow_error(s1));
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
            "config: microqueries exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaqueries exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subquery exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prequery exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: requery exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "bioinfo: subsequence length exceeds alignment window (not llm context)"
        ));
        assert!(!is_context_overflow_error(
            "kernel: microexceeds maximum sequence length in fused op (not context)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromaximum sequence length exceeded in synthetic fixture"
        ));
        assert!(!is_context_overflow_error(
            "parser: microinput sequence is too long for amino-acid field (pipeline limit)"
        ));
        assert!(!is_context_overflow_error(
            "templating: microprompt length exceeds macro substitution depth (not context)"
        ));
        assert!(!is_context_overflow_error(
            "forms: microprompt too long for field schema (pipeline limit)"
        ));
        assert!(!is_context_overflow_error(
            "macros: microprompt is too long for expansion buffer (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "templates: microprompt has more tokens than the lexer allows (not model window)"
        ));
        assert!(!is_context_overflow_error(
            "ui: microprompt exceeds the context menu width (layout)"
        ));
        assert!(!is_context_overflow_error(
            "metrics: microcontext overflow counter reset (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "parser: microcontext length exceeded in field width (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromaximum context length in synthetic fixture (not model)"
        ));
        assert!(!is_context_overflow_error(
            "fixture: microexceeds the model's context window in stress test (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microexceeds the model context in batch column (telemetry)"
        ));
        assert!(!is_context_overflow_error(
            "log: microexceeded the model context in rollup row (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "cli: microexceed the model context is not a valid flag (typo)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromodel context exceeded in fixture name (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromodel context limit was exceeded in synthetic metric (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "fixture: microexceeds the context window in UI layout test (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microcontext window exceeded in accessibility tree (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "billing: microexceeded the context limit is not an overflow message"
        ));
        assert!(!is_context_overflow_error(
            "billing: microexceeds the context limit is not an overflow message"
        ));
        assert!(!is_context_overflow_error(
            "billing: microexceed the context limit is not an overflow message"
        ));
        assert!(!is_context_overflow_error(
            "lint: microrequested more tokens than column width in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "ui: microfit in the context panel is a layout hint (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microlarger than the context sidebar (not model window)"
        ));
        assert!(!is_context_overflow_error(
            "a11y: microoutside the context window in devtools tree (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "a11y: microoutside of the context window in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microbeyond the context window in CSS var name (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microbeyond the context panel width (not model window)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microover the context window in test label (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "css: microbeyond context window token in stylesheet (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "css: microover context window in shorthand (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microoutside context window in selector (not model)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microoutside of context window in snapshot (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "metrics: microran out of context handles (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microrunning out of context switches in parser (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microcontext size exceeded column width in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microexceeded context size metric is a dashboard label (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "ui: microexceeds available context panel width (layout)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microcontext limit exceeded in CSS var (not model)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microexceeds context length in flex line (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "gpu: microinsufficient context for draw call (not llm window)"
        ));
        assert!(!is_context_overflow_error(
            "layout: micronot enough context in grid track (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "ui: micropast the context sidebar edge (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microreached the context limit in test harness (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "metrics: microhit the context limit counter label (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microover the context limit line in diagram (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "parser: microcontext exhausted the small-string buffer (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "ui: request too long for microcontext label (layout)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microconversation is too long for variable name (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "log: truncated microcontext window in debug view (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "gpu: kv cache is full but microcontext binding failed (not llm window)"
        ));
        assert!(!is_context_overflow_error(
            "layout: prefill microcontext panel exceeds width (not model)"
        ));
        assert!(!is_context_overflow_error(
            "scheduler: microinsufficient remaining context switches (os)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microexceeds the context size hint in UI mock (not model)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microcontext size exceeds column header in sheet (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "log: microexceeded the context size in rollup (telemetry)"
        ));
        assert!(!is_context_overflow_error(
            "api: microcontext token limit exceeded rate (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microexceeds the context token limit label in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "log: microexceeded the context token limit in column name (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microexceeds this model's context field in schema (not window)"
        ));
        assert!(!is_context_overflow_error(
            "fixture: microexceeded this model's context row in CSV (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "cli: microexceed this model's context flag typo (not overflow)"
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
            "layout: messages exceed UI bounds (microcontext window width)"
        ));
        assert!(!is_context_overflow_error(
            "billing: total tokens exceed microcontext row counter (telemetry)"
        ));
        assert!(!is_context_overflow_error(
            "API: prompt tokens exceed microwindow title length (not llm)"
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
            "config: microcontents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacontents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subcontent exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: precontent exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: recontent exceed the model's context window on this request"
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
            "config: microcalls exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabatches exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subcall exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: precall exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: recall exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microitems exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaentries exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subentry exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microchunks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metadocuments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subdocument exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfiles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metelines exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subline exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromaximum context exceeded in synthetic metric (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromax context row in dashboard grid (not llm window)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microcontext length limit exceeded in column name (telemetry)"
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
            "API: strings exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: strings exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: string exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: string exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: strings exceed per-field length limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: strings exceeded daily ingest cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "codec: string exceed max UTF-8 width on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "parser: substring exceed lexer token budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microstrings exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metastrings exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: arrays exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: arrays exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: array exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: array exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: arrays exceed per-batch element limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: arrays exceeded daily request cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: array exceed max nesting depth on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microarrays exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaarrays exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subarray exceed lexer recursion budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: objects exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: objects exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: object exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: object exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: objects exceed per-request reference limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: objects exceeded daily create cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: object exceed max property count on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microobjects exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaobjects exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subobject exceed deserialization budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: elements exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: elements exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: element exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: element exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: elements exceed per-batch cardinality limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: elements exceeded daily ingest cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: element exceed max tensor rank on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microelements exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaelements exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subelement exceed nesting budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: nodes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: nodes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: node exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: node exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: nodes exceed per-cluster membership limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: nodes exceeded daily provision cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: node exceed max depth on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micronodes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metanodes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subnode exceed traversal budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: edges exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: edges exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: edge exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: edge exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: edges exceed per-graph fan-out limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: edges exceeded daily relationship cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: edge exceed max adjacency on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microedges exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaedges exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subedge exceed traversal budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "wedge exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: vertices exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: vertices exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: vertex exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: vertex exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: vertices exceed per-mesh topology limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: vertices exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: vertex exceed max valence on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvertices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavertices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvertex exceed fan-out budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervertex exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: faces exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: faces exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: face exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: face exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: faces exceed per-mesh polygon limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: faces exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: face exceed max facet count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfaces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metafaces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subface exceed facet budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "surface exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: triangles exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: triangles exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: triangle exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: triangle exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: triangles exceed per-mesh primitive limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: triangles exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: triangle exceed max strip count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microtriangles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatriangles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subtriangle exceed strip budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supertriangle exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: polygons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: polygons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: polygon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: polygon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: polygons exceed per-layer ring limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: polygons exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: polygon exceed max hole count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropolygons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapolygons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subpolygon exceed ring budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superpolygon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: meshes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: meshes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: mesh exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: mesh exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: meshes exceed per-draw mesh limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: meshes exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: mesh exceed max submesh count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromeshes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metameshes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submesh exceed LOD budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermesh exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: voxels exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: voxels exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: voxel exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: voxel exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: voxels exceed per-chunk grid limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: voxels exceeded daily volume cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: voxel exceed max brick count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvoxels exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavoxels exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvoxel exceed LOD budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervoxel exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: particles exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: particles exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: particle exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: particle exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: particles exceed per-emitter rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: particles exceeded daily simulation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: particle exceed max burst count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microparticles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaparticles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subparticle exceed LOD budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superparticle exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: molecules exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: molecules exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: molecule exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: molecule exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: molecules exceed per-reaction rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: molecules exceeded daily structure cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: molecule exceed max bond count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromolecules exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamolecules exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submolecule exceed valence budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermolecule exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: atoms exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: atoms exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: atom exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: atom exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: atoms exceed per-basis rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: atoms exceeded daily structure cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: atom exceed max valence on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microatoms exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaatoms exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subatom exceed shell budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superatom exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: ions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: ions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: ion exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: ion exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: ions exceed per-species rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: ions exceeded daily plasma cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: ion exceed max charge on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subion exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superion exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: million exceed per-client rate limits for this endpoint"
        ));
        assert!(is_context_overflow_error(
            "API: electrons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: electrons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: electron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: electron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: electrons exceed per-beam rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: electrons exceeded daily cathode cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: electron exceed max spin slots on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microelectrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaelectrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subelectron exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superelectron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: protons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: protons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: proton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: proton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: protons exceed per-beam rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: protons exceeded daily nucleus cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: proton exceed max charge states on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microprotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaprotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subproton exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superproton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: neutrons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: neutrons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: neutron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: neutron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: neutrons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: neutrons exceeded daily cross-section cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: neutron exceed max scattering channels on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microneutrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaneutrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subneutron exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superneutron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: quarks exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: quarks exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: quark exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: quark exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: quarks exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: quarks exceeded daily flavor cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: quark exceed max Wilson lines on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microquarks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaquarks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subquark exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superquark exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: gluons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: gluons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: gluon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: gluon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: gluons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: gluons exceeded daily color-octet cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: gluon exceed max gauge links on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microgluons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metagluons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subgluon exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supergluon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: bosons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bosons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: boson exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: boson exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bosons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bosons exceeded daily mode-occupancy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: boson exceed max virtual lines on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbosons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabosons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subboson exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superboson exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: leptons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: leptons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: lepton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: lepton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: leptons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: leptons exceeded daily family cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: lepton exceed max weak vertices on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microleptons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaleptons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: sublepton exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superlepton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: hadrons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: hadrons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: hadron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: hadron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: hadrons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: hadrons exceeded daily jet-multiplicity cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: hadron exceed max fragmentation on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microhadrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metahadrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subhadron exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superhadron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: photons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: photons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: photon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: photon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: photons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: photons exceeded daily fluence cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: photon exceed max beam occupancy on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microphotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaphotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subphoton exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superphoton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: phonons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: phonons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: phonon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: phonon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: phonons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: phonons exceeded daily branch-occupancy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: phonon exceed max mode occupancy on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microphonons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaphonons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subphonon exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superphonon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: excitons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: excitons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: exciton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: exciton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: excitons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: excitons exceeded daily well-occupancy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: exciton exceed max pair density on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microexcitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaexcitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subexciton exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superexciton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: polarons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: polarons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: polaron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: polaron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: polarons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: polarons exceeded daily lattice-deformation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: polaron exceed max coupling budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropolarons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapolarons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subpolaron exceed deformation budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superpolaron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: plasmons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: plasmons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: plasmon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: plasmon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: plasmons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: plasmons exceeded daily near-field mode cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: plasmon exceed max dispersion budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microplasmons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaplasmons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subplasmon exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superplasmon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: solitons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: solitons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: soliton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: soliton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: solitons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: solitons exceeded daily nonlinear-mode cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: soliton exceed max pulse budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microsolitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metasolitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subsoliton exceed dispersion budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supersoliton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: instantons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: instantons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: instanton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: instanton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: instantons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: instantons exceeded daily gauge-orbit cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: instanton exceed max tunneling budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microinstantons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metainstantons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subinstanton exceed sector budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superinstanton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: skyrmions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: skyrmions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: skyrmion exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: skyrmion exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: skyrmions exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: skyrmions exceeded daily track cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: skyrmion exceed max topological-charge budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microskyrmions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaskyrmions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subskyrmion exceed winding budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superskyrmion exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: magnons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: magnons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: magnon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: magnon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: magnons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: magnons exceeded daily spin-wave cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: magnon exceed max k-space budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromagnons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamagnons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submagnon exceed branch budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermagnon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: rotons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: rotons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: roton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: roton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: rotons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: rotons exceeded daily Landau-level cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: roton exceed max dispersion budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microrotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metarotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subroton exceed branch budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superroton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: anyons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: anyons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: anyon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: anyon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: anyons exceed per-braid rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: anyons exceeded daily fusion-channel cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: anyon exceed max braiding budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microanyons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaanyons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subanyon exceed charge budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superanyon exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "geology: canyon exceed maximum depth on this survey (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: fluxons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: fluxons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: fluxon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: fluxon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: fluxons exceed per-array rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: fluxons exceeded daily SQUID cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: fluxon exceed max shunt budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfluxons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metafluxons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subfluxon exceed bias budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superfluxon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: vortices exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: vortices exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: vortex exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: vortex exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: vortices exceed per-mesh rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: vortices exceeded daily circulation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: vortex exceed max swirl budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvortices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavortices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvortex exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervortex exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: disclinations exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: disclinations exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: disclination exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: disclination exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: disclinations exceed per-domain rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: disclinations exceeded daily Frank cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: disclination exceed max line-tension budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microdisclinations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metadisclinations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subdisclination exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superdisclination exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: dislocations exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: dislocations exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: dislocation exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: dislocation exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: dislocations exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: dislocations exceeded daily slip-system cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: dislocation exceed max Burgers-vector budget on this crystal field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microdislocations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metadislocations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subdislocation exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superdislocation exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: vacancies exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: vacancies exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: vacancy exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: vacancy exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: vacancies exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: vacancies exceeded daily formation-energy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: vacancy exceed max lattice-site budget on this crystal field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvacancies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavacancies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvacancy exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervacancy exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: interstitials exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: interstitials exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: interstitial exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: interstitial exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: interstitials exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: interstitials exceeded daily migration-barrier cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: interstitial exceed max defect-site budget on this crystal field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microinterstitials exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metainterstitials exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subinterstitial exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superinterstitial exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: voids exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: voids exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: void exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: void exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: voids exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: voids exceeded daily free-volume cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: void exceed max cavity budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvoids exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavoids exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvoid exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervoid exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: pores exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: pores exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: pore exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: pore exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: pores exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: pores exceeded daily porosity cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: pore exceed max throat budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropores exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapores exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subpore exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superpore exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "spore exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: inclusions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: inclusions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: inclusion exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: inclusion exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: inclusions exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: inclusions exceeded daily second-phase cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: inclusion exceed max particle budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microinclusions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metainclusions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subinclusion exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superinclusion exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reinclusion exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: clusters exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: clusters exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: cluster exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: cluster exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: clusters exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: clusters exceeded daily linkage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: cluster exceed max graph budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microclusters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaclusters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subcluster exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supercluster exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "recluster exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: grains exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: grains exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: grain exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: grain exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: grains exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: grains exceeded daily grain-boundary cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: grain exceed max crystallite budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micrograins exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metagrains exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subgrain exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supergrain exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "regrain exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: phases exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: phases exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: phase exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: phase exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: phases exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: phases exceeded daily coexistence cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: phase exceed max phase-fraction budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microphases exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaphases exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subphase exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superphase exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "prephase exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "rephase exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: crystals exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: crystals exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: crystal exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: crystal exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: crystals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: crystals exceeded daily unit-cell cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: crystal exceed max lattice budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcrystals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacrystals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subcrystal exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supercrystal exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "precrystal exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "recrystal exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: unit cells exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: unit cells exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: unit cell exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: unit cell exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: unit cells exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: unit cells exceeded daily Bravais-lattice cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: unit cell exceed max basis budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microunitcells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaunitcells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subunitcell exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superunitcell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preunitcell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reunitcell exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: primitive cells exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: primitive cells exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: primitive cell exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: primitive cell exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: primitive cells exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: primitive cells exceeded daily Wigner–Seitz cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: primitive cell exceed max basis budget on this mesh field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microprimitivecells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaprimitivecells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subprimitivecell exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superprimitivecell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preprimitivecell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reprimitivecell exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: supercells exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: supercells exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: supercell exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: supercell exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: supercells exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: supercells exceeded daily k-point mesh cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: supercell exceed max replica budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microsupercells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metasupercells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subsupercell exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supersupercell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "presupercell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "resupercell exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: k-points exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: k-points exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: k-point exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: k-point exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: k-points exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: k-points exceeded Monkhorst–Pack mesh cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: k-point exceed max irreducible-zone budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microk-points exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metak-points exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subk-point exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superk-point exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "prek-point exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "rek-point exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: q-points exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: q-points exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: q-point exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: q-point exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: q-points exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: q-points exceeded phonon-branch mesh cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: q-point exceed max Brillouin-zone q-path budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microq-points exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaq-points exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subq-point exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superq-point exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preq-point exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "req-point exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: bands exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bands exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: band exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: band exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bands exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bands exceeded daily empty-state cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: band exceed max k-path band budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbands exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabands exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subband exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superband exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preband exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reband exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: orbitals exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: orbitals exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: orbital exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: orbital exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: orbitals exceeded active-space cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: orbital exceed max Gaussian-type basis budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: suborbital exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reorbital exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: basis functions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: basis functions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: basis function exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: basis function exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: basis functions exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: basis functions exceeded auxiliary-basis cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: basis function exceed max contracted-Gaussian budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbasis functions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabasis functions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subbasis function exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superbasis function exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "prebasis function exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "rebasis function exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: auxiliary functions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: auxiliary functions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: auxiliary function exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: auxiliary function exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: auxiliary functions exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: auxiliary functions exceeded RI-JK shell cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: auxiliary function exceed max density-fitting budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microauxiliary functions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaauxiliary functions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subauxiliary function exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superauxiliary function exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preauxiliary function exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reauxiliary function exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: primitive gaussians exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: primitive gaussians exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: primitive gaussian exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: primitive gaussian exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: primitive gaussians exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: primitive gaussians exceeded PGF shell cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: primitive gaussian exceed max contracted-primitive budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microprimitive gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaprimitive gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subprimitive gaussian exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superprimitive gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preprimitive gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reprimitive gaussian exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: contracted gaussians exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: contracted gaussians exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: contracted gaussian exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: contracted gaussian exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: contracted gaussians exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: contracted gaussians exceeded CGF shell cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: contracted gaussian exceed max contraction-depth budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcontracted gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacontracted gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subcontracted gaussian exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supercontracted gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "precontracted gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "recontracted gaussian exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: spherical gaussians exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: spherical gaussians exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: spherical gaussian exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: spherical gaussian exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: spherical gaussians exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: spherical gaussians exceeded SGF shell cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: spherical gaussian exceed max angular-momentum budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microspherical gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaspherical gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subspherical gaussian exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superspherical gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "prespherical gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "respherical gaussian exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: cartesian gaussians exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: cartesian gaussians exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: cartesian gaussian exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: cartesian gaussian exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: cartesian gaussians exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: cartesian gaussians exceeded Cartesian-GTO center cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: cartesian gaussian exceed max exponent budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcartesian gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacartesian gaussians exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subcartesian gaussian exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supercartesian gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "precartesian gaussian exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "recartesian gaussian exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: gaussian shells exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: gaussian shells exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: gaussian shell exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: gaussian shell exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: gaussian shells exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: gaussian shells exceeded per-L / shell-block cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: gaussian shell exceed max contraction budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microgaussian shells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metagaussian shells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subgaussian shell exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supergaussian shell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "pregaussian shell exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "regaussian shell exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: density matrices exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: density matrices exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: density matrix exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: density matrix exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: density matrices exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: density matrices exceeded per-orbital / N-representability cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: density matrix exceed max rank budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microdensity matrices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metadensity matrices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subdensity matrix exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superdensity matrix exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "predensity matrix exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "redensity matrix exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: molecular orbitals exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: molecular orbitals exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: molecular orbital exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: molecular orbital exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: molecular orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: molecular orbitals exceeded active-space / MO-basis cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: molecular orbital exceed max active-space budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromolecularorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamolecularorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submolecular orbital exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermolecularorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "premolecularorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "remolecularorbital exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: atomic orbitals exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: atomic orbitals exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: atomic orbital exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: atomic orbital exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: atomic orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: atomic orbitals exceeded AO-basis / valence-shell cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: atomic orbital exceed max valence budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microatomicorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaatomicorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subatomic orbital exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superatomicorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preatomicorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reatomicorbital exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: wave functions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: wave functions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: wave function exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: wave function exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: wave functions exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: wave functions exceeded CI-vector / orbital-basis cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: wave function exceed max CI expansion budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microwavefunctions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metawavefunctions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subwave function exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superwavefunction exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "prewavefunction exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "rewavefunction exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: slater determinants exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: slater determinants exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: slater determinant exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: slater determinant exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: slater determinants exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: slater determinants exceeded FCI / active-space cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: slater determinant exceed max expansion budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microslaterdeterminants exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaslaterdeterminants exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subslater determinant exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superslaterdeterminant exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preslaterdeterminant exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reslaterdeterminant exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: configuration state functions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: configuration state functions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: configuration state function exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: configuration state function exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: configuration state functions exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: configuration state functions exceeded active-space / CSF-count cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: configuration state function exceed max CSF budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microconfigurationstatefunctions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaconfigurationstatefunctions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subconfiguration state function exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superconfigurationstatefunction exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preconfiguration state function exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reconfiguration state function exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: csf coefficients exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: csf coefficients exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: csf coefficient exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: csf coefficient exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: csf coefficients exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: csf coefficients exceeded CI expansion / active-space cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: csf coefficient exceed max variational budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcsfcoefficients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacsfcoefficients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subcsf coefficient exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supercsfcoefficient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "precsf coefficient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "recsf coefficient exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: ci coefficients exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: ci coefficients exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: ci coefficient exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: ci coefficient exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: ci coefficients exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: ci coefficients exceeded CI-vector / expansion cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: ci coefficient exceed max variational budget on this field (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: mo coefficients exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: mo coefficients exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: mo coefficient exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: mo coefficient exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: mo coefficients exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: mo coefficients exceeded LCAO / MO-basis cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: mo coefficient exceed max variational budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromocoefficients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamocoefficients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submo coefficient exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermocoefficient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "premo coefficient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "remo coefficient exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: natural orbitals exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: natural orbitals exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: natural orbital exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: natural orbital exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: natural orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: natural orbitals exceeded NO-truncation / occupation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: natural orbital exceed max variational budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micronaturalorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metanaturalorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subnatural orbital exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supernaturalorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "prenaturalorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "renaturalorbital exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: occupied orbitals exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: occupied orbitals exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: occupied orbital exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: occupied orbital exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: occupied orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: occupied orbitals exceeded frozen-core / active-space cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: occupied orbital exceed max variational budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microoccupiedorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaoccupiedorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: suboccupied orbital exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: preoccupied orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: unoccupied orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(is_context_overflow_error(
            "API: canonical orbitals exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: canonical orbitals exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: canonical orbital exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: canonical orbital exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: canonical orbitals exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: canonical orbitals exceeded CASSCF / active-space cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: canonical orbital exceed max variational budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcanonicalorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacanonicalorbitals exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subcanonical orbital exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supercanonicalorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "precanonicalorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "recanonicalorbital exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microcicoefficients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacicoefficients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subci coefficient exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supercicoefficient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preci coefficient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reci coefficient exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: electrons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: electrons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: electron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: electron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: electrons exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: electrons exceeded shell-occupancy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: electron exceed max valence budget on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microelectrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaelectrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subelectron exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superelectron exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "preelectron exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "reelectron exceed the model's context window on this request"
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
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
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
    fn proactive_impl_compacts_oldest_huge_message_first() {
        let mut msgs = vec![
            make_msg("system", "sys"),
            make_msg("user", &"u".repeat(20_000)),
        ];
        let n =
            proactively_compact_tool_results_for_context_budget_impl(&mut msgs, 1024, 0.12, 4096);
        assert!(n > 0, "expected at least one compaction step");
        assert!(
            msgs[1]
                .content
                .contains("compacted proactively for context budget"),
            "marker: {}",
            &msgs[1].content[msgs[1].content.len().saturating_sub(120)..]
        );
        assert!(msgs[1].content.chars().count() < 20_000);
    }

    #[test]
    fn proactive_impl_phase_b_shaves_when_each_under_max_chars() {
        let chunk = "word ".repeat(800);
        let mut msgs = vec![make_msg("system", "p")];
        for _ in 0..12 {
            msgs.push(make_msg("user", &chunk));
        }
        let n =
            proactively_compact_tool_results_for_context_budget_impl(&mut msgs, 2048, 0.12, 8192);
        assert!(n > 0, "expected phase-B shrink steps, got {}", n);
        assert!(msgs[1].content.contains("compacted proactively"));
    }

    #[test]
    fn overflow_retry_marker_distinct_from_proactive() {
        let mut msgs = vec![make_msg("system", "p"), make_msg("user", &"z".repeat(6000))];
        let n = truncate_oversized_tool_results(&mut msgs, 1000);
        assert_eq!(n, 1);
        assert!(msgs[1].content.contains("due to context limit"));
        assert!(!msgs[1].content.contains("compacted proactively"));
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
