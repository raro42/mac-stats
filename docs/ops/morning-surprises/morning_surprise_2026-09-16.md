# Morning surprise — 2026-09-16

Overnight Track B (20:00–06:00 local). Digester open stayed empty most ticks. Standing backlog drove filter-miss calm, a take-note latency fix, Details / Top Processes / Disk Cleanup / Debug Log / Perplexity glance calm, Temp · CPU · GPU · FREQ ring calm, battery · LPM · Power calm, Disk Cleanup meta-card calm, AI Chat last-answer glance calm, Last run panel calm, then AI Chat turn glance calm at the start of the next overnight window.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1105** | AI Chat turn glance calm — soft green when turns exist and nothing is sending (last-answer / Monitors all-up parity); sending keeps accent |
| **v0.1.1104** | Disk Cleanup Last run calm — soft green when a last run exists without skipped files (Reclaimable is-clean / Details is-ok parity); skips stay amber; not-yet-run stays neutral |
| **v0.1.1103** | AI Chat last-answer glance calm — soft green when a successful last answer is ready to copy (Perplexity Ready / Debug Quiet / Monitors all-up parity); failed turns stay red. Design review; `feature-ai-chat.png` recapture deferred (Screen Recording TCC) |
| **v0.1.1102** | Disk Cleanup Next run · Runs when calm — soft green when next run is ahead and periodic is on; due amber; periodic off cyan |
| **v0.1.1101** | Disk Cleanup Enabled scopes calm — soft green when every scope is on (Reclaimable is-clean / Details parity); scopes off stay amber |
| **v0.1.1100** | Disk Cleanup Reclaimable calm — soft green on Reclaimable now when nothing pending (collapsed is-clean / Details parity); reclaim stays amber |
| **v0.1.1099** | Power low-draw calm — soft green on the Power chip when combined CPU+GPU draw is under 20 W (battery/LPM/Details parity); ≥20 W amber |
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

Empty filter panes feel calm instead of cold. Take-note asks no longer burn ~23s on Brave when you only wanted a memory bullet. Collapsed Details, Top Processes, Disk Cleanup, Debug Log, and Perplexity glances share the same calm green language when things are fine — and the CPU · GPU · FREQ · Temp rings join that calm when below hot / Nominal. The battery chip, LPM Off chip, and Power chip now match that language when charge is healthy, Low Power Mode is off, and draw stays under 20 W. When Disk Cleanup is open, Reclaimable / Enabled scopes / Next run / Runs when / Last run share that calm when nothing is pending, every scope is on, the next run is still ahead, periodic cleanup is on, and the last run finished without skips. AI Chat’s last-answer glance and turn glance now match that calm when a good reply is ready to copy and when turns exist with nothing in flight. Hot / Fair / reclaim / due / errors / needs-key / low battery / LPM On / elevated watts / periodic off / skipped files / sending still use stronger or amber/cyan/accent cues.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Processes / Monitors; recapture AI Chat when Screen Recording TCC allows)
- Sibling Hermes/OpenClaw ports with clear user fitness
- Next calm: chat-model online wash parity; Agent Ops Discord Ready stronger parity if needed; time-remaining strip calm when the estimate is healthy
