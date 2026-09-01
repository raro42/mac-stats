# Morning surprise — 2026-09-02

## Shipped overnight

**v0.1.796 — Agent Ops Mastodon health attention glance**

When Mastodon is not fully configured, Agent Ops now shows an amber attention strip under Perplexity:

- **Mastodon · Not set · add URL + token** when both fields are missing
- **Mastodon · Partial · missing access token** or **missing instance URL** when only one is set

Click opens Settings → Credentials and focuses the missing field. Backend adds a config-only Mastodon probe to `get_feature_health` (`/mastodon` instant-lane parity).

## Context

- Digester had no open candidates (9 turns, all instant/direct noise filtered).
- Design review not due (grace on stale screenshots).
- Debug log clean in the last 3h (1 non-product line filtered).

## Next up

- Agent Ops Telegram/Slack health chain (after Mastodon).
- Recapture `feature-ai-chat` screenshot when TCC allows.
- p50 / sibling ports as backlog fuel.
