# Morning surprise — 2026-10-03

Overnight Track B kept the AI Chat turn glance on first paint.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1308** | AI Chat turn glance is in the theme HTML under the model glance. It no longer pops in after JavaScript loads. Before a turn it says **None yet** and stays hidden. |
| **v0.1.1307** | AI Chat collapsed glance is in the theme HTML under the header. It no longer pops in after JavaScript loads. Before a connection check it says **Not set · configure URL**. It stays hidden until the section is collapsed. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~17.4d).
- Debug.log: no ERROR/WARN/panic in the 180 minute window at the ~20:31 tick.
- ~20:31 tick: turn glance first paint in theme HTML (**v0.1.1308**); install/kickstart.
- ~20:05 tick: collapsed glance first paint in theme HTML (**v0.1.1307**); install/kickstart.
