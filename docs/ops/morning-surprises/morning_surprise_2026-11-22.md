# Morning surprise — 2026-11-22

Overnight autoresearch (Track B) kept shipping operator instant lanes. Digester open stayed empty; design-review stayed in grace (Mac TCC); layout-daily already marked data-poster for the day.

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.2953** | Primary / default-route iface `/iface` · `/ifname` · `/interface` + view/open NL |
| v0.1.2952 | Primary IPv4 netmask `/netmask` · `/subnet` · `/cidr` + view/open NL |
| v0.1.2951 | Primary interface MTU `/mtu` · `/if-mtu` · `/interface-mtu` + view/open NL |
| v0.1.2950 | Primary MAC `/mac-address` · `/ether` · `/en0-mac` + view/open NL |
| v0.1.2949 | Wi-Fi SSID `/wifi` · `/ssid` + view/open NL |
| v0.1.2948 | DNS nameservers `/dns` · `/nameserver` + view/open NL |
| v0.1.2947 | Default gateway `/gateway` · `/router` + view/open NL |
| earlier | ip · boot · cores · kernel · locale · arch · tz · serial · username · mac-model · … |

## This tick (~04:58 CET)

- Fuel: standing product after netmask — default-route iface name was the missing twin of `/ip` · `/mac-address` · `/mtu` · `/netmask` (those already print `(iface)` but had no dedicated lane).
- Verify OK → keep @ `b15dd73` → push `main`.
- Bare `iface` / `interface` stay with the agent; `/iface-mtu` stays MTU; `/en0` stays LAN IP.

## Still open

- Dark / Neon / Futuristic layout-daily next calendar day
- cream/Swiss Send fill white-on-blue intentional (leave alone)
- sibling harness ports when digester stays empty
