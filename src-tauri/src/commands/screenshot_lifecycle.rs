//! Screenshot path markers for agent-router history and age/size pruning under `~/.mac-stats/screenshots/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Duration as ChronoDuration, NaiveDateTime, TimeZone, Utc};

use crate::config::Config;
use crate::mac_stats_info;

/// Parse leading `YYYYMMDD_HHMMSS` from screenshot basename (e.g. `20260321_143000_current.png`).
pub(crate) fn parse_screenshot_filename_timestamp(stem: &str) -> Option<DateTime<Utc>> {
    let prefix = stem.get(..15)?;
    if prefix.as_bytes().get(8) != Some(&b'_') {
        return None;
    }
    let date = prefix.get(..8)?;
    let time = prefix.get(9..15)?;
    if !date.chars().all(|c| c.is_ascii_digit()) || !time.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let naive =
        NaiveDateTime::parse_from_str(&format!("{}_{}", date, time), "%Y%m%d_%H%M%S").ok()?;
    Some(naive.and_utc())
}

/// Append `[screenshot: <path>]` lines for paths not already present in `text` (dedup).
pub(crate) fn append_screenshot_path_markers_dedup(text: &mut String, paths: &[PathBuf]) {
    for p in paths {
        let line = format!("[screenshot: {}]", p.display());
        if text.contains(&line) {
            continue;
        }
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&line);
        text.push('\n');
    }
}

struct ScreenshotEntry {
    path: PathBuf,
    size: u64,
    name_ts: Option<DateTime<Utc>>,
    mtime: SystemTime,
}

fn collect_screenshot_entries(dir: &Path) -> Vec<ScreenshotEntry> {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for ent in read_dir.flatten() {
        let path = ent.path();
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let Some(stem) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
            continue;
        };
        let name_ts = parse_screenshot_filename_timestamp(&stem);
        let mtime = meta.modified().unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
        out.push(ScreenshotEntry {
            path,
            size: meta.len(),
            name_ts,
            mtime,
        });
    }
    out
}

fn system_time_to_utc(t: SystemTime) -> DateTime<Utc> {
    let Ok(dur) = t.duration_since(SystemTime::UNIX_EPOCH) else {
        return Utc::now();
    };
    Utc.timestamp_opt(dur.as_secs() as i64, dur.subsec_nanos())
        .single()
        .unwrap_or_else(Utc::now)
}

/// Delete screenshots older than configured age (filename timestamp); then enforce optional total size cap (oldest first).
/// Logs at info when anything was removed.
pub fn prune_old_screenshots() {
    let dir = Config::screenshots_dir();
    if !dir.is_dir() {
        return;
    }

    let max_age_days = Config::screenshot_prune_max_age_days();
    let max_total_bytes = Config::screenshot_prune_max_total_bytes();
    if max_age_days == 0 && max_total_bytes == 0 {
        return;
    }

    let now = Utc::now();
    let mut deleted = 0u64;
    let mut freed: u64 = 0;

    // --- Age-based prune (filename timestamp only; unknown patterns skipped) ---
    if max_age_days > 0 {
        let cutoff = now - ChronoDuration::days(max_age_days as i64);
        let entries = collect_screenshot_entries(&dir);
        for e in entries {
            if let Some(ts) = e.name_ts {
                if ts < cutoff {
                    match fs::remove_file(&e.path) {
                        Ok(()) => {
                            deleted += 1;
                            freed = freed.saturating_add(e.size);
                        }
                        Err(err) => {
                            crate::mac_stats_debug!(
                                "screenshots/prune",
                                "Failed to remove old screenshot {:?}: {}",
                                e.path,
                                err
                            );
                        }
                    }
                }
            }
        }
    }

    // --- Total size cap (oldest first by filename ts, else mtime) ---
    if max_total_bytes > 0 {
        let mut entries = collect_screenshot_entries(&dir);
        entries.sort_by(|a, b| {
            let ta = a.name_ts.unwrap_or_else(|| system_time_to_utc(a.mtime));
            let tb = b.name_ts.unwrap_or_else(|| system_time_to_utc(b.mtime));
            ta.cmp(&tb).then_with(|| a.path.cmp(&b.path))
        });

        let mut total: u64 = entries.iter().map(|e| e.size).sum();
        for e in entries {
            if total <= max_total_bytes {
                break;
            }
            match fs::remove_file(&e.path) {
                Ok(()) => {
                    deleted += 1;
                    freed = freed.saturating_add(e.size);
                    total = total.saturating_sub(e.size);
                }
                Err(err) => {
                    crate::mac_stats_debug!(
                        "screenshots/prune",
                        "Failed to remove screenshot for size cap {:?}: {}",
                        e.path,
                        err
                    );
                }
            }
        }
    }

    if deleted > 0 {
        mac_stats_info!(
            "screenshots/prune",
            "Screenshot prune: removed {} file(s), freed {} byte(s) (age_days_cap={} size_cap_bytes={})",
            deleted,
            freed,
            max_age_days,
            max_total_bytes
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timestamp_from_stem() {
        let ts = parse_screenshot_filename_timestamp("20260321_143000_current").unwrap();
        assert_eq!(ts.format("%Y%m%d_%H%M%S").to_string(), "20260321_143000");
    }

    #[test]
    fn parse_rejects_bad_stem() {
        assert!(parse_screenshot_filename_timestamp("nope").is_none());
        assert!(parse_screenshot_filename_timestamp("20260321143000_x").is_none());
    }

    #[test]
    fn append_markers_dedup() {
        let mut t = String::from("hello");
        let p = PathBuf::from("/tmp/a.png");
        append_screenshot_path_markers_dedup(&mut t, std::slice::from_ref(&p));
        assert!(t.contains("[screenshot: /tmp/a.png]"));
        append_screenshot_path_markers_dedup(&mut t, std::slice::from_ref(&p));
        let n = t.matches("[screenshot:").count();
        assert_eq!(n, 1);
    }
}
