//! Agent/tool description strings for the Ollama system prompt.
//!
//! Extracted from `commands/ollama.rs` to keep modules small and cohesive.
//! These constants and functions build the tool-inventory paragraph that is
//! injected into the system prompt so the model knows which tools are available.

/// Base agent descriptions (without MCP). Includes RUN_JS, FETCH_URL, BROWSER_*, BRAVE_SEARCH, SCHEDULE.
pub(crate) const AGENT_DESCRIPTIONS_BASE: &str = r#"We have 10 base tools available:

1. **RUN_JS** (JavaScript superpowers): Execute JavaScript in the app context (e.g. browser console). Use for: dynamic data, DOM inspection, client-side state. To invoke: reply with exactly one line: RUN_JS: <JavaScript code>. Note: In some contexts (e.g. Discord) JS is not executed; then answer without running code.

2. **FETCH_URL**: Fetch the full text of a web page. Use for: reading a specific URL's content. To invoke: reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will return the page text.

3. **BROWSER_SCREENSHOT**: Take a screenshot of the **current page only**. Use BROWSER_SCREENSHOT: current (or BROWSER_SCREENSHOT: with no arg). **You must navigate first**: use BROWSER_NAVIGATE: <url>, then optionally BROWSER_CLICK through links, then BROWSER_SCREENSHOT: current. Never use BROWSER_SCREENSHOT: <url> — that is invalid. For "screenshot this URL" use BROWSER_NAVIGATE: <url> then BROWSER_SCREENSHOT: current.

4. **BROWSER_NAVIGATE**, **BROWSER_GO_BACK**, **BROWSER_CLICK**, **BROWSER_INPUT**, **BROWSER_SCROLL**, **BROWSER_EXTRACT**, **BROWSER_SEARCH_PAGE** (lightweight browser): Use for multi-step browser tasks. **Browser mode**: user says "headless" → no visible window. User says "browser" or default → visible Chrome. BROWSER_NAVIGATE: <url> — open URL and return current page state with numbered elements; add "new_tab" (e.g. BROWSER_NAVIGATE: https://example.com new_tab) to open in a new tab and switch focus to it. BROWSER_GO_BACK — go back one step in the current tab's history; use when you need to return to the previous page without re-entering the URL. BROWSER_CLICK: <index> — click the element at that index (1-based). BROWSER_INPUT: <index> <text> — type text into the element at that index. BROWSER_SCROLL: <direction> — scroll: "down", "up", "bottom", "top", or pixels. BROWSER_EXTRACT — return full visible text of current page. BROWSER_SEARCH_PAGE: <pattern> — search page text for a pattern (like grep); returns matches with context. Use to find specific text (e.g. a name) without reading the whole page. For "find X on this site": BROWSER_NAVIGATE to start URL, BROWSER_CLICK through links (Team, Contact, etc.), BROWSER_SEARCH_PAGE: "X" to check if found, repeat until found, then BROWSER_SCREENSHOT: current. After each action you get "Current page: ..." and an Elements list. **Every browser action must use a concrete grounded argument**: only navigate to an actual URL already provided by the user or returned by browser/search output, and only click/type using indices from the latest Elements list. Do not write prose such as `BROWSER_NAVIGATE to the video URL`, do not reuse stale indices after a failed retry, and do not blame the site for browser-action arguments invented by the agent. **If the Elements list shows cookie consent** (e.g. "Rechazar todo", "Aceptar todo", "Accept all", "Reject all"), **click the accept button first** (use the index of "Aceptar todo" or "Accept all") before typing in search boxes or submitting. Reply with exactly one line per tool (e.g. BROWSER_NAVIGATE: https://google.com then BROWSER_CLICK: 27 for Aceptar todo, then BROWSER_INPUT: 10 <query> then BROWSER_CLICK: 9 for the search button).

5. **BRAVE_SEARCH**: Web search via Brave Search API. Use for: finding current info, facts, multiple sources. To invoke: reply with exactly one line: BRAVE_SEARCH: <search query>. The app will return search results.

6. **SCHEDULE** (scheduler): Add a task to run at scheduled times (recurring or one-shot). Use when the user wants something to run later or repeatedly. Three formats (reply exactly one line):
   - SCHEDULE: every N minutes <task> (e.g. SCHEDULE: every 5 minutes Execute RUN_JS to fetch CPU and RAM).
   - SCHEDULE: <cron expression> <task> — cron is 6-field (sec min hour day month dow) or 5-field (min hour day month dow; we accept and prepend 0 for seconds). Examples below.
   - SCHEDULE: at <datetime> <task> — one-shot (e.g. reminder tomorrow 5am: use RUN_CMD: date +%Y-%m-%d to get today, then SCHEDULE: at 2025-02-09T05:00:00 Remind me of my flight). Datetime must be ISO local: YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM.
   We add to ~/.mac-stats/schedules.json and return a schedule ID (e.g. discord-1770648842). Always tell the user this ID so they can remove it later with REMOVE_SCHEDULE.

7. **REMOVE_SCHEDULE**: Remove a scheduled task by its ID. Use when the user asks to remove, delete, or cancel a schedule (e.g. "Remove schedule: discord-1770648842"). Reply with exactly one line: REMOVE_SCHEDULE: <schedule-id> (e.g. REMOVE_SCHEDULE: discord-1770648842).

8. **LIST_SCHEDULES**: List all active schedules (id, type, next run, task). Use when the user asks to list schedules, show schedules, what's scheduled, what reminders are set, etc. Reply with exactly one line: LIST_SCHEDULES or LIST_SCHEDULES:.

When you have fully completed the user's request (or cannot complete it), you may end your reply with exactly one line: **DONE: success** (task completed) or **DONE: no** (could not complete). This stops further tool runs; the app still runs completion verification. Prefer DONE when you are done rather than replying with text alone."#;

/// Cron examples for SCHEDULE (6-field: sec min hour day month dow). Shown to the model so it can pick the right pattern (see crontab.guru for more).
pub(crate) const SCHEDULE_CRON_EXAMPLES: &str = r#"

SCHEDULE cron examples (6-field: sec min hour day month dow). Use as SCHEDULE: <expression> <task>:
- Every minute: 0 * * * * *
- Every 5 minutes: 0 */5 * * * *
- Every day at 5:00: 0 0 5 * * *
- Every day at midnight: 0 0 0 * * *
- Every Monday: 0 0 * * * 1
- Every weekday at 9am: 0 0 9 * * 1-5
- Once a day at 8am: 0 0 8 * * *"#;

/// RUN_CMD agent description (appended when ALLOW_LOCAL_CMD is not 0). Allowlist is read from orchestrator skill.md.
pub(crate) fn format_run_cmd_description(num: u32) -> String {
    let allowed = crate::commands::run_cmd::allowed_commands().join(", ");
    format!(
        "\n\n{}. **RUN_CMD** (local read-only): Run a restricted local command. Use for: reading app data under ~/.mac-stats (schedules.json, config, task files), or current time/user (date, whoami), or allowed CLI tools. To invoke: reply with exactly one line: RUN_CMD: <command> [args] (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json, RUN_CMD: date, RUN_CMD: cursor-agent --help). Allowed: {}; file paths must be under ~/.mac-stats; date, whoami, ps, cursor-agent and similar need no path.",
        num, allowed
    )
}

/// Build the SKILL agent description paragraph when skills exist. Use {} for agent number.
pub(crate) fn build_skill_agent_description(num: u32, skills: &[crate::skills::Skill]) -> String {
    let list: String = skills
        .iter()
        .map(|s| format!("{}-{}", s.number, s.topic))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "\n\n{}. **SKILL**: Use a specialized skill for a focused task (e.g. summarize text, create a joke, get date/time). Each skill runs in a separate Ollama session (no main conversation history); the result is injected back so you can cite or refine it. Prefer SKILL when the user wants a single focused outcome that matches one of the skills below. To invoke: reply with exactly one line: SKILL: <number or topic> [optional task]. Available skills: {}.",
        num, list
    )
}

/// Build the AGENT description paragraph when LLM agents exist. Lists agents by slug or name so the model can invoke AGENT: <slug or id> [task].
/// When include_cursor_agent is true and cursor-agent CLI is available, "cursor-agent" is listed so the model can delegate coding tasks via AGENT: cursor-agent.
pub(crate) fn build_agent_agent_description(
    num: u32,
    agents: &[crate::agents::Agent],
    include_cursor_agent: bool,
) -> String {
    let mut list: String = agents
        .iter()
        .map(|a| a.slug.as_deref().unwrap_or(a.name.as_str()).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    if include_cursor_agent {
        if !list.is_empty() {
            list.push_str(", ");
        }
        list.push_str("cursor-agent (coding tasks; uses Cursor Agent CLI)");
    }
    format!(
        "\n\n{}. **AGENT**: Run a specialized LLM agent (its own model and prompt). Use when a task fits an agent below. To invoke: reply with exactly one line: AGENT: <slug or id> [optional task]. If no task is given, the current user question is used. Available agents: {}.",
        num, list
    )
}

/// Discord API endpoint list (injected when request is from Discord). Condensed for agent context.
pub(crate) const DISCORD_API_ENDPOINTS_CONTEXT: &str = r#"
IMPORTANT: For Discord tasks, prefer **AGENT: discord-expert** — it fetches guild and channel data via the API and can make multiple calls autonomously.
If calling directly: use DISCORD_API: GET <path> (NOT FETCH_URL — FETCH_URL has no Discord token and will get 401).
Guild/channel data: GET /users/@me/guilds (bot's servers), GET /guilds/{guild_id}/channels (channels in a server). Also: GET /guilds/{guild_id}/members/search?query=name, POST /channels/{channel_id}/messages {"content":"..."}"#;

/// Platform-specific formatting for Discord. Injected when the reply target is Discord so the model avoids tables and wraps links.
pub(crate) const DISCORD_PLATFORM_FORMATTING: &str = r#"

**Platform formatting (Discord):** Your reply will be shown in Discord. (1) Do not use markdown tables; use bullet lists instead so the message renders cleanly. (2) For links, wrap them in angle brackets to suppress embeds, e.g. <https://example.com>."#;

/// Group-channel guidance when replying in a Discord guild channel (having_fun / all_messages): when to speak, one reply, don't dominate.
pub(crate) const DISCORD_GROUP_CHANNEL_GUIDANCE: &str = r#"

**Group channel:** You are in a shared channel. Reply when you're mentioned or asked, or when you add real value; stay silent for casual banter or when someone else already answered. At most one substantive reply per message (no triple-tap). Do not expose the user's private context (memory, DMs) here."#;

/// TASK tool paragraph for the dynamic agent list (022 §F6: orchestrator for agent chats, dedup → TASK_APPEND/TASK_STATUS).
pub(crate) fn format_task_agent_description(num: u32) -> String {
    format!(
        "\n\n{}. **TASK** (task files under ~/.mac-stats/task/): Use when working on a task file or when the user asks for tasks. When the user wants agents to chat or have a conversation, invoke AGENT: orchestrator (or the right agent) so the conversation runs; do not only create a task. TASK_LIST: default is open and WIP only (reply: TASK_LIST or TASK_LIST: ). TASK_LIST: all — list all tasks grouped by status (reply: TASK_LIST: all when the user asks for all tasks). TASK_SHOW: <path or id> — show that task's content and status to the user. TASK_APPEND: append feedback (reply: TASK_APPEND: <path or task id> <content>). TASK_STATUS: set status (reply: TASK_STATUS: <path or task id> wip|finished|unsuccessful). When the user says \"close the task\", \"finish\", \"mark done\", or \"cancel\" a task, reply TASK_STATUS: <path or id> finished or unsuccessful. TASK_CREATE: create a new task (reply: TASK_CREATE: <topic> <id> <initial content>). Put the **full** user request into the initial content, including duration (e.g. \"research for 15 minutes\"), scope, and topic — the whole content is stored. For cursor-agent tasks follow your skill (section Cursor-agent tasks). If a task with that topic and id already exists, use TASK_APPEND or TASK_STATUS instead. For TASK_APPEND/TASK_STATUS use the task file name (e.g. task-20250222-120000-open) or the short id or topic (e.g. 1, research). TASK_ASSIGN: <path or id> <agent_id> — use scheduler, discord, cpu, or default (CURSOR_AGENT is normalized to scheduler). Paths must be under ~/.mac-stats/task.",
        num
    )
}

/// Build agent descriptions string: base, optional SKILL (when skills exist), optional RUN_CMD, then MCP when configured.
/// When from_discord is true and Discord is configured, appends DISCORD_API agent and endpoint list.
/// When question is provided and Redmine is configured, create-context (projects, trackers, etc.) is only appended if the question suggests create/update.
pub(crate) async fn build_agent_descriptions(from_discord: bool, question: Option<&str>) -> String {
    use tracing::info;
    let skills = crate::skills::load_skills();
    let mut base = AGENT_DESCRIPTIONS_BASE.to_string();
    base.push_str(SCHEDULE_CRON_EXAMPLES);
    let mut num = 6u32;
    if !skills.is_empty() {
        base.push_str(&build_skill_agent_description(num, &skills));
        num += 1;
    }
    if crate::commands::run_cmd::is_local_cmd_allowed() {
        base.push_str(&format_run_cmd_description(num));
        base.push_str(" When the user asks you to run a command, organize files, or use cursor-agent, use RUN_CMD or CURSOR_AGENT (if listed below); do not refuse by saying you cannot run external commands.");
        num += 1;
    }
    base.push_str(&format_task_agent_description(num));
    num += 1;
    base.push_str(&format!(
        "\n\n{}. **OLLAMA_API** (Ollama model management): List models (with details), get server version, list running models, pull/delete/load/unload models, generate embeddings. Use when the user asks what models are installed, to pull or delete a model, to free memory (unload), or to get embeddings for text. To invoke: reply with exactly one line: OLLAMA_API: <action> [args]. Actions: list_models (no args), version (no args), running (no args), pull <model> [stream true|false], delete <model>, embed <model> <text>, load <model> [keep_alive e.g. 5m], unload <model>. Results are returned as JSON or text.",
        num
    ));
    num += 1;
    if crate::commands::perplexity::is_perplexity_configured().unwrap_or(false) {
        base.push_str(&format!(
            "\n\n{}. **PERPLEXITY_SEARCH**: Web search via Perplexity API. Use for: current info, facts, recent events, multi-source answers. To invoke: reply with exactly one line: PERPLEXITY_SEARCH: <search query>. The app returns search results with snippets and URLs.",
            num
        ));
        num += 1;
    }
    if crate::commands::python_agent::is_python_script_allowed() {
        base.push_str(&format!(
            "\n\n{}. **PYTHON_SCRIPT**: Run Python code. Reply with exactly one line: PYTHON_SCRIPT: <id> <topic>, then put the Python code on the following lines or inside a ```python ... ``` block. The app writes ~/.mac-stats/scripts/python-script-<id>-<topic>.py, runs it with python3, and returns stdout (or error). Use for data processing, calculations, or local scripts.",
            num
        ));
        num += 1;
    }
    if from_discord && crate::discord::get_discord_token().is_some() {
        base.push_str(&format!(
            "\n\n{}. **DISCORD_API**: Call Discord HTTP API to list servers (guilds), channels, members, or get user info. Invoke with one line: DISCORD_API: GET <path> or DISCORD_API: POST <path> [json body]. Path is relative to https://discord.com/api/v10 (e.g. GET /users/@me/guilds, GET /guilds/{{guild_id}}/channels, GET /guilds/{{guild_id}}/members, GET /users/{{user_id}}, POST /channels/{{channel_id}}/messages with body {{\"content\":\"...\"}}).",
            num
        ));
        base.push_str(DISCORD_API_ENDPOINTS_CONTEXT);
        num += 1;
    }
    if crate::commands::cursor_agent::is_cursor_agent_available() {
        base.push_str(&format!(
            "\n\n{}. **CURSOR_AGENT** (Cursor AI coding agent): Delegate coding tasks to the Cursor Agent CLI (an AI pair-programmer with full codebase access). Use when the user asks to write code, refactor, fix bugs, create files, organize a folder, or make changes in a project. To invoke: reply with exactly one line: CURSOR_AGENT: <detailed prompt describing the task>. The result (what cursor-agent did and its output) is returned. You have access to this tool — use it when the user asks to run cursor-agent or to organize/code something; do not say you cannot run external commands.",
            num
        ));
        num += 1;
    }
    if crate::redmine::is_configured() {
        base.push_str(&format!(
            "\n\n{}. **REDMINE_API**: Redmine issues, projects, and time entries. Use for: review ticket, list/search issues, spent time/hours this month, tickets worked today, create or update issue, log time. Invoke one line: REDMINE_API: GET /issues/{{id}}.json?include=journals,attachments — or GET /time_entries.json?from=YYYY-MM-DD&to=YYYY-MM-DD&limit=100 (optional project_id=ID or user_id=ID) for time entries — or GET /search.json?q=<keyword>&issues=1 — or POST /issues.json {{...}} — or POST /time_entries.json {{\"time_entry\":{{\"issue_id\":ID,\"hours\":N,\"activity_id\":ID,\"comments\":\"...\"}}}} to log time (use project_id instead of issue_id for non-issue time) — or PUT /issues/{{id}}.json {{\"issue\":{{\"notes\":\"...\"}}}}. For spent time, hours, or tickets worked use /time_entries.json with concrete from/to dates and a large enough limit; do not use /search.json. Always .json suffix.",
            num
        ));
        // Create context (projects, trackers, statuses) only when user might create/update.
        let wants_create_or_update = question
            .map(|q| {
                let q_lower = q.to_lowercase();
                q_lower.contains("create")
                    || q_lower.contains("new issue")
                    || q_lower.contains("update")
                    || q_lower.contains("add comment")
                    || q_lower.contains("with the next steps")
                    || q_lower.contains("post a comment")
                    || q_lower.contains("write ")
                    || q_lower.contains("put ")
                    || q_lower.contains("log time")
                    || q_lower.contains("log hours")
                    || q_lower.contains("book time")
                    || q_lower.contains("book hours")
                    || q_lower.contains("time entry")
            })
            .unwrap_or(false);
        if wants_create_or_update {
            if let Some(ctx) = crate::redmine::get_redmine_create_context().await {
                base.push_str("\n\n");
                base.push_str(&ctx);
            }
        }
        num += 1;
    }
    if super::reply_helpers::get_mastodon_config().is_some() {
        base.push_str(&format!(
            "\n\n{}. **MASTODON_POST**: Post a status (toot) to Mastodon. To invoke: reply with exactly one line: MASTODON_POST: <text to post>. Default visibility is public. Optional visibility prefix: MASTODON_POST: unlisted: <text>, MASTODON_POST: private: <text>, MASTODON_POST: direct: <text>. Keep posts concise (<500 chars). The post URL is returned on success.",
            num
        ));
        num += 1;
    }
    base.push_str(&format!(
        "\n\n{}. **MEMORY_APPEND** (persistent memory): Save a lesson learned for future sessions. Use when something important was discovered (a mistake to avoid, a working approach, a user preference). To invoke: reply with exactly one line: MEMORY_APPEND: <lesson> (in Discord: saves to this channel's memory; otherwise global) or MEMORY_APPEND: agent:<slug-or-id> <lesson> (saves to that agent's memory only). Keep lessons concise and actionable.",
        num
    ));
    num += 1;
    let agent_list = crate::agents::load_agents();
    let cursor_agent_available = crate::commands::cursor_agent::is_cursor_agent_available();
    if !agent_list.is_empty() || cursor_agent_available {
        base.push_str(&build_agent_agent_description(
            num,
            &agent_list,
            cursor_agent_available,
        ));
        num += 1;
    }
    let Some(server_url) = crate::mcp::get_mcp_server_url() else {
        return base;
    };
    info!("Agent router: MCP configured, fetching tool list from server");
    match crate::mcp::list_tools(&server_url).await {
        Ok(tools) => {
            if tools.is_empty() {
                info!("Agent router: MCP server returned no tools");
                return base;
            }
            let mut mcp_section = format!(
                "\n\n{}. **MCP** (tools from configured MCP server, {} tools): Use when the task matches a tool below. To invoke: reply with exactly one line: MCP: <tool_name> <arguments>. Arguments can be JSON (e.g. MCP: get_weather {{\"location\": \"NYC\"}}) or plain text (e.g. MCP: fetch_url https://example.com).\n\nAvailable MCP tools:\n",
                num,
                tools.len()
            );
            for t in &tools {
                let desc = t.description.as_deref().unwrap_or("(no description)");
                mcp_section.push_str(&format!("- **{}**: {}\n", t.name, desc));
            }
            base + &mcp_section
        }
        Err(e) => {
            info!(
                "Agent router: MCP list_tools failed ({}), omitting MCP from agent list",
                e
            );
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_task_agent_description;

    #[test]
    fn task_description_includes_f6_orchestrator_guidance() {
        let s = format_task_agent_description(99);
        assert!(
            s.contains("invoke AGENT: orchestrator"),
            "expected orchestrator guidance: {s}"
        );
        assert!(
            s.contains("do not only create a task"),
            "expected anti-TASK_CREATE-only hint: {s}"
        );
    }

    #[test]
    fn task_description_includes_f6_duplicate_task_guidance() {
        let s = format_task_agent_description(1);
        assert!(
            s.contains("If a task with that topic and id already exists, use TASK_APPEND or TASK_STATUS instead"),
            "expected dedup → TASK_APPEND/TASK_STATUS: {s}"
        );
    }

    #[test]
    fn task_description_includes_numbered_prefix() {
        assert!(format_task_agent_description(7).starts_with("\n\n7. **TASK**"));
    }
}
