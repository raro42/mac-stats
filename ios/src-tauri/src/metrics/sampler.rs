//! Sampling loop: CPU, RAM, network and thermal state every second; battery every
//! 30 s and storage every 60 s.
//!
//! iOS freezes the app in the background, so the loop only runs in the
//! foreground. On return, if more than `GAP` has passed since the last tick, a gap
//! is marked and the CPU and network baselines are reset; otherwise the first
//! reading would average over all the time the app was frozen.

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

pub(super) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn due(last: Option<Instant>, every: Duration, now: Instant) -> bool {
    last.is_none_or(|t| now.duration_since(t) >= every)
}

/// `UIDevice` can only be used on the main thread.
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

/// Whether the app is on screen. During a Background App Refresh wake-up it is not, and
/// those samples are marked as background ones. `UIApplication` needs the main thread.
#[cfg(target_os = "ios")]
async fn in_foreground(app: &AppHandle) -> bool {
    use objc2_ui_kit::{UIApplication, UIApplicationState};
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sent = app.run_on_main_thread(move || {
        let state = objc2::MainThreadMarker::new().map(|mtm| UIApplication::sharedApplication(mtm).applicationState());
        let _ = tx.send(state.is_none_or(|s| s != UIApplicationState::Background));
    });
    if sent.is_err() {
        return true;
    }
    tokio::time::timeout(Duration::from_millis(500), rx).await.ok().and_then(Result::ok).unwrap_or(true)
}

#[cfg(not(target_os = "ios"))]
async fn in_foreground(_app: &AppHandle) -> bool {
    true
}

/// During a background wake-up, the minute being filled is also saved every this many
/// samples, so it survives if iOS terminates the app before the minute closes.
const BACKGROUND_SAVE_EVERY: u32 = 5;

/// In debug builds, `IOS_STATS_FAKE_THERMAL=fair|serious|critical` forces the thermal
/// state to test the indicator and the chat warnings without heating up the iPhone.
fn thermal() -> apple::Thermal {
    if cfg!(debug_assertions) {
        match std::env::var("IOS_STATS_FAKE_THERMAL").as_deref() {
            Ok("fair") => return apple::Thermal::Fair,
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

        let thermal = thermal();
        let foreground = in_foreground(&app).await;
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
            thermal,
            low_power: apple::low_power_mode(),
        };
        let point = Point {
            ts,
            cpu,
            ram: ram_used
                .filter(|_| ram_total > 0)
                .map(|used| (used as f64 / ram_total as f64 * 100.0) as f32),
            app_mb: app_footprint.map(|bytes| (bytes as f64 / 1_048_576.0) as f32),
            thermal: Some(thermal),
            bg: !foreground,
            n: 1,
            gap: false,
        };

        let (closed, partial) = {
            let mut inner = state.lock();
            let closed = inner.history.push(point);
            let partial = inner
                .history
                .current_minute()
                .filter(|m| !foreground && m.n % BACKGROUND_SAVE_EVERY == 0);
            inner.latest = Some(snapshot.clone());
            if let Some(channel) = &inner.subscriber {
                if channel.send(snapshot).is_err() {
                    inner.subscriber = None;
                }
            }
            (closed, partial)
        };
        for minute in closed.iter().chain(partial.iter()) {
            state.save(minute);
        }
    }
}
