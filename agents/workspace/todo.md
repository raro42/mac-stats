# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1408)

- [x] Tauri `WindowEvent::Focused` → park/resume idle polls (macOS + Linux)
- [x] Skip sparkline history IPC seed on chart-line boot; boot idle 180s
- [x] Ring gauges skip paints under ~70%; GPU warm defer 1800s
- [x] `cargo check` / ratchet verify; sync-dist; bump v0.1.1408 + CHANGELOG
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
v0.1.1408 structural occlusion park for #14 (Focused event + no boot history seed). Linux cannot prove macOS Graphics and Media <1%; leave open for tester / 004.
