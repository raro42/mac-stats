## Installation

### Recommended: DMG

[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source:

```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```

Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### macOS Gatekeeper

If macOS blocks the app, Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord bot**: Integrates with the Discord bot for real-time chat and tool invocation.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Available Tool Agents

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in `python3 <script_path>` (no shell). Stdout and stderr are captured. |

## Python Script Agent

The PYTHON_SCRIPT agent lets Ollama create and run Python scripts. The app writes the script to `~/.mac-stats/scripts/python-script-<id>-<topic>.py`, runs it with `python3`, and injects stdout (on success) or an error message (on failure) back into the conversation.

### Overview

- **Agent name**: PYTHON_SCRIPT
- **Invocation**: Ollama replies with one line: `PYTHON_SCRIPT: <id> <topic>`, then provides the Python code either on the following lines or inside a ` ```python ... ``` ` block.
- The app creates the script file, runs `python3 <path>`, and injects the result (stdout if exit code 0, else exit code and stderr) back into the conversation.

### Setup

- **Default**: PYTHON_SCRIPT is **enabled** by default. No extra install; the app uses the system `python3`.
- **Disable**: Set `ALLOW_PYTHON_SCRIPT=0` (or `false`, `no`, `off`) in the environment or in `.config.env` (current directory, `src-tauri/`, or `~/.mac-stats/.config.env`) to disable the agent.

### Invocation Format

1. **Header line**: `PYTHON_SCRIPT: <id> <topic>` (e.g. `PYTHON_SCRIPT: 1 process-data`). Id and topic are sanitized for filenames (only `[a-zA-Z0-9_-]` kept).
2. **Code**: Either
   - Put the Python code on the lines immediately after the header, or
   - Put it inside a fenced block: ` ```python ` … ` ``` `.

### Behaviour

- Script path: `~/.mac-stats/scripts/python-script-<id>-<topic>.py` (scripts directory is created if needed).
- Execution: `python3 <script_path>` (no shell). Stdout and stderr are captured.
- **Exit code 0**: The app injects: `Python script result:\n\n{stdout}\n\nUse this to answer the user's question.`
- **Non-zero exit**: The app injects: `PYTHON_SCRIPT failed: exit code N: {stderr}. Answer without this result.`
- The tool loop (Discord, scheduler) supports PYTHON_SCRIPT like other tools; each call counts as one tool iteration (max 5).

### Security

- **No shell**: Only `Command::new("python3").arg(script_path)` is used.
- **Paths**: Scripts are only written under `~/.mac-stats/scripts/`. Id and topic are sanitized so the filename cannot contain `/` or `..`.
- Scripts run with the same privileges as the app (no root).

## Open tasks:

- Investigate why some users report issues with the Python script agent.
- Improve the documentation for the Python script agent to make it more user-friendly.
- Review the security of the app to ensure it is robust against potential vulnerabilities.