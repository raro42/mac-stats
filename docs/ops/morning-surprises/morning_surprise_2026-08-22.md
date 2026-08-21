# Morning surprise — 2026-08-22

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight (2026-08-21 20:00–…)

| Version | What |
|--------|------|
| **v0.1.572** | Menu bar **GHz** amber cue when cached CPU frequency **≥ 3.5 GHz** (power-strip freq is-hot parity; fresh `FREQ_CACHE` only) |
| **v0.1.573** | Menu bar **Up** amber cue when system uptime **≥ 7 days** (power-strip Up `is-long` parity; cheap `sysinfo` uptime) |
| **v0.1.574** | Menu bar **Bat** amber cue when cached battery **≤ 20%** and not charging (power-strip battery `is-low` wash parity) |
| **v0.1.575** | Menu bar **Mon** amber cue when any UP monitor responds **≥ 2000 ms** (Monitors summary slowest / latency parity; red **Mon ✕** still wins on DOWN) |
| **v0.1.576** | AI Chat **click-to-copy on messages** (Copied flash; drag-select safe; last-answer / processes / monitors parity) |
| **v0.1.577** | AI Chat **message keyboard nav** (↑↓ / j k · selected wash · Esc · c copy; Monitors / Top Processes listbox parity) |
| **v0.1.578** | Perplexity Search **result keyboard nav** (↑↓ / j k · Enter opens · c copies URL · Esc; Monitors / AI Chat listbox parity) |
| **v0.1.579** | Debug Log **line keyboard nav** (↑↓ / j k · selected wash · Enter/c copy · Esc; ERROR/WARN tint; auto-refresh skip) |
| **v0.1.580** | Disk Cleanup **row Copied flash + listbox chrome keyboard** (green Copied wash on scope/category; ↑↓ / j k from listbox → first/last) |
| **v0.1.581** | Top Processes **row Copied wash** (click name / `c` → green Copied badge on row; name button still flashes; Disk Cleanup / Debug Log parity) |

## Latest tick (~23:51)

- Digester open empty; design-review grace (feature-ai-chat ~7.46d recommended); debug.log quiet (no ERROR/WARN clusters).
- Fuel: Top Processes row Copied overlay (after Disk Cleanup copy wash; standing design-review listbox parity).
- **Keep** green Copied badge wash on process row; Discord Ready after install (0.1.581).
- Screenshot recapture deferred (TCC / warm window).

## Next fuel

- Digester Discord traffic if any
- Deferred screenshots when TCC allows
- Prefer non-Top-Processes-copy fuel: p50 latency / sibling ports / design-review when due / Monitors row Copied overlay if still a gap / listbox chrome first/last on process-list
