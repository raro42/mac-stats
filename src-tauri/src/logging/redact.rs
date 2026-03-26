//! Pattern-based secret redaction for log output.
//!
//! Loaded from [`init_from_env`](init_from_env) during tracing setup. When enabled (default),
//! all lines written to the log file and stderr pass through [`redact_secrets`]. Disable with
//! `LOG_REDACTION=0` in the environment or `~/.mac-stats/.config.env` for raw debugging output.

use regex::Captures;
use regex::Regex;
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use tracing_subscriber::fmt::writer::MakeWriter;

static REDACTION_ENABLED: AtomicBool = AtomicBool::new(true);
static COMPILED: std::sync::OnceLock<RedactRules> = std::sync::OnceLock::new();

struct RedactRules {
    pem: Regex,
    bearer: Regex,
    auth_header: Regex,
    sk_openai: Regex,
    ghp: Regex,
    github_pat: Regex,
    slack_bot: Regex,
    slack_app: Regex,
    long_base64: Regex,
    extras: Vec<Regex>,
}

impl RedactRules {
    fn apply(&self, text: &str) -> String {
        let mut out = self
            .pem
            .replace_all(text, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
            .into_owned();
        out = self
            .bearer
            .replace_all(&out, |c: &Captures| {
                format!(
                    "Bearer {}",
                    mask_secret(c.get(1).unwrap().as_str())
                )
            })
            .into_owned();
        out = self
            .auth_header
            .replace_all(&out, |c: &Captures| {
                format!(
                    "Authorization: Bearer {}",
                    mask_secret(c.get(1).unwrap().as_str())
                )
            })
            .into_owned();
        out = self
            .sk_openai
            .replace_all(&out, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
            .into_owned();
        out = self
            .ghp
            .replace_all(&out, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
            .into_owned();
        out = self
            .github_pat
            .replace_all(&out, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
            .into_owned();
        out = self
            .slack_bot
            .replace_all(&out, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
            .into_owned();
        out = self
            .slack_app
            .replace_all(&out, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
            .into_owned();
        out = self
            .long_base64
            .replace_all(&out, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
            .into_owned();
        for re in &self.extras {
            out = re
                .replace_all(&out, |c: &Captures| mask_secret(c.get(0).unwrap().as_str()))
                .into_owned();
        }
        out
    }
}

/// Mask a matched secret: first 4 + `…` + last 4 Unicode scalars; under 12 chars → `<redacted>`.
fn mask_secret(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    let n = chars.len();
    if n < 12 {
        return "<redacted>".to_string();
    }
    let first: String = chars.iter().take(4).collect();
    let last: String = chars.iter().skip(n.saturating_sub(4)).collect();
    format!("{first}…{last}")
}

fn compile_rules() -> RedactRules {
    let extras = collect_extra_regexes();
    RedactRules {
        pem: Regex::new(
            r"(?s)-----BEGIN [A-Z0-9 -]+-----[\r\n]+.*?[\r\n]+-----END [A-Z0-9 -]+-----",
        )
        .expect("PEM redact regex"),
        bearer: Regex::new(r"(?i)Bearer\s+(\S+)").expect("Bearer redact regex"),
        auth_header: Regex::new(r"(?i)Authorization:\s*Bearer\s+(\S+)").expect("Auth header regex"),
        sk_openai: Regex::new(r"\bsk-[A-Za-z0-9_-]{8,}\b").expect("sk- redact regex"),
        ghp: Regex::new(r"\bghp_[A-Za-z0-9]{20,}\b").expect("ghp_ redact regex"),
        github_pat: Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b").expect("github_pat regex"),
        slack_bot: Regex::new(r"\bxox[bap]-[0-9]+-[A-Za-z0-9-]{10,}\b").expect("xoxb regex"),
        slack_app: Regex::new(r"\bxapp-[0-9]+-[A-Za-z0-9-]{10,}\b").expect("xapp regex"),
        // Rust `regex` has no look-around; long runs of base64 alphabet may rarely false-positive in prose.
        long_base64: Regex::new(r"[A-Za-z0-9+/]{41,}={0,2}").expect("base64 redact regex"),
        extras,
    }
}

fn collect_extra_regexes() -> Vec<Regex> {
    let mut raw = std::env::var("LOG_REDACT_EXTRA_REGEX").unwrap_or_default();
    if raw.trim().is_empty() {
        raw = read_config_key_from_env_files("LOG_REDACT_EXTRA_REGEX")
            .or_else(|| read_config_key_from_env_files("LOG-REDACT-EXTRA-REGEX"))
            .unwrap_or_default();
    }
    let mut out = Vec::new();
    for part in raw.split(';') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        match Regex::new(p) {
            Ok(re) => out.push(re),
            Err(e) => {
                // Subscriber may not exist yet during init; still surface misconfiguration.
                eprintln!("mac-stats: LOG_REDACT_EXTRA_REGEX: skipped invalid pattern: {e}");
            }
        }
    }
    out
}

fn read_config_key_from_env_files(key: &str) -> Option<String> {
    let key_eq = format!("{key}=");
    let key_dash = key.replace('_', "-");
    let key_dash_eq = format!("{key_dash}=");

    let try_file = |path: &Path| -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        for line in content.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix(&key_eq).or_else(|| t.strip_prefix(&key_dash_eq)) {
                return Some(rest.trim().to_string());
            }
        }
        None
    };

    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".config.env");
        if p.is_file() {
            if let Some(v) = try_file(&p) {
                return Some(v);
            }
        }
        let p_src = cwd.join("src-tauri").join(".config.env");
        if p_src.is_file() {
            if let Some(v) = try_file(&p_src) {
                return Some(v);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            return try_file(&p);
        }
    }
    None
}

fn log_redaction_enabled_from_env_var() -> Option<bool> {
    let Ok(v) = std::env::var("LOG_REDACTION") else {
        return None;
    };
    let v = v.trim().to_lowercase();
    if matches!(v.as_str(), "0" | "false" | "no" | "off") {
        Some(false)
    } else {
        Some(true)
    }
}

fn log_redaction_disabled_in_file(content: &str) -> Option<bool> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t
            .strip_prefix("LOG_REDACTION=")
            .or_else(|| t.strip_prefix("LOG-REDACTION="))
        {
            let v = rest.trim().to_lowercase();
            return Some(matches!(v.as_str(), "0" | "false" | "no" | "off"));
        }
    }
    None
}

fn log_redaction_enabled_from_files() -> Option<bool> {
    let try_file = |path: &Path| -> Option<bool> {
        let content = std::fs::read_to_string(path).ok()?;
        log_redaction_disabled_in_file(&content).map(|disabled| !disabled)
    };

    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".config.env");
        if p.is_file() {
            if let Some(e) = try_file(&p) {
                return Some(e);
            }
        }
        let p_src = cwd.join("src-tauri").join(".config.env");
        if p_src.is_file() {
            if let Some(e) = try_file(&p_src) {
                return Some(e);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            return try_file(&p);
        }
    }
    None
}

/// Resolve whether log redaction is enabled (default: true).
pub fn log_redaction_enabled() -> bool {
    if let Some(on) = log_redaction_enabled_from_env_var() {
        return on;
    }
    log_redaction_enabled_from_files().unwrap_or(true)
}

/// Call once before building tracing layers (reads env and `.config.env` for flags and extra regexes).
pub fn init_from_env() {
    let enabled = log_redaction_enabled();
    REDACTION_ENABLED.store(enabled, Ordering::Relaxed);
    let _ = COMPILED.set(compile_rules());
}

/// Whether redaction is on after [`init_from_env`] (for startup log line after the subscriber exists).
pub fn redaction_active() -> bool {
    REDACTION_ENABLED.load(Ordering::Relaxed)
}

/// Redact known secret patterns in arbitrary text (for file + legacy log paths).
pub fn redact_secrets(text: &str) -> String {
    if !REDACTION_ENABLED.load(Ordering::Relaxed) {
        return text.to_string();
    }
    let Some(rules) = COMPILED.get() else {
        return text.to_string();
    };
    rules.apply(text)
}

/// Writes complete lines to `inner`, running [`redact_secrets`] on each line when `enabled`.
pub struct RedactingLineWriter<W> {
    inner: W,
    buf: Vec<u8>,
    enabled: bool,
}

impl<W: Write> RedactingLineWriter<W> {
    pub fn new(inner: W, enabled: bool) -> Self {
        Self {
            inner,
            buf: Vec::new(),
            enabled,
        }
    }

    fn flush_buf_line(&mut self, had_nl: bool) -> io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        let line = if had_nl && self.buf.last() == Some(&b'\n') {
            &self.buf[..self.buf.len() - 1]
        } else {
            &self.buf[..]
        };
        let text = String::from_utf8_lossy(line);
        let out = if self.enabled {
            redact_secrets(text.as_ref())
        } else {
            text.into_owned()
        };
        self.inner.write_all(out.as_bytes())?;
        if had_nl {
            self.inner.write_all(b"\n")?;
        }
        self.buf.clear();
        Ok(())
    }
}

impl<W: Write> Write for RedactingLineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = buf.len();
        for chunk in buf.split_inclusive(|&b| b == b'\n') {
            if chunk.is_empty() {
                continue;
            }
            if chunk.ends_with(b"\n") {
                self.buf.extend_from_slice(&chunk[..chunk.len() - 1]);
                self.flush_buf_line(true)?;
            } else {
                self.buf.extend_from_slice(chunk);
            }
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buf_line(false)?;
        self.inner.flush()
    }
}

/// Forwards to a [`MutexGuard`] over a [`std::fs::File`].
pub struct FileMutexWriter<'a> {
    guard: MutexGuard<'a, std::fs::File>,
}

impl Write for FileMutexWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.guard.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.guard.flush()
    }
}

/// [`MakeWriter`] that locks the shared log file and wraps it with [`RedactingLineWriter`].
pub struct RedactingFileMakeWriter {
    file: Mutex<std::fs::File>,
    enabled: bool,
}

impl RedactingFileMakeWriter {
    pub fn new(file: std::fs::File, enabled: bool) -> Self {
        Self {
            file: Mutex::new(file),
            enabled,
        }
    }
}

impl<'a> MakeWriter<'a> for RedactingFileMakeWriter {
    type Writer = RedactingLineWriter<FileMutexWriter<'a>>;

    fn make_writer(&'a self) -> Self::Writer {
        let guard = self
            .file
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        RedactingLineWriter::new(FileMutexWriter { guard }, self.enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_short_is_fully_redacted() {
        assert_eq!(mask_secret("short"), "<redacted>");
        let eleven = "12345678901";
        assert_eq!(eleven.chars().count(), 11);
        assert_eq!(mask_secret(eleven), "<redacted>");
    }

    #[test]
    fn mask_long_preserves_ends() {
        let s = "sk-abcdefghijklmnopqrstuvwxyz";
        let m = mask_secret(s);
        assert!(m.starts_with("sk-a"));
        assert!(m.ends_with("wxyz"));
        assert!(m.contains('…'));
    }

    #[test]
    fn bearer_redacted_in_rules() {
        let rules = compile_rules();
        let out = rules.apply("token is Bearer abcdefghijklmnopqr_stuvwxyz end");
        assert!(!out.contains("abcdefghijklmnopqr_stuvwxyz"));
        assert!(out.contains("Bear"));
        assert!(out.contains('…'));
    }
}
