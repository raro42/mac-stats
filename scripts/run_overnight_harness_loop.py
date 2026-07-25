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
    "(1) python3 scripts/digest_agent_runs.py "
    "(2) python3 scripts/watch_sibling_harnesses.py "
    "(3) read docs/autoresearch/program.md and ~/.mac-stats/improvements/{latest,sibling_harness,loop_backlog}.md "
    "(4) skim ~/.mac-stats/debug.log for errors "
    "(5) if no open digester candidate and no clear high-impact bug/sibling port: quiet tick — backlog only, no filler ship "
    "(6) else ONE experiment: START_SHA=$(git rev-parse HEAD); implement; "
    "python3 scripts/autoresearch_ratchet.py verify; "
    "on fail: python3 scripts/autoresearch_ratchet.py discard --start-sha $START_SHA -d '…'; "
    "on pass: commit (no agent attribution), python3 scripts/autoresearch_ratchet.py keep -d '…', "
    "bump patch+CHANGELOG when shipping behavior, sync-dist/install/kickstart when runtime changes, push when reasonable "
    "(7) update loop_backlog.md. Do not ask the user. Quiet outside 20:00–06:00."
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
