# Morning surprise — 2026-09-05

Overnight Track B (autoresearch) ships for Ralf.

| Version | What |
|---------|------|
| **v0.1.870** | Instant lane: **runs.jsonl size** — `runs size` / `how big is runs.jsonl` / `runs file size` return on-disk size without Ollama; no list/count; does not steal `runs path` / `runs age` / `/insights`. |
| **v0.1.869** | Instant lane: **results.tsv size** — `results.tsv size` / `how big is results.tsv` / `results file size` return on-disk size without Ollama; no dump; does not steal path / age / `improvements path`. |
| **v0.1.868** | Instant lane: **results.tsv age** — `results.tsv age` / `how old is results.tsv` / `when was results.tsv updated` return last-write age (mtime) without Ollama; no dump; does not steal `results.tsv path` / `improvements path`. |
| **v0.1.867** | Instant lane: **runs.jsonl age** — `runs age` / `how old is runs.jsonl` / `when was runs updated` return last-write age (mtime) without Ollama; no list/count; does not steal `runs path` / `/insights`. |
| **v0.1.866** | Instant lane: **results.tsv path** — autoresearch ratchet results file path without Ollama. |
| **v0.1.865** | Agent Ops **Filter attention glance** (On/Off · Live/Files · Jobs/Deliveries · Discord/Core · Runs lanes). |

## Notes
- Digester open stayed empty (stale Elmasnow weather already shipped).
- Design review in grace (`feature-ai-chat` ~21d); no TCC screenshot this tick.
- Nightly keep/discard satisfied (multiple keeps in `results.tsv`).
