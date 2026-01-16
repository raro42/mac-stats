use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::process::Command;
use std::sync::OnceLock;
use std::sync::Mutex;
use sysinfo::{Disks, System};
use macsmc::Smc;
// IOReport types kept for future use (extern block still references them)
use core_foundation::base::CFTypeRef;
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation::string::CFStringRef;

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
use objc2::declare::ClassBuilder;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, NSObject, Sel};
use objc2::{msg_send, AnyThread, ClassType, MainThreadMarker, sel};
use objc2_app_kit::{
    NSAboutPanelOptionApplicationName, NSAboutPanelOptionApplicationVersion,
    NSAboutPanelOptionCredits, NSAboutPanelOptionVersion, NSApplication, NSColor, NSFont,
    NSFontWeightRegular, NSFontWeightSemibold, NSBaselineOffsetAttributeName,
    NSFontAttributeName, NSForegroundColorAttributeName, NSParagraphStyleAttributeName,
    NSMutableParagraphStyle, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength, NSTextAlignment, NSTextTab, NSTextTabOptionKey, NSEvent,
};
use objc2_foundation::{
    NSArray, NSDictionary, NSMutableAttributedString, NSMutableDictionary, NSNumber,
    NSAttributedString, NSProcessInfo, NSRange, NSString,
};
use tauri::{Manager, WindowBuilder, WindowUrl};
use std::sync::atomic::{AtomicU8, Ordering};

// Debug verbosity level: 0 = none, 1 = -v, 2 = -vv, 3 = -vvv
static VERBOSITY: AtomicU8 = AtomicU8::new(0);

// Log file path - accessible when running as root
const LOG_FILE_PATH: &str = "/Users/raro42/projects/mac-stats/src-tauri/.cursor/debug.log";

// Debug logging macros with timestamps
fn format_timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    let total_millis = secs * 1000 + (nanos / 1_000_000) as u64;
    let millis = total_millis % 1000;
    let secs = (total_millis / 1000) % 60;
    let mins = (total_millis / 60000) % 60;
    let hours = (total_millis / 3600000) % 24;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
}

// Write log entry to both terminal and log file
fn write_log_entry(level_str: &str, message: &str) {
    let timestamp = format_timestamp();
    let log_line = format!("[{}] [{}] {}", timestamp, level_str, message);
    
    // Write to terminal (stderr)
    eprintln!("{}", log_line);
    
    // Write to log file
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_PATH)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", log_line);
    }
}

// Write structured log entry (JSON) to log file
fn write_structured_log(location: &str, message: &str, data: &serde_json::Value, hypothesis_id: &str) {
    let log_data = serde_json::json!({
        "location": location,
        "message": message,
        "data": data,
        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
        "sessionId": "debug-session",
        "runId": "run3",
        "hypothesisId": hypothesis_id
    });
    
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_PATH)
    {
        use std::io::Write;
        if let Ok(json_str) = serde_json::to_string(&log_data) {
            let _ = writeln!(file, "{}", json_str);
        }
    }
    
    // Also write human-readable version to terminal
    eprintln!("[DEBUG] {}: {} (hypothesis: {})", location, message, hypothesis_id);
}

macro_rules! debug {
    ($level:expr, $($arg:tt)*) => {
        {
            use std::sync::atomic::Ordering;
            if VERBOSITY.load(Ordering::Relaxed) >= $level {
                let level_str = match $level {
                    1 => "INFO",
                    2 => "DEBUG",
                    3 => "TRACE",
                    _ => "LOG",
                };
                let message = format!($($arg)*);
                write_log_entry(level_str, &message);
            }
        }
    };
}

macro_rules! debug1 {
    ($($arg:tt)*) => { debug!(1, $($arg)*); };
}

macro_rules! debug2 {
    ($($arg:tt)*) => { debug!(2, $($arg)*); };
}

macro_rules! debug3 {
    ($($arg:tt)*) => { debug!(3, $($arg)*); };
}

pub fn set_verbosity(level: u8) {
    VERBOSITY.store(level, Ordering::Relaxed);
    if level > 0 {
        eprintln!("Debug verbosity level set to: {}", level);
    }
}

static SYSTEM: Mutex<Option<System>> = Mutex::new(None);
static DISKS: Mutex<Option<Disks>> = Mutex::new(None);
static LAST_SYSTEM_REFRESH: Mutex<Option<std::time::Instant>> = Mutex::new(None);
thread_local! {
    static STATUS_ITEM: RefCell<Option<Retained<NSStatusItem>>> = RefCell::new(None);
    static CLICK_HANDLER: RefCell<Option<Retained<AnyObject>>> = RefCell::new(None);
}
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
static MENU_BAR_TEXT: Mutex<Option<String>> = Mutex::new(None);
const BUILD_DATE: &str = env!("BUILD_DATE");

// Cache expensive resources to avoid recreating every second
// Note: IOReport subscriptions are very expensive to create/destroy - removed for performance
// SMC cannot be cached in static (not Sync) - will create on demand but cache access checks
static CHIP_INFO_CACHE: OnceLock<String> = OnceLock::new();
static ACCESS_CACHE: Mutex<Option<(bool, bool, bool, bool)>> = Mutex::new(None); // temp, freq, cpu_power, gpu_power

#[derive(serde::Serialize, Clone)]
struct SystemMetrics {
    cpu: f32,
    gpu: f32,
    ram: f32,
    disk: f32,
}

#[derive(serde::Serialize)]
struct ProcessUsage {
    name: String,
    cpu: f32,
}

#[derive(serde::Serialize)]
struct CpuDetails {
    usage: f32,
    temperature: f32,
    frequency: f32,
    cpu_power: f32,
    gpu_power: f32,
    load_1: f64,
    load_5: f64,
    load_15: f64,
    uptime_secs: u64,
    top_processes: Vec<ProcessUsage>,
    chip_info: String,
    // Access flags - true if we can read the value, false if access is denied
    can_read_temperature: bool,
    can_read_frequency: bool,
    can_read_cpu_power: bool,
    can_read_gpu_power: bool,
}

#[allow(dead_code)]
fn get_chip_info() -> String {
    // Cache chip info - only fetch once
    CHIP_INFO_CACHE.get_or_init(|| {
        // Get chip information from system_profiler (JSON format)
        let output = Command::new("sh")
            .arg("-c")
            .arg("system_profiler SPHardwareDataType -json 2>/dev/null")
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Try to parse JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(hardware) = json.get("SPHardwareDataType").and_then(|v| v.as_array()).and_then(|a| a.get(0)) {
                        let chip_type = hardware.get("chip_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let num_procs = hardware.get("number_processors")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        
                        if !chip_type.is_empty() {
                            // Format: "Apple M3 • 16 cores" or similar
                            let mut info = chip_type.to_string();
                            if !num_procs.is_empty() {
                                // number_processors format: "proc 1:8:8" (total:performance:efficiency)
                                let num_procs_clean = num_procs.strip_prefix("proc ").unwrap_or(num_procs);
                                let parts: Vec<&str> = num_procs_clean.split(':').collect();
                                if parts.len() >= 3 {
                                    let p_cores = parts.get(1).unwrap_or(&"0");
                                    let e_cores = parts.get(2).unwrap_or(&"0");
                                    let total: u32 = p_cores.parse().unwrap_or(0) + e_cores.parse().unwrap_or(0);
                                    if total > 0 {
                                        info.push_str(&format!(" • {} cores", total));
                                    }
                                } else if parts.len() == 1 {
                                    // Single number (total cores)
                                    if let Ok(total) = parts[0].parse::<u32>() {
                                        if total > 0 {
                                            info.push_str(&format!(" • {} cores", total));
                                        }
                                    }
                                }
                            }
                            return info;
                        }
                    }
                }
            }
        }
        
        // Fallback: try sysctl for Intel Macs
        let output = Command::new("sh")
            .arg("-c")
            .arg("sysctl -n machdep.cpu.brand_string 2>/dev/null | head -1")
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let trimmed = stdout.trim();
                if !trimmed.is_empty() && trimmed.len() < 50 {
                    return trimmed.to_string();
                }
            }
        }
        
        "—".to_string()
    }).clone()
}

fn get_gpu_usage() -> f32 {
    // GPU usage reading is expensive (ioreg commands) - return 0 for now to save CPU
    // TODO: Cache or optimize if GPU usage is needed
    0.0
}

#[allow(dead_code)]
fn can_read_temperature() -> bool {
    // Check cache first (only check once, then cache)
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((temp, _, _, _)) = cache.as_ref() {
        return *temp;
    }
    
    // Check if we can access temperature data (only once)
    let can_read = {
        // Try SMC first
        if let Ok(mut smc) = Smc::connect() {
            smc.cpu_temperature().is_ok()
        } else {
            // Fallback to NSProcessInfo (always available)
            if let Some(_mtm) = MainThreadMarker::new() {
                let process_info = NSProcessInfo::processInfo();
                let thermal_state = process_info.thermalState();
                thermal_state.0 <= 3 // Valid thermal state
            } else {
                false
            }
        }
    };
    
    // Cache the result permanently
    if let Some((_, freq, cpu_power, gpu_power)) = cache.as_ref() {
        *cache = Some((can_read, *freq, *cpu_power, *gpu_power));
    } else {
        *cache = Some((can_read, false, false, false));
    }
    
    can_read
}

#[allow(dead_code)]
fn get_cpu_temperature() -> f32 {
    // Try SMC (create connection each time - Smc is not Sync so can't cache in static)
    // But this is still cheaper than IOReport
    if let Ok(mut smc) = Smc::connect() {
        if let Ok(temps) = smc.cpu_temperature() {
            let die_temp: f64 = temps.die.into();
            let prox_temp: f64 = temps.proximity.into();
            
            let temp = if die_temp > 0.0 {
                die_temp
            } else if prox_temp > 0.0 {
                prox_temp
            } else {
                0.0
            };
            if temp > 0.0 {
                return temp as f32;
            }
        }
    }
    
    // Fallback to NSProcessInfo.thermalState (always available, user space, very cheap)
    if let Some(_mtm) = MainThreadMarker::new() {
        let process_info = NSProcessInfo::processInfo();
        let thermal_state = process_info.thermalState();
        let state_value = thermal_state.0;
        
        // Map thermal state to approximate temperature
        match state_value {
            0 => 40.0,  // Nominal
            1 => 60.0,  // Fair
            2 => 80.0,  // Serious
            3 => 95.0,  // Critical
            _ => 0.0,
        }
    } else {
        0.0
    }
}

#[allow(dead_code)]
fn can_read_frequency() -> bool {
    // Check cache first
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((_, freq, _, _)) = cache.as_ref() {
        return *freq;
    }
    
    // Check if sysctl works (much cheaper than IOReport)
    let output = Command::new("sh")
        .arg("-c")
        .arg("sysctl -n hw.cpufrequency_max 2>/dev/null || sysctl -n hw.cpufrequency 2>/dev/null || echo ''")
        .output();
    
    let can_read = if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            !stdout.trim().is_empty() && stdout.trim() != "0"
        } else {
            false
        }
    } else {
        false
    };
    
    // Update cache
    if let Some((temp, _, cpu_power, gpu_power)) = cache.as_ref() {
        *cache = Some((*temp, can_read, *cpu_power, *gpu_power));
    } else {
        *cache = Some((false, can_read, false, false));
    }
    
    can_read
}

#[allow(dead_code)]
fn get_cpu_frequency() -> f32 {
    // IOReport is too expensive to call every second - skip it for now
    // Only use sysctl fallback which is much cheaper
    let output = Command::new("sh")
        .arg("-c")
        .arg("sysctl -n hw.cpufrequency_max 2>/dev/null || sysctl -n hw.cpufrequency 2>/dev/null || echo '0'")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            if !trimmed.is_empty() && trimmed != "0" {
                if let Ok(freq_hz) = trimmed.parse::<f64>() {
                    if freq_hz > 0.0 {
                        let freq_ghz = (freq_hz / 1_000_000_000.0) as f32;
                        if freq_ghz > 0.1 && freq_ghz < 10.0 {
                            return freq_ghz;
                        }
                    }
                }
            }
        }
    }
    
    0.0
}

#[allow(dead_code)]
fn can_read_cpu_power() -> bool {
    // Check cache first
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((_, _, cpu_power, _)) = cache.as_ref() {
        return *cpu_power;
    }
    
    // IOReport is too expensive - check once and cache
    // For now, assume false unless we can verify cheaply
    let can_read = false; // IOReport too expensive to check every time
    
    // Update cache
    if let Some((temp, freq, _, gpu_power)) = cache.as_ref() {
        *cache = Some((*temp, *freq, can_read, *gpu_power));
    } else {
        *cache = Some((false, false, can_read, false));
    }
    
    can_read
}

#[allow(dead_code)]
fn get_cpu_power() -> f32 {
    // IOReport is too expensive to call every second - return 0 for now
    // TODO: Implement proper caching if power reading is needed
    0.0
}

#[allow(dead_code)]
fn can_read_gpu_power() -> bool {
    // Check cache first
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((_, _, _, gpu_power)) = cache.as_ref() {
        return *gpu_power;
    }
    
    // IOReport is too expensive - check once and cache
    let can_read = false; // IOReport too expensive to check every time
    
    // Update cache
    if let Some((temp, freq, cpu_power, _)) = cache.as_ref() {
        *cache = Some((*temp, *freq, *cpu_power, can_read));
    } else {
        *cache = Some((false, false, false, can_read));
    }
    
    can_read
}

#[allow(dead_code)]
fn get_gpu_power() -> f32 {
    // IOReport is too expensive to call every second - return 0 for now
    // TODO: Implement proper caching if power reading is needed
    0.0
}

fn get_metrics() -> SystemMetrics {
    // #region agent log
    let start_time = std::time::Instant::now();
    write_structured_log("lib.rs:512", "get_metrics ENTRY", &serde_json::json!({"timestamp": start_time.elapsed().as_millis()}), "CPU1");
    // #endregion
    debug3!("get_metrics() called");
    
    // Check if we should refresh (only every 2 seconds to reduce CPU usage)
    // Use try_lock to avoid blocking
    let refresh_check_start = std::time::Instant::now();
    let should_refresh = match LAST_SYSTEM_REFRESH.try_lock() {
        Ok(mut last_refresh) => {
            let now = std::time::Instant::now();
            let should = last_refresh.map(|lr| now.duration_since(lr).as_secs() >= 2).unwrap_or(true);
            if should {
                *last_refresh = Some(now);
                debug3!("Refresh allowed (1 second passed)");
            } else {
                debug3!("Refresh skipped (less than 1 second since last)");
            }
            should
        },
        Err(_) => {
            // Lock held - skip refresh to avoid blocking
            debug3!("Refresh skipped (lock held)");
            false
        }
    };
    // #region agent log
    write_structured_log("lib.rs:534", "refresh_check_duration", &serde_json::json!({"duration_ms": refresh_check_start.elapsed().as_millis()}), "CPU1");
    // #endregion
    
    // Use try_lock ONCE - if locked, return cached values immediately (no retry loop)
    let cpu_ram_start = std::time::Instant::now();
    let (cpu_usage, ram_usage) = match SYSTEM.try_lock() {
        Ok(mut sys) => {
            if sys.is_none() {
                debug3!("Creating new System instance");
                // Create outside lock scope if possible, but we need the lock to store it
                *sys = Some(System::new());
            }
            let sys = sys.as_mut().unwrap();
            
            // Only refresh if enough time has passed (reduces CPU usage)
            if should_refresh {
                // #region agent log
                let refresh_start = std::time::Instant::now();
                // #endregion
                debug3!("Refreshing CPU usage and memory");
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                // #region agent log
                write_structured_log("lib.rs:553", "refresh_cpu_memory_duration", &serde_json::json!({"duration_ms": refresh_start.elapsed().as_millis()}), "CPU1");
                // #endregion
            }

            let cpu = sys.global_cpu_usage();
            let ram = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;
            debug3!("CPU usage: {}%, RAM usage: {}%", cpu, ram);
            
            (cpu, ram)
        },
        Err(_) => {
            // Lock held - return zeros immediately, no retry to avoid CPU spinning
            debug3!("SYSTEM mutex locked, using cached zeros");
            (0.0, 0.0)
        }
    };
    // #region agent log
    write_structured_log("lib.rs:576", "cpu_ram_duration", &serde_json::json!({"duration_ms": cpu_ram_start.elapsed().as_millis()}), "CPU1");
    // #endregion
    
    // Get disk usage - use try_lock and skip refresh entirely
    let disk_start = std::time::Instant::now();
    let mut disk_usage = 0.0;
    match DISKS.try_lock() {
        Ok(mut disks) => {
            if disks.is_none() {
                debug3!("Creating new Disks instance (will refresh once)");
                *disks = Some(Disks::new());
                // Refresh once on creation, then never again
                if let Some(ref mut d) = disks.as_mut() {
                    debug3!("Initial disk refresh (one time only)");
                    d.refresh(false);
                }
            }
            let disks = disks.as_mut().unwrap();
            // DON'T refresh - just read cached values to avoid blocking
            debug3!("Reading disk info (no refresh)");
            for disk in disks.list() {
                let total = disk.total_space() as f64;
                let available = disk.available_space() as f64;
                if total > 0.0 {
                    disk_usage = ((total - available) / total * 100.0) as f32;
                    debug3!("Disk usage: {}% (total: {}, available: {})", disk_usage, total, available);
                    break;
                }
            }
        },
        Err(_) => {
            debug1!("WARNING: DISKS mutex is locked, using 0% for disk");
        }
    }
    // #region agent log
    write_structured_log("lib.rs:607", "disk_duration", &serde_json::json!({"duration_ms": disk_start.elapsed().as_millis()}), "CPU1");
    // #endregion
    
    let gpu_start = std::time::Instant::now();
    let gpu_usage = get_gpu_usage();
    // #region agent log
    write_structured_log("lib.rs:609", "gpu_duration", &serde_json::json!({"duration_ms": gpu_start.elapsed().as_millis()}), "CPU1");
    // #endregion
    debug3!("GPU usage: {}%", gpu_usage);

    let metrics = SystemMetrics {
        cpu: cpu_usage,
        gpu: gpu_usage,
        ram: ram_usage,
        disk: disk_usage,
    };
    debug3!("Returning metrics: CPU={}%, GPU={}%, RAM={}%, DISK={}%", 
        metrics.cpu, metrics.gpu, metrics.ram, metrics.disk);
    // #region agent log
    write_structured_log("lib.rs:620", "get_metrics EXIT", &serde_json::json!({"total_duration_ms": start_time.elapsed().as_millis()}), "CPU1");
    // #endregion
    metrics
}

#[tauri::command]
fn get_cpu_details() -> CpuDetails {
    // #region agent log
    let start_time = std::time::Instant::now();
    write_structured_log("lib.rs:624", "get_cpu_details ENTRY", &serde_json::json!({"timestamp": start_time.elapsed().as_millis()}), "CPU2");
    // #endregion
    debug3!("get_cpu_details() called");
    
    // CRITICAL: Use try_lock ONCE - if locked, return defaults immediately
    // This prevents blocking the main thread when the window opens
    let (usage, load, uptime_secs, top_processes) = match SYSTEM.try_lock() {
        Ok(mut sys) => {
            if sys.is_none() {
                // Don't create System here - it's expensive and blocks
                // Return defaults and let background thread create it
                debug1!("WARNING: SYSTEM is None in get_cpu_details, returning defaults");
                write_structured_log("lib.rs:616", "SYSTEM is None, returning defaults", &serde_json::json!({}), "L");
                (0.0, sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 }, 0, Vec::new())
            } else {
                let sys = sys.as_mut().unwrap();
                // CRITICAL: Don't refresh here - it's expensive and blocks
                // Just read existing values without refreshing
                let usage = sys.global_cpu_usage();
                let load = sysinfo::System::load_average();
                let uptime_secs = sysinfo::System::uptime();
                
                // Limit process collection to avoid blocking
                // #region agent log
                let proc_start = std::time::Instant::now();
                // #endregion
                let mut processes: Vec<ProcessUsage> = sys
                    .processes()
                    .values()
                    .take(100) // Limit to first 100 processes to avoid blocking
                    .map(|proc| ProcessUsage {
                        name: proc.name().to_string_lossy().to_string(),
                        cpu: proc.cpu_usage(),
                    })
                    .collect();
                processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
                processes.truncate(8);
                // #region agent log
                write_structured_log("lib.rs:656", "process_collection_duration", &serde_json::json!({"duration_ms": proc_start.elapsed().as_millis(), "count": processes.len()}), "CPU2");
                // #endregion
                
                write_structured_log("lib.rs:637", "get_cpu_details got data from SYSTEM", &serde_json::json!({"usage": usage, "process_count": processes.len()}), "L");
                (usage, load, uptime_secs, processes)
            }
        },
        Err(_) => {
            // Lock is held - return defaults immediately, don't retry
            debug1!("WARNING: SYSTEM mutex locked in get_cpu_details, returning defaults immediately");
            write_structured_log("lib.rs:644", "SYSTEM locked, returning defaults", &serde_json::json!({}), "L");
            (0.0, sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 }, 0, Vec::new())
        }
    };

    // CRITICAL: Use cached values or defaults - don't call expensive functions
    // SMC calls and other operations can block the main thread
    // Use try_lock for cache access too
    let (temperature, frequency, cpu_power, gpu_power, chip_info, can_read_temperature, can_read_frequency, can_read_cpu_power, can_read_gpu_power) = {
        // Try to get cached access flags without blocking
        match ACCESS_CACHE.try_lock() {
            Ok(mut access_cache) => {
                let (can_read_temp, can_read_freq, can_read_cpu_p, can_read_gpu_p) = 
                    if let Some(cached) = access_cache.as_ref() {
                        *cached
                    } else {
                        // First time - use defaults, don't check (expensive)
                        let result = (false, false, false, false);
                        *access_cache = Some(result);
                        result
                    };
                
                // Use cached chip info or default
                let chip = CHIP_INFO_CACHE.get().cloned().unwrap_or_else(|| "—".to_string());
                
                // Return defaults for expensive values - they'll be populated on next refresh
                (0.0, 0.0, 0.0, 0.0, chip, can_read_temp, can_read_freq, can_read_cpu_p, can_read_gpu_p)
            },
            Err(_) => {
                // Cache locked - return all defaults
                let chip = CHIP_INFO_CACHE.get().cloned().unwrap_or_else(|| "—".to_string());
                (0.0, 0.0, 0.0, 0.0, chip, false, false, false, false)
            }
        }
    };

    // #region agent log
    write_structured_log("lib.rs:743", "get_cpu_details EXIT", &serde_json::json!({"total_duration_ms": start_time.elapsed().as_millis(), "usage": usage, "process_count": top_processes.len()}), "CPU2");
    // #endregion
    
    CpuDetails {
        usage,
        temperature,
        frequency,
        cpu_power,
        gpu_power,
        load_1: load.one,
        load_5: load.five,
        load_15: load.fifteen,
        uptime_secs,
        top_processes,
        chip_info,
        can_read_temperature,
        can_read_frequency,
        can_read_cpu_power,
        can_read_gpu_power,
    }
}

fn build_status_text(metrics: &SystemMetrics) -> String {
    let label_line = "CPU\tGPU\tRAM\tSSD".to_string();
    let value_line = format!(
        "{:.0}%\t{:.0}%\t{:.0}%\t{:.0}%",
        metrics.cpu.round() as i32,
        metrics.gpu.round() as i32,
        metrics.ram.round() as i32,
        metrics.disk.round() as i32
    );
    format!("{label_line}\n{value_line}")
}

fn as_any<T: objc2::Message>(obj: &T) -> &AnyObject {
    unsafe { &*(obj as *const T as *const AnyObject) }
}

fn process_menu_bar_update() {
    // #region agent log
    let update_start = std::time::Instant::now();
    write_structured_log("lib.rs:782", "process_menu_bar_update ENTRY", &serde_json::json!({}), "CPU4");
    // #endregion
    // This function must be called from the main thread
    if let Some(mtm) = MainThreadMarker::new() {
        write_structured_log("lib.rs:672", "MainThreadMarker obtained", &serde_json::json!({}), "G");
        let update_text = {
            if let Ok(mut pending) = MENU_BAR_TEXT.try_lock() {
                pending.take()
            } else {
                write_structured_log("lib.rs:675", "MENU_BAR_TEXT lock failed", &serde_json::json!({}), "G");
                return;
            }
        };
        
        if let Some(text) = update_text {
            write_structured_log("lib.rs:682", "Processing menu bar update", &serde_json::json!({"text": text.clone()}), "G");
            debug3!("Processing menu bar update: '{}'", text);
            let attributed = make_attributed_title(&text);
            STATUS_ITEM.with(|cell| {
                if let Some(item) = cell.borrow().as_ref() {
                    if let Some(button) = item.button(mtm) {
                        write_structured_log("lib.rs:690", "About to call setAttributedTitle", &serde_json::json!({}), "H");
                        button.setAttributedTitle(&attributed);
                        write_structured_log("lib.rs:691", "setAttributedTitle called", &serde_json::json!({}), "H");
                        debug3!("Menu bar text updated successfully");
                    } else {
                        write_structured_log("lib.rs:693", "Button not found", &serde_json::json!({}), "G");
                    }
                }
            });
        } else {
            write_structured_log("lib.rs:681", "No update text available", &serde_json::json!({}), "G");
        }
    } else {
        write_structured_log("lib.rs:671", "MainThreadMarker::new() FAILED", &serde_json::json!({}), "G");
    }
    // #region agent log
    write_structured_log("lib.rs:820", "process_menu_bar_update EXIT", &serde_json::json!({"duration_ms": update_start.elapsed().as_millis()}), "CPU4");
    // #endregion
}

fn make_attributed_title(text: &str) -> Retained<NSMutableAttributedString> {
    let ns_text = NSString::from_str(text);
    let attributed = NSMutableAttributedString::from_nsstring(&ns_text);
    let length = ns_text.length();
    let full_range = NSRange { location: 0, length };

    let label_len = text
        .split('\n')
        .next()
        .unwrap_or("")
        .encode_utf16()
        .count();
    let value_len = text
        .split('\n')
        .nth(1)
        .unwrap_or("")
        .encode_utf16()
        .count();
    let label_range = NSRange {
        location: 0,
        length: label_len,
    };
    let value_range = NSRange {
        location: label_len + 1,
        length: value_len,
    };

    let label_font = NSFont::monospacedSystemFontOfSize_weight(8.5, unsafe {
        NSFontWeightRegular
    });
    let value_font = NSFont::monospacedSystemFontOfSize_weight(12.5, unsafe {
        NSFontWeightSemibold
    });
    // Use controlTextColor for menu bar - this works better than labelColor in status bar context
    // labelColor can sometimes turn black in menu bar, so use controlTextColor which adapts properly
    let color = NSColor::controlTextColor();
    let paragraph = NSMutableParagraphStyle::new();
    paragraph.setLineSpacing(-2.0);
    paragraph.setLineHeightMultiple(0.75);
    paragraph.setAlignment(NSTextAlignment::Left);
    paragraph.setDefaultTabInterval(38.0);

    let options: Retained<NSDictionary<NSTextTabOptionKey, AnyObject>> = NSDictionary::new();
    let tab1 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            38.0,
            &options,
        )
    };
    let tab2 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            76.0,
            &options,
        )
    };
    let tab3 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            114.0,
            &options,
        )
    };
    let tab4 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            152.0,
            &options,
        )
    };
    let tabs = NSArray::from_slice(&[&*tab1, &*tab2, &*tab3, &*tab4]);
    paragraph.setTabStops(Some(&tabs));
    let baseline_offset = NSNumber::new_f64(-4.8);

    unsafe {
        attributed.addAttribute_value_range(&NSFontAttributeName, as_any(&*label_font), label_range);
        attributed.addAttribute_value_range(&NSFontAttributeName, as_any(&*value_font), value_range);
        attributed.addAttribute_value_range(
            &NSForegroundColorAttributeName,
            as_any(&*color),
            full_range,
        );
        attributed.addAttribute_value_range(
            &NSParagraphStyleAttributeName,
            as_any(&*paragraph),
            full_range,
        );
        attributed.addAttribute_value_range(
            &NSBaselineOffsetAttributeName,
            as_any(&*baseline_offset),
            full_range,
        );
    }

    attributed
}

fn setup_status_item() {
    let mtm = MainThreadMarker::new().unwrap();
    let status_bar = NSStatusBar::systemStatusBar();
    let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

    let handler_class = click_handler_class();
    debug2!("Creating handler instance from class");
    write_structured_log("lib.rs:864", "About to create handler instance", &serde_json::json!({"class": format!("{:?}", handler_class)}), "J");
    
    // Verify class responds to selector before creating instance
    let action_sel = sel!(onStatusItemClick:);
    let selector_name = action_sel.name().to_string_lossy();
    let class_responds = unsafe {
        let responds: bool = msg_send![handler_class, instancesRespondToSelector: action_sel];
        responds
    };
    debug1!("Handler class responds to selector '{}': {}", selector_name, class_responds);
    write_structured_log("lib.rs:867", "Class selector check", &serde_json::json!({"selector": selector_name, "responds": class_responds}), "J");
    
    let handler: Retained<AnyObject> =
        unsafe { Retained::from_raw(msg_send![handler_class, new]) }
            .expect("click handler");
    debug3!("Handler instance created: {:?}", handler);
    write_structured_log("lib.rs:872", "Handler instance created", &serde_json::json!({"handler": format!("{:p}", &*handler)}), "J");
    
    // Verify instance responds to selector
    let instance_responds = unsafe {
        let responds: bool = msg_send![&*handler, respondsToSelector: action_sel];
        responds
    };
    debug1!("Handler instance responds to selector: {}", instance_responds);
    write_structured_log("lib.rs:877", "Instance selector check", &serde_json::json!({"responds": instance_responds}), "J");
    
    if !instance_responds {
        debug1!("ERROR: Handler instance does NOT respond to selector!");
        write_structured_log("lib.rs:880", "ERROR: Instance does not respond to selector", &serde_json::json!({}), "J");
    }
    
    // CRITICAL: Store handler in thread-local FIRST to keep it alive
    // The button will also retain it when we set it as target, but we keep our own reference
    CLICK_HANDLER.with(|cell| {
        *cell.borrow_mut() = Some(handler.clone());
        debug3!("Handler stored in CLICK_HANDLER thread-local (retained)");
        write_structured_log("lib.rs:875", "Handler stored in CLICK_HANDLER", &serde_json::json!({}), "J");
    });

    // CRITICAL: Do NOT set a menu on the status item if we want button action to work
    // Setting a menu disables the button's action/target behavior
    // Instead, use the button's action directly and handle events properly
    let action = sel!(onStatusItemClick:);
    
    if let Some(button) = status_item.button(mtm) {
        debug2!("Setting up button target and action (NO menu set)...");
        write_structured_log("lib.rs:881", "Setting button target/action (no menu)", &serde_json::json!({"handler": format!("{:p}", &*handler), "action": action.name()}), "J");
        unsafe {
            // Set target and action on the button
            button.setTarget(Some(&*handler));
            button.setAction(Some(action));
            button.setEnabled(true);
            
            // CRITICAL: Use sendActionOn to specify which events trigger the action
            // This is required for NSStatusBarButton to work properly
            // sendActionOn returns the previous mask, we want left mouse up events
            // NSEventMask is a bitmask - use LeftMouseUpMask
            use objc2_app_kit::NSEventMask;
            let event_mask = NSEventMask::LeftMouseUp;
            let _previous_mask = button.sendActionOn(event_mask);
            
            write_structured_log("lib.rs:892", "Button target/action and sendAction set", &serde_json::json!({}), "J");
            debug3!("Button target, action, and sendAction set");
            
            // Verify setup
            if let Some(target) = button.target() {
                debug3!("Button target verified: {:?}", target);
                write_structured_log("lib.rs:920", "Button target verified", &serde_json::json!({"target": format!("{:p}", target)}), "J");
                
                // CRITICAL: Verify target responds to the action selector
                let target_responds = {
                    let responds: bool = msg_send![&*target, respondsToSelector: action];
                    responds
                };
                let selector_name = action.name().to_string_lossy();
                debug1!("Button target responds to action selector '{}': {}", selector_name, target_responds);
                write_structured_log("lib.rs:925", "Target respondsToSelector check", &serde_json::json!({"responds": target_responds, "selector": selector_name}), "J");
                
                if !target_responds {
                    debug1!("ERROR: Button target does NOT respond to action selector!");
                    write_structured_log("lib.rs:928", "ERROR: Target does not respond to selector", &serde_json::json!({}), "J");
                }
            }
            if let Some(set_action) = button.action() {
                debug3!("Button action verified: {:?}", set_action.name());
                write_structured_log("lib.rs:931", "Button action verified", &serde_json::json!({"action": set_action.name()}), "J");
            }
            
            // CRITICAL: Check if button is enabled
            let is_enabled = button.isEnabled();
            debug1!("Button isEnabled: {}", is_enabled);
            write_structured_log("lib.rs:954", "Button enabled check", &serde_json::json!({"enabled": is_enabled}), "J");
            
            // CRITICAL: Try manually sending the action to verify it works
            if let Some(target) = button.target() {
                debug1!("Attempting to manually send action to verify it works...");
                write_structured_log("lib.rs:958", "Manual action send attempt", &serde_json::json!({}), "J");
                let action_sent = {
                    use objc2_app_kit::NSApplication;
                    let app = NSApplication::sharedApplication(mtm);
                    let sent: bool = msg_send![&*app, sendAction: action, to: &*target, from: &*button];
                    sent
                };
                debug1!("Manual sendAction result: {}", action_sent);
                write_structured_log("lib.rs:963", "Manual sendAction result", &serde_json::json!({"sent": action_sent}), "J");
            }
        }
        debug2!("Button target and action set (no menu)");
    } else {
        debug1!("ERROR: Could not get button from status item!");
        write_structured_log("lib.rs:907", "ERROR: Button not found", &serde_json::json!({}), "J");
    }
    
    // Handler is already stored in CLICK_HANDLER above, so it's retained
    // The button should also retain it via setTarget, so we have double retention
    debug3!("Handler retention: stored in CLICK_HANDLER and set as button target");

    STATUS_ITEM.with(|cell| {
        *cell.borrow_mut() = Some(status_item);
    });
    debug2!("Status item setup complete");
    
    // Start automatic menu bar updates by scheduling the first update
    // The handler will reschedule itself every 2 seconds
    if let Some(handler) = CLICK_HANDLER.with(|cell| cell.borrow().clone()) {
        let update_sel = sel!(processMenuBarUpdate:);
        unsafe {
            // Schedule first update after 2 seconds
            let _: () = msg_send![&*handler, performSelector: update_sel, withObject: std::ptr::null_mut::<AnyObject>(), afterDelay: 2.0];
            debug1!("Scheduled automatic menu bar updates (first update in 2 seconds)");
            write_structured_log("lib.rs:997", "Automatic updates scheduled", &serde_json::json!({}), "M");
        }
    } else {
        debug1!("WARNING: Could not get handler for automatic updates");
    }
}

fn click_handler_class() -> &'static AnyClass {
    static REGISTER: OnceLock<&'static AnyClass> = OnceLock::new();
    REGISTER.get_or_init(|| {
        let name = unsafe { CStr::from_bytes_with_nul_unchecked(b"MacStatsStatusHandler\0") };
        debug2!("Creating Objective-C class: {:?}", name);
        let mut builder = ClassBuilder::new(name, NSObject::class()).expect("class already exists");
        
        // Add method to process menu bar updates (called automatically every 2 seconds)
        extern "C-unwind" fn process_menu_bar_update_timer(
            this: &AnyObject,
            _cmd: Sel,
            _sender: *mut AnyObject,
        ) {
            // This is called from Objective-C runtime, we're on the main thread
            // #region agent log
            let timer_start = std::time::Instant::now();
            write_structured_log("lib.rs:1031", "process_menu_bar_update_timer ENTRY", &serde_json::json!({}), "CPU3");
            // #endregion
            process_menu_bar_update();
            // #region agent log
            write_structured_log("lib.rs:1038", "process_menu_bar_update_timer EXIT", &serde_json::json!({"duration_ms": timer_start.elapsed().as_millis()}), "CPU3");
            // #endregion
            
            // Schedule next update in 2 seconds
            let sel = sel!(processMenuBarUpdate:);
            unsafe {
                let _: () = msg_send![this, performSelector: sel, withObject: std::ptr::null_mut::<AnyObject>(), afterDelay: 2.0];
            }
        }
        
        extern "C-unwind" fn on_status_item_click(
            this: &AnyObject,
            _cmd: Sel,
            sender: *mut AnyObject,
        ) {
            // This is called from Objective-C runtime, we're on the main thread
            // CRITICAL: Log immediately to verify this function is called
            write_structured_log("lib.rs:961", "Click handler FUNCTION CALLED", &serde_json::json!({"this": format!("{:p}", this), "sender": format!("{:p}", sender)}), "J");
            debug1!("Click handler called! cmd={:?}, sender={:p}, this={:p}", _cmd, sender, this);
            
            // Note: The menu will show briefly, but that's okay - the action fires
            // We could hide it immediately, but for now let's just let it work
            
            // Process any pending menu bar updates while we're on the main thread
            process_menu_bar_update();
            
            // Get event info immediately while we're on the main thread
            let mtm = match MainThreadMarker::new() {
                Some(mtm) => mtm,
                None => {
                    debug1!("ERROR: Could not get MainThreadMarker!");
                    return;
                }
            };
            
            let app = NSApplication::sharedApplication(mtm);
            let is_right_click = app
                .currentEvent()
                .map(|event: Retained<NSEvent>| {
                    let button_number = event.buttonNumber();
                    debug3!("Event button number: {}", button_number);
                    button_number == 1
                })
                .unwrap_or(false);
            debug2!("Is right click: {}", is_right_click);
            
            if is_right_click {
                debug1!("Showing about panel");
                show_about_panel();
            } else {
                debug1!("Left click - toggling CPU window");
                write_structured_log("lib.rs:1127", "Click handler: about to toggle window", &serde_json::json!({}), "I");
                // We're already on main thread, so we can access the window directly
                if let Some(app_handle) = APP_HANDLE.get() {
                    write_structured_log("lib.rs:1129", "APP_HANDLE found", &serde_json::json!({}), "I");
                    
                    // Check if window exists and is visible
                    if let Some(window) = app_handle.get_window("cpu") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        write_structured_log("lib.rs:1132", "CPU window found", &serde_json::json!({"is_visible": is_visible}), "I");
                        
                        if is_visible {
                            // Window is visible - close it completely to save CPU
                            debug1!("CPU window is visible, closing it completely...");
                            let _ = window.close();
                            write_structured_log("lib.rs:1138", "Window closed completely", &serde_json::json!({}), "I");
                        } else {
                            // Window exists but is hidden - show it
                            debug1!("CPU window exists but is hidden, showing it...");
                            write_structured_log("lib.rs:1141", "Before set_always_on_top", &serde_json::json!({}), "I");
                            let _ = window.set_always_on_top(true);
                            write_structured_log("lib.rs:1142", "Before show", &serde_json::json!({}), "I");
                            let _ = window.show();
                            write_structured_log("lib.rs:1143", "Before set_focus", &serde_json::json!({}), "I");
                            let _ = window.set_focus();
                            write_structured_log("lib.rs:1144", "Before unminimize", &serde_json::json!({}), "I");
                            let _ = window.unminimize();
                            debug1!("Window show commands sent");
                            write_structured_log("lib.rs:1147", "Window show commands completed", &serde_json::json!({}), "I");
                        }
                    } else {
                        // Window doesn't exist - create and show it
                        debug1!("CPU window doesn't exist, creating it...");
                        write_structured_log("lib.rs:1151", "Creating new CPU window", &serde_json::json!({}), "I");
                        create_cpu_window(app_handle);
                    }
                } else {
                    write_structured_log("lib.rs:1156", "APP_HANDLE not available", &serde_json::json!({}), "I");
                    debug1!("APP_HANDLE not available!");
                }
            }
        }
        unsafe {
            let action_sel = sel!(onStatusItemClick:);
            debug2!("Adding method: {:?}", action_sel.name());
            builder.add_method(
                action_sel,
                on_status_item_click as extern "C-unwind" fn(_, _, _),
            );
            
            let update_sel = sel!(processMenuBarUpdate:);
            debug2!("Adding method: {:?}", update_sel.name());
            builder.add_method(
                update_sel,
                process_menu_bar_update_timer as extern "C-unwind" fn(_, _, _),
            );
        }
        let registered_class = builder.register();
        debug2!("Objective-C class registered: {:?}", registered_class);
        
        // CRITICAL: Verify the class responds to the selector
        let action_sel = sel!(onStatusItemClick:);
        let selector_name = action_sel.name().to_string_lossy();
        let responds = unsafe {
            let responds: bool = msg_send![registered_class, instancesRespondToSelector: action_sel];
            responds
        };
        debug1!("Class responds to selector '{}': {}", selector_name, responds);
        write_structured_log("lib.rs:1014", "Class selector verification", &serde_json::json!({"selector": selector_name, "responds": responds}), "J");
        
        if !responds {
            debug1!("ERROR: Class does NOT respond to selector! Method registration may have failed!");
            write_structured_log("lib.rs:1019", "ERROR: Class does not respond to selector", &serde_json::json!({}), "J");
        }
        
        registered_class
    })
}

// show_cpu_window is now inlined in the click handler since we're already on main thread

fn show_about_panel() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    let name = NSString::from_str("mac_stats");
    let version = NSString::from_str(env!("CARGO_PKG_VERSION"));
    let build = NSString::from_str(BUILD_DATE);
    let authors = NSString::from_str(env!("CARGO_PKG_AUTHORS"));
    let credits_text = NSString::from_str(&format!(
        "Author: {}\nBuild: {}",
        authors, BUILD_DATE
    ));
    let credits = NSAttributedString::from_nsstring(&credits_text);

    let keys = unsafe {
        [
            &*NSAboutPanelOptionApplicationName,
            &*NSAboutPanelOptionApplicationVersion,
            &*NSAboutPanelOptionVersion,
            &*NSAboutPanelOptionCredits,
        ]
    };
    let values: [&AnyObject; 4] = [
        as_any(&*name),
        as_any(&*version),
        as_any(&*build),
        as_any(&*credits),
    ];
    let options = NSMutableDictionary::from_slices(&keys, &values);

    unsafe {
        app.orderFrontStandardAboutPanelWithOptions(options.as_ref());
    }
}

fn create_cpu_window(app_handle: &tauri::AppHandle) {
    debug1!("Creating CPU window...");
    write_structured_log("lib.rs:1165", "create_cpu_window ENTRY", &serde_json::json!({}), "I");
    
    let cpu_window = WindowBuilder::new(
        app_handle,
        "cpu",
        WindowUrl::App("cpu.html".into()),
    )
    .title("CPU")
    .visible(true)  // Show immediately when created
    .inner_size(400.0, 700.0)
    .resizable(true)
    .always_on_top(true)
    .build();
    
    match cpu_window {
        Ok(window) => {
            debug1!("CPU window created successfully");
            write_structured_log("lib.rs:1178", "CPU window created successfully", &serde_json::json!({}), "I");
            let _ = window.set_always_on_top(true);
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.unminimize();
            debug1!("CPU window shown and focused");
            write_structured_log("lib.rs:1183", "CPU window shown and focused", &serde_json::json!({}), "I");
        },
        Err(e) => {
            debug1!("ERROR: Failed to create CPU window: {:?}", e);
            write_structured_log("lib.rs:1186", "ERROR: Failed to create CPU window", &serde_json::json!({"error": format!("{:?}", e)}), "I");
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
                
                loop {
                    debug3!("Update loop: getting metrics...");
                    let metrics = get_metrics();
                    let text = build_status_text(&metrics);
                    debug2!("Update loop: status text: '{}'", text);
                    
                    // Store update in static variable
                    if let Ok(mut pending) = MENU_BAR_TEXT.lock() {
                        *pending = Some(text);
                        debug3!("Menu bar update stored");
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