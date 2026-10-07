# UNTESTED-14 — tauri://localhost CPU (#14)

## Plan
- [x] Read FEATURE-CODER / agents.md / lessons; pick lowest GitHub FEAT/WIP (WIP-14)
- [x] Convert AI Chat message rows → opaque `#ffffff` (v0.1.1619)
- [x] Sync dist; CHANGELOG; cargo check
- [x] Rename WIP- → UNTESTED-
- [x] Commit + push origin/main; GitHub comment via gh-safe (issue left open)

## Review
- **v0.1.1619** — AI Chat message rows (hover · focus · selected · Copied badge) mix washes against opaque `#ffffff`. No glass alpha on the row background, focus ring, or Copied badge. Continues #14 WebView compositor cuts. Issue left open for tester / 004.
