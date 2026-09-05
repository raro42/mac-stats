# Morning surprise — 2026-09-05

Overnight Track B (autoresearch) shipped **instant directory-size lanes** so operators can ask disk use without waking Ollama.

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.876** | Instant lane: **uploads** directory size (`uploads size`, `how big are uploads`, …) — recursive bytes under `~/.mac-stats/uploads/` |
| **v0.1.875** | Instant lane: **tmp** directory size |
| **v0.1.874** | Instant lane: **screenshots** directory size |
| **v0.1.873** | Instant lane: **improvements** directory size |
| **v0.1.872** | Instant lane: **digest.md / latest.md** file size |
| **v0.1.871** | Instant lane: **digest** (`latest.json`) file size |
| **v0.1.870** | Instant lane: **runs.jsonl** size |
| **v0.1.869** | Instant lane: **results.tsv** size |
| **v0.1.868** | Instant lane: **results.tsv** age |
| **v0.1.867** | Instant lane: **runs.jsonl** age |

## Digester / design review

- Digester **open** empty (Elmasnow weather already filtered as shipped).
- Design review **due=false** (grace); stale PNGs still aged (`feature-ai-chat` ~21.7d) — screenshot polish deferred (TCC).
- No ERROR/WARN clusters in the recent debug.log window.

## Next night fuel

- Dir sizes still open: traces / pdfs / agents / session / task / cleanup-quarantine.
- Or a design-review polish when screenshot capture works.
