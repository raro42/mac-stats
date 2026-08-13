# Overnight quiet failure — 2026-08-12

## Symptom

Harness printed many `AGENT_LOOP_TICK_harness` lines overnight. No commits. No `results.tsv` keep/discard. Digester still listed stale `feature-agent-ops.png` (~14.5d).

## Cause

`scripts/run_overnight_harness_loop.py` only **printed** a prompt JSON line. It assumed a Cursor IDE `AGENT_LOOP` consumer would run the agent. That consumer did not run. Quiet night.

## Fix

The loop now spawns `agent` / `cursor-agent` with `-p --force --trust --workspace <repo>` each overnight tick. Lock: `~/.mac-stats/improvements/overnight_agent.pid`. Log: `overnight_agent.log`. Daytime still sleeps until 20:00.

## Related

- `~/.mac-stats/improvements/morning_surprise_2026-08-12.md`
- `docs/043_overnight_design_review.md`
- v0.1.369

## 2026-08-13 follow-up

Second quiet night: spawn-capable harness started on 2026-08-12 daytime, then **exited before 20:00** (no `overnight_agent.log`, no commits). Cause: ad-hoc `nohup` process without KeepAlive.

**Fix:** `scripts/install-overnight-harness-launchagent.sh` installs `com.raro42.mac-stats-overnight-harness` with `KeepAlive`.
