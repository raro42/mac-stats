# Morning surprise — 2026-09-21

Overnight Track B (20:00–06:00 window starting 2026-09-20).

## Shipped

### v0.1.1206 — Agent Ops Insights Slowest empty calm (~23:58)
- When Runs Insights has turns but Slowest is empty, show warm “Nothing slow” + solid accent wash (Digest Queue clear parity).
- Digester Slowest clear no longer hides the section.

### v0.1.1205 — Agent Ops Digest open empty calm (~23:35)
- When Insights has runs but digest open is zero, Queue clear uses warm title + solid accent wash (true-empty / filter-miss parity).
- Replaces the dashed muted one-line empty.

### v0.1.1204 — Agent Ops true-empty calm (~23:05)
- Empty Agents, Sessions, Schedules, Knowledge, and Runs tabs use warm title + solid accent wash (filter-miss / overview Ready parity).
- Knowledge empty copy drops the raw home-path dump.

### v0.1.1203 — Debug Log inventory counts (~22:35)
- “How many log errors”, “error count”, “count the errors”, and “number of warnings” answer with the tail count (no LLM).
- “Delete these errors”, “clear the log”, and “export warnings” stay with the agent.

### v0.1.1202 — Keep / discard inventory counts (~22:10)
- “Count the keeps”, “how many keeps”, and “number of discards” answer with the ratchet count (no LLM).
- “Delete these keeps”, “clear the discards”, and “export keeps” stay with the agent.


| Version | What got better |
|---------|-----------------|
| **v0.1.1206** | Agent Ops Insights Slowest empty calm (warm Nothing slow when digester Slowest is clear). |
| **v0.1.1205** | Agent Ops Digest open empty calm (warm title + solid wash when queue clear). |
| **v0.1.1204** | Agent Ops true-empty calm (warm title + solid wash). Knowledge empty copy softens. |
| **v0.1.1203** | Debug Log inventory counts answer only inventory asks (“count the errors”, …). “Delete these errors”, “clear the log”, “export warnings” stay with the agent. |
| **v0.1.1202** | Keep / discard inventory counts answer only inventory asks (“count the keeps”, …). “Delete these keeps”, “clear the discards”, “export keeps” stay with the agent. |
| **v0.1.1197** | Session count answers only inventory asks (“how many sessions”, …). Verb phrases like “edit this session” stay with the agent. |
| **v0.1.1198** | Operator inventory counts (agents, monitors, tasks, skills, plugins, knowledge) answer only inventory asks. “Delete this agent”, “run this skill”, “check this monitor” stay with the agent. |
| **v0.1.1199** | Runs inventory counts answer only inventory asks (“how many runs”, “run count”, …). “Delete these runs”, “clear the runs”, “export runs” stay with the agent. |
| **v0.1.1200** | Digest open inventory counts answer only inventory asks (“how many open candidates”, “digest count”, “open count”). “Open digest” stays the snapshot; “delete open candidates” stays with the agent. |
| **v0.1.1201** | Schedule / delivery inventory counts answer only inventory asks (“how many schedules”, “job count”, “count the schedules”, …). “Delete these schedules”, “clear the jobs”, “export deliveries”, “pause this schedule”, and “run this job” stay with the agent. |

## Tried / context

- Digester open stayed empty (no Slowest / open candidates).
- Design review: shipped Insights Slowest empty calm (**v0.1.1206**) after Digest open empty calm (**v0.1.1205**) and true-empty tabs (**v0.1.1204**); recapture `feature-agent-ops.png` still deferred (Screen Recording TCC).
- Debug.log: Having-fun idle Ollama timeout WARN — already soft-pathed; not product-owned ERROR fuel.

## Fitness

Insights Queue clear matches Agent Ops true-empty calm. Digest open empty no longer looks dashed and muted when the overnight queue is clear.
