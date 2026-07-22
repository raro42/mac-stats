//! Browser / CDP-related `Config` getters (split from `config/mod.rs` for maintainability).

use super::Config;
use std::path::PathBuf;

impl Config {
    /// Read-only file roots for **BROWSER_UPLOAD** (model-supplied paths must resolve under here or [`Self::screenshots_dir`]): `$HOME/.mac-stats/uploads/`
    pub fn browser_uploads_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("uploads")
        } else {
            std::env::temp_dir().join("mac-stats-uploads")
        }
    }

    /// Persisted browser cookie jar (CDP `Network.getAllCookies` snapshot): `$HOME/.mac-stats/browser_storage_state.json`
    pub fn browser_storage_state_json_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("browser_storage_state.json")
        } else {
            std::env::temp_dir().join("mac-stats-browser_storage_state.json")
        }
    }

    /// Domain-scoped secrets for `BROWSER_INPUT` `<secret>name</secret>` substitution: `$HOME/.mac-stats/browser-credentials.toml`
    pub fn browser_credentials_toml_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("browser-credentials.toml")
        } else {
            std::env::temp_dir().join("mac-stats-browser-credentials.toml")
        }
    }

    /// PDF exports directory (outbound Discord attachments when PDF tools write here): `$HOME/.mac-stats/pdfs/`
    pub fn pdfs_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("pdfs")
        } else {
            std::env::temp_dir().join("mac-stats-pdfs")
        }
    }

    /// CDP trace archives (`Tracing.start` / `ReturnAsStream` JSON): `$HOME/.mac-stats/traces/`
    pub fn browser_cdp_traces_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("traces")
        } else {
            std::env::temp_dir().join("mac-stats-traces")
        }
    }

    /// When **true**, mac-stats records a Chrome DevTools trace over the CDP WebSocket for each cached browser session
    /// (from first successful `BROWSER_*` CDP use until idle timeout, explicit session clear, or app shutdown).
    /// Default **false** (no extra WebSocket, no `Tracing` overhead). Config: **`browserCdpTraceEnabled`**.
    /// Env: **`MAC_STATS_BROWSER_CDP_TRACE_ENABLED`** (`true`/`1`/`yes` / `false`/`0`/`no`).
    pub fn browser_cdp_trace_enabled() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_ENABLED") {
            let lower = s.to_lowercase();
            if lower == "true" || lower == "1" || lower == "yes" {
                return true;
            }
            if lower == "false" || lower == "0" || lower == "no" {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json.get("browserCdpTraceEnabled").and_then(|v| v.as_bool()) {
                    return b;
                }
            }
        }
        false
    }

    /// Wall-clock cap from the **first** `Tracing.start` in this process (minutes). **`0`** = no time limit while enabled.
    /// Config: **`browserCdpTraceWallClockMinutes`**. Env: **`MAC_STATS_BROWSER_CDP_TRACE_MINUTES`** (clamped **0..=10080**).
    pub fn browser_cdp_trace_wall_clock_minutes() -> u64 {
        const MAX_MIN: u64 = 10080;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_MINUTES") {
            if let Ok(n) = s.parse::<u64>() {
                return n.min(MAX_MIN);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpTraceWallClockMinutes")
                    .and_then(|v| v.as_u64())
                {
                    return n.min(MAX_MIN);
                }
            }
        }
        0
    }

    /// Max bytes read from the CDP trace stream for one session file. Config: **`browserCdpTraceMaxFileBytes`**.
    /// Default **52428800** (50 MiB). Clamped **1048576..=524288000** (1 MiB–500 MiB).
    pub fn browser_cdp_trace_max_file_bytes() -> u64 {
        const DEFAULT: u64 = 50 * 1024 * 1024;
        const MIN: u64 = 1024 * 1024;
        const MAX: u64 = 500 * 1024 * 1024;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_MAX_FILE_BYTES") {
            if let Ok(n) = s.parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpTraceMaxFileBytes")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Max `*_cdp_trace.json` files to keep under `browser_cdp_traces_dir`; oldest removed after each successful write.
    /// **`0`** disables pruning. Default **10**. Config: **`browserCdpTraceMaxRetainedFiles`**. Clamped **0..=500**.
    pub fn browser_cdp_trace_max_retained_files() -> usize {
        const DEFAULT: usize = 10;
        const MAX: usize = 500;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_MAX_RETAINED_FILES") {
            if let Ok(n) = s.parse::<usize>() {
                return n.min(MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpTraceMaxRetainedFiles")
                    .and_then(|v| v.as_u64())
                {
                    return (n as usize).min(MAX);
                }
            }
        }
        DEFAULT
    }

    /// Browser download artifacts directory (reserved for outbound attachments): `$HOME/.mac-stats/browser-downloads/`
    pub fn browser_downloads_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("browser-downloads")
        } else {
            std::env::temp_dir().join("mac-stats-browser-downloads")
        }
    }

    /// Extra directory roots allowed for outbound attachments (Discord, etc.), from `config.json` **`extraAttachmentRoots`**.
    ///
    /// Each entry is a path string: absolute, or `~/…`, or relative to `$HOME`. After canonicalization, the directory must lie under canonical `$HOME/.mac-stats` or under canonical `$HOME`; otherwise it is skipped with a log line.
    pub fn extra_attachment_roots() -> Vec<PathBuf> {
        let config_path = Self::config_file_path();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return Vec::new();
        };
        let Some(arr) = json.get("extraAttachmentRoots").and_then(|v| v.as_array()) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for v in arr {
            let Some(s) = v.as_str() else {
                continue;
            };
            let p = Self::resolve_user_config_path(s);
            out.push(p);
        }
        out
    }

    /// Resolve a path from config: `~/` → `$HOME/…`; relative → `$HOME/<path>`; absolute unchanged.
    fn resolve_user_config_path(raw: &str) -> PathBuf {
        let t = raw.trim();
        if let Some(rest) = t.strip_prefix("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(rest);
            }
        }
        let p = PathBuf::from(t);
        if p.is_absolute() {
            p
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(t)
        } else {
            p
        }
    }

    /// Idle timeout in seconds for the CDP browser session. If the browser is not used for this long, it is closed.
    /// Default: 300 (5 minutes). Config: config.json `browserIdleTimeoutSecs`.
    /// Env override: `MAC_STATS_BROWSER_IDLE_TIMEOUT_SECS`. Clamped to 30..=3600.
    pub fn browser_idle_timeout_secs() -> u64 {
        const DEFAULT: u64 = 300;
        const MIN: u64 = 30;
        const MAX: u64 = 3600;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_IDLE_TIMEOUT_SECS") {
            if let Ok(n) = s.parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserIdleTimeoutSecs").and_then(|v| v.as_u64()) {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// TCP port on loopback for Chromium remote debugging (CDP HTTP `/json/version` and visible auto-launch).
    ///
    /// Default **9222**. Config: `browserCdpPort`; env: `MAC_STATS_BROWSER_CDP_PORT`. Clamped to **1024–65535**.
    /// When attaching to a browser you started yourself, use the same port here and start the browser with
    /// `--remote-debugging-port=<port>`.
    pub fn browser_cdp_port() -> u16 {
        const DEFAULT: u16 = 9222;
        const MIN: u16 = 1024;
        const MAX: u16 = 65535;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_PORT") {
            if let Ok(n) = s.trim().parse::<u16>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserCdpPort").and_then(|v| v.as_u64()) {
                    if n <= u64::from(u16::MAX) {
                        return (n as u16).clamp(MIN, MAX);
                    }
                }
            }
        }
        DEFAULT
    }

    /// Max wall-clock time to poll loopback **`/json/version`** after mac-stats spawns visible Chromium (default **15** seconds).
    ///
    /// Each probe uses a short HTTP timeout in the browser agent; this value bounds total wait when startup is slow.
    /// Config: `browserCdpPostLaunchMaxWaitSecs`; env: `MAC_STATS_BROWSER_CDP_POST_LAUNCH_MAX_WAIT_SECS`. Clamped to **3–120**.
    pub fn browser_cdp_post_launch_max_wait_secs() -> u64 {
        const DEFAULT: u64 = 15;
        const MIN: u64 = 3;
        const MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_POST_LAUNCH_MAX_WAIT_SECS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpPostLaunchMaxWaitSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Pause between failed post-launch CDP HTTP probes (default **250** ms).
    ///
    /// Config: `browserCdpPostLaunchPollIntervalMs`; env: `MAC_STATS_BROWSER_CDP_POST_LAUNCH_POLL_MS`. Clamped to **50–2000**.
    pub fn browser_cdp_post_launch_poll_interval_ms() -> u64 {
        const DEFAULT: u64 = 250;
        const MIN: u64 = 50;
        const MAX: u64 = 2000;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_POST_LAUNCH_POLL_MS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpPostLaunchPollIntervalMs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Per-request HTTP timeout in seconds for CDP discovery (`GET /json/version` and equivalent probes).
    ///
    /// Default **5** (matches the historical `get_ws_url` single-shot timeout). Config: **`browserCdpHttpTimeoutSecs`**;
    /// env: **`MAC_STATS_BROWSER_CDP_HTTP_TIMEOUT_SECS`**. Clamped to **1–60** (minimum avoids unusable sub-second probes; maximum avoids accidental long hangs).
    pub fn browser_cdp_http_timeout_secs() -> u64 {
        const DEFAULT: u64 = 5;
        const MIN: u64 = 1;
        const MAX: u64 = 60;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_HTTP_TIMEOUT_SECS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpHttpTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Max seconds without CDP traffic before headless_chrome closes the WebSocket (attach + headless launch).
    ///
    /// Must exceed long LLM turns between **BROWSER_*** tools or Chrome will time out (default was 30s in the
    /// launcher; attach previously used a shorter idle). Default **600** (10 minutes). Config:
    /// **`browserCdpIdleTimeoutSecs`**; env: **`MAC_STATS_BROWSER_CDP_IDLE_TIMEOUT_SECS`**. Clamped **30–3600**.
    ///
    /// If **`browserCdpIdleTimeoutSecs`** is unset, legacy **`browserCdpWsConnectTimeoutSecs`** / env
    /// **`MAC_STATS_BROWSER_CDP_WS_CONNECT_TIMEOUT_SECS`** (clamped **5–120**) is used so existing configs keep
    /// working; otherwise the default above applies.
    pub fn browser_cdp_idle_timeout_secs() -> u64 {
        const DEFAULT: u64 = 600;
        const MIN: u64 = 30;
        const MAX: u64 = 3600;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_IDLE_TIMEOUT_SECS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpIdleTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        const LEGACY_MIN: u64 = 5;
        const LEGACY_MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_WS_CONNECT_TIMEOUT_SECS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(LEGACY_MIN, LEGACY_MAX).clamp(MIN, MAX);
            }
        }
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpWsConnectTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(LEGACY_MIN, LEGACY_MAX).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Alias for [`Self::browser_cdp_idle_timeout_secs`] (historical name: `Browser::connect_with_timeout` uses this
    /// duration as **idle**, not a separate TCP handshake timeout).
    pub fn browser_cdp_ws_connect_timeout_secs() -> u64 {
        Self::browser_cdp_idle_timeout_secs()
    }

    /// CDP [`Browser.grantPermissions`](https://chromedevtools.github.io/devtools-protocol/tot/Browser/#method-grantPermissions)
    /// permission names to apply **once** when mac-stats opens a **new** CDP browser session (not on tab switch or session reuse).
    ///
    /// Default **empty** (no behaviour change). Each entry must be a Chromium **`Browser.PermissionType`** string, for example:
    /// `geolocation`, `notifications`, `clipboardReadWrite`, `clipboardSanitizedWrite`, `durableStorage`, `idleDetection`, …
    /// (see the protocol / DevTools schema; typos are skipped with a warning). **Microphone/camera-class** permissions such as
    /// `audioCapture` and `videoCapture` are **not** implied—list them explicitly only if you intend to grant them. Failures are
    /// logged at **warn** and do not abort the session. Config: **`browserCdpGrantPermissions`** (JSON array of strings).
    /// Env: **`MAC_STATS_BROWSER_CDP_GRANT_PERMISSIONS`** — comma-separated list (optional; replaces config array when set).
    pub fn browser_cdp_grant_permissions() -> Vec<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_GRANT_PERMISSIONS") {
            let v: Vec<String> = s
                .split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect();
            if !v.is_empty() {
                return v;
            }
        }
        let config_path = Self::config_file_path();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return Vec::new();
        };
        let Some(arr) = json
            .get("browserCdpGrantPermissions")
            .and_then(|v| v.as_array())
        else {
            return Vec::new();
        };
        arr.iter()
            .filter_map(|v| {
                v.as_str()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
            })
            .collect()
    }

    /// Default visible Chromium path when `browserChromiumExecutable` is unset (Google Chrome on macOS, `google-chrome` on Linux).
    #[cfg(target_os = "macos")]
    pub fn default_browser_chromium_executable() -> PathBuf {
        PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
    }

    /// Default visible Chromium path when `browserChromiumExecutable` is unset (Google Chrome on macOS, `google-chrome` on Linux).
    #[cfg(not(target_os = "macos"))]
    pub fn default_browser_chromium_executable() -> PathBuf {
        PathBuf::from("google-chrome")
    }

    fn browser_chromium_executable_raw_opt() -> Option<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CHROMIUM_EXECUTABLE") {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json
                    .get("browserChromiumExecutable")
                    .and_then(|v| v.as_str())
                {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
        None
    }

    /// True when **`browserChromiumExecutable`** or **`MAC_STATS_BROWSER_CHROMIUM_EXECUTABLE`** is set (non-empty).
    /// Used to avoid silently falling back to a different Chromium binary after a failed launch.
    pub fn browser_chromium_executable_configured() -> bool {
        Self::browser_chromium_executable_raw_opt().is_some()
    }

    /// Resolved Chromium executable for visible CDP launch: configured path (with `~/` expansion) or platform default.
    pub fn browser_chromium_executable_path() -> PathBuf {
        match Self::browser_chromium_executable_raw_opt() {
            Some(s) => Self::resolve_user_config_path(&s),
            None => Self::default_browser_chromium_executable(),
        }
    }

    /// Optional Chromium user-data directory for visible launches (and headless when set). `~/` is expanded like other config paths.
    ///
    /// Config: `browserChromiumUserDataDir`; env: `MAC_STATS_BROWSER_CHROMIUM_USER_DATA_DIR`.
    pub fn browser_chromium_user_data_dir() -> Option<PathBuf> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CHROMIUM_USER_DATA_DIR") {
            let t = s.trim();
            if !t.is_empty() {
                return Some(Self::resolve_user_config_path(t));
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json
                    .get("browserChromiumUserDataDir")
                    .and_then(|v| v.as_str())
                {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(Self::resolve_user_config_path(t));
                    }
                }
            }
        }
        None
    }

    /// Optional proxy username for **CDP Chrome only** (`Fetch.authRequired` when the challenge source is **Proxy**).
    ///
    /// No effect unless [`Self::browser_cdp_proxy_password`] is also a non-empty string. Not applied to mac-stats’
    /// own `reqwest` / **FETCH_URL** client. Config: `browserCdpProxyUsername`. Env: `MAC_STATS_BROWSER_CDP_PROXY_USERNAME`.
    pub fn browser_cdp_proxy_username() -> Option<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_PROXY_USERNAME") {
            let t = s.trim();
            if t.is_empty() {
                return None;
            }
            return Some(t.to_string());
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json.get("browserCdpProxyUsername").and_then(|v| v.as_str()) {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
        None
    }

    /// Optional proxy password for **CDP Chrome only** (pairs with [`Self::browser_cdp_proxy_username`]).
    ///
    /// Config: `browserCdpProxyPassword`. Env: `MAC_STATS_BROWSER_CDP_PROXY_PASSWORD`. Values are never logged.
    pub fn browser_cdp_proxy_password() -> Option<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_PROXY_PASSWORD") {
            let t = s.trim();
            if t.is_empty() {
                return None;
            }
            return Some(t.to_string());
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json.get("browserCdpProxyPassword").and_then(|v| v.as_str()) {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
        None
    }

    /// True when both CDP proxy username and password are configured with non-empty values.
    pub fn browser_cdp_proxy_credentials_active() -> bool {
        match (
            Self::browser_cdp_proxy_username(),
            Self::browser_cdp_proxy_password(),
        ) {
            (Some(u), Some(p)) => !u.trim().is_empty() && !p.trim().is_empty(),
            _ => false,
        }
    }

    /// Master enable/disable switch for all browser automation tools (BROWSER_*).
    ///
    /// When set to `false`, the app refuses BROWSER_* tool calls (no Chrome/CDP launch and no HTTP fallback).
    /// Default: `true`.
    /// Config: config.json `browserToolsEnabled` (boolean).
    pub fn browser_tools_enabled() -> bool {
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json.get("browserToolsEnabled").and_then(|v| v.as_bool()) {
                    return b;
                }
            }
        }
        true
    }

    /// Enable/disable **RUN_JS** (model-triggered JavaScript executed on the host via Node.js).
    ///
    /// When `false`, the tool loop returns a stable refusal without spawning Node. Orthogonal to
    /// **BROWSER_*** tools (`browserToolsEnabled`). Default: `true`.
    /// Config: `config.json` **`runJsEnabled`** (boolean).
    pub fn run_js_enabled() -> bool {
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json.get("runJsEnabled").and_then(|v| v.as_bool()) {
                    return b;
                }
            }
        }
        true
    }

    /// Pause (seconds) after each browser tool completes and before the next `BROWSER_*` tool runs
    /// in the same agent request, so the page can settle (DOM, network, animations). The first
    /// browser tool in a request has no leading wait. Non-browser tools (e.g. FETCH_URL) are unaffected.
    ///
    /// Default: `0` (no extra delay). Config: `browserWaitBetweenActionsSecs` (number, fractional OK).
    /// Env: `MAC_STATS_BROWSER_WAIT_BETWEEN_ACTIONS_SECS`. Clamped to `0.0..=60.0`.
    pub fn browser_wait_between_actions_secs() -> f64 {
        const DEFAULT: f64 = 0.0;
        const MAX: f64 = 60.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_WAIT_BETWEEN_ACTIONS_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return n.clamp(0.0, MAX);
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserWaitBetweenActionsSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return n.clamp(0.0, MAX);
                        }
                    }
                }
            }
        }
        DEFAULT
    }

    /// Minimum time (seconds) to wait after `wait_until_navigated` completes on the CDP path,
    /// before returning page state (in addition to optional network-idle wait). Mirrors browser-use
    /// `minimum_wait_page_load_time`; default **1.5s** gives SPAs time to hydrate before SPA
    /// readiness and state capture (lower in config for fast static pages; SPA retry still applies
    /// separately — see `browserSpaRetryEnabled`).
    ///
    /// Default **1.5**. Config: `browserPostNavigateMinDwellSecs`.
    /// Env: `MAC_STATS_BROWSER_POST_NAV_MIN_DWELL_SECS`. Clamped to `0.0..=10.0`.
    ///
    /// Applies uniformly (same-domain shorter **navigation timeout** does not skip this dwell).
    ///
    /// Second return value is a short label for logs (`env …`, `config.json …`, or `default`) so
    /// operators can tell why a non-default dwell (e.g. **0.25**) appears when they expected **1.5**.
    pub fn browser_post_navigate_min_dwell_secs_resolved() -> (f64, &'static str) {
        const DEFAULT: f64 = 1.5;
        const MAX: f64 = 10.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_MIN_DWELL_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return (
                        n.clamp(0.0, MAX),
                        "env MAC_STATS_BROWSER_POST_NAV_MIN_DWELL_SECS",
                    );
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserPostNavigateMinDwellSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return (
                                n.clamp(0.0, MAX),
                                "config.json browserPostNavigateMinDwellSecs",
                            );
                        }
                    }
                }
            }
        }
        (DEFAULT, "default")
    }

    pub fn browser_post_navigate_min_dwell_secs() -> f64 {
        Self::browser_post_navigate_min_dwell_secs_resolved().0
    }

    /// When true, after the minimum dwell mac-stats waits until **Network** CDP shows no in-flight
    /// requests for `browser_post_navigate_network_idle_quiet_secs`, capped by
    /// `browser_post_navigate_network_idle_max_extra_secs`. Default **false** (latency-preserving).
    ///
    /// Config: `browserPostNavigateNetworkIdleEnabled`. Env:
    /// `MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_ENABLED` (`1`/`true`/`yes` / `0`/`false`/`no`).
    pub fn browser_post_navigate_network_idle_enabled() -> bool {
        const DEFAULT: bool = false;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_ENABLED") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserPostNavigateNetworkIdleEnabled")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        DEFAULT
    }

    /// Required quiet period (seconds) with **zero** in-flight Network requests before navigation
    /// stabilization completes. Config: `browserPostNavigateNetworkIdleQuietSecs`. Env:
    /// `MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_QUIET_SECS`. Default **0.5**, clamped to `0.0..=10.0`.
    pub fn browser_post_navigate_network_idle_quiet_secs() -> f64 {
        const DEFAULT: f64 = 0.5;
        const MAX: f64 = 10.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_QUIET_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return n.clamp(0.0, MAX);
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserPostNavigateNetworkIdleQuietSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return n.clamp(0.0, MAX);
                        }
                    }
                }
            }
        }
        DEFAULT
    }

    /// Hard cap (seconds) for the optional post-navigate **network in-flight** wait. When elapsed,
    /// stabilization proceeds even if requests remain (logged at debug). Config:
    /// `browserPostNavigateNetworkIdleMaxExtraSecs`. Env:
    /// `MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_MAX_EXTRA_SECS`. Default **5.0**, clamped to `0.0..=120.0`.
    /// **0** disables the network-idle phase entirely (min dwell still applies).
    pub fn browser_post_navigate_network_idle_max_extra_secs() -> f64 {
        const DEFAULT: f64 = 5.0;
        const MAX: f64 = 120.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_MAX_EXTRA_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return n.clamp(0.0, MAX);
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserPostNavigateNetworkIdleMaxExtraSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return n.clamp(0.0, MAX);
                        }
                    }
                }
            }
        }
        DEFAULT
    }

    /// Maximum navigation wait timeout in seconds for BROWSER_NAVIGATE, BROWSER_GO_BACK, BROWSER_GO_FORWARD, and BROWSER_RELOAD. Slow or stuck navigations fail with a clear message instead of hanging. Config: config.json `browserNavigationTimeoutSecs`. Env: `MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS`. Default 30, clamped to 5..=120.
    pub fn browser_navigation_timeout_secs() -> u64 {
        const DEFAULT: u64 = 30;
        const MIN: u64 = 5;
        const MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS") {
            if let Ok(n) = s.parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserNavigationTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Optional shorter navigation wait timeout in seconds for same-domain navigations (BROWSER_NAVIGATE when target host equals current page host). When `None`, same-domain uses the same timeout as cross-domain. Config: config.json `browserSameDomainNavigationTimeoutSecs`. Env: `MAC_STATS_BROWSER_SAME_DOMAIN_NAVIGATION_TIMEOUT_SECS`. When set, clamped to 1..=120. Typical value 5 (or 3 to mirror browser-use).
    pub fn browser_same_domain_navigation_timeout_secs() -> Option<u64> {
        const MIN: u64 = 1;
        const MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_SAME_DOMAIN_NAVIGATION_TIMEOUT_SECS") {
            if let Ok(n) = s.parse::<u64>() {
                if n > 0 {
                    return Some(n.clamp(MIN, MAX));
                }
                return None;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserSameDomainNavigationTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    if n > 0 {
                        return Some(n.clamp(MIN, MAX));
                    }
                }
            }
        }
        None
    }

    /// Maximum open CDP **page** tabs before best-effort pruning (OpenClaw-style tab cap). **0** disables (default).
    /// When the count exceeds this value, the automation session closes older non-focused tabs until within the cap.
    /// Config: `config.json` key `browserMaxPageTabs`. Env: `MAC_STATS_BROWSER_MAX_PAGE_TABS`. Clamped to 0..=64.
    pub fn browser_max_page_tabs() -> usize {
        const DEFAULT: usize = 0;
        const MAX: usize = 64;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_MAX_PAGE_TABS") {
            if let Ok(n) = s.parse::<usize>() {
                return n.min(MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserMaxPageTabs").and_then(|v| v.as_u64()) {
                    return (n as usize).min(MAX);
                }
            }
        }
        DEFAULT
    }

    /// Whether to include bounded CDP console and page-level JavaScript error diagnostics
    /// in `BROWSER_NAVIGATE` and `BROWSER_EXTRACT` tool results.
    ///
    /// Off by default to preserve existing context size and tool output stability.
    /// Config: config.json `browserIncludeDiagnosticsInState` (boolean);
    /// Env: `MAC_STATS_BROWSER_INCLUDE_DIAGNOSTICS_IN_STATE` (true/1/yes or false/0/no).
    pub fn browser_include_diagnostics_in_state() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_INCLUDE_DIAGNOSTICS_IN_STATE") {
            let lower = s.to_lowercase();
            return matches!(lower.as_str(), "1" | "true" | "yes" | "on");
        }

        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserIncludeDiagnosticsInState")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }

        false
    }

    /// After `BROWSER_NAVIGATE`, detect skeleton/blank pages (sparse text vs. many DOM nodes) and retry
    /// with extra waits plus a full reload (browser-use style). Baseline post-navigate sleep is unchanged.
    /// Disable for fast static pages or tests. Config: `browserSpaRetryEnabled` (boolean).
    /// Env: `MAC_STATS_BROWSER_SPA_RETRY_ENABLED` (true/1/yes or false/0/no). Default **true**.
    pub fn browser_spa_retry_enabled() -> bool {
        const DEFAULT: bool = true;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_SPA_RETRY_ENABLED") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json.get("browserSpaRetryEnabled").and_then(|v| v.as_bool()) {
                    return b;
                }
            }
        }
        DEFAULT
    }

    /// Browser viewport width in pixels (CDP/headless window size). Config: config.json `browserViewportWidth`.
    /// Default 1800. Clamped to 800..=3840.
    pub fn browser_viewport_width() -> u32 {
        const DEFAULT: u32 = 1800;
        const MIN: u32 = 800;
        const MAX: u32 = 3840;
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserViewportWidth").and_then(|v| v.as_u64()) {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Perplexity search: max number of results to request from the API. Config: config.json `perplexityMaxResults`.
    /// Default 8. Clamped to 1..=20. Env override: `MAC_STATS_PERPLEXITY_MAX_RESULTS`.
    pub fn perplexity_max_results() -> u32 {
        const DEFAULT: u32 = 8;
        const MIN: u32 = 1;
        const MAX: u32 = 20;
        if let Ok(s) = std::env::var("MAC_STATS_PERPLEXITY_MAX_RESULTS") {
            if let Ok(n) = s.parse::<u32>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("perplexityMaxResults").and_then(|v| v.as_u64()) {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Perplexity search: max characters per snippet when formatting results for the model. Config: config.json `perplexitySnippetMaxChars`.
    /// Default 280. Clamped to 80..=2000. Env override: `MAC_STATS_PERPLEXITY_SNIPPET_MAX_CHARS`.
    pub fn perplexity_snippet_max_chars() -> usize {
        const DEFAULT: usize = 280;
        const MIN: usize = 80;
        const MAX: usize = 2000;
        if let Ok(s) = std::env::var("MAC_STATS_PERPLEXITY_SNIPPET_MAX_CHARS") {
            if let Ok(n) = s.parse::<usize>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("perplexitySnippetMaxChars")
                    .and_then(|v| v.as_u64())
                {
                    return (n as usize).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Browser viewport height in pixels (CDP/headless window size). Config: config.json `browserViewportHeight`.
    /// Default 2400. Clamped to 600..=2160.
    pub fn browser_viewport_height() -> u32 {
        const DEFAULT: u32 = 2400;
        const MIN: u32 = 600;
        const MAX: u32 = 2160;
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserViewportHeight").and_then(|v| v.as_u64()) {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Optional CDP `Emulation.setDeviceMetricsOverride` width/height (CSS pixels). **Both** `browserCdpEmulateViewportWidth`
    /// and **`browserCdpEmulateViewportHeight`** must be set in `config.json` (or both env vars) to enable; if only one is set,
    /// a warning is logged and emulation is disabled. Clamped to 200..=3840 × 200..=2160.
    /// Env: **`MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_WIDTH`**, **`MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_HEIGHT`** (non-empty parses as u32).
    pub fn browser_cdp_emulate_viewport_dimensions() -> Option<(u32, u32)> {
        const MIN_W: u32 = 200;
        const MAX_W: u32 = 3840;
        const MIN_H: u32 = 200;
        const MAX_H: u32 = 2160;
        let w_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_WIDTH")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<u32>().ok()
                }
            });
        let h_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_HEIGHT")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<u32>().ok()
                }
            });
        let config_path = Self::config_file_path();
        let (w_cfg, h_cfg) = if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let w = json.get("browserCdpEmulateViewportWidth").and_then(|v| {
                    if v.is_null() {
                        None
                    } else {
                        v.as_u64().map(|n| n as u32)
                    }
                });
                let h = json.get("browserCdpEmulateViewportHeight").and_then(|v| {
                    if v.is_null() {
                        None
                    } else {
                        v.as_u64().map(|n| n as u32)
                    }
                });
                (w, h)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        let w = w_env.or(w_cfg);
        let h = h_env.or(h_cfg);
        match (w, h) {
            (Some(w), Some(h)) => Some((w.clamp(MIN_W, MAX_W), h.clamp(MIN_H, MAX_H))),
            (None, None) => None,
            _ => {
                tracing::warn!(
                    "config: browserCdpEmulateViewportWidth and browserCdpEmulateViewportHeight must both be set to enable CDP device-metrics emulation; ignoring partial config"
                );
                None
            }
        }
    }

    /// Device scale factor for [`Self::browser_cdp_emulate_viewport_dimensions`] when enabled. Default **1.0**. Clamped 0.1..=10.0.
    /// Config: **`browserCdpEmulateDeviceScaleFactor`**. Env: **`MAC_STATS_BROWSER_CDP_EMULATE_DEVICE_SCALE_FACTOR`**.
    pub fn browser_cdp_emulate_device_scale_factor() -> f64 {
        const DEFAULT: f64 = 1.0;
        const MIN: f64 = 0.1;
        const MAX: f64 = 10.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_DEVICE_SCALE_FACTOR") {
            let t = s.trim();
            if !t.is_empty() {
                if let Ok(f) = t.parse::<f64>() {
                    if f.is_finite() {
                        return f.clamp(MIN, MAX);
                    }
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(f) = json
                    .get("browserCdpEmulateDeviceScaleFactor")
                    .and_then(|v| v.as_f64())
                {
                    if f.is_finite() {
                        return f.clamp(MIN, MAX);
                    }
                }
            }
        }
        DEFAULT
    }

    /// Mobile flag for CDP device-metrics emulation when [`Self::browser_cdp_emulate_viewport_dimensions`] is enabled. Default **false**.
    /// Config: **`browserCdpEmulateMobile`**. Env: **`MAC_STATS_BROWSER_CDP_EMULATE_MOBILE`** (`true`/`1`/`yes` / `false`/`0`/`no`).
    pub fn browser_cdp_emulate_mobile() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_MOBILE") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserCdpEmulateMobile")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
    }

    /// Optional CDP `Emulation.setGeolocationOverride`: **both** latitude and longitude must be set (config or env), or geolocation emulation is off
    /// (`Emulation.clearGeolocationOverride`). Optional accuracy (meters). Lat clamped −90..=90, lon −180..=180, accuracy 0.1..=100_000 when set.
    /// Config: **`browserCdpEmulateGeolocationLatitude`**, **`browserCdpEmulateGeolocationLongitude`**, optional **`browserCdpEmulateGeolocationAccuracy`**.
    /// Env: **`MAC_STATS_BROWSER_CDP_EMULATE_GEO_LATITUDE`**, **`MAC_STATS_BROWSER_CDP_EMULATE_GEO_LONGITUDE`**, optional **`MAC_STATS_BROWSER_CDP_EMULATE_GEO_ACCURACY`**.
    pub fn browser_cdp_emulate_geolocation() -> Option<(f64, f64, Option<f64>)> {
        const MIN_LAT: f64 = -90.0;
        const MAX_LAT: f64 = 90.0;
        const MIN_LON: f64 = -180.0;
        const MAX_LON: f64 = 180.0;
        const MIN_ACC: f64 = 0.1;
        const MAX_ACC: f64 = 100_000.0;

        let lat_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_GEO_LATITUDE")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<f64>().ok()
                }
            });
        let lon_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_GEO_LONGITUDE")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<f64>().ok()
                }
            });
        let acc_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_GEO_ACCURACY")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<f64>().ok()
                }
            });

        let config_path = Self::config_file_path();
        let (lat_cfg, lon_cfg, acc_cfg) = if let Ok(content) = std::fs::read_to_string(&config_path)
        {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let lat = json
                    .get("browserCdpEmulateGeolocationLatitude")
                    .and_then(|v| if v.is_null() { None } else { v.as_f64() });
                let lon = json
                    .get("browserCdpEmulateGeolocationLongitude")
                    .and_then(|v| if v.is_null() { None } else { v.as_f64() });
                let acc = json
                    .get("browserCdpEmulateGeolocationAccuracy")
                    .and_then(|v| if v.is_null() { None } else { v.as_f64() });
                (lat, lon, acc)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let lat = lat_env.or(lat_cfg);
        let lon = lon_env.or(lon_cfg);
        let acc = acc_env.or(acc_cfg);

        match (lat, lon) {
            (Some(la), Some(lo)) => {
                if !la.is_finite() || !lo.is_finite() {
                    tracing::warn!(
                        "config: browserCdpEmulateGeolocationLatitude/Longitude must be finite; clearing geolocation emulation"
                    );
                    return None;
                }
                let la = la.clamp(MIN_LAT, MAX_LAT);
                let lo = lo.clamp(MIN_LON, MAX_LON);
                let acc_opt = acc
                    .filter(|a| a.is_finite())
                    .map(|a| a.clamp(MIN_ACC, MAX_ACC));
                Some((la, lo, acc_opt))
            }
            (None, None) => None,
            _ => {
                tracing::warn!(
                    "config: browserCdpEmulateGeolocationLatitude and browserCdpEmulateGeolocationLongitude must both be set to enable CDP geolocation emulation; ignoring partial config"
                );
                None
            }
        }
    }

    /// Maximum size in bytes for one browser-produced artifact: raw/annotated PNG from **BROWSER_SCREENSHOT**,
    /// PDF bytes from **BROWSER_SAVE_PDF** / **Page.printToPDF**, and the same limit when loading those files for Discord or vision.
    /// Config: `config.json` **`browserArtifactMaxBytes`**. Default **10485760** (10 MiB). Clamped to **65536**–**104857600** (64 KiB–100 MiB).
    pub fn browser_artifact_max_bytes() -> u64 {
        const DEFAULT: u64 = 10 * 1024 * 1024;
        const MIN: u64 = 64 * 1024;
        const MAX: u64 = 100 * 1024 * 1024;
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserArtifactMaxBytes").and_then(|v| v.as_u64()) {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// When **true**, **BROWSER_SAVE_PDF** passes **`printBackground: true`** to CDP `Page.printToPDF`.
    /// Default **false** (smaller files, text-forward). Config: **`browserPrintPdfBackground`** (boolean).
    /// Env override: **`MAC_STATS_BROWSER_PRINT_PDF_BACKGROUND`** (`true`/`1`/`yes` / `false`/`0`/`no`).
    pub fn browser_print_pdf_background() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_PRINT_PDF_BACKGROUND") {
            let lower = s.to_lowercase();
            if lower == "true" || lower == "1" || lower == "yes" {
                return true;
            }
            if lower == "false" || lower == "0" || lower == "no" {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserPrintPdfBackground")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
    }

    /// Optional width/height for resizing images before they are sent to a vision LLM (e.g. completion verification).
    /// Both `browserLlmScreenshotWidth` and `browserLlmScreenshotHeight` must be set in `config.json`; each is
    /// clamped to 100..=8192. If either is missing, returns `None` (full-size image bytes are sent, base64-only).
    /// Does not change files on disk or Discord attachments.
    pub fn browser_llm_screenshot_size() -> Option<(u32, u32)> {
        const MIN: u32 = 100;
        const MAX: u32 = 8192;
        let config_path = Self::config_file_path();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return None;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return None;
        };
        let w = json
            .get("browserLlmScreenshotWidth")
            .and_then(|v| v.as_u64())
            .map(|n| (n as u32).clamp(MIN, MAX));
        let h = json
            .get("browserLlmScreenshotHeight")
            .and_then(|v| v.as_u64())
            .map(|n| (n as u32).clamp(MIN, MAX));
        match (w, h) {
            (Some(w), Some(h)) => Some((w, h)),
            (None, None) => None,
            _ => {
                tracing::warn!(
                    "config: browserLlmScreenshotWidth/Height must both be set to enable LLM screenshot resize; ignoring partial config"
                );
                None
            }
        }
    }
}
