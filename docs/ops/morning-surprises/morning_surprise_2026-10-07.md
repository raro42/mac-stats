# Morning surprise — 2026-10-07

Overnight Track B kept shipping GitHub **#14** WebView compositor cuts (opaque washes instead of glass alpha).

## Shipped tonight
- **v0.1.1585** — Top Processes row pinned / hover / focus / active / selected: opaque accent wash, no glass drop shadow.
- **v0.1.1584** — Details value hover / focus / selected opaque wash.
- **v0.1.1583** — Ring / power-strip copy hover + focus opaque wash.
- **v0.1.1582** — Ring / power-strip Copied flash opaque wash.
- Earlier in the window: Agent Ops / AI Chat / Debug Log / Perplexity / Disk Cleanup / Monitors / Top Processes Copied flashes (v0.1.1575–1581).

## Tried / context
- Digester open stayed empty; design review in grace (feature-agent-ops still the stale screenshot).
- Sibling harnesses unavailable on this host (missing git checkouts).
- Linux webkit floor still not the macOS Graphics and Media gate — each cut still needs macOS proof for #14 close.

## Ratchet
- Keep @ `9538daaf` — v0.1.1585 Top Processes row interaction opaque wash (#14).
