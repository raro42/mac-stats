//! Deterministic pre-routing: skip the LLM planning step for unambiguous patterns.
//!
//! Screenshot + URL → BROWSER_SCREENSHOT; "run <command>" → RUN_CMD;
//! "fetch <URL>" → FETCH_URL; "search <query>" → BRAVE_SEARCH / PERPLEXITY_SEARCH;
//! ticket → REDMINE_API; "list schedules" → LIST_SCHEDULES;
//! "list tasks" → TASK_LIST; "show task <id>" → TASK_SHOW;
//! "list models" → OLLAMA_API: list_models.

use tracing::info;

use crate::commands::redmine_helpers::{
    extract_ticket_id, is_redmine_time_entries_request, redmine_request_for_routing,
    redmine_time_entries_range,
};
use crate::commands::reply_helpers::{
    extract_last_prefixed_argument, extract_screenshot_recommendation, extract_url_from_question,
};

/// Try to pre-route the question to a tool without asking the LLM.
///
/// Returns `Some(recommendation_string)` when the question unambiguously maps to a tool,
/// or `None` when the LLM planning step is needed.
pub(crate) fn compute_pre_routed_recommendation(
    question: &str,
    request_for_verification: &str,
    is_verification_retry: bool,
) -> Option<String> {
    extract_screenshot_recommendation(question).or_else(|| {
        let run_cmd_rec = try_pre_route_run_cmd(question);
        if run_cmd_rec.is_some() {
            return run_cmd_rec;
        }
        // Google SERP HTML via FETCH_URL is huge and low-signal — rewrite to web search.
        let google_rec = try_pre_route_google_serp_as_search(question);
        if google_rec.is_some() {
            return google_rec;
        }
        let fetch_rec = try_pre_route_fetch_url(question);
        if fetch_rec.is_some() {
            return fetch_rec;
        }
        let lighthouse_rec = try_pre_route_lighthouse_pagespeed(question);
        if lighthouse_rec.is_some() {
            return lighthouse_rec;
        }
        let weather_rec = try_pre_route_weather(question);
        if weather_rec.is_some() {
            return weather_rec;
        }
        let search_rec = try_pre_route_web_search(question);
        if search_rec.is_some() {
            return search_rec;
        }
        let mgmt_rec = try_pre_route_management_commands(question);
        if mgmt_rec.is_some() {
            return mgmt_rec;
        }
        let discord_rec = try_pre_route_discord_api(question);
        if discord_rec.is_some() {
            return discord_rec;
        }
        try_pre_route_redmine(question, request_for_verification, is_verification_retry)
    })
}

/// Google `/search?q=…` SERPs are a bad FETCH_URL target — rewrite to Brave/Perplexity.
fn try_pre_route_google_serp_as_search(question: &str) -> Option<String> {
    let url = extract_url_from_question(question)?;
    let query = google_serp_search_query(&url)?;
    let brave_ok = crate::commands::brave::get_brave_api_key().is_some();
    let perplexity_ok = crate::commands::perplexity::is_perplexity_configured().unwrap_or(false);
    if !brave_ok && !perplexity_ok {
        return None;
    }
    let (tool, label) = if brave_ok {
        ("BRAVE_SEARCH", "google SERP rewrite")
    } else {
        ("PERPLEXITY_SEARCH", "google SERP rewrite")
    };
    info!(
        "Agent router: pre-routed to {} ({}): {}",
        tool,
        label,
        crate::logging::ellipse(&query, 80)
    );
    Some(format!("{tool}: {query}"))
}

/// Extract `q=` from a Google `/search` URL (also used when FETCH_URL still hits a SERP).
pub(crate) fn google_serp_search_query(url_or_arg: &str) -> Option<String> {
    let url = extract_url_from_question(url_or_arg).unwrap_or_else(|| url_or_arg.trim().to_string());
    let parsed = url::Url::parse(&url).ok()?;
    let host = parsed.host_str()?.to_lowercase();
    let is_google = host == "google.com"
        || host == "www.google.com"
        || host.ends_with(".google.com");
    if !is_google {
        return None;
    }
    if !parsed.path().starts_with("/search") {
        return None;
    }
    let q_param = parsed
        .query_pairs()
        .find(|(k, _)| k == "q")
        .map(|(_, v)| v.into_owned())?;
    let query = q_param.trim();
    if query.is_empty() {
        None
    } else {
        Some(query.to_string())
    }
}

/// "run <command>" / "RUN_CMD: <command>" → `RUN_CMD: <arg>`.
fn try_pre_route_run_cmd(question: &str) -> Option<String> {
    if !crate::commands::run_cmd::is_local_cmd_allowed() {
        return None;
    }
    let q = question.trim();
    let q_lower = q.to_lowercase();
    let cmd_rest = if let Some(cmd) = extract_last_prefixed_argument(q, "RUN_CMD:") {
        cmd
    } else if q_lower.starts_with("run command:") {
        q[12..].trim().to_string()
    } else if q_lower.starts_with("run ") {
        q[4..].trim().to_string()
    } else {
        String::new()
    };
    if cmd_rest.is_empty() {
        return None;
    }
    let rec = format!("RUN_CMD: {}", cmd_rest);
    info!(
        "Agent router: pre-routed to RUN_CMD (run command): {}",
        crate::logging::ellipse(&cmd_rest, 60)
    );
    Some(rec)
}

/// "fetch <URL>" / "FETCH_URL: <URL>" / "get the page at <URL>" → `FETCH_URL: <url>`.
///
/// Only triggers when the question contains a URL and clear fetch/read intent.
/// Does NOT trigger for browser/navigate/screenshot patterns (handled upstream).
fn try_pre_route_fetch_url(question: &str) -> Option<String> {
    let q = question.trim();
    let q_lower = q.to_lowercase();

    // Skip if the question looks like a browser/navigate task (screenshot pre-route
    // already ran, but we also avoid "navigate to" / "open in browser" patterns).
    if q_lower.contains("screenshot")
        || q_lower.contains("navigate")
        || q_lower.contains("click")
        || q_lower.contains("scroll")
        || (q_lower.contains("open") && q_lower.contains("browser"))
    {
        return None;
    }

    // Explicit FETCH_URL: prefix
    if let Some(arg) = extract_last_prefixed_argument(q, "FETCH_URL:") {
        if let Some(url) = extract_url_from_question(&arg) {
            info!(
                "Agent router: pre-routed to FETCH_URL (explicit prefix): {}",
                crate::logging::ellipse(&url, 80)
            );
            return Some(format!("FETCH_URL: {url}"));
        }
    }

    // Must contain a URL for the remaining keyword-based detection
    let url = extract_url_from_question(q)?;

    let has_fetch_intent = q_lower.contains("fetch ")
        || q_lower.contains("get the page")
        || q_lower.contains("get the content")
        || q_lower.contains("get the html")
        || q_lower.contains("read the page")
        || q_lower.contains("read the url")
        || q_lower.contains("read the site")
        || q_lower.contains("what's on ")
        || q_lower.contains("what is on ")
        || q_lower.contains("summarize the page")
        || q_lower.contains("summarize this page")
        || q_lower.contains("summarize this url")
        || q_lower.contains("summarize the url")
        || q_lower.contains("summarize the site")
        || q_lower.contains("summarise the page")
        || q_lower.contains("summarise this url");

    if has_fetch_intent {
        info!(
            "Agent router: pre-routed to FETCH_URL (keyword + URL): {}",
            crate::logging::ellipse(&url, 80)
        );
        return Some(format!("FETCH_URL: {url}"));
    }

    None
}

/// "Review example.com using lighthouse / pagespeed" → open PageSpeed Insight URL in browser.
fn try_pre_route_lighthouse_pagespeed(question: &str) -> Option<String> {
    let q = question.trim();
    let q_lower = q.to_lowercase();
    if !(q_lower.contains("lighthouse") || q_lower.contains("pagespeed")) {
        return None;
    }
    if q_lower.contains("and then ")
        || q_lower.contains("skill:")
        || q_lower.contains("cursor_agent:")
        || q_lower.contains("redmine")
    {
        return None;
    }

    // Prefer an explicit http(s) URL when present.
    let site = if let Some(url) = extract_url_from_question(q) {
        url
    } else {
        extract_bare_hostname(&q_lower)?
    };
    let site = site.trim_end_matches('/').to_string();
    let encoded = site
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | ':' | '/' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect::<String>();
    let analysis = format!("https://pagespeed.web.dev/analysis?url={encoded}");
    info!(
        "Agent router: pre-routed to BROWSER_SCREENSHOT (lighthouse/pagespeed): {}",
        crate::logging::ellipse(&analysis, 100)
    );
    Some(format!("BROWSER_SCREENSHOT: {analysis}"))
}

/// First bare hostname like `satisfecho.de` / `www.example.com` in the question.
fn extract_bare_hostname(q_lower: &str) -> Option<String> {
    for token in q_lower
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '"' | '\'' | ')' | '('))
    {
        let t = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-');
        if t.len() < 4 || !t.contains('.') {
            continue;
        }
        if t.starts_with("http") {
            continue;
        }
        let parts: Vec<&str> = t.split('.').collect();
        if parts.len() < 2 {
            continue;
        }
        let tld = parts.last().copied().unwrap_or("");
        if tld.len() < 2 || tld.len() > 24 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        if parts
            .iter()
            .any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        {
            continue;
        }
        return Some(format!("https://{t}"));
    }
    None
}

/// Weather / "wether" questions.
/// Clear place names skip pre-route so the agent router uses Open-Meteo instant (0 LLM).
/// Ambiguous weather asks still pre-route to Brave/Perplexity (with Open-Meteo grounding).
fn try_pre_route_weather(question: &str) -> Option<String> {
    if !crate::commands::weather_grounding::looks_like_weather_query(question) {
        return None;
    }
    if crate::commands::weather_grounding::can_instant_weather(question) {
        info!(
            "Agent router: weather has clear place — skip search pre-route (Open-Meteo instant)"
        );
        return None;
    }
    let q = question.trim();
    let brave_ok = crate::commands::brave::get_brave_api_key().is_some();
    let perplexity_ok = crate::commands::perplexity::is_perplexity_configured().unwrap_or(false);
    let query = format!("weather {}", q);
    if brave_ok {
        info!(
            "Agent router: pre-routed weather → BRAVE_SEARCH: {}",
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("BRAVE_SEARCH: {query}"));
    }
    if perplexity_ok {
        info!(
            "Agent router: pre-routed weather → PERPLEXITY_SEARCH: {}",
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("PERPLEXITY_SEARCH: {query}"));
    }
    Some(format!("BRAVE_SEARCH: {query}"))
}

/// "search for <query>" / "google <query>" / "BRAVE_SEARCH: <query>" → web search tool.
///
/// Routes to BRAVE_SEARCH when Brave is configured, or PERPLEXITY_SEARCH when only
/// Perplexity is configured. Explicit "PERPLEXITY_SEARCH: <query>" always routes to
/// Perplexity (when configured). "research <query>" prefers Perplexity over Brave.
///
/// Skips pre-routing when the question contains multi-step indicators (browser actions,
/// "and then", "send to", etc.) that need LLM planning.
fn try_pre_route_web_search(question: &str) -> Option<String> {
    let q = question.trim();
    let q_lower = q.to_lowercase();

    // Skip multi-step / compound requests that need LLM planning.
    if q_lower.contains("and then ")
        || q_lower.contains("after that ")
        || q_lower.contains("send to ")
        || q_lower.contains("post to ")
        || q_lower.contains("screenshot")
        || q_lower.contains("navigate")
        || q_lower.contains("click")
    {
        return None;
    }

    let brave_ok = crate::commands::brave::get_brave_api_key().is_some();
    let perplexity_ok = crate::commands::perplexity::is_perplexity_configured().unwrap_or(false);

    if !brave_ok && !perplexity_ok {
        return None;
    }

    // Explicit "PERPLEXITY_SEARCH: <query>"
    if perplexity_ok {
        if let Some(arg) = extract_last_prefixed_argument(q, "PERPLEXITY_SEARCH:") {
            let query = arg.trim().to_string();
            if !query.is_empty() {
                info!(
                    "Agent router: pre-routed to PERPLEXITY_SEARCH (explicit prefix): {}",
                    crate::logging::ellipse(&query, 80)
                );
                return Some(format!("PERPLEXITY_SEARCH: {query}"));
            }
        }
        // Natural language: "Research using perplexity: …" / "using perplexity: …"
        for marker in [
            "research using perplexity:",
            "using perplexity:",
            "perplexity search:",
            "perplexity:",
        ] {
            if let Some(pos) = q_lower.find(marker) {
                let before = q_lower[..pos].trim();
                if !before.is_empty()
                    && !before.ends_with("please")
                    && !before.ends_with("can you")
                    && !before.ends_with("could you")
                    && !before.ends_with("pls")
                {
                    continue;
                }
                let query = q[pos + marker.len()..].trim().trim_end_matches('?').trim();
                if !query.is_empty() {
                    info!(
                        "Agent router: pre-routed to PERPLEXITY_SEARCH (natural language): {}",
                        crate::logging::ellipse(query, 80)
                    );
                    return Some(format!("PERPLEXITY_SEARCH: {query}"));
                }
            }
        }
    }

    // Explicit "BRAVE_SEARCH: <query>"
    if brave_ok {
        if let Some(arg) = extract_last_prefixed_argument(q, "BRAVE_SEARCH:") {
            let query = arg.trim().to_string();
            if !query.is_empty() {
                info!(
                    "Agent router: pre-routed to BRAVE_SEARCH (explicit prefix): {}",
                    crate::logging::ellipse(&query, 80)
                );
                return Some(format!("BRAVE_SEARCH: {query}"));
            }
        }
    }

    // Bare / short news asks ("Any news?", "today's headlines") — prefer Perplexity.
    if crate::commands::perplexity_helpers::is_news_query(q) && q.chars().count() <= 96 {
        let query = bare_news_search_query(&q_lower, q);
        if !query.is_empty() {
            let (tool, label) = if perplexity_ok {
                ("PERPLEXITY_SEARCH", "news ask")
            } else if brave_ok {
                ("BRAVE_SEARCH", "news ask")
            } else {
                return None;
            };
            info!(
                "Agent router: pre-routed to {} ({}): {}",
                tool,
                label,
                crate::logging::ellipse(&query, 80)
            );
            return Some(format!("{tool}: {query}"));
        }
    }

    // Short topic dumps ("IT, AI, Stocks, BTC, nerd stuff") — treat as web search.
    if let Some(query) = topic_dump_search_query(q) {
        let (tool, label) = if brave_ok {
            ("BRAVE_SEARCH", "topic dump")
        } else {
            ("PERPLEXITY_SEARCH", "topic dump")
        };
        info!(
            "Agent router: pre-routed to {} ({}): {}",
            tool,
            label,
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("{tool}: {query}"));
    }

    // Clear flight / airline route asks ("Flights from Atlanta to MTY", "Viva Aerobus flights…").
    if let Some(query) = flight_search_query(q) {
        let (tool, label) = if brave_ok {
            ("BRAVE_SEARCH", "flight ask")
        } else {
            ("PERPLEXITY_SEARCH", "flight ask")
        };
        info!(
            "Agent router: pre-routed to {} ({}): {}",
            tool,
            label,
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("{tool}: {query}"));
    }

    // Conference / event date reviews ("Review IBM techxchange dates in Atlanta…").
    if let Some(query) = event_dates_search_query(q) {
        let (tool, label) = if brave_ok {
            ("BRAVE_SEARCH", "event dates")
        } else if perplexity_ok {
            ("PERPLEXITY_SEARCH", "event dates")
        } else {
            return None;
        };
        info!(
            "Agent router: pre-routed to {} ({}): {}",
            tool,
            label,
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("{tool}: {query}"));
    }

    // Multi-leg travel plans ("travel to Atlanta in October and … Los mochis afterward").
    if let Some(query) = travel_plan_search_query(q) {
        let (tool, label) = if perplexity_ok {
            ("PERPLEXITY_SEARCH", "travel plan")
        } else if brave_ok {
            ("BRAVE_SEARCH", "travel plan")
        } else {
            return None;
        };
        info!(
            "Agent router: pre-routed to {} ({}): {}",
            tool,
            label,
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("{tool}: {query}"));
    }

    // Short airport hop chains without the word "flight" ("I would like ATL to Monterrey to LMM").
    if let Some(query) = airport_hop_search_query(q) {
        let (tool, label) = if brave_ok {
            ("BRAVE_SEARCH", "airport hop")
        } else {
            ("PERPLEXITY_SEARCH", "airport hop")
        };
        info!(
            "Agent router: pre-routed to {} ({}): {}",
            tool,
            label,
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("{tool}: {query}"));
    }

    // Bare person/company research topics ("Florian Fischer delivery hero problem").
    if let Some(query) = bare_research_topic_query(q) {
        let (tool, label) = if perplexity_ok {
            ("PERPLEXITY_SEARCH", "bare research topic")
        } else if brave_ok {
            ("BRAVE_SEARCH", "bare research topic")
        } else {
            return None;
        };
        info!(
            "Agent router: pre-routed to {} ({}): {}",
            tool,
            label,
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("{tool}: {query}"));
    }

    // Keyword-based search intent detection.
    // Extract the search query from the question after the keyword.
    let search_query = extract_search_query(&q_lower, q);
    if let Some((query, is_research)) = search_query {
        if query.is_empty() {
            return None;
        }
        // "research" prefers Perplexity; "search" / "google" / "look up" prefers Brave.
        let (tool, label) = if is_research && perplexity_ok {
            ("PERPLEXITY_SEARCH", "research keyword")
        } else if brave_ok {
            ("BRAVE_SEARCH", "search keyword")
        } else {
            ("PERPLEXITY_SEARCH", "search keyword (Brave unavailable)")
        };
        info!(
            "Agent router: pre-routed to {} ({}): {}",
            tool,
            label,
            crate::logging::ellipse(&query, 80)
        );
        return Some(format!("{tool}: {query}"));
    }

    None
}

/// Search string for short news asks. Bare "any news?" gets a default headline query.
fn bare_news_search_query(q_lower: &str, q_original: &str) -> String {
    let stripped = q_original.trim().trim_end_matches('?').trim();
    let compact = q_lower
        .trim()
        .trim_end_matches('?')
        .trim()
        .replace(['!', '.'], "");
    let bare = matches!(
        compact.as_str(),
        "any news"
            | "news"
            | "the news"
            | "whats the news"
            | "what's the news"
            | "what is the news"
            | "latest news"
            | "todays headlines"
            | "today's headlines"
            | "headlines"
            | "top stories"
            | "breaking news"
            | "current events"
    ) || (compact.starts_with("any news") && compact.chars().count() <= 24);
    if bare {
        "top world and technology headlines today".to_string()
    } else {
        stripped.to_string()
    }
}

/// Comma-heavy short topic lists without an explicit search verb.
fn topic_dump_search_query(q: &str) -> Option<String> {
    let trimmed = q.trim().trim_end_matches('?').trim();
    let len = trimmed.chars().count();
    if !(12..=96).contains(&len) {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("http")
        || lower.contains("redmine")
        || lower.contains("skill:")
        || lower.contains("cursor_agent:")
        || lower.contains("search ")
        || lower.contains("research ")
        || lower.contains("look up")
        || lower.contains(" and then ")
    {
        return None;
    }
    let commas = trimmed.matches(',').count();
    if commas < 2 {
        return None;
    }
    // Prefer lists of short tokens, not long prose clauses.
    let parts: Vec<&str> = trimmed.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
    if parts.len() < 3 {
        return None;
    }
    if parts.iter().any(|p| p.split_whitespace().count() > 6) {
        return None;
    }
    Some(trimmed.to_string())
}

/// Clear flight / airline route asks — skip planning LLM and search immediately.
/// Complex multi-leg planning ("I want to be in Atlanta two days before…") stays with the agent.
fn flight_search_query(q: &str) -> Option<String> {
    let trimmed = q.trim().trim_end_matches('?').trim();
    let len = trimmed.chars().count();
    if !(12..=140).contains(&len) {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("http")
        || lower.contains("skill:")
        || lower.contains("cursor_agent:")
        || lower.contains("redmine")
        || lower.contains(" and then ")
        || lower.contains("book ")
        || lower.contains("remember ")
        || lower.contains("save ")
        || lower.contains("schedule")
        || lower.contains("memory")
    {
        return None;
    }
    let has_flight = lower.contains("flight")
        || lower.contains("flights")
        || lower.contains("aerobus")
        || lower.contains("airline")
        || lower.contains("airlines");
    if !has_flight {
        return None;
    }
    let has_route = lower.contains(" from ")
        || lower.contains(" to ")
        || lower.contains("→")
        || lower.contains("->")
        || lower.contains(" → ");
    let what_about_airline = lower.starts_with("what about ") && has_flight;
    if !has_route && !what_about_airline {
        return None;
    }
    // Long itinerary prose with dates / "I want" stays with the planner.
    if lower.contains("i want")
        || lower.contains("two days before")
        || lower.contains("around ")
        || lower.contains("itinerary")
    {
        return None;
    }
    Some(trimmed.to_string())
}

/// Conference / summit / event date looks — skip planning when the ask is clearly web lookup.
/// Travel constraints after the event name ("I want to be in Atlanta two days before") stay in the
/// search query so the tool result can still ground the reply.
fn event_dates_search_query(q: &str) -> Option<String> {
    let trimmed = q.trim().trim_end_matches('?').trim();
    let len = trimmed.chars().count();
    if !(18..=240).contains(&len) {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("http")
        || lower.contains("skill:")
        || lower.contains("cursor_agent:")
        || lower.contains("redmine")
        || lower.contains(" and then ")
        || lower.contains("remember ")
        || lower.contains("save ")
        || lower.contains("schedule")
        || lower.contains("memory")
        || lower.contains("book ")
    {
        return None;
    }
    let eventish = lower.contains("techxchange")
        || lower.contains("conference")
        || lower.contains("summit")
        || lower.contains("meetup")
        || lower.contains("symposium")
        || lower.contains("tradeshow")
        || lower.contains("trade show")
        || lower.contains(" expo")
        || lower.starts_with("expo ")
        || lower.contains("event dates")
        || lower.contains("dates for ")
        || (lower.contains(" dates") && (lower.contains(" in ") || lower.contains(" at ")));
    if !eventish {
        return None;
    }
    let reviewish = lower.starts_with("review ")
        || lower.starts_with("check ")
        || lower.starts_with("find ")
        || lower.starts_with("look up ")
        || lower.starts_with("lookup ")
        || lower.contains("when is ")
        || lower.contains("what are the dates")
        || lower.contains("dates in ")
        || lower.contains("dates for ");
    if !reviewish {
        return None;
    }
    Some(trimmed.to_string())
}

/// Multi-city travel plans without an explicit "flight" word — prefer Perplexity research.
/// Preference dumps ("I want to be back in Barcelona… LMM-MEX-BCN") stay with the instant lane.
fn travel_plan_search_query(q: &str) -> Option<String> {
    let trimmed = q.trim().trim_end_matches('?').trim();
    let len = trimmed.chars().count();
    if !(28..=240).contains(&len) {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("http")
        || lower.contains("skill:")
        || lower.contains("cursor_agent:")
        || lower.contains("redmine")
        || lower.contains(" and then ")
        || lower.contains("remember ")
        || lower.contains("save ")
        || lower.contains("schedule")
        || lower.contains("memory")
        || lower.contains("book ")
        || lower.contains("techxchange")
        || lower.contains("conference")
        || lower.contains("review ")
    {
        return None;
    }
    // Preference / return-home dumps are handled by the instant lane.
    if lower.contains("i want to be back")
        || lower.contains("i want to be in")
        || (lower.contains("lmm") && lower.contains("mex") && lower.contains("bcn"))
    {
        return None;
    }
    let travelish = lower.contains("travel")
        || lower.contains("travelling")
        || lower.contains("traveling")
        || lower.contains("trip to")
        || lower.contains("going to travel")
        || lower.contains("we are going to");
    if !travelish {
        return None;
    }
    let multi = lower.contains(" and want to go to ")
        || lower.contains(" and then go to ")
        || lower.contains(" afterward")
        || lower.contains(" afterwards")
        || lower.contains(" after that")
        || lower.contains(" then to ")
        || lower.contains(" and then to ");
    if !multi {
        return None;
    }
    let places = [
        "atlanta",
        "mochis",
        "monterrey",
        "barcelona",
        "mexico",
        "madrid",
        "miami",
    ];
    let place_hits = places.iter().filter(|p| lower.contains(*p)).count();
    let month = [
        "january", "february", "march", "april", "may", "june", "july", "august", "september",
        "october", "november", "december",
    ]
    .iter()
    .any(|m| lower.contains(m));
    // Need two named places, or one place + month when a second-leg cue is present.
    if place_hits < 2 && !(place_hits >= 1 && month) {
        return None;
    }
    Some(trimmed.to_string())
}

/// Short airport-code hop chains without an explicit "flight" word.
/// Conversational corrections ("you missed that leg", "I live in BCN…") stay with the agent.
fn airport_hop_search_query(q: &str) -> Option<String> {
    let trimmed = q.trim().trim_end_matches('?').trim();
    let len = trimmed.chars().count();
    if !(10..=120).contains(&len) {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("http")
        || lower.contains("skill:")
        || lower.contains("cursor_agent:")
        || lower.contains("redmine")
        || lower.contains(" and then ")
        || lower.contains("book ")
        || lower.contains("remember ")
        || lower.contains("save ")
        || lower.contains("schedule")
        || lower.contains("memory")
        || lower.contains("messing")
        || lower.contains("i live")
        || lower.contains("you missed")
        || lower.contains("still have to")
        || lower.contains("two days before")
        || lower.contains("around ")
        || lower.contains("itinerary")
        || lower.contains("october")
        || lower.contains("november")
    {
        return None;
    }
    // Already handled by flight_search_query.
    if lower.contains("flight") || lower.contains("airline") || lower.contains("aerobus") {
        return None;
    }
    let has_hop = lower.contains(" to ")
        || lower.contains(" - ")
        || lower.contains("→")
        || lower.contains("->");
    if !has_hop {
        return None;
    }
    const AIRPORTS: &[&str] = &[
        "atl", "bcn", "mty", "lmm", "mex", "jfk", "mad", "lax", "ord", "sfo", "mia", "ewr",
    ];
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let code_hits = AIRPORTS
        .iter()
        .filter(|code| tokens.iter().any(|t| t == *code))
        .count();
    let place_hits = [
        "monterrey",
        "barcelona",
        "atlanta",
        "mochis",
        "los mochis",
    ]
    .iter()
    .filter(|p| lower.contains(*p))
    .count();
    if code_hits + place_hits < 2 {
        return None;
    }
    Some(format!("flights {trimmed}"))
}

/// Short bare research topics without an explicit search verb (person + company + "problem").
fn bare_research_topic_query(q: &str) -> Option<String> {
    let trimmed = q.trim().trim_end_matches('?').trim();
    let len = trimmed.chars().count();
    let words = trimmed.split_whitespace().count();
    if !(16..=100).contains(&len) || !(3..=10).contains(&words) {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("http")
        || lower.contains("skill:")
        || lower.contains("cursor_agent:")
        || lower.contains("redmine")
        || lower.contains("search ")
        || lower.contains("research ")
        || lower.contains("look up")
        || lower.contains("google ")
        || lower.contains("please ")
        || lower.contains("can you")
        || lower.contains("could you")
        || lower.contains(" and then ")
        || lower.contains("flight")
        || lower.contains("weather")
    {
        return None;
    }
    let researchy = lower.contains(" problem")
        || lower.ends_with("problem")
        || lower.contains(" scandal")
        || lower.contains(" controversy")
        || lower.contains(" lawsuit")
        || lower.contains(" ceo");
    if !researchy {
        return None;
    }
    Some(trimmed.to_string())
}

/// Extract a search query from keyword patterns. Returns `(query, is_research)`.
/// `is_research` is true for "research ..." patterns (prefers Perplexity).
fn extract_search_query(q_lower: &str, q_original: &str) -> Option<(String, bool)> {
    // Ordered by specificity: longer patterns first to avoid partial matches.
    let patterns: &[(&str, bool)] = &[
        ("search the web for ", false),
        ("search the internet for ", false),
        ("web search for ", false),
        ("web search ", false),
        ("search online for ", false),
        ("search for ", false),
        ("look up ", false),
        ("lookup ", false),
        ("google ", false),
        ("research ", true),
        ("search ", false),
    ];

    for &(pattern, is_research) in patterns {
        if let Some(pos) = q_lower.find(pattern) {
            let before = q_lower[..pos].trim();
            if !before.is_empty()
                && !before.ends_with("please")
                && !before.ends_with("can you")
                && !before.ends_with("could you")
                && !before.ends_with("pls")
            {
                continue;
            }
            let query = q_original[pos + pattern.len()..].trim().to_string();
            let query = query.trim_end_matches('?').trim().to_string();
            if !query.is_empty() {
                return Some((query, is_research));
            }
            // A matching pattern with empty query means the user typed
            // the keyword but no search terms — stop here instead of
            // falling through to a shorter, less specific pattern.
            return None;
        }
    }
    None
}

/// Management commands: LIST_SCHEDULES, TASK_LIST, TASK_SHOW, OLLAMA_API list_models.
///
/// These are simple, unambiguous commands that don't need LLM planning.
fn try_pre_route_management_commands(question: &str) -> Option<String> {
    let q = question.trim();
    let q_lower = q.to_lowercase();

    // Skip multi-step / compound requests that need LLM planning.
    if q_lower.contains("and then ")
        || q_lower.contains("after that ")
        || q_lower.contains("send to ")
        || q_lower.contains("post to ")
    {
        return None;
    }

    // Explicit prefixes always win.
    if q_lower.starts_with("list_schedules") {
        info!("Agent router: pre-routed to LIST_SCHEDULES (explicit prefix)");
        return Some("LIST_SCHEDULES:".to_string());
    }
    if q_lower.starts_with("task_list") {
        let arg = if q.len() > "TASK_LIST:".len() {
            q["TASK_LIST:".len()..].trim()
        } else {
            ""
        };
        info!("Agent router: pre-routed to TASK_LIST (explicit prefix)");
        return Some(format!("TASK_LIST: {arg}"));
    }
    if let Some(arg) = extract_last_prefixed_argument(q, "TASK_SHOW:") {
        let arg = arg.trim().to_string();
        if !arg.is_empty() {
            info!(
                "Agent router: pre-routed to TASK_SHOW (explicit prefix): {}",
                crate::logging::ellipse(&arg, 40)
            );
            return Some(format!("TASK_SHOW: {arg}"));
        }
    }
    if let Some(arg) = extract_last_prefixed_argument(q, "OLLAMA_API:") {
        let arg = arg.trim().to_string();
        if !arg.is_empty() {
            info!(
                "Agent router: pre-routed to OLLAMA_API (explicit prefix): {}",
                crate::logging::ellipse(&arg, 40)
            );
            return Some(format!("OLLAMA_API: {arg}"));
        }
    }

    // Keyword-based detection for schedules.
    if let Some(rec) = try_pre_route_list_schedules(&q_lower) {
        return Some(rec);
    }

    // Keyword-based detection for tasks.
    if let Some(rec) = try_pre_route_task_commands(&q_lower, q) {
        return Some(rec);
    }

    // Keyword-based detection for Ollama model management.
    try_pre_route_ollama_api(&q_lower)
}

/// "list schedules", "show schedules", "what's scheduled" → LIST_SCHEDULES.
fn try_pre_route_list_schedules(q_lower: &str) -> Option<String> {
    let is_list_schedules = q_lower == "list schedules"
        || q_lower == "show schedules"
        || q_lower == "show my schedules"
        || q_lower == "list my schedules"
        || q_lower.starts_with("what's scheduled")
        || q_lower.starts_with("what is scheduled")
        || q_lower.starts_with("what are my schedules")
        || q_lower == "schedules"
        || q_lower == "my schedules";

    if is_list_schedules {
        info!("Agent router: pre-routed to LIST_SCHEDULES (keyword)");
        return Some("LIST_SCHEDULES:".to_string());
    }
    None
}

/// "list tasks", "show tasks", "show task <id>" → TASK_LIST or TASK_SHOW.
fn try_pre_route_task_commands(q_lower: &str, q_original: &str) -> Option<String> {
    // TASK_LIST: "list tasks", "show tasks", "tasks", etc.
    let is_task_list = q_lower == "list tasks"
        || q_lower == "show tasks"
        || q_lower == "list my tasks"
        || q_lower == "show my tasks"
        || q_lower == "tasks"
        || q_lower == "my tasks"
        || q_lower == "open tasks"
        || q_lower == "list open tasks"
        || q_lower == "all tasks"
        || q_lower == "list all tasks";

    if is_task_list {
        let arg = if q_lower.contains("all") { "all" } else { "" };
        info!("Agent router: pre-routed to TASK_LIST (keyword)");
        return Some(format!("TASK_LIST: {arg}"));
    }

    // TASK_SHOW: "show task <id>", "task <id>" when <id> is a number or path-like string.
    let show_prefixes: &[&str] = &["show task ", "show me task ", "task details "];
    for prefix in show_prefixes {
        if let Some(rest) = q_lower.strip_prefix(prefix) {
            let arg = q_original[q_original.len() - rest.len()..].trim();
            if !arg.is_empty() {
                info!(
                    "Agent router: pre-routed to TASK_SHOW (keyword): {}",
                    crate::logging::ellipse(arg, 40)
                );
                return Some(format!("TASK_SHOW: {arg}"));
            }
        }
    }

    try_pre_route_task_create(q_lower, q_original)
}

/// "create a task for coder to …" / "create a task about …" → TASK_CREATE.
fn try_pre_route_task_create(q_lower: &str, q_original: &str) -> Option<String> {
    if q_lower.len() > 600 {
        return None;
    }
    if q_lower.contains("and then ")
        || q_lower.contains("after that ")
        || q_lower.contains("schedule ")
        || q_lower.contains("http")
        || q_lower.contains("skill:")
        || q_lower.contains("cursor_agent:")
    {
        return None;
    }

    let (topic, content_from_lower_idx) = if let Some(rest) = q_lower.strip_prefix("create a task for ")
    {
        let rest = rest.trim();
        if rest.is_empty() {
            return None;
        }
        // "coder to improve …" → topic=coder, content=full original
        let topic = rest
            .split_whitespace()
            .next()
            .unwrap_or("task")
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .to_string();
        if topic.is_empty() {
            return None;
        }
        (topic, q_lower.find("create a task for ").unwrap_or(0))
    } else if let Some(pos) = q_lower.find("create a task about ") {
        let after = &q_lower[pos + "create a task about ".len()..];
        if after.trim().is_empty() {
            return None;
        }
        ("task".to_string(), pos)
    } else if let Some(pos) = q_lower.find("create a task:") {
        let after = &q_lower[pos + "create a task:".len()..];
        if after.trim().is_empty() {
            return None;
        }
        ("task".to_string(), pos)
    } else if let Some(rest) = q_lower.strip_prefix("create a task ") {
        let rest = rest.trim();
        if rest.is_empty() || rest == "for" || rest == "about" {
            return None;
        }
        ("task".to_string(), 0)
    } else {
        return None;
    };

    let content = q_original.trim();
    if content.chars().count() < 12 {
        return None;
    }
    let _ = content_from_lower_idx; // content is the full user ask (keeps context)
    let topic = crate::commands::curated_memory::slugify_note_id(&topic);
    info!(
        "Agent router: pre-routed to TASK_CREATE (keyword): topic={} content={}",
        topic,
        crate::logging::ellipse(content, 80)
    );
    Some(format!("TASK_CREATE: {topic} 1 {content}"))
}

/// "list models", "what models", "ollama models" → OLLAMA_API: list_models.
/// "pull model <name>" → OLLAMA_API: pull <name>.
/// "unload model <name>" → OLLAMA_API: unload <name>.
fn try_pre_route_ollama_api(q_lower: &str) -> Option<String> {
    let is_list_models = q_lower == "list models"
        || q_lower == "list ollama models"
        || q_lower == "show models"
        || q_lower == "show ollama models"
        || q_lower == "ollama models"
        || q_lower == "what models are available"
        || q_lower == "what models are installed"
        || q_lower == "which models are available"
        || q_lower == "which models are installed"
        || q_lower == "what models do i have"
        || q_lower == "installed models"
        || q_lower == "available models";

    if is_list_models {
        info!("Agent router: pre-routed to OLLAMA_API: list_models (keyword)");
        return Some("OLLAMA_API: list_models".to_string());
    }

    // "pull model <name>" / "pull <name>"
    let pull_prefixes: &[&str] = &["pull model ", "pull ollama model ", "ollama pull "];
    for prefix in pull_prefixes {
        if let Some(rest) = q_lower.strip_prefix(prefix) {
            let model = rest.trim();
            if !model.is_empty() {
                info!(
                    "Agent router: pre-routed to OLLAMA_API: pull (keyword): {}",
                    model
                );
                return Some(format!("OLLAMA_API: pull {model}"));
            }
        }
    }

    // "unload model <name>" / "unload <name>"
    let unload_prefixes: &[&str] = &["unload model ", "unload ollama model ", "ollama unload "];
    for prefix in unload_prefixes {
        if let Some(rest) = q_lower.strip_prefix(prefix) {
            let model = rest.trim();
            if !model.is_empty() {
                info!(
                    "Agent router: pre-routed to OLLAMA_API: unload (keyword): {}",
                    model
                );
                return Some(format!("OLLAMA_API: unload {model}"));
            }
        }
    }

    // "running models" / "loaded models"
    if q_lower == "running models"
        || q_lower == "loaded models"
        || q_lower == "what models are running"
        || q_lower == "which models are running"
        || q_lower == "which models are loaded"
    {
        info!("Agent router: pre-routed to OLLAMA_API: running (keyword)");
        return Some("OLLAMA_API: running".to_string());
    }

    None
}

/// Discord API pre-routing: explicit prefix and common Discord query patterns.
///
/// Only triggers when a Discord bot token is configured. Routes to:
/// - `DISCORD_API: <path>` for explicit prefixes
/// - `DISCORD_API: GET /users/@me/guilds` for "list servers" / "show servers"
/// - `AGENT: discord-expert <task>` for queries that need multi-step Discord flows
///   (e.g. "list channels", "list members" — require guild context the expert discovers)
fn try_pre_route_discord_api(question: &str) -> Option<String> {
    crate::discord::get_discord_token()?;
    let result = match_discord_api_pattern(question)?;
    info!(
        "Agent router: pre-routed to {} (Discord)",
        crate::logging::ellipse(&result, 80)
    );
    Some(result)
}

/// Pure pattern matching for Discord API pre-routing (no token check).
/// Returns the recommendation string or None.
fn match_discord_api_pattern(question: &str) -> Option<String> {
    let q = question.trim();
    let q_lower = q.to_lowercase();

    // Skip multi-step / compound requests that need LLM planning.
    if q_lower.contains("and then ")
        || q_lower.contains("after that ")
        || q_lower.contains("screenshot")
    {
        return None;
    }

    // Explicit DISCORD_API: prefix always wins.
    if let Some(arg) = extract_last_prefixed_argument(q, "DISCORD_API:") {
        let arg = arg.trim().to_string();
        if !arg.is_empty() {
            return Some(format!("DISCORD_API: {arg}"));
        }
    }

    // "list servers" / "show servers" / "my servers" → direct API call (no guild context needed)
    if q_lower == "list servers"
        || q_lower == "show servers"
        || q_lower == "my servers"
        || q_lower == "list my servers"
        || q_lower == "show my servers"
        || q_lower == "list discord servers"
        || q_lower == "show discord servers"
        || q_lower == "what servers am i in"
        || q_lower == "which servers am i in"
        || q_lower == "discord servers"
    {
        return Some("DISCORD_API: GET /users/@me/guilds".to_string());
    }

    // Queries that need guild context → delegate to discord-expert agent.
    // "list channels" / "show channels"
    let is_channel_query = q_lower == "list channels"
        || q_lower == "show channels"
        || q_lower == "list discord channels"
        || q_lower == "show discord channels"
        || q_lower == "what channels are there"
        || q_lower.starts_with("list channels in ")
        || q_lower.starts_with("show channels in ");

    if is_channel_query {
        return Some(format!("AGENT: discord-expert {q}"));
    }

    // "list members" / "show members" / "who's in the server"
    let is_member_query = q_lower == "list members"
        || q_lower == "show members"
        || q_lower == "list server members"
        || q_lower == "show server members"
        || q_lower == "list discord members"
        || q_lower.starts_with("who is in ")
        || q_lower.starts_with("who's in ")
        || q_lower.starts_with("list members in ")
        || q_lower.starts_with("show members in ");

    if is_member_query {
        return Some(format!("AGENT: discord-expert {q}"));
    }

    None
}

/// Ticket / time-entries patterns → `REDMINE_API: GET /...`.
fn try_pre_route_redmine(
    question: &str,
    request_for_verification: &str,
    is_verification_retry: bool,
) -> Option<String> {
    if !crate::redmine::is_configured() {
        return None;
    }
    let q = question.trim();
    let redmine_request =
        redmine_request_for_routing(q, request_for_verification, is_verification_retry);
    let redmine_request_lower = redmine_request.to_lowercase();

    if is_redmine_time_entries_request(redmine_request) {
        let (from, to) = redmine_time_entries_range(redmine_request);
        let rec = format!(
            "REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100",
            from, to
        );
        info!(
            "Agent router: pre-routed to REDMINE_API for time entries ({}..{})",
            from, to
        );
        return Some(rec);
    }

    let ticket_id = extract_ticket_id(&redmine_request_lower);
    let wants_update = redmine_request_lower.contains("update")
        || redmine_request_lower.contains("add comment")
        || redmine_request_lower.contains("with the next steps")
        || redmine_request_lower.contains("post a comment")
        || redmine_request_lower.contains("write ")
        || redmine_request_lower.contains("put ");
    ticket_id
        .filter(|_| {
            redmine_request_lower.contains("ticket")
                || redmine_request_lower.contains("issue")
                || redmine_request_lower.contains("redmine")
        })
        .filter(|_| !wants_update)
        .map(|id| {
            let rec = format!(
                "REDMINE_API: GET /issues/{}.json?include=journals,attachments",
                id
            );
            info!("Agent router: pre-routed to REDMINE_API for ticket #{}", id);
            rec
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_url_explicit_prefix() {
        let r = try_pre_route_fetch_url("FETCH_URL: https://example.com");
        assert_eq!(r, Some("FETCH_URL: https://example.com".to_string()));
    }

    #[test]
    fn fetch_url_explicit_prefix_case_insensitive() {
        let r = try_pre_route_fetch_url("fetch_url: https://example.com/page");
        assert_eq!(r, Some("FETCH_URL: https://example.com/page".to_string()));
    }

    #[test]
    fn fetch_url_keyword_fetch() {
        let r = try_pre_route_fetch_url("fetch https://example.com");
        assert_eq!(r, Some("FETCH_URL: https://example.com".to_string()));
    }

    #[test]
    fn fetch_url_keyword_get_the_page() {
        let r = try_pre_route_fetch_url("get the page at https://docs.rs/tokio");
        assert_eq!(r, Some("FETCH_URL: https://docs.rs/tokio".to_string()));
    }

    #[test]
    fn fetch_url_keyword_get_the_content() {
        let r = try_pre_route_fetch_url("get the content of https://example.com/api");
        assert_eq!(r, Some("FETCH_URL: https://example.com/api".to_string()));
    }

    #[test]
    fn fetch_url_keyword_read_the_page() {
        let r = try_pre_route_fetch_url("read the page https://example.com");
        assert_eq!(r, Some("FETCH_URL: https://example.com".to_string()));
    }

    #[test]
    fn fetch_url_keyword_summarize() {
        let r = try_pre_route_fetch_url("summarize the page at https://blog.example.com/post");
        assert_eq!(
            r,
            Some("FETCH_URL: https://blog.example.com/post".to_string())
        );
    }

    #[test]
    fn fetch_url_keyword_summarise_british() {
        let r = try_pre_route_fetch_url("summarise this url https://example.com");
        assert_eq!(r, Some("FETCH_URL: https://example.com".to_string()));
    }

    #[test]
    fn fetch_url_keyword_whats_on() {
        let r = try_pre_route_fetch_url("what's on https://news.example.com?");
        assert_eq!(r, Some("FETCH_URL: https://news.example.com".to_string()));
    }

    #[test]
    fn fetch_url_no_url_returns_none() {
        assert_eq!(try_pre_route_fetch_url("fetch some data"), None);
    }

    #[test]
    fn fetch_url_no_intent_returns_none() {
        assert_eq!(
            try_pre_route_fetch_url("tell me about https://example.com"),
            None
        );
    }

    #[test]
    fn fetch_url_screenshot_skipped() {
        assert_eq!(
            try_pre_route_fetch_url("take a screenshot of https://example.com"),
            None
        );
    }

    #[test]
    fn fetch_url_navigate_skipped() {
        assert_eq!(
            try_pre_route_fetch_url("navigate to https://example.com and find the price"),
            None
        );
    }

    #[test]
    fn fetch_url_click_skipped() {
        assert_eq!(
            try_pre_route_fetch_url("fetch https://example.com and click the button"),
            None
        );
    }

    #[test]
    fn fetch_url_open_in_browser_skipped() {
        assert_eq!(
            try_pre_route_fetch_url("open https://example.com in the browser"),
            None
        );
    }

    #[test]
    fn fetch_url_strips_trailing_punctuation() {
        let r = try_pre_route_fetch_url("fetch https://example.com.");
        assert_eq!(r, Some("FETCH_URL: https://example.com".to_string()));
    }

    #[test]
    fn fetch_url_http_scheme() {
        let r = try_pre_route_fetch_url("fetch http://localhost:8080/api");
        assert_eq!(r, Some("FETCH_URL: http://localhost:8080/api".to_string()));
    }

    #[test]
    fn google_serp_rewrites_to_search() {
        let q = "Extract interesting people from https://www.google.com/search?q=site%3Afacebook.com+ceo";
        assert_eq!(
            google_serp_search_query(
                "https://www.google.com/search?q=site%3Afacebook.com+ceo"
            )
            .as_deref(),
            Some("site:facebook.com ceo")
        );
        let r = try_pre_route_google_serp_as_search(q);
        // When neither Brave nor Perplexity is configured in the test env, rewrite is skipped.
        if crate::commands::brave::get_brave_api_key().is_some()
            || crate::commands::perplexity::is_perplexity_configured().unwrap_or(false)
        {
            let rec = r.expect("expected search rewrite");
            assert!(
                rec.starts_with("BRAVE_SEARCH: ") || rec.starts_with("PERPLEXITY_SEARCH: "),
                "{rec}"
            );
            assert!(rec.contains("facebook"), "{rec}");
        }
    }

    #[test]
    fn lighthouse_pagespeed_pre_route_bare_domain() {
        let r = try_pre_route_lighthouse_pagespeed(
            "Review satisfecho.de using lighthouse from chrome especially and only focus on SEO",
        );
        let rec = r.expect("expected BROWSER_SCREENSHOT");
        assert!(rec.starts_with("BROWSER_SCREENSHOT: https://pagespeed.web.dev/analysis?url="), "{rec}");
        assert!(rec.contains("satisfecho.de"), "{rec}");
    }

    #[test]
    fn lighthouse_pagespeed_requires_signal() {
        assert_eq!(
            try_pre_route_lighthouse_pagespeed("Review satisfecho.de homepage copy"),
            None
        );
    }

    // --- extract_search_query tests ---

    #[test]
    fn search_query_search_for() {
        let r = extract_search_query(
            "search for rust async patterns",
            "search for rust async patterns",
        );
        assert_eq!(r, Some(("rust async patterns".to_string(), false)));
    }

    #[test]
    fn search_query_search_plain() {
        let r = extract_search_query("search latest rust release", "search latest rust release");
        assert_eq!(r, Some(("latest rust release".to_string(), false)));
    }

    #[test]
    fn search_query_google() {
        let r = extract_search_query(
            "google best restaurants in berlin",
            "google best restaurants in berlin",
        );
        assert_eq!(r, Some(("best restaurants in berlin".to_string(), false)));
    }

    #[test]
    fn search_query_look_up() {
        let r = extract_search_query(
            "look up tauri v2 documentation",
            "look up tauri v2 documentation",
        );
        assert_eq!(r, Some(("tauri v2 documentation".to_string(), false)));
    }

    #[test]
    fn search_query_lookup_no_space() {
        let r = extract_search_query("lookup tokio runtime", "lookup tokio runtime");
        assert_eq!(r, Some(("tokio runtime".to_string(), false)));
    }

    #[test]
    fn search_query_web_search() {
        let r = extract_search_query(
            "web search for climate change 2026",
            "web search for climate change 2026",
        );
        assert_eq!(r, Some(("climate change 2026".to_string(), false)));
    }

    #[test]
    fn search_query_web_search_no_for() {
        let r = extract_search_query("web search rust tauri", "web search rust tauri");
        assert_eq!(r, Some(("rust tauri".to_string(), false)));
    }

    #[test]
    fn search_query_search_the_web_for() {
        let r = extract_search_query(
            "search the web for apple silicon m4",
            "search the web for apple silicon m4",
        );
        assert_eq!(r, Some(("apple silicon m4".to_string(), false)));
    }

    #[test]
    fn search_query_search_the_internet_for() {
        let r = extract_search_query(
            "search the internet for ollama models",
            "search the internet for ollama models",
        );
        assert_eq!(r, Some(("ollama models".to_string(), false)));
    }

    #[test]
    fn search_query_search_online_for() {
        let r = extract_search_query(
            "search online for weather berlin",
            "search online for weather berlin",
        );
        assert_eq!(r, Some(("weather berlin".to_string(), false)));
    }

    #[test]
    fn search_query_research_is_research() {
        let r = extract_search_query(
            "research quantum computing advances",
            "research quantum computing advances",
        );
        assert_eq!(r, Some(("quantum computing advances".to_string(), true)));
    }

    #[test]
    fn research_using_perplexity_pre_route() {
        let q = "Research using perplexity: Florian Fischer delivery hero problem";
        let r = try_pre_route_web_search(q);
        if crate::commands::perplexity::is_perplexity_configured().unwrap_or(false) {
            let rec = r.expect("expected PERPLEXITY_SEARCH when configured");
            assert!(rec.starts_with("PERPLEXITY_SEARCH: "), "{rec}");
            assert!(rec.contains("Florian Fischer"), "{rec}");
        }
    }

    #[test]
    fn search_query_with_please_prefix() {
        let r = extract_search_query(
            "please search for openai news",
            "please search for openai news",
        );
        assert_eq!(r, Some(("openai news".to_string(), false)));
    }

    #[test]
    fn bare_news_query_defaults_headline_search() {
        assert_eq!(
            bare_news_search_query("any news?", "Any news?"),
            "top world and technology headlines today"
        );
        assert_eq!(
            bare_news_search_query("todays headlines", "todays headlines"),
            "top world and technology headlines today"
        );
        assert_eq!(
            bare_news_search_query(
                "what's the latest news about barcelona?",
                "What's the latest news about Barcelona?"
            ),
            "What's the latest news about Barcelona"
        );
    }

    #[test]
    fn topic_dump_search_query_accepts_short_lists() {
        assert_eq!(
            topic_dump_search_query("IT, AI, Stocks, BTC, nerd stuff"),
            Some("IT, AI, Stocks, BTC, nerd stuff".to_string())
        );
        assert_eq!(topic_dump_search_query("just one topic"), None);
        assert_eq!(topic_dump_search_query("search for IT, AI, Stocks"), None);
    }

    #[test]
    fn flight_search_query_accepts_clear_routes() {
        assert_eq!(
            flight_search_query("Flights from Atlanta to Monterey to mochis"),
            Some("Flights from Atlanta to Monterey to mochis".to_string())
        );
        assert_eq!(
            flight_search_query("What about Viva Aerobus flights from MTY to LMM?"),
            Some("What about Viva Aerobus flights from MTY to LMM".to_string())
        );
        assert_eq!(
            flight_search_query(
                "Review IBM techxchange dates in Atlanta. I want to be in Atlanta two days before"
            ),
            None
        );
        assert_eq!(flight_search_query("book flights to Atlanta tomorrow"), None);
    }

    #[test]
    fn event_dates_search_query_accepts_conference_reviews() {
        let q =
            "Review IBM techxchange dates in Atlanta. I want to be in Atlanta two days before";
        assert_eq!(event_dates_search_query(q), Some(q.to_string()));
        assert_eq!(
            event_dates_search_query("Check WWDC conference dates for 2026"),
            Some("Check WWDC conference dates for 2026".to_string())
        );
        assert_eq!(
            event_dates_search_query("Flights from Atlanta to Monterey to mochis"),
            None
        );
        assert_eq!(
            event_dates_search_query("save the techxchange dates in memory"),
            None
        );
    }

    #[test]
    fn travel_plan_search_query_accepts_multi_city() {
        let q =
            "We are going to travel to Atlanta in October and want to go to Los mochis afterward";
        assert_eq!(travel_plan_search_query(q), Some(q.to_string()));
        assert_eq!(
            travel_plan_search_query(
                "I want to be back in Barcelona around 14 of November. LMM - MEX - BCN"
            ),
            None
        );
        assert_eq!(
            travel_plan_search_query("We are going to travel to Atlanta in October"),
            None
        );
    }

    #[test]
    fn airport_hop_and_bare_research_pre_route() {
        assert_eq!(
            airport_hop_search_query("I would like ATL to Monterrey to LMM"),
            Some("flights I would like ATL to Monterrey to LMM".to_string())
        );
        assert_eq!(
            airport_hop_search_query("We still have to go BCN - ATL. You missed that leg"),
            None
        );
        assert_eq!(
            bare_research_topic_query("Florian Fischer delivery hero problem"),
            Some("Florian Fischer delivery hero problem".to_string())
        );
        assert_eq!(
            bare_research_topic_query("please search for Florian Fischer"),
            None
        );
    }

    #[test]
    fn search_query_with_can_you_prefix() {
        let r = extract_search_query(
            "can you search for tauri plugins",
            "can you search for tauri plugins",
        );
        assert_eq!(r, Some(("tauri plugins".to_string(), false)));
    }

    #[test]
    fn search_query_with_could_you_prefix() {
        let r = extract_search_query(
            "could you google macos 15 features",
            "could you google macos 15 features",
        );
        assert_eq!(r, Some(("macos 15 features".to_string(), false)));
    }

    #[test]
    fn search_query_strips_trailing_question_mark() {
        let r = extract_search_query("search for what is serde?", "search for what is serde?");
        assert_eq!(r, Some(("what is serde".to_string(), false)));
    }

    #[test]
    fn search_query_empty_after_keyword_returns_none() {
        assert_eq!(extract_search_query("search for ", "search for "), None);
    }

    #[test]
    fn search_query_no_match_returns_none() {
        assert_eq!(
            extract_search_query("tell me about rust", "tell me about rust"),
            None
        );
    }

    #[test]
    fn search_query_embedded_search_not_at_start() {
        assert_eq!(
            extract_search_query(
                "i want to search for something and then send it",
                "i want to search for something and then send it",
            ),
            None
        );
    }

    #[test]
    fn search_query_pls_prefix() {
        let r = extract_search_query("pls search for new iphone", "pls search for new iphone");
        assert_eq!(r, Some(("new iphone".to_string(), false)));
    }

    #[test]
    fn search_query_longer_pattern_preferred() {
        let r = extract_search_query("search the web for tauri v2", "search the web for tauri v2");
        assert_eq!(r, Some(("tauri v2".to_string(), false)));
    }

    #[test]
    fn search_query_case_preserved_in_output() {
        let r = extract_search_query(
            "search for Rust Async Patterns",
            "search for Rust Async Patterns",
        );
        assert_eq!(r, Some(("Rust Async Patterns".to_string(), false)));
    }

    // --- LIST_SCHEDULES pre-route tests ---

    #[test]
    fn list_schedules_exact() {
        assert_eq!(
            try_pre_route_list_schedules("list schedules"),
            Some("LIST_SCHEDULES:".to_string())
        );
    }

    #[test]
    fn list_schedules_show() {
        assert_eq!(
            try_pre_route_list_schedules("show schedules"),
            Some("LIST_SCHEDULES:".to_string())
        );
    }

    #[test]
    fn list_schedules_whats_scheduled() {
        assert_eq!(
            try_pre_route_list_schedules("what's scheduled"),
            Some("LIST_SCHEDULES:".to_string())
        );
    }

    #[test]
    fn list_schedules_what_is_scheduled() {
        assert_eq!(
            try_pre_route_list_schedules("what is scheduled"),
            Some("LIST_SCHEDULES:".to_string())
        );
    }

    #[test]
    fn list_schedules_my_schedules() {
        assert_eq!(
            try_pre_route_list_schedules("my schedules"),
            Some("LIST_SCHEDULES:".to_string())
        );
    }

    #[test]
    fn list_schedules_bare_word() {
        assert_eq!(
            try_pre_route_list_schedules("schedules"),
            Some("LIST_SCHEDULES:".to_string())
        );
    }

    #[test]
    fn list_schedules_no_match() {
        assert_eq!(
            try_pre_route_list_schedules("schedule a task for tomorrow"),
            None
        );
    }

    // --- TASK_LIST / TASK_SHOW pre-route tests ---

    #[test]
    fn task_list_exact() {
        assert_eq!(
            try_pre_route_task_commands("list tasks", "list tasks"),
            Some("TASK_LIST: ".to_string())
        );
    }

    #[test]
    fn task_list_show_tasks() {
        assert_eq!(
            try_pre_route_task_commands("show tasks", "show tasks"),
            Some("TASK_LIST: ".to_string())
        );
    }

    #[test]
    fn task_list_bare_tasks() {
        assert_eq!(
            try_pre_route_task_commands("tasks", "tasks"),
            Some("TASK_LIST: ".to_string())
        );
    }

    #[test]
    fn task_list_all() {
        assert_eq!(
            try_pre_route_task_commands("all tasks", "all tasks"),
            Some("TASK_LIST: all".to_string())
        );
    }

    #[test]
    fn task_list_list_all() {
        assert_eq!(
            try_pre_route_task_commands("list all tasks", "list all tasks"),
            Some("TASK_LIST: all".to_string())
        );
    }

    #[test]
    fn task_list_open_tasks() {
        assert_eq!(
            try_pre_route_task_commands("open tasks", "open tasks"),
            Some("TASK_LIST: ".to_string())
        );
    }

    #[test]
    fn task_show_by_id() {
        assert_eq!(
            try_pre_route_task_commands("show task 42", "show task 42"),
            Some("TASK_SHOW: 42".to_string())
        );
    }

    #[test]
    fn task_show_by_name() {
        assert_eq!(
            try_pre_route_task_commands("show task research", "show task research"),
            Some("TASK_SHOW: research".to_string())
        );
    }

    #[test]
    fn task_show_me_task() {
        assert_eq!(
            try_pre_route_task_commands("show me task 7", "show me task 7"),
            Some("TASK_SHOW: 7".to_string())
        );
    }

    #[test]
    fn task_create_pre_route() {
        let r = try_pre_route_task_commands(
            "create a task for coder to improve your knowledge saving from this discussion.",
            "Create a task for coder to improve your knowledge saving from this discussion.",
        );
        let rec = r.expect("expected TASK_CREATE");
        assert!(rec.starts_with("TASK_CREATE: coder 1 "), "{rec}");
        assert!(rec.contains("knowledge saving"), "{rec}");

        let r2 = try_pre_route_task_commands(
            "create a task about testing",
            "create a task about testing",
        );
        let rec2 = r2.expect("expected TASK_CREATE about");
        assert!(rec2.starts_with("TASK_CREATE: task 1 "), "{rec2}");
    }

    #[test]
    fn task_create_skipped_for_compound() {
        assert_eq!(
            try_pre_route_task_commands(
                "create a task about testing and then schedule it",
                "create a task about testing and then schedule it"
            ),
            None
        );
    }

    // --- OLLAMA_API pre-route tests ---

    #[test]
    fn ollama_list_models() {
        assert_eq!(
            try_pre_route_ollama_api("list models"),
            Some("OLLAMA_API: list_models".to_string())
        );
    }

    #[test]
    fn ollama_show_models() {
        assert_eq!(
            try_pre_route_ollama_api("show models"),
            Some("OLLAMA_API: list_models".to_string())
        );
    }

    #[test]
    fn ollama_models_installed() {
        assert_eq!(
            try_pre_route_ollama_api("what models are installed"),
            Some("OLLAMA_API: list_models".to_string())
        );
    }

    #[test]
    fn ollama_available_models() {
        assert_eq!(
            try_pre_route_ollama_api("available models"),
            Some("OLLAMA_API: list_models".to_string())
        );
    }

    #[test]
    fn ollama_which_models() {
        assert_eq!(
            try_pre_route_ollama_api("which models are available"),
            Some("OLLAMA_API: list_models".to_string())
        );
    }

    #[test]
    fn ollama_pull_model() {
        assert_eq!(
            try_pre_route_ollama_api("pull model llama3"),
            Some("OLLAMA_API: pull llama3".to_string())
        );
    }

    #[test]
    fn ollama_pull_model_with_tag() {
        assert_eq!(
            try_pre_route_ollama_api("pull model qwen3:latest"),
            Some("OLLAMA_API: pull qwen3:latest".to_string())
        );
    }

    #[test]
    fn ollama_unload_model() {
        assert_eq!(
            try_pre_route_ollama_api("unload model llama3"),
            Some("OLLAMA_API: unload llama3".to_string())
        );
    }

    #[test]
    fn ollama_running_models() {
        assert_eq!(
            try_pre_route_ollama_api("running models"),
            Some("OLLAMA_API: running".to_string())
        );
    }

    #[test]
    fn ollama_what_running() {
        assert_eq!(
            try_pre_route_ollama_api("what models are running"),
            Some("OLLAMA_API: running".to_string())
        );
    }

    #[test]
    fn ollama_no_match() {
        assert_eq!(try_pre_route_ollama_api("tell me about llama3"), None);
    }

    // --- Management commands compound / skip tests ---

    #[test]
    fn management_multi_step_skipped() {
        assert_eq!(
            try_pre_route_management_commands("list schedules and then remove the first one"),
            None
        );
    }

    #[test]
    fn management_explicit_list_schedules_prefix() {
        let r = try_pre_route_management_commands("LIST_SCHEDULES:");
        assert_eq!(r, Some("LIST_SCHEDULES:".to_string()));
    }

    #[test]
    fn management_explicit_task_list_prefix() {
        let r = try_pre_route_management_commands("TASK_LIST: all");
        assert_eq!(r, Some("TASK_LIST: all".to_string()));
    }

    #[test]
    fn management_explicit_task_show_prefix() {
        let r = try_pre_route_management_commands("TASK_SHOW: 42");
        assert_eq!(r, Some("TASK_SHOW: 42".to_string()));
    }

    #[test]
    fn management_explicit_ollama_api_prefix() {
        let r = try_pre_route_management_commands("OLLAMA_API: list_models");
        assert_eq!(r, Some("OLLAMA_API: list_models".to_string()));
    }

    // --- DISCORD_API pre-route tests (use match_discord_api_pattern to bypass token check) ---

    #[test]
    fn discord_explicit_prefix() {
        assert_eq!(
            match_discord_api_pattern("DISCORD_API: GET /users/@me/guilds"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_explicit_prefix_post() {
        assert_eq!(
            match_discord_api_pattern(
                "DISCORD_API: POST /channels/123/messages {\"content\":\"hi\"}"
            ),
            Some("DISCORD_API: POST /channels/123/messages {\"content\":\"hi\"}".to_string())
        );
    }

    #[test]
    fn discord_list_servers() {
        assert_eq!(
            match_discord_api_pattern("list servers"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_show_servers() {
        assert_eq!(
            match_discord_api_pattern("show servers"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_my_servers() {
        assert_eq!(
            match_discord_api_pattern("my servers"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_list_my_servers() {
        assert_eq!(
            match_discord_api_pattern("list my servers"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_what_servers() {
        assert_eq!(
            match_discord_api_pattern("what servers am i in"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_which_servers() {
        assert_eq!(
            match_discord_api_pattern("which servers am i in"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_discord_servers() {
        assert_eq!(
            match_discord_api_pattern("discord servers"),
            Some("DISCORD_API: GET /users/@me/guilds".to_string())
        );
    }

    #[test]
    fn discord_list_channels_delegates() {
        assert_eq!(
            match_discord_api_pattern("list channels"),
            Some("AGENT: discord-expert list channels".to_string())
        );
    }

    #[test]
    fn discord_show_channels_delegates() {
        assert_eq!(
            match_discord_api_pattern("show channels"),
            Some("AGENT: discord-expert show channels".to_string())
        );
    }

    #[test]
    fn discord_list_channels_in_server() {
        assert_eq!(
            match_discord_api_pattern("list channels in my server"),
            Some("AGENT: discord-expert list channels in my server".to_string())
        );
    }

    #[test]
    fn discord_what_channels() {
        assert_eq!(
            match_discord_api_pattern("what channels are there"),
            Some("AGENT: discord-expert what channels are there".to_string())
        );
    }

    #[test]
    fn discord_list_members_delegates() {
        assert_eq!(
            match_discord_api_pattern("list members"),
            Some("AGENT: discord-expert list members".to_string())
        );
    }

    #[test]
    fn discord_show_server_members() {
        assert_eq!(
            match_discord_api_pattern("show server members"),
            Some("AGENT: discord-expert show server members".to_string())
        );
    }

    #[test]
    fn discord_whos_in_server() {
        assert_eq!(
            match_discord_api_pattern("who's in the server"),
            Some("AGENT: discord-expert who's in the server".to_string())
        );
    }

    #[test]
    fn discord_who_is_in() {
        assert_eq!(
            match_discord_api_pattern("who is in the gaming server"),
            Some("AGENT: discord-expert who is in the gaming server".to_string())
        );
    }

    #[test]
    fn discord_list_members_in() {
        assert_eq!(
            match_discord_api_pattern("list members in my server"),
            Some("AGENT: discord-expert list members in my server".to_string())
        );
    }

    #[test]
    fn discord_multi_step_skipped() {
        assert_eq!(
            match_discord_api_pattern("list servers and then send a message"),
            None
        );
    }

    #[test]
    fn discord_screenshot_skipped() {
        assert_eq!(
            match_discord_api_pattern("screenshot the discord servers page"),
            None
        );
    }

    #[test]
    fn discord_no_match() {
        assert_eq!(match_discord_api_pattern("tell me about discord"), None);
    }

    #[test]
    fn discord_empty_prefix() {
        assert_eq!(match_discord_api_pattern("DISCORD_API:"), None);
    }
}
