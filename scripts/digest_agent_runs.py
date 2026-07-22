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
# Redmine keys synced into ~/.mac-stats/.config.env for LaunchAgent installs.
SHIPPED_REDMINE_HOME_CONFIG = datetime(2026, 7, 21, 10, 25, tzinfo=timezone.utc)
# v0.1.206 — overnight / last-night improvements asks are instant.
SHIPPED_INSTANT_OVERNIGHT_IMPROVEMENTS = datetime(2026, 7, 22, 8, 40, tzinfo=timezone.utc)
# v0.1.215 — Discord reach / see-channels / other-agent meta asks are instant.
SHIPPED_INSTANT_DISCORD_REACH = datetime(2026, 7, 22, 20, 0, tzinfo=timezone.utc)
# v0.1.164 — short acks / sign-offs are instant.
SHIPPED_INSTANT_SHORT_ACK = datetime(2026, 7, 21, 20, 10, tzinfo=timezone.utc)
# v0.1.176 — short identity/role affirmations are instant.
SHIPPED_INSTANT_IDENTITY = datetime(2026, 7, 22, 0, 15, tzinfo=timezone.utc)
# v0.1.224 — Redmine user-chat capability asks are instant.
SHIPPED_INSTANT_REDMINE_USER_CHAT = datetime(2026, 7, 22, 22, 50, tzinfo=timezone.utc)


def looks_like_short_ack(q: str) -> bool:
    """Mirror fast_lane::is_short_ack_or_signoff (normalized lower text)."""
    if "?" in q:
        return False
    n = q.strip()
    if n in (
        "ok",
        "okay",
        "k",
        "kk",
        "cool",
        "nice",
        "nice one",
        "nice answer",
        "got it",
        "all good",
        "np",
        "no worries",
        "bye",
        "goodbye",
        "cya",
        "see you",
        "later",
        "perfect",
        "great",
        "awesome",
        "neat",
        "sweet",
        "alright",
        "sounds good",
        "fair enough",
        "👍",
        "👌",
    ):
        return True
    if len(n) > 140:
        return False
    starts = n.startswith(
        ("ok", "okay", "cool", "nice", "got it", "alright", "no worries", "sounds good")
    )
    if not starts:
        return False
    return (
        len(n) <= 48
        or "no worries" in n
        or "bye" in n
        or "myself" in n
        or "later" in n
        or "all good" in n
        or "find out" in n
    )


def looks_like_identity_affirmation(q: str) -> bool:
    n = q.strip()
    if "?" in n or len(n) > 180:
        return False
    if not (n.startswith("you are ") or n.startswith("you're ") or n.startswith("youre ")):
        return False
    return any(
        x in n
        for x in (
            "working for",
            "online",
            "assistant",
            " agent",
            "bot",
            "on various channel",
        )
    )


def looks_like_wakeup(q: str) -> bool:
    return "wake-up" in q or "wakeup" in q or "wake up" in q


def looks_like_overnight_improvements(q: str) -> bool:
    if not (
        "improvement" in q
        or "what shipped" in q
        or "what changed" in q
        or "coding session" in q
    ):
        return False
    return "last night" in q or "overnight" in q or "coding session" in q


def looks_like_version_ask(q: str) -> bool:
    n = q.strip()
    if "version" not in n:
        return False
    # Avoid "version of the API" style task asks
    return (
        "what version" in n
        or n.startswith("version")
        or "your version" in n
        or "which version" in n
        or n in ("version?", "version")
    )


def looks_like_discord_reach(q: str) -> bool:
    n = q.strip()
    discordish = any(x in n for x in ("discord", "amvara", "server", "guild", "channel"))
    if discordish and (
        "talking on" in n
        or "ok talking" in n
        or "okay talking" in n
        or "cross check" in n
        or "are you online" in n
        or "are you connected" in n
    ):
        return True
    about_channels = "channel" in n
    about_other = any(
        x in n
        for x in (
            "another agent",
            "other agent",
            "other agents",
            "another bot",
            "other bot",
            "other bots",
        )
    )
    if not about_channels and not about_other:
        return False
    return any(
        x in n
        for x in (
            "can you see",
            "do you see",
            "see channels",
            "talking to",
            "talk to another",
            "talk to other",
            "are you talking",
            "may you",
            "be talking",
        )
    )


def is_now_instant_slowest_noise(r: dict) -> bool:
    """Drop historical turns from Slowest when they match shipped instant patterns."""
    if (r.get("lane") or "") == "instant":
        return is_trivial_instant_noise(r)
    q = (r.get("question_preview") or "").lower()
    tools = r.get("tools") or []
    tool_steps = int(r.get("tool_steps") or 0)
    ts = parse_ts(str(r.get("ts", "")))
    # Pre-Open-Meteo weather that burned Brave/Perplexity.
    if ts is not None and ts < SHIPPED_INSTANT_WEATHER and (
        "weather" in q or "wether" in q
    ):
        if any(
            "BRAVE" in str(t).upper() or "PERPLEXITY" in str(t).upper() for t in tools
        ):
            return True
    # Pre-home-config Redmine LaunchAgent misses.
    if (
        ts is not None
        and ts < SHIPPED_REDMINE_HOME_CONFIG
        and "REDMINE_API" in [str(t) for t in tools]
        and "redmine" in q
    ):
        return True
    if tools or tool_steps > 0:
        return False
    if looks_like_short_ack(q) or looks_like_identity_affirmation(q):
        return True
    if looks_like_wakeup(q) or looks_like_overnight_improvements(q):
        return True
    if looks_like_version_ask(q) or looks_like_discord_reach(q):
        return True
    # Redmine user-chat capability (v0.1.224).
    if (
        "redmine" in q
        and ("talk to" in q or "chat with" in q or "message " in q)
        and "ticket" not in q
        and "issue" not in q
    ):
        return True
    return False


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
    if ts < SHIPPED_REDMINE_HOME_CONFIG and "redmine" in ql and "home config" in hl:
        return True
    if ts < SHIPPED_INSTANT_OVERNIGHT_IMPROVEMENTS and (
        "improvement" in ql
        or "what shipped" in ql
        or "what changed" in ql
    ):
        if (
            "last night" in ql
            or "overnight" in ql
            or "coding session" in ql
        ) and ("zero-tool" in hl or "instant" in hl):
            return True
    if ts < SHIPPED_INSTANT_DISCORD_REACH and (
        "channel" in ql
        or "another agent" in ql
        or "other agent" in ql
        or "other bot" in ql
        or "discord" in ql
        or "amvara" in ql
    ):
        if (
            "can you see" in ql
            or "see channels" in ql
            or "talking to" in ql
            or "talking on" in ql
            or "ok talking" in ql
            or "okay talking" in ql
            or "cross check" in ql
            or "may you" in ql
            or "be talking" in ql
        ) and ("zero-tool" in hl or "instant" in hl):
            return True
    if ts < SHIPPED_INSTANT_SHORT_ACK and (
        "short ack" in hl or "sign-off" in hl or "sign off" in hl
    ):
        return True
    if ts < SHIPPED_INSTANT_IDENTITY and "identity" in hl:
        return True
    if ts < SHIPPED_INSTANT_SHORT_ACK and looks_like_short_ack(ql) and (
        "zero-tool" in hl or "instant" in hl
    ):
        return True
    if ts < SHIPPED_INSTANT_IDENTITY and looks_like_identity_affirmation(ql) and (
        "zero-tool" in hl or "instant" in hl
    ):
        return True
    if ts < SHIPPED_INSTANT_REDMINE_USER_CHAT and "redmine" in ql and (
        "talk to" in ql or "chat with" in ql or "user chat" in hl or "redmine user" in hl
    ):
        if "ticket" not in ql and ("zero-tool" in hl or "instant" in hl or "capability" in hl):
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

    slow_pool = [
        r
        for r in runs
        if not is_trivial_instant_noise(r) and not is_now_instant_slowest_noise(r)
    ]
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
        elif (
            wall >= 5_000
            and "REDMINE_API" in [str(t) for t in tools]
            and "redmine" in q
            and ts is not None
            and ts < SHIPPED_REDMINE_HOME_CONFIG
        ):
            # Pre-home-config sync: LaunchAgent could not see src-tauri/.config.env keys.
            hint = "Sync REDMINE_* into ~/.mac-stats/.config.env (LaunchAgent home config)"
        elif wall >= 5_000 and lane in ("lite", "direct", "full") and (
            not tools and tool_steps == 0
        ):
            if looks_like_short_ack(q):
                hint = "Promote to INSTANT short ack/sign-off lane"
            elif looks_like_identity_affirmation(q):
                hint = "Promote to INSTANT identity affirmation lane"
            elif (
                "redmine" in q
                and ("talk to" in q or "chat with" in q or "message " in q)
                and "ticket" not in q
                and "issue" not in q
            ):
                hint = "Promote to INSTANT Redmine user-chat capability lane"
            elif "version" in q:
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
