# Morning surprise — 2026-08-30

Ralf, overnight Track B kept shipping product instant lanes (not digester-empty quiet).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.718** | `/details` · `/details hot` · `/load` — Details Load · RAM · Up (Hot: Load≥4 · RAM≥85%); ring kb hint drop All · Hot |
| **v0.1.719** | `/battery` · `/bat` · `/heat` · `/thermal` · `/lpm` — power-strip Bat · Heat · LPM chips (Bat≤20% · Heat Fair+ · LPM On hot) |
| **v0.1.720** | `/cpu` · `/gpu` · `/freq` · `/temp` — ring chips (CPU≥50% · GPU≥15% · Freq≥3.5 GHz · Temp≥70°C hot) |
| **v0.1.721** | `/ram` · `/ssd` · `/uptime` — power-strip RAM · SSD · Up chips (RAM/SSD≥85% hot · Up≥7d long) |

## Tried / context

- Digester **open** empty (fast instant or filtered turns).
- Design review **due=false** (grace); recommended surface still `feature-ai-chat`.
- `debug.log` scan: no ERROR/WARN/panic clusters in the last window.
- Fuel: standing backlog **p50** — remaining strip chips after Bat · Heat · LPM / ring chips.

## Why it helps

Asks like `/ram`, `/ssd`, `/uptime`, or `disk usage` skip Ollama and return one strip chip (hot/long marks match the power strip). Full `/strip` still lists every chip; `/disk` still means Disk Cleanup.

## Next fuel

- Design review when grace ends (stale feature screens; `feature-ai-chat` recommended).
- Digester open / product-owned `debug.log` errors when present.
- Other tool-heavy p50 patterns when digester surfaces them.
