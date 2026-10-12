# Morning surprise — 2026-10-13

Overnight Track B kept cutting glass `opacity` for GitHub **#14**. Design review / standing queue: AI Chat Clear chip, shared Clear/Search/pin header, then shared Clean now disabled.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1901** | Shared Disk Cleanup Clean now disabled (`.disk-cleanup-toolbar .disk-cleanup-primary:disabled`) opaque type — no glass `opacity` while Clean now waits (design review `feature-disk-cleanup`; Mac capture deferred) |
| **v0.1.1900** | Apple AI Chat filter Clear (`.chat-filter-clear` resting · hover) opaque type — no glass `opacity` on Clear beside All · You · Assistant · Errors (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1899** | Shared Top Processes pin column header (`.process-list-header-pin`) opaque type — no glass `opacity` on the ★ header (design review `feature-processes`; Mac capture deferred) |
| **v0.1.1898** | Shared Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on muted chrome keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted look without that layer cost. Clean now disabled now matches Apple and the shared Search/Clear disabled path.

## Still open

Shared `.process-pin` still glass at 0.45 (Apple override opaque in v0.1.1896). Shared `.chat-filter-clear` still glass (Apple override opaque in v0.1.1900). Layout-daily dark still due (screens missing). Recapture `feature-disk-cleanup` / `feature-ai-chat` / `feature-cpu-metrics` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `063b19a2` — v0.1.1901 shared `.disk-cleanup-primary:disabled` opaque.
