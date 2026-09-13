# Morning surprise — 2026-09-13

Overnight Track B kept shipping Runs / metrics instant-lane NL so operator chat stays off the full Ollama path.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1044** | `/failed` · `/slow` · `/instant` · `/lite` · `/direct` — `view` / `see` / `show me` / `open` / `list the` (+ day window) → Runs lane reports (exact open only) |
| **v0.1.1043** | `/hot` · `/pinned` — same NL family → Top Processes Hot/Pinned lists |
| **v0.1.1042** | `/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/uptime` → strip chips |
| **v0.1.1041** | `/cpu` · `/gpu` · `/freq` · `/temp` → ring chips |
| **v0.1.1040** | `/rings` · `/strip` · `/details` → metrics gateways |

## Fuel notes

- Digester **open** stayed empty all night — standing backlog p50 NL expand (not quiet-default).
- Design review still in grace (`due=false`); CPU metrics screen ~6d, others much older (TCC / recapture when due).
- No product-owned `debug.log` ERROR/WARN clusters in the scan window.

## Try it

In AI Chat or Discord:

- `view failed` · `open slow 7` · `list the instant` · `see lite` · `show me the direct`
- Still works: `/failed` · `/slow` · `/instant 3` · `/lite` · `/direct`

## Next night

- Digester Slowest filters for the new Runs-lane phrases if they show up as noise.
- Design review when due (prefer stale feature screens).
- Sibling ports only when they clearly map (Hermes insights extras / session UX).
