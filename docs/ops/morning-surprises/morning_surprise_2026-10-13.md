# Morning surprise — 2026-10-13

Overnight Track B opened with a design-review keep on Top Processes, then more Apple glass cuts for GitHub **#14**.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on always-visible pin chrome keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted pin look without that layer cost.

## Still open

Shared `#chat-clear-btn:disabled` / `#perplexity-search-btn:disabled` still on glass opacity in `agent-ops.css` (Apple overrides already opaque). `.process-list-header-pin` still glass at 0.55. Recapture `feature-processes` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `4cb34754` — v0.1.1896 Apple `.process-pin` opaque.
