# Morning surprise — 2026-09-08

Overnight Track B kept shipping p50 instant lanes and one AI Chat design polish.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.949** | Instant lane: before-reset transcript age (`before reset transcript age`, `before-reset transcript age`, `how old is before reset transcript`, `when was before reset transcript updated`, `last_session_before_reset.jsonl age`, …) — mtime only; no dump/hook; does not steal path / size / before-compaction / session reset phrases |
| **v0.1.948** | Instant lane: Discord channel memory age (`discord memory age`, `memory-discord age`, `channel memory age`, `how old is discord memory`, `when was discord memory updated`, …) — newest `memory-discord-*.md` mtime; no dump; does not steal path / size / `/knowledge discord` |
| **v0.1.947** | Instant lane: session-memory age (`session memory age`, `how old is session memory`, `when was session memory updated`, …) — newest `session-memory-*.md` mtime; no dump; does not steal path / size / session folder / `/sessions` |
| **v0.1.946** | Instant lane: notes folder age (`notes age`, `how old are notes`, `memory folder age`, …) — newest file mtime; no dump; does not steal path / size / memory.md age / bare `memory age`; session-memory size plural/`.md` detector fix |
| **v0.1.945** | Instant lane: `memory.md` age (`memory.md age`, `curated memory age`, `how old is memory.md`, …) — mtime only; no dump; does not steal path / size / notes / bare `memory age` |
| **v0.1.944** | Instant lane: `execution_prompt.md` age (`execution age`, `how old is execution`, …) — mtime only; no dump; does not steal path / size / planning |
| **v0.1.943** | Instant lane: `planning_prompt.md` age |
| **v0.1.942** | AI Chat filter-miss calm (You/Assistant/Errors empty → warm titled hint + cue wash; design review / `feature-ai-chat`) |
| **v0.1.941** | Instant lane: `agent.json` age |
| **v0.1.940** | Instant lane: `testing.md` age |
| **v0.1.939** | Instant lane: `skill.md` age |
| **v0.1.938** | AI Chat empty Ready calm (design review) |
| **v0.1.937** | Instant lane: `mood.md` age |
| **v0.1.936** | Instant lane: `soul.md` age |

## Still open

- Recapture stale `docs/screens/feature-ai-chat.png` when TCC / on-screen window allows (polish already shipped; one design-review/night cap used).
- Next p50 fuel: before-compaction transcript age, Ori vault age (and remaining age lanes).
- Sibling: Hermes insights / session UX ports when digester is quiet.

Generated: 2026-09-08T23:32:00+02:00
