# Morning surprise — 2026-09-13

Overnight Track B kept shipping instant-lane NL so operator Discord asks stay off the slow direct lane.

## Shipped tonight (keep)

| Version | What |
| --- | --- |
| **v0.1.1046** | Digest open NL: `view digest` / `see digest` / `show me the digest` / `open digest` / `list the digest` (+ open-candidates variants) → cached open-candidate snapshot (no digester spawn). Bare `digest` / `/digest` / `show me digest` / `refresh` / `rescan` still refresh. |
| **v0.1.1045** | Perplexity last-search NL: view/see/show me/open/list-the + top/snippet variants. |
| **v0.1.1044** | Runs-lane NL: `/failed` · `/slow` · `/instant` · `/lite` · `/direct` view/see/show me/open/list-the. |
| **v0.1.1043** | Hot/Pinned NL: `/hot` · `/pinned` view/see/show me/open/list-the. |
| **v0.1.1042** | Power-strip chips: battery/heat/lpm/ram/ssd/uptime NL expand. |
| **v0.1.1041** | Ring chips: cpu/gpu/freq/temp NL expand. |
| **v0.1.1040** | Rings/strip/details NL expand. |
| **v0.1.1039** | Insights/help/ops NL expand. |
| **v0.1.1038** | Status/health/version NL expand. |
| **v0.1.1037** | Discord/ollama/perplexity key/cursor NL expand. |
| **v0.1.1036** | Redmine/brave/mastodon/mcp NL expand. |

## Fuel notes

- Digester **open** stayed empty all night — ticks pulled standing-backlog p50 NL expands (not quiet-default).
- Design review: `due=false` (grace); recommended surface CPU metrics ~6.0d when TCC allows a fresh shot.
- `debug.log`: no ERROR/WARN/panic clusters in the scan window.

## Try in Discord / AI Chat

- `view digest` / `show me the digest` / `list the digest` → open candidates (read-only)
- `/digest` or `rescan digest` → refresh digester
- `view perplexity` / `open top results` → last-search list
