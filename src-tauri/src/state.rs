use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use sysinfo::{Disks, System};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSStatusItem;
use tauri::AppHandle;

// System state
pub(crate) static SYSTEM: Mutex<Option<System>> = Mutex::new(None);
pub(crate) static DISKS: Mutex<Option<Disks>> = Mutex::new(None);
pub(crate) static LAST_SYSTEM_REFRESH: Mutex<Option<Instant>> = Mutex::new(None);

// UI state
thread_local! {
    pub(crate) static STATUS_ITEM: RefCell<Option<Retained<NSStatusItem>>> = RefCell::new(None);
    pub(crate) static CLICK_HANDLER: RefCell<Option<Retained<AnyObject>>> = RefCell::new(None);
}
pub(crate) static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
pub(crate) static MENU_BAR_TEXT: Mutex<Option<String>> = Mutex::new(None);

// Caches
pub(crate) static CHIP_INFO_CACHE: OnceLock<String> = OnceLock::new();
pub(crate) static ACCESS_CACHE: Mutex<Option<(bool, bool, bool, bool)>> = Mutex::new(None); // temp, freq, cpu_power, gpu_power

// Temperature cache: (temperature_value, last_update_timestamp)
pub(crate) static TEMP_CACHE: Mutex<Option<(f32, Instant)>> = Mutex::new(None);
pub(crate) static M3_TEMP_KEY: Mutex<Option<String>> = Mutex::new(None);

// Frequency cache: (frequency_value_ghz, last_update_timestamp)
pub(crate) static FREQ_CACHE: Mutex<Option<(f32, Instant)>> = Mutex::new(None);
#[allow(dead_code)]
pub(crate) static M3_FREQ_KEY: Mutex<Option<String>> = Mutex::new(None);
pub(crate) static NOMINAL_FREQ: OnceLock<f32> = OnceLock::new();
pub(crate) static LAST_FREQ_READ: Mutex<Option<Instant>> = Mutex::new(None);
pub(crate) static LAST_TEMP_UPDATE: Mutex<Option<Instant>> = Mutex::new(None);

// IOReport state
pub(crate) static IOREPORT_SUBSCRIPTION: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static IOREPORT_CHANNELS: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static IOREPORT_SUBSCRIPTION_DICT: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static IOREPORT_ORIGINAL_CHANNELS: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static LAST_IOREPORT_SAMPLE: Mutex<Option<(usize, Instant)>> = Mutex::new(None);
