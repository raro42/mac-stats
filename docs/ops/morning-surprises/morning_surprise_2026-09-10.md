# Morning surprise — 2026-09-10

Overnight Track B kept shipping instant-lane operator paths/sizes/ages so Discord asks skip Ollama.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.977** | Instant lane: `loop_backlog.md` size (`loop backlog size`, `how big is loop_backlog.md`, `harness tick log size`, …) — stat only; no dump; does not steal path / age / improvements / results.tsv |
| **v0.1.976** | Instant lane: `loop_backlog.md` path (`loop backlog path`, `where is loop_backlog.md`, `harness tick log path`, …) — path only; no dump; does not steal improvements / results.tsv |
| **v0.1.975** | Instant lane: `digest.md` / `latest.md` age (`digest.md age`, `how old is digest.md`, …) — `latest.md` mtime; does not steal cache `digest age` |
| **v0.1.974** | Instant lane: `session_reset_phrases.md` age |
| **v0.1.973** | Instant lane: `escalation_patterns.md` age |
| **v0.1.972** | Instant lane: `browser_storage_state.json` age |
| **v0.1.971** | Instant lane: `browser-credentials.toml` age |
| **v0.1.970** | Instant lane: `downloads-organizer-state.json` age |
| **v0.1.969** | Instant lane: `downloads-organizer-rules.md` age |
| **v0.1.968** | Instant lane: `cookie_reject_patterns.md` age |

## Also tried / status

- Digester open: empty (wake-up instant noise + one lite BRAVE_SEARCH “Review logs”).
- Design review: not due (grace; CPU metrics ~3.0d).
- Next fuel: `loop_backlog` age · sibling/standing path · digester open / debug.log when present.

## Fitness

Operators can ask how big `loop_backlog.md` is and get an instant byte size — one less direct-lane round trip for overnight harness status.
