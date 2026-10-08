# Morning surprise — 2026-10-09

Overnight Track B kept shipping Apple theme opaque keyboard-hint type for GitHub #14 (WebView idle / glass alpha out of always-visible chrome).

## Shipped tonight

| Version | What |
|--------|------|
| **v0.1.1781** | Monitors add-form toolbar kb-hint (`.monitor-add-toolbar-kb-hint`) opaque `color-mix` on the white add-form panel |
| **v0.1.1780** | Ollama settings toolbar kb-hint (`.ollama-settings-toolbar-kb-hint`) opaque `color-mix` on the white settings card |
| **v0.1.1779** | Ollama settings header kb-hint (`.ollama-settings-header-kb-hint`) opaque `color-mix` on the white settings card |
| **v0.1.1778** | Settings header kb-hint (`.settings-header-kb-hint`) opaque `color-mix` on the white settings card |
| **v0.1.1777** | Credentials section kb-hint opaque |
| **v0.1.1776** | Product setting kb-hint opaque |
| **v0.1.1775** | Appearance setting kb-hint opaque |
| **v0.1.1773** | Theme-list kb-hint opaque |
| **v0.1.1771** | Slack settings toolbar kb-hint opaque |

## Why it matters

Glass `opacity` on keyboard hints still forced compositor blending on Settings and Monitors chrome. Opaque `color-mix` keeps the same look without that alpha.

## Next

- `.disk-cleanup-toolbar-kb-hint` (same pattern)
- Refresh `feature-cpu-metrics` / `feature-agent-ops` screenshots when a Mac is reachable (design-review grace)

## Digester

Open candidates: empty. Fuel was standing backlog P2 / #14.
