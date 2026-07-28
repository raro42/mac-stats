# Overnight design review

Periodic **screenshot + polish** pass so the product looks and works better every few nights — not only on Wednesday’s `ui-weekly-review`.

## Why

Overnight digester often goes empty after instant-lane wins. Design review is **standing fuel**: capture what users see, fix one visible gap, ship. Quiet nights with stale screenshots are a marketing and UX loss.

## Cadence

- **Due** when `python3 scripts/overnight_design_review.py` exits 0 with `due=true` (default: any tracked feature screen older than **3 days**, or never captured).
- At most **one** design-review experiment per overnight window (can share the night with other ticks for prep).
- Wednesday ~11:00 `ui-weekly-review` remains the Agent Ops–focused weekly skill; overnight may polish any CPU-window surface (metrics, Agent Ops, AI chat, processes).

## Surfaces (rotate)

| Priority | Screen file | What to open |
|----------|-------------|--------------|
| 1 | `screens/feature-agent-ops.png` | CPU window → Agent Ops expanded |
| 2 | `screens/feature-ai-chat.png` | CPU window → AI / Ollama chat |
| 3 | `screens/feature-processes.png` | CPU window → process list |
| 4 | `screens/feature-cpu-metrics.png` | CPU window → metrics rings |

Real UI ships from `src-tauri/dist/themes/<theme>/cpu.html` + `src/agent-ops.{js,css}` / theme CSS — not `dashboard.html` alone. See `docs/041_ui_command_center.md` and `docs/042_dashboard_orphan.md`.

## Procedure (one experiment)

1. `python3 scripts/overnight_design_review.py` → pick the recommended surface.
2. Ensure mac-stats is running; open the CPU window to that view.
3. Capture **window-only** PNG into `screens/<feature-….png>` (Swift CGWindowList / `screencapture -l`, not full desktop).
4. Pick **one** visible improvement: spacing, empty state, hierarchy, contrast, copy, affordance — make the view nicer or clearer.
5. Edit source under `src/` (and theme files as needed), `./scripts/sync-dist.sh`.
6. `python3 scripts/autoresearch_ratchet.py verify` → commit → keep → bump patch + CHANGELOG → install if runtime UI.
7. Note before/after in morning surprise.

## Non-goals

- Digester heuristics, Discord gateway, tool parsers (unless blocking the polish)
- Multi-surface redesign in one tick
- Dark/purple glow fashion pass that fights the existing glass system
