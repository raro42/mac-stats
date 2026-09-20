# Morning surprise — 2026-09-21

Overnight Track B (20:00–06:00 window starting 2026-09-20).

## Shipped

| Version | What got better |
|---------|-----------------|
| **v0.1.1197** | Session count answers only inventory asks (“how many sessions”, …). Verb phrases like “edit this session” stay with the agent. |
| **v0.1.1198** | Operator inventory counts (agents, monitors, tasks, skills, plugins, knowledge) answer only inventory asks. “Delete this agent”, “run this skill”, “check this monitor” stay with the agent. |
| **v0.1.1199** | Runs inventory counts answer only inventory asks (“how many runs”, “run count”, …). “Delete these runs”, “clear the runs”, “export runs” stay with the agent. |
| **v0.1.1200** | Digest open inventory counts answer only inventory asks (“how many open candidates”, “digest count”, “open count”). “Open digest” stays the snapshot; “delete open candidates” stays with the agent. |
| **v0.1.1201** | Schedule / delivery inventory counts answer only inventory asks (“how many schedules”, “job count”, “count the schedules”, …). “Delete these schedules”, “clear the jobs”, “export deliveries”, “pause this schedule”, and “run this job” stay with the agent. |

## Tried / context

- Digester open stayed empty (no Slowest / open candidates).
- Design review: `feature-agent-ops` in grace (~5d); deferred for inventory-count correctness.
- Debug.log: Having-fun idle Ollama timeout WARN — already soft-pathed; not product-owned ERROR fuel.

## Fitness

Schedule and delivery counts stay instant for inventory asks. Verb phrases on schedules/jobs/deliveries no longer get a bare count. “Count the schedules” and similar inventory phrasing now hit the instant lane too.
