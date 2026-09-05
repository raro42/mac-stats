# Morning surprise — 2026-09-05

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.871** | Instant lane: **digest file size** — `digest size`, `how big is the digest`, `latest.json size` return on-disk size from digester `latest.json` without Ollama (stat only; no digester spawn / open dump; does not steal `digest age` / `digest open` / `/digest`) |
| **v0.1.870** | Instant lane: **runs.jsonl size** — `runs size`, `how big is runs.jsonl` (stat only; no list/count) |
| **v0.1.869** | Instant lane: **results.tsv size** — `results.tsv size`, `how big is results.tsv` (stat only; no dump) |
| **v0.1.868** | Instant lane: **results.tsv age** — mtime only |
| **v0.1.867** | Instant lane: **runs.jsonl age** — mtime only |
| **v0.1.866** | Instant lane: **results.tsv path** |
| **v0.1.865** | Agent Ops Filter attention glance |

## This tick (~03:45)

- Digester **open** empty (Elmasnow weather already stale/shipped).
- Design review **due=false** (grace; `feature-ai-chat` still aged).
- Fuel: standing backlog p50 — digest `latest.json` size after runs.jsonl size.
- Ratchet **keep** @ `74e8105d`.

## Tried / notes

- No discard this tick.
- Debug.log: no ERROR/WARN clusters in the scan window.
- Next fuel: more p50 instant gaps, or design-review screenshot when TCC allows.

_Generated for digester / instant “what shipped overnight” asks._
