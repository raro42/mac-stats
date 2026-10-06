# Morning surprise — 2026-10-06

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1424** | Blur cancels open-path first-metrics + late-open idle (not only focus-resume). Skip gauge DOM after `get_cpu_details` if already occluded. Chart-line cancels pending sparkline unpark on park. Focus re-schedules metrics if arming never happened. |
| v0.1.1423 | Further idle-defer of metrics / focus interval arm (prior tick). |
| v0.1.1420–1422 | Idle-defer focus-resume secondary polls; longer open warm-up defers. |

## Why it matters

Alt-tab during the first minutes used to leave idle callbacks armed. They could still wire gauges and allocate sparkline GPU while the shell was parked. Mid-IPC paint after an alt-tab could wake WebKit the same way.

## Still open

- GitHub **#14** until macOS Activity Monitor shows the webview under ~1%.
- Design-review screenshot for `feature-agent-ops` when TCC allows.
