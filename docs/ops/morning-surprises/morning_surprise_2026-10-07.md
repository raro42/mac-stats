# Morning surprise — 2026-10-07

Overnight Track B kept shipping GitHub **#14** WebView compositor cuts (opaque washes instead of glass alpha).

## Shipped tonight
- **v0.1.1612** — Agent Ops loading shell: opaque wash (no glass alpha).
- **v0.1.1611** — Agent Ops Overview Open links (resting · hover · focus-visible): opaque wash (no glass alpha).
- **v0.1.1610** — Agent Ops copy chips (resting · hover · focus-visible): opaque wash (no glass alpha).
- **v0.1.1609** — Agent Ops empty panels and the Clear filter button: opaque wash (no glass alpha).
- **v0.1.1608** — Agent Ops On · Off badges: opaque wash (no glass alpha).
- **v0.1.1607** — Agent Ops list rows (resting · hover · focus · selected): opaque wash.
- **v0.1.1606** — Agent Ops Runs list rows (Lite · Slow · Fail): opaque wash.
- **v0.1.1605** — Agent Ops Runs filter chips (All · Instant · Lite · Direct · Slow · Fail) and Clear: opaque wash (no glass alpha).
- **v0.1.1604** — Agent Ops Knowledge filter chips (All · Discord · Core) and Clear: opaque wash (no glass alpha).
- **v0.1.1603** — Agent Ops Schedules filter chips (All · Jobs · Deliveries) and Clear: opaque wash (no glass alpha).
- **v0.1.1602** — Agent Ops Agents filter chips (All · On · Off) and Clear: opaque wash (no glass alpha).
- **v0.1.1601** — Agent Ops Sessions filter chips (All · Live · Files) and Clear: opaque wash (no glass alpha).
- **v0.1.1600** — Agent Ops tab strip (tabs, file tabs, count pills): opaque wash.
- **v0.1.1599** — Ring, battery, and power Copied flashes sit above the value with left and right. No translate.
- **v0.1.1598** — Agent Ops filter input, match chip, Clear, and just-cleared flash: opaque wash.
- **v0.1.1597** — Disk Cleanup scope filter chips (All · On · Off) and Clear: opaque wash.
- **v0.1.1596** — Disk Cleanup category filter chips (All · Reclaim · Big · Clean) and Clear: opaque wash.
- **v0.1.1595** — Debug Log filter chips (All · Error · Warn) and Clear: opaque wash.
- **v0.1.1594** — Monitor history tick tips: left/top placement, opaque fill, no translate or shadow.
- **v0.1.1593** — Perplexity filter chips (All · Top · Snippet) and Clear: opaque wash.
- **v0.1.1592** — AI Chat filter chips (All · You · Assistant · Errors) and Clear: opaque wash.
- **v0.1.1591** — Agent Ops overview cards: opaque wash; soft hover drop shadows removed.
- **v0.1.1590** — Rings filter chips (All · Hot): opaque wash.
- **v0.1.1589** — External / Monitors filter chips (All · Up · Down · Slow) and Clear: opaque wash.
- Earlier in the window: Agent Ops health, Top Processes filters/pin/row, Details / ring / Copied flashes.

## Tried / context
- Digester open stayed empty; design review in grace (feature-agent-ops still the stale screenshot).
- Sibling harnesses unavailable on this host (missing git checkouts).
- Linux webkit floor still not the macOS Graphics and Media gate. Each cut still needs macOS proof for #14 close.

## Ratchet
- Keep — v0.1.1612 Agent Ops loading shell opaque wash (#14).
- Discard — local Runs chip wash duplicated origin v0.1.1605.
- Discard — local tab wash duplicated origin v0.1.1600. Local Schedules wash duplicated origin v0.1.1603.
- Keep — v0.1.1611 Agent Ops Overview Open link resting · hover · focus-visible opaque wash (#14).
- Keep — v0.1.1610 Agent Ops copy-chip resting · hover · focus-visible opaque wash (#14).
- Keep — v0.1.1609 Agent Ops empty panel + Clear filter opaque wash (#14).
- Keep — v0.1.1604 Agent Ops Knowledge All · Discord · Core filter chip opaque wash (#14).
- Keep @ `27f9dd35` — v0.1.1601 Agent Ops Sessions All · Live · Files filter chip opaque wash (#14).
- Keep — v0.1.1600 Agent Ops tab strip / count pills opaque wash (#14).
- Keep — v0.1.1599 metric Copied flash uses left and right. No translate (#14).
- Keep @ `8f5d9e58` — v0.1.1596 Disk Cleanup category filter chip opaque wash (#14).
- Keep @ `aec88e78` — v0.1.1595 Debug Log filter chip opaque wash (#14).
- Keep @ `228fbb0e` — v0.1.1594 monitor tick tips without translate (#14).
- Keep @ `d5bb39b2` — v0.1.1593 Perplexity filter chip opaque wash (#14).
- Keep @ `00643870` — v0.1.1592 AI Chat filter chip opaque wash (#14).
