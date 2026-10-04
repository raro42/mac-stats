# agents/

Agent-ops homes for this **repo**. Not the live app agent files.

## GitHub issue loop

Steered by the prompts in this directory. Pickup → coder → tester → close.

| Step | Role | Prompt |
|------|------|--------|
| 001 | Issue pickup | [001-issue-reviewer.md](001-issue-reviewer.md) · `issue_checker.py` |
| 002 | Coder | [006-feature-coder/FEATURE-CODER.md](006-feature-coder/FEATURE-CODER.md) |
| 003 | Tester | [testing/TESTER.md](testing/TESTER.md) |
| 004 | Close | [004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md](004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md) |
| 006 | Log scan | [log-monitor/](log-monitor/) |
| 007 | Quality (weekly) | [007-quality-monitor/](007-quality-monitor/) |
| 008 | Git flush (daily) | `scripts/overnight_git_flush.py` |

```bash
./agents/mac-stats-cursor-loop.sh once|loop|status
python3 scripts/install_mac_stats_agent_units.py   # Linux systemd + watchdog
```

**Single instance:** `agents/state/loop.pid` (gitignored). `start-unattended.command` refuses a second start unless `AGENT_LOOP_FORCE_RESTART=1`. Skip issue **#3** (`MAC_STATS_SKIP_ISSUES`). Overnight autoresearch stays a separate LaunchAgent (`scripts/run_overnight_harness_loop.py`).

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
| [006-feature-coder/](006-feature-coder/) | FEAT backlog (`FEAT-D*` table; GitHub `FEAT-<n>` files win) |
| [005-openclaw-reviewer/](005-openclaw-reviewer/) | OpenClaw port review notes (not this loop) |
| [004-closing-reviewer/](004-closing-reviewer/) | Closing-reviewer prompt |
| [workspace/](workspace/) | Session todo / lessons (agents-file skill) |
| [state/](state/) | Local `loop.pid` / stamps (gitignored) |

## What does **not** live here

| Path | Purpose |
|------|---------|
| Root [`agents.md`](../agents.md) | Project instructions for Cursor / Claude (stays at repo root) |
| `src-tauri/defaults/agents/` | Bundled default agents (`include_str!`) |
| `~/.mac-stats/agents/` | Live user agents, skills, memory (runtime) |
| `~/.mac-stats/debug.log` | Live app log (monitor reads; do not move) |
