# Morning surprise — 2026-10-06

Overnight Track B kept shipping GitHub #14 (CPU window WebView idle).

## Shipped tonight
- **v0.1.1412** — Deferred open wiring no longer bails when the window is occluded. Keyboard/copy/strip always bind; metrics always arm. refresh() still no-ops while occluded. Fixes stuck UI after alt-tab-on-open.
- **v0.1.1411** — Longer idle defer (120s metrics/version IPC; 60s DOM wiring). Sparklines stay parked through open (late idle unpark 120s). Ring skip ~99%. Hot-wash classList churn skipped when signature unchanged. Wider sparkline deadband. CSS freezes mix-blend-mode + metric transforms.
- **v0.1.1410** — No forced process refresh on open; 30s idle IPC; parked sparklines; ring skip ~95%; no AGX GPU sampler warm on open.
- **v0.1.1409 / 1408** — Focused park/resume, CSS freeze, history IPC skip, deeper occlusion park.

## Tried / still open
- Digester open stayed empty. Design review still in grace (feature-agent-ops screenshot stale).
- Debug.log: Ollama-down model_cache / circuit-open noise (already rate-limited in v0.1.1397).
- #14 still open until macOS Activity Monitor shows tauri://localhost / Graphics and Media under ~1% with the CPU window open and after alt-tab.

## Why this is a surprise
Not a quiet digester night. Product code moved the #14 ratchet again, including a correctness fix so idle defer does not brick controls when open starts unfocused.
