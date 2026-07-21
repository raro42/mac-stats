# Quick start

See the full guide: **[GETTING_STARTED.md](GETTING_STARTED.md)**.

## Just the monitor

```bash
brew tap raro42/mac-stats https://github.com/raro42/mac-stats
brew install --cask mac-stats
open -a mac-stats
```

Or from a clone: `./scripts/quickstart.sh`

## Monitor + AI

1. Install as above.
2. `curl -fsSL https://ollama.com/install.sh | sh && ollama pull llama3.2`
3. Settings → **Enable local AI agent** (or `aiAgentEnabled: true` in `~/.mac-stats/config.json`), restart.
4. Ask: *What's my CPU temp?*

**Apple Silicon only.** AI is opt-in — the base app needs zero AI config.
