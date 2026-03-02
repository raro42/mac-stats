# mac-stats

**The AI agent that just gets it done. All local.**

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. It also sits in your menu bar and shows CPU, GPU, RAM, and disk when you need it. Built with Rust and Tauri.

<img src="screens/apple.png" alt="mac-stats" width="420">

📋 [Changelog](CHANGELOG.md) · 📸 [Screenshots & themes](screens/README.md)

---

## All local

- **AI** — Ollama on your Mac; models and inference stay on-device.
- **Your data** — Config, memory, sessions, and tasks in `~/.mac-stats`. Nothing sent to a vendor. Secrets in `~/.mac-stats/.config.env` or Keychain.
- **Optional network** — Discord, FETCH_URL, Brave Search, website checks only when you use them.
- **Metrics** — CPU, GPU, RAM, disk, temperature, processes read from your machine when you open the window.

No subscription. No lock-in. Works offline for chat and monitoring.

---

## Features

### AI & agents (Ollama, local)
- **Chat** — In the app window or via Discord. Code execution (JS), **FETCH_URL**, **BRAVE_SEARCH**, **RUN_CMD** (allowlisted), retry and correction.
- **Memory** — Global and per-agent `memory.md`; **MEMORY_APPEND**; session compaction writes lessons to memory.
- **Discord bot** — Optional. @mentions, DMs, or “having_fun” mode; per-channel model/agent. Full Ollama + tools; CLI: `mac_stats discord send <channel_id> <message>`.
- **Tasks** — `~/.mac-stats/task/` with **TASK_LIST**, **TASK_CREATE**, **TASK_STATUS**, assignees, scheduler loop.
- **Scheduler** — Cron or one-shot (`~/.mac-stats/schedules.json`); tasks through Ollama; optional Discord reply channel.
- **MCP** — Tools from any MCP server (HTTP/SSE or stdio).
- **Agents** — Multiple LLM agents under `~/.mac-stats/agents/` (orchestrator, coder, Discord expert, etc.); **AGENT:** delegates. Editable prompts in `~/.mac-stats/prompts/` and `soul.md`.
- **PYTHON_SCRIPT** — Ollama can run Python under `~/.mac-stats/scripts/` (disable with `ALLOW_PYTHON_SCRIPT=0`).

### UI
- Menu bar + expandable window; status dashboard (monitors, Ollama, etc.); 9 themes. Scrollable, collapsible sections.

### System monitoring (background)
- Real-time CPU, RAM, disk, GPU in the menu bar; temperature, frequency, battery. Process list with top consumers; click for details. Low CPU: &lt;0.1% with window closed, ~3% with window open.
- **Monitoring & alerts** — Website and social monitoring; alert rules and channels (Telegram, Slack, etc.).

### Known limitation
- **Window frame** — “Window Frame” in settings applies to new windows; existing ones update after close/reopen. Stored in `~/.mac-stats/config.json`.

---

## Installation

### DMG (recommended)

Download the latest [mac-stats release](https://github.com/raro42/mac-stats/releases/latest) and drag the app to Applications.

**If macOS says the DMG is “damaged”:** Gatekeeper is blocking the unsigned app. The file is fine.

- **Easiest:** Right-click the DMG → **Open** → confirm **Open** in the dialog.
- **Or in Terminal:**  
  `xattr -d com.apple.quarantine ~/Downloads/mac-stats_*.dmg`  
  If the app itself is blocked after install:  
  `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

### Build from source

```bash
# One-liner (downloads run script, builds, runs)
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run
```

Or clone and build:

```bash
git clone https://github.com/raro42/mac-stats.git
cd mac-stats
./run
```

Manual: `cd src-tauri && cargo build --release` then run `./target/release/mac_stats`.

---

## Usage

- **Chat** — Open the window (click the menu bar icon or run with `--cpu`) and use the AI panel. Verbosity: `-v` / `-vv` / `-vvv`.
- **Discord** — Configure `~/.mac-stats/discord_channels.json` and ensure your bot token is set; the agent responds to @mentions, DMs, or in having_fun channels.
- **Monitoring** — Click any percentage in the menu bar (CPU, GPU, RAM, Disk) to open the details window. ⌘W to hide; click again to toggle; ⌘Q to quit. CPU use: &lt;0.1% with window closed, ~3% with window open.

---

## Development

**Prerequisites:** Rust

```bash
./run dev
```

Run `cargo audit` (if available) in `src-tauri/` before release to check for known vulnerabilities.

### Agent workflow

This repo is edited by the mac-stats-reviewer Coder agent in place (yolo mode). For direct edits, open Cursor with **`mac-stats-agent-workspace.code-workspace`** (in the mac-stats-reviewer repo) so both repos are in the workspace; then the Coder can edit files here directly. See [docs/agent_workflow.md](docs/agent_workflow.md).

---

## Inspiration & notes

Local AI agent stack first; system monitoring lives in the menu bar when you need it. Inspired by [Stats](https://github.com/exelban/stats) by exelban (low CPU, native metrics). Built with Rust + Tauri; metrics use libproc, SMC, IOReport where appropriate.

- Menu bar: every 1–2s | Window: 1s | Process list: 15s | Window stays on top when open

---

## Contact

Questions or ideas? [Discord](https://discord.com/users/687953899566530588) or open an issue on GitHub.
