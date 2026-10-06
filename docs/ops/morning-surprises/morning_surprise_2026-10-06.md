# Morning surprise — 2026-10-06

Overnight Track B kept shipping GitHub #14 (CPU window WebView idle), plus a correctness fix so idle defer does not strand section wiring.

## Shipped tonight
- **v0.1.1416** — Focused-open again schedules monitoring features (idle ≤300s). v0.1.1415 left that to Focused/resume or the 10m late fallback; when Focused(true) races past load, Monitors / AI Chat / Disk Cleanup could stay unwired. Metrics stay idle ≤30s.
- **v0.1.1415** — First `get_cpu_details` idle ≤30s (was 5s). Focus resume no longer forces a full process-list rebuild. Version/update IPC idle ≤120s after metrics. Monitoring / Agent Ops idle ≤300s.
- **v0.1.1414 / 1413 / 1412** — Deferred open IPC, focus-gated first metrics, occluded deferred-init correctness.
- **v0.1.1411–1408** — Longer idle defer, Focused park/resume, canvas occlusion park.

## Tried / still open
- Digester open stayed empty. Design review still in grace (feature-agent-ops screenshot stale).
- Debug.log: quiet in the last 180m (earlier Ollama-down noise already rate-limited in v0.1.1397).
- #14 still open until macOS Activity Monitor shows tauri://localhost / Graphics and Media under ~1% with the CPU window open and after alt-tab.

## Why this is a surprise
Not a quiet digester night. Product code moved the #14 ratchet again, including a fix so section wiring still arms when Focused races past load.
