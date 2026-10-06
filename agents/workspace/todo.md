# WIP-14 — next compositor cut (v0.1.1580)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1579: AI Chat message Copied flash (`.chat-message[role='option'|'button'].is-just-copied`) still mixed against transparent
- [x] Add opaque green wash (`#ffffff` mix), `box-shadow: none`
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1580`; CHANGELOG entry; standing_backlog note
- [x] Prepend Implementation notes on task file; rename WIP → UNTESTED
- [x] `cargo check` / ratchet verify
- [ ] Commit + push `origin/main`; do not close #14

## Review
Opaque wash on AI Chat message Copied flash. Same pattern as Debug Log / Perplexity / process-row. Issue #14 left open for tester / 004.
