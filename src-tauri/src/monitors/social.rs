//! Social media monitoring implementation (Mastodon/X)

use super::{Monitor, MonitorType, MonitorStatus, MonitorCheck};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use chrono::Utc;
use crate::security;

/// Mastodon monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MastodonMonitor {
    pub id: String,
    pub name: String,
    pub instance_url: String, // e.g., "https://mastodon.social"
    pub account_username: String,
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
    pub last_status: Option<MonitorStatus>,
    pub last_mention_count: u64,
}

impl MastodonMonitor {
    pub fn new(id: String, name: String, instance_url: String, account_username: String) -> Self {
        Self {
            id,
            name,
            instance_url,
            account_username,
            enabled: true,
            check_interval_secs: 300, // 5 minutes
            last_check: None,
            last_status: None,
            last_mention_count: 0,
        }
    }

    /// Get API token from Keychain
    fn get_api_token(&self) -> Result<String> {
        let account = format!("mastodon_{}", self.id);
        security::get_credential(&account)?
            .context("Mastodon API token not found in Keychain")
    }

    /// Check for new mentions
    pub fn check_mentions(&self) -> Result<u64> {
        let token = self.get_api_token()?;
        
        // Mastodon API: GET /api/v1/notifications
        let url = format!("{}/api/v1/notifications", self.instance_url);
        
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()?;

        if response.status().is_success() {
            let notifications: Vec<serde_json::Value> = response.json()?;
            // Count mentions (type: "mention")
            let mention_count = notifications
                .iter()
                .filter(|n| n.get("type").and_then(|t| t.as_str()) == Some("mention"))
                .count() as u64;
            
            Ok(mention_count)
        } else {
            Err(anyhow::anyhow!("Mastodon API error: {}", response.status()))
        }
    }
}

impl MonitorCheck for MastodonMonitor {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn check(&self) -> Result<MonitorStatus> {
        use tracing::info;
        
        info!("Monitor: Starting Mastodon check - ID: {}, Instance: {}, Account: {}", 
              self.id, self.instance_url, self.account_username);
        
        match self.check_mentions() {
            Ok(mention_count) => {
                info!("Monitor: Mastodon check successful - ID: {}, Instance: {}, Account: {}, Mentions: {}", 
                      self.id, self.instance_url, self.account_username, mention_count);
                
                Ok(MonitorStatus {
                    is_up: true,
                    response_time_ms: None,
                    error: None,
                    checked_at: Utc::now(),
                })
            }
            Err(e) => {
                info!("Monitor: Mastodon check failed - ID: {}, Instance: {}, Account: {}, Error: {}", 
                      self.id, self.instance_url, self.account_username, e);
                Ok(MonitorStatus {
                    is_up: false,
                    response_time_ms: None,
                    error: Some(format!("Failed to check mentions: {}", e)),
                    checked_at: Utc::now(),
                })
            }
        }
    }
}

impl From<MastodonMonitor> for Monitor {
    fn from(mm: MastodonMonitor) -> Self {
        Monitor {
            id: mm.id,
            name: mm.name,
            monitor_type: MonitorType::Mastodon,
            enabled: mm.enabled,
            last_check: mm.last_check,
            last_status: mm.last_status,
        }
    }
}

/// Twitter/X monitor (placeholder - API is restricted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterMonitor {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
    pub last_status: Option<MonitorStatus>,
}

impl TwitterMonitor {
    #[allow(dead_code)] // Part of API, may be used in future
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            enabled: true,
            check_interval_secs: 300,
            last_check: None,
            last_status: None,
        }
    }
}

impl MonitorCheck for TwitterMonitor {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn check(&self) -> Result<MonitorStatus> {
        // Twitter/X API is heavily restricted and requires paid access
        // This is a placeholder implementation
        Ok(MonitorStatus {
            is_up: false,
            response_time_ms: None,
            error: Some("Twitter/X API requires paid access and is not implemented".to_string()),
            checked_at: Utc::now(),
        })
    }
}

impl From<TwitterMonitor> for Monitor {
    fn from(tm: TwitterMonitor) -> Self {
        Monitor {
            id: tm.id,
            name: tm.name,
            monitor_type: MonitorType::Twitter,
            enabled: tm.enabled,
            last_check: tm.last_check,
            last_status: tm.last_status,
        }
    }
}
