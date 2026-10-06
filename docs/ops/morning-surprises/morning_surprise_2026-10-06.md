# Morning surprise — 2026-10-06

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1449** | Data-poster metric cards stay parked on open. Bar and line charts do not allocate buffers or draw on first paint. The same idle unpark as the history charts draws them later. Canvas markup starts at 1×1. |
| **v0.1.1448** | Collapsed Agent Ops skips filter, overview, and keyboard wiring on monitoring idle. Expand still hydrates once. |
| **v0.1.1447** | Capture `MAC_STATS_OPEN_SECTION` / `openUiSection` is baked into `cpu.html?open=` when the window is created. Agent Ops no longer calls `take_open_ui_section` after load. |
| **v0.1.1446** | Section collapse skips `get_cpu_window_ui_state` on monitoring warm-up. Agent Ops capture is one invoke (no retry loop). Collapsed Disk Cleanup skips glance IPC on resume. |
| **v0.1.1445** | Data-poster history charts stay parked on open. First paint does not allocate canvas buffers or fetch history. The same idle unpark as the other themes draws them later. |
| **v0.1.1444** | Collapsed Top Processes skips `get_pinned_process_names` on open and resume. Pins paint from localStorage. Expand hydrates from disk. |
| **v0.1.1443** | Collapsed External / Monitors skips `list_monitor_statuses` on open, resume, and the hourly timer. The icon paints last-known up/down from localStorage. Expand still hydrates the list. |
| **v0.1.1442** | Discord icon skips `is_discord_gateway_ready` on open/resume. Last-known connected state paints from localStorage. Click and Settings still check. |
| **v0.1.1441** | Settings credential Save/Clear (Brave through Signal) wires on Settings open, not on monitoring idle. |
| **v0.1.1440** | Collapsed Debug Log skips `read_debug_log` glance IPC. Expand arms the glance poll. |
| **v0.1.1439** | Idle-thought Ollama timeouts log one warning per five minutes, even when several fire in the same second. The rest stay debug. |
| **v0.1.1438** | Monitors summary, list, and settings list use one `list_monitor_statuses` IPC. The 24h history probe waits until sparkline unpark or history seed. |
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

Data-poster metric cards no longer draw bar and line charts on the first paint. Those charts wait for the same idle unpark as the history charts. Open does not allocate full canvas bitmaps for them.

## Still open

- ~21:10 tick: data-poster metric-card charts stay parked on open (**v0.1.1449**). Rebased past parallel #14 ships v0.1.1446–1448.
- GitHub **#14** until macOS Activity Monitor shows the webview under ~1%.
- Design-review screenshot for `feature-agent-ops` when TCC allows.
