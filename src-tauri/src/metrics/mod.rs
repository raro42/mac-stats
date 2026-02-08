//! System metrics collection module
//!
//! This module provides functions to collect and cache system metrics:
//! - CPU, RAM, Disk, GPU usage
//! - Temperature readings (via SMC)
//! - CPU frequency (via IOReport)
//! - Power consumption (CPU/GPU)
//! - Process information
//! - Metrics history with adaptive downsampling
//!
//! All metrics are cached to reduce system load and improve performance.

pub mod history;

use std::process::Command;
use sysinfo::{Disks, System};
use macsmc::Smc;
use tauri::Manager;
use battery::{Manager as BatteryManager, State};

use crate::state::*;
use crate::logging::write_structured_log;

// Import debug macros - they're exported at crate root via #[macro_export]
// These now use tracing internally but maintain the same interface
#[allow(unused_imports)]
use crate::{debug1, debug2, debug3};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemMetrics {
    pub cpu: f32,
    pub gpu: f32,
    pub ram: f32,
    pub disk: f32,
}

impl SystemMetrics {
    /// Check if metrics are valid (not all zeros for critical metrics)
    /// Returns false if CPU, GPU, and RAM are all 0% (invalid state)
    /// This can happen during initialization or when locks are held
    pub fn is_valid(&self) -> bool {
        // If CPU, GPU, and RAM are all 0%, this is invalid data
        // Disk can be 0% legitimately, so we don't check it
        let critical_metrics_zero = self.cpu == 0.0 && self.gpu == 0.0 && self.ram == 0.0;
        !critical_metrics_zero
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProcessUsage {
    pub name: String,
    pub cpu: f32,
    pub pid: u32,
}

#[derive(serde::Serialize)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub parent_pid: Option<u32>,
    pub parent_name: Option<String>,
    pub start_time: u64,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub effective_user_id: Option<String>,
    pub effective_user_name: Option<String>,
    pub memory: u64,
    pub virtual_memory: u64,
    pub disk_read: u64,
    pub disk_written: u64,
    pub total_cpu_time: u64, // Total CPU time in milliseconds
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
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
    pub battery_level: f32, // Battery level as percentage (0-100), or -1.0 if not available
    pub is_charging: bool,  // True if battery is charging, false if discharging or no battery
    pub has_battery: bool,  // True if device has a battery
}

/// Get chip information (cached)
pub fn get_chip_info() -> String {
    // Cache chip info - only fetch once
    CHIP_INFO_CACHE.get_or_init(|| {
        // Get chip information from system_profiler (JSON format)
        let output = Command::new("/usr/sbin/system_profiler")
            .arg("SPHardwareDataType")
            .arg("-json")
            .stderr(std::process::Stdio::null())
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
                                        debug3!("Chip info formatted: '{}' (from chip_type='{}', num_procs='{}')", info, chip_type, num_procs);
                                    } else {
                                        debug3!("Chip info: total_cores is 0, not adding core count");
                                    }
                                } else {
                                    debug3!("Chip info: could not parse total_cores from '{}'", num_procs);
                                }
                            } else {
                                debug3!("Chip info: num_procs is empty, chip_type='{}'", chip_type);
                            }
                            debug3!("Chip info returning: '{}'", info);
                            return info;
                        } else {
                            debug3!("Chip info: chip_type is empty");
                        }
                    }
                }
            }
        }
        
        // Fallback: try sysctl for Intel Macs
        let output = Command::new("/usr/sbin/sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .stderr(std::process::Stdio::null())
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
    // Check cache first - GPU usage reading is expensive, so we cache for 2 seconds
    if let Ok(cache) = GPU_USAGE_CACHE.try_lock() {
        if let Some((usage, timestamp)) = cache.as_ref() {
            // Return cached value if less than 2 seconds old
            if timestamp.elapsed().as_secs() < 2 {
                debug3!("GPU usage from cache: {}%", usage);
                return *usage;
            }
        }
    }
    
    // Cache miss or expired - read GPU usage
    // On macOS, GPU utilization can be read from ioreg
    // Try reading from IOGPUWrangler or AGXAccelerator
    let gpu_usage = read_gpu_usage_from_system();
    
    // Update cache
    if let Ok(mut cache) = GPU_USAGE_CACHE.try_lock() {
        *cache = Some((gpu_usage, std::time::Instant::now()));
        debug3!("GPU usage updated: {}%", gpu_usage);
    }
    
    gpu_usage
}

/// Read GPU usage from system (ioreg or other methods)
/// Returns GPU utilization as a percentage (0.0-100.0)
fn read_gpu_usage_from_system() -> f32 {
    // Method 1: Try AGXAccelerator (Apple Silicon GPUs)
    // This is the most reliable method on Apple Silicon Macs
    // The PerformanceStatistics dictionary contains "Device Utilization %"
    let output = Command::new("/usr/sbin/ioreg")
        .arg("-r")
        .arg("-d")
        .arg("1")
        .arg("-w")
        .arg("0")
        .arg("-c")
        .arg("AGXAccelerator")
        .stderr(std::process::Stdio::null())
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug3!("ioreg AGXAccelerator output length: {} bytes", stdout.len());
                
                // Look for "Device Utilization %" in PerformanceStatistics
                // Format: "Device Utilization %"=22 (within a JSON-like dictionary)
                for line in stdout.lines() {
                    // Look for Device Utilization % (most accurate)
                    if line.contains("Device Utilization %") {
                        debug3!("Found 'Device Utilization %' in line: {}", line);
                        if let Some(percent) = extract_percentage_after_key(line, "Device Utilization %") {
                            if percent >= 0.0 && percent <= 100.0 {
                                debug3!("GPU usage from ioreg (Device Utilization %): {}%", percent);
                                return percent;
                            } else {
                                debug3!("GPU usage value {}% is out of range (0-100)", percent);
                            }
                        } else {
                            debug3!("Failed to extract percentage from line containing 'Device Utilization %'");
                        }
                    }
                    // Fallback to Renderer Utilization % if Device Utilization not found
                    if line.contains("Renderer Utilization %") {
                        debug3!("Found 'Renderer Utilization %' in line: {}", line);
                        if let Some(percent) = extract_percentage_after_key(line, "Renderer Utilization %") {
                            if percent >= 0.0 && percent <= 100.0 {
                                debug3!("GPU usage from ioreg (Renderer Utilization %): {}%", percent);
                                return percent;
                            }
                        }
                    }
                    // Fallback to Tiler Utilization % if others not found
                    if line.contains("Tiler Utilization %") {
                        debug3!("Found 'Tiler Utilization %' in line: {}", line);
                        if let Some(percent) = extract_percentage_after_key(line, "Tiler Utilization %") {
                            if percent >= 0.0 && percent <= 100.0 {
                                debug3!("GPU usage from ioreg (Tiler Utilization %): {}%", percent);
                                return percent;
                            }
                        }
                    }
                }
                debug3!("ioreg AGXAccelerator: No utilization found in output");
            } else {
                debug3!("ioreg AGXAccelerator command failed with status: {:?}", output.status);
            }
        }
        Err(e) => {
            debug3!("Failed to execute ioreg AGXAccelerator command: {}", e);
        }
    }
    
    // Method 2: Try IOGPUWrangler (Intel Macs or older systems)
    let output = Command::new("/usr/sbin/ioreg")
        .arg("-r")
        .arg("-d")
        .arg("1")
        .arg("-w")
        .arg("0")
        .arg("-c")
        .arg("IOGPUWrangler")
        .stderr(std::process::Stdio::null())
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Utilization") || line.contains("utilization") {
                    if let Some(percent) = extract_percentage_from_line(line) {
                        if percent >= 0.0 && percent <= 100.0 {
                            debug3!("GPU usage from ioreg (IOGPUWrangler): {}%", percent);
                            return percent;
                        }
                    }
                }
            }
        }
    }
    
    // If we can't read GPU usage, return 0.0
    // This is better than showing incorrect data
    debug3!("GPU usage: could not read from system, returning 0%");
    0.0
}

/// Extract percentage value after a specific key in a line
/// Looks for patterns like "Device Utilization %"=22 or Device Utilization %=22
/// The key must be followed by = and then a number
fn extract_percentage_after_key(line: &str, key: &str) -> Option<f32> {
    // Find the key in the line (with or without quotes)
    let key_variants = [
        format!("\"{}\"", key),  // "Device Utilization %"
        key.to_string(),          // Device Utilization %
    ];
    
    for key_variant in &key_variants {
        if let Some(key_pos) = line.find(key_variant) {
            // Find the = sign after the key
            let after_key = &line[key_pos + key_variant.len()..];
            if let Some(eq_pos) = after_key.find('=') {
                let after_eq = &after_key[eq_pos + 1..];
                // Extract the number after =
                // Remove any leading/trailing whitespace, quotes, commas
                let trimmed = after_eq.trim()
                    .trim_start_matches('"')
                    .trim_start_matches(' ')
                    .trim_end_matches(',')
                    .trim_end_matches('"')
                    .trim_end_matches('}');
                
                debug3!("Extracting from '{}' after key '{}'", trimmed, key_variant);
                
                // Try to parse the first number (before any comma or closing brace)
                // Handle cases like "22," or "22}" or just "22"
                let num_str: String = trimmed.chars()
                    .take_while(|c| c.is_numeric() || *c == '.')
                    .collect();
                
                if !num_str.is_empty() {
                    if let Ok(num) = num_str.parse::<f32>() {
                        if num >= 0.0 && num <= 100.0 {
                            debug3!("Successfully extracted {}% from '{}'", num, trimmed);
                            return Some(num);
                        } else {
                            debug3!("Value {} is out of range (0-100)", num);
                        }
                    } else {
                        debug3!("Failed to parse '{}' as f32", num_str);
                    }
                }
                
                // Fallback: try parsing the whole trimmed string
                if let Ok(num) = trimmed.parse::<f32>() {
                    if num >= 0.0 && num <= 100.0 {
                        debug3!("Successfully extracted {}% (fallback parse)", num);
                        return Some(num);
                    }
                }
                
                // Also try splitting by whitespace in case there's extra text
                for word in trimmed.split_whitespace() {
                    let cleaned = word.trim_end_matches(',').trim_end_matches('}');
                    if let Ok(num) = cleaned.parse::<f32>() {
                        if num >= 0.0 && num <= 100.0 {
                            debug3!("Successfully extracted {}% (from split)", num);
                            return Some(num);
                        }
                    }
                }
            }
        }
    }
    
    debug3!("Could not extract percentage after key '{}' in line", key);
    None
}

/// Extract percentage value from a line of text (fallback method)
/// Looks for patterns like "= 45" or "45%" or similar
fn extract_percentage_from_line(line: &str) -> Option<f32> {
    // Try to find "=" followed by a number (most common format)
    if let Some(eq_pos) = line.find('=') {
        let after_eq = &line[eq_pos + 1..];
        // Extract the first number after =
        // Remove any trailing commas or other punctuation
        let trimmed = after_eq.trim().trim_end_matches(',');
        if let Ok(num) = trimmed.parse::<f32>() {
            if num >= 0.0 && num <= 100.0 {
                return Some(num);
            }
        }
        // Also try splitting by whitespace in case there's extra text
        for word in after_eq.split_whitespace() {
            let cleaned = word.trim_end_matches(',');
            if let Ok(num) = cleaned.parse::<f32>() {
                if num >= 0.0 && num <= 100.0 {
                    return Some(num);
                }
            }
        }
    }
    
    // Try to find a percentage sign
    if let Some(percent_pos) = line.find('%') {
        // Look backwards from % to find the number
        let before_percent = &line[..percent_pos];
        // Extract the last number before %
        if let Some(num_str) = before_percent.split_whitespace().last() {
            if let Ok(num) = num_str.parse::<f32>() {
                return Some(num);
            }
        }
    }
    
    // Try to find any number between 0-100 in the line
    for word in line.split_whitespace() {
        // Remove common punctuation but keep decimal point
        let cleaned = word.trim_matches(|c: char| !c.is_numeric() && c != '.' && c != '-');
        if let Ok(num) = cleaned.parse::<f32>() {
            if num >= 0.0 && num <= 100.0 {
                return Some(num);
            }
        }
    }
    
    None
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
    
    // OPTIMIZATION Phase 3: Use OnceLock for faster access (no locking required)
    *CAN_READ_TEMPERATURE.get_or_init(|| {
        debug3!("can_read_temperature: First time check - trying SMC connection...");
        let can_read = if let Ok(mut smc) = Smc::connect() {
            // Connection succeeded - we can attempt to read (even if it returns 0.0)
            match smc.cpu_temperature() {
                Ok(_) => {
                    // SMC read succeeded (even if temp is 0.0, the read worked)
                    debug3!("SMC connection and read succeeded - can_read_temperature=true");
                    true
                },
                Err(e) => {
                    // SMC read failed
                    debug3!("SMC read failed: {:?} - can_read_temperature=false", e);
                    false
                }
            }
        } else {
            // SMC connection failed - can't read
            debug3!("SMC connection failed - can_read_temperature=false");
            false
        };

        debug3!("can_read_temperature: Cached result: {}", can_read);
        can_read
    })
}

// Get nominal CPU frequency using sysctl (cheap, no sudo required)
// This gives base/nominal frequency, not dynamic frequency
pub(crate) fn get_nominal_frequency() -> f32 {
    *NOMINAL_FREQ.get_or_init(|| {
        // Try hw.tbfrequency * kern.clockrate.hz approach (works on Apple Silicon)
        let tbfreq_output = Command::new("/usr/sbin/sysctl")
            .arg("-n")
            .arg("hw.tbfrequency")
            .stderr(std::process::Stdio::null())
            .output();
        
        // kern.clockrate.hz doesn't work directly - need to parse the struct
        // Call sysctl directly and parse the output
        let clockrate_output = Command::new("/usr/sbin/sysctl")
            .arg("kern.clockrate")
            .stderr(std::process::Stdio::null())
            .output();
        
        // Try standard cpufrequency (works on Intel)
        // Try cpufrequency_max first, then fallback to cpufrequency
        let cpufreq_output = Command::new("/usr/sbin/sysctl")
            .arg("-n")
            .arg("hw.cpufrequency_max")
            .stderr(std::process::Stdio::null())
            .output();
        
        // Try tbfrequency * clockrate first (Apple Silicon)
        // Formula: cpu_freq_hz = hw.tbfrequency * kern.clockrate.hz
        // This gives nominal/base frequency, not dynamic frequency
        if let (Ok(tb), Ok(clock)) = (tbfreq_output, clockrate_output) {
            if tb.status.success() && clock.status.success() {
                let tb_str = String::from_utf8_lossy(&tb.stdout).trim().to_string();
                // Parse clockrate output: "kern.clockrate: { hz = 100, tick = 10000, tickadj = 2, ... }"
                // Extract "hz = <number>" from the output
                let clock_str = String::from_utf8_lossy(&clock.stdout);
                let hz_value = clock_str
                    .lines()
                    .flat_map(|line| {
                        // Look for "hz = <number>" pattern
                        line.split_whitespace()
                            .collect::<Vec<_>>()
                            .windows(3)
                            .find_map(|w| {
                                if w[0] == "hz" && w[1] == "=" {
                                    w[2].trim_end_matches(',').parse::<f64>().ok()
                                } else {
                                    None
                                }
                            })
                    })
                    .next()
                    .unwrap_or(0.0);
                
                debug3!("tbfrequency: '{}', clockrate.hz: '{}'", tb_str, hz_value);
                if let Ok(tb_hz) = tb_str.parse::<f64>() {
                    debug3!("Parsed: tb_hz={}, clock_hz={}", tb_hz, hz_value);
                    if tb_hz > 0.0 && hz_value > 0.0 {
                        // Formula: tbfrequency * clockrate.hz = CPU frequency in Hz
                        let freq_hz = tb_hz * hz_value;
                        let freq_ghz = (freq_hz / 1_000_000_000.0) as f32;
                        debug3!("Computed: freq_hz={}, freq_ghz={:.2}", freq_hz, freq_ghz);
                        if freq_ghz > 0.1 && freq_ghz < 10.0 {
                            debug3!("Nominal frequency computed: {:.2} GHz (tbfreq * clockrate.hz)", freq_ghz);
                            return freq_ghz;
                        } else {
                            debug3!("Computed frequency {:.2} GHz is out of range (0.1-10.0)", freq_ghz);
                        }
                    } else {
                        debug3!("tb_hz or clock_hz is zero: tb_hz={}, clock_hz={}", tb_hz, hz_value);
                    }
                } else {
                    debug3!("Failed to parse tbfrequency as number");
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
                                debug3!("Nominal frequency from sysctl: {:.2} GHz", freq_ghz);
                                return freq_ghz;
                            }
                        }
                    }
                }
            }
        }
        
        // Try cpufrequency fallback (without _max)
        let cpufreq_fallback = Command::new("/usr/sbin/sysctl")
            .arg("-n")
            .arg("hw.cpufrequency")
            .stderr(std::process::Stdio::null())
            .output();
        
        if let Ok(output) = cpufreq_fallback {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let trimmed = stdout.trim();
                if !trimmed.is_empty() && trimmed != "0" {
                    if let Ok(freq_hz) = trimmed.parse::<f64>() {
                        if freq_hz > 0.0 {
                            let freq_ghz = (freq_hz / 1_000_000_000.0) as f32;
                            if freq_ghz > 0.1 && freq_ghz < 10.0 {
                                debug3!("Nominal frequency from sysctl (fallback): {:.2} GHz", freq_ghz);
                                return freq_ghz;
                            }
                        }
                    }
                }
            }
        }
        
        debug3!("Could not determine nominal frequency, using 0.0");
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
    
    // OPTIMIZATION Phase 3: Use OnceLock for faster access (no locking required)
    *CAN_READ_FREQUENCY.get_or_init(|| {
        debug3!("can_read_frequency: First time check - trying nominal frequency computation...");
        let nominal = get_nominal_frequency();
        let can_read = nominal > 0.0;

        debug3!("can_read_frequency: Cached result: {} (nominal={:.2} GHz)", can_read, nominal);
        can_read
    })
}

#[allow(dead_code)]
pub fn can_read_cpu_power() -> bool {
    // OPTIMIZATION Phase 3: Use OnceLock for faster access (no locking required)
    // First check if it's been explicitly set
    if let Some(can_read) = CAN_READ_CPU_POWER.get() {
        return *can_read;
    }
    
    // If not set yet, check if we have power cache or actual power values
    // This handles the case where power reading works but the flag hasn't been set yet
    if let Ok(cache) = crate::state::POWER_CACHE.try_lock() {
        if cache.is_some() {
            // We have a power cache, so we can read power
            return true;
        }
    }
    
    // Default to false if nothing indicates we can read power
    false
}

#[allow(dead_code)]
pub fn can_read_gpu_power() -> bool {
    // OPTIMIZATION Phase 3: Use OnceLock for faster access (no locking required)
    // First check if it's been explicitly set
    if let Some(can_read) = CAN_READ_GPU_POWER.get() {
        return *can_read;
    }
    
    // If not set yet, check if we have power cache or actual power values
    // This handles the case where power reading works but the flag hasn't been set yet
    if let Ok(cache) = crate::state::POWER_CACHE.try_lock() {
        if cache.is_some() {
            // We have a power cache, so we can read power
            return true;
        }
    }
    
    // Default to false if nothing indicates we can read power
    false
}

/// Get battery level and charging state (cached)
/// Returns (battery_level_percent, is_charging, has_battery)
/// battery_level_percent: 0-100 if battery exists, -1.0 if no battery
/// is_charging: true if charging, false if discharging or no battery
/// has_battery: true if device has a battery
/// 
/// CRITICAL: Only reads fresh data when CPU window is visible to save CPU.
/// Returns cached values when window is closed.
pub fn get_battery_info() -> (f32, bool, bool) {
    // Check cache first (battery state doesn't change rapidly)
    // Battery reading via IOKit is lightweight, but we only read when window is visible
    if let Ok(cache) = crate::state::BATTERY_CACHE.try_lock() {
        if let Some((level, charging, timestamp)) = cache.as_ref() {
            // Check if CPU window is visible before doing fresh read
            let window_visible = crate::state::APP_HANDLE.get()
                .and_then(|app_handle| {
                    app_handle.get_window("cpu").and_then(|window| {
                        window.is_visible().ok().filter(|&visible| visible)
                    })
                })
                .is_some();
            
            // If window is closed, always return cache (even if stale) to save CPU
            if !window_visible {
                debug3!("Battery info from cache (window closed): {:.1}%, charging={}, has_battery={}", 
                    level, charging, *level >= 0.0);
                return (*level, *charging, *level >= 0.0);
            }
            
            // If window is visible, use cache if fresh (less than 1 second old)
            if timestamp.elapsed().as_secs() < 1 {
                debug3!("Battery info from cache: {:.1}%, charging={}, has_battery={}", 
                    level, charging, *level >= 0.0);
                return (*level, *charging, *level >= 0.0);
            }
        } else {
            // No cache - check if window is visible before reading
            let window_visible = crate::state::APP_HANDLE.get()
                .and_then(|app_handle| {
                    app_handle.get_window("cpu").and_then(|window| {
                        window.is_visible().ok().filter(|&visible| visible)
                    })
                })
                .is_some();
            
            if !window_visible {
                // Window closed and no cache - return default values to save CPU
                debug3!("Battery info: window closed, no cache, returning defaults");
                return (-1.0, false, false);
            }
        }
    }
    
    // Read battery info using battery crate (only if window is visible)
    let result = match BatteryManager::new() {
        Ok(manager) => {
            match manager.batteries() {
                Ok(mut batteries) => {
                    if let Some(battery_result) = batteries.next() {
                        match battery_result {
                            Ok(battery) => {
                                // Get battery percentage
                                let percentage = battery.state_of_charge().get::<battery::units::ratio::percent>();
                                let is_charging = matches!(battery.state(), State::Charging);
                                
                                debug3!("Battery read: {:.1}%, charging={}", percentage, is_charging);
                                
                                // Update cache
                                if let Ok(mut cache) = crate::state::BATTERY_CACHE.try_lock() {
                                    *cache = Some((percentage as f32, is_charging, std::time::Instant::now()));
                                }
                                
                                (percentage as f32, is_charging, true)
                            },
                            Err(e) => {
                                debug3!("Failed to read battery: {:?}", e);
                                (-1.0, false, false)
                            }
                        }
                    } else {
                        // No battery found
                        debug3!("No battery found on this system");
                        (-1.0, false, false)
                    }
                },
                Err(e) => {
                    debug3!("Failed to enumerate batteries: {:?}", e);
                    (-1.0, false, false)
                }
            }
        },
        Err(e) => {
            debug3!("Failed to create battery manager: {:?}", e);
            (-1.0, false, false)
        }
    };
    
    result
}

/// Get CPU and GPU power consumption (cached)
/// Returns (cpu_power_watts, gpu_power_watts)
/// 
/// CRITICAL: Only reads fresh data when CPU window is visible to save CPU.
/// Returns cached values when window is closed.
/// Power is read from IOReport every 5 seconds when window is visible.
pub fn get_power_consumption() -> (f32, f32) {
    // Check if CPU window is visible - if not, return cache or 0.0 to save CPU
    let window_visible = crate::state::APP_HANDLE.get()
        .and_then(|app_handle| {
            app_handle.get_window("cpu").and_then(|window| {
                window.is_visible().ok().filter(|&visible| visible)
            })
        })
        .is_some();
    
    // Check cache first
    // IOReport power reading is expensive, so we cache longer
    if let Ok(cache) = crate::state::POWER_CACHE.try_lock() {
        if let Some((cpu_power, gpu_power, timestamp)) = cache.as_ref() {
            // If window is closed, always return cache (even if stale) to save CPU
            if !window_visible {
                debug3!("Power consumption from cache (window closed): CPU={:.2}W, GPU={:.2}W", cpu_power, gpu_power);
                return (*cpu_power, *gpu_power);
            }
            
            // If window is visible, use cache if fresh (less than 6 seconds old)
            // Background thread updates every 5 seconds
            if timestamp.elapsed().as_secs() < 6 {
                debug3!("Power consumption from cache: CPU={:.2}W, GPU={:.2}W", cpu_power, gpu_power);
                return (*cpu_power, *gpu_power);
            } else {
                // Cache is stale, but return last known values instead of 0.0 to prevent flickering
                // Background thread will update it soon
                debug3!("Power consumption from stale cache: CPU={:.2}W, GPU={:.2}W (age: {}s)", 
                    cpu_power, gpu_power, timestamp.elapsed().as_secs());
                return (*cpu_power, *gpu_power);
            }
        } else {
            // No cache - if window is closed, return 0.0 to save CPU
            if !window_visible {
                debug3!("Power consumption: window closed, no cache, returning 0.0W");
                return (0.0, 0.0);
            }
        }
    }
    
    // Power reading is handled by background thread when window is visible
    // This function just returns cached values
    // If no cache exists yet, return last successful reading or 0.0
    // CRITICAL: Never return 0.0 if we have a cache - always return cached values
    // This prevents flickering when cache exists but is being queried before update
    if let Ok(cache) = crate::state::POWER_CACHE.try_lock() {
        if let Some((cpu_power, gpu_power, _)) = cache.as_ref() {
            // We have a cache - return it even if it seems stale
            // This prevents flickering to 0.0
            // Also update the last successful reading so we have a fallback if lock fails later
            if let Ok(mut last_successful) = crate::state::LAST_SUCCESSFUL_POWER.try_lock() {
                *last_successful = Some((*cpu_power, *gpu_power));
            }
            debug3!("Power consumption: returning cached values (no fresh update yet): CPU={:.2}W, GPU={:.2}W",
                cpu_power, gpu_power);
            return (*cpu_power, *gpu_power);
        }
    } else {
        // CRITICAL: try_lock() failed (background thread has the lock)
        // Return last successful reading instead of 0.0 to prevent flickering
        if let Ok(last_successful) = crate::state::LAST_SUCCESSFUL_POWER.try_lock() {
            if let Some((cpu_power, gpu_power)) = last_successful.as_ref() {
                debug3!("Power consumption: returning last successful values (lock failed): CPU={:.2}W, GPU={:.2}W",
                    cpu_power, gpu_power);
                return (*cpu_power, *gpu_power);
            }
        }
    }

    // No cache and no previous successful reading - return 0.0 (initial state)
    debug3!("Power consumption: no cache and no previous reading, returning 0.0W");
    (0.0, 0.0)
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
            let is_new_instance = sys.is_none();
            if is_new_instance {
                debug3!("Creating new System instance");
                // Create outside lock scope if possible, but we need the lock to store it
                *sys = Some(System::new());
            }
            let sys = sys.as_mut().unwrap();
            
            // CRITICAL: Always refresh on first use to get valid data
            // After that, only refresh if enough time has passed (reduces CPU usage)
            if is_new_instance || should_refresh {
                debug3!("Refreshing CPU usage and memory (is_new={}, should_refresh={})", is_new_instance, should_refresh);
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
            // This will be caught by is_valid() check and update will be skipped
            debug3!("SYSTEM mutex locked, returning zeros (update will be skipped if invalid)");
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
            debug3!("WARNING: DISKS mutex is locked, using 0% for disk");
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

/// Get application version from Cargo.toml
/// This is called by the frontend to always display the correct version
#[tauri::command]
pub fn get_app_version() -> String {
    crate::config::Config::version()
}

/// Embedded changelog content (compiled into binary at build time)
/// This ensures the changelog is always available regardless of where the executable is located.
/// Path is relative to this file (src-tauri/src/metrics/mod.rs):
///   ../../../CHANGELOG.md = project root/CHANGELOG.md
const EMBEDDED_CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

/// Get changelog content from CHANGELOG.md
/// Returns the changelog as a string
/// 
/// This function tries multiple strategies in order:
/// 1. First tries to read from the project root (for development - allows live updates)
/// 2. Falls back to embedded changelog (compiled into binary - always works)
#[tauri::command]
pub fn get_changelog() -> Result<String, String> {
    
    // Strategy 1: Try to read from project root (development builds)
    // This allows the changelog to be updated without recompiling during development
    if let Ok(cwd) = std::env::current_dir() {
        let cwd_changelog = cwd.join("CHANGELOG.md");
        if cwd_changelog.exists() {
            debug3!("Reading changelog from current directory: {:?}", cwd_changelog);
            if let Ok(content) = std::fs::read_to_string(&cwd_changelog) {
                if !content.trim().is_empty() {
                    return Ok(content);
                }
            }
        }
        
        // Also try going up from current directory (in case we're in a subdirectory)
        if let Some(parent) = cwd.parent() {
            let parent_changelog = parent.join("CHANGELOG.md");
            if parent_changelog.exists() {
                debug3!("Reading changelog from parent directory: {:?}", parent_changelog);
                if let Ok(content) = std::fs::read_to_string(&parent_changelog) {
                    if !content.trim().is_empty() {
                        return Ok(content);
                    }
                }
            }
        }
    }
    
    // Strategy 2: Try to read from executable's directory hierarchy
    // In development: target/debug/mac_stats or target/release/mac_stats
    // In production: mac-stats.app/Contents/MacOS/mac_stats
    if let Ok(exe_path) = std::env::current_exe() {
        let mut current = exe_path.parent();
        
        // Try going up to 6 levels (should cover most cases)
        for _ in 0..6 {
            if let Some(dir) = current {
                let changelog_path = dir.join("CHANGELOG.md");
                if changelog_path.exists() {
                    debug3!("Reading changelog from: {:?}", changelog_path);
                    if let Ok(content) = std::fs::read_to_string(&changelog_path) {
                        if !content.trim().is_empty() {
                            return Ok(content);
                        }
                    }
                }
                current = dir.parent();
            } else {
                break;
            }
        }
    }
    
    // Strategy 3: Use embedded changelog (compiled into binary)
    // This always works and is the most reliable fallback
    let embedded_len = EMBEDDED_CHANGELOG.len();
    debug3!("Embedded changelog length: {} bytes", embedded_len);
    
    if !EMBEDDED_CHANGELOG.trim().is_empty() {
        debug3!("Using embedded changelog (compiled into binary)");
        return Ok(EMBEDDED_CHANGELOG.to_string());
    }
    
    // If embedded changelog is also empty, return an error with helpful info
    let error_msg = format!(
        "Changelog is not available. Embedded changelog is empty ({} bytes). \
        The CHANGELOG.md file may not have been found at compile time. \
        Please ensure CHANGELOG.md exists at the project root and rebuild the app.",
        embedded_len
    );
    debug3!("{}", error_msg);
    Err(error_msg)
}

/// Get window decorations preference
#[tauri::command]
pub fn get_window_decorations() -> bool {
    // Read from config file (allows changes without recompiling)
    crate::config::Config::get_window_decorations()
}

/// Set window decorations preference
#[tauri::command]
pub fn set_window_decorations(decorations: bool) -> Result<(), String> {
    use crate::config::Config;
    
    // Update Rust state
    use crate::state::WINDOW_DECORATIONS;
    if let Ok(mut pref) = WINDOW_DECORATIONS.lock() {
        *pref = decorations;
    }
    
    // Write to config file so it persists and works without recompiling
    let config_path = Config::config_file_path();
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(format!("Failed to create config directory: {}", e));
        }
    }
    
    let config = serde_json::json!({
        "windowDecorations": decorations
    });
    
    if let Err(e) = std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap_or_else(|_| config.to_string())) {
        return Err(format!("Failed to write config file: {}", e));
    }
    
    crate::debug3!("Window decorations preference set to: {} (saved to config file)", decorations);
    Ok(())
}

#[tauri::command]
pub fn get_cpu_details() -> CpuDetails {
    // STEP 5: Rate limiting - prevent get_cpu_details from being called too frequently
    // BUT: Always allow process cache age check - processes need to refresh every 5s
    // Rate limit other expensive operations, but check process cache on every call
    let should_allow_full_call = match crate::state::LAST_CPU_DETAILS_CALL.try_lock() {
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
    
    // CRITICAL: Always check process cache age, even if rate-limited
    // This ensures processes refresh every 5 seconds as requested
    let should_check_process_cache = true;
    
    if !should_allow_full_call {
        debug3!("get_cpu_details() rate limited - returning cached values for most metrics");
        // Return cached values immediately without doing expensive work
        // BUT: Still check and refresh process cache if stale (>5s)
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
        
        // CRITICAL: Check process cache age even when rate-limited
        // If stale (>5s), refresh it now (process refresh is the priority)
        let processes = if should_check_process_cache {
            let should_collect_processes = crate::state::APP_HANDLE.get()
                .and_then(|app_handle| {
                    app_handle.get_window("cpu").and_then(|window| {
                        window.is_visible().ok().filter(|&visible| visible)
                    })
                })
                .is_some();
            
            if should_collect_processes {
                match crate::state::PROCESS_CACHE.try_lock() {
                    Ok(cache) => {
                        if let Some((procs, timestamp)) = cache.as_ref() {
                            let age_secs = timestamp.elapsed().as_secs();
                            if age_secs >= 5 {
                                // Cache is stale - refresh now even if rate-limited
                                debug3!("Process cache is stale ({}s) - refreshing now (even though rate-limited)", age_secs);
                                // Need SYSTEM lock to refresh processes
                                match crate::state::SYSTEM.try_lock() {
                                    Ok(mut sys) => {
                                        if let Some(sys) = sys.as_mut() {
                                            use sysinfo::ProcessesToUpdate;
                                            sys.refresh_processes(ProcessesToUpdate::All, true);
                                            
                                            let mut processes: Vec<crate::metrics::ProcessUsage> = sys
                                                .processes()
                                                .iter()
                                                .map(|(pid, proc)| crate::metrics::ProcessUsage {
                                                    name: proc.name().to_string_lossy().to_string(),
                                                    cpu: proc.cpu_usage(),
                                                    pid: pid.as_u32(),
                                                })
                                                .collect();
                                            
                                            processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
                                            processes.truncate(8);
                                            
                                            // Update cache
                                            if let Ok(mut process_cache) = crate::state::PROCESS_CACHE.try_lock() {
                                                *process_cache = Some((processes.clone(), std::time::Instant::now()));
                                                debug3!("Process cache refreshed (rate-limited call)");
                                            }
                                            
                                            processes
                                        } else {
                                            procs.clone()
                                        }
                                    },
                                    Err(_) => procs.clone(), // SYSTEM locked, return cached
                                }
                            } else {
                                procs.clone()
                            }
                        } else {
                            Vec::new()
                        }
                    },
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        // Get cached battery and power info
        let (battery_level, is_charging, has_battery) = crate::state::BATTERY_CACHE.try_lock()
            .ok()
            .and_then(|c| c.as_ref().map(|(level, charging, _)| (*level, *charging, *level >= 0.0)))
            .unwrap_or((-1.0, false, false));
        
        // Use get_power_consumption() for consistent cache handling
        // This ensures we always return cached values (even if stale) instead of 0.0
        let (cpu_power, gpu_power) = get_power_consumption();
        
        // Check if we actually have power values (even if 0, if we have a cache entry, we can read power)
        // This is more reliable than checking the flags, which might not be set yet
        let has_power_cache = crate::state::POWER_CACHE.try_lock()
            .ok()
            .map(|c| c.is_some())
            .unwrap_or(false);
        
        // If we have power cache, we can read power (even if values are currently 0)
        // This prevents showing "Requires root privileges" when we're just waiting for the first read
        // OR if we have actual power values > 0, we definitely can read power
        let can_read_cpu_power = has_power_cache || cpu_power > 0.0 || crate::metrics::can_read_cpu_power();
        let can_read_gpu_power = has_power_cache || gpu_power > 0.0 || crate::metrics::can_read_gpu_power();
        
        return CpuDetails {
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
            top_processes: processes,
            chip_info: crate::metrics::get_chip_info(),
            can_read_temperature: crate::metrics::can_read_temperature(),
            can_read_frequency: crate::metrics::can_read_frequency(),
            can_read_cpu_power,
            can_read_gpu_power,
            battery_level,
            is_charging,
            has_battery,
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
    
    // CRITICAL: Use try_lock ONCE - if locked, return cached values immediately
    // This prevents blocking the main thread when the window opens
    let (usage, load, uptime_secs, top_processes) = match SYSTEM.try_lock() {
        Ok(mut sys) => {
            if sys.is_none() {
                // System not initialized yet - return cached/fallback values immediately
                // Don't wait for initialization - return what we have NOW
                debug3!("SYSTEM is None - returning cached/fallback values for instant display");
                let load = sysinfo::System::load_average();
                let uptime_secs = sysinfo::System::uptime();
                // Try to get cached processes, otherwise empty
                let processes = crate::state::PROCESS_CACHE.try_lock()
                    .ok()
                    .and_then(|c| c.as_ref().map(|(p, _)| p.clone()))
                    .unwrap_or_default();
                // Return 0.0 for usage (will be updated on next refresh)
                (0.0, load, uptime_secs, processes)
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
                    // STEP 4: Cache process list for 5 seconds when window is open (refresh every 5s)
                    // CRITICAL: Always check cache first and return immediately if available
                    // This prevents blocking on expensive refresh_processes() when window first opens
                    let cached_processes = match PROCESS_CACHE.try_lock() {
                        Ok(cache) => {
                            cache.as_ref().map(|(procs, timestamp)| {
                                let age_secs = timestamp.elapsed().as_secs();
                                (procs.clone(), age_secs)
                            })
                        },
                        Err(_) => None, // Lock held, skip cache check
                    };
                    
                    // If we have cached data, check if it's still fresh (<10 seconds)
                    // OPTIMIZATION Phase 1: Increased from 5s to 10s to reduce process enumeration overhead
                    // BUT: If cache is empty (None), always refresh immediately for instant display
                    if let Some((cached_procs, age_secs)) = cached_processes {
                        if age_secs < 10 {
                            // Cache is less than 10 seconds old - return immediately
                            // This prevents blocking and reduces CPU usage
                            debug3!("Returning cached process list (age: {}s) - refresh every 10s", age_secs);
                            cached_procs
                        } else {
                            // Cache is stale (>5s) - refresh now
                            debug3!("Process cache is stale ({}s), refreshing now (5s interval)", age_secs);
                            use sysinfo::ProcessesToUpdate;
                            sys.refresh_processes(ProcessesToUpdate::All, true);
                            
                            // Collect ALL processes first (HashMap iteration order is undefined)
                            // Then sort by CPU usage to get the actual top processes
                            let mut processes: Vec<ProcessUsage> = sys
                                .processes()
                                .iter()
                                .map(|(pid, proc)| ProcessUsage {
                                    name: proc.name().to_string_lossy().to_string(),
                                    cpu: proc.cpu_usage(),
                                    pid: pid.as_u32(),
                                })
                                .collect();
                            
                            // Sort by CPU usage (descending) to get actual top processes
                            processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
                            
                            // Take top 8 after sorting
                            processes.truncate(8);
                            
                            // Update cache
                            if let Ok(mut cache) = PROCESS_CACHE.try_lock() {
                                *cache = Some((processes.clone(), std::time::Instant::now()));
                                debug3!("Process cache updated (refreshed from system)");
                            }
                            
                            processes
                        }
                    } else {
                        // No cache available - refresh now (first time or cache was cleared)
                        // This is the only case where we block on refresh_processes()
                        // CRITICAL: This happens when window first opens (cache was cleared)
                        debug3!("Process cache is empty, refreshing now immediately (window just opened)");
                        use sysinfo::ProcessesToUpdate;
                        sys.refresh_processes(ProcessesToUpdate::All, true);
                        
                        // Collect ALL processes first (HashMap iteration order is undefined)
                        // Then sort by CPU usage to get the actual top processes
                        let mut processes: Vec<ProcessUsage> = sys
                            .processes()
                            .iter()
                            .map(|(pid, proc)| ProcessUsage {
                                name: proc.name().to_string_lossy().to_string(),
                                cpu: proc.cpu_usage(),
                                pid: pid.as_u32(),
                            })
                            .collect();
                        
                        // Sort by CPU usage (descending) to get actual top processes
                        processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
                        
                        // Take top 8 after sorting
                        processes.truncate(8);
                        
                        // Update cache
                        if let Ok(mut cache) = PROCESS_CACHE.try_lock() {
                            *cache = Some((processes.clone(), std::time::Instant::now()));
                            debug3!("Process cache updated (refreshed from system)");
                        }
                        
                        processes
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
            debug3!("WARNING: SYSTEM mutex locked in get_cpu_details, returning defaults immediately");
            write_structured_log("lib.rs:697", "SYSTEM locked, returning defaults", &serde_json::json!({}), "L");
            (0.0, sysinfo::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 }, 0, Vec::new())
        }
    };

        // CRITICAL: Use cached values or defaults - don't call expensive functions
        // SMC calls and other operations can block the main thread
        // OPTIMIZATION Phase 3: Use OnceLock for fast capability flag access (no locking)
        let (temperature, frequency, p_core_frequency, e_core_frequency, cpu_power, gpu_power, chip_info, can_read_temperature, can_read_frequency, can_read_cpu_power, can_read_gpu_power, battery_level, is_charging, has_battery) = {
        // Get cached access flags (fast OnceLock access, no blocking)
        let _can_read_temp = CAN_READ_TEMPERATURE.get().copied().unwrap_or(false);
        let can_read_freq = CAN_READ_FREQUENCY.get().copied().unwrap_or(false);
        let can_read_cpu_p = CAN_READ_CPU_POWER.get().copied().unwrap_or(false);
        let can_read_gpu_p = CAN_READ_GPU_POWER.get().copied().unwrap_or(false);
        
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
        let freq_logging = crate::state::FREQUENCY_LOGGING_ENABLED.lock()
            .map(|f| *f)
            .unwrap_or(false);
        
        let p_core_frequency = match P_CORE_FREQ_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((freq, timestamp)) = cache.as_ref() {
                    let age_secs = timestamp.elapsed().as_secs();
                    if age_secs < 35 {
                        if freq_logging {
                            debug3!("P-core frequency from cache: {:.2} GHz (age: {}s)", *freq, age_secs);
                        }
                        *freq
                    } else {
                        if freq_logging {
                            debug3!("P-core frequency cache is stale ({}s old), falling back to nominal", age_secs);
                        }
                        get_nominal_frequency() // Fallback to nominal if stale
                    }
                } else {
                    if freq_logging {
                        debug3!("P-core frequency cache is empty, falling back to nominal");
                    }
                    get_nominal_frequency()
                }
            },
            Err(_) => {
                if freq_logging {
                    debug3!("P-core frequency cache is locked, falling back to nominal");
                }
                get_nominal_frequency()
            },
        };
        
        let e_core_frequency = match E_CORE_FREQ_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((freq, timestamp)) = cache.as_ref() {
                    let age_secs = timestamp.elapsed().as_secs();
                    if age_secs < 35 {
                        if freq_logging {
                            debug3!("E-core frequency from cache: {:.2} GHz (age: {}s)", *freq, age_secs);
                        }
                        *freq
                    } else {
                        if freq_logging {
                            debug3!("E-core frequency cache is stale ({}s old), falling back to nominal", age_secs);
                        }
                        get_nominal_frequency() // Fallback to nominal if stale
                    }
                } else {
                    if freq_logging {
                        debug3!("E-core frequency cache is empty, falling back to nominal");
                    }
                    get_nominal_frequency()
                }
            },
            Err(_) => {
                if freq_logging {
                    debug3!("E-core frequency cache is locked, falling back to nominal");
                }
                get_nominal_frequency()
            },
        };
        
        // Use cached chip info or default - ensure it's initialized by calling get_chip_info()
        let chip = get_chip_info();
        
        // Get power consumption (cached)
        let (cpu_power_val, gpu_power_val) = get_power_consumption();
        
        // Get battery info (cached)
        let (battery_level_val, is_charging_val, has_battery_val) = get_battery_info();
        
        // Return cached temperature, frequency, power, battery, and defaults for other expensive values
        (temperature, frequency, p_core_frequency, e_core_frequency, cpu_power_val, gpu_power_val, chip, can_read_temp, can_read_freq, can_read_cpu_p, can_read_gpu_p, battery_level_val, is_charging_val, has_battery_val)
    };
    
    // Log data being sent to frontend for debugging
    let power_logging = crate::state::POWER_USAGE_LOGGING_ENABLED.lock()
        .map(|f| *f)
        .unwrap_or(false);
    
    if power_logging {
        debug3!("get_cpu_details returning: temperature={:.1}°C, frequency={:.2} GHz, cpu_power={:.2}W, gpu_power={:.2}W, battery={:.1}%, charging={}, has_battery={}", 
            temperature, frequency, cpu_power, gpu_power, battery_level, is_charging, has_battery);
    } else {
        debug3!("get_cpu_details returning: temperature={:.1}°C, frequency={:.2} GHz, can_read_temperature={}, can_read_frequency={}", temperature, frequency, can_read_temperature, can_read_frequency);
    }
    
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
        battery_level,
        is_charging,
        has_battery,
    }
}

/// Get detailed information about a specific process by PID
#[tauri::command]
pub fn get_process_details(pid: u32) -> Result<ProcessDetails, String> {
    use sysinfo::Pid;
    
    debug3!("get_process_details() called for PID: {}", pid);
    
    // CRITICAL: Only refresh processes if CPU window is visible (saves CPU)
    // Process details modal is part of the CPU window, so check window visibility
    let should_refresh_processes = APP_HANDLE.get()
        .and_then(|app_handle| {
            app_handle.get_window("cpu").and_then(|window| {
                window.is_visible().ok().filter(|&visible| visible)
            })
        })
        .is_some();
    
    if !should_refresh_processes {
        debug3!("CPU window not visible, skipping process refresh in get_process_details");
        // Still try to get the process from cache if available, but don't refresh
    }
    
    // Use try_lock to avoid blocking - collect all data while lock is held
    match SYSTEM.try_lock() {
        Ok(mut sys) => {
            if sys.is_none() {
                return Err("System not initialized".to_string());
            }
            let sys = sys.as_mut().unwrap();
            
            // Only refresh all processes if CPU window is visible (saves CPU)
            if should_refresh_processes {
                use sysinfo::ProcessesToUpdate;
                sys.refresh_processes(ProcessesToUpdate::All, true);
            }
            
            // Get the process while lock is held
            if let Some(proc) = sys.process(Pid::from_u32(pid)) {
                // Get parent process information while lock is still held
                let (parent_pid, parent_name) = if let Some(ppid) = proc.parent() {
                    let parent_proc = sys.process(ppid);
                    let parent_name = parent_proc
                        .map(|p| p.name().to_string_lossy().to_string());
                    (Some(ppid.as_u32()), parent_name)
                } else {
                    (None, None)
                };
                
                // Get user ID and username information
                // Parse the UID string to get the numeric value for getpwuid
                let (user_id, user_name) = if let Some(uid) = proc.user_id() {
                    let uid_str = uid.to_string();
                    let username = uid_str.parse::<u32>()
                        .ok()
                        .and_then(|uid_value| get_username_from_uid(uid_value));
                    (Some(uid_str), username)
                } else {
                    (None, None)
                };
                
                let (effective_user_id, effective_user_name) = if let Some(euid) = proc.effective_user_id() {
                    let euid_str = euid.to_string();
                    let username = euid_str.parse::<u32>()
                        .ok()
                        .and_then(|euid_value| get_username_from_uid(euid_value));
                    (Some(euid_str), username)
                } else {
                    (None, None)
                };
                
                // Get total CPU time (in milliseconds)
                // sysinfo 0.35 provides accumulated_cpu_time() method
                let total_cpu_time = proc.accumulated_cpu_time();
                
                // Collect all data before lock is released
                let details = ProcessDetails {
                    pid,
                    name: proc.name().to_string_lossy().to_string(),
                    cpu: proc.cpu_usage(),
                    parent_pid,
                    parent_name,
                    start_time: proc.start_time(),
                    user_id,
                    user_name,
                    effective_user_id,
                    effective_user_name,
                    memory: proc.memory(),
                    virtual_memory: proc.virtual_memory(),
                    disk_read: proc.disk_usage().total_read_bytes,
                    disk_written: proc.disk_usage().total_written_bytes,
                    total_cpu_time,
                };
                
                debug3!("Process details retrieved for PID {}: {}", pid, details.name);
                Ok(details)
            } else {
                Err(format!("Process with PID {} not found", pid))
            }
        },
        Err(_) => {
            Err("System lock unavailable".to_string())
        }
    }
}

/// Get username from UID using getpwuid
fn get_username_from_uid(uid: u32) -> Option<String> {
    unsafe {
        use libc::{getpwuid, passwd, c_char};
        let pw = getpwuid(uid);
        if pw.is_null() {
            return None;
        }
        let passwd: *const passwd = pw;
        let pw_name = (*passwd).pw_name;
        if pw_name.is_null() {
            return None;
        }
        let c_str: *const c_char = pw_name;
        let c_str_slice = std::ffi::CStr::from_ptr(c_str);
        c_str_slice.to_str().ok().map(|s| s.to_string())
    }
}

/// Force quit a process by PID
#[tauri::command]
pub fn force_quit_process(pid: u32) -> Result<(), String> {
    debug3!("force_quit_process() called for PID: {}", pid);

    // Use kill -9 to force quit the process
    let output = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                debug3!("Successfully force quit process PID: {}", pid);
                Ok(())
            } else {
                let error_msg = String::from_utf8_lossy(&result.stderr);
                debug3!("Failed to force quit process PID {}: {}", pid, error_msg);
                Err(format!("Failed to force quit process: {}", error_msg))
            }
        },
        Err(e) => {
            debug3!("Error executing kill command for PID {}: {}", pid, e);
            Err(format!("Failed to execute kill command: {}", e))
        }
    }
}

/// Get metrics history for a given time range
///
/// # Arguments
/// * `time_range_seconds` - Time range to query: 300 (5m), 3600 (1h), 21600 (6h), 604800 (7d)
/// * `max_display_points` - Optional max points for display width optimization
///
/// # Returns
/// History query result with points and metadata
#[tauri::command]
pub fn get_metrics_history(
    time_range_seconds: u64,
    max_display_points: Option<usize>,
) -> Result<history::HistoryQueryResult, String> {
    debug3!("get_metrics_history() called with time_range_seconds={}, max_display_points={:?}",
        time_range_seconds, max_display_points);

    // Try to get history buffer with non-blocking lock
    match METRICS_HISTORY.try_lock() {
        Ok(history_opt) => {
            if let Some(history) = history_opt.as_ref() {
                let points = history.query(time_range_seconds, max_display_points);
                let oldest = history.oldest_timestamp();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                debug3!("get_metrics_history: returning {} points, oldest_ts={:?}, newest_ts={}",
                    points.len(), oldest, now);

                Ok(history::HistoryQueryResult {
                    points,
                    time_range_seconds,
                    oldest_available_timestamp: oldest,
                    newest_available_timestamp: Some(now),
                })
            } else {
                debug3!("get_metrics_history: history buffer not initialized yet");
                Ok(history::HistoryQueryResult {
                    points: Vec::new(),
                    time_range_seconds,
                    oldest_available_timestamp: None,
                    newest_available_timestamp: None,
                })
            }
        },
        Err(e) => {
            debug3!("get_metrics_history: lock contention - {}", e);
            Err("History buffer temporarily unavailable".to_string())
        }
    }
}
