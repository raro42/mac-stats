# Morning surprise — 2026-09-22

Overnight Track B (autoresearch) kept shipping Agent Ops empty-calm polish while digester open stayed empty.

## Shipped tonight (highlights)

| Version | What got better |
|---------|-----------------|
| **v0.1.1239** | Health **Last delivery** empty **summary preview** say **None yet** (not a bare age) |
| v0.1.1238 | Health **Next schedule** empty **task preview** say **None yet** (not a bare ETA) |
| v0.1.1237 | Live / Sessions empty **preview** say **None yet** (not omitted thinner meta) |
| v0.1.1236 | Knowledge / Sessions / Live empty **size**, **lines**, or **msgs** say **Unknown** (not `undefined` / `NaN MB`) |
| v0.1.1235 | Schedule / delivery empty **task** or **summary** say **None yet** on Overview + Schedules lists |
| v0.1.1234 | Delivery empty schedule id → **Unknown** (not placeholder `schedule`) |
| v0.1.1233 | Insights Slowest empty wall/lane/question → **Unknown** / **None yet** |
| v0.1.1232 | Insights Candidates empty kind/reason/question → **Unknown** / **None yet** |
| v0.1.1231 | Runs/Schedules empty question/id → **None yet** / **Unknown** |
| v0.1.1230 | Knowledge/Sessions empty file title → **Unknown** |

## Context

- Digester open: empty all night (instant lane filters noise).
- Design review: `feature-agent-ops.png` in grace (~6.39d); empty-calm list parity was better fuel than a quiet tick.
- Debug.log: no ERROR/WARN/panic clusters in scan windows.
- Ratchet: keeps only (no discards this window).

## For Ralf

Open Agent Ops → health **Last delivery**. When the newest delivery has an age but no summary, the line now says **2h · None yet** instead of a bare age. Next schedule already matched this pattern at **v0.1.1238**.
