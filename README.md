# mac-stats

**Local agent harness. Same loop as OpenClaw / Hermes — on your Mac.**

[![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

Rust + Tauri menu-bar app for **macOS** (Apple Silicon–first): **live system metrics** and a **local Ollama agent** that acts like a real harness—**native tool calling**, one **model → tools → model** loop, Discord, tasks, scheduler, and MCP—without shipping your data to a vendor backend.

**The hook:** Werner (Discord / in-app chat) defaults to a **direct harness**—no criteria / plan / verify meta-LLM tax. Tools are sent as OpenAI/Ollama **`tools` schemas**; the model emits structured **`tool_calls`** (text `TOOL: arg` remains a fallback). Trivial asks (time, ping) answer in milliseconds with **zero** LLM calls. Details: [docs/039_werner_harness_parity.md](docs/039_werner_harness_parity.md).

<img src="screens/data-poster.png" alt="mac-stats Data Poster theme" width="500">

📋 [Changelog](CHANGELOG.md) · 🗺️ [AI roadmap (tasks & tools)](docs/006_roadmap_ai_tasks.md) · 📸 [Screenshots & themes](screens/README.md)

---

## What mac-stats ships with

### Menu bar & glass UI

- **CPU, GPU, RAM, and disk** in the menu bar; open the window for **temperature**, **CPU frequency** (IOReport where available), **battery**, and a **process list** with top consumers.
- **Nine themes**, collapsible sections, and a dashboard for **monitors**, **Ollama**, and more—designed to feel native on macOS.
- **Low overhead** in normal use: on the order of **~0.5% CPU** with the window closed, **under ~1%** with the CPU window open.

### Local AI agent (Ollama) — harness-first

- **Direct by default** (`agentHarnessMode: "direct"`) — one execute tool-loop, same shape as OpenClaw/Hermes. Set `"classic"` only if you want the old criteria → plan → verify pipeline.
- **Native tools by default** (`agentNativeTools: true`) — schemas on `/api/chat`; compact tool catalog in the prompt; text-line tools still work as fallback.
- **Chat in the app** or **via Discord (Werner)** with the same engine: multi-step tool loop, sub-agents (**`AGENT:`**), **skills**, **memory** (`memory.md`, **MEMORY_APPEND**), and **session compaction**.
- **Web & research:** **`FETCH_URL`** (server-side, **SSRF-hardened**), **`BRAVE_SEARCH`**, **`PERPLEXITY_SEARCH`**. Weather asks get **Open-Meteo grounding** so search snippets cannot invent temperatures.
- **Browser automation (CDP):** navigate, click, scroll, extract, and **`BROWSER_SCREENSHOT`** → PNG under `~/.mac-stats/screenshots/`.
- **Automation & coding:** **`RUN_CMD`** (permissive allowlist under `$HOME`; shells / `sudo` / `rm` still blocked), **`RUN_JS`**, **`PYTHON_SCRIPT`**, **`CURSOR_AGENT`** when the Cursor CLI is on `PATH`.
- **Integrations:** **`DISCORD_API`**, **`MASTODON_POST`**, **Redmine**, optional **plugins**, **any MCP server** (stdio or HTTP/SSE), e.g. [Ori Mnemos](docs/038_ori_mnemos_mcp.md).
- **Fast paths:** instant lane (time / ping / thanks), deterministic **pre-routing** (fetch, search, weather, screenshot+URL, …), **context-overflow recovery**, loop guards.

### Discord, tasks, scheduler, monitoring

- **Discord bot** — @mentions, DMs, per-channel modes (**mention_only**, **all_messages**, **having_fun**), optional **draft messages** that update while tools run, **debouncing** for rapid messages, **429** handling with backoff.
- **Tasks** — markdown task files under `~/.mac-stats/task/` (**TASK_CREATE**, **TASK_LIST**, **TASK_STATUS**, assignees, append/show).
- **Scheduler** — cron or one-shot jobs in `schedules.json`; Ollama-powered runs with optional Discord replies and a **per-task wall-clock timeout**.
- **Monitors & alerts** — website checks, social (e.g. Mastodon mentions), rules and channels (Telegram, Slack, Mastodon, …); background **due checks** with stats persisted to disk.

### Privacy & operations

- **Models and inference stay on your Mac** (Ollama). Config, agents, sessions, tasks, and logs live under **`~/.mac-stats/`**. Optional network features only run when you configure them.
- **Structured logging** via `tracing`; tune console noise with **`MAC_STATS_LOG`** ([subsystem filter](docs/039_mac_stats_log_subsystems.md)). Tail **`~/.mac-stats/debug.log`** when debugging.

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

## All local (quick reminder)

- **AI** — Ollama on your Mac; models and inference stay on-device unless you point at a remote endpoint yourself.
- **Your data** — Config, memory, sessions, and tasks in `~/.mac-stats`. Secrets in `~/.mac-stats/.config.env` or Keychain.
- **Optional network** — Discord, Mastodon, FETCH_URL, Brave Search, Perplexity, website checks—only when you configure and use them.
- **Metrics** — Read from your machine (temperature, frequency, process list) when the window is open; menu bar stays light when it’s closed.

No subscription. No lock-in. Works offline for chat and core monitoring. Grab a model from [Ollama](https://ollama.com) and you’re running locally.

---

## Configuration

All settings live under `~/.mac-stats/`:

```
~/.mac-stats/
├── config.json            # Window, timeouts, agentHarnessMode, agentNativeTools, browser*, …
├── .config.env            # Secrets (Discord, Mastodon, API keys, Perplexity) — never commit ;-)
├── discord_channels.json  # Per-channel modes (mention_only, all_messages, having_fun)
├── schedules.json         # Cron and one-shot tasks
├── user-info.json         # Per-user details (Discord id → display_name, notes, timezone)
├── agents/                # LLM agents (orchestrator, coder, etc.), soul.md, memory.md
│   ├── escalation_patterns.md   # Phrases that trigger “try harder”; user-editable, auto-adds when you complain
│   ├── session_reset_phrases.md # Phrases that clear session (e.g. “new topic”, “reset”)
│   ├── prompts/           # planning_prompt.md, execution_prompt.md
│   └── skills/            # skill-<n>-<topic>.md for different agent personalities
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
- **Harness** — Default **direct** loop + **native tool schemas** ([parity notes](docs/039_werner_harness_parity.md)). Optional `agentHarnessMode: "classic"` restores criteria/plan/verify.
- **Chat** — In the app window or via Discord (Werner). **FETCH_URL**, **BRAVE_SEARCH**, **PERPLEXITY_SEARCH**, **RUN_CMD** (permissive allowlist), **MASTODON_POST**, weather grounded via Open-Meteo.
- **Escalation / “try harder”** — Edit `~/.mac-stats/agents/escalation_patterns.md`; one phrase per line. When your message matches one, we run a stronger pass (+10 tool steps). New phrases you use get auto-added.
- **Memory** — Global and per-agent `memory.md`; **MEMORY_APPEND**; session compaction writes lessons to memory.
- **Discord bot** — Optional. @mentions, DMs, or having_fun mode; per-channel model/agent. Full Ollama + tools.
- **Tasks** — `~/.mac-stats/task/` with **TASK_LIST**, **TASK_CREATE**, **TASK_STATUS**, assignees, scheduler loop.
- **Scheduler** — Cron or one-shot (`~/.mac-stats/schedules.json`); tasks through Ollama; optional Discord reply channel.
- **MCP** — Tools from any MCP server (HTTP/SSE or stdio). [Ori Mnemos](docs/038_ori_mnemos_mcp.md) vault via `MCP_SERVER_STDIO`.
- **Agents** — Multiple LLM agents under `~/.mac-stats/agents/` (orchestrator, coder, Discord expert, etc.); **AGENT:** delegates. Local models by role; cloud only when you configure it ([design](docs/030_agent_model_assignment_plan_DONE.md)). Editable prompts in `~/.mac-stats/agents/prompts/` and `soul.md` in `agents/`.
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

**Harness first, metrics always on.** The agent loop is deliberately aligned with [OpenClaw](https://github.com/openclaw/openclaw) and [Hermes](https://github.com/NousResearch/hermes-agent) (native tools, single act–observe loop)—running on **local Ollama**, with system monitoring in the menu bar when you need it. Also inspired by [Stats](https://github.com/exelban/stats) (low CPU, native metrics) and [browser-use](https://github.com/browser-use/browser-use). Built with Rust + Tauri; metrics use libproc, SMC, IOReport where appropriate.

- Menu bar: refresh every 1–2s | Window: 1s | Process list: 15s

---

## Contact

Questions or ideas? [Discord](https://discord.com/users/687953899566530588) or open an issue on GitHub.

**We’d love your feedback.** If you’ve tried mac-stats—or you’re thinking about it—drop a note in [**this discussion**](https://github.com/raro42/mac-stats/issues/3). What works, what doesn’t, and what you’d like to see next. It all helps.

---

[MIT License](https://opensource.org/licenses/MIT)
