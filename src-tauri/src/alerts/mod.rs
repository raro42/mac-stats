//! Alert system module
//! 
//! Rule-based alerting with channel-agnostic core.
//! Supports multiple notification channels: Telegram, Slack, Signal, Mastodon.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::collections::HashMap;

pub mod rules;
pub mod channels;

use rules::AlertRule;
use channels::AlertChannel;

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub rule: AlertRule,
    pub channels: Vec<String>, // Channel IDs
    pub enabled: bool,
    pub last_triggered: Option<DateTime<Utc>>,
    pub cooldown_secs: u64, // Prevent spam
}

impl Alert {
    #[allow(dead_code)] // Part of API, may be used in future
    pub fn new(id: String, name: String, rule: AlertRule) -> Self {
        Self {
            id,
            name,
            rule,
            channels: Vec::new(),
            enabled: true,
            last_triggered: None,
            cooldown_secs: 300, // 5 minutes default cooldown
        }
    }

    /// Check if alert should fire (respects cooldown)
    pub fn should_trigger(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Check cooldown
        if let Some(last_triggered) = self.last_triggered {
            let elapsed = Utc::now().signed_duration_since(last_triggered).num_seconds();
            if elapsed < self.cooldown_secs as i64 {
                return false;
            }
        }

        true
    }
}

/// Alert context (data passed to rule evaluation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertContext {
    pub monitor_id: Option<String>,
    pub monitor_status: Option<crate::monitors::MonitorStatus>,
    pub system_metrics: Option<crate::metrics::SystemMetrics>,
    pub cpu_details: Option<crate::metrics::CpuDetails>,
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Alert manager (handles rule evaluation and channel delivery)
pub struct AlertManager {
    alerts: HashMap<String, Alert>,
    channels: HashMap<String, Box<dyn AlertChannel>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.insert(alert.id.clone(), alert);
    }

    pub fn remove_alert(&mut self, alert_id: &str) {
        self.alerts.remove(alert_id);
    }

    #[allow(dead_code)] // Part of API, may be used in future
    pub fn register_channel(&mut self, channel_id: String, channel: Box<dyn AlertChannel>) {
        self.channels.insert(channel_id, channel);
    }

    /// Evaluate all alerts against context
    pub fn evaluate(&mut self, context: AlertContext) -> Result<Vec<String>> {
        let mut triggered_alerts = Vec::new();

        for (alert_id, alert) in self.alerts.iter_mut() {
            if !alert.should_trigger() {
                continue;
            }

            // Evaluate rule
            if alert.rule.evaluate(&context)? {
                // Trigger alert
                let message = format!("Alert triggered: {}", alert.name);
                
                // Send to all configured channels
                for channel_id in &alert.channels {
                    if let Some(channel) = self.channels.get_mut(channel_id.as_str()) {
                        if let Err(e) = channel.send(&message, &context) {
                            tracing::error!("Failed to send alert to channel {}: {}", channel_id, e);
                        }
                    }
                }

                alert.last_triggered = Some(Utc::now());
                triggered_alerts.push(alert_id.clone());
            }
        }

        Ok(triggered_alerts)
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
