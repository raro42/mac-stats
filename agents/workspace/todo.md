# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1402)

- [x] Bump UI polls/TTL to 1800s; backend/`get_cpu_details`/TEMP to 300s; process cache 1800s
- [x] Ring skip ~20%; wider sparkline deadband; idle boot 8s; GPU warm defer 60s; TEMP cache max age 450s
- [x] Skip clearing process cache + rate limiter on window open (reuse warm cache; less open spike)
- [x] `cargo check` in src-tauri/; sync-dist; bump v0.1.1402 + CHANGELOG
- [ ] Rename WIP-14 → UNTESTED-14; commit + push; gh-safe comment (do not close #14)

## Review
v0.1.1402 ready for tester. Linux cannot prove macOS Graphics and Media <1%; leave #14 open.
