//! Atomic temp-then-rename writes for browser artifacts (screenshots, PDFs) under `~/.mac-stats`.
//! Avoids exposing truncated files if the process dies mid-write.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::mac_stats_debug;

static ATOMIC_WRITE_SEQ: AtomicU64 = AtomicU64::new(0);

const MAX_UNTRUSTED_BASENAME_LEN: usize = 200;

/// Sanitize a single path basename derived from URLs or other untrusted text.
/// Removes ASCII control characters, rejects `.` / `..`, caps length, and maps characters
/// that are unsafe or awkward in filenames to `_`.
pub fn sanitize_untrusted_basename(input: &str, empty_fallback: &str) -> String {
    let filtered: String = input
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| match c {
            '/' | '\\' | ':' | '?' | '*' | '"' | '<' | '>' | '|' | '[' | ']' | '%' | '@'
            | '\u{202e}' => '_',
            _ => c,
        })
        .collect();
    let trimmed = filtered.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return empty_fallback.to_string();
    }
    let mut out = trimmed.to_string();
    if out.len() > MAX_UNTRUSTED_BASENAME_LEN {
        let mut end = MAX_UNTRUSTED_BASENAME_LEN;
        while end > 0 && !out.is_char_boundary(end) {
            end -= 1;
        }
        out.truncate(end);
    }
    let out = out.trim();
    if out.is_empty() || out == "." || out == ".." {
        empty_fallback.to_string()
    } else {
        out.to_string()
    }
}

/// Write `bytes` to `dest_dir/final_filename` by writing a unique `*.part` file in the same
/// directory, syncing, then renaming into place. On error, the temp file is removed best-effort.
pub fn write_bytes_atomic_same_dir(
    dest_dir: &Path,
    final_filename: &str,
    bytes: &[u8],
) -> Result<PathBuf, String> {
    if final_filename.is_empty() || final_filename.contains('/') || final_filename.contains('\\') {
        return Err("atomic write: invalid final filename".to_string());
    }
    let final_path = dest_dir.join(final_filename);
    let seq = ATOMIC_WRITE_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp_name = format!(
        ".mac-stats-artifact.{}.{}.part",
        std::process::id(),
        seq
    );
    let tmp_path = dest_dir.join(tmp_name);

    let write_result = (|| -> Result<(), String> {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(|e| format!("open temp artifact: {}", e))?;
        f.write_all(bytes).map_err(|e| format!("write temp artifact: {}", e))?;
        f.sync_all().map_err(|e| format!("sync temp artifact: {}", e))?;
        drop(f);
        std::fs::rename(&tmp_path, &final_path)
            .map_err(|e| format!("finalize artifact rename: {}", e))?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    } else {
        mac_stats_debug!(
            "browser/cdp",
            "artifact_atomic: finalized {} ({} bytes)",
            final_path.display(),
            bytes.len()
        );
    }

    write_result.map(|_| final_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sanitize_basename_empty_fallback() {
        assert_eq!(sanitize_untrusted_basename("", "page"), "page");
        assert_eq!(sanitize_untrusted_basename("  ", "page"), "page");
        assert_eq!(sanitize_untrusted_basename(".", "x"), "x");
        assert_eq!(sanitize_untrusted_basename("..", "x"), "x");
    }

    #[test]
    fn sanitize_basename_keeps_hostlike() {
        assert_eq!(
            sanitize_untrusted_basename("www.example.com", "page"),
            "www.example.com"
        );
    }

    #[test]
    fn sanitize_basename_maps_separators() {
        assert_eq!(
            sanitize_untrusted_basename("a:b/c?x", "page"),
            "a_b_c_x"
        );
    }

    #[test]
    fn write_bytes_atomic_creates_file() {
        let dir = std::env::temp_dir().join(format!(
            "mac-stats-artifact-atomic-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let name = "test-out.bin";
        let p = write_bytes_atomic_same_dir(&dir, name, b"hello").unwrap();
        assert_eq!(p, dir.join(name));
        assert_eq!(fs::read(&p).unwrap(), b"hello");
        let _ = fs::remove_dir_all(&dir);
    }
}
