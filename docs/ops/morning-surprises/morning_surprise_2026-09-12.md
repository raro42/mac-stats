# Morning surprise — 2026-09-12

Overnight Track B kept shipping operator NL so Discord/chat hits Agent Ops lists without Ollama.

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.1017** | `/logs` view/see/show me/open/list-the + digester Slowest filters |
| **v0.1.1018** | `/processes` NL expand |
| **v0.1.1019** | `/monitors` NL expand |
| **v0.1.1020** | `/disk` NL expand |
| **v0.1.1021** | `/schedules` NL expand |
| **v0.1.1022** | `/sessions` NL expand (exact `open sessions` only) |
| **v0.1.1023** | `/knowledge` NL expand |
| **v0.1.1024** | `/agents` NL expand (exact `open agents` only; path/size/age safe) |
| **v0.1.1025** | `/skills` NL expand (exact `open skills` only; path/size/age safe) |

## Fuel notes

- Digester open stayed empty; standing backlog p50 drove each tick.
- Design review still in grace (CPU metrics ~5d recommended when due).
- No product ERROR/WARN clusters in the last debug.log window.

## Try in Discord / AI Chat

- `view skills` · `show me the skills` · `open skills` · `list the skills`
- Still works: `/skills` · `list skills` · `skills catalog`
- Still separate: `skills path` · `skills size` · `skills age` · `SKILL: …`

Updated: 2026-09-12 ~03:05
