#!/usr/bin/env python3
"""Trigger GitHub Actions CI (macOS cargo check) only when a batch is due.

Policy: do not run on every push. Local `cargo check` is the day-to-day gate.
Remote CI is for intentional macOS proof after a real batch of patches.

Gates (all must pass unless --force):
  - On branch main (or master)
  - `gh` available and authenticated
  - No CI run already queued or in progress
  - At most one trigger per local calendar day (stamp file)
  - Patch delta since last *successful* CI ≥ MIN_PATCH_DELTA (default 10), OR
    release is already due (same soft/hard gates as maybe_cut_github_release)

Does not cut a release. Pair with scripts/maybe_cut_github_release.py overnight;
Release workflow still builds the DMG when a tag is cut.

Usage:
  python3 scripts/maybe_run_ci.py
  python3 scripts/maybe_run_ci.py --dry-run
  python3 scripts/maybe_run_ci.py --force
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from shutil import which

ROOT = Path(__file__).resolve().parents[1]
IMPROVEMENTS = Path.home() / ".mac-stats" / "improvements"
STAMP = IMPROVEMENTS / "overnight_ci_trigger_date.txt"

# Soft batch: enough patches since last green CI to bother macOS runners.
MIN_PATCH_DELTA = 10
# Align with maybe_cut_github_release so a release-due night also refreshes CI.
RELEASE_MIN_PATCH_DELTA = 20
RELEASE_MIN_DAYS = 7
RELEASE_MIN_PATCH_SOFT = 5


def run(args: list[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=check,
    )


def cargo_version(text: str | None = None) -> str:
    raw = text if text is not None else (ROOT / "src-tauri" / "Cargo.toml").read_text()
    m = re.search(r'^version\s*=\s*"([^"]+)"', raw, re.M)
    if not m:
        raise SystemExit("maybe_run_ci: no version in Cargo.toml")
    return m.group(1)


def parse_semver(v: str) -> tuple[int, int, int]:
    parts = v.lstrip("v").split(".")
    if len(parts) != 3:
        raise ValueError(v)
    return int(parts[0]), int(parts[1]), int(parts[2])


def patch_delta(newer: str, older: str) -> int:
    a = parse_semver(newer)
    b = parse_semver(older)
    if a[:2] != b[:2]:
        return 999
    return a[2] - b[2]


def branch_name() -> str:
    return run(["git", "rev-parse", "--abbrev-ref", "HEAD"]).stdout.strip()


def already_triggered_today() -> bool:
    if not STAMP.exists():
        return False
    return STAMP.read_text().strip() == datetime.now().date().isoformat()


def mark_triggered_today() -> None:
    STAMP.parent.mkdir(parents=True, exist_ok=True)
    STAMP.write_text(datetime.now().date().isoformat() + "\n")


def latest_release_tag() -> str | None:
    proc = run(
        [
            "gh",
            "release",
            "list",
            "--limit",
            "1",
            "--json",
            "tagName",
            "-q",
            ".[0].tagName",
        ],
        check=False,
    )
    tag = (proc.stdout or "").strip()
    if proc.returncode != 0 or not tag or tag == "null":
        return None
    return tag


def latest_release_published_at() -> datetime | None:
    proc = run(
        [
            "gh",
            "release",
            "list",
            "--limit",
            "1",
            "--json",
            "publishedAt",
            "-q",
            ".[0].publishedAt",
        ],
        check=False,
    )
    raw = (proc.stdout or "").strip().strip('"')
    if proc.returncode != 0 or not raw or raw == "null":
        return None
    try:
        return datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError:
        return None


def release_due(ver: str) -> tuple[bool, int, int, str | None]:
    prev = latest_release_tag()
    delta = patch_delta(ver, prev.lstrip("v")) if prev else 999
    published = latest_release_published_at()
    days = 999
    if published is not None:
        days = (datetime.now(timezone.utc) - published).days
    due = delta >= RELEASE_MIN_PATCH_DELTA or (
        days >= RELEASE_MIN_DAYS and delta >= RELEASE_MIN_PATCH_SOFT
    )
    return due, delta, days, prev


def ci_busy() -> bool:
    proc = run(
        [
            "gh",
            "run",
            "list",
            "--workflow",
            "ci.yml",
            "--limit",
            "5",
            "--json",
            "status",
        ],
        check=False,
    )
    if proc.returncode != 0:
        return False
    try:
        rows = json.loads(proc.stdout or "[]")
    except json.JSONDecodeError:
        return False
    return any(r.get("status") in ("queued", "in_progress", "waiting") for r in rows)


def last_successful_ci_version() -> str | None:
    """Cargo version at the HEAD of the latest successful CI run, if git can see it."""
    proc = run(
        [
            "gh",
            "run",
            "list",
            "--workflow",
            "ci.yml",
            "--status",
            "completed",
            "--limit",
            "20",
            "--json",
            "conclusion,headSha,event",
        ],
        check=False,
    )
    if proc.returncode != 0:
        return None
    try:
        rows = json.loads(proc.stdout or "[]")
    except json.JSONDecodeError:
        return None
    for row in rows:
        if row.get("conclusion") != "success":
            continue
        sha = (row.get("headSha") or "").strip()
        if not sha:
            continue
        show = run(
            ["git", "show", f"{sha}:src-tauri/Cargo.toml"],
            check=False,
        )
        if show.returncode != 0 or not show.stdout:
            continue
        try:
            return cargo_version(show.stdout)
        except SystemExit:
            continue
    return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--force", action="store_true", help="Skip delta/day/busy gates")
    args = ap.parse_args()

    if which("gh") is None:
        print("maybe_run_ci: gh not on PATH", file=sys.stderr)
        return 1

    br = branch_name()
    if br not in ("main", "master") and not args.force:
        print(f"maybe_run_ci: skip (branch={br})")
        return 0

    if already_triggered_today() and not args.force:
        print("maybe_run_ci: skip (already triggered today)")
        return 0

    if ci_busy() and not args.force:
        print("maybe_run_ci: skip (CI already queued or in progress)")
        return 0

    ver = cargo_version()
    last_ci = last_successful_ci_version()
    since_ci = patch_delta(ver, last_ci) if last_ci else 999
    rel_due, rel_delta, rel_days, prev = release_due(ver)

    batch_due = since_ci >= MIN_PATCH_DELTA
    due = batch_due or rel_due
    if not due and not args.force:
        print(
            f"maybe_run_ci: skip "
            f"(cargo={ver} last_ci={last_ci or 'none'} since_ci={since_ci} "
            f"latest={prev} release_delta={rel_delta} days={rel_days})"
        )
        return 0

    reason = []
    if batch_due or args.force:
        reason.append(f"since_ci={since_ci}>={MIN_PATCH_DELTA}")
    if rel_due:
        reason.append(f"release_due delta={rel_delta} days={rel_days}")
    if args.force and not reason:
        reason.append("force")

    print(
        f"maybe_run_ci: trigger CI on {br} at v{ver} "
        f"({', '.join(reason)}; last_ci={last_ci or 'none'})"
    )
    if args.dry_run:
        return 0

    trig = run(
        ["gh", "workflow", "run", "ci.yml", "--ref", br],
        check=False,
    )
    if trig.returncode != 0:
        print((trig.stderr or trig.stdout or "").strip(), file=sys.stderr)
        return trig.returncode

    mark_triggered_today()
    print("maybe_run_ci: workflow_dispatch submitted")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
