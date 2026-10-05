#!/usr/bin/env python3
"""Install reboot-safe systemd user units for the mac-stats overnight harness.

Linux counterpart of scripts/install-overnight-harness-launchagent.sh (macOS).
Writes units under the user systemd directory (not the repo).
Quiet during daytime; spawns Cursor agent CLI only 20:00–06:00 local.
"""

from __future__ import annotations

import getpass
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UNIT_DIR = Path.home() / ".config" / "systemd" / "user"
IMPROVEMENTS = Path.home() / ".mac-stats" / "improvements"
HOME = Path.home()
PYTHON = sys.executable

PATH_VALUE = (
    f"{HOME}/.cargo/bin:"
    f"{HOME}/.local/bin:"
    f"{HOME}/.local/share/mise/installs/gh/latest/gh_2.100.0_linux_amd64/bin:"
    f"/usr/local/bin:/usr/bin"
)

STDOUT_LOG = IMPROVEMENTS / "overnight_harness_loop.stdout.log"
STDERR_LOG = IMPROVEMENTS / "overnight_harness_loop.stderr.log"

HARNESS_SERVICE = f"""[Unit]
Description=mac-stats overnight autoresearch harness (20:00–06:00)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory={ROOT}
Environment=HOME={HOME}
Environment=PATH={PATH_VALUE}
ExecStart={PYTHON} -u {ROOT}/scripts/run_overnight_harness_loop.py
Restart=always
RestartSec=30
KillMode=mixed
TimeoutStopSec=90
StandardOutput=append:{STDOUT_LOG}
StandardError=append:{STDERR_LOG}

[Install]
WantedBy=default.target
"""

# Shared watchdog (issue loop + overnight) — reinstall so both units are covered.
WATCHDOG_SERVICE = f"""[Unit]
Description=mac-stats agent loops watchdog
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
    IMPROVEMENTS.mkdir(parents=True, exist_ok=True)
    STDOUT_LOG.touch(exist_ok=True)
    STDERR_LOG.touch(exist_ok=True)

    (UNIT_DIR / "mac-stats-overnight-harness.service").write_text(
        HARNESS_SERVICE, encoding="utf-8"
    )
    (UNIT_DIR / "mac-stats-watchdog.service").write_text(
        WATCHDOG_SERVICE, encoding="utf-8"
    )
    (UNIT_DIR / "mac-stats-watchdog.timer").write_text(WATCHDOG_TIMER, encoding="utf-8")
    print(f"wrote units in {UNIT_DIR}")
    print(f"logs: {STDOUT_LOG}")
    print(f"      {STDERR_LOG}")

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

    # Prefer systemd ownership over an ad-hoc nohup copy.
    subprocess.run(
        ["pkill", "-f", "run_overnight_harness_loop.py"],
        check=False,
        capture_output=True,
    )

    run(["systemctl", "--user", "daemon-reload"])
    run(["systemctl", "--user", "enable", "--now", "mac-stats-overnight-harness.service"])
    run(["systemctl", "--user", "enable", "--now", "mac-stats-watchdog.timer"])
    run(["systemctl", "--user", "start", "mac-stats-watchdog.service"])

    print("--- status ---")
    subprocess.run(
        [
            "systemctl",
            "--user",
            "status",
            "mac-stats-overnight-harness.service",
            "--no-pager",
        ],
        check=False,
    )
    subprocess.run(
        ["systemctl", "--user", "list-timers", "mac-stats-*.timer", "--no-pager"],
        check=False,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
