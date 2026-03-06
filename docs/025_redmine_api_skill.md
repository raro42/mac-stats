## Installation

- **DMG (recommended):** [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.
- **Build from source:**
  ```bash
  git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
  ```

## At a Glance

- **Menu Bar**: Displays CPU, GPU, RAM, and disk usage; click to open the details window.
- **AI Chat**: Ollama chat in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents (Invocable Tools)

Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Available Tools

| Agent | Invocation | Purpose |
|-------|------------|---------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript |

## Redmine API

Use `REDMINE_API` for Redmine. Reply with exactly one line: `REDMINE_API: <METHOD> <path> [body]`.

The standalone Redmine agent runs in an isolated tool loop: it should see only the current task and `REDMINE_API` results, not session-compactor output or other agent summaries.
That loop now normalizes `RECOMMEND:` wrappers and inline mixed-tool text before parsing, then executes the first allowed Redmine call instead of getting stuck on a leading unsupported tool like `RUN_CMD`.

### GET (Read)

- Full issue with comments and attachments: `REDMINE_API: GET /issues/1234.json?include=journals,attachments`
- My open issues: `REDMINE_API: GET /issues.json?assigned_to_id=me&status_id=open`
- Project issues: `REDMINE_API: GET /issues.json?project_id=ID&status_id=open&limit=25`
- List projects: `REDMINE_API: GET /projects.json`
- **Time entries (spent time/hours):** `REDMINE_API: GET /time_entries.json?from=YYYY-MM-DD&to=YYYY-MM-DD&limit=100`. Use current-month range for “this month”.
- **Tickets worked on today:** use the same-day range, for example `REDMINE_API: GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100`.
- **Tickets worked on yesterday:** use the previous UTC day, for example `REDMINE_API: GET /time_entries.json?from=2026-03-05&to=2026-03-05&limit=100`.
- The standalone Redmine agent now receives the current local date and UTC date in its runtime context. For “today”, it should use the injected current UTC date unless the task explicitly asks for local time.
- Only add optional filters like `project_id` or `user_id` if the user explicitly asked for them.
- Derive the concrete dates directly in the Redmine plan. Do not chain `RUN_CMD` and `REDMINE_API` just to compute the date.
- Do not use `/search.json` for time entries.
- Search issues by keyword: `REDMINE_API: GET /search.json?q=<keyword>&issues=1&limit=100`

### POST (Create)

- Create issue: `REDMINE_API: POST /issues.json {"issue":{"project_id":2,"tracker_id":1,"status_id":1,"priority_id":2,"is_private":false,"subject":"...","description":""}}` (optional: `assigned_to_id`:5)

### PUT (Add Comment)

- Add comment: `REDMINE_API: PUT /issues/<id>.json {"issue":{"notes":"..."}}`
- Add private note: `REDMINE_API: PUT /issues/<id>.json {"issue":{"notes":"...","private_notes":true}}`

## Open tasks:

- Investigate why Gatekeeper blocks the app on some systems.
- Implement a more robust way to handle Redmine API errors.
- Improve the documentation for the `REDMINE_API` command.
- Consider adding support for other Redmine features, such as issue attachments and custom fields.