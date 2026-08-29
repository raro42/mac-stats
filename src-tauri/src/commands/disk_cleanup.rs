//! Configurable disk cleanup: mac-stats data plus opt-in Trash / Downloads / Temp / custom paths.

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::commands::screenshot_lifecycle::parse_screenshot_filename_timestamp;
use crate::config::Config;
use crate::mac_stats_info;

const STATE_FILE: &str = "disk_cleanup.json";
const DEFAULT_INTERVAL_HOURS: u64 = 24;
const MAX_SCAN_FILES: usize = 50_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCategory {
    pub id: String,
    pub label: String,
    pub path_hint: String,
    pub policy: String,
    pub file_count: u64,
    pub bytes: u64,
    pub sample_names: Vec<String>,
    #[serde(default)]
    pub scope_id: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCategoryDelta {
    pub id: String,
    pub label: String,
    pub files_removed: u64,
    pub bytes_freed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskCleanupLastRun {
    pub at_utc: String,
    pub trigger: String,
    pub files_removed: u64,
    pub bytes_freed: u64,
    /// Soft-delete only: files left in place because Trash move failed (no permanent fallback).
    #[serde(default)]
    pub files_skipped: u64,
    pub categories: Vec<CleanupCategoryDelta>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskCleanupScope {
    pub id: String,
    /// `mac-stats` | `trash` | `downloads` | `temp` | `path`
    pub kind: String,
    pub label: String,
    pub enabled: bool,
    #[serde(default)]
    pub path: Option<String>,
    /// Age threshold for path-like scopes. `mac-stats` uses its own policies.
    #[serde(default)]
    pub max_age_days: Option<u32>,
    /// Walk subdirectories (path/trash/temp). Downloads default false (top-level files only).
    #[serde(default)]
    pub recursive: bool,
    /// Builtin scopes cannot be deleted from the UI (only disabled).
    #[serde(default)]
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskCleanupStatus {
    pub root_hint: String,
    pub reclaimable_bytes: u64,
    pub reclaimable_files: u64,
    pub categories: Vec<CleanupCategory>,
    pub scopes: Vec<DiskCleanupScope>,
    pub last_run: Option<DiskCleanupLastRun>,
    pub next_run_utc: Option<String>,
    pub next_run_label: String,
    pub interval_hours: u64,
    pub triggers: Vec<String>,
    pub enabled_scope_summary: String,
    /// When true (default), cleaned files are moved to `~/.Trash` instead of unlinked.
    /// The Trash scope itself is always permanently deleted.
    pub soft_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PersistedState {
    last_run: Option<DiskCleanupLastRun>,
    next_run_utc: Option<String>,
}

fn state_path() -> PathBuf {
    Config::config_file_path()
        .parent()
        .map(|p| p.join(STATE_FILE))
        .unwrap_or_else(|| std::env::temp_dir().join(STATE_FILE))
}

fn load_state() -> PersistedState {
    let path = state_path();
    let Ok(text) = fs::read_to_string(&path) else {
        return PersistedState::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn save_state(state: &PersistedState) {
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(pretty) = serde_json::to_string_pretty(state) {
        let _ = crate::config::write_text_atomic(&path, &pretty);
    }
}

fn read_config_value() -> Value {
    let path = Config::config_file_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return json!({});
    };
    serde_json::from_str(&content).unwrap_or(json!({}))
}

fn write_config_value(v: &Value) -> Result<(), String> {
    let path = Config::config_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    crate::config::write_text_atomic(&path, &pretty)
}

fn interval_hours() -> u64 {
    if let Ok(s) = std::env::var("MAC_STATS_DISK_CLEANUP_INTERVAL_HOURS") {
        if let Ok(n) = s.parse::<u64>() {
            return n.min(24 * 30);
        }
    }
    if let Some(n) = read_config_value()
        .get("diskCleanupIntervalHours")
        .and_then(|v| v.as_u64())
    {
        return n.min(24 * 30);
    }
    DEFAULT_INTERVAL_HOURS
}

/// Soft-delete (move to Trash) is the default. Set `diskCleanupSoftDelete: false` for permanent delete.
fn soft_delete_enabled() -> bool {
    if let Ok(s) = std::env::var("MAC_STATS_DISK_CLEANUP_SOFT_DELETE") {
        let t = s.trim().to_ascii_lowercase();
        return !(t == "0" || t == "false" || t == "no" || t == "off");
    }
    read_config_value()
        .get("diskCleanupSoftDelete")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

fn save_soft_delete(enabled: bool) -> Result<(), String> {
    let mut cfg = read_config_value();
    cfg["diskCleanupSoftDelete"] = json!(enabled);
    write_config_value(&cfg)
}

fn trash_dir() -> PathBuf {
    home_dir().join(".Trash")
}

/// Soft-delete target that stays inside `~/.mac-stats` (no macOS Downloads/Trash TCC prompts).
fn quarantine_dir() -> PathBuf {
    home_dir().join(".mac-stats").join("cleanup-quarantine")
}

/// Folders that trigger macOS “access files in …” prompts when scanned or written.
fn path_is_tcc_sensitive(path: &Path) -> bool {
    let home = home_dir();
    let sensitive = [
        home.join("Downloads"),
        home.join("Desktop"),
        home.join("Documents"),
        home.join(".Trash"),
    ];
    let Ok(canon) = path.canonicalize() else {
        let s = path.to_string_lossy();
        return sensitive.iter().any(|p| {
            s == p.to_string_lossy() || path.starts_with(p) || s.starts_with(&*p.to_string_lossy())
        });
    };
    sensitive.iter().any(|p| {
        let Ok(pc) = p.canonicalize() else {
            return canon.starts_with(p);
        };
        canon == pc || canon.starts_with(&pc)
    })
}

fn scope_is_tcc_sensitive(scope: &DiskCleanupScope) -> bool {
    match scope.kind.as_str() {
        "downloads" | "trash" => true,
        "path" => resolve_scope_roots(scope)
            .iter()
            .any(|(root, _)| path_is_tcc_sensitive(root)),
        _ => false,
    }
}

fn auto_trigger(trigger: &str) -> bool {
    matches!(trigger, "startup" | "periodic")
}

fn path_is_under_trash(path: &Path) -> bool {
    let trash = trash_dir();
    let Ok(canon) = path.canonicalize() else {
        return path.starts_with(&trash);
    };
    let Ok(trash_canon) = trash.canonicalize() else {
        return canon.starts_with(&trash);
    };
    canon.starts_with(&trash_canon)
}

/// Move `path` into `soft_root` (system Trash or app quarantine) with a collision-safe name.
fn move_to_soft_root(path: &Path, soft_root: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("not a file".into());
    }
    if path_is_under_trash(path) || path.starts_with(soft_root) {
        return fs::remove_file(path).map_err(|e| e.to_string());
    }
    fs::create_dir_all(soft_root).map_err(|e| e.to_string())?;
    let base = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| "missing file name".to_string())?;
    let mut dest = soft_root.join(&base);
    if dest.exists() {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| base.clone());
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        dest = soft_root.join(format!(
            "{} {}{}",
            stem,
            Utc::now().format("%Y-%m-%d %H.%M.%S"),
            ext
        ));
        let mut n = 2u32;
        while dest.exists() {
            dest = soft_root.join(format!(
                "{} {} ({}){}",
                stem,
                Utc::now().format("%Y-%m-%d %H.%M.%S"),
                n,
                ext
            ));
            n += 1;
            if n > 50 {
                break;
            }
        }
    }
    match fs::rename(path, &dest) {
        Ok(()) => Ok(()),
        Err(e) => {
            match fs::copy(path, &dest) {
                Ok(_) => match fs::remove_file(path) {
                    Ok(()) => Ok(()),
                    Err(rm_err) => {
                        let _ = fs::remove_file(&dest);
                        Err(format!(
                            "copy-to-soft-root ok but remove source failed: {rm_err}; rename was: {e}"
                        ))
                    }
                },
                Err(_) => Err(e.to_string()),
            }
        }
    }
}

#[allow(dead_code)]
fn move_to_trash(path: &Path) -> Result<(), String> {
    move_to_soft_root(path, &trash_dir())
}

/// Soft-delete moves to `soft_root`; permanent unlinks. Paths already in Trash are always unlinked.
///
/// When soft-delete is on and the move fails (EPERM, cross-volume, etc.), **skip** the file.
/// Never fall back to permanent delete — that would violate the user's recoverability choice.
fn remove_cleaned_file(path: &Path, soft: bool, soft_root: &Path) -> bool {
    if soft && !path_is_under_trash(path) {
        match move_to_soft_root(path, soft_root) {
            Ok(()) => true,
            Err(err) => {
                crate::mac_stats_debug!(
                    "disk_cleanup",
                    "Soft-delete failed for {:?}: {}; skipping (no permanent fallback)",
                    path,
                    err
                );
                false
            }
        }
    } else {
        fs::remove_file(path).is_ok()
    }
}

fn disposal_policy_suffix(soft: bool, force_permanent: bool) -> &'static str {
    if force_permanent || !soft {
        " · permanent delete"
    } else {
        " · move to Trash"
    }
}

fn default_scopes() -> Vec<DiskCleanupScope> {
    vec![
        DiskCleanupScope {
            id: "mac-stats".into(),
            kind: "mac-stats".into(),
            label: "mac-stats data".into(),
            enabled: true,
            path: Some("~/.mac-stats".into()),
            max_age_days: None,
            recursive: false,
            builtin: true,
        },
        DiskCleanupScope {
            id: "trash".into(),
            kind: "trash".into(),
            label: "Trash".into(),
            enabled: false,
            path: Some("~/.Trash".into()),
            max_age_days: Some(7),
            recursive: true,
            builtin: true,
        },
        DiskCleanupScope {
            id: "downloads".into(),
            kind: "downloads".into(),
            label: "Downloads".into(),
            enabled: false,
            path: Some("~/Downloads".into()),
            max_age_days: Some(90),
            recursive: false,
            builtin: true,
        },
        DiskCleanupScope {
            id: "temp".into(),
            kind: "temp".into(),
            label: "Temp".into(),
            enabled: false,
            path: None,
            max_age_days: Some(3),
            recursive: true,
            builtin: true,
        },
    ]
}

fn normalize_scope(mut s: DiskCleanupScope) -> Option<DiskCleanupScope> {
    s.id = s.id.trim().to_string();
    s.kind = s.kind.trim().to_lowercase();
    s.label = s.label.trim().to_string();
    if s.id.is_empty() || s.label.is_empty() {
        return None;
    }
    match s.kind.as_str() {
        "mac-stats" | "trash" | "downloads" | "temp" | "path" => {}
        _ => return None,
    }
    if let Some(days) = s.max_age_days {
        s.max_age_days = Some(days.min(3650));
    }
    if s.kind == "path" {
        let p = s.path.as_deref().unwrap_or("").trim();
        if p.is_empty() {
            return None;
        }
        s.path = Some(p.to_string());
        if s.max_age_days.unwrap_or(0) == 0 {
            s.max_age_days = Some(30);
        }
    }
    if matches!(s.kind.as_str(), "trash" | "downloads" | "temp") && s.max_age_days.unwrap_or(0) == 0
    {
        s.max_age_days = Some(match s.kind.as_str() {
            "trash" => 7,
            "downloads" => 90,
            _ => 3,
        });
    }
    Some(s)
}

fn merge_scopes(saved: Vec<DiskCleanupScope>) -> Vec<DiskCleanupScope> {
    let mut out = default_scopes();
    for s in saved {
        let Some(s) = normalize_scope(s) else {
            continue;
        };
        if let Some(existing) = out.iter_mut().find(|e| e.id == s.id) {
            // Preserve builtin flag; allow enable/path/age/label edits.
            existing.enabled = s.enabled;
            existing.label = s.label;
            if s.path.is_some() {
                existing.path = s.path;
            }
            if s.max_age_days.is_some() {
                existing.max_age_days = s.max_age_days;
            }
            existing.recursive = s.recursive;
        } else if s.kind == "path" {
            let mut custom = s;
            custom.builtin = false;
            out.push(custom);
        }
    }
    out
}

fn load_scopes() -> Vec<DiskCleanupScope> {
    let cfg = read_config_value();
    let saved = cfg
        .get("diskCleanupScopes")
        .cloned()
        .and_then(|v| serde_json::from_value::<Vec<DiskCleanupScope>>(v).ok())
        .unwrap_or_default();
    merge_scopes(saved)
}

fn save_scopes(scopes: &[DiskCleanupScope]) -> Result<(), String> {
    let mut cfg = read_config_value();
    let cleaned: Vec<DiskCleanupScope> = scopes.iter().cloned().filter_map(normalize_scope).collect();
    // Ensure builtins remain present
    let merged = merge_scopes(cleaned);
    cfg["diskCleanupScopes"] = serde_json::to_value(&merged).map_err(|e| e.to_string())?;
    write_config_value(&cfg)?;
    Ok(())
}

fn expand_user_path(raw: &str) -> PathBuf {
    let t = raw.trim();
    if t == "~" {
        return PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()));
    }
    if let Some(rest) = t.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(t)
}

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()))
}

fn path_is_forbidden(path: &Path) -> bool {
    let Ok(canon) = path.canonicalize() else {
        // Non-existent is ok for preview; still block obvious roots by string.
        let s = path.to_string_lossy();
        return matches!(
            s.as_ref(),
            "/" | "/System" | "/Applications" | "/usr" | "/bin" | "/sbin" | "/private"
        );
    };
    let home = home_dir();
    if canon == Path::new("/") {
        return true;
    }
    if canon == home {
        return true;
    }
    let forbidden_prefixes = [
        PathBuf::from("/System"),
        PathBuf::from("/Applications"),
        PathBuf::from("/usr"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/Library"),
        PathBuf::from("/private/var/db"),
    ];
    for p in &forbidden_prefixes {
        if canon == *p || canon.starts_with(p) {
            // Allow /tmp and /private/tmp via temp scope separately
            if canon.starts_with("/private/tmp") || canon.starts_with("/tmp") {
                continue;
            }
            if *p == PathBuf::from("/Library") && canon.starts_with(home.join("Library")) {
                continue;
            }
            return true;
        }
    }
    false
}

fn resolve_scope_roots(scope: &DiskCleanupScope) -> Vec<(PathBuf, String)> {
    match scope.kind.as_str() {
        "mac-stats" => vec![(
            Config::config_file_path()
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| home_dir().join(".mac-stats")),
            "~/.mac-stats".into(),
        )],
        "trash" => {
            let p = scope
                .path
                .as_deref()
                .map(expand_user_path)
                .unwrap_or_else(|| home_dir().join(".Trash"));
            vec![(p, "~/.Trash".into())]
        }
        "downloads" => {
            let raw = scope
                .path
                .clone()
                .unwrap_or_else(|| "~/Downloads".into());
            let p = expand_user_path(&raw);
            vec![(p, raw)]
        }
        "temp" => {
            let mut roots = vec![(
                std::env::temp_dir(),
                std::env::temp_dir().display().to_string(),
            )];
            let tmp = PathBuf::from("/tmp");
            if tmp.is_dir() && !roots.iter().any(|(p, _)| p == &tmp) {
                roots.push((tmp, "/tmp".into()));
            }
            roots
        }
        "path" => {
            let raw = scope.path.clone().unwrap_or_default();
            let p = expand_user_path(&raw);
            vec![(p, raw)]
        }
        _ => Vec::new(),
    }
}

fn system_time_to_utc(t: SystemTime) -> DateTime<Utc> {
    let Ok(dur) = t.duration_since(SystemTime::UNIX_EPOCH) else {
        return Utc::now();
    };
    DateTime::from_timestamp(dur.as_secs() as i64, dur.subsec_nanos()).unwrap_or_else(Utc::now)
}

pub(crate) fn format_bytes(n: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let x = n as f64;
    if x >= GB {
        format!("{:.1} GB", x / GB)
    } else if x >= MB {
        format!("{:.1} MB", x / MB)
    } else if x >= KB {
        format!("{:.0} KB", x / KB)
    } else {
        format!("{} B", n)
    }
}

fn push_sample(samples: &mut Vec<String>, name: String) {
    if samples.len() < 5 {
        samples.push(name);
    }
}

fn empty_cat(id: &str, label: &str, path: &str, policy: &str, scope_id: &str, enabled: bool) -> CleanupCategory {
    CleanupCategory {
        id: id.into(),
        label: label.into(),
        path_hint: path.into(),
        policy: policy.into(),
        file_count: 0,
        bytes: 0,
        sample_names: Vec::new(),
        scope_id: Some(scope_id.into()),
        enabled,
    }
}

/// Collect files older than `max_age_days` under `root` (files only; never deletes directories).
/// Skips root-owned / immutable / unlink-blocked files so preview matches what Clean now can remove.
fn collect_aged_files(root: &Path, max_age_days: u32, recursive: bool) -> Vec<(PathBuf, u64)> {
    if max_age_days == 0 || !root.is_dir() || path_is_forbidden(root) {
        return Vec::new();
    }
    let cutoff = SystemTime::now() - Duration::from_secs(u64::from(max_age_days) * 24 * 3600);
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let mut visited = 0usize;
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else {
            continue;
        };
        for ent in rd.flatten() {
            visited += 1;
            if visited > MAX_SCAN_FILES {
                return out;
            }
            let path = ent.path();
            // Never follow symlinks
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                if recursive {
                    if !path_is_forbidden(&path) {
                        stack.push(path);
                    }
                }
                continue;
            }
            if !meta.is_file() {
                continue;
            }
            if !file_is_user_reclaimable(&path, &meta) {
                continue;
            }
            let Ok(mtime) = meta.modified() else {
                continue;
            };
            if mtime < cutoff {
                out.push((path, meta.len()));
            }
        }
    }
    out
}

/// True when this process can likely unlink `path` (preview + Clean now).
/// Drops root-owned Microsoft AutoUpdate leftovers in `/var/folders/…/T` that
/// carry `uchg` and only produce soft-delete EPERM spam.
#[cfg(unix)]
fn file_is_user_reclaimable(path: &Path, meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    let uid = unsafe { libc::getuid() };
    if meta.uid() != uid {
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        // UF_IMMUTABLE (uchg) / SF_IMMUTABLE — cannot unlink without clearing flags as root.
        use std::os::unix::ffi::OsStrExt;
        const UF_IMMUTABLE: u32 = 0x0000_0002;
        const SF_IMMUTABLE: u32 = 0x0002_0000;
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if let Ok(c) = std::ffi::CString::new(path.as_os_str().as_bytes()) {
            if unsafe { libc::stat(c.as_ptr(), &mut st) } == 0
                && (st.st_flags & (UF_IMMUTABLE | SF_IMMUTABLE)) != 0
            {
                return false;
            }
        }
    }
    // Unlink needs write on the parent directory.
    if let Some(parent) = path.parent() {
        use std::os::unix::ffi::OsStrExt;
        if let Ok(c) = std::ffi::CString::new(parent.as_os_str().as_bytes()) {
            if unsafe { libc::access(c.as_ptr(), libc::W_OK) } != 0 {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

#[cfg(not(unix))]
fn file_is_user_reclaimable(_path: &Path, _meta: &fs::Metadata) -> bool {
    true
}

fn preview_aged_scope(scope: &DiskCleanupScope, touch_user_folders: bool) -> CleanupCategory {
    let soft = soft_delete_enabled();
    let force_permanent = scope.kind == "trash";
    let days = scope.max_age_days.unwrap_or(0);
    let policy = if !scope.enabled {
        "disabled".into()
    } else if days == 0 {
        "no age policy".into()
    } else {
        format!(
            "age > {}d{}{}",
            days,
            if scope.recursive {
                " · recursive"
            } else {
                " · top-level"
            },
            disposal_policy_suffix(soft, force_permanent)
        )
    };
    let roots = resolve_scope_roots(scope);
    let path_hint = roots
        .first()
        .map(|(_, h)| h.clone())
        .unwrap_or_else(|| scope.path.clone().unwrap_or_default());

    if !scope.enabled {
        return empty_cat(
            &scope.id,
            &scope.label,
            &path_hint,
            &policy,
            &scope.id,
            false,
        );
    }

    // Avoid macOS TCC prompts on routine status polls (Downloads / Trash / Desktop / Documents).
    if !touch_user_folders && scope_is_tcc_sensitive(scope) {
        return empty_cat(
            &scope.id,
            &scope.label,
            &path_hint,
            &format!("{policy} · not scanned (Refresh or Clean now)"),
            &scope.id,
            true,
        );
    }

    let mut marked = Vec::new();
    let mut blocked = false;
    for (root, _) in &roots {
        if path_is_forbidden(root) {
            blocked = true;
            continue;
        }
        marked.extend(collect_aged_files(root, days, scope.recursive));
    }

    let mut samples = Vec::new();
    let mut bytes = 0u64;
    for (p, sz) in &marked {
        bytes = bytes.saturating_add(*sz);
        if let Some(n) = p.file_name() {
            push_sample(&mut samples, n.to_string_lossy().into_owned());
        }
    }
    let policy = if blocked && marked.is_empty() {
        format!("{policy} · path blocked")
    } else {
        policy
    };
    CleanupCategory {
        id: scope.id.clone(),
        label: scope.label.clone(),
        path_hint,
        policy,
        file_count: marked.len() as u64,
        bytes,
        sample_names: samples,
        scope_id: Some(scope.id.clone()),
        enabled: true,
    }
}

/// Returns `(deleted, freed_bytes, soft_skips)`. Soft skips are soft-root move failures only.
fn apply_aged_scope(scope: &DiskCleanupScope, soft_root: &Path) -> (u64, u64, u64) {
    if !scope.enabled {
        return (0, 0, 0);
    }
    let days = scope.max_age_days.unwrap_or(0);
    // Trash scope empties old Trash entries — always permanent.
    let soft = soft_delete_enabled() && scope.kind != "trash";
    let mut deleted = 0u64;
    let mut freed = 0u64;
    let mut skipped = 0u64;
    for (root, _) in resolve_scope_roots(scope) {
        if path_is_forbidden(&root) {
            continue;
        }
        for (path, sz) in collect_aged_files(&root, days, scope.recursive) {
            if remove_cleaned_file(&path, soft, soft_root) {
                deleted += 1;
                freed = freed.saturating_add(sz);
            } else if soft {
                skipped += 1;
            }
        }
    }
    (deleted, freed, skipped)
}

fn collect_screenshot_reclaim() -> Vec<(PathBuf, u64)> {
    let dir = Config::screenshots_dir();
    let max_age_days = Config::screenshot_prune_max_age_days();
    let max_total = Config::screenshot_prune_max_total_bytes();
    let mut would: Vec<(PathBuf, u64, DateTime<Utc>)> = Vec::new();
    if !dir.is_dir() || (max_age_days == 0 && max_total == 0) {
        return Vec::new();
    }
    let Ok(rd) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let now = Utc::now();
    let cutoff = now - ChronoDuration::days(max_age_days as i64);
    let mut all: Vec<(PathBuf, u64, DateTime<Utc>)> = Vec::new();
    for ent in rd.flatten() {
        let path = ent.path();
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ts = parse_screenshot_filename_timestamp(&stem).unwrap_or_else(|| {
            system_time_to_utc(meta.modified().unwrap_or(SystemTime::UNIX_EPOCH))
        });
        all.push((path, meta.len(), ts));
    }
    if max_age_days > 0 {
        for (p, sz, ts) in &all {
            if *ts < cutoff {
                would.push((p.clone(), *sz, *ts));
            }
        }
    }
    if max_total > 0 {
        let mut rest: Vec<_> = all
            .into_iter()
            .filter(|(p, _, _)| !would.iter().any(|(w, _, _)| w == p))
            .collect();
        rest.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.cmp(&b.0)));
        let mut total: u64 = rest.iter().map(|e| e.1).sum();
        total = total.saturating_add(would.iter().map(|e| e.1).sum());
        for (p, sz, ts) in rest {
            if total <= max_total {
                break;
            }
            would.push((p, sz, ts));
            total = total.saturating_sub(sz);
        }
    }
    would.into_iter().map(|(p, sz, _)| (p, sz)).collect()
}

fn collect_pdf_reclaim() -> Vec<(PathBuf, u64)> {
    let dir = Config::pdfs_dir();
    let max_age_days = Config::screenshot_prune_max_age_days();
    let mut out = Vec::new();
    if !dir.is_dir() || max_age_days == 0 {
        return out;
    }
    let cutoff = Utc::now() - ChronoDuration::days(max_age_days as i64);
    let Ok(rd) = fs::read_dir(&dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            != Some(true)
        {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let ts = parse_screenshot_filename_timestamp(stem).or_else(|| {
            ent.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(system_time_to_utc)
        });
        let Some(ts) = ts else {
            continue;
        };
        if ts >= cutoff {
            continue;
        }
        let sz = ent.metadata().map(|m| m.len()).unwrap_or(0);
        out.push((path, sz));
    }
    out
}

fn collect_session_reclaim() -> Vec<(PathBuf, u64)> {
    let dir = Config::session_dir();
    let max_age_days = Config::session_prune_max_age_days();
    let max_files = Config::session_prune_max_files();
    let mut entries: Vec<(PathBuf, SystemTime, u64)> = Vec::new();
    if dir.is_dir() {
        if let Ok(rd) = fs::read_dir(&dir) {
            for ent in rd.flatten() {
                let path = ent.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with("session-memory-") || !name.ends_with(".md") {
                    continue;
                }
                let Ok(meta) = fs::metadata(&path) else {
                    continue;
                };
                if !meta.is_file() {
                    continue;
                }
                entries.push((
                    path,
                    meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    meta.len(),
                ));
            }
        }
    }
    let mut marked: Vec<(PathBuf, u64)> = Vec::new();
    let now = SystemTime::now();
    if max_age_days > 0 {
        let max_age = Duration::from_secs(u64::from(max_age_days) * 24 * 3600);
        entries.retain(|(path, mtime, size)| {
            let too_old = now
                .duration_since(*mtime)
                .map(|d| d > max_age)
                .unwrap_or(false);
            if too_old {
                marked.push((path.clone(), *size));
                false
            } else {
                true
            }
        });
    }
    if max_files > 0 && entries.len() > max_files {
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (path, _, size) in entries.into_iter().skip(max_files) {
            marked.push((path, size));
        }
    }
    marked
}

fn collect_browser_download_reclaim() -> Vec<(PathBuf, u64)> {
    let dir = Config::browser_downloads_dir();
    let max_age = Duration::from_secs(24 * 3600);
    let mut out = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    let now = SystemTime::now();
    let Ok(rd) = fs::read_dir(&dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let Ok(mt) = meta.modified() else {
            continue;
        };
        let age = now.duration_since(mt).unwrap_or(Duration::ZERO);
        if age > max_age {
            out.push((path, meta.len()));
        }
    }
    out
}

fn collect_sic_backup_reclaim() -> Vec<(PathBuf, u64)> {
    let dir = Config::config_file_path()
        .parent()
        .map(|p| p.join("sic"))
        .unwrap_or_else(|| PathBuf::from("sic"));
    let mut out = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    const MAX_AGE_SECS: u64 = 14 * 24 * 60 * 60;
    let Ok(rd) = fs::read_dir(&dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with("debug.log.") {
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if now.saturating_sub(mtime) > MAX_AGE_SECS {
            out.push((path, meta.len()));
        }
    }
    out
}

fn preview_screenshots(scope_id: &str, enabled: bool) -> CleanupCategory {
    let soft = soft_delete_enabled();
    let max_age_days = Config::screenshot_prune_max_age_days();
    let max_total = Config::screenshot_prune_max_total_bytes();
    let policy = if !enabled {
        "disabled (scope off)".to_string()
    } else if max_age_days == 0 && max_total == 0 {
        "disabled".to_string()
    } else {
        let mut parts = Vec::new();
        if max_age_days > 0 {
            parts.push(format!("age > {}d", max_age_days));
        }
        if max_total > 0 {
            parts.push(format!("cap {}", format_bytes(max_total)));
        }
        parts.join(" · ") + disposal_policy_suffix(soft, false)
    };

    if !enabled {
        return empty_cat(
            "screenshots",
            "Screenshots",
            "~/.mac-stats/screenshots",
            &policy,
            scope_id,
            false,
        );
    }

    let would = collect_screenshot_reclaim();
    let mut samples = Vec::new();
    let mut bytes = 0u64;
    for (p, sz) in &would {
        bytes = bytes.saturating_add(*sz);
        if let Some(n) = p.file_name() {
            push_sample(&mut samples, n.to_string_lossy().into_owned());
        }
    }
    CleanupCategory {
        id: "screenshots".into(),
        label: "Screenshots".into(),
        path_hint: "~/.mac-stats/screenshots".into(),
        policy,
        file_count: would.len() as u64,
        bytes,
        sample_names: samples,
        scope_id: Some(scope_id.into()),
        enabled,
    }
}

fn preview_pdfs(scope_id: &str, enabled: bool) -> CleanupCategory {
    let soft = soft_delete_enabled();
    let max_age_days = Config::screenshot_prune_max_age_days();
    let policy = if !enabled {
        "disabled (scope off)".into()
    } else if max_age_days == 0 {
        "disabled".into()
    } else {
        format!("age > {}d{}", max_age_days, disposal_policy_suffix(soft, false))
    };
    if !enabled {
        return empty_cat("pdfs", "PDFs", "~/.mac-stats/pdfs", &policy, scope_id, false);
    }
    let marked = collect_pdf_reclaim();
    let mut samples = Vec::new();
    let mut bytes = 0u64;
    for (p, sz) in &marked {
        bytes = bytes.saturating_add(*sz);
        if let Some(n) = p.file_name() {
            push_sample(&mut samples, n.to_string_lossy().into_owned());
        }
    }
    CleanupCategory {
        id: "pdfs".into(),
        label: "PDFs".into(),
        path_hint: "~/.mac-stats/pdfs".into(),
        policy,
        file_count: marked.len() as u64,
        bytes,
        sample_names: samples,
        scope_id: Some(scope_id.into()),
        enabled,
    }
}

fn preview_sessions(scope_id: &str, enabled: bool) -> CleanupCategory {
    let dir = Config::session_dir();
    let max_age_days = Config::session_prune_max_age_days();
    let max_files = Config::session_prune_max_files();
    let mut parts = Vec::new();
    if max_age_days > 0 {
        parts.push(format!("age > {}d", max_age_days));
    }
    if max_files > 0 {
        parts.push(format!("keep newest {}", max_files));
    }
    let policy = if !enabled {
        "disabled (scope off)".into()
    } else if parts.is_empty() {
        "disabled".into()
    } else {
        parts.join(" · ")
    };
    if !enabled {
        return empty_cat(
            "sessions",
            "Session transcripts",
            "~/.mac-stats/session",
            &policy,
            scope_id,
            false,
        );
    }

    let mut entries: Vec<(PathBuf, SystemTime, u64)> = Vec::new();
    if dir.is_dir() {
        if let Ok(rd) = fs::read_dir(&dir) {
            for ent in rd.flatten() {
                let path = ent.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with("session-memory-") || !name.ends_with(".md") {
                    continue;
                }
                let Ok(meta) = fs::metadata(&path) else {
                    continue;
                };
                if !meta.is_file() {
                    continue;
                }
                entries.push((
                    path,
                    meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    meta.len(),
                ));
            }
        }
    }

    let mut marked: Vec<(PathBuf, u64)> = Vec::new();
    let now = SystemTime::now();
    if max_age_days > 0 {
        let max_age = Duration::from_secs(u64::from(max_age_days) * 24 * 3600);
        entries.retain(|(path, mtime, size)| {
            let too_old = now
                .duration_since(*mtime)
                .map(|d| d > max_age)
                .unwrap_or(false);
            if too_old {
                marked.push((path.clone(), *size));
                false
            } else {
                true
            }
        });
    }
    if max_files > 0 && entries.len() > max_files {
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (path, _, size) in entries.into_iter().skip(max_files) {
            marked.push((path, size));
        }
    }

    let mut samples = Vec::new();
    let mut bytes = 0u64;
    for (p, sz) in &marked {
        bytes = bytes.saturating_add(*sz);
        if let Some(n) = p.file_name() {
            push_sample(&mut samples, n.to_string_lossy().into_owned());
        }
    }
    CleanupCategory {
        id: "sessions".into(),
        label: "Session transcripts".into(),
        path_hint: "~/.mac-stats/session".into(),
        policy,
        file_count: marked.len() as u64,
        bytes,
        sample_names: samples,
        scope_id: Some(scope_id.into()),
        enabled,
    }
}

fn preview_runs_jsonl(scope_id: &str, enabled: bool) -> CleanupCategory {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    let max = Config::runs_prune_max_lines();
    let policy = if !enabled {
        "disabled (scope off)".into()
    } else if max == 0 {
        "disabled".into()
    } else {
        format!("keep newest {} lines", max)
    };
    if !enabled {
        return empty_cat(
            "runs",
            "Agent run log",
            "~/.mac-stats/runs.jsonl",
            &policy,
            scope_id,
            false,
        );
    }
    let mut file_count = 0u64;
    let mut bytes = 0u64;
    let mut samples = Vec::new();
    if max > 0 {
        if let Ok(text) = fs::read_to_string(&path) {
            let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.len() > max {
                let excess = &lines[..lines.len() - max];
                file_count = excess.len() as u64;
                bytes = excess.iter().map(|l| l.len() as u64 + 1).sum();
                push_sample(
                    &mut samples,
                    format!("{} excess line(s) in runs.jsonl", excess.len()),
                );
            }
        }
    }
    CleanupCategory {
        id: "runs".into(),
        label: "Agent run log".into(),
        path_hint: "~/.mac-stats/runs.jsonl".into(),
        policy,
        file_count,
        bytes,
        sample_names: samples,
        scope_id: Some(scope_id.into()),
        enabled,
    }
}

fn preview_browser_downloads(scope_id: &str, enabled: bool) -> CleanupCategory {
    let dir = Config::browser_downloads_dir();
    let max_age = Duration::from_secs(24 * 3600);
    let policy = if !enabled {
        "disabled (scope off)".to_string()
    } else {
        "age > 24h".to_string()
    };
    if !enabled {
        return empty_cat(
            "browser_downloads",
            "Browser downloads",
            "~/.mac-stats/browser-downloads",
            &policy,
            scope_id,
            false,
        );
    }
    let mut count = 0u64;
    let mut bytes = 0u64;
    let mut samples = Vec::new();
    if dir.is_dir() {
        let now = SystemTime::now();
        if let Ok(rd) = fs::read_dir(&dir) {
            for ent in rd.flatten() {
                let path = ent.path();
                let Ok(meta) = fs::metadata(&path) else {
                    continue;
                };
                if !meta.is_file() {
                    continue;
                }
                let Ok(mt) = meta.modified() else {
                    continue;
                };
                let age = now.duration_since(mt).unwrap_or(Duration::ZERO);
                if age > max_age {
                    count += 1;
                    bytes = bytes.saturating_add(meta.len());
                    if let Some(n) = path.file_name() {
                        push_sample(&mut samples, n.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    CleanupCategory {
        id: "browser_downloads".into(),
        label: "Browser downloads".into(),
        path_hint: "~/.mac-stats/browser-downloads".into(),
        policy,
        file_count: count,
        bytes,
        sample_names: samples,
        scope_id: Some(scope_id.into()),
        enabled,
    }
}

fn preview_sic_backups(scope_id: &str, enabled: bool) -> CleanupCategory {
    let dir = Config::config_file_path()
        .parent()
        .map(|p| p.join("sic"))
        .unwrap_or_else(|| PathBuf::from("sic"));
    let policy = if !enabled {
        "disabled (scope off)".to_string()
    } else {
        "age > 14d".to_string()
    };
    if !enabled {
        return empty_cat(
            "log_backups",
            "Old log backups",
            "~/.mac-stats/sic/",
            &policy,
            scope_id,
            false,
        );
    }
    let mut count = 0u64;
    let mut bytes = 0u64;
    let mut samples = Vec::new();
    if dir.is_dir() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        const MAX_AGE_SECS: u64 = 14 * 24 * 60 * 60;
        if let Ok(rd) = fs::read_dir(&dir) {
            for ent in rd.flatten() {
                let path = ent.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with("debug.log.") {
                    continue;
                }
                let Ok(meta) = fs::metadata(&path) else {
                    continue;
                };
                if !meta.is_file() {
                    continue;
                }
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if now.saturating_sub(mtime) > MAX_AGE_SECS {
                    count += 1;
                    bytes = bytes.saturating_add(meta.len());
                    push_sample(&mut samples, name.to_string());
                }
            }
        }
    }
    CleanupCategory {
        id: "log_backups".into(),
        label: "Old log backups".into(),
        path_hint: "~/.mac-stats/sic/".into(),
        policy,
        file_count: count,
        bytes,
        sample_names: samples,
        scope_id: Some(scope_id.into()),
        enabled,
    }
}

fn preview_mac_stats_scope(scope: &DiskCleanupScope) -> Vec<CleanupCategory> {
    let on = scope.enabled;
    let id = scope.id.as_str();
    vec![
        preview_screenshots(id, on),
        preview_pdfs(id, on),
        preview_sessions(id, on),
        preview_runs_jsonl(id, on),
        preview_browser_downloads(id, on),
        preview_sic_backups(id, on),
    ]
}

fn build_preview_categories(
    scopes: &[DiskCleanupScope],
    touch_user_folders: bool,
) -> Vec<CleanupCategory> {
    let mut out = Vec::new();
    for scope in scopes {
        match scope.kind.as_str() {
            "mac-stats" => out.extend(preview_mac_stats_scope(scope)),
            "trash" | "downloads" | "temp" | "path" => {
                out.push(preview_aged_scope(scope, touch_user_folders))
            }
            _ => {}
        }
    }
    out
}

/// Soft path returns count of files left in place when soft-root move failed.
fn apply_mac_stats_scope(soft: bool, soft_root: &Path) -> u64 {
    if soft {
        // Move reclaimable app data to soft_root (Trash or in-app quarantine).
        let mut skipped = 0u64;
        for (path, _) in collect_screenshot_reclaim() {
            if !remove_cleaned_file(&path, true, soft_root) {
                skipped += 1;
            }
        }
        for (path, _) in collect_pdf_reclaim() {
            if !remove_cleaned_file(&path, true, soft_root) {
                skipped += 1;
            }
        }
        for (path, _) in collect_session_reclaim() {
            if !remove_cleaned_file(&path, true, soft_root) {
                skipped += 1;
            }
        }
        for (path, _) in collect_browser_download_reclaim() {
            if !remove_cleaned_file(&path, true, soft_root) {
                skipped += 1;
            }
        }
        for (path, _) in collect_sic_backup_reclaim() {
            if !remove_cleaned_file(&path, true, soft_root) {
                skipped += 1;
            }
        }
        let _ = crate::commands::run_telemetry::prune_runs_jsonl_if_needed();
        crate::browser_agent::prune_cdp_traces_best_effort();
        crate::logging::prune_companion_logs_best_effort();
        return skipped;
    }
    crate::commands::screenshot_lifecycle::prune_old_screenshots();
    crate::commands::screenshot_lifecycle::prune_old_pdfs();
    let _ = crate::session_memory::prune_old_session_files();
    let _ = crate::commands::run_telemetry::prune_runs_jsonl_if_needed();
    crate::browser_agent::cdp_downloads::prune_old_browser_downloads(Duration::from_secs(
        24 * 3600,
    ));
    crate::browser_agent::prune_cdp_traces_best_effort();
    crate::logging::prune_companion_logs_best_effort();
    0
}

fn compute_next_run_utc(last: Option<&DiskCleanupLastRun>, hours: u64) -> Option<DateTime<Utc>> {
    if hours == 0 {
        return None;
    }
    let base = last
        .and_then(|r| DateTime::parse_from_rfc3339(&r.at_utc).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    Some(base + ChronoDuration::hours(hours as i64))
}

fn next_run_label(next: Option<DateTime<Utc>>, hours: u64) -> String {
    if hours == 0 {
        return "Next app launch only (periodic off)".into();
    }
    let Some(next) = next else {
        return "Next app launch".into();
    };
    let now = Utc::now();
    if next <= now {
        return "Due now (or next app launch)".into();
    }
    let mins = (next - now).num_minutes().max(1);
    if mins < 90 {
        format!("In ~{} min · also on app launch", mins)
    } else if mins < 60 * 36 {
        format!("In ~{} h · also on app launch", (mins + 30) / 60)
    } else {
        format!(
            "{} · also on app launch",
            next.format("%Y-%m-%d %H:%M UTC")
        )
    }
}

fn deltas_from_preview(
    before: &[CleanupCategory],
    after: &[CleanupCategory],
) -> Vec<CleanupCategoryDelta> {
    let mut out = Vec::new();
    for b in before {
        let a = after.iter().find(|c| c.id == b.id);
        let (af, ab) = a.map(|c| (c.file_count, c.bytes)).unwrap_or((0, 0));
        let files = b.file_count.saturating_sub(af);
        let bytes = b.bytes.saturating_sub(ab);
        if files > 0 || bytes > 0 {
            out.push(CleanupCategoryDelta {
                id: b.id.clone(),
                label: b.label.clone(),
                files_removed: files,
                bytes_freed: bytes,
            });
        }
    }
    out
}

fn enabled_scope_summary(scopes: &[DiskCleanupScope]) -> String {
    let enabled: Vec<&str> = scopes
        .iter()
        .filter(|s| s.enabled)
        .map(|s| s.label.as_str())
        .collect();
    if enabled.is_empty() {
        "None enabled".into()
    } else {
        enabled.join(" · ")
    }
}

fn build_status_from(
    state: PersistedState,
    scopes: Vec<DiskCleanupScope>,
    categories: Vec<CleanupCategory>,
) -> DiskCleanupStatus {
    let hours = interval_hours();
    let reclaimable_bytes: u64 = categories
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.bytes)
        .sum();
    let reclaimable_files: u64 = categories
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.file_count)
        .sum();
    let next_dt = compute_next_run_utc(state.last_run.as_ref(), hours);
    let next_run_utc = next_dt.map(|d| d.to_rfc3339());
    let next_run_label = next_run_label(next_dt, hours);
    let summary = enabled_scope_summary(&scopes);
    let soft = soft_delete_enabled();
    DiskCleanupStatus {
        root_hint: summary.clone(),
        reclaimable_bytes,
        reclaimable_files,
        categories,
        scopes,
        last_run: state.last_run,
        next_run_utc,
        next_run_label,
        interval_hours: hours,
        triggers: vec![
            "App launch (mac-stats data only; no Downloads/Trash scan)".into(),
            if hours > 0 {
                format!("Every {}h while running (mac-stats data only)", hours)
            } else {
                "Periodic: off".into()
            },
            "Manual Clean now (all enabled scopes)".into(),
        ],
        enabled_scope_summary: summary,
        soft_delete: soft,
    }
}

pub fn get_status(touch_user_folders: bool) -> DiskCleanupStatus {
    let state = load_state();
    let scopes = load_scopes();
    let categories = build_preview_categories(&scopes, touch_user_folders);
    build_status_from(state, scopes, categories)
}

fn soft_skip_note_suffix(skipped: u64) -> String {
    if skipped == 0 {
        String::new()
    } else {
        format!(
            "; skipped {} (could not soft-delete — left in place)",
            skipped
        )
    }
}

pub fn run_now(trigger: &str) -> DiskCleanupStatus {
    let scopes = load_scopes();
    let is_auto = auto_trigger(trigger);
    // Auto runs must not touch Downloads/Trash (macOS TCC). Soft-delete goes to
    // ~/.mac-stats/cleanup-quarantine instead of ~/.Trash.
    let soft_root = if is_auto {
        quarantine_dir()
    } else {
        trash_dir()
    };
    let touch_preview = !is_auto;
    let before = build_preview_categories(&scopes, touch_preview);
    let mut files_skipped = 0u64;
    let mut skipped_tcc_scopes = 0u32;

    for scope in &scopes {
        if !scope.enabled {
            continue;
        }
        if is_auto && scope_is_tcc_sensitive(scope) {
            skipped_tcc_scopes += 1;
            continue;
        }
        match scope.kind.as_str() {
            "mac-stats" => {
                files_skipped = files_skipped
                    .saturating_add(apply_mac_stats_scope(soft_delete_enabled(), &soft_root));
            }
            "trash" | "downloads" | "temp" | "path" => {
                let (d, f, sk) = apply_aged_scope(scope, &soft_root);
                files_skipped = files_skipped.saturating_add(sk);
                if d > 0 || sk > 0 {
                    mac_stats_info!(
                        "disk_cleanup",
                        "Scope {}: removed {} file(s), freed {} byte(s), soft-skipped {}",
                        scope.id,
                        d,
                        f,
                        sk
                    );
                }
            }
            _ => {}
        }
    }

    let after = build_preview_categories(&scopes, touch_preview);
    let categories = deltas_from_preview(&before, &after);
    let files_removed: u64 = categories.iter().map(|c| c.files_removed).sum();
    let bytes_freed: u64 = categories.iter().map(|c| c.bytes_freed).sum();

    let hours = interval_hours();
    let at = Utc::now();
    let soft_dest_note = if is_auto && soft_delete_enabled() {
        " · auto soft-delete → ~/.mac-stats/cleanup-quarantine"
    } else {
        ""
    };
    let tcc_skip_note = if skipped_tcc_scopes > 0 {
        format!(
            " Skipped {} Downloads/Trash/path scope(s) on auto run (use Clean now).",
            skipped_tcc_scopes
        )
    } else {
        String::new()
    };
    let last = DiskCleanupLastRun {
        at_utc: at.to_rfc3339(),
        trigger: trigger.to_string(),
        files_removed,
        bytes_freed,
        files_skipped,
        categories,
        note: if files_removed == 0 && bytes_freed == 0 && files_skipped == 0 {
            Some(format!(
                "Nothing to clean under enabled scopes.{}{}",
                soft_dest_note, tcc_skip_note
            ))
        } else if files_removed == 0 && bytes_freed == 0 {
            Some(format!(
                "Nothing moved; skipped {} (could not soft-delete — left in place).{}{}",
                files_skipped, soft_dest_note, tcc_skip_note
            ))
        } else {
            let how = if soft_delete_enabled() {
                if is_auto {
                    "Moved to cleanup-quarantine"
                } else {
                    "Moved to Trash"
                }
            } else {
                "Permanently deleted"
            };
            Some(format!(
                "{} {} across {} item(s) (Trash scope always permanent){}.{}{}",
                how,
                format_bytes(bytes_freed),
                files_removed,
                soft_skip_note_suffix(files_skipped),
                soft_dest_note,
                tcc_skip_note
            ))
        },
    };
    let next = if hours > 0 {
        Some((at + ChronoDuration::hours(hours as i64)).to_rfc3339())
    } else {
        None
    };
    let state = PersistedState {
        last_run: Some(last),
        next_run_utc: next,
    };
    save_state(&state);

    mac_stats_info!(
        "disk_cleanup",
        "Disk cleanup ({}): removed {} item(s), freed {} byte(s), soft-skipped {}, tcc-scopes-skipped {}",
        trigger,
        files_removed,
        bytes_freed,
        files_skipped,
        skipped_tcc_scopes
    );

    build_status_from(state, scopes, after)
}

pub fn run_if_due() {
    let hours = interval_hours();
    if hours == 0 {
        return;
    }
    let state = load_state();
    let due = match state.next_run_utc.as_deref() {
        Some(s) => DateTime::parse_from_rfc3339(s)
            .map(|d| d.with_timezone(&Utc) <= Utc::now())
            .unwrap_or(true),
        None => false,
    };
    if due {
        let _ = run_now("periodic");
    }
}

#[tauri::command]
pub fn get_disk_cleanup_status(deep: Option<bool>) -> DiskCleanupStatus {
    get_status(deep.unwrap_or(false))
}

#[tauri::command]
pub fn run_disk_cleanup_now() -> DiskCleanupStatus {
    run_now("manual")
}

#[tauri::command]
pub fn get_disk_cleanup_scopes() -> Vec<DiskCleanupScope> {
    load_scopes()
}

#[tauri::command]
pub fn set_disk_cleanup_scopes(scopes: Vec<DiskCleanupScope>) -> Result<DiskCleanupStatus, String> {
    save_scopes(&scopes)?;
    Ok(get_status(false))
}

#[tauri::command]
pub fn set_disk_cleanup_soft_delete(soft_delete: bool) -> Result<DiskCleanupStatus, String> {
    save_soft_delete(soft_delete)?;
    Ok(get_status(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_scales() {
        assert!(format_bytes(500).contains('B'));
        assert!(format_bytes(5_000).contains("KB"));
        assert!(format_bytes(5_000_000).contains("MB"));
    }

    #[test]
    fn next_label_periodic_off() {
        assert!(next_run_label(None, 0).contains("off"));
    }

    #[test]
    fn merge_keeps_builtins_and_customs() {
        let saved = vec![DiskCleanupScope {
            id: "my-cache".into(),
            kind: "path".into(),
            label: "My cache".into(),
            enabled: true,
            path: Some("~/Library/Caches/foo".into()),
            max_age_days: Some(14),
            recursive: true,
            builtin: false,
        }];
        let merged = merge_scopes(saved);
        assert!(merged.iter().any(|s| s.id == "mac-stats"));
        assert!(merged.iter().any(|s| s.id == "trash"));
        assert!(merged.iter().any(|s| s.id == "my-cache" && s.enabled));
    }

    #[test]
    fn forbid_home_and_root() {
        assert!(path_is_forbidden(Path::new("/")));
        assert!(path_is_forbidden(&home_dir()));
    }

    #[test]
    fn soft_delete_skips_when_trash_move_fails() {
        // Soft path must not unlink when soft-root move fails (recoverability).
        let dir = std::env::temp_dir().join(format!(
            "mac-stats-soft-skip-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("kept.txt");
        fs::write(&file, b"keep-me").expect("write");
        let not_a_file = dir.join("subdir");
        fs::create_dir_all(&not_a_file).expect("subdir");
        let soft_root = dir.join("soft-root");
        fs::create_dir_all(&soft_root).expect("soft root");
        assert!(
            !remove_cleaned_file(&not_a_file, true, &soft_root),
            "soft-delete must skip when move fails"
        );
        assert!(file.exists(), "unrelated file must remain");
        assert!(
            not_a_file.exists(),
            "failed soft target must remain (no permanent fallback)"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn user_owned_temp_file_is_reclaimable() {
        let dir = std::env::temp_dir().join(format!(
            "mac-stats-reclaim-ok-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("ok.txt");
        fs::write(&file, b"x").expect("write");
        let meta = fs::symlink_metadata(&file).expect("meta");
        assert!(file_is_user_reclaimable(&file, &meta));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn permanent_delete_unlinks_file() {
        let dir = std::env::temp_dir().join(format!(
            "mac-stats-perm-del-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("gone.txt");
        fs::write(&file, b"bye").expect("write");
        let soft_root = dir.join("soft-root");
        assert!(remove_cleaned_file(&file, false, &soft_root));
        assert!(!file.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn soft_skip_note_suffix_formats() {
        assert_eq!(soft_skip_note_suffix(0), "");
        assert!(soft_skip_note_suffix(2).contains("skipped 2"));
        assert!(soft_skip_note_suffix(1).contains("left in place"));
    }

    #[test]
    fn last_run_deserializes_without_files_skipped() {
        let v = serde_json::json!({
            "atUtc": "2026-08-14T00:00:00Z",
            "trigger": "manual",
            "filesRemoved": 1,
            "bytesFreed": 10,
            "categories": [],
            "note": "ok"
        });
        let last: DiskCleanupLastRun = serde_json::from_value(v).expect("parse");
        assert_eq!(last.files_skipped, 0);
        assert_eq!(last.files_removed, 1);
    }
}
