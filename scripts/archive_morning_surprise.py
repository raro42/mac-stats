#!/usr/bin/env python3
"""Copy morning_surprise_YYYY-MM-DD.md from ~/.mac-stats into the repo archive."""

from __future__ import annotations

import argparse
import shutil
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / "docs" / "ops" / "morning-surprises"
HOME_DIR = Path.home() / ".mac-stats" / "improvements"


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--date",
        default=date.today().isoformat(),
        help="Calendar date YYYY-MM-DD (default: today local)",
    )
    args = p.parse_args()
    name = f"morning_surprise_{args.date}.md"
    src = HOME_DIR / name
    if not src.is_file():
        print(f"missing: {src}", file=sys.stderr)
        return 1
    ARCHIVE.mkdir(parents=True, exist_ok=True)
    dst = ARCHIVE / name
    shutil.copy2(src, dst)
    print(f"archived {src} -> {dst}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
