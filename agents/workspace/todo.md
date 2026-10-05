# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1404)

- [x] Park sparkline canvases on blur / `document.hidden`; unpark on focus
- [x] Skip paints when `!document.hasFocus()` (macOS occluded-but-visible)
- [x] History chart `content-visibility` / `contain`; DPR cap 1; opaque canvases
- [x] Ring gauges skip under ~30%; skip ring/DOM/refresh while occluded
- [x] `cargo check` in src-tauri/; sync-dist; bump v0.1.1404 + CHANGELOG
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
v0.1.1404 compositor/occlusion cut for #14. Linux cannot prove macOS Graphics and Media <1%; leave open for tester / 004.
