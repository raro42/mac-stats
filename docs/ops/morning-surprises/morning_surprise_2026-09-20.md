# Morning surprise — 2026-09-20

Overnight Track B (20:00–06:00 local, window opened 2026-09-19 20:00). Digester open: empty. Debug log quiet (idle-thought timeout already soft-pathed).

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1177** | Top RAM instant — “what’s using the most RAM?”, “which process is using the most memory”, and “what’s eating the RAM” name that one process (no LLM). `/processes` still lists Top Processes. “Hot processes” stays Hot. “How much RAM” stays the percent chip. “How big is memory” stays installed RAM. “Why” stays with the agent. |
| **v0.1.1176** | Top CPU instant — “what’s using the most CPU?”, “which process is using the most CPU”, and “what’s eating the CPU” name that one process (no LLM). `/processes` still lists Top Processes. “Hot processes” stays Hot. “How much CPU” stays the CPU ring. “Why” stays with the agent. `CURSOR_AGENT:` tool calls no longer answer with the agent count. |
| **v0.1.1175** | Disk free instant — “how much free space?”, “how much space is left”, and “how many GB free” answer with free disk bytes (no LLM); “how much disk is used” and bare “free space” still use the percent chip; Disk Cleanup stays on `/disk`; “why” stays with the agent |

## Why it matters

Ask “what’s using the most RAM?” and you get one process name. The chat model does not have to look that up.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Agent Ops is in grace; recapture when Screen Recording allows)
- Instant-lane leftovers: “compact this session” still hits a count path; config changes (`set url`, `change model`) stay with the agent on purpose
