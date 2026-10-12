# Morning surprise — 2026-10-13

Overnight Track B kept cutting glass `opacity` for GitHub **#14** (design-review fuel). Top Processes pin, then shared Clear, then shared Perplexity Search.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1898** | Shared Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on disabled Search keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted look without that layer cost. Non-Apple themes now match the Apple Search disabled path.

## Still open

Shared `.disk-cleanup-toolbar .disk-cleanup-primary:disabled` still on glass opacity in `agent-ops.css` (Apple override already opaque). `.process-list-header-pin` still glass at 0.55. Shared `button.ops-badge:disabled` / `.ops-agent-edit-actions .btn-primary:disabled` still glass. Layout-daily Data Poster still due (screens missing). Recapture `feature-processes` / `feature-ai-chat` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `3aafdf5d` — v0.1.1898 shared `#perplexity-search-btn:disabled` opaque.
