# Memory and topic handling

How mac-stats keeps context focused when conversations mix topics or switch threads — and how that compares to OpenClaw and Hermes.

---

## How others do it

**OpenClaw** (sibling repo: `../openclaw`) focuses on *size* and *user control*:

- **Compaction:** Summarises older conversation into a compact summary (stored in JSONL). Triggered when context nears the model window; no “current question” or topic in the summarisation — it’s generic. User can run **`/compact`** manually.
- **Fresh start:** **`/new`** or **`/reset`** starts a new session id (no prior context).
- **Pre-compaction memory flush:** Before compaction, a silent turn writes durable notes to e.g. `memory/YYYY-MM-DD.md` so they survive summarisation.
- **Session pruning:** Trims *tool results only* (for Anthropic cache TTL); user/assistant messages stay. We don’t have separate tool-result messages, so that pattern doesn’t apply to us.

**Hermes-agent** (`../hermes-agent`) is a skills/workspace repo (SKILL.md). It doesn’t implement session or memory logic; any comparison is with whatever consumes those skills (e.g. OpenClaw).

---

## What mac-stats does (and how it works)

Everything below is **implemented** and in use.

### 1. Topic-aware compaction

When history has **8+ messages**, we compact into a short summary. The compactor is told:

- Summarise only what’s **relevant to the user’s current question**.
- If the question is clearly a **new topic**, output: *“Previous context covered different topics; not needed for this request.”*

When we see that phrase, we inject **no** prior context for that run — the model gets only the current question. So mixed threads don’t pollute the reply.

### 2. User-initiated reset

Say things like **“clear session”**, **“new topic”**, **“start over”**, **“reset”** (and equivalents in German, Spanish, French, Italian, Portuguese, Dutch). We clear the session for that channel and reply with a fresh context. Your message is still processed; we just drop the old conversation. See `docs/007_discord_agent.md` for the full phrase list.

### 3. Memory when we don’t get a result

- **Every run:** The default (non-agent) path loads memory and injects it into the system prompt. **Global** lessons live in `~/.mac-stats/agents/memory.md`. In **Discord** we also load **per-channel** memory from `~/.mac-stats/agents/memory-discord-{channel_id}.md`, so the model sees both global and this channel's lessons.
- **When verification says "not satisfied":** We search memory (global + current channel when in Discord) by keywords, take the top 5 matching lines, and either (a) feed them into a retry or (b) append "From past sessions" to the reply.

### 4. New-topic check (local only)

When we have **2+ prior messages** and the model is **local** (not cloud), we run one short LLM call: *“Is the user starting a new topic (NEW_TOPIC) or continuing the same thread (SAME_TOPIC)?”* If the answer is **NEW_TOPIC**, we use no prior context for that request. We skip this check on cloud models to avoid extra cost.

### 5. No tool-result pruning (by design)

We don’t trim “tool results” separately — our tool output lives inside assistant/user messages. Compaction and topic handling (above) are how we keep context under control. No pruning layer; that's intentional.

### 6. Per-channel memory (Discord)

**Werner does not mix context or memory between channels.** Conversation history is already keyed by channel (and DMs are a channel). **Persistent memory** is now per channel too:

- **Conversation context:** Each channel (and each DM) has its own session and compacted summary. Nothing from #general is sent when replying in DMs or in #random.
- **Lesson memory:** When replying in a Discord channel (or DM), we load **global** `memory.md` plus **channel** `memory-discord-{channel_id}.md`. Lessons from compaction and **MEMORY_APPEND** in that channel are written to the channel file. So DMs stay private; each server channel has its own lesson file. No cross-channel leakage.

---

## Quick reference

| Need | What we do |
|------|------------|
| Focus on current question | Topic-aware compaction; “not needed” → no prior context |
| Fresh start | Phrases like “clear session”, “new topic”, “reset” (multi-language) |
| Use past lessons every time | Global `memory.md` + per-channel `memory-discord-{id}.md` in Discord |
| Help when we can’t satisfy the request | Search memory on verification “no”; inject into retry or append “From past sessions” |
| Detect new topic without user saying so | One short LLM check (local model only); NEW_TOPIC → empty history |
| Keep channels separate | Session and memory keyed by channel id; no mixing between DMs and channels |

---

*Sibling repos: OpenClaw = `../openclaw`, Hermes = `../hermes-agent`. Happy chatting!*
