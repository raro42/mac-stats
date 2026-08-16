#!/usr/bin/env python3
"""Commit + push any pending safe work (nightly ~23:00 backstop).

Usage:
  python3 scripts/overnight_git_flush.py
  python3 scripts/overnight_git_flush.py --dry-run

Skips secrets. Does not force-push. Exits 0 when clean or flush succeeded.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SECRET_HINTS = (
    ".env",
    ".config.env",
    "credentials",
    "secret",
    "id_rsa",
    "id_ed25519",
    ".pem",
    "token.json",
)


def run(args: list[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=check,
    )


def porcelain() -> list[str]:
    out = run(["git", "status", "--porcelain"]).stdout.splitlines()
    return [ln for ln in ln_filter(out)]


def ln_filter(lines: list[str]) -> list[str]:
    return [ln for ln in lines if ln.strip()]


def is_secret_path(path: str) -> bool:
    low = path.lower()
    return any(h in low for h in SECRET_HINTS)


def paths_from_porcelain(lines: list[str]) -> list[str]:
    paths: list[str] = []
    for ln in lines:
        # XY PATH or XY ORIG -> PATH
        rest = ln[3:] if len(ln) > 3 else ln
        if " -> " in rest:
            rest = rest.split(" -> ", 1)[1]
        paths.append(rest.strip())
    return paths


def refresh_star_history() -> None:
    """Regenerate star-history.svg when GitHub stars changed (no empty rewrites)."""
    script = ROOT / "scripts" / "generate_star_history_svg.py"
    if not script.is_file():
        return
    proc = run(["python3", str(script)], check=False)
    out = (proc.stdout or "").strip()
    err = (proc.stderr or "").strip()
    if out:
        print(f"overnight_git_flush: star-history {out}")
    if proc.returncode != 0:
        print(
            f"overnight_git_flush: star-history refresh failed (exit {proc.returncode})",
            file=sys.stderr,
        )
        if err:
            print(err, file=sys.stderr)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    # Nightly: refresh the README star chart before the flush decision.
    if not args.dry_run:
        refresh_star_history()
    else:
        print("overnight_git_flush: dry-run — skip star-history refresh")

    lines = porcelain()
    if not lines:
        print("overnight_git_flush: clean — nothing to do")
        return 0

    paths = paths_from_porcelain(lines)
    secret = [p for p in paths if is_secret_path(p)]
    safe = [p for p in paths if not is_secret_path(p)]
    if secret:
        print("overnight_git_flush: skipping secret-looking paths:")
        for p in secret:
            print(f"  - {p}")
    if not safe:
        print("overnight_git_flush: only secret-looking dirty files — abort")
        return 1

    print("overnight_git_flush: pending:")
    for p in safe:
        print(f"  - {p}")

    if args.dry_run:
        print("overnight_git_flush: dry-run — no commit")
        return 0

    # Stage safe paths (respect renames via git add -u + add new)
    for p in safe:
        run(["git", "add", "-A", "--", p], check=False)

    # Re-check index has something
    staged = run(["git", "diff", "--cached", "--name-only"]).stdout.strip()
    if not staged:
        print("overnight_git_flush: nothing staged after filter — abort")
        return 1

    msg = (
        "Flush pending work (nightly 23:00 backstop).\n\n"
        "Auto-commit of finished-but-unpushed changes so the tree does not stay dirty overnight."
    )
    run(["git", "commit", "-m", msg])
    print("overnight_git_flush: committed")

    push = run(["git", "push", "origin", "HEAD"], check=False)
    if push.returncode != 0:
        print(push.stdout)
        print(push.stderr, file=sys.stderr)
        print("overnight_git_flush: commit ok, push failed", file=sys.stderr)
        return push.returncode

    print("overnight_git_flush: pushed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
