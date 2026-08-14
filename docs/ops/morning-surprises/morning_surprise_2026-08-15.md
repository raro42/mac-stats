# Morning surprise — 2026-08-15

Overnight Track B (20:00–06:00 local, window opened 2026-08-14).

## Shipped

| Version | What |
|---------|------|
| **v0.1.401** | Top Processes: j/k selection + Esc clears; `P` pin/unpin; keyboard hint (Monitors / Disk Cleanup parity) |
| **v0.1.402** | Monitors: `d` toggles detail panel; Esc closes open detail before clearing selection; hint + tooltips |
| **v0.1.403** | Top Processes: PageUp/PageDown jump ~5 rows; `d` opens details; Esc closes details modal before clearing selection |

## Design review

- Not due at 20:00 / 20:27 / 20:53 ticks (feature screens ok/grace; ai-chat ~0.34d).
- Next due surface: wait for age >3d or re-shoot when TCC allows.

## Digester

- Open candidates: none (latency n/a after noise filters).
- Fuel used: standing backlog keyboard UX on Top Processes, Monitors detail, then Processes paging/`d`.

## Notes

- Nightly minimum satisfied: keep rows in `autoresearch/results.tsv` for v0.1.401–403.
- Quiet ticks: 0 so far this window.
- Installed `/Applications/mac-stats.app` at 0.1.403; Discord Ready after kickstart.
