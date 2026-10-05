# Morning surprise — 2026-10-06

Overnight autoresearch kept shipping against GitHub **#14** (tauri://localhost CPU). Digester open was empty; design review stayed in grace.

## Shipped tonight

| Version | What |
|--------|------|
| **v0.1.1393** | Metrics/history **45s**; backend loop **15s**; sparklines **12** points + cached opaque backdrop; collapsed Top Processes → glance chips only; Debug Log auto-refresh **10s**; Apple/Light/Dark opaque action + power-strip chrome |
| **v0.1.1392** | 30s polls; 12s backend; 16 sparkline points; warn/error-only tauri-logger; opacity-only icon imgs |
| **v0.1.1391** | 20s polls; 8s backend; 24 sparkline points; Light/Dark flat chrome; digest test race fix |
| **v0.1.1390** | 15s polls; 5s backend while open; 36 sparkline points; age-token matcher fix |
| **v0.1.1388** | Ollama model-list fail cooldown + WARN ≤1/5min; earlier WebView idle cuts |

## Still open

- GitHub **#14** needs a **macOS** Activity Monitor pass (`tauri://localhost` / Graphics and Media under ~1%). Linux webkit2gtk still has a high blank-page floor, so this rack cannot CLOSE the issue alone.

## Not a surprise (skipped)

- Empty digester alone — standing backlog / #14 used instead.
