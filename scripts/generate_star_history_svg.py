#!/usr/bin/env python3
"""Build docs/screens/star-history.svg from GitHub stargazer timestamps.

Requires `gh` authenticated for this repo (owner/collaborator).
Usage: python3 scripts/generate_star_history_svg.py
"""

from __future__ import annotations

import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = "raro42/mac-stats"
OUT = Path(__file__).resolve().parents[1] / "docs" / "screens" / "star-history.svg"


def fetch_starred_at() -> list[datetime]:
    cmd = [
        "gh",
        "api",
        f"repos/{REPO}/stargazers",
        "--paginate",
        "-H",
        "Accept: application/vnd.github.star+json",
        "--jq",
        ".[].starred_at",
    ]
    raw = subprocess.check_output(cmd, text=True)
    times: list[datetime] = []
    for line in raw.splitlines():
        line = line.strip()
        if not line:
            continue
        times.append(datetime.fromisoformat(line.replace("Z", "+00:00")))
    times.sort()
    return times


def cumulative(times: list[datetime]) -> list[tuple[datetime, int]]:
    return [(t, i + 1) for i, t in enumerate(times)]


def svg_path(points: list[tuple[float, float]]) -> str:
    if not points:
        return ""
    parts = [f"M {points[0][0]:.1f} {points[0][1]:.1f}"]
    for x, y in points[1:]:
        parts.append(f"L {x:.1f} {y:.1f}")
    return " ".join(parts)


def render(points: list[tuple[datetime, int]]) -> str:
    width, height = 720, 220
    pad_l, pad_r, pad_t, pad_b = 48, 24, 36, 40
    plot_w = width - pad_l - pad_r
    plot_h = height - pad_t - pad_b

    if not points:
        return (
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
            f'role="img" aria-label="Star growth chart (no stars yet)">'
            f'<rect width="100%" height="100%" fill="#fafafa" rx="12"/>'
            f'<text x="{width/2}" y="{height/2}" text-anchor="middle" '
            f'fill="#666" font-family="ui-sans-serif,system-ui,sans-serif" font-size="14">'
            f"No stars yet — be the first?</text></svg>\n"
        )

    t0 = points[0][0]
    t1 = points[-1][0]
    span = max((t1 - t0).total_seconds(), 1.0)
    n = points[-1][1]
    ymax = max(n, 1)

    xy: list[tuple[float, float]] = []
    for t, count in points:
        x = pad_l + ((t - t0).total_seconds() / span) * plot_w
        y = pad_t + plot_h - (count / ymax) * plot_h
        xy.append((x, y))

    # Extend flat to "now" so the line doesn't stop mid-plot.
    now = datetime.now(timezone.utc)
    if now > t1:
        x_now = pad_l + min(1.0, (now - t0).total_seconds() / span) * plot_w
        xy.append((x_now, xy[-1][1]))

    line = svg_path(xy)
    area = line + f" L {xy[-1][0]:.1f} {pad_t + plot_h:.1f} L {xy[0][0]:.1f} {pad_t + plot_h:.1f} Z"
    last = points[-1]
    title = f"Star growth — {n} ★"
    subtitle = f"Since {t0.date().isoformat()} · last star {last[0].date().isoformat()}"

    # Light-friendly chart (GitHub README default). Soft teal line.
    return f"""\
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}"
  role="img" aria-label="{title}">
  <title>{title}</title>
  <defs>
    <linearGradient id="fill" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#0a7ea4" stop-opacity="0.22"/>
      <stop offset="100%" stop-color="#0a7ea4" stop-opacity="0.02"/>
    </linearGradient>
  </defs>
  <rect width="100%" height="100%" fill="#f7f8fa" rx="12"/>
  <text x="{pad_l}" y="22" fill="#1d1d1f"
    font-family="ui-sans-serif, system-ui, -apple-system, sans-serif"
    font-size="15" font-weight="600">{title}</text>
  <text x="{pad_l}" y="40" fill="#6e6e73"
    font-family="ui-sans-serif, system-ui, -apple-system, sans-serif"
    font-size="11">{subtitle}</text>
  <line x1="{pad_l}" y1="{pad_t + plot_h}" x2="{pad_l + plot_w}" y2="{pad_t + plot_h}"
    stroke="#d2d2d7" stroke-width="1"/>
  <line x1="{pad_l}" y1="{pad_t}" x2="{pad_l}" y2="{pad_t + plot_h}"
    stroke="#d2d2d7" stroke-width="1"/>
  <path d="{area}" fill="url(#fill)"/>
  <path d="{line}" fill="none" stroke="#0a7ea4" stroke-width="2.5"
    stroke-linecap="round" stroke-linejoin="round"/>
  <circle cx="{xy[-1][0]:.1f}" cy="{xy[-1][1]:.1f}" r="4.5" fill="#0a7ea4"/>
  <text x="{pad_l - 8}" y="{pad_t + 4}" text-anchor="end" fill="#6e6e73"
    font-family="ui-sans-serif, system-ui, sans-serif" font-size="10">{ymax}</text>
  <text x="{pad_l - 8}" y="{pad_t + plot_h}" text-anchor="end" fill="#6e6e73"
    font-family="ui-sans-serif, system-ui, sans-serif" font-size="10">0</text>
  <text x="{pad_l}" y="{height - 12}" fill="#6e6e73"
    font-family="ui-sans-serif, system-ui, sans-serif" font-size="10">{t0.date().isoformat()}</text>
  <text x="{pad_l + plot_w}" y="{height - 12}" text-anchor="end" fill="#6e6e73"
    font-family="ui-sans-serif, system-ui, sans-serif" font-size="10">now</text>
</svg>
"""


def main() -> int:
    try:
        times = fetch_starred_at()
    except subprocess.CalledProcessError as exc:
        print(f"gh api failed: {exc}", file=sys.stderr)
        return 1
    chart = render(cumulative(times))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(chart, encoding="utf-8")
    meta = {"repo": REPO, "stars": len(times), "out": str(OUT)}
    print(json.dumps(meta))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
