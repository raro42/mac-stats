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
| `ollama/chat` | Agent router / `answer_with_ollama_and_fetch` (`commands/ollama.rs`) |

Other modules still use default `tracing` targets (often the Rust module path). Until migrated, those lines appear on the console when `MAC_STATS_LOG` is **unset**; when it **is** set, they are hidden from stderr but remain in the log file.

## Macros

Use the crate-root macros so new logs participate in filtering:

- `mac_stats_trace!("subsystem/path", …)`
- `mac_stats_debug!("subsystem/path", …)`
- `mac_stats_info!("subsystem/path", …)`
- `mac_stats_warn!("subsystem/path", …)`

The first argument is a string literal (no `mac_stats::` prefix); it is expanded to target `mac_stats::<literal>`.

Legacy `debug1!` / `debug2!` / `debug3!` macros do not set subsystem targets and are not affected by `MAC_STATS_LOG`.
