//! Alert rule evaluation

use super::AlertContext;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Alert rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertRule {
    /// Site down for >= N minutes
    SiteDown { minutes: u64 },
    /// New mentions >= N in 1 hour
    NewMentions { count: u64, hours: u64 },
    /// Battery level < N%
    BatteryLow { threshold: f32 },
    /// Temperature > N°C sustained
    TemperatureHigh { threshold: f32, duration_secs: u64 },
    /// CPU usage > N% sustained
    CpuHigh { threshold: f32, duration_secs: u64 },
    /// Custom rule (plugin-based)
    Custom {
        plugin_id: String,
        config: serde_json::Value,
    },
}

impl AlertRule {
    /// Duration (in seconds) that the condition must be sustained before the alert fires.
    /// Returns 0 for rules that fire immediately on a single reading.
    pub fn required_duration_secs(&self) -> u64 {
        match self {
            AlertRule::TemperatureHigh { duration_secs, .. } => *duration_secs,
            AlertRule::CpuHigh { duration_secs, .. } => *duration_secs,
            _ => 0,
        }
    }

    /// Evaluate rule against context
    pub fn evaluate(&self, context: &AlertContext) -> Result<bool> {
        match self {
            AlertRule::SiteDown { minutes } => {
                // Check if monitor is down for >= minutes
                if let Some(ref status) = context.monitor_status {
                    if !status.is_up {
                        // Calculate downtime duration
                        let downtime = chrono::Utc::now()
                            .signed_duration_since(status.checked_at)
                            .num_minutes();
                        return Ok(downtime >= *minutes as i64);
                    }
                }
                Ok(false)
            }
            AlertRule::NewMentions { count, hours } => {
                if let Some(ref status) = context.monitor_status {
                    if let Some(ts_val) = status.extra.get("mention_timestamps") {
                        let cutoff = Utc::now() - chrono::Duration::hours(*hours as i64);
                        let recent = ts_val
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .filter_map(|s| s.parse::<DateTime<Utc>>().ok())
                                    .filter(|dt| *dt >= cutoff)
                                    .count() as u64
                            })
                            .unwrap_or(0);
                        return Ok(recent >= *count);
                    }
                }
                Ok(false)
            }
            AlertRule::BatteryLow { threshold } => {
                if let Some(ref cpu_details) = context.cpu_details {
                    if cpu_details.has_battery && cpu_details.battery_level >= 0.0 {
                        return Ok(cpu_details.battery_level < *threshold);
                    }
                }
                Ok(false)
            }
            AlertRule::TemperatureHigh {
                threshold,
                duration_secs: _,
            } => {
                if let Some(ref cpu_details) = context.cpu_details {
                    return Ok(cpu_details.temperature > *threshold);
                }
                Ok(false)
            }
            AlertRule::CpuHigh {
                threshold,
                duration_secs: _,
            } => {
                if let Some(ref system_metrics) = context.system_metrics {
                    return Ok(system_metrics.cpu > *threshold);
                }
                Ok(false)
            }
            // NOTE: TemperatureHigh/CpuHigh return true for the instantaneous
            // condition (threshold exceeded). The sustained-duration check
            // (duration_secs) is enforced by AlertManager::evaluate().
            AlertRule::Custom {
                plugin_id: _,
                config: _,
            } => {
                // Custom rules are evaluated by plugins
                Ok(false)
            }
        }
    }
}
