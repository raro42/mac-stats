## Installation

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Gatekeeper workaround
If macOS blocks the app, right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

* **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
* **AI Chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
* **Suspicious-pattern heuristics (log-only):** Text from web fetches, Discord messages, scheduler/task prompts, heartbeat checklists, browser extract, and CLI questions is scanned for a small set of regex categories (prompt-injection–style phrases, destructive shell tropes, etc.). On match, mac-stats emits **one** structured `tracing` line at subsystem `security/untrusted` (INFO, or WARN when a high-severity category matched) with `source`, `content_len`, `pattern_count`, and `labels` — **not** the raw body. Implementation: `commands/suspicious_patterns.rs`. Content is never dropped or altered by this scan.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Router and Agents

### Short Answers

1. **Can agent-000 (or any other agent) explicitly call the router to execute things on the API?**
   Only the entry-point model can. When a request starts (e.g. from Discord with `agent: 000`), that agent is the one in the router loop: it doesn’t “call” the router as a separate service; it is the model the router is talking to. So agent-000 can “call the router” in the sense that it outputs tool lines (`FETCH_URL: ...`, `TASK_CREATE: ...`, `AGENT: 002 ...`) and the **router** (the Rust code in `answer_with_ollama_and_fetch`) executes them.
   Sub-agents (e.g. 001, 002) cannot. When the orchestrator outputs `AGENT: 002 <task>`, the app runs that agent in a **single** Ollama request (no tool list, no router). So 002 cannot output `FETCH_URL:` or `TASK_APPEND:` and have it executed.

2. **Is the router involved in the discussion between agents?**
   Yes. The **router is the only thing that talks to the entry-point model** and runs tools. When the entry-point outputs `AGENT: 002 <task>`, the router runs 002 (one turn), gets the reply, and injects it back into the router’s conversation with the entry-point. So the “discussion” is: **router ↔ entry-point (orchestrator)**. Sub-agents do not talk to the router and do not talk to each other; they only receive a task and return a single reply.

3. **How do agents and the API router work together?**
   - **One** “main” conversation: router (Rust) + **entry-point model** (e.g. agent-000 when overridden, or default Ollama model).
   - The entry-point gets the **full tool list** (FETCH_URL, BRAVE_SEARCH, AGENT, TASK_*, SCHEDULE, RUN_CMD, OLLAMA_API, etc.) and runs in a **tool loop**: reply → parse tool line → execute → inject result → call model again.
   - **Ollama `/api/chat` queue** (`ollama_queue.rs`): Outbound chat requests share a **global** concurrency limit (default **1** simultaneous `/api/chat` call app-wide) and a **per-key FIFO** wait (e.g. `discord:<channel id>`, `scheduler`, `cpu_ui`, `cli`). This avoids piling concurrent generations onto one GPU. Configure `ollamaGlobalConcurrency` in `~/.mac-stats/config.json` (1–16) or `MAC_STATS_OLLAMA_GLOBAL_CONCURRENCY`; the limit is read **once** on first queue use per process (restart to pick up changes). At `-vv`, `debug2` lines in the log include `ollama/queue:`: while blocked on a key, `blocked waiting for key slot` with `per_key_waiters_ahead`; after the slot is taken, lines include `key_wait_ms` and global semaphore availability. Unit coverage: `cargo test ollama_http_queue` (from `src-tauri`).
   - **Context assembly** (`commands/context_assembler.rs`): CPU-window chat and the agent router share the `ContextAssembler` trait (token budget from model `num_ctx` minus a safety margin, oldest-first history trim, shared metric/prompt fragments). Budget and trim lines log at tracing target **`mac_stats::ollama/chat`** (same subsystem as other Ollama router logs) so they show in `~/.mac-stats/debug.log` when the app runs with **`-vv`**. Search for the substring `context_assembler:` (e.g. `resolved CPU chat token budget`, `agent router effective context budget`, `trimmed history to token budget`).
   - Before every Ollama `/api/chat` request, **`sanitize_conversation_history`** (`commands/conversation_sanitize.rs`) runs on the message list: it inserts a short synthetic user line when an assistant message still contains tool calls but the following message does not look like a tool result, and it strips obvious orphan tool-output blobs when the prior turn had no tool call. Repairs are logged at `-vv` (`debug2`). This mirrors the idea of a transcript repair pass so partial or corrupted session history is less likely to confuse the model.
   - **Internal event bus** (`events/mod.rs`): default handlers register from Tauri `lib.rs` setup (menu bar) and on the first `answer_with_ollama_and_fetch` call (CLI `discord run-ollama`, `agent test`, scheduler — idempotent). After each tool dispatch in the router tool loop, `tool:invoked` is emitted (name, success, duration). After the browser agent writes a screenshot PNG, `screenshot:saved` is emitted; default handlers log at `events/screenshot` and forward `ATTACH:` to the active agent status channel when a Discord (or other) tool run is in progress.
   - **AGENT** is one of those tools: when the entry-point replies `AGENT: <id or slug> <task>`, the router calls `run_agent_ollama_session(agent, task)` — a **single** Ollama request with that agent’s prompt and model, **no** tool list. The sub-agent’s reply is then injected back to the entry-point.
   - So: **router** = executor of all tools; **entry-point agent** = the only “mind” that can ask the router to run tools (including running other agents). **Sub-agents** = tools that return text; they have no API.

### Directive tags (delivery hints)

The model may embed inline markers `[[thread_reply]]`, `[[attach_screenshot]]`, and `[[split_long]]` in **plain assistant text** (not tool lines). They are stripped before the user sees the reply and are removed from session history sent back to the model. Parsing lives in `commands/directive_tags.rs`; the router merges `[[attach_screenshot]]` with the latest `BROWSER_SCREENSHOT` path, and Discord uses the other two for inline replies and paragraph-oriented multi-message splits. Execution and CPU default system prompts document these tags for the model.

## Flow Diagram (Conceptual)

```
User (e.g. Discord)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  ROUTER (answer_with_ollama_and_fetch)                          │
│  - Builds tool list (FETCH_URL, AGENT, TASK_*, SCHEDULE, …)     │
│  - One “main” conversation with the ENTRY-POINT model           │
│  - Tool loop: get reply → if tool line → execute → inject result │
└─────────────────────────────────────────────────────────────────┘
    │
    │  entry-point = agent_override (e.g. agent-000) or default
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  ENTRY-POINT MODEL (e.g. orchestrator / agent-000)              │
│  - Sees full API; can output: FETCH_URL:, TASK_CREATE:,         │
│    AGENT: 002 <task>, etc.                                       │
│  - When it outputs AGENT: 002 …, router runs sub-agent 002      │
└─────────────────────────────────────────────────────────────────┘
    │
    │  when entry-point outputs "AGENT: 002 <task>"
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  run_agent_ollama_session(agent_002, task)                       │
│  - Single Ollama request: system=002’s prompt, user=task        │
│  - No tool list; 002 cannot call FETCH_URL, TASK_*, etc.         │
│  - Returns one reply → router injects it back to entry-point     │
└─────────────────────────────────────────────────────────────────┘
```

## Implications

- **Orchestrator (e.g. agent-000)** should have the Router API Commands in its `skill.md` so it knows how to use FETCH_URL, TASK_*, AGENT, etc. It is the only agent that gets to use those commands in that request.
- **Specialist agents (001, 002, 003)** only need to do their job in one turn; they cannot schedule, create tasks, or call other agents via the router. If a specialist “needs” to do that, the orchestrator must do it (e.g. orchestrator calls 002, gets result, then orchestrator does TASK_APPEND or SCHEDULE).
- There is no “agent calling the router as an API” in a separate request: the router is the loop that drives the entry-point. So “explicitly call the router” = the entry-point outputting tool lines in that same conversation.

## Untrusted web text (homoglyphs)

Text from **FETCH_URL** (after HTML cleaning), **BROWSER_EXTRACT** (CDP markdown with innerText fallback), and the HTTP browser fallback’s assembled body text is still **attacker-controlled**. Before it enters the router tool loop, mac-stats can normalize Unicode homoglyphs: fullwidth ASCII (Halfwidth and Fullwidth Forms) maps to plain ASCII, and common “angle” confusables (guillemets, CJK angle quotes, mathematical angle brackets, fullwidth `<` / `>`) map to ASCII `<` and `>`. Implementation: `commands/text_normalize.rs`, applied at those boundaries only (not the user’s own typed messages). This is defence in depth for delimiter spoofing; it does **not** replace random boundary IDs or guarantee safety. Default **on**; set `config.json` `normalizeUntrustedHomoglyphs` to `false` or env `MAC_STATS_NORMALIZE_UNTRUSTED_HOMOGLYPHS=false` to disable.

**Untrusted boundary envelopes:** Externally sourced blobs (fetched pages, Discord user text, API JSON, search results, `RUN_CMD` stdout, scheduler/task payloads, heartbeat checklist text, etc.) are wrapped with a per-call random hex boundary in `commands/untrusted_content.rs` before injection into prompts. Tool-line parsing (`commands/tool_parsing.rs`) strips well-formed `<<<MS_UNTRUSTED_BEGIN:…>>>` / `<<<MS_UNTRUSTED_END:…>>>` regions first so lines like `RUN_CMD:` inside fetched HTML are not treated as real tool invocations. At `-vv`, `init_tracing` enables `ollama/untrusted=debug` alongside `mac_stats=debug` so wrap events are not filtered out (target is not under `mac_stats::`). Events use message `wrapped untrusted content for LLM prompt` with fields `boundary_id`, `label`, `content_chars`; `debug.log` uses `with_target(false)` so grep for that message or `boundary_id=` rather than the target string.

**Tool registry:** Canonical tool names, line prefixes, the compact “registry” paragraph prepended to the agent system prompt, and the inline `then TOOL:` splitter regex are all driven by `commands/tool_registry.rs` (`TOOLS` slice). Parsing logs `mac_stats::tool_parse` warnings when a line looks like `UNKNOWN_ALL_CAPS:` but is not registered (model hallucination). Handler dispatch remains in `commands/tool_loop.rs`.

## Specialist agents

**Definition**: Any agent that is **not** the entry-point is a specialist from the router’s perspective. The entry-point (e.g. agent-000) is the only one in the tool loop; all others are invoked via `AGENT: <id or slug or name> <task>` and run in a **single** Ollama request with **no** tool list.

**Invocation**: The orchestrator (or any entry-point) outputs one line: `AGENT: 002 write a small Python script` or `AGENT: senior-coder refactor this function`. The router resolves the selector (id `002`, slug `senior-coder`, or name) to an agent, then calls `run_agent_ollama_session(agent, task)`.

**What the specialist receives**:
- **User message**: Only the `<task>` string (e.g. “write a small Python script”). No main-conversation history, no tool list.
- **System prompt**: That agent’s assembled prompt (soul + mood + skill from `~/.mac-stats/agents/agent-NNN-<name>/`).

**Where they live**: Under `~/.mac-stats/agents/` (and defaults in the app under `src-tauri/defaults/agents/`). Each agent has `agent.json`, `skill.md`, `mood.md`, `soul.md`, and optionally `testing.md`. See **docs/017_llm_agents.md** for the default agent table and roles.

**Default specialists** (orchestrator 000 delegates to these):

| ID   | Name / slug           | Role / purpose        |
|------|------------------------|------------------------|
| 001  | General Assistant      | General Q&A             |
| 002  | Coder (senior-coder)   | Code generation        |
| 003  | Generalist             | Fast replies           |
| 004  | Discord Expert         | Discord API specialist |
| 005  | Task Runner            | Task file execution    |
| 006  | Redmine                | Redmine ticket review/search/create/update |

**Limitation**: Specialists cannot call FETCH_URL, TASK_*, SCHEDULE, or another AGENT. If the task requires tools or multi-step work, the orchestrator must run the tool (or another agent) and use the result in the main conversation.

## Agent initialization and model resolution

Agents are **loaded from disk on each use** (`agents::load_agents()`): no in-memory cache of the list. Each call reads `~/.mac-stats/agents/`, discovers `agent-<id>` directories, and for each loads `agent.json` and `skill.md` (required), plus optional `soul.md`, `mood.md`, and memory files. Default agents are written by `Config::ensure_defaults()` at startup (agent.json only if missing; skill/testing overwritten from bundle).

**Model resolution** runs when a **ModelCatalog** is available. The catalog is built at startup by `ensure_ollama_agent_ready_at_startup()` (async): it fetches Ollama `/api/tags`, classifies models (general, code, small, vision, etc.), and caches the catalog. When `load_agents()` runs, it calls `resolve_agent_models(agents, catalog)` if the catalog is set; that resolves each agent’s `model_role` (e.g. `"general"`, `"code"`) to an actual model name. If the catalog is **not** set yet (e.g. Ollama still starting or first request before startup task completed), model resolution is skipped and agents with only `model_role` (no explicit `model`) keep `model = None` until the next load after the catalog is ready. The app logs: `Agents: model catalog not yet available, model_role resolution skipped (Ollama may still be starting)` when this happens.

**Startup order**: `ensure_defaults()` runs synchronously; then `ensure_ollama_agent_ready_at_startup()` is spawned (async). So the first few requests (e.g. Discord or CPU window) might call `load_agents()` before the catalog exists. In practice the startup task usually completes within seconds. If Ollama is down or slow, agents may temporarily have no resolved model; once the catalog is set, every subsequent `load_agents()` will resolve correctly.

**Failure modes** that prevent an agent from loading (and are logged): missing or unreadable `agent.json`, invalid JSON in `agent.json`, missing or empty `skill.md`. Disabled agents (`enabled: false`) are skipped with a debug log.

## References

- **Tool loop**: `commands/ollama.rs` → `answer_with_ollama_and_fetch`, `parse_tool_from_response`, `parse_all_tools_from_response`, and `commands/tool_loop.rs` → `run_tool_loop` (`while tool_count < max_tool_iterations`). **Failure budget:** consecutive tool errors or failed follow-up Ollama calls increment a counter; successes reset it. When the counter reaches `maxConsecutiveToolFailures` (default **3**, `config.json` / env `MAC_STATS_MAX_CONSECUTIVE_TOOL_FAILURES`), the loop stops and returns partial results with a clear message. **FETCH_URL** outcomes that count as tool errors for the budget include the user-visible failure lines from `network_tool_dispatch::handle_fetch_url` / discord redirect (e.g. “That URL could not be fetched…”, 401 short-circuit, `Discord API failed…`), not only the `FETCH_URL error:` prefix from rare `Err` returns. **Multi-tool batch:** after a successful in-batch browser navigation, remaining tools in that batch are skipped so the next model turn sees fresh page state (browser-use-style stale guard). **FETCH_URL** body path: `commands/network_tool_dispatch.rs` → `html_cleaning::clean_html` → `text_normalize::apply_untrusted_homoglyph_normalization`. Plans like `RUN_CMD: date then REDMINE_API GET /time_entries.json?...` are normalized and split into separate steps so each tool runs in sequence (not one RUN_CMD with the whole chain). **Proactive tool-result context budget (opt-in):** when `proactiveToolResultContextBudgetEnabled` is true in `config.json` or `MAC_STATS_PROACTIVE_CTX_BUDGET=1`, before the first execution Ollama call and before each follow-up call in the tool loop, `commands/content_reduction.rs` estimates total message tokens (same heuristic as context assembly) and compacts older/larger non-system messages when the estimate exceeds a headroom threshold (`proactiveContextBudgetHeadroomRatio`, default **0.12**; env `MAC_STATS_PROACTIVE_CTX_HEADROOM_RATIO`). Compacted bodies append `[compacted proactively for context budget: …]`; the existing context-overflow retry path (`[truncated from … due to context limit]`) remains unchanged as a safety net.
- **Partial progress on timeout:** `commands/partial_progress.rs` — optional `PartialProgressCapture` on `OllamaRequest` records tool names, short argument summaries (not full tool results), and the latest assistant text snippet (~200 chars). Scheduler wall-clock timeout (`schedulerTaskTimeoutSecs`), heartbeat timeout, and Discord error replies append a formatted summary when `OllamaRunError::should_attach_partial_progress` is true (`TIMEOUT` / `SERVICE_UNAVAILABLE`, or timeout-shaped text on `INTERNAL_ERROR`); full-turn wall-clock timeout success replies embed the same block in `turn_lifecycle::finalize_turn_timeout`. Browser runs also add current page URL/title from `get_last_browser_state_snapshot` when `BROWSER_*` tools were used.
- **Full-turn wall-clock timeout (hung turn recovery):** After model resolution, the router runs under `tokio::time::timeout_at` with a deadline shared across verification retries (`OllamaRequest::turn_deadline`). Defaults: Discord `agentRouterTurnTimeoutSecsDiscord` (300s), in-app CPU path `agentRouterTurnTimeoutSecsUi` (180s), remote-without-Discord (scheduler / task runner / heartbeat) `agentRouterTurnTimeoutSecsRemote` (300s); optional per-request `turn_timeout_secs`. On expiry, an output gate stops Discord status lines, draft updates, and `ATTACH:` forwards from the tool loop; cleanup tries `about:blank` on the active tab only if the timed-out `request_id` still owns the coordination slot (`commands/turn_lifecycle.rs`), within `agentRouterTurnTimeoutCleanupGraceSecs` (default 3s). The user receives an `OllamaReply` whose text states the budget; post-timeout verification and task-file append do not run because the turn future was cancelled.
- **Abort cutoff (stale inbound guard):** On turn timeout, `finalize_turn_timeout` records an in-memory **abort cutoff** per coordination key (`commands/abort_cutoff.rs`, same key as `turn_lifecycle::coordination_key`). Discord inbound messages are dropped (no reply, no session append) if their message time is strictly before that cutoff; scheduler Ollama runs use the schedule’s **due** time (and optional `reply_to_channel_id` so the key matches the Discord channel); heartbeat uses the delivery channel when configured. A fresh Discord conversation (session reset phrases / `new session:`) clears the cutoff for that channel. CPU window chat clears the shared non-Discord slot cutoff on the same reset phrases / `new session:` prefix, and otherwise checks that slot at `ollama_chat_with_execution` entry (synthetic event id + `Utc::now()`). Cutoff state is not persisted to disk.
- **Sub-agent run (no tools)**: `run_agent_ollama_session` in `commands/ollama.rs` (single request, no tool list).
- **Router API snippet for orchestrator**: `docs/agent_000_router_commands_snippet.md`, `docs/017_llm_agents.md`.

## More advanced tool commands (future)

Current tools use a single-line form `TOOL_NAME: <argument>`. Possible future improvements (for consideration; not scheduled):

- **Structured or multi-argument tools**: Allow key-value or JSON-style arguments (e.g. `FETCH_URL: {"url":"...","max_bytes":50000}`) for tools that need more than one parameter, with clear parsing and backward compatibility.
- **Tool result streaming**: For long-running or large results (e.g. FETCH_URL of a big page), stream chunks into the conversation instead of one large injection.
- **Compound or batch hints**: Let the model request multiple tools in one turn with a defined order (e.g. "run A then B") while still executing one tool per step for safety and clarity; or explicit "batch" syntax for read-only tools.
- **Tool schema in prompt**: Expose a minimal schema (name, description, argument shape) so the model can choose tools more reliably; keep the prompt size manageable (see `docs/034_tool_prompt_scaling.md`).

Implementation would live in `commands/ollama.rs` (parsing, tool loop) and possibly in agent prompts (router snippet). No code change in this FEAT; this section records scope for future work.

## Retry and failover taxonomy (task-008 Phase 6)

This section documents when the router and related components **retry** (same operation again) or **fail over** (try an alternative path), and when they **fail** without retry. Implementation lives in `commands/ollama.rs`, `discord/api.rs`, `discord/mod.rs`, `browser_agent/mod.rs`, and `scheduler/mod.rs`.

### Retry (one extra attempt)

| Component | Trigger | Behavior | Location |
|-----------|---------|----------|----------|
| **Ollama API** | 503 or request timeout | Retry once after a short delay; then return user-facing error. | `commands/ollama.rs` (task-001) |
| **Completion verification** | Verification says "not satisfied" | One retry with "complete remaining steps" prompt; same `request_id`; no second retry. | `answer_with_ollama_and_fetch` with `retry_on_verification_no`; retry path passes `false` so we don’t retry again. |
| **Discord API** | Safe transport errors (pre-connect / DNS / refused) on sends; **GET** 5xx | One retry after delay for those cases only. **POST** send: no 5xx retry; no retry on timeout/reset (avoid duplicates). 429 still uses Retry-After loop. | `discord/api.rs`, `discord/mod.rs` |
| **Browser CDP** | Connection/session error | Clear session and retry once (`with_connection_retry`). | `browser_agent/mod.rs` |
| **Browser CDP** | Hung tab / dead Chrome while WebSocket still open | Before CDP tool work: optional `kill(0)` on launched child PID, then `Runtime.evaluate("1+1")` under a 2s cap (`check_browser_alive`); on failure clear session and return “Browser unresponsive…” **immediately** (no extra `with_connection_retry` pass in the same tool call). | `browser_agent/mod.rs` |
| **BROWSER_NAVIGATE** | CDP navigate fails | Ensure Chrome on port, retry CDP; if still failing, try HTTP fallback. | `commands/ollama.rs` (tool handler) |
| **Session compaction** | Compaction run fails (e.g. Ollama error) | Retry once after 3s; then log and leave session unchanged; next cycle will try again. | `commands/ollama.rs` (periodic compaction) |
| **Having-fun (Discord)** | Idle thought request times out | Retry once after delay. | `discord/mod.rs` |

### No retry (fail and report)

- **Ollama**: After the one retry for 503/timeout; or on 4xx / non-retryable errors.
- **Verification**: Second "not satisfied" → we do not retry again; user sees answer plus optional disclaimer.
- **Discord API**: After one retry for **safe** transport errors only (not timeout/reset); **POST** messages do not retry on 5xx; or on 4xx (e.g. bad request, forbidden). Gateway reply send retries once only for safe / rate-limit-classified errors.
- **Browser**: After connection retry or CDP→HTTP failover; or on non-transient errors (e.g. invalid URL). **CDP health check failure** (`Browser unresponsive`): no second retry—the session is already cleared inside `check_browser_alive`.
- **Compaction**: When we **skip** compaction (e.g. fewer than 2 conversational messages), we do not count that as a failure and do not retry in the same cycle.

### Summary

- **Retry** = at most one extra attempt for transient failures (timeout, 503, connection, verification NO).
- **Failover** = try alternative path once (e.g. CDP → HTTP for BROWSER_NAVIGATE).
- **Fail** = return error or partial result to the user; no further retries in that request.

## Open tasks:
- ~~Investigate why some agents are not being properly initialized.~~ **Done:** § "Agent initialization and model resolution" above (load from disk each time; model resolution depends on ModelCatalog from `ensure_ollama_agent_ready_at_startup`; race when catalog not yet set; failure modes logged). Added log when catalog missing in `agents/mod.rs`.
- ~~Improve the documentation for specialist agents.~~ **Done:** § "Specialist agents" above (definition, invocation, what they receive, where they live, default table, limitation).
- ~~Consider adding support for more advanced tool commands.~~ **Done:** § "More advanced tool commands (future)" above (options: structured args, result streaming, compound/batch hints, tool schema; scope for future implementation).
- ~~task-008 Phase 6: Retry/failover taxonomy.~~ **Done:** § "Retry and failover taxonomy" above (retry table, no-retry cases, summary); doc-only.