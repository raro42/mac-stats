# Morning surprise — 2026-10-10

Overnight Track B kept real product polish (not digester-empty quiet).

| Version | What shipped |
|---------|----------------|
| **v0.1.1848** | Agent Ops `.ops-detail-meta` — Agents soul/skill/mood path uses opaque `color-mix` (no glass `opacity`). |
| **v0.1.1847** | Agent Ops `.ops-hint` — Schedules · Runs helper lines use opaque `color-mix` (no glass `opacity`). |
| **v0.1.1846** | Disk Cleanup `.disk-cleanup-scope-kind` — scope kind label opaque `color-mix`. |
| **v0.1.1845** | Disk Cleanup `.disk-cleanup-item-policy` · `.disk-cleanup-item-samples` opaque `color-mix`. |
| **v0.1.1844** | Monitors empty summary · collapsed glance opaque `color-mix` (design-review). |
| **v0.1.1843** | Disk Cleanup `.disk-cleanup-item-stat` opaque `color-mix` (design-review). |
| **v0.1.1842** | Agent Ops `.ops-overview-count` opaque `color-mix` (design-review). |

## Why it matters

Apple theme text that sat on `opacity` still forced WebView compositing (#14). Opaque `color-mix` against shell/card fills keeps the same look without glass alpha.

## Next fuel

`.ops-agent-save-status`, `.ops-keyboard-hint` (same #14 ratchet). `.ops-filter-caption` is unused CSS — skip unless wired into the DOM.
