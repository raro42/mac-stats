# RUN_CMD Agent (Local Command Execution)

The RUN_CMD agent lets Ollama read app data by running restricted local commands. Only read-only commands are allowed, and only for paths under `~/.mac-stats`. No shell is used; the app parses the command and arguments and runs them via `std::process::Command`.

## Overview

- **Agent name**: RUN_CMD  
- **Invocation**: Ollama replies with one line: `RUN_CMD: <command> [args]` (e.g. `RUN_CMD: cat ~/.mac-stats/schedules.json` or `RUN_CMD: ls ~/.mac-stats`).  
- The app runs the command (no shell), validates that any path arguments are under `~/.mac-stats`, and injects stdout (or an error message) back into the conversation.

When RUN_CMD is disabled via `ALLOW_LOCAL_CMD=0`, the agent is omitted from the list Ollama sees so the model does not try to use it.

## Setup

- **Default**: RUN_CMD is **enabled** by default (no API key or URL required).
- **Disable**: Set `ALLOW_LOCAL_CMD=0` (or `false`, `no`, `off`) in the environment or in `.config.env` (current directory, `src-tauri/`, or `~/.mac-stats/.config.env`) to disable the agent in locked-down setups.

## Allowlist and path rules

- **Allowed commands**: `cat`, `head`, `tail`, `ls`, `grep`. Any other command is rejected with "Command not allowed". For `grep`, use e.g. `RUN_CMD: grep pattern ~/.mac-stats/task/file.md` (pattern and path required).
- **Paths**: Any argument that looks like a path (contains `/` or starts with `~`) must resolve to a location under `~/.mac-stats`. Paths are expanded (`~` → `$HOME`) and validated (canonical form must be under the permitted base). Paths outside `~/.mac-stats` are rejected with "Path not allowed (must be under ~/.mac-stats)."
- **No shell**: The app does not invoke a shell. It splits the RUN_CMD argument on whitespace, validates the command and path args, and runs the binary with the given arguments.
- **`ls` with no path**: If the user invokes `RUN_CMD: ls` with no arguments, the app runs `ls` with the permitted base directory (`~/.mac-stats`) so only that directory is listed.

## Behaviour

- When RUN_CMD is enabled, the agent list sent to Ollama in the planning and execution steps includes RUN_CMD (as agent 5 when MCP is not configured, or 5 with MCP as 6).
- Ollama can reply with `RUN_CMD: cat ~/.mac-stats/schedules.json`. The app runs the command, validates paths, and injects the result (or an error) into the conversation.
- The tool loop (Discord, scheduler, and when wired CPU-window flow) supports RUN_CMD like other tools; each RUN_CMD call counts as one tool iteration (max 5).

## Security

- Only the five commands above are allowed (cat, head, tail, ls, grep). No `find`, `sed`, or shell.
- Path validation ensures no escape from `~/.mac-stats` (canonical path check).
- Do not pass user input to a shell; all execution is via `Command::new(cmd).args(args)`.

## Where it’s used

- **Discord bot**: When RUN_CMD is enabled, Ollama can output `RUN_CMD: <command> [args]`. The app runs it and gives the result back to Ollama.
- **Scheduler**: Same pipeline; scheduled tasks that go through Ollama can use RUN_CMD.
- **CPU window chat**: When the CPU-window flow uses the same tool loop, RUN_CMD is available there too.

## References

- **Code:** `src-tauri/src/commands/run_cmd.rs`, `src-tauri/src/commands/ollama.rs` (tool loop, agent descriptions)
- **All agents:** `docs/100_all_agents.md`
