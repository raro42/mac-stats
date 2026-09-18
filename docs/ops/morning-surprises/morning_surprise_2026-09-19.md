# Morning surprise — 2026-09-19

Overnight Track B (20:00–06:00 local, window opened 2026-09-18 20:00). Digester open: empty. Debug log quiet.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1160** | Battery-left instant — “how much battery is left?”, “battery left”, and “is the battery low” answer from the Bat chip (%, charging, no LLM); “why is the battery low” stays with the agent |
| **v0.1.1159** | How-hot instant — “how hot is the CPU?”, “is the CPU hot”, and “how hot” answer from the Temp ring (°C, no LLM); “is the GPU hot” uses the GPU ring; “why is the CPU hot” stays with the agent |
| **v0.1.1158** | Ollama URL instant — “what’s the ollama url?”, “ollama endpoint”, and “where is ollama” answer from the Ollama Ready chip (host + model, no LLM); “set the ollama url” stays a config change |
| **v0.1.1157** | Which-model instant — “which model are you?” and `/model` answer from the Ollama Ready chip (model name, no LLM); plural “which models” still lists models |
| **v0.1.1156** | CPU ring and sparkline Hot soft parity — hot metric cards and matching history charts at 7%/30% (Monitors Slow); Fair stays quieter; louder 12%/28% rest and 16%/44% pulse peak removed |
| **v0.1.1155** | Futuristic theme sidebar icon status soft parity — section icon good / warning / bad soft ok at 7%/28% and warn·bad at 7%/30% (apple Ready calm / Monitors Slow·Down); louder ~12%/28% tint removed |
| **v0.1.1154** | Agent Ops active selection soft parity — selected tab, count pill, and overview card wash at 7%/28% (Ready calm); louder 12%/40% tint removed |
| **v0.1.1153** | Architect theme sidebar icon status soft parity — section icon good / warning / bad soft ok at 7%/28% and warn·bad at 7%/30% (apple Ready calm / Monitors Slow·Down); louder ~10%/22% tint removed |

## Why it matters

Ask “how much battery is left?” and you get the charge at once. The chat model does not have to look that up.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Processes / Monitors; recapture AI Chat and Agent Ops when Screen Recording allows)
- Instant-lane leftovers: config changes (`set url`, `change model`) stay with the agent on purpose
