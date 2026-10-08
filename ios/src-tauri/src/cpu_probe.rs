//! CPU probe (debug builds only): with `IOS_STATS_CPU_PROBE=<seconds>` the app waits 15 s
//! for the page to settle, then averages the iPhone's total CPU usage (all processes,
//! including the WebKit processes that draw the page) over that many seconds and writes
//! `Documents/cpu-probe.json`. Used to compare the cost of the UI themes on a device when
//! Instruments is not available. Leave the iPhone untouched while it runs.

use std::time::Duration;

use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::chat::ChatState;
use crate::metrics::MetricsState;

pub async fn run(app: AppHandle, seconds: u64) {
    let metrics = app.state::<MetricsState>().inner().clone();
    tokio::time::sleep(Duration::from_secs(15)).await;

    let mut samples = Vec::new();
    let mut last_ts = 0;
    while samples.len() < seconds as usize {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Some(s) = metrics.latest() {
            if s.ts != last_ts {
                last_ts = s.ts;
                if let Some(cpu) = s.cpu {
                    samples.push(cpu);
                }
            }
        }
    }

    let n = samples.len() as f32;
    let mean = samples.iter().sum::<f32>() / n;
    let mut sorted = samples.clone();
    sorted.sort_by(f32::total_cmp);
    let report = json!({
        "theme": app.state::<ChatState>().settings().theme,
        "seconds": samples.len(),
        "meanCpu": mean,
        "medianCpu": sorted[sorted.len() / 2],
        "maxCpu": sorted.last(),
        "samples": samples,
    });
    println!("CPU_PROBE {report}");
    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Documents/cpu-probe.json");
    let _ = std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap_or_default());
}
