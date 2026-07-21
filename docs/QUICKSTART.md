# Quick start

**Goal:** menu-bar stats + local AI chat in about a minute.

## 1. Requirements

- **macOS on Apple Silicon** (arm64). Intel Macs are not supported by the published DMG/cask.
- **[Ollama](https://ollama.com)** installed and running, with at least one model:

```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama3.2
```

- Optional network features (Discord, Brave, Perplexity, Redmine) need keys in `~/.mac-stats/.config.env` — see [CONFIG.md](CONFIG.md).

## 2. Install the app

**Homebrew (preferred):**

```bash
brew tap raro42/mac-stats https://github.com/raro42/mac-stats
brew install --cask mac-stats
```

**DMG:** [Latest release](https://github.com/raro42/mac-stats/releases/latest) → open the `.dmg` → drag **mac-stats** to Applications.

If Gatekeeper blocks an unsigned build: Right-click → **Open**. See [NOTARIZATION.md](NOTARIZATION.md).

## 3. First run

1. Launch **mac-stats** (menu bar icon appears).
2. Click the menu bar → open the window.
3. Expand **AI Chat (Ollama)** (or click the Ollama icon) → ask a question.

If chat does nothing: confirm `ollama list` shows a model and Ollama is running (`ollama serve` if needed).

## Next

- Discord bot: [007_discord_agent.md](007_discord_agent.md)
- Config & secrets: [CONFIG.md](CONFIG.md)
- Full feature list: [../README.md](../README.md#full-features)
