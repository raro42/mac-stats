# Morning surprise — 2026-10-13

Overnight Track B kept cutting glass `opacity` for GitHub **#14**. Latest: shared rings filter count.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1907** | Shared rings filter count (`.rings-filter-count`) opaque type — no glass `opacity` on the Hot count beside All · Hot (design review `feature-cpu-metrics`; Mac capture deferred) |
| **v0.1.1906** | Shared Agent Ops close (`.ops-close-btn` resting · hover) opaque type — no glass `opacity` on the × in the Agent Ops header (Apple close opaque in v0.1.1856) |
| **v0.1.1905** | Shared rings filter chips (`.rings-filter-chip` resting · hover · active) opaque type — no glass `opacity` on All · Hot beside the metric rings (design review `feature-cpu-metrics`; Mac capture deferred) |
| **v0.1.1904** | Shared Agent Ops filter Clear (`.ops-filter-clear` resting · hover) opaque type — no glass `opacity` on Clear beside Ops search match chips (Apple Clear already opaque) |
| **v0.1.1903** | Shared AI Chat filter Clear (`.chat-filter-clear` resting · hover) opaque type — no glass `opacity` on Clear beside All · You · Assistant · Errors |
| **v0.1.1902** | Shared Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin control (Apple pin opaque in v0.1.1896) |
| **v0.1.1901** | Shared Disk Cleanup Clean now disabled (`.disk-cleanup-toolbar .disk-cleanup-primary:disabled`) opaque type — no glass `opacity` while Clean now waits (design review `feature-disk-cleanup`; Mac capture deferred) |
| **v0.1.1900** | Apple AI Chat filter Clear (`.chat-filter-clear` resting · hover) opaque type — no glass `opacity` on Clear beside All · You · Assistant · Errors (design review `feature-ai-chat`; Mac capture deferred) |
| **v0.1.1899** | Shared Top Processes pin column header (`.process-list-header-pin`) opaque type — no glass `opacity` on the ★ header |
| **v0.1.1898** | Shared Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits |
| **v0.1.1897** | Shared AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits |
| **v0.1.1896** | Apple Top Processes pin (`.process-pin` resting · hover · focus · pinned) opaque type — no glass `opacity` on the pin (design review `feature-processes`; Mac capture deferred) |

Also on main since the prior morning note (daytime): **v0.1.1895** Data Poster / Dark Top Processes contrast (#18); **v0.1.1894** update banner brew hint (#17); **v0.1.1893** Changelog Close/Escape (#16).

## Why it matters

Glass `opacity` on muted chrome keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the muted look without that layer cost. Rings Hot counts now match the opaque All · Hot chips.

## Still open

Shared `.chat-filter-count` / `.processes-filter-count` still glass at 0.9 (zero 0.42). Layout-daily dark still due (screens missing). Recapture `feature-agent-ops` / `feature-cpu-metrics` / `feature-ai-chat` / `feature-processes` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `47c1c4ec` — v0.1.1907 shared `.rings-filter-count` opaque.
