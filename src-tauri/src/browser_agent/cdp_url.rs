//! CDP endpoint URL hygiene: HTTP(S) bases for `/json/*` discovery, WebSocket host rewrite
//! for mis-advertised bind addresses (e.g. `0.0.0.0`), and redaction for logs/errors.

use regex::Regex;
use std::sync::OnceLock;
use url::Url;

fn strip_devtools_browser_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if let Some(i) = trimmed.find("/devtools/browser") {
        let prefix = trimmed[..i].trim_end_matches('/');
        if prefix.is_empty() {
            "/".to_string()
        } else {
            prefix.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

fn strip_trailing_cdp_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if let Some(prefix) = trimmed.strip_suffix("/cdp") {
        let cut = prefix.trim_end_matches('/');
        if cut.is_empty() {
            "/".to_string()
        } else {
            cut.to_string()
        }
    } else if trimmed == "cdp" || trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

/// From a CDP **HTTP(S) root**, **WebSocket debugger URL**, or similar, derive the HTTP(S) URL
/// suitable for appending `/json/version`, `/json/list`, etc. (OpenClaw-style normalization).
pub(crate) fn cdp_http_base_from_endpoint(input: &str) -> Result<Url, String> {
    let mut u = Url::parse(input).map_err(|e| format!("parse CDP endpoint URL: {}", e))?;
    let new_scheme = match u.scheme() {
        "ws" => "http",
        "wss" => "https",
        "http" => "http",
        "https" => "https",
        other => {
            return Err(format!(
                "unsupported CDP URL scheme {:?} (expected ws, wss, http, https)",
                other
            ));
        }
    };
    u.set_scheme(new_scheme)
        .map_err(|_| "failed to normalize CDP URL scheme".to_string())?;
    let path = u.path();
    let path = strip_devtools_browser_path(path);
    let path = strip_trailing_cdp_path(&path);
    u.set_path(&path);
    u.set_query(None);
    u.set_fragment(None);
    Ok(u)
}

fn ws_host_should_rewrite(host: Option<&str>) -> bool {
    matches!(host, Some("0.0.0.0") | Some("::") | Some("[::]"))
}

/// After reading `webSocketDebuggerUrl` from `/json/version`, rewrite advertised bind-all hosts
/// to the host the client used for HTTP discovery so the WebSocket handshake works from the operator machine.
pub(crate) fn rewrite_ws_debugger_host_for_discovery(ws_url: &str, discovery_host: &str) -> String {
    let Ok(mut u) = Url::parse(ws_url) else {
        return ws_url.to_string();
    };
    if !ws_host_should_rewrite(u.host_str()) {
        return ws_url.to_string();
    }
    if u.set_host(Some(discovery_host)).is_err() {
        return ws_url.to_string();
    }
    u.to_string()
}

fn sensitive_query_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "token"
            | "key"
            | "auth"
            | "apikey"
            | "api_key"
            | "secret"
            | "password"
            | "access_token"
            | "refresh_token"
    )
}

/// Remove userinfo and mask common auth-related query parameters; best-effort if parsing fails.
pub(crate) fn redact_cdp_url(s: &str) -> String {
    if let Ok(mut u) = Url::parse(s) {
        let _ = u.set_username("");
        let _ = u.set_password(None);
        let pairs: Vec<(String, String)> = u.query_pairs().into_owned().collect();
        if pairs.is_empty() {
            return u.to_string();
        }
        let mut ser = url::form_urlencoded::Serializer::new(String::new());
        for (k, v) in pairs {
            if sensitive_query_key(&k) {
                // Avoid angle brackets — form_urlencoded would percent-encode them.
                ser.append_pair(&k, "redacted");
            } else {
                ser.append_pair(&k, &v);
            }
        }
        u.set_query(Some(&ser.finish()));
        return u.to_string();
    }
    best_effort_redact_raw(s)
}

fn userinfo_redact_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?P<scheme>[a-zA-Z][a-zA-Z0-9+.-]*://)(?P<userinfo>[^/?#]+)@")
            .expect("userinfo redact regex")
    })
}

fn best_effort_redact_raw(s: &str) -> String {
    let mut out = userinfo_redact_regex()
        .replace_all(s, "${scheme}")
        .into_owned();
    // Mask token=value in query-ish suffix without full URL parse
    let token_re =
        Regex::new(r"(?i)([?&])(token|key|auth|apikey|api_key|secret|password)=([^&\s#]+)")
            .expect("token redact regex");
    out = token_re
        .replace_all(&out, |caps: &regex::Captures| {
            format!("{}{}=redacted", &caps[1], &caps[2])
        })
        .into_owned();
    out
}

/// `http://{host}:{port}/json/version` for CDP discovery (host is the attach/discovery host).
pub(crate) fn json_version_probe_url(host: &str, port: u16) -> String {
    format!("http://{}:{}/json/version", host, port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_base_from_ws_devtools_browser_path() {
        let base = cdp_http_base_from_endpoint("ws://127.0.0.1:9222/devtools/browser/abc-def-000")
            .unwrap();
        assert_eq!(base.as_str(), "http://127.0.0.1:9222/");
    }

    #[test]
    fn http_base_from_wss_trailing_cdp() {
        let base = cdp_http_base_from_endpoint("wss://example.com/cdp").unwrap();
        assert_eq!(base.as_str(), "https://example.com/");
    }

    #[test]
    fn rewrite_zero_host_uses_discovery_host() {
        let out = rewrite_ws_debugger_host_for_discovery(
            "ws://0.0.0.0:9222/devtools/browser/x",
            "127.0.0.1",
        );
        assert!(out.starts_with("ws://127.0.0.1:9222/"));
        assert!(!out.contains("0.0.0.0"));
    }

    #[test]
    fn rewrite_ipv6_unspecified_host() {
        let out = rewrite_ws_debugger_host_for_discovery(
            "ws://[::]:9222/devtools/browser/x",
            "127.0.0.1",
        );
        assert!(out.contains("127.0.0.1"));
        assert!(!out.contains("[::]"));
    }

    #[test]
    fn redact_removes_userinfo_and_token_query() {
        let s = "ws://alice:hunter2@host:9222/p?token=sekrit&foo=1";
        let r = redact_cdp_url(s);
        assert!(!r.contains("alice"));
        assert!(!r.contains("hunter2"));
        assert!(!r.contains("sekrit"));
        assert!(r.contains("token=redacted") || r.contains("token%3Dredacted"));
        assert!(r.contains("foo=1"));
    }

    #[test]
    fn redact_best_effort_unparseable_strips_userinfo() {
        let s = "ws://u:p@not a valid url";
        let r = redact_cdp_url(s);
        assert!(!r.contains("u:p@"));
    }
}
