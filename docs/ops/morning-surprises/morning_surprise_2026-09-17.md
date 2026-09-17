# Morning surprise — 2026-09-17

Overnight Track B (20:00–06:00 local). Digester open empty. Design review not due. Standing backlog / Next → Ops MCP/Cursor/Perplexity/Mastodon/Telegram/Slack/Signal soft parity.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1136** | Ops MCP/Cursor/Perplexity/Mastodon/Telegram/Slack/Signal soft parity — Not-set / warn / bad attention glances soft alert at 7% / 30% border (Discord Offline / Reconnect); louder 8%/34% tint removed |
| **v0.1.1135** | Ops Redmine/Ollama/Brave/Browser soft parity — Not-set / warn / bad attention glances soft alert at 7% / 30% border (Discord Offline / Reconnect); louder 8%/34% tint removed |
| **v0.1.1134** | Ops Digest open soft parity — Digest open attention glance soft alert at 7% / 30% border (Monitors Slow / Discord Reconnect); louder 8%/34% tint removed |
| **v0.1.1133** | Ops Discord Offline/Reconnect soft parity — Offline / Reconnect attention glances soft alert at 7% / 30% border (Monitors Down / Slow); louder 8%/34% tint removed |
| **v0.1.1132** | Perplexity panel has-error soft parity — error attention glance + last-search has-error soft alert at 7% / 30% border (Monitors Down / Key-not-set); louder 8%/34% (last-glance 8%/35%) tint removed |
| **v0.1.1131** | Perplexity panel key-not-set soft parity — Key-not-set attention glance soft alert at 7% / 30% border (Monitors Down / Settings credentials Not-set); louder 8%/34% tint removed |
| **v0.1.1130** | Settings credentials key-not-set soft parity — Discord / Perplexity / Brave / Redmine / Mastodon / MCP / Browser / Cursor / Telegram / Slack not-set (and partial) glances soft alert at 7% / 30% border (Monitors Down / Chat Offline Not-set); louder 8%/34% tint removed |
| **v0.1.1129** | AI Chat offline/no-model soft parity — empty offline / no-model / circuit panes, model glance, and Offline · No model · Errors · Not-set attention soft alert at 7% / 30% border (Monitors Slow / Down); louder 8%/34% (circuit 10%/40%) tint removed |
| **v0.1.1128** | Settings product attention soft parity — AI Off / Compact On / Judge Off / Downloads Off / Ori Off / Having-fun Off / Voice STT Off glances soft alert at 7% / 30% border (Monitors Slow / Help open soft); louder 10%/38% tint removed |
| **v0.1.1127** | Disk Cleanup last-run has-skip soft parity — Last run panel with skips soft alert at 7% / 30% border (Monitors Slow / reclaim); louder 10%/28% tint removed; clean last-run green stays soft |
| **v0.1.1126** | Disk Cleanup periodic-off soft parity — Runs when meta-card soft accent at 7% / 28% border (Ops Off / Files / Deliveries / Discord); louder 8%/34% cyan tint removed; scopes-off already soft |
| **v0.1.1125** | Disk Cleanup reclaim soft parity — Reclaimable now + Next run due meta-cards soft alert at 7% / 30% border (Monitors Slow / Big); louder 8%/34% tint removed; attention reclaim glances stay slightly softer |
| **v0.1.1124** | Ops Runs has-fail/has-slow soft parity — Fail · N / Slow · N attention glances soft alert at 7% / 30% border (Fail/Slow filter); louder 8%/34% tint removed |
| **v0.1.1123** | Monitors summary/collapsed has-down soft parity — summary + collapsed Down glances soft alert at 7% / 30% border (Down attention / Down filter); louder 8%/34% tint removed |
| **v0.1.1122** | Monitors Down/Slow attention soft parity — Down · N / Slow · N glances soft alert at 7% / 30% border (Down/Slow filter); louder 8%/34% tint removed |
| **v0.1.1121** | Disk Cleanup has-big attention soft parity — Disk · Big glance soft alert at 7% / 30% border (Monitors Slow / Big filter); louder 8%/34% tint removed |
| **v0.1.1120** | Processes Hot attention soft parity — Hot · N hot glance soft alert at 7% / 30% border (Monitors Slow); louder 8%/34% tint removed |
| **v0.1.1119** | Disk Cleanup Big filter attention soft parity — Disk · Big glance soft alert at 7% / 30% border (Monitors Slow); louder 8%/34% tint removed; Reclaim stays softer; Clean stays green |
| **v0.1.1118** | Ops Slow/Fail filter attention soft parity — Slow / Fail glances soft alert at 7% / 30% border (Monitors Slow / Down); louder 8%/34% tint removed |
| **v0.1.1117** | Ops accent filter attention soft parity — Off / Files / Deliveries / Discord glances soft accent at 7% / 28% border (Ready / `.is-filter`); louder 8%/34% tint removed |
| **v0.1.1116** | Ops filter attention soft parity — On / Live / Jobs / Core / Instant / Lite / Direct glances soft green at 7% / 28% border (Ready / Monitors Up); louder 8%/34% tint removed |
| **v0.1.1115** | Disk Cleanup due attention soft parity — Disk · Due glance soft green at 7% / 28% border (Ready calm / collapsed is-due); louder 8%/34% tint removed |

## Why it matters

MCP / Cursor / Perplexity / Mastodon / Telegram / Slack / Signal feature-health glances now match Discord Offline / Reconnect soft washes (7% fill, 30% border) — less shouty when a key is missing or a service is degraded, still clearly actionable.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (feature-agent-ops ~2d; Processes / Monitors; recapture AI Chat when Screen Recording TCC allows)
- Soft-parity leftovers: other louder attention washes still at 8%/34% or 10%/38% if any remain; or sibling Hermes/OpenClaw ports with clear user fitness
