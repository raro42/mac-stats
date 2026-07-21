# Send finished task summary back to the requesting channel

## Goal

When mac-stats starts a new task and the request came from a channel (e.g. Discord), we send the **finished task summary** back to that channel. The input flow can come from any channel (Discord today; WhatsApp, Slack, etc. later).

## Design

1. **Task file stores reply-to**  
   When a task is created from Discord (`TASK_CREATE` with `discord_reply_channel_id` set), we write `## Reply-to: discord <channel_id>` into the task file. Format is extensible (e.g. `whatsapp ...`, `slack ...` later).

2. **Runner sends when task finishes**  
   `run_task_until_finished` accepts an optional `reply_to_discord_channel` (from the caller, e.g. scheduler) and an optional `message_prefix` (e.g. `[Schedule: id] `). When the task finishes (or was already finished), if we have a channel — from the override or from the task file’s `## Reply-to: discord <id>` — we send a single message: `Task finished.\n\n{summary}` (with optional prefix) to that Discord channel.

3. **Scheduler**  
   When the scheduler runs a `TASK:` or `TASK_RUN:` entry that has `reply_to_channel_id`, it passes that channel and a schedule prefix to the runner. The runner sends the finished summary; the scheduler does **not** send again (`already_sent_to_discord`).

4. **Review loop / other callers**  
   When the task runner is invoked without a channel (e.g. review loop, CLI), it still checks the task file for `## Reply-to: discord <id>`. If present (task was created from Discord), the runner sends the summary when the task finishes.

## Code

- **Task module** (`task/mod.rs`): `create_task(..., reply_to_discord_channel: Option<u64>)`, `get_reply_to_discord_channel(path) -> Option<u64>`.
- **Ollama TASK_CREATE** (`commands/ollama.rs`): passes `discord_reply_channel_id` into `create_task` when the request is from Discord.
- **Task runner** (`task/runner.rs`): `run_task_until_finished(..., reply_to_discord_channel, message_prefix)`; on success sends via `send_finished_summary_if_channel` and returns `(summary, already_sent_to_discord)`.
- **Scheduler** (`scheduler/mod.rs`): passes channel and prefix to the runner; only sends from the loop when `!already_sent_to_discord`.

## Extending to other channels

The task file format `## Reply-to: discord 123` can be extended with other keys (e.g. `whatsapp`, `slack`). The runner currently only acts on `discord`; future work would branch on the key and call the right send API for that channel.
