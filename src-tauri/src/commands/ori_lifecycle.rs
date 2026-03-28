//! Optional Ori Mnemos lifecycle hooks (config-gated).
//!
//! Uses the vault on disk (`ORI_VAULT` / `MAC_STATS_ORI_VAULT`): read `self/` + `ops/` for a
//! lightweight session briefing, `ori query similar` for prefetch (subprocess, cwd = vault),
//! and inbox markdown writes for capture (CLI `ori add` does not accept body; we write compatible
//! inbox notes). See `docs/ori-lifecycle.md`.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tokio::process::Command as TokioCommand;
use tracing::{debug, info, warn};

use crate::config::Config;

const ORI_TARGET: &str = "mac_stats::ori";

fn session_orient_key(source: &str, session_id: u64) -> String {
    format!("{}-{}", source, session_id)
}

fn oriented_sessions() -> &'static Mutex<HashSet<String>> {
    static S: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashSet::new()))
}

fn prefetch_last() -> &'static Mutex<HashMap<String, Instant>> {
    static S: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn capture_dedupe() -> &'static Mutex<HashMap<String, (u64, Instant)>> {
    static S: OnceLock<Mutex<HashMap<String, (u64, Instant)>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Call when a session is cleared so the next turn can run orient again.
pub(crate) fn on_session_cleared(source: &str, session_id: u64) {
    let key = session_orient_key(source, session_id);
    if let Ok(mut g) = oriented_sessions().lock() {
        g.remove(&key);
    }
    if let Ok(mut g) = prefetch_last().lock() {
        g.remove(&key);
    }
}

static VAULT_CONFIG_ERROR_LOGGED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Resolved vault root (must contain `.ori`), or None.
pub(crate) fn resolved_vault_root() -> Option<PathBuf> {
    if !Config::ori_lifecycle_enabled() {
        return None;
    }
    let raw = Config::ori_vault_path_raw();
    if raw.trim().is_empty() {
        if !VAULT_CONFIG_ERROR_LOGGED.swap(true, Ordering::SeqCst) {
            warn!(
                target: ORI_TARGET,
                "Ori lifecycle enabled but no vault path set (set MAC_STATS_ORI_VAULT or ORI_VAULT)"
            );
        }
        return None;
    }
    let p = Config::expand_user_path_str(raw.trim())?;
    if !p.join(".ori").is_file() {
        if !VAULT_CONFIG_ERROR_LOGGED.swap(true, Ordering::SeqCst) {
            warn!(
                target: ORI_TARGET,
                "Ori vault path missing .ori marker: {}",
                p.display()
            );
        }
        return None;
    }
    Some(p)
}

/// True when this compaction/router source should skip orient + prefetch (scheduler automation).
pub(crate) fn ori_skip_automation_sources(hook_source: &str) -> bool {
    if !Config::ori_skip_for_scheduler() {
        return false;
    }
    matches!(
        hook_source,
        "scheduler" | "heartbeat" | "task_runner" | "task-runner"
    )
}

fn strip_secretish_lines(text: &str) -> String {
    let lower_needles = [
        "api_key",
        "api-key",
        "apikey",
        "password",
        "secret",
        "token",
        "bearer ",
        "authorization:",
    ];
    text.lines()
        .filter(|line| {
            let l = line.to_lowercase();
            !lower_needles.iter().any(|n| l.contains(n))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_file_cap(path: &Path, max_bytes: usize) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; max_bytes];
    let n = f.read(&mut buf).ok()?;
    buf.truncate(n);
    let mut s = String::from_utf8_lossy(&buf).into_owned();
    if n == max_bytes {
        let safe = s.floor_char_boundary(max_bytes);
        s.truncate(safe);
        s.push_str(" [truncated]");
    }
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Build a bounded "briefing" from vault markdown (MCP `ori_orient` is richer; CLI has no orient).
pub(crate) fn build_orient_section_from_vault(vault: &Path, max_total: usize) -> Option<String> {
    let per_file = (max_total / 4).max(1500);
    let mut parts: Vec<String> = Vec::new();
    let mut used = 0usize;
    for rel in [
        "self/identity.md",
        "self/goals.md",
        "self/methodology.md",
        "ops/daily.md",
        "ops/reminders.md",
    ] {
        if used >= max_total {
            break;
        }
        let p = vault.join(rel);
        if let Some(chunk) = read_file_cap(&p, per_file.min(max_total - used)) {
            let cleaned = strip_secretish_lines(&chunk);
            if cleaned.trim().is_empty() {
                continue;
            }
            let header = rel.replace('/', " · ");
            let piece = format!("### {}\n\n{}", header, cleaned);
            used = used.saturating_add(piece.len());
            parts.push(piece);
        }
    }
    if parts.is_empty() {
        return None;
    }
    let body = parts.join("\n\n");
    let mut section = format!("\n\n## Ori session briefing\n\n{}", body);
    if section.len() > max_total {
        let cut = section.floor_char_boundary(max_total.saturating_sub(20));
        section.truncate(cut);
        section.push_str("\n… [truncated]");
    }
    Some(section)
}

/// Mark session as oriented after a successful briefing.
pub(crate) fn mark_session_oriented(source: &str, session_id: u64) {
    let key = session_orient_key(source, session_id);
    if let Ok(mut g) = oriented_sessions().lock() {
        g.insert(key);
    }
}

pub(crate) fn session_already_oriented(source: &str, session_id: u64) -> bool {
    let key = session_orient_key(source, session_id);
    oriented_sessions()
        .lock()
        .map(|g| g.contains(&key))
        .unwrap_or(false)
}

/// Compute orient section once per request when hooks + new session; does not mark oriented (caller marks after inject).
pub(crate) fn maybe_build_orient_section(
    vault: &Path,
    hook_source: &str,
    session_id: u64,
    prepared_history_empty: bool,
) -> Option<String> {
    if !Config::ori_hook_orient_on_session_start() {
        return None;
    }
    if ori_skip_automation_sources(hook_source) {
        return None;
    }
    if !prepared_history_empty || session_already_oriented(hook_source, session_id) {
        return None;
    }
    let cap = Config::ori_orient_max_chars();
    let s = build_orient_section_from_vault(vault, cap)?;
    info!(
        target: ORI_TARGET,
        hook_source,
        session_id,
        chars = s.len(),
        "injected Ori session briefing (vault file excerpts)"
    );
    Some(s)
}

fn prefetch_cooldown_key(source: &str, session_id: u64) -> String {
    session_orient_key(source, session_id)
}

pub(crate) async fn maybe_run_prefetch_section(
    vault: &Path,
    hook_source: &str,
    session_id: u64,
    query: &str,
) -> Option<String> {
    if !Config::ori_prefetch_enabled() {
        return None;
    }
    if ori_skip_automation_sources(hook_source) {
        return None;
    }
    let key = prefetch_cooldown_key(hook_source, session_id);
    let cd = Config::ori_prefetch_cooldown_secs();
    if cd > 0 {
        if let Ok(g) = prefetch_last().lock() {
            if let Some(t0) = g.get(&key) {
                if t0.elapsed() < Duration::from_secs(cd) {
                    debug!(
                        target: ORI_TARGET,
                        hook_source,
                        session_id,
                        "Ori prefetch skipped (cooldown)"
                    );
                    return None;
                }
            }
        }
    }

    let q = query.trim();
    if q.is_empty() {
        return None;
    }
    let q: String = q.chars().take(500).collect();
    let bin = Config::ori_binary();
    let limit = Config::ori_prefetch_top_k().clamp(1, 20);
    let timeout = Duration::from_secs(Config::ori_prefetch_timeout_secs().clamp(1, 60));

    let vault_owned = vault.to_path_buf();
    let out = tokio::time::timeout(
        timeout,
        TokioCommand::new(&bin)
            .current_dir(&vault_owned)
            .arg("query")
            .arg("similar")
            .arg(&q)
            .arg("--limit")
            .arg(limit.to_string())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output(),
    )
    .await;

    if let Ok(mut g) = prefetch_last().lock() {
        g.insert(key, Instant::now());
    }

    let output = match out {
        Ok(Ok(o)) if o.status.success() => o.stdout,
        Ok(Ok(o)) => {
            debug!(
                target: ORI_TARGET,
                status = ?o.status,
                "Ori prefetch: ori query similar failed"
            );
            return None;
        }
        Ok(Err(e)) => {
            warn!(target: ORI_TARGET, "Ori prefetch: spawn failed: {}", e);
            return None;
        }
        Err(_) => {
            warn!(target: ORI_TARGET, "Ori prefetch: timeout (skipped)");
            return None;
        }
    };

    let json_s = String::from_utf8_lossy(&output);
    let v: serde_json::Value = serde_json::from_str(json_s.trim()).ok()?;
    let results = v.get("data")?.get("results")?.as_array()?;
    let mut lines: Vec<String> = Vec::new();
    for r in results.iter().take(limit as usize) {
        let title = r.get("title").and_then(|x| x.as_str()).unwrap_or("note");
        let score = r.get("score").and_then(|x| x.as_f64());
        let line = if let Some(s) = score {
            format!("- **{}** (score {:.3})", title, s)
        } else {
            format!("- **{}**", title)
        };
        lines.push(line);
    }
    if lines.is_empty() {
        return None;
    }
    let body = lines.join("\n");
    let body = strip_secretish_lines(&body);
    let mut section = format!("\n\n## Possibly relevant vault notes\n\n{}", body);
    let cap = Config::ori_prefetch_max_chars();
    if section.len() > cap {
        let cut = section.floor_char_boundary(cap.saturating_sub(20));
        section.truncate(cut);
        section.push_str("\n… [truncated]");
    }
    info!(
        target: ORI_TARGET,
        hook_source,
        session_id,
        chars = section.len(),
        "injected Ori prefetch (query similar)"
    );
    Some(section)
}

fn slugify_title(title: &str) -> String {
    let s: String = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let slug = s.split_whitespace().collect::<Vec<_>>().join("-");
    if slug.is_empty() {
        format!("note-{}", chrono::Utc::now().timestamp())
    } else {
        slug
    }
}

fn write_inbox_capture(vault: &Path, title: &str, body: &str) -> Result<PathBuf, String> {
    if title.len() < 10 || !title.contains(' ') {
        return Err("Ori capture title must be multi-word (>=10 chars)".to_string());
    }
    let inbox = vault.join("inbox");
    std::fs::create_dir_all(&inbox).map_err(|e| format!("inbox mkdir: {}", e))?;
    let created = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let yaml = format!(
        "---\ndescription: \"mac-stats capture\"\ntype: insight\nproject: []\nstatus: inbox\ncreated: {created}\nlast_accessed: {created}\naccess_count: 0\n---\n\n# {title}\n\n{body}\n"
    );
    let base = slugify_title(title);
    let mut path = inbox.join(format!("{}.md", base));
    let mut n = 2u32;
    while path.exists() {
        path = inbox.join(format!("{}-{}.md", base, n));
        n += 1;
    }
    std::fs::write(&path, yaml).map_err(|e| format!("write inbox: {}", e))?;
    Ok(path)
}

fn hash_excerpt(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn should_dedupe_capture(source: &str, session_id: u64, excerpt: &str) -> bool {
    let key = format!("{}-{}-dedupe", source, session_id);
    let hh = hash_excerpt(excerpt);
    let now = Instant::now();
    if let Ok(mut g) = capture_dedupe().lock() {
        if let Some((prev_h, t0)) = g.get(&key).copied() {
            if prev_h == hh && now.duration_since(t0) < Duration::from_secs(24 * 3600) {
                return true;
            }
        }
        g.insert(key, (hh, now));
    }
    false
}

/// After successful compaction with lesson text (already filtered for conversational value).
pub(crate) fn maybe_capture_compaction_fire_and_forget(
    lessons: Option<&str>,
    hook_source: &str,
    session_id: u64,
    request_id: &str,
    discord_channel_id: Option<u64>,
) {
    if !Config::ori_hook_capture_on_compaction() {
        return;
    }
    let Some(vault) = resolved_vault_root() else {
        return;
    };
    if ori_skip_automation_sources(hook_source) {
        return;
    }
    if discord_channel_id.is_some_and(crate::discord::is_discord_channel_having_fun) {
        return;
    }
    let mode = Config::ori_compaction_capture_mode();
    if matches!(mode.as_str(), "off" | "") {
        return;
    }
    let Some(lesson_text) = lessons.map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };

    let excerpt = match mode.as_str() {
        "full_lessons_duplicate" => lesson_text.to_string(),
        _ => {
            let capped = lesson_text.chars().take(6000).collect::<String>();
            format!(
                "mac-stats compaction excerpt (request {})\n\n{}",
                request_id, capped
            )
        }
    };
    let excerpt = strip_secretish_lines(&excerpt);
    if excerpt.trim().len() < 20 {
        return;
    }
    if mode.as_str() == "excerpt_to_ori" && should_dedupe_capture(hook_source, session_id, &excerpt)
    {
        debug!(
            target: ORI_TARGET,
            hook_source,
            session_id,
            "Ori compaction capture skipped (24h dedupe)"
        );
        return;
    }

    let title = format!(
        "mac-stats compaction lessons {} {}",
        hook_source,
        chrono::Utc::now().format("%Y-%m-%d %H%M UTC")
    );
    let title_owned = title;
    let body_owned = excerpt;
    let rid = request_id.to_string();
    std::thread::spawn(
        move || match write_inbox_capture(&vault, &title_owned, &body_owned) {
            Ok(p) => {
                info!(
                    target: ORI_TARGET,
                    path = %p.display(),
                    request_id = %rid,
                    "Ori compaction capture wrote inbox note"
                );
            }
            Err(e) => {
                warn!(target: ORI_TARGET, "Ori compaction capture failed: {}", e);
            }
        },
    );
}

/// Before `clear_session` removes state: best-effort capture in a background thread.
/// `conversational_count` must be from [`crate::session_memory::count_conversational_messages`].
pub(crate) fn maybe_capture_before_session_reset_fire_and_forget(
    source: &str,
    session_id: u64,
    messages: Vec<(String, String)>,
    conversational_count: usize,
) {
    if !Config::ori_hook_before_session_reset() {
        return;
    }
    let Some(vault) = resolved_vault_root() else {
        return;
    };
    if conversational_count < crate::commands::compaction::MIN_CONVERSATIONAL_FOR_COMPACTION {
        debug!(
            target: ORI_TARGET,
            source,
            session_id,
            "Ori reset capture skipped (no conversational value)"
        );
        return;
    }
    let mut transcript = String::new();
    for (role, content) in messages.iter().rev().take(24).rev() {
        transcript.push_str(&format!("[{}]: {}\n\n", role, content));
    }
    let transcript = transcript
        .chars()
        .take(Config::ori_reset_capture_max_chars())
        .collect::<String>();
    let transcript = strip_secretish_lines(&transcript);
    if transcript.trim().len() < 20 {
        return;
    }
    let title = format!(
        "mac-stats session reset {} {}",
        source,
        chrono::Utc::now().format("%Y-%m-%d %H%M UTC")
    );
    let source_owned = source.to_string();
    std::thread::spawn(
        move || match write_inbox_capture(&vault, &title, &transcript) {
            Ok(p) => {
                info!(
                    target: ORI_TARGET,
                    path = %p.display(),
                    source = %source_owned,
                    session_id,
                    "Ori before-reset capture wrote inbox note"
                );
            }
            Err(e) => {
                warn!(target: ORI_TARGET, "Ori before-reset capture failed: {}", e);
            }
        },
    );
}
