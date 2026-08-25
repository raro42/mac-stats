# Morning surprise — 2026-08-25

Overnight Track B shipped **v0.1.627** — real CPU-window fixes after a quiet gap: Low Power Mode toggle works again, GPU joins the history sparklines, and Agent Ops stays fully hidden when you close it.

## Latest keep (this tick)

**v0.1.627** — bundled product fixes:

- **LPM strip** — chip restored on the power strip; click toggles Low Power Mode via `pmset powermode` (admin prompt); On/Off updates every refresh.
- **History sparklines** — GPU chart added (CPU · GPU · Freq · Temp); temp chart draws without waiting on `can_read_temperature`; Y-scale follows live data.
- **Section state** — icon-line sections remember open/closed across restarts (`config.json` + localStorage).
- **Agent Ops** — closed means fully hidden (no header or Discord glance).
- **Changelog** — body↔header toolbar wrap (last version → Close; title ← last version).

## Tonight's keeps

| Version | What |
|---------|------|
| **v0.1.627** | LPM toggle + GPU sparklines + section persistence + Agent Ops hide + changelog wrap |

## Notes

- Digester open empty (10 turns, 7 instant); design review not due (feature-ai-chat ~11.3d grace).
- `debug.log`: disk-at-91% Discord chatter only (no product ERROR cluster).
- Installed release @ v0.1.627; Discord Ready after install.
