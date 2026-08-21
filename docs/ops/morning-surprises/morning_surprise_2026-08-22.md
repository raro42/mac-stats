# Morning surprise — 2026-08-22

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight (2026-08-21 20:00–…)

| Version | What |
|--------|------|
| **v0.1.572** | Menu bar **GHz** amber cue when cached CPU frequency **≥ 3.5 GHz** (power-strip freq is-hot parity; fresh `FREQ_CACHE` only) |
| **v0.1.573** | Menu bar **Up** amber cue when system uptime **≥ 7 days** (power-strip Up `is-long` parity; cheap `sysinfo` uptime) |
| **v0.1.574** | Menu bar **Bat** amber cue when cached battery **≤ 20%** and not charging (power-strip battery `is-low` wash parity) |
| **v0.1.575** | Menu bar **Mon** amber cue when any UP monitor responds **≥ 2000 ms** (Monitors summary slowest / latency parity; red **Mon ✕** still wins on DOWN) |

## Latest tick (~21:16)

- Digester open empty; design-review grace (feature-ai-chat ~7.36d); debug.log quiet (no ERROR/WARN clusters).
- Fuel: standing backlog next after Bat warn — **Monitors latency cue**.
- **Keep** amber **Mon** when all checks are up but any site is ≥ 2 s; summary amber wash also for a single slow UP site; Discord Ready after install/kickstart (0.1.575).

## Next fuel

- Digester Discord traffic if any
- Deferred screenshots when TCC allows
- Prefer non-Mon-latency fuel: p50 latency / sibling ports / design-review when due
