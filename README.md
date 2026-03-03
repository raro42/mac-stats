# mac-stats

**The AI agent that just gets it done. All local.**

[![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

<img src="screens/data-poster.png" alt="mac-stats Data Poster theme" width="500">

📋 [Changelog](CHANGELOG.md) · 📸 [Screenshots & themes](screens/README.md)

---

## Install

**DMG (recommended):** [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

**Build from source:**
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

**If macOS blocks the app:** Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## At a glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord bot** — Optional; @mentions, DMs, having_fun mode (let your Mac chat with other bots when bored—yes, it gets weird); full Ollama + tools.
- **Tasks & scheduler** — Task files under `~/.mac-stats/task/`; cron or one-shot; optional Discord reply.
- **All local** — Models and data on your Mac; no cloud backend; works offline.
- **Low CPU** — ~0.5% with window closed, &lt;1% with CPU window open.

---

## All local

- **AI** — Ollama on your Mac; models and inference stay on-device.
- **Your data** — Config, memory, sessions, and tasks in `~/.mac-stats`. Nothing sent to a vendor. Secrets in `~/.mac-stats/.config.env` or Keychain.
- **Optional network** — Discord, Mastodon, FETCH_URL, Brave Search, Perplexity (web search), website checks only when you use them.
- **Metrics** — Read from your machine when you open the window (temperature, frequency, process list).

No subscription. No lock-in. Works offline for chat and monitoring. All you need is a nice pet from [Ollama](https://ollama.com)—the kind that runs on your Mac and never asks for a subscription.

---

## Configuration

All settings live under `~/.mac-stats/`:

```
~/.mac-stats/
├── config.json            # Window decorations, scheduler interval, ollamaChatTimeoutSecs, browserViewportWidth/Height
├── .config.env            # Secrets (Discord, Mastodon, API keys, Perplexity) — never commit ;-)
├── discord_channels.json  # Per-channel modes (mention_only, all_messages, having_fun)
├── escalation_patterns.md # Phrases that trigger “try harder” (e.g. “think harder”, “you are stupid”); user-editable, auto-adds when you complain
├── schedules.json         # Cron and one-shot tasks
├── user-info.json         # Per-user details (Discord id → display_name, notes, timezone)
├── agents/                # LLM agents (orchestrator, coder, etc.), soul.md, memory.md
├── prompts/               # Editable planning_prompt.md, execution_prompt.md
├── skills/                # skill-<n>-<topic>.md for different agent personalities
├── task/                  # Task files (TASK_LIST, TASK_CREATE, TASK_STATUS)
├── scripts/               # PYTHON_SCRIPT output
├── session/               # Conversation sessions (compacted to memory)
├── screenshots/           # BROWSER_SCREENSHOT output
└── debug.log              # App logs (tail -f ~/.mac-stats/debug.log)
```

---

## Commands

Binary name `mac_stats`; app shows as **mac-stats**. From repo root unless noted.

| Command | Description |
|---------|-------------|
| `mac_stats` | Start app (menu bar + optional CPU window) |
| `mac_stats --cpu` | Start with CPU window open |
| `mac_stats -v` / `-vv` / `-vvv` | Verbosity for debug.log |
| `mac_stats discord send <channel_id> <message>` | Post to Discord from CLI |
| `./run dev` | Development mode, hot reload (repo root) |

---

## Features

### AI & agents (Ollama, local)
- **Chat** — In the app window or via Discord. Code execution (JS), **FETCH_URL**, **BRAVE_SEARCH**, **PERPLEXITY_SEARCH** (optional; API key in env, `.config.env`, or Keychain/Settings), **RUN_CMD** (allowlisted), **MASTODON_POST** (toot from the agent), retry and correction.
- **Completion verification** — We extract 1–3 success criteria at the start and ask “Did we satisfy the request?” at the end; if not, we append a disclaimer. Heuristic: “screenshot requested but none attached” → note. See [docs/025_expectation_check_design_DONE.md](docs/025_expectation_check_design_DONE.md).
- **Escalation / “try harder”** — Edit `escalation_patterns.md` (see config tree); one phrase per line. When your message matches one, we run a stronger pass (+10 tool steps). New phrases you use get auto-added.
- **Memory** — Global and per-agent `memory.md`; **MEMORY_APPEND**; session compaction writes lessons to memory.
- **Discord bot** — Optional. @mentions, DMs, or having_fun mode (your Mac chats with other bots when bored); per-channel model/agent. Full Ollama + tools.
- **Tasks** — `~/.mac-stats/task/` with **TASK_LIST**, **TASK_CREATE**, **TASK_STATUS**, assignees, scheduler loop.
- **Scheduler** — Cron or one-shot (`~/.mac-stats/schedules.json`); tasks through Ollama; optional Discord reply channel.
- **MCP** — Tools from any MCP server (HTTP/SSE or stdio).
- **Agents** — Multiple LLM agents under `~/.mac-stats/agents/` (orchestrator, coder, Discord expert, etc.); **AGENT:** delegates. Local models by role; cloud only when you configure it ([design](docs/030_agent_model_assignment_plan_DONE.md)). Editable prompts in `~/.mac-stats/prompts/` and `soul.md`.
- **cursor-agent** — When the [Cursor Agent CLI](https://cursor.com) is on PATH, agents can delegate via **CURSOR_AGENT:** or **RUN_CMD: cursor-agent**; see [docs/012_cursor_agent_tasks.md](docs/012_cursor_agent_tasks.md).
- **PYTHON_SCRIPT** — Ollama can run Python under `~/.mac-stats/scripts/` (disable with `ALLOW_PYTHON_SCRIPT=0`).

### UI
- Menu bar + expandable window; status dashboard (monitors, Ollama, etc.); 9 themes. Scrollable, collapsible sections.

### System monitoring (background)
- Real-time CPU, GPU, RAM, disk in the menu bar; temperature, frequency, battery; process list with top consumers. Click for details.
- **Monitoring & alerts** — Website and social monitoring (Mastodon mentions); alert rules and channels (Telegram, Slack, Mastodon, etc.).

---

## Usage

- **Chat** — Open the window (click the menu bar icon or run with `--cpu`) and use the AI panel. Verbosity: `-v` / `-vv` / `-vvv`.
- **Discord** — Configure `~/.mac-stats/discord_channels.json` and ensure your bot token is set; the agent responds to @mentions, DMs, or in having_fun channels (where your Mac can chill and talk to other bots). See [Discord setup and channel modes](docs/007_discord_agent.md) for details.
- **Mastodon** — Set `MASTODON_INSTANCE_URL` and `MASTODON_ACCESS_TOKEN` in env or `~/.mac-stats/.config.env`; the agent can **MASTODON_POST** toots, and you can add Mastodon monitors (mentions) and alert channels. No X.com yet ;-) — let's see who implements it first.
- **Monitoring** — Click any percentage in the menu bar to open the details window. ⌘W to hide; click again to toggle; ⌘Q to quit.

---

## Development

**Prerequisites:** Rust. From repo root: `./run dev`. Run `cargo audit` in `src-tauri/` before release. Coder-agent workflow: [docs/agent_workflow.md](docs/agent_workflow.md).

---

## Inspiration & notes

Local AI agent stack first; system monitoring lives in the menu bar when you need it. Inspired by [Stats](https://github.com/exelban/stats) by exelban (low CPU, native metrics), [OpenClaw](https://github.com/openclaw/openclaw), [browser-use](https://github.com/browser-use/browser-use), and [Hermes](https://github.com/NousResearch/hermes-agent) by Nous Research. Built with Rust + Tauri; metrics use libproc, SMC, IOReport where appropriate.

- Menu bar: refresh every 1–2s | Window: 1s | Process list: 15s

---

## Contact

Questions or ideas? [Discord](https://discord.com/users/687953899566530588) or open an issue on GitHub.

**We’d love your feedback.** If you’ve tried mac-stats—or you’re thinking about it—drop a note in [**this discussion**](https://github.com/raro42/mac-stats/issues/3). What works, what doesn’t, and what you’d like to see next. It all helps.

---

[MIT License](https://opensource.org/licenses/MIT)
