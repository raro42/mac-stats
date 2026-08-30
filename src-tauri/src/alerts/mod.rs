//! Alert system module
//!
//! Rule-based alerting with channel-agnostic core.
//! Supports multiple notification channels: Telegram, Slack, Signal, Mastodon.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod channels;
pub mod rules;

use channels::AlertChannel;
use rules::AlertRule;

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
            let elapsed = Utc::now()
                .signed_duration_since(last_triggered)
                .num_seconds();
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
    /// Tracks when each alert's condition first became true (for sustained-duration rules).
    /// Key: alert id. Cleared when the condition becomes false.
    condition_since: HashMap<String, DateTime<Utc>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: HashMap::new(),
            channels: HashMap::new(),
            condition_since: HashMap::new(),
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

    /// Remove an alert channel by id (used by Tauri command remove_alert_channel).
    pub fn remove_channel(&mut self, channel_id: &str) {
        self.channels.remove(channel_id);
    }

    /// List registered alert channel IDs (for Settings UI).
    pub fn list_channel_ids(&self) -> Vec<String> {
        self.channels.keys().cloned().collect()
    }

    /// Count registered channels whose `get_name()` matches (Telegram / Slack / Signal / Mastodon).
    pub fn count_channels_named(&self, name: &str) -> usize {
        self.channels
            .values()
            .filter(|c| c.get_name() == name)
            .count()
    }

    /// Evaluate all alerts against context.
    /// For rules with a `duration_secs` requirement (TemperatureHigh, CpuHigh), the condition
    /// must be true for at least that many consecutive seconds before the alert fires.
    pub fn evaluate(&mut self, context: AlertContext) -> Result<Vec<String>> {
        let mut triggered_alerts = Vec::new();
        let now = Utc::now();

        let alert_ids: Vec<String> = self.alerts.keys().cloned().collect();
        for alert_id in alert_ids {
            let alert = match self.alerts.get(&alert_id) {
                Some(a) => a,
                None => continue,
            };
            if !alert.should_trigger() {
                continue;
            }

            let condition_met = alert.rule.evaluate(&context)?;
            let required_secs = alert.rule.required_duration_secs();

            if condition_met {
                if required_secs == 0 {
                    // Immediate rule — fire right away
                } else {
                    let first_true = *self.condition_since.entry(alert_id.clone()).or_insert(now);
                    let sustained = now.signed_duration_since(first_true).num_seconds();
                    if sustained < required_secs as i64 {
                        continue; // not sustained long enough yet
                    }
                    // Sustained long enough; clear tracker so cooldown governs re-fire
                    self.condition_since.remove(&alert_id);
                }
            } else {
                self.condition_since.remove(&alert_id);
                continue;
            }

            // Trigger alert
            let alert = self.alerts.get_mut(&alert_id).unwrap();
            let message = format!("Alert triggered: {}", alert.name);

            for channel_id in &alert.channels {
                if let Some(channel) = self.channels.get_mut(channel_id.as_str()) {
                    if let Err(e) = channel.send(&message, &context) {
                        tracing::error!("Failed to send alert to channel {}: {}", channel_id, e);
                    }
                }
            }

            alert.last_triggered = Some(now);
            triggered_alerts.push(alert_id);
        }

        Ok(triggered_alerts)
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
