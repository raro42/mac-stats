# Morning surprise — 2026-08-19

Overnight Track B (design review / standing backlog). Digester open empty; design-review due=false (grace).

## Shipped tonight

| Version | What got nicer |
|---------|----------------|
| **v0.1.526** | Disk Cleanup Next automatic run meta-card click (due → Clean now; else scroll to last run; green wash when due) |
| **v0.1.527** | AI Chat empty-state starter chips show brief **In composer** flash when clicked (Load into AI Chat parity; still no auto-send) |
| **v0.1.528** | Disk Cleanup **Runs when** meta-card click (scrolls to enabled scopes; soft blue wash when periodic cleanup is off) |
| **v0.1.529** | Disk Cleanup **Last run** panel click (opens first category cleaned last run; reclaim/scopes fallback; amber wash when Trash skips) |
| **v0.1.530** | Monitors summary click opens the **slowest** site when all monitors are up (DOWN still opens first DOWN row; soft amber wash on summary) |
| **v0.1.531** | Top Processes **Top CPU** glance strip under the header — shows hottest process; click opens details (Monitors slowest-summary parity; amber wash when CPU ≥ 15%) |
| **v0.1.532** | AI Chat **turn glance** strip — turn count + last question preview; click scrolls to latest message and focuses composer (accent wash while Sending…) |
| **v0.1.533** | Debug Log **error/warn glance** strip — ERROR/WARN counts from the log tail (polls every 60s even when collapsed); click expands Debug Log and filters to errors (or warnings when no errors); red/amber wash |
| **v0.1.534** | CPU metrics **GPU % on the battery/power strip** (RAM / menu-bar parity). Click scrolls to the GPU ring and flashes it. Soft amber wash when GPU ≥ 15%. |

## Tried / notes

- Prep: digester empty (10 turns, 7 instant); design-review due=false; debug.log quiet in 180m scan (single-instance busy WARN only).
- Screenshot recapture still deferred (no on-screen CPU window / TCC); prior feature-cpu-metrics.png kept in grace.
- Latest keep: v0.1.534 installed after this tick.

## Not a quiet night

Ninth keep of the 2026-08-19 window — GPU strip after Debug Log error/warn glance.
