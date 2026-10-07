//! iPhone monitor: a Rust sampler that sends each reading to the frontend
//! through a Tauri `Channel` and keeps the history for the charts.

pub mod apple;
pub mod history;
pub mod mach;
pub mod net;
mod sampler;

use std::sync::{Arc, Mutex, MutexGuard};

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Manager, State};

use apple::{Battery, Storage, Thermal};
use history::{History, Point};

/// A complete device reading.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Milliseconds since the Unix epoch.
    pub ts: i64,
    /// Total CPU usage in %; `None` on the first tick or right after a gap.
    pub cpu: Option<f32>,
    pub ram_used: Option<u64>,
    pub ram_total: u64,
    /// Memory that iOS attributes to this app.
    pub app_footprint: Option<u64>,
    /// Headroom before iOS kills the app (only on a real iPhone).
    pub app_available: Option<u64>,
    /// Bytes per second.
    pub net_down: Option<f64>,
    pub net_up: Option<f64>,
    pub battery: Option<Battery>,
    pub storage: Option<Storage>,
    pub thermal: Thermal,
    pub low_power: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub model: String,
    pub identifier: String,
    pub os_version: String,
    pub cores: usize,
    pub ram_total: u64,
    pub simulator: bool,
}

#[derive(Default)]
struct Inner {
    latest: Option<Snapshot>,
    history: History,
    subscriber: Option<Channel<Snapshot>>,
}

#[derive(Clone, Default)]
pub struct MetricsState(Arc<Mutex<Inner>>);

impl MetricsState {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Latest complete reading (the chat uses it to give the model context).
    pub fn latest(&self) -> Option<Snapshot> {
        self.lock().latest.clone()
    }
}

/// Registers the shared state and starts sampling.
pub fn init(app: &AppHandle) {
    let state = MetricsState::default();
    app.manage(state.clone());
    tauri::async_runtime::spawn(sampler::run(app.clone(), state));
}

/// Subscribes the view to readings and returns the latest one, if any.
/// There is a single window, so a new subscription replaces the previous one.
#[tauri::command]
pub fn metrics_subscribe(
    state: State<'_, MetricsState>,
    on_sample: Channel<Snapshot>,
) -> Option<Snapshot> {
    let mut inner = state.lock();
    inner.subscriber = Some(on_sample);
    inner.latest.clone()
}

/// History for the charts: `"5m"` (seconds) or `"1h"` (minutes).
#[tauri::command]
pub fn metrics_history(state: State<'_, MetricsState>, range: String) -> Vec<Point> {
    let inner = state.lock();
    match range.as_str() {
        "1h" => inner.history.minutes(),
        _ => inner.history.seconds(),
    }
}

#[tauri::command]
pub fn device_info() -> DeviceInfo {
    let (identifier, simulator) = apple::model_identifier();
    DeviceInfo {
        model: apple::marketing_name(&identifier),
        identifier,
        os_version: apple::os_version(),
        cores: apple::active_cores(),
        ram_total: apple::physical_memory(),
        simulator,
    }
}
