# Morning surprise — 2026-11-22

Overnight autoresearch (Track B) kept shipping operator instant lanes. Digester open stayed empty; design-review stayed in grace (Mac TCC); layout-daily already marked data-poster for the day.

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.2955** | Primary link status · media / speed `/link-status` · `/media` · `/link-speed` + view/open NL |
| v0.1.2954 | Primary IPv4 broadcast `/broadcast` · `/bcast` + view/open NL |
| v0.1.2953 | Primary / default-route iface `/iface` · `/ifname` · `/interface` + view/open NL |
| v0.1.2952 | Primary IPv4 netmask `/netmask` · `/subnet` · `/cidr` + view/open NL |
| v0.1.2951 | Primary interface MTU `/mtu` · `/if-mtu` · `/interface-mtu` + view/open NL |
| v0.1.2950 | Primary MAC `/mac-address` · `/ether` · `/en0-mac` + view/open NL |
| earlier | wifi · dns · gateway · ip · boot · cores · kernel · locale · arch · tz · serial · … |

## This tick (~05:25 CET)

- Fuel: standing product after broadcast — `ifconfig` `status:` + `media:` on the default-route iface had no dedicated slash+NL while `/mtu` · `/mac-address` · `/iface` · `/broadcast` already did.
- Verify OK → keep @ `47aa870` → push `main` (also backfilled missing 2954 keep row).
- Bare `status` stays app health; bare `media` / `speed` / `link` stay with the agent; `/link-mtu` stays MTU; `/link-address` stays MAC.

## Still open

- Dark / Neon / Futuristic layout-daily next calendar day
- cream/Swiss Send fill white-on-blue intentional (leave alone)
- sibling harness ports when digester stays empty
