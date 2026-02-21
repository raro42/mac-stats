# Scheduler Agent

The scheduler agent runs tasks at scheduled times. It reads `~/.mac-stats/schedules.json` at startup and periodically reloads it. When a scheduled time is due, it executes the task using Ollama + tools (or runs FETCH_URL/BRAVE_SEARCH directly if the task is a tool line).

## File location

- **Path:** `$HOME/.mac-stats/schedules.json`
- The directory `~/.mac-stats/` is created automatically if missing. Create or edit the file by hand (no UI in v1).

## Format

```json
{
  "schedules": [
    {
      "id": "daily-weather",
      "cron": "0 0 9 * * *",
      "task": "Check the weather for today and summarize in one sentence"
    },
    {
      "id": "reminder",
      "at": "2025-02-09T18:00:00",
      "task": "Remind: call John"
    }
  ]
}
```

- **id** (optional): Label for logging; useful for debugging.
- **cron** (recurring): 6-field cron expression: second, minute, hour, day of month, month, day of week. Times are interpreted in **local time**. Example: `0 0 9 * * *` = 09:00:00 every day.
- **at** (one-shot): Single run at the given datetime. Use ISO 8601 (e.g. `2025-02-09T18:00:00` or with timezone). Also accepts local format `YYYY-MM-DDTHH:MM:SS`. If the time is in the past when the file is loaded, that entry is skipped (no next run).
- **task** (required): What to run.
  - **Free text:** Passed to Ollama as the question; the app uses the same pipeline as the Discord agent (planning + FETCH_URL/BRAVE_SEARCH/RUN_JS loop). Ollama decides which agents to use.
  - **Direct tool:** If the task string starts with `FETCH_URL: <url>` or `BRAVE_SEARCH: <query>`, that tool is run directly (no Ollama call). Useful for simple recurring fetches or searches.
- **reply_to_channel_id** (optional): Discord channel ID (numeric string). When set, the scheduler sends the task result (Ollama reply) to that channel when the task runs. Used when a schedule was created from Discord (DM or channel mention) so the user sees the result in the same place they asked.

Each entry must have exactly one of `cron` or `at`. Invalid entries are skipped and a warning is logged. **Deduplication:** If you add a schedule with the same `cron` and same `task` (after normalizing whitespace) as an existing entry, the add is skipped and no duplicate is created.

## Check interval (reload frequency)

How often the scheduler reloads `schedules.json` is configurable. Default: **every 60 seconds** (one minute).

- **Config file:** `~/.mac-stats/config.json` — add `"schedulerCheckIntervalSecs": 120` (number, seconds). Clamped to 1–86400.
- **Env override:** `MAC_STATS_SCHEDULER_CHECK_SECS=30` (takes precedence over config.json).

The scheduler never sleeps longer than this interval before re-reading the file, so new or changed entries are picked up within at most one interval.

## Behaviour

- The scheduler runs in a background thread started when the app starts (same as the Discord gateway).
- It loads the schedule file, computes the next run time for each entry (cron = next match in local time; at = that time if in the future), and sleeps until the soonest time (capped by the check interval above so the file is reloaded regularly).
- When a time is due, it runs that entry’s task (Ollama or direct tool). If the entry has `reply_to_channel_id` (e.g. from Discord), the result is posted to that channel (DM or the channel where the user was mentioned).
- Errors (Ollama down, fetch failure, etc.) are logged; the loop continues. One-shot entries with `at` in the past never run and are skipped on each load.

## References

- **Code:** `src-tauri/src/scheduler/mod.rs`
- **List schedules:** When the user or agent asks to "list schedules", the model can invoke **LIST_SCHEDULES** (or **LIST_SCHEDULES:**). The app returns a formatted list of active schedules (id, cron/one-shot, next run time, task preview). See agent tools in `commands/ollama.rs` and `scheduler::list_schedules_formatted()`.
- **Config:** `Config::schedules_file_path()`, `Config::ensure_schedules_directory()`, `Config::scheduler_check_interval_secs()` in `config/mod.rs`
- **All agents:** `docs/100_all_agents.md`
