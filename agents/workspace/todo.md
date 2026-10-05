# Session todo — GitHub #14 tauri://localhost CPU (follow-up #12)

- [x] Stop Agent Ops collapsed-glance IPC while the pane is icon-hidden
- [x] Align UI polls/TTL to 180s; backend/`get_cpu_details`/TEMP 60s; process cache 180s
- [x] Sparklines: 2 points; HISTORY_POINTS 8; seed maxDisplayPoints 4; skip draws when hidden
- [x] GPU sampler warm defer 8s; seed skip on focus if last seed <180s
- [x] Settings modal: content-visibility hidden when closed (apple/light/dark)
- [x] `cargo check` in src-tauri/; bump v0.1.1398 + CHANGELOG; sync-dist
- [x] Rename WIP-14 → UNTESTED-14; commit + push; gh-safe comment (do not close #14)

## Review
v0.1.1398: 180s UI polls, no Agent Ops glance IPC while icon-hidden, 60s backend/`get_cpu_details`, 2 sparkline points, skip canvas when document.hidden, closed settings content-visibility, GPU warm deferred 8s. macOS Activity Monitor remains the <1% acceptance gate.
