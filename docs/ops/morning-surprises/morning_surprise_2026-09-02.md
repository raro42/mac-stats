# Morning surprise — 2026-09-02

## Shipped overnight

**v0.1.799 — Agent Ops Signal health attention glance**

When Signal alerts are not wired (REST API pending), Agent Ops now shows an amber attention strip under Slack:

- **Signal · Not wired · REST API pending** when no Signal channels are registered
- **Signal · Partial · N channel(s) · REST API pending** when channels exist but REST is not wired

Click opens Settings → Credentials and scrolls to the Signal note. Backend adds a config-only Signal probe to `get_feature_health` (`/signal` instant-lane parity).

**v0.1.798 — Agent Ops Slack health attention glance** (earlier tick)

Slack Not set/Partial strip under Telegram with Settings webhook focus.

**v0.1.797 — Agent Ops Telegram health attention glance** (earlier tick)

Telegram Not set/Partial strip under Mastodon with Settings focus.

## Context

- Digester had no open candidates (9 turns, all instant/direct noise filtered).
- Design review not due (grace on stale screenshots).
- Debug log clean in the last 3h (1 non-product line filtered).

## Next up

- Pivot to `feature-ai-chat` polish or p50 latency patterns.
- Recapture `feature-ai-chat` screenshot when TCC allows.
- Sibling ports (Hermes insights / session UX) as backlog fuel.
