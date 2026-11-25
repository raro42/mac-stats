# Morning surprise — 2026-11-25

Overnight Track B (mac-stats product ratchet). Digester open empty; design-review grace; layout-daily data-poster marked (Dark next). Avoided another soft-state keep-pNN.

## Shipped
- **v0.1.3072** — Power history instant status from `pmset -g history`
  - Slash + NL: `/pmhistory` · `/pm-history` · `/pmset-history` · `/power-history` · `/sleep-wake-history` · `/sw-history` · `/mac-pmhistory` (+ view/open/list)
  - Sleep/wake UUID timeline · `none yet` when disabled; does not steal `/pmlog` · `/pmuuid` · `/pmstats` · metrics history.json path/size/age
- **v0.1.3071** — Theme HTML Tab tip parity (all shells; finish first-paint warmer tips after 3070)
- **v0.1.3070** — Theme HTML kb-hint seeds: drop leftover “Home/End move”
- **v0.1.3069** — Perplexity Search · setup + Debug Log toolbar kb-hints
- **v0.1.3068** — Settings leftover kb-hints
- **v0.1.3067** — CPU metrics rings-filter + filter-chip fallback kb-hints
- **v0.1.3066** — Power user-clients from `pmset -g userclients`
- **v0.1.3065** — Agent Ops leftover kb-hints
- **v0.1.3064** — Battery charge limit from `pmset -g battlimit`
- Earlier tonight: System state · Raw battery · Sleep/Wake UUID · Accessory power · Power everything / live / adapters / custom / …

## Fuel choice
Digester empty; layout-daily Dark waits for next calendar day. Last unwired non-blocking `pmset -g` getter was `history` (`disk` mirrors `custom`; `uuidlog` blocks). Shipped `/pmhistory` (v0.1.3072).

## Next
- Dark layout-daily (next calendar quota)
- Soft-state leftover closed for SoftDisabled…userclients/pmhistory
- Remaining useful non-blocking `pmset -g` adapters only if clearly distinct
- not another keep-pNN clone
