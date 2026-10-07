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

Version **v0.1.1722** (follow-up after v0.1.1721).

Profiler note (Linux webkit2gtk): a blank `cpu.html` still pegs WebKitWebProcess near a full core. That host floor is not the macOS `tauri://localhost` / Graphics and Media gate. Product cuts below still matter on macOS.

Changes (Apple window title type skips glass blend):

- `src-tauri/dist/themes/apple/cpu.css` — `.apple-title h1` `color` mix against opaque shell fill (`color-mix(in srgb, #1c1c1e 92%, #f2f2f6)`). Was `rgba(28, 28, 30, 0.92)` glass under the always-visible product title. Icon-btn type opaque in v0.1.1721.

Tester: open CPU window on macOS, warm ≥30s. Confirm title stays solid (no glass alpha through to the shell). Gauges/sparklines still update. Check Activity Monitor Graphics and Media / `tauri://localhost` toward <1%. Do not close GitHub #14.

## Prior cuts (summary)

Opaque hairline / wash series through **v0.1.1721** (icon-btn type · icon-line type · text/muted · panel tokens · modal-backdrop · ring-track · markdown table/hr · Add form · monitor-history · popover headers · chat-messages border · icon-strip / section dividers · shell · history charts · metric cards · Details/Top Processes · scrollbars · battery strip · History controls · menus · popovers · and earlier #14 opaque washes). Full prior Implementation + Linux tester FAIL reports lived in the previous 1MB WIP dump and were trimmed here so the queue stays readable.
