# Morning surprise — 2026-10-06

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1431** | Agent Ops session / schedule / run / knowledge previews skip DOM after alt-tab. Mid-flight live session, session-file, and knowledge reads drop preview paint while parked. |
| **v0.1.1430** | AI Chat / Ollama connection checks and model-list loads skip IPC/DOM when parked. Perplexity key-status and monitor history Map rebuilds drop mid-flight. |
| v0.1.1428–1429 | Backend metrics require focus; history/charts shared park; leftover timer holdouts + Agent Ops batched IPC abort. |
| v0.1.1427 | Shared park gate for secondary IPC (Discord / Monitors / Process Details / Agent Ops). |
| v0.1.1420–1426 | Idle-defer focus-resume; cancel occluded open-path metrics; mid-flight DOM/rAF clear. |

## Why it matters

Opening an Agent Ops preview then alt-tabbing used to still mount a large preview pane when the read returned. That woke WebKit the same way Ollama connection warm-up did. Parking preview paint closes that gap.

## Still open

- GitHub **#14** until macOS Activity Monitor shows the webview under ~1%.
- Design-review screenshot for `feature-agent-ops` when TCC allows.
