#!/usr/bin/env python3
"""
Plot mac-stats-reviewer Track A autopilot results.tsv (experiment vs judge score).

Usage:
  python3 scripts/plot_autoresearch_snapshot.py \\
    docs/autoresearch-snapshots/amvara8-005-openclaw-24h_results.tsv

Writes <input_basename>.png and <input_basename>.svg next to the TSV unless --out-dir is set.
"""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np


STATUS_COLORS = {
    "baseline": "#2563eb",
    "keep": "#16a34a",
    "discard": "#94a3b8",
    "keep_tie_max": "#0d9488",
}


def load_rows(path: Path) -> list[dict[str, str]]:
    text = path.read_text(encoding="utf-8")
    return list(csv.DictReader(text.splitlines()))


def load_state_json(path: Path | None) -> dict | None:
    if path is None or not path.is_file():
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "tsv",
        type=Path,
        help="Path to results.tsv (e.g. from amvara8 run dir)",
    )
    ap.add_argument(
        "--state",
        type=Path,
        default=None,
        help="Optional state.json for subtitle (same snapshot dir as TSV)",
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=None,
        help="Output directory (default: same directory as TSV)",
    )
    ap.add_argument(
        "--dpi",
        type=int,
        default=150,
        help="PNG resolution (default 150)",
    )
    args = ap.parse_args()

    tsv = args.tsv.resolve()
    rows = load_rows(tsv)
    if not rows:
        raise SystemExit(f"no rows in {tsv}")

    exps = [int(r["experiment"]) for r in rows]
    scores = [int(r["score"]) for r in rows]
    max_score = int(rows[0]["max_score"])
    statuses = [r["status"] for r in rows]

    best_running: list[int] = []
    b = scores[0]
    for s in scores:
        b = max(b, s)
        best_running.append(b)

    out_dir = (args.out_dir or tsv.parent).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    base = tsv.stem

    state = load_state_json(args.state)
    if state is None:
        candidate = tsv.parent / tsv.name.replace("_results.tsv", "_state.json")
        state = load_state_json(candidate)

    subtitle_parts = [
        f"n={len(rows)} experiments  ·  max {max_score}/15",
    ]
    if state:
        subtitle_parts.append(
            f"state exp={state.get('experiment')} best={state.get('best_score')} @ {state.get('updated_utc', '')[:19]}Z"
        )

    try:
        plt.style.use("seaborn-v0_8-whitegrid")
    except OSError:
        plt.style.use("ggplot")
    fig, axes = plt.subplots(
        2,
        1,
        figsize=(11, 7),
        height_ratios=[2.2, 1],
        constrained_layout=True,
    )
    ax0, ax1 = axes

    for status, color in STATUS_COLORS.items():
        xs = [e for e, st in zip(exps, statuses) if st == status]
        ys = [s for s, st in zip(scores, statuses) if st == status]
        if xs:
            ax0.scatter(
                xs,
                ys,
                c=color,
                s=36,
                alpha=0.85,
                edgecolors="white",
                linewidths=0.4,
                label=status.replace("_", " "),
                zorder=3,
            )

    ax0.plot(exps, scores, color="#64748b", linewidth=0.9, alpha=0.35, zorder=1)
    ax0.plot(
        exps,
        best_running,
        color="#c026d3",
        linewidth=2,
        label="best so far",
        zorder=2,
    )
    ax0.axhline(max_score, color="#e11d48", linestyle="--", linewidth=1, alpha=0.6, label=f"ceiling ({max_score})")

    ax0.set_xlabel("experiment")
    ax0.set_ylabel("judge score")
    ax0.set_title(f"Track A autopilot — {tsv.name}")
    ax0.set_ylim(max(0, min(scores) - 1), max_score + 0.5)
    ax0.legend(loc="lower right", fontsize=8, ncol=2)
    fig.text(0.5, 0.02, " · ".join(subtitle_parts), ha="center", fontsize=9, color="#475569")

    # Histogram of scores
    lo, hi = min(scores), max(scores)
    edges = np.arange(lo, hi + 2) - 0.5
    ax1.hist(scores, bins=edges, color="#64748b", edgecolor="white")
    ax1.set_xlabel("score")
    ax1.set_ylabel("count")
    ax1.set_title("Score distribution")

    png_path = out_dir / f"{base}.png"
    svg_path = out_dir / f"{base}.svg"
    fig.savefig(png_path, dpi=args.dpi)
    fig.savefig(svg_path)
    plt.close(fig)

    print(f"Wrote {png_path}")
    print(f"Wrote {svg_path}")


if __name__ == "__main__":
    main()
