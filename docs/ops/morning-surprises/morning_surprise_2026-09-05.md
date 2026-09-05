# Morning surprise — 2026-09-05

Overnight Track B kept shipping **instant-lane** size/age/path operators so Discord/AI Chat skips Ollama for operator housekeeping.

## Shipped tonight (highlights)

| Version | What |
|---------|------|
| **v0.1.875** | Instant: **tmp directory size** (`tmp size`, `how big is tmp`, `tmp folder size`, …) — recursive file bytes under `~/.mac-stats/tmp/`; no list dump; does not steal path/prune/temperature |
| **v0.1.874** | Instant: **screenshots directory size** (`screenshots size`, `how big are screenshots`, …) — recursive under BROWSER_SCREENSHOT dir |
| **v0.1.873** | Instant: **improvements directory size** — recursive under `~/.mac-stats/improvements/` |
| **v0.1.872** | Instant: **digest.md / latest.md size** — `latest.md` stat only |
| **v0.1.871** | Instant: **digest file size** — `latest.json` stat only |
| **v0.1.870** | Instant: **runs.jsonl size** |
| **v0.1.869** | Instant: **results.tsv size** |
| **v0.1.868** | Instant: **results.tsv age** |
| **v0.1.867** | Instant: **runs.jsonl age** |
| **v0.1.866** | Instant: **results.tsv path** |
| **v0.1.865** | Agent Ops Filter attention glance (On/Off · Live/Files · Jobs/Deliveries · Discord/Core · Runs lanes) |

## Tick notes (~05:35)

- Digester **open** empty; design review **due=false** (grace; screens aged).
- Fuel: standing backlog p50 latency → tmp dir size (after screenshots size).
- Debug log scan: no ERROR/WARN clusters in window.
- Shared helper: `dir_total_bytes` (improvements + screenshots + tmp).

## Try it

```text
tmp size
how big is tmp
tmp folder size
tmp path
screenshots size
```
