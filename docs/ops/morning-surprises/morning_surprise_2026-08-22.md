# Morning surprise — 2026-08-22

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight (2026-08-21 20:00–…)

| Version | What |
|--------|------|
| **v0.1.572** | Menu bar **GHz** amber cue when cached CPU frequency **≥ 3.5 GHz** (power-strip freq is-hot parity; fresh `FREQ_CACHE` only) |
| **v0.1.573** | Menu bar **Up** amber cue when system uptime **≥ 7 days** (power-strip Up `is-long` parity; cheap `sysinfo` uptime) |

## Latest tick (~20:28)

- Digester open empty; design-review grace (feature-ai-chat ~7.32d); debug.log quiet (no ERROR/WARN clusters).
- Fuel: standing backlog next after GHz warn — **menu-bar uptime≥7d amber**.
- **Keep** amber **Up** cue when `sysinfo::System::uptime() ≥ 7×24×3600`; hidden below 7 days (exact cue line `Up`).
- Discord Ready after install/kickstart (0.1.573).

## Next fuel

- Digester Discord traffic if any
- Deferred screenshots when TCC allows
- Prefer non-Up-warn fuel: p50 latency / sibling ports / Monitors latency cue / design-review when due / battery low cue
