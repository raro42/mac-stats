# Morning surprise — 2026-09-02

## Shipped overnight

**v0.1.797 — Agent Ops Telegram health attention glance**

When Telegram alerts are not fully configured, Agent Ops now shows an amber attention strip under Mastodon:

- **Telegram · Not set · add bot token + chat id** when both fields are missing
- **Telegram · Partial · missing chat id** or **missing bot token** when only one is set

Click opens Settings → Credentials and focuses the missing field. Backend adds a config-only Telegram probe to `get_feature_health` (`/telegram` instant-lane parity).

**v0.1.796 — Agent Ops Mastodon health attention glance** (earlier tick)

When Mastodon is not fully configured, Agent Ops shows an amber strip under Perplexity with the same click-to-Settings pattern.

## Context

- Digester had no open candidates (9 turns, all instant/direct noise filtered).
- Design review not due (grace on stale screenshots).
- Debug log clean in the last 3h (1 non-product line filtered).

## Next up

- Agent Ops Slack health chain (after Telegram).
- Recapture `feature-ai-chat` screenshot when TCC allows.
- p50 / sibling ports as backlog fuel.
