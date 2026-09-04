# Morning surprise — 2026-09-05


## Overnight keep — v0.1.865 (~01:05)

**Agent Ops Filter attention glance** — On/Off · Live/Files · Jobs/Deliveries · Discord/Core · Runs lanes show **Filter · …** above chips; click → All. Fail/Slow strip defers on Runs Fail/Slow Filter. Design-review fuel (`feature-agent-ops`).


Overnight Track B kept shipping instant-lane path wins for operator latency.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.864** | Instant lane: **LaunchAgent plist path** — `launchagent path`, `where is launchagent`, `mac-stats.plist`, `harness plist` → `~/Library/LaunchAgents/com.raro42.mac-stats.plist` + overnight harness plist (no Ollama; no load/unload; does not steal overnight-improvements asks) |
| **v0.1.863** | Instant lane: **session-memory path** — `session memory path`, `where is session memory`, `session-memory path` → `session/session-memory-<id>-<ts>-<topic>.md` (no Ollama; no list/dump; does not steal session dir / `/sessions`) |
| **v0.1.862** | Instant lane: Discord channel memory path (`memory-discord-<id>.md`) |
| **v0.1.861** | Instant lane: before-compaction transcript path |
| **v0.1.860** | Instant lane: before-reset transcript path |

## Why it matters

Path-only asks used to fall through to Ollama (or the wrong folder lane). Operators can now get KeepAlive LaunchAgent plist paths and session-memory file patterns in the instant lane.

## Digester / design review

- Digester open: empty (Elmasnow weather already filtered as shipped).
- Design review: grace (due=false); `feature-ai-chat` still aged — screenshot when TCC allows.

## Next

Non-path operator latency wins, Agent Ops filter attention glances, or digester open when present.
