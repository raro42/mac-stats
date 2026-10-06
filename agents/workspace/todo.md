# Coder — #14 v0.1.1465

## Plan
- [x] Opaque CPU/GPU/Freq/Temp ring card status washes (no glass alpha)
- [x] Mix against opaque card fill; drop ring box-shadow
- [x] Bump v0.1.1465; CHANGELOG; standing backlog
- [x] `cargo check` in src-tauri; sync dist
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1465** for GitHub #14. CPU, GPU, Freq, and Temp ring card hot / calm / Fair washes no longer keep a glass compositor blend on the open CPU window. Issue left open for tester / 004 (macOS Activity Monitor).
