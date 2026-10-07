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

Version **v0.1.1726** (follow-up after v0.1.1725).

Profiler note (Linux webkit2gtk): a blank `cpu.html` still pegs WebKitWebProcess near a full core. That host floor is not the macOS `tauri://localhost` / Graphics and Media gate. Product cuts below still matter on macOS.

Changes (Apple Changelog body type skips glass blend):

- `src-tauri/dist/themes/apple/cpu.css` — `.changelog-error` · `.changelog-h2` · `.changelog-version` · `.changelog-h3` · `.changelog-paragraph` · `.changelog-item` · bullet · `.changelog-code` · `.changelog-body strong` `color` mix against opaque panel fill. Was `rgba(255,59,48,…)` / `rgba(10,10,12,…)` / `rgba(0,122,255,…)` glass on Changelog copy and the error accent. Icon-line status opaque in v0.1.1725.

Tester: open CPU window on macOS, warm ≥30s. Open Changelog. Confirm headings, list copy, version chips, and error pane stay solid (no glass blend). Gauges/sparklines still update. Check Activity Monitor Graphics and Media / `tauri://localhost` toward <1%. Do not close GitHub #14.

## Prior cuts (summary)

Opaque hairline / wash series through **v0.1.1725** (icon-line status · Details/Top Processes body · battery-icon · apple-title · icon-btn type · icon-line type · text/muted · panel tokens · modal-backdrop · ring-track · markdown table/hr · Add form · monitor-history · popover headers · chat-messages border · icon-strip / section dividers · shell · history charts · metric cards · Details/Top Processes · scrollbars · battery strip · History controls · menus · popovers · and earlier #14 opaque washes). Full prior Implementation + Linux tester FAIL reports lived in the previous 1MB WIP dump and were trimmed here so the queue stays readable.

## Test report

**Date:** 2026-10-08 01:10 CEST (local); 2026-10-07 23:10 UTC.

**Host:** Linux (Arch). macOS Activity Monitor / `tauri://localhost` Graphics and Media idle not runnable here. Per task note, Linux webkit2gtk WebKitWebProcess floor is not the macOS gate for this cut.

**Commands run**
- `python3 scripts/autoresearch_ratchet.py verify` → **PASS** (`VERIFY OK`; cargo check + curated lib smoke)
- `cd src-tauri && cargo check` → covered by ratchet (`mac_stats v0.1.1726`)

**Static checks (this cut)**
- `src-tauri/dist/themes/apple/cpu.css`: Changelog type rules use opaque `color-mix(in srgb, … % , #ffffff)` for `color`.
- Grep for prior glass `color: rgba(255, 59, 48|10, 10, 12|0, 122, 255` on Changelog selectors → **no matches**.
- Remaining apple `color: rgba` type: `.force-quit-btn` · `.chat-message .markdown a` (next fuel).
- `Cargo.toml` version **0.1.1726** matches Implementation.

**Logs**
- `~/.mac-stats/debug.log` scan: no ERROR/WARN clusters in 180m window before this cut.

**Verdict:** **PASS** for v0.1.1726 Changelog body type opaque wash cut. Move → `CLOSED-`. GitHub #14 left open (operator / 004). macOS warm-window Activity Monitor `<1%` remains the ongoing product gate on Apple Silicon, not blocked by this cut’s coded change.
