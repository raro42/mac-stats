#!/usr/bin/env python3
"""Overnight-only harness loop (20:00–06:00 local).

Prints AGENT_LOOP_TICK_harness for observability, then **spawns** the Cursor
`agent` CLI so work actually runs (print-only ticks were a quiet failure mode).

Also runs a **~23:00 pending git flush** (commit+push dirty safe files) once per night
so finished work never sits uncommitted. See scripts/overnight_git_flush.py and
.cursor/rules/no-uncommitted-leftovers.mdc.

Does not spawn agents during daytime — only prints AGENT_LOOP_SLEEP_harness then.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import time
from datetime import datetime, timedelta
from pathlib import Path

SENTINEL = "AGENT_LOOP_TICK_harness"
SLEEP_NOTE = "AGENT_LOOP_SLEEP_harness"
FLUSH_NOTE = "AGENT_LOOP_FLUSH_git"
AGENT_NOTE = "AGENT_LOOP_AGENT"
START_H, END_H = 20, 6
FLUSH_H = 23
INTERVAL = 1200
# Leave a little headroom inside the 20m cadence.
AGENT_TIMEOUT_S = 1100
ROOT = Path(__file__).resolve().parents[1]
IMPROVEMENTS = Path.home() / ".mac-stats" / "improvements"
FLUSH_STAMP = IMPROVEMENTS / "overnight_git_flush_date.txt"
AGENT_LOCK = IMPROVEMENTS / "overnight_agent.pid"
AGENT_LOG = IMPROVEMENTS / "overnight_agent.log"

PROMPT = (
    "Overnight mac-stats autoresearch tick (surprise Ralf). Follow docs/autoresearch/program.md. "
    "Nightly minimum: ≥1 keep OR discard in results.tsv per 20:00–06:00 window; quiet is failure mode (max 1 quiet tick/night). "
    "Git discipline: when an experiment finishes, commit + push immediately (no dirty leftovers). "
    "Around 23:00 the loop also runs scripts/overnight_git_flush.py as a backstop. "
    "(1) python3 scripts/digest_agent_runs.py "
    "(2) python3 scripts/watch_sibling_harnesses.py "
    "(3) python3 scripts/overnight_design_review.py "
    "(4) read docs/autoresearch/{program,standing_backlog}.md and "
    "~/.mac-stats/improvements/{latest,sibling_harness,loop_backlog,standing_backlog}.md "
    "(5) skim ~/.mac-stats/debug.log for errors "
    "(run python3 scripts/scan_debug_log_errors.py; see agents/log-monitor/) "
    "(6) pick fuel: open digester → design review if due → standing backlog → sibling/debug; "
    "do NOT quiet-default when backlog/design-review has work "
    "(7) ONE experiment: START_SHA=$(git rev-parse HEAD); implement; "
    "python3 scripts/autoresearch_ratchet.py verify; "
    "on fail: python3 scripts/autoresearch_ratchet.py discard --start-sha $START_SHA -d '…'; "
    "on pass: commit (no agent attribution), python3 scripts/autoresearch_ratchet.py keep -d '…', "
    "bump patch+CHANGELOG when shipping behavior, sync-dist/install/kickstart when runtime changes, "
    "git push origin HEAD "
    "(8) update loop_backlog.md. Do not ask the user. Before ~05:50 write/refresh "
    "~/.mac-stats/improvements/morning_surprise_YYYY-MM-DD.md with what shipped/tried "
    "then python3 scripts/archive_morning_surprise.py and commit the docs/ops/morning-surprises/ copy "
    "(empty digester alone is not a surprise). Quiet outside 20:00–06:00."
)


def in_window(now: datetime) -> bool:
    return now.hour >= START_H or now.hour < END_H


def seconds_until_window(now: datetime) -> int:
    target = now.replace(hour=START_H, minute=0, second=0, microsecond=0)
    if now >= target:
        target += timedelta(days=1)
    return max(1, int((target - now).total_seconds()))


def flush_due(now: datetime) -> bool:
    """True once per local calendar night after 23:00 (while still in overnight window)."""
    if not in_window(now):
        return False
    # Only in the evening segment (20:00–23:59), not after midnight 00–06.
    if now.hour < FLUSH_H:
        return False
    today = now.date().isoformat()
    if FLUSH_STAMP.exists() and FLUSH_STAMP.read_text().strip() == today:
        return False
    return True


def run_git_flush(now: datetime) -> None:
    FLUSH_STAMP.parent.mkdir(parents=True, exist_ok=True)
    script = ROOT / "scripts" / "overnight_git_flush.py"
    print(
        f'{FLUSH_NOTE} {{"at":"{now.isoformat(timespec="seconds")}","script":"{script}"}}',
        flush=True,
    )
    proc = subprocess.run(
        ["python3", str(script)],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    if proc.stdout:
        print(proc.stdout.rstrip(), flush=True)
    if proc.stderr:
        print(proc.stderr.rstrip(), flush=True)
    print(
        f'{FLUSH_NOTE} {{"exit":{proc.returncode}}}',
        flush=True,
    )
    # Stamp even on clean (exit 0) so we do not retry every tick; retry next night if failed hard.
    if proc.returncode == 0:
        FLUSH_STAMP.write_text(now.date().isoformat() + "\n")
    elif "clean" in (proc.stdout or ""):
        FLUSH_STAMP.write_text(now.date().isoformat() + "\n")


def agent_still_running() -> bool:
    if not AGENT_LOCK.exists():
        return False
    try:
        pid = int(AGENT_LOCK.read_text().strip())
    except ValueError:
        AGENT_LOCK.unlink(missing_ok=True)
        return False
    try:
        os.kill(pid, 0)
    except OSError:
        AGENT_LOCK.unlink(missing_ok=True)
        return False
    return True


def resolve_agent_bin() -> str | None:
    for name in ("agent", "cursor-agent"):
        path = shutil.which(name)
        if path:
            return path
    # Common install location when PATH is thin (launchd / limited shells).
    home_local = Path.home() / ".local" / "bin"
    for name in ("agent", "cursor-agent"):
        cand = home_local / name
        if cand.is_file() and os.access(cand, os.X_OK):
            return str(cand)
    return None


def run_agent_tick(now: datetime) -> None:
    """Spawn Cursor agent CLI for one overnight experiment."""
    IMPROVEMENTS.mkdir(parents=True, exist_ok=True)
    payload = PROMPT.replace("\\", "\\\\").replace('"', '\\"')
    print(f'{SENTINEL} {{"prompt":"{payload}"}}', flush=True)

    if agent_still_running():
        print(
            f'{AGENT_NOTE} {{"skip":"busy","pid_file":"{AGENT_LOCK}"}}',
            flush=True,
        )
        return

    agent_bin = resolve_agent_bin()
    if not agent_bin:
        print(
            f'{AGENT_NOTE} {{"error":"agent CLI not on PATH","hint":"install cursor agent CLI"}}',
            flush=True,
        )
        return

    env = os.environ.copy()
    # Ensure ~/.local/bin stays available inside the child.
    local_bin = str(Path.home() / ".local" / "bin")
    env["PATH"] = local_bin + os.pathsep + env.get("PATH", "")

    cmd = [
        agent_bin,
        "-p",
        "--force",
        "--trust",
        "--workspace",
        str(ROOT),
        "--output-format",
        "text",
        PROMPT,
    ]
    print(
        f'{AGENT_NOTE} {{"start":"{now.isoformat(timespec="seconds")}","bin":"{agent_bin}"}}',
        flush=True,
    )
    with AGENT_LOG.open("a", encoding="utf-8") as log:
        log.write(f"\n--- {now.isoformat(timespec='seconds')} ---\n")
        log.flush()
        try:
            proc = subprocess.Popen(
                cmd,
                cwd=str(ROOT),
                env=env,
                stdout=log,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
        except OSError as exc:
            print(f'{AGENT_NOTE} {{"error":"spawn failed","detail":"{exc}"}}', flush=True)
            return
        AGENT_LOCK.write_text(str(proc.pid) + "\n")
        try:
            code = proc.wait(timeout=AGENT_TIMEOUT_S)
        except subprocess.TimeoutExpired:
            proc.kill()
            try:
                proc.wait(timeout=30)
            except subprocess.TimeoutExpired:
                pass
            print(
                f'{AGENT_NOTE} {{"exit":"timeout","seconds":{AGENT_TIMEOUT_S}}}',
                flush=True,
            )
            AGENT_LOCK.unlink(missing_ok=True)
            return
        AGENT_LOCK.unlink(missing_ok=True)
        print(f'{AGENT_NOTE} {{"exit":{code}}}', flush=True)


def main() -> None:
    now = datetime.now().astimezone()
    if in_window(now):
        print(
            f'{SLEEP_NOTE} {{"until":"next-tick","seconds":{INTERVAL},"policy":"overnight-only-autoresearch","spawn":"agent-cli"}}',
            flush=True,
        )
    else:
        print(
            f'{SLEEP_NOTE} {{"until":"20:00","policy":"overnight-only-autoresearch","spawn":"agent-cli"}}',
            flush=True,
        )
    while True:
        now = datetime.now().astimezone()
        if not in_window(now):
            wait = seconds_until_window(now)
            time.sleep(min(wait, 1800))
            continue

        if flush_due(now):
            run_git_flush(now)

        # Tick first (do not burn the first 20 minutes of the night on sleep-only).
        run_agent_tick(now)

        time.sleep(INTERVAL)


if __name__ == "__main__":
    main()
