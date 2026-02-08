# PYTHON_SCRIPT Agent

The PYTHON_SCRIPT agent lets Ollama create and run Python scripts. The app writes the script to `~/.mac-stats/scripts/python-script-<id>-<topic>.py`, runs it with `python3`, and injects stdout (on success) or an error message (on failure) back into the conversation.

## Overview

- **Agent name**: PYTHON_SCRIPT
- **Invocation**: Ollama replies with one line: `PYTHON_SCRIPT: <id> <topic>`, then provides the Python code either on the following lines or inside a ` ```python ... ``` ` block.
- The app creates the script file, runs `python3 <path>`, and injects the result (stdout if exit code 0, else exit code and stderr) back into the conversation.

When PYTHON_SCRIPT is disabled via `ALLOW_PYTHON_SCRIPT=0`, the agent is omitted from the list Ollama sees.

## Setup

- **Default**: PYTHON_SCRIPT is **enabled** by default. No extra install; the app uses the system `python3`.
- **Disable**: Set `ALLOW_PYTHON_SCRIPT=0` (or `false`, `no`, `off`) in the environment or in `.config.env` (current directory, `src-tauri/`, or `~/.mac-stats/.config.env`) to disable the agent.

## Invocation format

1. **Header line**: `PYTHON_SCRIPT: <id> <topic>` (e.g. `PYTHON_SCRIPT: 1 process-data`). Id and topic are sanitized for filenames (only `[a-zA-Z0-9_-]` kept).
2. **Code**: Either
   - Put the Python code on the lines immediately after the header, or
   - Put it inside a fenced block: ` ```python ` … ` ``` `.

If no code is found (empty body), the app returns an error asking for code on the next lines or in a ```python block.

## Behaviour

- Script path: `~/.mac-stats/scripts/python-script-<id>-<topic>.py` (scripts directory is created if needed).
- Execution: `python3 <script_path>` (no shell). Stdout and stderr are captured.
- **Exit code 0**: The app injects: `Python script result:\n\n{stdout}\n\nUse this to answer the user's question.`
- **Non-zero exit**: The app injects: `PYTHON_SCRIPT failed: exit code N: {stderr}. Answer without this result.`
- The tool loop (Discord, scheduler) supports PYTHON_SCRIPT like other tools; each call counts as one tool iteration (max 5).

## Security

- **No shell**: Only `Command::new("python3").arg(script_path)` is used.
- **Paths**: Scripts are only written under `~/.mac-stats/scripts/`. Id and topic are sanitized so the filename cannot contain `/` or `..`.
- Scripts run with the same privileges as the app (no root).

## Where it's used

- **Discord bot**: When PYTHON_SCRIPT is enabled, Ollama can output `PYTHON_SCRIPT: <id> <topic>` and the code; the app runs it and gives the result back to Ollama.
- **Scheduler**: Same pipeline; scheduled tasks that go through Ollama can use PYTHON_SCRIPT.
- **CPU window chat**: Not in initial scope; can be added to the CPU-window tool loop later if desired.

## References

- **Code:** `src-tauri/src/commands/python_agent.rs`, `src-tauri/src/commands/ollama.rs` (tool loop, agent descriptions, `parse_python_script_from_response`)
- **Config:** `Config::scripts_dir()`, `Config::ensure_scripts_directory()` in `config/mod.rs`
- **All agents:** `docs/100_all_agents.md`
