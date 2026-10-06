# Coder — #14 v0.1.1435

## Plan
- [x] Park `loadCpuUiSections` retry + clear promise on park; resume re-merge
- [x] Park `hydratePinnedProcessNamesFromDisk` start + mid-flight
- [x] Compact: localStorage on open; Settings Product syncs backend + layout
- [x] Drop duplicate `autoConfigureOllama` from monitoring idle
- [x] Agent Ops: bail `loadCpuUiSections` / `take_open_ui_section` wait while parked
- [x] Bump to v0.1.1435; CHANGELOG; sync-dist
- [x] `cargo check`
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1435** for GitHub #14. Monitoring idle UI-state/pin hydrate park; Compact localStorage open path; no second Ollama configure; Agent Ops wait-loop park. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
