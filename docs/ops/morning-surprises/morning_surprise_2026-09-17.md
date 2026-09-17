# Morning surprise — 2026-09-17

Overnight Track B (20:00–06:00 local). Digester open empty. Design review not due. Standing backlog / Next → Monitors Down/Slow attention soft parity.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1122** | Monitors Down/Slow attention soft parity — Down · N / Slow · N glances soft alert at 7% / 30% border (Down/Slow filter); louder 8%/34% tint removed |
| **v0.1.1121** | Disk Cleanup has-big attention soft parity — Disk · Big glance soft alert at 7% / 30% border (Monitors Slow / Big filter); louder 8%/34% tint removed |
| **v0.1.1120** | Processes Hot attention soft parity — Hot · N hot glance soft alert at 7% / 30% border (Monitors Slow); louder 8%/34% tint removed |
| **v0.1.1119** | Disk Cleanup Big filter attention soft parity — Disk · Big glance soft alert at 7% / 30% border (Monitors Slow); louder 8%/34% tint removed; Reclaim stays softer; Clean stays green |
| **v0.1.1118** | Ops Slow/Fail filter attention soft parity — Slow / Fail glances soft alert at 7% / 30% border (Monitors Slow / Down); louder 8%/34% tint removed |
| **v0.1.1117** | Ops accent filter attention soft parity — Off / Files / Deliveries / Discord glances soft accent at 7% / 28% border (Ready / `.is-filter`); louder 8%/34% tint removed |
| **v0.1.1116** | Ops filter attention soft parity — On / Live / Jobs / Core / Instant / Lite / Direct glances soft green at 7% / 28% border (Ready / Monitors Up); louder 8%/34% tint removed |
| **v0.1.1115** | Disk Cleanup due attention soft parity — Disk · Due glance soft green at 7% / 28% border (Ready calm / collapsed is-due); louder 8%/34% tint removed |

Also continues from late 2026-09-16 evening keeps (v0.1.1105–1114) in the same overnight window.

## Why it matters

When a site is Down or Slow, the Monitors attention strip now matches the Down/Slow filter soft alert (7% fill, 30% border) — less shouty beside Up / Filter glances, still clearly red or amber.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Processes / Monitors; recapture AI Chat when Screen Recording TCC allows)
- Next calm: Monitors summary/collapsed has-down soft parity; ops-runs has-fail/has-slow if still louder; Disk Cleanup reclaim if desired; sibling Hermes/OpenClaw ports with clear user fitness
