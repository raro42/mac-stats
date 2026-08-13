//! Website monitoring implementation

use super::{Monitor, MonitorCheck, MonitorStatus, MonitorType};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Website monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteMonitor {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub timeout_secs: u64,
    pub check_interval_secs: u64,
    pub expected_status_code: Option<u16>,
    pub verify_ssl: bool,
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
    pub last_status: Option<MonitorStatus>,
}

impl WebsiteMonitor {
    pub fn new(id: String, name: String, url: String) -> Self {
        Self {
            id,
            name,
            url,
            enabled: true,
            timeout_secs: 10,
            check_interval_secs: 60,
            expected_status_code: Some(200),
            verify_ssl: true,
            last_check: None,
            last_status: None,
        }
    }

    /// Validate URL format
    pub fn validate_url(&self) -> Result<()> {
        Url::parse(&self.url).context("Invalid URL format")?;
        Ok(())
    }
}

impl MonitorCheck for WebsiteMonitor {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn check(&self) -> Result<MonitorStatus> {
        use tracing::trace;

        trace!("Monitor: Starting website check - ID: {}, URL: {}, Timeout: {}s (SSL verification disabled - accepting invalid certificates)", 
              self.id, self.url, self.timeout_secs);

        let start_time = std::time::Instant::now();

        // Create HTTP client with timeout
        // Always accept invalid certificates for monitoring purposes (allows monitoring self-signed or expired certs)
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .danger_accept_invalid_certs(true) // Always accept invalid certificates
            .build()
            .context("Failed to create HTTP client")?;

        // Perform HTTP request
        let response = client.get(&self.url).send();

        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        let checked_at = Utc::now();

        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                let is_up = self
                    .expected_status_code
                    .map(|expected| status_code == expected)
                    .unwrap_or(resp.status().is_success());

                if is_up {
                    trace!("Monitor: Website check successful - ID: {}, URL: {}, Status code: {}, Response time: {}ms", 
                          self.id, self.url, status_code, elapsed_ms);
                } else {
                    trace!("Monitor: Website check failed - ID: {}, URL: {}, Status code: {} (expected: {:?}), Response time: {}ms", 
                          self.id, self.url, status_code, self.expected_status_code, elapsed_ms);
                }

                Ok(MonitorStatus {
                    is_up,
                    response_time_ms: Some(elapsed_ms),
                    error: if is_up {
                        None
                    } else {
                        Some(format!("Unexpected status code: {}", status_code))
                    },
                    checked_at,
                    extra: Default::default(),
                })
            }
            Err(e) => {
                let short = classify_website_request_error(&e);
                trace!(
                    "Monitor: Website check error - ID: {}, URL: {}, Error: {} (raw: {}), Response time: {}ms",
                    self.id,
                    self.url,
                    short,
                    e,
                    elapsed_ms
                );
                Ok(MonitorStatus {
                    is_up: false,
                    response_time_ms: Some(elapsed_ms),
                    error: Some(short),
                    checked_at,
                    extra: Default::default(),
                })
            }
        }
    }
}

/// Short operator-facing reason for a failed website probe (UI + debug.log).
pub(crate) fn classify_website_request_error(err: &reqwest::Error) -> String {
    classify_website_error_text(&err.to_string(), err.is_timeout(), err.is_connect())
}

fn classify_website_error_text(raw: &str, is_timeout: bool, is_connect: bool) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("dns error")
        || lower.contains("failed to lookup")
        || lower.contains("nodename nor servname")
        || lower.contains("name or service not known")
        || lower.contains("no such host")
    {
        return "DNS lookup failed".to_string();
    }
    if is_timeout || lower.contains("timed out") || lower.contains("timeout") {
        return "Connection timed out".to_string();
    }
    if lower.contains("certificate")
        || lower.contains("tls")
        || lower.contains("ssl")
        || lower.contains("handshake")
    {
        return "TLS/SSL error".to_string();
    }
    if lower.contains("connection refused") || lower.contains("actively refused") {
        return "Connection refused".to_string();
    }
    if lower.contains("network is unreachable") || lower.contains("no route to host") {
        return "Network unreachable".to_string();
    }
    if is_connect || lower.contains("error trying to connect") || lower.contains("connect error") {
        return "Connection failed".to_string();
    }
    "Request failed".to_string()
}

#[cfg(test)]
mod tests {
    use super::classify_website_error_text;

    #[test]
    fn classifies_dns_lookup_failure() {
        let raw = "error sending request for url (https://www.econsultants.es/): \
                   error trying to connect: dns error: failed to lookup address information: \
                   nodename nor servname provided, or not known";
        assert_eq!(
            classify_website_error_text(raw, false, true),
            "DNS lookup failed"
        );
    }

    #[test]
    fn classifies_timeout() {
        assert_eq!(
            classify_website_error_text("operation timed out", true, false),
            "Connection timed out"
        );
    }

    #[test]
    fn classifies_tls_and_refused() {
        assert_eq!(
            classify_website_error_text("invalid certificate: UnknownIssuer", false, false),
            "TLS/SSL error"
        );
        assert_eq!(
            classify_website_error_text("tcp connect error: Connection refused", false, true),
            "Connection refused"
        );
    }

    #[test]
    fn classifies_generic_connect() {
        assert_eq!(
            classify_website_error_text("error trying to connect: …", false, true),
            "Connection failed"
        );
        assert_eq!(
            classify_website_error_text("something odd happened", false, false),
            "Request failed"
        );
    }
}

impl From<WebsiteMonitor> for Monitor {
    fn from(wm: WebsiteMonitor) -> Self {
        Monitor {
            id: wm.id,
            name: wm.name,
            monitor_type: MonitorType::Website,
            enabled: wm.enabled,
            last_check: wm.last_check,
            last_status: wm.last_status,
        }
    }
}
