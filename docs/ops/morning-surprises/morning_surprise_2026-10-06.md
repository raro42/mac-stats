# Morning surprise — 2026-10-06

Overnight autoresearch kept shipping against GitHub **#14** (tauri://localhost CPU) and Ollama-down **debug.log** spam. Digester open was empty; design review stayed in grace.

## Shipped tonight

| Version | What |
|--------|------|
| **v0.1.1397** | Circuit opened WARN **≤1/5min**; model-list fail cooldown **5m**; shared `/api/tags` waiters log once. UI polls/TTL **120s**; backend/`get_cpu_details` **45s**; sparklines **4** points; Debug Log **120s** + blur pause |
| **v0.1.1396** | Metrics/history **90s**; process cache **90s**; `get_cpu_details` floor **30s**; backend **30s**; sparklines **6** points; Debug Log **60s** |
| **v0.1.1394** | Process cache **60s**; polls **60s**; backend **20s**; sparklines **8** |
| **v0.1.1393** | Metrics/history **45s**; backend loop **15s**; sparklines **12** + cached opaque backdrop; collapsed Top Processes glance-only |
| **v0.1.1390–1392** | Earlier idle/IPC cuts (polls, sparkline points, opaque chrome) |
| **v0.1.1388** | Ollama model-list fail cooldown + WARN ≤1/5min (first log-012 pass) |

## Still open

- GitHub **#14** needs a **macOS** Activity Monitor pass (`tauri://localhost` / Graphics and Media under ~1%). Linux webkit2gtk still has a high blank-page floor, so this rack cannot CLOSE the issue alone.

## Not a surprise (skipped)

- Empty digester alone — standing backlog / #14 / log-012 used instead.
