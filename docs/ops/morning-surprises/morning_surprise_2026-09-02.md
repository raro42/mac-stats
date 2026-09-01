# Morning surprise — 2026-09-02

## Shipped overnight

**v0.1.800 — Instant lane: next schedule / next job**

Short asks like `next schedule`, `when is the next job`, and `what's the next schedule` now return the upcoming fire time without Ollama (Discord + AI Chat). Matches Agent Ops **Next schedule** health card parity. Cuts p50 direct latency for common operator schedule checks.

**v0.1.799 — Agent Ops Signal health attention glance**

When Signal alerts are not wired (REST API pending), Agent Ops shows an amber attention strip under Slack:

- **Signal · Not wired · REST API pending** when no Signal channels are registered
- **Signal · Partial · N channel(s) · REST API pending** when channels exist but REST is not wired

Click opens Settings → Credentials and scrolls to the Signal note.

**v0.1.798 — Agent Ops Slack health attention glance** (earlier tick)

Slack Not set/Partial strip under Telegram with Settings webhook focus.

**v0.1.797 — Agent Ops Telegram health attention glance** (earlier tick)

Telegram Not set/Partial strip under Mastodon with Settings focus.

## Context

- Digester had no open candidates (9 turns, all instant/direct noise filtered).
- Design review not due (grace on stale screenshots).
- Debug log clean in the last 3h.

## Next up

- More p50 instant patterns (tool-heavy asks still on direct lane).
- Recapture `feature-ai-chat` screenshot when TCC allows.
- Sibling ports (Hermes insights / session UX) as backlog fuel.
