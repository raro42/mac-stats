# Morning surprise — 2026-09-13

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1049** | Instant `/recent-keeps` · `recent keeps` · `list recent keeps` · `tonight keep list` (+ discard variants) — short tonight keep/discard list from `results.tsv` (newest first; capped at 5) |
| **v0.1.1048** | Instant `/last-keep` · `last keep` · `latest keep` · `what was the last keep` (+ discard variants) — newest keep/discard row from `results.tsv` (one description; no dump) |
| **v0.1.1047** | Instant `/keeps` · `keeps tonight` · `keep count` · `discard count` · `ratchet summary` — keep/discard counts (tonight + all-time) |
| **v0.1.1046** | Instant Digest open NL (`view digest` / `see digest` / `show me the digest` / `open digest` / `list the digest`) |
| **v0.1.1045** | Instant `/perplexity` last-search NL expand |
| **v0.1.1044** | Instant Runs-lane NL (`/failed` · `/slow` · `/instant` · `/lite` · `/direct`) |
| **v0.1.1043** | Instant Hot/Pinned NL (`/hot` · `/pinned`) |
| **v0.1.1042** | Instant power-strip chip NL |
| **v0.1.1041** | Instant ring-chip NL |
| **v0.1.1040** | Instant `/rings` · `/strip` · `/details` NL |

## This tick (~04:30)

- Digester open empty; design review not due (grace).
- Experiment: Hermes-style ratchet **tonight list** after counts + last-row.
- Keep in `results.tsv`; install + kickstart.

## Try it

```text
/recent-keeps
recent keeps
list recent keeps
tonight keep list
/recent-discards
/last-keep
/keeps
```

## Not a surprise alone

Empty digester / “stayed on version X” is not the surprise — the table above is.
