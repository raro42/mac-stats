# Agent session context and memory (short-term / long-term)

## Current state

- **CPU window**: Frontend sends `conversation_history` to `ollama_chat_with_execution`; the main chat has multi-turn context within the request.
- **Discord**: Calls `answer_with_ollama_and_fetch` with only the current `question`. **No conversation_history** is passed. `session_memory::add_message` stores each user/assistant message in memory and persists to `~/.mac-stats/session/session-memory-*.md` when there are more than 3 messages, but that stored history is **never loaded back** into the next Ollama request. So each Discord turn is stateless from the model's point of view.
- **SKILL / AGENT sub-calls**: `run_skill_ollama_session` and (planned) `run_agent_ollama_session` are **one-shot**: system prompt + single user message, no prior messages.

---

## 1. Session context (who sees what)

| Scope | What gets context | How |
|-------|-------------------|-----|
| **Main conversation** (user ↔ entry agent, e.g. orchestrator) | The **entry agent** sees the ongoing conversation: current question + prior user/assistant messages in that session. | Pass **conversation_history** (with a cap) into the pipeline that builds the planning and execution prompts. |
| **AGENT: sub-calls** (orchestrator → specialist) | The **specialist** sees only the **task** string and its own system prompt (soul + mood + skill). No main-conversation history. | Keep `run_agent_ollama_session(agent, user_message)` one-shot. Optionally later: append a short "Context from main conversation: …" (e.g. last user message or last N chars) to the task when invoking the agent. |
| **Session identity** | Per entry point: CPU window = one logical session (frontend owns history); Discord = one session per channel (or per channel+user if you want). | Session id: CPU = e.g. window id or "cpu"; Discord = channel id. Use for short-term and (if needed) long-term memory keys. |

So: **session context** = the list of prior messages in the **main** conversation only. Sub-agents do not get that list unless we explicitly add a small context snippet later.

---

## 2. Short-term memory

- **Definition**: The recent messages in the current session (current request + prior turns in the same channel/window).
- **Implementation**:
  - **CPU window**: Already has short-term memory via `conversation_history` from the frontend. Optionally cap (e.g. last 20 messages or last 4k tokens) to avoid overflow.
  - **Discord**: Add **loading** of short-term memory before calling Ollama:
    1. **Expose** a function in `session_memory` to return the current in-memory messages for a key (e.g. `get_messages(source, session_id) -> Vec<(role, content)>`). If you persist and then clear in-memory state, optionally add "load latest session file for this channel" so that after app restart the first request in a channel can still get recent history.
    2. **Before** `answer_with_ollama_and_fetch`, call `session_memory::get_messages("discord", channel_id)` (or equivalent). Convert to `Vec<ChatMessage>`, apply a **cap** (e.g. last 20 messages or last 8k tokens), and pass as **conversation_history**.
    3. **Extend** `answer_with_ollama_and_fetch` to accept an optional `conversation_history: Option<Vec<ChatMessage>>`. When present, prepend those messages (in order) to the planning and execution prompts so the model sees the recent conversation. Cap total history (message count or token count) in one place to avoid exceeding model context.
  - **Scheduler / other entry points**: If they ever call the same pipeline, pass `None` for history or their own session key and load logic.
- **Cap**: Define a single place (e.g. last N messages or M tokens). Suggest N = 20 messages or M = 8192 tokens as a default; document so it can be tuned.

---

## 3. Long-term memory

- **Definition**: Information that persists across sessions (restarts, different days) and can be injected into the agent's context.
- **Existing**: Session files in `~/.mac-stats/session/session-memory-<topic>-<sessionid>-<timestamp>.md` are already long-term persistence of a **conversation transcript**, but they are not currently loaded to resume context. So "long-term" today = archive only.
- **Optional: resume session**: For Discord, optionally on first message in a channel after restart, load the **most recent** session file for that channel (match by session_id in filename) and use it to seed short-term memory (e.g. last K messages from that file). That gives continuity across restarts without a separate "long-term memory" store.
- **Optional: semantic long-term memory**: A separate store for facts, preferences, or summaries (e.g. "User prefers brief answers"; "Project X is in Python"). Not in current scope; if added later:
  - **Where**: e.g. `~/.mac-stats/memory/` with per-user or per-agent files (e.g. `user-<discord_id>.json` or `agent-<id>-memory.md`).
  - **Format**: Simple (e.g. JSON array of `{ "fact": "...", "updated": "..." }` or a markdown list).
  - **When**: Load at start of request, inject into system prompt or a "Long-term context" block so the entry agent (and optionally sub-agents) see it. Updates could be manual (user edits file) or via a future "remember this" tool that appends to the file.
  - **Scope**: Document as optional/future; no implementation required for the first agent rollout.

---

## 4. Summary table

| Type | What it is | Where it lives | How agents use it |
|------|------------|----------------|-------------------|
| **Session context** | Current conversation turn + prior messages in same session | In-request (CPU: frontend; Discord: session_memory in-memory + optional load from session file) | Passed as `conversation_history` to the **main** agent only; sub-agents (AGENT:) get task only. |
| **Short-term memory** | Recent messages in session (capped) | session_memory (Discord); frontend (CPU) | Capped list of (role, content) passed as conversation_history. |
| **Long-term memory** | Persisted transcript (session files); optional semantic facts | `~/.mac-stats/session/*.md`; optional `~/.mac-stats/memory/` | Session files: optional reload to seed short-term. Semantic: optional inject into system prompt (future). |

---

## 5. Implementation order (for session + short-term)

1. **session_memory**: Add `get_messages(source, session_id) -> Option<Vec<(String, String)>>` (or equivalent) returning the current in-memory messages for that key. If you later want "resume from file", add a function to load the latest session file for a given source+session_id and return messages.
2. **answer_with_ollama_and_fetch**: Add parameter `conversation_history: Option<Vec<ChatMessage>>`. When building the planning and execution message lists, prepend these messages (after the system block, in chronological order) and apply the chosen cap (e.g. last 20 messages).
3. **Discord**: Before calling `answer_with_ollama_and_fetch`, call `session_memory::get_messages("discord", channel_id)`, convert to `Vec<ChatMessage>`, cap, and pass as `conversation_history`.
4. **AGENT: sub-calls**: Leave `run_agent_ollama_session(agent, task)` one-shot (no main-conversation history). Optionally later: append a one-line "Context: &lt;last user message&gt;" to the task string when invoking.

No change to CPU window flow except possibly a cap on history length if the frontend sends a very long list.

---

## 6. Behaviour after changes

- **Discord**: Each request sees the last N messages in that channel (from in-memory, and after restart optionally from the latest session file). The model can refer to what was said earlier in the channel.
- **CPU window**: Unchanged; frontend continues to send conversation_history; optionally cap in the backend.
- **AGENT: calls**: Specialist still gets only the task string and its own prompt; no change unless you add an optional context snippet later.

This gives clear rules for session context and short-term memory and a path for optional long-term memory without implementing it in the first iteration.
