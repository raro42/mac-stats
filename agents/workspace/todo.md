# Session todo — WIP-14 tauri://localhost CPU (#14)

GitHub WIP-14 is the lowest-numbered open GitHub task. Continue WebView idle cuts (v0.1.1424 after v0.1.1423).

- [x] Structural occlusion cancel for open-path metrics / late fallback / sparkline unpark
- [x] Keep 1423 idle timeout floor (no further doubling)
- [x] CHANGELOG + bump `0.1.1424`
- [x] sync-dist + `cargo check` in `src-tauri/`
- [x] Update WIP-14 implementation notes; rename → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Do **not** close GitHub #14

## Review
Shipped **v0.1.1424** #14 follow-up: blur/pause cancels pending open-path first-metrics and late-open idle handles; occluded open-path arms bail; focus re-schedules if never armed; skip post-IPC DOM/paint while occluded; chart-line cancels pending sparkline unpark on park. `cargo check` pass. Left as UNTESTED for macOS Activity Monitor pass. Issue #14 stays open.
