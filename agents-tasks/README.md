# agents-tasks

Tasks created by the logfile scanner from `~/.mac-stats/debug.log` (cron every 10 minutes). Each task is dated and sourced from the log scan run.

## Summary

| Id       | Topic                        | Status | Created (UTC)     | Summary |
|----------|------------------------------|--------|-------------------|---------|
| log-001  | discord-fetch-url            | done   | 2026-02-23 09:55  | Discord FETCH_URL for discord.com: redirect to DISCORD_API or hint (ollama.rs FETCH_URL discord.com branch). |
| log-002  | log-rotation                 | done   | 2026-02-23 09:55  | debug.log 10MB truncation at startup (logging/mod.rs). |
| log-003  | temperature-unavailable      | done   | 2026-02-23 10:00  | Temperature: N/A when can_read but 0.0 (metrics format_metrics_for_ai_context). |
| log-004  | image-fetch-404              | done   | 2026-02-23 10:00  | sanitize_image_error_content in discord/mod.rs. |
| log-005  | operator-read-scope-error    | done   | 2026-02-23 10:00  | sanitize_discord_api_error in discord/api.rs, used in ollama.rs. |
| log-006  | duplicate-user-messages-ollama | done | 2026-02-23 10:00  | deduplicate_consecutive_messages in ollama.rs send_ollama_chat_messages. |
| log-007  | loop-protection-visibility   | done   | 2026-02-23 10:00  | Per-channel drop counter + DEBUG summary every 60s (discord/mod.rs). |
| log-008  | discord-fetch-url-redmine-hint | done | 2026-02-23 11:00  | FETCH_URL failure message + redmine_hint in ollama.rs agent router. |
| log-009  | redmine-api-422-updated-on   | done   | 2026-02-23 11:00  | Redmine 422 → friendly message (redmine/mod.rs redmine_api_request). |

## Scan run log

See `scan-log.md` for per-run details (scan time, tasks created, notes).

## Task format

- **Id**: log-NNN
- **Topic**: kebab-case identifier
- **Created**: date and time (UTC) when the task was created by the scanner
- **Source**: ~/.mac-stats/debug.log and scan date
