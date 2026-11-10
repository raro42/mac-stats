#!/usr/bin/env python3
"""Restart enabled mac-stats agent user units if they are down.

Covers:
  - mac-stats-agent-loop.service (GitHub issue loop)
  - mac-stats-overnight-harness.service (overnight autoresearch)

Also reaps orphaned Cursor-agent python workers left in the mac-stats cwd
(bare `python3` / `-c` heredoc leftovers that peg a core after the agent exits).
"""

from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path

UNITS = (
    "mac-stats-agent-loop.service",
    "mac-stats-overnight-harness.service",
)

ROOT = Path(__file__).resolve().parents[1]
HARNESS = ROOT / "scripts" / "run_overnight_harness_loop.py"


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, text=True, capture_output=True, check=False)


def ensure_unit(unit: str) -> int:
    enabled = run(["systemctl", "--user", "is-enabled", unit])
    if enabled.returncode != 0:
        print(f"watchdog: {unit} not enabled; leave stopped")
        return 0
    active = run(["systemctl", "--user", "is-active", unit])
    if active.stdout.strip() == "active":
        print(f"watchdog: {unit} active")
        return 0
    print(f"watchdog: {unit} down; restart")
    start = run(["systemctl", "--user", "start", unit])
    if start.returncode != 0:
        sys.stderr.write(start.stderr or start.stdout or f"start failed: {unit}\n")
        return start.returncode or 1
    print(f"watchdog: started {unit}")
    return 0


def reap_orphan_agent_pythons() -> int:
    """Load reap helper from the overnight harness module (single source of truth)."""
    if not HARNESS.is_file():
        print("watchdog: harness script missing; skip orphan reap")
        return 0
    spec = importlib.util.spec_from_file_location("mac_stats_overnight_harness", HARNESS)
    if spec is None or spec.loader is None:
        print("watchdog: cannot load harness module; skip orphan reap")
        return 0
    mod = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(mod)
    except Exception as exc:  # noqa: BLE001 — watchdog must not die on import noise
        print(f"watchdog: harness import failed ({exc}); skip orphan reap")
        return 0
    reap = getattr(mod, "reap_orphan_cursor_pythons", None)
    if not callable(reap):
        print("watchdog: reap_orphan_cursor_pythons missing; skip")
        return 0
    killed = reap()
    if killed:
        print(f"watchdog: reaped orphan cursor pythons {killed}")
    else:
        print("watchdog: no orphan cursor pythons")
    return 0


def main() -> int:
    rc = 0
    for unit in UNITS:
        unit_rc = ensure_unit(unit)
        if unit_rc != 0:
            rc = unit_rc
    reap_rc = reap_orphan_agent_pythons()
    if reap_rc != 0:
        rc = reap_rc
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
