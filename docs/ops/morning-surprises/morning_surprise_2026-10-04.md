# Morning surprise — 2026-10-04

Overnight Track B kept the Agent Ops overview keyboard hint on first paint.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1335** | Agent Ops overview keyboard hint is in the theme HTML under the overview cards. It no longer pops in after JavaScript loads. The line says how to move across Agents, Schedules, Live, Knowledge, Recent chats, Runs, and Digest. Keyboard tips stay out of the layout. |
| **v0.1.1334** | Agent Ops filter-row keyboard hint is in the theme HTML under the search box, the match count, and Clear. It no longer pops in after JavaScript loads. It stays hidden until a search shows the match count and Clear. The line says how to move across those controls. Keyboard tips stay out of the layout. |
| **v0.1.1333** | Agent Ops edit-actions keyboard hint is in the theme HTML under Save, Load into AI Chat, and Back. It no longer pops in after JavaScript loads. The line says how to move across those actions. The action row has its id, so the keys attach on theme shells. Keyboard tips stay out of the layout. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~18.43d).
- Debug.log: no ERROR/WARN/panic in the 180 minute window.
- ~21:24 tick: Agent Ops overview keyboard hint first paint in theme HTML (**v0.1.1335**); install/kickstart.
- ~21:00 tick: Agent Ops filter-row keyboard hint first paint in theme HTML (**v0.1.1334**); install/kickstart.
- ~20:30 tick: Agent Ops edit-actions keyboard hint first paint in theme HTML (**v0.1.1333**); install/kickstart.
