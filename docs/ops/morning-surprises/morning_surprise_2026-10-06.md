# Morning surprise — 2026-10-06

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1488** | The Agent Ops Runs Fail/Slow glance mixes fail and slow washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1487** | The Debug Log Error/Warn glance mixes error and warn-only washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1486** | The Disk Cleanup Reclaim/Due glance mixes Big, Reclaim, and Due washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1485** | The Disk Cleanup filter glance mixes All, Reclaim, Big, and Clean washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1484** | The Top Processes Hot glance mixes the hot-count wash against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1483** | The Top Processes Filter glance mixes All, Pinned, and Hot washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1482** | The External / Monitors filter glance mixes All, Up, Down, and Slow washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1481** | The External / Monitors Down/Slow glance mixes down and slow washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1480** | The AI Chat offline attention glance mixes offline, no-model, ready, continue, sending, filter, errors, last-answer, and copied washes against an opaque fill. |
| **v0.1.1479** | The AI Chat errors glance mixes the failed-turn wash against an opaque fill. |
| **v0.1.1478** | The AI Chat last-answer glance mixes ready, error, and copied washes against an opaque fill. |
| **v0.1.1477** | The AI Chat turn glance mixes sending and calm washes against an opaque fill. |
| **v0.1.1476** | Agent Ops, process rows, logs, and Disk Cleanup no longer lift on hover or scale on press. Copied badges sit with margin, not a vertical translate. |
| **v0.1.1475** | The AI Chat model / connection glance mixes online, no-model, offline, and circuit washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1474** | CPU, GPU, Freq, and Temp history charts mix hot, calm, and Fair washes against an opaque fill. No glass alpha or ring shadow. |
| **v0.1.1473** | The AI Chat collapsed glance mixes online, offline, active, and error washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1472** | Press and hover no longer scale or lift. The Add button and the chat connection dot stay still. Apple theme, result, log, and Send controls drop the one-pixel lift. |
| **v0.1.1471** | Agent Ops collapsed glance mixes ready, warn, and offline washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1470** | Disk Cleanup collapsed glance mixes reclaim, due, scopes-off, and clean washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1469** | External / Monitors collapsed glance mixes up, down, and slow washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1468** | Top Processes keep-header glances (CPU · GPU · RAM) mix calm and hot washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1467** | Ring progress strokes draw their start in the path. Dark, Futuristic, Neon, Material, and Swiss no longer use a CSS rotate. |
| **v0.1.1466** | Details collapsed glance (Load · RAM · Up) mixes calm and hot washes against an opaque fill. No glass alpha or hover shadow. |
| **v0.1.1465** | CPU, GPU, Freq, and Temp ring cards mix hot, calm, and Fair washes against an opaque fill. No glass alpha or ring shadow. |
| **v0.1.1464** | Battery, power, Low Power Mode, and time-remaining status washes mix against an opaque fill. No glass alpha or ring shadow. |
| **v0.1.1463** | Settings toggle knobs sit with left offset, not a translate. An on switch no longer keeps a Graphics and Media layer. |
| **v0.1.1462** | Low Power Mode toggle sits on an opaque track. No glass alpha, inset highlight, or knob drop shadow. |
| **v0.1.1461** | Section icon chips sit on opaque fills. No glass alpha, inset highlight, or hover drop shadow. Status washes mix against an opaque color. |
| **v0.1.1460** | Ring numbers and the line under them center without a translate. Those labels no longer keep a Graphics and Media layer while the window is open. |
| **v0.1.1459** | Low Power Mode knob sits with left offset, not a translate. The battery strip does not keep a transform layer while LPM is on. |
| **v0.1.1458** | Section icons skip transform layers. Hover and press do not lift or scale those chips. The Monitors status dot uses offset, not translate. |
| **v0.1.1457** | Header Refresh and Settings skip transform layers. The divider uses offset, not translate. Hover and press do not lift or scale those buttons. |
| **v0.1.1456** | Ring gauges center without a translate. The SVG no longer keeps a Graphics and Media layer while the window is open. |
| **v0.1.1455** | Sparkline canvases skip GPU bind on open. Hover or Refresh still draws them. |
| **v0.1.1454** | Collapsed monitors, chat, logs, and Agent Ops skip idle-callback wiring on open. A click or Tab still opens them. |
| **v0.1.1453** | Settings and the changelog stay unwired until you open them. Collapsed AI Chat, Debug Log, and Disk Cleanup wait for expand. |
| **v0.1.1452** | Data-poster history charts skip theme-color reads on open. Colors load when a chart draws or a tooltip shows. |
| **v0.1.1451** | Sparkline park binds and hides the HTML canvases on open. Window resize does not allocate GPU buffers while parked. |
| **v0.1.1450** | History sparklines stay parked after the first metrics poll. Theme markup starts at 1×1. Hover, Refresh, or alt-tab resume still draws them. |
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

A Disk Cleanup filter (All, Reclaim, Big, or Clean) no longer paints that glance with a glass blend. The fill is opaque.

## Still open

- GitHub **#14** until macOS Activity Monitor shows the webview under ~1%.
- The Disk Cleanup Reclaim/Due glance still uses a glass fill. Monitor tick tips still use a translate. The refresh button still declares a rotate while it fetches. Agent Ops filter glances still use a glass fill.
- Design-review screenshot for `feature-agent-ops` when TCC allows.
