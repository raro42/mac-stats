# Morning surprise — 2026-09-12

Overnight Track B kept shipping operator NL so Discord/chat hits Agent Ops lists and Ready chips without Ollama.

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
| **v0.1.1028** | `/browser` · `/cdp` NL expand (exact `open browser` / `open cdp` only; path/size/age / credentials / downloads / cookies safe) |
| **v0.1.1029** | `/judge` · `/ai` · `/ai-agent` NL expand (exact `open judge` / `open ai` / `open ai agent` only; not run/score/enable judge, OpenAI, ask/chat, `/agents`) |

## Fuel notes

- Digester open stayed empty; standing backlog p50 drove each tick.
- Design review still in grace (CPU metrics ~5.08d recommended when due).
- No product ERROR/WARN clusters in the last debug.log window.

## Try in Discord / AI Chat

- `view judge` · `show me the judge` · `open judge` · `list the judge`
- `view ai` · `show me the ai` · `open ai` · `open ai agent` · `list the ai`
- Still works: `/judge` · `/ai` · `/ai-agent` · `judge status` · `is ai ready`
- Still separate: `run the judge` · `judge this reply` · `enable judge` · `openai` · `ask ai` · `chat with ai` · `/agents` · `list agents`

Updated: 2026-09-12 ~04:55
