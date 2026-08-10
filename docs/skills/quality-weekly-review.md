---
name: quality-weekly-review
description: Weekly repo quality monitor for mac-stats — root clutter, dead npm scaffolding, stale paths after moves, broken marketing links.
---

# Quality weekly review

Repo root: `~/projects/mac-stats`. Policy: `docs/044_repo_quality_hygiene.md`. Agent home: `agents/007-quality-monitor/`.

## Do

1. Run `python3 scripts/scan_repo_quality.py` and read the report.
2. Fix every **fail** (delete dead scaffolding, move orphans, repair links/scripts). Prefer the smallest correct change.
3. Re-run the scanner until it exits **0** (or only **warn** items remain with a short note why).
4. **Ship via Cursor Agent** (Discord alone must not claim it committed). Emit:
   `CURSOR_AGENT: in ~/projects/mac-stats apply quality-weekly fixes if still dirty, then commit and push to origin`
5. Reply briefly: what failed, what you fixed, scanner exit code. If Cursor Agent was invoked, say so.

## Do not

- Rewrite CHANGELOG archaeology (that is Monday `changelog-weekly-review`)
- Ship Agent Ops CSS polish under this skill (that is Wednesday `ui-weekly-review`)
- Truncate `~/.mac-stats/debug.log` (use `agents/log-monitor/` for logs)
- Stop at the Discord git safety refusal — finish with `CURSOR_AGENT`
- Install npm packages “just in case”
