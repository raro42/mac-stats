# Morning surprise — 2026-10-06

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1427** | Shared park gate for secondary IPC. Mid-flight Discord icon, monitors summary/list, history availability, Process Details, and Agent Ops auto-refresh skip paint after alt-tab. Blur clears Process Details live refresh; focus re-arms if the modal is still open. |
| v0.1.1426 | Mid-flight DOM rAF clear; version tip/update skip; history-seed abort while parked. |
| v0.1.1425 | Blur cancels remaining occluded idle schedules (version, unpark, history seed, monitoring, Agent Ops init). |
| v0.1.1424 | Blur cancels open-path first-metrics + late-open idle; skip gauge DOM after mid-IPC alt-tab; chart-line cancels unpark on park. |
| v0.1.1420–1423 | Idle-defer focus-resume secondary polls; longer open warm-up defers. |

## Why it matters

Alt-tab used to leave Discord / Monitors / Agent Ops / Process Details free to finish IPC and rebuild DOM while the shell was parked. That woke WebKit the same way gauge mid-flight paint did. The shared pause gate closes that gap for the secondary surfaces.

## Still open

- GitHub **#14** until macOS Activity Monitor shows the webview under ~1%.
- Design-review screenshot for `feature-agent-ops` when TCC allows.
