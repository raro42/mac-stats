# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1407)

- [x] Stronger occlusion park: `display: none` on parked trees + freeze will-change/transform
- [x] Ring gauges skip paints under ~60%; chart-line boot idle 120s; wider sample deadband
- [x] GPU warm defer 960s; park canvases with `display: none`
- [x] `cargo check` in src-tauri/; sync-dist; bump v0.1.1407 + CHANGELOG
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
v0.1.1407 stronger compositor/occlusion park for #14 (`display: none` body park, ring 60%, boot 120s, GPU warm 960s). Linux cannot prove macOS Graphics and Media <1%; leave open for tester / 004.
