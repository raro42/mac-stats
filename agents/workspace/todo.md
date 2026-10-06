# Coder tick — GitHub #14

- [x] Pick lowest GitHub task: WIP-14 (tauri://localhost CPU)
- [x] Settings Telegram not-set/partial glance: mix against opaque `#ffffff`, drop hover/focus shadows
- [x] Version 0.1.1515 + CHANGELOG + sync-dist + cargo check
- [x] Rename WIP-14 → UNTESTED-14, commit, push, comment (do not close)

## Review

Settings Credentials Telegram not-set/partial glance no longer composites a glass alpha wash or hover drop shadow. Same pattern as Settings Cursor agent (v0.1.1514). `cargo check` in `src-tauri/` passed (existing unused warnings only). Linux cannot measure macOS Graphics and Media; tester should use Activity Monitor on a focused CPU window.
