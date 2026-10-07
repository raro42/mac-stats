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

Version **v0.1.1721** (follow-up after v0.1.1720).

Profiler note (Linux webkit2gtk): a blank `cpu.html` still pegs WebKitWebProcess near a full core. That host floor is not the macOS `tauri://localhost` / Graphics and Media gate. Product cuts below still matter on macOS.

Changes (Apple section icon-strip glyph type skips glass blend):

- `src-tauri/dist/themes/apple/cpu.css` — `.icon-btn` resting / hover `color` mix against opaque chip fills (`color-mix(in srgb, #1e1e22 60%, #ececf1)` / `82%` against `#ffffff`). Was `rgba(30, 30, 34, 0.60)` / `0.82` glass under the always-visible Monitors · AI Chat · … strip icons. Icon-line type opaque in v0.1.1720.

Tester: open CPU window on macOS, warm ≥30s. Confirm strip icons stay solid (no glass alpha through to the chip). Gauges/sparklines still update. Check Activity Monitor Graphics and Media / `tauri://localhost` toward <1%. Do not close GitHub #14.

## Prior cuts (summary)

Opaque hairline / wash series through **v0.1.1720** (icon-line type · text/muted · panel tokens · modal-backdrop · ring-track · markdown table/hr · Add form · monitor-history · popover headers · chat-messages border · icon-strip / section dividers · shell · history charts · metric cards · Details/Top Processes · scrollbars · battery strip · History controls · menus · popovers · and earlier #14 opaque washes). Full prior Implementation + Linux tester FAIL reports lived in the previous 1MB WIP dump and were trimmed here so the queue stays readable.

## Test report

**Date:** 2026-10-08 ~00:05 CEST (local) / 2026-10-07 ~22:05 UTC  
**Host:** Linux (Arch), not macOS  

**Result:** FAIL / blocked (macOS gate not verified)

### Commands

| Command | Result |
|---|---|
| `cargo check` in `src-tauri/` | PASS (v0.1.1721; warnings only, unused imports/dead code) |
| `cargo test` in `src-tauri/` | PASS (1359 lib tests; 0 failed) |

### Static (this cut)

- `src-tauri/dist/themes/apple/cpu.css` `.icon-btn` resting color: `color-mix(in srgb, #1e1e22 60%, #ececf1)` — present.
- `.icon-btn:hover` color: `color-mix(in srgb, #1e1e22 82%, #ffffff)` — present.
- No glass `rgba(...)` on `.icon-btn` / `:hover` glyph color (opaque chip-mix as claimed).

### macOS acceptance (required)

- Open CPU window, warm ≥30s, strip icons solid, gauges/sparklines update, Activity Monitor Graphics and Media / `tauri://localhost` toward below 1%: **not run** (Linux host).
- Task notes Linux webkit2gtk blank-`cpu.html` floor is not the macOS gate; product CSS still matters on macOS only.

### Logs

- `~/.mac-stats/debug.log` tail: Ollama localhost refused / circuit-open noise only; nothing tied to this Apple icon-strip CSS cut.

### Outcome

Back to `agents/tasks/WIP-14-…`. Needs a macOS tester pass on Activity Monitor before CLOSED. GitHub #14 left open.
