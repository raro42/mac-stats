# Morning surprise — 2026-10-12

Overnight Track B kept shipping Apple / dark-shell glass cuts for GitHub **#14**.

## Shipped this night (latest first)

| Version | What |
|---------|------|
| **v0.1.1888** | Disk Cleanup Clean now disabled (`.disk-cleanup-primary:disabled`) opaque type — no glass `opacity` while Clean now waits |
| **v0.1.1887** | Perplexity Search disabled (`#perplexity-search-btn:disabled`) opaque type — no glass `opacity` while Search waits |
| **v0.1.1886** | AI Chat Clear disabled (`#chat-clear-btn:disabled`) opaque type — no glass `opacity` while Clear waits |
| **v0.1.1885** | AI Chat Send disabled (`#chat-send-btn:disabled`) opaque type/fill — no glass `opacity` while Send waits |
| **v0.1.1884** | Agent Ops Save disabled (`.ops-agent-edit-actions .btn-primary:disabled`) opaque type |
| **v0.1.1883** | Agent Ops badge disabled (`button.ops-badge:disabled`) opaque type |
| **v0.1.1882** | Agent Ops Off badge (`.ops-badge.off`) opaque type |
| **v0.1.1881** | Agent Ops loading (`.ops-loading`) opaque type |
| **v0.1.1880** | Agent Ops filter caption (`.ops-filter-caption`) opaque type |
| **v0.1.1879** | Data Poster / Dark Monitors filter attention glance dark remap |
| **v0.1.1878** | Data Poster / Dark Monitors Down/Slow attention glance dark remap (layout daily) |

## Why it matters

Glass `opacity` on busy chrome keeps WebView Graphics and Media hot. Opaque `color-mix` washes keep the same muted look without that layer cost.

## Still open

`.force-quit-btn:disabled` glass opacity in Apple theme (still `opacity: 0.7`). Recapture `feature-disk-cleanup.png` / `feature-ai-chat.png` when a Mac CPU window + Screen Recording TCC allow.

## Ratchet

Latest keep: `d5bd22e9` — v0.1.1888 Disk Cleanup Clean now disabled opaque.
