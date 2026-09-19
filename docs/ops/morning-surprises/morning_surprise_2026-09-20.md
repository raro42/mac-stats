# Morning surprise — 2026-09-20

Overnight Track B (20:00–06:00 local, window opened 2026-09-19 20:00). Digester open: empty. Debug log quiet.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1186** | Session count — “export this session”, “share this session”, and “archive this session” stay with the agent. They no longer answer with the session count. “How many sessions” still counts. |
| **v0.1.1185** | Session count — “fork this session”, “duplicate this session”, and “clone this session” stay with the agent. They no longer answer with the session count. “How many sessions” still counts. |
| **v0.1.1184** | Session count — “resume this session”, “open this session”, “switch this session”, and “continue this session” stay with the agent. They no longer answer with the session count. “Open sessions” still lists. “How many sessions” still counts. |
| **v0.1.1183** | Session count — “summarize this session”, “rename this session”, “session summary”, and “title this session” stay with the agent. They no longer answer with the session count. “How many sessions” still counts. |
| **v0.1.1182** | Session count — “delete this session”, “remove this session”, “end this session”, and “close this session” stay with the agent. They no longer answer with the session count. “How many sessions” still counts. |
| **v0.1.1181** | Session count — “reset this session”, “clear this session”, and “new session” stay with the agent. They no longer answer with the session count. “How many sessions” still counts. Phrase-file path, size, and age stay on those lanes. |
| **v0.1.1180** | Session count — “compact this session” stays with the agent. It no longer answers with the session count. “How many sessions” still counts. |
| **v0.1.1179** | App uptime instant — “how long have you been running?”, “app uptime”, and “process uptime” answer with how long mac-stats has been up (no LLM). “What’s the uptime” and “how long has the Mac been up” still use the Up chip. “Why” stays with the agent. |
| **v0.1.1178** | Top GPU instant — “what’s using the most GPU?”, “which process is using the most GPU”, and “what’s eating the GPU” name that one process (no LLM). `/processes` still lists Top Processes. “Hot processes” stays Hot. “How much GPU” and “is the GPU hot” stay the GPU ring. “Why” stays with the agent. |
| **v0.1.1177** | Top RAM instant — “what’s using the most RAM?”, “which process is using the most memory”, and “what’s eating the RAM” name that one process (no LLM). `/processes` still lists Top Processes. “Hot processes” stays Hot. “How much RAM” stays the percent chip. “How big is memory” stays installed RAM. “Why” stays with the agent. |
| **v0.1.1176** | Top CPU instant — “what’s using the most CPU?”, “which process is using the most CPU”, and “what’s eating the CPU” name that one process (no LLM). `/processes` still lists Top Processes. “Hot processes” stays Hot. “How much CPU” stays the CPU ring. “Why” stays with the agent. `CURSOR_AGENT:` tool calls no longer answer with the agent count. |
| **v0.1.1175** | Disk free instant — “how much free space?”, “how much space is left”, and “how many GB free” answer with free disk bytes (no LLM); “how much disk is used” and bare “free space” still use the percent chip; Disk Cleanup stays on `/disk`; “why” stays with the agent |

## Why it matters

Ask “export this session” or “archive this session” and the agent handles it. You no longer get a session count. “How many sessions” still counts.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Agent Ops is in grace; recapture when Screen Recording allows)
- Instant-lane leftovers: “search this session” and “save this session” still match the session count. “Open sessions” stays the list. “Start over” and “fresh start” do not contain “session”, so they never hit that count.
