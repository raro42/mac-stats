# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1400 follow-up)

- [x] Bump UI polls/TTL to 600s; backend/`get_cpu_details`/TEMP to 120s; process cache 600s
- [x] HISTORY_POINTS 2; ring skip ~10%; defer chart-line boot; processes collapsed content-visibility
- [x] GPU warm defer 20s; seed maxDisplayPoints 2; history availability 600s
- [x] `cargo check` in src-tauri/; sync-dist; bump v0.1.1400 + CHANGELOG
- [x] Rename WIP-14 → UNTESTED-14; commit + push; ratchet keep @ 8a5d63b
- [x] Tester claimed TESTING-14 (active)

## Review
Shipped **v0.1.1400**. Ratchet VERIFY OK + keep. #14 stays open until macOS Activity Monitor shows webview under ~1%.
