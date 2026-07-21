#!/usr/bin/env python3
"""Digest ~/.mac-stats/runs.jsonl into improvement candidates.

Usage:
  ./scripts/digest_agent_runs.py
  ./scripts/digest_agent_runs.py --days 7 --out ~/.mac-stats/improvements/latest.md

No LLM required — heuristics only. Feed output into tasks / fast-lane rules.

Suppresses candidates for patterns already shipped (instant version/time/weather)
when the turn timestamp is older than the ship cutoff — those are stale telemetry,
not open work.
"""

from __future__ import annotations

import argparse
import json
import os
import statistics
import tempfile
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path

# Instant lanes / Open-Meteo weather landed ~2026-07-20 afternoon–evening UTC.
SHIPPED_INSTANT_VERSION = datetime(2026, 7, 20, 15, 45, tzinfo=timezone.utc)
SHIPPED_INSTANT_TIME = datetime(2026, 7, 20, 14, 0, tzinfo=timezone.utc)
SHIPPED_INSTANT_WEATHER = datetime(2026, 7, 20, 21, 0, tzinfo=timezone.utc)
SHIPPED_GREETING = datetime(2026, 7, 20, 14, 0, tzinfo=timezone.utc)
SHIPPED_INSTANT_WAKEUP = datetime(2026, 7, 21, 4, 30, tzinfo=timezone.utc)
# v0.1.133 — scheduled SKILL tasks no longer instant-refused for commit/push.
SHIPPED_SKILL_GIT_FASTLANE = datetime(2026, 7, 21, 9, 10, tzinfo=timezone.utc)


def atomic_write_text(path: Path, text: str) -> None:
    """Hermes-style crash-safe write: temp + fsync + os.replace."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_path = tempfile.mkstemp(
        dir=str(path.parent),
        prefix=f".{path.stem}_",
        suffix=".tmp",
    )
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(text)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp_path, path)
    except BaseException:
        try:
            os.unlink(tmp_path)
        except OSError:
            pass
        raise


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


def is_stale_shipped_candidate(hint: str, q: str, ts: datetime | None) -> bool:
    """True when this candidate was already fixed after `ts` (stale digester noise)."""
    if ts is None:
        return False
    hl = hint.lower()
    ql = q.lower()
    if "instant version" in hl and ts < SHIPPED_INSTANT_VERSION:
        return True
    if "instant time" in hl and ts < SHIPPED_INSTANT_TIME:
        return True
    if "greeting" in hl and ts < SHIPPED_GREETING:
        return True
    # Pre-Open-Meteo-instant weather that burned Brave (~28s).
    if ts < SHIPPED_INSTANT_WEATHER and ("wether" in ql or "weather" in ql):
        if (
            "open-meteo" in hl
            or "weather via search" in hl
            or "brave" in hl
            or "zero-tool" in hl
            or "instant" in hl
        ):
            return True
    if "version" in ql and "instant version" in hl and ts < SHIPPED_INSTANT_VERSION:
        return True
    if ts < SHIPPED_INSTANT_WAKEUP and (
        "wake-up" in ql or "wakeup" in ql or "wake up" in ql
    ):
        if "zero-tool" in hl or "instant" in hl or "wake" in hl:
            return True
    if ts < SHIPPED_SKILL_GIT_FASTLANE and "skill:" in ql:
        if (
            "git fast-lane" in hl
            or "ui-weekly" in ql
            or "changelog-weekly" in ql
            or "commit" in ql
            or "push" in ql
        ):
            return True
    return False


def is_trivial_instant_noise(r: dict) -> bool:
    """Drop sub-100ms instant turns from Slowest (clock/ping/false-positive refuse)."""
    if (r.get("lane") or "") != "instant":
        return False
    return int(r.get("wall_ms") or 0) < 100


def is_skill_git_fastlane_false_positive(r: dict) -> bool:
    """Pre-v0.1.133: SKILL weekly tasks hit Instant git refusal (0 LLM).

    `question_preview` is often truncated before trailing `commit+push`, so also
    match known weekly skill ids.
    """
    q = (r.get("question_preview") or "").lower()
    if "skill:" not in q:
        return False
    skillish = (
        "ui-weekly" in q
        or "changelog-weekly" in q
        or "commit" in q
        or "push" in q
        or "cursor_agent" in q
    )
    if not skillish:
        return False
    return (r.get("lane") or "") == "instant"

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
        atomic_write_text(args.out, "\n".join(lines) + "\n")
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

    slow_pool = [r for r in runs if not is_trivial_instant_noise(r)]
    slow = sorted(slow_pool, key=lambda r: int(r.get("wall_ms") or 0), reverse=True)[:15]
    lines.append("## Slowest 15")
    for r in slow:
        lines.append(
            f"- {int(r.get('wall_ms') or 0):>7} ms  lane=`{r.get('lane')}`  "
            f"tools={r.get('tools')}  q=`{r.get('question_preview', '')[:80]}`"
        )
    lines.append("")

    candidates: list[tuple] = []
    stale: list[tuple] = []
    for r in runs:
        q = (r.get("question_preview") or "").lower()
        wall = int(r.get("wall_ms") or 0)
        lane = r.get("lane") or "?"
        tools = r.get("tools") or []
        tool_steps = int(r.get("tool_steps") or 0)
        ts = parse_ts(str(r.get("ts", "")))
        hint = None
        if is_skill_git_fastlane_false_positive(r):
            hint = "Scheduled SKILL blocked by git fast-lane — exempt SKILL/CURSOR_AGENT"
        elif wall >= 5_000 and lane in ("lite", "direct", "full") and (
            not tools and tool_steps == 0
        ):
            if "version" in q:
                hint = "Promote to INSTANT version lane"
            elif "time" in q or "uhr" in q or "hora" in q or "date" in q:
                hint = "Promote to INSTANT time/date lane"
            elif q.strip() in ("ping", "hi", "hello", "hey", "thanks", "thank you"):
                hint = "Promote to INSTANT greeting/thanks lane"
            elif wall >= 15_000:
                hint = "Zero-tool slow turn — consider instant/pre-route or smaller model"
        elif lane == "full" and wall >= 15_000:
            if "time" in q or "uhr" in q or "hora" in q:
                hint = "Promote to INSTANT time/date lane"
            elif q.startswith("search ") or "search for" in q or "look up" in q:
                hint = "Ensure BRAVE/PERPLEXITY pre-route + LITE (skip criteria/verify)"
            elif q.startswith("ping") or q in ("hi", "hello", "hey"):
                hint = "Promote to INSTANT greeting lane"
            elif wall > 60_000 and not r.get("pre_routed"):
                hint = "Investigate meta-LLM cost (criteria/topic/plan/verify); consider lite or smaller judge model"
        elif (
            wall >= 15_000
            and lane == "direct"
            and tools
            and ("weather" in q or "wether" in q)
            and any("BRAVE" in str(t).upper() or "PERPLEXITY" in str(t).upper() for t in tools)
        ):
            hint = "Weather via search — prefer Open-Meteo INSTANT when place is clear"
        if not hint:
            continue
        preview = r.get("question_preview", "")
        rid = r.get("request_id")
        row = (wall, hint, preview, rid, ts)
        if is_stale_shipped_candidate(hint, preview, ts):
            stale.append(row)
        else:
            candidates.append(row)

    lines.append("## Improvement candidates")
    if not candidates:
        lines.append("_None this window (open)._")
    else:
        seen = set()
        for wall, hint, q, rid, _ts in sorted(candidates, reverse=True)[:20]:
            key = (hint, q[:40])
            if key in seen:
                continue
            seen.add(key)
            lines.append(f"- **{hint}** — {wall} ms — `{q[:100]}` (`{rid}`)")
    lines.append("")

    lines.append("## Stale / already shipped (ignored)")
    if not stale:
        lines.append("_None._")
    else:
        lines.append(
            "_These matched heuristics but predate the instant-lane / Open-Meteo ship cutoff — not open work._"
        )
        seen = set()
        for wall, hint, q, rid, ts in sorted(stale, reverse=True)[:15]:
            key = (hint, q[:40])
            if key in seen:
                continue
            seen.add(key)
            ts_s = ts.isoformat() if ts else "?"
            lines.append(f"- ~~{hint}~~ — {wall} ms — `{q[:80]}` (`{rid}`, {ts_s})")
    lines.append("")

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
    if candidates:
        lines.append("1. Implement open candidates in `fast_lane.rs` / `pre_routing.rs` / tools.")
    else:
        lines.append("1. No open digester candidates — prefer sibling-harness ports or fresh Discord traffic.")
    lines.append("2. Re-run digest after a day of Discord traffic.")
    lines.append("3. Keep judge on-failure-only (`agentJudgeOnFailureOnly: true`).")
    lines.append("")

    text = "\n".join(lines) + "\n"
    atomic_write_text(args.out, text)

    # Machine-readable summary for Agent Ops.
    json_out = args.out.with_suffix(".json")
    open_items = [
        {
            "wall_ms": wall,
            "hint": hint,
            "question_preview": (q or "")[:120],
            "request_id": rid,
            "ts": ts.isoformat() if ts else None,
        }
        for wall, hint, q, rid, ts in sorted(candidates, reverse=True)[:20]
    ]
    stale_items = [
        {
            "wall_ms": wall,
            "hint": hint,
            "question_preview": (q or "")[:120],
            "request_id": rid,
            "ts": ts.isoformat() if ts else None,
        }
        for wall, hint, q, rid, ts in sorted(stale, reverse=True)[:20]
    ]
    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "days": args.days,
        "turns": len(runs),
        "open_count": len(candidates),
        "stale_count": len(stale),
        "p50_ms": int(statistics.median(walls)) if walls else 0,
        "max_ms": max(walls) if walls else 0,
        "by_lane": dict(by_lane),
        "open": open_items,
        "stale": stale_items,
        "markdown_path": str(args.out),
        "source": "python",
    }
    atomic_write_text(json_out, json.dumps(payload, indent=2) + "\n")

    print(text)
    print(f"Wrote {args.out}")
    print(f"Wrote {json_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
