#!/usr/bin/env python3
"""Gate text before it is posted to a public GitHub repository.

Exit codes:
  0 — text is allowed (prints text to stdout; with --redact, prints redacted text)
  1 — blocked (findings on stderr); with --redact, still prints redacted text and exits 0
     only if --soft-redact is set; default is hard-fail (exit 1) when findings exist
  2 — usage / IO error

Design: fail closed. Prefer blocking a comment over leaking a secret.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass

# Patterns that must never appear in public GitHub text or committed task dumps.
RULES: list[tuple[str, re.Pattern[str]]] = [
    ("private_key", re.compile(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----")),
    ("aws_access_key", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("github_pat", re.compile(r"\b(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{20,}\b")),
    ("github_fine_grained", re.compile(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b")),
    ("openai_key", re.compile(r"\bsk-[A-Za-z0-9]{20,}\b")),
    ("slack_token", re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b")),
    ("generic_bearer", re.compile(r"\bBearer\s+[A-Za-z0-9\-._~+/]+=*\b", re.I)),
    ("jwt", re.compile(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b")),
    ("password_assign", re.compile(r"(?i)(password|passwd|pwd|secret|api[_-]?key|token)\s*[=:]\s*\S+")),
    ("connection_string", re.compile(r"(?i)(postgres|mysql|mongodb|redis)://[^\s]+")),
    ("home_users", re.compile(r"(?i)(/Users/[^\s'\"]+|/home/[^\s'\"]+|C:\\\\Users\\\\[^\s'\"]+)")),
    ("ssh_path", re.compile(r"(?i)\.ssh[/\\]")),
    ("aws_path", re.compile(r"(?i)\.aws[/\\]")),
    ("env_file", re.compile(r"(?i)(^|[^\w])\.env(\.|$|\b)|config\.env")),
    ("proton_drive", re.compile(r"(?i)ProtonDrive")),
    ("icloud_path", re.compile(r"(?i)Library/Mobile Documents|iCloudDrive")),
    ("email", re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")),
    # International-style numbers only (+country…) — avoid matching issue/line numbers.
    ("phone_intl", re.compile(r"(?<!\w)\+\d{1,3}[\s.-]?(?:\(?\d{1,4}\)?[\s.-]?){2,4}\d{2,4}(?!\w)")),
    ("long_base64", re.compile(r"\b(?:[A-Za-z0-9+/]{60,}={0,2})\b")),
]


@dataclass
class Finding:
    rule: str
    snippet: str


def find_issues(text: str) -> list[Finding]:
    found: list[Finding] = []
    for name, pat in RULES:
        for m in pat.finditer(text):
            snip = m.group(0)
            if len(snip) > 80:
                snip = snip[:77] + "..."
            found.append(Finding(name, snip))
    return found


def redact(text: str) -> str:
    out = text
    for name, pat in RULES:
        out = pat.sub(f"[REDACTED:{name}]", out)
    return out


def summarize_for_task(text: str, limit: int = 400) -> str:
    """Safe short summary for FEAT task files — never the raw issue body."""
    clean = redact(text or "")
    clean = re.sub(r"\s+", " ", clean).strip()
    if not clean:
        return "[No safe summary — issue body empty or fully redacted]"
    if len(clean) > limit:
        return clean[: limit - 3] + "..."
    return clean


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("path", nargs="?", help="File to check (default: stdin)")
    ap.add_argument("--redact", action="store_true", help="Print redacted text to stdout")
    ap.add_argument(
        "--soft-redact",
        action="store_true",
        help="With --redact, exit 0 even if findings existed (still redacts)",
    )
    ap.add_argument("--summarize", action="store_true", help="Print a short safe summary only")
    args = ap.parse_args()

    try:
        if args.path:
            text = open(args.path, encoding="utf-8", errors="replace").read()
        else:
            text = sys.stdin.read()
    except OSError as e:
        print(f"redact_public_text: IO error: {e}", file=sys.stderr)
        return 2

    findings = find_issues(text)
    if findings:
        print(f"redact_public_text: BLOCKED ({len(findings)} finding(s))", file=sys.stderr)
        for f in findings[:20]:
            print(f"  - {f.rule}: {f.snippet!r}", file=sys.stderr)
        if len(findings) > 20:
            print(f"  … {len(findings) - 20} more", file=sys.stderr)

    if args.summarize:
        print(summarize_for_task(text))
        return 0 if args.soft_redact or not findings else 1

    if args.redact:
        print(redact(text), end="" if text.endswith("\n") or text == "" else "\n")
        if findings and not args.soft_redact:
            return 1
        return 0

    if findings:
        return 1
    print(text, end="" if text.endswith("\n") or text == "" else "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
