# Morning surprise — 2026-10-20

Overnight Track B kept shipping glass-opacity cuts (#14) while `feature-cpu-metrics.png` capture stayed blocked (`screencapture -l` unavailable / no CPU window on screen).

## Shipped this tick (~01:29 CEST)

| Version | Keep | What got better |
|---------|------|-----------------|
| **v0.1.2144** | Neon AI Chat field and Send opaque | `#chat-input` and `#chat-send-btn` mix on `#08120c`. No WebView glass rgba |
| **v0.1.2143** | Dark AI Chat field and Send opaque | `#chat-input` and `#chat-send-btn` mix on `#0a0a0a`. No WebView glass rgba |
| **v0.1.2142** | Light AI Chat field and Send opaque | `#chat-input` and `#chat-send-btn` mix on `#f7f7fa`. No WebView glass rgba |
| **v0.1.2141** | Material AI Chat field and Send opaque | `#chat-input` and `#chat-send-btn` mix on `#ebe4d9`. No WebView glass rgba |

## Still open

1. Recapture `docs/screens/feature-cpu-metrics.png` when window capture works
2. Futuristic `#chat-input` still uses rgba glass on the field
3. Other theme composer fields may still carry glass rgba
4. Layout-daily data-poster / dark / neon / futuristic / architect screens still missing
