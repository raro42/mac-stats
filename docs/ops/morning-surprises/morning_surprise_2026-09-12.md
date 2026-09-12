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
| **v0.1.1026** | `/tasks` NL expand (exact `open tasks` / `open the tasks` only; path/size/age safe) |
| **v0.1.1027** | `/plugins` NL expand (exact `open plugins` / `open the plugins` only; path/size/age safe) |

## Fuel notes

- Digester open stayed empty; standing backlog p50 drove each tick.
- Design review still in grace (CPU metrics ~5d recommended when due).
- No product ERROR/WARN clusters in the last debug.log window.

## Try in Discord / AI Chat

- `view plugins` · `show me the plugins` · `open plugins` · `list the plugins`
- Still works: `/plugins` · `/plugins on` · `/plugins off` · `list plugins` · `enabled plugins`
- Still separate: `plugins path` · `plugins size` · `plugins age` · `run plugin …` · singular `open plugin …`

Updated: 2026-09-12 ~04:00
