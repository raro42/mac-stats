//! Hard caps for browser-produced binary artifacts (screenshots, print-to-PDF), aligned with
//! operator-visible limits before large buffers are written or read back for Discord / vision.

use std::path::Path;

use crate::config::Config;
use crate::mac_stats_warn;

/// Reject when an in-memory buffer is already over the cap (before writing the final file).
pub fn ensure_buffer_within_browser_artifact_cap(
    actual: usize,
    artifact_kind: &str,
) -> Result<(), String> {
    let cap_u = Config::browser_artifact_max_bytes();
    let cap = cap_u as usize;
    if actual <= cap {
        return Ok(());
    }
    let cap_mib = cap_u as f64 / (1024.0 * 1024.0);
    let actual_mib = actual as f64 / (1024.0 * 1024.0);
    mac_stats_warn!(
        "browser",
        "browser artifact rejected ({}): actual={} bytes ({:.2} MiB), limit={} bytes ({:.2} MiB); try smaller browserViewportWidth/Height, partial capture, or FETCH_URL for page text",
        artifact_kind,
        actual,
        actual_mib,
        cap,
        cap_mib
    );
    Err(format!(
        "Browser artifact exceeds browserArtifactMaxBytes ({} bytes, max {} bytes ≈ {:.1} MiB). Try smaller browserViewportWidth/Height in config, or use FETCH_URL for page text instead of a huge screenshot.",
        actual,
        cap,
        cap_mib
    ))
}

/// `stat` the path; returns file size if within cap. On oversize logs WARN and returns Err (skip attach / do not read).
pub fn stat_path_within_browser_artifact_cap(
    path: &Path,
    artifact_kind: &str,
) -> Result<u64, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let len = meta.len();
    let cap = Config::browser_artifact_max_bytes();
    if len > cap {
        let cap_mib = cap as f64 / (1024.0 * 1024.0);
        mac_stats_warn!(
            "browser",
            "browser artifact file over limit ({}): path={} actual={} bytes limit={} bytes ({:.2} MiB)",
            artifact_kind,
            path.display(),
            len,
            cap,
            cap_mib
        );
        Err(format!(
            "File exceeds browserArtifactMaxBytes ({} bytes, max {} bytes)",
            len, cap
        ))
    } else {
        Ok(len)
    }
}
