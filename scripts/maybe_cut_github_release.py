#!/usr/bin/env python3
"""Cut a GitHub Release when Cargo.toml is meaningfully ahead of Latest.

Policy (Ralf, 2026-08-29): do not wait to be asked. When the tree has shipped
enough patch versions since the last GitHub Release, tag + `gh release create`
so CI can attach the DMG (`.github/workflows/release.yml`).

Gates (all must pass unless --force):
  - On branch main (or master)
  - Working tree clean
  - Cargo.toml version > latest GitHub release tag
  - Patch delta ≥ MIN_PATCH_DELTA (default 20), OR
    days since last release ≥ MIN_DAYS (default 7) and delta ≥ MIN_PATCH_SOFT (5)
  - At most one successful cut per local calendar day (stamp file)
  - `gh` available and authenticated

Does not bump Homebrew here — wait for the DMG asset, then
`scripts/print-release-checksums.sh` + edit Casks/mac-stats.rb +
`scripts/sync-homebrew-tap.sh`.

Usage:
  python3 scripts/maybe_cut_github_release.py
  python3 scripts/maybe_cut_github_release.py --dry-run
  python3 scripts/maybe_cut_github_release.py --force
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IMPROVEMENTS = Path.home() / ".mac-stats" / "improvements"
STAMP = IMPROVEMENTS / "overnight_github_release_date.txt"

MIN_PATCH_DELTA = 20
MIN_DAYS = 7
MIN_PATCH_SOFT = 5


def run(args: list[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=check,
    )


def cargo_version() -> str:
    text = (ROOT / "src-tauri" / "Cargo.toml").read_text()
    m = re.search(r'^version\s*=\s*"([^"]+)"', text, re.M)
    if not m:
        raise SystemExit("maybe_cut_github_release: no version in Cargo.toml")
    return m.group(1)


def parse_semver(v: str) -> tuple[int, int, int]:
    parts = v.lstrip("v").split(".")
    if len(parts) != 3:
        raise ValueError(v)
    return int(parts[0]), int(parts[1]), int(parts[2])


def latest_release_tag() -> str | None:
    proc = run(
        [
            "gh",
            "release",
            "list",
            "--limit",
            "1",
            "--json",
            "tagName,publishedAt",
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


def patch_delta(newer: str, older: str) -> int:
    a = parse_semver(newer)
    b = parse_semver(older)
    if a[:2] != b[:2]:
        # Minor/major jump — treat as large enough.
        return 999
    return a[2] - b[2]


def branch_name() -> str:
    return run(["git", "rev-parse", "--abbrev-ref", "HEAD"]).stdout.strip()


def working_tree_clean() -> bool:
    return not run(["git", "status", "--porcelain"]).stdout.strip()


def tag_exists(tag: str) -> bool:
    proc = run(["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag}"], check=False)
    return proc.returncode == 0


def build_notes(ver: str, prev: str | None) -> str:
    prev_disp = prev or "previous release"
    return f"""## mac-stats {ver}

GitHub Release catch-up from **{prev_disp}** → **v{ver}**.

This tag embeds the current `main` product line (Agent Ops, instant operators, CPU window polish, Disk Cleanup, monitors, and related fixes). Per-patch notes live in [CHANGELOG.md](https://github.com/raro42/mac-stats/blob/main/CHANGELOG.md).

### Highlights since {prev_disp}

- **Agent Ops** — Command Center filters, health strip, enable/disable agents in the UI, Runs Instant / Lite / Direct / Slow / Fail
- **Instant operators** — Discord + AI Chat parity for agents, sessions, knowledge, schedules, monitors, disk, logs, processes, Perplexity, rings, power strip, and more
- **CPU window** — icon-line full hide, temp sparkline feed, quieter chrome (no layout-shifting keyboard tip essays), GitHub URL tooltip
- **Reliability** — menu-bar disk refresh, install refuses stale release binaries, overnight harness keep/discard discipline

### Install

DMG attaches via CI (`.github/workflows/release.yml`). Homebrew cask updates after the asset SHA is known.

```bash
brew upgrade --cask mac-stats
# or download the DMG from this release
```
"""


def already_cut_today() -> bool:
    if not STAMP.exists():
        return False
    return STAMP.read_text().strip() == datetime.now().date().isoformat()


def mark_cut_today() -> None:
    STAMP.parent.mkdir(parents=True, exist_ok=True)
    STAMP.write_text(datetime.now().date().isoformat() + "\n")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--force", action="store_true", help="Skip delta/day gates")
    args = ap.parse_args()

    if not shutil_which("gh"):
        print("maybe_cut_github_release: gh not on PATH", file=sys.stderr)
        return 1

    br = branch_name()
    if br not in ("main", "master") and not args.force:
        print(f"maybe_cut_github_release: skip (branch={br})")
        return 0

    if not working_tree_clean() and not args.force:
        print("maybe_cut_github_release: skip (dirty tree)")
        return 0

    if already_cut_today() and not args.force:
        print("maybe_cut_github_release: skip (already cut today)")
        return 0

    ver = cargo_version()
    tag = f"v{ver}"
    prev = latest_release_tag()

    if prev == tag:
        print(f"maybe_cut_github_release: skip (Latest already {tag})")
        return 0

    if tag_exists(tag) and not args.force:
        # Tag exists but maybe no GH release — still try create; gh will error if dup.
        pass

    delta = patch_delta(ver, prev.lstrip("v")) if prev else 999
    published = latest_release_published_at()
    days = 999
    if published is not None:
        days = (datetime.now(timezone.utc) - published).days

    due = delta >= MIN_PATCH_DELTA or (
        days >= MIN_DAYS and delta >= MIN_PATCH_SOFT
    )
    if not due and not args.force:
        print(
            f"maybe_cut_github_release: skip "
            f"(cargo={ver} latest={prev} delta={delta} days={days})"
        )
        return 0

    notes = build_notes(ver, prev)
    print(
        f"maybe_cut_github_release: cut {tag} "
        f"(from {prev}, delta={delta}, days={days})"
    )
    if args.dry_run:
        print("--- notes ---")
        print(notes)
        return 0

    # Prefer annotated tag on HEAD so the release points at current main.
    if not tag_exists(tag):
        run(["git", "tag", "-a", tag, "-m", f"Release {tag}"])
        push = run(["git", "push", "origin", tag], check=False)
        if push.returncode != 0:
            print(push.stderr or push.stdout, file=sys.stderr)
            return push.returncode

    notes_path = IMPROVEMENTS / f"release_notes_{ver}.md"
    IMPROVEMENTS.mkdir(parents=True, exist_ok=True)
    notes_path.write_text(notes)

    create = run(
        [
            "gh",
            "release",
            "create",
            tag,
            "--title",
            f"Release {tag}",
            "--notes-file",
            str(notes_path),
            "--verify-tag",
        ],
        check=False,
    )
    if create.returncode != 0:
        err = (create.stderr or create.stdout or "").strip()
        # Tag push may have already triggered workflow; release might exist.
        if "already exists" in err.lower():
            print(f"maybe_cut_github_release: release {tag} already exists")
            mark_cut_today()
            return 0
        print(err, file=sys.stderr)
        return create.returncode

    mark_cut_today()
    print(f"maybe_cut_github_release: created {tag}")
    print(
        "maybe_cut_github_release: CI will attach DMG; "
        "then update Casks/mac-stats.rb + sync-homebrew-tap.sh"
    )
    return 0


def shutil_which(name: str) -> str | None:
    from shutil import which

    return which(name)


if __name__ == "__main__":
    raise SystemExit(main())
