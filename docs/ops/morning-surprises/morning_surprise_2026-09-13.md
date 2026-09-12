# Morning surprise — 2026-09-13

Overnight Track B kept shipping operator instant NL. Digester open stayed empty; design review stayed in grace. Fuel came from standing backlog p50.

## Shipped tonight (local window into 13 Sep)

| Version | What |
|---------|------|
| **v0.1.1040** | `/rings` · `/strip` · `/details` view/see/show me/open/list-the NL |
| **v0.1.1041** | `/cpu` · `/gpu` · `/freq` · `/temp` ring-chip NL |
| **v0.1.1042** | `/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/uptime` strip-chip NL |
| **v0.1.1043** | `/hot` · `/pinned` Hot/Pinned view/see/show me/open/list-the NL |

## Latest keep (~01:40)

**v0.1.1043** — Instant lane: `view hot` / `see hot` / `show me the hot` / `open hot` / `list the hot` (and the same for pinned / pinned processes) join `/hot` · `/pinned` Top Processes Hot or Pinned lists. Exact open only — not rings/strip/details Hot, not pinned path/size/age.

## Also tried / context

- Digester open empty; no ERROR/WARN/panic clusters in the 180m debug.log window.
- Design review: due=false (grace); recommended surface still CPU metrics (~5.9d).
- Sibling harness: OpenClaw/Hermes commits noted; no port this tick (NL p50 first).

## How to poke it

In AI Chat or Discord: `view hot`, `open pinned`, `show me the pinned`, `list the hot`.
