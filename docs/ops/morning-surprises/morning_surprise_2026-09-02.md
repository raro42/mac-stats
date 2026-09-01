# Morning surprise — 2026-09-02

## Shipped overnight

**v0.1.798 — Agent Ops Slack health attention glance**

When Slack alerts are not fully configured, Agent Ops now shows an amber attention strip under Telegram:

- **Slack · Not set · add webhook URL** when no webhook is saved
- **Slack · Partial · save again to register** when webhook exists but channel is not registered
- **Slack · Partial · missing webhook** when channels are registered without a webhook

Click opens Settings → Credentials and focuses the Slack webhook field. Backend adds a config-only Slack probe to `get_feature_health` (`/slack` instant-lane parity).

**v0.1.797 — Agent Ops Telegram health attention glance** (earlier tick)

When Telegram alerts are not fully configured, Agent Ops shows an amber strip under Mastodon with the same click-to-Settings pattern.

**v0.1.796 — Agent Ops Mastodon health attention glance** (earlier tick)

Mastodon Not set/Partial strip under Perplexity with Settings focus.

## Context

- Digester had no open candidates (9 turns, all instant/direct noise filtered).
- Design review not due (grace on stale screenshots).
- Debug log clean in the last 3h (1 non-product line filtered).

## Next up

- Agent Ops Signal health chain (honest Not wired placeholder).
- Recapture `feature-ai-chat` screenshot when TCC allows.
- p50 / sibling ports as backlog fuel.
