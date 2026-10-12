# Morning surprise — 2026-10-13

Overnight Track B kept cutting glass `opacity` on wait-state chrome for GitHub **#14**, after a design-review keep on Top Processes earlier in the window.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1898** | Shared Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits |
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — design review `feature-processes` (Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Disabled Search and Clear used glass `opacity` on shared CSS. That keeps WebView Graphics and Media warmer than an opaque `color-mix` wash with the same muted look. Apple already had those overrides; shared themes catch up.

## Still open

Shared `.process-list-header-pin` still glass at 0.55 in `agent-ops.css`. Recapture `feature-ai-chat.png` / Perplexity when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `3aafdf5d` — v0.1.1898 shared `#perplexity-search-btn:disabled` opaque.
