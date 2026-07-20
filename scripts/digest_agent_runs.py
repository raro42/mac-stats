#!/usr/bin/env python3
"""Digest ~/.mac-stats/runs.jsonl into improvement candidates.

Usage:
  ./scripts/digest_agent_runs.py
  ./scripts/digest_agent_runs.py --days 7 --out ~/.mac-stats/improvements/latest.md

No LLM required — heuristics only. Feed output into tasks / fast-lane rules.
"""

from __future__ import annotations

import argparse
import json
import statistics
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path


def default_runs_path() -> Path:
    return Path.home() / ".mac-stats" / "runs.jsonl"


def parse_ts(s: str) -> datetime | None:
    try:
        if s.endswith("Z"):
            s = s[:-1] + "+00:00"
        return datetime.fromisoformat(s)
    except Exception:
        return None


def load_runs(path: Path, since: datetime) -> list[dict]:
    if not path.is_file():
        return []
    out = []
    with path.open() as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            ts = parse_ts(str(rec.get("ts", "")))
            if ts is None or ts < since:
                continue
            out.append(rec)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", type=Path, default=default_runs_path())
    ap.add_argument("--days", type=int, default=7)
    ap.add_argument(
        "--out",
        type=Path,
        default=Path.home() / ".mac-stats" / "improvements" / "latest.md",
    )
    args = ap.parse_args()

    since = datetime.now(timezone.utc) - timedelta(days=args.days)
    runs = load_runs(args.runs, since)
    args.out.parent.mkdir(parents=True, exist_ok=True)

    lines: list[str] = []
    lines.append(f"# Agent run digest ({args.days}d)")
    lines.append("")
    lines.append(f"Generated: {datetime.now(timezone.utc).isoformat()}")
    lines.append(f"Source: `{args.runs}`")
    lines.append(f"Turns: **{len(runs)}**")
    lines.append("")

    if not runs:
        lines.append("_No runs in window. Ask Werner something, then re-run._")
        args.out.write_text("\n".join(lines) + "\n")
        print(args.out)
        return 0

    by_lane = Counter(r.get("lane", "?") for r in runs)
    lines.append("## Lane mix")
    for k, v in by_lane.most_common():
        lines.append(f"- `{k}`: {v}")
    lines.append("")

    walls = [int(r.get("wall_ms") or 0) for r in runs]
    lines.append("## Latency")
    lines.append(f"- p50: **{int(statistics.median(walls))} ms**")
    if len(walls) >= 2:
        lines.append(f"- mean: **{int(statistics.mean(walls))} ms**")
    lines.append(f"- max: **{max(walls)} ms**")
    lines.append("")

    slow = sorted(runs, key=lambda r: int(r.get("wall_ms") or 0), reverse=True)[:15]
    lines.append("## Slowest 15")
    for r in slow:
        lines.append(
            f"- {int(r.get('wall_ms') or 0):>7} ms  lane=`{r.get('lane')}`  "
            f"tools={r.get('tools')}  q=`{r.get('question_preview', '')[:80]}`"
        )
    lines.append("")

    # Candidates: full-lane turns that look pre-routable / trivial and were slow
    candidates = []
    for r in runs:
        if r.get("lane") != "full":
            continue
        q = (r.get("question_preview") or "").lower()
        wall = int(r.get("wall_ms") or 0)
        if wall < 15_000:
            continue
        hint = None
        if "time" in q or "uhr" in q or "hora" in q:
            hint = "Promote to INSTANT time/date lane"
        elif q.startswith("search ") or "search for" in q or "look up" in q:
            hint = "Ensure BRAVE/PERPLEXITY pre-route + LITE (skip criteria/verify)"
        elif q.startswith("ping") or q in ("hi", "hello", "hey"):
            hint = "Promote to INSTANT greeting lane"
        elif wall > 60_000 and not r.get("pre_routed"):
            hint = "Investigate meta-LLM cost (criteria/topic/plan/verify); consider lite or smaller judge model"
        if hint:
            candidates.append((wall, hint, r.get("question_preview", ""), r.get("request_id")))

    lines.append("## Improvement candidates")
    if not candidates:
        lines.append("_None this window._")
    else:
        seen = set()
        for wall, hint, q, rid in sorted(candidates, reverse=True)[:20]:
            key = (hint, q[:40])
            if key in seen:
                continue
            seen.add(key)
            lines.append(f"- **{hint}** — {wall} ms — `{q[:100]}` (`{rid}`)")
    lines.append("")

    # Budget violations: full lane with many skipped=false and high wall
    waste = [
        r
        for r in runs
        if r.get("lane") == "full"
        and not r.get("skipped_criteria_llm")
        and int(r.get("wall_ms") or 0) > 45_000
        and int(r.get("tool_steps") or 0) <= 1
    ]
    lines.append("## High-waste pattern (full meta + ≤1 tool)")
    lines.append(f"Count: **{len(waste)}**")
    for r in waste[:10]:
        lines.append(
            f"- {int(r.get('wall_ms') or 0)} ms — `{r.get('question_preview', '')[:90]}`"
        )
    lines.append("")
    lines.append("## Next actions")
    lines.append("1. Implement top candidates as code in `fast_lane.rs` / `pre_routing.rs`.")
    lines.append("2. Re-run digest after a day of Discord traffic.")
    lines.append("3. Keep judge on-failure-only (`agentJudgeOnFailureOnly: true`).")
    lines.append("")

    text = "\n".join(lines) + "\n"
    args.out.write_text(text)
    print(text)
    print(f"Wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
