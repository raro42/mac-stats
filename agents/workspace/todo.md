# UNTESTED-14 — tauri://localhost CPU (#14)

## Plan
- [x] Read FEATURE-CODER / agents.md / lessons; pick lowest GitHub FEAT/WIP (WIP-14)
- [x] Convert Agent Ops agent editor focus · dirty → opaque `#ffffff` (v0.1.1616; remote took 1615 for AI Chat empty)
- [x] Sync dist; CHANGELOG; cargo check
- [x] Rename WIP- → UNTESTED-; rebase onto remote 1615
- [x] Commit + push origin/main; GitHub comment via gh-safe (issue left open)

## Review
- **v0.1.1616** — Agent Ops agent editor (`textarea.ops-agent-editor` focus · dirty) mixes washes against opaque `#ffffff`. No glass alpha on focus ring / dirty border. Continues #14 WebView compositor cuts. Issue left open for tester / 004.
