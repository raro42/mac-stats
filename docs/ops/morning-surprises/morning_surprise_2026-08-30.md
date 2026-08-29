# Morning surprise — 2026-08-30

Ralf, overnight Track B kept shipping product instant lanes (not digester-empty quiet).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.718** | `/details` · `/details hot` · `/load` — Details Load · RAM · Up (Hot: Load≥4 · RAM≥85%); ring kb hint drop All · Hot |
| **v0.1.719** | `/battery` · `/bat` · `/heat` · `/thermal` · `/lpm` — power-strip Bat · Heat · LPM chips (Bat≤20% · Heat Fair+ · LPM On hot) |
| **v0.1.720** | `/cpu` · `/gpu` · `/freq` · `/temp` — ring chips (CPU≥50% · GPU≥15% · Freq≥3.5 GHz · Temp≥70°C hot) |

## Tried / context

- Digester **open** empty (fast instant or filtered turns).
- Design review **due=false** (grace); recommended surface still `feature-ai-chat`.
- `debug.log` scan: no ERROR/WARN/panic clusters in the last window.
- Fuel: standing backlog **p50** — focused ring chips without Discord/AI Chat instant (after Bat · Heat · LPM).

## Why it helps

Asks like `/cpu`, `/gpu`, `/temp`, or `what's the cpu` skip Ollama and return one ring (hot marks match the gauges). Full `/rings` still lists every ring; `/hot` still means processes.

## Next fuel

- Design review when grace ends (stale feature screens; `feature-ai-chat` recommended).
- Remaining strip chips (`/ram` · `/ssd` · `/uptime`) or digester open / product-owned `debug.log` errors when present.
- Other tool-heavy p50 patterns when digester surfaces them.
