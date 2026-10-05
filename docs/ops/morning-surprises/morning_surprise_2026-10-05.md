# Morning surprise — 2026-10-05

Overnight Track B put the Process Details header keyboard hint in the theme HTML.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1362** | Process Details header keyboard hint is in the theme HTML between the title and Close. It no longer pops in after JavaScript loads. It stays hidden until the title and Close are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1361** | Monitors add-form keyboard hint is in the theme HTML under the URL field, Cancel, and Add Monitor. It no longer pops in after JavaScript loads. It stays hidden until the add form is open. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1360** | Process Details Force Quit keyboard hint stays hidden until Advanced is open. Force Quit is not on screen while that section is closed. Keyboard tips stay out of the layout. |
| **v0.1.1359** | Process Details Force Quit keyboard hint is in the theme HTML inside the force-quit section. It no longer pops in after JavaScript loads. It stays hidden until Advanced and Force Quit are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1358** | Process Details name and PID keyboard hint is in the theme HTML inside the process detail hero. It no longer pops in after JavaScript loads. It stays hidden until the name and the PID are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1357** | Details keyboard hint is in the theme HTML above the Details grid. It no longer pops in after JavaScript loads. The line says how to move across those values. A collapsed Details section still hides the line. Keyboard tips stay out of the layout. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~19.4d).
- Debug.log: single-instance lock WARN (another launch exited). No panic.
- ~21:56 tick: Process Details header keyboard hint first paint in theme HTML (**v0.1.1362**); install/kickstart.
- ~21:30 tick: Monitors add-form keyboard hint first paint in theme HTML (**v0.1.1361**); install/kickstart.
- ~21:05 tick: Force Quit keyboard hint stays hidden until Advanced is open (**v0.1.1360**); install/kickstart.
- ~05:40 tick: Process Details Force Quit keyboard hint first paint in theme HTML (**v0.1.1359**); sync-dist + ratchet keep.
- ~20:31 tick: Process Details hero keyboard hint first paint in theme HTML (**v0.1.1358**).
- ~20:00 tick: Details keyboard hint first paint in theme HTML (**v0.1.1357**).
