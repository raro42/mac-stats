# Morning surprise — 2026-11-25

## Shipped overnight
- **v0.1.3060** — Accessory power instant status from `pmset -g accps`
  - Slash + NL: `/accps` · `/pmaccps` · `/pm-accps` · `/pmset-accps` · `/accessory-power` · `/accessory-batteries` · `/peripheral-batteries` · `/bt-batteries` · `/mac-accps` (+ view/open/list)
  - Shows drawing source + InternalBattery / Magic Keyboard / Magic Mouse rows (present devices only)
  - Does not steal `/pmps` · `/battery` · `/ups` · `/adapter` · `/pmadapters` · `/pmeverything` · `/pmlive`
- Earlier same night (already on main before this tick): Power everything **v0.1.3059**, Power live **v0.1.3058**, Power adapters **v0.1.3057**, …

## Context
- Digester open empty; design-review due=false (grace / Mac TCC); layout-daily data-poster marked today (Dark next calendar day).
- Fuel choice: standing backlog after `pmeverything` — `history` can block; `accps` is fast and user-facing (BT peripheral %).

## Next
- Dark layout-daily next calendar day
- Optional: `pmset -g battlimit` / `rawbatt` if useful; avoid blocking `history`/`thermlog`
