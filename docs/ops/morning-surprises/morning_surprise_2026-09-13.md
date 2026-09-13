# Morning surprise — 2026-09-13

Overnight Track B (autoresearch) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1052** | Instant `/longest-streak` · `longest streak` · `best streak` · `record streak` (+ max-streak / longest-discard) — longest keep/discard streak from `results.tsv` (all-time + tonight since 20:00; record only; no dump) |
| **v0.1.1051** | Instant `/keep-streak` · `keep streak` · `current streak` · `ratchet streak` (+ discard-streak / `/streak`) — consecutive keep/discard streak from `results.tsv` (current + tonight since 20:00; streak only; no dump) |
| **v0.1.1050** | Instant `/keep-rate` · `keep rate` · `hit rate` · `ratchet hit rate` · `keep percentage` (+ discard-rate variants) — tonight + all-time keep % from `results.tsv` (rate only; no dump) |
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

## This tick (~06:00)

- Digester open empty; design review not due (grace).
- Experiment: Hermes-style ratchet **longest streak** after counts + last-row + tonight list + hit rate + current streak.
- Keep in `results.tsv`; install + kickstart.

## Try it

```text
/longest-streak
longest streak
best streak
record streak
```

## Night status

- Nightly minimum: satisfied (many keeps in the 20:00–06:00 window).
- Quiet ticks: none required.
