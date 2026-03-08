# mac-stats: Local AI Agent for macOS

## 📦 Installation
- **DMG (recommended):** [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications
- **Build from source:**  
  ```bash
  git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
  ```
  Or one-liner:  
  `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

**Note:** If blocked by Gatekeeper:  
`xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## 📌 Key Features
- **Menu bar** – CPU, GPU, RAM, disk metrics at a glance (click for details)
- **AI chat** – Ollama integration for:
  - Discord bot
  - Task runner
  - Scheduler
  - MCP (Mac-specific tools)
- **Real-time monitoring** – Minimalist, always-on system metrics
- **Local execution** – No cloud, no telemetry

---

## 🤖 Agent Overview

### Tool Agents (Ollama can invoke)
| Agent | Invocation | Purpose | Implementation |
|-------|-----------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <URL>` | Fetch web page content (server-side) | `commands/browser.rs` |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <query>` | Web search via Brave API | `commands/brave.rs` (requires `BRAVE_API_KEY`) |
| **RUN_JS** | `RUN_JS: <code>` | Execute JavaScript in CPU window | CPU window context |

### Entry-Point Agents
- **Discord** – Bot integration for chat and task scheduling
- **CPU window** – Chat interface with code execution
- **Cursor** – Mouse gesture-based interaction (experimental)

---

## ⏱️ Scheduler Agent

### Configuration
- **File path:** `$HOME/.mac-stats/schedules.json`
- **Full reference:** See **docs/data_files_reference.md** for JSON structure, fields, time interpretation (cron and `at` = local time), and examples.
- **Format:**
  ```json
  {
    "schedules": [
      {
        "id": "daily-weather",
        "cron": "0 0 9 * * *",
        "task": "Check the weather for today and summarize in one sentence"
      }
    ]
  }
  ```
- **Fields:**
  - `cron` (recurring) – 6-field local time cron expression
  - `at` (one-shot) – ISO 8601 datetime (e.g. `2025-02-09T18:00:00`)
  - `task` – Free text for Ollama planning or direct tool command
  - `reply_to_channel_id` – Discord channel ID for result posting

### Behavior
- Runs in background thread
- Reloads schedules every **60 seconds** (configurable via `config.json` or `MAC_STATS_SCHEDULER_CHECK_SECS`)
- Executes tasks using Ollama + tools or direct tool calls
- Supports deduplication by `cron` + `task` (whitespace-normalized)

---

## 📄 References
- **Code:** `src-tauri/src/scheduler/mod.rs`
- **List schedules:** `LIST_SCHEDULES` agent tool (see `commands/ollama.rs`)
- **Config:** `Config::schedules_file_path()`, `Config::scheduler_check_interval_secs()`

---

## 🛠️ Open tasks:
- Add a scheduler UI for creating and editing schedules instead of relying on manual `schedules.json` edits.
- Consider support for multiple API keys in scheduler-driven tool flows.
- Improve error handling for scheduler tool invocations.
- ~~Clarify whether cron expressions should be interpreted in local time or UTC.~~ **Done:** documented in **docs/data_files_reference.md** (cron and `at` = local time).
- Review deduplication behavior for identical `cron` + `task` pairs.

---