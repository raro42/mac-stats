#!/usr/bin/env python3
"""Restart the mac-stats GitHub issue loop user unit if it is enabled and down."""

from __future__ import annotations

import subprocess
import sys

UNIT = "mac-stats-agent-loop.service"


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, text=True, capture_output=True, check=False)


def main() -> int:
    enabled = run(["systemctl", "--user", "is-enabled", UNIT])
    if enabled.returncode != 0:
        print(f"watchdog: {UNIT} not enabled; leave stopped")
        return 0
    active = run(["systemctl", "--user", "is-active", UNIT])
    if active.stdout.strip() == "active":
        print(f"watchdog: {UNIT} active")
        return 0
    print(f"watchdog: {UNIT} down; restart")
    start = run(["systemctl", "--user", "start", UNIT])
    if start.returncode != 0:
        sys.stderr.write(start.stderr or start.stdout or "start failed\n")
        return start.returncode or 1
    print(f"watchdog: started {UNIT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
