# Morning surprise — 2026-09-12

Overnight Track B kept shipping operator instant-lane NL. Digester open stayed empty; design review stayed in grace. Standing backlog p50 drove the night.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1036** | `/redmine` · `/brave` · `/mastodon` · `/mcp` — `view` / `see` / `show me the` / `open` / `list the` (+ brave search / mcp server) → Ready / Not set / Partial (config only; no tickets / web search / toot / MCP tools) |
| **v0.1.1035** | `/telegram` · `/slack` · `/signal` · `/alerts` — `view` / `see` / `show me the` / `open` / `list the` (+ bot / webhook / app / alert channels) → Ready / Not set / Partial (config only; no live send) |
| **v0.1.1034** | `/voice` · `/stt` NL expand + `/having_fun` show-me normalizer fix |
| **v0.1.1033** | `/having_fun` · `/fun` · `/idle` NL expand |
| **v0.1.1032** | `/ori` · `/mnemos` NL expand |
| **v0.1.1031** | `/downloads` · `/organizer` NL expand (earlier window) |

## Fuel notes

- Digester: open empty all ticks (do not quiet-default).
- Design review: `due=false`; recommended `feature-cpu-metrics.png` (~5.8d) still in grace.
- debug.log: no ERROR/WARN/panic clusters in scan window.
- Next: digester/debug when present; design review when due; `/cursor` · `/perplexity key` · `/ollama` NL parity if still thin; sibling ports when a clear fit shows.

## Ratchet

Keep rows in `~/.mac-stats/improvements/autoresearch/results.tsv` for each ship (nightly minimum met).
