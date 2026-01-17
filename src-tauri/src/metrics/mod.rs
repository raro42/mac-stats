//! System metrics collection module
//! 
//! This module provides functions to collect and cache system metrics:
//! - CPU, RAM, Disk, GPU usage
//! - Temperature readings (via SMC)
//! - CPU frequency (via IOReport)
//! - Power consumption (CPU/GPU)
//! - Process information
//! 
//! All metrics are cached to reduce system load and improve performance.

use std::process::Command;
use sysinfo::{Disks, System};
use macsmc::Smc;
use tauri::Manager;

use crate::state::*;
use crate::logging::write_structured_log;

// Import debug macros - they're exported at crate root via #[macro_export]
// These now use tracing internally but maintain the same interface
#[allow(unused_imports)]
use crate::{debug1, debug2, debug3};

#[derive(serde::Serialize, Clone)]
pub struct SystemMetrics {
    pub cpu: f32,
    pub gpu: f32,
    pub ram: f32,
    pub disk: f32,
}

#[derive(serde::Serialize, Clone)]
pub struct ProcessUsage {
    pub name: String,
    pub cpu: f32,
}

#[derive(serde::Serialize)]
pub struct CpuDetails {
    pub usage: f32,
    pub temperature: f32,
    pub frequency: f32,
    pub p_core_frequency: f32,
    pub e_core_frequency: f32,
    pub cpu_power: f32,
    pub gpu_power: f32,
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
    pub uptime_secs: u64,
    pub top_processes: Vec<ProcessUsage>,
    pub chip_info: String,
    // Access flags - true if we can read the value, false if access is denied
    pub can_read_temperature: bool,
    pub can_read_frequency: bool,
    pub can_read_cpu_power: bool,
    pub can_read_gpu_power: bool,
}

/// Get chip information (cached)
pub fn get_chip_info() -> String {
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
                            // Format: "Apple M3 · 16 cores" (using middle dot · not bullet •)
                            let mut info = chip_type.to_string();
                            if !num_procs.is_empty() {
                                // number_processors format: "proc 16:12:4" (total:performance:efficiency)
                                let num_procs_clean = num_procs.strip_prefix("proc ").unwrap_or(num_procs);
                                let parts: Vec<&str> = num_procs_clean.split(':').collect();
                                
                                // Try to get total cores
                                let total_cores = if parts.len() >= 3 {
                                    // Format: "16:12:4" - first number is total cores
                                    parts.get(0).and_then(|s| s.parse::<u32>().ok())
                                        .or_else(|| {
                                            // Fallback: add P + E cores if first number fails
                                            let p_cores = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                                            let e_cores = parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                                            if p_cores > 0 || e_cores > 0 {
                                                Some(p_cores + e_cores)
                                            } else {
                                                None
                                            }
                                        })
                                } else if parts.len() == 1 {
                                    // Single number (total cores)
                                    parts[0].parse::<u32>().ok()
                                } else {
                                    None
                                };
                                
                                if let Some(total) = total_cores {
                                    if total > 0 {
                                        info.push_str(&format!(" · {} cores", total));
                                        debug2!("Chip info formatted: '{}' (from chip_type='{}', num_procs='{}')", info, chip_type, num_procs);
                                    } else {
                                        debug2!("Chip info: total_cores is 0, not adding core count");
                                    }
                                } else {
                                    debug2!("Chip info: could not parse total_cores from '{}'", num_procs);
                                }
                            } else {
                                debug2!("Chip info: num_procs is empty, chip_type='{}'", chip_type);
                            }
                            debug2!("Chip info returning: '{}'", info);
                            return info;
                        } else {
                            debug2!("Chip info: chip_type is empty");
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

pub fn get_gpu_usage() -> f32 {
    // GPU usage reading is expensive (ioreg commands) - return 0 for now to save CPU
    // TODO: Cache or optimize if GPU usage is needed
    0.0
}

pub fn can_read_temperature() -> bool {
    // Check if we have a valid cached temperature (indicates SMC access works)
    // This is more efficient than checking SMC directly
    if let Ok(cache) = TEMP_CACHE.try_lock() {
        if let Some((temp, timestamp)) = cache.as_ref() {
            // If we have a recent temperature reading, SMC access works
            // Increased from 10s to 20s to match the 15s reading frequency
            if *temp > 0.0 && timestamp.elapsed().as_secs() < 20 {
                debug3!("can_read_temperature: true (from TEMP_CACHE with temp={:.1}°C)", *temp);
                return true;
            }
        }
    }
    
    // Fallback: check ACCESS_CACHE (one-time check, cached permanently)
    if let Ok(mut cache) = ACCESS_CACHE.try_lock() {
        if let Some((temp, _, _, _)) = cache.as_ref() {
            debug3!("can_read_temperature: {} (from ACCESS_CACHE)", *temp);
            return *temp;
        }
        
        // First time check - try SMC (only once)
        // Even if SMC returns 0.0, the connection succeeded, so we "can read" it
        // (it just means the Mac model doesn't expose temperature via standard keys)
        debug2!("can_read_temperature: First time check - trying SMC connection...");
        let can_read = if let Ok(mut smc) = Smc::connect() {
            // Connection succeeded - we can attempt to read (even if it returns 0.0)
            match smc.cpu_temperature() {
                Ok(_) => {
                    // SMC read succeeded (even if temp is 0.0, the read worked)
                    debug2!("SMC connection and read succeeded - can_read_temperature=true");
                    true
                },
                Err(e) => {
                    // SMC read failed
                    debug2!("SMC read failed: {:?} - can_read_temperature=false", e);
                    false
                }
            }
        } else {
            // SMC connection failed - can't read
            debug2!("SMC connection failed - can_read_temperature=false");
            false
        };
        
        // Cache the result permanently
        if let Some((_, freq, cpu_power, gpu_power)) = cache.as_ref() {
            *cache = Some((can_read, *freq, *cpu_power, *gpu_power));
        } else {
            *cache = Some((can_read, false, false, false));
        }
        
        debug2!("can_read_temperature: Cached result: {}", can_read);
        can_read
    } else {
        // Lock held - return false (non-blocking)
        debug3!("can_read_temperature: ACCESS_CACHE locked, returning false");
        false
    }
}

// Get nominal CPU frequency using sysctl (cheap, no sudo required)
// This gives base/nominal frequency, not dynamic frequency
pub(crate) fn get_nominal_frequency() -> f32 {
    *NOMINAL_FREQ.get_or_init(|| {
        // Try hw.tbfrequency * kern.clockrate.hz approach (works on Apple Silicon)
        let tbfreq_output = Command::new("sh")
            .arg("-c")
            .arg("sysctl -n hw.tbfrequency 2>/dev/null || echo '0'")
            .output();
        
        // kern.clockrate.hz doesn't work directly - need to parse the struct
        let clockrate_output = Command::new("sh")
            .arg("-c")
            .arg("sysctl kern.clockrate 2>/dev/null | grep -o 'hz = [0-9]*' | head -1 | grep -o '[0-9]*' || echo '0'")
            .output();
        
        // Try standard cpufrequency (works on Intel)
        let cpufreq_output = Command::new("sh")
            .arg("-c")
            .arg("sysctl -n hw.cpufrequency_max 2>/dev/null || sysctl -n hw.cpufrequency 2>/dev/null || echo '0'")
            .output();
        
        // Try tbfrequency * clockrate first (Apple Silicon)
        // Formula: cpu_freq_hz = hw.tbfrequency * kern.clockrate.hz
        // This gives nominal/base frequency, not dynamic frequency
        if let (Ok(tb), Ok(clock)) = (tbfreq_output, clockrate_output) {
            if tb.status.success() && clock.status.success() {
                let tb_str = String::from_utf8_lossy(&tb.stdout).trim().to_string();
                let clock_str = String::from_utf8_lossy(&clock.stdout).trim().to_string();
                debug3!("tbfrequency: '{}', clockrate.hz: '{}'", tb_str, clock_str);
                if let (Ok(tb_hz), Ok(clock_hz)) = (tb_str.parse::<f64>(), clock_str.parse::<f64>()) {
                    debug3!("Parsed: tb_hz={}, clock_hz={}", tb_hz, clock_hz);
                    if tb_hz > 0.0 && clock_hz > 0.0 {
                        // Formula: tbfrequency * clockrate.hz = CPU frequency in Hz
                        let freq_hz = tb_hz * clock_hz;
                        let freq_ghz = (freq_hz / 1_000_000_000.0) as f32;
                        debug3!("Computed: freq_hz={}, freq_ghz={:.2}", freq_hz, freq_ghz);
                        if freq_ghz > 0.1 && freq_ghz < 10.0 {
                            debug2!("Nominal frequency computed: {:.2} GHz (tbfreq * clockrate.hz)", freq_ghz);
                            return freq_ghz;
                        } else {
                            debug3!("Computed frequency {:.2} GHz is out of range (0.1-10.0)", freq_ghz);
                        }
                    } else {
                        debug3!("tb_hz or clock_hz is zero: tb_hz={}, clock_hz={}", tb_hz, clock_hz);
                    }
                } else {
                    debug3!("Failed to parse tbfrequency or clockrate as numbers");
                }
            } else {
                debug3!("sysctl commands failed: tb.status={:?}, clock.status={:?}", tb.status, clock.status);
            }
        } else {
            debug3!("Failed to execute sysctl commands for tbfrequency/clockrate");
        }
        
        // Fallback to standard cpufrequency (Intel)
        if let Ok(output) = cpufreq_output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let trimmed = stdout.trim();
                if !trimmed.is_empty() && trimmed != "0" {
                    if let Ok(freq_hz) = trimmed.parse::<f64>() {
                        if freq_hz > 0.0 {
                            let freq_ghz = (freq_hz / 1_000_000_000.0) as f32;
                            if freq_ghz > 0.1 && freq_ghz < 10.0 {
                                debug2!("Nominal frequency from sysctl: {:.2} GHz", freq_ghz);
                                return freq_ghz;
                            }
                        }
                    }
                }
            }
        }
        
        debug2!("Could not determine nominal frequency, using 0.0");
        0.0
    })
}

/// Check if frequency reading is available
/// Currently unused but kept for potential future use.
#[allow(dead_code)]
pub fn can_read_frequency() -> bool {
    // Check if we have a valid cached frequency (indicates frequency reading works)
    if let Ok(cache) = FREQ_CACHE.try_lock() {
        if let Some((freq, timestamp)) = cache.as_ref() {
            // If we have a recent frequency reading, frequency access works
            if *freq > 0.0 && timestamp.elapsed().as_secs() < 20 {
                debug3!("can_read_frequency: true (from FREQ_CACHE with freq={:.2} GHz)", *freq);
                return true;
            }
        }
    }
    
    // Fallback: check ACCESS_CACHE (one-time check, cached permanently)
    if let Ok(mut cache) = ACCESS_CACHE.try_lock() {
        if let Some((_, freq, _, _)) = cache.as_ref() {
            debug3!("can_read_frequency: {} (from ACCESS_CACHE)", *freq);
            return *freq;
        }
        
        // First time check - try to compute nominal frequency (cheap, no sudo)
        debug2!("can_read_frequency: First time check - trying nominal frequency computation...");
        let nominal = get_nominal_frequency();
        let can_read = nominal > 0.0;
        
        // Cache the result permanently
        if let Some((temp, _, cpu_power, gpu_power)) = cache.as_ref() {
            *cache = Some((*temp, can_read, *cpu_power, *gpu_power));
        } else {
            *cache = Some((false, can_read, false, false));
        }
        
        debug2!("can_read_frequency: Cached result: {} (nominal={:.2} GHz)", can_read, nominal);
        can_read
    } else {
        // Lock held - return false (non-blocking)
        debug3!("can_read_frequency: ACCESS_CACHE locked, returning false");
        false
    }
}

#[allow(dead_code)]
pub fn can_read_cpu_power() -> bool {
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
pub fn can_read_gpu_power() -> bool {
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

#[tauri::command]
pub fn get_metrics() -> SystemMetrics {
    debug3!("get_metrics() called");
    
    // Fast metrics refresh: every 2 seconds for menu bar responsiveness
    // This is cheap because we use cached values and only refresh when needed
    // Use try_lock to avoid blocking
    let should_refresh = match LAST_SYSTEM_REFRESH.try_lock() {
        Ok(mut last_refresh) => {
            let now = std::time::Instant::now();
            let should = last_refresh.map(|lr| now.duration_since(lr).as_secs() >= 2).unwrap_or(true);
            if should {
                *last_refresh = Some(now);
                debug3!("Refresh allowed (2 seconds passed)");
            } else {
                debug3!("Refresh skipped (less than 2 seconds since last)");
            }
            should
        },
        Err(_) => {
            // Lock held - skip refresh to avoid blocking
            debug3!("Refresh skipped (lock held)");
            false
        }
    };
    
    // Use try_lock ONCE - if locked, return cached values immediately (no retry loop)
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
                debug3!("Refreshing CPU usage and memory");
                sys.refresh_cpu_usage();
                sys.refresh_memory();
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
    
    // Use try_lock ONCE for disk - if locked, return cached value immediately
    let disk_usage = match DISKS.try_lock() {
        Ok(mut disks) => {
            if disks.is_none() {
                debug3!("Creating new Disks instance (will refresh once)");
                let mut new_disks = Disks::new();
                // Only refresh once on creation (expensive operation)
                if should_refresh {
                    debug3!("Initial disk refresh (one time only)");
                    new_disks.refresh(false);
                }
                *disks = Some(new_disks);
            }
            debug3!("Reading disk info (no refresh)");
            let disks = disks.as_ref().unwrap();
            if let Some(disk) = disks.list().first() {
                let total = disk.total_space();
                let available = disk.available_space();
                if total > 0 {
                    let disk_usage = ((total - available) as f32 / total as f32) * 100.0;
                    debug3!("Disk usage: {}% (total: {}, available: {})", disk_usage, total, available);
                    disk_usage
                } else {
                    0.0
                }
            } else {
                0.0
            }
        },
        Err(_) => {
            // Lock held - return zero immediately, no retry
            debug1!("WARNING: DISKS mutex is locked, using 0% for disk");
            0.0
        }
    };
    
    let gpu_usage = get_gpu_usage();
    debug3!("GPU usage: {}%", gpu_usage);
    
    let metrics = SystemMetrics {
        cpu: cpu_usage,
        gpu: gpu_usage,
        ram: ram_usage,
        disk: disk_usage,
    };
    
    debug3!("Returning metrics: CPU={}%, GPU={}%, RAM={}%, DISK={}%", 
        metrics.cpu, metrics.gpu, metrics.ram, metrics.disk);
    
    metrics
}

#[tauri::command]
pub fn get_cpu_details() -> CpuDetails {
    // STEP 5: Rate limiting - prevent get_cpu_details from being called too frequently
    // Even if frontend calls it more often, we'll throttle to max once per 2 seconds
    let should_allow_call = match crate::state::LAST_CPU_DETAILS_CALL.try_lock() {
        Ok(mut last_call) => {
            let now = std::time::Instant::now();
            let should = last_call.as_ref()
                .map(|lc| now.duration_since(*lc).as_secs_f64() >= 2.0)
                .unwrap_or(true);
            if should {
                *last_call = Some(now);
                true
            } else {
                false
            }
        },
        Err(_) => {
            // Lock held - allow call (non-blocking)
            true
        }
    };
    
    if !should_allow_call {
        debug3!("get_cpu_details() rate limited - returning cached values");
        // Return cached values immediately without doing any work
        // This prevents CPU spikes from excessive calls
        let (usage, load, uptime_secs) = match crate::state::SYSTEM.try_lock() {
            Ok(sys) => {
                if let Some(sys) = sys.as_ref() {
                    (sys.global_cpu_usage(), sysinfo::System::load_average(), sysinfo::System::uptime())
                } else {
                    (0.0, sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 }, 0)
                }
            },
            Err(_) => (0.0, sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 }, 0),
        };
        
        // Return cached values only
        let (temperature, frequency, p_core_frequency, e_core_frequency) = (
            crate::state::TEMP_CACHE.try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(t, _)| *t))
                .unwrap_or(0.0),
            crate::state::FREQ_CACHE.try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(f, _)| *f))
                .unwrap_or(crate::metrics::get_nominal_frequency()),
            crate::state::P_CORE_FREQ_CACHE.try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(f, _)| *f))
                .unwrap_or(0.0),
            crate::state::E_CORE_FREQ_CACHE.try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(f, _)| *f))
                .unwrap_or(0.0),
        );
        
        let processes = crate::state::PROCESS_CACHE.try_lock()
            .ok()
            .and_then(|c| c.as_ref().map(|(p, _)| p.clone()))
            .unwrap_or_default();
        
        return CpuDetails {
            usage,
            temperature,
            frequency,
            p_core_frequency,
            e_core_frequency,
            cpu_power: 0.0,
            gpu_power: 0.0,
            load_1: load.one,
            load_5: load.five,
            load_15: load.fifteen,
            uptime_secs,
            top_processes: processes,
            chip_info: crate::metrics::get_chip_info(),
            can_read_temperature: crate::metrics::can_read_temperature(),
            can_read_frequency: crate::metrics::can_read_frequency(),
            can_read_cpu_power: false,
            can_read_gpu_power: false,
        };
    }
    
    debug3!("get_cpu_details() called");
    
    // CRITICAL: Only collect processes if CPU window exists and is visible to save CPU
    // Check window existence and visibility before doing expensive process collection
    // If window was closed (destroyed), get_window returns None, so no processes collected
    let should_collect_processes = APP_HANDLE.get()
        .and_then(|app_handle| {
            app_handle.get_window("cpu").and_then(|window| {
                // Window exists - check if it's visible
                window.is_visible().ok().filter(|&visible| visible)
            })
        })
        .is_some();
    
    // CRITICAL: Use try_lock ONCE - if locked, return defaults immediately
    // This prevents blocking the main thread when the window opens
    let (usage, load, uptime_secs, top_processes) = match SYSTEM.try_lock() {
        Ok(mut sys) => {
            if sys.is_none() {
                // Don't create System here - it's expensive and blocks
                // Return defaults and let background thread create it
                debug1!("WARNING: SYSTEM is None in get_cpu_details, returning defaults");
                write_structured_log("lib.rs:658", "SYSTEM is None, returning defaults", &serde_json::json!({}), "L");
                (0.0, sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 }, 0, Vec::new())
            } else {
                let sys = sys.as_mut().unwrap();
                // CRITICAL: Don't refresh here - it's expensive and blocks
                // Just read existing values without refreshing
                let usage = sys.global_cpu_usage();
                let load = sysinfo::System::load_average();
                let uptime_secs = sysinfo::System::uptime();
                debug3!("System uptime: {} seconds ({} days, {} hours, {} minutes)", 
                    uptime_secs, 
                    uptime_secs / 86400,
                    (uptime_secs % 86400) / 3600,
                    (uptime_secs % 3600) / 60);
                
                // Only collect processes if window is visible (saves CPU when window is closed)
                let processes = if should_collect_processes {
                    // STEP 4: Cache process list for 20 seconds to avoid expensive refresh on every call
                    // Increased from 15s to 20s to further reduce CPU usage
                    // Check cache first - only refresh if cache is empty or older than 20 seconds
                    let should_refresh_processes = match PROCESS_CACHE.try_lock() {
                        Ok(cache) => {
                            cache.as_ref()
                                .map(|(_, timestamp)| timestamp.elapsed().as_secs() >= 20)
                                .unwrap_or(true) // Cache is empty, need to refresh
                        },
                        Err(_) => false, // Lock held, use cached value if available
                    };
                    
                    if should_refresh_processes {
                        // STEP 4: Refresh processes (expensive operation, but cached for 20 seconds)
                        // This is necessary because processes() returns empty if not refreshed
                        use sysinfo::ProcessesToUpdate;
                        sys.refresh_processes(ProcessesToUpdate::All, true);
                        
                        // Collect ALL processes first (HashMap iteration order is undefined)
                        // Then sort by CPU usage to get the actual top processes
                        let mut processes: Vec<ProcessUsage> = sys
                            .processes()
                            .values()
                            .map(|proc| ProcessUsage {
                                name: proc.name().to_string_lossy().to_string(),
                                cpu: proc.cpu_usage(),
                            })
                            .collect();
                        
                        // Sort by CPU usage (descending) to get actual top processes
                        processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
                        
                        // Take top 8 after sorting
                        processes.truncate(8);
                        
                        // Update cache
                        if let Ok(mut cache) = PROCESS_CACHE.try_lock() {
                            *cache = Some((processes.clone(), std::time::Instant::now()));
                            debug2!("Process cache updated (refreshed from system)");
                        }
                        
                        processes
                    } else {
                        // Use cached process list
                        match PROCESS_CACHE.try_lock() {
                            Ok(cache) => {
                                if let Some((procs, _)) = cache.as_ref() {
                                    debug2!("Using cached process list ({} processes)", procs.len());
                                    procs.clone()
                                } else {
                                    debug2!("Process cache is empty, returning empty list");
                                    Vec::new()
                                }
                            },
                            Err(_) => {
                                debug2!("Process cache lock held, returning empty list");
                                Vec::new()
                            }
                        }
                    }
                } else {
                    // Window is not visible - return empty process list to save CPU
                    debug3!("Window not visible, skipping process collection");
                    Vec::new()
                };
                
                (usage, load, uptime_secs, processes)
            }
        },
        Err(_) => {
            // Lock is held - return defaults immediately, don't retry
            debug1!("WARNING: SYSTEM mutex locked in get_cpu_details, returning defaults immediately");
            write_structured_log("lib.rs:697", "SYSTEM locked, returning defaults", &serde_json::json!({}), "L");
            (0.0, sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 }, 0, Vec::new())
        }
    };

    // CRITICAL: Use cached values or defaults - don't call expensive functions
    // SMC calls and other operations can block the main thread
    // Use try_lock for cache access too
    let (temperature, frequency, p_core_frequency, e_core_frequency, cpu_power, gpu_power, chip_info, can_read_temperature, can_read_frequency, can_read_cpu_power, can_read_gpu_power) = {
        // Try to get cached access flags without blocking
        let (_can_read_temp, can_read_freq, can_read_cpu_p, can_read_gpu_p) = match ACCESS_CACHE.try_lock() {
            Ok(mut access_cache) => {
                if let Some(cached) = access_cache.as_ref() {
                    *cached
                } else {
                    // First time - use defaults, don't check (expensive)
                    let result = (false, false, false, false);
                    *access_cache = Some(result);
                    result
                }
            },
            Err(_) => {
                // Cache locked - return defaults
                (false, false, false, false)
            }
        };
        
        // CRITICAL: Read temperature from cache (updated by background thread)
        // Non-blocking read - returns 0.0 if cache is locked or stale
        // Cache is valid for up to 20 seconds (background thread updates every 15 seconds)
        let temperature = match TEMP_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((temp, timestamp)) = cache.as_ref() {
                    // Only use cached value if it's fresh (less than 20 seconds old)
                    // Increased from 10s to 20s to match the 15s reading frequency
                    if timestamp.elapsed().as_secs() < 20 {
                        *temp
                    } else {
                        debug3!("Temperature cache is stale ({}s old), using 0.0", timestamp.elapsed().as_secs());
                        0.0
                    }
                } else {
                    0.0
                }
            },
            Err(_) => {
                // Cache locked - return 0.0 (non-blocking)
                0.0
            }
        };
        
        // Check if we can read temperature (uses efficient cache check)
        let can_read_temp = can_read_temperature();
        
        // CRITICAL: Read frequency from cache (updated by background thread)
        // Non-blocking read - returns nominal frequency if cache is locked or stale
        // Cache is valid for up to 35 seconds (background thread updates every 30 seconds)
        let frequency = match FREQ_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((freq, timestamp)) = cache.as_ref() {
                    // Only use cached value if it's fresh (less than 35 seconds old)
                    if timestamp.elapsed().as_secs() < 35 {
                        *freq
                    } else {
                        // Cache is stale, fallback to nominal
                        debug3!("Frequency cache is stale ({}s old), using nominal frequency", timestamp.elapsed().as_secs());
                        get_nominal_frequency()
                    }
                } else {
                    // No cached value, use nominal
                    get_nominal_frequency()
                }
            },
            Err(_) => {
                // Cache locked - return nominal frequency (non-blocking)
                get_nominal_frequency()
            }
        };
        
        // Read P-core and E-core frequencies from cache
        let p_core_frequency = match P_CORE_FREQ_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((freq, timestamp)) = cache.as_ref() {
                    if timestamp.elapsed().as_secs() < 35 {
                        *freq
                    } else {
                        get_nominal_frequency() // Fallback to nominal if stale
                    }
                } else {
                    get_nominal_frequency()
                }
            },
            Err(_) => get_nominal_frequency(),
        };
        
        let e_core_frequency = match E_CORE_FREQ_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((freq, timestamp)) = cache.as_ref() {
                    if timestamp.elapsed().as_secs() < 35 {
                        *freq
                    } else {
                        get_nominal_frequency() // Fallback to nominal if stale
                    }
                } else {
                    get_nominal_frequency()
                }
            },
            Err(_) => get_nominal_frequency(),
        };
        
        // Use cached chip info or default - ensure it's initialized by calling get_chip_info()
        let chip = get_chip_info();
        
        // Return cached temperature, frequency, and defaults for other expensive values
        (temperature, frequency, p_core_frequency, e_core_frequency, 0.0, 0.0, chip, can_read_temp, can_read_freq, can_read_cpu_p, can_read_gpu_p)
    };

    // Log data being sent to frontend for debugging
    debug2!("get_cpu_details returning: temperature={:.1}°C, frequency={:.2} GHz, can_read_temperature={}, can_read_frequency={}", temperature, frequency, can_read_temperature, can_read_frequency);
    
    CpuDetails {
        usage,
        temperature,
        frequency,
        p_core_frequency,
        e_core_frequency,
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
