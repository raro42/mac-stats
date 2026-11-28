# Overnight fuel pick — 2026-11-28 ~01:33 CET

Digester empty · design-review due=false · layout-daily data-poster marked today · Quiet/Empty/Ops first-paint closed (v0.1.3146).

## Chosen experiment

**OpenClaw focus page title on `/browser` · `/cdp` Warm**

## Why

- OpenClaw `browser tabs` lists **title** first (`title` then URL); mac-stats Warm already shows focus index + **page host** (v0.1.3135) but not title.
- `Tab::get_target_info().title` is already used for LLM tab lines; glance snapshot only reads URL host today.
- Not cost/token (runs.jsonl has no tokens — still out of scope).
- Not a Quiet/Empty/Monitors/Disk/Ops first-paint / tracked / owner / last-fail / focus-page-host clone.

## Files

- `src-tauri/src/browser_agent/mod.rs` — `CachedBrowserSessionGlance::Warm` + `snapshot_cached_browser_session`
- `src-tauri/src/commands/harness_ops/status_chips.inc.rs` — format bit + help line
- `src-tauri/src/commands/harness_ops/mod.rs` — chip unit tests
- `src-tauri/src/browser_agent/tests.rs` — title glance helper tests if extracted

## Fitness

Keep if Warm shows a calm truncated focus title when known, and omits blank / New Tab / untitled noise without probing `/json/version`.
