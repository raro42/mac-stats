# UI Command Center (Agent Ops)

Agent Ops is the **operator-facing control center** for mac-stats: schedules, live sessions, knowledge, recent chats, and run health—above AI Chat.

## Layout (v0.1.130+)

1. **Health cards** — version, Discord Ready, next schedule, last delivery, digest open/stale
2. **Overview grid** — Schedules / Live / Knowledge / Recent chats (always visible when expanded)
3. **Detail tabs** — Agents, Sessions, Schedules, Knowledge, Runs

Data comes from existing Tauri commands (`list_schedules`, `list_live_sessions`, `list_memory_files`, `list_session_files`, `get_runs_insights`, …). Prefer presentation changes over new backends.

## Weekly UI review

**Cadence:** Wednesdays ~11:00 local (separate from harness overnight and Monday CHANGELOG hygiene).

**Skill:** `~/.mac-stats/agents/skills/ui-weekly-review/SKILL.md`

**Checklist:**

1. Open dashboard → Agent Ops expanded.
2. Confirm overview shows schedules, live, knowledge, recent chats without opening tabs.
3. One visible polish (spacing, empty state, card copy, max-height)—not a tool-parser fix.
4. Screenshot or note in Discord; commit + push when reasonable.

## Non-goals for the UI loop

- Digester heuristics, Discord gateway reconnect logic, tool XML parsers
- Competing with the overnight harness every 20 minutes
