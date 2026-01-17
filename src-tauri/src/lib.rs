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

// IOReport helper functions removed - IOReport operations were too expensive for real-time monitoring
// If needed in the future, these can be re-implemented with proper caching
use objc2::MainThreadMarker;
use tauri::Manager;

// Use write_structured_log from logging module
use logging::write_structured_log;

// Use state from state module
use state::*;

// Use metrics from metrics module (only re-export what's needed)

// Re-export for Tauri commands
pub use metrics::{SystemMetrics, CpuDetails, get_cpu_details, get_metrics};

// UI functions are now in ui module
use ui::status_bar::{build_status_text, setup_status_item, create_cpu_window, make_attributed_title};

// Old metrics functions removed - now in metrics module

// Old metrics function implementations removed - now in metrics module
// Removing duplicate make_attributed_title - keeping the one below

// Old function implementations removed - see metrics module and correct implementations below

// All old metrics function implementations removed - they are now in the metrics module

// All old metrics function implementations removed - they are now in the metrics module

// Wrong make_attributed_title function removed - correct one is below

// All old metrics function implementations removed - they are now in the metrics module

// All old metrics function implementations removed - they are now in the metrics module

// Wrong make_attributed_title function removed - correct one is below

// All old metrics function implementations removed - they are now in the metrics module

// All old metrics function implementations removed - they are now in the metrics module

// Wrong make_attributed_title and all old metrics functions removed - correct implementations are below

// All old metrics function implementations removed - they are now in the metrics module

// All old metrics function implementations removed - they are now in the metrics module

// All old metrics function implementations removed - they are now in the metrics module

// Wrong make_attributed_title function removed - correct one is below

// All old metrics function implementations removed - they are now in the metrics module

// All old metrics function implementations removed - they are now in the metrics module
// All UI functions removed - now in ui module

// UI functions are imported from ui module (see imports at top)

pub fn run_with_cpu_window() {
    debug1!("Running with -cpu flag: will open CPU window after setup");
    run_internal(true)
}

pub fn run() {
    run_internal(false)
}

fn run_internal(open_cpu_window: bool) {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_cpu_details])
        .setup(move |app| {
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
                    let text = build_status_text(&metrics);
                    debug2!("Update loop: status text: '{}'", text);
                    
                    // Store update in static variable
                    if let Ok(mut pending) = MENU_BAR_TEXT.lock() {
                        *pending = Some(text);
                        debug3!("Menu bar update stored");
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
                                    // CRITICAL: Update ACCESS_CACHE to indicate SMC works
                                    // This ensures can_read_temperature() returns true
                                    if let Ok(mut cache) = ACCESS_CACHE.try_lock() {
                                        if let Some((_, freq, cpu_power, gpu_power)) = cache.as_ref() {
                                            *cache = Some((true, *freq, *cpu_power, *gpu_power));
                                        } else {
                                            *cache = Some((true, false, false, false));
                                        }
                                        debug2!("ACCESS_CACHE updated: can_read_temperature=true (SMC connection successful)");
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
                                        // Store original channels_dict for iterating channel structure
                                        if let Ok(mut orig_channels_storage) = IOREPORT_ORIGINAL_CHANNELS.try_lock() {
                                            *orig_channels_storage = Some(channels_dict as usize);
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
                                            
                                            // Store subscription_dict (contains channel structure we can iterate)
                                            if let Ok(mut sub_dict_storage) = IOREPORT_SUBSCRIPTION_DICT.try_lock() {
                                                *sub_dict_storage = Some(subscription_dict as usize);
                                            }
                                            
                                            // Store channels dictionary for sampling (needed for IOReportCreateSamples)
                                            if let Ok(mut channels_storage) = IOREPORT_CHANNELS.try_lock() {
                                                // Store the channels_mut dictionary pointer
                                                *channels_storage = Some(channels_mut.as_concrete_TypeRef() as usize);
                                            }
                                            
                                            debug2!("IOReport subscription created successfully for CPU frequency (handle={:p}, dict={:p})", subscription_ptr, subscription_dict);
                                            
                                            // Update ACCESS_CACHE to indicate frequency reading works
                                            if let Ok(mut cache) = ACCESS_CACHE.try_lock() {
                                                if let Some((temp, _, cpu_power, gpu_power)) = cache.as_ref() {
                                                    *cache = Some((*temp, true, *cpu_power, *gpu_power));
                                                } else {
                                                    *cache = Some((false, true, false, false));
                                                }
                                                debug2!("ACCESS_CACHE updated: can_read_frequency=true (IOReport subscription created)");
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
                        
                        // CRITICAL: Only read temperature every 5 seconds to reduce CPU usage
                        // all_data() iteration is VERY expensive - limit it as much as possible
                        // STEP 3: Reduce temperature reading frequency from 5s to 15s to save CPU
                        // Temperature doesn't change rapidly, so 15s is still responsive
                        let should_read_temp_now = if let Ok(mut last) = LAST_TEMP_UPDATE.lock() {
                            let should = last.as_ref()
                                .map(|t| t.elapsed().as_secs() >= 15)
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
                        
                        if should_read_freq {
                            debug3!("should_read_freq=true, attempting IOReport frequency read");
                            let mut freq: f32 = 0.0;
                            let mut p_core_freq: f32 = 0.0;
                            let mut e_core_freq: f32 = 0.0;
                            
                            // Try IOReport first (real-time frequency via native API)
                            if let Ok(sub) = IOREPORT_SUBSCRIPTION.try_lock() {
                                if let Some(subscription_usize) = sub.as_ref() {
                                    let subscription_ptr = *subscription_usize as *mut c_void;
                                    
                                    if !subscription_ptr.is_null() {
                                        unsafe {
                                            // Get channels dictionary for sampling
                                            // We need to use the channels dictionary that was used to create the subscription
                                            let channels_ptr = if let Ok(channels_storage) = IOREPORT_CHANNELS.try_lock() {
                                                channels_storage.as_ref().map(|&usize_ptr| usize_ptr as CFMutableDictionaryRef)
                                            } else {
                                                None
                                            };
                                            
                                            // Create sample from subscription
                                            // CRITICAL: IOReportCreateSamples requires the channels dictionary
                                            // Use stored channels dictionary (the one used to create the subscription)
                                            let channels_ref = channels_ptr.unwrap_or(std::ptr::null_mut());
                                            let sample = IOReportCreateSamples(
                                                subscription_ptr as *const c_void,
                                                channels_ref, // Use stored channels dictionary
                                                std::ptr::null(), // options
                                            );
                                            
                                            if channels_ref.is_null() {
                                                debug3!("Using NULL channels for IOReportCreateSamples (may fail)");
                                            } else {
                                                debug3!("Using stored channels dictionary for IOReportCreateSamples");
                                            }
                                            
                                            // #region agent log
                                            write_structured_log(
                                                "lib.rs:1843",
                                                "IOReportCreateSamples returned",
                                                &serde_json::json!({
                                                    "sample_ptr": format!("{:p}", sample),
                                                    "channels_ptr": format!("{:p}", channels_ref)
                                                }),
                                                "A",
                                            );
                                            // #endregion
                                            
                                            if !sample.is_null() {
                                                // Store sample for potential future delta calculation
                                                if let Ok(mut last_sample) = LAST_IOREPORT_SAMPLE.try_lock() {
                                                    *last_sample = Some((sample as usize, std::time::Instant::now()));
                                                }
                                                
                                                // CRITICAL: Use original channels_dict to iterate channels and extract frequency
                                                // The sample structure is complex - we'll use the original channels_dict
                                                // which contains the actual channel dictionaries we can safely query
                                                let original_channels_dict = if let Ok(orig_channels_storage) = IOREPORT_ORIGINAL_CHANNELS.try_lock() {
                                                    orig_channels_storage.as_ref().map(|&dict_usize| dict_usize as CFDictionaryRef)
                                                } else {
                                                    None
                                                };
                                                
                                                if let Some(orig_channels) = original_channels_dict {
                                                    use core_foundation::string::CFString;
                                                    
                                                    // Declare FFI functions for CFDictionary iteration
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDictionaryGetCount(theDict: CFDictionaryRef) -> i32;
    fn CFDictionaryGetKeysAndValues(
        theDict: CFDictionaryRef,
        keys: *mut *const c_void,
        values: *mut *const c_void,
    );
    fn CFGetTypeID(cf: CFTypeRef) -> u64;
    fn CFDictionaryGetTypeID() -> u64;
    fn CFArrayGetTypeID() -> u64;
    fn CFStringGetTypeID() -> u64;
}
                                                    
                                                    // Note: unsafe block is nested inside outer unsafe block, but kept for clarity
                                                    {
                                                        // Get count of channels in original channels_dict
                                                        let channels_count = CFDictionaryGetCount(orig_channels);
                                                        debug3!("Original channels_dict has {} channels", channels_count);
                                                        
                                                        if channels_count > 0 {
                                                            // Allocate buffers for keys and values from original channels_dict
                                                            let mut channel_keys_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count as usize];
                                                            let mut channel_values_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count as usize];
                                                            
                                                            // Get all channel keys and values from original channels_dict
                                                            CFDictionaryGetKeysAndValues(
                                                                orig_channels,
                                                                channel_keys_buf.as_mut_ptr(),
                                                                channel_values_buf.as_mut_ptr(),
                                                            );
                                                            
                                                            // Log channel keys to understand structure
                                                            for i in 0..(channels_count as usize) {
                                                                let channel_key_ref = channel_keys_buf[i] as CFStringRef;
                                                                if !channel_key_ref.is_null() {
                                                                    let channel_key_str = CFString::wrap_under_get_rule(channel_key_ref);
                                                                    debug3!("Entry {}: key='{}', value_ptr={:p}", i, channel_key_str.to_string(), channel_values_buf[i]);
                                                                }
                                                            }
                                                            
                                                            // CRITICAL: Find "IOReportChannels" key - this contains the actual channel array/dictionary
                                                            // PARANOID MODE: Add type checking and null guards
                                                            let mut actual_channels_ref: CFDictionaryRef = std::ptr::null_mut();
                                                            for i in 0..(channels_count as usize) {
                                                                let channel_key_ref = channel_keys_buf[i] as CFStringRef;
                                                                if channel_key_ref.is_null() {
                                                                    debug3!("Entry {}: key is null, skipping", i);
                                                                    continue;
                                                                }
                                                                
                                                                // PARANOID: Verify key is actually a CFString
                                                                let key_type_id = CFGetTypeID(channel_key_ref as CFTypeRef);
                                                                let string_type_id = CFStringGetTypeID();
                                                                if key_type_id != string_type_id {
                                                                    debug3!("Entry {}: key is not CFString (type_id={}, expected={}), skipping", i, key_type_id, string_type_id);
                                                                    continue;
                                                                }
                                                                
                                                                let channel_key_str = CFString::wrap_under_get_rule(channel_key_ref);
                                                                let key_str = channel_key_str.to_string();
                                                                debug3!("Entry {}: key='{}'", i, key_str);
                                                                
                                                                if key_str == "IOReportChannels" {
                                                                    let value_ptr = channel_values_buf[i];
                                                                    if value_ptr.is_null() {
                                                                        debug3!("Entry {}: IOReportChannels value is null!", i);
                                                                        continue;
                                                                    }
                                                                    
                                                                    // PARANOID: Verify value type before casting
                                                                    let value_type_id = CFGetTypeID(value_ptr as CFTypeRef);
                                                                    let dict_type_id = CFDictionaryGetTypeID();
                                                                    let array_type_id = CFArrayGetTypeID();
                                                                    // #region agent log
                                                                    write_structured_log(
                                                                        "lib.rs:1946",
                                                                        "IOReportChannels value type check",
                                                                        &serde_json::json!({
                                                                            "value_ptr": format!("{:p}", value_ptr),
                                                                            "value_type_id": value_type_id,
                                                                            "dict_type_id": dict_type_id,
                                                                            "array_type_id": array_type_id
                                                                        }),
                                                                        "A",
                                                                    );
                                                                    // #endregion
                                                                    
                                                                    debug3!("Entry {}: IOReportChannels value type_id={}, dict_type_id={}, array_type_id={}", 
                                                                        i, value_type_id, dict_type_id, array_type_id);
                                                                    
                                                                    if value_type_id == dict_type_id {
                                                                        actual_channels_ref = value_ptr as CFDictionaryRef;
                                                                        debug3!("Found IOReportChannels entry (CFDictionary), value_ptr={:p}", actual_channels_ref);
                                                                        break;
                                                                    } else if value_type_id == array_type_id {
                                                                        debug3!("IOReportChannels is CFArray (not CFDictionary), cannot iterate as dictionary");
                                                                        // TODO: Handle CFArray case if needed
                                                                        continue;
                                                                    } else {
                                                                        debug3!("IOReportChannels value is neither CFDictionary nor CFArray (type_id={}), skipping", value_type_id);
                                                                        continue;
                                                                    }
                                                                }
                                                            }
                                                            
                                                            if actual_channels_ref.is_null() {
                                                                debug3!("IOReportChannels key not found in channels_dict, cannot parse frequency");
                                                            } else {
                                                                // IOReportChannels is a dictionary/array of actual channel dictionaries
                                                                let actual_channels_count = CFDictionaryGetCount(actual_channels_ref);
                                                                debug3!("IOReportChannels contains {} actual channels", actual_channels_count);
                                                                
                                                                if actual_channels_count > 0 {
                                                                    // PARANOID: Verify actual_channels_ref is still valid
                                                                    if actual_channels_ref.is_null() {
                                                                        debug3!("actual_channels_ref is null before CFDictionaryGetKeysAndValues!");
                                                                    } else {
                                                                        // Allocate buffers for actual channel keys and values
                                                                        let mut actual_channel_keys: Vec<*const c_void> = vec![std::ptr::null(); actual_channels_count as usize];
                                                                        let mut actual_channel_values: Vec<*const c_void> = vec![std::ptr::null(); actual_channels_count as usize];
                                                                        
                                                                        debug3!("About to call CFDictionaryGetKeysAndValues on {:p}", actual_channels_ref);
                                                                        CFDictionaryGetKeysAndValues(
                                                                            actual_channels_ref,
                                                                            actual_channel_keys.as_mut_ptr(),
                                                                            actual_channel_values.as_mut_ptr(),
                                                                        );
                                                                        debug3!("CFDictionaryGetKeysAndValues completed successfully");
                                                                        // #region agent log
                                                                        write_structured_log(
                                                                            "lib.rs:1996",
                                                                            "IOReportChannels keys/values loaded",
                                                                            &serde_json::json!({
                                                                                "channels_count": actual_channels_count,
                                                                                "channels_ptr": format!("{:p}", actual_channels_ref)
                                                                            }),
                                                                            "B",
                                                                        );
                                                                        // #endregion
                                                                        
                                                                        // PARANOID: Log actual channel keys with type checking
                                                                        for i in 0..(actual_channels_count as usize) {
                                                                            let actual_key_ptr = actual_channel_keys[i];
                                                                            if actual_key_ptr.is_null() {
                                                                                debug3!("Actual channel {}: key is null", i);
                                                                                continue;
                                                                            }
                                                                            
                                                                            // PARANOID: Verify key type
                                                                            let key_type_id = CFGetTypeID(actual_key_ptr as CFTypeRef);
                                                                            let string_type_id = CFStringGetTypeID();
                                                                            if key_type_id != string_type_id {
                                                                                debug3!("Actual channel {}: key is not CFString (type_id={}), skipping", i, key_type_id);
                                                                                continue;
                                                                            }
                                                                            
                                                                            let actual_key_ref = actual_key_ptr as CFStringRef;
                                                                            let actual_key_str = CFString::wrap_under_get_rule(actual_key_ref);
                                                                            let value_ptr = actual_channel_values[i];
                                                                            debug3!("Actual channel {}: key='{}', value_ptr={:p}", i, actual_key_str.to_string(), value_ptr);
                                                                            
                                                                            // PARANOID: Check value type
                                                                            if !value_ptr.is_null() {
                                                                                let value_type_id = CFGetTypeID(value_ptr as CFTypeRef);
                                                                                debug3!("Actual channel {}: value type_id={}", i, value_type_id);
                                                                            }
                                                                        }
                                                                        
                                                                        // Look for channels with frequency information
                                                                        // Track P-core and E-core frequencies separately
                                                                        let mut max_freq_mhz: f64 = 0.0;
                                                                        let mut total_residency: f64 = 0.0;
                                                                        let mut weighted_freq_sum: f64 = 0.0;
                                                                        let mut p_core_max_freq_mhz: f64 = 0.0;
                                                                        let mut p_core_total_residency: f64 = 0.0;
                                                                        let mut p_core_weighted_freq_sum: f64 = 0.0;
                                                                        let mut e_core_max_freq_mhz: f64 = 0.0;
                                                                        let mut e_core_total_residency: f64 = 0.0;
                                                                        let mut e_core_weighted_freq_sum: f64 = 0.0;
                                                                        
                                                                        // Iterate through actual channels
                                                                        debug3!("Iterating through {} actual channels to find performance states", actual_channels_count);
                                                                        for i in 0..(actual_channels_count as usize) {
                                                                            let mut channel_ref_to_use: CFDictionaryRef = actual_channel_values[i] as CFDictionaryRef;
                                                                            debug3!("Entry {}: value_ref = {:p}", i, channel_ref_to_use);
                                                                            if channel_ref_to_use.is_null() {
                                                                                debug3!("Entry {} value is null, skipping", i);
                                                                                continue;
                                                                            }
                                                                            
                                                                            // CRITICAL: IOReportChannelGetChannelName can crash if called on invalid channel references
                                                                            // The values in IOReportChannels are likely channel IDs/keys, not channel dictionaries
                                                                            // We need to look up the actual channel dictionaries from the original channels_dict
                                                                            
                                                                            // Get the channel key from IOReportChannels (this is the channel ID/name)
                                                                            let actual_key_ref = actual_channel_keys[i] as CFStringRef;
                                                                            if actual_key_ref.is_null() {
                                                                                debug3!("Entry {}: key is null, skipping", i);
                                                                                continue;
                                                                            }
                                                                            
                                                                            let channel_key_str = CFString::wrap_under_get_rule(actual_key_ref);
                                                                            let channel_key = channel_key_str.to_string();
                                                                            debug3!("Entry {}: channel_key='{}'", i, channel_key);
                                                                            
                                                                            // Look up the actual channel dictionary from the original channels_dict using the channel_key
                                                                            // The original channels_dict contains the actual channel dictionaries we can safely query
                                                                            let mut found_channel = false;
                                                                            let mut channel_name_ref: CFStringRef = std::ptr::null_mut();
                                                                            
                                                                            for orig_i in 0..(channels_count as usize) {
                                                                                let orig_key_ref = channel_keys_buf[orig_i] as CFStringRef;
                                                                                if !orig_key_ref.is_null() {
                                                                                    let orig_key_str = CFString::wrap_under_get_rule(orig_key_ref);
                                                                                    let orig_key = orig_key_str.to_string();
                                                                                    
                                                                                    // Check if this key matches our channel_key, or if it's in the IOReportChannels structure
                                                                                    // The channel_key from IOReportChannels should help us find the right channel
                                                                                    // For now, let's try to find channels that contain "Performance" in their structure
                                                                                    // by checking if we can safely get the channel name
                                                                                    
                                                                                    let orig_value_ptr = channel_values_buf[orig_i];
                                                                                    if orig_value_ptr.is_null() {
                                                                                        continue;
                                                                                    }
                                                                                    
                                                                                    // PARANOID: Verify value type before casting
                                                                                    let orig_value_type_id = CFGetTypeID(orig_value_ptr as CFTypeRef);
                                                                                    let dict_type_id = CFDictionaryGetTypeID();
                                                                                    if orig_value_type_id != dict_type_id {
                                                                                        debug3!("Original channel {}: value is not CFDictionary (type_id={}), skipping", orig_i, orig_value_type_id);
                                                                                        continue;
                                                                                    }
                                                                                    
                                                                                    let orig_channel_ref = orig_value_ptr as CFDictionaryRef;
                                                                                    if orig_key != "QueryOpts" && orig_key != "IOReportChannels" {
                                                                                        // PARANOID: Verify channel_ref is valid before calling IOReport API
                                                                                        if orig_channel_ref.is_null() {
                                                                                            debug3!("Original channel {}: channel_ref is null before IOReportChannelGetChannelName!", orig_i);
                                                                                            continue;
                                                                                        }
                                                                                        let orig_channel_type_id = CFGetTypeID(orig_channel_ref as CFTypeRef);
                                                                                        let dict_type_id = CFDictionaryGetTypeID();
                                                                                        
                                                                                        // CRITICAL: Only call IOReportChannelGetChannelName on valid CFDictionary types
                                                                                        // Calling it on non-dictionary types will crash with foreign exception
                                                                                        if orig_channel_type_id != dict_type_id {
                                                                                            debug3!("Original channel {}: value is not CFDictionary (type_id={}), skipping IOReportChannelGetChannelName", orig_i, orig_channel_type_id);
                                                                                            continue;
                                                                                        }
                                                                                        
                                                                                        // #region agent log
                                                                                        write_structured_log(
                                                                                            "lib.rs:2099",
                                                                                            "About to call IOReportChannelGetChannelName",
                                                                                            &serde_json::json!({
                                                                                                "channel_ptr": format!("{:p}", orig_channel_ref),
                                                                                                "channel_type_id": orig_channel_type_id,
                                                                                                "channel_key": orig_key
                                                                                            }),
                                                                                            "B",
                                                                                        );
                                                                                        // #endregion
                                                                                        debug3!("About to call IOReportChannelGetChannelName on {:p} (key='{}')", orig_channel_ref, orig_key);
                                                                                        // Try to get channel name - this is the actual channel dictionary
                                                                                        // CRITICAL: This can throw foreign exceptions if channel_ref is invalid
                                                                                        // We've verified it's a CFDictionary, but it might still not be a valid IOReport channel
                                                                                        let test_name_ref = IOReportChannelGetChannelName(orig_channel_ref);
                                                                                        // Validate result immediately
                                                                                        if test_name_ref.is_null() {
                                                                                            debug3!("IOReportChannelGetChannelName returned null for channel '{}'", orig_key);
                                                                                        }
                                                                                        debug3!("IOReportChannelGetChannelName returned {:p}", test_name_ref);
                                                                                        // #region agent log
                                                                                        write_structured_log(
                                                                                            "lib.rs:2102",
                                                                                            "IOReportChannelGetChannelName returned",
                                                                                            &serde_json::json!({
                                                                                                "channel_ptr": format!("{:p}", orig_channel_ref),
                                                                                                "name_ptr": format!("{:p}", test_name_ref)
                                                                                            }),
                                                                                            "C",
                                                                                        );
                                                                                        // #endregion
                                                                                        if !test_name_ref.is_null() {
                                                                                            let test_channel_name = CFString::wrap_under_get_rule(test_name_ref);
                                                                                            let test_channel_name_str = test_channel_name.to_string();
                                                                                            debug3!("Original channel {}: key='{}', name='{}'", orig_i, orig_key, test_channel_name_str);
                                                                                            
                                                                                            // Check if this is a performance state channel
                                                                                            if test_channel_name_str.contains("Performance") || 
                                                                                               test_channel_name_str.contains("P-Cluster") ||
                                                                                               test_channel_name_str.contains("E-Cluster") ||
                                                                                               test_channel_name_str.contains("CPU") {
                                                                                                // Found a performance state channel - use it
                                                                                                channel_ref_to_use = orig_channel_ref;
                                                                                                channel_name_ref = test_name_ref;
                                                                                                found_channel = true;
                                                                                                debug3!("Found performance state channel: '{}' (key='{}')", test_channel_name_str, orig_key);
                                                                                                break;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            
                                                                            if !found_channel {
                                                                                debug3!("Entry {}: Could not find performance state channel in original channels_dict, skipping", i);
                                                                                continue;
                                                                            }
                                                                            
                                                                            // If channel name is null, might be nested structure
                                                                            if channel_name_ref.is_null() {
                                                                                debug3!("Entry {} channel name is null - might be nested structure", i);
                                                                                let nested_count = CFDictionaryGetCount(channel_ref_to_use);
                                                                                debug3!("Entry {} is a CFDictionary with {} nested entries", i, nested_count);
                                                                                if nested_count > 0 {
                                                                                    // This might be a nested structure - iterate it
                                                                                    let mut nested_keys: Vec<*const c_void> = vec![std::ptr::null(); nested_count as usize];
                                                                                    let mut nested_values: Vec<*const c_void> = vec![std::ptr::null(); nested_count as usize];
                                                                                    CFDictionaryGetKeysAndValues(
                                                                                        channel_ref_to_use,
                                                                                        nested_keys.as_mut_ptr(),
                                                                                        nested_values.as_mut_ptr(),
                                                                                    );
                                                                                    for j in 0..(nested_count as usize) {
                                                                                        let nested_channel_ref = nested_values[j] as CFDictionaryRef;
                                                                                        if !nested_channel_ref.is_null() {
                                                                                            // CRITICAL: Validate type before calling IOReportChannelGetChannelName
                                                                                            // This prevents foreign exceptions from invalid channel references
                                                                                            let nested_type_id = CFGetTypeID(nested_channel_ref as CFTypeRef);
                                                                                            let dict_type_id = CFDictionaryGetTypeID();
                                                                                            if nested_type_id != dict_type_id {
                                                                                                debug3!("  Nested entry {}: not CFDictionary (type_id={}), skipping", j, nested_type_id);
                                                                                                continue;
                                                                                            }
                                                                                            
                                                                                            // CRITICAL: This can throw foreign exceptions if channel_ref is invalid
                                                                                            let test_name_ref = IOReportChannelGetChannelName(nested_channel_ref);
                                                                                            if !test_name_ref.is_null() {
                                                                                                let nested_channel_name = CFString::wrap_under_get_rule(test_name_ref);
                                                                                                debug3!("  Nested entry {}: channel='{}'", j, nested_channel_name.to_string());
                                                                                                // Use nested channel instead
                                                                                                channel_ref_to_use = nested_channel_ref;
                                                                                                channel_name_ref = test_name_ref;
                                                                                                break;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                                if channel_name_ref.is_null() {
                                                                                    debug3!("Entry {}: no valid channel found, skipping", i);
                                                                                    continue;
                                                                                }
                                                                            }
                                                                            
                                                                            let channel_name = CFString::wrap_under_get_rule(channel_name_ref);
                                                                            let channel_name_str = channel_name.to_string();
                                                                            debug3!("Processing channel: name='{}'", channel_name_str);
                                                                            
                                                                            // Look for performance state channels (they contain frequency info)
                                                                            // Determine if this is a P-core or E-core channel
                                                                            let is_p_core = channel_name_str.contains("P-Cluster") || 
                                                                                          (channel_name_str.contains("Performance") && !channel_name_str.contains("E-Cluster") && !channel_name_str.contains("Efficiency"));
                                                                            let is_e_core = channel_name_str.contains("E-Cluster") || channel_name_str.contains("Efficiency");
                                                                            
                                                                            if channel_name_str.contains("Performance") || 
                                                                               channel_name_str.contains("P-Cluster") ||
                                                                               channel_name_str.contains("E-Cluster") ||
                                                                               channel_name_str.contains("CPU") {
                                                                                debug3!("Found performance state channel: '{}' (P-core: {}, E-core: {})", channel_name_str, is_p_core, is_e_core);
                                                                                
                                                                                // CRITICAL: Validate channel_ref_to_use before calling IOReport functions
                                                                                // These functions can throw foreign exceptions if called on invalid references
                                                                                if channel_ref_to_use.is_null() {
                                                                                    debug3!("Channel '{}' reference is null, skipping state iteration", channel_name_str);
                                                                                    continue;
                                                                                }
                                                                                
                                                                                // Get state count (number of performance states)
                                                                                let state_count = IOReportStateGetCount(channel_ref_to_use);
                                                                                debug3!("Channel '{}' has {} performance states", channel_name_str, state_count);
                                                                                
                                                                                // Iterate through states to find active frequency
                                                                                for state_idx in 0..state_count {
                                                                                    // Get state name (e.g., "P0", "P1", "IDLE", or frequency like "2400 MHz")
                                                                                    let state_name_ref = IOReportStateGetNameForIndex(channel_ref_to_use, state_idx);
                                                                                    if state_name_ref.is_null() {
                                                                                        continue;
                                                                                    }
                                                                                    
                                                                                    let state_name = CFString::wrap_under_get_rule(state_name_ref);
                                                                                    let state_name_str = state_name.to_string();
                                                                                    debug3!("  State {}: name='{}'", state_idx, state_name_str);
                                                                                    
                                                                                    // Get residency (time spent in this state)
                                                                                    let residency_ns = IOReportStateGetResidency(channel_ref_to_use, state_idx);
                                                                                    debug3!("  State {}: residency={} ns", state_idx, residency_ns);
                                                                                    
                                                                                    // Try to extract frequency from state name
                                                                                    // Patterns: "2400 MHz", "4000 MHz", "P0", "P1", etc.
                                                                                    let residency_ratio = residency_ns as f64 / 1_000_000_000.0; // Convert ns to seconds
                                                                                    
                                                                                    if state_name_str.contains("MHz") {
                                                                                        // Extract frequency value from name (e.g., "2400 MHz")
                                                                                        // Try multiple parsing strategies
                                                                                        let mhz_val = state_name_str
                                                                                            .split_whitespace()
                                                                                            .find_map(|s| s.parse::<f64>().ok());
                                                                                        
                                                                                        if let Some(mhz_val) = mhz_val {
                                                                                            if mhz_val > 0.0 && mhz_val < 10000.0 { // Sanity check: 0-10 GHz
                                                                                                // Update overall frequency
                                                                                                if mhz_val > max_freq_mhz {
                                                                                                    max_freq_mhz = mhz_val;
                                                                                                }
                                                                                                weighted_freq_sum += mhz_val * residency_ratio;
                                                                                                total_residency += residency_ratio;
                                                                                                
                                                                                                // Update P-core or E-core specific frequency
                                                                                                if is_p_core {
                                                                                                    if mhz_val > p_core_max_freq_mhz {
                                                                                                        p_core_max_freq_mhz = mhz_val;
                                                                                                    }
                                                                                                    p_core_weighted_freq_sum += mhz_val * residency_ratio;
                                                                                                    p_core_total_residency += residency_ratio;
                                                                                                } else if is_e_core {
                                                                                                    if mhz_val > e_core_max_freq_mhz {
                                                                                                        e_core_max_freq_mhz = mhz_val;
                                                                                                    }
                                                                                                    e_core_weighted_freq_sum += mhz_val * residency_ratio;
                                                                                                    e_core_total_residency += residency_ratio;
                                                                                                }
                                                                                                debug3!("  State {}: extracted {} MHz from name '{}'", state_idx, mhz_val, state_name_str);
                                                                                            } else {
                                                                                                debug3!("  State {}: frequency {} MHz out of range, skipping", state_idx, mhz_val);
                                                                                            }
                                                                                        } else {
                                                                                            debug3!("  State {}: could not parse frequency from '{}'", state_idx, state_name_str);
                                                                                        }
                                                                                    } else if state_name_str.starts_with("P") {
                                                                                        // P-state (P0, P1, etc.) - estimate frequency based on state index
                                                                                        // P0 = max freq, P1 = slightly lower, etc.
                                                                                        // For Apple Silicon: P0 is typically 3.5-4.0 GHz for P-cores, 2.0-2.5 GHz for E-cores
                                                                                        let estimated_freq = if is_p_core {
                                                                                            // P-core: higher frequency
                                                                                            match state_idx {
                                                                                                0 => 4000.0, // P0 = max
                                                                                                1 => 3500.0, // P1
                                                                                                2 => 3000.0, // P2
                                                                                                _ => 2500.0, // Lower states
                                                                                            }
                                                                                        } else if is_e_core {
                                                                                            // E-core: lower frequency
                                                                                            match state_idx {
                                                                                                0 => 2400.0, // E0 = max
                                                                                                1 => 2000.0, // E1
                                                                                                _ => 1500.0, // Lower states
                                                                                            }
                                                                                        } else {
                                                                                            // Unknown cluster type - use conservative estimate
                                                                                            match state_idx {
                                                                                                0 => 3000.0, // P0 equivalent
                                                                                                _ => 2000.0,
                                                                                            }
                                                                                        };
                                                                                        
                                                                                        // Update overall frequency
                                                                                        weighted_freq_sum += estimated_freq * residency_ratio;
                                                                                        total_residency += residency_ratio;
                                                                                        if estimated_freq > max_freq_mhz {
                                                                                            max_freq_mhz = estimated_freq;
                                                                                        }
                                                                                        
                                                                                        // Update P-core or E-core specific frequency
                                                                                        if is_p_core {
                                                                                            p_core_weighted_freq_sum += estimated_freq * residency_ratio;
                                                                                            p_core_total_residency += residency_ratio;
                                                                                            if estimated_freq > p_core_max_freq_mhz {
                                                                                                p_core_max_freq_mhz = estimated_freq;
                                                                                            }
                                                                                        } else if is_e_core {
                                                                                            e_core_weighted_freq_sum += estimated_freq * residency_ratio;
                                                                                            e_core_total_residency += residency_ratio;
                                                                                            if estimated_freq > e_core_max_freq_mhz {
                                                                                                e_core_max_freq_mhz = estimated_freq;
                                                                                            }
                                                                                        }
                                                                                        debug3!("  State {}: estimated {} MHz from P-state '{}'", state_idx, estimated_freq, state_name_str);
                                                                                    } else {
                                                                                        debug3!("  State {}: name '{}' doesn't match frequency patterns, skipping", state_idx, state_name_str);
                                                                                    }
                                                                                } // closes for state_idx
                                                                            } // closes if channel_name_str.contains
                                                                        } // closes for i in 0..actual_channels_count
                                                                        
                                                                        // Calculate frequency: use weighted average if available, otherwise max
                                                                        if total_residency > 0.0 {
                                                                            freq = (weighted_freq_sum / total_residency / 1000.0) as f32; // Convert MHz to GHz
                                                                            debug2!("IOReport frequency parsed: {:.2} GHz (weighted average from {} states)", freq, total_residency);
                                                                        } else if max_freq_mhz > 0.0 {
                                                                            freq = (max_freq_mhz / 1000.0) as f32; // Convert MHz to GHz
                                                                            debug2!("IOReport frequency parsed: {:.2} GHz (max frequency)", freq);
                                                                        } else {
                                                                            debug3!("Could not extract frequency from IOReport (no valid states found in {} actual channels)", actual_channels_count);
                                                                        }
                                                                        
                                                                        // Calculate P-core frequency
                                                                        if p_core_total_residency > 0.0 {
                                                                            p_core_freq = (p_core_weighted_freq_sum / p_core_total_residency / 1000.0) as f32; // Convert MHz to GHz
                                                                            debug2!("IOReport P-core frequency parsed: {:.2} GHz (weighted average)", p_core_freq);
                                                                        } else if p_core_max_freq_mhz > 0.0 {
                                                                            p_core_freq = (p_core_max_freq_mhz / 1000.0) as f32; // Convert MHz to GHz
                                                                            debug2!("IOReport P-core frequency parsed: {:.2} GHz (max frequency)", p_core_freq);
                                                                        }
                                                                        
                                                                        // Calculate E-core frequency
                                                                        if e_core_total_residency > 0.0 {
                                                                            e_core_freq = (e_core_weighted_freq_sum / e_core_total_residency / 1000.0) as f32; // Convert MHz to GHz
                                                                            debug2!("IOReport E-core frequency parsed: {:.2} GHz (weighted average)", e_core_freq);
                                                                        } else if e_core_max_freq_mhz > 0.0 {
                                                                            e_core_freq = (e_core_max_freq_mhz / 1000.0) as f32; // Convert MHz to GHz
                                                                            debug2!("IOReport E-core frequency parsed: {:.2} GHz (max frequency)", e_core_freq);
                                                                        }
                                                                    } // closes else block for actual_channels_ref null check
                                                                } else {
                                                                    debug3!("IOReportChannels is empty (no actual channels)");
                                                                }
                                                            }
                                                        } else {
                                                            debug3!("Original channels_dict is empty (no channels)");
                                                        }
                                                    } // closes inner block
                                                } else {
                                                    debug3!("Original channels_dict not available, cannot parse frequency");
                                                }
                                            } else {
                                                debug3!("Failed to create IOReport sample (sample is null)");
                                            }
                                        } // closes outer unsafe block
                                    } else {
                                        debug3!("Subscription pointer is null, cannot create sample");
                                    }
                                } else {
                                    debug3!("IOReport subscription not available");
                                }
                            } else {
                                debug3!("should_read_freq=false, skipping frequency update");
                            }
                            
                            // CRITICAL: Only use nominal frequency as fallback if IOReport completely failed
                            // If IOReport returned 0.0, it means parsing failed - don't overwrite cache with nominal
                            // Only update cache if we got a real frequency from IOReport
                            if freq > 0.0 {
                                if let Ok(mut cache) = FREQ_CACHE.try_lock() {
                                    *cache = Some((freq, std::time::Instant::now()));
                                    debug2!("Frequency cache updated from IOReport: {:.2} GHz", freq);
                                }
                                
                                // Update P-core frequency cache
                                if p_core_freq > 0.0 {
                                    if let Ok(mut cache) = P_CORE_FREQ_CACHE.try_lock() {
                                        *cache = Some((p_core_freq, std::time::Instant::now()));
                                        debug2!("P-core frequency cache updated: {:.2} GHz", p_core_freq);
                                    }
                                }
                                
                                // Update E-core frequency cache
                                if e_core_freq > 0.0 {
                                    if let Ok(mut cache) = E_CORE_FREQ_CACHE.try_lock() {
                                        *cache = Some((e_core_freq, std::time::Instant::now()));
                                        debug2!("E-core frequency cache updated: {:.2} GHz", e_core_freq);
                                    }
                                }
                                
                                // CRITICAL: Update ACCESS_CACHE to indicate frequency reading works
                                if let Ok(mut access_cache) = ACCESS_CACHE.try_lock() {
                                    if let Some((temp, _, cpu_power, gpu_power)) = access_cache.as_ref() {
                                        *access_cache = Some((*temp, true, *cpu_power, *gpu_power));
                                    } else {
                                        *access_cache = Some((false, true, false, false));
                                    }
                                    debug2!("ACCESS_CACHE updated: can_read_frequency=true (IOReport frequency read successfully)");
                                }
                            } else {
                                // IOReport parsing failed - log why and don't update cache
                                // This prevents overwriting a good cached value with nominal frequency
                                debug2!("IOReport frequency parsing failed (freq=0.0) - keeping existing cache value if available");
                                
                                // Only use nominal as fallback if cache is empty (first time)
                                if let Ok(cache) = FREQ_CACHE.try_lock() {
                                    if cache.is_none() {
                                        // Cache is empty - use nominal as initial value
                                        let nominal = metrics::get_nominal_frequency();
                                        if nominal > 0.0 {
                                            debug2!("Using nominal frequency as initial value: {:.2} GHz (IOReport not available yet)", nominal);
                                            // Don't update cache here - let IOReport populate it when it works
                                        }
                                    } else {
                                        debug3!("Keeping existing cached frequency value (IOReport parsing failed)");
                                    }
                                }
                            }
                        } else {
                            debug3!("should_read_freq=false, skipping frequency update");
                        }
                    } else {
                        // CPU window is not visible - clear SMC connection and IOReport subscription to save resources
                        if smc_connection.is_some() {
                            smc_connection = None;
                            debug3!("CPU window closed, SMC connection released");
                        }
                        
                        // CRITICAL: Clear IOReport subscription when window closes to save CPU
                        // Note: IOReport doesn't have an explicit destroy function in the API
                        // The subscription will be cleaned up when the process exits
                        // For now, just clear the reference
                        if let Ok(mut sub) = IOREPORT_SUBSCRIPTION.try_lock() {
                            if sub.is_some() {
                                *sub = None;
                                debug2!("CPU window closed, IOReport subscription cleared");
                                
                                // Clear channels dictionary
                                if let Ok(mut channels_storage) = IOREPORT_CHANNELS.try_lock() {
                                    *channels_storage = None;
                                }
                                
                                // Clear last sample
                                if let Ok(mut last_sample) = LAST_IOREPORT_SAMPLE.try_lock() {
                                    *last_sample = None;
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
