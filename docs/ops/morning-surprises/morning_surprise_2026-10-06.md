# Morning surprise — 2026-10-06

Overnight Track B kept cutting CPU-window WebView idle work for GitHub #14, and fixed operator “age” matching so Linux `cargo test` harness_ops goes green again.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1390** | Metrics / history poll **15s**. Backend metric loop stays **5s** while the CPU window is open (was 2s). Sparklines keep **36** points. Apple / Light use flat opaque panels + CSS `contain`. Whole-token `age` (not `average` / `usage` / `agent`). Empty digest-open still says **0 open candidates**. |
| **v0.1.1389** | Stroke-only sparklines, 12s polls, blur pauses idle polls, lazy marked/hljs, Apple opaque chrome. |
| **v0.1.1388** | Opaque macOS window, 8s polls, pause more when hidden, Ollama model-list WARN cooldown. |

## Still open

- GitHub **#14** — macOS Activity Monitor `tauri://localhost` / Graphics and Media under ~1% (Linux webkit2gtk blank-page floor is not that gate). Tester has TESTING-14.
- Design-review screenshot for `feature-agent-ops` when Screen Recording TCC allows.

## Digester

Open candidates: none this window. Night still moved the ratchet via standing backlog (#14).
