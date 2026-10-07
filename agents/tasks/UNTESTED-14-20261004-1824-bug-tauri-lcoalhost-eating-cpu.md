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

Version **v0.1.1723** (follow-up after v0.1.1722).

Profiler note (Linux webkit2gtk): a blank `cpu.html` still pegs WebKitWebProcess near a full core. That host floor is not the macOS `tauri://localhost` / Graphics and Media gate. Product cuts below still matter on macOS.

Changes (Apple battery strip glyph type skips glass blend):

- `src-tauri/dist/themes/apple/cpu.css` — `.battery-icon` resting / charging `color` mix against opaque strip fill (`color-mix(in srgb, #0c0c10 50%, #ececf1)` / `#34c759 80%` against `#ececf1`). Was `rgba(12, 12, 16, 0.50)` / `rgba(52, 199, 89, 0.8)` glass under the always-visible Bat icon. Window title opaque in v0.1.1722.

Tester: open CPU window on macOS, warm ≥30s. Confirm Bat glyph stays solid (resting and charging). Gauges/sparklines still update. Check Activity Monitor Graphics and Media / `tauri://localhost` toward <1%. Do not close GitHub #14.

## Prior cuts (summary)

Opaque hairline / wash series through **v0.1.1722** (apple-title · icon-btn type · icon-line type · text/muted · panel tokens · modal-backdrop · ring-track · markdown table/hr · Add form · monitor-history · popover headers · chat-messages border · icon-strip / section dividers · shell · history charts · metric cards · Details/Top Processes · scrollbars · battery strip · History controls · menus · popovers · and earlier #14 opaque washes). Full prior Implementation + Linux tester FAIL reports lived in the previous 1MB WIP dump and were trimmed here so the queue stays readable.

## Test report

**Date:** 2026-10-08 ~00:15 CEST (local) / 2026-10-07 ~22:15 UTC  
**Host:** Linux (Arch), not macOS

**Result:** FAIL / blocked (macOS gate not verified)

### Commands

| Command | Result |
|---|---|
| `cargo check` in `src-tauri/` | PASS (v0.1.1723; warnings only, unused imports/dead code) |
| `cargo test` in `src-tauri/` | PASS (1359 lib tests; 0 failed; 1 doc-test ignored) |

### Static (this cut)

- `src-tauri/dist/themes/apple/cpu.css` `.battery-icon` resting color: `color-mix(in srgb, #0c0c10 50%, #ececf1)` — present.
- `.battery-icon.charging` color: `color-mix(in srgb, #34c759 80%, #ececf1)` — present.
- No glass `rgba(12, 12, 16, …)` / `rgba(52, 199, 89, …)` on Bat glyph colors (opaque strip-mix as claimed).
- Prior cut still present: `.apple-title h1` opaque `color-mix(in srgb, #1c1c1e 92%, #f2f2f6)`.

### macOS acceptance (required)

- Open CPU window, warm ≥30s, Bat glyph solid (resting/charging), gauges/sparklines update, Activity Monitor Graphics and Media / `tauri://localhost` toward below 1%: **not run** (Linux host).
- Task notes Linux webkit2gtk blank-`cpu.html` floor is not the macOS gate; product CSS still matters on macOS only.

### Logs

- `~/.mac-stats/debug.log` tail: Ollama localhost refused / circuit-open noise only; nothing tied to this Apple battery-glyph CSS cut. No secrets pasted.

### Outcome

Back to `agents/tasks/WIP-14-…`. Needs a macOS tester pass on Activity Monitor before CLOSED. GitHub #14 left open.

