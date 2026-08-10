# agents/

Agent-ops homes for this **repo**. Not the live app agent files.

## Standing rules (always)

1. **Always test** after a runtime change (`cargo check` / relevant tests / task verification). Use [testing/](testing/) for queue work.
2. **Always read logs** after start/restart or a failed behaviour. Skim `~/.mac-stats/debug.log` for related ERROR / WARN / panic. Use [log-monitor/](log-monitor/) for structured scans.

## What lives here

| Path | Purpose |
|------|---------|
| [testing/](testing/) | Tester prompt + in-flight `TESTING-*` files |
| [log-monitor/](log-monitor/) | Scan `~/.mac-stats/debug.log` for errors (read-only) |
| [tasks/](tasks/) | Queue/archive: `UNTESTED-*`, `CLOSED-*`, `WIP-*`, `FEAT-*` |
| [agents-tasks/](agents-tasks/) | Log findings (`log-NNN`) + legacy scanner tasks |
| [007-quality-monitor/](007-quality-monitor/) | Weekly repo quality (root clutter, dead scaffolding) |
| [006-feature-coder/](006-feature-coder/) | FEAT backlog |
| [005-openclaw-reviewer/](005-openclaw-reviewer/) | OpenClaw port review notes |
| [004-closing-reviewer/](004-closing-reviewer/) | Closing-reviewer prompt |
| [workspace/](workspace/) | Session todo / lessons (agents-file skill) |

## What does **not** live here

| Path | Purpose |
|------|---------|
| Root [`agents.md`](../agents.md) | Project instructions for Cursor / Claude (stays at repo root) |
| `src-tauri/defaults/agents/` | Bundled default agents (`include_str!`) |
| `~/.mac-stats/agents/` | Live user agents, skills, memory (runtime) |
| `~/.mac-stats/debug.log` | Live app log (monitor reads; do not move) |
