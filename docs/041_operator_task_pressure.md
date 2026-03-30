# Operator automation pressure summary

mac-stats exposes a single JSON snapshot (Tauri command `get_operator_task_pressure_summary`, Settings → **Schedules** → *Automation pressure*) so operators can answer “is automation backed up?” without tailing long logs.

## Fields (high level)

| Section | Meaning |
|--------|---------|
| `scheduler` | Parsed `~/.mac-stats/schedules.json`: how many entries exist, how many have a computable next run, seconds until the soonest fire, and how many fires are within the next 120 seconds (local time). **Not** “scheduler is running a job right now,” and **not** missed runs while the app was stopped. |
| `ollamaHttpQueue` | Global permit pool + per-key FIFO for `/api/chat` when the queue is acquired. `waiters` = other requests waiting on that key. **Skipped** when `skip_ollama_queue` is used. |
| `sessionKeyedQueue` | Full-turn serialization per Discord channel / task session. `busySessionKeys` = mutex held (another turn may be waiting). **No** per-key waiter depth. |
| `taskFiles` | Markdown tasks under `~/.mac-stats/task/` by filename status (`open`, `wip`, …). |
| `ollamaRouterErrorCounts` | Cumulative counts by error **code** since process start (not a sliding window). |

## Logs

With non-trivial activity (queues, WIP tasks, imminent schedules, or any router error count), the app emits roughly every 90s:

`target`: `mac_stats::operator_task_pressure`, message `operator automation pressure snapshot`, field `summary` = one compact line.

Related subsystems: `ollama/queue` (per-request), scheduler loop (`Scheduler:`), task review (`Task review:`).
