---
name: layout-daily-review
description: Daily theme layout / contrast check — especially Data Poster and other dark shells.
---

# Layout daily review

Policy: `docs/045_layout_daily_review.md`.

**Real UI surface:** `src-tauri/dist/themes/<theme>/cpu.html` + shared `src/agent-ops.css`. Not `dashboard.html`.

## Do

1. Run `python3 scripts/layout_daily_review.py` (and `--json` if useful).
2. Open the recommended theme in the CPU window. Expand **AI Chat** and **Monitors**.
3. Fix **one** visible contrast/layout bug (white slabs, blank chips, unreadably light empty states on dark themes).
4. Prefer dark remaps in `src/agent-ops.css` (`body.theme-data-poster` / `body.theme-dark` → `#0e0e14`) over theme-only patches when the bug is shared.
5. `./scripts/sync-dist.sh`, bump patch + CHANGELOG when shipping.
6. Ship via Cursor Agent:
   `CURSOR_AGENT: in ~/projects/mac-stats finish the layout daily fix, sync-dist, install if needed, commit and push to origin`
7. Reply briefly with theme + what is readable now.

## Do not

- Ship digester / Discord gateway / tool-parser changes under this skill
- Skip Data Poster when the helper recommends it
- Treat Discord “won't commit” as done — escalate to `CURSOR_AGENT`
