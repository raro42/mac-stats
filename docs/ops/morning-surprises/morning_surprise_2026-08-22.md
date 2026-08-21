# Morning surprise — 2026-08-22

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight (2026-08-21 20:00–…)

| Version | What |
|--------|------|
| **v0.1.572** | Menu bar **GHz** amber cue when cached CPU frequency **≥ 3.5 GHz** (power-strip freq is-hot parity; fresh `FREQ_CACHE` only) |
| **v0.1.573** | Menu bar **Up** amber cue when system uptime **≥ 7 days** (power-strip Up `is-long` parity; cheap `sysinfo` uptime) |
| **v0.1.574** | Menu bar **Bat** amber cue when cached battery **≤ 20%** and not charging (power-strip battery `is-low` wash parity) |

## Latest tick (~20:52)

- Digester open empty; design-review grace (feature-ai-chat ~7.34d); debug.log quiet (no ERROR/WARN clusters).
- Fuel: standing backlog next after Up warn — **menu-bar battery≤20% amber**.
- **Keep** amber **Bat** cue when `BATTERY_CACHE` level ≤ 20 and not charging; soft amber wash on `.battery-info.is-low`; hidden when charging / above 20% / missing cache.
- Discord Ready after install/kickstart (0.1.574).

## Next fuel

- Digester Discord traffic if any
- Deferred screenshots when TCC allows
- Prefer non-Bat-warn fuel: p50 latency / sibling ports / Monitors latency cue / design-review when due
