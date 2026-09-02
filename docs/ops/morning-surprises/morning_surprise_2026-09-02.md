# Morning surprise — 2026-09-02

## Shipped overnight

**v0.1.810 — Instant lane: debug log age**

`log age`, `how old is the log`, `when was log updated`, and similar short asks now return the debug.log last-write age from file mtime without Ollama or reading the tail (Discord + AI Chat). Complements v0.1.807–809 count, size, and path instant lanes.

**v0.1.809 — Instant lane: debug log path**

`where is the log`, `log file path`, `debug log path`, and similar short asks now return the debug.log path on disk without Ollama or reading the tail (Discord + AI Chat). Complements v0.1.807 error/warn counts and v0.1.808 file size. Settings → View logs still opens the file.

**v0.1.808 — Instant lane: debug log file size**

`log file size`, `how big is the log`, `debug log size`, and similar short asks now return the debug.log file size on disk without Ollama or reading the tail (Discord + AI Chat). Complements v0.1.807 error/warn counts. `/logs` still lists lines.

**v0.1.807 — Instant lane: debug log error/warn count**

`how many errors in the log`, `log error count`, `debug log count`, and similar short asks now return ERROR/WARN totals from the debug.log tail without Ollama or a full line dump (Discord + AI Chat). Matches Debug Log glance parity. `/logs` still lists lines.

**v0.1.806 — Instant lane: digest age read-only**

`digest age`, `how old is the digest`, `when was digest updated`, and similar short asks now return the cached digest timestamp plus open/stale counts from `latest.json` without re-running the Python digester (Discord + AI Chat). `/digest` still refreshes.

**v0.1.805 — Instant lane: digest open read-only**

`digest open`, `open candidates`, and similar short asks now return cached open hints from `latest.json` without re-running the Python digester (Discord + AI Chat). `/digest` still refreshes.

**v0.1.804 — Instant lane: runs count**

Short asks like `how many runs`, `run count`, `how many failed runs`, and lane-specific counts now return `runs.jsonl` totals without Ollama (Discord + AI Chat).

**v0.1.803 — Instant lane: operator inventory counts**

Short asks like `how many agents`, `how many monitors`, `task count`, and `how many open candidates` now return inventory totals without Ollama (Discord + AI Chat).

**v0.1.802 — Instant lane: schedule / delivery count**

Short asks like `how many schedules` and `how many deliveries` now return scheduler totals without Ollama (Discord + AI Chat).

**v0.1.801 — Instant lane: last delivery**

Short asks like `last delivery` and `when was the last delivery` now return the most recent scheduler delivery without Ollama (Discord + AI Chat).

**v0.1.800 — Instant lane: next schedule / next job**

Short asks like `next schedule` and `when is the next job` now return the upcoming fire time without Ollama (Discord + AI Chat).

**v0.1.799 — Agent Ops Signal health attention glance**

When Signal alerts are not wired (REST API pending), Agent Ops shows an amber attention strip under Slack. Click opens Settings → Credentials and scrolls to the Signal note.

## Context

- Digester had no open candidates (9 turns, all instant/direct noise filtered).
- Design review not due (grace on stale screenshots).
- Debug log: single-instance lock noise only (KeepAlive thrash; rate-limited in v0.1.381).

## Next up

- More p50 instant patterns (tool-heavy asks still on direct lane).
- Recapture `feature-ai-chat` screenshot when TCC allows.
- Sibling ports (Hermes insights / session UX) as backlog fuel.
