# Morning surprise — 2026-09-07

Overnight Track B kept shipping p50 instant file-size lanes while digester open stayed empty.

## Shipped tonight (so far)

| Version | What |
|---------|------|
| **v0.1.912** | Instant: downloads-organizer-rules.md size (`organizer rules size` / `how big is downloads organizer rules`; no dump; path / state / `/downloads` untouched) |
| **v0.1.911** | Instant: `cookie_reject_patterns.md` size (`cookie reject size` / `how big is cookie reject patterns`; no dump; path lane untouched) |
| **v0.1.910** | Instant: `session_reset_phrases.md` size |
| **v0.1.909** | Instant: `escalation_patterns.md` size |
| **v0.1.908** | Instant: curated `memory.md` size |
| **v0.1.907** | Instant: per-agent `agent.json` size |

## Digester / design review

- Digester open: empty (stale Elmasnow weather; `Review logs` already instant).
- Design review: grace (not due); recommended surface still `feature-ai-chat` (~23.5d).

## Why it matters

Operators can ask how big the downloads organizer rules file is without waking Ollama or Brave — same pattern as cookie-reject / session-reset size lanes.
