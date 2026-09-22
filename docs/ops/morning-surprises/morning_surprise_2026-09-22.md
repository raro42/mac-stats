# Morning surprise — 2026-09-22

Overnight Track B (autoresearch) kept shipping Agent Ops empty-calm polish while digester open stayed empty.

## Shipped tonight (highlights)

| Version | What got better |
|---------|-----------------|
| **v0.1.1235** | Schedule / delivery empty **task** or **summary** say **None yet** on Overview + Schedules lists (preview already matched) |
| v0.1.1234 | Delivery empty schedule id → **Unknown** (not placeholder `schedule`) |
| v0.1.1233 | Insights Slowest empty wall/lane/question → **Unknown** / **None yet** |
| v0.1.1232 | Insights Candidates empty kind/reason/question → **Unknown** / **None yet** |
| v0.1.1231 | Runs/Schedules empty question/id → **None yet** / **Unknown** |
| v0.1.1230 | Knowledge/Sessions empty file title → **Unknown** |
| v0.1.1229 | Agents empty name/slug → **Unknown** |
| v0.1.1228 | Live empty source/session id → **Unknown** |
| v0.1.1227 | Sessions/Knowledge empty meta → **Unknown** |

## Context

- Digester open: empty all night (instant lane filters noise).
- Design review: `feature-agent-ops.png` in grace (~6.3d); empty-calm list parity was better fuel than a quiet tick.
- Debug.log: no ERROR/WARN/panic clusters in scan windows.
- Ratchet: keeps only (no discards this window).

## For Ralf

Open Agent Ops → Schedules. Jobs without a task body and deliveries without a summary now say **None yet** in the list, same as the preview pane.
