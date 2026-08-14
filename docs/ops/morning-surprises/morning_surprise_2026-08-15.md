# Morning surprise — 2026-08-15

Overnight Track B (20:00–06:00 local, window opened 2026-08-14).

## Shipped

| Version | What |
|---------|------|
| **v0.1.401** | Top Processes: j/k selection + Esc clears; `P` pin/unpin; keyboard hint (Monitors / Disk Cleanup parity) |
| **v0.1.402** | Monitors: `d` toggles detail panel; Esc closes open detail before clearing selection; hint + tooltips |
| **v0.1.403** | Top Processes: PageUp/PageDown jump ~5 rows; `d` opens details; Esc closes details modal before clearing selection |
| **v0.1.404** | Monitors: PageUp/PageDown jump ~5 rows (Top Processes / Agent Ops parity); hint + tooltips |
| **v0.1.405** | Disk Cleanup: PageUp/PageDown jump ~5 rows on scopes and categories; hints + tooltips |
| **v0.1.406** | Disk Cleanup soft-delete: skip file when Trash move fails (no permanent-delete fallback on EPERM) |
| **v0.1.407** | Disk Cleanup last-run note counts soft-delete skips when Trash move fails (files left in place) |
| **v0.1.408** | AI Chat Send: busy-guard (Sending…) blocks double Enter/click; brief Sent flash on success |

## Design review

- Not due at 20:00–23:11 ticks (feature screens ok/grace; ai-chat ~0.43d).
- Next due surface: wait for age >3d or re-shoot when TCC allows.

## Digester

- Open candidates: none (latency n/a after noise filters).
- Fuel used: standing backlog keyboard UX (v0.1.401–405), soft-delete safety (v0.1.406), skip visibility (v0.1.407), AI Chat Send feedback (v0.1.408).

## Notes

- Nightly minimum satisfied: keep rows in `autoresearch/results.tsv` for v0.1.401–408.
- Quiet ticks: 0 so far this window.
- Installed `/Applications/mac-stats.app` at 0.1.408; Discord Ready after kickstart.
