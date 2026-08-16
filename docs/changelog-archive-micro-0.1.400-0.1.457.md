# Changelog archive — micro versions 0.1.400–0.1.457

These were **incremental commits** between GitHub Releases **v0.1.399** and **v0.1.458**.
They are not separate GitHub Releases. The user-facing summary is under **[0.1.458]** in `CHANGELOG.md`.

---

## [0.1.457] - 2026-08-16

### Fixed
- Dark (TUI): drop `height: 100%` on `.glass-panel` so the green panel border follows content height instead of cutting mid-window.

## [0.1.456] - 2026-08-16

### Fixed
- Disk Cleanup Temp preview no longer lists root-owned / immutable files (e.g. stuck Microsoft AutoUpdate `.pkg` in `/var/folders/…/T`) that Clean now cannot remove.

## [0.1.455] - 2026-08-16

### Fixed
- Dark (TUI) Disk Cleanup: meta cards stretch to equal height so short “Enabled scopes” no longer leaves a broken green border line; softer hairline borders on those cards.

## [0.1.454] - 2026-08-16

### Changed
- CPU window scrollbars: nearly invisible (2px, transparent thumb until hover) across themes and nested panes.

## [0.1.453] - 2026-08-16

### Changed
- Agent Ops Knowledge: preview shows a click-to-copy file path chip with a Copied flash (list + overview; Esc clears).

## [0.1.452] - 2026-08-16

### Changed
- Agent Ops Schedules: click a schedule or delivery row to preview the full task or summary (Esc dismisses; rows still truncate in the list).

## [0.1.451] - 2026-08-16

### Changed
- Agent Ops Sessions: preview shows a click-to-copy session id / file slug chip with a Copied flash (live + saved; overview shortcuts included).

## [0.1.450] - 2026-08-16

### Changed
- Agent Ops design review: health cards highlight with an accent ring when their linked tab is active (parity with overview cards; status wash stays visible).

## [0.1.449] - 2026-08-16

### Changed
- Agent Ops list tabs: true-empty states (schedules, deliveries, agents, sessions, knowledge, runs) use a clear title plus a short hint instead of a one-line stub.

## [0.1.448] - 2026-08-16

### Changed
- Monitors: click (or Enter/Space) a monitor URL to copy it; shows a Copied flash and blocks double copy while flashing (list + detail; survives live refresh).

## [0.1.447] - 2026-08-16

### Changed
- Agent Ops lists: selected rows (↑/↓ · j/k · click) use an accent wash so the keyboard focus is easier to see.

## [0.1.446] - 2026-08-16

### Changed
- Agent Ops overview: empty cards (schedules / live / knowledge / recent) show an Open-tab action so you can jump straight to the right detail tab.

## [0.1.445] - 2026-08-16

### Changed
- Top Processes detail: click (or Enter/Space) the PID to copy it; shows a Copied flash and blocks double copy while flashing (survives live refresh).

## [0.1.444] - 2026-08-16

### Changed
- Disk Cleanup: toggling Move cleaned items to Trash shows a brief Saved flash after a successful save (and blocks double toggle while flashing).

## [0.1.443] - 2026-08-16

### Changed
- Agent Ops: when a list filter has no matches, the empty state shows a Clear filter action and briefly highlights the filter field after clear.

## [0.1.442] - 2026-08-16

### Changed
- Settings product toggles (AI agent, compact menu bar, compact CPU window): after a successful save the label shows a brief Saved flash.

## [0.1.441] - 2026-08-16

### Changed
- Debug Log path hint: click (or Enter/Space) copies the log path and shows a Copied flash; does not toggle the section.

## [0.1.440] - 2026-08-16

### Changed
- Theme / app version label: opening the changelog shows an Opened flash and blocks double click while flashing.

## [0.1.439] - 2026-08-15

### Changed
- Footer: GitHub link shows Opening… then an Opened flash and blocks double click while in flight or flashing.

## [0.1.438] - 2026-08-15

### Changed
- Settings: Help / cheat sheet shows an Opened flash on the control when the sheet opens and blocks double click while flashing.

## [0.1.437] - 2026-08-15

### Changed
- Top Processes: pin/unpin (★ or P) shows a brief Pinned/Unpinned flash on the star and blocks double toggle while flashing.

## [0.1.436] - 2026-08-15

### Changed
- CPU window: header Refresh (↻) spins while fetching, shows a brief ✓ flash, and blocks double click while in flight or flashing.

## [0.1.435] - 2026-08-15

### Changed
- Settings: View logs shows Opening… then an Opened flash on the control and blocks double click while in flight or flashing.

## [0.1.434] - 2026-08-15

### Changed
- AI Chat system-prompt: Reset to Default shows Resetting… then a Reset flash on the control and blocks double click while flashing (popover stays open; Save still persists).

## [0.1.433] - 2026-08-15

### Changed
- Settings: Reset to monitor defaults shows Resetting… then a Reset flash on the control and blocks double click while in flight or flashing.

## [0.1.432] - 2026-08-15

### Changed
- Agent Ops Sessions: Load into AI Chat shows a Loaded flash on the control and blocks double Enter/click/dblclick while flashing.

## [0.1.431] - 2026-08-15

### Changed
- Agent Ops design review: active tabs (and agent file tabs) use the accent wash so the selected tab matches overview cards; on/off badges use clearer green glass.

## [0.1.430] - 2026-08-15

### Changed
- Agent Ops design review: overview cards highlight when their linked tab is active; health ok/warn/bad cards use a soft status wash (Digest open is easier to spot).

## [0.1.429] - 2026-08-15

### Changed
- CPU window open: lower polling (UI 2s, process details 5s, background loop 2s); stop stacking per-process `ioreg` on every GPU gauge tick; process details refreshes only the selected PID (not all processes).

## [0.1.428] - 2026-08-15

### Fixed
- Process Details: keep the Advanced section open (and Force Quit confirm state) across the 2s live refresh.

## [0.1.427] - 2026-08-15

### Fixed
- Top Processes GPU%: use `ioreg -l` for AGX client properties; do not cache an empty first sample (warm-up can complete). Sampler also advances when the GPU gauge refreshes.

## [0.1.426] - 2026-08-15

### Added
- Top Processes: GPU% column (Apple Silicon AGX client time). High-GPU processes join the list even when CPU is low — so 100% GPU is attributable.

## [0.1.425] - 2026-08-15

### Fixed
- Disk Cleanup no longer triggers macOS Downloads/Trash access prompts on every launch: auto runs only clean `~/.mac-stats` data; soft-delete on auto goes to `~/.mac-stats/cleanup-quarantine` (not `~/.Trash`). Status preview skips Downloads/Trash until Refresh or Clean now.

## [0.1.424] - 2026-08-15
### Changed
- **Force Quit Process** — busy-guard (`Quitting…`) while the quit runs; brief `Quit` flash on success; blocks double confirm/click in flight.

## [0.1.423] - 2026-08-15
### Changed
- **Logs Open in editor** — busy-guard (`Opening…`) + `Opened` flash on the control; blocks double click while in flight (Refresh parity).

## [0.1.422] - 2026-08-15
### Changed
- **Disk Cleanup Add scope** — busy-guard (`Adding…`) + `Added` flash on the control; blocks double click and Enter while in flight.

## [0.1.421] - 2026-08-15
### Added
- **Monitors** — `Delete` / `Backspace` removes the selected monitor (Disk Cleanup parity); detail panel **Remove** with `Removing…` busy-guard; Settings Remove shares the same path.

## [0.1.420] - 2026-08-15
### Changed
- **Disk Cleanup Save scopes** — busy-guard (`Saving…`) + `Saved` flash on the control; blocks double click (and ⌘/Ctrl+S) while in flight.

## [0.1.419] - 2026-08-15
### Changed
- **Perplexity Save key / Clear key** — busy-guard (`Saving…` / `Clearing…`) + `Saved` / `Cleared` flash on the control (Settings and inline setup); blocks double click while in flight.

## [0.1.418] - 2026-08-15
### Changed
- **Monitors Add Monitor** — busy-guard (`Adding…`) + `Added` flash on the control; blocks double click; keeps the add form open so the confirmation is visible, then closes.

## [0.1.417] - 2026-08-15
### Changed
- **AI Chat system-prompt Save** — busy-guard (`Saving…`) + `Saved` flash on the control; blocks double click; keeps the settings popover open so the confirmation is visible (no silent close).

## [0.1.416] - 2026-08-15
### Changed
- **Agent Ops Save** (soul/skill/mood) — busy-guard (`Saving…`) + `Saved` flash on the control; blocks double click while in flight (status line still updates).

## [0.1.415] - 2026-08-15
### Changed
- **Discord Save token / Clear token** — busy-guard (`Saving…` / `Clearing…`) + `Saved` / `Cleared` flash on the control; blocks double click while in flight (status line still updates).

## [0.1.414] - 2026-08-15
### Changed
- **Logs Refresh** — busy-guard (`Refreshing…`) + `Refreshed` flash after a manual refresh; blocks double click while in flight. Auto-refresh stays silent.

## [0.1.413] - 2026-08-15
### Changed
- **Monitors Check now** — busy-guard (`Checking…`) + `Checked` flash after a successful manual check; blocks double Enter/click while in flight.

## [0.1.412] - 2026-08-15

### Changed
- Disk Cleanup Refresh: disable while in flight (blocks double click); show Refreshing… then a brief Refreshed flash on success. Clean now: brief Cleaned flash after a successful run (still shows Cleaning… while busy); Refresh stays disabled during cleanup.

## [0.1.411] - 2026-08-15

### Changed
- Agent Ops Refresh and Refresh digest: disable while in flight (blocks double click / `r` / `R`); show Refreshing… then a brief Refreshed flash on success. Auto-refresh stays silent (no flash).

## [0.1.410] - 2026-08-14

### Changed
- Perplexity Search: disable while a search is in flight (blocks double click/Enter); shows Searching… then a brief Searched flash on success. Enter in the query field runs search.

## [0.1.409] - 2026-08-14

### Added
- AI Chat Clear button next to Send: clears the thread and shows a brief Cleared flash; disabled while empty or while a reply is in flight.

## [0.1.408] - 2026-08-14

### Changed
- AI Chat Send: disable while a reply is in flight (blocks double Enter/click); shows Sending… then a brief Sent flash on success.

## [0.1.407] - 2026-08-14

### Changed
- Disk Cleanup last-run note counts soft-delete skips when Trash move fails (files left in place; no silent under-count after Clean now).

## [0.1.406] - 2026-08-14

### Fixed
- Disk Cleanup soft-delete: if move to Trash fails (e.g. EPERM on protected cache files), skip the file instead of permanently deleting it. Soft-delete means recoverable; permanent delete stays an explicit opt-out.

## [0.1.405] - 2026-08-14

### Changed
- Disk Cleanup: PageUp/PageDown jump ~5 rows on scopes and categories (Monitors / Top Processes parity); hints and row tooltips updated.

## [0.1.404] - 2026-08-14

### Changed
- Monitors: PageUp/PageDown jump ~5 rows (Top Processes / Agent Ops parity); hint and row tooltips updated.

## [0.1.403] - 2026-08-14

### Changed
- Top Processes: PageUp/PageDown jump ~5 rows; `d` opens details (Monitors muscle memory); Esc closes the details modal before clearing selection; hint and row tooltips updated.

## [0.1.402] - 2026-08-14

### Changed
- Monitors: `d` toggles the detail panel from the keyboard; Esc closes an open detail before clearing selection; hint and row tooltips updated.

## [0.1.401] - 2026-08-14

### Changed
- Top Processes: j/k selection + Esc clears focus/selection; `P` pins/unpins; keyboard hint (Monitors / Disk Cleanup parity).

## [0.1.400] - 2026-08-14

### Fixed
- CPU window scrollbars: thin, transparent track, thumb only on hover — no thick grey WKWebView bar on Dark (TUI) and other themes.

