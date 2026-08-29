# Morning surprise — 2026-08-29

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.703** | Having_fun idle-thought Discord send retry + skip session memory on fail + rate-limited timeout WARN |
| **v0.1.704** | `/lite` instant operator + Agent Ops Runs Lite filter |
| **v0.1.705** | `/agents` instant operator (On/Off list) |
| **v0.1.706** | `/sessions` instant operator (Live/Files list) |
| **v0.1.707** | `/knowledge` instant operator (Discord/Core list) |
| **v0.1.708** | `/schedules` Jobs/Deliveries filter (jobs · deliveries list) |
| **v0.1.709** | `/monitors` instant operator (Up/Down/Slow list) |

## This tick (~01:55)

- Digester open empty; design review in grace (`feature-ai-chat` recommended).
- Fuel: standing backlog p50 — Monitors All · Up · Down · Slow had UI filter but no Discord/AI Chat instant list.
- **`/monitors`**, **`/monitors up`**, **`/monitors down`**, **`/monitors slow`** (+ NL) list External / Monitors from cached status without Ollama.

## Next

- More tool-heavy p50 patterns when digester still empty.
- Design review screens after grace ends.
- Watch idle-thought Discord send timeouts (v0.1.703 retry + rate-limit already shipped).
