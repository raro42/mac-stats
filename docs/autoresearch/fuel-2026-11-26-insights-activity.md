# Overnight fuel pick — 2026-11-26 ~04:18 CET

Digester empty · design-review due=false · layout-daily marked today · `/pmdisk` shipped (v0.1.3103).

## Shipped this tick (v0.1.3104)

**Sources** (`by_entry` from `runs.jsonl` `entry`) on Discord `/insights` + Agent Ops Runs Insights. Activity (busiest day/hour · streak) stays next-tick fuel.

## Recommended experiment

**Hermes `/insights` leftover → mac-stats:** add **entry (platform) breakdown** + **activity (busiest day/hour · active days · streak)** to the existing Runs Insights surface.

### Current gap

- Hermes `agent/insights.py` `format_gateway` (commit `b52b37a`) already shows Platforms + Busiest day/hour + active days + streak.
- mac-stats `/insights` (`format_runs_insights_gateway`) and Agent Ops Runs Insights already show turns / lanes / tools / slowest / candidates / digest / schedules — **not** entry mix or activity.
- `~/.mac-stats/runs.jsonl` already has `entry` (`discord` · `agent_router`) and `ts` — no new telemetry.

### Exact files

1. `src-tauri/src/commands/harness_ops/agent_ops.inc.rs` — extend `RunsInsights` + `compute_runs_insights_for` (`by_entry`, `busiest_day`, `busiest_hour`, `active_days`, `max_streak`).
2. `src-tauri/src/commands/harness_ops/insights.inc.rs` — append `Entries:` + `Busiest:` lines in `format_runs_insights_gateway` (keep Discord ≤~1800 chars).
3. `src/agent-ops.js` (+ `scripts/sync-dist.sh` → `src-tauri/dist/agent-ops.js`) — Entries + Busiest sections under Lanes (calm empty copy).
4. Optional: unit tests next to existing insights helpers in `src-tauri/src/commands/harness_ops/mod.rs`.
5. Ship: `src-tauri/Cargo.toml` version · `CHANGELOG.md` Unreleased.

### Why it helps

Operators see where turns come from (Discord vs router) and when the bot is busiest — same glance Hermes already gives — without opening digester dumps.

### Avoid (clones / blocks)

- keep-pNN ratchet glances
- section-open theme washes (series complete)
- thin pmset alias clones (`disk` ≡ `custom` already `/pmdisk`; remaining `uuidlog` / `*log` streams block)
- Full Hermes cost/token/model port (runs.jsonl has no tokens/models)
- New `/activity` slash (collides with `/activity-level` User activity)
- OpenClaw full session-resume mutation tonight (list already `/sessions`; resume stays agent / Agent Ops Load into AI Chat)

### pmset remainder (not this tick)

Shipped non-blocking getters cover custom/disk alias, powerstate summary, history soft-disabled, etc. Streaming/blocking left: `uuidlog`, `pslog`, `rawlog`, `thermlog`, `assertionslog`, `sysloadlog`, `powerstatelog` — skip.
