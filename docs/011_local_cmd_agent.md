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
3.  The reply is parsed; only a line in the form `RUN_CMD: <command>` is accepted. If the model’s reply is not parseable or is another tool, the app sends one more prompt asking for *exactly one line: `RUN_CMD: <command>`* with no other text; if that also fails to parse, the app returns a clear message that the corrected command was not in the required format.
4.  The corrected command is executed. If it succeeds, the output is used. If it fails again, the loop repeats (up to 3 retries).
5.  If all retries fail, or the model never returns a valid RUN_CMD line, the last error (or a format-failure message) is returned.

This handles the common case where the model appends plan commentary to the command arg (e.g. `cat file.json to view the schedule, then REMOVE_SCHEDULE: <id>`), which causes `cat` to fail on the extra words. The retry prompt forces the model to output just the clean command.

## Security

*   Only commands in the allowlist (from the orchestrator’s skill.md or the built-in default) are allowed. No `find`, `sed`, or shell.
*   Path validation ensures no escape from `~/.mac-stats` (canonical path check).
*   Execution is via `sh -c "<stage>"`; the first token (command) must be in the allowlist and path-like tokens are validated to be under `~/.mac-stats`.

### Security review (measures in place)

| Measure | Purpose |
|--------|--------|
| **Allowlist** | Only commands from orchestrator `skill.md` (§ RUN_CMD allowlist) or built-in default (`cat`, `head`, `tail`, `ls`, `grep`, `date`, `whoami`, `ps`, `wc`, `uptime`, `cursor-agent`) can run. No arbitrary binaries. |
| **Path validation** | For path-required commands (`cat`, `head`, `tail`, `grep`), any path-like argument is canonicalized and must be under `~/.mac-stats`. Prevents reading or escaping outside app data. |
| **Shell scope** | `sh -c` runs only the user/agent-provided string; no extra shell profile or global env beyond what the app has. Pipelines and redirects are allowed but each stage's first token must be allowlisted. |
| **cursor-agent caveat** | `cursor-agent` is allowlisted but its arguments are not path-validated; it runs user/agent-controlled prompts in the user environment. To lock down, remove it from the allowlist in the orchestrator's `skill.md`. |
| **ALLOW_LOCAL_CMD** | Set to `0` / `false` / `no` (env or `.config.env`) to disable RUN_CMD entirely in locked-down setups. |

Together these prevent unauthorized access to files outside `~/.mac-stats` and limit execution to a fixed set of commands; the main residual risk is abuse of `cursor-agent` if left on the allowlist.

### Shell injection considerations

The full pipeline stage string is passed to `sh -c "<stage>"`. Only the **first token** is allowlisted and path-like tokens are validated; the rest of the stage is not parsed for further commands. So shell metacharacters in the same stage can run additional commands:

- Example: `RUN_CMD: cat ~/.mac-stats/x; echo pwned` — first token `cat` is allowed, path `~/.mac-stats/x` is valid; the shell then also runs `echo pwned`.
- Same applies to `&&`, `||`, command substitution (backticks or `$(...)`), and newlines within the stage.

**Intentional design:** Pipelines and redirects are supported (e.g. `cat file | grep x`, `date > out`), so the app does not strip or reject shell syntax. The trust boundary is the source of the RUN_CMD line (Ollama model or user). Mitigations in place:

- Allowlist restricts which *leading* command can run (no arbitrary binaries).
- Path validation restricts which files can be read (under `~/.mac-stats`).
- Disable RUN_CMD via `ALLOW_LOCAL_CMD=0` or remove `cursor-agent` from the allowlist for strict lock-down.

A future "strict mode" could run only the first token plus path-validated arguments via `Command::new(cmd).args(...)` without a shell, at the cost of breaking pipelines and redirects; not implemented.

## Where it’s Used

*   **Discord bot**: When RUN_CMD is enabled, Ollama can output `RUN_CMD: <command> [args]`. The app runs it and gives the result back to Ollama.
*   **Scheduler**: Same pipeline; scheduled tasks that go through Ollama can use RUN_CMD.
*   **CPU window chat**: When the CPU-window flow uses the same tool loop, RUN_CMD is available there too.

## References

*   **Code:** `src-tauri/src/commands/run_cmd.rs`, `src-tauri/src/commands/ollama.rs` (tool loop, agent descriptions)
*   **All agents:** `docs/100_all_agents.md`

## More RUN_CMD features (design only)

This section considers possible extensions. No code changes are required; it informs future work.

### More commands

| Candidate | Use case | Notes |
|-----------|----------|--------|
| **sort** | Order lines (e.g. `cat file \| sort`) | Read-only; pipelines already allow it if allowlisted. Low risk. |
| **uniq** | Deduplicate lines | Read-only; useful with `sort`. Low risk. |
| **cut** | Extract columns (e.g. `cut -f1 -d,`) | Read-only; no path required when used in pipe. Low risk. |
| **tr** | Translate/delete characters | Read-only; no path. Low risk. |
| **jq** | JSON query (e.g. `cat file.json \| jq .schedules`) | Very useful for app JSON under `~/.mac-stats`; path comes from prior stage. If allowlisted, path validation applies to any path-like arg (e.g. `jq .x file.json` → `file.json` must be under base). Medium value; requires `jq` on PATH. |
| **xargs** | Build commands from stdin | Higher risk (can run arbitrary commands); generally not recommended for allowlist. |
| **awk** / **sed** | Text processing | Powerful; sed can write files. If ever added, restrict to read-only usage (e.g. sed with no `-i`); complex to enforce. Prefer grep/cut/jq for clarity. |

**Recommendation:** Adding `sort`, `uniq`, `cut`, `tr` to the default allowlist (or documenting them as optional in skill.md) is low-risk and improves usefulness. `jq` is optional and environment-dependent. Avoid `xargs`, and treat `awk`/`sed` as out of scope unless strictly read-only and documented.

### Path validation (current and possible improvements)

**Current behaviour:**

- Path-like tokens: any token that contains `/` or starts with `~` is treated as a path. `~` is expanded with `$HOME`; the path (or its parent if the path does not exist yet) is canonicalized and must be under the permitted base (`~/.mac-stats`).
- Path-required commands: `cat`, `head`, `tail`, `grep` must receive a path (or pipe input); the app returns a clear error if none is given.
- `cursor-agent` is excluded from path validation (runs in user environment with full args).
- Each pipeline stage is validated; path-like tokens in every stage must be under base.

**Possible improvements (design only):**

1. **Stricter “path-like” detection:** Today, a token like `foo/bar` is validated even if it is not a file path (e.g. a URL path). Tightening could reduce false positives but might miss edge cases (e.g. paths with spaces). Current behaviour is conservative (reject unless under base).
2. **Explicit “safe” path allowlist:** Instead of “any path under `~/.mac-stats`”, allow only known subdirs (e.g. `schedules.json`, `config.json`, `agents/`). Would reduce risk if the app later allowed write access in a subdir; not needed for current read-only design.
3. **Validation of redirect targets:** Commands like `cat x > ~/.mac-stats/out` run in the shell; the app does not parse redirects. If write-capable commands were ever allowlisted, redirect targets would need to be validated; currently not required (writes are not in scope).
4. **Better error when path is missing:** For path-required commands, the retry loop already helps. Optional: in the error message, suggest exact examples (e.g. `RUN_CMD: cat ~/.mac-stats/schedules.json`).

No code changes are implied; the above is for documentation and future consideration.

## Open tasks

RUN_CMD open tasks are tracked in **006-feature-coder/FEATURE-CODER.md**. Completed:

*   ~~Update the documentation to better reflect the current implementation and usage of the RUN_CMD agent.~~ **Done:** doc updated to match code (shell execution, allowlist section case-insensitive, pipelines, duplicate detection, TASK_APPEND full output, RUN_CMD naming, retry count, tool iterations).
*   ~~Review the security measures in place to prevent unauthorized access to the app's data and functionality.~~ **Done:** § "Security review (measures in place)" above (allowlist, path validation, shell scope, cursor-agent caveat, ALLOW_LOCAL_CMD).
*   ~~Improve RUN_CMD retry loop (error handling / UX).~~ **Done:** only RUN_CMD lines accepted in fix suggestion; one format-only retry when parse fails; clearer user-facing messages (format required, could not get corrected command).
*   ~~Consider more RUN_CMD features (more commands, path validation).~~ **Done:** § "More RUN_CMD features (design only)" above (candidate commands, path validation current behaviour and possible improvements); doc/design only, no code.