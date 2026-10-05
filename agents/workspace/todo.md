# Session todo — GitHub #14 tauri://localhost CPU (follow-up #4)

- [x] Pick lowest GitHub task (WIP-14 / issue 14)
- [x] Pause idle polls on window `blur` (macOS stays `visible` when occluded)
- [x] Opaque Apple primary surfaces (no rgba panel blend); drop ring `drop-shadow` filters
- [x] Stroke-only sparklines (no area fill); metrics poll 12s; process list 30s
- [x] Lazy-load marked/hljs (remove always-on CDN from Apple theme head)
- [x] content-visibility collapsed sections on remaining themes
- [x] Sync dist, bump version, CHANGELOG
- [x] `cargo check` in src-tauri/
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit and push origin/main (do not close GitHub #14)

## Review

v0.1.1389 cuts remaining Apple compositor chrome (gradient mask, soft shadows, rgba panels), pauses polls on blur, slows metrics to 12s, stroke-only sparklines, and lazy-loads markdown libs. Linux webkit2gtk blank-page floor remains; macOS Activity Monitor is the acceptance gate.
