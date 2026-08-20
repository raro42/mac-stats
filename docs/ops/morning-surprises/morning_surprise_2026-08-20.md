# Morning surprise — 2026-08-20

Overnight Track B kept shipping UI polish. Digester open stayed empty (instant-heavy window).

## Shipped tonight (highlights)

| Version | What |
|---------|------|
| **v0.1.552** | External / Monitors **collapsed glance** — up/down (or empty) summary under the header when collapsed; click expands to first DOWN / slowest / Add; light 30s summary poll while collapsed (Debug Log / Perplexity parity) |
| v0.1.551 | Perplexity **last-search glance** |
| v0.1.550 | CPU metrics **uptime** on battery/power strip |
| v0.1.549 | AI Chat **All · You · Assistant** filter chips |
| v0.1.548 | Disk Cleanup **All · Reclaim · Clean** filter chips |
| v0.1.547 | Top Processes **Top RAM glance** |

## This tick (~21:25)

- Digester: open empty; design-review due=false (grace); fuel = standing design review / loop_backlog next (Monitors collapsed glance).
- **Keep** @ `29045b0` — verify green; install + kickstart; Discord process up.
- Screenshot: deferred (TCC / no capture); `overnight_design_review.py --mark-polished feature-monitors`.

## Next fuel

- Digester Discord traffic if open candidates appear.
- Prefer non-monitors-collapsed-glance fuel (e.g. p50 latency / Disk Cleanup collapsed glance / Discord Ready glance).
