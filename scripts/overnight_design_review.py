#!/usr/bin/env python3
"""Overnight design-review helper — which feature screenshot is stale?

Usage:
  python3 scripts/overnight_design_review.py
  python3 scripts/overnight_design_review.py --max-age-days 3 --json

Exit 0 always (informational). Prints due=true|false and a recommended surface.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCREENS = ROOT / "screens"
GRACE_DIR = SCREENS / ".polish-grace"

# Rotate priority: Agent Ops / chat / processes often go stale first.
SURFACES = (
    ("feature-agent-ops.png", "CPU window → Agent Ops expanded"),
    ("feature-ai-chat.png", "CPU window → AI / Ollama chat"),
    ("feature-processes.png", "CPU window → process list"),
    ("feature-cpu-metrics.png", "CPU window → metrics rings"),
    ("feature-disk-cleanup.png", "CPU window → Disk Cleanup (scopes expanded)"),
    ("feature-monitors.png", "CPU window → External / Monitors"),
)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--max-age-days", type=float, default=3.0)
    ap.add_argument("--json", action="store_true")
    ap.add_argument(
        "--mark-polished",
        metavar="STEM",
        help="Record polish-without-capture grace for a surface stem (e.g. feature-ai-chat)",
    )
    args = ap.parse_args()
    now = time.time()
    max_age = args.max_age_days * 86400
    grace_max = 7.0 * 86400

    if args.mark_polished:
        GRACE_DIR.mkdir(parents=True, exist_ok=True)
        stem = args.mark_polished.removesuffix(".png")
        (GRACE_DIR / stem).write_text(f"polished {time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}\n")
        print(f"marked polish grace: {stem}")
        return

    rows = []
    for name, how in SURFACES:
        path = SCREENS / name
        grace = GRACE_DIR / Path(name).stem
        in_grace = grace.is_file() and (now - grace.stat().st_mtime) < grace_max
        if not path.is_file():
            rows.append(
                {
                    "file": name,
                    "how": how,
                    "exists": False,
                    "age_days": None,
                    "stale": not in_grace,
                    "polish_grace": in_grace,
                }
            )
            continue
        age = now - path.stat().st_mtime
        rows.append(
            {
                "file": name,
                "how": how,
                "exists": True,
                "age_days": round(age / 86400, 2),
                "stale": (age >= max_age) and not in_grace,
                "polish_grace": in_grace,
                "path": str(path),
            }
        )

    stale = [r for r in rows if r["stale"]]
    pick = stale[0] if stale else min(
        (r for r in rows if r["exists"]),
        key=lambda r: r["age_days"] or 0,
        default=None,
    )
    due = bool(stale)

    payload = {
        "due": due,
        "max_age_days": args.max_age_days,
        "recommended": pick,
        "surfaces": rows,
        "policy": "docs/043_overnight_design_review.md",
    }

    if args.json:
        print(json.dumps(payload, indent=2))
        return

    print(f"due={'true' if due else 'false'}  max_age_days={args.max_age_days}")
    if pick:
        age_s = "missing" if not pick.get("exists") else f"{pick['age_days']}d"
        print(f"recommended: {pick['file']} ({age_s}) — {pick['how']}")
    print("policy: docs/043_overnight_design_review.md")
    for r in rows:
        mark = "STALE" if r["stale"] else ("grace" if r.get("polish_grace") else "ok")
        age_s = "missing" if not r["exists"] else f"{r['age_days']}d"
        print(f"  [{mark}] {r['file']}  {age_s}")


if __name__ == "__main__":
    main()
