#!/usr/bin/env python3
"""Fail if harness_ops grows past budget or pNN clones return."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "src-tauri/src/commands/harness_ops"
# Post-collapse budget (was 135k in one file). Tighten over time.
BUDGET_TOTAL = 28000
BUDGET_FILE = 10000
PNN = re.compile(
    r"fn looks_like_results_tsv_p\d+_request|"
    r"fn format_results_tsv_p\d+_gateway|"
    r"fn count_results_tsv_p\d+\b"
)


def main() -> int:
    if not DIR.is_dir():
        print("missing", DIR, file=sys.stderr)
        return 1
    total = 0
    bad: list[str] = []
    for p in sorted(DIR.glob("*")):
        if p.suffix not in {".rs"} and not p.name.endswith(".inc.rs"):
            continue
        if not p.is_file():
            continue
        text = p.read_text()
        n = len(text.splitlines())
        total += n
        if n > BUDGET_FILE:
            bad.append(f"{p.name}: {n} lines > {BUDGET_FILE}")
        for m in PNN.finditer(text):
            bad.append(f"{p.name}: forbidden pNN clone: {m.group(0)}")
    if total > BUDGET_TOTAL:
        bad.append(f"total {total} lines > {BUDGET_TOTAL}")
    print(f"harness_ops total_lines={total}")
    if bad:
        print("FAIL:", file=sys.stderr)
        for b in bad:
            print(" ", b, file=sys.stderr)
        return 1
    print("OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
