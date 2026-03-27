//! Domain allow/block lists for BROWSER_* navigation (CDP and HTTP fallback).
//!
//! - **Allowlist:** `BROWSER_ALLOWED_DOMAINS` or `MAC_STATS_BROWSER_ALLOWED_DOMAINS` (comma-separated).
//!   Empty / unset = allow any host that passes the blocklist.
//! - **Blocklist:** Bundled defaults in `data/default_browser_blocklist.txt`, replaced entirely if
//!   `~/.mac-stats/browser-security-blocklist.txt` exists, plus `BROWSER_BLOCKED_DOMAINS` /
//!   `MAC_STATS_BROWSER_BLOCKED_DOMAINS` and optional `config.json` `browserBlockedDomains`.
//!
//! Also reads the same keys from `.config.env` (cwd, `src-tauri/`, `~/.mac-stats/`) like other agents.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use url::Url;

const DEFAULT_BLOCKLIST: &str = include_str!("data/default_browser_blocklist.txt");

fn home_mac_stats_blocklist_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        Path::new(&h)
            .join(".mac-stats")
            .join("browser-security-blocklist.txt")
    })
}

fn parse_domain_list_line(line: &str) -> Option<String> {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') {
        return None;
    }
    Some(t.to_ascii_lowercase())
}

fn split_comma_patterns(s: &str) -> Vec<String> {
    s.split(',').filter_map(parse_domain_list_line).collect()
}

fn read_key_from_config_env(path: &Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().find(|l| l.trim().starts_with(key))?;
    let (_, v) = line.split_once('=')?;
    let v = v.trim();
    if v.is_empty() {
        return None;
    }
    Some(v.to_string())
}

fn read_browser_list_from_config_files(key: &str) -> Option<String> {
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".config.env");
        if p.is_file() {
            if let Some(v) = read_key_from_config_env(&p, key) {
                return Some(v);
            }
        }
        let p_src = cwd.join("src-tauri").join(".config.env");
        if p_src.is_file() {
            if let Some(v) = read_key_from_config_env(&p_src, key) {
                return Some(v);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            if let Some(v) = read_key_from_config_env(&p, key) {
                return Some(v);
            }
        }
    }
    None
}

fn read_browser_list_json_array_or_string(key: &str) -> Option<Vec<String>> {
    let config_path = crate::config::Config::config_file_path();
    let content = std::fs::read_to_string(&config_path).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    let v = json.get(key)?;
    if let Some(s) = v.as_str() {
        let p = split_comma_patterns(s);
        return if p.is_empty() { None } else { Some(p) };
    }
    let arr = v.as_array()?;
    let mut out = Vec::new();
    for x in arr {
        if let Some(s) = x.as_str() {
            if let Some(p) = parse_domain_list_line(s) {
                out.push(p);
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn effective_allowed_patterns() -> Vec<String> {
    let from_env = std::env::var("MAC_STATS_BROWSER_ALLOWED_DOMAINS")
        .ok()
        .or_else(|| std::env::var("BROWSER_ALLOWED_DOMAINS").ok());
    if let Some(s) = from_env {
        let v = split_comma_patterns(&s);
        if !v.is_empty() {
            return v;
        }
    }
    if let Some(s) = read_browser_list_from_config_files("BROWSER_ALLOWED_DOMAINS=") {
        let v = split_comma_patterns(&s);
        if !v.is_empty() {
            return v;
        }
    }
    read_browser_list_json_array_or_string("browserAllowedDomains").unwrap_or_default()
}

fn base_blocked_patterns_from_files() -> Vec<String> {
    if let Some(p) = home_mac_stats_blocklist_path() {
        if p.is_file() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                let v: Vec<String> = content.lines().filter_map(parse_domain_list_line).collect();
                if !v.is_empty() {
                    return v;
                }
            }
        }
    }
    DEFAULT_BLOCKLIST
        .lines()
        .filter_map(parse_domain_list_line)
        .collect()
}

fn extra_blocked_from_config() -> Vec<String> {
    let from_env = std::env::var("MAC_STATS_BROWSER_BLOCKED_DOMAINS")
        .ok()
        .or_else(|| std::env::var("BROWSER_BLOCKED_DOMAINS").ok());
    if let Some(s) = from_env {
        let v = split_comma_patterns(&s);
        if !v.is_empty() {
            return v;
        }
    }
    if let Some(s) = read_browser_list_from_config_files("BROWSER_BLOCKED_DOMAINS=") {
        let v = split_comma_patterns(&s);
        if !v.is_empty() {
            return v;
        }
    }
    read_browser_list_json_array_or_string("browserBlockedDomains").unwrap_or_default()
}

fn blocked_patterns_merged() -> Vec<String> {
    let mut v = base_blocked_patterns_from_files();
    v.extend(extra_blocked_from_config());
    v.sort();
    v.dedup();
    v
}

fn is_glob_pattern(p: &str) -> bool {
    p.contains('*')
}

fn host_matches_glob(host: &str, glob_pat: &str) -> bool {
    let Some(rest) = glob_pat.strip_prefix("*.") else {
        return false;
    };
    if rest.is_empty() {
        return false;
    }
    host == rest || host.ends_with(&format!(".{rest}"))
}

fn host_matches_plain(host: &str, pat: &str) -> bool {
    if pat.is_empty() {
        return false;
    }
    host == pat || host.ends_with(&format!(".{pat}"))
}

fn host_matches_pattern(host: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }
    if is_glob_pattern(pattern) {
        host_matches_glob(host, pattern)
    } else {
        host_matches_plain(host, pattern)
    }
}

struct DomainPolicyEngine {
    allowed: Vec<String>,
    blocked_globs: Vec<String>,
    blocked_plain: Vec<String>,
    blocked_exact: HashSet<String>,
}

impl DomainPolicyEngine {
    fn load() -> Self {
        let allowed = effective_allowed_patterns();
        let blocked = blocked_patterns_merged();
        let mut blocked_globs = Vec::new();
        let mut blocked_plain = Vec::new();
        for p in blocked {
            if is_glob_pattern(&p) {
                blocked_globs.push(p);
            } else {
                blocked_plain.push(p);
            }
        }
        let blocked_exact: HashSet<String> = if blocked_plain.len() > 100 {
            blocked_plain.iter().cloned().collect()
        } else {
            HashSet::new()
        };
        Self {
            allowed,
            blocked_globs,
            blocked_plain,
            blocked_exact,
        }
    }

    fn host_blocked(&self, host: &str) -> bool {
        if !self.blocked_exact.is_empty() && self.blocked_exact.contains(host) {
            return true;
        }
        for p in &self.blocked_plain {
            if host_matches_plain(host, p) {
                return true;
            }
        }
        for g in &self.blocked_globs {
            if host_matches_glob(host, g) {
                return true;
            }
        }
        false
    }

    fn host_allowed_by_whitelist(&self, host: &str) -> bool {
        if self.allowed.is_empty() {
            return true;
        }
        for p in &self.allowed {
            if host_matches_pattern(host, p) {
                return true;
            }
        }
        false
    }

    fn navigation_permitted(&self, host: &str) -> bool {
        if self.host_blocked(host) {
            return false;
        }
        self.host_allowed_by_whitelist(host)
    }
}

fn policy_engine() -> DomainPolicyEngine {
    DomainPolicyEngine::load()
}

/// True for `http:` / `https:` URLs that have a host (domain policy applies).
pub fn url_scheme_subject_to_domain_policy(url: &str) -> bool {
    let Ok(u) = Url::parse(url) else {
        return false;
    };
    matches!(u.scheme(), "http" | "https") && u.host_str().is_some()
}

fn host_for_policy(url: &str) -> Option<String> {
    let u = Url::parse(url).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    u.host_str().map(|h| h.to_ascii_lowercase())
}

/// Public API: returns whether navigation to `url` is permitted (allowlist + blocklist).
pub fn is_navigation_allowed(url: &str) -> bool {
    let Some(host) = host_for_policy(url) else {
        return true;
    };
    policy_engine().navigation_permitted(&host)
}

/// When navigation is not allowed, human-readable host for errors (no credentials/path).
pub fn host_label_for_policy_message(url: &str) -> String {
    host_for_policy(url).unwrap_or_else(|| "(invalid URL)".to_string())
}

/// `Err` message when the model supplied URL must be blocked before navigation.
pub fn navigation_precheck_error(url: &str) -> Option<String> {
    if !url_scheme_subject_to_domain_policy(url) {
        return None;
    }
    if is_navigation_allowed(url) {
        return None;
    }
    let host = host_label_for_policy_message(url);
    crate::mac_stats_warn!(
        "browser/security",
        "Browser navigation security: blocked request to host {}",
        host
    );
    Some(format!("Navigation to {} blocked by security policy", host))
}

/// Used by HTTP fallback and CDP after redirects / clicks.
pub fn require_navigation_allowed(url: &str) -> Result<(), String> {
    if let Some(e) = navigation_precheck_error(url) {
        Err(e)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_local_matches() {
        assert!(host_matches_glob("m.local", "*.local"));
        assert!(host_matches_glob("local", "*.local"));
        assert!(!host_matches_glob("notlocal", "*.local"));
    }

    #[test]
    fn plain_parent_matches_subdomain() {
        assert!(host_matches_plain("www.chase.com", "chase.com"));
        assert!(host_matches_plain("chase.com", "chase.com"));
        assert!(!host_matches_plain("evilchase.com", "chase.com"));
    }
}
