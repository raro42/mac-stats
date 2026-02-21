//! Website monitoring implementation

use super::{Monitor, MonitorType, MonitorStatus, MonitorCheck};
use serde::{Deserialize, Serialize};
use url::Url;
use anyhow::{Result, Context};
use std::time::Duration;
use chrono::Utc;

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
        Url::parse(&self.url)
            .context("Invalid URL format")?;
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
        let response = client
            .get(&self.url)
            .send();

        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        let checked_at = Utc::now();

        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                let is_up = self.expected_status_code
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
                })
            }
            Err(e) => {
                trace!("Monitor: Website check error - ID: {}, URL: {}, Error: {}, Response time: {}ms", 
                      self.id, self.url, e, elapsed_ms);
                Ok(MonitorStatus {
                    is_up: false,
                    response_time_ms: Some(elapsed_ms),
                    error: Some(format!("Request failed: {}", e)),
                    checked_at,
                })
            }
        }
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
