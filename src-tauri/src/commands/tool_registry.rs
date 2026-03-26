//! Declarative registry of agent tool names and metadata (single source of truth).
//!
//! Tool line prefixes, prompt summaries, and parse-time validation derive from [`TOOLS`].
//! Dispatch handlers remain in [`crate::commands::tool_loop::dispatch_tool`].

use std::sync::OnceLock;

/// Metadata for one invocable agent tool (names, prompt text, parsing hints).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub accepts_argument: bool,
    pub requires_browser: bool,
    /// Tool changes the browser page or tab context; queued tools after it may be unsafe.
    pub terminates_navigation: bool,
}

/// Canonical tool list: names, descriptions, and flags. Keep in sync with `dispatch_tool`.
pub(crate) static TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "FETCH_URL",
        description: "Fetch and return cleaned text for an HTTP(S) URL.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BRAVE_SEARCH",
        description: "Web search via Brave Search API.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_SCREENSHOT",
        description: "Capture the current browser tab; use after BROWSER_NAVIGATE.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_SAVE_PDF",
        description: "Export the current tab to PDF (CDP Page.printToPDF); use after BROWSER_NAVIGATE. Not supported in HTTP fallback.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_NAVIGATE",
        description: "Open a URL in the browser (optionally new_tab).",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: true,
    },
    ToolDef {
        name: "BROWSER_GO_BACK",
        description: "History back in the focused tab.",
        accepts_argument: false,
        requires_browser: true,
        terminates_navigation: true,
    },
    ToolDef {
        name: "BROWSER_GO_FORWARD",
        description: "History forward in the focused tab.",
        accepts_argument: false,
        requires_browser: true,
        terminates_navigation: true,
    },
    ToolDef {
        name: "BROWSER_RELOAD",
        description: "Reload the current page; optional hard/nocache.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: true,
    },
    ToolDef {
        name: "BROWSER_CLEAR_COOKIES",
        description: "Clear persisted browser cookies and CDP cookie jar.",
        accepts_argument: false,
        requires_browser: true,
        terminates_navigation: true,
    },
    ToolDef {
        name: "BROWSER_SWITCH_TAB",
        description: "Focus a tab by 0-based index.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: true,
    },
    ToolDef {
        name: "BROWSER_CLOSE_TAB",
        description: "Close a tab by 0-based index.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: true,
    },
    ToolDef {
        name: "BROWSER_CLICK",
        description: "Click element index or viewport coordinates.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_HOVER",
        description: "Move pointer to element index (hover menus; CDP mouseMoved only).",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_DRAG",
        description: "Drag from one element index to another (reorder, sliders, drop zones).",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_INPUT",
        description: "Type into an element by index.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_UPLOAD",
        description: "Attach files to a file input via CDP.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_KEYS",
        description: "Send an allowlisted key chord to the page.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_SCROLL",
        description: "Scroll the page (direction or pixels).",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_EXTRACT",
        description: "Extract page content as markdown (optional no_images).",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_SEARCH_PAGE",
        description: "Substring search over visible text with optional CSS scope.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_QUERY",
        description: "querySelectorAll with optional attribute list.",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "BROWSER_DOWNLOAD",
        description: "Wait for a file download to finish (optional timeout seconds, default 30, max 120).",
        accepts_argument: true,
        requires_browser: true,
        terminates_navigation: false,
    },
    ToolDef {
        name: "PERPLEXITY_SEARCH",
        description: "Web search via Perplexity API.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "RUN_JS",
        description: "Execute JavaScript on the host via Node when runJsEnabled is true in ~/.mac-stats/config.json (default true; set false to refuse).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "SKILL",
        description: "Run a numbered skill in a side session.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "AGENT",
        description: "Delegate to another LLM agent by slug.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "RUN_CMD",
        description: "Run an allowlisted local shell command.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "SCHEDULE",
        description: "Add a cron, interval, or one-shot schedule entry.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "SCHEDULER",
        description: "Alias for SCHEDULE (same line format; normalized to SCHEDULE).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "REMOVE_SCHEDULE",
        description: "Remove a schedule by id.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "LIST_SCHEDULES",
        description: "List active schedules (optional LIST_SCHEDULES: ).",
        accepts_argument: false,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "TASK_LIST",
        description: "List task files (optional TASK_LIST: all).",
        accepts_argument: false,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "TASK_SHOW",
        description: "Show one task file by path or id.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "TASK_APPEND",
        description: "Append content to a task file.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "TASK_STATUS",
        description: "Set task status (wip / finished / unsuccessful).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "TASK_CREATE",
        description: "Create a new task file.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "TASK_ASSIGN",
        description: "Assign a task to an agent id.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "TASK_SLEEP",
        description: "Record a sleep/wait instruction on a task.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "OLLAMA_API",
        description: "Ollama server management (models, pull, embed, etc.).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "MCP",
        description: "Invoke a tool on the configured MCP server.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "PYTHON_SCRIPT",
        description: "Run a Python script under ~/.mac-stats/scripts/.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "DISCORD_API",
        description: "Discord REST API call (path + optional JSON body).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "CURSOR_AGENT",
        description: "Delegate a coding task to the Cursor Agent CLI.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "REDMINE_API",
        description: "Redmine REST API call for issues, time entries, etc.",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "MEMORY_APPEND",
        description: "Append a persistent memory note (global, channel, or agent).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
    ToolDef {
        name: "DONE",
        description: "End the tool loop (success or no).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: true,
    },
    ToolDef {
        name: "MASTODON_POST",
        description: "Post a status to Mastodon (optional visibility prefix).",
        accepts_argument: true,
        requires_browser: false,
        terminates_navigation: false,
    },
];

static TOOL_LINE_PREFIXES_CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Upper-case tool prefixes with trailing colon, derived from [`TOOLS`].
pub(crate) fn tool_line_prefixes() -> &'static [&'static str] {
    TOOL_LINE_PREFIXES_CACHE
        .get_or_init(|| {
            TOOLS
                .iter()
                .map(|t| {
                    let s = format!("{}:", t.name);
                    Box::leak(s.into_boxed_str()) as &'static str
                })
                .collect()
        })
        .as_slice()
}

static KNOWN_PREFIX_SET: OnceLock<std::collections::HashSet<String>> = OnceLock::new();

fn known_prefix_upper_set() -> &'static std::collections::HashSet<String> {
    KNOWN_PREFIX_SET.get_or_init(|| {
        tool_line_prefixes()
            .iter()
            .map(|p| p.to_ascii_uppercase())
            .collect()
    })
}

/// True if `prefix` (e.g. `FETCH_URL:`) is a registered tool line prefix (ASCII case-insensitive).
pub(crate) fn is_known_tool_prefix(prefix: &str) -> bool {
    known_prefix_upper_set().contains(&prefix.to_ascii_uppercase())
}

/// Normalizes the parsed tool label for `dispatch_tool` (`SCHEDULER` → `SCHEDULE`).
pub(crate) fn map_scheduler_alias(name: &str) -> &str {
    if name.eq_ignore_ascii_case("SCHEDULER") {
        "SCHEDULE"
    } else {
        name
    }
}

/// Short registry section for the system prompt (canonical names + one-line descriptions).
pub(crate) fn tool_descriptions_for_prompt() -> String {
    let mut out = String::from(
        "**Canonical tool registry** (exact names; each tool is one line `NAME: args` unless noted):\n\n",
    );
    for t in TOOLS {
        if t.name == "SCHEDULER" {
            continue;
        }
        out.push_str("- **");
        out.push_str(t.name);
        out.push_str("** — ");
        let desc = if t.name == "RUN_JS" && !crate::config::Config::run_js_enabled() {
            "Disabled: runJsEnabled=false in ~/.mac-stats/config.json. Do not call; returns runJsEnabled=false refusal."
        } else {
            t.description
        };
        out.push_str(desc);
        out.push('\n');
    }
    out.push_str("\n**Scheduler synonym:** `SCHEDULER:` is accepted and treated as `SCHEDULE:`.\n");
    out
}

static INLINE_TOOL_CHAIN_RE: OnceLock<regex::Regex> = OnceLock::new();

/// Regex used to split inline tool chains (`then TOOL: arg`), built from [`TOOLS`].
pub(crate) fn inline_tool_chain_regex() -> &'static regex::Regex {
    INLINE_TOOL_CHAIN_RE.get_or_init(|| {
        let mut names: Vec<&'static str> = TOOLS.iter().map(|t| t.name).collect();
        names.sort_by_key(|n| std::cmp::Reverse(n.len()));
        let alt = names.join("|");
        let pat = format!(
            r"(?i)(?:\b(?:then|and then|and|after that|afterward|afterwards|next|finally)\b|;|->)\s+({})(?::)?\s+",
            alt
        );
        regex::Regex::new(&pat).expect("inline tool chain regex must compile")
    })
}

/// If `search` looks like `TOOL_LIKE:` but is not registered, log a warning (model hallucination).
pub(crate) fn warn_if_unknown_tool_like_prefix(search: &str) {
    let s = search.trim();
    let Some(colon_pos) = s.find(':') else {
        return;
    };
    let head = s[..colon_pos].trim();
    if head.len() < 2 || !head.contains('_') {
        return;
    }
    if !head
        .bytes()
        .all(|b| b.is_ascii_uppercase() || b == b'_' || b.is_ascii_digit())
    {
        return;
    }
    if !head.starts_with(|c: char| c.is_ascii_uppercase()) {
        return;
    }
    let pseudo_prefix = format!("{}:", head);
    if is_known_tool_prefix(&pseudo_prefix) {
        return;
    }
    crate::mac_stats_warn!(
        "tool_parse",
        "Unknown tool-like prefix in model line (not in registry): {} (line prefix {:?})",
        pseudo_prefix,
        s.chars().take(120).collect::<String>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_line_prefixes_match_tool_names() {
        let prefs = tool_line_prefixes();
        assert_eq!(prefs.len(), TOOLS.len());
        for (p, t) in prefs.iter().zip(TOOLS.iter()) {
            assert_eq!(*p, format!("{}:", t.name));
        }
    }

    #[test]
    fn map_scheduler_alias_normalizes_scheduler() {
        assert_eq!(map_scheduler_alias("SCHEDULER"), "SCHEDULE");
        assert_eq!(map_scheduler_alias("scheduler"), "SCHEDULE");
        assert_eq!(map_scheduler_alias("SCHEDULE"), "SCHEDULE");
    }

    #[test]
    fn tool_descriptions_lists_all_dispatch_tools_except_scheduler_alias() {
        let s = tool_descriptions_for_prompt();
        for t in TOOLS {
            if t.name == "SCHEDULER" {
                continue;
            }
            assert!(
                s.contains(&format!("**{}**", t.name)),
                "missing tool {} in prompt block",
                t.name
            );
        }
    }

    /// Every `dispatch_tool` arm tool name should appear in the registry (static guard).
    #[test]
    fn registry_covers_dispatch_tool_arms() {
        const DISPATCH_TOOLS: &[&str] = &[
            "FETCH_URL",
            "BROWSER_SCREENSHOT",
            "BROWSER_SAVE_PDF",
            "BROWSER_NAVIGATE",
            "BROWSER_GO_BACK",
            "BROWSER_GO_FORWARD",
            "BROWSER_RELOAD",
            "BROWSER_CLEAR_COOKIES",
            "BROWSER_SWITCH_TAB",
            "BROWSER_CLOSE_TAB",
            "BROWSER_CLICK",
            "BROWSER_HOVER",
            "BROWSER_DRAG",
            "BROWSER_INPUT",
            "BROWSER_UPLOAD",
            "BROWSER_KEYS",
            "BROWSER_SCROLL",
            "BROWSER_EXTRACT",
            "BROWSER_SEARCH_PAGE",
            "BROWSER_QUERY",
            "BROWSER_DOWNLOAD",
            "BRAVE_SEARCH",
            "PERPLEXITY_SEARCH",
            "RUN_JS",
            "SKILL",
            "AGENT",
            "SCHEDULE",
            "REMOVE_SCHEDULE",
            "LIST_SCHEDULES",
            "RUN_CMD",
            "PYTHON_SCRIPT",
            "DISCORD_API",
            "OLLAMA_API",
            "TASK_APPEND",
            "TASK_STATUS",
            "TASK_CREATE",
            "TASK_SHOW",
            "TASK_ASSIGN",
            "TASK_SLEEP",
            "TASK_LIST",
            "MCP",
            "CURSOR_AGENT",
            "REDMINE_API",
            "MASTODON_POST",
            "MEMORY_APPEND",
        ];
        for name in DISPATCH_TOOLS {
            assert!(
                TOOLS.iter().any(|t| t.name == *name),
                "dispatch tool {} missing from TOOLS registry",
                name
            );
        }
    }
}
