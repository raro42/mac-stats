# Morning surprise — 2026-08-25

Overnight Track B shipped **v0.1.627** through **v0.1.630** — CPU-window fixes, AI Chat operator instant lane, Ollama settings keyboard polish, then a **p50 latency fix** for the CPU chat path.

## Latest keep (this tick)

**v0.1.630** — AI Chat **instant lane before ui-chat queue** — `/status`, `/insights`, greetings, version asks, and other zero-LLM replies no longer wait behind an in-flight Ollama turn (v0.1.628 only wired the agent router). **stop/cancel/interrupt** cooperative ack before the queue (Discord parity).

## Tonight's keeps

| Version | What |
|---------|------|
| **v0.1.630** | AI Chat instant lane + stop/cancel before ui-chat queue |
| **v0.1.629** | Ollama settings header↔body toolbar chain |
| **v0.1.628** | Operator instant lane in AI Chat (Discord gateway parity) |
| **v0.1.627** | LPM toggle + GPU sparklines + section persistence + Agent Ops hide + changelog wrap |

## Notes

- Digester open empty (10 turns, 7 instant); design review not due (feature-ai-chat ~11.36d grace).
- `debug.log`: quiet (180m scan — no ERROR/WARN clusters).
- Installed release @ v0.1.630; Discord Ready after install.
