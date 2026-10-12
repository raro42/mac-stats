# Morning surprise — 2026-10-13

Overnight Track B kept cutting glass `opacity` for GitHub **#14**. Design review picked up stale `feature-ai-chat` after grace expired, then the standing glass queue (shared Clear, Search, pin header).

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1900** | Apple AI Chat filter Clear (`.chat-filter-clear` resting · hover) opaque type — no glass `opacity` on Clear beside All · You · Assistant · Errors (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1899** | Shared Top Processes pin column header (`.process-list-header-pin`) opaque type — no glass `opacity` on the ★ header (design review `feature-processes`; Mac capture deferred) |
| **v0.1.1898** | Shared Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on muted chrome keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted look without that layer cost. AI Chat Clear beside the role chips now matches the Agent Ops Clear chips.

## Still open

Shared `.disk-cleanup-toolbar .disk-cleanup-primary:disabled` still on glass opacity in `agent-ops.css` (Apple override already opaque). Shared `.process-pin` still glass at 0.45; shared `.chat-filter-clear` still glass at 0.92 (Apple override opaque in v0.1.1900). Layout-daily dark still due (screens missing). Recapture `feature-ai-chat` / `feature-processes` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `f32d711c` — v0.1.1900 Apple `.chat-filter-clear` opaque.
