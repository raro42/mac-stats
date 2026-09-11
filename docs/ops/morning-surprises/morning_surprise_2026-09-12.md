# Morning surprise — 2026-09-12

Overnight Track B kept shipping instant-lane NL so operator asks skip the slow path.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1021** | `/schedules` — `view` / `see` / `show me` / `open` / `list the` schedules → Jobs · Deliveries instant |
| **v0.1.1020** | `/disk` — same NL family for Disk Cleanup (On · Off · Reclaim · Big · Clean) |
| **v0.1.1019** | `/monitors` — same NL family (Up · Down · Slow) |
| **v0.1.1018** | `/processes` — same NL family (Hot · Pinned) |
| **v0.1.1017** | `/logs` — view/see/show me/open/list + digester Slowest filters |

## Also earlier in the window

- Agent Ops filter Clear chips closed out through Schedules (**v0.1.1009–1016**).

## Night health

- Digester open: empty most ticks (standing backlog / design-review fuel).
- Design review: grace (not due); CPU metrics screen oldest ~4.9d when due again.
- Debug log scan: no ERROR/WARN/panic clusters in the watch window.
- Ratchet: keeps landed in `results.tsv` (nightly minimum satisfied).

## Try in AI Chat / Discord

```text
view schedules
show me the schedules
open schedules
list the schedules
```

## Next fuel

- `/sessions` NL parity (careful with `open ` exclusion).
- Design review when due.
- Digester open / product-owned debug.log errors when they appear.
