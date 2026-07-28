#!/usr/bin/env python3
"""Overnight-only harness loop (20:00–06:00 local). Emits AGENT_LOOP_TICK_harness.

Does not notify during daytime — only prints AGENT_LOOP_SLEEP_harness then.
"""

from __future__ import annotations

import time
from datetime import datetime, timedelta

SENTINEL = "AGENT_LOOP_TICK_harness"
SLEEP_NOTE = "AGENT_LOOP_SLEEP_harness"
START_H, END_H = 20, 6
INTERVAL = 1200

PROMPT = (
    "Overnight mac-stats autoresearch tick (surprise Ralf). Follow docs/autoresearch/program.md. "
    "Nightly minimum: ≥1 keep OR discard in results.tsv per 20:00–06:00 window; quiet is failure mode (max 1 quiet tick/night). "
    "(1) python3 scripts/digest_agent_runs.py "
    "(2) python3 scripts/watch_sibling_harnesses.py "
    "(3) python3 scripts/overnight_design_review.py "
    "(4) read docs/autoresearch/{program,standing_backlog}.md and "
    "~/.mac-stats/improvements/{latest,sibling_harness,loop_backlog,standing_backlog}.md "
    "(5) skim ~/.mac-stats/debug.log for errors "
    "(6) pick fuel: open digester → design review if due → standing backlog → sibling/debug; "
    "do NOT quiet-default when backlog/design-review has work "
    "(7) ONE experiment: START_SHA=$(git rev-parse HEAD); implement; "
    "python3 scripts/autoresearch_ratchet.py verify; "
    "on fail: python3 scripts/autoresearch_ratchet.py discard --start-sha $START_SHA -d '…'; "
    "on pass: commit (no agent attribution), python3 scripts/autoresearch_ratchet.py keep -d '…', "
    "bump patch+CHANGELOG when shipping behavior, sync-dist/install/kickstart when runtime changes, push when reasonable "
    "(8) update loop_backlog.md. Do not ask the user. Before ~05:50 write/refresh "
    "~/.mac-stats/improvements/morning_surprise_YYYY-MM-DD.md with what shipped/tried "
    "(empty digester alone is not a surprise). Quiet outside 20:00–06:00."
)


def in_window(now: datetime) -> bool:
    return now.hour >= START_H or now.hour < END_H


def seconds_until_window(now: datetime) -> int:
    target = now.replace(hour=START_H, minute=0, second=0, microsecond=0)
    if now >= target:
        target += timedelta(days=1)
    return max(1, int((target - now).total_seconds()))


def main() -> None:
    now = datetime.now().astimezone()
    if in_window(now):
        print(
            f'{SLEEP_NOTE} {{"until":"next-tick","seconds":{INTERVAL},"policy":"overnight-only-autoresearch"}}',
            flush=True,
        )
    else:
        print(
            f'{SLEEP_NOTE} {{"until":"20:00","policy":"overnight-only-autoresearch"}}',
            flush=True,
        )
    while True:
        now = datetime.now().astimezone()
        if not in_window(now):
            wait = seconds_until_window(now)
            time.sleep(min(wait, 1800))
            continue
        time.sleep(INTERVAL)
        now = datetime.now().astimezone()
        if not in_window(now):
            continue
        payload = PROMPT.replace("\\", "\\\\").replace('"', '\\"')
        print(f'{SENTINEL} {{"prompt":"{payload}"}}', flush=True)


if __name__ == "__main__":
    main()
