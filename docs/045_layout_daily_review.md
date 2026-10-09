# Layout daily review

Once-a-day **theme layout** check so dark shells (especially Data Poster) do not stay broken for weeks.

Complements Wednesday `ui-weekly-review` (Agent Ops polish) and overnight design review (feature screenshots). This pass is **theme contrast / layout**, not digester fuel.

## Why

Shared `agent-ops.css` opaque washes mix against `#ffffff` for Apple. Dark themes (`data-poster`, `dark`) need remaps onto `#0e0e14`. Missing remaps show as white slabs, blank filter chips, and unreadable AI Chat empty states.

## Cadence

- **Daily ~10:30** local — Werner schedule `discord-layout-daily` + skill `layout-daily-review`
- Overnight harness may also run `python3 scripts/layout_daily_review.py` when the helper says `due=true` (theme not checked today)

## Surfaces (rotate)

| Priority | Theme | What to open |
|----------|-------|--------------|
| 1 | `data-poster` | CPU window → AI Chat + Monitors expanded |
| 2 | `dark` | Same |
| 3 | `neon` / `futuristic` | Same if dark ink |
| 4 | Active light theme | Spot-check only (Apple washes are intended) |

Capture window-only PNG under `docs/screens/theme-<name>.png` when polishing (warm-up ≥30s; see `docs/screens/README.md`).

## Procedure

1. `python3 scripts/layout_daily_review.py` → pick recommended theme / CSS hits.
2. Open CPU window on that theme. Expand AI Chat and Monitors.
3. Check: glance bars readable, filter chip labels visible, chat empty state contrast OK, no white slabs on dark ink.
4. One visible fix (usually `src/agent-ops.css` dark remaps, or theme `cpu.css`).
5. `./scripts/sync-dist.sh`, bump patch + CHANGELOG when shipping, install/restart if runtime UI.
6. Finish with `CURSOR_AGENT: in ~/projects/mac-stats commit and push`.

## Non-goals

- Digester / Discord gateway / tool parsers
- Full redesign of every theme in one day
- Competing with the overnight 20-minute harness every tick (one layout experiment per day is enough)
