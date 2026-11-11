// included by gateway.rs — status_chips
/// Fail words for Insights and run counts. Zero is calm (overview “no fails”), not a problem count.
pub(super) fn fail_status_bit(n: usize) -> String {
    if n == 0 {
        "**no fails**".to_string()
    } else if n == 1 {
        "**1** fail".to_string()
    } else {
        format!("**{n}** fails")
    }
}


/// True for short `/status` / `/health` / `/version` operator asks — not free-form “status of …”.
/// View/see/show me/open/list-the NL (v0.1.1038). Exact `open status` / `open the status` only
/// (same for health / version). After normalize strips `show me` / `show`, prefer `the status`
/// etc. so `show me the status` still matches.
pub fn looks_like_status_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains(" of ")
        || n.contains(" for ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" about ")
        || n.contains("path")
        || n.contains("where")
        || n.contains("size")
        || contains_age_token(&n)
        || n.contains("changelog")
        || n.contains("ship")
        || n.contains("bump")
        || n.contains("release")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/status"
            | "status"
            | "the status"
            | "bot status"
            | "app status"
            | "mac-stats status"
            | "system status"
            | "view status"
            | "view the status"
            | "see status"
            | "see the status"
            | "show status"
            | "show the status"
            | "show me status"
            | "show me the status"
            | "open status"
            | "open the status"
            | "list status"
            | "list the status"
            | "/health"
            | "health"
            | "the health"
            | "health check"
            | "bot health"
            | "system health"
            | "app health"
            | "mac-stats health"
            | "how healthy"
            | "are you healthy"
            | "view health"
            | "view the health"
            | "see health"
            | "see the health"
            | "show health"
            | "show the health"
            | "show me health"
            | "show me the health"
            | "open health"
            | "open the health"
            | "list health"
            | "list the health"
            | "/version"
            | "version"
            | "the version"
            | "app version"
            | "mac-stats version"
            | "what version"
            | "which version"
            | "view version"
            | "view the version"
            | "see version"
            | "see the version"
            | "show version"
            | "show the version"
            | "show me version"
            | "show me the version"
            | "open version"
            | "open the version"
            | "list version"
            | "list the version"
            | "is everything ok"
            | "everything ok"
            | "everything working"
            | "all systems go"
            | "systems ok"
            | "system ok"
            | "all good on your end"
            | "you all good"
            | "are you ok"
            | "are you okay"
    )
}


/// One-screen operator status: version, Discord gateway, digest, next schedule, last delivery.
pub fn format_status_gateway() -> String {
    let version = crate::config::Config::version();
    let digest = load_digest_summary();
    let snap = crate::scheduler::scheduler_operator_snapshot();
    let mut lines = vec![
        format!("**mac-stats v{version}**"),
        crate::discord::format_discord_gateway_insights_line(),
        format!(
            "Digest: {} · {}{}",
            digest_open_status_bit(digest.open_count),
            digest_stale_status_bit(digest.stale_count),
            if digest.source.is_empty() {
                String::new()
            } else {
                format!(" · {}", digest.source)
            }
        ),
    ];
    let mut sched = format!("Schedules: **{}**", snap.total_entries);
    if let Some(secs) = snap.seconds_until_next_fire {
        let when = if secs < 3600 {
            format!("{}m", (secs / 60).max(1))
        } else {
            format!("{}h", (secs + 1800) / 3600)
        };
        let preview = snap
            .next_task_preview
            .as_deref()
            .map(|p| format!(" ({})", p.chars().take(36).collect::<String>()))
            .unwrap_or_default();
        sched.push_str(&format!(" · next {when}{preview}"));
    }
    lines.push(sched);
    if let Some(last) = crate::scheduler::list_scheduler_delivery_awareness()
        .into_iter()
        .next()
    {
        let preview: String = last.summary.chars().take(72).collect();
        lines.push(format!("Last delivery: {} · {}", last.utc, preview));
    }
    lines.join("\n")
}


/// True for focused Discord gateway asks (`/discord` · Ready/Offline) — not Knowledge Discord,
/// not free-form “post to Discord…”.
/// View/see/show me/open/list-the NL (v0.1.1037). Exact `open discord` / `open the discord` only.
pub fn looks_like_discord_gateway_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Knowledge / memory / post / message stay with their own handlers or the agent.
    // Path/json asks go to discord_channels.json instant (v0.1.837).
    if n.contains("knowledge")
        || n.contains("memory")
        || n.contains("post")
        || n.contains("send")
        || n.contains("message")
        || n.contains("channel list")
        || n.contains("list channel")
        || n.contains("guild")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("path")
        || n.contains("where")
        || n.contains("location")
        || n.contains("folder")
        || n.contains("directory")
        || n.contains("json")
        || n.contains("discord_channels")
        || n.contains("channels.json")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/discord"
            | "discord"
            | "discord status"
            | "discord gateway"
            | "discord ready"
            | "discord offline"
            | "gateway status"
            | "gateway ready"
            | "bot gateway"
            | "show discord"
            | "show the discord"
            | "show me discord"
            | "show me the discord"
            | "view discord"
            | "view the discord"
            | "see discord"
            | "see the discord"
            | "open discord"
            | "open the discord"
            | "list discord"
            | "list the discord"
            | "the discord"
            | "is discord ready"
            | "is discord online"
            | "is discord connected"
            | "is discord offline"
            | "is the bot ready"
            | "is the bot connected"
            | "is the gateway ready"
            | "is the gateway connected"
            | "discord connection"
            | "discord reconnect"
            | "discord disconnects"
            | "how's discord"
            | "hows discord"
            | "how's the discord"
            | "hows the discord"
            | "how's the gateway"
            | "hows the gateway"
    )
}


/// Zero-LLM Discord Ready / Offline chip (Agent Ops collapsed-glance parity).
pub fn format_discord_gateway_chip() -> String {
    let ready = crate::discord::discord_bot_gateway_ready();
    let token = crate::discord::discord_bot_token_configured();
    let (ready_n, resume_n, disc_n) = crate::discord::discord_gateway_reconnect_stats();
    let stage = crate::discord::discord_last_shard_stage()
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "unknown".into());
    let ready_ago = crate::discord::discord_last_ready_at()
        .map(|t| format!("{}s ago", t.elapsed().as_secs()))
        .unwrap_or_else(|| "never".into());
    let stage_lower = stage.to_lowercase();
    if !token {
        return "**Discord** · Offline · no token in Keychain".to_string();
    }
    if stage_lower == "disconnected" || !ready {
        let mut line = format!("**Discord** · Offline · last Ready {ready_ago}");
        if disc_n > 0 {
            line.push_str(&format!(" · disc×{disc_n}"));
        }
        if !stage.is_empty() && stage != "unknown" {
            line.push_str(&format!(" · stage={stage}"));
        }
        return line;
    }
    let mut line = if resume_n > 0 && disc_n == 0 && stage_lower == "resuming" {
        format!("**Discord** · Resuming · {ready_ago}")
    } else {
        format!("**Discord** · Ready · {ready_ago}")
    };
    if disc_n > 0 {
        line.push_str(&format!(" · disc×{disc_n}"));
    } else if resume_n > 0 && !line.contains("Resuming") {
        line.push_str(&format!(" · res×{resume_n}"));
    }
    line.push_str(&format!(" · stage={stage} · ready×{ready_n}"));
    line
}


/// True for focused Ollama Ready/Offline asks (`/ollama` · menu-bar ✕ / AI Chat glance parity) —
/// not pull/list/chat/API free-form.
/// View/see/show me/open/list-the NL (v0.1.1037). Exact `open ollama` / `open the ollama` /
/// `open llm` / `open the llm` only — not pull/list models/chat.
pub fn looks_like_ollama_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Pull / list / chat / configure stay with agent or OLLAMA_API pre-route.
    // Exact `list ollama` / `list the ollama` / `list llm` stay Ready chips (allow list).
    if n.contains("pull")
        || n.contains("push")
        || n.contains("list model")
        || n.contains("list models")
        || n.contains("unload")
        || n.contains("load model")
        || n.contains("embed")
        || n.contains("api")
        || n.contains("chat with")
        || n.contains("ask ollama")
        || n.contains("ask the model")
        || n.contains("install")
        || n.contains("download")
        || n.contains("change model")
        || n.contains("set model")
        || n.contains("set url")
        || n.contains("configure")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" ticket")
        || n.contains("redmine")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ollama"
            | "/llm"
            | "ollama"
            | "llm"
            | "ollama status"
            | "llm status"
            | "ollama ready"
            | "ollama offline"
            | "llm ready"
            | "llm offline"
            | "show ollama"
            | "show the ollama"
            | "show me ollama"
            | "show me the ollama"
            | "show llm"
            | "show the llm"
            | "show me llm"
            | "show me the llm"
            | "view ollama"
            | "view the ollama"
            | "view llm"
            | "view the llm"
            | "see ollama"
            | "see the ollama"
            | "see llm"
            | "see the llm"
            | "open ollama"
            | "open the ollama"
            | "open llm"
            | "open the llm"
            | "list ollama"
            | "list the ollama"
            | "list llm"
            | "list the llm"
            | "the ollama"
            | "the llm"
            | "is ollama ready"
            | "is ollama online"
            | "is ollama connected"
            | "is ollama offline"
            | "is ollama down"
            | "is the llm ready"
            | "is the llm online"
            | "is the llm connected"
            | "is the llm offline"
            | "is the llm down"
            | "ollama connection"
            | "llm connection"
            | "ollama circuit"
            | "how's ollama"
            | "hows ollama"
            | "how's the llm"
            | "hows the llm"
            | "how's ollama doing"
            | "hows ollama doing"
            | "/model"
            | "which model"
            | "what model"
            | "current model"
            | "your model"
            | "model name"
            | "ollama model"
            | "which model are you"
            | "what model are you"
            | "which model are you using"
            | "what model are you using"
            | "which model are you running"
            | "what model are you running"
            | "which model are you running on"
            | "what model are you running on"
            | "which model am i using"
            | "what model am i using"
            | "which model is configured"
            | "what model is configured"
            | "ollama endpoint"
            | "ollama url"
            | "ollama host"
            | "ollama address"
            | "ollama server"
            | "the ollama endpoint"
            | "the ollama url"
            | "the ollama host"
            | "the ollama server"
            | "llm endpoint"
            | "llm url"
            | "llm host"
            | "where is ollama"
            | "where's ollama"
            | "wheres ollama"
            | "where is the ollama"
            | "where is the llm"
            | "what is the ollama url"
            | "what's the ollama url"
            | "whats the ollama url"
            | "what is the ollama endpoint"
            | "what's the ollama endpoint"
            | "whats the ollama endpoint"
            | "what is the ollama host"
            | "what's the ollama host"
            | "whats the ollama host"
    )
}


/// Zero-LLM Ollama Ready / Offline chip (menu-bar Ollama ✕ + AI Chat model-glance parity).
pub fn format_ollama_ready_chip() -> String {
    let cfg = crate::commands::ollama_config::get_ollama_config();
    let circuit_open = crate::ollama::ollama_http_circuit_is_open_for_menu();
    let Some(c) = cfg else {
        return "**Ollama** · Not set · configure URL".to_string();
    };
    let ep = shorten_ollama_endpoint_for_chip(&c.endpoint);
    let model = {
        let m = c.model.trim();
        if m.is_empty() {
            "no model".to_string()
        } else {
            m.to_string()
        }
    };
    if circuit_open {
        return format!("**Ollama** · Offline · circuit open · {model} · {ep}");
    }
    let mut line = if model == "no model" {
        format!("**Ollama** · Ready · no model · pick one · {ep}")
    } else {
        format!("**Ollama** · Ready · {model} · {ep}")
    };
    if let Some(backend) = c.detected_backend.as_deref() {
        if !backend.is_empty() && backend != "unknown" {
            line.push_str(&format!(" · {backend}"));
        }
    }
    line
}


/// True for focused Redmine Ready/config asks (`/redmine` · Agent Ops health parity) —
/// not ticket/issue/time-entry/API free-form.
pub fn looks_like_redmine_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Tickets, issues, time, API how-to stay with pre-route / agent.
    if n.contains("ticket")
        || n.contains("issue")
        || n.contains("time entr")
        || n.contains("spent")
        || n.contains(" hours")
        || n.contains("api")
        || n.contains("create")
        || n.contains("update")
        || n.contains("comment")
        || n.contains("search")
        || n.contains("review")
        || n.contains("journal")
        || n.contains("attachment")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/redmine"
            | "redmine"
            | "redmine status"
            | "redmine ready"
            | "redmine offline"
            | "redmine configured"
            | "redmine health"
            | "show redmine"
            | "show the redmine"
            | "show me redmine"
            | "show me the redmine"
            | "view redmine"
            | "view the redmine"
            | "see redmine"
            | "see the redmine"
            | "open redmine"
            | "open the redmine"
            | "list redmine"
            | "list the redmine"
            | "the redmine"
            | "is redmine ready"
            | "is redmine online"
            | "is redmine connected"
            | "is redmine offline"
            | "is redmine configured"
            | "is redmine set up"
            | "is redmine setup"
            | "redmine connection"
            | "how's redmine"
            | "hows redmine"
            | "how's the redmine"
            | "hows the redmine"
            | "redmine url"
            | "redmine key"
    )
}


/// Zero-LLM Redmine Ready / Not set chip (Agent Ops health Redmine parity; config only).
pub fn format_redmine_ready_chip() -> String {
    let url = crate::redmine::get_redmine_url();
    let key = crate::redmine::get_redmine_api_key();
    match (url.as_deref(), key.as_deref()) {
        (None, None) => {
            "**Redmine** · Not set · add URL + API key (Settings or .config.env)".to_string()
        }
        (Some(_), None) => {
            "**Redmine** · Partial · URL set · missing API key (Settings)".to_string()
        }
        (None, Some(_)) => {
            "**Redmine** · Partial · API key set · missing URL (Settings)".to_string()
        }
        (Some(u), Some(_)) => {
            let host = shorten_redmine_url_for_chip(u);
            format!("**Redmine** · Ready · {host} · key set")
        }
    }
}


/// True for focused Brave Search key/config asks (`/brave` · Settings key parity) —
/// not web-search / BRAVE_SEARCH free-form.
pub fn looks_like_brave_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Search queries and tool how-to stay with pre-route / agent.
    if n.contains("search for")
        || n.contains("look up")
        || n.contains("look for")
        || n.contains("google")
        || n.contains("research")
        || n.contains("brave_search")
        || n.contains("find ")
        || n.contains("web search")
        || n.contains("query")
        || n.contains("results")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.starts_with("search ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    // Bare "brave search" often means "search the web" — require status/key/ready cues.
    if n == "brave search" {
        return false;
    }
    matches!(
        n.as_str(),
        "/brave"
            | "brave"
            | "brave status"
            | "brave ready"
            | "brave offline"
            | "brave configured"
            | "brave health"
            | "brave key"
            | "show brave"
            | "show the brave"
            | "show me brave"
            | "show me the brave"
            | "show brave search"
            | "show the brave search"
            | "show me brave search"
            | "show me the brave search"
            | "view brave"
            | "view the brave"
            | "view brave search"
            | "view the brave search"
            | "see brave"
            | "see the brave"
            | "see brave search"
            | "see the brave search"
            | "open brave"
            | "open the brave"
            | "open brave search"
            | "open the brave search"
            | "list brave"
            | "list the brave"
            | "list brave search"
            | "list the brave search"
            | "the brave"
            | "the brave search"
            | "is brave ready"
            | "is brave online"
            | "is brave connected"
            | "is brave offline"
            | "is brave configured"
            | "is brave set up"
            | "is brave setup"
            | "brave connection"
            | "how's brave"
            | "hows brave"
            | "how's the brave"
            | "hows the brave"
            | "brave search status"
            | "brave search key"
            | "brave search ready"
            | "brave search health"
            | "brave search configured"
            | "is brave search ready"
            | "is brave search configured"
            | "is brave search set up"
            | "is brave search setup"
            | "how's brave search"
            | "hows brave search"
    )
}


/// Zero-LLM Brave Search Ready / Not set chip (config only; no live ping / quota burn).
pub fn format_brave_ready_chip() -> String {
    match crate::commands::brave::get_brave_api_key() {
        Some(_) => "**Brave Search** · Ready · key set".to_string(),
        None => "**Brave Search** · Not set · add API key (Settings or BRAVE_API_KEY)".to_string(),
    }
}


/// True for focused Perplexity API key/config asks (`/perplexity key` · Settings key parity) —
/// not last-search Top/Snippet (`/perplexity`) or new search free-form.
/// View/see/show me/open/list-the NL (v0.1.1037) — key phrases only (`view perplexity key`,
/// not bare `view perplexity` / `list perplexity` which stay last-search).
pub fn looks_like_perplexity_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Last-search list, live search, and how-to stay with `/perplexity` / pre-route / agent.
    if n.contains("search for")
        || n.contains("look up")
        || n.contains("look for")
        || n.contains("research")
        || n.contains("results")
        || n.contains("snippet")
        || n.contains(" last ")
        || n.starts_with("last ")
        || n.contains("top result")
        || n == "top results"
        || n.contains("query")
        || n.contains("find ")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.contains("path")
        || n.contains("where")
        || n.contains("location")
        || n.contains("folder")
        || n.contains("directory")
        || n.contains("json")
        || n.contains("perplexity_last")
        || n.contains("cache file")
        // Live search: "perplexity search for …" (status/key/ready stay in the allow list below).
        || (n.starts_with("perplexity search ")
            && !matches!(
                n.as_str(),
                "perplexity search status"
                    | "perplexity search key"
                    | "perplexity search ready"
                    | "perplexity search health"
                    | "perplexity search configured"
            ))
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    // Bare `/perplexity` / `perplexity` / `perplexity search` + view/see/open NL stay last-search list.
    if matches!(
        n.as_str(),
        "/perplexity"
            | "perplexity"
            | "the perplexity"
            | "perplexity search"
            | "/perplexity top"
            | "perplexity top"
            | "view perplexity top"
            | "see perplexity top"
            | "open perplexity top"
            | "list the perplexity top"
            | "/perplexity snippet"
            | "perplexity snippet"
            | "view perplexity snippet"
            | "see perplexity snippet"
            | "open perplexity snippet"
            | "list the perplexity snippet"
            | "/top"
            | "/snippet"
            | "snippet"
            | "snippets"
            | "last search"
            | "the last search"
            | "last perplexity"
            | "the last perplexity"
            | "list perplexity"
            | "list the perplexity"
            | "show perplexity"
            | "show the perplexity"
            | "show me perplexity"
            | "show me the perplexity"
            | "view perplexity"
            | "view the perplexity"
            | "see perplexity"
            | "see the perplexity"
            | "open perplexity"
            | "open the perplexity"
            | "view last search"
            | "see last search"
            | "open last search"
            | "list the last search"
            | "search results"
            | "perplexity results"
            | "view perplexity results"
            | "see perplexity results"
            | "open perplexity results"
            | "list the perplexity results"
            | "top results"
            | "the top results"
            | "view top results"
            | "see top results"
            | "open top results"
            | "list the top results"
            | "snippet results"
            | "the snippet results"
            | "view snippet results"
            | "see snippet results"
            | "open snippet results"
            | "list the snippet results"
    ) {
        return false;
    }
    matches!(
        n.as_str(),
        "/perplexity key"
            | "perplexity key"
            | "perplexity key status"
            | "perplexity status"
            | "perplexity ready"
            | "perplexity offline"
            | "perplexity configured"
            | "perplexity health"
            | "show perplexity key"
            | "show the perplexity key"
            | "show me perplexity key"
            | "show me the perplexity key"
            | "view perplexity key"
            | "view the perplexity key"
            | "see perplexity key"
            | "see the perplexity key"
            | "open perplexity key"
            | "open the perplexity key"
            | "list perplexity key"
            | "list the perplexity key"
            | "the perplexity key"
            | "is perplexity ready"
            | "is perplexity online"
            | "is perplexity connected"
            | "is perplexity offline"
            | "is perplexity configured"
            | "is perplexity set up"
            | "is perplexity setup"
            | "perplexity connection"
            | "how's perplexity"
            | "hows perplexity"
            | "how's the perplexity"
            | "hows the perplexity"
            | "perplexity search status"
            | "perplexity search key"
            | "perplexity search ready"
            | "perplexity search health"
            | "perplexity search configured"
            | "show perplexity search key"
            | "show the perplexity search key"
            | "view perplexity search key"
            | "view the perplexity search key"
            | "see perplexity search key"
            | "see the perplexity search key"
            | "open perplexity search key"
            | "open the perplexity search key"
            | "list perplexity search key"
            | "list the perplexity search key"
            | "the perplexity search key"
            | "is perplexity search ready"
            | "is perplexity search configured"
            | "is perplexity search set up"
            | "is perplexity search setup"
            | "how's perplexity search"
            | "hows perplexity search"
    )
}


/// Zero-LLM Perplexity Ready / Not set chip (Settings key parity; config only; no live probe).
pub fn format_perplexity_ready_chip() -> String {
    match crate::commands::perplexity::get_perplexity_api_key() {
        Some(_) => "**Perplexity Search** · Ready · key set".to_string(),
        None => "**Perplexity Search** · Not set · add API key".to_string(),
    }
}


/// True for focused Mastodon Ready/config asks (`/mastodon` · MASTODON_POST config parity) —
/// not toot/post/timeline free-form.
pub fn looks_like_mastodon_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Posts, toots, timeline, and tool how-to stay with pre-route / agent.
    if n.contains("post ")
        || n.starts_with("post ")
        || n.contains("toot")
        || n.contains("publish")
        || n.contains("timeline")
        || n.contains("follow")
        || n.contains("boost")
        || n.contains("favourite")
        || n.contains("favorite")
        || n.contains("mastodon_post")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/mastodon"
            | "mastodon"
            | "mastodon status"
            | "mastodon ready"
            | "mastodon offline"
            | "mastodon configured"
            | "mastodon health"
            | "mastodon key"
            | "mastodon token"
            | "mastodon url"
            | "show mastodon"
            | "show the mastodon"
            | "show me mastodon"
            | "show me the mastodon"
            | "view mastodon"
            | "view the mastodon"
            | "see mastodon"
            | "see the mastodon"
            | "open mastodon"
            | "open the mastodon"
            | "list mastodon"
            | "list the mastodon"
            | "the mastodon"
            | "is mastodon ready"
            | "is mastodon online"
            | "is mastodon connected"
            | "is mastodon offline"
            | "is mastodon configured"
            | "is mastodon set up"
            | "is mastodon setup"
            | "mastodon connection"
            | "how's mastodon"
            | "hows mastodon"
            | "how's the mastodon"
            | "hows the mastodon"
    )
}


/// Zero-LLM Mastodon Ready / Not set / Partial chip (config only; no live probe).
pub fn format_mastodon_ready_chip() -> String {
    let url = crate::commands::reply_helpers::get_mastodon_instance_url();
    let token = crate::commands::reply_helpers::get_mastodon_access_token();
    match (url.as_deref(), token.as_deref()) {
        (None, None) => {
            "**Mastodon** · Not set · add instance URL + access token (Settings or .config.env)"
                .to_string()
        }
        (Some(_), None) => {
            "**Mastodon** · Partial · URL set · missing access token (Settings)".to_string()
        }
        (None, Some(_)) => {
            "**Mastodon** · Partial · token set · missing instance URL (Settings)".to_string()
        }
        (Some(u), Some(_)) => {
            let host = shorten_redmine_url_for_chip(u);
            format!("**Mastodon** · Ready · {host} · token set")
        }
    }
}


/// True for focused MCP Ready/config asks (`/mcp` · MCP_SERVER_* config parity) —
/// not `MCP: <tool>` invocations or tool how-to.
pub fn looks_like_mcp_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Tool invocations and how-to stay with pre-route / agent.
    // Exact `list mcp` / `list the mcp` stay Ready chips (allow list); tool dumps stay out.
    if n.starts_with("mcp:")
        || n.contains("mcp:")
        || n.contains("mcp tool")
        || n.contains("list tools")
        || n.contains("list mcp tool")
        || n.contains("call mcp")
        || n.contains("use mcp")
        || n.contains("invoke")
        || n.contains("ori_")
        || n.contains("mnemos")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/mcp"
            | "mcp"
            | "mcp status"
            | "mcp ready"
            | "mcp offline"
            | "mcp configured"
            | "mcp health"
            | "mcp key"
            | "mcp url"
            | "mcp stdio"
            | "mcp server"
            | "mcp connection"
            | "show mcp"
            | "show the mcp"
            | "show me mcp"
            | "show me the mcp"
            | "show mcp server"
            | "show the mcp server"
            | "show me mcp server"
            | "show me the mcp server"
            | "view mcp"
            | "view the mcp"
            | "view mcp server"
            | "view the mcp server"
            | "see mcp"
            | "see the mcp"
            | "see mcp server"
            | "see the mcp server"
            | "open mcp"
            | "open the mcp"
            | "open mcp server"
            | "open the mcp server"
            | "list mcp"
            | "list the mcp"
            | "list mcp server"
            | "list the mcp server"
            | "the mcp"
            | "the mcp server"
            | "is mcp ready"
            | "is mcp online"
            | "is mcp connected"
            | "is mcp offline"
            | "is mcp configured"
            | "is mcp set up"
            | "is mcp setup"
            | "how's mcp"
            | "hows mcp"
            | "how's the mcp"
            | "hows the mcp"
            | "mcp server status"
            | "mcp server ready"
            | "mcp server health"
            | "mcp server configured"
            | "is mcp server ready"
            | "is mcp server configured"
            | "is mcp server set up"
            | "is mcp server setup"
            | "how's mcp server"
            | "hows mcp server"
    )
}


/// Zero-LLM MCP Ready / Not set chip (config only; no tools/list live probe).
pub fn format_mcp_ready_chip() -> String {
    match crate::mcp::get_mcp_server_url() {
        None => {
            "**MCP** · Not set · add MCP_SERVER_URL or MCP_SERVER_STDIO (Settings or .config.env)"
                .to_string()
        }
        Some(cfg) => {
            if let Some(rest) = cfg.strip_prefix("stdio:") {
                let cmd = rest
                    .split('|')
                    .next()
                    .unwrap_or("stdio")
                    .trim();
                let short = if cmd.chars().count() > 36 {
                    let mut s: String = cmd.chars().take(33).collect();
                    s.push('…');
                    s
                } else {
                    cmd.to_string()
                };
                format!("**MCP** · Ready · stdio · {short}")
            } else {
                let host = shorten_redmine_url_for_chip(&cfg);
                format!("**MCP** · Ready · http · {host}")
            }
        }
    }
}


/// True for focused Cursor agent Ready/PATH asks (`/cursor` · `/cursor-agent`) —
/// not `CURSOR_AGENT:` tool invocations or coding handoffs.
/// View/see/show me/open/list-the NL (v0.1.1037). Exact `open cursor` / `open the cursor` /
/// `open cursor agent` / `open cursor-agent` only — not CURSOR_AGENT: / ask/run/use.
pub fn looks_like_cursor_agent_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Tool invocations and coding tasks stay with pre-route / agent.
    if n.contains("cursor_agent:")
        || n.contains("cursor-agent:")
        || n.starts_with("cursor_agent ")
        || n.contains("run cursor")
        || n.contains("ask cursor")
        || n.contains("use cursor")
        || n.contains("invoke")
        || n.contains("implement")
        || n.contains("refactor")
        || n.contains("fix ")
        || n.contains("write ")
        || n.contains("commit")
        || n.contains("push")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/cursor"
            | "/cursor-agent"
            | "cursor"
            | "cursor agent"
            | "cursor-agent"
            | "cursor_agent"
            | "cursor status"
            | "cursor agent status"
            | "cursor-agent status"
            | "cursor ready"
            | "cursor agent ready"
            | "cursor-agent ready"
            | "cursor offline"
            | "cursor agent offline"
            | "cursor-agent offline"
            | "cursor configured"
            | "cursor agent configured"
            | "cursor-agent configured"
            | "cursor health"
            | "cursor agent health"
            | "cursor-agent health"
            | "cursor path"
            | "cursor agent path"
            | "cursor-agent path"
            | "show cursor"
            | "show the cursor"
            | "show me cursor"
            | "show me the cursor"
            | "show cursor agent"
            | "show the cursor agent"
            | "show me cursor agent"
            | "show me the cursor agent"
            | "show cursor-agent"
            | "show the cursor-agent"
            | "show me cursor-agent"
            | "show me the cursor-agent"
            | "view cursor"
            | "view the cursor"
            | "view cursor agent"
            | "view the cursor agent"
            | "view cursor-agent"
            | "view the cursor-agent"
            | "see cursor"
            | "see the cursor"
            | "see cursor agent"
            | "see the cursor agent"
            | "see cursor-agent"
            | "see the cursor-agent"
            | "open cursor"
            | "open the cursor"
            | "open cursor agent"
            | "open the cursor agent"
            | "open cursor-agent"
            | "open the cursor-agent"
            | "list cursor"
            | "list the cursor"
            | "list cursor agent"
            | "list the cursor agent"
            | "list cursor-agent"
            | "list the cursor-agent"
            | "the cursor"
            | "the cursor agent"
            | "the cursor-agent"
            | "is cursor ready"
            | "is cursor online"
            | "is cursor connected"
            | "is cursor offline"
            | "is cursor configured"
            | "is cursor set up"
            | "is cursor setup"
            | "is cursor agent ready"
            | "is cursor agent online"
            | "is cursor agent connected"
            | "is cursor agent offline"
            | "is cursor agent configured"
            | "is cursor agent set up"
            | "is cursor agent setup"
            | "is cursor-agent ready"
            | "is cursor-agent online"
            | "is cursor-agent connected"
            | "is cursor-agent offline"
            | "is cursor-agent configured"
            | "is cursor-agent set up"
            | "is cursor-agent setup"
            | "cursor connection"
            | "cursor agent connection"
            | "cursor-agent connection"
            | "how's cursor"
            | "hows cursor"
            | "how's the cursor"
            | "hows the cursor"
            | "how's cursor agent"
            | "hows cursor agent"
            | "how's the cursor agent"
            | "hows the cursor agent"
            | "how's cursor-agent"
            | "hows cursor-agent"
            | "how's the cursor-agent"
            | "hows the cursor-agent"
    )
}


/// Zero-LLM Cursor agent Ready / Not set chip (PATH / Settings path; no CLI probe).
pub fn format_cursor_agent_ready_chip() -> String {
    let ws = crate::commands::cursor_agent::cursor_agent_workspace();
    let ws_short = std::path::Path::new(&ws)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&ws);
    if crate::commands::cursor_agent::is_cursor_agent_available() {
        let bin = crate::commands::cursor_agent::cursor_agent_executable();
        let bin_short = std::path::Path::new(&bin)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&bin);
        if crate::commands::cursor_agent::cursor_agent_executable_configured() {
            format!("**Cursor agent** · Ready · `{bin_short}` · `{ws_short}`")
        } else {
            format!("**Cursor agent** · Ready · `{bin_short}` on PATH · `{ws_short}`")
        }
    } else {
        "**Cursor agent** · Not set · install `cursor-agent` on PATH or set path in Settings".to_string()
    }
}


/// True for focused Browser / CDP Ready asks (`/browser` · `/cdp`) —
/// not BROWSER_* tools, screenshots, or navigate/click tasks.
/// View/see/show me/open/list-the NL (v0.1.1028). Exact `open browser` / `open the browser` /
/// `open cdp` / `open the cdp` only — not `open page` / `open url` (already rejected).
/// Path/size/age / credentials / downloads / cookies lanes stay separate.
pub fn looks_like_browser_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Tool invocations and browse tasks stay with pre-route / agent.
    if n.starts_with("browser_")
        || n.contains("browser_")
        || n.contains("browser:")
        || n.contains("screenshot")
        || n.contains("navigate")
        || n.contains("click")
        || n.contains("scroll")
        || n.contains("hover")
        || n.contains("browse ")
        || n.contains("open page")
        || n.contains("open url")
        || n.contains("take a")
        || n.contains("capture")
        || n.contains("http://")
        || n.contains("https://")
        || n.contains("www.")
        || n.contains("launch chrome")
        || n.contains("start chrome")
        || n.contains("start chromium")
        || n.contains("invoke")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.contains("path")
        || n.contains("folder")
        || n.contains("directory")
        || n.contains("where")
        || n.contains("location")
        || n.contains("size")
        || n.contains("how big")
        || n.contains("how old")
        || n.ends_with(" age")
        || n.contains(" age ")
        || n.contains("credential")
        || n.contains("download")
        || n.contains("cookie")
        || n.contains("storage")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/browser"
            | "/cdp"
            | "browser"
            | "cdp"
            | "browser status"
            | "browser ready"
            | "browser offline"
            | "browser configured"
            | "browser health"
            | "browser tools"
            | "cdp status"
            | "cdp ready"
            | "cdp offline"
            | "cdp configured"
            | "cdp health"
            | "cdp port"
            | "chromium status"
            | "chromium ready"
            | "list browser"
            | "list the browser"
            | "list cdp"
            | "list the cdp"
            | "show browser"
            | "show the browser"
            | "show me browser"
            | "show me the browser"
            | "show cdp"
            | "show the cdp"
            | "show me cdp"
            | "show me the cdp"
            | "view browser"
            | "view the browser"
            | "view cdp"
            | "view the cdp"
            | "see browser"
            | "see the browser"
            | "see cdp"
            | "see the cdp"
            | "open browser"
            | "open the browser"
            | "open cdp"
            | "open the cdp"
            | "the browser"
            | "the cdp"
            | "is browser ready"
            | "is browser online"
            | "is browser connected"
            | "is browser offline"
            | "is browser configured"
            | "is browser set up"
            | "is browser setup"
            | "is browser enabled"
            | "is cdp ready"
            | "is cdp online"
            | "is cdp connected"
            | "is cdp offline"
            | "is cdp configured"
            | "is cdp set up"
            | "is cdp setup"
            | "how's browser"
            | "hows browser"
            | "how's the browser"
            | "hows the browser"
            | "how's cdp"
            | "hows cdp"
            | "how's the cdp"
            | "hows the cdp"
            | "browser connection"
            | "cdp connection"
            | "browser cdp"
            | "cdp browser"
    )
}


/// Zero-LLM Browser / CDP Ready chip (config + binary path only; no live `/json/version` probe).
pub fn format_browser_ready_chip() -> String {
    let port = crate::config::Config::browser_cdp_port();
    if !crate::config::Config::browser_tools_enabled() {
        return format!(
            "**Browser** · Off · set `browserToolsEnabled` true · CDP {port}"
        );
    }
    let path = crate::config::Config::browser_chromium_executable_path();
    let chrome_ok = if path.is_absolute() {
        path.is_file()
    } else {
        // Relative / PATH-style name (Linux default): treat as present without probing PATH.
        true
    };
    let short = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Chromium");
    let short = if short.chars().count() > 28 {
        let mut s: String = short.chars().take(25).collect();
        s.push('…');
        s
    } else {
        short.to_string()
    };
    if chrome_ok {
        format!("**Browser** · Ready · CDP {port} · `{short}` · idle until BROWSER_*")
    } else if crate::config::Config::browser_chromium_executable_configured() {
        format!(
            "**Browser** · Not set · Chromium missing · CDP {port} · fix path in Settings or `browserChromiumExecutable`"
        )
    } else {
        format!(
            "**Browser** · Not set · install Google Chrome · CDP {port} (Settings or `browserChromiumExecutable`)"
        )
    }
}


/// True for focused agent-judge Ready/config asks (`/judge`) —
/// not “run the judge”, score this turn, or enable/disable how-tos.
/// View/see/show me/open/list-the NL (v0.1.1029). Exact `open judge` / `open the judge` /
/// `open agent judge` only — not `judge this` / run / score / enable.
pub fn looks_like_judge_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Actions / how-tos stay with pre-route / agent.
    if n.contains("run judge")
        || n.contains("run the judge")
        || n.contains("judge this")
        || n.contains("judge that")
        || n.contains("judge my")
        || n.contains("judge the")
        || n.contains("score this")
        || n.contains("score the")
        || n.contains("verdict for")
        || n.contains("enable judge")
        || n.contains("disable judge")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("invoke")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/judge"
            | "judge"
            | "judge status"
            | "judge ready"
            | "judge offline"
            | "judge configured"
            | "judge health"
            | "judge mode"
            | "agent judge"
            | "agent judge status"
            | "agent judge ready"
            | "agent judge configured"
            | "agent judge health"
            | "agent judge mode"
            | "list judge"
            | "list the judge"
            | "list agent judge"
            | "list the agent judge"
            | "show judge"
            | "show the judge"
            | "show me judge"
            | "show me the judge"
            | "show agent judge"
            | "show the agent judge"
            | "show me agent judge"
            | "show me the agent judge"
            | "view judge"
            | "view the judge"
            | "view agent judge"
            | "view the agent judge"
            | "see judge"
            | "see the judge"
            | "see agent judge"
            | "see the agent judge"
            | "open judge"
            | "open the judge"
            | "open agent judge"
            | "open the agent judge"
            | "the judge"
            | "the agent judge"
            | "is judge ready"
            | "is judge online"
            | "is judge offline"
            | "is judge configured"
            | "is judge set up"
            | "is judge setup"
            | "is judge enabled"
            | "is agent judge ready"
            | "is agent judge configured"
            | "is agent judge enabled"
            | "how's judge"
            | "hows judge"
            | "how's the judge"
            | "hows the judge"
            | "how's agent judge"
            | "hows agent judge"
            | "judge connection"
            | "agent judge connection"
            | "failure only judge"
            | "failure-only judge"
            | "judge on failure"
            | "judge on failure only"
    )
}


/// Zero-LLM agent-judge Ready / Off chip (config only; does not run the judge).
pub fn format_judge_ready_chip() -> String {
    if !crate::config::Config::agent_judge_enabled() {
        return "**Judge** · Off · enable in Settings Product (or `agentJudgeEnabled` true)"
            .to_string();
    }
    if crate::config::Config::agent_judge_on_failure_only() {
        "**Judge** · Ready · failure-only (default) · Settings Product".to_string()
    } else {
        "**Judge** · Ready · every run · Settings Product".to_string()
    }
}


/// True for focused product AI On/Off asks (`/ai` · `/ai-agent`) —
/// not enable/disable how-tos, `/agents` catalog, or chat-with-AI tasks.
/// View/see/show me/open/list-the NL (v0.1.1029). Exact `open ai` / `open the ai` /
/// `open ai agent` / `open the ai agent` / `open ai-agent` only — not OpenAI / ask / chat /
/// `/agents` catalog.
pub fn looks_like_ai_agent_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Actions / how-tos / agent catalog stay with pre-route / agent / `/agents`.
    if n.contains("enable ai")
        || n.contains("disable ai")
        || n.contains("enable the ai")
        || n.contains("disable the ai")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("switch on")
        || n.contains("switch off")
        || n.starts_with("/agents")
        || n == "agents"
        || n.contains("list agents")
        || n.contains("openai")
        || n.contains("open ai api")
        || n.contains("chat with")
        || n.contains("talk to")
        || n.contains("ask ai")
        || n.contains("ask the ai")
        || n.contains("message ")
        || n.contains("invoke")
        || n.contains("create")
        || n.contains("update")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ai"
            | "/ai-agent"
            | "ai"
            | "ai status"
            | "ai ready"
            | "ai offline"
            | "ai configured"
            | "ai health"
            | "ai mode"
            | "ai on"
            | "ai off"
            | "ai agent"
            | "ai agent status"
            | "ai agent ready"
            | "ai agent offline"
            | "ai agent configured"
            | "ai agent health"
            | "ai agent mode"
            | "ai agent on"
            | "ai agent off"
            | "local ai"
            | "local ai status"
            | "local ai agent"
            | "local ai ready"
            | "list ai"
            | "list the ai"
            | "list ai agent"
            | "list the ai agent"
            | "list ai-agent"
            | "show ai"
            | "show the ai"
            | "show me ai"
            | "show me the ai"
            | "show ai agent"
            | "show the ai agent"
            | "show me ai agent"
            | "show me the ai agent"
            | "show ai-agent"
            | "view ai"
            | "view the ai"
            | "view ai agent"
            | "view the ai agent"
            | "view ai-agent"
            | "see ai"
            | "see the ai"
            | "see ai agent"
            | "see the ai agent"
            | "see ai-agent"
            | "open ai"
            | "open the ai"
            | "open ai agent"
            | "open the ai agent"
            | "open ai-agent"
            | "the ai"
            | "the ai agent"
            | "is ai ready"
            | "is ai online"
            | "is ai offline"
            | "is ai configured"
            | "is ai set up"
            | "is ai setup"
            | "is ai enabled"
            | "is ai on"
            | "is ai off"
            | "is the ai on"
            | "is the ai off"
            | "is the ai ready"
            | "is the ai enabled"
            | "is ai agent ready"
            | "is ai agent configured"
            | "is ai agent enabled"
            | "is ai agent on"
            | "is ai agent off"
            | "how's ai"
            | "hows ai"
            | "how's the ai"
            | "hows the ai"
            | "how's ai agent"
            | "hows ai agent"
            | "ai connection"
            | "ai agent connection"
    )
}


/// Zero-LLM product AI On / Off chip (`aiAgentEnabled`; config only; does not toggle).
pub fn format_ai_agent_ready_chip() -> String {
    if crate::config::Config::ai_agent_enabled() {
        "**AI** · On · Discord · schedules · Ollama tools · Settings Product".to_string()
    } else {
        "**AI** · Off · monitor-only · enable in Settings Product (or `aiAgentEnabled` true)"
            .to_string()
    }
}


/// True for focused Settings Product compact asks (`/compact` · `/menu-bar` · `/cpu-window`) —
/// not session compaction, enable/disable how-tos, or layout redesign tasks.
/// View/see/show me/open/list-the NL (v0.1.1030). Exact `open compact` / `open menu-bar` /
/// `open cpu-window` (and close variants) only — not compaction / enable / open-page tasks.
pub fn looks_like_compact_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Compaction / memory / actions stay with pre-route / agent.
    if n.contains("compaction")
        || n.contains("compact memory")
        || n.contains("compact session")
        || n.contains("compact context")
        || n.contains("context compact")
        || n.contains("compact this")
        || n.contains("compact the")
        || n.contains("make compact")
        || n.contains("enable compact")
        || n.contains("disable compact")
        || n.contains("run compact")
        || n.contains("run compaction")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("switch on")
        || n.contains("switch off")
        || n.contains("invoke")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/compact"
            | "/compact menu"
            | "/compact window"
            | "/menu-bar"
            | "/menubar"
            | "/cpu-window"
            | "compact"
            | "compact status"
            | "compact mode"
            | "compact settings"
            | "compact ready"
            | "compact health"
            | "compact on"
            | "compact off"
            | "menu bar status"
            | "menu bar ready"
            | "menu bar compact"
            | "menubar status"
            | "menubar compact"
            | "cpu window status"
            | "cpu window ready"
            | "cpu window compact"
            | "cpu-window status"
            | "cpu-window compact"
            | "list compact"
            | "list the compact"
            | "list menu bar"
            | "list the menu bar"
            | "list menu-bar"
            | "list menubar"
            | "list cpu window"
            | "list the cpu window"
            | "list cpu-window"
            | "view compact"
            | "view the compact"
            | "view menu bar"
            | "view the menu bar"
            | "view menu-bar"
            | "view menubar"
            | "view cpu window"
            | "view the cpu window"
            | "view cpu-window"
            | "see compact"
            | "see the compact"
            | "see menu bar"
            | "see the menu bar"
            | "see menu-bar"
            | "see menubar"
            | "see cpu window"
            | "see the cpu window"
            | "see cpu-window"
            | "show compact"
            | "show the compact"
            | "show me compact"
            | "show me the compact"
            | "show menu bar"
            | "show the menu bar"
            | "show me menu bar"
            | "show me the menu bar"
            | "show menu-bar"
            | "show menubar"
            | "show me menu-bar"
            | "show me menubar"
            | "show cpu window"
            | "show the cpu window"
            | "show me cpu window"
            | "show me the cpu window"
            | "show cpu-window"
            | "show me cpu-window"
            | "show menu bar compact"
            | "show cpu window compact"
            | "open compact"
            | "open the compact"
            | "open menu bar"
            | "open the menu bar"
            | "open menu-bar"
            | "open menubar"
            | "open cpu window"
            | "open the cpu window"
            | "open cpu-window"
            | "open the cpu-window"
            | "the compact"
            | "the menu bar"
            | "the cpu window"
            | "is compact on"
            | "is compact off"
            | "is menu bar compact"
            | "is menubar compact"
            | "is cpu window compact"
            | "is cpu-window compact"
            | "how's compact"
            | "hows compact"
            | "how's the compact"
            | "hows the compact"
            | "how's menu bar"
            | "hows menu bar"
            | "how's the menu bar"
            | "hows the menu bar"
            | "how's menu bar compact"
            | "hows menu bar compact"
            | "how's cpu window"
            | "hows cpu window"
            | "how's the cpu window"
            | "hows the cpu window"
            | "how's cpu window compact"
            | "hows cpu window compact"
            | "compact menu bar"
            | "compact cpu window"
            | "compact cpu-window"
    )
}


/// Zero-LLM Settings Product compact chip (`menuBarCompact` · `cpuWindowCompact`; config only).
pub fn format_compact_ready_chip() -> String {
    let menu_on = crate::config::Config::menu_bar_compact();
    let window_on = crate::config::Config::cpu_window_compact();
    let menu = if menu_on {
        "Menu bar On"
    } else {
        "Menu bar Off"
    };
    let window = if window_on {
        "CPU window On"
    } else {
        "CPU window Off"
    };
    // When either compact toggle is On, point at the Product glance (expand for full UI).
    if menu_on || window_on {
        format!(
            "**Compact** · {menu} · {window} · expand in Settings Product (or turn toggles Off)"
        )
    } else {
        format!("**Compact** · {menu} · {window} · Settings Product (config only)")
    }
}


/// True for focused Downloads organizer Ready/config asks (`/downloads` · `/organizer`) —
/// not Disk Cleanup `/disk`, BROWSER_DOWNLOAD, run-now, or enable/disable how-tos.
/// View/see/show me/open/list-the NL (v0.1.1031). Exact `open downloads` / `open organizer`
/// (and close variants) only — not `open download …`, run-now, or path/size/age lanes.
pub fn looks_like_downloads_organizer_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Actions / other surfaces stay with pre-route / agent / `/disk`.
    // Path-only rules/state asks stay on the downloads-organizer-rules (and future state) lanes.
    if n.contains("browser_download")
        || n.contains("browser download")
        || n.contains("download file")
        || n.contains("download this")
        || n.contains("download the")
        || n.contains("download from")
        || n.contains("download url")
        || n.contains("download http")
        || n.contains("run organizer")
        || n.contains("run downloads")
        || n.contains("organize now")
        || n.contains("organize my")
        || n.contains("clean now")
        || n.contains("/disk")
        || n.contains("disk cleanup")
        || n.contains("enable download")
        || n.contains("disable download")
        || n.contains("enable organizer")
        || n.contains("disable organizer")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("switch on")
        || n.contains("switch off")
        || n.contains("invoke")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.contains("rules")
        || n.contains("rules.md")
        || n.contains("organizer-rules")
        || n.contains("organizer rules")
        || n.contains("downloads-organizer-rules")
        || n.contains("downloads organizer rules")
        || n.contains("state.json")
        || n.contains("organizer-state")
        || n.contains("organizer state")
        || n.contains("downloads-organizer-state")
        || n.contains("path")
        || n.contains("where is")
        || n.contains("where are")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/downloads"
            | "/organizer"
            | "/downloads-organizer"
            | "downloads"
            | "downloads organizer"
            | "downloads status"
            | "downloads ready"
            | "downloads health"
            | "downloads on"
            | "downloads off"
            | "organizer"
            | "organizer status"
            | "organizer ready"
            | "organizer health"
            | "organizer on"
            | "organizer off"
            | "list downloads"
            | "list the downloads"
            | "list organizer"
            | "list the organizer"
            | "list downloads organizer"
            | "list the downloads organizer"
            | "view downloads"
            | "view the downloads"
            | "view organizer"
            | "view the organizer"
            | "view downloads organizer"
            | "view the downloads organizer"
            | "see downloads"
            | "see the downloads"
            | "see organizer"
            | "see the organizer"
            | "see downloads organizer"
            | "see the downloads organizer"
            | "show downloads"
            | "show the downloads"
            | "show me downloads"
            | "show me the downloads"
            | "show organizer"
            | "show the organizer"
            | "show me organizer"
            | "show me the organizer"
            | "show downloads organizer"
            | "show the downloads organizer"
            | "show me downloads organizer"
            | "show me the downloads organizer"
            | "open downloads"
            | "open the downloads"
            | "open organizer"
            | "open the organizer"
            | "open downloads organizer"
            | "open the downloads organizer"
            | "the downloads"
            | "the organizer"
            | "is downloads ready"
            | "is downloads on"
            | "is downloads off"
            | "is downloads enabled"
            | "is organizer ready"
            | "is organizer on"
            | "is organizer off"
            | "is organizer enabled"
            | "how's downloads"
            | "hows downloads"
            | "how's the downloads"
            | "hows the downloads"
            | "how's organizer"
            | "hows organizer"
            | "how's the organizer"
            | "hows the organizer"
            | "downloads organizer status"
            | "downloads organizer ready"
    )
}


/// Zero-LLM Downloads organizer Ready chip (config + last-run summary; no run-now).
pub fn format_downloads_organizer_ready_chip() -> String {
    let st = crate::commands::downloads_organizer::get_downloads_organizer_status();
    if !st.enabled {
        return "**Downloads** · Off · enable in Settings Product (or `downloadsOrganizerEnabled` true)"
            .to_string();
    }
    let interval = st.interval.trim();
    let dry = if st.dry_run { "dry-run" } else { "live" };
    let path = {
        let raw = st.path_raw.trim();
        if raw.is_empty() {
            "~/Downloads".to_string()
        } else if let Ok(home) = std::env::var("HOME") {
            if raw.starts_with(&home) {
                format!("~{}", &raw[home.len()..])
            } else {
                raw.to_string()
            }
        } else {
            raw.to_string()
        }
    };
    let last = match st.last_run_utc.as_deref() {
        Some(ts) if !ts.is_empty() => {
            let short = ts.get(..19).unwrap_or(ts);
            format!(
                "last {short}Z · moved {} · skip {} · fail {}",
                st.moved, st.skipped, st.failed
            )
        }
        _ => "last · never".to_string(),
    };
    format!(
        "**Downloads** · On · {interval} · {dry} · {path} · {last} · Settings Product"
    )
}


/// True for focused Ori Mnemos lifecycle Ready/config asks (`/ori` · `/mnemos`) —
/// not MCP `ori_*` tools, markdown MEMORY_APPEND, scrub memory, or enable/disable how-tos.
pub fn looks_like_ori_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Tool / memory actions and how-tos stay with pre-route / agent / scrub / MCP.
    // Vault path-only asks go to looks_like_ori_vault_path_request (not Ready).
    if n.starts_with("mcp:")
        || n.contains("mcp:")
        || n.contains("ori_")
        || n.contains("memory_append")
        || n.contains("memory append")
        || n.contains("scrub memory")
        || n.contains("save memory")
        || n.contains("append memory")
        || n.contains("write memory")
        || n.contains("ori query")
        || n.contains("ori orient")
        || n.contains("ori add")
        || n.contains("ori promote")
        || n.contains("ori serve")
        || n.contains("run ori")
        || n.contains("call ori")
        || n.contains("use ori")
        || n.contains("enable ori")
        || n.contains("disable ori")
        || n.contains("enable mnemos")
        || n.contains("disable mnemos")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("switch on")
        || n.contains("switch off")
        || n.contains("invoke")
        || n.contains("create")
        || n.contains("update")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || ((n.contains("vault") || n.contains("ori-vault") || n.contains("orivault"))
            && (n.contains("path")
                || n.contains("where")
                || n.contains("location")
                || n.contains("folder")
                || n.contains("directory")
                || n.contains("dir")))
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ori"
            | "/mnemos"
            | "/ori-mnemos"
            | "ori"
            | "mnemos"
            | "ori mnemos"
            | "ori-mnemos"
            | "ori status"
            | "ori ready"
            | "ori health"
            | "ori on"
            | "ori off"
            | "ori vault"
            | "ori lifecycle"
            | "mnemos status"
            | "mnemos ready"
            | "mnemos health"
            | "mnemos on"
            | "mnemos off"
            | "mnemos vault"
            | "list ori"
            | "list the ori"
            | "list mnemos"
            | "list the mnemos"
            | "list ori mnemos"
            | "list the ori mnemos"
            | "list ori-mnemos"
            | "list the ori-mnemos"
            | "view ori"
            | "view the ori"
            | "view mnemos"
            | "view the mnemos"
            | "view ori mnemos"
            | "view the ori mnemos"
            | "view ori-mnemos"
            | "view the ori-mnemos"
            | "see ori"
            | "see the ori"
            | "see mnemos"
            | "see the mnemos"
            | "see ori mnemos"
            | "see the ori mnemos"
            | "see ori-mnemos"
            | "see the ori-mnemos"
            | "show ori"
            | "show the ori"
            | "show me ori"
            | "show me the ori"
            | "show mnemos"
            | "show the mnemos"
            | "show me mnemos"
            | "show me the mnemos"
            | "show ori mnemos"
            | "show the ori mnemos"
            | "show me ori mnemos"
            | "show me the ori mnemos"
            | "show ori-mnemos"
            | "show the ori-mnemos"
            | "show me ori-mnemos"
            | "show me the ori-mnemos"
            | "open ori"
            | "open the ori"
            | "open mnemos"
            | "open the mnemos"
            | "open ori mnemos"
            | "open the ori mnemos"
            | "open ori-mnemos"
            | "open the ori-mnemos"
            | "the ori"
            | "the mnemos"
            | "is ori ready"
            | "is ori on"
            | "is ori off"
            | "is ori enabled"
            | "is ori configured"
            | "is ori set up"
            | "is ori setup"
            | "is mnemos ready"
            | "is mnemos on"
            | "is mnemos off"
            | "is mnemos enabled"
            | "is mnemos configured"
            | "is mnemos set up"
            | "is mnemos setup"
            | "how's ori"
            | "hows ori"
            | "how's the ori"
            | "hows the ori"
            | "how's mnemos"
            | "hows mnemos"
            | "how's the mnemos"
            | "hows the mnemos"
            | "ori mnemos status"
            | "ori mnemos ready"
            | "ori lifecycle status"
            | "ori lifecycle ready"
    )
}


/// True for focused Having fun / idle-thought Ready/config asks (`/having_fun` · `/fun` · `/idle`) —
/// not send/post idle thoughts, enable/disable how-tos, or free-form “have fun …”.
pub fn looks_like_having_fun_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("send ")
        || n.contains("post ")
        || n.starts_with("post ")
        || n.contains("message ")
        || n.contains("reply to")
        || n.contains("have fun")
        || n.contains("having fun with")
        || n.contains("enable having")
        || n.contains("disable having")
        || n.contains("enable idle")
        || n.contains("disable idle")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("switch on")
        || n.contains("switch off")
        || n.contains("create")
        || n.contains("update")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/having_fun"
            | "/having-fun"
            | "/fun"
            | "/idle"
            | "/idle-thought"
            | "/idle-thoughts"
            | "having fun"
            | "having-fun"
            | "having_fun"
            | "having fun status"
            | "having fun ready"
            | "having fun health"
            | "having fun on"
            | "having fun off"
            | "idle thoughts"
            | "idle thought"
            | "idle thoughts status"
            | "idle thought status"
            | "idle status"
            | "idle ready"
            | "idle health"
            | "idle on"
            | "idle off"
            | "fun status"
            | "fun ready"
            | "fun health"
            | "fun on"
            | "fun off"
            | "show having fun"
            | "show the having fun"
            | "show me having fun"
            | "show me the having fun"
            | "show idle"
            | "show the idle"
            | "show me idle"
            | "show me the idle"
            | "show idle thoughts"
            | "show the idle thoughts"
            | "show me idle thoughts"
            | "show me the idle thoughts"
            | "show fun"
            | "show the fun"
            | "show me fun"
            | "show me the fun"
            | "view having fun"
            | "view the having fun"
            | "view idle"
            | "view the idle"
            | "view idle thoughts"
            | "view the idle thoughts"
            | "view fun"
            | "view the fun"
            | "see having fun"
            | "see the having fun"
            | "see idle"
            | "see the idle"
            | "see idle thoughts"
            | "see the idle thoughts"
            | "see fun"
            | "see the fun"
            | "open having fun"
            | "open the having fun"
            | "open idle"
            | "open the idle"
            | "open idle thoughts"
            | "open the idle thoughts"
            | "open fun"
            | "open the fun"
            | "list having fun"
            | "list the having fun"
            | "list idle"
            | "list the idle"
            | "list idle thoughts"
            | "list the idle thoughts"
            | "list fun"
            | "list the fun"
            | "the having fun"
            | "the fun"
            | "the idle"
            | "the idle thoughts"
            | "is having fun ready"
            | "is having fun on"
            | "is having fun off"
            | "is having fun enabled"
            | "is having fun configured"
            | "is having fun set up"
            | "is having fun setup"
            | "is idle ready"
            | "is idle on"
            | "is idle off"
            | "is idle enabled"
            | "is idle configured"
            | "how's having fun"
            | "hows having fun"
            | "how's the having fun"
            | "hows the having fun"
            | "how's idle"
            | "hows idle"
            | "how's the idle"
            | "hows the idle"
            | "how's idle thoughts"
            | "hows idle thoughts"
            | "how's fun"
            | "hows fun"
    )
}


/// Zero-LLM Having fun / idle-thought Ready chip (`discord_channels.json` only; no send).
pub fn format_having_fun_ready_chip() -> String {
    crate::discord::format_having_fun_ready_chip()
}


/// True for focused Discord voice STT Ready/config asks (`/voice` · `/stt`) —
/// not live transcription, send-voice, or how-to enable.
pub fn looks_like_voice_stt_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("transcribe")
        || n.contains("transcript")
        || n.contains("convert")
        || n.contains("send voice")
        || n.contains("post voice")
        || n.contains("voice note")
        || n.contains("voice message")
        || n.contains("voice memo")
        || n.contains("record ")
        || n.contains("listen")
        || n.contains("play ")
        || n.contains("enable voice")
        || n.contains("disable voice")
        || n.contains("enable stt")
        || n.contains("disable stt")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("switch on")
        || n.contains("switch off")
        || n.contains("create")
        || n.contains("update")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("message ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/voice"
            | "/stt"
            | "/speech"
            | "/transcription"
            | "voice"
            | "stt"
            | "speech"
            | "voice status"
            | "voice ready"
            | "voice health"
            | "voice on"
            | "voice off"
            | "stt status"
            | "stt ready"
            | "stt health"
            | "stt on"
            | "stt off"
            | "speech status"
            | "speech ready"
            | "speech health"
            | "speech to text"
            | "speech-to-text"
            | "voice stt"
            | "voice transcription"
            | "discord voice"
            | "discord stt"
            | "show voice"
            | "show the voice"
            | "show me voice"
            | "show me the voice"
            | "show stt"
            | "show the stt"
            | "show me stt"
            | "show me the stt"
            | "show speech"
            | "show the speech"
            | "show me speech"
            | "show me the speech"
            | "show speech to text"
            | "show the speech to text"
            | "show me speech to text"
            | "show me the speech to text"
            | "show speech-to-text"
            | "show the speech-to-text"
            | "show me speech-to-text"
            | "show me the speech-to-text"
            | "show voice stt"
            | "show the voice stt"
            | "show me voice stt"
            | "show me the voice stt"
            | "show discord voice"
            | "show the discord voice"
            | "show me discord voice"
            | "show me the discord voice"
            | "show discord stt"
            | "show the discord stt"
            | "show me discord stt"
            | "show me the discord stt"
            | "view voice"
            | "view the voice"
            | "view stt"
            | "view the stt"
            | "view speech"
            | "view the speech"
            | "view speech to text"
            | "view the speech to text"
            | "view speech-to-text"
            | "view the speech-to-text"
            | "view voice stt"
            | "view the voice stt"
            | "view discord voice"
            | "view the discord voice"
            | "view discord stt"
            | "view the discord stt"
            | "see voice"
            | "see the voice"
            | "see stt"
            | "see the stt"
            | "see speech"
            | "see the speech"
            | "see speech to text"
            | "see the speech to text"
            | "see speech-to-text"
            | "see the speech-to-text"
            | "see voice stt"
            | "see the voice stt"
            | "see discord voice"
            | "see the discord voice"
            | "see discord stt"
            | "see the discord stt"
            | "open voice"
            | "open the voice"
            | "open stt"
            | "open the stt"
            | "open speech"
            | "open the speech"
            | "open speech to text"
            | "open the speech to text"
            | "open speech-to-text"
            | "open the speech-to-text"
            | "open voice stt"
            | "open the voice stt"
            | "open discord voice"
            | "open the discord voice"
            | "open discord stt"
            | "open the discord stt"
            | "list voice"
            | "list the voice"
            | "list stt"
            | "list the stt"
            | "list speech"
            | "list the speech"
            | "list speech to text"
            | "list the speech to text"
            | "list speech-to-text"
            | "list the speech-to-text"
            | "list voice stt"
            | "list the voice stt"
            | "list discord voice"
            | "list the discord voice"
            | "list discord stt"
            | "list the discord stt"
            | "the voice"
            | "the stt"
            | "the speech"
            | "the speech to text"
            | "the speech-to-text"
            | "the voice stt"
            | "the discord voice"
            | "the discord stt"
            | "is voice ready"
            | "is voice on"
            | "is voice off"
            | "is voice enabled"
            | "is voice configured"
            | "is voice set up"
            | "is voice setup"
            | "is stt ready"
            | "is stt on"
            | "is stt off"
            | "is stt enabled"
            | "is stt configured"
            | "is speech ready"
            | "how's voice"
            | "hows voice"
            | "how's the voice"
            | "hows the voice"
            | "how's stt"
            | "hows stt"
            | "how's the stt"
            | "hows the stt"
            | "how's speech"
            | "hows speech"
    )
}


/// Zero-LLM Discord voice STT Ready chip (model + ffmpeg + Ollama config; no transcribe).
pub fn format_voice_stt_ready_chip() -> String {
    crate::discord::format_voice_stt_ready_chip()
}


/// Zero-LLM Ori Mnemos lifecycle Ready chip (env/config only; no `ori` subprocess / MCP probe).
pub fn format_ori_ready_chip() -> String {
    use crate::config::Config;
    if !Config::ori_lifecycle_enabled() {
        return "**Ori** · Off · enable in Settings Product (or `oriLifecycleEnabled` / `MAC_STATS_ORI_LIFECYCLE_ENABLED`)"
            .to_string();
    }
    let raw = Config::ori_vault_path_raw();
    let (vault_part, vault_ready) = if raw.trim().is_empty() {
        ("vault not set".to_string(), false)
    } else {
        match Config::expand_user_path_str(raw.trim()) {
            Some(p) if p.join(".ori").is_file() => (
                format!("vault {}", shorten_ori_vault_for_chip(&p)),
                true,
            ),
            Some(p) => (
                format!(
                    "vault {} · missing .ori",
                    shorten_ori_vault_for_chip(&p)
                ),
                false,
            ),
            None => ("vault path invalid".to_string(), false),
        }
    };
    let state = if vault_ready { "Ready" } else { "Partial" };
    let orient = if Config::ori_hook_orient_on_session_start() {
        "orient On"
    } else {
        "orient Off"
    };
    let prefetch = if Config::ori_prefetch_enabled() {
        "prefetch On"
    } else {
        "prefetch Off"
    };
    let capture = if Config::ori_hook_capture_on_compaction() {
        "capture On"
    } else {
        "capture Off"
    };
    let bin = Config::ori_binary();
    let bin_short = if bin.chars().count() > 24 {
        let mut s: String = bin.chars().take(21).collect();
        s.push('…');
        s
    } else {
        bin
    };
    format!(
        "**Ori** · {state} · {vault_part} · {orient} · {prefetch} · {capture} · `{bin_short}` · Settings Product"
    )
}


/// True for focused Telegram alert Ready/config asks (`/telegram`) — not send/post.
pub fn looks_like_telegram_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 || alert_ready_reject_noise(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/telegram"
            | "telegram"
            | "telegram status"
            | "telegram ready"
            | "telegram offline"
            | "telegram configured"
            | "telegram health"
            | "telegram key"
            | "telegram token"
            | "telegram bot"
            | "telegram alert"
            | "telegram alerts"
            | "show telegram"
            | "show the telegram"
            | "show me telegram"
            | "show me the telegram"
            | "show telegram bot"
            | "show the telegram bot"
            | "show me telegram bot"
            | "show me the telegram bot"
            | "show telegram alerts"
            | "show the telegram alerts"
            | "show me telegram alerts"
            | "show me the telegram alerts"
            | "view telegram"
            | "view the telegram"
            | "view telegram bot"
            | "view the telegram bot"
            | "view telegram alerts"
            | "view the telegram alerts"
            | "see telegram"
            | "see the telegram"
            | "see telegram bot"
            | "see the telegram bot"
            | "see telegram alerts"
            | "see the telegram alerts"
            | "open telegram"
            | "open the telegram"
            | "open telegram bot"
            | "open the telegram bot"
            | "open telegram alerts"
            | "open the telegram alerts"
            | "list telegram"
            | "list the telegram"
            | "list telegram bot"
            | "list the telegram bot"
            | "list telegram alerts"
            | "list the telegram alerts"
            | "the telegram"
            | "the telegram bot"
            | "the telegram alerts"
            | "is telegram ready"
            | "is telegram online"
            | "is telegram connected"
            | "is telegram offline"
            | "is telegram configured"
            | "is telegram set up"
            | "is telegram setup"
            | "telegram connection"
            | "how's telegram"
            | "hows telegram"
            | "how's the telegram"
            | "hows the telegram"
            | "telegram bot status"
            | "telegram bot ready"
            | "is telegram bot ready"
            | "is telegram bot configured"
            | "how's telegram bot"
            | "hows telegram bot"
    )
}


/// Zero-LLM Telegram alert Ready / Not set / Partial chip (config only; no live send).
pub fn format_telegram_ready_chip() -> String {
    let registered =
        crate::commands::alerts::count_registered_alert_channels("Telegram");
    let tokens = crate::commands::alerts::count_alert_keychain_prefix("telegram_bot_");
    let chat = crate::commands::alerts::get_telegram_chat_id().is_some();
    match (registered > 0, tokens > 0, chat) {
        (false, false, false) => {
            "**Telegram** · Not set · add bot token + chat id (Settings)".to_string()
        }
        (true, false, _) => {
            format!(
                "**Telegram** · Partial · {registered} channel(s) · missing bot token (Settings)"
            )
        }
        (false, true, false) => {
            format!("**Telegram** · Partial · {tokens} token(s) · missing chat id (Settings)")
        }
        (false, true, true) => {
            "**Telegram** · Partial · token + chat set · Save again to register (Settings)"
                .to_string()
        }
        (false, false, true) => {
            "**Telegram** · Partial · chat id set · missing bot token (Settings)".to_string()
        }
        (true, true, _) => {
            format!("**Telegram** · Ready · {registered} channel(s) · token set")
        }
    }
}


/// True for focused Slack alert Ready/config asks (`/slack`) — not post/notify.
pub fn looks_like_slack_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 || alert_ready_reject_noise(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/slack"
            | "slack"
            | "slack status"
            | "slack ready"
            | "slack offline"
            | "slack configured"
            | "slack health"
            | "slack key"
            | "slack token"
            | "slack webhook"
            | "slack alert"
            | "slack alerts"
            | "show slack"
            | "show the slack"
            | "show me slack"
            | "show me the slack"
            | "show slack webhook"
            | "show the slack webhook"
            | "show me slack webhook"
            | "show me the slack webhook"
            | "show slack alerts"
            | "show the slack alerts"
            | "show me slack alerts"
            | "show me the slack alerts"
            | "view slack"
            | "view the slack"
            | "view slack webhook"
            | "view the slack webhook"
            | "view slack alerts"
            | "view the slack alerts"
            | "see slack"
            | "see the slack"
            | "see slack webhook"
            | "see the slack webhook"
            | "see slack alerts"
            | "see the slack alerts"
            | "open slack"
            | "open the slack"
            | "open slack webhook"
            | "open the slack webhook"
            | "open slack alerts"
            | "open the slack alerts"
            | "list slack"
            | "list the slack"
            | "list slack webhook"
            | "list the slack webhook"
            | "list slack alerts"
            | "list the slack alerts"
            | "the slack"
            | "the slack webhook"
            | "the slack alerts"
            | "is slack ready"
            | "is slack online"
            | "is slack connected"
            | "is slack offline"
            | "is slack configured"
            | "is slack set up"
            | "is slack setup"
            | "slack connection"
            | "how's slack"
            | "hows slack"
            | "how's the slack"
            | "hows the slack"
            | "slack webhook status"
            | "slack webhook ready"
            | "is slack webhook ready"
            | "is slack webhook configured"
            | "how's slack webhook"
            | "hows slack webhook"
    )
}


/// Zero-LLM Slack alert Ready / Not set / Partial chip (config only; no live send).
pub fn format_slack_ready_chip() -> String {
    let registered = crate::commands::alerts::count_registered_alert_channels("Slack");
    let hooks = crate::commands::alerts::count_alert_keychain_prefix("slack_webhook_");
    let settings = crate::commands::alerts::get_slack_webhook().is_some();
    match (registered > 0, hooks > 0, settings) {
        (false, false, _) => {
            "**Slack** · Not set · add webhook URL (Settings)".to_string()
        }
        (true, false, _) => {
            format!(
                "**Slack** · Partial · {registered} channel(s) · missing webhook (Settings)"
            )
        }
        (false, true, true) => {
            "**Slack** · Partial · webhook set · Save again to register (Settings)".to_string()
        }
        (false, true, false) => {
            format!("**Slack** · Partial · {hooks} webhook(s) · register channel (Settings)")
        }
        (true, true, _) => {
            format!("**Slack** · Ready · {registered} channel(s) · webhook set")
        }
    }
}


/// True for focused Signal alert Ready/config asks (`/signal`) — not send.
pub fn looks_like_signal_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 || alert_ready_reject_noise(&n) {
        return false;
    }
    // OS signal / SIGTERM confusion stays with agent.
    if n.contains("sigterm")
        || n.contains("sigint")
        || n.contains("sighup")
        || n.contains("kill ")
        || n.contains("process signal")
        || n == "signal handler"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/signal"
            | "signal"
            | "signal status"
            | "signal ready"
            | "signal offline"
            | "signal configured"
            | "signal health"
            | "signal key"
            | "signal token"
            | "signal alert"
            | "signal alerts"
            | "show signal"
            | "show the signal"
            | "show me signal"
            | "show me the signal"
            | "show signal app"
            | "show the signal app"
            | "show me signal app"
            | "show me the signal app"
            | "show signal alerts"
            | "show the signal alerts"
            | "show me signal alerts"
            | "show me the signal alerts"
            | "view signal"
            | "view the signal"
            | "view signal app"
            | "view the signal app"
            | "view signal alerts"
            | "view the signal alerts"
            | "see signal"
            | "see the signal"
            | "see signal app"
            | "see the signal app"
            | "see signal alerts"
            | "see the signal alerts"
            | "open signal"
            | "open the signal"
            | "open signal app"
            | "open the signal app"
            | "open signal alerts"
            | "open the signal alerts"
            | "list signal"
            | "list the signal"
            | "list signal app"
            | "list the signal app"
            | "list signal alerts"
            | "list the signal alerts"
            | "the signal"
            | "the signal app"
            | "the signal alerts"
            | "is signal ready"
            | "is signal online"
            | "is signal connected"
            | "is signal offline"
            | "is signal configured"
            | "is signal set up"
            | "is signal setup"
            | "signal connection"
            | "how's signal"
            | "hows signal"
            | "how's the signal"
            | "hows the signal"
            | "signal app"
            | "signal app status"
            | "is signal app ready"
            | "how's signal app"
            | "hows signal app"
    )
}


/// Zero-LLM Signal alert chip (placeholder channel; config only; no live send).
pub fn format_signal_ready_chip() -> String {
    let registered = crate::commands::alerts::count_registered_alert_channels("Signal");
    if registered > 0 {
        format!(
            "**Signal** · Partial · {registered} channel(s) · REST API not wired yet (Settings)"
        )
    } else {
        "**Signal** · Not wired · REST API not wired yet (Settings)".to_string()
    }
}


/// True for focused alert-channels summary asks (`/alerts`) — not fire/create rules.
pub fn looks_like_alerts_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 || alert_ready_reject_noise(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/alerts"
            | "alerts"
            | "alert channels"
            | "alert channel"
            | "alerts status"
            | "alert status"
            | "alerts ready"
            | "alerts configured"
            | "alerts health"
            | "show alerts"
            | "show the alerts"
            | "show me alerts"
            | "show me the alerts"
            | "show alert channels"
            | "show the alert channels"
            | "show me alert channels"
            | "show me the alert channels"
            | "view alerts"
            | "view the alerts"
            | "view alert channels"
            | "view the alert channels"
            | "see alerts"
            | "see the alerts"
            | "see alert channels"
            | "see the alert channels"
            | "open alerts"
            | "open the alerts"
            | "open alert channels"
            | "open the alert channels"
            | "list alerts"
            | "list the alerts"
            | "list alert channels"
            | "list the alert channels"
            | "the alerts"
            | "the alert channels"
            | "is alerts ready"
            | "is alerts configured"
            | "is alerts set up"
            | "is alerts setup"
            | "are alerts ready"
            | "are alerts configured"
            | "are alert channels ready"
            | "are alert channels configured"
            | "how's alerts"
            | "hows alerts"
            | "how's the alerts"
            | "hows the alerts"
            | "alert channels status"
            | "alert channels ready"
    )
}


/// Zero-LLM alert-channels summary (Telegram · Slack · Signal; config only).
pub fn format_alerts_ready_chip() -> String {
    let tg = format_telegram_ready_chip()
        .trim_start_matches("**Telegram** · ")
        .to_string();
    let sl = format_slack_ready_chip()
        .trim_start_matches("**Slack** · ")
        .to_string();
    let sg = format_signal_ready_chip()
        .trim_start_matches("**Signal** · ")
        .to_string();
    format!("**Alerts** · Telegram {tg} · Slack {sl} · Signal {sg}")
}


/// True for `/ops` / `/help` operator command list — not free-form “help me with …”.
/// View/see/show me/open/list-the NL (v0.1.1039). Exact `open help` / `open the help` only
/// (same for ops). After normalize strips `show me` / `show`, prefer `the help` / `the ops`
/// so `show me the help` still matches.
pub fn looks_like_ops_help_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Free-form help (“help me write…”) stays with the agent.
    if n.starts_with("help me")
        || n.starts_with("help with")
        || n.contains(" write ")
        || n.contains(" about ")
        || n.contains(" for ")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("ticket")
        || n.contains("redmine")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ops"
            | "ops"
            | "the ops"
            | "/ops help"
            | "ops help"
            | "view ops"
            | "view the ops"
            | "see ops"
            | "see the ops"
            | "show ops"
            | "show the ops"
            | "show me ops"
            | "show me the ops"
            | "open ops"
            | "open the ops"
            | "list ops"
            | "list the ops"
            | "/help"
            | "help"
            | "the help"
            | "operator help"
            | "operator commands"
            | "bot commands"
            | "/commands"
            | "commands"
            | "command list"
            | "list commands"
            | "command menu"
            | "what commands"
            | "what can you do"
            | "available commands"
            | "view help"
            | "view the help"
            | "see help"
            | "see the help"
            | "show help"
            | "show the help"
            | "show me help"
            | "show me the help"
            | "open help"
            | "open the help"
            | "list help"
            | "list the help"
            | "view commands"
            | "view the commands"
            | "see commands"
            | "see the commands"
            | "open commands"
            | "open the commands"
            | "list the commands"
    )
}


/// Short Discord menu of cheap operator commands (no Ollama).
pub fn format_ops_help_gateway() -> String {
    let version = crate::config::Config::version();
    format!(
        "**mac-stats v{version} — operator commands** (instant, no Ollama)\n\
• `/status` · `/health` · `/version` · `view status` · `see status` · `show me the status` · `open status` · `list the status` · `view health` · `open health` · `view version` · `open version` — one-screen health (exact open only; not ticket/status-of …)\n\
• `/discord` · `view discord` · `see discord` · `show me the discord` · `open discord` · `list the discord` — Discord Ready / Offline (Agent Ops glance; reconnect cues)\n\
• `/ollama` · `/llm` · `/model` · `which model` · `what model are you` · `ollama url` · `ollama endpoint` · `where is ollama` · `view ollama` · `see ollama` · `show me the ollama` · `open ollama` · `list the ollama` · `view llm` · `open llm` — Ollama Ready / Offline, including the configured model name and endpoint (menu-bar ✕ · AI Chat glance; circuit). Plural `which models` stays a model list, not this chip. `set url` stays a config change.\n\
• `/redmine` · `view redmine` · `see redmine` · `show me the redmine` · `open redmine` · `list the redmine` — Redmine Ready / Not set (Agent Ops health; URL + key; no live probe)\n\
• `/brave` · `view brave` · `see brave` · `show me the brave` · `open brave` · `list the brave` · `view brave search` · `open brave search` — Brave Search Ready / Not set (API key; no live probe)\n\
• `/perplexity key` · `view perplexity key` · `see perplexity key` · `show me the perplexity key` · `open perplexity key` · `list the perplexity key` — Perplexity Ready / Not set (API key; no live probe; does not steal `/perplexity` last-search)\n\
• `/mastodon` · `view mastodon` · `see mastodon` · `show me the mastodon` · `open mastodon` · `list the mastodon` — Mastodon Ready / Not set (instance URL + token; Settings or .config.env; no live probe)\n\
• `/mcp` · `view mcp` · `see mcp` · `show me the mcp` · `open mcp` · `list the mcp` · `view mcp server` · `open mcp server` — MCP Ready / Not set (MCP_SERVER_URL or MCP_SERVER_STDIO; Settings or .config.env; no live probe)\n\
• `/cursor` · `/cursor-agent` · `view cursor` · `see cursor` · `show me the cursor` · `open cursor` · `list the cursor` · `view cursor agent` · `open cursor-agent` — Cursor agent Ready / Not set (`cursor-agent` on PATH or Settings path; no CLI probe)\n\
• `/browser` · `/cdp` · `view browser` · `see browser` · `show me the browser` · `open browser` · `list the browser` · `view cdp` · `see cdp` · `show me the cdp` · `open cdp` · `list the cdp` — Browser / CDP Ready / Off / Not set (Chromium path + port; Settings or config.json; no live probe)\n\
• `/judge` · `view judge` · `see judge` · `show me the judge` · `open judge` · `list the judge` — Judge Ready / Off (Settings Product · agentJudgeEnabled · failure-only; config only, no judge run)\n\
• `/ai` · `/ai-agent` · `view ai` · `see ai` · `show me the ai` · `open ai` · `list the ai` · `view ai agent` · `open ai agent` — AI On / Off (Settings Product · aiAgentEnabled; config only, no toggle; does not steal `/agents`)\n\
• `/compact` · `/menu-bar` · `/cpu-window` · `view compact` · `see compact` · `show me the compact` · `open compact` · `list the compact` · `view menu bar` · `open menu-bar` · `view cpu window` · `open cpu-window` — Compact Menu bar / CPU window On/Off (menuBarCompact · cpuWindowCompact; config only; does not steal compaction)\n\
• `/downloads` · `/organizer` · `view downloads` · `see downloads` · `show me the downloads` · `open downloads` · `list the downloads` · `view organizer` · `see organizer` · `show me the organizer` · `open organizer` · `list the organizer` — Downloads organizer On/Off (Settings Product · interval · dry-run · path · last run; config only; does not steal `/disk` or BROWSER_DOWNLOAD)\n\
• `/ori` · `/mnemos` · `view ori` · `see ori` · `show me the ori` · `open ori` · `list the ori` · `view mnemos` · `see mnemos` · `show me the mnemos` · `open mnemos` · `list the mnemos` — Ori Mnemos lifecycle Ready / Off / Partial (Settings Product · ORI_VAULT · orient · prefetch · capture; config only; does not steal MCP `ori_*` / MEMORY_APPEND / scrub / vault path·size·age)\n\
• `ori vault size` · `how big is ori vault` · `mnemos vault size` — Ori vault folder size on disk (recursive file bytes; no list/MCP; does not steal `ori vault path` / `/ori` Ready)\n\
• `ori vault path` · `where is ori vault` · `ORI_VAULT path` — Ori vault root path (config/env only; `ori vault size` for on-disk bytes; no list/MCP; does not steal `/ori` Ready)\n\
• `/having_fun` · `/fun` · `/idle` · `view having fun` · `see having fun` · `show me the having fun` · `open having fun` · `list the having fun` · `view fun` · `open fun` · `view idle` · `see idle` · `show me the idle` · `open idle` · `list the idle` · `view idle thoughts` · `open idle thoughts` — Having fun / idle thoughts On/Off (Settings Product · channel count · idle · reply delays; config only; does not steal send/post)\n\
• `/voice` · `/stt` · `view voice` · `see voice` · `show me the voice` · `open voice` · `list the voice` · `view stt` · `see stt` · `show me the stt` · `open stt` · `list the stt` · `view speech` · `open speech to text` — Discord voice STT Ready / Off / Partial / Not set (Settings Product · model · ffmpeg · Ollama; config only; does not steal transcribe)\n\
• `/telegram` · `/slack` · `/signal` · `/alerts` · `view telegram` · `see telegram` · `show me the telegram` · `open telegram` · `list the telegram` · `view slack` · `see slack` · `show me the slack` · `open slack` · `list the slack` · `view signal` · `see signal` · `show me the signal` · `open signal` · `list the signal` · `view alerts` · `see alerts` · `show me the alerts` · `open alerts` · `list the alerts` — alert channel Ready / Not set (Keychain + registry; config only; no live send)\n\
• `/insights` · `/insights 7` · `view insights` · `see insights` · `show me the insights` · `open insights` · `list the insights` — runs.jsonl report (+ optional day window; exact open only — not insights on …)\n\
• `/failed` · `/failed 7` · `view failed` · `see failed` · `show me the failed` · `open failed` · `list the failed` — recent failed turns from runs.jsonl (exact open only — not why-did / ticket)\n\
• `/slow` · `/slow 7` · `view slow` · `see slow` · `show me the slow` · `open slow` · `list the slow` — recent slow turns (≥{slow_ms} ms wall time; exact open only — not why-is / monitor)\n\
• `/instant` · `/lite` · `/direct` · `/instant 7` · `view instant` · `see instant` · `open instant` · `list the instant` · `view lite` · `open lite` · `view direct` · `open direct` — recent instant-, lite-, or direct-lane turns (exact open only — not make-it)\n\
• `/agents` · `/agents on` · `/agents off` · `view agents` · `see agents` · `show me the agents` · `open agents` · `list the agents` — Agent Ops On/Off list\n\
• `/skills` · `view skills` · `see skills` · `show me the skills` · `open skills` · `list the skills` — installed skills catalog (Hermes skills_list; no SKILL: run)\n\
• `/tasks` · `/tasks all` · `view tasks` · `see tasks` · `show me the tasks` · `open tasks` · `list the tasks` — Active (open·WIP) or All task files under `~/.mac-stats/task/`\n\
• `/plugins` · `/plugins on` · `/plugins off` · `view plugins` · `see plugins` · `show me the plugins` · `open plugins` · `list the plugins` — registered script plugins On/Off list (no script run)\n\
• `/sessions` · `/sessions live` · `/sessions files` · `view sessions` · `see sessions` · `show me the sessions` · `open sessions` · `list the sessions` — Agent Ops Live/Files list\n\
• `/knowledge` · `/knowledge discord` · `/knowledge core` · `view knowledge` · `see knowledge` · `show me the knowledge` · `open knowledge` · `list the knowledge` — Agent Ops Knowledge list\n\
• `/schedules` · `/schedules jobs` · `/schedules deliveries` · `/cron list` · `view schedules` · `see schedules` · `show me the schedules` · `open schedules` · `list the schedules` — Agent Ops Jobs/Deliveries list\n\
• `/monitors` · `/monitors up` · `/monitors down` · `/monitors slow` · `view monitors` · `see monitors` · `show me the monitors` · `open monitors` · `list the monitors` — External / Monitors list\n\
• `/disk` · `/disk on` · `/disk off` · `/disk reclaim` · `/disk big` · `/disk clean` · `view disk` · `see disk` · `show me the disk cleanup` · `open disk` · `list the disk cleanup` — Disk Cleanup list\n\
• `/logs` · `/logs error` · `/logs warn` · `review logs` · `check logs` · `view logs` · `see logs` · `show me the logs` · `open logs` — Debug Log Error/Warn list\n\
• `how many log errors` · `log warn count` · `count the errors` · `number of warnings` — Debug Log error/warn counts (tail; inventory count asks only — no line dump; `delete these errors` / `clear the log` / `export warnings` stay with the agent)\n\
• `log file size` · `how big is the log` — Debug Log file size on disk (stat only)\n\
• `where is the log` · `log file path` — Debug Log path on disk (config only)\n\
• `log age` · `how old is the log` — Debug Log last write age (mtime; stat only)\n\
• `where is config` · `config path` · `mac-stats home` — config.json + data home paths (config only; `config size` / `config age` for bytes / mtime)\n\
• `config size` · `config.json size` · `how big is config` — config.json file size on disk (stat only; no dump; does not steal `config path` / `config age` / `.config.env`)\n\
• `config age` · `config.json age` · `how old is config` · `when was config updated` — config.json last write age (mtime; no dump; does not steal `config path` / `config size` / `.config.env`)\n\
• `config.env size` · `.config.env size` · `how big is .config.env` · `secrets env size` — `.config.env` file size on disk (stat only; no key dump; does not steal `config.env path` / `config.env age` / `config size`)\n\
• `config.env age` · `.config.env age` · `how old is .config.env` · `when was config.env updated` — `.config.env` last write age (mtime; no key dump; does not steal `config.env path` / `config.env size` / `config age`)\n\
• `config.env path` · `where is .config.env` · `config env path` — `~/.mac-stats/.config.env` path only (no key dump; `config.env size` / `config.env age` for bytes / mtime)\n\
• `improvements path` · `where is the improvements folder` · `autoresearch path` — `~/.mac-stats/improvements/` path only (no list; does not steal overnight improvements asks; `improvements size` / `improvements age` for bytes / mtime)\n\
• `improvements size` · `how big is the improvements folder` · `improvements dir size` — improvements folder size on disk (recursive file bytes; no list dump; does not steal `improvements path` / `improvements age`)\n\
• `improvements age` · `how old is the improvements folder` · `improvements dir age` · `when was improvements updated` — improvements folder last write age (newest file mtime; no list dump; does not steal `improvements path` / size / `results.tsv age` / overnight content)\n\
• `results.tsv path` · `where is results.tsv` · `autoresearch results path` · `ratchet results path` — `~/.mac-stats/improvements/autoresearch/results.tsv` path only (no dump; does not steal `improvements path` / `loop backlog path`)\n\
• `/keeps` · `keeps tonight` · `keep count` · `how many keeps` · `count the keeps` · `number of keeps` · `discard count` · `count the discards` · `discards tonight` · `ratchet summary` · `view keeps` · `see keeps` · `show me the keeps` · `open keeps` · `list the keeps` — keep/discard counts from results.tsv (tonight since 20:00 + all-time; inventory count asks only — no row dump; `delete these keeps` / `clear the discards` / `export keeps` stay with the agent; not path/size/age / morning surprise)\n\
• `/longest-streak` · `longest streak` · `best streak` · `record streak` · `view longest streak` / `open longest streak` — longest keep/discard streak from results.tsv (all-time + tonight since 20:00 only — no row dump; not current streak / counts / rate / last-row / recent list / path/size/age / morning surprise)\n\
• `/keep-streak` · `keep streak` · `current streak` · `ratchet streak` · `view keep streak` / `open keep streak` — consecutive keep/discard streak from results.tsv (current + tonight since 20:00 only — no row dump; not counts / rate / last-row / recent list / path/size/age / morning surprise)\n\
• `/keep-rate` · `keep rate` · `hit rate` · `ratchet hit rate` · `keep percentage` · `view keep rate` / `open keep rate` — keep/discard hit rate from results.tsv (tonight + all-time percentages only — no row dump; not counts / last-row / recent list / path/size/age / morning surprise)\n\
• `/recent-keeps` · `recent keeps` · `list recent keeps` · `tonight keep list` · `/recent-discards` · `recent discards` — short tonight keep/discard list from results.tsv (newest first; capped at 5; not counts / last-row / path/size/age / morning surprise)\n\
• `/last-keep` · `last keep` · `latest keep` · `what was the last keep` · `view last keep` · `/last-discard` · `last discard` · `latest discard` · `what was the last discard` — newest keep or discard row from results.tsv (one description only — no full dump; not counts / path/size/age / morning surprise)\n\
• `/first-keep` · `first keep` · `first keep tonight` · `earliest keep` · `what was the first keep` · `view first keep` / `open first keep` · `/first-discard` · `first discard tonight` — earliest keep or discard row tonight since 20:00 (one description only — no full dump; not `/last-keep` / `/since-keep` / `/keep-pace` / `/keep-median` / recent / counts / rate / streak / path/size/age / morning surprise)\n\
• `/keep-p98` · `keep p98` · `p98 keep gap` · `98th percentile keep gap` · `p98 gap between keeps` · `view keep p98` / `open keep p98` · `/discard-p98` · `discard p98` · `p98 discard gap` — p98 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p98 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p50` · `keep p50` · `p50 keep gap` · `50th percentile keep gap` · `p50 gap between keeps` · `view keep p50` / `open keep p50` · `/discard-p50` · `discard p50` · `p50 discard gap` — p50 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p50 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p20` · `keep p20` · `p20 keep gap` · `20th percentile keep gap` · `p20 gap between keeps` · `view keep p20` / `open keep p20` · `/discard-p20` · `discard p20` · `p20 discard gap` — p20 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p20 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p80` · `keep p80` · `p80 keep gap` · `80th percentile keep gap` · `p80 gap between keeps` · `view keep p80` / `open keep p80` · `/discard-p80` · `discard p80` · `p80 discard gap` — p80 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p80 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p15` · `keep p15` · `p15 keep gap` · `15th percentile keep gap` · `p15 gap between keeps` · `view keep p15` / `open keep p15` · `/discard-p15` · `discard p15` · `p15 discard gap` — p15 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p15 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p80` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p85` · `keep p85` · `p85 keep gap` · `85th percentile keep gap` · `p85 gap between keeps` · `view keep p85` / `open keep p85` · `/discard-p85` · `discard p85` · `p85 discard gap` — p85 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p85 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p80` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p40` · `keep p40` · `p40 keep gap` · `40th percentile keep gap` · `p40 gap between keeps` · `view keep p40` / `open keep p40` · `/discard-p40` · `discard p40` · `p40 discard gap` — p40 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p40 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p80` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p60` · `keep p60` · `p60 keep gap` · `60th percentile keep gap` · `p60 gap between keeps` · `view keep p60` / `open keep p60` · `/discard-p60` · `discard p60` · `p60 discard gap` — p60 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p60 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p80` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p70` · `keep p70` · `p70 keep gap` · `70th percentile keep gap` · `p70 gap between keeps` · `view keep p70` / `open keep p70` · `/discard-p70` · `discard p70` · `p70 discard gap` — p70 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p70 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p80` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p30` · `keep p30` · `p30 keep gap` · `30th percentile keep gap` · `p30 gap between keeps` · `view keep p30` / `open keep p30` · `/discard-p30` · `discard p30` · `p30 discard gap` — p30 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p30 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p80` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p35` · `keep p35` · `p35 keep gap` · `35th percentile keep gap` · `p35 gap between keeps` · `view keep p35` / `open keep p35` · `/discard-p35` · `discard p35` · `p35 discard gap` — p35 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p35 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p20` / `/keep-p80` / `/keep-p75` / `/keep-p50` / `/keep-p98` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p99` · `keep p99` · `p99 keep gap` · `99th percentile keep gap` · `p99 gap between keeps` · `view keep p99` / `open keep p99` · `/discard-p99` · `discard p99` · `p99 discard gap` — p99 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p99 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p1` · `keep p1` · `p1 keep gap` · `1st percentile keep gap` · `p1 gap between keeps` · `view keep p1` / `open keep p1` · `/discard-p1` · `discard p1` · `p1 discard gap` — p1 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p1 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p5` / `/keep-p2` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p2` · `keep p2` · `p2 keep gap` · `2nd percentile keep gap` · `p2 gap between keeps` · `view keep p2` / `open keep p2` · `/discard-p2` · `discard p2` · `p2 discard gap` — p2 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p2 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p5` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p9` · `keep p9` · `p9 keep gap` · `9th percentile keep gap` · `p9 gap between keeps` · `view keep p9` / `open keep p9` · `/discard-p9` · `discard p9` · `p9 discard gap` — p9 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p9 only — no row dump; digit-safe vs `/keep-p90` / `/keep-p95` / `/keep-p98` / `/keep-p99` / `/keep-p92`; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p8` · `keep p8` · `p8 keep gap` · `8th percentile keep gap` · `p8 gap between keeps` · `view keep p8` / `open keep p8` · `/discard-p8` · `discard p8` · `p8 discard gap` — p8 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p8 only — no row dump; digit-safe vs `/keep-p80` / `/keep-p85`; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p5` · `keep p5` · `p5 keep gap` · `5th percentile keep gap` · `p5 gap between keeps` · `view keep p5` / `open keep p5` · `/discard-p5` · `discard p5` · `p5 discard gap` — p5 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p5 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p75` · `keep p75` · `p75 keep gap` · `75th percentile keep gap` · `p75 gap between keeps` · `view keep p75` / `open keep p75` · `/discard-p75` · `discard p75` · `p75 discard gap` — p75 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p75 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p25` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p25` · `keep p25` · `p25 keep gap` · `25th percentile keep gap` · `p25 gap between keeps` · `view keep p25` / `open keep p25` · `/discard-p25` · `discard p25` · `p25 discard gap` — p25 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p25 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p10` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p10` · `keep p10` · `p10 keep gap` · `10th percentile keep gap` · `p10 gap between keeps` · `view keep p10` / `open keep p10` · `/discard-p10` · `discard p10` · `p10 discard gap` — p10 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p10 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p95` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p95` · `keep p95` · `p95 keep gap` · `95th percentile keep gap` · `p95 gap between keeps` · `view keep p95` / `open keep p95` · `/discard-p95` · `discard p95` · `p95 discard gap` — p95 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p95 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-p10` / `/keep-p25` / `/keep-p75` / `/keep-p99` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-histogram` · `keep histogram` · `gap histogram` · `keep gap histogram` · `histogram keep gap` · `view keep histogram` / `open keep histogram` · `/discard-histogram` · `discard histogram` · `discard gap histogram` — minute-bucket histogram (<5m · 5–10m · 10–20m · 20–30m · 30–60m · ≥60m) of gaps between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; histogram only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-gini` · `keep gini` · `gini keep gap` · `gini coefficient keep gap` · `gini gap between keeps` · `view keep gini` / `open keep gini` · `/discard-gini` · `discard gini` · `gini discard gap` — Gini coefficient (0–1) of gap lengths between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; Gini only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-mode` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-mode` · `keep mode` · `mode keep gap` · `modal keep gap` · `mode gap between keeps` · `view keep mode` / `open keep mode` · `/discard-mode` · `discard mode` · `mode discard gap` — modal (most frequent minute-bucketed) gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; mode only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/keep-gini` / `/keep-histogram` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-entropy` · `keep entropy` · `entropy keep gap` · `shannon entropy keep gap` · `entropy gap between keeps` · `view keep entropy` / `open keep entropy` · `/discard-entropy` · `discard entropy` · `entropy discard gap` — Shannon entropy (bits) of minute-bucketed gaps between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; entropy only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-mode` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-kurtosis` · `keep kurtosis` · `kurtosis keep gap` · `excess kurtosis keep gap` · `kurtosis gap between keeps` · `view keep kurtosis` / `open keep kurtosis` · `/discard-kurtosis` · `discard kurtosis` · `kurtosis discard gap` — excess kurtosis (Fisher G2) gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; kurtosis only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-entropy` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-skew` · `keep skew` · `skew keep gap` · `skewness keep gap` · `skew gap between keeps` · `view keep skew` / `open keep skew` · `/discard-skew` · `discard skew` · `skew discard gap` — skewness (Fisher–Pearson G1) gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; skew only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-kurtosis` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-cv` · `keep cv` · `cv keep gap` · `coefficient of variation keep gap` · `cv gap between keeps` · `view keep cv` / `open keep cv` · `/discard-cv` · `discard cv` · `cv discard gap` — CV (coefficient of variation = std/mean %) gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; CV only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-mad` · `keep mad` · `mad keep gap` · `median absolute deviation keep gap` · `mad gap between keeps` · `view keep mad` / `open keep mad` · `/discard-mad` · `discard mad` · `mad discard gap` — MAD (median absolute deviation) gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; MAD only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-cv` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-std` · `keep std` · `std keep gap` · `standard deviation keep gap` · `std gap between keeps` · `view keep std` / `open keep std` · `/discard-std` · `discard std` · `std discard gap` — sample std gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; std only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-mad` / `/keep-cv` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-iqr` · `keep iqr` · `iqr keep gap` · `interquartile keep gap` · `iqr gap between keeps` · `view keep iqr` / `open keep iqr` · `/discard-iqr` · `discard iqr` · `iqr discard gap` — IQR (Q3−Q1) gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; IQR only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-p90` · `keep p90` · `p90 keep gap` · `90th percentile keep gap` · `p90 gap between keeps` · `view keep p90` / `open keep p90` · `/discard-p90` · `discard p90` · `p90 discard gap` — p90 gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; p90 only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-iqr` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-range` · `keep range` · `keep gap range` · `min max keep gap` · `shortest and longest keep gap` · `view keep range` / `open keep range` · `/discard-range` · `discard range` · `discard gap range` — min–max gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; range only — no row dump; not `/keep-pace` / `/keep-median` / `/keep-p90` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-median` · `keep median` · `median keep gap` · `median gap between keeps` · `median time between keeps` · `view keep median` / `open keep median` · `/discard-median` · `discard median` · `median discard gap` — median gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; median only — no row dump; not `/keep-pace` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/keep-pace` · `keep pace` · `keep gap` · `time between keeps` · `average keep gap` · `average time between keeps` · `view keep pace` / `open keep pace` · `/discard-pace` · `discard pace` · `time between discards` — average gap between consecutive keep or discard rows from results.tsv (tonight since 20:00 + all-time; pace only — no row dump; not `/keep-median` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `/since-keep` · `since last keep` · `time since last keep` · `how long since last keep` · `how long ago was the last keep` · `view since keep` / `open since keep` · `/since-discard` · `since last discard` — age since newest keep or discard row (age only — no description dump; not `/last-keep` / `/first-keep` / `/keep-pace` / `/keep-median` / counts / rate / streak / recent / path/size/age / morning surprise)\n\
• `results.tsv size` · `how big is results.tsv` · `results file size` — results.tsv size on disk (stat only; no dump)\n\
• `results.tsv age` · `how old is results.tsv` · `when was results.tsv updated` — results.tsv last write age (mtime; no dump)\n\
• `loop backlog path` · `where is loop_backlog.md` · `harness tick log path` — `~/.mac-stats/improvements/loop_backlog.md` path only (no dump; does not steal `improvements path` / `results.tsv path`; `loop backlog size` for bytes · `loop backlog age` for mtime)\n\
• `loop backlog size` · `how big is loop_backlog.md` · `harness tick log size` — loop_backlog.md size on disk (stat only; no dump; does not steal `loop backlog path` / age / `improvements size` / `results.tsv size`)\n\
• `loop backlog age` · `how old is loop_backlog.md` · `harness tick log age` · `when was loop backlog updated` — loop_backlog.md last write age (mtime; no dump; does not steal path / size / `improvements age` / `results.tsv age`)\n\
• `sibling harness path` · `where is sibling_harness.md` · `openclaw hermes scan path` — `~/.mac-stats/improvements/sibling_harness.md` path only (no dump; does not steal `improvements path` / `loop backlog path` / standing backlog; `sibling harness size` for bytes · `sibling harness age` for mtime)\n\
• `sibling harness size` · `how big is sibling_harness.md` · `openclaw hermes scan size` · `overnight sibling harness size` — sibling_harness.md size on disk (stat only; no dump; does not steal `sibling harness path` / age / `improvements size` / `loop backlog size` / standing backlog)\n\
• `sibling harness age` · `how old is sibling_harness.md` · `openclaw hermes scan age` · `when was sibling harness updated` · `overnight sibling harness age` — sibling_harness.md last write age (mtime; no dump; does not steal path / size / `improvements age` / `loop backlog age` / standing backlog)\n\
• `standing backlog path` · `where is standing_backlog.md` · `overnight standing backlog path` · `track b backlog path` — `~/.mac-stats/improvements/standing_backlog.md` path only (no dump; does not steal `improvements path` / `loop backlog path` / `sibling harness path`; `standing backlog size` for bytes · `standing backlog age` for mtime)\n\
• `standing backlog size` · `how big is standing_backlog.md` · `overnight standing backlog size` · `track b backlog size` — standing_backlog.md size on disk (stat only; no dump; does not steal `standing backlog path` / age / `improvements size` / `loop backlog size` / `sibling harness size`)\n\
• `standing backlog age` · `how old is standing_backlog.md` · `overnight standing backlog age` · `when was standing backlog updated` · `track b backlog age` — standing_backlog.md last write age (mtime; no dump; does not steal path / size / `improvements age` / `loop backlog age` / `sibling harness age`)\n\
• `morning surprise path` · `where is morning_surprise.md` · `where is the morning surprise` · `overnight morning surprise path` — today's `morning_surprise_YYYY-MM-DD.md` path only (no dump; does not steal `improvements path` / standing / sibling / loop; `morning surprise size` for bytes · `morning surprise age` for mtime; ask *morning surprise?* for highlights)\n\
• `morning surprise size` · `how big is morning_surprise.md` · `overnight morning surprise size` · `today's morning surprise size` — today's morning surprise note size on disk (stat only; no dump; does not steal path / age / `improvements size` / standing / sibling / loop)\n\
• `morning surprise age` · `how old is morning_surprise.md` · `overnight morning surprise age` · `when was morning surprise updated` · `today's morning surprise age` — today's morning surprise note last write age (mtime; no dump; does not steal path / size / `improvements age` / standing / sibling / loop)\n\
• `overnight agent log path` · `where is overnight_agent.log` · `overnight_agent.log path` · `harness agent log path` — `~/.mac-stats/improvements/overnight_agent.log` path only (no dump/tail; does not steal `debug.log path` / morning surprise / improvements / loop / sibling / standing; `overnight agent log size` for bytes · `overnight agent log age` for mtime)\n\
• `overnight agent log size` · `how big is overnight_agent.log` · `overnight_agent.log size` · `harness agent log size` — overnight_agent.log size on disk (stat only; no dump/tail; does not steal path / age / `debug.log size` / morning surprise / improvements / loop / sibling / standing)\n\
• `overnight agent log age` · `how old is overnight_agent.log` · `overnight_agent.log age` · `harness agent log age` · `when was overnight agent log updated` — overnight_agent.log last write age (mtime; no dump/tail; does not steal path / size / `debug.log age` / morning surprise / improvements / loop / sibling / standing)\n\
• `harness loop stdout path` · `where is overnight_harness_loop.stdout.log` · `overnight_harness_loop.stdout.log path` · `harness stdout path` — `~/.mac-stats/improvements/overnight_harness_loop.stdout.log` path only (no dump/tail; does not steal overnight_agent.log / stderr / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; `harness loop stdout size` for bytes · `harness loop stdout age` for mtime)\n\
• `harness loop stdout size` · `how big is overnight_harness_loop.stdout.log` · `overnight_harness_loop.stdout.log size` · `harness stdout size` — overnight_harness_loop.stdout.log size on disk (stat only; no dump/tail; does not steal path / age / overnight_agent.log / stderr / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `harness loop stdout age` · `how old is overnight_harness_loop.stdout.log` · `overnight_harness_loop.stdout.log age` · `harness stdout age` · `when was harness loop stdout updated` — overnight_harness_loop.stdout.log last write age (mtime; no dump/tail; does not steal path / size / overnight_agent.log / stderr / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `harness loop stderr path` · `where is overnight_harness_loop.stderr.log` · `overnight_harness_loop.stderr.log path` · `harness stderr path` — `~/.mac-stats/improvements/overnight_harness_loop.stderr.log` path only (no dump/tail; does not steal overnight_agent.log / stdout / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; `harness loop stderr size` for bytes · `harness loop stderr age` for mtime)\n\
• `launchd stderr path` · `where is launchd.stderr.log` · `launchd.stderr.log path` · `mac-stats launchd stderr path` — `~/.mac-stats/launchd.stderr.log` path only (no dump/tail; does not steal harness loop stderr / LaunchAgent plist / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; `launchd stderr size` / `launchd stderr age` for bytes / mtime)\n\
• `launchd stderr size` · `how big is launchd.stderr.log` · `launchd.stderr.log size` · `mac-stats launchd stderr size` — `~/.mac-stats/launchd.stderr.log` size on disk (stat only; no dump/tail; does not steal path / age / harness loop stderr / LaunchAgent plist / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `launchd stderr age` · `how old is launchd.stderr.log` · `launchd.stderr.log age` · `mac-stats launchd stderr age` · `when was launchd stderr updated` — `~/.mac-stats/launchd.stderr.log` last write age (mtime; no dump/tail; does not steal path / size / harness loop stderr / LaunchAgent plist / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `launchd stdout path` · `where is launchd.stdout.log` · `launchd.stdout.log path` · `mac-stats launchd stdout path` — `~/.mac-stats/launchd.stdout.log` path only (no dump/tail; does not steal harness loop stdout / launchd stderr / LaunchAgent plist / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; `launchd stdout size` / `launchd stdout age` for bytes / mtime)\n\
• `launchd stdout size` · `how big is launchd.stdout.log` · `launchd.stdout.log size` · `mac-stats launchd stdout size` — `~/.mac-stats/launchd.stdout.log` size on disk (stat only; no dump/tail; does not steal path / age / harness loop stdout / launchd stderr / LaunchAgent plist / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `launchd stdout age` · `how old is launchd.stdout.log` · `launchd.stdout.log age` · `mac-stats launchd stdout age` · `when was launchd stdout updated` — `~/.mac-stats/launchd.stdout.log` last write age (mtime; no dump/tail; does not steal path / size / harness loop stdout / launchd stderr / LaunchAgent plist / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `harness loop stderr size` · `how big is overnight_harness_loop.stderr.log` · `overnight_harness_loop.stderr.log size` · `harness stderr size` — overnight_harness_loop.stderr.log size on disk (stat only; no dump/tail; does not steal path / age / overnight_agent.log / stdout / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `harness loop stderr age` · `how old is overnight_harness_loop.stderr.log` · `overnight_harness_loop.stderr.log age` · `harness stderr age` · `when was harness loop stderr updated` — overnight_harness_loop.stderr.log last write age (mtime; no dump/tail; does not steal path / size / overnight_agent.log / stdout / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing)\n\
• `credential accounts size` · `credential_accounts.json size` · `how big is credential accounts` · `keychain accounts size` — credential_accounts.json file size on disk (stat only; no dump; does not steal `credential accounts path` / `credential accounts age` / browser credentials)\n\
• `credential accounts age` · `credential_accounts.json age` · `how old is credential accounts` · `when was credential accounts updated` — credential_accounts.json last write age (mtime; no dump; does not steal `credential accounts path` / `credential accounts size` / browser credentials)\n\
• `credential accounts path` · `where is credential_accounts.json` · `keychain accounts path` — Keychain account-name list file (config only; no list/dump; `credential accounts size` / `credential accounts age` for bytes / mtime; does not steal browser credentials)\n\
• `escalation size` · `escalation_patterns.md size` · `how big is escalation patterns` · `escalation patterns size` — escalation_patterns.md file size on disk (stat only; no dump; does not steal `escalation patterns path` / `escalation patterns age` / session-reset / cookie reject)\n\
• `escalation age` · `escalation_patterns.md age` · `how old is escalation patterns` · `when was escalation patterns updated` · `escalation patterns age` — escalation_patterns.md last write age (mtime; no dump; does not steal `escalation patterns path` / size / session-reset / cookie reject)\n\
• `escalation patterns path` · `where is escalation_patterns.md` · `escalation file path` — escalation phrases file (config only; no list/append; `escalation patterns size` / `escalation patterns age` for bytes / mtime; does not steal session-reset)\n\
• `session reset size` · `session_reset_phrases.md size` · `how big is session reset phrases` · `session reset phrases size` — session_reset_phrases.md file size on disk (stat only; no dump; does not steal `session reset phrases path` / `session reset phrases age` / escalation / cookie reject)\n\
• `session reset age` · `session_reset_phrases.md age` · `how old is session reset phrases` · `when was session reset phrases updated` · `session reset phrases age` — session_reset_phrases.md last write age (mtime; no dump; does not steal `session reset phrases path` / size / escalation / cookie reject)\n\
• `session reset phrases path` · `where is session_reset_phrases.md` · `reset phrases path` — session reset phrases file (config only; no list/clear; `session reset phrases size` / `session reset phrases age` for bytes / mtime; does not steal escalation)\n\
• `before reset transcript size` · `before-reset transcript size` · `how big is before reset transcript` · `last_session_before_reset.jsonl size` — before-reset transcript size on disk (stat only; no dump; does not steal `before reset transcript path` / `before reset transcript age` / before-compaction / session reset phrases)\n\
• `before reset transcript age` · `before-reset transcript age` · `how old is before reset transcript` · `when was before reset transcript updated` · `last_session_before_reset.jsonl age` — before-reset transcript last write age (mtime; no dump; does not steal `before reset transcript path` / size / before-compaction / session reset phrases)\n\
• `before reset transcript path` · `where is before reset transcript` · `last_session_before_reset.jsonl` — before-reset JSONL path (config/env only; no dump/hook; `before reset transcript size` / `before reset transcript age` for bytes / mtime; does not steal before-compaction)\n\
• `before compaction transcript size` · `before-compaction transcript size` · `how big is before compaction transcript` · `last_session_before_compaction.jsonl size` — before-compaction transcript size on disk (stat only; no dump; does not steal `before compaction transcript path` / `before compaction transcript age` / before-reset / session reset phrases)\n\
• `before compaction transcript age` · `before-compaction transcript age` · `how old is before compaction transcript` · `when was before compaction transcript updated` · `last_session_before_compaction.jsonl age` — before-compaction transcript last write age (mtime; no dump; does not steal `before compaction transcript path` / size / before-reset / session reset phrases)\n\
• `before compaction transcript path` · `where is before compaction transcript` · `last_session_before_compaction.jsonl` — before-compaction JSONL path (config/env only; no dump/hook; `before compaction transcript size` / `before compaction transcript age` for bytes / mtime; does not steal before-reset)\n\
• `cookie reject size` · `cookie_reject_patterns.md size` · `how big is cookie reject patterns` · `cookie reject patterns size` — cookie_reject_patterns.md file size on disk (stat only; no dump; does not steal `cookie reject patterns path` / `cookie reject patterns age` / session-reset / escalation)\n\
• `cookie reject age` · `cookie_reject_patterns.md age` · `how old is cookie reject patterns` · `when was cookie reject patterns updated` · `cookie reject patterns age` — cookie_reject_patterns.md last write age (mtime; no dump; does not steal `cookie reject patterns path` / size / session-reset / escalation)\n\
• `cookie reject patterns path` · `where is cookie_reject_patterns.md` · `cookie reject path` · `reject patterns path` — cookie reject patterns file (config only; no list/edit; `cookie reject patterns size` / `cookie reject patterns age` for bytes / mtime; does not steal browser cookies)\n\
• `organizer rules size` · `downloads-organizer-rules.md size` · `how big is downloads organizer rules` · `downloads organizer rules size` — downloads-organizer-rules.md file size on disk (stat only; no dump; does not steal `downloads organizer rules path` / `downloads organizer rules age` / organizer state / `/downloads`)\n\
• `organizer rules age` · `downloads-organizer-rules.md age` · `how old is downloads organizer rules` · `when was downloads organizer rules updated` · `downloads organizer rules age` — downloads-organizer-rules.md last write age (mtime; no dump; does not steal `downloads organizer rules path` / size / organizer state / `/downloads`)\n\
• `downloads organizer rules path` · `where is downloads-organizer-rules.md` · `organizer rules path` — Downloads organizer rules file (config only; no list/run; `downloads organizer rules size` / `downloads organizer rules age` for bytes / mtime; does not steal `/downloads`)\n\
• `organizer state size` · `downloads-organizer-state.json size` · `how big is downloads organizer state` · `downloads organizer state size` — downloads-organizer-state.json file size on disk (stat only; no dump; does not steal `downloads organizer state path` / `downloads organizer state age` / rules / `/downloads`)\n\
• `organizer state age` · `downloads-organizer-state.json age` · `how old is downloads organizer state` · `when was downloads organizer state updated` · `downloads organizer state age` — downloads-organizer-state.json last write age (mtime; no dump; does not steal `downloads organizer state path` / size / rules / `/downloads`)\n\
• `downloads organizer state path` · `where is downloads-organizer-state.json` · `organizer state path` — Downloads organizer state file (config only; no dump/run; `downloads organizer state size` / `downloads organizer state age` for bytes / mtime; does not steal `/downloads`)\n\
• `screenshot path` · `where are screenshots` · `screenshot folder` — BROWSER_SCREENSHOT save dir (config only; `screenshots size` / `screenshots age` for disk use / mtime)\n\
• `screenshots size` · `how big are screenshots` · `screenshots folder size` — screenshots folder size on disk (recursive file bytes; no list dump; does not steal `screenshot path` / `screenshots age` / take/list)\n\
• `screenshots age` · `how old are screenshots` · `screenshots folder age` · `when was screenshots updated` — screenshots folder last write age (newest file mtime; no list dump; does not steal `screenshot path` / size / take/list)\n\
• `runs path` · `where is runs.jsonl` · `runs file path` — runs.jsonl path (config only; no list/count)\n\
• `runs size` · `how big is runs.jsonl` · `runs file size` — runs.jsonl size on disk (stat only; no list/count)\n\
• `runs age` · `how old is runs.jsonl` · `when was runs updated` — runs.jsonl last write age (mtime; no list/count)\n\
• `task age` · `how old are tasks` · `task folder age` · `when was tasks updated` — task folder last write age (newest file mtime; no list dump; does not steal `task path` / size / `/tasks`)\n\
• `task size` · `how big are tasks` · `task folder size` — task folder size on disk (recursive file bytes; no list dump; does not steal `task path` / `task age` / `/tasks`)\n\
• `task path` · `where is the task folder` · `task directory` — `~/.mac-stats/task/` path (config only; no list/create; `task size` / `task age` for bytes / mtime)\n\
• `notes size` · `how big are notes` · `memory folder size` · `notes folder size` — notes folder size on disk (recursive file bytes; no list dump; does not steal `memory path` / `notes path` / `notes age` / scrub / bare `memory size` installed RAM)\n\
• `memory size` · `how big is memory` · `how many gb of ram` · `ram capacity` — installed RAM plus used (no LLM; `how much ram` stays the % chip; notes size / `memory.md` size stay on disk; `why` stays with the agent)\n\
• `what chip is this` · `what processor` · `which chip` · `cpu name` — Apple chip name plus core count (no LLM; `what's the cpu` stays the CPU ring; `which model` stays Ollama; `why` and hot chips stay with the agent)\n\
• `how much free space` · `how much space is left` · `how many gb free` · `how much disk is free` — SSD free bytes (no LLM; `how much disk is used` and bare `free space` stay the % chip; Disk Cleanup stays on `/disk`; `why` stays with the agent)\n\
• `notes age` · `how old are notes` · `memory folder age` · `notes folder age` — notes folder last write age (newest file mtime; no list dump; does not steal `memory path` / `notes path` / `notes size` / `memory.md age` / bare `memory age`)\n\
• `memory path` · `notes path` · `where are notes` · `notes folder` — `~/.mac-stats/agents/notes/` + `memory.md` (config only; no list/save; `notes size` / `notes age` for disk use / mtime)\n\
• `memory.md size` · `curated memory size` · `how big is memory.md` · `memory file size` — curated `agents/memory.md` size on disk (stat only; no dump; does not steal `memory.md path` / `memory.md age` / `notes size` / bare `memory size`)\n\
• `memory.md age` · `curated memory age` · `how old is memory.md` · `when was memory.md updated` — curated `agents/memory.md` last write age (mtime; no dump; does not steal `memory.md path` / `memory.md size` / notes folder / bare `memory age`)\n\
• `memory.md path` · `where is memory.md` · `curated memory path` — curated `agents/memory.md` only (config only; no dump/edit; `memory.md size` / `memory.md age` for bytes / mtime; does not steal `memory path` / notes folder)\n\
• `soul size` · `soul.md size` · `how big is soul` · `soul file size` — soul.md file size on disk (stat only; no dump; does not steal `soul path` / `soul age` / mood / agents)\n\
• `soul age` · `soul.md age` · `how old is soul` · `when was soul updated` — soul.md last write age (mtime; no dump; does not steal `soul path` / `soul size` / mood / agents)\n\
• `soul path` · `where is soul.md` · `soul file path` — shared `agents/soul.md` (config only; no dump/edit; `soul size` / `soul age` for bytes / mtime; does not steal `/agents`)\n\
• `mood size` · `mood.md size` · `how big is mood` · `mood file size` — per-agent mood.md size on disk (stat only; no dump; does not steal `mood path` / `mood age` / soul / agents)\n\
• `mood age` · `mood.md age` · `how old is mood` · `when was mood updated` — newest per-agent mood.md last write age (mtime; no dump; does not steal `mood path` / `mood size` / soul / agents)\n\
• `mood path` · `where is mood.md` · `mood file path` — per-agent `agent-<id>/mood.md` (config only; no dump/edit; `mood size` / `mood age` for bytes / mtime; does not steal `/agents`)\n\
• `skill.md size` · `skill file size` · `how big is skill.md` — per-agent skill.md size on disk (stat only; no dump; does not steal `skill.md path` / `skill.md age` / `skills size` / `/skills`)\n\
• `skill.md age` · `skill file age` · `how old is skill.md` · `when was skill.md updated` — newest per-agent skill.md last write age (mtime; no dump; does not steal `skill.md path` / `skill.md size` / `skills age` / mood / soul)\n\
• `skill.md` · `where is skill.md` · `skill file path` — per-agent `agent-<id>/skill.md` (config only; no dump/edit; `skill.md size` / `skill.md age` for bytes / mtime; does not steal `/skills`)\n\
• `testing size` · `testing.md size` · `how big is testing.md` · `testing file size` — per-agent testing.md size on disk (stat only; no dump; does not steal `testing.md path` / `testing.md age` / run tests / `/agents`)\n\
• `testing age` · `testing.md age` · `how old is testing` · `when was testing updated` — newest per-agent testing.md last write age (mtime; no dump; does not steal `testing.md path` / `testing.md size` / skill / mood / soul; does not run tests)\n\
• `testing.md` · `where is testing.md` · `testing path` · `testing file path` — per-agent `agent-<id>/testing.md` (config only; no dump/edit/run; `testing.md size` / `testing.md age` for bytes / mtime; does not steal `/agents`)\n\
• `agent.json size` · `agent config size` · `how big is agent.json` — per-agent agent.json size on disk (stat only; no dump; does not steal `agent.json path` / `agent.json age` / `agents size` / `config.json`)\n\
• `agent.json age` · `agent config age` · `how old is agent.json` · `when was agent.json updated` — newest per-agent agent.json last write age (mtime; no dump; does not steal `agent.json path` / `agent.json size` / agents / config.json)\n\
• `agent.json` · `where is agent.json` · `agent.json path` · `agent config path` — per-agent `agent-<id>/agent.json` (config only; no dump/edit; `agent.json size` / `agent.json age` for bytes / mtime; does not steal `agents path` / `config path`)\n\
• `discord memory size` · `memory-discord size` · `how big is discord memory` · `channel memory size` — Discord channel `memory-discord-*.md` size on disk (stat only; no dump; does not steal `discord memory path` / `discord memory age` / `/knowledge discord` / `memory.md size`)\n\
• `discord memory age` · `memory-discord age` · `how old is discord memory` · `when was discord memory updated` · `channel memory age` — newest `memory-discord-*.md` last write age (mtime; no dump; does not steal `discord memory path` / `discord memory size` / `/knowledge discord`)\n\
• `discord memory path` · `where is discord memory` · `memory-discord path` — Discord channel `memory-discord-<id>.md` under agents/ (config only; no list/dump; `discord memory size` / `discord memory age` for bytes / mtime; does not steal `/knowledge discord`)\n\
• `session memory size` · `session-memory size` · `how big is session memory` — `session-memory-*.md` size on disk (stat only; no dump; does not steal `session memory path` / `session memory age` / `session size` / `/sessions`)\n\
• `session memory age` · `session-memory age` · `how old is session memory` · `when was session memory updated` — newest `session-memory-*.md` last write age (mtime; no dump; does not steal `session memory path` / `session memory size` / `session size` / `/sessions`)\n\
• `session memory path` · `where is session memory` · `session-memory path` — `session/session-memory-<id>-<ts>-<topic>.md` (config only; no list/dump; `session memory size` / `session memory age` for bytes / mtime; does not steal `session path` / `/sessions`)\n\
• `launchagent age` · `how old is the launchagent` · `mac-stats.plist age` · `harness plist age` — LaunchAgent plist last write ages (mtime; no dump; does not steal `launchagent path` / size / load/unload)\n\
• `launchagent size` · `how big is the launchagent` · `mac-stats.plist size` · `harness plist size` — LaunchAgent plist sizes on disk (stat only; no dump; does not steal `launchagent path` / age / load/unload)\n\
• `launchagent path` · `where is launchagent` · `mac-stats.plist` · `harness plist` — `~/Library/LaunchAgents/com.raro42.mac-stats.plist` + overnight harness plist (path only; `launchagent size` / `launchagent age` for bytes / mtime; no load/unload)\n\
• `session age` · `how old are sessions` · `session folder age` · `when was sessions updated` — session folder last write age (newest file mtime; no list dump; does not steal `session path` / size / `session memory age` / `/sessions`)\n\
• `session size` · `how big are sessions` · `session folder size` — session folder size on disk (recursive file bytes; no list dump; does not steal `session path` / `session age` / `/sessions`)\n\
• `session path` · `where is the session folder` · `session directory` — `~/.mac-stats/session/` path (config only; no list/resume; `session size` / `session age` for bytes / mtime)\n\
• `agents age` · `how old are agents` · `agents folder age` · `when was agents updated` — agents folder last write age (newest file mtime; no list dump; does not steal `agents path` / size / `agent.json age` / `/agents`)\n\
• `agents size` · `how big are agents` · `agents folder size` — agents folder size on disk (recursive file bytes; no list dump; does not steal `agents path` / `agents age` / `/agents`)\n\
• `skills age` · `how old are skills` · `skills folder age` · `when was skills updated` — skills folder last write age (newest file mtime; no list dump; does not steal `skills path` / size / `skill.md age` / `/skills`)\n\
• `skills size` · `how big are skills` · `skills folder size` — skills folder size on disk (recursive file bytes; no list dump; does not steal `skills path` / `skills age` / `/skills` / `skill.md size`)\n\
• `plugins age` · `scripts age` · `how old are plugins` · `plugins folder age` · `when was plugins updated` — plugins/scripts folder last write age (newest file mtime; no list dump; does not steal `plugins path` / size / `/plugins`)\n\
• `plugins size` · `scripts size` · `how big are plugins` · `plugins folder size` — plugins/scripts folder size on disk (recursive file bytes; no list dump; does not steal `plugins path` / `plugins age` / `/plugins`)\n\
• `prompts age` · `how old are prompts` · `prompts folder age` · `when was prompts updated` — prompts folder last write age (newest file mtime; no list dump; does not steal `prompts path` / size / planning·execution file ages)\n\
• `prompts size` · `how big are prompts` · `prompts folder size` — prompts folder size on disk (recursive file bytes; no list dump; does not steal `prompts path` / `prompts age` / planning·execution file paths)\n\
• `planning size` · `planning_prompt.md size` · `how big is planning_prompt.md` · `planning prompt size` — planning_prompt.md size on disk (stat only; no dump; does not steal `planning_prompt.md path` / `planning_prompt.md age` / `prompts size`)\n\
• `planning age` · `planning_prompt.md age` · `how old is planning` · `when was planning updated` — planning_prompt.md last write age (mtime; no dump; does not steal `planning_prompt.md path` / `planning_prompt.md size` / `prompts path` / execution)\n\
• `planning_prompt.md` · `where is planning_prompt.md` · `planning prompt path` · `planning path` — `agents/prompts/planning_prompt.md` (config only; no dump/edit; `planning_prompt.md size` / `planning_prompt.md age` for bytes / mtime; does not steal `prompts path`)\n\
• `execution size` · `execution_prompt.md size` · `how big is execution_prompt.md` · `execution prompt size` — execution_prompt.md size on disk (stat only; no dump; does not steal `execution_prompt.md path` / `execution_prompt.md age` / `prompts size`)\n\
• `execution age` · `execution_prompt.md age` · `how old is execution` · `when was execution updated` — execution_prompt.md last write age (mtime; no dump; does not steal `execution_prompt.md path` / `execution_prompt.md size` / `prompts path` / planning)\n\
• `execution_prompt.md` · `where is execution_prompt.md` · `execution prompt path` · `execution path` — `agents/prompts/execution_prompt.md` (config only; no dump/edit; `execution_prompt.md size` / `execution_prompt.md age` for bytes / mtime; does not steal `prompts path`)\n\
• `agents path` · `where is the agents folder` · `agents directory` — `~/.mac-stats/agents/` path (config only; no list/create; `agents size` / `agents age` for disk use / mtime)\n\
• `skills path` · `where is the skills folder` · `skills directory` — `~/.mac-stats/agents/skills/` path (config only; no list/run; `skills size` / `skills age` for disk use / mtime)\n\
• `plugins path` · `scripts path` · `where is the plugins folder` — `~/.mac-stats/scripts/` path (config only; no list/run; `plugins size` / `plugins age` for disk use / mtime)\n\
• `prompts path` · `where is the prompts folder` · `prompts directory` — `~/.mac-stats/agents/prompts/` path (config only; no open/edit; `prompts size` / `prompts age` for disk use / mtime)\n\
• `tmp path` · `where is the tmp folder` · `temp directory` — `~/.mac-stats/tmp/` path (config only; no list/prune; `tmp size` / `tmp age` for bytes / mtime)\n\
• `tmp size` · `how big is tmp` · `tmp folder size` — tmp folder size on disk (recursive file bytes; no list dump; does not steal `tmp path` / `tmp age` / prune)\n\
• `tmp age` · `how old is tmp` · `tmp folder age` · `when was tmp updated` — tmp folder last write age (newest file mtime; no list dump; does not steal `tmp path` / `tmp size` / prune)\n\
• `uploads age` · `how old are uploads` · `uploads folder age` · `when was uploads updated` — uploads folder last write age (newest file mtime; no list dump; does not steal `uploads path` / `uploads size` / upload)\n\
• `uploads size` · `how big are uploads` · `uploads folder size` — uploads folder size on disk (recursive file bytes; no list dump; does not steal `uploads path` / `uploads age` / upload)\n\
• `uploads path` · `where is the uploads folder` · `upload directory` — `~/.mac-stats/uploads/` path (config only; no list/upload; `uploads size` / `uploads age` for bytes / mtime)\n\
• `traces age` · `how old are traces` · `traces folder age` · `when was traces updated` · `cdp traces age` — CDP traces folder last write age (newest file mtime; no list dump; does not steal `traces path` / `traces size` / prune)\n\
• `traces size` · `how big are traces` · `traces folder size` · `cdp traces size` — CDP traces folder size on disk (recursive file bytes; no list dump; does not steal `traces path` / `traces age` / prune)\n\
• `traces path` · `where is the traces folder` · `cdp traces` — `~/.mac-stats/traces/` path (config only; no list/prune; `traces size` / `traces age` for bytes / mtime)\n\
• `pdfs age` · `how old are pdfs` · `pdfs folder age` · `when was pdfs updated` · `pdf exports age` — PDF exports folder last write age (newest file mtime; no list dump; does not steal `pdfs path` / `pdfs size` / save)\n\
• `pdfs size` · `how big are pdfs` · `pdfs folder size` — PDF exports folder size on disk (recursive file bytes; no list dump; does not steal `pdfs path` / `pdfs age` / save)\n\
• `pdfs path` · `where is the pdfs folder` · `pdf directory` — `~/.mac-stats/pdfs/` path (config only; no list/save; `pdfs size` / `pdfs age` for bytes / mtime)\n\
• `browser credentials size` · `browser-credentials.toml size` · `how big are browser credentials` · `browser credentials file size` — browser-credentials.toml file size on disk (stat only; no dump; does not steal `browser credentials path` / `browser credentials age` / storage state / credential accounts)\n\
• `browser credentials age` · `browser-credentials.toml age` · `how old are browser credentials` · `when was browser credentials updated` — browser-credentials.toml last write age (mtime; no dump; does not steal `browser credentials path` / size / storage state / credential accounts)\n\
• `browser credentials path` · `where are browser credentials` · `browser-credentials.toml` — credentials TOML path (config only; no list/edit; `browser credentials size` / `browser credentials age` for bytes / mtime)\n\
• `storage state size` · `browser_storage_state.json size` · `how big are browser cookies` · `browser cookies size` — browser_storage_state.json file size on disk (stat only; no dump; does not steal `storage state path` / `storage state age` / cookie reject / browser credentials)\n\
• `storage state age` · `browser_storage_state.json age` · `how old are browser cookies` · `when was storage state updated` — browser_storage_state.json last write age (mtime; no dump; does not steal `storage state path` / size / cookie reject / browser credentials)\n\
• `storage state path` · `where are browser cookies` · `browser_storage_state.json` — cookie jar path (config only; no list/clear; `storage state size` / `storage state age` for bytes / mtime)\n\
• `browser downloads age` · `how old are browser downloads` · `browser-downloads age` · `when was browser downloads updated` · `cdp downloads age` — CDP download folder last write age (newest file mtime; no list dump; does not steal `browser downloads path` / `browser downloads size` / `/downloads`)\n\
• `browser downloads size` · `how big are browser downloads` · `browser-downloads size` — CDP download folder size on disk (recursive file bytes; no list dump; does not steal `browser downloads path` / `browser downloads age` / `/downloads`)\n\
• `browser downloads path` · `where are browser downloads` · `browser-downloads` — CDP download dir (config only; no list/prune; `browser downloads size` / `browser downloads age` for bytes / mtime; does not steal `/downloads`)\n\
• `cleanup quarantine size` · `how big is quarantine` · `quarantine folder size` — cleanup-quarantine folder size on disk (recursive file bytes; no list dump; does not steal `cleanup quarantine path` / `/disk`)\n\
• `cleanup quarantine path` · `where is cleanup-quarantine` · `quarantine folder` — Disk Cleanup soft-delete dir (config only; no list/prune; `cleanup quarantine size` for disk use; does not steal `/disk`)\n\
• `pinned processes size` · `pinned_processes.json size` · `how big is pinned processes` — pinned_processes.json file size on disk (stat only; no dump; does not steal `pinned processes path` / `pinned processes age` / `/pinned`)\n\
• `pinned processes age` · `pinned_processes.json age` · `how old is pinned processes` · `when was pinned processes updated` — pinned_processes.json last write age (mtime; no dump; does not steal `pinned processes path` / `pinned processes size` / `/pinned`)\n\
• `pinned processes path` · `where is pinned_processes.json` · `pin file path` — Top Processes favorites file (config only; no list/pin; `pinned processes size` / `pinned processes age` for bytes / mtime; does not steal `/pinned`)\n\
• `schedules size` · `schedules.json size` · `how big is schedules` — schedules.json file size on disk (stat only; no dump; does not steal `schedules path` / `schedules age` / `/schedules`)\n\
• `schedules age` · `schedules.json age` · `how old is schedules` · `when was schedules updated` — schedules.json last write age (mtime; no dump; does not steal `schedules path` / `schedules size` / `/schedules`)\n\
• `schedules path` · `where is schedules.json` · `schedule file path` — Jobs/deliveries config file (config only; no list/create; `schedules size` / `schedules age` for bytes / mtime; does not steal `/schedules`)\n\
• `monitors size` · `monitors.json size` · `how big is monitors` — monitors.json file size on disk (stat only; no dump; does not steal `monitors path` / `monitors age` / `/monitors`)\n\
• `monitors age` · `monitors.json age` · `how old is monitors` · `when was monitors updated` — monitors.json last write age (mtime; no dump; does not steal `monitors path` / `monitors size` / `/monitors`)\n\
• `monitors path` · `where is monitors.json` · `monitor file path` — External / website monitors config file (config only; no list/add/check; `monitors size` / `monitors age` for bytes / mtime; does not steal `/monitors`)\n\
• `history size` · `history.json size` · `how big is history` — history.json file size on disk (stat only; no dump; does not steal `history path` / `history age` / chat history)\n\
• `history age` · `history.json age` · `how old is history` · `when was history updated` — history.json last write age (mtime; no dump; does not steal `history path` / `history size` / chat history)\n\
• `history path` · `where is history.json` · `metrics history file` — CPU / metrics sparkline buffer file (config only; no dump/charts; `history size` / `history age` for bytes / mtime; does not steal chat history)\n\
• `disk cleanup size` · `disk_cleanup.json size` · `how big is disk cleanup` — disk_cleanup.json file size on disk (stat only; no dump; does not steal `disk cleanup path` / `disk cleanup age` / `/disk` / quarantine)\n\
• `disk cleanup age` · `disk_cleanup.json age` · `how old is disk cleanup` · `when was disk cleanup updated` — disk_cleanup.json last write age (mtime; no dump; does not steal `disk cleanup path` / `disk cleanup size` / `/disk`)\n\
• `disk cleanup path` · `where is disk_cleanup.json` · `cleanup file path` — Disk Cleanup scopes file (config only; no list/reclaim; `disk cleanup size` / `disk cleanup age` for bytes / mtime; does not steal `/disk`)\n\
• `perplexity last size` · `perplexity_last.json size` · `how big is perplexity last` — perplexity_last.json file size on disk (stat only; no dump; does not steal `perplexity last path` / `perplexity last age` / `/perplexity`)\n\
• `perplexity last age` · `perplexity_last.json age` · `how old is perplexity last` · `when was perplexity last updated` — perplexity_last.json last write age (mtime; no dump; does not steal `perplexity last path` / `perplexity last size` / `/perplexity`)\n\
• `perplexity last path` · `where is perplexity_last.json` · `last search file` — last Perplexity Search cache file (config only; no Top/Snippet dump; `perplexity last size` / `perplexity last age` for bytes / mtime; does not steal `/perplexity`)\n\
• `discord channels size` · `discord_channels.json size` · `how big is discord channels` — discord_channels.json file size on disk (stat only; no dump; does not steal `discord channels path` / `discord channels age` / `/discord`)\n\
• `discord channels age` · `discord_channels.json age` · `how old is discord channels` · `when was discord channels updated` — discord_channels.json last write age (mtime; no dump; does not steal `discord channels path` / `discord channels size` / `/discord`)\n\
• `discord channels path` · `where is discord_channels.json` · `channels.json` — Discord per-channel config file (config only; no list/edit; `discord channels size` / `discord channels age` for bytes / mtime; does not steal `/discord`)\n\
• `delivery awareness size` · `scheduler_delivery_awareness.json size` · `how big is delivery awareness` — scheduler_delivery_awareness.json file size on disk (stat only; no dump; does not steal `delivery awareness path` / `delivery awareness age` / `last delivery` / `/schedules`)\n\
• `delivery awareness age` · `scheduler_delivery_awareness.json age` · `how old is delivery awareness` · `when was delivery awareness updated` — scheduler_delivery_awareness.json last write age (mtime; no dump; does not steal `delivery awareness path` / `delivery awareness size` / `last delivery` / `/schedules`)\n\
• `delivery awareness path` · `where is scheduler_delivery_awareness.json` · `awareness file path` — scheduler Discord delivery log file (config only; no list; `delivery awareness size` / `delivery awareness age` for bytes / mtime; does not steal `last delivery` / `/schedules`)\n\
• `user info size` · `user-info.json size` · `how big is user info` — user-info.json file size on disk (stat only; no dump; does not steal `user info path` / `user info age` / who-am-i)\n\
• `user info age` · `user-info.json age` · `how old is user info` · `when was user info updated` — user-info.json last write age (mtime; no dump; does not steal `user info path` / `user info size` / who-am-i)\n\
• `user info path` · `where is user-info.json` · `user-info path` — Discord display-name map file (config only; no list/edit; `user info size` / `user info age` for bytes / mtime)\n\
• `/processes` · `/processes hot` · `/hot` · `/processes pinned` · `/pinned` · `view processes` · `see processes` · `show me the processes` · `open processes` · `list the processes` · `view hot` · `see hot` · `show me the hot` · `open hot` · `list the hot` · `view pinned` · `see pinned` · `show me the pinned` · `open pinned` · `list the pinned` — Top Processes Hot/Pinned list (exact open only — not rings/strip/details Hot · not pinned path/size/age)\n\
• `what's using the most cpu` · `which process is using the most cpu` · `what's eating the cpu` — one Top CPU process (no LLM; `/processes` stays the list; `hot processes` stays Hot; `how much cpu` stays the CPU ring; `why` stays with the agent)\n\
• `what's using the most ram` · `which process is using the most memory` · `what's eating the ram` — one Top RAM process (no LLM; `/processes` stays the list; `hot processes` stays Hot; `how much ram` stays the % chip; `how big is memory` stays installed RAM; `why` stays with the agent)\n\
• `what's using the most gpu` · `which process is using the most gpu` · `what's eating the gpu` — one Top GPU process (no LLM; `/processes` stays the list; `hot processes` stays Hot; `how much gpu` and `is the gpu hot` stay the GPU ring; `why` stays with the agent)\n\
• `how long have you been running` · `app uptime` · `process uptime` — mac-stats process uptime (no LLM; `what's the uptime` and `how long has the mac been up` stay the Up chip; `why` stays with the agent)\n\
• `/rings` · `/rings hot` · `view rings` · `see rings` · `show me the rings` · `open rings` · `list the rings` — CPU rings All/Hot list (menu-bar amber thresholds; exact open only)\n\
• `/cpu` · `/gpu` · `/freq` · `/temp` · `view cpu` · `see cpu` · `show me the cpu` · `open cpu` · `list the cpu` · `view gpu` · `open freq` · `open temp` · `how hot` · `is the cpu hot` · `is the gpu hot` · `how much cpu` · `is the cpu high` · `is the cpu busy` · `how much gpu` · `is the gpu high` · `is the gpu busy` · `how fast` · `clock speed` · `is the frequency high` · `p core frequency` · `e core frequency` · `how fast are the p cores` · `how fast are the e cores` — CPU · GPU · Freq · Temp ring chips, plus P-core / E-core clocks (exact open only; not rings / details / cpu window; `why is the cpu hot`, `why is the cpu high`, `why is the gpu high`, `why is the frequency high`, and `why is the p core high` stay with the agent; `how fast is the cpu` stays the Freq ring)\n\
• `/strip` · `/strip hot` · `/power` · `view strip` · `see strip` · `show me the strip` · `open strip` · `list the strip` · `view power` · `open power` — power strip All/Hot list (menu-bar amber / attention cues; exact open only)\n\
• `/battery` · `/bat` · `/heat` · `/thermal` · `/lpm` · `/ram` · `/ssd` · `/uptime` · `view battery` · `see battery` · `show me the battery` · `open battery` · `list the battery` · `how much battery` · `battery left` · `is the battery low` · `is it charging` · `is the battery charging` · `is it plugged in` · `how much ram` · `how much memory` · `is the ram high` · `how much disk` · `how much ssd` · `how much storage` · `is the disk full` · `how much power` · `power draw` · `how many watts` · `is the power high` · `view heat` · `open thermal` · `is the heat high` · `is the thermal high` · `is it throttling` · `view lpm` · `open ram` · `view ssd` · `open uptime` — power-strip Bat · Heat · LPM · RAM · SSD · Up chips, plus Power watts (exact open only; not `/power` strip / low power mode / disk cleanup / details / path·size·age; `why is the battery low`, `why is it charging`, `why is the ram high`, `why is the disk full`, `why is the power high`, and `why is the heat high` stay with the agent; `is the cpu hot` stays the Temp ring)\n\
• `/details` · `/details hot` · `/load` · `view details` · `see details` · `show me the details` · `open details` · `list the details` · `view load` · `open load` · `is the load high` · `how's the load` · `how high is the load` · `how loaded` · `5 minute load` · `15 minute load` · `load 5` · `load 15` — Details Load · RAM · Up, plus Load 1m / 5m / 15m chips (Load≥4 hot; exact open only; `/load` stays the full panel; `why is the load high` and `why is the 5 minute load high` stay with the agent)\n\
• `/perplexity` · `/perplexity top` · `/perplexity snippet` · `view perplexity` · `see perplexity` · `show me the perplexity` · `open perplexity` · `list the perplexity` · `view last search` · `open top results` · `list the snippet results` — last Perplexity Top/Snippet list (exact open only — not key / live search / path·size·age)\n\
• `/digest` · `refresh digest` · `run digester` · `rescan digest` — refresh digester (latest.md/json)\n\
• `digest open` · `view digest` · `see digest` · `show me the digest` · `open digest` · `list the digest` · `view open candidates` — cached open candidates (exact open only — no digester spawn; not path/size/age; bare `digest` / `show me digest` still refreshes)\n\
• `digest age` — cached digest timestamp (no digester spawn)\n\
• `digest size` · `how big is the digest` · `latest.json size` — digest `latest.json` size on disk (stat only; no digester spawn)\n\
• `digest.md size` · `latest.md size` · `how big is digest.md` — digest `latest.md` size on disk (stat only; no digester spawn; does not steal `digest size`)\n\
• `digest.md age` · `latest.md age` · `how old is digest.md` · `when was latest.md updated` — digest `latest.md` last write age (mtime; no digester spawn; does not steal `digest age`)\n\
• `scrub memory` — remove polluted memory lines\n\
• `stop` / `cancel` / `interrupt` — interrupt an in-flight run\n\
• `/ops` · `/help` · `view help` · `see help` · `show me the help` · `open help` · `list the help` · `view ops` · `open ops` · `list the ops` — this menu (exact open only — not help me / help with …)\n\
\n\
**Scheduled:** wake-up 06:00 · CHANGELOG hygiene Mondays 10:00 · UI review Wednesdays 11:00 (`docs/041_ui_command_center.md`)",
        slow_ms = OPS_RUNS_SLOW_MS,
    )
}

