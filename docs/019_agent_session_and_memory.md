## Installation

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

* **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
* **AI chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents (What Ollama Can Invoke)

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Agent Session Context and Memory (Short-term / Long-term)

### Current State

* **CPU window**: Frontend sends `conversation_history` to `ollama_chat_with_execution`; the main chat has multi-turn context within the request.
* **Discord**: **Session history is now passed into Ollama.** Before each request we call `session_memory::get_messages("discord", channel_id)` (and if empty, `load_messages_from_latest_session_file` to resume from the latest session file after restart). That history is converted to `ChatMessage` and passed as `conversation_history` into `answer_with_ollama_and_fetch`, so the model sees prior turns and can resolve "there", "it", "El Masnou", etc. Capped at 20 messages. Messages are still stored with `add_message` and persisted when > 3.

### Session Context (Who Sees What)

| Scope | What gets context | How |
|-------|-------------------|-----|
| **Main conversation** (user ↔ entry agent, e.g. orchestrator) | The **entry agent** sees the ongoing conversation: current question + prior user/assistant messages in that session. | Pass **conversation_history** (with a cap) into the pipeline that builds the planning and execution prompts. |
| **AGENT: sub-calls** (orchestrator → specialist) | The **specialist** sees only the **task** string and its own system prompt (soul + mood + skill). No main-conversation history. | Keep `run_agent_ollama_session(agent, user_message)` one-shot. Optionally later: append a short "Context from main conversation: …" (e.g. last user message or last N chars) to the task when invoking the agent. |
| **Session identity** | Per entry point: CPU window = one logical session (frontend owns history); Discord = one session per channel (or per channel+user if you want). | Session id: CPU = e.g. window id or "cpu"; Discord = channel id. Use for short-term and (if needed) long-term memory keys. |

So: **session context** = the list of prior messages in the **main** conversation only. Sub-agents do not get that list unless we explicitly add a small context snippet later.

### Short-term Memory

* **Definition**: The recent messages in the current session (current request + prior turns in the same channel/window).
* **Implementation**:
  - **CPU window**: Already has short-term memory via `conversation_history` from the frontend. Optionally cap (e.g. last 20 messages or last 4k tokens) to avoid overflow.
  - **Discord**: Add **loading** of short-term memory before calling Ollama:
    1. **Expose** a function in `session_memory` to return the current in-memory messages for a key (e.g. `get_messages(source, session_id) -> Vec<(role, content)>`). If you persist and then clear in-memory state, optionally add "load latest session file for this channel" so that after app restart the first request in a channel can still get recent history.
    2. **Before** `answer_with_ollama_and_fetch`, call `session_memory::get_messages("discord", channel_id)` (or equivalent). Convert to `Vec<ChatMessage>`, apply a **cap** (e.g. last 20 messages or last 8k tokens), and pass as **conversation_history**.
    3. **Extend** `answer_with_ollama_and_fetch` to accept an optional `conversation_history: Option<Vec<ChatMessage>>`. When present, prepend those messages (in order) to the planning and execution prompts so the model sees the recent conversation. Cap total history (message count or token count) in one place to avoid exceeding model context.
  - **Scheduler / other entry points**: If they ever call the same pipeline, pass `None` for history or their own session key and load logic.
* **Cap**: Define a single place (e.g. last N messages or M tokens). Suggest N = 20 messages or M = 8192 tokens as a default; document so it can be tuned.

### Session Startup (recommended order)

Before doing anything else in a session, the agent should load context in this order:

1. **Soul** — Read `~/.mac-stats/agents/soul.md` (or the agent’s own soul if overridden). Defines who the agent is.
2. **User info** — Read "who you’re helping": `~/.mac-stats/user-info.json` (or equivalent) when present; Discord user details are injected by the app when available.
3. **Daily memory** — If the app supports daily memory files (e.g. `~/.mac-stats/memory/YYYY-MM-DD.md`), read **today’s** and **yesterday’s** files so the agent has recent context. (Optional; not all setups use daily files.)
4. **Global/personal memory (main session only)** — Read `~/.mac-stats/agents/memory.md` **only in main session** (in-app chat or Discord DM). Never load global/personal memory in Discord guild channels or having_fun — it holds personal context that must not leak into shared channels.

On **session reset** (user says "new topic", "reset", etc.), the app clears the conversation and injects a Session Startup instruction plus **current date/time (UTC)** so the agent knows which daily files to read and can greet with correct date awareness.

### Long-term Memory

* **Definition**: Information that persists across sessions (restarts, different days) and can be injected into the agent's context.
* **Main-session only**: Global memory (`~/.mac-stats/agents/memory.md`) is **only** loaded in **main session** (in-app or Discord DM). In Discord guild channels and having_fun, the app does **not** inject global memory, so personal context cannot leak. Per-channel memory (`memory-discord-{channel_id}.md`) is still loaded for that channel.
* **Existing**: Session files in `~/.mac-stats/session/session-memory-<topic>-<sessionid>-<timestamp>.md` are already long-term persistence of a **conversation transcript**, but they are not currently loaded to resume context. So "long-term" today = archive only.
* **Optional: resume session**: For Discord, optionally on first message in a channel after restart, load the **most recent** session file for that channel (match by session_id in filename) and use it to seed short-term memory (e.g. last K messages from that file). That gives continuity across restarts without a separate "long-term memory" store.
* **Optional: semantic long-term memory**: A separate store for facts, preferences, or summaries (e.g. "User prefers brief answers"; "Project X is in Python"). Not in current scope; if added later:
  - **Where**: e.g. `~/.mac-stats/memory/` with per-user or per-agent files (e.g. `user-<discord_id>.json` or `agent-<id>-memory.md`).
  - **Format**: Simple (e.g. JSON array of `{ "fact": "...", "updated": "..." }` or a markdown list).
  - **When**: Load at start of request, inject into system prompt or a "Long-term context" block so the entry agent (and optionally sub-agents) see it. Updates could be manual (user edits file) or via a future "remember this" tool that appends to the file.
  - **Scope**: Document as optional/future; no implementation required for the first agent rollout.

## Summary Table

| Type | What it is | Where it lives | How agents use it |
|------|------------|----------------|-------------------|
| **Session context** | Current conversation turn + prior messages in same session | In-request (CPU: frontend; Discord: session_memory in-memory + optional load from session file) | Passed as `conversation_history` to the **main** agent only; sub-agents (AGENT:) get task only. |
| **Short-term memory** | Recent messages in session (capped) | session_memory (Discord); frontend (CPU) | Capped list of (role, content) passed as conversation_history. |
| **Long-term memory** | Persisted transcript (session files); optional semantic facts | `~/.mac-stats/session/*.md`; optional `~/.mac-stats/memory/` | Session files: optional reload to seed short-term. Semantic: optional inject into system prompt (future). |

## Implementation Order (for session + short-term)

1. **session_memory**: Add `get_messages(source, session_id) -> Option<Vec<(String, String)>>` (or equivalent) returning the current in-memory messages for that key. If you later want "resume from file", add a function to load the latest session file for a given source+session_id and return messages.
2. **answer_with_ollama_and_fetch**: Add parameter `conversation_history: Option<Vec<ChatMessage>>`. When building the planning and execution message lists, prepend these messages (after the system block, in chronological order) and apply the chosen cap (e.g. last 20 messages).
3. **Discord**: Before calling `answer_with_ollama_and_fetch`, call `session_memory::get_messages("discord", channel_id)`, convert to `Vec<ChatMessage>`, cap, and pass as `conversation_history`.
4. **AGENT: sub-calls**: Leave `run_agent_ollama_session(agent, task)` one-shot (no main-conversation history). Optionally later: append a one-line "Context: <last user message>" to the task string when invoking.

## Behaviour after Changes

- **Discord**: Each request sees the last **20** messages in that channel (from in-memory, and after restart optionally from the latest session file). The model can refer to what was said earlier in the channel.
- **CPU window**: Unchanged; frontend continues to send `conversation_history`; the backend applies the same cap before planning/execution.
- **AGENT: calls**: Specialist still gets only the task string and its own prompt; no change unless you add an optional context snippet later.

## Current Implementation Notes

- The main router currently caps `conversation_history` to the last **20** messages before planning/execution.
- Discord loads prior turns from in-memory session storage first and falls back to the latest persisted session file after restart.
- Session compaction and session replacement are already wired into the main router path; the remaining work here is review/tuning, not first-time implementation.
- For Discord **having_fun** channels, session compaction uses a fixed minimal context (no LLM call) so compacted CONTEXT never invents task or platform themes from casual chat.

## Session memory implementation review (done)

- **Correctness**: Internal artifacts (verifier prompts, criteria extraction, tool dumps) are filtered on add and on load; empty content is not stored. Lock poison is handled (early return, no panic). Persist clones under lock then drops lock before I/O; clear/replace persist before mutating. **Fix applied:** `parse_session_file` was skipping the first block because the file starts with `## User` and the split leaves the first segment with a leading `## `; the parser now trims `## ` from each block so the first User/Assistant block is recognized.
- **Efficiency**: Lock is held only for in-memory reads/writes and a single clone for persist; I/O runs outside the lock. Message count is small (typical cap 20); list_sessions is O(sessions). Redundant normalization on read (get_messages) is acceptable; stored messages are already normalized on add.
- **Storage structure**: In-memory store is a `HashMap<key, SessionState>` with `Vec<(String, String)>` messages; persistence is one file per session (append-only style, new file per persist with timestamp). For typical channel/session counts this is fine; no change needed for now.

### Conversation-history storage structure (review)

- **In-memory:** One `HashMap<String, SessionState>` keyed by `"{source}-{session_id}"` (e.g. `discord-12345`). Each `SessionState` holds a `Vec<(String, String)>` of (role, content), plus topic_slug, created_at, last_activity. Lookup is O(1); message count per session is small (capped at 20 conversational at use site). For typical usage (dozens of Discord channels plus main session), this is efficient; no index or different structure needed.
- **Persistence:** Each persist writes a **new** file `session-memory-{session_id}-{timestamp}-{topic}.md` under `~/.mac-stats/session/`. That yields multiple files per channel over time. Resume uses the **latest** file (by mtime) for that session_id. Trade-off: many small files give a clear history and simple recovery; a single append-only file per channel would require rotation and more complex truncation. **Recommendation:** Keep current design. Revisit only if we observe thousands of session files per channel or I/O becomes a bottleneck (e.g. `list_sessions` + many `load_messages_from_latest_session_file` calls); then consider (a) single file per channel with append + periodic rotation, or (b) compaction that keeps only the latest N files per session_id.
- **Conclusion:** No optimization required for current scale. Documented here so future changes have a clear baseline and trigger.

### Manual edit of long-term memory (future)

- **Current:** Long-term context is loaded from files under `~/.mac-stats/` (e.g. `agents/memory.md`, `session/*.md`). Users can edit these files directly on disk; the app does not provide an in-app or Discord UI to edit them.
- **Possible enhancement:** A mechanism for users to manually edit or update long-term memory could include: (a) a Tauri command or Settings UI to append/update entries in a designated memory file; (b) a Discord command or keyword (e.g. "remember: …") that appends a fact to the main session memory file; or (c) a "Memory" tab in Settings that lists and edits the same files the agent loads. Out of scope for the current rollout; implement when users request it or when semantic long-term memory is added.

## Agent router time limits (Tauri / menu-bar defaults vs long runs)

The mac-stats agent is a **Tauri menu-bar app**: defaults favour responsive turns (short per-call HTTP timeouts and modest full-run wall clocks). That differs from a 48-hour “session until I stop it” style stack unless you opt in via config.

**Two different clocks:**

1. **`ollamaChatTimeoutSecs`** — caps **one** Ollama `/api/chat` HTTP request (planning, a follow-up after tools, verification, etc.). A long multi-tool run performs **many** such calls; each must finish within this limit.
2. **`agentRouterTurnTimeoutSecsDiscord` / `Ui` / `Remote`** — caps the **entire** `answer_with_ollama_and_fetch` turn (criteria + planning + tool loop + verification) for that entry path. This is the **session wall-clock** for one logical agent reply. You may raise it up to **48 hours** (`172800` seconds) for unattended jobs; defaults stay at **300s** (Discord/remote) or **180s** (in-app) so the UI and channels do not hang.

**Tool rounds:** When no per-agent override is used, default **max tool iterations** are configurable per path: `agentRouterMaxToolIterationsDiscord`, `agentRouterMaxToolIterationsUi`, `agentRouterMaxToolIterationsRemote` (default **15** each). An agent’s `max_tool_iterations` in `agent.json` overrides these for that agent.

**Other limits** (browser idle, scheduler task timeout, consecutive failure budget, optional tool-loop detection) stack independently; see the matrix in `src-tauri/src/commands/agent_session_limits.rs`.

**User-visible errors** usually name the limit (e.g. per-request timeout vs session wall-clock vs tool iteration cap) so operators can tell which knob to turn.

## Open tasks:

- ~~Review whether the `session_memory` implementation is correct and efficient.~~ **Done:** see "Session memory implementation review" above; parser fix for first block in `session_memory.rs`.
- ~~Review whether the current conversation-history storage structure should be optimized.~~ **Done:** see "Conversation-history storage structure (review)" above; current design kept; revisit only if scale or I/O demands it.
- ~~Consider adding a mechanism for users to manually edit or update their long-term memory.~~ **Done:** documented above as future consideration ("Manual edit of long-term memory"); no implementation.