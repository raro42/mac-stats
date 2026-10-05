# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1405)

- [x] Toggle `html.is-occluded` on blur/hidden; CSS parks heavy paint trees
- [x] Ring gauges skip paints under ~40%; canvases hide layer on park
- [x] chart-line boot idle timeout 30s; history park visibility
- [x] GPU warm defer 240s
- [x] `cargo check` in src-tauri/; sync-dist; bump v0.1.1405 + CHANGELOG
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
v0.1.1405 compositor/occlusion park for #14. Linux cannot prove macOS Graphics and Media <1%; leave open for tester / 004.
