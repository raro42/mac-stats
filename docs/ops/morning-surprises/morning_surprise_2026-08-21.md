# Morning surprise — 2026-08-21

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight (20:00–…)

Continues from late 2026-08-20 window (local night).

| Version | What |
|---------|------|
| **v0.1.549–557** | Prior keeps (AI Chat chips → Debug Log keep-header); see `morning_surprise_2026-08-20.md` |
| **v0.1.558** | Top Processes **collapsed keep-header** (header + Top CPU/GPU/RAM glances; Waiting · processes) |
| **v0.1.559** | Details **collapsed keep-header** (Load · RAM · Up glance; Waiting · details; amber wash Load≥4 / RAM≥85%) |
| **v0.1.560** | CPU metrics **Heat / thermal on the battery/power strip** (°C-band bands; click → temp ring) |
| **v0.1.561** | Heat prefers Apple **`NSProcessInfo.thermalState`** (OS Nominal/Fair/Serious/Critical; °C-band fallback; AI Thermal pressure) |

## Latest tick (~01:28)

- Digester open empty; design-review grace (feature-ai-chat ~6.5d); debug.log quiet (soul dump only).
- Fuel: standing backlog next after thermal strip — **NSProcessInfo.thermalState** wire-up.
- **Keep** `CpuDetails.thermal_state` via `ffi::objc::read_process_thermal_state`; Heat strip + temp ring subtext prefer OS pressure.
- Discord Ready after install/kickstart.

## Next fuel

- Digester Discord traffic if any
- Deferred screenshots when TCC allows
- Prefer non-thermalState fuel: p50 latency / Low Power Mode glance / sibling ports
