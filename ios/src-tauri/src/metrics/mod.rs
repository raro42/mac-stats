//! Monitor del iPhone: un muestreador en Rust que envía cada lectura al frontend
//! por un `Channel` de Tauri y guarda el historial para las gráficas.

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

/// Una lectura completa del dispositivo.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Milisegundos desde la época Unix.
    pub ts: i64,
    /// Uso total de CPU en %; `None` en el primer tick o justo después de un hueco.
    pub cpu: Option<f32>,
    pub ram_used: Option<u64>,
    pub ram_total: u64,
    /// Memoria que iOS atribuye a esta app.
    pub app_footprint: Option<u64>,
    /// Margen antes de que iOS cierre la app (solo en un iPhone real).
    pub app_available: Option<u64>,
    /// Bytes por segundo.
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

    /// Última lectura completa (la usa el chat para darle contexto al modelo).
    pub fn latest(&self) -> Option<Snapshot> {
        self.lock().latest.clone()
    }
}

/// Registra el estado compartido y arranca el muestreo.
pub fn init(app: &AppHandle) {
    let state = MetricsState::default();
    app.manage(state.clone());
    tauri::async_runtime::spawn(sampler::run(app.clone(), state));
}

/// Suscribe la vista a las lecturas y devuelve la última, si existe.
/// Hay una sola ventana, así que la suscripción nueva sustituye a la anterior.
#[tauri::command]
pub fn metrics_subscribe(
    state: State<'_, MetricsState>,
    on_sample: Channel<Snapshot>,
) -> Option<Snapshot> {
    let mut inner = state.lock();
    inner.subscriber = Some(on_sample);
    inner.latest.clone()
}

/// Historial para las gráficas: `"5m"` (segundos) o `"1h"` (minutos).
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
