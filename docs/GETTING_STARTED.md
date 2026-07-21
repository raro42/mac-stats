# Getting Started

**Menu-bar system monitor for Apple Silicon (optional local AI agent).**

## Choose a path

### A) Just the monitor (recommended first)

1. Install:
   ```bash
   brew tap raro42/mac-stats https://github.com/raro42/mac-stats
   brew install --cask mac-stats
   ```
   Or from a clone: `./scripts/quickstart.sh`
2. Open **mac-stats** — menu bar shows **CPU** (and °C when available).
3. Click the menu bar for the glass window (themes, processes, monitors).

AI features stay **off** until you enable them (`aiAgentEnabled`).

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
