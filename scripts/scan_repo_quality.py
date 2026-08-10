#!/usr/bin/env python3
"""Heuristic repo-quality scan for mac-stats (weekly quality monitor).

Exit 0 = no fails (warns allowed). Exit 1 = at least one fail.

Usage:
  python3 scripts/scan_repo_quality.py
  python3 scripts/scan_repo_quality.py --json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Paths that must NOT exist at repo root (moved or deleted).
FORBIDDEN_ROOT = (
    "package.json",
    "package-lock.json",
    "screens",
    "tasks",
    "agents-tasks",
    "003-tester",
    "004-closing-reviewer",
    "005-openclaw-reviewer",
    "006-feature-coder",
)

# Paths that should exist.
REQUIRED = (
    "agents/README.md",
    "docs/screens/README.md",
    "docs/screens/mac-stats-features.mp4",
    "docs/044_repo_quality_hygiene.md",
    "docs/skills/quality-weekly-review.md",
    "agents/007-quality-monitor/PROMPT.md",
    "src-tauri/Cargo.toml",
    "run",
)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    fails: list[str] = []
    warns: list[str] = []

    for name in FORBIDDEN_ROOT:
        p = ROOT / name
        if p.exists():
            fails.append(f"forbidden root path still present: {name}")

    for rel in REQUIRED:
        if not (ROOT / rel).is_file():
            fails.append(f"missing required file: {rel}")

    readme = ROOT / "README.md"
    if readme.is_file():
        text = readme.read_text(errors="replace")
        if "docs/screens/" not in text:
            fails.append("README.md has no docs/screens/ references")
        if "screens/mac-stats-features.mp4" in text and "docs/screens/mac-stats-features" not in text:
            fails.append("README.md still links old screens/ demo path")
        # Prefer a playable URL (CDN) or docs/screens path — bare GitHub blob is weak UX
        if "mac-stats-features.mp4" in text and "jsdelivr" not in text and "cdn." not in text:
            warns.append(
                "README demo video has no CDN/playable URL (GitHub blob may not preview MP4)"
            )

    gitignore = ROOT / ".gitignore"
    if gitignore.is_file():
        gi = gitignore.read_text(errors="replace")
        if "\nscreens/.polish-grace/" in gi or gi.startswith("screens/.polish-grace/"):
            fails.append(".gitignore still names root screens/.polish-grace")
        elif "docs/screens/.polish-grace/" not in gi:
            warns.append(".gitignore missing docs/screens/.polish-grace/")

    for rel, needle, msg in (
        ("scripts/overnight_design_review.py", 'ROOT / "screens"', "overnight_design_review still uses root screens/"),
        ("scripts/digest_agent_runs.py", 'repo_root / "screens"', "digest_agent_runs still uses root screens/"),
    ):
        p = ROOT / rel
        if p.is_file() and needle in p.read_text(errors="replace"):
            fails.append(msg)

    # Root litter: unexpected top-level dirs that look like agent/task leftovers
    suspicious_prefixes = ("UNTESTED-", "CLOSED-", "WIP-", "FEAT-", "TESTING-", "TESTPLAN-")
    for child in ROOT.iterdir():
        if child.name.startswith(suspicious_prefixes):
            fails.append(f"task-like file at repo root: {child.name}")

    # Optional: empty node_modules leftover (gitignored)
    nm = ROOT / "node_modules"
    if nm.is_dir():
        warns.append("local node_modules/ present (gitignored) — safe to rm -rf if unused")

    report = {"fails": fails, "warns": warns, "ok": not fails}
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"repo={ROOT}")
        if fails:
            print(f"FAILS ({len(fails)}):")
            for f in fails:
                print(f"  - {f}")
        else:
            print("FAILS: none")
        if warns:
            print(f"WARNS ({len(warns)}):")
            for w in warns:
                print(f"  - {w}")
        else:
            print("WARNS: none")
        print("ok" if not fails else "needs_fix")

    return 1 if fails else 0


if __name__ == "__main__":
    raise SystemExit(main())
