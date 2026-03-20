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
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in the WebView context; result returned to Ollama. |
| **PYTHON_SCRIPT** | `PYTHON_SCRIPT: <id> <topic>` + code | Run a Python script (data processing, calculations). | Writes `~/.mac-stats/scripts/python-script-<id>-<topic>.py`, runs `python3`, returns stdout or error. See below. |

## Python Script Agent

The **PYTHON_SCRIPT** agent lets Ollama create and run Python scripts on your Mac. The app writes the script to a file under `~/.mac-stats/scripts/`, runs it with `python3`, and injects stdout (on success) or an error message (on failure) back into the conversation so the model can answer using the result.

### When to use it

- **Data processing**: Parse CSV/JSON, aggregate numbers, filter lists.
- **Calculations**: Math, statistics, or anything easier in Python than in the model’s reply.
- **Local scripts**: One-off scripts that read/write files under your home directory (with the same permissions as the app).

The model will choose PYTHON_SCRIPT when the user asks for computation, file processing, or “run a Python script” style tasks.

### Overview

- **Agent name**: PYTHON_SCRIPT
- **Invocation**: Ollama replies with one line: `PYTHON_SCRIPT: <id> <topic>`, then provides the Python code on the following lines or inside a ` ```python ... ``` ` block.
- The app creates the script file, runs `python3 <path>`, and injects the result (stdout if exit code 0, else exit code and stderr) back into the conversation.

### Setup

- **Default**: PYTHON_SCRIPT is **enabled**. The app uses the system `python3` (no extra install required).
- **Disable**: Set `ALLOW_PYTHON_SCRIPT=0` (or `false`, `no`, `off`) in the environment or in a `.config.env` file. The app checks, in order:
  1. Environment variable `ALLOW_PYTHON_SCRIPT`
  2. `.config.env` in the current working directory
  3. `src-tauri/.config.env` (when running from repo root)
  4. `~/.mac-stats/.config.env`
- **Scripts directory**: Scripts are stored under `~/.mac-stats/scripts/`. The directory is created automatically when the first script is run.

### Invocation format

1. **Header line**: `PYTHON_SCRIPT: <id> <topic>` (e.g. `PYTHON_SCRIPT: 1 process-data`). Use short alphanumeric id and topic; they are sanitized for filenames (only `[a-zA-Z0-9_-]` kept; other characters become `_`).
2. **Code**: Either
   - Put the Python code on the lines immediately after the header, or
   - Put it inside a fenced block: ` ```python ` … ` ``` `.

**Example (inline code):**

```
PYTHON_SCRIPT: 1 sum
total = sum(range(101))
print(total)
```

**Example (fenced block):** Reply with `PYTHON_SCRIPT: 2 stats` then a line with ` ```python `, the code, then ` ``` `.

### Behaviour

- **Script path**: `~/.mac-stats/scripts/python-script-<id>-<topic>.py` (e.g. `python-script-1-sum.py`). The scripts directory is created if it does not exist.
- **Execution**: `python3 <script_path>` (no shell). Stdout and stderr are captured. There is no execution timeout; long-running scripts can block the tool until they finish.
- **Exit code 0**: The app injects: `Python script result:\n\n{stdout}\n\nUse this to answer the user's question.`
- **Non-zero exit**: The app injects: `PYTHON_SCRIPT failed: exit code N: {stderr}. Answer without this result.`
- **Tool loop**: Each PYTHON_SCRIPT call counts as one tool iteration (subject to the run’s tool cap, typically 15).

### Security

- **No shell**: The app runs only `python3 <script_path>`; no shell is used, so shell metacharacters in id/topic do not change behaviour.
- **Paths**: Scripts are only written under `~/.mac-stats/scripts/`. Id and topic are sanitized so the filename cannot contain `/` or `..`.
- **Privileges**: Scripts run with the same user and permissions as the app (no root).

### Security review (measures in place)

| Measure | Purpose |
|--------|--------|
| **No shell** | Execution is `Command::new("python3").arg(script_path)`; id and topic are not passed to a shell, so injection via id/topic is not possible. |
| **Filename sanitization** | `sanitize_filename_part()` keeps only `[a-zA-Z0-9_-]`; `/`, `..`, and other characters become `_`, so the script path cannot escape `~/.mac-stats/scripts/`. |
| **Fixed script directory** | Scripts are written only to `Config::scripts_dir()` (`~/.mac-stats/scripts/` or temp fallback); no user-controlled path. |
| **Same uid** | Scripts run as the same user as the app; no privilege escalation. |
| **ALLOW_PYTHON_SCRIPT** | Set to `0` / `false` / `no` (env or `.config.env`) to disable PYTHON_SCRIPT entirely in locked-down setups. |

**Trust boundary:** The script *body* is provided by the model (Ollama) based on user requests. The app does not sandbox Python execution: scripts can read/write anywhere the app’s user can, use the network, spawn processes, and run indefinitely (no timeout). This is intentional so that data processing and local scripts work; the risk is accepted in the same way as RUN_CMD (agent-controlled code runs with user privileges). Mitigations: disable the agent via `ALLOW_PYTHON_SCRIPT=0` when not needed; run the app in a restricted environment if you want to limit script capabilities.

### Troubleshooting

| Issue | What to check |
|-------|----------------|
| **“python3: command not found”** | Ensure `python3` is on your `PATH` when the app runs (e.g. launch from Terminal, or install Python and ensure the app’s environment sees it). |
| **“PYTHON_SCRIPT is not available”** | The agent is disabled. Remove `ALLOW_PYTHON_SCRIPT=0` from your environment and from any `.config.env` the app reads (see Setup above). |
| **Script fails with import or syntax errors** | The script is run with the system `python3`; it only has access to the standard library and packages installed for that interpreter. Check `python3 --version` and install any required packages (e.g. `pip3 install …`) for the same interpreter. |
| **Where are my scripts?** | All scripts are under `~/.mac-stats/scripts/` with names like `python-script-<id>-<topic>.py`. You can inspect or run them manually. |
| **Empty or missing result** | Ensure the script prints to stdout (e.g. `print(...)`). Only stdout is returned; stderr is shown only when the script exits with a non-zero code. |

## Open tasks

- ~~Investigate why some users report issues with the Python script agent.~~ **Done:** Improved diagnostics: script path in user-facing error; on failure and on spawn failure, `tracing::warn!` logs script path, exit code, and stderr preview (500 chars) to `~/.mac-stats/debug.log` for easier debugging.
- ~~Improve the documentation for the Python script agent to make it more user-friendly.~~ **Done:** When to use, setup (config precedence), invocation examples, behaviour (path, no timeout, tool cap), security, troubleshooting table.
- ~~Review the security of the app to ensure it is robust against potential vulnerabilities.~~ **Done:** § "Security review (measures in place)" above (no shell, filename sanitization, fixed directory, same uid, ALLOW_PYTHON_SCRIPT; trust boundary and caveats). Open task tracked in 006-feature-coder/FEATURE-CODER.md.