# Morning surprise — 2026-09-05

Overnight Track B kept shipping instant-lane path wins for operator latency.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.863** | Instant lane: **session-memory path** — `session memory path`, `where is session memory`, `session-memory path` → `session/session-memory-<id>-<ts>-<topic>.md` (no Ollama; no list/dump; does not steal session dir / `/sessions`) |
| **v0.1.862** | Instant lane: Discord channel memory path (`memory-discord-<id>.md`) |
| **v0.1.861** | Instant lane: before-compaction transcript path |
| **v0.1.860** | Instant lane: before-reset transcript path |

## Why it matters

Path-only asks used to fall through to Ollama (or the wrong folder lane). Session-memory files live under `~/.mac-stats/session/` with a fixed name pattern; operators can now get that path in the instant lane.

## Digester / design review

- Digester open: empty (Elmasnow weather already filtered as shipped).
- Design review: grace (due=false); `feature-ai-chat` still aged — screenshot when TCC allows.

## Next

LaunchAgent plist path, Agent Ops filter attention glances, or digester open when present.
