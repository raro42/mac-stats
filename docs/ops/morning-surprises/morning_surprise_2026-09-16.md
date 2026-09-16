# Morning surprise — 2026-09-16

Overnight Track B (20:00–06:00 local). Digester open stayed empty most ticks; design review not due. Standing backlog drove filter-miss calm, a take-note latency fix, then Details / Top Processes / Disk Cleanup / Debug Log / Perplexity glance calm, Temp ring Nominal calm, CPU · GPU · FREQ ring calm, battery healthy calm, then LPM Off calm.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1098** | LPM Off calm — soft green on the LPM chip when Low Power Mode is Off (battery/Details/ring parity); On keeps enabled green |
| **v0.1.1097** | Battery healthy calm — soft green on the battery chip when above 20% or charging (Details/Monitors/ring parity); low stays amber |
| **v0.1.1096** | CPU · GPU · FREQ ring calm — soft green on rings + sparklines when below hot (Temp Nominal / Details parity) |
| **v0.1.1095** | Temp ring Nominal calm — soft green on Temperature ring + TEMP sparkline when Heat is Nominal below hot (Details/Monitors parity) |
| **v0.1.1094** | Perplexity collapsed glance calm — soft green when Ready · search (Monitors/Disk/Debug Log parity) |
| **v0.1.1093** | Debug Log collapsed glance calm — soft green when Quiet · clean (Monitors/Disk/Details parity) |
| **v0.1.1092** | Disk Cleanup collapsed glance calm — soft green when clean (Monitors/Details parity) |
| **v0.1.1091** | Top Processes glance calm — Top CPU · GPU · RAM soft green when below hot (Details/Monitors parity) |
| **v0.1.1090** | Details collapsed glance calm — soft green when Load/RAM fine (Monitors/Disk parity) |
| **v0.1.1089** | Take note instant — `Take note:` / `note to self:` / `remember this:` → curated MEMORY_APPEND (no Brave) |
| **v0.1.1088** | Debug Log filter-miss calm — warm “Nothing here yet” + soft-green wash (Error/Warn empty) |
| **v0.1.1087** | Perplexity filter-miss calm — warm title + accent wash (Top/Snippet) |
| **v0.1.1086** | Disk Cleanup filter-miss calm — warm title + solid wash |
| **v0.1.1085** | Monitors filter-miss calm — warm title + solid wash |
| **v0.1.1084** | Top Processes filter-miss calm — warm title + solid wash |
| **v0.1.1083** | Having-fun idle Ollama soft timeout (120s wall · WARN · +15m backoff) |
| **v0.1.1082** | Agent Ops Overview empty calm + `feature-agent-ops.png` recapture |

## Why it matters

Empty filter panes feel calm instead of cold. Take-note asks no longer burn ~23s on Brave when you only wanted a memory bullet. Collapsed Details, Top Processes, Disk Cleanup, Debug Log, and Perplexity glances share the same calm green language when things are fine — and the CPU · GPU · FREQ · Temp rings join that calm when below hot / Nominal. The battery chip and LPM Off chip now match that language when charge is healthy and Low Power Mode is off. Hot / Fair / reclaim / errors / needs-key / low battery / LPM On still use stronger or amber cues.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (AI Chat / Processes / Monitors still old)
- Sibling Hermes/OpenClaw ports with clear user fitness
- Next calm: power chip (low draw), or Heat Nominal if strip chips return
