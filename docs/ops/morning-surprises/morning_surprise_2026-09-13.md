# Morning surprise — 2026-09-13

Overnight Track B kept shipping operator instant NL. Digester open stayed empty; design review stayed in grace. Fuel came from standing backlog p50.

## Shipped tonight (local window into 13 Sep)

| Version | What |
| --- | --- |
| **v0.1.1041** | `/cpu` · `/gpu` · `/freq` · `/temp` — `view` / `see` / `show me the …` / `open` / `list the …` → one-chip ring replies. Exact open only (not rings / details / cpu window / path·size·age). |
| **v0.1.1040** | `/rings` · `/strip` · `/details` — `view` / `see` / `show me the …` / `open` / `list the …` (+ Hot / power / load) → CPU rings, power strip, or Details Load · RAM · Up. Exact open only. |
| **v0.1.1039** | `/insights` · `/help` · `/ops` NL expand. |
| **v0.1.1038** | `/status` · `/health` · `/version` NL expand. |
| **v0.1.1037** | `/discord` · `/ollama` · `/perplexity key` · `/cursor` NL expand. |

## This tick (~00:40)

1. Digester open empty — did not quiet-default.
2. Design review `due=false` (CPU metrics recommended ~5.9d, still grace).
3. Experiment: expand `/cpu` · `/gpu` · `/freq` · `/temp` ring-chip NL to match gateway slash parity from v0.1.1040.
4. Ratchet **keep** @ `78e428fc`; push + install/kickstart.

## Next fuel

- Digester open / debug.log product errors when present.
- Design review when due (recapture stale feature screens when TCC allows).
- Sibling ports (Hermes insights extras / session UX).
- Power-strip chip NL (`/battery`·`/heat`·`/lpm`·`/ram`·`/ssd`·`/uptime`) view/see/open parity if still thin.
