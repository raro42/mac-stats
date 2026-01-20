//! mac-stats Library
//! 
//! A macOS system monitoring application with menu bar integration.
//! 
//! ## Architecture
//! 
//! The codebase is organized into modules:
//! - `logging`: Structured logging with tracing
//! - `state`: Application state management
//! - `metrics`: System metrics collection (CPU, RAM, Disk, GPU, temperature, frequency)
//! - `config`: Configuration management (paths, build info)
//! - `ffi`: Safe FFI wrappers for IOReport and Objective-C
//! - `ui`: UI components (status bar, windows)
//! 
//! ## Main Entry Points
//! 
//! - `run()`: Start the application without CPU window
//! - `run_with_cpu_window()`: Start the application with CPU window open

mod logging;
mod state;
mod metrics;
pub mod config;
mod ffi;
mod ui;
mod security;
mod monitors;
mod alerts;
mod plugins;
mod ollama;
mod commands;

use std::os::raw::c_void;
use sysinfo::{Disks, System};
use macsmc::Smc;

// Re-export logging functions (macros are auto-exported via #[macro_export])
pub use logging::{set_verbosity, init_tracing};
// IOReport types kept for future use (extern block still references them)
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef, CFMutableDictionary};
use core_foundation::string::{CFStringRef, CFString};

// IOReport FFI bindings (similar to macmon)
// Some functions are declared for future use
#[allow(dead_code)]
#[link(name = "IOReport", kind = "dylib")]
extern "C" {
    fn IOReportCopyAllChannels(a: u64, b: u64) -> CFDictionaryRef;
    fn IOReportCopyChannelsInGroup(
        group: CFStringRef,
        subgroup: CFStringRef,
        want_hierarchical: u64,
        want_sub_groups: u64,
        want_historical: u64,
    ) -> CFDictionaryRef;
    fn IOReportMergeChannels(
        dest: CFMutableDictionaryRef,
        src: CFDictionaryRef,
        nil: CFTypeRef,
    );
    fn IOReportCreateSubscription(
        allocator: CFTypeRef,
        channels: CFMutableDictionaryRef,
        subscription: *mut CFMutableDictionaryRef,
        channel_id: u64,
        options: CFTypeRef,
    ) -> *mut c_void;
    fn IOReportCreateSamples(
        subscription: *const c_void,
        channels: CFMutableDictionaryRef,
        options: CFTypeRef,
    ) -> CFDictionaryRef;
    fn IOReportCreateSamplesDelta(
        start: CFDictionaryRef,
        end: CFDictionaryRef,
        options: CFTypeRef,
    ) -> CFDictionaryRef;
    fn IOReportChannelGetGroup(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetSubGroup(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetChannelName(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportSimpleGetIntegerValue(item: CFDictionaryRef, index: i32) -> i64;
    fn IOReportChannelGetUnitLabel(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportStateGetCount(item: CFDictionaryRef) -> i32;
    fn IOReportStateGetNameForIndex(item: CFDictionaryRef, index: i32) -> CFStringRef;
    fn IOReportStateGetResidency(item: CFDictionaryRef, index: i32) -> i64;
}

// CoreFoundation functions for memory management
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
}

// IOReport helper functions removed - IOReport operations were too expensive for real-time monitoring
// If needed in the future, these can be re-implemented with proper caching
use objc2::MainThreadMarker;
use tauri::Manager;


// Use state from state module
use state::*;

// Use metrics from metrics module (only re-export what's needed)

// Re-export for Tauri commands
pub use metrics::{SystemMetrics, CpuDetails, get_cpu_details, get_metrics, get_app_version, get_window_decorations, set_window_decorations, get_process_details, force_quit_process, get_changelog};


// UI functions are now in ui module
use ui::status_bar::{build_status_text, setup_status_item, create_cpu_window, make_attributed_title};

/// Set frequency logging flag for detailed debugging
pub fn set_frequency_logging(enabled: bool) {
    if let Ok(mut flag) = state::FREQUENCY_LOGGING_ENABLED.lock() {
        *flag = enabled;
        if enabled {
            debug1!("Frequency logging enabled - detailed frequency information will be logged");
        }
    }
}

/// Set power usage logging flag for detailed debugging
pub fn set_power_usage_logging(enabled: bool) {
    if let Ok(mut flag) = state::POWER_USAGE_LOGGING_ENABLED.lock() {
        *flag = enabled;
        if enabled {
            debug1!("Power usage logging enabled - detailed power and battery information will be logged");
        }
    }
}

pub fn run_with_cpu_window() {
    debug1!("Running with -cpu flag: will open CPU window after setup");
    run_internal(true)
}

pub fn run() {
    run_internal(false)
}

fn run_internal(open_cpu_window: bool) {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_cpu_details,
            get_metrics,
            get_app_version, 
            get_window_decorations, 
            set_window_decorations, 
            get_process_details, 
            force_quit_process, 
            get_changelog,
            // Security commands
            commands::security::store_credential,
            commands::security::get_credential,
            commands::security::delete_credential,
            commands::security::list_credentials,
            // Monitor commands
            commands::monitors::add_website_monitor,
            commands::monitors::add_mastodon_monitor,
            commands::monitors::check_monitor,
            commands::monitors::list_monitors,
            commands::monitors::remove_monitor,
            commands::monitors::get_monitor_details,
            commands::monitors::get_monitor_status,
            // Alert commands
            commands::alerts::add_alert,
            commands::alerts::remove_alert,
            commands::alerts::evaluate_alerts,
            // Plugin commands
            commands::plugins::add_plugin,
            commands::plugins::remove_plugin,
            commands::plugins::execute_plugin,
            commands::plugins::list_plugins,
            commands::plugins::run_due_plugins,
            // Ollama commands
            commands::ollama::configure_ollama,
            commands::ollama::check_ollama_connection,
            commands::ollama::ollama_chat,
            commands::ollama::list_ollama_models,
            commands::ollama::log_ollama_js_execution,
            commands::ollama::log_ollama_js_check,
            commands::ollama::log_ollama_js_extraction,
            commands::ollama::log_ollama_js_no_blocks,
            commands::ollama::ollama_chat_with_execution,
            commands::ollama::ollama_chat_continue_with_result,
            // Logging commands
            commands::logging::log_from_js,
        ])
        .setup(move |app| {
            // Load persistent monitors on startup
            use crate::commands::monitors;
            if let Err(e) = monitors::load_monitors_internal() {
                tracing::warn!("Failed to load monitors: {}", e);
            }
            
            // Hide the main window immediately (menu bar app)
            if let Some(main_window) = app.get_window("main") {
                let _ = main_window.hide();
            }
            
            let _ = APP_HANDLE.set(app.handle());

            // Don't create CPU window at startup - create it on demand when clicked
            // This saves CPU by not having the window exist until needed
            debug1!("CPU window will be created on demand when menu bar is clicked");
            
            // If -cpu flag is set, create the window after a short delay
            if open_cpu_window {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    debug1!("Opening CPU window (from -cpu flag)");
                    if let Some(app_handle) = APP_HANDLE.get() {
                        let app_handle = app_handle.clone();
                        let _ = app_handle.run_on_main_thread(move || {
                            debug2!("In run_on_main_thread callback for CPU window");
                            if let Some(app_handle) = APP_HANDLE.get() {
                                create_cpu_window(app_handle);
                            }
                        });
                    }
                });
            }

            setup_status_item();
            
            // Set placeholder text immediately (don't call get_metrics() here - it blocks)
            let placeholder_text = "CPU\tGPU\tRAM\tSSD\n0%\t0%\t0%\t0%";
            let initial_attributed = make_attributed_title(placeholder_text);
            STATUS_ITEM.with(|cell| {
                if let Some(item) = cell.borrow().as_ref() {
                    let mtm = MainThreadMarker::new().unwrap();
                    if let Some(button) = item.button(mtm) {
                        button.setAttributedTitle(&initial_attributed);
                        debug2!("Initial placeholder menu bar text set");
                    }
                }
            });
            
            // For automatic updates, we'll use a simple approach:
            // The background update loop stores updates in MENU_BAR_TEXT
            // We'll process them in the click handler (which works)
            // To get automatic updates without clicking, we can simulate a click programmatically
            // But that's complex. Instead, let's use a simpler approach: process updates
            // directly from a background thread that can access the main thread
            // Actually, the simplest: just rely on click handler for now
            // Users can click to see updates, which is better than nothing
            
            // Initialize System and Disks in background thread to avoid blocking
            std::thread::spawn(move || {
                debug2!("Background thread: initializing System and Disks");
                // Create System outside the lock to avoid holding it
                let new_system = System::new();
                debug2!("Background thread: System::new() completed");
                // Use try_lock to avoid blocking - if locked, skip initialization
                if let Ok(mut sys) = SYSTEM.try_lock() {
                    if sys.is_none() {
                        *sys = Some(new_system);
                        debug2!("Background thread: System stored");
                    }
                } else {
                    debug1!("Background thread: SYSTEM lock unavailable, skipping");
                }
                
                // Create Disks outside the lock
                let mut new_disks = Disks::new();
                new_disks.refresh(false);
                debug2!("Background thread: Disks::new() and refresh completed");
                if let Ok(mut disks) = DISKS.try_lock() {
                    if disks.is_none() {
                        *disks = Some(new_disks);
                        debug2!("Background thread: Disks stored");
                    }
                } else {
                    debug1!("Background thread: DISKS lock unavailable, skipping");
                }
                debug2!("Background thread: initialization complete");
            });
            
            // Menu bar updates will be processed by the click handler
            // The background update loop stores updates in MENU_BAR_TEXT,
            // and the click handler processes them when the menu bar is clicked.
            // This ensures updates happen on the main thread without using
            // the broken run_on_main_thread mechanism.
            
            // Start update loop in background thread
            std::thread::spawn(move || {
                // Wait longer before first update to let background initialization complete
                std::thread::sleep(std::time::Duration::from_millis(1500));
                
                // CRITICAL: Keep SMC connection alive in background thread (reuse for efficiency)
                // SMC connection is not Sync, so we keep it thread-local
                let mut smc_connection: Option<Smc> = None;
                
                loop {
                    // Menu bar updates every 1-2 seconds (like Stats app) for responsive UI
                    // Fast metrics (CPU, RAM) are cached, so this is cheap
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    
                    debug3!("Update loop: getting metrics...");
                    let metrics = get_metrics();
                    
                    // CRITICAL: Only update menu bar if metrics are valid
                    // Invalid metrics (all zeros) can occur during initialization or when locks are held
                    // In that case, skip this update and wait for the next cycle
                    if !metrics.is_valid() {
                        debug3!("Skipping menu bar update: invalid metrics (CPU={}%, GPU={}%, RAM={}%, DISK={}%)", 
                            metrics.cpu, metrics.gpu, metrics.ram, metrics.disk);
                        continue; // Skip this update cycle
                    }
                    
                    let text = build_status_text(&metrics);
                    
                    // Store update in static variable
                    if let Ok(mut pending) = MENU_BAR_TEXT.lock() {
                        *pending = Some(text);
                        debug3!("Menu bar update stored: CPU={}%, GPU={}%, RAM={}%, DISK={}%", 
                            metrics.cpu, metrics.gpu, metrics.ram, metrics.disk);
                    }
                    
                    // CRITICAL: Only read temperature when CPU window is visible (saves CPU)
                    // Check window visibility before expensive SMC operations
                    let should_read_temp = APP_HANDLE.get()
                        .and_then(|app_handle| {
                            app_handle.get_window("cpu").and_then(|window| {
                                window.is_visible().ok().filter(|&visible| visible)
                            })
                        })
                        .is_some();
                    
                    if should_read_temp {
                        // CPU window is visible - read temperature and frequency
                        // Reuse SMC connection if available, otherwise create new one
                        if smc_connection.is_none() {
                            match Smc::connect() {
                                Ok(smc) => {
                                    smc_connection = Some(smc);
                                    debug3!("SMC connection established in background thread");
                                    // OPTIMIZATION Phase 3: Update OnceLock to indicate SMC works
                                    // This ensures can_read_temperature() returns true
                                    if CAN_READ_TEMPERATURE.set(true).is_ok() {
                                        debug2!("CAN_READ_TEMPERATURE set to true (SMC connection successful)");
                                    }
                                },
                                Err(e) => {
                                    debug2!("Failed to connect to SMC: {:?}", e);
                                    // Will retry on next iteration
                                }
                            }
                        }
                        
                        // CRITICAL: Create IOReport subscription for frequency reading (once, when window opens)
                        // This is expensive to create, so we keep it alive and reuse it
                        // Implementation follows exelban/stats approach: use IOReport API directly
                        if let Ok(mut sub) = IOREPORT_SUBSCRIPTION.try_lock() {
                            if sub.is_none() {
                                // Create IOReport subscription for CPU frequency channels
                                // Group: "CPU Stats", SubGroup: "CPU Core Performance States"
                                unsafe {
                                    // Create CFString objects for group and subgroup
                                    let group_cf = CFString::from_static_string("CPU Stats");
                                    let subgroup_cf = CFString::from_static_string("CPU Core Performance States");
                                    
                                    // Get channels in the CPU Performance States group
                                    let channels_dict = IOReportCopyChannelsInGroup(
                                        group_cf.as_concrete_TypeRef(),
                                        subgroup_cf.as_concrete_TypeRef(),
                                        0, // want_hierarchical
                                        0, // want_sub_groups
                                        0, // want_historical
                                    );
                                    
                                    if !channels_dict.is_null() {
                                        // CRITICAL: Retain channels_dict before storing (Create/Copy rule)
                                        CFRetain(channels_dict as CFTypeRef);
                                        // Store original channels_dict for iterating channel structure
                                        if let Ok(mut orig_channels_storage) = IOREPORT_ORIGINAL_CHANNELS.try_lock() {
                                            // Release old one if it exists
                                            if let Some(old_dict_usize) = orig_channels_storage.take() {
                                                let old_dict = old_dict_usize as CFDictionaryRef;
                                                if !old_dict.is_null() {
                                                    CFRelease(old_dict as CFTypeRef);
                                                }
                                            }
                                            *orig_channels_storage = Some(channels_dict as usize);
                                        } else {
                                            // Lock failed, release the retained dict
                                            CFRelease(channels_dict as CFTypeRef);
                                        }
                                        
                                        // Create mutable dictionary for subscription
                                        // We need to merge the channels into a mutable dictionary
                                        // For IOReport, we use CFString keys and CFType values
                                        use core_foundation::base::CFType;
                                        let channels_mut: CFMutableDictionary<CFString, CFType> = CFMutableDictionary::new();
                                        
                                        // Merge channels into our mutable dictionary
                                        IOReportMergeChannels(
                                            channels_mut.as_concrete_TypeRef(),
                                            channels_dict,
                                            std::ptr::null(),
                                        );
                                        
                                        // Create subscription
                                        // IOReportCreateSubscription returns the subscription handle as *mut c_void
                                        // and also fills in subscription_dict with channel information
                                        let mut subscription_dict: CFMutableDictionaryRef = std::ptr::null_mut();
                                        
                                        let subscription_ptr = IOReportCreateSubscription(
                                            std::ptr::null(), // allocator
                                            channels_mut.as_concrete_TypeRef(),
                                            &mut subscription_dict,
                                            0, // channel_id
                                            std::ptr::null(), // options
                                        );
                                        
                                        // The subscription handle is the return value, not the dictionary
                                        if !subscription_ptr.is_null() {
                                            *sub = Some(subscription_ptr as usize);
                                            
                                            // CRITICAL: Retain subscription_dict before storing
                                            if !subscription_dict.is_null() {
                                                CFRetain(subscription_dict as CFTypeRef);
                                                // Store subscription_dict (contains channel structure we can iterate)
                                                if let Ok(mut sub_dict_storage) = IOREPORT_SUBSCRIPTION_DICT.try_lock() {
                                                    // Release old one if it exists
                                                    if let Some(old_dict_usize) = sub_dict_storage.take() {
                                                        let old_dict = old_dict_usize as CFMutableDictionaryRef;
                                                        if !old_dict.is_null() {
                                                            CFRelease(old_dict as CFTypeRef);
                                                        }
                                                    }
                                                    *sub_dict_storage = Some(subscription_dict as usize);
                                                } else {
                                                    // Lock failed, release the retained dict
                                                    CFRelease(subscription_dict as CFTypeRef);
                                                }
                                            }
                                            
                                            // Store channels dictionary for sampling (needed for IOReportCreateSamples)
                                            // CRITICAL: Retain the dictionary to avoid use-after-free crashes
                                            CFRetain(channels_mut.as_concrete_TypeRef() as CFTypeRef);
                                            if let Ok(mut channels_storage) = IOREPORT_CHANNELS.try_lock() {
                                                // Release old one if it exists
                                                if let Some(old_ptr) = channels_storage.take() {
                                                    CFRelease(old_ptr as CFTypeRef);
                                                }
                                                *channels_storage = Some(channels_mut.as_concrete_TypeRef() as usize);
                                            } else {
                                                // Lock failed, release the retained dict
                                                CFRelease(channels_mut.as_concrete_TypeRef() as CFTypeRef);
                                            }
                                            
                                            debug2!("IOReport subscription created successfully for CPU frequency (handle={:p}, dict={:p})", subscription_ptr, subscription_dict);
                                            
                                            // OPTIMIZATION Phase 3: Update OnceLock to indicate frequency reading works
                                            // OPTIMIZATION Phase 3: Update OnceLock to indicate frequency reading works
                                            if CAN_READ_FREQUENCY.set(true).is_ok() {
                                                debug2!("CAN_READ_FREQUENCY set to true (IOReport subscription created)");
                                            }
                                        } else {
                                            debug2!("Failed to create IOReport subscription: subscription_ptr is null, subscription_dict={:p}", subscription_dict);
                                        }
                                    } else {
                                        debug2!("No CPU Performance States channels found in IOReport");
                                    }
                                }
                            }
                        }
                        
                        // CRITICAL: Create IOReport subscription for power reading (once, when window opens)
                        // This is expensive to create, so we keep it alive and reuse it
                        // Power channels are in groups like "CPU Stats" / "CPU Power" or "GPU Stats" / "GPU Power"
                        if let Ok(mut power_sub) = IOREPORT_POWER_SUBSCRIPTION.try_lock() {
                            if power_sub.is_none() {
                                // Try to find power channels - common groups:
                                // "CPU Stats" / "CPU Power" or "CPU Energy"
                                // "GPU Stats" / "GPU Power" or "GPU Energy"
                                unsafe {
                                    // Try multiple power channel combinations
                                    // Power channels vary by Mac model and macOS version
                                    // Based on research: "Energy Model" group is commonly used for power
                                    let mut power_channels_dict: CFDictionaryRef = std::ptr::null_mut();
                                    let mut found_channel_name = String::new();
                                    
                                    // Try "Energy Model" group first (common for power metrics)
                                    let energy_model_group_cf = CFString::from_static_string("Energy Model");
                                    let energy_model_dict = IOReportCopyChannelsInGroup(
                                        energy_model_group_cf.as_concrete_TypeRef(),
                                        std::ptr::null(), // NULL subgroup = all subgroups
                                        0, 0, 0,
                                    );
                                    if !energy_model_dict.is_null() {
                                        use core_foundation::dictionary::CFDictionaryGetCount;
                                        let count = CFDictionaryGetCount(energy_model_dict);
                                        if count > 0 {
                                            power_channels_dict = energy_model_dict;
                                            found_channel_name = "Energy Model (all subgroups)".to_string();
                                            debug1!("Found power channels: Energy Model ({} entries)", count);
                                        } else {
                                            CFRelease(energy_model_dict as CFTypeRef);
                                        }
                                    }
                                    
                                    // If Energy Model didn't work, try CPU Power
                                    if power_channels_dict.is_null() {
                                        let cpu_group_cf = CFString::from_static_string("CPU Stats");
                                        let cpu_power_subgroup_cf = CFString::from_static_string("CPU Power");
                                        let cpu_channels_dict = IOReportCopyChannelsInGroup(
                                            cpu_group_cf.as_concrete_TypeRef(),
                                            cpu_power_subgroup_cf.as_concrete_TypeRef(),
                                            0, 0, 0,
                                        );
                                        if !cpu_channels_dict.is_null() {
                                            power_channels_dict = cpu_channels_dict;
                                            found_channel_name = "CPU Stats / CPU Power".to_string();
                                            debug1!("Found power channels: CPU Stats / CPU Power");
                                        } else {
                                            // Try CPU Energy
                                            let cpu_energy_subgroup_cf = CFString::from_static_string("CPU Energy");
                                            let cpu_energy_channels_dict = IOReportCopyChannelsInGroup(
                                                cpu_group_cf.as_concrete_TypeRef(),
                                                cpu_energy_subgroup_cf.as_concrete_TypeRef(),
                                                0, 0, 0,
                                            );
                                            if !cpu_energy_channels_dict.is_null() {
                                                power_channels_dict = cpu_energy_channels_dict;
                                                found_channel_name = "CPU Stats / CPU Energy".to_string();
                                                debug1!("Found power channels: CPU Stats / CPU Energy");
                                            } else {
                                                // Try GPU Power
                                                let gpu_group_cf = CFString::from_static_string("GPU Stats");
                                                let gpu_power_subgroup_cf = CFString::from_static_string("GPU Power");
                                                let gpu_channels_dict = IOReportCopyChannelsInGroup(
                                                    gpu_group_cf.as_concrete_TypeRef(),
                                                    gpu_power_subgroup_cf.as_concrete_TypeRef(),
                                                    0, 0, 0,
                                                );
                                                if !gpu_channels_dict.is_null() {
                                                    power_channels_dict = gpu_channels_dict;
                                                    found_channel_name = "GPU Stats / GPU Power".to_string();
                                                    debug1!("Found power channels: GPU Stats / GPU Power");
                                                } else {
                                                    // Try GPU Energy
                                                    let gpu_energy_subgroup_cf = CFString::from_static_string("GPU Energy");
                                                    let gpu_energy_channels_dict = IOReportCopyChannelsInGroup(
                                                        gpu_group_cf.as_concrete_TypeRef(),
                                                        gpu_energy_subgroup_cf.as_concrete_TypeRef(),
                                                        0, 0, 0,
                                                    );
                                                    if !gpu_energy_channels_dict.is_null() {
                                                        power_channels_dict = gpu_energy_channels_dict;
                                                        found_channel_name = "GPU Stats / GPU Energy".to_string();
                                                        debug1!("Found power channels: GPU Stats / GPU Energy");
                                                    } else {
                                                        debug1!("No power channels found - tried: Energy Model, CPU Power, CPU Energy, GPU Power, GPU Energy");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    if !power_channels_dict.is_null() {
                                        // Check channel count before proceeding
                                        use core_foundation::dictionary::CFDictionaryGetCount;
                                        let channel_count = CFDictionaryGetCount(power_channels_dict);
                                        debug1!("Power channels dictionary has {} entries", channel_count);
                                        
                                        if channel_count == 0 {
                                            debug1!("Power channels dictionary is empty - cannot create subscription");
                                            CFRelease(power_channels_dict as CFTypeRef);
                                        } else {
                                            // CRITICAL: Extract actual channels from nested structure
                                            // IOReportCopyChannelsInGroup returns a dict with "IOReportChannels" key
                                            // containing the actual channel dictionaries
                                            use core_foundation::base::CFGetTypeID;
                                            use core_foundation::dictionary::CFDictionaryGetTypeID;
                                            use core_foundation::string::CFStringGetTypeID;
                                            
                                            let actual_channels_dict = {
                                                use core_foundation::dictionary::CFDictionaryGetCount;
                                                
                                                let keys_count = CFDictionaryGetCount(power_channels_dict);
                                                let mut keys_buf: Vec<*const c_void> = vec![std::ptr::null(); keys_count as usize];
                                                let mut values_buf: Vec<*const c_void> = vec![std::ptr::null(); keys_count as usize];
                                                
                                                extern "C" {
                                                    fn CFDictionaryGetKeysAndValues(
                                                        theDict: CFDictionaryRef,
                                                        keys: *mut *const c_void,
                                                        values: *mut *const c_void,
                                                    );
                                                }
                                                
                                                CFDictionaryGetKeysAndValues(
                                                    power_channels_dict,
                                                    keys_buf.as_mut_ptr(),
                                                    values_buf.as_mut_ptr(),
                                                );
                                                
                                                // Log all keys to understand structure
                                                for i in 0..(keys_count as usize) {
                                                    let key_ref = keys_buf[i] as CFStringRef;
                                                    if !key_ref.is_null() {
                                                        let key_type_id = CFGetTypeID(key_ref as CFTypeRef);
                                                        let string_type_id = CFStringGetTypeID();
                                                        if key_type_id == string_type_id {
                                                            let key_str = CFString::wrap_under_get_rule(key_ref);
                                                            let key_name = key_str.to_string();
                                                            debug1!("Power channels dict key[{}]: '{}'", i, key_name);
                                                            
                                                            let value_ptr = values_buf[i];
                                                            if !value_ptr.is_null() {
                                                                let value_type_id = CFGetTypeID(value_ptr as CFTypeRef);
                                                                let dict_type_id = CFDictionaryGetTypeID();
                                                                extern "C" {
                                                                    fn CFArrayGetTypeID() -> u64;
                                                                }
                                                                let array_type_id = CFArrayGetTypeID();
                                                                debug1!("  Value type_id={}, dict_type_id={}, array_type_id={}", value_type_id, dict_type_id, array_type_id);
                                                                
                                                                if value_type_id == dict_type_id {
                                                                    let nested_dict = value_ptr as CFDictionaryRef;
                                                                    let nested_count = CFDictionaryGetCount(nested_dict);
                                                                    debug1!("  Nested dict has {} entries", nested_count);
                                                                } else if value_type_id as u64 == array_type_id {
                                                                    extern "C" {
                                                                        fn CFArrayGetCount(theArray: *const c_void) -> i32;
                                                                    }
                                                                    let array_count = CFArrayGetCount(value_ptr as *const c_void);
                                                                    debug1!("  Nested array has {} entries", array_count);
                                                                    // If this is IOReportChannels array, we need to extract it
                                                                    if key_name == "IOReportChannels" && array_count > 0 {
                                                                        // For arrays, we need to process them differently
                                                                        // The array contains channel dictionaries directly
                                                                        debug1!("  Found IOReportChannels array with {} channels", array_count);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                // For Energy Model, IOReportChannels is an array, not a dict
                                                // We need to store the original dict (with IOReportChannels array) for channel name lookup
                                                // IOReportMergeChannels will handle the array structure when creating subscription
                                                debug1!("Using original power channels dict (contains IOReportChannels array)");
                                                CFRetain(power_channels_dict as CFTypeRef);
                                                power_channels_dict
                                            };
                                            
                                            // Retain and store original channels dict (the wrapper with IOReportChannels)
                                            // This is needed for channel name lookup during power reading
                                            // The actual_channels_dict is what we'll merge, but we store the wrapper for lookup
                                            CFRetain(power_channels_dict as CFTypeRef);
                                            if let Ok(mut orig_storage) = IOREPORT_POWER_ORIGINAL_CHANNELS.try_lock() {
                                                if let Some(old_dict) = orig_storage.take() {
                                                    CFRelease(old_dict as CFTypeRef);
                                                }
                                                // Store the wrapper dict (contains IOReportChannels array) for name lookup
                                                *orig_storage = Some(power_channels_dict as usize);
                                                debug1!("Stored original power channels dict in IOREPORT_POWER_ORIGINAL_CHANNELS");
                                            } else {
                                                CFRelease(power_channels_dict as CFTypeRef);
                                            }
                                            
                                            // Create mutable dictionary for subscription
                                            // CRITICAL: Try using the channels dict directly first, then merge if needed
                                            use core_foundation::base::CFType;
                                            let power_channels_mut: CFMutableDictionary<CFString, CFType> = CFMutableDictionary::new();
                                            
                                            debug1!("Merging power channels into mutable dictionary...");
                                            IOReportMergeChannels(
                                                power_channels_mut.as_concrete_TypeRef(),
                                                actual_channels_dict,
                                                std::ptr::null(),
                                            );
                                            
                                            // Release the extracted dict after merging (we've copied its contents)
                                            CFRelease(actual_channels_dict as CFTypeRef);
                                            
                                            // Check merged dictionary count
                                            use core_foundation::dictionary::CFDictionaryGetCount;
                                            let merged_count = CFDictionaryGetCount(power_channels_mut.as_concrete_TypeRef());
                                            debug1!("Merged power channels dictionary has {} entries", merged_count);
                                            
                                            // If merge resulted in 0 entries, try using the original dict directly
                                            // This might work if the structure is already in the correct format
                                            let channels_for_subscription = if merged_count == 0 {
                                                debug1!("Merge resulted in 0 entries, trying to use channels dict directly");
                                                // Retain the actual_channels_dict again since we'll use it directly
                                                CFRetain(actual_channels_dict as CFTypeRef);
                                                actual_channels_dict as CFMutableDictionaryRef
                                            } else {
                                                // Release the extracted channels dict (we've merged it)
                                                CFRelease(actual_channels_dict as CFTypeRef);
                                                power_channels_mut.as_concrete_TypeRef()
                                            };
                                            
                                            // Create subscription
                                            let mut power_subscription_dict: CFMutableDictionaryRef = std::ptr::null_mut();
                                            debug1!("Creating IOReport power subscription...");
                                            let power_subscription_ptr = IOReportCreateSubscription(
                                                std::ptr::null(),
                                                channels_for_subscription,
                                                &mut power_subscription_dict,
                                                0,
                                                std::ptr::null(),
                                            );
                                            
                                            // If we used the direct dict and subscription failed, release it
                                            if merged_count == 0 && power_subscription_ptr.is_null() {
                                                CFRelease(channels_for_subscription as CFTypeRef);
                                            }
                                            
                                            if !power_subscription_ptr.is_null() {
                                                debug1!("IOReport power subscription created successfully!");
                                            *power_sub = Some(power_subscription_ptr as usize);
                                            
                                            if !power_subscription_dict.is_null() {
                                                CFRetain(power_subscription_dict as CFTypeRef);
                                                if let Ok(mut sub_dict_storage) = IOREPORT_POWER_SUBSCRIPTION_DICT.try_lock() {
                                                    if let Some(old_dict) = sub_dict_storage.take() {
                                                        CFRelease(old_dict as CFTypeRef);
                                                    }
                                                    *sub_dict_storage = Some(power_subscription_dict as usize);
                                                } else {
                                                    CFRelease(power_subscription_dict as CFTypeRef);
                                                }
                                            }
                                            
                                            CFRetain(power_channels_mut.as_concrete_TypeRef() as CFTypeRef);
                                            if let Ok(mut channels_storage) = IOREPORT_POWER_CHANNELS.try_lock() {
                                                if let Some(old_ptr) = channels_storage.take() {
                                                    CFRelease(old_ptr as CFTypeRef);
                                                }
                                                *channels_storage = Some(power_channels_mut.as_concrete_TypeRef() as usize);
                                            } else {
                                                CFRelease(power_channels_mut.as_concrete_TypeRef() as CFTypeRef);
                                            }
                                            
                                            debug1!("IOReport power subscription created successfully (handle={:p}, channels={})", power_subscription_ptr, found_channel_name);
                                            
                                            if CAN_READ_CPU_POWER.set(true).is_ok() {
                                                debug1!("CAN_READ_CPU_POWER set to true");
                                            }
                                            if CAN_READ_GPU_POWER.set(true).is_ok() {
                                                debug1!("CAN_READ_GPU_POWER set to true");
                                            }
                                            } else {
                                                debug1!("Failed to create IOReport power subscription: subscription_ptr is null");
                                                debug1!("This may indicate the power channels require different handling or permissions");
                                                // Release the retained channels dict since subscription failed
                                                if let Ok(mut orig_storage) = IOREPORT_POWER_ORIGINAL_CHANNELS.try_lock() {
                                                    if let Some(dict) = orig_storage.take() {
                                                        CFRelease(dict as CFTypeRef);
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        debug1!("No power channels found in IOReport (tried: CPU Power, CPU Energy, GPU Power, GPU Energy)");
                                        debug1!("Power consumption will show 0.0W - power channels may not be available on this Mac model");
                                    }
                                }
                            }
                        }
                        
                        // CRITICAL: Only read temperature every 5 seconds to reduce CPU usage
                        // all_data() iteration is VERY expensive - limit it as much as possible
                        // STEP 3: Reduce temperature reading frequency from 5s to 15s to save CPU
                        // Temperature doesn't change rapidly, so 15s is still responsive
                        let should_read_temp_now = if let Ok(mut last) = LAST_TEMP_UPDATE.lock() {
                            let should = last.as_ref()
                                .map(|t| t.elapsed().as_secs() >= 20)
                                .unwrap_or(true);
                            if should {
                                *last = Some(std::time::Instant::now());
                            }
                            should
                        } else {
                            false
                        };
                        
                        // Only actually read temperature if enough time has passed
                        if should_read_temp_now {
                            // Read temperature using existing connection
                            if let Some(ref mut smc) = smc_connection {
                                // First try standard cpu_temperature() method (works for M1/M2)
                                let mut temp = 0.0;
                                match smc.cpu_temperature() {
                                    Ok(temps) => {
                                        let die_temp: f64 = temps.die.into();
                                        let prox_temp: f64 = temps.proximity.into();
                                        
                                        // Priority: die > proximity
                                        temp = if die_temp > 0.0 {
                                            die_temp
                                        } else if prox_temp > 0.0 {
                                            prox_temp
                                        } else {
                                            0.0
                                        };
                                    },
                                    Err(_) => {
                                        // Standard method failed, continue to raw key reading
                                    }
                                }
                                
                                // If standard method returned 0.0, try reading M3 Max raw keys directly
                                // These are the keys that exelban/stats uses for M3 Max
                                if temp == 0.0 {
                                    // Check if we've already discovered a working M3 key
                                    let cached_key = M3_TEMP_KEY.lock().ok().and_then(|k| k.clone());
                                    
                                    if let Some(key_name) = cached_key {
                                        // CRITICAL: Use direct key reading instead of all_data() iteration
                                        // This is MUCH more efficient - avoids iterating through all SMC keys
                                        // Try to read the specific key directly
                                        // Note: macsmc may not have direct key reading, so we'll limit all_data() usage
                                        // Only call all_data() if we absolutely need to, and limit iteration
                                        if let Ok(data_iter) = smc.all_data() {
                                            // CRITICAL: Break early once we find our key - don't iterate all keys
                                            for dbg_result in data_iter {
                                                if let Ok(dbg) = dbg_result {
                                                    if dbg.key == key_name {
                                                        if let Ok(Some(macsmc::DataValue::Float(val))) = dbg.value {
                                                            if val > 0.0 {
                                                                temp = val as f64;
                                                                debug3!("Temperature read from cached M3 key {}: {:.1}°C", key_name, temp);
                                                                break; // Early exit
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // First time: discover which M3 key works
                                        // CRITICAL: Only iterate through keys once, then cache the result
                                        // Try known M3 Max temperature keys (same as exelban/stats uses)
                                        let m3_keys = ["Tf04", "Tf09", "Tf0A", "Tf0B", "Tf0D", "Tf0E"];
                                        if let Ok(data_iter) = smc.all_data() {
                                            // CRITICAL: Break early once we find a working key
                                            for dbg_result in data_iter {
                                                if let Ok(dbg) = dbg_result {
                                                    // Check if this is one of our target M3 keys
                                                    if m3_keys.contains(&dbg.key.as_str()) {
                                                        if let Ok(Some(macsmc::DataValue::Float(val))) = dbg.value {
                                                            if val > 0.0 {
                                                                temp = val as f64;
                                                                // Cache this key for future use
                                                                if let Ok(mut cached) = M3_TEMP_KEY.lock() {
                                                                    *cached = Some(dbg.key.clone());
                                                                    debug2!("Discovered working M3 temperature key: {} = {:.1}°C", dbg.key, temp);
                                                                }
                                                                break; // Early exit - use first valid temperature found
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if temp > 0.0 {
                                    // Update cache with new temperature and timestamp
                                    if let Ok(mut cache) = TEMP_CACHE.try_lock() {
                                        *cache = Some((temp as f32, std::time::Instant::now()));
                                        debug2!("Temperature updated in cache: {:.1}°C", temp);
                                    } else {
                                        debug2!("Temperature cache lock failed, skipping update");
                                    }
                                } else {
                                    debug3!("Temperature read returned 0.0 - no valid temperature found");
                                    // Don't update cache - keep previous value if available
                                }
                            }
                        } else {
                            // Skip temperature reading entirely - too soon since last read
                            debug3!("Skipping temperature read (too soon since last read, all_data() is expensive)");
                            // Don't call all_data() at all - just skip
                        }
                        
                        // STEP 3: Read CPU frequency from IOReport (real-time, dynamic)
                        // This is the same approach exelban/stats uses - efficient native API
                        // CPU EFFICIENCY: Only read frequency every 20 seconds (IOReport sampling still has overhead)
                        // Increased from 10s to 20s to save CPU - frequency doesn't change that rapidly
                        let should_read_freq = if let Ok(mut last) = LAST_FREQ_READ.lock() {
                            debug2!("========> LAST_FREQ_READ: {:?}", last);
                            let should = last.as_ref()
                                .map(|t| t.elapsed().as_secs() >= 30)
                                .unwrap_or(true);
                            if should {
                                *last = Some(std::time::Instant::now());
                            }
                            should
                        } else {
                            false
                        };
                        
                        if should_read_freq {
                            debug3!("should_read_freq=true, attempting IOReport frequency read");
                            
                            // Check if frequency logging is enabled
                            let freq_logging = state::FREQUENCY_LOGGING_ENABLED.lock()
                                .map(|f| *f)
                                .unwrap_or(false);
                            
                            let mut freq: f32 = 0.0;
                            let mut p_core_freq: f32 = 0.0;
                            let mut e_core_freq: f32 = 0.0;
                            
                            // Try IOReport first (real-time frequency via native API)
                            let freq_result = if let Ok(sub) = IOREPORT_SUBSCRIPTION.try_lock() {
                                if let Some(subscription_usize) = sub.as_ref() {
                                    let subscription_ptr = *subscription_usize as *mut c_void;
                                    
                                    if subscription_ptr.is_null() {
                                        debug3!("Subscription pointer is null, cannot create sample");
                                        None
                                    } else {
                                        // Get channels dictionary for sampling
                                        let channels_ptr = if let Ok(channels_storage) = IOREPORT_CHANNELS.try_lock() {
                                            channels_storage.as_ref().map(|&usize_ptr| usize_ptr as CFMutableDictionaryRef)
                                        } else {
                                            None
                                        };
                                        
                                        let channels_ref = channels_ptr.unwrap_or(std::ptr::null_mut());
                                        if channels_ref.is_null() {
                                            debug3!("Using NULL channels for IOReportCreateSamples (may fail)");
                                        } else {
                                            debug3!("Using stored channels dictionary for IOReportCreateSamples");
                                        }
                                        
                                        // Get original channels dictionary
                                        let original_channels_dict = if let Ok(orig_channels_storage) = IOREPORT_ORIGINAL_CHANNELS.try_lock() {
                                            orig_channels_storage.as_ref().map(|&dict_usize| dict_usize as CFDictionaryRef)
                                        } else {
                                            None
                                        };
                                        
                                        // Get last sample for delta calculation
                                        let last_sample = if let Ok(last_sample_storage) = LAST_IOREPORT_SAMPLE.try_lock() {
                                            last_sample_storage.as_ref().map(|&(sample_usize, _)| sample_usize as CFDictionaryRef)
                                        } else {
                                            None
                                        };
                                        
                                        // Use the extracted frequency reading function
                                        unsafe {
                                            use ffi::ioreport::read_frequencies_from_ioreport;
                                            
                                            let (result, current_sample_opt) = read_frequencies_from_ioreport(
                                                subscription_ptr as *const c_void,
                                                channels_ref,
                                                original_channels_dict,
                                                last_sample,
                                                freq_logging,
                                            );
                                            
                                            // Store current sample for next delta calculation
                                            if let Some(current_sample) = current_sample_opt {
                                                // Retain the sample before storing (Core Foundation ownership rule)
                                                let retained_sample = CFRetain(current_sample as CFTypeRef) as CFDictionaryRef;
                                                if let Ok(mut last_sample_storage) = LAST_IOREPORT_SAMPLE.try_lock() {
                                                    // Release old sample if it exists
                                                    if let Some((old_sample_usize, _)) = last_sample_storage.take() {
                                                        let old_sample = old_sample_usize as CFDictionaryRef;
                                                        if !old_sample.is_null() {
                                                            CFRelease(old_sample as CFTypeRef);
                                                        }
                                                    }
                                                    // Store retained sample
                                                    *last_sample_storage = Some((retained_sample as usize, std::time::Instant::now()));
                                                } else {
                                                    // Lock failed, release the retained sample
                                                    CFRelease(retained_sample as CFTypeRef);
                                                }
                                                // Release the original sample (we've retained a copy)
                                                CFRelease(current_sample as CFTypeRef);
                                            }
                                            
                                            Some(result)
                                        }
                                    }
                                } else {
                                    debug3!("IOReport subscription not available");
                                    None
                                }
                            } else {
                                debug3!("IOReport subscription lock failed");
                                None
                            };
                            
                            // Update frequency values from result
                            if let Some(freq_result) = freq_result {
                                freq = freq_result.overall;
                                p_core_freq = freq_result.p_core;
                                e_core_freq = freq_result.e_core;
                            }
                            
                            // CRITICAL: Only use nominal frequency as fallback if IOReport completely failed
                            // If IOReport returned 0.0, it means parsing failed - don't overwrite cache with nominal
                            // Only update cache if we got a real frequency from IOReport
                            let freq_logging = state::FREQUENCY_LOGGING_ENABLED.lock()
                                .map(|f| *f)
                                .unwrap_or(false);
                            
                            if freq > 0.0 {
                                if let Ok(mut cache) = FREQ_CACHE.try_lock() {
                                    *cache = Some((freq, std::time::Instant::now()));
                                    if freq_logging {
                                        debug1!("Overall frequency cache updated: {:.2} GHz", freq);
                                    } else {
                                        debug2!("Frequency cache updated from IOReport: {:.2} GHz", freq);
                                    }
                                }
                                
                                // Update P-core frequency cache
                                if p_core_freq > 0.0 {
                                    if let Ok(mut cache) = P_CORE_FREQ_CACHE.try_lock() {
                                        *cache = Some((p_core_freq, std::time::Instant::now()));
                                        if freq_logging {
                                            debug1!("P-core frequency cache updated: {:.2} GHz", p_core_freq);
                                        } else {
                                            debug2!("P-core frequency cache updated: {:.2} GHz", p_core_freq);
                                        }
                                    }
                                } else {
                                    if freq_logging {
                                        debug1!("P-core frequency is 0.0 - NOT updating cache");
                                    }
                                }
                                
                                // Update E-core frequency cache
                                if e_core_freq > 0.0 {
                                    if let Ok(mut cache) = E_CORE_FREQ_CACHE.try_lock() {
                                        *cache = Some((e_core_freq, std::time::Instant::now()));
                                        if freq_logging {
                                            debug1!("E-core frequency cache updated: {:.2} GHz", e_core_freq);
                                        } else {
                                            debug2!("E-core frequency cache updated: {:.2} GHz", e_core_freq);
                                        }
                                    }
                                } else {
                                    if freq_logging {
                                        debug1!("E-core frequency is 0.0 - NOT updating cache");
                                    }
                                }
                                
                                // OPTIMIZATION Phase 3: Update OnceLock to indicate frequency reading works
                                if CAN_READ_FREQUENCY.set(true).is_ok() {
                                    debug2!("CAN_READ_FREQUENCY set to true (IOReport frequency read successfully)");
                                }
                            } else {
                                // This prevents overwriting a good cached value with nominal frequency
                                debug2!("IOReport frequency parsing failed (freq=0.0) - keeping existing cache value if available");
                                
                                // Only initialize cache with nominal frequency if it's completely empty
                                if let Ok(mut cache) = FREQ_CACHE.try_lock() {
                                    if cache.is_none() {
                                        let nominal = metrics::get_nominal_frequency();
                                        *cache = Some((nominal, std::time::Instant::now()));
                                        debug2!("Using nominal frequency as initial value: {:.2} GHz (IOReport not available yet)", nominal);
                                    } else {
                                        debug3!("Keeping existing cached frequency value (IOReport parsing failed)");
                                    }
                                }
                            }
                        } else {
                            debug2!("should_read_freq=false, skipping frequency update");
                        }
                        
                        // CRITICAL: Only read battery and power when CPU window is visible
                        // This ensures menu bar (which only shows CPU/RAM/Disk) remains super lightweight
                        // Battery reading via IOKit is lightweight, but we still only read when window is visible
                        // Battery state can change (charging/discharging), so we read frequently when visible
                        let (battery_level, is_charging, has_battery) = metrics::get_battery_info();
                        let power_logging = state::POWER_USAGE_LOGGING_ENABLED.lock()
                            .map(|f| *f)
                            .unwrap_or(false);
                        if power_logging && has_battery {
                            debug1!("Battery updated: {:.1}%, charging={}", battery_level, is_charging);
                        }
                        
                        // Read power consumption from IOReport
                        // Power reading is expensive (IOReport), so we read it every 5 seconds
                        let should_read_power = if let Ok(mut last) = LAST_POWER_READ_TIME.lock() {
                            let should = last.as_ref()
                                .map(|t| t.elapsed().as_secs() >= 5)
                                .unwrap_or(true);
                            if should {
                                *last = Some(std::time::Instant::now());
                            }
                            should
                        } else {
                            false
                        };
                        
                        if should_read_power {
                            debug1!("Reading power from IOReport (should_read_power=true)...");
                            // Read power from IOReport
                            let power_result = if let Ok(power_sub) = IOREPORT_POWER_SUBSCRIPTION.try_lock() {
                                debug1!("Power subscription lock acquired");
                                if let Some(subscription_usize) = power_sub.as_ref() {
                                    let subscription_ptr = *subscription_usize as *mut c_void;
                                    debug1!("Power subscription found: {:p}", subscription_ptr);
                                    
                                    if !subscription_ptr.is_null() {
                                        debug1!("Subscription pointer is valid, proceeding with power read...");
                                        let channels_ptr = if let Ok(channels_storage) = IOREPORT_POWER_CHANNELS.try_lock() {
                                            channels_storage.as_ref().map(|&usize_ptr| usize_ptr as CFMutableDictionaryRef)
                                        } else {
                                            None
                                        };
                                        
                                        let channels_ref = channels_ptr.unwrap_or(std::ptr::null_mut());
                                        
                                        let original_channels_dict = if let Ok(orig_storage) = IOREPORT_POWER_ORIGINAL_CHANNELS.try_lock() {
                                            orig_storage.as_ref().map(|&dict_usize| dict_usize as CFDictionaryRef)
                                        } else {
                                            None
                                        };
                                        
                                        debug1!("Power reading: original_channels_dict.is_some()={}, channels_ref.is_null()={}", 
                                            original_channels_dict.is_some(), channels_ref.is_null());
                                        
                                        let last_sample = if let Ok(last_sample_storage) = LAST_IOREPORT_POWER_SAMPLE.try_lock() {
                                            last_sample_storage.as_ref().map(|&(sample_usize, _)| sample_usize as CFDictionaryRef)
                                        } else {
                                            None
                                        };
                                        
                                        let last_read_time = LAST_POWER_READ_TIME.lock()
                                            .ok()
                                            .and_then(|t| *t);
                                        
                                        unsafe {
                                            use ffi::ioreport::read_power_from_ioreport;
                                            
                                            debug1!("Calling read_power_from_ioreport...");
                                            let (result, current_sample_opt) = read_power_from_ioreport(
                                                subscription_ptr as *const c_void,
                                                channels_ref,
                                                original_channels_dict,
                                                last_sample,
                                                last_read_time,
                                                power_logging,
                                            );
                                            debug1!("read_power_from_ioreport returned: CPU={:.2}W, GPU={:.2}W", result.cpu_power, result.gpu_power);
                                            
                                            // Store current sample for next delta calculation
                                            if let Some(current_sample) = current_sample_opt {
                                                let retained_sample = CFRetain(current_sample as CFTypeRef) as CFDictionaryRef;
                                                if let Ok(mut last_sample_storage) = LAST_IOREPORT_POWER_SAMPLE.try_lock() {
                                                    if let Some((old_sample_usize, _)) = last_sample_storage.take() {
                                                        let old_sample = old_sample_usize as CFDictionaryRef;
                                                        if !old_sample.is_null() {
                                                            CFRelease(old_sample as CFTypeRef);
                                                        }
                                                    }
                                                    *last_sample_storage = Some((retained_sample as usize, std::time::Instant::now()));
                                                } else {
                                                    CFRelease(retained_sample as CFTypeRef);
                                                }
                                                CFRelease(current_sample as CFTypeRef);
                                            }
                                            
                                            Some(result)
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    debug1!("Power subscription is None");
                                    None
                                }
                            } else {
                                debug1!("Power subscription lock failed");
                                None
                            };
                            
                            if let Some(power_data) = power_result {
                                // Update cache
                                if let Ok(mut cache) = POWER_CACHE.try_lock() {
                                    *cache = Some((power_data.cpu_power, power_data.gpu_power, std::time::Instant::now()));
                                    debug1!("Power cache updated: CPU={:.2}W, GPU={:.2}W", power_data.cpu_power, power_data.gpu_power);
                                }
                            } else {
                                debug1!("Power reading returned None - subscription may not be available");
                            }
                        }
                        
                        // Get cached power values for logging
                        let (cpu_power, gpu_power) = POWER_CACHE.try_lock()
                            .ok()
                            .and_then(|c| c.as_ref().map(|(cpu, gpu, _)| (*cpu, *gpu)))
                            .unwrap_or((0.0, 0.0));
                        
                        if power_logging && (cpu_power > 0.0 || gpu_power > 0.0) {
                            debug1!("Power: CPU={:.2}W, GPU={:.2}W", cpu_power, gpu_power);
                        }
                    } else {
                        // CPU window is not visible - DO NOT read battery or power to save CPU
                        // Menu bar only needs CPU/RAM/Disk which are already lightweight
                        debug3!("CPU window closed - skipping battery and power reads to save CPU");
                        // CPU window is not visible - clear SMC connection and IOReport subscription to save resources
                        if smc_connection.is_some() {
                            smc_connection = None;
                            debug3!("CPU window closed, SMC connection released");
                        }
                        
                        // CRITICAL: Clear IOReport subscriptions when window closes to save CPU
                        // Note: IOReport doesn't have an explicit destroy function in the API
                        // The subscription will be cleaned up when the process exits
                        // For now, just clear the reference
                        if let Ok(mut sub) = IOREPORT_SUBSCRIPTION.try_lock() {
                            if sub.is_some() {
                                *sub = None;
                                debug2!("CPU window closed, IOReport frequency subscription cleared");
                                
                                // Clear channels dictionary
                                if let Ok(mut channels_storage) = IOREPORT_CHANNELS.try_lock() {
                                    if let Some(ptr) = *channels_storage {
                                        unsafe {
                                            CFRelease(ptr as CFTypeRef);
                                        }
                                    }
                                    *channels_storage = None;
                                }
                                
                                // Clear last sample
                                if let Ok(mut last_sample) = LAST_IOREPORT_SAMPLE.try_lock() {
                                    *last_sample = None;
                                }
                            }
                        }
                        
                        // Clear power subscription
                        if let Ok(mut power_sub) = IOREPORT_POWER_SUBSCRIPTION.try_lock() {
                            if power_sub.is_some() {
                                *power_sub = None;
                                debug2!("CPU window closed, IOReport power subscription cleared");
                                
                                // Clear power channels dictionary
                                if let Ok(mut channels_storage) = IOREPORT_POWER_CHANNELS.try_lock() {
                                    if let Some(ptr) = *channels_storage {
                                        unsafe {
                                            CFRelease(ptr as CFTypeRef);
                                        }
                                    }
                                    *channels_storage = None;
                                }
                                
                                // Clear last power sample
                                if let Ok(mut last_sample) = LAST_IOREPORT_POWER_SAMPLE.try_lock() {
                                    if let Some((sample_usize, _)) = last_sample.take() {
                                        let sample = sample_usize as CFDictionaryRef;
                                        if !sample.is_null() {
                                            unsafe {
                                                CFRelease(sample as CFTypeRef);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // NOTE: Automatic menu bar updates are not implemented because:
                    // - run_on_main_thread callbacks don't execute (Tauri limitation)
                    // - performSelector doesn't fire reliably  
                    // Menu bar will update when user clicks on it (click handler works)
                    // Updates are stored in MENU_BAR_TEXT and processed on click
                    
                    // Update menu bar every 2 seconds to reduce CPU usage
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
