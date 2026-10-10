# Morning surprise — 2026-10-10

Overnight autoresearch kept shipping Apple theme opaque keyboard-hint type (#14 / WebView idle).

## Latest keep

**v0.1.1819** — Apple Debug Log toolbar keyboard hint (`.logs-toolbar-kb-hint`) mixes type against opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Refresh · Open in editor · Auto-refresh move hint.

## Tonight so far (keeps)

- v0.1.1819 logs-toolbar-kb-hint
- v0.1.1818 perplexity-search-kb-hint
- v0.1.1817 perplexity-kb-hint
- v0.1.1816 chat-kb-hint
- v0.1.1815 chat-composer kb-hint
- v0.1.1814 chat-empty kb-hint
- v0.1.1813 ops-insights kb-hint
- v0.1.1812 ops-preview-row kb-hint
- v0.1.1811 ops-agent-edit-actions kb-hint
- v0.1.1810 ops-file-tab kb-hint
- (earlier: ops-refresh / filter / overview / health / tab-bar · changelog · process details · header/footer · sparklines · rings · power strip · …)

## Tried / raced

- Local perplexity-search-kb-hint experiment raced with another tick that already shipped **v0.1.1818**; continued with logs-toolbar hint.

## Next fuel

`.logs-kb-hint` / `.perplexity-setup-kb-hint` still glass opacity. Dark layout daily next day; capture feature-cpu-metrics when Mac is reachable.

## Context

- Digester open empty; design review in grace; layout daily data-poster already marked for 2026-10-10.
- Sibling OpenClaw/Hermes git missing on this host.
- Debug.log: no ERROR/WARN clusters in scan window.
- Written 2026-10-10 05:51 CEST.
