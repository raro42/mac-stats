# Morning surprise — 2026-10-06

Overnight Track B cut WebView idle work for GitHub #14 and quieted Ollama-down model-list WARN spam in debug.log.

## Shipped

| Version | What |
| --- | --- |
| **v0.1.1388** | Ollama model-list: 30s fail cooldown + at most one WARN / 5 minutes when Ollama is off (log-012). CPU window: macOS opaque (`transparent(false)`), metrics every 8s, pause more polls when hidden, opaque sparklines, slower history poll, dark theme no infinite hover glow. |
| **v0.1.1387** | Apple opaque shell, instant ring paints, 5s metrics / 30s Discord icon, WKWebView background Suspend (#14). |
| **v0.1.1386** | Static Agent Ops loading / Force Quit confirm; theme cpu.css drop live backdrop-filter; thinking dots static. |
| **v0.1.1385** | Changelog body keyboard hint first paint in theme HTML. |
| **v0.1.1384** | Agent Ops row-selection Tips keyboard hint first paint in theme HTML. |
| **v0.1.1383** | AI Chat message-list keyboard hint first paint in theme HTML. |

## Tried / context

- Digester open stayed empty; design review still in grace.
- Fuel: P2 debug.log log-012 (41 model_cache WARNs) plus standing #14 idle cuts.
- Linux WebKit still hot on blank pages; #14 needs macOS Activity Monitor before close.

## For Ralf

Open the CPU window on this build. With Ollama off, `debug.log` should stay calm (one WARN every few minutes, not a burst). Activity Monitor (Graphics and Media / `tauri://localhost`) should be quieter when the window is hidden. #14 stays open until the webview sits under ~1% on macOS.

## Tick notes

- ~22:55 tick: Ollama model-list WARN cooldown + #14 idle cuts (**v0.1.1388**); sync-dist + ratchet keep.
- ~22:20–22:40: prior #14 compositor / idle work (**v0.1.1386–1387**); tester FAIL on Linux under-1% AC.
