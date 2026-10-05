# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1401 follow-up)

- [x] Bump UI polls/TTL to 900s; backend/`get_cpu_details`/TEMP to 180s; process cache 900s
- [x] Focus resume skip recent metrics IPC; ring skip ~15%; wider sparkline deadband; idle boot 4s
- [x] GPU warm defer 30s; TEMP cache max age 270s
- [x] `cargo check` in src-tauri/; sync-dist; bump v0.1.1401 + CHANGELOG
- [x] Rename WIP-14 → UNTESTED-14; commit + push; gh-safe comment (do not close #14)

## Review
v0.1.1401 shipped for #14. Linux cannot prove macOS Graphics and Media <1%; left open for tester / 004.
