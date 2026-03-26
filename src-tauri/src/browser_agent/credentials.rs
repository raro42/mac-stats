//! Domain-scoped credential placeholders for `BROWSER_INPUT`.
//!
//! Secrets live in `~/.mac-stats/browser-credentials.toml` (see [`crate::config::Config::browser_credentials_toml_path`]).
//! The model uses `<secret>placeholder_name</secret>` in tool input; values are substituted at CDP/HTTP input time only.

use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use regex::Regex;
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;

use crate::config::Config;
use crate::mac_stats_debug;

fn secret_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)<secret>([^<]+)</secret>").expect("secret tag regex")
    })
}

fn host_matches_pattern(host: &str, pattern: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return false;
    }
    let pat = pattern.to_ascii_lowercase();
    if pat == "*" {
        return !host.is_empty();
    }
    if let Some(suffix) = pat.strip_prefix("*.") {
        if suffix.is_empty() {
            return false;
        }
        return host == suffix || host.ends_with(&format!(".{suffix}"));
    }
    host == pat
}

fn parse_credentials_toml_ordered(content: &str) -> Vec<(String, HashMap<String, String>)> {
    let root: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for (pattern, val) in root {
        if pattern.starts_with('#') {
            continue;
        }
        let toml::Value::Table(inner) = val else {
            continue;
        };
        let mut creds = HashMap::new();
        for (k, v) in inner {
            if let toml::Value::String(s) = v {
                creds.insert(k, s);
            }
        }
        if !creds.is_empty() {
            out.push((pattern, creds));
        }
    }
    out
}

/// Load domain → credentials in file order (TOML map order).
fn load_domain_entries() -> Vec<(String, HashMap<String, String>)> {
    ensure_default_credentials_file();
    let path = Config::browser_credentials_toml_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    parse_credentials_toml_ordered(&content)
}

fn merged_credentials_for_url(current_url: &str) -> HashMap<String, String> {
    let host = match Url::parse(current_url) {
        Ok(u) => u
            .host_str()
            .map(|h| h.to_ascii_lowercase())
            .unwrap_or_default(),
        Err(_) => String::new(),
    };
    if host.is_empty() {
        return HashMap::new();
    }
    let mut merged = HashMap::new();
    for (pattern, creds) in load_domain_entries() {
        if host_matches_pattern(&host, &pattern) {
            merged.extend(creds);
        }
    }
    merged
}

/// Placeholder names available for the current page URL (no secret values).
pub fn get_available_placeholders(current_url: &str) -> Vec<String> {
    let mut names: Vec<String> = merged_credentials_for_url(current_url).into_keys().collect();
    names.sort();
    names.dedup();
    names
}

/// Append a short hint for the LLM listing `<secret>…</secret>` tags (only names).
pub fn append_available_credentials_hint(current_url: &str, out: &mut String) {
    let placeholders = get_available_placeholders(current_url);
    if placeholders.is_empty() {
        return;
    }
    out.push_str("Available credentials for this page (use in BROWSER_INPUT as shown; values are filled in by the app, not typed literally): ");
    for (i, name) in placeholders.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str("<secret>");
        out.push_str(name);
        out.push_str("</secret>");
    }
    out.push('\n');
}

fn totp_code_from_base32(secret_b32: &str) -> Result<String, String> {
    let cleaned: String = secret_b32
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    let bytes = Secret::Encoded(cleaned)
        .to_bytes()
        .map_err(|e| format!("TOTP secret (base32): {}", e))?;
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, bytes)
        .map_err(|e| format!("TOTP init: {}", e))?;
    totp
        .generate_current()
        .map_err(|e| format!("TOTP generate: {}", e))
}

/// Replace every `<secret>name</secret>` with the configured value for the current page host.
/// Keys ending in `_2fa_code` are treated as Base32 TOTP secrets; a fresh 6-digit code is generated per substitution.
pub fn substitute_secret_tags_in_input(current_url: &str, input: &str) -> Result<String, String> {
    if !secret_tag_re().is_match(input) {
        return Ok(input.to_string());
    }
    let creds = merged_credentials_for_url(current_url);
    if creds.is_empty() {
        return Err(
            "BROWSER_INPUT contains <secret>…</secret> but no credentials match this page URL. Add a matching host pattern in ~/.mac-stats/browser-credentials.toml."
                .to_string(),
        );
    }

    let mut substitutions: u32 = 0;
    let mut out = String::with_capacity(input.len());
    let mut last = 0usize;
    for cap in secret_tag_re().captures_iter(input) {
        let m = cap.get(0).expect("full match");
        out.push_str(&input[last..m.start()]);
        last = m.end();

        let name = cap.get(1).map(|g| g.as_str().trim()).unwrap_or("");
        if name.is_empty() {
            return Err("BROWSER_INPUT: empty <secret></secret> placeholder name.".to_string());
        }
        let value = creds.get(name).ok_or_else(|| {
            format!(
                "BROWSER_INPUT: unknown credential placeholder {:?} for this page (not listed for the current URL in browser-credentials.toml).",
                name
            )
        })?;

        let resolved = if name.ends_with("_2fa_code") {
            totp_code_from_base32(value)?
        } else {
            value.clone()
        };
        substitutions = substitutions.saturating_add(1);
        out.push_str(&resolved);
    }
    out.push_str(&input[last..]);

    if substitutions > 0 {
        mac_stats_debug!(
            "browser/credentials",
            "BROWSER_INPUT: substituted {} <secret> placeholder(s) (values not logged)",
            substitutions
        );
    }
    Ok(out)
}

/// Create `~/.mac-stats/browser-credentials.toml` with a header comment and mode `0600` when missing.
pub fn ensure_default_credentials_file() {
    let path = Config::browser_credentials_toml_path();
    if path.exists() {
        return;
    }
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    const HEADER: &str = r#"# Browser credential placeholders for BROWSER_INPUT (chmod 600 recommended).
# Each [host-pattern] section maps placeholder names to secret values. Use placeholders in tool input as:
#   BROWSER_INPUT: 3 <secret>my_password</secret>
# Host patterns: exact host (example.com) or wildcard (*.example.com) matches subdomains and the apex.
# Keys ending with _2fa_code must hold a Base32 TOTP shared secret; a fresh 6-digit code is generated at input time.
# Example (replace with your own patterns and remove this comment block as needed):
#
# ["*.login.example.org"]
# app_password = "your-secret-here"
# vendor_2fa_code = "BASE32SECRET"

"#;
    if std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(HEADER.as_bytes())?;
            Ok(())
        })
        .is_err()
    {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_glob_matches() {
        assert!(host_matches_pattern("api.github.com", "*.github.com"));
        assert!(host_matches_pattern("github.com", "*.github.com"));
        assert!(!host_matches_pattern("evilgithub.com", "*.github.com"));
        assert!(host_matches_pattern("example.com", "example.com"));
        assert!(!host_matches_pattern("www.example.com", "example.com"));
    }
}
