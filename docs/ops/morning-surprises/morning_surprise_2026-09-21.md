# Morning surprise — 2026-09-21

Overnight Track B (20:00–06:00 local, window opened 2026-09-20 20:00). Digester open: empty. Debug log: Having-fun idle Ollama timeout (already soft-pathed).

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1197** | Session count — only inventory asks count sessions (“how many sessions”, “session count”, “number of sessions”, “count sessions”). “Edit this session”, “hide this session”, “move this session”, “copy this session”, “refresh this session”, and “update this session” stay with the agent. “How many sessions” still counts. “Open sessions” still lists. The verb-exclusion list is gone. |

## Why it matters

Ask “edit this session” or “refresh this session” and the agent handles it. You no longer get a session count. “How many sessions” still counts.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Agent Ops is in grace; recapture when Screen Recording allows)
