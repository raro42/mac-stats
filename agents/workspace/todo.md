# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1414)

- [x] Stop parse-time `loadCpuUiSections()` IPC (localStorage seed only; backend on demand)
- [x] Slow `get_cpu_window_ui_state` retry (50ms → 500ms)
- [x] Defer `initMonitoringFeatures` (100ms → idle ≤120s); idempotent
- [x] Focus-gate version/update IPC + DOM wire (no 60s/120s idle wake); late 10m fallback
- [x] Defer `initRingGauges` into DOM wire
- [x] Defer Agent Ops init + slow open-section retries
- [x] Bump 0.1.1414, CHANGELOG, sync-dist
- [x] cargo check; rename WIP→UNTESTED
- [x] commit + push origin/main

## Review
Shipped `566bec58` v0.1.1414. Task at `UNTESTED-14-…`. Issue #14 left open.
