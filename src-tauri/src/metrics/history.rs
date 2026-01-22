//! Metrics history collection with adaptive tiered downsampling
//!
//! This module implements efficient backend-driven metrics history storage
//! with automatic downsampling to maintain constant memory usage:
//!
//! - Tier 1 (1s):  Last 5 minutes   = 300 points  (~13 KB)
//! - Tier 2 (1m):  Last 1 hour      = 60 points   (~2.6 KB)
//! - Tier 3 (5m):  Last 6 hours     = 72 points   (~3.2 KB)
//! - Tier 4 (1h):  Last 7 days      = 168 points  (~7.2 KB)
//!
//! Total memory: ~26 KB (negligible)
//!
//! ## Design
//!
//! - Each tier is a VecDeque that auto-evicts oldest points when full
//! - Points are automatically promoted from Tier 1 → 2 → 3 → 4 via downsampling
//! - Downsampling uses rolling averages of all metrics for smooth transitions
//! - Frontend can query any time range and specify max_points for display

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// A single point in metrics history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: i64,           // Unix timestamp (seconds)
    pub cpu: f32,                 // CPU usage percentage (0-100)
    pub gpu: f32,                 // GPU usage percentage (0-100)
    pub ram: f32,                 // RAM usage percentage (0-100)
    pub disk: f32,                // Disk usage percentage (0-100)
    pub temperature: f32,         // Temperature in Celsius
    pub frequency: f32,           // CPU frequency in GHz
    pub p_core_frequency: f32,    // P-core frequency in GHz
    pub e_core_frequency: f32,    // E-core frequency in GHz
    pub cpu_power: f32,           // CPU power consumption in Watts
    pub gpu_power: f32,           // GPU power consumption in Watts
    pub battery_level: f32,       // Battery level (0-100), or -1.0 if N/A
}

impl MetricPoint {
    /// Create a new metric point from current metrics
    pub fn from_metrics(
        cpu: f32,
        gpu: f32,
        ram: f32,
        disk: f32,
        temperature: f32,
        frequency: f32,
        p_core_frequency: f32,
        e_core_frequency: f32,
        cpu_power: f32,
        gpu_power: f32,
        battery_level: f32,
    ) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            cpu,
            gpu,
            ram,
            disk,
            temperature,
            frequency,
            p_core_frequency,
            e_core_frequency,
            cpu_power,
            gpu_power,
            battery_level,
        }
    }

    /// Average multiple points together (for downsampling)
    pub fn average(points: &[MetricPoint]) -> Self {
        if points.is_empty() {
            return Self {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
                cpu: 0.0,
                gpu: 0.0,
                ram: 0.0,
                disk: 0.0,
                temperature: 0.0,
                frequency: 0.0,
                p_core_frequency: 0.0,
                e_core_frequency: 0.0,
                cpu_power: 0.0,
                gpu_power: 0.0,
                battery_level: -1.0,
            };
        }

        let count = points.len() as f32;
        Self {
            timestamp: points[points.len() / 2].timestamp, // Use middle timestamp
            cpu: points.iter().map(|p| p.cpu).sum::<f32>() / count,
            gpu: points.iter().map(|p| p.gpu).sum::<f32>() / count,
            ram: points.iter().map(|p| p.ram).sum::<f32>() / count,
            disk: points.iter().map(|p| p.disk).sum::<f32>() / count,
            temperature: points.iter().map(|p| p.temperature).sum::<f32>() / count,
            frequency: points.iter().map(|p| p.frequency).sum::<f32>() / count,
            p_core_frequency: points.iter().map(|p| p.p_core_frequency).sum::<f32>() / count,
            e_core_frequency: points.iter().map(|p| p.e_core_frequency).sum::<f32>() / count,
            cpu_power: points.iter().map(|p| p.cpu_power).sum::<f32>() / count,
            gpu_power: points.iter().map(|p| p.gpu_power).sum::<f32>() / count,
            battery_level: points.iter().map(|p| p.battery_level).sum::<f32>() / count,
        }
    }
}

/// Adaptive tiered metrics history buffer
pub struct HistoryBuffer {
    /// Tier 1: 1-second granularity, last 5 minutes (300 points)
    tier1_1s: VecDeque<MetricPoint>,
    /// Tier 2: 1-minute granularity, last 1 hour (60 points)
    tier2_1m: VecDeque<MetricPoint>,
    /// Tier 3: 5-minute granularity, last 6 hours (72 points)
    tier3_5m: VecDeque<MetricPoint>,
    /// Tier 4: 1-hour granularity, last 7 days (168 points)
    tier4_1h: VecDeque<MetricPoint>,

    /// Last timestamp we processed a Tier 2 downsampling
    last_tier2_downsample: i64,
    /// Last timestamp we processed a Tier 3 downsampling
    last_tier3_downsample: i64,
    /// Last timestamp we processed a Tier 4 downsampling
    last_tier4_downsample: i64,
}

impl HistoryBuffer {
    /// Create a new history buffer with empty tiers
    pub fn new() -> Self {
        Self {
            tier1_1s: VecDeque::with_capacity(301),     // 300 + 1 for overflow
            tier2_1m: VecDeque::with_capacity(61),      // 60 + 1 for overflow
            tier3_5m: VecDeque::with_capacity(73),      // 72 + 1 for overflow
            tier4_1h: VecDeque::with_capacity(169),     // 168 + 1 for overflow
            last_tier2_downsample: 0,
            last_tier3_downsample: 0,
            last_tier4_downsample: 0,
        }
    }

    /// Add a new metric point to the history
    pub fn push(&mut self, point: MetricPoint) {
        let timestamp = point.timestamp;

        // Add to Tier 1
        self.tier1_1s.push_back(point.clone());
        if self.tier1_1s.len() > 300 {
            self.tier1_1s.pop_front();
        }

        // Auto-downsample to Tier 2 every 60 seconds (60 1-second points)
        if timestamp - self.last_tier2_downsample >= 60 {
            self.downsample_to_tier2();
            self.last_tier2_downsample = timestamp;
        }

        // Auto-downsample to Tier 3 every 300 seconds (5 minutes, 60 1-minute points)
        if timestamp - self.last_tier3_downsample >= 300 {
            self.downsample_to_tier3();
            self.last_tier3_downsample = timestamp;
        }

        // Auto-downsample to Tier 4 every 3600 seconds (1 hour, 72 5-minute points)
        if timestamp - self.last_tier4_downsample >= 3600 {
            self.downsample_to_tier4();
            self.last_tier4_downsample = timestamp;
        }
    }

    /// Downsample from Tier 1 to Tier 2 (average every 60 points into 1)
    fn downsample_to_tier2(&mut self) {
        if self.tier1_1s.len() < 60 {
            return; // Not enough points yet
        }

        // Take last 60 points from Tier 1
        let points_to_downsample: Vec<_> = self.tier1_1s.iter().rev().take(60).cloned().collect();
        if points_to_downsample.len() == 60 {
            let mut points_to_downsample = points_to_downsample;
            points_to_downsample.reverse();
            let averaged = MetricPoint::average(&points_to_downsample);
            self.tier2_1m.push_back(averaged);
            if self.tier2_1m.len() > 60 {
                self.tier2_1m.pop_front();
            }
        }
    }

    /// Downsample from Tier 2 to Tier 3 (average every 5 points into 1, representing 5 minutes)
    fn downsample_to_tier3(&mut self) {
        if self.tier2_1m.len() < 5 {
            return; // Not enough points yet
        }

        // Take last 5 points from Tier 2 (5 minutes of 1-minute data)
        let points_to_downsample: Vec<_> = self.tier2_1m.iter().rev().take(5).cloned().collect();
        if points_to_downsample.len() == 5 {
            let mut points_to_downsample = points_to_downsample;
            points_to_downsample.reverse();
            let averaged = MetricPoint::average(&points_to_downsample);
            self.tier3_5m.push_back(averaged);
            if self.tier3_5m.len() > 72 {
                self.tier3_5m.pop_front();
            }
        }
    }

    /// Downsample from Tier 3 to Tier 4 (average every 12 points into 1, representing 1 hour)
    fn downsample_to_tier4(&mut self) {
        if self.tier3_5m.len() < 12 {
            return; // Not enough points yet (12 * 5min = 60 min = 1 hour)
        }

        // Take last 12 points from Tier 3 (1 hour of 5-minute data)
        let points_to_downsample: Vec<_> = self.tier3_5m.iter().rev().take(12).cloned().collect();
        if points_to_downsample.len() == 12 {
            let mut points_to_downsample = points_to_downsample;
            points_to_downsample.reverse();
            let averaged = MetricPoint::average(&points_to_downsample);
            self.tier4_1h.push_back(averaged);
            if self.tier4_1h.len() > 168 {
                self.tier4_1h.pop_front();
            }
        }
    }

    /// Get total number of data points across all tiers
    #[allow(dead_code)] // Used in tests
    pub fn total_points(&self) -> usize {
        self.tier1_1s.len() + self.tier2_1m.len() + self.tier3_5m.len() + self.tier4_1h.len()
    }

    /// Get memory usage estimate in bytes
    #[allow(dead_code)] // Utility method for debugging/monitoring
    pub fn estimate_memory_bytes(&self) -> usize {
        // Each point is roughly 100 bytes when serialized
        const BYTES_PER_POINT: usize = 100;
        self.total_points() * BYTES_PER_POINT
    }

    /// Get the oldest timestamp in history
    pub fn oldest_timestamp(&self) -> Option<i64> {
        [
            self.tier4_1h.front().map(|p| p.timestamp),
            self.tier3_5m.front().map(|p| p.timestamp),
            self.tier2_1m.front().map(|p| p.timestamp),
            self.tier1_1s.front().map(|p| p.timestamp),
        ]
        .iter()
        .filter_map(|ts| *ts)
        .min()
    }

    /// Query history for a given time range with optional downsampling for display
    pub fn query(&self, time_range_seconds: u64, max_display_points: Option<usize>) -> Vec<MetricPoint> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let start_time = now - time_range_seconds as i64;

        let mut points = Vec::new();

        // Select appropriate tier based on time range
        match time_range_seconds {
            0..=300 => {
                // Last 5 minutes: use Tier 1 (1s granularity)
                for p in &self.tier1_1s {
                    if p.timestamp >= start_time {
                        points.push(p.clone());
                    }
                }
            }
            301..=3600 => {
                // Up to 1 hour: use Tier 2 (1m granularity) + remaining from Tier 1
                for p in &self.tier2_1m {
                    if p.timestamp >= start_time {
                        points.push(p.clone());
                    }
                }
                // Add recent Tier 1 data (last 5 minutes)
                for p in &self.tier1_1s {
                    if p.timestamp > now - 300 {
                        points.push(p.clone());
                    }
                }
                points.sort_by_key(|p| p.timestamp);
            }
            3601..=21600 => {
                // Up to 6 hours: use Tier 3 (5m granularity) + remaining from Tier 2
                for p in &self.tier3_5m {
                    if p.timestamp >= start_time {
                        points.push(p.clone());
                    }
                }
                // Add recent Tier 2 data
                for p in &self.tier2_1m {
                    if p.timestamp > now - 3600 {
                        points.push(p.clone());
                    }
                }
                points.sort_by_key(|p| p.timestamp);
            }
            _ => {
                // More than 6 hours: use Tier 4 (1h granularity) + remaining from Tier 3
                for p in &self.tier4_1h {
                    if p.timestamp >= start_time {
                        points.push(p.clone());
                    }
                }
                // Add recent Tier 3 data
                for p in &self.tier3_5m {
                    if p.timestamp > now - 21600 {
                        points.push(p.clone());
                    }
                }
                points.sort_by_key(|p| p.timestamp);
            }
        }

        // Apply display width downsampling if needed
        if let Some(max_points) = max_display_points {
            if points.len() > max_points {
                self.downsample_for_display(&points, max_points)
            } else {
                points
            }
        } else {
            points
        }
    }

    /// Downsample points for screen display (every nth point)
    fn downsample_for_display(&self, points: &[MetricPoint], target_count: usize) -> Vec<MetricPoint> {
        if points.is_empty() {
            return Vec::new();
        }

        if points.len() <= target_count {
            return points.to_vec();
        }

        let step = (points.len() + target_count - 1) / target_count;
        points.iter().step_by(step).cloned().collect()
    }
}

impl Default for HistoryBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryQueryResult {
    pub points: Vec<MetricPoint>,
    pub time_range_seconds: u64,
    pub oldest_available_timestamp: Option<i64>,
    pub newest_available_timestamp: Option<i64>,
}

impl HistoryBuffer {
    /// Optional: Save history to disk for persistence across restarts
    /// Saves to ~/.mac-stats/history.json
    #[allow(dead_code)] // Reserved for future persistence feature
    pub fn save_to_disk(&self) -> Result<(), String> {
        let home = std::env::var("HOME").map_err(|_| "Could not determine HOME directory".to_string())?;
        let history_dir = std::path::Path::new(&home).join(".mac-stats");
        let history_file = history_dir.join("history.json");

        // Serialize all tiers
        let all_points = serde_json::json!({
            "tier1_1s": self.tier1_1s.iter().collect::<Vec<_>>(),
            "tier2_1m": self.tier2_1m.iter().collect::<Vec<_>>(),
            "tier3_5m": self.tier3_5m.iter().collect::<Vec<_>>(),
            "tier4_1h": self.tier4_1h.iter().collect::<Vec<_>>(),
            "saved_at": chrono::Local::now().to_rfc3339(),
        });

        let json_str = serde_json::to_string_pretty(&all_points)
            .map_err(|e| format!("Serialization error: {}", e))?;

        std::fs::write(history_file, json_str)
            .map_err(|e| format!("Failed to write history file: {}", e))?;

        Ok(())
    }

    /// Optional: Load history from disk
    /// Loads from ~/.mac-stats/history.json if it exists
    #[allow(dead_code)] // Reserved for future persistence feature
    pub fn load_from_disk() -> Result<Self, String> {
        let home = std::env::var("HOME").map_err(|_| "Could not determine HOME directory".to_string())?;
        let history_dir = std::path::Path::new(&home).join(".mac-stats");
        let history_file = history_dir.join("history.json");

        if !history_file.exists() {
            return Ok(Self::new()); // Return empty buffer if file doesn't exist
        }

        let json_str = std::fs::read_to_string(history_file)
            .map_err(|e| format!("Failed to read history file: {}", e))?;

        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse history JSON: {}", e))?;

        // Reconstruct buffers from JSON
        let mut buffer = Self::new();

        if let Some(tier1) = data["tier1_1s"].as_array() {
            for point_val in tier1 {
                if let Ok(point) = serde_json::from_value::<MetricPoint>(point_val.clone()) {
                    buffer.tier1_1s.push_back(point);
                }
            }
        }

        if let Some(tier2) = data["tier2_1m"].as_array() {
            for point_val in tier2 {
                if let Ok(point) = serde_json::from_value::<MetricPoint>(point_val.clone()) {
                    buffer.tier2_1m.push_back(point);
                }
            }
        }

        if let Some(tier3) = data["tier3_5m"].as_array() {
            for point_val in tier3 {
                if let Ok(point) = serde_json::from_value::<MetricPoint>(point_val.clone()) {
                    buffer.tier3_5m.push_back(point);
                }
            }
        }

        if let Some(tier4) = data["tier4_1h"].as_array() {
            for point_val in tier4 {
                if let Ok(point) = serde_json::from_value::<MetricPoint>(point_val.clone()) {
                    buffer.tier4_1h.push_back(point);
                }
            }
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_point_average() {
        let points = vec![
            MetricPoint::from_metrics(10.0, 5.0, 20.0, 30.0, 50.0, 2.0, 2.0, 1.5, 5.0, 3.0, 80.0),
            MetricPoint::from_metrics(20.0, 10.0, 30.0, 40.0, 60.0, 2.1, 2.1, 1.6, 6.0, 4.0, 70.0),
            MetricPoint::from_metrics(30.0, 15.0, 40.0, 50.0, 70.0, 2.2, 2.2, 1.7, 7.0, 5.0, 60.0),
        ];

        let avg = MetricPoint::average(&points);
        assert_eq!(avg.cpu, 20.0);
        assert_eq!(avg.gpu, 10.0);
        assert_eq!(avg.ram, 30.0);
    }

    #[test]
    fn test_history_buffer_creation() {
        let buffer = HistoryBuffer::new();
        assert_eq!(buffer.total_points(), 0);
    }

    #[test]
    fn test_history_buffer_push() {
        let mut buffer = HistoryBuffer::new();
        let point = MetricPoint::from_metrics(50.0, 30.0, 60.0, 70.0, 65.0, 2.5, 2.5, 1.8, 8.0, 6.0, 100.0);
        buffer.push(point);
        assert_eq!(buffer.tier1_1s.len(), 1);
    }
}
