//! Alert rule evaluation

use super::AlertContext;
use serde::{Deserialize, Serialize};
use anyhow::Result;

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
    Custom { plugin_id: String, config: serde_json::Value },
}

impl AlertRule {
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
            AlertRule::NewMentions { count: _, hours: _ } => {
                // TODO: Implement mention counting logic
                // This would require tracking mention history
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
            AlertRule::TemperatureHigh { threshold, duration_secs: _ } => {
                if let Some(ref cpu_details) = context.cpu_details {
                    return Ok(cpu_details.temperature > *threshold);
                }
                Ok(false)
            }
            AlertRule::CpuHigh { threshold, duration_secs: _ } => {
                if let Some(ref system_metrics) = context.system_metrics {
                    return Ok(system_metrics.cpu > *threshold);
                }
                Ok(false)
            }
            AlertRule::Custom { plugin_id: _, config: _ } => {
                // Custom rules are evaluated by plugins
                Ok(false)
            }
        }
    }
}
