# Data files reference: schedules.json and user-info.json

Short reference for the JSON files under `~/.mac-stats/` used by the scheduler and Discord agent. For usage (SCHEDULE tool, Discord, etc.) see **docs/007_discord_agent.md** and **docs/009_scheduler_agent.md**.

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
- **Deduplication:** Adding a schedule with the same `cron` and same task (after normalizing whitespace) as an existing entry is treated as duplicate and not added again.
- **Empty task:** Entries with empty `task` are skipped at load time.

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

- **Read:** When handling a Discord (or other) message; if the author’s id is in the file, the bot merges "User details: …" into the agent context.
- **Written:** When a user messages the bot, the app updates (or adds) their `display_name` from Discord so the file stays in sync. New users get a minimal entry with `id` and `display_name`.

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

## See also

- **docs/007_discord_agent.md** — SCHEDULE/REMOVE_SCHEDULE, user-info in context, maxSchedules.
- **docs/009_scheduler_agent.md** — Scheduler behavior, cron formats, file path.
