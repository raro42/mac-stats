#!/usr/bin/env python3
"""Once-per-night wipe of mac-stats Rust build artifacts.

Overnight autoresearch rebuilds `src-tauri/target` (debug + release). That tree
has repeatedly grown past 100 GiB. Disk Cleanup already wipes `target/debug` at
≥20 GiB, but release was left alone and still filled the disk.

This script runs from `run_overnight_harness_loop.py` once per overnight window
(before agent ticks burn another cold rebuild on top of yesterday's tree).

Safe rules:
- Skip if cargo/rustc/tauri looks busy.
- Skip if `target/` is missing or under MIN_BYTES (nothing useful to reclaim).
- Prefer `cargo clean`; fall back to `rm -rf target`.
- Optionally thin local Time Machine snapshots so APFS returns the space.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from datetime import datetime, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TAURI = ROOT / "src-tauri"
TARGET = TAURI / "target"
IMPROVEMENTS = Path.home() / ".mac-stats" / "improvements"
STAMP = IMPROVEMENTS / "overnight_rust_target_clean_date.txt"
# Do not bother cleaning tiny trees (partial builds).
MIN_BYTES = 1 * 1024 * 1024 * 1024  # 1 GiB
START_H, END_H = 20, 6


def night_id(now: datetime) -> str:
    """Calendar date of the overnight window start (20:00)."""
    d = now.date()
    if now.hour < END_H:
        d = d - timedelta(days=1)
    return d.isoformat()


def already_cleaned(now: datetime) -> bool:
    if not STAMP.is_file():
        return False
    return STAMP.read_text().strip() == night_id(now)


def mark_cleaned(now: datetime) -> None:
    IMPROVEMENTS.mkdir(parents=True, exist_ok=True)
    STAMP.write_text(night_id(now) + "\n")


def dir_size_bytes(path: Path) -> int:
    total = 0
    if not path.is_dir():
        return 0
    for root, _dirs, files in os.walk(path):
        for name in files:
            try:
                total += (Path(root) / name).stat().st_size
            except OSError:
                continue
    return total


def build_busy() -> bool:
    """True when cargo or rustc is running (exact process name; ignore PATH hits)."""
    for name in ("cargo", "rustc"):
        try:
            subprocess.check_output(["pgrep", "-x", name], text=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            continue
    return False


def run_cargo_clean() -> tuple[int, str]:
    cargo = shutil.which("cargo")
    if not cargo:
        return 127, "cargo not on PATH"
    proc = subprocess.run(
        [cargo, "clean"],
        cwd=str(TAURI),
        text=True,
        capture_output=True,
    )
    detail = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, detail.strip()


def rm_target() -> None:
    if TARGET.is_dir():
        shutil.rmtree(TARGET, ignore_errors=True)


def thin_snapshots() -> None:
    # Best-effort; macOS only. Ignore failures (needs privileges / TM present).
    if sys.platform != "darwin" or shutil.which("tmutil") is None:
        return
    subprocess.run(
        ["tmutil", "thinlocalsnapshots", "/", "1000000000000", "4"],
        capture_output=True,
        text=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--force",
        action="store_true",
        help="Ignore the once-per-night stamp (still skip when build is busy).",
    )
    parser.add_argument(
        "--no-thin",
        action="store_true",
        help="Skip tmutil thinlocalsnapshots after clean.",
    )
    args = parser.parse_args()
    now = datetime.now().astimezone()

    if not args.force and already_cleaned(now):
        print(f'AGENT_LOOP_RUST_CLEAN {{"skip":"already","night":"{night_id(now)}"}}')
        return 0

    if not TARGET.is_dir():
        mark_cleaned(now)
        print(f'AGENT_LOOP_RUST_CLEAN {{"skip":"no-target","night":"{night_id(now)}"}}')
        return 0

    size = dir_size_bytes(TARGET)
    if size < MIN_BYTES:
        mark_cleaned(now)
        print(
            f'AGENT_LOOP_RUST_CLEAN {{"skip":"small","bytes":{size},"night":"{night_id(now)}"}}'
        )
        return 0

    if build_busy():
        print(
            f'AGENT_LOOP_RUST_CLEAN {{"skip":"busy","bytes":{size},"night":"{night_id(now)}"}}'
        )
        return 0

    print(
        f'AGENT_LOOP_RUST_CLEAN {{"start":"{now.isoformat(timespec="seconds")}",'
        f'"bytes":{size},"night":"{night_id(now)}"}}'
    )
    code, detail = run_cargo_clean()
    if code != 0 or TARGET.is_dir():
        rm_target()
        method = "rm-rf" if code != 0 else "cargo-clean+rm"
        if detail:
            print(detail[:500], file=sys.stderr)
    else:
        method = "cargo-clean"

    after = dir_size_bytes(TARGET) if TARGET.is_dir() else 0
    if not args.no_thin:
        thin_snapshots()

    mark_cleaned(now)
    print(
        f'AGENT_LOOP_RUST_CLEAN {{"done":true,"method":"{method}",'
        f'"bytes_before":{size},"bytes_after":{after},"night":"{night_id(now)}"}}'
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
