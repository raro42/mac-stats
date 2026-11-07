# Morning surprise — 2026-11-08

Overnight Track B kept shipping Dark/Neon/Futuristic opaque CRT catch-ups (#14) while Screen Recording TCC still blocks window capture of stale `docs/screens/feature-agent-ops.png` (~53d).

## Shipped tonight (selected)

- **v0.1.2457** — Monitors `.monitor-item.is-pending` / `.is-removing` CRT (`#0a0a0a` / `#08120c` / `#0b0b10`).
- **v0.1.2456** — Agent Ops disabled Save · Disk Cleanup path hover · Monitors `.is-checking` CRT.
- **v0.1.2455** — Perplexity filter chips + counts CRT.
- Earlier in the window: section kb-hints / muted captions, process rows, empty chips, filter-miss muted, Perplexity muted, LPM busy, metrics kb-hints, Processes filter chips, Agent Ops collapsed glances (through ~v0.1.2445+).

## Tried / deferred

- Mac `screencapture -l` for `feature-agent-ops.png` — TCC / no on-screen CPU window (polish-without-capture grace).
- Digester Slowest empty after instant noise filters — design-review + standing P2 #14 carried the night.

## Next

- Remaining Dark CRT leftovers: `.process-empty` · `.logs-viewer.is-empty` · Disk Cleanup empty/disabled/scope-rec · `#chat-input:focus`.
- Recapture stale feature screens when Screen Recording allows.
