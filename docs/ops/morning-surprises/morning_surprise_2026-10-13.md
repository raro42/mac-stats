# Morning surprise — 2026-10-13

Overnight Track B opened with a design-review keep on Top Processes, then shared AI Chat Clear glass cut for GitHub **#14**.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on always-visible pin chrome and disabled Clear keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted look without that layer cost.

## Still open

Shared `#perplexity-search-btn:disabled` still on glass opacity in `agent-ops.css` (Apple override already opaque). `.process-list-header-pin` still glass at 0.55. Layout-daily Data Poster still due. Recapture `feature-processes` / `feature-ai-chat` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `e10c076b` — v0.1.1897 shared `#chat-clear-btn:disabled` opaque.
