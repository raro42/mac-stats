#!/usr/bin/env python3
"""Install reboot-safe systemd user units for the mac-stats GitHub issue loop.

Writes units under the user systemd directory (not the repo).
Does not copy tokens into the repo. Relies on `gh` already authenticated.
"""

from __future__ import annotations

import getpass
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UNIT_DIR = Path.home() / ".config" / "systemd" / "user"
HOME = Path.home()
PYTHON = sys.executable

PATH_VALUE = (
    f"{HOME}/.cargo/bin:"
    f"{HOME}/.local/bin:"
    f"{HOME}/.local/share/mise/installs/gh/latest/gh_2.100.0_linux_amd64/bin:"
    f"/usr/local/bin:/usr/bin"
)

AGENT_SERVICE = f"""[Unit]
Description=mac-stats GitHub issue agent loop
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory={ROOT}
Environment=HOME={HOME}
Environment=PATH={PATH_VALUE}
Environment=AGENT_USE_CURSOR=1
Environment=AGENT_LOOP_SLEEP_MINUTES=5
Environment=AGENT_LOOP_BUSY_SLEEP_SECONDS=15
Environment=AGENT_GIT_SYNC=1
Environment=MAC_STATS_GH_REPO=raro42/mac-stats
Environment=MAC_STATS_SKIP_ISSUES=3
ExecStart=/usr/bin/bash {ROOT}/agents/mac-stats-cursor-loop.sh loop
Restart=always
RestartSec=20
KillMode=mixed
TimeoutStopSec=60

[Install]
WantedBy=default.target
"""

WATCHDOG_SERVICE = f"""[Unit]
Description=mac-stats agent loop watchdog
After=network-online.target

[Service]
Type=oneshot
WorkingDirectory={ROOT}
Environment=HOME={HOME}
Environment=PATH={PATH_VALUE}
ExecStart={PYTHON} {ROOT}/scripts/mac_stats_watchdog.py
"""

WATCHDOG_TIMER = """[Unit]
Description=Run mac-stats watchdog every 5 minutes

[Timer]
OnBootSec=2min
OnUnitActiveSec=5min
AccuracySec=30s
Persistent=true
Unit=mac-stats-watchdog.service

[Install]
WantedBy=timers.target
"""


def run(cmd: list[str]) -> None:
    print("+", " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True)


def main() -> int:
    UNIT_DIR.mkdir(parents=True, exist_ok=True)
    (UNIT_DIR / "mac-stats-agent-loop.service").write_text(AGENT_SERVICE, encoding="utf-8")
    (UNIT_DIR / "mac-stats-watchdog.service").write_text(WATCHDOG_SERVICE, encoding="utf-8")
    (UNIT_DIR / "mac-stats-watchdog.timer").write_text(WATCHDOG_TIMER, encoding="utf-8")
    print(f"wrote units in {UNIT_DIR}")

    user = os.environ.get("USER") or getpass.getuser()
    linger = subprocess.run(
        ["loginctl", "show-user", user, "-p", "Linger"],
        text=True,
        capture_output=True,
        check=False,
    ).stdout
    if "Linger=yes" not in linger:
        print("NOTE: user linger is off. Loop may stop at logout.")
        print("Enable with: loginctl enable-linger")

    run(["systemctl", "--user", "daemon-reload"])
    run(["systemctl", "--user", "enable", "--now", "mac-stats-agent-loop.service"])
    run(["systemctl", "--user", "enable", "--now", "mac-stats-watchdog.timer"])
    run(["systemctl", "--user", "start", "mac-stats-watchdog.service"])

    print("--- status ---")
    subprocess.run(
        ["systemctl", "--user", "status", "mac-stats-agent-loop.service", "--no-pager"],
        check=False,
    )
    subprocess.run(
        ["systemctl", "--user", "list-timers", "mac-stats-*.timer", "--no-pager"],
        check=False,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
