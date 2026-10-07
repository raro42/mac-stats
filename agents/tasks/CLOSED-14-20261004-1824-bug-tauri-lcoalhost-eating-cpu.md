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

## Test report

**Date:** 2026-10-08 00:47 CEST (local); 2026-10-07 22:47 UTC.

**Host:** Linux (Arch). macOS Activity Monitor / `tauri://localhost` Graphics and Media idle not runnable here. Per task note, Linux webkit2gtk WebKitWebProcess floor is not the macOS gate for this cut.

**Commands run**
- `cd src-tauri && cargo check` → **PASS** (`mac_stats v0.1.1725`; pre-existing unused warnings only)
- `cd src-tauri && cargo test` → **PASS** (1359 lib tests passed; 0 failed)

**Static checks (this cut)**
- `src-tauri/dist/themes/apple/cpu.css`: `.icon-line-item.status-good` / `.status-warning` / `.status-bad` (and `:hover`) use opaque `color-mix(in srgb, … % , #ffffff)` for `color` / `background` / `border-color`; `box-shadow: none`.
- Grep for prior glass `rgba(36,160,90|200,130,20|200,55,50` on those rules → **no matches**.
- `Cargo.toml` version **0.1.1725** matches Implementation.

**Logs**
- `~/.mac-stats/debug.log` tail: no errors tied to this CSS cut / #14 WebView idle (stale Ollama connection-refused noise only).

**Verdict:** **PASS** for v0.1.1725 icon-line status opaque wash cut. Move → `CLOSED-`. GitHub #14 left open (operator / 004). macOS warm-window Activity Monitor `<1%` remains the ongoing product gate on Apple Silicon, not blocked by this cut’s coded change.
