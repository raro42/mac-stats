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

Version **v0.1.1720** (follow-up after v0.1.1719).

Profiler note (Linux webkit2gtk): a blank `cpu.html` still pegs WebKitWebProcess near a full core. That host floor is not the macOS `tauri://localhost` / Graphics and Media gate. Product cuts below still matter on macOS.

Changes (Apple icon-line strip glyph type skips glass blend):

- `src-tauri/dist/themes/apple/cpu.css` — `.icon-line-item` resting / hover `color` mix against opaque chip fills (`color-mix(in srgb, #0c0c10 42%, #ffffff)` / `72%` against `#ececf1`). Was `rgba(12, 12, 16, 0.42)` / `0.72` glass under the always-visible Monitors · AI Chat · … strip glyphs. Primary / muted type tokens opaque in v0.1.1719.

Tester: open CPU window on macOS, warm ≥30s. Confirm icon-line strip glyphs stay solid (no glass alpha through to the chip). Gauges/sparklines still update. Check Activity Monitor Graphics and Media / `tauri://localhost` toward <1%. Do not close GitHub #14.

## Prior cuts (summary)

Opaque hairline / wash series through **v0.1.1719** (text/muted · panel tokens · modal-backdrop · ring-track · markdown table/hr · Add form · monitor-history · popover headers · chat-messages border · icon-strip / section dividers · shell · history charts · metric cards · Details/Top Processes · scrollbars · battery strip · History controls · menus · popovers · and earlier #14 opaque washes). Full prior Implementation + Linux tester FAIL reports lived in the previous 1MB WIP dump and were trimmed here so the queue stays readable.

## Test report

- **Date:** 2026-10-07 23:59 CEST (local); UTC 2026-10-07 21:59
- **Host:** Linux (Arch) — no macOS Activity Monitor / `tauri://localhost` Graphics and Media sample possible
- **Commands run:** `cd src-tauri && cargo check`; `cd src-tauri && cargo test`
- **Static check:** `.icon-line-item` resting `color: color-mix(in srgb, #0c0c10 42%, #ffffff)`; hover `color-mix(in srgb, #0c0c10 72%, #ececf1)` against opaque chip fills — matches Implementation (v0.1.1720)
- **debug.log:** no errors related to this CSS cut (stale Ollama connection refused only; unrelated)

| Step | Command / check | Result |
|------|-----------------|--------|
| Check | `cargo check` | **pass** (warnings only; preexisting unused items) |
| Tests | `cargo test` | **pass** — 1359 passed; 0 failed; 1 ignored (doc-test) |
| CSS cut present | apple `cpu.css` icon-line resting/hover opaque mix | **pass** |
| macOS warm ≥30s UI | open CPU window; solid strip glyphs; gauges/sparklines | **blocked** — Linux host |
| Activity Monitor gate | Graphics and Media / `tauri://localhost` toward &lt;1% | **blocked** — Linux host; webkit2gtk floor is not this gate (see Profiler note) |

- **Outcome:** **fail / blocked** — build and static CSS verification pass, but acceptance criteria require macOS Activity Monitor confirmation. Returning to `WIP-` for a macOS tester. GitHub #14 left open.
