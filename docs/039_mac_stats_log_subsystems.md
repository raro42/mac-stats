# Subsystem-scoped console logging (`MAC_STATS_LOG`)

mac-stats uses `tracing` with a shared log file (`~/.mac-stats/debug.log`) and stderr. Global verbosity is still controlled only by `-v` / `-vv` / `-vvv` (the app ignores `RUST_LOG` for the main filter).

## `MAC_STATS_LOG`

Optional **console-only** filter: comma-separated subsystem names (case-insensitive). When set:

- **Stderr** shows only events whose `tracing` target is under `mac_stats::<name>` or a child path `mac_stats::<name>/…` (e.g. `mac_stats::browser/cdp` matches `browser` and `browser/cdp`).
- **`~/.mac-stats/debug.log` is unchanged** — all events that pass the global verbosity filter are still written to the file.

On startup, if the variable is set, a one-line notice is printed to stderr listing the active names.

Examples:

```bash
MAC_STATS_LOG=browser,ollama ./mac_stats -vvv
```

Parent names include children: `MAC_STATS_LOG=ollama` shows both `mac_stats::ollama/api` (HTTP client) and `mac_stats::ollama/chat` (agent router orchestration).

## Canonical subsystem names

Aligned with `Subsystem` in `src-tauri/src/logging/subsystem.rs`: `metrics`, `ollama`, `discord`, `browser`, `monitors`, `alerts`, `scheduler`, `plugins`, `ui`, `config`. Hierarchical targets in use today include:

| Path | Area |
|------|------|
| `browser`, `browser/cdp`, `browser/http_fallback` | CDP agent, HTTP fallback |
| `ollama/api` | Ollama HTTP client (`ollama/mod.rs`) |
| `circuit` | Per-service circuit breaker state transitions (Ollama `/api/chat` + `/api/tags`, Discord outbound HTTP sends): `Circuit opened for … after N consecutive failures` (warn), `Circuit closed for … — service recovered` (info) (`circuit_breaker.rs`). When **`MAC_STATS_DEBUG_FORCE_OPEN_OLLAMA_CIRCUIT`** is set, one info line notes debug gating for Ollama (`ollama/mod.rs`). |
| `ollama/chat` | Agent router / `answer_with_ollama_and_fetch` (`commands/ollama.rs`); typed `OllamaRunError` codes and per-code counters (`commands/ollama_run_error.rs`), with `get_ollama_run_error_metrics` for debugging |
| `ollama/queue` | Global + per-key Ollama HTTP slot (`ollama_queue.rs`); e.g. `Ollama HTTP queue: global permit acquired (key=…)` when a turn holds the semaphore |
| `scheduler/heartbeat` | Periodic heartbeat checklist + HEARTBEAT_OK handling (`scheduler/heartbeat.rs`); startup: `Heartbeat loop started (app async runtime; same Tokio as Tauri)` |
| `events` | Internal event bus: one-time INFO when default handlers register (`internal event bus: default handlers registered`) |
| `events/screenshot`, `events/tool` | Internal event bus handlers (`src-tauri/src/events/mod.rs`): screenshot saved path, tool invocation telemetry |

Other modules still use default `tracing` targets (often the Rust module path). Until migrated, those lines appear on the console when `MAC_STATS_LOG` is **unset**; when it **is** set, they are hidden from stderr but remain in the log file.

## Macros

Use the crate-root macros so new logs participate in filtering:

- `mac_stats_trace!("subsystem/path", …)`
- `mac_stats_debug!("subsystem/path", …)`
- `mac_stats_info!("subsystem/path", …)`
- `mac_stats_warn!("subsystem/path", …)`

The first argument is a string literal (no `mac_stats::` prefix); it is expanded to target `mac_stats::<literal>`.

Legacy `debug1!` / `debug2!` / `debug3!` macros do not set subsystem targets and are not affected by `MAC_STATS_LOG`.

## Log secret redaction

By default, lines written to **`~/.mac-stats/debug.log`** and **stderr** pass through a redaction pass (see `src-tauri/src/logging/redact.rs`): common token shapes (`Bearer …`, `sk-…`, `ghp_…`, Slack-style `xox…` / `xapp…`, PEM blocks, long base64-like runs, plus optional extra regexes) are masked as `abcd…wxyz` (or `<redacted>` when the match is short).

- **Disable** for raw debugging: `LOG_REDACTION=0` (or `false` / `no` / `off`) in the environment or in `~/.mac-stats/.config.env` (same key). Env wins over the file when both apply.
- **Extra patterns:** `LOG_REDACT_EXTRA_REGEX` — semicolon-separated Rust regex patterns, from env or `.config.env`. Invalid patterns are skipped with a stderr warning at startup.

Keychain helpers never log the raw secret; at trace level they may log a masked preview only (`security/mod.rs`).
