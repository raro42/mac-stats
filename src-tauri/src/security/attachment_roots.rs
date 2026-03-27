//! Central allowlist for outbound Discord (and similar) file attachments.
//!
//! Paths must lie under canonicalized directory roots: default mac-stats artifact
//! trees plus optional `config.json` entries. This blocks arbitrary filesystem reads
//! when tool output supplies a path string.

use std::path::{Path, PathBuf};

use crate::config::Config;

/// Ordered built-in roots under `~/.mac-stats/` used for outbound attachments.
/// Entries that do not yet exist on disk are skipped until created (same as prior screenshots-only behaviour).
fn default_mac_stats_attachment_subtrees() -> [PathBuf; 3] {
    [
        Config::screenshots_dir(),
        Config::pdfs_dir(),
        Config::browser_downloads_dir(),
    ]
}

fn canonical_home_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    PathBuf::from(home).canonicalize().ok()
}

fn canonical_mac_stats_state_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    PathBuf::from(home).join(".mac-stats").canonicalize().ok()
}

/// Canonical directory roots that may contain attachable files.
pub fn canonical_outbound_attachment_root_dirs() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    for p in default_mac_stats_attachment_subtrees() {
        if let Ok(c) = p.canonicalize() {
            if c.is_dir() {
                roots.push(c);
            }
        }
    }

    let home_canon = canonical_home_dir();
    let mac_stats_canon = canonical_mac_stats_state_dir();

    for p in Config::extra_attachment_roots() {
        if let Ok(c) = p.canonicalize() {
            if !c.is_dir() {
                tracing::debug!("extraAttachmentRoots: skip non-directory {}", c.display());
                continue;
            }
            let allowed_base = match (&mac_stats_canon, &home_canon) {
                (Some(ms), Some(hm)) => c.starts_with(ms) || c.starts_with(hm),
                (Some(ms), None) => c.starts_with(ms),
                (None, Some(hm)) => c.starts_with(hm),
                (None, None) => false,
            };
            if !allowed_base {
                tracing::warn!(
                    "extraAttachmentRoots: rejected (must be under ~/.mac-stats or $HOME): {}",
                    c.display()
                );
                continue;
            }
            roots.push(c);
        } else {
            tracing::debug!(
                "extraAttachmentRoots: skip missing or unreadable path {}",
                p.display()
            );
        }
    }

    roots.sort();
    roots.dedup();
    roots
}

/// True if `path` resolves to a file under one of the allowed attachment roots.
pub fn is_allowed_outbound_attachment_path(path: &Path) -> bool {
    let Ok(canon_path) = path.canonicalize() else {
        return false;
    };

    let roots = canonical_outbound_attachment_root_dirs();
    if roots.is_empty() {
        tracing::debug!(
            "outbound attachment: no allowed roots (path={})",
            path.display()
        );
        return false;
    }

    for r in &roots {
        if canon_path.starts_with(r) {
            return true;
        }
    }

    tracing::debug!(
        "outbound attachment rejected: {} not under allowed roots",
        canon_path.display()
    );
    false
}
