#!/usr/bin/env python3
"""Peek sibling harnesses (OpenClaw + Hermes) for ideas worth porting to mac-stats.

Writes ~/.mac-stats/improvements/sibling_harness.md — no network, local git only.
"""
from __future__ import annotations

import subprocess
from datetime import datetime, timezone
from pathlib import Path

HOME = Path.home()
OUT = HOME / ".mac-stats" / "improvements" / "sibling_harness.md"
REPOS = [
    ("OpenClaw", HOME / "projects" / "openclaw"),
    ("Hermes", HOME / "projects" / "hermes-agent"),
]


def recent_commits(repo: Path, n: int = 12) -> list[str]:
    if not (repo / ".git").exists():
        return [f"(missing git at {repo})"]
    try:
        out = subprocess.check_output(
            ["git", "-C", str(repo), "log", f"-{n}", "--oneline", "--no-decorate"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
        return [ln for ln in out.splitlines() if ln.strip()]
    except Exception as e:
        return [f"(git log failed: {e})"]


def interesting_paths(repo: Path) -> list[str]:
    hints = [
        "**/tool*.ts",
        "**/tool*.py",
        "**/session*.ts",
        "**/session*.py",
        "**/agents/**",
        "**/ui/**/sessions*",
        "**/ui/**/agents*",
    ]
    found: list[str] = []
    for pat in hints:
        try:
            out = subprocess.check_output(
                ["git", "-C", str(repo), "ls-files", pat],
                text=True,
                stderr=subprocess.DEVNULL,
            )
            for ln in out.splitlines()[:8]:
                found.append(ln)
        except Exception:
            pass
    return found[:24]


def main() -> int:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        f"# Sibling harness scan",
        f"",
        f"Generated: {datetime.now(timezone.utc).isoformat()}",
        f"",
        f"Use this with `runs.jsonl` digests to pick **one** port per improvement tick.",
        f"",
    ]
    for name, path in REPOS:
        lines.append(f"## {name} (`{path}`)")
        lines.append("")
        lines.append("### Recent commits")
        for c in recent_commits(path):
            lines.append(f"- `{c}`")
        lines.append("")
        lines.append("### Interesting paths")
        paths = interesting_paths(path)
        if not paths:
            lines.append("- _(none matched)_")
        else:
            for p in paths:
                lines.append(f"- `{p}`")
        lines.append("")
    lines.append("## Porting lens (mac-stats)")
    lines.append("")
    lines.append("1. Native tool loop fidelity (schemas + tool role messages).")
    lines.append("2. Session list / resume UX (Agent Ops dashboard).")
    lines.append("3. Bounded memory + compaction hygiene.")
    lines.append("4. Insights / run telemetry → instant-lane candidates.")
    lines.append("5. Discord reconnect robustness.")
    lines.append("")
    OUT.write_text("\n".join(lines) + "\n")
    print(OUT.read_text())
    print(f"Wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
