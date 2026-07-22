# Dashboard UI — orphaned (quarantine)

**Status:** Not loaded by the live menu-bar app. Kept in `src/dashboard.*` until unique flows are ported.

## Why

The CPU window is created as `WebviewUrl::App("cpu.html")` → `themes/<theme>/cpu.html` ([`status_bar.rs`](../src-tauri/src/ui/status_bar.rs)). `dashboard.html` is never opened.

Install (`scripts/install-to-applications.sh`) **no longer copies** `dashboard.html` / `.js` / `.css`.

## Port inventory (before hard delete)

| Dashboard Settings tab | Live equivalent today | Gap |
|------------------------|----------------------|-----|
| Monitors | CPU monitors popover + list | Mostly covered |
| Alert channels | None in CPU UI | **Must port** before delete |
| Schedules | Agent Ops → Schedules (list); add/remove hinted as “Settings → Schedules” | **Add/remove UI** still dashboard-only |
| Skills | Agent Ops / skill listing elsewhere | Confirm parity |
| Ollama (endpoint/model) | CPU Ollama connection indicator + popover (system prompt) | Endpoint/model Apply may still be richer on dashboard |
| Downloads organizer | None in CPU UI | **Must port** before delete |

CPU gear Settings covers: theme, window frame, AI agent toggle, menu bar compact, Discord/Perplexity credentials.

## Do not

- Wire the menu-bar window to `dashboard.html` again
- Re-add dashboard files to the install copy list without an explicit product decision

## Delete when

All **Gap** rows above are implemented in the CPU window / Agent Ops (or explicitly retired as product).
