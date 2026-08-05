# mac-stats features

Product feature list for [mac-stats](https://github.com/raro42/mac-stats/) — menu-bar system monitor for Apple Silicon (optional local AI agent).

Screenshots: [screens/](screens/) · [screens/README.md](screens/README.md)

---

## Core monitor (always on)

| Feature | Notes |
|---------|--------|
| Menu-bar glance | Compact **CPU + SSD** (and °C when known); expand with `menuBarCompact: false` for CPU/GPU/RAM/SSD |
| Glass CPU window | Nine themes; ring gauges for CPU, GPU, frequency, temperature |
| History sparklines | CPU → Frequency → Temperature under the gauges |
| Top processes | Sortable list, pin favorites, process details, Advanced Force Quit |
| Website monitors | See below |
| Low overhead | On the order of ~0.5% idle (menu bar only) |

## Website monitors (External / Monitors)

HTTP(S) uptime checks in the CPU window — part of the core monitor (no AI required).

| Detail | Notes |
|--------|--------|
| Summary | *N / M sites up · Avg … ms* |
| Per site | Status dot, URL, latency, error text when down |
| History bars | Recent check history (green up / red down) |
| Menu bar | Red **Mon ✕** cue when any site is down |
| Manage | `…` menu on the section — add / remove / check |

<img src="screens/feature-monitors.png" alt="External / Monitors" width="360">

## Disk Cleanup

Preview and reclaim reclaimable files with **configurable scopes**:

| Scope | Default | Notes |
|-------|---------|--------|
| **mac-stats data** | On | Age policies under `~/.mac-stats` (screenshots, PDFs, traces, …) |
| **Trash** | Off | `~/.Trash` — set max age (days) + recurse |
| **Downloads** | Off | `~/Downloads` — top-level by default; age threshold |
| **Temp** | Off | System temp + `/tmp` |
| **Custom path** | — | Label + path + age + recursive (saved in config) |

Runs on **app launch**, every **24h while running** (configurable), and **Clean now**. State in `~/.mac-stats/disk_cleanup.json`; scopes in `config.json` → `diskCleanupScopes`.

<img src="screens/feature-disk-cleanup.png" alt="Disk Cleanup scopes" width="360">

## Local AI agent (opt-in)

Off until `aiAgentEnabled: true` (Settings or config).

| Feature | Notes |
|---------|--------|
| Ollama chat | Local LLM; code execution loop |
| Discord (Werner) | Bot + channel modes |
| Tools | FETCH_URL, Brave, Perplexity, CDP browser, RUN_CMD, skills, MCP, … |
| Tasks & scheduler | Cron / at schedules while the app is running |
| Agent Ops | Command Center overview (schedules, live, knowledge, recent chats) |

## Themes

Apple · Architect · Dark (TUI) · Data Poster · Futuristic · Light · Material · Neon · Swiss Minimalistic — gallery in [screens/README.md](screens/README.md).

## Privacy

**No cloud telemetry.** Config and data under `~/.mac-stats/`. Secrets in Keychain and/or `.config.env`.

## See also

- [README.md](README.md) — install & quick start  
- [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md)  
- [docs/CONFIG.md](docs/CONFIG.md)  
- [CHANGELOG.md](CHANGELOG.md)  
