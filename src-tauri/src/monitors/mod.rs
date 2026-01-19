//! External monitoring module
//! 
//! Monitors external resources:
//! - Websites (HTTP/HTTPS uptime, response times, SSL errors)
//! - Social media (Mastodon/X mentions)
//! - APIs (custom endpoints)

pub mod website;
pub mod social;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;

/// Monitor type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MonitorType {
    Website,
    Mastodon,
    Twitter,
    Custom,
}

/// Base monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub monitor_type: MonitorType,
    pub enabled: bool,
    pub last_check: Option<DateTime<Utc>>,
    pub last_status: Option<MonitorStatus>,
}

/// Monitor status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub is_up: bool,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
    pub checked_at: DateTime<Utc>,
}

/// Monitor result from a check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Part of API, may be used in future
pub struct MonitorResult {
    pub monitor_id: String,
    pub status: MonitorStatus,
}

/// Trait for monitor implementations
pub trait MonitorCheck {
    fn check(&self) -> Result<MonitorStatus>;
    #[allow(dead_code)] // Part of trait API, may be used in future
    fn get_id(&self) -> &str;
}
