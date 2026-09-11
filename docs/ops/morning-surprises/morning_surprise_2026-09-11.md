# Morning surprise — 2026-09-11

Ralf: overnight Track B kept shipping.

## Shipped tonight (so far)

| Version | What |
|---------|------|
| **v0.1.1010** | Perplexity Search filter **Clear** beside All · Top · Snippet (Cleared flash; filter-miss Clear filter CTA; Debug Log/Disk/Monitors/Processes/AI Chat parity) |
| **v0.1.1009** | Debug Log filter **Clear** beside All · Error · Warn (Cleared flash; filter-miss Clear filter CTA; Disk/Monitors/Processes/AI Chat parity) |
| **v0.1.1008** | Auto-enable AI when local Ollama answers on fresh installs (GitHub #11) |
| **v0.1.1007** | Instant lane: `launchd.stdout.log` age |
| **v0.1.1006** | Instant lane: `launchd.stdout.log` size |
| **v0.1.1005** | Instant lane: `launchd.stdout.log` path |
| **v0.1.1004** | Instant lane: `launchd.stderr.log` age |
| **v0.1.1003** | Instant lane: `launchd.stderr.log` size |
| **v0.1.1002** | Instant lane: `launchd.stderr.log` path |
| **v0.1.1001** | Instant lane: harness loop stderr age |
| **v0.1.1000** | Instant lane: harness loop stderr size |
| **v0.1.999** | Instant lane: harness loop stderr path |
| **v0.1.998** | Disk Cleanup filter Clear chip |
| **v0.1.997** | Monitors filter Clear chip |
| **v0.1.996** | Top Processes filter Clear chip |

## Fuel notes
- Digester open empty; design review in grace (~4.7d on CPU metrics).
- 20:28 tick picked Perplexity Clear (standing Clear-parity gap from loop_backlog) instead of another path/size/age lane.
- Next: digester/debug errors when present; design review when due; Ops Clear gaps if any; sibling ports.

## How to try
1. Open CPU window → Perplexity Search (after a search with results).
2. Tap **Top** or **Snippet**.
3. Clear appears beside the chips; tap it (or the filter-miss **Clear filter** CTA) → **Cleared** flash → back to All.
