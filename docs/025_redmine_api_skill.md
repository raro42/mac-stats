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
- For text-only time-entry reports, stay in the `REDMINE_API /time_entries.json` flow. Do not switch to browser/screenshot steps or single-issue inspection unless the user explicitly asks for that.
- If a retry is needed, keep the same requested date window and return a user-facing summary from the actual Redmine result or failure. Do not return raw tool directives as the final answer.
- If Redmine is not configured, the URL is invalid, or the host/DNS is unreachable, stop at that blocker. Say the fetch could not be completed and that no Redmine data was fetched; do not claim that no tickets/time entries were found and do not retry the same call in the same turn.
- Do not use `/search.json` for time entries.
- Search issues by keyword: `REDMINE_API: GET /search.json?q=<keyword>&issues=1&limit=100`

### POST (Create)

- Create issue: `REDMINE_API: POST /issues.json {"issue":{"project_id":2,"tracker_id":1,"status_id":1,"priority_id":2,"is_private":false,"subject":"...","description":""}}` (optional: `assigned_to_id`:5)
- Log time entry: `REDMINE_API: POST /time_entries.json {"time_entry":{"issue_id":1234,"hours":1.5,"activity_id":9,"comments":"Investigated bug"}}` (use `project_id` instead of `issue_id` for non-issue time; `spent_on` defaults to today if omitted, format `YYYY-MM-DD`)

### PUT (Add Comment)

- Add comment: `REDMINE_API: PUT /issues/<id>.json {"issue":{"notes":"..."}}`
- Add private note: `REDMINE_API: PUT /issues/<id>.json {"issue":{"notes":"...","private_notes":true}}`

## Configuration

Redmine is configured via environment variables or a config file. The app reads (in order):

1. **Environment**: `REDMINE_URL`, `REDMINE_API_KEY`
2. **Config file**: `.config.env` or `.env.config` in current directory, `src-tauri`, or `~/.mac-stats/.config.env`

Format: one key per line, e.g. `REDMINE_URL=https://redmine.example.com` and `REDMINE_API_KEY=your-api-key`. Values may be quoted. If both env and file set a key, env wins.

- **REDMINE_URL**: Base URL of the Redmine server (no trailing slash). Must be HTTPS in production.
- **REDMINE_API_KEY**: Your Redmine API key (My account → API access). Required for all requests; sent as `X-Redmine-API-Key` header.

If either is missing, the tool returns: "Redmine not configured (REDMINE_URL missing...)" or "...REDMINE_API_KEY missing...".

## Error handling

| Situation | What the user sees |
|-----------|--------------------|
| Not configured | "Redmine not configured (REDMINE_URL missing from env or .config.env)" or same for REDMINE_API_KEY. |
| Invalid or expired API key (401) | "Redmine API 401: Unauthorized" plus a short body snippet. Check API key and that it is enabled in Redmine. |
| Not found (404) | "Redmine API 404: Not Found" plus path/body. Check issue ID, project ID, or path. |
| Validation error (422) | Friendly message: "Redmine didn't accept the query (invalid parameters, e.g. updated_on). Use a date in YYYY-MM-DD or a range like YYYY-MM-DD..YYYY-MM-DD." For other 422 cases the response body may contain field errors. |
| Other HTTP error (4xx/5xx) | "Redmine API &lt;status&gt;: &lt;first 500 chars of body&gt;". |
| Timeout / network failure | "Redmine request failed: &lt;error&gt;" (e.g. connection timeout after 20s). |
| Invalid JSON body (POST/PUT) | "Invalid JSON body: &lt;parse error&gt;". |
| Method/path not allowed | "Redmine API: method X not allowed for path Y (GET anything, POST /issues.json to create, PUT only issues/{id}.json)". |

All errors are returned as the tool result so the LLM (and user) can see why the call failed and adjust.

## Implementation

- **Module**: `src-tauri/src/redmine/mod.rs`
- **Entry**: `redmine_api_request(method, path, body)` — used by the agent router when the model replies with `REDMINE_API: ...`
- **Create context**: Projects, trackers, statuses, priorities, and time entry activities are fetched and cached (5 min TTL) for the create-issue and log-time flows; see `build_agent_descriptions` in the router.
- **Time entries**: GET `/time_entries.json` is handled by a dedicated path that aggregates and summarizes entries (see `fetch_and_summarize_time_entries`).

## Open tasks

Redmine open tasks and completed items are tracked in **agents/006-feature-coder/FEATURE-CODER.md**. Completed:

- ~~Implement a more robust way to handle Redmine API errors.~~ **Done:** 401/404/422 and generic status get clear messages (401: check API key; 404: check ID/path; 422: date format hint); see `redmine_api_request` in `redmine/mod.rs`.
- ~~Improve the documentation for the `REDMINE_API` command.~~ **Done:** Configuration, Error handling (table), and Implementation sections added above.

- ~~Add time entry creation (POST /time_entries.json).~~ **Done:** allowed POST path, activity IDs in create context, agent prompt updated, "log time/hours/book time" triggers context injection.

Remaining: consider adding support for other Redmine features (e.g. issue attachments, custom fields).