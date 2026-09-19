# Morning surprise — 2026-09-19

Overnight Track B (20:00–06:00 local, window opened 2026-09-18 20:00). Digester open: empty. Debug log quiet.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1170** | Heat-high instant — “is the heat high?”, “is the thermal high”, and “is it throttling” answer from the Heat chip (thermal state, no LLM); “why is the heat high” stays with the agent; “is the CPU hot” still uses the Temp ring; “hot processes” still lists hot processes |
| **v0.1.1169** | P-core / E-core clock instant — “how fast are the P-cores?”, “p core frequency”, and “is the P-core high” answer from the P-core clock (GHz, no LLM); “e core frequency” and “how fast are the E-cores” use the E-core clock; “why is the P-core high” stays with the agent; “how fast is the CPU” still uses the Freq ring |
| **v0.1.1168** | Load 5m / 15m instant — “what’s the 5 minute load?”, “15 minute load”, and “is the 5 minute load high” answer from the Load 5m or 15m chip (no LLM); “why is the 5 minute load high” stays with the agent; `/load` still opens the full Details panel; “is the load high” still uses the 1-minute Load chip |
| **v0.1.1167** | Load-high instant — “is the load high?”, “how's the load”, and “how high is the load” answer from the Load chip (1-minute load, no LLM); “why is the load high” stays with the agent; `/load` still opens the full Details panel |
| **v0.1.1166** | Power-draw instant — “how much power is used?”, “power draw”, and “is the power high” answer from the Power chip (CPU+GPU watts, no LLM); “why is the power high” stays with the agent; `/power` still opens the full strip; Low Power Mode stays on `/lpm` |
| **v0.1.1165** | Clock-speed instant — “how fast is the CPU?”, “clock speed”, and “is the frequency high” answer from the Freq ring (GHz, no LLM); “why is the frequency high” stays with the agent; “how much CPU” still uses the CPU ring |
| **v0.1.1164** | GPU-used instant — “how much GPU is used?”, “is the GPU high”, and “is the GPU busy” answer from the GPU ring (%, no LLM); “why is the GPU high” stays with the agent; “is the GPU hot” still uses the GPU ring |
| **v0.1.1163** | CPU-used instant — “how much CPU is used?”, “is the CPU high”, and “is the CPU busy” answer from the CPU ring (%, no LLM); “why is the CPU high” stays with the agent; “hot processes” still lists hot processes |
| **v0.1.1162** | Disk-used instant — “how much disk is used?”, “how much storage”, and “is the disk full” answer from the SSD chip (%, no LLM); “why is the disk full” stays with the agent; Disk Cleanup stays on `/disk` |
| **v0.1.1161** | RAM-used instant — “how much RAM is used?”, “how much memory”, and “is the RAM high” answer from the RAM chip (%, no LLM); “why is the RAM high” stays with the agent |
| **v0.1.1160** | Battery-left instant — “how much battery is left?”, “battery left”, and “is the battery low” answer from the Bat chip (%, charging, no LLM); “why is the battery low” stays with the agent |
| **v0.1.1159** | How-hot instant — “how hot is the CPU?”, “is the CPU hot”, and “how hot” answer from the Temp ring (°C, no LLM); “is the GPU hot” uses the GPU ring; “why is the CPU hot” stays with the agent |
| **v0.1.1158** | Ollama URL instant — “what’s the ollama url?”, “ollama endpoint”, and “where is ollama” answer from the Ollama Ready chip (host + model, no LLM); “set the ollama url” stays a config change |
| **v0.1.1157** | Which-model instant — “which model are you?” and `/model` answer from the Ollama Ready chip (model name, no LLM); plural “which models” still lists models |
| **v0.1.1156** | CPU ring and sparkline Hot soft parity — hot metric cards and matching history charts at 7%/30% (Monitors Slow); Fair stays quieter; louder 12%/28% rest and 16%/44% pulse peak removed |
| **v0.1.1155** | Futuristic theme sidebar icon status soft parity — section icon good / warning / bad soft ok at 7%/28% and warn·bad at 7%/30% (apple Ready calm / Monitors Slow·Down); louder ~12%/28% tint removed |
| **v0.1.1154** | Agent Ops active selection soft parity — selected tab, count pill, and overview card wash at 7%/28% (Ready calm); louder 12%/40% tint removed |
| **v0.1.1153** | Architect theme sidebar icon status soft parity — section icon good / warning / bad soft ok at 7%/28% and warn·bad at 7%/30% (apple Ready calm / Monitors Slow·Down); louder ~10%/22% tint removed |

## Why it matters

Ask “is the heat high?” and you get the thermal state at once. The chat model does not have to look that up.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Processes / Monitors; recapture AI Chat and Agent Ops when Screen Recording allows)
- Instant-lane leftovers: config changes (`set url`, `change model`) stay with the agent on purpose
