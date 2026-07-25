#!/usr/bin/env python3
"""mac-stats autoresearch ratchet (Karpathy-style keep/discard).

Usage:
  python3 scripts/autoresearch_ratchet.py baseline
  python3 scripts/autoresearch_ratchet.py verify [--test-filter REGEX]
  python3 scripts/autoresearch_ratchet.py keep --description "…"
  python3 scripts/autoresearch_ratchet.py discard --start-sha SHA --description "…"
  python3 scripts/autoresearch_ratchet.py status

Results (untracked): ~/.mac-stats/improvements/autoresearch/results.tsv
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SRC_TAURI = REPO / "src-tauri"
RESULTS_DIR = Path.home() / ".mac-stats" / "improvements" / "autoresearch"
RESULTS_TSV = RESULTS_DIR / "results.tsv"
HEADER = "ts\tcommit\tstatus\tdescription\n"

# Default smoke tests — fast, cover recent harness-sensitive surfaces.
DEFAULT_TEST_FILTER = "curated_memory|memory_save_multiline|detects_pollution|slugify_basic|thin_summary"


def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=str(cwd or REPO),
        text=True,
        capture_output=True,
    )


def git_rev() -> str:
    r = run(["git", "rev-parse", "--short=7", "HEAD"])
    return (r.stdout or "unknown").strip() or "unknown"


def ensure_results() -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    if not RESULTS_TSV.exists():
        RESULTS_TSV.write_text(HEADER, encoding="utf-8")


def append_row(status: str, description: str, commit: str | None = None) -> None:
    ensure_results()
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    commit = commit or git_rev()
    desc = (description or "").replace("\t", " ").replace("\n", " ").strip()
    with RESULTS_TSV.open("a", encoding="utf-8") as f:
        f.write(f"{ts}\t{commit}\t{status}\t{desc}\n")
    print(f"ratchet: logged {status} @ {commit} — {desc}")


def cmd_baseline(_args: argparse.Namespace) -> int:
    ensure_results()
    print(f"ratchet: repo={REPO}")
    print(f"ratchet: HEAD={git_rev()}")
    print(f"ratchet: results={RESULTS_TSV}")
    rc = cmd_verify(
        argparse.Namespace(test_filter=DEFAULT_TEST_FILTER, skip_tests=False)
    )
    if rc == 0:
        append_row("baseline", "verify ok at experiment start")
    else:
        append_row("crash", "baseline verify failed")
    return rc


def cmd_verify(args: argparse.Namespace) -> int:
    print("ratchet: cargo check …")
    check = run(["cargo", "check"], cwd=SRC_TAURI)
    if check.returncode != 0:
        sys.stderr.write(check.stderr or check.stdout or "cargo check failed\n")
        print("ratchet: VERIFY FAIL (cargo check)")
        return check.returncode or 1

    if getattr(args, "skip_tests", False):
        print("ratchet: VERIFY OK (check only)")
        return 0

    filt = (args.test_filter or DEFAULT_TEST_FILTER).strip()
    print(f"ratchet: cargo test --lib '{filt}' …")
    test = run(
        ["cargo", "test", "--lib", filt, "--", "--test-threads=4"],
        cwd=SRC_TAURI,
    )
    if test.returncode != 0:
        # Show tail of failure
        out = (test.stdout or "") + (test.stderr or "")
        sys.stderr.write("\n".join(out.splitlines()[-40:]) + "\n")
        print("ratchet: VERIFY FAIL (tests)")
        return test.returncode or 1

    print("ratchet: VERIFY OK")
    return 0


def cmd_keep(args: argparse.Namespace) -> int:
    append_row("keep", args.description or "kept")
    return 0


def cmd_discard(args: argparse.Namespace) -> int:
    start = (args.start_sha or "").strip()
    if not start:
        print("ratchet: discard requires --start-sha", file=sys.stderr)
        return 2
    # Safety: only reset if dirty or HEAD moved past start
    head = run(["git", "rev-parse", "HEAD"]).stdout.strip()
    start_full = run(["git", "rev-parse", start]).stdout.strip()
    if not start_full:
        print(f"ratchet: bad --start-sha {start!r}", file=sys.stderr)
        return 2
    status = run(["git", "status", "--porcelain"])
    dirty = bool(status.stdout.strip())
    if head != start_full or dirty:
        print(f"ratchet: resetting to {start_full[:7]} (was {head[:7]}, dirty={dirty})")
        reset = run(["git", "reset", "--hard", start_full])
        if reset.returncode != 0:
            sys.stderr.write(reset.stderr or reset.stdout or "git reset failed\n")
            append_row("crash", f"reset failed: {args.description or ''}")
            return reset.returncode or 1
        clean = run(["git", "clean", "-fd", "--", "src", "src-tauri/src", "src-tauri/dist"])
        if clean.returncode != 0:
            sys.stderr.write(clean.stderr or "")
    else:
        print("ratchet: already at start sha; nothing to reset")
    append_row("discard", args.description or "discarded", commit=start_full[:7])
    return 0


def cmd_status(_args: argparse.Namespace) -> int:
    ensure_results()
    text = RESULTS_TSV.read_text(encoding="utf-8")
    lines = [ln for ln in text.splitlines() if ln.strip()]
    print(f"ratchet: {RESULTS_TSV} ({max(0, len(lines) - 1)} rows)")
    for ln in lines[-12:]:
        print(ln)
    keeps = sum(1 for ln in lines[1:] if "\tkeep\t" in ln)
    discards = sum(1 for ln in lines[1:] if "\tdiscard\t" in ln)
    print(f"ratchet: summary keep={keeps} discard={discards}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)

    sub.add_parser("baseline", help="Record HEAD and run verify")
    v = sub.add_parser("verify", help="cargo check + focused tests")
    v.add_argument(
        "--test-filter",
        default=DEFAULT_TEST_FILTER,
        help="cargo test --lib filter regex",
    )
    v.add_argument(
        "--skip-tests",
        action="store_true",
        help="Only cargo check (faster; weaker gate)",
    )
    k = sub.add_parser("keep", help="Log a successful keep")
    k.add_argument("--description", "-d", required=True)
    d = sub.add_parser("discard", help="Hard-reset to start sha and log discard")
    d.add_argument("--start-sha", required=True)
    d.add_argument("--description", "-d", default="discarded")
    sub.add_parser("status", help="Show recent results.tsv rows")

    args = ap.parse_args()
    if args.cmd == "baseline":
        return cmd_baseline(args)
    if args.cmd == "verify":
        return cmd_verify(args)
    if args.cmd == "keep":
        return cmd_keep(args)
    if args.cmd == "discard":
        return cmd_discard(args)
    if args.cmd == "status":
        return cmd_status(args)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
