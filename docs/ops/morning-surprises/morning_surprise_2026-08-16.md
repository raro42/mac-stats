# Morning surprise — 2026-08-16

Overnight Track B (20:00–06:00 starting 2026-08-15).

## Shipped
- **v0.1.430** — Agent Ops design review: overview cards highlight when their linked tab is active; health ok/warn/bad cards use a soft status wash so Digest open is easier to spot.
- **v0.1.431** — Agent Ops design review: active tabs (and agent file tabs) use the accent wash so the selected tab matches overview cards; on/off badges use clearer green glass.
- **v0.1.432** — Agent Ops Sessions: Load into AI Chat shows a Loaded flash on the control and blocks double Enter/click/dblclick while flashing.
- **v0.1.433** — Settings: Reset to monitor defaults shows Resetting… then a Reset flash on the control and blocks double click while in flight or flashing.
- **v0.1.434** — AI Chat system-prompt: Reset to Default shows Resetting… then a Reset flash; blocks double click while flashing (popover stays open; Save still persists).

## Tried / notes
- Digester Slowest empty; design-review still due on `feature-agent-ops.png` / `feature-cpu-metrics.png` (~3.5d) because window-only recapture remains TCC-blocked.
- After design-review keeps + Load flash + Settings Reset, tick picked AI Chat Reset to Default busy-guard (non-Settings-reset-adjacent).
- Install/kickstart: Discord Ready expected on 0.1.434 after release install.

## Ratchet
- keep @ e4bc3a8 — v0.1.430 Agent Ops overview active + health wash
- keep @ 2ec6cd6 — v0.1.431 Agent Ops tab accent + badge glass
- keep @ 6f12433 — v0.1.432 Agent Ops Load into AI Chat Loaded flash
- keep @ 2338ce3 — v0.1.433 Settings Reset defaults busy-guard + Reset flash
- keep @ ae8fccd — v0.1.434 AI Chat Reset to Default busy-guard + Reset flash
