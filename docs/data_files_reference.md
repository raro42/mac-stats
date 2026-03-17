# Data files reference: schedules.json, user-info.json, session_reset_phrases.md

Short reference for key data files under `~/.mac-stats/` used by the scheduler, Discord agent, and session memory. For usage (SCHEDULE tool, Discord, session reset) see **docs/007_discord_agent.md**, **docs/009_scheduler_agent.md**, and **docs/019_agent_session_and_memory.md**.

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

### Data structure and performance

Schedules are stored as a **JSON array** for simplicity, human readability, and backward compatibility. Operations that look up by `id` (e.g. REMOVE_SCHEDULE) or check for duplicates (add_schedule / add_schedule_at) do a single pass over the array (O(n)). Typical usage keeps N small (tens of schedules, capped by maxSchedules). For that scale, O(n) is acceptable and no in-memory index or file-format change is required. If we ever need O(1) lookup by id at larger scale, options would be: (a) build a HashMap by id after parsing and use it only in memory, or (b) migrate the file format to an object keyed by id, with a one-time migration for existing array files. No change is implemented at this time.

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

## Memory files (agents)

**Paths:**  
- **Global:** `$HOME/.mac-stats/agents/memory.md` — loaded only in main session (in-app CPU window or Discord DM).  
- **Main session:** `$HOME/.mac-stats/agents/memory-main.md` — loaded when the request is from the in-app CPU window (no Discord channel), so the main session has its own persistent memory like Discord channels.  
- **Per-channel (Discord):** `$HOME/.mac-stats/agents/memory-discord-{channel_id}.md` — loaded when replying in that Discord channel or DM.

**Purpose:** Inject lessons and context into the agent. Global memory is personal/long-term; main and per-channel memory keep context separate (in-app vs each Discord channel). Memory search (“From past sessions”) uses global + main when in-app, or global + channel when Discord.

**See:** **docs/035_memory_and_topic_handling.md**.

---

## See also

- **docs/007_discord_agent.md** — SCHEDULE/REMOVE_SCHEDULE, user-info in context, maxSchedules.
- **docs/009_scheduler_agent.md** — Scheduler behavior, cron formats, file path.
