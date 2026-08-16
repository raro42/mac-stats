# Getting Started

**Your Mac already knows how busy it is. Now you can too — from the menu bar.**

Apple Silicon only. Optional local AI agent when you want company (off by default, stays on your machine).

## Choose a path

### A) Just the monitor (recommended first)

1. Install (pick one):
   ```bash
   curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/main/install.sh | bash
   ```
   Or Homebrew by hand ([homebrew.md](homebrew.md): tap first — bare `brew install --cask mac-stats` fails), or from a clone: `./scripts/quickstart.sh`
2. Open **mac-stats** — menu bar shows **CPU** (and °C when available).
3. Click the menu bar for the glass window (themes, processes, monitors, Disk Cleanup).

If macOS says the DMG or app is **“damaged”** (also after Homebrew), that is Gatekeeper on an unsigned build — not a bad file. Run:

```bash
xattr -rd com.apple.quarantine /Applications/mac-stats.app
open -a mac-stats
```

Details: [README Quick start](../README.md#if-macos-says-the-dmg--app-is-damaged) and [NOTARIZATION.md](NOTARIZATION.md).

AI features stay **off** until you enable them (`aiAgentEnabled`). Full feature list: [FEATURES.md](../FEATURES.md).

### B) Monitor + AI agent

1. Complete path A.
2. Install Ollama + a model:
   ```bash
   curl -fsSL https://ollama.com/install.sh | sh
   ollama pull llama3.2
   ```
3. Enable AI: **Settings → Enable local AI agent**, or copy [`config.example.json`](../config.example.json) keys into `~/.mac-stats/config.json`.
4. Restart mac-stats (LaunchAgent / quit+open).
5. **First AI query:** open **AI Chat (Ollama)** and ask:
   > What's my CPU temp?

## Config files

| File | Purpose |
|------|---------|
| [`config.minimal.json`](../config.minimal.json) | Monitor-only defaults |
| [`config.example.json`](../config.example.json) | AI enabled + common knobs |
| `~/.mac-stats/schedules.json` | Scheduler (templated on first seed) |
| `~/.mac-stats/discord_channels.json` | Discord channel modes |

See [CONFIG.md](CONFIG.md), [QUICKSTART.md](QUICKSTART.md), [homebrew.md](homebrew.md), [NOTARIZATION.md](NOTARIZATION.md).

## Help in the app

Settings → **Help / Command cheat sheet**, and **Reset to monitor defaults** (sets `aiAgentEnabled: false`, `menuBarCompact: true` without deleting Keychain secrets).

## Roadmap & changelog

- [CHANGELOG.md](../CHANGELOG.md)
- [ROADMAP.md](ROADMAP.md)
- Design history: [design/](design/)
