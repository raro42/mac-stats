## Installation
### Recommended Method
Download the latest release from [GitHub](https://github.com/raro42/mac-stats/releases/latest) and drag the `.dmg` file to the Applications folder.

### Building from Source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Workaround for Gatekeeper
If macOS blocks the app, right-click the `.dmg` file and select **Open**, then confirm. Alternatively, run `xattr -rd com.apple.quarantine /Applications/mac-stats.app` after installation.

## At a Glance
### Menu Bar
- Displays CPU, GPU, RAM, and disk usage at a glance.
- Click to open the details window.

### AI Chat
- Ollama chat available in the app or via Discord.
- Supports FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, and MCP.

## Tool Agents
Ollama can invoke the following tool agents:
| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch web page's body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Agent Workflow
This repo is edited by the **mac-stats-reviewer** Coder agent. Changes are applied **in place** (yolo mode): the Coder edits files directly and does not produce patches.

## Direct Edits (yolo mode)
For the Coder to edit files in this repo directly:
1. Open Cursor with **`mac-stats-agent-workspace.code-workspace`** (located in the mac-stats-reviewer repo).
2. That workspace includes both **mac-stats** and **mac-stats-reviewer**, so the Coder can edit files here in place.
3. Do **not** produce patches or ask for permission; edit files directly.

## Open tasks:
- Improve the documentation for tool agents and their invocations.
- Consider implementing a more robust patching system for the Coder agent.
- Review the Brave Search API usage and ensure it complies with the Brave API key requirements.