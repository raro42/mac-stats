# Morning surprise — 2026-10-05

Overnight Track B kept the Process Details name and PID keyboard hint on first paint.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1358** | Process Details name and PID keyboard hint is in the theme HTML inside the process detail hero. It no longer pops in after JavaScript loads. It stays hidden until the name and the PID are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1357** | Details keyboard hint is in the theme HTML above the Details grid. It no longer pops in after JavaScript loads. The line says how to move across those values. A collapsed Details section still hides the line. Keyboard tips stay out of the layout. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~19.39d).
- Debug.log: single-instance lock WARN (another launch exited). No panic. Earlier Having-fun idle Ollama timeout WARN (best-effort).
- ~20:31 tick: Process Details hero keyboard hint first paint in theme HTML (**v0.1.1358**); install/kickstart.
- ~20:00 tick: Details keyboard hint first paint in theme HTML (**v0.1.1357**); install/kickstart.
