# Morning surprise — 2026-09-16

Overnight Track B (20:00–06:00 local). Digester open stayed empty most ticks. Standing backlog drove filter-miss calm, a take-note latency fix, Details / Top Processes / Disk Cleanup / Debug Log / Perplexity glance calm, Temp · CPU · GPU · FREQ ring calm, battery · LPM · Power · time-remaining calm, Disk Cleanup meta-card calm, AI Chat last-answer / turn / model-online / Ready glance calm, Discord idle-thought 503 retry from tonight’s debug.log, then good-news filter-miss soft parity.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1110** | Good-news filter-miss soft parity — Fail / Hot / Errors / Warn / Down / Slow / Reclaim / Big / Off-empty panes soft green at 7% (Ready calm / Monitors all-up); louder 10%/38% tint removed |
| **v0.1.1109** | AI Chat Ready calm soft parity — empty Ready pane + Chat · Ready attention glance soft green at 7% (model-online / turn / Monitors all-up parity); stronger tint removed; offline / no-model / errors stay amber or red |
| **v0.1.1108** | AI Chat model online calm — connected model glance soft green at 7% (collapsed AI Chat / turn glance / Monitors all-up parity); offline / no-model stays amber |
| **v0.1.1107** | Discord idle-thought 503 safe retry — treat `Service Unavailable` as a one-shot safe outbound retry (~1.5s backoff) so brief Discord outages do not drop Having-fun idle sends on the first failure |
| **v0.1.1106** | Time-remaining strip calm — soft green when the battery estimate is ≥2h (battery / LPM / Power parity); under 1h amber; mid-range neutral |
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

Empty filter panes feel calm instead of cold. Take-note asks no longer burn ~23s on Brave when you only wanted a memory bullet. Collapsed Details, Top Processes, Disk Cleanup, Debug Log, and Perplexity glances share the same calm green language when things are fine — and the CPU · GPU · FREQ · Temp rings join that calm when below hot / Nominal. The battery chip, LPM Off chip, Power chip, and time-remaining chip now match that language when charge is healthy, Low Power Mode is off, draw stays under 20 W, and the remaining estimate is ≥2h. When Disk Cleanup is open, Reclaimable / Enabled scopes / Next run / Runs when / Last run share that calm when nothing is pending, every scope is on, the next run is still ahead, periodic cleanup is on, and the last run finished without skips. AI Chat’s last-answer glance, turn glance, connected model glance, empty Ready pane, and Chat · Ready attention glance now match that calm when a good reply is ready, turns exist with nothing in flight, Ollama is online, or the chat is empty and ready for a starter. Good-news empties (Fail / Hot / Errors / Warn / Down / Slow / Reclaim / Big / Off) now use the same soft 7% green as Ready — no louder tint when nothing bad is on the list. Brief Discord 503 outages no longer drop Having-fun idle thoughts on the first failure — one safe retry with a short backoff. Hot / Fair / reclaim / due / errors / needs-key / low battery / short remaining / LPM On / elevated watts / periodic off / skipped files / sending / offline still use stronger or amber/cyan/accent cues.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Processes / Monitors; recapture AI Chat when Screen Recording TCC allows)
- Sibling Hermes/OpenClaw ports with clear user fitness
- Next calm: accent filter-miss soft parity (10%→7%) if still loud vs Ready; SSD strip calm if reintroduced
