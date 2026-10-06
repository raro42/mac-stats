# Morning surprise — 2026-10-06

Overnight Track B kept shipping GitHub #14 (CPU window WebView idle), including a focus-resume race that undid deferred sparkline unpark.

## Shipped tonight
- **v0.1.1420** — Focus resume no longer unparks sparklines or restarts Discord / logs / history / Disk Cleanup / Agent Ops / Monitors polls on the focus event. Those wait for idle (≤30s), matching chart-line unpark. Blur cancels a pending idle resume so alt-tab churn does not stack GPU + secondary IPC with gauges.
- **v0.1.1419** — First `get_cpu_details` idle ≤120s; sparkline unpark after first poll ≤120s; chart-line focus unpark ≤30s; monitoring / Agent Ops ≤900s.
- **v0.1.1418 / 1416 / 1415** — Further open-path defer, focused-open monitoring restore, first metrics idle ≤30s.
- **v0.1.1414–1408** — Deferred open IPC, focus-gated metrics, Focused park/resume, canvas occlusion park.

## Tried / still open
- Digester open stayed empty. Design review still in grace (feature-agent-ops screenshot stale ~20d).
- Debug.log: no ERROR/WARN clusters in the last 180m.
- #14 still open until macOS Activity Monitor shows tauri://localhost / Graphics and Media under ~1% with the CPU window open and after alt-tab.

## Why this is a surprise
Not a quiet digester night. Product code moved the #14 ratchet again: focus resume was still instantly unparking canvas and restarting section polls, undoing the deferred unpark path.
