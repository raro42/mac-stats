//! Periodically organize files in the user's Downloads folder using markdown rules.
//! See `docs/024_downloads_organizer.md`.

use crate::config::Config;
use chrono::{DateTime, Local, NaiveTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Parsed rules + optional catch-all folder (relative path under Downloads root).
#[derive(Debug, Clone)]
pub struct ParsedRules {
    pub catch_all: Option<String>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub extensions: Vec<String>,
    pub glob: Option<Regex>,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizerState {
    pub last_run_utc: Option<DateTime<Utc>>,
    pub last_dry_run: bool,
    pub moved: u32,
    pub skipped: u32,
    pub failed: u32,
    pub rules_error: Option<String>,
}

impl Default for OrganizerState {
    fn default() -> Self {
        Self {
            last_run_utc: None,
            last_dry_run: true,
            moved: 0,
            skipped: 0,
            failed: 0,
            rules_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlannedMove {
    pub src: PathBuf,
    pub dest: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub dry_run: bool,
    pub moved: u32,
    pub skipped: u32,
    pub failed: u32,
    pub planned: Vec<PlannedMove>,
}

/// Parse `ruleset_version: N` from footer; informational only.
pub fn parse_ruleset_version(content: &str) -> Option<u32> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("ruleset_version:") {
            return rest.trim().parse().ok();
        }
    }
    None
}

fn glob_to_regex(pat: &str) -> Result<Regex, String> {
    let mut s = String::with_capacity(pat.len() + 2);
    s.push('^');
    for c in pat.chars() {
        match c {
            '*' => s.push_str(".*"),
            '?' => s.push('.'),
            '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                s.push('\\');
                s.push(c);
            }
            '.' => s.push_str("\\."),
            _ => s.push(c),
        }
    }
    s.push('$');
    Regex::new(&s).map_err(|e| format!("invalid glob: {}", e))
}

fn parse_kv_line(line: &str) -> Option<(String, String)> {
    let t = line.trim();
    let t = t.strip_prefix('-')?.trim();
    let (k, v) = t.split_once(':')?;
    Some((k.trim().to_lowercase(), v.trim().to_string()))
}

/// Parse markdown rules. Returns error message for invalid rules (no partial rules applied).
pub fn parse_rules_markdown(content: &str) -> Result<ParsedRules, String> {
    let mut catch_all: Option<String> = None;
    let mut rules: Vec<Rule> = Vec::new();
    let mut block: Option<&'static str> = None;
    let mut buf: Vec<(String, String)> = Vec::new();

    fn flush_rule(
        buf: &[(String, String)],
        rules: &mut Vec<Rule>,
        line_hint: usize,
    ) -> Result<(), String> {
        if buf.is_empty() {
            return Ok(());
        }
        let mut exts: Vec<String> = Vec::new();
        let mut glob: Option<String> = None;
        let mut destination: Option<String> = None;
        for (k, v) in buf {
            match k.as_str() {
                "match_extensions" => {
                    for p in v.split(',') {
                        let p = p.trim().trim_start_matches('.').to_lowercase();
                        if !p.is_empty() {
                            exts.push(p);
                        }
                    }
                }
                "match_glob" => {
                    let g = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !g.is_empty() {
                        glob = Some(g);
                    }
                }
                "destination" => {
                    let d = v.trim().trim_matches('/').to_string();
                    if d.is_empty() {
                        return Err(format!(
                            "line ~{}: destination cannot be empty",
                            line_hint
                        ));
                    }
                    destination = Some(d);
                }
                _ => {}
            }
        }
        let destination = destination.ok_or_else(|| {
            format!(
                "line ~{}: Rule block missing destination",
                line_hint
            )
        })?;
        if exts.is_empty() && glob.is_none() {
            return Err(format!(
                "line ~{}: Rule must have match_extensions or match_glob",
                line_hint
            ));
        }
        let glob_re = match &glob {
            Some(g) => Some(glob_to_regex(g)?),
            None => None,
        };
        rules.push(Rule {
            extensions: exts,
            glob: glob_re,
            destination: destination.clone(),
        });
        Ok(())
    }

    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("## ") {
            let title = rest.trim().to_lowercase();
            if title.starts_with("settings") {
                if block == Some("rule") {
                    flush_rule(&buf, &mut rules, i + 1)?;
                    buf.clear();
                }
                block = Some("settings");
                buf.clear();
            } else if title.starts_with("rule") {
                if block == Some("rule") {
                    flush_rule(&buf, &mut rules, i + 1)?;
                } else if block == Some("settings") {
                    for (k, v) in &buf {
                        if k == "catch_all" {
                            let d = v.trim().trim_matches('/').to_string();
                            if !d.is_empty() {
                                catch_all = Some(d);
                            }
                        }
                    }
                    buf.clear();
                }
                block = Some("rule");
                buf.clear();
            } else {
                if block == Some("rule") {
                    flush_rule(&buf, &mut rules, i + 1)?;
                    buf.clear();
                }
                block = None;
            }
            continue;
        }
        if let Some((k, v)) = parse_kv_line(t) {
            if block.is_some() {
                buf.push((k, v));
            }
        }
    }

    if block == Some("rule") {
        flush_rule(&buf, &mut rules, content.lines().count())?;
    } else if block == Some("settings") {
        for (k, v) in &buf {
            if k == "catch_all" {
                let d = v.trim().trim_matches('/').to_string();
                if !d.is_empty() {
                    catch_all = Some(d);
                }
            }
        }
    }

    Ok(ParsedRules { catch_all, rules })
}

fn default_ignore(name: &str) -> bool {
    let lower = name.to_lowercase();
    if name == ".DS_Store" || lower == ".ds_store" {
        return true;
    }
    if lower.ends_with(".crdownload") || lower.ends_with(".part") {
        return true;
    }
    false
}

fn extension_of(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

fn matches_rule(rule: &Rule, file_name: &str) -> bool {
    if let Some(re) = &rule.glob {
        if re.is_match(file_name) {
            return true;
        }
    }
    if let Some(ext) = extension_of(file_name) {
        if rule.extensions.iter().any(|e| e == &ext) {
            return true;
        }
    }
    false
}

fn first_matching_destination<'a>(rules: &'a ParsedRules, file_name: &str) -> Option<&'a str> {
    for r in &rules.rules {
        if matches_rule(r, file_name) {
            return Some(r.destination.as_str());
        }
    }
    rules.catch_all.as_deref()
}

/// Resolve destination path with collision handling: `name (1).ext`, `name (2).ext`, ...
pub fn unique_destination(root: &Path, rel_dir: &str, file_name: &str) -> PathBuf {
    let dest_dir = root.join(rel_dir);
    let mut candidate = dest_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(file_name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();
    let mut n = 1u32;
    loop {
        let fname = format!("{} ({}){}", stem, n, ext);
        candidate = dest_dir.join(&fname);
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
        if n > 10_000 {
            return dest_dir.join(format!("{}_{}", file_name, n));
        }
    }
}

/// Build a deterministic move plan (sorted by source path). Does not touch the filesystem.
pub fn plan_moves(root: &Path, rules: &ParsedRules) -> Result<Vec<PlannedMove>, String> {
    if !root.is_dir() {
        return Err(format!("Downloads root is not a directory: {:?}", root));
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(root)
        .map_err(|e| format!("read_dir {:?}: {}", root, e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    entries.sort();

    let mut out: Vec<PlannedMove> = Vec::new();
    for src in entries {
        let name = src
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("bad file name {:?}", src))?;
        if default_ignore(name) {
            continue;
        }
        let Some(dest_rel) = first_matching_destination(rules, name) else {
            continue;
        };
        let dest = unique_destination(root, dest_rel, name);
        if dest == src {
            continue;
        }
        out.push(PlannedMove { src, dest });
    }
    Ok(out)
}

/// Apply moves sequentially; per-file errors are warnings, not fatal.
pub fn apply_moves(plan: &[PlannedMove], dry_run: bool) -> RunSummary {
    let mut moved = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;
    let mut planned = Vec::new();

    for m in plan {
        if dry_run {
            debug!(
                "Downloads organizer (dry-run): would move {:?} -> {:?}",
                m.src, m.dest
            );
            planned.push(m.clone());
            continue;
        }
        if let Some(parent) = m.dest.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!(
                    "Downloads organizer: skip {:?} (mkdir {:?}): {}",
                    m.src, parent, e
                );
                skipped += 1;
                continue;
            }
        }
        match fs::rename(&m.src, &m.dest) {
            Ok(()) => {
                debug!(
                    "Downloads organizer: moved {:?} -> {:?}",
                    m.src, m.dest
                );
                moved += 1;
            }
            Err(e) if e.kind() == ErrorKind::CrossesDevices => {
                if let Err(e2) = fs::copy(&m.src, &m.dest).and_then(|_| fs::remove_file(&m.src)) {
                    warn!(
                        "Downloads organizer: failed {:?} -> {:?}: {}",
                        m.src, m.dest, e2
                    );
                    failed += 1;
                } else {
                    moved += 1;
                }
            }
            Err(e) => {
                warn!(
                    "Downloads organizer: failed {:?} -> {:?}: {}",
                    m.src, m.dest, e
                );
                failed += 1;
            }
        }
    }

    let moved = if dry_run {
        planned.len() as u32
    } else {
        moved
    };
    RunSummary {
        dry_run,
        moved,
        skipped,
        failed,
        planned: if dry_run { planned } else { Vec::new() },
    }
}

pub(crate) fn load_organizer_state() -> OrganizerState {
    let path = Config::downloads_organizer_state_path();
    let s = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return OrganizerState::default(),
    };
    serde_json::from_str(&s).unwrap_or_default()
}

fn save_state(state: &OrganizerState) {
    let path = Config::downloads_organizer_state_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = atomic_write(&path, json.as_bytes());
    }
}

fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    let mut f = fs::File::create(&tmp)?;
    f.write_all(data)?;
    f.sync_all()?;
    drop(f);
    fs::rename(&tmp, path)
}

/// Expand `~` and resolve default `~/Downloads`.
pub fn resolve_downloads_root_from_config() -> Result<PathBuf, String> {
    let configured = Config::downloads_organizer_path_raw();
    let expanded = expand_home_path(&configured)?;
    validate_organizer_root(&expanded)?;
    Ok(expanded)
}

fn expand_home_path(s: &str) -> Result<PathBuf, String> {
    let t = s.trim();
    if t.is_empty() {
        return default_downloads_dir();
    }
    if let Some(rest) = t.strip_prefix("~/") {
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        return Ok(PathBuf::from(home).join(rest));
    }
    if t == "~" {
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        return Ok(PathBuf::from(home));
    }
    Ok(PathBuf::from(t))
}

fn default_downloads_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(PathBuf::from(home).join("Downloads"))
}

/// Organizer root must stay under the user's home directory (after canonicalize).
pub fn validate_organizer_root(path: &Path) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    let home_pb = PathBuf::from(&home);
    let home_canon = fs::canonicalize(&home_pb).unwrap_or(home_pb.clone());
    let p = if path.exists() {
        fs::canonicalize(path).map_err(|e| format!("invalid path {:?}: {}", path, e))?
    } else {
        path.to_path_buf()
    };
    if !p.starts_with(&home_canon) {
        return Err(format!(
            "downloads organizer path must be under home directory: {:?}",
            p
        ));
    }
    Ok(())
}

fn interval_off() -> bool {
    Config::downloads_organizer_interval() == "off"
}

fn is_due(last: Option<DateTime<Utc>>) -> bool {
    if !Config::downloads_organizer_enabled() || interval_off() {
        return false;
    }
    let now = Utc::now();
    match Config::downloads_organizer_interval().as_str() {
        "hourly" => match last {
            None => true,
            Some(t) => (now - t).num_seconds() >= 3600,
        },
        "daily" => daily_due(last, Config::downloads_organizer_daily_at_local()),
        _ => false,
    }
}

/// Run once per local day after `hh:mm`, if we have not already run since that time today.
fn daily_due(last: Option<DateTime<Utc>>, hhmm: (u32, u32)) -> bool {
    let now = Local::now();
    let (h, m) = hhmm;
    let Some(target_t) = NaiveTime::from_hms_opt(h, m, 0) else {
        return false;
    };
    if now.time() < target_t {
        return false;
    }
    match last {
        None => true,
        Some(lu) => {
            let ll = lu.with_timezone(&Local);
            ll.date_naive() < now.date_naive() || ll.time() < target_t
        }
    }
}

/// One full organizer run: resolve root, parse rules, plan, apply or dry-run, persist state.
pub fn run_organizer_pass() {
    let mut state = load_organizer_state();

    let root = match resolve_downloads_root_from_config() {
        Ok(p) => p,
        Err(e) => {
            warn!("Downloads organizer: bad root path: {}", e);
            state.rules_error = Some(e.clone());
            state.last_run_utc = Some(Utc::now());
            save_state(&state);
            return;
        }
    };

    let rules_path = Config::downloads_organizer_rules_path();
    let rules_content = match fs::read_to_string(&rules_path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Downloads organizer: cannot read rules {:?}: {}", rules_path, e);
            state.rules_error = Some(format!("read rules: {}", e));
            state.last_run_utc = Some(Utc::now());
            save_state(&state);
            return;
        }
    };

    let parsed = match parse_rules_markdown(&rules_content) {
        Ok(p) => p,
        Err(e) => {
            warn!("Downloads organizer: invalid rules — skipping run: {}", e);
            state.rules_error = Some(e.clone());
            state.last_run_utc = Some(Utc::now());
            save_state(&state);
            return;
        }
    };
    let _ = parse_ruleset_version(&rules_content);

    let dry = Config::downloads_organizer_dry_run();
    let plan = match plan_moves(&root, &parsed) {
        Ok(p) => p,
        Err(e) => {
            warn!("Downloads organizer: plan failed: {}", e);
            state.rules_error = Some(e.clone());
            state.last_run_utc = Some(Utc::now());
            save_state(&state);
            return;
        }
    };

    state.rules_error = None;
    let summary = apply_moves(&plan, dry);
    info!(
        "Downloads organizer: {} — moved={}, skipped={}, failed={}, dry_run={}",
        root.display(),
        summary.moved,
        summary.skipped,
        summary.failed,
        dry
    );

    state.last_run_utc = Some(Utc::now());
    state.last_dry_run = dry;
    state.moved = summary.moved;
    state.skipped = summary.skipped;
    state.failed = summary.failed;
    save_state(&state);
}

/// Background tick: run only when enabled, interval not `off`, and schedule is due.
pub fn run_if_due() {
    if !Config::downloads_organizer_enabled() || interval_off() {
        return;
    }
    let state = load_organizer_state();
    if !is_due(state.last_run_utc) {
        return;
    }
    run_organizer_pass();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

    fn tmp_organizer_dir() -> PathBuf {
        let n = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!(
            "mac-stats-downloads-organizer-test-{}-{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn parser_extensions_and_glob() {
        let md = r#"
## Settings
- catch_all: Other

## Rule
- match_extensions: png, PDF
- destination: Images

## Rule
- match_glob: "*.dmg"
- destination: Inst
"#;
        let p = parse_rules_markdown(md).unwrap();
        assert_eq!(p.catch_all.as_deref(), Some("Other"));
        assert_eq!(p.rules.len(), 2);
        assert!(p.rules[0].glob.is_none());
        assert_eq!(p.rules[0].extensions, vec!["png", "pdf"]);
        assert!(p.rules[1].glob.is_some());
    }

    #[test]
    fn first_match_wins() {
        let md = r#"
## Rule
- match_extensions: txt
- destination: A

## Rule
- match_extensions: txt
- destination: B
"#;
        let p = parse_rules_markdown(md).unwrap();
        assert_eq!(
            first_matching_destination(&p, "x.txt"),
            Some("A")
        );
    }

    #[test]
    fn plan_respects_catch_all_and_skips_crdownload() {
        let tmp = tmp_organizer_dir();
        let a = tmp.join("a.png");
        let b = tmp.join("b.nope");
        let c = tmp.join("x.crdownload");
        std::fs::write(&a, "1").unwrap();
        std::fs::write(&b, "2").unwrap();
        std::fs::write(&c, "3").unwrap();
        let rules = parse_rules_markdown(
            r##"
## Settings
- catch_all: Unsorted

## Rule
- match_extensions: png
- destination: Img
"##,
        )
        .unwrap();
        let plan = plan_moves(&tmp, &rules).unwrap();
        let dests: Vec<_> = plan.iter().map(|m| m.dest.clone()).collect();
        assert!(dests.iter().any(|p| p.ends_with("Img/a.png")));
        assert!(dests.iter().any(|p| p.ends_with("Unsorted/b.nope")));
        assert!(!dests.iter().any(|p| p.to_string_lossy().contains("crdownload")));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn dry_run_does_not_move() {
        let tmp = tmp_organizer_dir();
        let a = tmp.join("a.txt");
        std::fs::write(&a, "x").unwrap();
        let rules = parse_rules_markdown(
            r#"
## Rule
- match_extensions: txt
- destination: T
"#,
        )
        .unwrap();
        let plan = plan_moves(&tmp, &rules).unwrap();
        assert_eq!(plan.len(), 1);
        let s = apply_moves(&plan, true);
        assert_eq!(s.moved, 1);
        assert!(a.exists());
        let s2 = apply_moves(&plan, false);
        assert_eq!(s2.moved, 1);
        assert!(!a.exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn collision_rename() {
        let tmp = tmp_organizer_dir();
        std::fs::create_dir_all(tmp.join("T")).unwrap();
        std::fs::write(tmp.join("T").join("a.txt"), "old").unwrap();
        let a = tmp.join("a.txt");
        std::fs::write(&a, "new").unwrap();
        let rules = parse_rules_markdown(
            r#"
## Rule
- match_extensions: txt
- destination: T
"#,
        )
        .unwrap();
        let plan = plan_moves(&tmp, &rules).unwrap();
        assert!(plan[0].dest.ends_with("a (1).txt"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
