# Morning surprise — 2026-08-15

Overnight Track B (20:00–06:00 local, window opened 2026-08-14).

## Shipped

| Version | What |
|---------|------|
| **v0.1.401** | Top Processes: j/k selection + Esc clears; `P` pin/unpin; keyboard hint (Monitors / Disk Cleanup parity) |
| **v0.1.402** | Monitors: `d` toggles detail panel; Esc closes open detail before clearing selection; hint + tooltips |
| **v0.1.403** | Top Processes: PageUp/PageDown jump ~5 rows; `d` opens details; Esc closes details modal before clearing selection |
| **v0.1.404** | Monitors: PageUp/PageDown jump ~5 rows (Top Processes / Agent Ops parity); hint + tooltips |
| **v0.1.405** | Disk Cleanup: PageUp/PageDown jump ~5 rows on scopes and categories; hints + tooltips |
| **v0.1.406** | Disk Cleanup soft-delete: skip file when Trash move fails (no permanent-delete fallback on EPERM) |
| **v0.1.407** | Disk Cleanup last-run note counts soft-delete skips when Trash move fails (files left in place) |
| **v0.1.408** | AI Chat Send: busy-guard (Sending…) blocks double Enter/click; brief Sent flash on success |
| **v0.1.409** | AI Chat Clear button next to Send + Cleared flash; disabled while empty or Sending… |
| **v0.1.410** | Perplexity Search: busy-guard (Searching…) blocks double click/Enter; Searched flash; Enter runs search |
| **v0.1.411** | Agent Ops Refresh / Refresh digest: busy-guard (Refreshing…) + Refreshed flash; auto-refresh silent |
| **v0.1.412** | Disk Cleanup Refresh: busy-guard (Refreshing…) + Refreshed flash; Clean now flashes Cleaned after success; Refresh disabled during cleanup |
| **v0.1.413** | Monitors Check now: busy-guard (Checking…) + Checked flash; blocks double Enter/click while in flight |
| **v0.1.414** | Logs Refresh: busy-guard (Refreshing…) + Refreshed flash; auto-refresh stays silent |
| **v0.1.415** | Discord Save/Clear token: busy-guard (Saving…/Clearing…) + Saved/Cleared flash; blocks double click |
| **v0.1.416** | Agent Ops Save (soul/skill/mood): busy-guard (Saving…) + Saved flash; blocks double click |
| **v0.1.417** | AI Chat system-prompt Save: busy-guard (Saving…) + Saved flash; popover stays open so confirmation is visible |
| **v0.1.418** | Monitors Add Monitor: busy-guard (Adding…) + Added flash; form stays open for confirmation, then closes |
| **v0.1.419** | Perplexity Save/Clear key: busy-guard (Saving…/Clearing…) + Saved/Cleared flash (Settings + inline setup); blocks double click |
| **v0.1.420** | Disk Cleanup Save scopes: busy-guard (Saving…) + Saved flash; blocks double click and ⌘/Ctrl+S while in flight |
| **v0.1.421** | Monitors: Delete/Backspace removes selected; detail Remove with Removing… busy-guard (Settings Remove shared) |
| **v0.1.422** | Disk Cleanup Add scope: busy-guard (Adding…) + Added flash; blocks double click and Enter while in flight |
| **v0.1.423** | Logs Open in editor: busy-guard (Opening…) + Opened flash; blocks double click while in flight |
| **v0.1.424** | Force Quit Process: busy-guard (Quitting…) while quit runs; brief Quit flash; blocks double confirm/click |

## Design review

- Not due at 20:00–05:55 ticks (feature screens ok/grace; ai-chat ~0.72d).
- Next due surface: wait for age >3d or re-shoot when TCC allows.

## Digester

- Open candidates: none (latency n/a after noise filters).
