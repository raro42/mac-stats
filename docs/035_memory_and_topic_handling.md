# mac-stats

## Overview  
**Local AI agent for macOS**  
- Menu bar: CPU, GPU, RAM, disk metrics  
- AI chat: Ollama, Discord, task runner, scheduler, MCP  
- No cloud, no telemetry  
- Built with Rust and Tauri  

---

## Installation  
**Recommended:**  
- Download DMG from [GitHub releases](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications  

**From source:**  
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run  
# Or one-liner: curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run  
```  

**Gatekeeper issues:**  
- Right-click DMG → **Open**  
- Or run: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`  

---

## Features  
- **Menu bar metrics** (click for details)  
- **AI chat** (Ollama in-app or via Discord)  
  - Tools: `FETCH_URL`, `BRAVE_SEARCH`, `RUN_CMD`, code execution, MCP  
- **Discord bot**  
- **Task scheduler**  
- **Real-time system monitoring**  

---

## Agents Overview  
### Tool agents (Ollama can invoke)  
| Agent | Invocation | Purpose |  
|-------|-----------|--------|  
| `FETCH_URL` | `FETCH_URL: <URL>` | Fetch web page content (server-side, no CORS) |  
| `BRAVE_SEARCH` | `BRAVE_SEARCH: <query>` | Web search via Brave API (requires `BRAVE_API_KEY`) |  
| `RUN_JS` | `RUN_JS: <code>` | Execute JavaScript in CPU window |  

**Other agents:**  
- **SCHEDULER** (info-only; Ollama can recommend but cannot invoke)  
- **Discord bot** (entry-point for chat)  
- **CPU window** (chat interface with execution)  

---

## Memory & Topic Handling  
### How mac-stats keeps context focused  
**Key strategies:**  
1. **Topic-aware compaction**  
   - Summarize context relevant to the current question  
   - If new topic: "Previous context not needed" → no prior context used  

2. **User-initiated reset**  
   - Phrases: "clear session", "new topic", "reset" (multi-language), or a leading **`new session:`** / **`new session `** prefix on the message.  
   - Clears session for the channel; the next message gets a **Session Startup** instruction plus current date/time (UTC) so the agent reloads soul, user-info, daily memory (today/yesterday), and in main session MEMORY (see Session Startup in docs/019).  
   - **Optional before-reset export:** `config.json` keys **`beforeResetTranscriptPath`** and **`beforeResetHook`** (or env `MAC_STATS_BEFORE_RESET_TRANSCRIPT_PATH` / `MAC_STATS_BEFORE_RESET_HOOK`) can write the pre-clear conversation as JSONL and run a non-blocking shell hook—see **docs/data_files_reference.md** (before-reset transcript export).  

3. **Memory injection**
   - **Global memory:** `~/.mac-stats/agents/memory.md` — injected **only in main session** (in-app chat or Discord DM). Never loaded in Discord guild channels or having_fun, to avoid leaking personal context.
   - **Main-session (in-app):** `~/.mac-stats/agents/memory-main.md` — injected when the request is from the CPU window (no Discord channel), so the main session has per-context memory like Discord channels.
   - **Discord per-channel:** `memory-discord-{channel_id}.md` — injected for that channel when present.

4. **New-topic detection (local only)**  
   - If 2+ prior messages: LLM check for "NEW_TOPIC" → no prior context  

5. **No tool-result pruning**  
   - Tool outputs are embedded in messages; compaction handles context control  

6. **Per-channel memory (Discord)**  
   - No cross-channel leakage  
   - DMs and channels have separate memory files  

---

## Memory pruning and compaction

Context is kept under control by **caps** and **compaction** (summarize-then-replace). Nothing is silently dropped without summarization when compaction runs.

### What gets capped

| What | Limit | Where |
|------|--------|--------|
| **Conversation history** (prior turns sent to the model) | Last **20** messages | Applied before planning/execution in the main router (`commands/ollama.rs`). Older messages are not sent; compaction can reduce them to one summary message. |

### When compaction runs

1. **On-request compaction**  
   When handling a request (Discord, agent router, **in-app CPU Ollama chat**, etc.), if the prior turns reach **≥ 8 messages** (last **20** capped), the app compacts before the main model call: those messages are sent to a small model to produce **CONTEXT** (summary) and optional **LESSONS**. **Discord** sessions are updated in `session_memory`; the CPU chat receives a compacted prior block in the request. **LESSONS** are appended to the appropriate memory file (`memory.md` / per-channel). If compaction fails, full history is kept for that turn.  
   - **In-app indicator:** While CPU chat compaction runs, the UI may show “Compacting context…” then briefly “Context compacted” via the **`mac-stats-compaction`** Tauri event (Discord unchanged).  
   - **Optional hooks / transcript:** **`beforeCompactionTranscriptPath`**, **`beforeCompactionHook`**, **`afterCompactionHook`** (and env overrides)—see **docs/data_files_reference.md** (compaction hooks).

2. **Periodic compaction (every 30 minutes)**  
   A background thread runs every 30 minutes and compacts each in-memory session that has **≥ 4 messages**. For each such session: lessons are appended to global memory; if the session has had **no activity for 30 minutes** it is **cleared**; if it is **active** (activity within 30 min) it is **replaced** with the summary (same as on-request).  
   See `run_periodic_session_compaction()` in `commands/compaction.rs` and the thread started from `lib.rs`.

3. **Discord having_fun channels**  
   For channels configured as having_fun, compaction does **not** call the LLM. A fixed minimal CONTEXT is stored so casual chat never gets summarized into task/platform themes. Log: `Session compaction: Discord having_fun channel <id> — using fixed minimal context (no LLM)`.

### Compaction performance

- **On-request**: One extra LLM call when history ≥ 8 messages; failure is non-fatal (full history kept for that request).
- **Periodic**: One compaction call per session that meets the threshold; runs in a background thread so it does not block the main flow.
- **New-topic**: When the new-topic check returns true, prior context is cleared and no compaction call is made for that turn.

### References

- **Full plan and behavior**: `docs/session_compaction_and_memory_plan_DONE.md`
- **Implementation**: `src-tauri/src/commands/compaction.rs` (`compact_conversation_history`, `run_periodic_session_compaction`), `session_history.rs` (`prepare_conversation_history`), `compaction_hooks.rs` (hooks + UI event), `ollama.rs` (router), `ollama_frontend_chat.rs` (CPU chat), `src/ollama.js` (indicator), `session_memory.rs` (`list_sessions`, `last_activity`)

---

## Quick Reference  
| Need | Action |  
|------|--------|  
| Focus on current question | Topic-aware compaction |  
| Fresh start | "Reset" phrases (multi-language) |  
| Use past lessons | Global + channel-specific memory |  
| Help on failed request | Search memory + append "From past sessions" |  
| Detect new topic | Local LLM check (no user input) |  
| Keep channels separate | Session/memory keyed by channel ID |  

---

## Open tasks

Memory/topic open tasks (per-channel memory, new-topic detection, pruning docs, multi-language reset, compaction) are tracked in **006-feature-coder/FEATURE-CODER.md**.

*Sibling repos: OpenClaw = `../openclaw`, Hermes = `../hermes-agent`*