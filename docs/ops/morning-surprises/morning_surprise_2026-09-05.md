# Morning surprise — 2026-09-05


## Overnight keep — v0.1.866 (~01:30)

**Instant lane: results.tsv path** — `results.tsv path`, `where is results.tsv`, `autoresearch results path`, `ratchet results path` → `~/.mac-stats/improvements/autoresearch/results.tsv` without Ollama (path only; no dump; does not steal `improvements path`).


Overnight Track B kept shipping instant-lane path wins for operator latency.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.866** | Instant lane: **results.tsv path** — keep/discard log path without Ollama |
| **v0.1.865** | Agent Ops Filter attention glance (On/Off · Live/Files · Jobs/Deliveries · Discord/Core · Runs lanes) |
| **v0.1.864** | Instant lane: **LaunchAgent plist path** — app + overnight harness plists |
| **v0.1.863** | Instant lane: **session-memory path** |
| **v0.1.862** | Instant lane: Discord channel memory path |

## Why it matters

Operators asking for the autoresearch keep/discard log used to fall through to Ollama or the improvements folder lane. Path-only asks now hit the instant lane.

## Digester / design review

- Digester open: empty (Elmasnow weather already filtered as shipped).
- Design review: grace (due=false); `feature-ai-chat` still aged — screenshot when TCC allows.

## Next

Non-path operator latency wins (e.g. runs.jsonl age), design-review polish when due, or digester open when present.
