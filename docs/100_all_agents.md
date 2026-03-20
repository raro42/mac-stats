## Install

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

### Menu Bar
- **CPU, GPU, RAM, disk at a glance; click to open the details window.**

### AI Chat
- **Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.**

### Discord Bot
- **Gateway bot for Discord DMs and @mentions.**

## Tool Agents (What Ollama Can Invoke)

### | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). |
| **BRAVE_SEARCH** | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). |
| **RUN_JS** | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in frontend and result sent back to Ollama. In **Discord**: not available; Ollama is told "JavaScript execution is not available in this context." |
| **SKILL** | Run a specialized skill (Markdown system prompt) in a separate Ollama session; result is injected back. | `skills.rs` → `load_skills()`, `find_skill_by_number_or_topic()`; `commands/ollama.rs` → `run_skill_ollama_session()`. Only listed when `~/.mac-stats/agents/skills/` has at least one `skill-<number>-<topic>.md`. |
| **RUN_CMD** | Run a restricted local command (read-only). | `commands/run_cmd.rs` → `run_local_command()`. Allowed: cat, head, tail, ls, grep, date, whoami, ps, wc, uptime, and cursor-agent when in the agent skill allowlist; file paths under ~/.mac-stats; date/whoami need no path. Disabled when `ALLOW_LOCAL_CMD=0`. |
| **TASK** | Create/update task files under ~/.mac-stats/task/ (append feedback, set status open/wip/finished). | `task/mod.rs` (helpers), `commands/ollama.rs` (tool loop). |
| **PYTHON_SCRIPT** | Write script to ~/.mac-stats/scripts/python-script-<id>-<topic>.py, run with python3, return stdout or error. | `commands/python_agent.rs` → `run_python_script()`. Disabled when `ALLOW_PYTHON_SCRIPT=0`. |
| **OLLAMA_API** | List models (full), get version, list running models, pull/delete/load/unload models, generate embeddings. Actions: list_models, version, running, pull, delete, embed, load, unload. | `commands/ollama.rs` (tool loop). |
| **MCP** | Run a tool from the configured MCP server (any server on the internet via HTTP/SSE). | `mcp/` or `commands/mcp.rs` → list tools, `call_tool()`. Requires `MCP_SERVER_URL` (env or `.config.env`). |
| **AGENT** | Run a specialized LLM agent (its own model and prompt: soul + mood + skill). | `agents/mod.rs` → `load_agents()`, `find_agent_by_id_or_name()`; `commands/ollama.rs` → `run_agent_ollama_session()`. Only listed when `~/.mac-stats/agents/` has at least one enabled `agent-<id>/`. |

### Notes on FETCH_URL and RUN_CMD

- `FETCH_URL` is a server-side `reqwest` fetch, not a browser-side fetch. That means normal browser CORS restrictions do not apply; failures are typically due to network, auth, redirects, invalid URLs, or the remote site itself.
- `RUN_CMD` is intentionally restricted: commands are allowlisted, file access is limited to `~/.mac-stats` where applicable, and the tool is designed for read-only local inspection rather than arbitrary shell execution.

## Entry-Point Agents (Who Calls Ollama and How)

### 2.1 Discord Agent (Gateway Bot)

- **Docs:** `docs/007_discord_agent.md`
- **Behaviour:** Listens for DMs and @mentions via Discord Gateway. For each relevant message it calls a shared "answer with Ollama + tools" API.
- **Flow:**
  1. **Planning:** Send user question + list of available tools; ask Ollama to reply with `RECOMMEND: <plan>` (which agents to use, in what order). No execution yet.
  2. **Execution:** Send system prompt + agent descriptions + the plan + user question. Then tool loop: if the model replies with `FETCH_URL:`, `BRAVE_SEARCH:`, `RUN_JS:`, `SKILL:`, `AGENT:`, `RUN_CMD:`, `PYTHON_SCRIPT:`, or `MCP:`, the app runs the tool (FETCH_URL/BRAVE_SEARCH/SKILL/AGENT/RUN_CMD/PYTHON_SCRIPT in Rust; RUN_JS only returns "not available"), appends the result to the conversation, and calls Ollama again. Up to 15 tool iterations (default; overridable per agent).
- **Rust:** `discord/mod.rs` (EventHandler) → `commands::ollama::answer_with_ollama_and_fetch(question, ..., model_override, options_override, skill_content)`. Token from env, `.config.env`, or Keychain (see 007).

## Open tasks:

See **006-feature-coder/FEATURE-CODER.md** for the current FEAT backlog.

- ~~Review whether `run_local_command` is sufficiently hardened against shell-injection-style misuse.~~ **Done:** § "Shell injection considerations" in docs/011_local_cmd_agent.md (full stage passed to `sh -c`; first token allowlisted, path validation; trust boundary and mitigations; strict-mode option documented as future).
- ~~Consider support for more advanced Python-script execution workflows.~~ Deferred: future/backlog (current PYTHON_SCRIPT tool is functional; advanced workflows tracked in FEATURE-CODER when scoped).
- ~~Consider whether additional external tool integrations belong in the agent layer.~~ Deferred: future/backlog (current tools cover web, search, code execution, browser, Redmine, MCP; new integrations added as needed).
- ~~Implement more robust handling for MCP server errors.~~ **Done:** Error-handling § in docs/010_mcp_agent.md; one retry for transient errors in mcp/mod.rs (list_tools, call_tool). See 006-feature-coder/FEATURE-CODER.md.
- ~~Improve the user interface for scheduling tasks.~~ **Done:** Settings → Schedules tab (list, add cron or one-shot, remove); Tauri commands `list_schedules`, `add_schedule`, `add_schedule_at`, `remove_schedule`. See docs/009_scheduler_agent.md and 006-feature-coder/FEATURE-CODER.md.