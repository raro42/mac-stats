# Feature health dashboard (backend)

mac-stats collects a **structured subsystem health report** after startup (about 2 seconds in, so Discord and Ollama can initialize) and stores it for the UI.

## Subsystems probed

| Name | What is checked |
|------|-----------------|
| Ollama | Client configured; `GET /api/tags` within timeout; configured model appears in the list |
| Discord | Bot token present (env / `.config.env` / Keychain); gateway received `Ready`, or **Degraded** with stage-aware text (first connect vs reconnect via Serenity `ShardStageUpdate` + client start time) |
| Browser (CDP) | Default Chrome path exists; `http://127.0.0.1:9222/json/version` responds |
| Brave Search | `BRAVE_API_KEY` present (env or `.config.env`) |
| Redmine | `REDMINE_URL` + `REDMINE_API_KEY`; `GET .../users/current.json` |
| SMC (temperature) | `Smc::connect()` within timeout |
| IOReport (CPU frequency) | CPU performance-state channel group visible to IOReport |
| Scheduler | Count of valid entries in `~/.mac-stats/schedules.json` |

## Logs

Lines are prefixed with `feature_health:` at **INFO** in `~/.mac-stats/debug.log` (one line per subsystem plus header/footer).

## Tauri API

- Command: `get_feature_health`
- Argument: optional `refresh` (boolean). If `true`, probes run again and the cache is updated. If the cache is empty, probes run on first read.

Types are JSON-serializable: `HealthStatus` (`ok` \| `degraded` \| `unavailable` \| `notConfigured`) and `FeatureHealth` (`name`, `status`, `message`, `checkedAt`).

## Implementation

- `src-tauri/src/feature_health.rs` — probes, cache, logging, command
- `src-tauri/src/ffi/ioreport.rs` — `probe_cpu_performance_channels_available()`
- `src-tauri/src/discord/mod.rs` — `discord_bot_token_configured()`, `discord_bot_gateway_ready()`, `discord_last_shard_stage()`, `discord_gateway_client_started_at()`; logs shard transitions under `mac_stats::discord/gateway`
- `src-tauri/src/scheduler/mod.rs` — `schedule_entry_count()`
