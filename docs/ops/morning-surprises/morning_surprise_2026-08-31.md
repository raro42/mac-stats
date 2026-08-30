# Morning surprise — 2026-08-31

Overnight Track B (mac-stats autoresearch). Digester open stayed empty; design review under grace — still shipped visible polish.

## Shipped

| Version | What |
|---------|------|
| **v0.1.746** | **Agent Ops Runs Fail/Slow glance** — attention strip under Refresh when recent runs Fail or Slow (≥2000 ms); click opens Runs Fail (prefer) or Slow filter (AI Chat Errors parity; `feature-agent-ops`). |
| **v0.1.745** | **AI Chat Errors glance** — failed-turn strip + Last error wash; click → Errors filter (`feature-ai-chat`). |
| **v0.1.744** | **`/voice` · `/stt` instant** — Discord voice STT Ready / Partial / Not set (config only). |
| **v0.1.743** | **`/having_fun` · `/fun` · `/idle` instant** — idle-thoughts On/Off chip. |

## Tried / notes

- Digester: no open Slowest / candidates this window.
- Screenshot recapture for stale feature screens deferred (TCC / CPU window not captured this tick).
- Idle-thought Discord send timeouts still rate-limited in `debug.log` — retry from v0.1.703; `/having_fun` for config glance.

## Fitness

Operators see Fail/Slow attention without opening the Runs tab first — same glance pattern as AI Chat Errors and Debug Log.
