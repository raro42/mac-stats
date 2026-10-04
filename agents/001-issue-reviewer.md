# 001 — GitHub issue pickup

The loop runs `python3 agents/issue_checker.py` every cycle.

## Do

- Turn **open** GitHub issues on `raro42/mac-stats` into `agents/tasks/FEAT-<number>-*.md`.
- Store a **short redacted summary** only. Never dump the raw issue body into git.
- Comment with `./scripts/gh-safe.sh` and add `agent:planned`.
- Skip **#3** (feedback wanted) and any number in `MAC_STATS_SKIP_ISSUES`.
- Skip issues that already have a FEAT/WIP/UNTESTED/TESTING/CLOSED file for that number.

## Do not

- Close the issue here.
- Pick the FEATURE-CODER `FEAT-D*` table. Those are a local backlog, not GitHub.
- Create a second task for an issue that is already in the queue.

Coder: `agents/006-feature-coder/FEATURE-CODER.md`. Tester: `agents/testing/TESTER.md`. Close: `agents/004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`.
