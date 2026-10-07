# WIP-14 — tauri://localhost CPU (#14)

## Plan
- [x] Read FEATURE-CODER / agents.md / lessons; pick lowest GitHub FEAT/WIP (WIP-14)
- [x] Convert AI Chat error bubbles → opaque `#ffffff` (v0.1.1622)
- [x] Convert AI Chat exec / answer cards (`.chat-exec-card` · `.chat-exec-code` · `.chat-answer-part` · final) → opaque `#ffffff` (v0.1.1621)
- [x] Sync dist; CHANGELOG; cargo check
- [x] Rename WIP- → UNTESTED-
- [x] Commit + push origin/main; GitHub comment via gh-safe (issue left open)

## Review
- **v0.1.1622** — AI Chat error bubbles (`.chat-message.assistant.is-error`) mix the wash against opaque `#ffffff`. No glass alpha on the error row background or border. Continues #14 WebView compositor cuts. Issue left open for tester / 004.
