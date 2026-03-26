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
- **Main chat awareness:** When a run **successfully** posts to Discord via `reply_to_channel_id`, the app records a short entry in `~/.mac-stats/scheduler_delivery_awareness.json` (deduped by a per-run `context_key`) and injects the latest entries into the **CPU window** Ollama **system** prompt on subsequent turns. Discord remains the source of truth for what was posted; the file + injection avoid split-brain when you later chat in-app. See **docs/data_files_reference.md** (scheduler_delivery_awareness.json). Settings → Schedules lists recent deliveries for operators.

### Heartbeat (optional, OpenClaw-style)

Separate from `schedules.json`: an optional **heartbeat** loop in `~/.mac-stats/config.json` under key `heartbeat`. When `enabled` is true, mac-stats runs one agent turn on a fixed interval with a **checklist** (file path, inline prompt, or built-in default). The model is instructed to reply **`HEARTBEAT_OK`** only when nothing needs the user’s attention; those replies are **not** posted to Discord. If the reply is substantive or omits the ack pattern, it can be delivered to a Discord channel when `replyToChannelId` is set; otherwise the text is only logged.

- **Config fields:** See **docs/data_files_reference.md** (`config.json` → heartbeat).
- **Timeout:** Same wall-clock cap as scheduler tasks (`schedulerTaskTimeoutSecs` / `MAC_STATS_SCHEDULER_TASK_TIMEOUT_SECS`).
- **Logs:** Subsystem target `mac_stats::scheduler/heartbeat` (see **docs/039_mac_stats_log_subsystems.md**).
- **Code:** `src-tauri/src/scheduler/heartbeat.rs`, `Config::heartbeat_settings()` in `config/mod.rs`, execution prompt hook in `commands/ollama.rs` / `prompt_assembly.rs` / `prompts/mod.rs`.

### Multiple API keys / endpoints (design)

**Current behaviour:** The scheduler uses a single, app-wide configuration for all schedules:

- **Ollama:** One endpoint and model (from Settings / `config.json`). Every schedule that runs via Ollama uses the same client.
- **Brave Search:** One API key (env `BRAVE_API_KEY` or Keychain). Direct `BRAVE_SEARCH: <query>` tasks use this key.
- **Discord:** One bot token. `reply_to_channel_id` posts to that bot’s channel.

**What “multiple API keys” could mean:**

1. **Per-schedule Brave key** – e.g. schedule A uses key from env, schedule B uses a different key (e.g. from Keychain by label, or a field in `schedules.json`). Use case: different projects or rate limits.
2. **Per-schedule Ollama endpoint/model** – e.g. one schedule uses a fast model, another a larger model. Would require optional `ollama_endpoint` / `ollama_model` on the schedule entry and a way to select client per run.
3. **Multiple Discord bots** – e.g. schedule posts to channel of bot A, another to bot B. Would require multiple tokens and channel→bot mapping; larger change.

**Options (no implementation):**

- **Env / config only:** Keep single keys; users who need multiple keys can run a second mac-stats instance (e.g. different config dir) or use separate scheduler processes. Document this as the current recommendation.
- **Per-schedule overrides:** Extend schedule entry with optional `brave_api_key_id`, `ollama_endpoint`, etc. Keys could be stored in Keychain with a label; scheduler would resolve by label. Adds complexity to UI and file format.
- **Named profiles:** Introduce “profiles” in config (e.g. `profiles: { "work": { "brave_api_key": "…" }, "personal": { … } }`) and an optional `profile` field on each schedule. Scheduler would load the right profile for the run.

No code change in this FEAT; this section records the investigation and design options for future work.

---

## 📄 References
- **Code:** `src-tauri/src/scheduler/mod.rs`, `src-tauri/src/scheduler/heartbeat.rs`
- **List schedules:** `LIST_SCHEDULES` agent tool (see `commands/ollama.rs`)
- **Config:** `Config::schedules_file_path()`, `Config::scheduler_check_interval_secs()`

---

## 🛠️ Open tasks

Scheduler open tasks (scheduler UI, multiple API keys) are tracked in **006-feature-coder/FEATURE-CODER.md**. Completed: error handling for scheduler tool invocations (failure message to Discord when `reply_to_channel_id` set); cron/`at` = local time (data_files_reference.md); deduplication for identical cron+task pairs (add_schedule / add_schedule_at).

---