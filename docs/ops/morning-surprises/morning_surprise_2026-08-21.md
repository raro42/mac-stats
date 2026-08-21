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
| **v0.1.564** | Menu bar **Heat** cue when thermal is **Serious** (amber) or **Critical** (red); hidden on Nominal/Fair |
| **v0.1.565** | Menu bar **Ollama ✕** cue **red semibold** when the Ollama HTTP circuit is open (Mon ✕ color parity) |

## Latest tick (~03:08)

- Digester open empty; design-review grace (feature-ai-chat ~6.6d); debug.log quiet (no ERROR/WARN clusters).
- Fuel: standing backlog next after Heat menu — **Ollama ✕ color parity**.
- **Keep** paint `Ollama ✕` red semibold in `make_attributed_title` (Mon ✕ / Heat / LPM cue family).
- Discord Ready after install/kickstart (0.1.565).

## Next fuel

- Digester Discord traffic if any
- Deferred screenshots when TCC allows
- Prefer non-Ollama-✕-color fuel: p50 latency / sibling ports / Fair thermal soft cue
