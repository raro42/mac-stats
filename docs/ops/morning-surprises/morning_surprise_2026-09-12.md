# Morning surprise — 2026-09-12

Overnight Track B kept shipping operator NL so Discord/chat hits Ready chips without Ollama.

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
| **v0.1.1030** | `/compact` · `/menu-bar` · `/cpu-window` NL expand (exact `open compact` / `open menu-bar` / `open cpu-window` only; not compaction / enable / run compaction) |
| **v0.1.1031** | `/downloads` · `/organizer` NL expand (exact `open downloads` / `open organizer` only; not run-now / enable / `/disk` / BROWSER_DOWNLOAD; path/size/age / rules/state safe) |
| **v0.1.1032** | `/ori` · `/mnemos` NL expand (exact `open ori` / `open mnemos` only; not MCP `ori_*` / MEMORY_APPEND / scrub / enable / vault path·size·age) |

## Evening window (20:00+)

| Version | What |
|---------|------|
| **v0.1.1032** | First keep of the 20:00–06:00 window — Ori Mnemos view/see/show me/open/list-the NL |

## Fuel notes

- Digester open stayed empty; standing backlog p50 drove each tick.
- Design review still in grace (CPU metrics ~5.73d recommended when due).
- No product ERROR/WARN clusters in the last debug.log window.

## Try in Discord / AI Chat

- `view ori` · `show me the ori` · `open ori` · `list the ori`
- `view mnemos` · `open mnemos` · `show me the mnemos` · `list the mnemos`
- Still works: `/ori` · `/mnemos` · `ori status` · `is ori ready`
- Still separate: `ori vault path` · `ori vault size` · `enable ori` · MCP `ori_*` · MEMORY_APPEND · scrub

Updated: 2026-09-12 ~20:45
