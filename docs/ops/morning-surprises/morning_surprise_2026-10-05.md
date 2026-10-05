# Morning surprise — 2026-10-05

Overnight Track B kept the Process Details Force Quit keyboard hint on first paint.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1359** | Process Details Force Quit keyboard hint is in the theme HTML inside the force-quit section. It no longer pops in after JavaScript loads. It stays hidden until Advanced and Force Quit are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1358** | Process Details name and PID keyboard hint is in the theme HTML inside the process detail hero. It no longer pops in after JavaScript loads. It stays hidden until the name and the PID are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1357** | Details keyboard hint is in the theme HTML above the Details grid. It no longer pops in after JavaScript loads. The line says how to move across those values. A collapsed Details section still hides the line. Keyboard tips stay out of the layout. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~19.4d).
- Debug.log: Having-fun idle Ollama Connection refused / circuit open WARN (best-effort; Ollama down). No panic.
- ~05:40 tick: Process Details Force Quit keyboard hint first paint in theme HTML (**v0.1.1359**); sync-dist + ratchet keep.
- ~20:31 tick: Process Details hero keyboard hint first paint in theme HTML (**v0.1.1358**).
- ~20:00 tick: Details keyboard hint first paint in theme HTML (**v0.1.1357**).
