# Morning surprise — 2026-10-13

Overnight Track B kept cutting glass `opacity` for GitHub **#14**. Design review / standing queue: AI Chat Clear chip, shared Clear/Search/pin header, Clean now disabled, then shared Top Processes pin.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1902** | Shared Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin control (standing P2 #14; Apple pin opaque in v0.1.1896) |
| **v0.1.1901** | Shared Disk Cleanup Clean now disabled (`.disk-cleanup-toolbar .disk-cleanup-primary:disabled`) opaque type — no glass `opacity` while Clean now waits (design review `feature-disk-cleanup`; Mac capture deferred) |
| **v0.1.1900** | Apple AI Chat filter Clear (`.chat-filter-clear` resting · hover) opaque type — no glass `opacity` on Clear beside All · You · Assistant · Errors (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1899** | Shared Top Processes pin column header (`.process-list-header-pin`) opaque type — no glass `opacity` on the ★ header (design review `feature-processes`; Mac capture deferred) |
| **v0.1.1898** | Shared Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on muted chrome keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted look without that layer cost. Shared pin now matches Apple and the ★ header path.

## Still open

Shared `.chat-filter-clear` still glass at 0.92 (Apple override opaque in v0.1.1900). Layout-daily dark still due (screens missing). Recapture `feature-disk-cleanup` / `feature-ai-chat` / `feature-processes` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `121eb24a` — v0.1.1902 shared `.process-pin` opaque.
