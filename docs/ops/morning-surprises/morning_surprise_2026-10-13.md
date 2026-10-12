# Morning surprise — 2026-10-13

Overnight Track B kept cutting glass `opacity` for GitHub **#14** (design-review / standing fuel). Top Processes pin, shared Clear, shared Perplexity Search, then the pin column header.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1899** | Shared Top Processes pin column header (`.process-list-header-pin`) opaque type — no glass `opacity` on the ★ header (design review `feature-processes`; Mac capture deferred) |
| **v0.1.1898** | Shared Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on muted chrome keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted look without that layer cost. The ★ column header now matches the Apple row-pin path.

## Still open

Shared `.disk-cleanup-toolbar .disk-cleanup-primary:disabled` still on glass opacity in `agent-ops.css` (Apple override already opaque). Shared `.process-pin` still glass at 0.45 (Apple override opaque in v0.1.1896). Layout-daily Data Poster still due (screens missing). Recapture `feature-processes` / `feature-ai-chat` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `9c472cbd` — v0.1.1899 shared `.process-list-header-pin` opaque.
