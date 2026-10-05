#!/usr/bin/env python3
"""Restart enabled mac-stats agent user units if they are down.

Covers:
  - mac-stats-agent-loop.service (GitHub issue loop)
  - mac-stats-overnight-harness.service (overnight autoresearch)
"""

from __future__ import annotations

import subprocess
import sys

UNITS = (
    "mac-stats-agent-loop.service",
    "mac-stats-overnight-harness.service",
)


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


def main() -> int:
    rc = 0
    for unit in UNITS:
        unit_rc = ensure_unit(unit)
        if unit_rc != 0:
            rc = unit_rc
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
