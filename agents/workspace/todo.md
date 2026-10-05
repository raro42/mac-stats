# Session todo — GitHub #14 tauri://localhost CPU (follow-up)

- [x] Pick lowest GitHub task (WIP-14 / issue 14)
- [x] Profile: blank HTML still ~100% WebKit on Linux webkit2gtk (host compositor floor)
- [x] Kill ring CSS stroke-dashoffset transitions (apple/architect + agent-ops override)
- [x] Flatten apple theme glass gradients to opaque fills; smaller card shadows
- [x] content-visibility on collapsed section bodies
- [x] Metrics poll 5s; Discord icon 30s; pause Discord when hidden
- [x] Opaque window background_color + BackgroundThrottlingPolicy::Suspend
- [x] cargo check in src-tauri/
- [x] CHANGELOG + version bump v0.1.1387
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit and push origin/main (do not close the GitHub issue)

## Review

Tester FAIL on v0.1.1356/1386 left WebView hot. This pass targets remaining compositor work (ring CSS tween, layered glass paint, idle polls, WKWebView background throttling). Linux blank-page A/B still pegs WebKit ~100%, so macOS Activity Monitor is the real gate for under ~1%.
