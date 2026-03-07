//! Redmine REST API client for the agent router.
//!
//! Provides GET, POST (create issue), and PUT (update issue / add notes) access to a Redmine instance.
//! Auth via API key (`X-Redmine-API-Key` header). Config read from env or `.config.env`:
//!   REDMINE_URL=https://redmine.example.com
//!   REDMINE_API_KEY=<key>
//!
//! When building agent descriptions, we fetch projects, trackers, statuses, and priorities
//! and inject them as context so the LLM can resolve names (e.g. "Create in AMVARA") to IDs.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use serde::Deserialize;
use tracing::{debug, info, warn};
use url::Url;

const MAX_RESPONSE_CHARS: usize = 12_000;
const TIMEOUT_SECS: u64 = 20;
const CONTEXT_CACHE_TTL_SECS: u64 = 300; // 5 minutes

/// Cache for Redmine create context (projects, trackers, statuses, priorities).
static REDMINE_CONTEXT_CACHE: Mutex<Option<(String, Instant)>> = Mutex::new(None);

#[derive(Debug, Deserialize)]
struct RedmineTimeEntriesResponse {
    #[serde(default)]
    time_entries: Vec<RedmineTimeEntry>,
    total_count: Option<usize>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
struct RedmineNamedRef {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct RedmineIssueRef {
    id: u64,
    #[serde(default)]
    subject: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct RedmineTimeEntry {
    id: u64,
    #[serde(default)]
    hours: f64,
    #[serde(default)]
    spent_on: Option<String>,
    #[serde(default)]
    comments: Option<String>,
    #[serde(default)]
    project: Option<RedmineNamedRef>,
    #[serde(default)]
    issue: Option<RedmineIssueRef>,
    #[serde(default)]
    user: Option<RedmineNamedRef>,
    #[serde(default)]
    activity: Option<RedmineNamedRef>,
}

#[derive(Debug, Default)]
struct TicketAggregate {
    subject: String,
    project: String,
    hours: f64,
    entries: usize,
    users: BTreeSet<String>,
    activities: BTreeSet<String>,
}

#[derive(Debug, Default)]
struct ProjectOnlyAggregate {
    label: String,
    hours: f64,
    entries: usize,
    users: BTreeSet<String>,
    activities: BTreeSet<String>,
}

/// Read a key from `.config.env` style files (KEY=value, one per line).
fn read_key_from_file(path: &Path, key_names: &[&str]) -> Option<String> {
    // Do not log file content or path; file may contain secrets.
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let t = line.trim();
        for &name in key_names {
            if let Some(val) = t.strip_prefix(name).and_then(|r| r.strip_prefix('=')) {
                let val = val.trim().trim_matches('\'').trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }
    None
}

/// Search env vars, then `.config.env` / `.env.config` in cwd, src-tauri, ~/.mac-stats.
fn read_config(env_name: &str, file_keys: &[&str]) -> Option<String> {
    if let Ok(v) = std::env::var(env_name) {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    let candidates: Vec<std::path::PathBuf> = [
        std::env::current_dir().ok().map(|d| d.join(".config.env")),
        std::env::current_dir().ok().map(|d| d.join(".env.config")),
        std::env::current_dir()
            .ok()
            .map(|d| d.join("..").join(".env.config")),
        std::env::var("HOME")
            .ok()
            .map(|h| Path::new(&h).join(".mac-stats").join(".config.env")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if let Some(val) = read_key_from_file(path, file_keys) {
            return Some(val);
        }
    }
    None
}

pub fn get_redmine_url() -> Option<String> {
    read_config("REDMINE_URL", &["REDMINE_URL", "REDMINE-URL"])
}

pub fn get_redmine_api_key() -> Option<String> {
    read_config("REDMINE_API_KEY", &["REDMINE_API_KEY", "REDMINE-API-KEY"])
}

/// Whether Redmine is configured (URL + API key both present).
pub fn is_configured() -> bool {
    get_redmine_url().is_some() && get_redmine_api_key().is_some()
}

/// PUT paths that are allowed. Currently: updating issues (adding notes).
fn is_allowed_put_path(path: &str) -> bool {
    let path = path.trim().trim_start_matches('/');
    let path_no_query = path.split('?').next().unwrap_or(path);
    if let Some(rest) = path_no_query.strip_prefix("issues/") {
        return rest.ends_with(".json") || rest.ends_with(".xml");
    }
    false
}

/// POST paths that are allowed. Currently: creating issues only.
fn is_allowed_post_path(path: &str) -> bool {
    let path = path.trim().trim_start_matches('/');
    let path_no_query = path.split('?').next().unwrap_or(path);
    path_no_query == "issues.json" || path_no_query == "issues.xml"
}

/// Internal: perform a GET request to Redmine, return response body or error.
async fn redmine_get(path: &str) -> Result<String, String> {
    let base_url = get_redmine_url().ok_or_else(|| "Redmine not configured".to_string())?;
    let api_key = get_redmine_api_key().ok_or_else(|| "Redmine API key missing".to_string())?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    redmine_get_with_client(&client, &base_url, &api_key, path).await
}

async fn redmine_get_with_client(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    path: &str,
) -> Result<String, String> {
    let path = path.trim_start_matches('/');
    let url = format!("{}/{}", base_url.trim_end_matches('/'), path);
    // Do not log request/response headers or bodies that may contain credentials.
    let resp = client
        .get(&url)
        .header("X-Redmine-API-Key", api_key)
        .header(
            "User-Agent",
            format!("mac-stats/{}", crate::config::Config::version()),
        )
        .send()
        .await
        .map_err(|e| format!("Redmine GET failed: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Redmine {} {}: {}",
            status,
            path,
            crate::logging::ellipse(&body, 200)
        ));
    }
    resp.text()
        .await
        .map_err(|e| format!("Redmine response: {}", e))
}

fn is_time_entries_path(path: &str) -> bool {
    let path = path.trim().trim_start_matches('/');
    path == "time_entries.json"
        || path.starts_with("time_entries.json?")
        || path == "time_entries.xml"
        || path.starts_with("time_entries.xml?")
}

fn with_query_params(path: &str, extra: &[(&str, &str)]) -> Result<String, String> {
    let joined = Url::parse("https://example.invalid")
        .and_then(|base| base.join(path))
        .map_err(|e| format!("Invalid Redmine path {}: {}", path, e))?;
    let mut params: Vec<(String, String)> = joined
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    for (key, value) in extra {
        params.retain(|(k, _)| k != key);
        params.push(((*key).to_string(), (*value).to_string()));
    }
    let mut rebuilt = joined.clone();
    rebuilt.set_query(None);
    {
        let mut query = rebuilt.query_pairs_mut();
        for (k, v) in &params {
            query.append_pair(k, v);
        }
    }
    let suffix = rebuilt
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    Ok(format!("{}{}", rebuilt.path(), suffix))
}

fn extract_time_entries_range(path: &str) -> Option<String> {
    let joined = Url::parse("https://example.invalid").ok()?.join(path).ok()?;
    let mut from = None;
    let mut to = None;
    let mut spent_on = None;
    for (k, v) in joined.query_pairs() {
        match k.as_ref() {
            "from" => from = Some(v.to_string()),
            "to" => to = Some(v.to_string()),
            "spent_on" => spent_on = Some(v.to_string()),
            _ => {}
        }
    }
    if let Some(day) = spent_on {
        Some(day)
    } else if let (Some(from), Some(to)) = (from, to) {
        Some(format!("{}..{}", from, to))
    } else {
        None
    }
}

fn extract_single_day_from_range(path: &str) -> Option<String> {
    let joined = Url::parse("https://example.invalid").ok()?.join(path).ok()?;
    let mut from = None;
    let mut to = None;
    for (k, v) in joined.query_pairs() {
        match k.as_ref() {
            "from" => from = Some(v.to_string()),
            "to" => to = Some(v.to_string()),
            _ => {}
        }
    }
    match (from, to) {
        (Some(from), Some(to)) if from == to => Some(from),
        _ => None,
    }
}

/// Parse projects from GET /projects.json response. Returns "id=name (identifier), ...".
fn parse_projects(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("projects")?.as_array()?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|p| {
            let id = p.get("id")?.as_i64()?;
            let name = p.get("name")?.as_str()?;
            let ident = p.get("identifier").and_then(|i| i.as_str()).unwrap_or("");
            if ident.is_empty() {
                Some(format!("{}={}", id, name))
            } else {
                Some(format!("{}={} ({})", id, name, ident))
            }
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Projects: {}.", parts.join(", ")))
}

/// Parse trackers from GET /trackers.json response.
fn parse_trackers(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("trackers")?.as_array()?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|t| {
            let id = t.get("id")?.as_i64()?;
            let name = t.get("name")?.as_str()?;
            Some(format!("{}={}", id, name))
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Trackers: {}.", parts.join(", ")))
}

/// Parse issue statuses from GET /issue_statuses.json response.
fn parse_issue_statuses(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v
        .get("issue_statuses")
        .and_then(|a| a.as_array())
        .or_else(|| v.as_array());
    let arr = arr?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|s| {
            let id = s.get("id")?.as_i64()?;
            let name = s.get("name")?.as_str()?;
            Some(format!("{}={}", id, name))
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Statuses: {}.", parts.join(", ")))
}

/// Parse issue priorities from GET /enumerations/issue_priorities.json response.
fn parse_priorities(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("issue_priorities")?.as_array()?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|p| {
            let id = p.get("id")?.as_i64()?;
            let name = p.get("name")?.as_str()?;
            Some(format!("{}={}", id, name))
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Priorities: {}.", parts.join(", ")))
}

async fn fetch_issue_subjects_for_entries(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    entries: &[RedmineTimeEntry],
) -> BTreeMap<u64, String> {
    let mut out = BTreeMap::new();
    let mut missing = BTreeSet::new();
    for entry in entries {
        if let Some(issue) = &entry.issue {
            if issue
                .subject
                .as_deref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                missing.insert(issue.id);
            }
        }
    }
    for id in missing {
        let path = format!("/issues/{}.json", id);
        let body = match redmine_get_with_client(client, base_url, api_key, &path).await {
            Ok(body) => body,
            Err(e) => {
                debug!("Redmine: failed to fetch issue {} for subject lookup: {}", id, e);
                continue;
            }
        };
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(subject) = value
                .get("issue")
                .and_then(|issue| issue.get("subject"))
                .and_then(|subject| subject.as_str())
            {
                out.insert(id, subject.to_string());
            }
        }
    }
    out
}

fn summarize_time_entries(
    path: &str,
    entries: &[RedmineTimeEntry],
    total_count: usize,
    fetched_count: usize,
    subjects_by_issue: &BTreeMap<u64, String>,
) -> String {
    let mut tickets: BTreeMap<u64, TicketAggregate> = BTreeMap::new();
    let mut project_only: BTreeMap<String, ProjectOnlyAggregate> = BTreeMap::new();
    let mut total_hours = 0.0f64;

    for entry in entries {
        total_hours += entry.hours;
        let user_name = entry
            .user
            .as_ref()
            .and_then(|u| u.name.clone())
            .unwrap_or_else(|| "unknown user".to_string());
        let activity_name = entry
            .activity
            .as_ref()
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| "unknown activity".to_string());
        let project_name = entry
            .project
            .as_ref()
            .and_then(|p| p.name.clone())
            .unwrap_or_else(|| "unknown project".to_string());

        if let Some(issue) = &entry.issue {
            let agg = tickets.entry(issue.id).or_default();
            agg.subject = issue
                .subject
                .clone()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| subjects_by_issue.get(&issue.id).cloned())
                .unwrap_or_else(|| "(subject unavailable)".to_string());
            agg.project = project_name;
            agg.hours += entry.hours;
            agg.entries += 1;
            agg.users.insert(user_name);
            agg.activities.insert(activity_name);
        } else {
            let key = format!("{}|{}", project_name, activity_name);
            let agg = project_only.entry(key).or_default();
            agg.label = project_name;
            agg.hours += entry.hours;
            agg.entries += 1;
            agg.users.insert(user_name);
            agg.activities.insert(activity_name);
        }
    }

    let mut out = String::new();
    out.push_str("Derived Redmine time-entry summary\n");
    if let Some(range) = extract_time_entries_range(path) {
        out.push_str(&format!("Range: {}\n", range));
    }
    out.push_str(&format!(
        "Fetched {} time entr{} (total available: {}). Total hours: {:.2}\n",
        fetched_count,
        if fetched_count == 1 { "y" } else { "ies" },
        total_count,
        total_hours
    ));

    let mut ticket_rows: Vec<(u64, &TicketAggregate)> =
        tickets.iter().map(|(id, agg)| (*id, agg)).collect();
    ticket_rows.sort_by(|a, b| {
        b.1.hours
            .partial_cmp(&a.1.hours)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    if ticket_rows.is_empty() {
        out.push_str("\nTickets worked:\n- None found in these time entries.\n");
    } else {
        out.push_str("\nTickets worked:\n");
        for (id, agg) in ticket_rows {
            let users = agg.users.iter().cloned().collect::<Vec<_>>().join(", ");
            let activities = agg.activities.iter().cloned().collect::<Vec<_>>().join(", ");
            out.push_str(&format!(
                "- #{} {} — {:.2}h across {} entr{} (project: {}; users: {}; activities: {})\n",
                id,
                agg.subject,
                agg.hours,
                agg.entries,
                if agg.entries == 1 { "y" } else { "ies" },
                agg.project,
                users,
                activities
            ));
        }
    }

    if !project_only.is_empty() {
        out.push_str("\nTime entries without issue:\n");
        let mut rows: Vec<&ProjectOnlyAggregate> = project_only.values().collect();
        rows.sort_by(|a, b| {
            b.hours
                .partial_cmp(&a.hours)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for agg in rows {
            let users = agg.users.iter().cloned().collect::<Vec<_>>().join(", ");
            let activities = agg.activities.iter().cloned().collect::<Vec<_>>().join(", ");
            out.push_str(&format!(
                "- {} — {:.2}h across {} entr{} (users: {}; activities: {})\n",
                agg.label,
                agg.hours,
                agg.entries,
                if agg.entries == 1 { "y" } else { "ies" },
                users,
                activities
            ));
        }
    }

    out.push_str("\nEntry details:\n");
    for entry in entries.iter().take(50) {
        let issue_label = entry
            .issue
            .as_ref()
            .map(|issue| {
                let subject = issue
                    .subject
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| subjects_by_issue.get(&issue.id).cloned())
                    .unwrap_or_else(|| "(subject unavailable)".to_string());
                format!("#{} {}", issue.id, subject)
            })
            .unwrap_or_else(|| "(no issue)".to_string());
        let project = entry
            .project
            .as_ref()
            .and_then(|p| p.name.clone())
            .unwrap_or_else(|| "unknown project".to_string());
        let user = entry
            .user
            .as_ref()
            .and_then(|u| u.name.clone())
            .unwrap_or_else(|| "unknown user".to_string());
        let activity = entry
            .activity
            .as_ref()
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| "unknown activity".to_string());
        let spent_on = entry
            .spent_on
            .clone()
            .unwrap_or_else(|| "(no date)".to_string());
        let comments = entry.comments.clone().unwrap_or_default();
        out.push_str(&format!(
            "- {} | entry {} | {} | {:.2}h | {} | project: {} | user: {}{}\n",
            spent_on,
            entry.id,
            issue_label,
            entry.hours,
            activity,
            project,
            user,
            if comments.trim().is_empty() {
                String::new()
            } else {
                format!(" | comments: {}", comments.trim())
            }
        ));
    }
    if entries.len() > 50 {
        out.push_str(&format!(
            "- … {} more entr{} omitted\n",
            entries.len() - 50,
            if entries.len() - 50 == 1 { "y" } else { "ies" }
        ));
    }
    out
}

async fn fetch_and_summarize_time_entries(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    path: &str,
) -> Result<String, String> {
    async fn fetch_pages(
        client: &reqwest::Client,
        base_url: &str,
        api_key: &str,
        path: &str,
    ) -> Result<(Vec<RedmineTimeEntry>, usize), String> {
        let mut all_entries = Vec::new();
        let mut total_count = 0usize;
        let mut offset = 0usize;
        let page_limit = 100usize;

        loop {
            let offset_s = offset.to_string();
            let limit_s = page_limit.to_string();
            let paged_path =
                with_query_params(path, &[("offset", &offset_s), ("limit", &limit_s)])?;
            let body = redmine_get_with_client(client, base_url, api_key, &paged_path).await?;
            let parsed = match serde_json::from_str::<RedmineTimeEntriesResponse>(&body) {
                Ok(parsed) => parsed,
                Err(e) => {
                    debug!(
                        "Redmine: could not parse time_entries response as structured JSON: {}",
                        e
                    );
                    return Err(body);
                }
            };
            if offset == 0 {
                total_count = parsed.total_count.unwrap_or(parsed.time_entries.len());
            }
            let page_len = parsed.time_entries.len();
            all_entries.extend(parsed.time_entries);
            if page_len == 0 {
                break;
            }
            let response_limit = parsed.limit.unwrap_or(page_len.max(1));
            let response_offset = parsed.offset.unwrap_or(offset);
            let next_offset = response_offset + response_limit;
            if all_entries.len() >= total_count || next_offset <= offset {
                break;
            }
            offset = next_offset;
        }
        Ok((all_entries, total_count))
    }

    let (mut all_entries, mut total_count) = match fetch_pages(client, base_url, api_key, path).await {
        Ok(v) => v,
        Err(body) => return Ok(body),
    };

    if all_entries.is_empty() {
        if let Some(day) = extract_single_day_from_range(path) {
            let spent_on_path = with_query_params(path, &[("spent_on", &day), ("from", ""), ("to", "")])?;
            let spent_on_path = spent_on_path
                .replace("from=&", "")
                .replace("&to=", "")
                .replace("?to=", "?");
            if let Ok((entries, count)) = fetch_pages(client, base_url, api_key, &spent_on_path).await {
                if !entries.is_empty() {
                    debug!(
                        "Redmine: same-day from/to returned no entries; spent_on={} returned {} entr{}",
                        day,
                        entries.len(),
                        if entries.len() == 1 { "y" } else { "ies" }
                    );
                    all_entries = entries;
                    total_count = count.max(all_entries.len());
                }
            }
        }
    }

    let subjects = fetch_issue_subjects_for_entries(client, base_url, api_key, &all_entries).await;
    Ok(summarize_time_entries(
        path,
        &all_entries,
        total_count.max(all_entries.len()),
        all_entries.len(),
        &subjects,
    ))
}

/// Fetch and format Redmine create context (projects, trackers, statuses, priorities).
/// Cached for CONTEXT_CACHE_TTL_SECS. Used so the LLM can resolve "Create in AMVARA" to project_id.
pub async fn get_redmine_create_context() -> Option<String> {
    {
        let guard = REDMINE_CONTEXT_CACHE.lock().ok()?;
        if let Some((ref ctx, instant)) = *guard {
            if instant.elapsed() < Duration::from_secs(CONTEXT_CACHE_TTL_SECS) && !ctx.is_empty() {
                debug!("Redmine create context: using cache ({} chars)", ctx.len());
                return Some(ctx.clone());
            }
        }
    }

    let base = "Current Redmine values (use when creating issues; match project/tracker/status/priority by name or id): ";
    let mut parts = Vec::new();

    if let Ok(body) = redmine_get("projects.json").await {
        if let Some(s) = parse_projects(&body) {
            parts.push(s);
        }
    } else {
        debug!("Redmine context: projects fetch failed");
    }
    if let Ok(body) = redmine_get("trackers.json").await {
        if let Some(s) = parse_trackers(&body) {
            parts.push(s);
        }
    }
    if let Ok(body) = redmine_get("issue_statuses.json").await {
        if let Some(s) = parse_issue_statuses(&body) {
            parts.push(s);
        }
    }
    if let Ok(body) = redmine_get("enumerations/issue_priorities.json").await {
        if let Some(s) = parse_priorities(&body) {
            parts.push(s);
        }
    }

    if parts.is_empty() {
        warn!("Redmine create context: no data fetched (all endpoints failed or empty)");
        return None;
    }

    let context = format!("{}{}", base, parts.join(" "));
    info!(
        "Redmine create context: {} chars (projects, trackers, statuses, priorities)",
        context.len()
    );

    {
        let mut guard = REDMINE_CONTEXT_CACHE.lock().ok()?;
        *guard = Some((context.clone(), Instant::now()));
    }
    Some(context)
}

/// Perform a Redmine API request.
///
/// - method: GET or PUT (PUT only for allowed paths).
/// - path: relative to base URL (e.g. `/issues/1234.json?include=journals`).
/// - body: optional JSON body for PUT.
pub async fn redmine_api_request(
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let base_url = get_redmine_url().ok_or_else(|| {
        "Redmine not configured (REDMINE_URL missing from env or .config.env)".to_string()
    })?;
    let api_key = get_redmine_api_key().ok_or_else(|| {
        "Redmine not configured (REDMINE_API_KEY missing from env or .config.env)".to_string()
    })?;

    let path = path.trim();
    if path.is_empty() {
        return Err("Redmine API path must not be empty".to_string());
    }
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    let method_upper = method.to_uppercase();
    let allowed = match method_upper.as_str() {
        "GET" => true,
        "PUT" => is_allowed_put_path(&path),
        "POST" => is_allowed_post_path(&path),
        _ => false,
    };
    if !allowed {
        return Err(format!(
            "Redmine API: method {} not allowed for path {} (GET anything, POST /issues.json to create, PUT only issues/{{id}}.json)",
            method_upper, path
        ));
    }

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    info!("Redmine API: {} {}", method_upper, url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    if method_upper == "GET" && is_time_entries_path(&path) {
        return fetch_and_summarize_time_entries(&client, &base_url, &api_key, &path).await;
    }

    // Do not log request/response headers or bodies that may contain credentials.
    let mut req = client
        .request(
            method_upper
                .parse()
                .map_err(|e| format!("Invalid method: {}", e))?,
            &url,
        )
        .header("X-Redmine-API-Key", &api_key)
        .header(
            "User-Agent",
            format!("mac-stats/{}", crate::config::Config::version()),
        );

    if method_upper == "PUT" || method_upper == "POST" {
        let body_str = body.unwrap_or("{}").trim();
        debug!("Redmine API {} body: {}", method_upper, body_str);
        let body_json: serde_json::Value =
            serde_json::from_str(body_str).map_err(|e| format!("Invalid JSON body: {}", e))?;
        req = req
            .header("Content-Type", "application/json")
            .json(&body_json);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Redmine request failed: {}", e))?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        info!(
            "Redmine API {} {} → {} body: {}",
            method_upper,
            path,
            status,
            crate::logging::ellipse(&body_text, 500)
        );
        if status.as_u16() == 422 {
            return Err("Redmine didn't accept the query (invalid parameters, e.g. updated_on). Use a date in YYYY-MM-DD or a range like YYYY-MM-DD..YYYY-MM-DD.".to_string());
        }
        return Err(format!(
            "Redmine API {}: {}",
            status,
            crate::logging::ellipse(&body_text, 500)
        ));
    }

    info!(
        "Redmine API {} {} → {} ({} chars)",
        method_upper,
        path,
        status,
        body_text.len()
    );

    if body_text.chars().count() > MAX_RESPONSE_CHARS {
        Ok(crate::logging::ellipse(&body_text, MAX_RESPONSE_CHARS))
    } else {
        Ok(body_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_entries_path_detection_works() {
        assert!(is_time_entries_path("/time_entries.json?from=2026-03-06&to=2026-03-06"));
        assert!(is_time_entries_path("time_entries.json"));
        assert!(!is_time_entries_path("/issues/7209.json"));
    }

    #[test]
    fn summarizes_ticket_list_from_time_entries() {
        let entries = vec![
            RedmineTimeEntry {
                id: 1,
                hours: 1.5,
                spent_on: Some("2026-03-06".to_string()),
                comments: Some("Investigated".to_string()),
                project: Some(RedmineNamedRef {
                    name: Some("Core".to_string()),
                }),
                issue: Some(RedmineIssueRef {
                    id: 7209,
                    subject: Some("Fix login".to_string()),
                }),
                user: Some(RedmineNamedRef {
                    name: Some("Ralf".to_string()),
                }),
                activity: Some(RedmineNamedRef {
                    name: Some("Development".to_string()),
                }),
            },
            RedmineTimeEntry {
                id: 2,
                hours: 0.5,
                spent_on: Some("2026-03-06".to_string()),
                comments: Some("Review".to_string()),
                project: Some(RedmineNamedRef {
                    name: Some("Core".to_string()),
                }),
                issue: Some(RedmineIssueRef {
                    id: 7209,
                    subject: Some("Fix login".to_string()),
                }),
                user: Some(RedmineNamedRef {
                    name: Some("Ralf".to_string()),
                }),
                activity: Some(RedmineNamedRef {
                    name: Some("Code Review".to_string()),
                }),
            },
            RedmineTimeEntry {
                id: 3,
                hours: 1.0,
                spent_on: Some("2026-03-06".to_string()),
                comments: Some("Standup".to_string()),
                project: Some(RedmineNamedRef {
                    name: Some("Internal".to_string()),
                }),
                issue: None,
                user: Some(RedmineNamedRef {
                    name: Some("Ralf".to_string()),
                }),
                activity: Some(RedmineNamedRef {
                    name: Some("Meeting".to_string()),
                }),
            },
        ];
        let summary = summarize_time_entries(
            "/time_entries.json?from=2026-03-06&to=2026-03-06",
            &entries,
            3,
            3,
            &BTreeMap::new(),
        );
        assert!(summary.contains("Range: 2026-03-06..2026-03-06"));
        assert!(summary.contains("#7209 Fix login — 2.00h"));
        assert!(summary.contains("Time entries without issue"));
        assert!(summary.contains("Internal — 1.00h"));
    }

    #[test]
    fn summarizes_ticket_list_uses_subject_lookup_when_time_entries_only_return_issue_ids() {
        let entries = vec![RedmineTimeEntry {
            id: 1,
            hours: 2.0,
            spent_on: Some("2026-03-06".to_string()),
            comments: Some("Worked on issue".to_string()),
            project: Some(RedmineNamedRef {
                name: Some("Core".to_string()),
            }),
            issue: Some(RedmineIssueRef {
                id: 7209,
                subject: None,
            }),
            user: Some(RedmineNamedRef {
                name: Some("Ralf".to_string()),
            }),
            activity: Some(RedmineNamedRef {
                name: Some("Development".to_string()),
            }),
        }];
        let mut subjects = BTreeMap::new();
        subjects.insert(7209, "Fetched subject from issue lookup".to_string());

        let summary = summarize_time_entries(
            "/time_entries.json?from=2026-03-06&to=2026-03-06&limit=100",
            &entries,
            1,
            1,
            &subjects,
        );

        assert!(summary.contains("#7209 Fetched subject from issue lookup — 2.00h"));
        assert!(!summary.contains("#7209 (subject unavailable)"));
    }
}
