# Morning surprise — 2026-10-10

Overnight Track B (autoresearch) kept shipping Apple keyboard-hint opacity cuts for GitHub **#14**.

## Shipped this night (latest first)

- **v0.1.1813** — Agent Ops Insights keyboard hint (`.ops-insights-kb-hint`) opaque `color-mix` type on the shell fill. No glass `opacity` on the Insights move hint.
- **v0.1.1812** — Agent Ops preview-row keyboard hint (`.ops-preview-row-kb-hint`) opaque type (Copy · Load into AI Chat).
- **v0.1.1811** — Agent Ops edit-actions keyboard hint (`.ops-agent-edit-actions-kb-hint`) opaque type (Save · Load · Back).
- **v0.1.1810** — Agent Ops file-tab keyboard hint (`.ops-file-tab-kb-hint`) opaque type (Soul · Skill · Mood).
- **v0.1.1809** — Agent Ops refresh-row keyboard hint opaque type.
- **v0.1.1808** — Agent Ops filter-row keyboard hint opaque type.
- **v0.1.1807** — Agent Ops overview keyboard hint opaque type.
- **v0.1.1806** — Agent Ops health-strip keyboard hint opaque type.

## Why it matters

Glass `opacity` on hint text still pulls the WebView compositor. Opaque `color-mix` keeps the same look without the alpha layer.

## Next fuel

- `.chat-empty-kb-hint` (then chat-composer / chat / perplexity / logs kb-hints) still use shared `opacity: 0.72`.
- Design-review screens still in grace (feature-agent-ops stale ~24d) — capture when a Mac is reachable.
- Layout daily: Dark / Neon still due on later calendar days.

_Updated: 2026-10-10 04:11 CEST_
