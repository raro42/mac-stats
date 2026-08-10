#!/usr/bin/env python3
"""Heuristic scan of ~/.mac-stats/debug.log for ERROR / WARN / panic clusters.

Read-only by default. Does not truncate or rotate the log.

Usage:
  python3 scripts/scan_debug_log_errors.py
  python3 scripts/scan_debug_log_errors.py --minutes 120
  python3 scripts/scan_debug_log_errors.py --write-finding
"""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path

DEFAULT_LOG = Path.home() / ".mac-stats" / "debug.log"
REPO_FINDINGS = Path(__file__).resolve().parents[1] / "agents" / "agents-tasks"

# Lines matching these are usually noise (extend carefully).
BENIGN = re.compile(
    r"(HTTP/2.*GoAway|SSRF blocked|could not resolve referenced message|"
    r"Soft-delete failed|Missing Access)",
    re.I,
)

INTERESTING = re.compile(
    r"\b(ERROR|WARN|panic|another instance is already running|"
    r"Gateway.*fail|Bot connected|exiting this launch)\b",
    re.I,
)

TS_RE = re.compile(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})")


def parse_ts(line: str) -> datetime | None:
    m = TS_RE.match(line)
    if not m:
        return None
    try:
        return datetime.strptime(m.group(1), "%Y-%m-%dT%H:%M:%S").replace(tzinfo=timezone.utc)
    except ValueError:
        return None


def signature(line: str) -> str:
    # Drop timestamps / PIDs-ish noise for clustering
    s = re.sub(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z?", "", line)
    s = re.sub(r"\b\d+\b", "N", s)
    s = re.sub(r"\s+", " ", s).strip()
    return s[:200]


def next_log_id(findings_dir: Path) -> int:
    n = 0
    for p in findings_dir.glob("log-*.md"):
        m = re.match(r"log-(\d+)", p.name)
        if m:
            n = max(n, int(m.group(1)))
    return n + 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--log", type=Path, default=DEFAULT_LOG)
    ap.add_argument("--minutes", type=int, default=180, help="Look back window (default 180)")
    ap.add_argument("--write-finding", action="store_true", help="Write one stub finding for top cluster")
    ap.add_argument("--max-lines", type=int, default=200000)
    args = ap.parse_args()

    if not args.log.is_file():
        print(f"No log at {args.log}", file=sys.stderr)
        return 1

    cutoff = datetime.now(timezone.utc) - timedelta(minutes=args.minutes)
    counts: Counter[str] = Counter()
    samples: dict[str, str] = {}
    total = 0
    matched = 0

    with args.log.open("r", errors="replace") as f:
        # Prefer tail for large files
        try:
            f.seek(0, 2)
            size = f.tell()
            f.seek(max(0, size - 8_000_000))
            if f.tell() > 0:
                f.readline()
        except OSError:
            f.seek(0)

        for line in f:
            total += 1
            if total > args.max_lines:
                break
            if not INTERESTING.search(line):
                continue
            if BENIGN.search(line):
                continue
            ts = parse_ts(line)
            if ts is not None and ts < cutoff:
                continue
            # Prefer ERROR/panic over routine INFO "Bot connected"
            if re.search(r"\bINFO\b", line) and not re.search(
                r"another instance|exiting this launch", line, re.I
            ):
                continue
            matched += 1
            sig = signature(line)
            counts[sig] += 1
            samples.setdefault(sig, line.rstrip()[:300])

    print(f"log={args.log}")
    print(f"window_minutes={args.minutes} interesting_lines={matched}")
    if not counts:
        print("No ERROR/WARN/panic clusters in window.")
        return 0

    print("\nTop clusters:")
    for sig, n in counts.most_common(15):
        print(f"  {n:4d}  {samples[sig]}")

    if args.write_finding:
        REPO_FINDINGS.mkdir(parents=True, exist_ok=True)
        top_sig, top_n = counts.most_common(1)[0]
        digest = hashlib.sha256(top_sig.encode()).hexdigest()[:10]
        # Skip if recent finding mentions digest
        for p in REPO_FINDINGS.glob("log-*.md"):
            if digest in p.read_text(errors="replace"):
                print(f"\nFinding already exists for signature {digest}: {p.name}")
                return 0
        nid = next_log_id(REPO_FINDINGS)
        slug = re.sub(r"[^a-z0-9]+", "-", top_sig.lower())[:40].strip("-") or "error"
        path = REPO_FINDINGS / f"log-{nid:03d}-{slug}.md"
        now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
        path.write_text(
            f"# log-{nid:03d}: {slug}\n\n"
            f"- Created: {now}\n"
            f"- Count in window: {top_n}\n"
            f"- Signature: `{digest}`\n\n"
            f"## Sample\n\n```\n{samples[top_sig]}\n```\n\n"
            f"## Next\n\nTriage in mac-stats; fix or mark benign in agents/log-monitor/README.md.\n"
        )
        print(f"\nWrote {path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
