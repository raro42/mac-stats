---
name: ui-weekly-review
description: Weekly Agent Ops Command Center UI polish — one visible layout/CSS fix, not harness/backend work.
---

# UI weekly review

Policy: `docs/041_ui_command_center.md`.

**Real UI surface:** menu-bar window loads `src-tauri/dist/themes/<theme>/cpu.html` (shared `src/agent-ops.js` / `src/agent-ops.css`). `dashboard.html` alone is not enough.

## Do

1. Open Agent Ops Command Center mentally: health cards + overview (Schedules, Live, Knowledge, Recent chats) + tabs.
2. Pick **one** visible improvement (empty state copy, card density, spacing, tab clarity, Load into Chat affordance, Perplexity icon presence across themes).
3. Implement in the correct files, run `./scripts/sync-dist.sh`, bump patch version if shipping, add a CHANGELOG user-facing bullet.
4. **Ship via Cursor Agent** (do not stop at the Discord git safety refusal). Emit:
   `CURSOR_AGENT: in ~/projects/mac-stats finish the UI weekly fix, sync-dist, install if needed, commit and push to origin`
5. Reply briefly with before/after intent (what is easier to see now).

## Do not

- Ship digester / tool-parser / Discord gateway changes under this skill
- Add a second overnight 20-minute UI loop competing with harness ticks
- Treat “I won't git commit from Discord” as done — escalate to `CURSOR_AGENT` instead
