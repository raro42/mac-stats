# Morning surprise — 2026-09-18

Overnight Track B (20:00–06:00 local, continuing from 2026-09-17 evening). Digester open empty. Design review not due. Soft-parity scan found Perplexity empty-error still on louder 8%/35%.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1143** | Perplexity empty-error soft parity — empty-error pane soft alert at 7% / 30% border (Monitors Down / Perplexity has-error); louder 8%/35% tint removed |
| **v0.1.1142** | AI Chat error-bubble soft parity — Assistant error message bubbles soft alert at 7% / 30% border (Errors glance / Monitors Down); louder 8%/28% tint removed |
| **v0.1.1141** | Ops Runs Fail/Slow list-row soft parity — Fail and Slow run rows soft alert at 7% / 30% border (Runs has-fail / has-slow); louder 8%/22% tint removed |
| **v0.1.1140** | Disk Cleanup Big list-row soft parity — Big reclaim rows soft alert at 7% / 30% border (Big attention / Monitors Slow); louder 8%/34% tint removed |
| **v0.1.1139** | Ollama collapsed has-errors + Settings Signal not-wired soft parity — soft alert at 7% / 30% border (Monitors Down / AI Chat Errors); louder 8%/34% tint removed |
| **v0.1.1138** | AI Chat Errors / last-answer has-errors soft parity — Errors glance + last-answer has-errors soft alert at 7% / 30% border (Monitors Down / Debug Log); louder 8%/34% tint removed |
| **v0.1.1137** | Debug Log has-errors soft parity — Error attention + collapsed error glances soft alert at 7% / 30% border (Monitors Down); louder 8%/34% tint removed |
| **v0.1.1136** | Ops MCP/Cursor/Perplexity/Mastodon/Telegram/Slack/Signal soft parity — Not-set / warn / bad attention glances soft alert at 7% / 30% border (Discord Offline / Reconnect); louder 8%/34% tint removed |
| **v0.1.1135** | Ops Redmine/Ollama/Brave/Browser soft parity — Not-set / warn / bad attention glances soft alert at 7% / 30% border (Discord Offline / Reconnect); louder 8%/34% tint removed |
| **v0.1.1134** | Ops Digest open soft parity — Digest open attention glance soft alert at 7% / 30% border (Monitors Slow / Discord Reconnect); louder 8%/34% tint removed |
| **v0.1.1133** | Ops Discord Offline/Reconnect soft parity — Offline / Reconnect attention glances soft alert at 7% / 30% border (Monitors Down / Slow); louder 8%/34% tint removed |
| **v0.1.1132** | Perplexity panel has-error soft parity — error attention glance + last-search has-error soft alert at 7% / 30% border (Monitors Down / Key-not-set); louder 8%/34% (last-glance 8%/35%) tint removed |
| **v0.1.1131** | Perplexity panel key-not-set soft parity — Key-not-set attention glance soft alert at 7% / 30% border (Monitors Down / Settings credentials Not-set); louder 8%/34% tint removed |
| **v0.1.1130** | Settings credentials key-not-set soft parity — Discord / Perplexity / Brave / Redmine / Mastodon / MCP / Browser / Cursor / Telegram / Slack not-set (and partial) glances soft alert at 7% / 30% border (Monitors Down / Chat Offline Not-set); louder 8%/34% tint removed |
| **v0.1.1129** | AI Chat offline/no-model soft parity — empty offline / no-model / circuit panes, model glance, and Offline · No model · Errors · Not-set attention soft alert at 7% / 30% border (Monitors Slow / Down); louder 8%/34% (circuit 10%/40%) tint removed |

## Why it matters

Perplexity empty-error panes now match the has-error glance wash (7% fill, 30% border) — the empty pane no longer shouts louder than the glance.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (feature-agent-ops ~2d; Processes / Monitors; recapture AI Chat when Screen Recording TCC allows)
- Soft-parity scan leftovers: changelog-error / monitors-empty error panes (8%/35%); sibling Hermes/OpenClaw ports with clear user fitness
