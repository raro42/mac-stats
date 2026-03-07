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
### Menu Bar
- CPU, GPU, RAM, disk at a glance; click to open the details window.

### AI Chat
- Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents
### Invocation
Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Agents
| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Session Compaction and Long-term Memory Plan
### Implemented
- **Session memory**: `SessionState` has `last_activity` (updated on `add_message` and `replace_session`). `list_sessions()` returns all in-memory sessions with source, session_id, message_count, last_activity.
- **30-minute periodic compaction**: A dedicated thread (started from `lib.rs`) runs every 30 minutes and calls `run_periodic_session_compaction()`. For each session with ≥ 4 messages: compacts to context + lessons, appends lessons to global `memory.md`; if **inactive** (no activity for 30 min) clears the session; if **active** replaces in-memory with the summary.
- **On-request compaction**: Unchanged — still triggers at ≥ 8 messages and writes lessons to global memory.
- **Having_fun time-of-day**: The having_fun system prompt now includes current local time and period-aware guidance (night / morning / afternoon / evening) so the bot can behave differently by time of day.

### Goals
1. **Compress the actual session every 8 messages** (keep current on-request compaction).
2. **Every 30 minutes**: compress all active sessions into long-term memory; then clear inactive sessions after compressing them into memory.
3. **Clarify how long-term memory is used** and how to benefit from it.

### Current State
### Session Compaction (on request)
- **Trigger**: When handling a request (Discord, scheduler, etc.), if the conversation history has **≥ 8 messages**, we compact before calling the model.
- **Discord having_fun**: For channels configured as having_fun, compaction does **not** call the LLM; it stores a fixed minimal CONTEXT so we never invent themes from casual chat. Log: `Session compaction: Discord having_fun channel <id> — using fixed minimal context (no LLM)`.
- **What happens** (non–having_fun): `compact_conversation_history()` sends the last 20 messages to a “small” model; it returns:
  - **CONTEXT**: short summary (max ~300 words) of verified facts and outcomes.
  - **LESSONS**: bullet points of lessons learned (tools that worked, corrections, mistakes to avoid).
- **Where it goes**:
  - Context replaces the in-memory session for that channel (one system message with the summary).
  - Lessons are appended **only to global memory**: `~/.mac-stats/agents/memory.md`.
- **30-minute compaction**: A background thread runs every 30 min and compacts in-memory sessions (see Implemented above).

### Long-term Memory
### Global Memory
- **Global memory**: `~/.mac-stats/agents/memory.md`  
  - Loaded for **every** agent when building the prompt (soul + mood + **memory** + skill).  
  - In `agents/mod.rs`, `build_combined_prompt()` injects it under a “Memory (lessons learned — follow these)” section.  
  - So **all agents already benefit** from whatever is in global memory.

- **Per-agent memory**: `~/.mac-stats/agents/agent-<id>/memory.md`  
  - Loaded only for that agent and concatenated with global memory.  
  - Used when the model calls **MEMORY_APPEND agent:<id>**.  
  - Session compaction **does not** write here today (only MEMORY_APPEND does).

### Session Storage
### In-memory
- **In-memory**: `session_memory` keeps a map `source + session_id` → `SessionState` (messages, topic_slug, created_at).  
  - `last_activity` (implemented) → defines “active” vs “inactive” (active = within 30 min; inactive = older).
- **On disk**: When a session has > 3 messages, we persist to  
  `~/.mac-stats/session/session-memory-<session_id>-<timestamp>-<topic>.md`.  
  - New file each time we persist (new timestamp); old files are never deleted.

## Desired Behavior
### Summary
1. **Keep**: On each request, if history ≥ 8 messages → compact, replace session with summary, append lessons to global memory (and optionally to agent memory when `agent_override` is set).
2. **Add**: Every **30 minutes**, a background job:
   - For every session (in-memory) that has “enough” messages (e.g. ≥ 4 or ≥ 8):
     - Build `ChatMessage` list from session messages.
     - Call `compact_conversation_history(&messages, "Periodic session compaction.")` (reuse existing function; current_question is only for the compactor prompt).
     - On success:
       - Append **lessons** to global memory (`Config::memory_file_path()`). Optionally append to a per-agent memory if we have a channel→agent mapping (future).
       - If **inactive** (last_activity < now - 30 min): call `session_memory::clear_session(source, session_id)`.
       - If **active**: call `session_memory::replace_session(source, session_id, vec![("system", context)])` so the next request sees the summary and we don’t keep a huge history.
3. **Define**:
   - **Active**: last message (or last activity) within the last **30 minutes**.
   - **Inactive**: last activity **older than 30 minutes** → compact into memory, then clear.

## Implementation Outline
### 1. Session Memory: support “list” and “last activity”
- **`session_memory.rs`**:
  - Add `last_activity: Option<chrono::DateTime<chrono::Local>>` to `SessionState`; set/update on `add_message()` (and when loading from disk we don’t have it, so use `created_at` or file mtime as proxy for “resumed” sessions if needed).
  - Add **`list_sessions(source: Option<&str>)`** (or `list_all_sessions()`) returning e.g. `Vec<(String, u64, usize, chrono::DateTime<chrono::Local>)>` = (source, session_id, message_count, last_activity). Only in-memory sessions are considered; disk-only sessions can be handled in a second phase (e.g. list session files and compact those too).
  - Optionally: add **`get_session_for_compaction(source, session_id)`** that returns the current messages (and last_activity) so the 30-min job can compact without holding the lock for a long time.

### 2. 30-minute Background Loop
- **Where to run**: Either:
  - **Option A**: Spawn a task inside the **Discord** tokio runtime (e.g. in `run_discord_client`, before or alongside the client), so the loop runs only when Discord is running; or
  - **Option B**: Start a **dedicated thread** (e.g. from `lib.rs`) with its own tokio runtime, so session compaction runs even when Discord is not connected. Prefer **Option B** if we want one place for “all session sources” (Discord today; others later).
- **Loop** (every 30 min):
  1. Call `session_memory::list_sessions(None)` (or equivalent).
  2. For each session with `message_count >= COMPACTION_THRESHOLD` (8) or a lower threshold (e.g. 4) for the 30-min pass:
     - Build `ChatMessage` list from session messages.
     - Call `compact_conversation_history(&messages, "Periodic session compaction.")` (reuse existing function; current_question is only for the compactor prompt).
     - On success:
       - Append **lessons** to global memory (`Config::memory_file_path()`). Optionally append to a per-agent memory if we have a channel→agent mapping (future).
       - If **inactive** (last_activity < now - 30 min): call `session_memory::clear_session(source, session_id)`.
       - If **active**: call `session_memory::replace_session(source, session_id, vec![("system", context)])` so the next request sees the summary and we don’t keep a huge history.
  3. Optionally: **prune old session files** on disk for sessions we just cleared (delete or move to an `archive/` subdir) to avoid unbounded growth of `~/.mac-stats/session/`.

### 3. Keep 8-message compaction on request unchanged
- No change to the existing logic in `answer_with_ollama_and_fetch`: when `raw_history.len() >= COMPACTION_THRESHOLD`, compact, write lessons to global (and optionally to `agent_override`’s memory when set), replace session with summary. This remains the “fast path” so long conversations get compacted as the user talks.

### 4. Optional: write compaction lessons to the “respective agent”
- When compaction runs (either on request or in the 30-min job), if we have an **agent_override** (on request) or a **channel→agent** config (for 30-min job), append lessons to that agent’s `memory.md` in addition to (or instead of) global.  
- This requires either passing `agent_override` through to the compaction success path (already available on request) or storing a default agent per Discord channel in `discord_channels.json` (new config). Start with **global only** for the 30-min job; add per-agent when we have a clear mapping.

### 5. Config Constants
- **COMPACTION_THRESHOLD** = 8 (already exists) for on-request.
- **PERIODIC_COMPACTION_INTERVAL_SECS** = 30 * 60 (30 min).
- **INACTIVE_THRESHOLD_SECS** = 30 * 60 (session is “inactive” if last_activity older than this).
- **PERIODIC_COMPACTION_MIN_MESSAGES** = 4 or 8 (minimum messages to run compaction in the 30-min pass; 4 keeps more in long-term memory, 8 matches on-request).

## Summary Table
| What | When | Where lessons go | Session after |
|------|------|------------------|----------------|
| On-request compaction | Every request with ≥ 8 messages | Global (and optionally agent if override) | Replaced with 1 system message (summary) |
| 30-min compaction (active session) | Every 30 min, session has ≥ N messages, last_activity within 30 min | Global (and optionally agent if mapped) | Replaced with 1 system message (summary) |
| 30-min compaction (inactive session) | Every 30 min, session has ≥ N messages, last_activity older than 30 min | Global (and optionally agent if mapped) | **Cleared** (removed from store; optional file prune) |