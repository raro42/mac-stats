//! Browser / URL fetch support for Ollama and AI tasks.
//!
//! Provides server-side page fetch (no CORS). Used by the Ollama tool protocol
//! (FETCH_URL) and can be invoked from the frontend.

use regex::Regex;
use std::net::{IpAddr, Ipv6Addr, ToSocketAddrs};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};
use url::Url;

/// Prefix for allowlist entries that match when the hostname **contains** the suffix (case-insensitive).
/// Plain entries without `*` remain **exact** hostname match for backward compatibility.
const SSRF_ALLOWLIST_CONTAINS_PREFIX: &str = "contains:";

/// Max response body size (chars) to avoid huge strings (e.g. 500 KB of text).
const MAX_BODY_CHARS: usize = 500_000;

/// Suffix appended when the body exceeds [`MAX_BODY_CHARS`] so the model sees explicit truncation (review D3).
const FETCH_BODY_TRUNC_SUFFIX: &str = " [content truncated]";

/// If `body` is longer than [`MAX_BODY_CHARS`] (Unicode scalar values), ellipse the middle and append
/// [`FETCH_BODY_TRUNC_SUFFIX`]. Total char count is at most [`MAX_BODY_CHARS`].
fn truncate_fetch_body_if_needed(body: String) -> String {
    if body.chars().count() <= MAX_BODY_CHARS {
        return body;
    }
    let suffix_len = FETCH_BODY_TRUNC_SUFFIX.chars().count();
    let budget = MAX_BODY_CHARS.saturating_sub(suffix_len);
    format!(
        "{}{}",
        crate::logging::ellipse(&body, budget),
        FETCH_BODY_TRUNC_SUFFIX
    )
}

/// Browser-like User-Agent so servers that block bots/scrapers allow the request (avoids 403).
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Extract the first URL-like token from text (e.g. FETCH_URL arg that may contain extra words).
/// Takes the first substring that starts with http:// or https:// and runs until first whitespace or newline.
/// Returns None if no such substring exists.
pub fn extract_first_url(arg: &str) -> Option<String> {
    let s = arg.trim();
    let start = s.find("https://").or_else(|| s.find("http://"))?;
    let rest = &s[start..];
    let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
    let url = rest[..end].trim_end_matches(['.', ',', ';', ':']);
    if url.is_empty() {
        return None;
    }
    Some(url.to_string())
}

/// Validate and normalize URL for fetch. Returns clear error for invalid or IDN URLs.
fn validate_fetch_url(url: &str) -> Result<Url, String> {
    let parsed = Url::parse(url).map_err(|e| {
        let err_str = e.to_string();
        if err_str.to_lowercase().contains("international domain name") || err_str.contains("IDN") {
            "Invalid URL: international domain names (IDN) are not supported. Use punycode (e.g. xn--...) or a different URL.".to_string()
        } else {
            format!("Invalid URL for FETCH_URL: {}", e)
        }
    })?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("URL must use http or https".to_string()),
    }
    Ok(parsed)
}

// ---------------------------------------------------------------------------
// SSRF protection: block private/loopback/link-local/metadata URLs
// ---------------------------------------------------------------------------

fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || is_ipv6_link_local(v6)
                || is_ipv6_unique_local(v6)
                || v6.to_ipv4_mapped().is_some_and(|v4| {
                    v4.is_loopback()
                        || v4.is_private()
                        || v4.is_link_local()
                        || v4.is_unspecified()
                        || v4.is_broadcast()
                })
        }
    }
}

fn is_ipv6_link_local(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

fn is_ipv6_unique_local(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

/// Whether `host` matches one `ssrfAllowedHosts` entry. Matching is always against the **hostname**
/// only (never the full URL). Semantics align with OpenClaw-style config: trim the entry; empty → no
/// match; `contains:` prefix → case-insensitive substring; `*` / `**` in the entry → glob converted
/// to a regex (other characters escaped, `*` and `**` each → `.*`); otherwise → case-insensitive
/// equality. Patterns are anchored to the full hostname so e.g. `*.corp.test` does not match
/// `evil.corp.test.evil.com`.
pub(crate) fn host_matches_ssrf_allowlist(host: &str, entry: &str) -> bool {
    let entry = entry.trim();
    if entry.is_empty() {
        return false;
    }
    let host_lower = host.to_ascii_lowercase();

    if let Some(needle) = entry.strip_prefix(SSRF_ALLOWLIST_CONTAINS_PREFIX) {
        let needle = needle.trim();
        if needle.is_empty() {
            return false;
        }
        return host_lower.contains(&needle.to_ascii_lowercase());
    }

    let pat_lower = entry.to_ascii_lowercase();
    if pat_lower.contains('*') {
        return host_glob_regex_matches(&host_lower, &pat_lower);
    }

    host_lower == pat_lower
}

fn host_glob_regex_matches(host_lower: &str, pattern_lower: &str) -> bool {
    let regex_src = host_glob_hostname_to_regex(pattern_lower);
    let Ok(re) = Regex::new(&regex_src) else {
        warn!(
            "SSRF allowlist: could not compile regex from glob {:?}, ignoring",
            pattern_lower
        );
        return false;
    };
    re.is_match(host_lower)
}

/// Build `^...$` regex: literal segments regex-escaped; each `*` run (including `**`) becomes `.*`.
fn host_glob_hostname_to_regex(pattern_lower: &str) -> String {
    let mut out = String::with_capacity(pattern_lower.len().saturating_mul(2));
    out.push('^');
    let mut literal = String::new();
    let mut chars = pattern_lower.chars().peekable();
    let flush = |lit: &mut String, o: &mut String| {
        if !lit.is_empty() {
            o.push_str(&regex::escape(lit));
            lit.clear();
        }
    };
    while let Some(c) = chars.next() {
        if c == '*' {
            flush(&mut literal, &mut out);
            if chars.peek() == Some(&'*') {
                chars.next();
            }
            out.push_str(".*");
        } else {
            literal.push(c);
        }
    }
    flush(&mut literal, &mut out);
    out.push('$');
    out
}

fn host_on_ssrf_allowlist(host: &str, allowed_hosts: &[String]) -> bool {
    allowed_hosts
        .iter()
        .any(|e| host_matches_ssrf_allowlist(host, e))
}

/// Validate that a URL does not target a private/loopback/link-local/metadata network.
/// Rejects URLs with userinfo (credentials). Resolves the hostname to IPs and rejects if
/// any resolved IP is on the blocklist. Hosts listed in `allowed_hosts` bypass the IP check.
pub fn validate_url_no_ssrf(url: &Url, allowed_hosts: &[String]) -> Result<(), String> {
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URL contains credentials (userinfo) and was blocked for security".to_string());
    }
    let host = url.host_str().ok_or("URL has no host")?;
    if host_on_ssrf_allowlist(host, allowed_hosts) {
        info!(
            "SSRF guard: host '{}' matches ssrfAllowedHosts, skipping IP check",
            host
        );
        return Ok(());
    }
    let port = url
        .port()
        .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
    let addr_str = format!("{}:{}", host, port);
    let addrs: Vec<std::net::SocketAddr> = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for '{}': {}", host, e))?
        .collect();
    if addrs.is_empty() {
        return Err(format!(
            "DNS resolution for '{}' returned no addresses",
            host
        ));
    }
    for addr in &addrs {
        if is_blocked_ip(&addr.ip()) {
            warn!(
                "SSRF guard: blocked URL {} — host '{}' resolves to private/loopback address {}",
                url,
                host,
                addr.ip()
            );
            return Err(format!(
                "URL targets a private network ({}) and was blocked to prevent SSRF",
                addr.ip()
            ));
        }
    }
    Ok(())
}

/// Validate a `file:` URL for **BROWSER_NAVIGATE** / CDP navigation. Only local `.html` / `.htm`
/// files are allowed; the path is canonicalized so symlink tricks cannot bypass the suffix check.
pub fn validate_file_url_for_browser_navigation(url: &Url) -> Result<PathBuf, String> {
    if !url.scheme().eq_ignore_ascii_case("file") {
        return Err("internal: expected file: URL".to_string());
    }
    let path = url
        .to_file_path()
        .map_err(|_| "file URL could not be converted to a local path".to_string())?;
    let meta =
        std::fs::metadata(&path).map_err(|e| format!("file URL path not accessible: {}", e))?;
    if !meta.is_file() {
        return Err("file navigation only allows a regular file (not a directory)".to_string());
    }
    let canon = std::fs::canonicalize(&path)
        .map_err(|e| format!("could not canonicalize file URL path: {}", e))?;
    let ext = canon
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("html") | Some("htm") => Ok(canon),
        _ => Err(
            "file navigation is limited to .html or .htm files (after resolving symlinks)"
                .to_string(),
        ),
    }
}

/// Normalize the operator/model URL string and run pre-navigation checks for CDP (`Page.navigate`).
///
/// - Adds `https://` when no scheme is present (existing behaviour).
/// - Preserves `file://` URLs; [`validate_file_url_for_browser_navigation`] applies.
/// - **http/https:** [`validate_url_no_ssrf`].
pub fn normalize_and_validate_cdp_navigation_url(url: &str) -> Result<String, String> {
    let u = url.trim();
    if u.is_empty() {
        return Err("Navigation URL is empty".to_string());
    }
    let normalized = if u.len() >= 7 && u[..7].eq_ignore_ascii_case("file://") {
        u.to_string()
    } else if u.starts_with("http://") || u.starts_with("https://") {
        u.to_string()
    } else {
        format!("https://{}", u)
    };
    match Url::parse(&normalized) {
        Ok(parsed) if parsed.scheme().eq_ignore_ascii_case("file") => {
            let canon = validate_file_url_for_browser_navigation(&parsed)?;
            Url::from_file_path(&canon)
                .map(|u| u.to_string())
                .map_err(|()| {
                    "file navigation: could not encode canonical path as file URL".to_string()
                })
        }
        Ok(parsed) => {
            let allowed = crate::config::Config::ssrf_allowed_hosts();
            validate_url_no_ssrf(&parsed, &allowed)?;
            Ok(normalized)
        }
        Err(_) => Ok(normalized),
    }
}

// ---------------------------------------------------------------------------
// Proxy env vs DNS-based SSRF (egress semantic gap)
// ---------------------------------------------------------------------------

/// Environment variable names commonly honored by reqwest, curl, and Chrome for HTTP(S) proxies.
const PROXY_ENV_VARS: &[&str] = &[
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
];

/// True when a standard HTTP(S) proxy environment variable is set to a non-empty value.
///
/// In that situation, [`validate_url_no_ssrf`] (direct DNS in Rust) may not describe how
/// [`reqwest`] or Chrome actually connects, so operator-facing behaviour should warn or block.
pub fn proxy_env_likely_active() -> bool {
    PROXY_ENV_VARS.iter().any(|key| {
        std::env::var(key)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    })
}

pub(crate) fn strict_ssrf_proxy_env_rejection(tool_label: &str) -> String {
    format!(
        "{} refused: HTTP_PROXY, HTTPS_PROXY, or ALL_PROXY (or lowercase variants) is set. \
DNS-based SSRF checks in mac-stats may not match actual egress: reqwest and Chrome can forward or re-resolve targets via the proxy. \
Unset those variables for the mac-stats process, or disable strict mode by setting strictSsrfRejectWhenProxyEnv to false in ~/.mac-stats/config.json (or MAC_STATS_STRICT_SSRF_REJECT_WHEN_PROXY_ENV=0).",
        tool_label
    )
}

/// If proxy env is active: when [`crate::config::Config::strict_ssrf_reject_when_proxy_env`] is on,
/// returns `Err` before outbound work. Otherwise returns `Ok(Some(line))` to prepend to tool output
/// (and logs once at `info` for this call). Returns `Ok(None)` when no proxy vars are set.
pub fn ssrf_proxy_env_notice_for_tool(tool_label: &str) -> Result<Option<String>, String> {
    if !proxy_env_likely_active() {
        return Ok(None);
    }
    if crate::config::Config::strict_ssrf_reject_when_proxy_env() {
        return Err(strict_ssrf_proxy_env_rejection(tool_label));
    }
    info!(
        target: "browser/security",
        "SSRF/proxy: env proxy vars set during {}; DNS SSRF pre-check may not match reqwest/Chrome egress",
        tool_label
    );
    Ok(Some(format!(
        "[SSRF / proxy] HTTP(S) proxy environment variables are set. DNS-based SSRF validation may not match how this request reaches the host via reqwest or Chrome. Unset HTTP_PROXY/HTTPS_PROXY/ALL_PROXY for the mac-stats process if you need strict SSRF semantics.\n"
    )))
}

/// Whether to inject model-facing proxy warnings for a server-side HTTP fetch.
#[derive(Clone, Copy, Debug)]
pub(crate) enum FetchProxyContext {
    /// Scheduler, generic Tauri `fetch_page`, etc.: apply strict fail-closed only; no log, no prepend.
    Background,
    /// Agent `FETCH_URL` (Discord/router/UI tool loop): strict block, else log + optional prepend.
    AgentTool,
}

fn ssrf_proxy_policy_for_http_fetch(
    context: FetchProxyContext,
    post: bool,
) -> Result<Option<String>, String> {
    match context {
        FetchProxyContext::Background => {
            if proxy_env_likely_active()
                && crate::config::Config::strict_ssrf_reject_when_proxy_env()
            {
                Err(strict_ssrf_proxy_env_rejection("FETCH_URL"))
            } else {
                Ok(None)
            }
        }
        FetchProxyContext::AgentTool => {
            let label = if post {
                "FETCH_URL (POST)"
            } else {
                "FETCH_URL"
            };
            ssrf_proxy_env_notice_for_tool(label)
        }
    }
}

/// Validate a redirect target the same way as the initial URL: resolve host and reject
/// blocklist IPs, empty resolution, or DNS failure. Does not follow redirects we cannot verify.
fn check_redirect_target_ssrf(url: &Url, allowed_hosts: &[String]) -> Result<(), std::io::Error> {
    if !url.username().is_empty() || url.password().is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Redirect with credentials blocked (SSRF guard)",
        ));
    }
    let Some(host) = url.host_str() else {
        return Ok(());
    };
    if host_on_ssrf_allowlist(host, allowed_hosts) {
        return Ok(());
    }
    let port = url
        .port()
        .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
    let addr_str = format!("{}:{}", host, port);
    let addrs: Vec<std::net::SocketAddr> = addr_str
        .to_socket_addrs()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "Redirect host DNS resolution failed for '{}' (SSRF guard): {}",
                    host, e
                ),
            )
        })?
        .collect();
    if addrs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "Redirect host '{}' resolved to no addresses (SSRF guard)",
                host
            ),
        ));
    }
    for addr in &addrs {
        if is_blocked_ip(&addr.ip()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "Redirect to private network ({}) blocked (SSRF guard)",
                    addr.ip()
                ),
            ));
        }
    }
    Ok(())
}

/// Build a reqwest redirect policy that blocks redirects to private/loopback networks.
fn ssrf_redirect_policy(allowed_hosts: Vec<String>) -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(move |attempt| {
        if let Err(e) = check_redirect_target_ssrf(attempt.url(), &allowed_hosts) {
            return attempt.error(e);
        }
        if attempt.previous().len() > 10 {
            attempt.stop()
        } else {
            attempt.follow()
        }
    })
}

/// Fetch a URL and return the response body as text.
/// Extracts first URL from arg (task-002), validates, uses same timeout/SSL policy as website monitors.
/// Used by schedulers and internal callers; does **not** prepend the agent SSRF/proxy notice (see
/// [`fetch_page_content_for_agent`]).
pub fn fetch_page_content(url: &str) -> Result<String, String> {
    fetch_page_content_with_proxy_context(url, FetchProxyContext::Background)
}

/// Same as [`fetch_page_content`], but for model-facing **FETCH_URL** tool paths: when proxy env vars
/// are set and strict mode is off, logs at `browser/security` and prepends a one-line notice to the body.
pub fn fetch_page_content_for_agent(url: &str) -> Result<String, String> {
    fetch_page_content_with_proxy_context(url, FetchProxyContext::AgentTool)
}

fn fetch_page_content_with_proxy_context(
    url: &str,
    proxy_ctx: FetchProxyContext,
) -> Result<String, String> {
    let raw = url.trim();
    let url_str = extract_first_url(raw).ok_or_else(|| {
        "Invalid URL for FETCH_URL: no http:// or https:// URL found. Provide a single URL only.".to_string()
    })?;
    let parsed = validate_fetch_url(&url_str)?;
    let allowed_hosts = crate::config::Config::ssrf_allowed_hosts();
    validate_url_no_ssrf(&parsed, &allowed_hosts)?;
    let proxy_note = ssrf_proxy_policy_for_http_fetch(proxy_ctx, false)?;
    let url = parsed.as_str();

    info!("Fetch page: GET {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .redirect(ssrf_redirect_policy(allowed_hosts))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let resp = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let code = status.as_u16();
        let reason = status.canonical_reason().unwrap_or("");
        warn!("Fetch page failed: {} {} for URL {}", code, reason, url);
        return Err(format!("HTTP {}: {}", code, reason));
    }

    let body = resp.text().map_err(|e| format!("Read body: {}", e))?;

    let body = truncate_fetch_body_if_needed(body);
    let n = body.chars().count();
    info!("Fetch page: fetched {} chars from {}", n, url);
    Ok(match proxy_note {
        Some(prefix) => format!("{}{}", prefix, body),
        None => body,
    })
}

/// POST `application/x-www-form-urlencoded` to a URL and return the response body as text.
/// Same validation, SSRF policy, redirect handling, timeout, and truncation as [`fetch_page_content`].
#[allow(dead_code)] // Public API for embedders; in-crate callers use `fetch_page_post_form_urlencoded_for_agent`.
pub fn fetch_page_post_form_urlencoded(
    url: &str,
    pairs: &[(String, String)],
) -> Result<String, String> {
    fetch_page_post_form_urlencoded_with_context(url, pairs, FetchProxyContext::Background)
}

pub(crate) fn fetch_page_post_form_urlencoded_for_agent(
    url: &str,
    pairs: &[(String, String)],
) -> Result<String, String> {
    fetch_page_post_form_urlencoded_with_context(url, pairs, FetchProxyContext::AgentTool)
}

fn fetch_page_post_form_urlencoded_with_context(
    url: &str,
    pairs: &[(String, String)],
    proxy_ctx: FetchProxyContext,
) -> Result<String, String> {
    let raw = url.trim();
    let url_str = extract_first_url(raw).ok_or_else(|| {
        "Invalid URL for POST: no http:// or https:// URL found. Provide a single URL only."
            .to_string()
    })?;
    let parsed = validate_fetch_url(&url_str)?;
    let allowed_hosts = crate::config::Config::ssrf_allowed_hosts();
    validate_url_no_ssrf(&parsed, &allowed_hosts)?;
    let proxy_note = ssrf_proxy_policy_for_http_fetch(proxy_ctx, true)?;
    let url = parsed.as_str();

    let mut ser = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in pairs {
        ser.append_pair(k, v);
    }
    let body = ser.finish();

    info!(
        "Fetch page: POST {} ({} urlencoded field(s), {} bytes body)",
        url,
        pairs.len(),
        body.len()
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .redirect(ssrf_redirect_policy(allowed_hosts))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let resp = client
        .post(url)
        .header("User-Agent", USER_AGENT)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .body(body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let code = status.as_u16();
        let reason = status.canonical_reason().unwrap_or("");
        warn!(
            "Fetch page POST failed: {} {} for URL {}",
            code, reason, url
        );
        return Err(format!("HTTP {}: {}", code, reason));
    }

    let body = resp.text().map_err(|e| format!("Read body: {}", e))?;
    let body = truncate_fetch_body_if_needed(body);
    let n = body.chars().count();
    info!("Fetch page: POST fetched {} chars from {}", n, url);
    Ok(match proxy_note {
        Some(prefix) => format!("{}{}", prefix, body),
        None => body,
    })
}

/// Tauri command: fetch a URL and return body as text (for frontend or tools).
#[tauri::command]
pub async fn fetch_page(url: String) -> Result<String, String> {
    let url = url.clone();
    tokio::task::spawn_blocking(move || fetch_page_content(&url))
        .await
        .map_err(|e| format!("Task join: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_ssrf_proxy_env_rejection_mentions_config() {
        let s = strict_ssrf_proxy_env_rejection("FETCH_URL");
        assert!(s.starts_with("FETCH_URL refused:"));
        assert!(s.contains("strictSsrfRejectWhenProxyEnv"));
        assert!(s.contains("MAC_STATS_STRICT_SSRF_REJECT_WHEN_PROXY_ENV"));
    }

    #[test]
    fn ssrf_blocks_loopback_ipv4() {
        let url = Url::parse("http://127.0.0.1/secret").unwrap();
        assert!(validate_url_no_ssrf(&url, &[]).is_err());
    }

    #[test]
    fn ssrf_blocks_loopback_ipv6() {
        let url = Url::parse("http://[::1]/secret").unwrap();
        assert!(validate_url_no_ssrf(&url, &[]).is_err());
    }

    #[test]
    fn ssrf_blocks_private_10() {
        let url = Url::parse("http://10.0.0.1/admin").unwrap();
        assert!(validate_url_no_ssrf(&url, &[]).is_err());
    }

    #[test]
    fn ssrf_blocks_private_172() {
        let url = Url::parse("http://172.16.0.1/").unwrap();
        assert!(validate_url_no_ssrf(&url, &[]).is_err());
    }

    #[test]
    fn ssrf_blocks_private_192() {
        let url = Url::parse("http://192.168.1.1/").unwrap();
        assert!(validate_url_no_ssrf(&url, &[]).is_err());
    }

    #[test]
    fn ssrf_blocks_metadata_endpoint() {
        let url = Url::parse("http://169.254.169.254/latest/meta-data/").unwrap();
        assert!(validate_url_no_ssrf(&url, &[]).is_err());
    }

    #[test]
    fn ssrf_blocks_userinfo() {
        let url = Url::parse("http://evil@127.0.0.1/").unwrap();
        let err = validate_url_no_ssrf(&url, &[]).unwrap_err();
        assert!(err.contains("credentials"));
    }

    #[test]
    fn ssrf_blocks_userinfo_with_password() {
        let url = Url::parse("http://user:pass@example.com/").unwrap();
        let err = validate_url_no_ssrf(&url, &[]).unwrap_err();
        assert!(err.contains("credentials"));
    }

    #[test]
    fn ssrf_allows_public_url() {
        let url = Url::parse("https://www.example.com/page").unwrap();
        assert!(validate_url_no_ssrf(&url, &[]).is_ok());
    }

    #[test]
    fn ssrf_allowlist_bypasses_check() {
        let url = Url::parse("http://127.0.0.1:3000/api").unwrap();
        let allowed = vec!["127.0.0.1".to_string()];
        assert!(validate_url_no_ssrf(&url, &allowed).is_ok());
    }

    #[test]
    fn ssrf_allowlist_case_insensitive() {
        let url = Url::parse("http://MyLocalService:8080/").unwrap();
        let allowed = vec!["mylocalservice".to_string()];
        assert!(validate_url_no_ssrf(&url, &allowed).is_ok());
    }

    #[test]
    fn ssrf_allowlist_wildcard_subdomain_corp_test() {
        let url = Url::parse("http://a.corp.test/path").unwrap();
        let allowed = vec!["*.corp.test".to_string()];
        assert!(validate_url_no_ssrf(&url, &allowed).is_ok());
        assert!(host_matches_ssrf_allowlist("a.corp.test", "*.corp.test"));
        assert!(!host_matches_ssrf_allowlist(
            "evil.corp.test.evil.com",
            "*.corp.test"
        ));
    }

    #[test]
    fn host_matches_ssrf_allowlist_internal_prefix_pattern() {
        assert!(host_matches_ssrf_allowlist(
            "internal-app1.local",
            "internal-*"
        ));
        assert!(!host_matches_ssrf_allowlist("app1.internal", "internal-*"));
    }

    #[test]
    fn ssrf_allowlist_wildcard_loopback_octet() {
        let url = Url::parse("http://127.0.0.1:8080/").unwrap();
        let allowed = vec!["127.0.*".to_string()];
        assert!(validate_url_no_ssrf(&url, &allowed).is_ok());
    }

    #[test]
    fn ssrf_allowlist_contains_prefix_bypass_private_ip_host() {
        let url = Url::parse("http://127.0.0.1:9999/").unwrap();
        let allowed = vec!["contains:127.0".to_string()];
        assert!(validate_url_no_ssrf(&url, &allowed).is_ok());
    }

    #[test]
    fn ssrf_redirect_check_wildcard_allowlist() {
        let url = Url::parse("http://127.0.0.1/").unwrap();
        let allowed = vec!["127.*".to_string()];
        assert!(check_redirect_target_ssrf(&url, &allowed).is_ok());
    }

    #[test]
    fn host_matches_ssrf_allowlist_empty_entry_no_match() {
        assert!(!host_matches_ssrf_allowlist("a.example.com", ""));
        assert!(!host_matches_ssrf_allowlist("a.example.com", "   "));
        assert!(!host_matches_ssrf_allowlist("a.example.com", "contains:"));
        assert!(!host_matches_ssrf_allowlist("a.example.com", "contains:  "));
    }

    #[test]
    fn host_matches_ssrf_allowlist_glob_star_matches_any_label() {
        assert!(host_matches_ssrf_allowlist("anything", "*"));
        assert!(host_matches_ssrf_allowlist("x.y.z", "*"));
    }

    #[test]
    fn host_matches_ssrf_allowlist_exact_still_not_substring() {
        assert!(!host_matches_ssrf_allowlist("notlocalhost", "localhost"));
        assert!(host_matches_ssrf_allowlist("localhost", "localhost"));
    }

    #[test]
    fn is_blocked_ip_loopback() {
        assert!(is_blocked_ip(&IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)));
        assert!(is_blocked_ip(&IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn is_blocked_ip_private_ranges() {
        assert!(is_blocked_ip(&IpAddr::V4(std::net::Ipv4Addr::new(
            10, 0, 0, 1
        ))));
        assert!(is_blocked_ip(&IpAddr::V4(std::net::Ipv4Addr::new(
            172, 16, 0, 1
        ))));
        assert!(is_blocked_ip(&IpAddr::V4(std::net::Ipv4Addr::new(
            192, 168, 0, 1
        ))));
    }

    #[test]
    fn is_blocked_ip_link_local() {
        assert!(is_blocked_ip(&IpAddr::V4(std::net::Ipv4Addr::new(
            169, 254, 169, 254
        ))));
    }

    #[test]
    fn is_blocked_ip_public_is_ok() {
        assert!(!is_blocked_ip(&IpAddr::V4(std::net::Ipv4Addr::new(
            8, 8, 8, 8
        ))));
        assert!(!is_blocked_ip(&IpAddr::V4(std::net::Ipv4Addr::new(
            1, 1, 1, 1
        ))));
    }

    #[test]
    fn is_blocked_ip_ipv4_mapped_broadcast() {
        // ::ffff:255.255.255.255 — must match plain IPv4 broadcast handling (SSRF review nit).
        let mapped = std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xffff, 0xffff);
        assert!(is_blocked_ip(&IpAddr::V6(mapped)));
    }

    #[test]
    fn truncate_fetch_body_short_unchanged() {
        let s = "hello".to_string();
        assert_eq!(truncate_fetch_body_if_needed(s.clone()), s);
    }

    #[test]
    fn truncate_fetch_body_uses_configured_max() {
        let over = MAX_BODY_CHARS + 100;
        let body: String = "x".repeat(over);
        let out = truncate_fetch_body_if_needed(body);
        assert!(out.ends_with(FETCH_BODY_TRUNC_SUFFIX));
        assert!(out.contains("..."));
        assert!(out.chars().count() <= MAX_BODY_CHARS);
    }

    #[test]
    fn truncate_fetch_body_ellipse_then_explicit_suffix_for_llm() {
        // 022 feature review F7: middle omission (`...` via `ellipse`) plus explicit
        // ` [content truncated]` so the model is not left guessing whether the page ended.
        let over = MAX_BODY_CHARS + 50_000;
        let body: String = "x".repeat(over);
        let out = truncate_fetch_body_if_needed(body);
        assert!(out.ends_with(FETCH_BODY_TRUNC_SUFFIX));
        let without_suffix = out.strip_suffix(FETCH_BODY_TRUNC_SUFFIX).expect("suffix");
        assert!(
            without_suffix.contains("..."),
            "ellipsed payload should show middle omission"
        );
        assert!(out.chars().count() <= MAX_BODY_CHARS);
    }

    #[test]
    fn extract_first_url_none_without_scheme() {
        assert_eq!(extract_first_url("hello world"), None);
        assert_eq!(extract_first_url(""), None);
    }

    #[test]
    fn extract_first_url_trims_arg_and_strips_trailing_punctuation() {
        assert_eq!(
            extract_first_url("  https://example.com/usecases. The rest"),
            Some("https://example.com/usecases".to_string())
        );
        assert_eq!(
            extract_first_url("see https://a.org/path, ok"),
            Some("https://a.org/path".to_string())
        );
        assert_eq!(
            extract_first_url("https://b.test/x;"),
            Some("https://b.test/x".to_string())
        );
    }

    #[test]
    fn extract_first_url_first_token_only() {
        assert_eq!(
            extract_first_url("FETCH_URL: https://one.com/ https://two.com/"),
            Some("https://one.com/".to_string())
        );
    }

    #[test]
    fn extract_first_url_prefers_https_substring_over_http() {
        // `find("https://")` is tried before `find("http://")` so a later https URL wins over an earlier http URL.
        assert_eq!(
            extract_first_url("http://old.example/ then https://new.example/path"),
            Some("https://new.example/path".to_string())
        );
    }

    #[test]
    fn extract_first_url_http_when_no_https() {
        assert_eq!(
            extract_first_url("use http://localhost:8080/api"),
            Some("http://localhost:8080/api".to_string())
        );
    }

    #[test]
    fn ssrf_redirect_check_blocks_private() {
        let url = Url::parse("http://192.168.1.1/").unwrap();
        let err = check_redirect_target_ssrf(&url, &[]).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(err.to_string().contains("private network"));
    }

    #[test]
    fn ssrf_redirect_check_allowlist_bypasses() {
        let url = Url::parse("http://127.0.0.1/").unwrap();
        let allowed = vec!["127.0.0.1".to_string()];
        assert!(check_redirect_target_ssrf(&url, &allowed).is_ok());
    }

    #[test]
    fn ssrf_redirect_check_allows_public() {
        let url = Url::parse("https://www.example.com/").unwrap();
        assert!(check_redirect_target_ssrf(&url, &[]).is_ok());
    }

    #[test]
    fn ssrf_redirect_dns_failure_is_error() {
        // .invalid is reserved (RFC 6761); should not resolve in normal DNS.
        let url = Url::parse("http://mac-stats-ssrf-redirect-test.invalid/").unwrap();
        let err = check_redirect_target_ssrf(&url, &[]).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        let msg = err.to_string();
        assert!(
            msg.contains("DNS resolution failed") || msg.contains("SSRF guard"),
            "unexpected message: {}",
            msg
        );
    }

    #[test]
    fn cdp_nav_url_accepts_existing_html_file() {
        let dir = std::env::temp_dir().join(format!("mac_stats_nav_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("t.html");
        std::fs::write(&p, "<html><body>x</body></html>").unwrap();
        let url = Url::from_file_path(&p).unwrap().to_string();
        let out = normalize_and_validate_cdp_navigation_url(&url).unwrap();
        assert!(out.starts_with("file:///"), "{}", out);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cdp_nav_url_rejects_non_html_file() {
        let dir =
            std::env::temp_dir().join(format!("mac_stats_nav_test_txt_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("x.txt");
        std::fs::write(&p, "nope").unwrap();
        let url = Url::from_file_path(&p).unwrap().to_string();
        assert!(normalize_and_validate_cdp_navigation_url(&url).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
