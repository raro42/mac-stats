# Dead-code inventory (`#[allow(dead_code)]`)

Audit date: 2026-07-22. Counts are approximate; prefer `rg 'allow\\(dead_code\\)' src-tauri/src`.

## Hotspots

| Area | Approx allows | Disposition |
|------|--------------:|-------------|
| `ffi/ioreport.rs`, `ffi/objc.rs` | ~19 | **Keep** — FFI surface for future / private API stubs |
| `alerts/channels.rs` (+ alerts/mod) | ~13+ | **Keep until** Alert channels UI is ported from dashboard (`docs/042_dashboard_orphan.md`); then wire or delete |
| `state.rs` | was 6 | Removed unused `ACCESS_CACHE` (2026-07-22). Remaining: `AppState` scaffold, `M3_FREQ_KEY`, `LAST_POWER_READ` / `LAST_BATTERY_READ` reserved rate-limit stamps — **keep or wire** in a metrics pass |
| `ollama/mod.rs` | ~7 | **Keep** — client helpers reserved for future endpoints |
| `commands/outbound_pipeline.rs` | crate-level | **Keep** — partially wired (Discord/UI); crate allow covers unused surface helpers (`BreakPreference` menu bar, etc.). Tighten allows later when menu-bar surface lands |
| `monitors/social.rs`, `mcp/mod.rs` | few | **Keep** — optional integrations |
| `logging/legacy.rs` | few | Shrink when legacy log bridge unused |

## Clear unused fixed this pass

- Deleted `state::ACCESS_CACHE` — zero readers after Phase 3 OnceLock split

## Do not mass-delete

- Alert channel structs (Telegram/Slack/Mastodon/Signal) — dead in UI but intentional API for future Settings port
- FFI `allow(dead_code)` — private Apple APIs; deleting breaks future work

## Next clippy pass

```bash
cd src-tauri && cargo clippy -- -W dead-code 2>&1 | head -80
```

Only remove items with zero call-graph hits and no documented “future surface” comment.
