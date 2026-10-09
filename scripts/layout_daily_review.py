#!/usr/bin/env python3
"""Daily theme layout helper — which dark theme needs a contrast/layout look?

Usage:
  python3 scripts/layout_daily_review.py
  python3 scripts/layout_daily_review.py --json
  python3 scripts/layout_daily_review.py --mark-checked data-poster

Exit 0 always (informational). Prints due=true|false and a recommended theme.
"""

from __future__ import annotations

import argparse
import json
import re
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
THEMES_DIR = ROOT / "src-tauri" / "dist" / "themes"
AGENT_OPS_CSS = ROOT / "src" / "agent-ops.css"
SCREENS = ROOT / "docs" / "screens"
STATE_DIR = Path.home() / ".mac-stats" / "improvements"
STATE_FILE = STATE_DIR / "layout_daily_review_state.json"

# Dark shells first — Apple #ffffff opaque washes hurt these hardest.
THEMES = (
    ("data-poster", "CPU window → AI Chat + Monitors (Data Poster)"),
    ("dark", "CPU window → AI Chat + Monitors (Dark)"),
    ("neon", "CPU window → AI Chat + Monitors (Neon)"),
    ("futuristic", "CPU window → AI Chat + Monitors (Futuristic)"),
    ("architect", "CPU window → AI Chat + Monitors (Architect)"),
)

# Shared Apple washes that dark themes must remap (substring markers).
WASH_SELECTORS = (
    ".chat-model-glance",
    ".chat-turn-glance",
    ".chat-empty",
    ".chat-filter-chip",
    ".monitors-filter-chip",
)


def _load_state() -> dict:
    if not STATE_FILE.is_file():
        return {}
    try:
        return json.loads(STATE_FILE.read_text())
    except Exception:
        return {}


def _save_state(state: dict) -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    STATE_FILE.write_text(json.dumps(state, indent=2) + "\n")


def _today() -> str:
    return time.strftime("%Y-%m-%d")


def _agent_ops_remap_hits(theme: str) -> list[str]:
    """Selectors still missing a body.theme-<name> rule (dark remap)."""
    if not AGENT_OPS_CSS.is_file():
        return []
    text = AGENT_OPS_CSS.read_text(errors="replace")
    missing = []
    for sel in WASH_SELECTORS:
        # Require the selector in a selector-list that also names body.theme-<theme>.
        paired = re.search(
            rf"body\.theme-{re.escape(theme)}[^{{]*{re.escape(sel)}[^{{]*\{{",
            text,
            re.MULTILINE | re.DOTALL,
        )
        if not paired:
            missing.append(sel)
    return missing


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    ap.add_argument(
        "--mark-checked",
        metavar="THEME",
        help="Record that this theme was reviewed today",
    )
    args = ap.parse_args()
    state = _load_state()
    today = _today()

    if args.mark_checked:
        theme = args.mark_checked.strip().lower()
        state["last_checked_day"] = today
        state["last_theme"] = theme
        checked = state.setdefault("checked", {})
        checked[theme] = today
        _save_state(state)
        print(f"marked layout check: {theme} ({today})")
        return

    rows = []
    for name, how in THEMES:
        theme_path = THEMES_DIR / name / "cpu.html"
        screen = SCREENS / f"theme-{name}.png"
        missing = _agent_ops_remap_hits(name) if name in ("data-poster", "dark") else []
        age_days = None
        if screen.is_file():
            age_days = round((time.time() - screen.stat().st_mtime) / 86400, 2)
        checked_day = (state.get("checked") or {}).get(name)
        checked_today = checked_day == today
        stale_screen = (age_days is None) or (age_days >= 7.0)
        needs_css = bool(missing)
        due_theme = (not checked_today) and (needs_css or stale_screen or not theme_path.is_file())
        rows.append(
            {
                "theme": name,
                "how": how,
                "exists": theme_path.is_file(),
                "screen_age_days": age_days,
                "checked_today": checked_today,
                "missing_remaps": missing,
                "due": due_theme,
            }
        )

    due_rows = [r for r in rows if r["due"]]
    # Prefer data-poster when anything is due.
    pick = None
    for prefer in ("data-poster", "dark"):
        for r in due_rows:
            if r["theme"] == prefer:
                pick = r
                break
        if pick:
            break
    if pick is None:
        pick = due_rows[0] if due_rows else rows[0]

    already_today = state.get("last_checked_day") == today
    due = bool(due_rows) and not already_today

    payload = {
        "due": due,
        "today": today,
        "recommended": pick,
        "themes": rows,
        "policy": "docs/045_layout_daily_review.md",
        "already_checked_today": already_today,
    }

    if args.json:
        print(json.dumps(payload, indent=2))
        return

    print(f"due={'true' if due else 'false'}  today={today}")
    if pick:
        miss = ",".join(pick.get("missing_remaps") or []) or "none"
        print(
            f"recommended: {pick['theme']} — {pick['how']} "
            f"(missing_remaps={miss})"
        )
    print("policy: docs/045_layout_daily_review.md")
    for r in rows:
        mark = "DUE" if r["due"] else ("ok" if r["checked_today"] else "idle")
        age = "missing" if r["screen_age_days"] is None else f"{r['screen_age_days']}d"
        miss = ",".join(r["missing_remaps"]) if r["missing_remaps"] else "-"
        print(f"  [{mark}] {r['theme']}  screen={age}  remaps_missing={miss}")


if __name__ == "__main__":
    main()
