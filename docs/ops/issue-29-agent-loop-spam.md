# Issue #29 — agent loop comment spam

**Date:** 2026-11-27  
**Issue:** https://github.com/raro42/mac-stats/issues/29  

## Symptom

GitHub issue #29 got ~**2500** comments. Almost all were the same line:

```text
Agent 002: coding (`WIP-29-20261125-1815-bug-msg-in-discord.md`). Tester gets UNTESTED- when the batch is ready.
```

Cadence was ~45s from 2026-11-25 18:15 UTC through 2026-11-27 01:34 UTC. Comments then stopped (API hit the ~2500-comment ceiling / WIP file gone).

## What went wrong

Three failures stacked:

1. **Comment before progress.** `step_002_coder` in `agents/mac-stats-cursor-loop.sh` posted a GitHub comment on **every** tick while a `WIP-*` task existed. There was no dedupe.
2. **Coder never advanced the queue.** The WIP file stayed `WIP-29-…` (never renamed to `UNTESTED-`). With live queue work, the loop used `AGENT_LOOP_BUSY_SLEEP_SECONDS` (default **15s**) instead of the idle 5m sleep — so it re-entered step 002 rapidly.
3. **Product bug was fixed elsewhere.** Overnight / manual work shipped the real fix as **v0.1.3089** (`71f845694`) and follow-ups (3090, 3093). The issue loop kept the orphan WIP and kept commenting.

Likely amplifier: cursor-agent off, lock skip, or instant exit — so each busy cycle was “comment + no rename”, not a long coding session.

## Product fix (already shipped)

See `docs/ops/issue-29-scheduler-untrusted-wrap.md`. Schedules / heartbeat / tasks / CLI no longer wrap operator text in `MS_UNTRUSTED`.

## Loop fix (this review)

In `agents/mac-stats-cursor-loop.sh`:

- `maybe_gh_comment` — post a status body **once** per issue until the body changes (stamp under `agents/state/`).
- When cursor-agent is off, do **not** use the busy 15s sleep for a live WIP queue (avoids hammering).
- Log a clear stuck-WIP warning when coding cannot run.

## Cleanup

- Close #29 as completed (product fix already on `main`).
- No `WIP-29` / `FEAT-29` task file remains under `agents/tasks/`.
