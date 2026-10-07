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

Version **v0.1.1718** (follow-up after v0.1.1717).

Profiler note (Linux webkit2gtk): a blank `cpu.html` still pegs WebKitWebProcess near a full core. That host floor is not the macOS `tauri://localhost` / Graphics and Media gate. Product cuts below still matter on macOS.

Changes (Apple modal / popover dimmers skip glass blend):

- `src-tauri/dist/themes/apple/cpu.css` — `--modal-backdrop` mixes against opaque body `#f2f2f6` (`color-mix(in srgb, #000000 30%, #f2f2f6)`). Was `rgba(0, 0, 0, 0.30)` glass under Settings · Monitors settings · AI Chat settings full-screen dimmers (`.settings-modal` · `.monitors-settings-popover` · `.ollama-settings-popover`). Ring tracks opaque in v0.1.1717.

Tester: open CPU window on macOS, open Settings and/or Monitors settings and/or AI Chat settings, warm ≥30s. Confirm the dimmer stays solid (no glass alpha through to the shell). Gauges/sparklines still update. Check Activity Monitor Graphics and Media / `tauri://localhost` toward <1%. Do not close GitHub #14.

## Prior cuts (summary)

Opaque hairline / wash series through **v0.1.1717** (ring-track · markdown table/hr · Add form · monitor-history · popover headers · chat-messages border · icon-strip / section dividers · shell · history charts · metric cards · Details/Top Processes · scrollbars · battery strip · History controls · menus · popovers · and earlier #14 opaque washes). Full prior Implementation + Linux tester FAIL reports lived in the previous 1MB WIP dump and were trimmed here so the queue stays readable.

## Test report

- **Date:** 2026-10-07 23:36 CEST (21:36 UTC)
- **Host:** Linux (webkit2gtk floor is not the macOS Graphics and Media gate; see Implementation note)
- **Result:** **pass** for v0.1.1718 Apple modal-backdrop opaque cut
- **GitHub:** issue #14 left open (do not close)

### Commands

- `cargo check` in `src-tauri/` — **pass** (`mac_stats` v0.1.1718; existing unused-code warnings only)
- `cargo test` in `src-tauri/` — **pass** (1359 lib tests passed; 0 failed; 1 doc-test ignored)

### Static checks (this cut)

- `src-tauri/dist/themes/apple/cpu.css`: `--modal-backdrop: color-mix(in srgb, #000000 30%, #f2f2f6)` (opaque mix against body `#f2f2f6`, not glass `rgba`)
- `.settings-modal`, `.monitors-settings-popover`, `.ollama-settings-popover` all use `background: var(--modal-backdrop)`
- `rg 'rgba\(0,\s*0,\s*0,\s*0\.3'` on that theme file — **no matches**
- `~/.mac-stats/debug.log` — no errors tied to this CSS cut (stale Ollama localhost connection-refused noise from 2026-10-05 only)

### Not run on this host

- Live macOS CPU window: open Settings / Monitors settings / AI Chat settings, warm ≥30s, Activity Monitor Graphics and Media / `tauri://localhost` toward <1%
- Product gate for #14 remains operator/macOS; this run only verifies the claimed v0.1.1718 theme cut + build/tests

### Outcome

Moved `agents/testing/active/TESTING-14-…` → `agents/tasks/CLOSED-14-…`. Did not close GitHub issue 14.
