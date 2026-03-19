## Installation

### DMG (recommended)

[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source

```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```

Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Gatekeeper workaround

If macOS blocks the app, right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

*   **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
*   **AI chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
*   **Discord bot**: When RUN_CMD is enabled, Ollama can output `RUN_CMD: <command> [args]`. The app runs it and gives the result back to Ollama.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Allowed Tools

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## RUN_CMD Agent

The RUN_CMD agent lets Ollama read app data by running restricted local commands. Only read-only commands are allowed, and only for paths under `~/.mac-stats`. Commands are executed via a shell (`sh -c "<command>"`) so that redirects (`>`, `>>`), pipes (`|`), and semicolons (`;`) work as expected.

### Overview

*   **Agent name**: RUN_CMD  
*   **Invocation**: Ollama replies with one line: `RUN_CMD: <command> [args]` (e.g. `RUN_CMD: cat ~/.mac-stats/schedules.json`, `RUN_CMD: date`, `RUN_CMD: whoami`, or `RUN_CMD: ls ~/.mac-stats`).  
*   The app runs the command via a shell (`sh -c`) so that redirects (`>`, `>>`), pipes (`|`), and semicolons (`;`) work. Path-like arguments are validated to be under `~/.mac-stats` where applicable. Stdout (or an error message) is injected back into the conversation.

## Setup

*   **Default**: RUN_CMD is **enabled** by default (no API key or URL required).
*   **Disable**: Set `ALLOW_LOCAL_CMD=0` (or `false`, `no`, `off`) in the environment or in `.config.env` to disable the agent in locked-down setups.

## Allowlist and Path Rules

*   **Allowlist source**: The list of allowed commands is read from the **first enabled orchestrator** agent’s `skill.md` (section `## RUN_CMD allowlist`, case-insensitive). One line, comma- or newline-separated (e.g. `cat, head, tail, ls, grep, date, whoami, ps, wc, uptime, cursor-agent`). If that section is missing or empty, the built-in default is used (same list). Edit `~/.mac-stats/agents/agent-000/skill.md` (or whichever agent is your orchestrator) to add or remove commands.
*   **Path-required commands**: Only `cat`, `head`, `tail`, and `grep` require a path argument under `~/.mac-stats`. All other allowed commands (e.g. `date`, `whoami`, `ps`, `cursor-agent`) can be run with no path.
*   **Security — cursor-agent**: `cursor-agent` in the allowlist runs user/ or agent-controlled prompts in the user environment; its arguments are not path-validated. It is a privileged capability. To lock down, remove `cursor-agent` from the RUN_CMD allowlist in your orchestrator’s `skill.md` (see `## RUN_CMD allowlist`).
*   **Paths**: Any argument that looks like a path (contains `/` or starts with `~`) must resolve to a location under `~/.mac-stats`. Paths are expanded (`~` → `$HOME`) and validated (canonical form must be under the permitted base). Paths outside `~/.mac-stats` are rejected with "Path not allowed (must be under ~/.mac-stats)."
*   **Shell execution**: The app runs each pipeline stage with `sh -c "<stage>"` so that redirects, pipes, and semicolons are interpreted. The first token of each stage must be in the allowlist; path-like arguments are validated to be under `~/.mac-stats`.
*   **`ls` with no path**: If the user invokes `RUN_CMD: ls` with no arguments, the app runs `ls` (no path), and the shell will run it from the current working directory; for listing app data use e.g. `RUN_CMD: ls ~/.mac-stats`. **`date` and `whoami`** need no path; use e.g. `RUN_CMD: date` or `RUN_CMD: whoami`.
*   **Pipelines**: Commands can be chained with `|` (e.g. `RUN_CMD: ps aux | grep tail`). Each stage runs via `sh -c`; the first token of each stage must be in the allowlist; path-like arguments in each stage must be under `~/.mac-stats`.

## Behaviour

*   When RUN_CMD is enabled, the agent list sent to Ollama in the planning and execution steps includes RUN_CMD (position depends on agent order; MCP if configured appears after).
*   Ollama invokes with `RUN_CMD: <command> [args]` (e.g. `RUN_CMD: cat ~/.mac-stats/schedules.json`). The app runs the command via shell, validates paths where required, and injects the result (or an error) into the conversation.
*   The tool loop (Discord, scheduler, CPU-window flow) supports RUN_CMD like other tools; each RUN_CMD call counts as one tool iteration (subject to the agent's max tool iterations).
*   **Duplicate detection**: If the model sends the same RUN_CMD argument as the previous run, the app skips execution and tells the model to use the result already in the conversation (avoids loops).
*   **TASK_APPEND after RUN_CMD**: When the model runs RUN_CMD then TASK_APPEND, the app appends the full command output to the task file (not a summary), so task files get the actual data.

## Retry Loop (AI-assisted error correction)

When a command fails (non-zero exit code), the app does **not** give up immediately. Instead it enters a retry loop (up to 3 retries):

1.  The error message (e.g. `cat: to: No such file or directory`) is sent to Ollama in a focused, minimal prompt: *"The command `<cmd>` failed with error: `<error>`. Reply with ONLY the corrected command: `RUN_CMD: <corrected command>`."*
2.  Ollama returns the corrected command (e.g. `RUN_CMD: cat ~/.mac-stats/schedules.json`).
3.  The corrected command is extracted via `parse_tool_from_response` and executed.
4.  If it succeeds, the output is used. If it fails again, the loop repeats (up to 3 retries).
5.  If all retries fail, the last error is returned.

This handles the common case where the model appends plan commentary to the command arg (e.g. `cat file.json to view the schedule, then REMOVE_SCHEDULE: <id>`), which causes `cat` to fail on the extra words. The retry prompt forces the model to output just the clean command.

## Security

*   Only commands in the allowlist (from the orchestrator’s skill.md or the built-in default) are allowed. No `find`, `sed`, or shell.
*   Path validation ensures no escape from `~/.mac-stats` (canonical path check).
*   Execution is via `sh -c "<stage>"`; the first token (command) must be in the allowlist and path-like tokens are validated to be under `~/.mac-stats`.

## Where it’s Used

*   **Discord bot**: When RUN_CMD is enabled, Ollama can output `RUN_CMD: <command> [args]`. The app runs it and gives the result back to Ollama.
*   **Scheduler**: Same pipeline; scheduled tasks that go through Ollama can use RUN_CMD.
*   **CPU window chat**: When the CPU-window flow uses the same tool loop, RUN_CMD is available there too.

## References

*   **Code:** `src-tauri/src/commands/run_cmd.rs`, `src-tauri/src/commands/ollama.rs` (tool loop, agent descriptions)
*   **All agents:** `docs/100_all_agents.md`

## Open tasks:

*   Improve the retry loop for better error handling and user experience.
*   Consider adding more features to the RUN_CMD agent, such as support for more commands or improved path validation.
*   Review the security measures in place to prevent unauthorized access to the app's data and functionality.
*   ~~Update the documentation to better reflect the current implementation and usage of the RUN_CMD agent.~~ **Done:** doc updated to match code (shell execution, allowlist section case-insensitive, pipelines, duplicate detection, TASK_APPEND full output, RUN_CMD naming, retry count, tool iterations).