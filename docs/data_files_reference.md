# Data files reference: schedules.json, config heartbeat, user-info.json, session_reset_phrases.md

Short reference for key data files under `~/.mac-stats/` used by the scheduler, Discord agent, optional heartbeat, and session memory. For usage (SCHEDULE tool, Discord, session reset) see **docs/007_discord_agent.md**, **docs/009_scheduler_agent.md**, and **docs/019_agent_session_and_memory.md**.

---

## schedules.json

**Path:** `$HOME/.mac-stats/schedules.json`  
**Purpose:** Defines when and what the scheduler runs (cron-style or one-shot). The scheduler checks file mtime each loop and reloads when the file changes.

### JSON structure

- **Top-level key:** `schedules` (array of objects).
- **Each entry** must have exactly one of `cron` or `at` (recurring vs one-shot).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | No | Unique identifier (e.g. `discord-1770648842`). Used by REMOVE_SCHEDULE. |
| `cron` | string | One of cron/at | Cron expression (5 or 6 field). Example: `0 */5 * * * *` (every 5 minutes). |
| `at` | string | One of cron/at | One-shot run at this datetime (ISO format). Example: `2025-02-09T05:00:00` or `2025-02-09 05:00`. |
| `task` | string | Yes | Task description passed to Ollama (e.g. "Check CPU and reply here"). |
| `reply_to_channel_id` | string | No | Discord channel ID; when set, scheduler posts the task result to this channel. |

### Time interpretation

- **Cron** and **at** are interpreted in **local time** (system timezone). The scheduler uses `chrono::Local` for next-run calculation and for parsing `at` (RFC3339 or `%Y-%m-%dT%H:%M:%S` without timezone = local).

### Example

```json
{
  "schedules": [
    {
      "id": "discord-1770648842",
      "cron": "0 */5 * * * *",
      "task": "Check CPU and reply here",
      "reply_to_channel_id": "1234567890123456789"
    },
    {
      "id": "reminder-1",
      "at": "2025-03-10T09:00:00",
      "task": "Remind me of my meeting"
    }
  ]
}
```

### Limits and behavior

- **maxSchedules:** Optional cap in `~/.mac-stats/config.json` (e.g. `"maxSchedules": 20`). When the number of entries reaches this limit, new SCHEDULE requests are rejected. Omit or `0` = no limit (clamped 1–1000).
- **Deduplication:** Adding a schedule with the same `cron` (or same `at` for one-shot) and same task (after normalizing whitespace) as an existing entry is treated as duplicate and not added again.
- **Empty task:** Entries with empty `task` are skipped at load time.

---

## config.json — `heartbeat` (optional)

**Path:** `$HOME/.mac-stats/config.json` (fragment; other keys unchanged)  
**Purpose:** Enable a periodic **heartbeat** agent turn (checklist + `HEARTBEAT_OK` silent ack). Independent of `schedules.json`.

### JSON shape

Top-level object key `heartbeat` (optional):

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | When true, the heartbeat thread runs. |
| `intervalSecs` | number | `1800` | Seconds between runs (clamped 60–86400). Overridden by env `MAC_STATS_HEARTBEAT_INTERVAL_SECS` when set. |
| `checklistPath` | string | omit | Path to markdown checklist (`~/…` allowed). If missing or unreadable, `checklistPrompt` or a built-in default is used. |
| `checklistPrompt` | string | omit | Inline checklist text when no file or as fallback after file read failure. |
| `replyToChannelId` | string | omit | Discord channel snowflake for non-ack replies. Omit or empty = log only (no Discord post). |
| `ackMaxChars` | number | `300` | If the reply starts or ends with `HEARTBEAT_OK` and the remaining text (trimmed) is at most this many characters, the reply is treated as a silent ack (no delivery). Clamped 0–2000. |

### Example

```json
{
  "heartbeat": {
    "enabled": true,
    "intervalSecs": 1800,
    "checklistPath": "~/.mac-stats/HEARTBEAT.md",
    "replyToChannelId": "1234567890123456789",
    "ackMaxChars": 300
  }
}
```

**Operator checklist:** Put short, actionable bullets in `HEARTBEAT.md` (e.g. email, calendar, mentions). Instruct the model to only surface real alerts; otherwise `HEARTBEAT_OK`.

---

### Data structure and performance (schedules.json)

Schedules are stored as a **JSON array** for simplicity, human readability, and backward compatibility. Operations that look up by `id` (e.g. REMOVE_SCHEDULE) or check for duplicates (add_schedule / add_schedule_at) do a single pass over the array (O(n)). Typical usage keeps N small (tens of schedules, capped by maxSchedules). For that scale, O(n) is acceptable and no in-memory index or file-format change is required. If we ever need O(1) lookup by id at larger scale, options would be: (a) build a HashMap by id after parsing and use it only in memory, or (b) migrate the file format to an object keyed by id, with a one-time migration for existing array files. No change is implemented at this time.

---

## scheduler_delivery_awareness.json

**Path:** `$HOME/.mac-stats/scheduler_delivery_awareness.json`  
**Purpose:** Append-only style log (bounded list, newest kept) of **successful** scheduler posts to Discord when `reply_to_channel_id` is set. The in-app CPU window Ollama chat injects a short summary of recent rows into the **system** prompt so the primary on-device conversation stays aligned with what was already sent to a channel. **Authoritative** user-visible delivery remains Discord; this file is for continuity and deduplication (per-run `context_key`), not a second outbound path.

### JSON structure

- **Top-level key:** `entries` (array of objects, oldest first on disk).
- **Each entry:** `context_key` (unique per successful delivery attempt), `utc` (RFC3339 UTC), optional `schedule_id`, `channel_id` (string), `summary` (truncated body that was posted).

### When rows are written

Only after Discord accepts the final user-visible message for that run (scheduler loop or task runner). Internal-only runs (e.g. `FETCH_URL` / `BRAVE_SEARCH` with no Discord post) do not add entries. Duplicate `context_key` values are ignored (idempotent).

---

## user-info.json

**Path:** `$HOME/.mac-stats/user-info.json`  
**Purpose:** Per-user details (e.g. Discord user id → display name, notes, timezone). Read when building context for a message; updated when a user messages the bot (display name sync).

### JSON structure

- **Top-level key:** `users` (array of objects).
- **Each entry:** `id` (required) plus optional fields from the table below.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | User identifier (e.g. Discord snowflake as string). |
| `display_name` | string | No | Override or stored name. When the user messages via Discord, the app updates this from the author’s display name if it differs. |
| `notes` | string | No | Free-form notes (e.g. preferences). |
| `timezone` | string | No | Timezone for time-sensitive answers (e.g. `Europe/Paris`). |
| `extra` | object | No | Key-value pairs for future use. |

### When the file is read and written

- **Read:** When handling a Discord (or other) message; if the author’s id is in the file, the bot merges "User details: …" into the agent context. Reads use an in-memory cache; the cache is invalidated when the file's modification time changes (e.g. external edit) or after a write.
- **Written:** When a user messages the bot, the app updates (or adds) their `display_name` from Discord so the file stays in sync. New users get a minimal entry with `id` and `display_name`. After a write, the cache is refreshed so the next read sees the new data without re-reading from disk.

### Example

```json
{
  "users": [
    {
      "id": "123456789012345678",
      "display_name": "Alice",
      "notes": "Prefers short answers.",
      "timezone": "Europe/Paris",
      "extra": { "language": "en" }
    }
  ]
}
```

---

## session_reset_phrases.md

**Path:** `$HOME/.mac-stats/agents/session_reset_phrases.md`  
**Purpose:** When a user message contains any of these phrases (case-insensitive substring), the app clears the session for that channel and starts fresh (Session Startup instruction + current date/time is injected). Used by the Discord handler and session memory; similar to OpenClaw’s session reset triggers, but in a simple text file.

### Format

- **One phrase per line.** Empty lines and lines starting with `#` are ignored.
- **Matching:** Case-insensitive substring: if the user message contains the phrase anywhere, the session is reset.
- **Default:** The app ships a default file with phrases in English, German, Spanish, French, Italian, Portuguese, and Dutch (e.g. “new topic”, “reset”, “clear session”, “neue sitzung”, “nueva sesión”, etc.). Users can add or remove lines.
- **Fallback:** If the file is missing or yields no phrases, a built-in list in `session_memory.rs` is used so reset still works.

### See also

- **docs/019_agent_session_and_memory.md** — Session Startup, session reset behavior.
- **docs/035_memory_and_topic_handling.md** — Memory and topic handling overview.

---

## Before-reset transcript export (optional)

**Purpose:** Immediately before a **user-triggered** Discord session clear (same triggers as `session_reset_phrases.md` or a leading `new session:` / `new session ` prefix), mac-stats can write the current conversational history to a JSONL file and optionally run a shell hook. This mirrors OpenClaw’s “before reset” idea for backups, memory extraction, or custom tooling—without blocking the reset.

### Configuration

- **`beforeResetTranscriptPath`** in `~/.mac-stats/config.json` — non-empty path for the JSONL file (`~/...` allowed). Env override: **`MAC_STATS_BEFORE_RESET_TRANSCRIPT_PATH`**.
- **`beforeResetHook`** in `config.json` — optional shell command run **after** a successful write, in a **background thread** (does not delay the bot). The transcript’s absolute path is **`$1`** and is also in env **`MAC_STATS_BEFORE_RESET_TRANSCRIPT`**. Additional env: **`MAC_STATS_BEFORE_RESET_REASON`** (`new_session_prefix` or `session_reset_phrase`), **`MAC_STATS_BEFORE_RESET_SOURCE`**, **`MAC_STATS_BEFORE_RESET_SESSION_ID`**. Env override: **`MAC_STATS_BEFORE_RESET_HOOK`**.
- If only the hook is set (no transcript path), the file defaults to **`$HOME/.mac-stats/agents/last_session_before_reset.jsonl`**.

### File format (JSONL)

- First line: metadata object with `"kind":"before_reset_meta"`, `source`, `session_id`, `reason`, `exported_at_utc` (RFC3339 UTC), `message_count`.
- Following lines: one JSON object per message: `{"role":"user"|"assistant","content":"..."}` (same conversational filtering as session memory; internal artifacts omitted).

Writes are best-effort: failures are logged; the session still clears. The hook is invoked as `/bin/sh -c '<your command> \"$1\"' _ <absolute-path>`.

### See also

- **docs/035_memory_and_topic_handling.md** — User-initiated reset behavior.

---

## Compaction hooks and before-compaction transcript (optional)

**Purpose:** Around each **session compaction** run (on-request router path, in-app CPU Ollama chat when history ≥ 8, and the 30-minute periodic pass), mac-stats can write a JSONL snapshot **before** the compactor runs and/or run **fire-and-forget** shell commands. After a **successful** compaction, an optional **after** hook runs (not on failure or skip). Hooks do not block compaction; failures are logged only—same spirit as before-reset export.

### Configuration (`~/.mac-stats/config.json` and env)

| Key | Env override |
|-----|----------------|
| **`beforeCompactionTranscriptPath`** | **`MAC_STATS_BEFORE_COMPACTION_TRANSCRIPT_PATH`** |
| **`beforeCompactionHook`** | **`MAC_STATS_BEFORE_COMPACTION_HOOK`** |
| **`afterCompactionHook`** | **`MAC_STATS_AFTER_COMPACTION_HOOK`** |

If **`beforeCompactionHook`** is set but no transcript path is configured, the default file is **`$HOME/.mac-stats/agents/last_session_before_compaction.jsonl`**.

### Before hook

- Invoked as `/bin/sh -c '<command> \"$1\"' _ <absolute-transcript-path>` after a successful write (same pattern as before-reset).
- Env: **`MAC_STATS_BEFORE_COMPACTION_TRANSCRIPT`**, **`MAC_STATS_BEFORE_COMPACTION_SOURCE`**, **`MAC_STATS_BEFORE_COMPACTION_SESSION_ID`**, **`MAC_STATS_BEFORE_COMPACTION_MESSAGE_COUNT`**, **`MAC_STATS_BEFORE_COMPACTION_REQUEST_ID`**.

JSONL first line metadata: `"kind":"before_compaction_meta"`, `source`, `session_id`, `request_id`, `exported_at_utc`, `message_count`; following lines are `role` / `content` objects for the messages about to be compacted.

### After hook (success only)

- No file argument; command runs with env only: **`MAC_STATS_AFTER_COMPACTION_SOURCE`**, **`MAC_STATS_AFTER_COMPACTION_SESSION_ID`**, **`MAC_STATS_AFTER_COMPACTION_MESSAGE_COUNT_BEFORE`**, **`MAC_STATS_AFTER_COMPACTION_LESSONS_WRITTEN`** (`true` / `false`), **`MAC_STATS_AFTER_COMPACTION_REQUEST_ID`**.
- Not invoked for **Discord having_fun** channels (fixed minimal context, no LLM compaction), so casual chat does not trigger post-compaction scripts. The **before** hook still runs for those sessions when configured.

### CPU window UI

The in-app chat listens for the Tauri event **`mac-stats-compaction`** (payload shape `{ "stream": "compaction", "data": { "phase": "start"|"end", "willRetry", "requestId", "ok"? } }`) to show a short “Compacting context…” / “Context compacted” line. **Discord** does not use this indicator.

### See also

- **docs/035_memory_and_topic_handling.md** — When compaction runs.

---

## Memory files (agents)

**Paths:**  
- **Global:** `$HOME/.mac-stats/agents/memory.md` — loaded only in main session (in-app CPU window or Discord DM).  
- **Main session:** `$HOME/.mac-stats/agents/memory-main.md` — loaded when the request is from the in-app CPU window (no Discord channel), so the main session has its own persistent memory like Discord channels.  
- **Per-channel (Discord):** `$HOME/.mac-stats/agents/memory-discord-{channel_id}.md` — loaded when replying in that Discord channel or DM.

**Purpose:** Inject lessons and context into the agent. Global memory is personal/long-term; main and per-channel memory keep context separate (in-app vs each Discord channel). Memory search (“From past sessions”) uses global + main when in-app, or global + channel when Discord.

**See:** **docs/035_memory_and_topic_handling.md**.

---

## credential_accounts.json

**Path:** `$HOME/.mac-stats/credential_accounts.json`  
**Purpose:** Persisted list of Keychain credential account names (e.g. `discord_bot_token`, `mastodon_api_key`). The security module updates this file on store/delete so `list_credentials()` can return account names without Keychain attribute enumeration. **Do not edit by hand**; the app maintains it. Format: JSON array of strings.

---

## See also

- **docs/007_discord_agent.md** — SCHEDULE/REMOVE_SCHEDULE, user-info in context, maxSchedules.
- **docs/009_scheduler_agent.md** — Scheduler behavior, cron formats, file path.
