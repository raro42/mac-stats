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
| **v0.1.562** | CPU metrics **Low Power Mode (LPM)** on the power strip (On/Off; click → Battery settings; green wash when On) |
| **v0.1.563** | Menu bar **green LPM** cue when Low Power Mode is on (Mon ✕ cue style; hidden when off) |
| **v0.1.564** | Menu bar **Heat** cue when thermal is **Serious** (amber) or **Critical** (red) |
| **v0.1.565** | Menu bar **Ollama ✕** cue **red semibold** when the Ollama HTTP circuit is open |
| **v0.1.566** | Menu bar **Heat** also for **Fair** (soft yellow); power-strip Heat soft yellow wash when Fair |
| **v0.1.567** | Menu bar **SSD** amber cue when disk used **≥ 85%** (power-strip SSD hot wash parity) |
| **v0.1.568** | Menu bar **RAM** amber cue when memory used **≥ 85%** + power-strip RAM hot wash (Details glance parity) |
| **v0.1.569** | Menu bar **CPU** amber cue when usage **≥ 50%** (power-strip CPU is-hot parity) |
| **v0.1.570** | Menu bar **GPU** amber cue when usage **≥ 15%** (power-strip GPU is-hot parity) |

## Latest tick (~05:19)

- Digester open empty; design-review grace (feature-ai-chat ~6.69d); debug.log quiet (no ERROR/WARN clusters).
- Fuel: standing backlog next after CPU warn — **menu-bar GPU≥15% amber**.
- **Keep** amber **GPU** cue under metrics when usage ≥ 15%; hidden below that threshold (exact cue line `GPU`, not the tabbed label row).
- Discord Ready after install/kickstart (0.1.570).

## Next fuel

- Digester Discord traffic if any
- Deferred screenshots when TCC allows
- Prefer non-GPU-warn fuel: p50 latency / sibling ports / menu-bar Temp≥70°C amber / Monitors latency cue
