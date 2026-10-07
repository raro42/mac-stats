# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1706)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1705 (battery strip)
- [x] Next glass cut: Apple Details / Top Processes scrollbar thumbs (still `rgba` glass)
- [x] Bump `Cargo.toml` → `0.1.1706`
- [x] Opaque `#ffffff` mix for `.apple-details` / `.apple-processes` scrollbar thumbs resting · hover
- [x] CHANGELOG `[0.1.1706]` entry
- [x] `cargo check` / ratchet verify
- [x] Commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.apple-details` / `.apple-processes` `::-webkit-scrollbar-thumb` resting · hover opaque mix (was `rgba(0,0,0,0.15/0.25)` glass).
- Note: v0.1.1705 commit message on origin mislabeled scrollbar; tree content is battery strip (sibling race). This tick ships the intended scrollbar cut as 1706.
- Changelog scrollbar thumbs still glass (next fuel).
