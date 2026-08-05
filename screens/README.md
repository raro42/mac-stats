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

| CPU metrics | Disk Cleanup |
|-------------|--------------|
| <img src="feature-cpu-metrics.png" alt="CPU metrics" width="280"> | <img src="feature-disk-cleanup.png" alt="Disk Cleanup" width="280"> |
| **Agent Ops** | **AI chat (Ollama)** |
| <img src="feature-agent-ops.png" alt="Agent Ops" width="280"> | <img src="feature-ai-chat.png" alt="AI chat" width="280"> |
| **Top processes** | |
| <img src="feature-processes.png" alt="Processes" width="280"> | |

### Short demo video

[mac-stats-features.mp4](mac-stats-features.mp4) — ~49s **live** window-only capture of the running app:

- ScreenCaptureKit recording of the CPU window (not a slideshow of stills)
- Walkthrough: live metrics → website monitors → Agent Ops → Ollama chat → back to metrics
- Letterboxed to 1920×1080; neural voiceover + ambient bed
- Repo: [github.com/raro42/mac-stats](https://github.com/raro42/mac-stats/)

Also linked from the [project README](../README.md#demo-video).

## How to capture (window-only)

1. Open the CPU window: `mac_stats --cpu` (or click the menu bar item).
2. **Wait at least 30 seconds** with the window open before capturing so the history graphs (temperature / usage / frequency sparklines) have enough samples and look filled-in. Do not shoot on a cold open.
3. Prefer **window capture**, not display capture:
   - ScreenCaptureKit / `screencapture -l <windowid>` for the mac-stats window only
   - Do **not** use `screencapture -D` (full display) for marketing assets
4. Optional helper for ad-hoc local shots: `./scripts/take-screenshot.sh` (full screen — avoid for repo assets).

## Refresh log

- **2026-08-05:** Added `feature-disk-cleanup.png` (window-only) for configurable cleanup scopes (Trash / Downloads / Temp / custom paths) — v0.1.355+.
- **2026-07-28:** Agent Ops empty-state polish shipped (v0.1.261). Window-only recapture of `feature-agent-ops.png` deferred — agent-session `screencapture` returned black frames (Screen Recording / TCC); keep prior asset until a permitted capture.
- **2026-07-26:** Recaptured `feature-cpu-metrics.png` (window-only, post SSD menu-bar / v0.1.257+).
