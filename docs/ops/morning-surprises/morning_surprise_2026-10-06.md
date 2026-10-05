# Morning surprise — 2026-10-06

Overnight autoresearch kept shipping against GitHub **#14** (tauri://localhost CPU) and earlier Ollama-down **debug.log** spam. Digester open was empty; design review stayed in grace.

## Shipped tonight

| Version | What |
|--------|------|
| **v0.1.1408** | Tauri `WindowEvent::Focused` parks / resumes idle polls; sparkline boot skips history IPC seed; ring skip ~**70%**; chart-line boot idle **180s**; GPU warm **1800s** |
| **v0.1.1407** | Occlusion park uses `display: none` on shells / `body`; ring skip ~**60%**; boot idle **120s**; GPU warm **960s** |
| **v0.1.1406** | Shell park via `visibility` + `content-visibility`; document root park; ring skip ~**50%**; boot idle **60s**; GPU warm **480s** |
| **v0.1.1405** | `html.is-occluded` parks heavy trees; ring skip ~**40%**; boot idle **30s**; GPU warm **240s** |
| **v0.1.1404** | Park sparkline canvases (1×1) on blur / `document.hidden`; skip paints when `!hasFocus()`; history `content-visibility: auto` + `contain: paint`; data-poster DPR 1 + opaque canvas; ring skip ~**30%** |
| **v0.1.1403** | UI polls/TTL **3600s**; backend/`get_cpu_details`/temp **600s**; process cache **3600s**; ring skip ~**25%**; GPU warm **120s** |
| **v0.1.1402** | UI polls/TTL **1800s**; backend **300s**; GPU warm **60s** |
| **v0.1.1401** | UI polls/TTL **900s**; backend **180s**; GPU warm **30s** |
| **v0.1.1400** | UI polls/TTL **600s**; backend/`get_cpu_details`/temp **120s**; process cache **600s**; HISTORY_POINTS **2**; ring skip ~**10%**; chart-line boot via `requestIdleCallback`; collapsed Top Processes `content-visibility`; GPU warm **20s** |
| **v0.1.1399** | UI polls/TTL **300s**; backend/`get_cpu_details` **90s**; sparklines **2** points; skip ring/DOM rAF when hidden; collapsed keep-header `content-visibility`; GPU warm **12s** |
| **v0.1.1398** | UI polls **180s**; no Agent Ops glance IPC while icon-hidden; backend **60s**; sparklines **2** points |
| **v0.1.1397** | Circuit opened WARN **≤1/5min**; model-list fail cooldown **5m**; shared `/api/tags` waiters log once. UI polls/TTL **120s**; backend/`get_cpu_details` **45s**; sparklines **4** points |
| **v0.1.1396** | Metrics/history **90s**; process cache **90s**; `get_cpu_details` floor **30s**; backend **30s**; sparklines **6** points |
| **v0.1.1394–1393** | Process cache / poll / backend / sparkline cuts; collapsed Top Processes glance-only |
| **v0.1.1390–1392** | Earlier idle/IPC cuts (polls, sparkline points, opaque chrome) |
| **v0.1.1388** | Ollama model-list fail cooldown + WARN ≤1/5min (first log-012 pass) |

## Still open

- GitHub **#14** needs a **macOS** Activity Monitor pass (`tauri://localhost` / Graphics and Media under ~1%). Linux webkit2gtk still has a high blank-page floor, so this rack cannot CLOSE the issue alone. Best new lever tonight: Tauri focus-driven park + no boot history seed (v0.1.1408).

## Not a surprise (skipped)

- Empty digester alone — standing backlog / #14 used instead.
