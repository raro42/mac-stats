//! Bucle de muestreo: CPU, RAM, red y estado térmico cada segundo; batería cada
//! 30 s y almacenamiento cada 60 s.
//!
//! iOS congela la app en segundo plano, así que el bucle solo corre en primer
//! plano. Al volver, si pasaron más de `GAP` desde el último tick, se marca un
//! hueco y se rehace la línea base de CPU y red; si no, la primera lectura
//! promediaría todo el tiempo que la app estuvo congelada.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::AppHandle;
use tokio::time::MissedTickBehavior;

use super::apple::{self, Battery, Storage};
use super::history::Point;
use super::mach::{self, CpuTicks};
use super::net::NetMeter;
use super::{MetricsState, Snapshot};

const TICK: Duration = Duration::from_secs(1);
const GAP: Duration = Duration::from_millis(2_500);
const BATTERY_EVERY: Duration = Duration::from_secs(30);
const STORAGE_EVERY: Duration = Duration::from_secs(60);

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn due(last: Option<Instant>, every: Duration, now: Instant) -> bool {
    last.is_none_or(|t| now.duration_since(t) >= every)
}

/// `UIDevice` solo se puede usar en el hilo principal.
#[cfg(target_os = "ios")]
async fn read_battery(app: &AppHandle) -> Option<Battery> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(apple::battery_on_main_thread());
    })
    .ok()?;
    tokio::time::timeout(Duration::from_secs(2), rx)
        .await
        .ok()?
        .ok()?
}

#[cfg(not(target_os = "ios"))]
async fn read_battery(_app: &AppHandle) -> Option<Battery> {
    None
}

/// En depuración, `IOS_STATS_FAKE_THERMAL=serious|critical` fuerza el estado térmico
/// para probar los avisos del chat sin calentar el iPhone.
fn thermal() -> apple::Thermal {
    if cfg!(debug_assertions) {
        match std::env::var("IOS_STATS_FAKE_THERMAL").as_deref() {
            Ok("serious") => return apple::Thermal::Serious,
            Ok("critical") => return apple::Thermal::Critical,
            _ => {}
        }
    }
    apple::thermal_state()
}

fn read_storage() -> Option<Storage> {
    let home = std::env::var("HOME").ok()?;
    apple::storage(&home)
}

pub(super) async fn run(app: AppHandle, state: MetricsState) {
    let ram_total = apple::physical_memory();
    let mut interval = tokio::time::interval(TICK);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut prev_ticks: Option<CpuTicks> = None;
    let mut net = NetMeter::default();
    let mut last_tick: Option<Instant> = None;
    let mut battery: Option<Battery> = None;
    let mut battery_at: Option<Instant> = None;
    let mut storage: Option<Storage> = None;
    let mut storage_at: Option<Instant> = None;

    loop {
        interval.tick().await;
        let now = Instant::now();
        let ts = now_ms();

        let resumed = last_tick.is_some_and(|t| now.duration_since(t) > GAP);
        last_tick = Some(now);
        if resumed {
            prev_ticks = None;
            net.reset();
            state.lock().history.push_gap(ts - 1);
        }

        let ticks = mach::cpu_ticks();
        let cpu = match (ticks, prev_ticks) {
            (Some(current), Some(prev)) => current.usage_since(&prev),
            _ => None,
        };
        prev_ticks = ticks;

        if resumed || due(battery_at, BATTERY_EVERY, now) {
            battery = read_battery(&app).await;
            battery_at = Some(now);
        }
        if resumed || due(storage_at, STORAGE_EVERY, now) {
            storage = read_storage();
            storage_at = Some(now);
        }

        let ram_used = mach::system_memory_used();
        let app_footprint = mach::app_footprint();
        let (net_down, net_up) = match net.sample() {
            Some((down, up)) => (Some(down), Some(up)),
            None => (None, None),
        };

        let snapshot = Snapshot {
            ts,
            cpu,
            ram_used,
            ram_total,
            app_footprint,
            app_available: mach::app_available_memory(),
            net_down,
            net_up,
            battery,
            storage,
            thermal: thermal(),
            low_power: apple::low_power_mode(),
        };
        let point = Point {
            ts,
            cpu,
            ram: ram_used
                .filter(|_| ram_total > 0)
                .map(|used| (used as f64 / ram_total as f64 * 100.0) as f32),
            app_mb: app_footprint.map(|bytes| (bytes as f64 / 1_048_576.0) as f32),
            gap: false,
        };

        let mut inner = state.lock();
        inner.history.push(point);
        inner.latest = Some(snapshot.clone());
        if let Some(channel) = &inner.subscriber {
            if channel.send(snapshot).is_err() {
                inner.subscriber = None;
            }
        }
    }
}
