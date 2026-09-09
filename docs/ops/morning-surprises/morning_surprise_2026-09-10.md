# Morning surprise — 2026-09-10

Overnight Track B kept shipping instant-lane file ages so Discord/operator asks skip Ollama.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.975** | Instant lane: `digest.md` / `latest.md` age (`digest.md age`, `how old is digest.md`, `when was latest.md updated`, …) — `latest.md` mtime; does not steal cache `digest age` |
| **v0.1.974** | Instant lane: `session_reset_phrases.md` age (`session reset age`, `how old is session reset phrases`, …) — mtime only; path/size/escalation/cookie reject safe |
| **v0.1.973** | Instant lane: `escalation_patterns.md` age (`escalation age`, `how old is escalation patterns`, …) — mtime only; path/size/session-reset/cookie reject safe |
| **v0.1.972** | Instant lane: `browser_storage_state.json` age |
| **v0.1.971** | Instant lane: `browser-credentials.toml` age |
| **v0.1.970** | Instant lane: `downloads-organizer-state.json` age |
| **v0.1.969** | Instant lane: `downloads-organizer-rules.md` age |
| **v0.1.968** | Instant lane: `cookie_reject_patterns.md` age |

## Also tried / status

- Digester open: empty (wake-up instant noise + one lite BRAVE_SEARCH “Review logs”).
- Design review: not due (grace; CPU metrics ~2.9d).
- Next fuel: more p50 gaps / digester open / debug.log when present; sibling ports.

## Fitness

Operators can ask how old `latest.md` is and get an instant mtime answer — separate from cache `digest age`, one less direct-lane round trip.
