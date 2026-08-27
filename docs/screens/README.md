# Screenshots & themes

Screenshots and a short demo reel for [mac-stats](https://github.com/raro42/mac-stats/).

**Privacy:** Capture the **mac-stats window only** (or a dedicated feature page). Never full-desktop grabs — they can include other apps and sensitive content.

## Theme gallery

| Apple | Architect | Data Poster |
|-------|-----------|-------------|
| <img src="apple.png" alt="Apple" width="280"> | <img src="architect.png" alt="Architect" width="280"> | <img src="data-poster.png" alt="Data Poster" width="280"> |
| **Dark (TUI)** | **Futuristic** | **Light** |
| <img src="dark-tui.png" alt="Dark TUI" width="280"> | <img src="futuristic.png" alt="Futuristic" width="280"> | <img src="light.png" alt="Light" width="280"> |
| **Material** | **Neon** | **Swiss Minimalistic** |
| <img src="material.png" alt="Material" width="280"> | <img src="neon.png" alt="Neon" width="280"> | <img src="swiss-minimalistic.png" alt="Swiss" width="280"> |

## Feature screenshots

| CPU metrics | Website monitors |
|-------------|------------------|
| <img src="feature-cpu-metrics.png" alt="CPU metrics" width="280"> | <img src="feature-monitors.png" alt="Monitors" width="280"> |
| **Disk Cleanup** | **Agent Ops** |
| <img src="feature-disk-cleanup.png" alt="Disk Cleanup" width="280"> | <img src="feature-agent-ops.png" alt="Agent Ops" width="280"> |
| **AI chat (Ollama)** | **Top processes** |
| <img src="feature-ai-chat.png" alt="AI chat" width="280"> | <img src="feature-processes.png" alt="Processes" width="280"> |

### Short demo video

[Play demo (~49s)](https://cdn.jsdelivr.net/gh/raro42/mac-stats@main/docs/screens/mac-stats-features.mp4) · [mac-stats-features.mp4](mac-stats-features.mp4) — **live** window-only capture of the running app:

- ScreenCaptureKit recording of the CPU window (not a slideshow of stills)
- Walkthrough: live metrics → website monitors → Agent Ops → Ollama chat → back to metrics
- Letterboxed to 1920×1080; neural voiceover + ambient bed
- Repo: [github.com/raro42/mac-stats](https://github.com/raro42/mac-stats/)

Also linked from the [project README](../../README.md#demo-video).

## Star growth chart

[`star-history.svg`](star-history.svg) — cumulative GitHub stars for the README / landing banner.

**Daily refresh:** the overnight harness (~23:00) runs `scripts/overnight_git_flush.py`, which calls `scripts/generate_star_history_svg.py`. The SVG updates only when the star list changes, then the flush commits and pushes if the tree is dirty.

Manual:

```bash
python3 scripts/generate_star_history_svg.py
```

We keep a local SVG because the public Star History embed needs a sealed token after GitHub restricted stargazer API access (2026).

## How to capture (window-only)

1. Open the CPU window: `mac_stats --cpu` (or click the menu bar item).
2. **Wait at least 30 seconds** with the window open before capturing so the history graphs (temperature / usage / frequency sparklines) have enough samples and look filled-in. Do not shoot on a cold open.
3. Prefer **window capture**, not display capture:
   - ScreenCaptureKit / `screencapture -l <windowid>` for the mac-stats window only
   - Do **not** use `screencapture -D` (full display) for marketing assets
4. Optional helper for ad-hoc local shots: `./scripts/take-screenshot.sh` (full screen — avoid for repo assets).

## Refresh log

- **2026-08-27 (~23:10):** Agent Ops keep-header + Discord Ready glance toolbar chain (v0.1.683). Recapture of `feature-agent-ops.png` deferred if Screen Recording TCC / no on-screen CPU window; prior Aug 12 asset kept; polish grace marked.
- **2026-08-27 (~22:06):** Disk Cleanup keep-header (v0.1.681). Recapture of `feature-disk-cleanup.png` deferred if Screen Recording TCC / no on-screen CPU window; prior Aug 13 asset kept; polish grace marked.
- **2026-08-26 (~05:25):** AI Chat empty title + starter-chip toolbar keyboard (v0.1.650). Recapture of `feature-ai-chat.png` attempted; if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`, prior Aug 14 asset kept; polish grace marked.
- **2026-08-18 (~21:20):** CPU metrics RAM on the battery/power strip (v0.1.520). Recapture of `feature-cpu-metrics.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior asset kept; polish grace marked.
- **2026-08-18 (~20:55):** Agent Ops empty Open AI Chat CTA (v0.1.519). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior Aug 12 asset kept; polish grace marked.
- **2026-08-18 (~20:30):** Monitors summary click + empty Add CTA (v0.1.518). Recapture of `feature-monitors.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior asset kept; polish grace marked.
- **2026-08-18 (~20:00):** Battery / power strip click-to-copy (v0.1.517). Recapture of `feature-cpu-metrics.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior asset kept; polish grace marked.
- **2026-08-18 (~05:30):** AI Chat empty-state starter chips (v0.1.514). Recapture of `feature-ai-chat.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior asset kept; polish grace marked.
- **2026-08-18 (~05:00):** CPU metrics click-to-copy ring values (v0.1.513). Recapture of `feature-cpu-metrics.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior asset kept; polish grace marked.
- **2026-08-18 (~04:30):** Disk Cleanup click-to-copy path (v0.1.512). Recapture of `feature-disk-cleanup.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior asset kept.
- **2026-08-18 (~03:35):** Agent Ops `c` copies selected id (v0.1.510). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC / no on-screen CPU window blocks Quartz/`screencapture -l`; prior Aug 12 asset kept.
- **2026-08-18 (~03:10):** Top Processes click-to-copy name (v0.1.509). Recapture of `feature-processes.png` still deferred (no on-screen CPU window for Quartz/`screencapture -l`); prior asset kept.
- **2026-08-18 (~02:40):** Agent Ops 0 Overview jump (v0.1.508). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks; prior Aug 12 asset kept.
- **2026-08-18 (~01:50):** Agent Ops filter-row Clear beside N/M chip (v0.1.506). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks; prior Aug 12 asset kept.
- **2026-08-18 (~00:30):** Agent Ops tab inventory count pills (v0.1.503). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks; prior Aug 12 asset kept.
- **2026-08-17 (~00:05):** Agent Ops Updated … ago stamp beside Refresh (v0.1.502). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks; prior Aug 12 asset kept.
- **2026-08-17 (~23:35):** Agent Ops tab digit badges 1–5 (v0.1.501). Recapture of `feature-agent-ops.png` still deferred (`screencapture -l` → could not create image / Screen Recording TCC); prior Aug 12 asset kept.
- **2026-08-17 (~23:00):** Agent Ops overview cards click/keyboard open linked tab (v0.1.500). Recapture of `feature-agent-ops.png` still deferred (no on-screen CPU window for Quartz/`screencapture -l`).
- **2026-08-17 (~22:35):** Agent Ops overview Recent ok/warn/bad wash (v0.1.499). Recapture of `feature-agent-ops.png` still deferred (no on-screen CPU window for Quartz/`screencapture -l`).
- **2026-08-17 (~05:25):** Agent Ops health Next schedule / Last delivery ok/warn/bad wash (v0.1.491). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks.
- **2026-08-17 (~04:06):** Agent Ops overview Agents card (v0.1.488). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks.
- **2026-08-17 (~03:42):** Agent Ops health Redmine → Redmine agent open (v0.1.487). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks.
- **2026-08-17 (~02:55):** Agent Ops health Version → primary agent open (v0.1.485). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks.
- **2026-08-17 (~02:00):** Agent Ops health Next schedule / Last delivery click-to-preview (v0.1.483). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks.
- **2026-08-17 (~01:35):** Agent Ops Runs Insights Slowest/Candidates click-to-preview (v0.1.482). Recapture of `feature-agent-ops.png` still deferred (no on-screen CPU window / Screen Recording TCC).
- **2026-08-17 (~00:25):** Agent Ops overview Live click-to-preview (v0.1.479). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks.
- **2026-08-17 (~00:01):** Agent Ops overview Recent click-to-preview (v0.1.478). Recapture of `feature-agent-ops.png` still deferred (no on-screen CPU window for Quartz/`screencapture -l`).
- **2026-08-16 (~23:38):** Agent Ops overview Schedules click-to-preview (v0.1.477). Recapture of `feature-agent-ops.png` still deferred (`screencapture -l` → could not create image).
- **2026-08-16 (evening):** Agent Ops Schedules Load into AI Chat (v0.1.474). Runs Load into AI Chat (v0.1.473). Schedules/deliveries id click-to-copy chip (v0.1.472). Runs request-id chip (v0.1.471). Agents slug/id chip (v0.1.469). Runs click-to-preview (v0.1.468). Data Poster inactive icons (v0.1.470). Recapture of `feature-agent-ops.png` still deferred (`screencapture -l` → could not create image).
- **2026-08-16:** Agent Ops Knowledge path click-to-copy (v0.1.453). Agent Ops Schedules/deliveries click-to-preview full task/summary (v0.1.452). Agent Ops Sessions click-to-copy id/slug chip (v0.1.451). Agent Ops health-card active-tab accent ring (v0.1.450). Agent Ops tab true-empty title+hint (v0.1.449). Agent Ops selected-row accent wash (v0.1.447). Agent Ops overview empty Open-tab CTAs (v0.1.446). Top Processes detail PID Copied flash (v0.1.445). Disk Cleanup soft-delete Saved flash (v0.1.444). Agent Ops filter-miss Clear filter (v0.1.443). Recapture of `feature-agent-ops.png` still deferred if Screen Recording TCC blocks `screencapture -l`.
- **2026-08-15:** Agent Ops active-tab accent wash + clearer on/off badges (v0.1.431). Recapture of `feature-agent-ops.png` deferred again — `screencapture -l` TCC (`could not create image`); prior Aug 12 asset kept.
- **2026-08-15:** Agent Ops overview active-tab highlight + health status wash (v0.1.430). Recapture of `feature-agent-ops.png` deferred — agent-session `screencapture -l/-R` hit TCC (`could not create image`); keep prior asset until a permitted capture.
- **2026-08-14:** Recaptured `feature-monitors.png`, `feature-ai-chat.png`, and `dark-tui.png` (window-only, ≥30s warm-up) after Monitors downtime tips (v0.1.394–397) and dark first-paint (v0.1.398). Open section uses `MAC_STATS_OPEN_SECTION`; installed-app theme lives in WebKit `com.raro42.mac-stats` localStorage (not the legacy `mac_stats` folder). Window owner name for `screencapture -l` is **`mac-stats`**.
- **2026-08-13:** Monitors summary up/down accents + row latency polish (v0.1.373); open with `MAC_STATS_OPEN_SECTION=monitors`. Recapture of `feature-monitors.png` deferred if Screen Recording TCC blocks.
- **2026-08-13:** Disk Cleanup reclaim accent + scope/item hover (v0.1.372); open with `MAC_STATS_OPEN_SECTION=disk-cleanup`. Recapture of `feature-disk-cleanup.png` deferred if Screen Recording TCC blocks.
- **2026-08-13:** Top Processes accent bars + pin header (v0.1.371); open with `MAC_STATS_OPEN_SECTION=processes`. Recapture of `feature-processes.png` deferred if Screen Recording TCC blocks.
- **2026-08-13:** AI Chat composer polish (v0.1.370); `feature-ai-chat.png` recapture deferred (Screen Recording TCC). Open AI Chat with `MAC_STATS_OPEN_SECTION=ai-chat`.
- **2026-08-12:** Recaptured `feature-agent-ops.png` and `feature-cpu-metrics.png` (window-only, ≥30s warm-up). Overnight harness now spawns the Cursor `agent` CLI (print-only ticks were a quiet failure). Open Agent Ops for capture with `MAC_STATS_OPEN_SECTION=agent-ops`.
- **2026-08-05:** Added `feature-monitors.png` (window-only) for External / Monitors (up/down, latency, history bars).
- **2026-08-05:** Added `feature-disk-cleanup.png` (window-only) for configurable cleanup scopes (Trash / Downloads / Temp / custom paths) — v0.1.355+.
- **2026-07-28:** Agent Ops empty-state polish shipped (v0.1.261). Window-only recapture of `feature-agent-ops.png` deferred — agent-session `screencapture` returned black frames (Screen Recording / TCC); keep prior asset until a permitted capture.
- **2026-07-26:** Recaptured `feature-cpu-metrics.png` (window-only, post SSD menu-bar / v0.1.257+).
