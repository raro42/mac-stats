# Scaling the system prompt: many tools (Redmine, Google, WhatsApp, MCP, …)

## The problem

The agent system prompt already packs a lot of tool text: base tools (RUN_JS, FETCH_URL, BROWSER_*, BRAVE, SCHEDULE), then TASK, OLLAMA_API, PERPLEXITY, PYTHON_SCRIPT, DISCORD_API, CURSOR_AGENT, **REDMINE_API** (long), MASTODON_POST, MEMORY_APPEND, AGENT, and MCP tools. Redmine alone is a dense paragraph plus create-context (projects, trackers, statuses, priorities). Adding Google API, WhatsApp, and hundreds of MCP servers would make the prompt huge: context limits, dilution, and the model struggling to pick the right tool.

**Yes, shortening the system prompt helps.** Less noise → better focus, fewer hallucinations, and more room for conversation and tool results.

## Principles for scaling

### 1. Pre-route instead of “model picks from a list”

We already do this for Redmine: if the user says “review redmine ticket 7209”, we **force** `REDMINE_API: GET /issues/7209.json?...` and never rely on the model to “choose” Redmine from a long list. Same idea for other domains:

- **Intent detection** (keywords, patterns, or a tiny router call): “user wants Redmine” → only add Redmine (short) + maybe FETCH_URL/DONE; “user wants to search the web” → PERPLEXITY or BRAVE + BROWSER_*; “user asked about Discord” → DISCORD or AGENT discord-expert.
- **Inject only 1–3 relevant tools per turn**, not the full catalog. The full list lives in code; the prompt is “this turn you have: X, Y, Z”.

So we don’t scale by listing hundreds of tools in one prompt; we scale by **routing** and then **only describing the tools that are relevant this turn**.

### 2. Short descriptors: name + when + one format

For each tool, the prompt should be:

- **Name** (e.g. REDMINE_API)
- **When to use** (one line): “Review/list/update Redmine issues.”
- **How to invoke** (one line): “REDMINE_API: GET /issues/{id}.json?include=journals,attachments” and “PUT /issues/{id}.json {\"issue\":{\"notes\":\"...\"}}”.

Detailed endpoint lists (all query params, date formats, create-issue example) belong in **code** or a **retrieval step**: when the model says “I want to create an issue”, we can inject a short “Create issue: REDMINE_API: POST /issues.json {\"issue\":{...}}” in the **next** turn or in the tool result, not in the initial system prompt.

### 3. On-demand “create context” (Redmine, etc.)

Redmine “create context” (projects, trackers, statuses, priorities) is only needed when the user will **create** or **update** an issue. For “review ticket 7209”, we don’t need it—we already pre-route to GET. So:

- **Don’t** append full create-context to every prompt when Redmine is configured.
- **Do** append it only when the planner or a previous tool indicated “create issue” / “update issue” (or we add a dedicated “Redmine create” tool that fetches context on first use).

That alone shortens the Redmine block a lot for the common “review” case.

### 4. MCP and external registries

MCP already gives us a list of tools with names and descriptions. For scaling:

- In the **main prompt**, list MCP tools as **name + one-line description** only (as we do now). No full parameter schemas in the system message.
- At **call time**, when the model outputs `MCP: tool_name ...`, we look up the full schema for `tool_name` and execute. So “hundreds of MCP tools” = hundreds of one-liners in the prompt (still a lot) or, better, **MCP tools are also intent-filtered**: if the user asked about Redmine, we don’t inject the full MCP list; we inject “Redmine, FETCH_URL, DONE” and maybe “MCP (N tools available for X)”. Or we have a two-step: “You have MCP. To list tools for a domain, say MCP: list_tools <domain>” and then we inject only that domain’s tools.

So the pattern is: **short list or filtered list in prompt; full schema at execution**.

### 5. One “reference” block, not inline essays

Option: system prompt ends with “Tool reference (short): [link or token]”. We don’t put the full reference in the prompt; we have a separate doc (or retrieval) that we only pull from when the model asks (“What’s the format for Redmine create?”) or when we detect confusion (e.g. tool error “invalid path”). That’s a bigger change but keeps the main prompt minimal.

## Concrete steps (now)

1. **Shorten REDMINE in the prompt**
   - Replace the long paragraph with: when to use (one line), GET issue (one line), PUT notes (one line), search (one line), create (one line). Remove the long endpoint list; rely on pre-route for “review” and on tool-result hints for “create” (e.g. after GET we already say “If the user asked to update, reply REDMINE_API: PUT ...”).
   - **Stop appending create-context (projects, trackers, statuses) to every request.** *Done:* In `build_agent_descriptions`, create-context is appended only when the question suggests create/update (same phrases as pre-route: create, new issue, update, add comment, with the next steps, post a comment, write, put). When `question` is None we do not add create-context.

2. **Pre-route more**
   - Any “review/list/update Redmine ticket N” → force the right REDMINE_API call so the model doesn’t need to “find” it in a long list.
   - Later: “send WhatsApp” → pre-route to WhatsApp tool; “search Google” → pre-route to Google Search, etc.

3. **Intent-based tool subset (later)**
   - Classify the user question (Redmine / Discord / web search / code / MCP / …) and build the agent description with **only** the tools for that domain (+ DONE, maybe FETCH_URL). So the prompt never contains “Redmine + Discord + Google + WhatsApp + 100 MCP tools” at once.

## Summary

| Now | Target |
|-----|--------|
| Long Redmine paragraph + full create-context every time | Short Redmine (when + GET/PUT/search/create one-liner); create-context only when creating/updating |
| All tools in one big prompt | Pre-route so we force the right tool; eventually only inject 1–3 relevant tools per turn |
| MCP: N tools with one-line each (still long if N is huge) | Same short list, but intent-filter so we only add “MCP (for this domain)” or “MCP: list_tools X” pattern |
| Model “picks” from 20+ tools | Router/pre-route picks; model chooses among 2–5 options |

Shortening the system prompt (and Redmine in particular) helps today; scaling to hundreds of tools is done by **not** putting them all in the prompt—pre-route, intent-based subset, and short descriptors with full docs at call time.
