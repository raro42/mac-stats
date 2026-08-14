# Morning surprise — 2026-08-12

## What failed overnight

The harness loop **ticked** (`AGENT_LOOP_TICK_harness` every 20m) but **did not run any agent**.
It only printed prompts. Nothing consumed them. Digester still recommended the stale Agent Ops screenshot (14.5d). No keep/discard. No morning surprise until this daytime fix.

## What shipped after the complaint

1. **Harness spawn fix** — `scripts/run_overnight_harness_loop.py` now spawns Cursor `agent -p --force --trust` each overnight tick (lock + log under `~/.mac-stats/improvements/`). Tick-first (no dead first 20 minutes).
2. **Design review** — refresh `docs/screens/feature-agent-ops.png` (+ CPU metrics) with window-only capture; `MAC_STATS_OPEN_SECTION=agent-ops` / `take_open_ui_section` for unattended open.
3. **Agent Ops copy** — digest empty state states that overnight must still ship backlog/design review (quiet = fail).

## Next night check

- `~/.mac-stats/improvements/overnight_agent.log` should grow during 20:00–06:00.
- `results.tsv` should gain ≥1 keep or discard.
- Design-review ages in `python3 scripts/overnight_design_review.py` should drop for recaptured screens.
