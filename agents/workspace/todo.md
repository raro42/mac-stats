# TESTING-14 → WIP-14 (v0.1.1678)

- [x] Read TESTER.md + lessons.md
- [x] Verify claimed v0.1.1678 Apple AI Chat composer shell opaque wash
- [x] `cargo check` in `src-tauri/` — pass
- [x] `cargo test` in `src-tauri/` — pass (1359 lib)
- [x] Static CSS check on `.chat-input-container` / `:focus-within`
- [x] `scan_debug_log_errors.py --minutes 180` — clean
- [x] Append Test report (v0.1.1678); move TESTING-14 → WIP-14
- [x] Do **not** close GitHub #14

## Review

v0.1.1678 static + cargo pass on Linux. Cannot measure macOS Graphics and Media / `tauri://localhost` <1% from this host → WIP (same gate as v0.1.1677). Needs macOS Activity Monitor pass before CLOSED.
