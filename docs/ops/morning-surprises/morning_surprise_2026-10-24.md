# Morning surprise — 2026-10-24

Overnight Track B kept shipping Data Poster opaque washes (standing P2 / GitHub #14). Digester stayed empty of Slowest turns; design review stayed due on stale feature screens — this night polished **Settings** toolbars and **Agent Ops** health type.

## Shipped last night

| Version | What got nicer |
|---------|----------------|
| **v0.1.2253** | Agent Ops health labels/values + Refresh row tip (`.ops-health-label` · `.ops-health-value` · `.ops-refresh-row-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`; refresh tip drops the light Apple wash) so the health strip and Refresh · Refresh digest stay readable |
| **v0.1.2252** | Settings toolbar keyboard hints (`.settings-header-kb-hint` · `.modal-header-kb-hint` · Discord·Perplexity·Brave·Redmine·Mastodon·MCP·Browser·Cursor·Telegram·Slack toolbars · `.theme-list-kb-hint` · `.appearance-setting-kb-hint` · `.product-setting-kb-hint` · `.credentials-section-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so tips under Settings title, credential rows, and Appearance / Product match Details / Processes / Ollama hints |
| **v0.1.2251** | Details / Processes keyboard hints (`.details-kb-hint` · `.processes-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so tips under the Details grid and Top Processes list match Monitors / Disk Cleanup hints |
| **v0.1.2250** | Monitors keyboard hints (`.monitors-kb-hint` · `.monitor-add-toolbar-kb-hint` · `.monitor-detail-toolbar-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so tips under the monitor list, Add form, and Check now · Remove match Disk Cleanup / window chrome hints |
| **v0.1.2249** | Disk Cleanup keyboard hints mix on shell `#0b0b10` (no early `#0e0e14`) |
| **v0.1.2248** | Window chrome keyboard hints mix on shell `#0b0b10` |

## Design review

- Helper said **due** — recommended `feature-agent-ops.png` (~38d stale); other feature screens also stale or in grace.
- Parallel ticks shipped Settings toolbar hints (v0.1.2252) and Agent Ops health type + Refresh tip (v0.1.2253). Window-only Mac recapture still deferred when `screencapture -l` / Screen Recording TCC is unavailable on this host.

## Next fuel

- Process-detail / force-quit / Debug Log / Perplexity kb-hints still on early `#0e0e14`.
- Mac capture of stale `feature-agent-ops.png` when TCC allows.
