# [bug] tauri://lcoalhost eating CPU

## GitHub Issues
- **Issue:** https://github.com/raro42/mac-stats/issues/14
- **14**

## Problem / goal
When opening the mac-stats window, `tauri://localhost` eats CPU. Reduce Graphics and Media / WebView idle toward below 1% on macOS.

## High-level instructions for coder
- Follow `agents/006-feature-coder/FEATURE-CODER.md`.
- Prefer repo-relative paths. Do not paste home paths or secrets.
- After implementation: `cargo check` in `src-tauri/`, then rename `WIP-` → `UNTESTED-`.
- Do **not** close the GitHub issue (004 does that).

## Implementation (coder)

Version **v0.1.1725** (follow-up after v0.1.1724).

Profiler note (Linux webkit2gtk): a blank `cpu.html` still pegs WebKitWebProcess near a full core. That host floor is not the macOS `tauri://localhost` / Graphics and Media gate. Product cuts below still matter on macOS.

Changes (Apple icon-line status type skips glass blend):

- `src-tauri/dist/themes/apple/cpu.css` — `.icon-line-item.status-good` · `.status-warning` · `.status-bad` resting · hover `color` mix against opaque chip fill. Was `rgba(36,160,90,…)` / `rgba(200,130,20,…)` / `rgba(200,55,50,…)` glass on Ready / Slow / Down strip status type. Details / Top Processes body opaque in v0.1.1724.

Tester: open CPU window on macOS, warm ≥30s. Confirm Monitors/Ops icon-line Ready · Slow · Down status type stays solid (no glass blend). Gauges/sparklines still update. Check Activity Monitor Graphics and Media / `tauri://localhost` toward <1%. Do not close GitHub #14.

## Prior cuts (summary)

Opaque hairline / wash series through **v0.1.1723** (battery-icon · apple-title · icon-btn type · icon-line type · text/muted · panel tokens · modal-backdrop · ring-track · markdown table/hr · Add form · monitor-history · popover headers · chat-messages border · icon-strip / section dividers · shell · history charts · metric cards · Details/Top Processes · scrollbars · battery strip · History controls · menus · popovers · and earlier #14 opaque washes). Full prior Implementation + Linux tester FAIL reports lived in the previous 1MB WIP dump and were trimmed here so the queue stays readable.
