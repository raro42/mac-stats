# Morning surprise — 2026-11-22

Overnight autoresearch (Track B) kept shipping operator instant lanes. Digester open stayed empty; design-review stayed in grace (Mac TCC); layout-daily already marked data-poster for the day.

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.2950** | Primary MAC `/mac-address` · `/ether` · `/en0-mac` + view/open NL |
| v0.1.2949 | Wi-Fi SSID `/wifi` · `/ssid` + view/open NL (parallel tick) |
| v0.1.2948 | DNS nameservers `/dns` · `/nameserver` + view/open NL |
| v0.1.2947 | Default gateway `/gateway` · `/router` + view/open NL |
| v0.1.2946 | Local LAN IPv4 `/ip` · `/lan-ip` + view/open NL |
| earlier | boot · cores · kernel · locale · arch · tz · serial · username · mac-model · hostname · macos · ssd-size · … |

## This tick (~04:15 CET)

- Fuel: standing product after remote Wi-Fi SSID — MAC was the missing link-layer twin of `/ip` · `/gateway` · `/dns`.
- Verify OK → keep @ `6997c3a71` → push `main`.
- `/wifi-id` intentionally **not** used (SSID owns wifi namespace).

## Still open

- Dark / Neon / Futuristic layout-daily next calendar day
- cream/Swiss Send fill white-on-blue intentional (leave alone)
- sibling harness ports when digester stays empty
