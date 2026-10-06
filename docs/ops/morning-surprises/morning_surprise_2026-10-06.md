# Morning surprise — 2026-10-06

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1437** | Collapsed External / Monitors skips history Map + full list IPC on monitoring warm-up. Icon wash uses a light `list_monitors` + `get_monitor_status` walk (no per-host details). Expand hydrates list/history once. |
| **v0.1.1436** | AI visibility from localStorage on open; `get_ai_agent_enabled` waits for Settings Product. Ollama no longer auto-configures on DOMContentLoaded; expand / AI-on resume arms `ensureInitialized`. |
| **v0.1.1435** | Monitoring idle parks UI-state retry + pin hydrate while occluded. Compact uses localStorage on open; backend compact sync waits for Settings Product. Drops duplicate Ollama configure on monitoring idle. Agent Ops wait loops bail after alt-tab. |
| **v0.1.1434** | Settings credential/decorations IPC waits until Settings opens. Process Details open + Settings Monitors list skip IPC/DOM while parked; resume rebuilds the Monitors list. Changelog version wiring drops the body MutationObserver. |
| **v0.1.1433** | Settings Product toggles load AI only on open; Discord / decorations / changelog / footer version park while occluded. |
| **v0.1.1432** | Settings credential status + AI Chat stream buffer + Agent Ops digest park while occluded. |
| **v0.1.1431** | Agent Ops session / schedule / run / knowledge previews skip DOM after alt-tab. |
| **v0.1.1430** | AI Chat / Ollama connection checks and model-list loads skip IPC/DOM when parked. Perplexity key-status and monitor history Map rebuilds drop mid-flight. |
| v0.1.1428–1429 | Backend metrics require focus; history/charts shared park; leftover timer holdouts + Agent Ops batched IPC abort. |
| v0.1.1427 | Shared park gate for secondary IPC (Discord / Monitors / Process Details / Agent Ops). |
| v0.1.1420–1426 | Idle-defer focus-resume; cancel occluded open-path metrics; mid-flight DOM/rAF clear. |

## Why it matters

Monitors stays collapsed for most open sessions. Skipping history + list rebuild (and host-detail IPC on the icon walk) cuts a fat fan-out that used to stack with every monitoring warm-up. Expand still gets the full list once.

## Still open

- GitHub **#14** until macOS Activity Monitor shows the webview under ~1%.
- Design-review screenshot for `feature-agent-ops` when TCC allows.
