# Morning surprise — 2026-10-07

Overnight Track B kept shipping GitHub **#14** WebView compositor cuts (opaque washes instead of glass alpha).

## Shipped tonight
- **v0.1.1594** — Monitor history tick tips sit above the bar with left and top. Opaque fill. No translate or shadow.
- **v0.1.1592** — AI Chat filter chips (All · You · Assistant · Errors) and Clear: opaque wash (no glass alpha).
- **v0.1.1591** — Agent Ops overview cards (Agents · Schedules · Sessions · Memory): opaque wash; soft hover drop shadows removed.
- **v0.1.1590** — Rings filter chips (All · Hot): opaque wash.
- **v0.1.1589** — External / Monitors filter chips (All · Up · Down · Slow) and Clear: opaque wash.
- **v0.1.1588** — Agent Ops health cards (Version · Discord · Redmine · Next schedule · Last delivery · Digest): opaque wash; active hover soft shadow removed.
- **v0.1.1587** — Top Processes filter chips (All · Pinned · Hot) and Clear: opaque wash.
- **v0.1.1586** — Top Processes pin hover / focus-visible: opaque accent wash.
- **v0.1.1585** — Top Processes row pinned / hover / focus / active / selected: opaque accent wash, no glass drop shadow.
- Earlier in the window: Details / ring / power-strip / Copied flashes (v0.1.1575–1584).

## Tried / context
- Digester open stayed empty; design review in grace (feature-agent-ops still the stale screenshot).
- Sibling harnesses unavailable on this host (missing git checkouts).
- Linux webkit floor still not the macOS Graphics and Media gate — each cut still needs macOS proof for #14 close.

## Ratchet
- Keep @ `00643870` — v0.1.1592 AI Chat filter chip opaque wash (#14).
- Keep @ `0e551588` — v0.1.1591 Agent Ops overview card opaque wash (#14).
- Keep @ `34078bdd` — v0.1.1590 Rings filter chip opaque wash (#14).
- Keep @ `a3e06189` — v0.1.1589 External/Monitors filter chip opaque wash (#14).
