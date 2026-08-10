# task-008: Overnight plan — request isolation, search hardening, and compaction safety

**Goal:** Eliminate context bleed between unrelated requests, make web/news answers consistently source-aware, and harden compaction/retry behavior so Discord/Ollama runs stay correct under long sessions and verification retries.

**Why this matters:**
- We already saw a real failure: a Discord request about Barcelona news correctly used Perplexity, but the verification retry mixed in old Redmine-style success criteria and returned the wrong kind of answer.
- Search retrieval is working, but result shaping and answer constraints are too weak, so verification rejects otherwise good answers for missing dates/sources.
- Session compaction and retry logic currently share too much ambient state, which increases the chance of cross-topic contamination.

## Phase 1: Request-local execution state ✅ Done

**Objective:** Keep each Discord/Ollama run self-contained so retries cannot inherit stale criteria, stale tool payloads, or stale task context.

**Implement:**
1. ✅ Introduce a request-local execution context struct for `answer_with_ollama_and_fetch` and/or the Discord call site. It should hold:
   - request id
   - original user question
   - trigger/channel/user metadata
   - extracted success criteria
   - topic classification result
   - retry count
   - local verification notes
2. ✅ Ensure verification retries reuse only request-local state, not ambient session memory.
3. ✅ If topic classification returns `NEW_TOPIC`, do not hydrate prior success-criteria-like context into the retry path.
4. ✅ Thread the request id through logs so one request can be followed end-to-end.

**Relevant code:**
- `src-tauri/src/commands/ollama.rs` — `RequestRunContext`, `request_id_override`, `retry_count`; retry path passes same request_id and request-local criteria.
- `src-tauri/src/discord/mod.rs` — call site passes `None, 0` for request_id_override and retry_count.

**Acceptance:**
- A retry for a Barcelona/news/search request cannot pick up Redmine/ticket/attachment criteria from a prior request in the same Discord session.
- Logs clearly show which criteria and retry notes belong to which request id.

## Phase 2: Separate conversation memory from execution artifacts ✅ Done

**Objective:** Persist chat history without polluting future turns with internal execution machinery.

**Implement:**
1. ✅ Define which messages are normal conversation and which are internal artifacts:
   - normal: user turns, final assistant replies
   - internal: completion-verifier prompts, criteria extraction prompts, large tool dumps, internal correction prompts, raw tool-return wrappers
2. ✅ Prevent internal artifacts from being written into normal Discord session memory: `session_memory.rs` — `is_internal_artifact()` filters by known patterns; `add_message()` skips persisting when content is internal; `get_messages()` and `parse_session_file()` exclude internal messages when loading; `replace_session()` filters compacted messages.
3. If some internal artifacts must be retained, store them in a separate internal lane or filter them out before reloading prior messages. (Current choice: filter only; no separate lane.)
4. ✅ Audit the code paths that call `crate::session_memory::add_message(...)` and session reload helpers — only `discord/mod.rs` calls `add_message`; session memory now filters on both write and read.

**Relevant code:**
- `src-tauri/src/session_memory.rs` — `is_internal_artifact()`, used in `add_message`, `get_messages`, `parse_session_file`, `replace_session`
- `src-tauri/src/discord/mod.rs` (call sites unchanged; filtering in session_memory)
- `src-tauri/src/commands/ollama.rs` (no add_message calls; uses replace_session which now filters)

**Acceptance:**
- After a request with verification retries, the next unrelated user message sees only prior conversational context, not verifier/tool meta-prompts.
- Session files remain useful for continuity but do not amplify retries or mixed-topic failure.

## Phase 3: Search result shaping for Perplexity and Brave ✅ Done

**Objective:** Make search outputs smaller, more structured, and easier for the model to turn into sourced answers.

**Implement:**
1. ✅ Keep the current source/date formatting improvements for Perplexity.
2. ✅ Add a shared result-shaping helper for web search tools that:
   - preserves `title`, `url`, `date`
   - truncates long snippets
   - deduplicates near-identical headlines or domains
   - caps result count
   - uses head+tail truncation if a formatted result blob is still too large
3. ✅ Apply the same shaping pattern to `BRAVE_SEARCH`.
4. Optionally add a recency filter for Perplexity when the user asks for `news`, `latest`, `recent`, `today`, or `this week`.

**Relevant code:**
- `src-tauri/src/search_result_shaping.rs` — shared `ShapableSearchResult`, `shape_search_results()`, `format_search_results_blob()` (head+tail).
- `src-tauri/src/commands/brave.rs` — parses API into `ShapableSearchResult`, shapes (280 chars/snippet, 10 results, 2/domain), formats with 12k blob cap; includes Brave `age` as date when present.
- `src-tauri/src/commands/ollama.rs`, `commands/perplexity.rs`, `perplexity/mod.rs` — Perplexity keeps existing news-specific shaping in ollama.

**Acceptance:**
- Search tools produce compact, source-rich result blobs with deterministic size limits.
- News requests consistently include enough metadata for the model to cite sources and dates.

## Phase 4: News-aware answering and verification ✅ Done

**Objective:** Align answer formatting with what verification expects.

**Implement:**
1. ✅ Detect news/current-events style requests: `is_news_query` expanded with "today", "this week".
2. ✅ Execution system gets `news_format_reminder` (short bullet list, 2+ named sources, dates when available, concise OK).
3. ✅ Success criteria override for news; `verification_news_format_note`: accept concise/bullet, require 2+ sources and dates when available.
4. ✅ On retry: existing narrow news hint (sources/dates, 3 bullets, no generic homepages). Original: On retry, pass narrow corrective instruction (e.g. add 2 sources and dates).

**Relevant code:**
- `src-tauri/src/commands/ollama.rs` — `is_news_query`, success_criteria, `news_format_reminder`, `verification_news_format_note`.

**Acceptance:**
- A Barcelona/news query yields a sourced answer on the first pass or a narrow corrective retry.
- Verification failures become specific and local, not cross-topic.

## Phase 5: Compaction hardening ✅ Done

**Objective:** Make session compaction safer and less likely to damage active work.

**Implement:**
1. ✅ Protect:
   - first system/task instructions — compactor prompt now instructs to preserve their gist and the most recent assistant/tool outcome
   - most recent user turn — already preserved in on-request path (compacted + current question)
   - most recent assistant/tool outcome — prompt: "PRESERVE in CONTEXT (2) the most recent assistant reply or tool outcome"
2. ✅ Skip compaction when a session contains no real conversational value or only synthetic/internal entries: `session_memory::count_conversational_messages`, `MIN_CONVERSATIONAL_FOR_COMPACTION` (2); skip in both on-request and periodic compaction with clear logs.
3. ✅ Improve compaction prompt guidance so active work, open decisions, and concrete results survive summarization (preserve open decisions and concrete results; preserve first system/task and latest outcome).
4. ✅ If compaction fails, degrade safely: keep full history; log "compaction skipped" (info) vs "compaction failed" (warn) with reason; periodic job does not retry when skip reason is "no conversational value", and logs "session unchanged; will retry next cycle" on failure.

**Relevant code:**
- `src-tauri/src/commands/ollama.rs` — `compact_conversation_history`, on-request path, `run_periodic_session_compaction`
- `src-tauri/src/session_memory.rs` — `count_conversational_messages`

**Acceptance:**
- Compaction no longer strips the current task intent or recent concrete outcomes.
- Logs make it clear whether compaction ran, skipped, or failed, and why.

## Phase 6: Retry and failover taxonomy

**Objective:** Retry only when it makes sense, and classify failures accurately.

**Implement:**
1. Separate transient failures from non-transient failures:
   - timeout
   - overloaded/rate-limit
   - auth/config
   - billing/quota
   - malformed provider response
2. Retry only transient categories and verification-formatting failures.
3. Do not run the same broad retry path for quota/auth failures.
4. Keep error text operator-friendly in logs and user-facing replies.

**Relevant code:**
- `src-tauri/src/commands/ollama.rs`
- provider-specific request helpers

**Acceptance:**
- Fewer useless retries.
- Better logs and less noisy recovery behavior.

## Phase 7: Observability and regression coverage

**Objective:** Make future failures obvious and easier to diagnose.

**Implement:**
1. ✅ Add logs for (done):
   - request id — included in all "Agent router [request_id]:" logs
   - topic classification decision — NEW_TOPIC / SAME_TOPIC logged; skip (verification retry or cloud model) at DEBUG
   - whether prior session memory was loaded — "prior session N messages (capped at 20)" or "no prior session"
   - criteria extracted for this request — "extracted N success criteria" with request_id; retry path already logs "reusing N request-local success criteria"
   - whether retry reused request-local criteria only — already logged at session start and when reusing criteria
   - search result count and shaped payload size — Brave: "got X results (shaped to Y, blob Z bytes)"; Perplexity: "PERPLEXITY_SEARCH returned N results, blob Z bytes" with request_id
   - compaction decision/result — "Session compaction [request_id]: skipped/produced context/wrote lessons"
2. Add focused regression coverage for (optional follow-up):
   - unrelated request after Redmine/ticket session
   - Barcelona/news query with Perplexity
   - verification retry with request-local criteria
   - session reload after restart
   - search result truncation preserving source/date fields

**Acceptance:**
- One log scan can explain why a retry happened and what state it used.
- The Barcelona-style bug is covered by regression logic.

## Suggested execution order

1. Phase 1 — request-local execution state
2. Phase 2 — separate conversation memory from execution artifacts
3. Phase 4 — news-aware answering and verification
4. Phase 3 — search result shaping
5. Phase 5 — compaction hardening
6. Phase 6 — retry/failover taxonomy
7. Phase 7 — observability and regression coverage

## Minimal high-value milestone

If time is limited overnight, do these first:
1. Request-local criteria and retry state
2. Stop persisting verifier/tool meta-prompts into normal session memory
3. Enforce sourced news answers with dates when available

That should directly fix the Barcelona-style failure with the best risk/reward ratio.

## Validation plan

1. Start from a Discord session with prior Redmine/ticket history.
2. Ask a fresh question like: “Can you look on the Internet for news involving Barcelona?”
3. Confirm:
   - topic classifier chooses `NEW_TOPIC`
   - only local criteria are used
   - Perplexity results are compact and source-rich
   - final answer includes sources and dates
   - verification does not inject stale task context
4. Repeat after app restart to verify persisted session reload is still safe.

## Out of scope for this task

- Large UI redesigns
- Replacing the session memory backend entirely
- Full provider abstraction rewrite
- New external services or dependencies

## Acceptance summary

The overnight work is successful when:
- unrelated retries no longer bleed old task context into new requests
- search/news answers consistently include sources and dates when available
- compaction and retries preserve current task intent
- logs clearly explain the flow of one request from intake to retry/final answer
