//! Application state management
//! 
//! This module manages global application state including:
//! - System information (CPU, RAM, Disk)
//! - UI state (status bar, menu items)
//! - Caches (temperature, frequency, chip info)
//! - IOReport subscriptions
//! 
//! Note: Some state remains global due to:
//! - Thread-local requirements (UI must be on main thread)
//! - Cross-thread access patterns
//! - Tauri's architecture requiring global handles
//! 
//! Future improvement: Consider consolidating into AppState struct
//! and passing it through Tauri's state management.

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
// Note: Thread-local is required for UI elements that must be accessed from main thread
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

// Process list cache: (process_list, last_update_timestamp)
// Cache processes for 30 seconds to avoid expensive refresh on every call
pub(crate) static PROCESS_CACHE: Mutex<Option<(Vec<crate::metrics::ProcessUsage>, Instant)>> = Mutex::new(None);
// P-core and E-core frequency caches: (frequency_value_ghz, last_update_timestamp)
pub(crate) static P_CORE_FREQ_CACHE: Mutex<Option<(f32, Instant)>> = Mutex::new(None);
pub(crate) static E_CORE_FREQ_CACHE: Mutex<Option<(f32, Instant)>> = Mutex::new(None);
#[allow(dead_code)]
pub(crate) static M3_FREQ_KEY: Mutex<Option<String>> = Mutex::new(None);
pub(crate) static NOMINAL_FREQ: OnceLock<f32> = OnceLock::new();
pub(crate) static LAST_FREQ_READ: Mutex<Option<Instant>> = Mutex::new(None);
pub(crate) static LAST_TEMP_UPDATE: Mutex<Option<Instant>> = Mutex::new(None);

// Rate limiting for get_cpu_details() - prevent excessive calls
pub(crate) static LAST_CPU_DETAILS_CALL: Mutex<Option<Instant>> = Mutex::new(None);

// IOReport state
pub(crate) static IOREPORT_SUBSCRIPTION: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static IOREPORT_CHANNELS: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static IOREPORT_SUBSCRIPTION_DICT: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static IOREPORT_ORIGINAL_CHANNELS: Mutex<Option<usize>> = Mutex::new(None);
pub(crate) static LAST_IOREPORT_SAMPLE: Mutex<Option<(usize, Instant)>> = Mutex::new(None);

/// Application state structure (future refactoring target)
/// 
/// This struct represents the ideal state organization.
/// Currently, state is stored in global statics for compatibility
/// with existing code and Tauri's architecture.
/// 
/// Future work: Migrate global statics to this struct and pass
/// it through Tauri's state management system.
#[allow(dead_code)]
pub struct AppState {
    // System information
    pub system: Option<System>,
    pub disks: Option<Disks>,
    pub last_system_refresh: Option<Instant>,
    
    // Caches
    pub chip_info: Option<String>,
    pub access_flags: Option<(bool, bool, bool, bool)>, // temp, freq, cpu_power, gpu_power
    pub temp_cache: Option<(f32, Instant)>,
    pub freq_cache: Option<(f32, Instant)>,
    pub nominal_freq: Option<f32>,
    
    // IOReport
    pub ioreport_subscription: Option<usize>,
    pub ioreport_channels: Option<usize>,
}

impl AppState {
    /// Create a new AppState instance
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            system: None,
            disks: None,
            last_system_refresh: None,
            chip_info: None,
            access_flags: None,
            temp_cache: None,
            freq_cache: None,
            nominal_freq: None,
            ioreport_subscription: None,
            ioreport_channels: None,
        }
    }
}
