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
pub(crate) mod gpu_processes;
#[cfg(target_os = "macos")]
pub(crate) mod smc_temperature;

use battery::{Manager as BatteryManager, State};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};
use tauri::Manager;

use crate::logging::write_structured_log;
use crate::state::*;

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
    /// Estimated GPU utilization % from AGX client time deltas (Apple Silicon). 0 if unknown.
    #[serde(default)]
    pub gpu: f32,
    pub pid: u32,
    /// Resident memory bytes (sysinfo). 0 if unknown / older payload.
    #[serde(default)]
    pub memory: u64,
}

#[derive(serde::Serialize)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    #[serde(default)]
    pub gpu: f32,
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

/// Real-time CPU/system snapshot returned by `get_cpu_details()`.
/// Rate-limited; see state.rs. Full API contract (fields, types, semantics, consumers): `docs/data-poster-charts-backend.md` § get_cpu_details() API contract.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CpuDetails {
    pub usage: f32,
    pub gpu_usage: f32,
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
    /// System RAM used / total as percent (0–100). Same basis as menu-bar RAM.
    #[serde(default)]
    pub ram_percent: f32,
    /// Bytes of used system memory (sysinfo).
    #[serde(default)]
    pub ram_used_bytes: u64,
    /// Bytes of total system memory (sysinfo).
    #[serde(default)]
    pub ram_total_bytes: u64,
    /// System disk used / total as percent (0–100). Same basis as menu-bar SSD.
    #[serde(default)]
    pub disk_percent: f32,
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
    /// Apple `NSProcessInfo.thermalState` as Nominal/Fair/Serious/Critical (empty if unknown).
    #[serde(default)]
    pub thermal_state: String,
    /// Apple `NSProcessInfo.isLowPowerModeEnabled` (Low Power Mode).
    #[serde(default)]
    pub low_power_mode: bool,
}

/// Cached macOS ProductName · ProductVersion · BuildVersion (`sw_vers`).
/// Used by the Discord/operator instant `/macos` · `/os-version` lane.
pub fn get_macos_version() -> String {
    MACOS_VERSION_CACHE
        .get_or_init(|| {
            let run = |flag: &str| -> Option<String> {
                let output = Command::new("/usr/bin/sw_vers")
                    .arg(flag)
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            let name = run("-productName").unwrap_or_else(|| "macOS".to_string());
            let ver = run("-productVersion").unwrap_or_default();
            let build = run("-buildVersion").unwrap_or_default();
            if ver.is_empty() && build.is_empty() {
                debug3!("macOS version: sw_vers empty");
                return String::new();
            }
            let mut info = name;
            if !ver.is_empty() {
                info.push(' ');
                info.push_str(&ver);
            }
            if !build.is_empty() {
                info.push_str(" (");
                info.push_str(&build);
                info.push(')');
            }
            debug3!("macOS version cached: '{}'", info);
            info
        })
        .clone()
}

/// Cached ComputerName · LocalHostName (`scutil` / `hostname`).
/// Used by the Discord/operator instant `/hostname` · `/computer-name` lane.
pub fn get_hostname() -> String {
    HOSTNAME_CACHE
        .get_or_init(|| {
            let scutil = |key: &str| -> Option<String> {
                let output = Command::new("/usr/sbin/scutil")
                    .args(["--get", key])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            let computer = scutil("ComputerName");
            let local = scutil("LocalHostName");
            let info = match (computer, local) {
                (Some(c), Some(l)) if !c.eq_ignore_ascii_case(&l) => format!("{c} ({l})"),
                (Some(c), _) => c,
                (_, Some(l)) => l,
                (None, None) => {
                    let output = Command::new("/bin/hostname")
                        .arg("-s")
                        .stderr(std::process::Stdio::null())
                        .output()
                        .ok();
                    output
                        .and_then(|o| {
                            if !o.status.success() {
                                return None;
                            }
                            let s = String::from_utf8_lossy(&o.stdout);
                            let t = s.trim();
                            if t.is_empty() {
                                None
                            } else {
                                Some(t.to_string())
                            }
                        })
                        .unwrap_or_default()
                }
            };
            if info.is_empty() {
                debug3!("hostname: scutil/hostname empty");
            } else {
                debug3!("hostname cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Cached Mac marketing name · machine model (`system_profiler` / `sysctl hw.model`).
/// Used by the Discord/operator instant `/mac-model` · `/hardware-model` lane.
/// Distinct from Apple chip (`/chip`) and Ollama (`/model`).
pub fn get_mac_model() -> String {
    MAC_MODEL_CACHE
        .get_or_init(|| {
            let from_profiler = || -> Option<String> {
                let output = Command::new("/usr/sbin/system_profiler")
                    .arg("SPHardwareDataType")
                    .arg("-json")
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
                let hardware = json
                    .get("SPHardwareDataType")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())?;
                let pick = |keys: &[&str]| -> Option<String> {
                    for k in keys {
                        if let Some(s) = hardware.get(*k).and_then(|v| v.as_str()) {
                            let t = s.trim();
                            if !t.is_empty() {
                                return Some(t.to_string());
                            }
                        }
                    }
                    None
                };
                let name = pick(&["machine_name", "model_name"]);
                let model = pick(&["machine_model", "model_identifier"]);
                match (name, model) {
                    (Some(n), Some(m)) if !n.eq_ignore_ascii_case(&m) => Some(format!("{n} · {m}")),
                    (Some(n), _) => Some(n),
                    (_, Some(m)) => Some(m),
                    (None, None) => None,
                }
            };
            let from_sysctl = || -> Option<String> {
                let output = Command::new("/usr/sbin/sysctl")
                    .arg("-n")
                    .arg("hw.model")
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            let info = from_profiler().or_else(from_sysctl).unwrap_or_default();
            if info.is_empty() {
                debug3!("mac model: system_profiler/sysctl empty");
            } else {
                debug3!("mac model cached: '{}'", info);
            }
            info
        })
        .clone()
}


/// Cached login username (`$USER` / `id -un` / `whoami`).
/// Used by the Discord/operator instant `/username` · `/whoami` lane.
pub fn get_username() -> String {
    USERNAME_CACHE
        .get_or_init(|| {
            let from_cmd = |bin: &str, args: &[&str]| -> Option<String> {
                let output = Command::new(bin)
                    .args(args)
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            let info = std::env::var("USER")
                .ok()
                .map(|u| u.trim().to_string())
                .filter(|u| !u.is_empty())
                .or_else(|| from_cmd("/usr/bin/id", &["-un"]))
                .or_else(|| from_cmd("/usr/bin/whoami", &[]))
                .unwrap_or_default();
            if info.is_empty() {
                debug3!("username: USER/id/whoami empty");
            } else {
                debug3!("username cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Cached hardware serial number (`system_profiler` / `ioreg` / DMI product_serial).
/// Used by the Discord/operator instant `/serial` · `/serial-number` lane.
/// Distinct from Mac model (`/mac-model`) and chip (`/chip`).
pub fn get_serial_number() -> String {
    SERIAL_NUMBER_CACHE
        .get_or_init(|| {
            let usable = |s: &str| -> Option<String> {
                let t = s.trim();
                if t.is_empty() {
                    return None;
                }
                let lower = t.to_ascii_lowercase();
                // DMI / VM placeholders are not useful answers.
                if matches!(
                    lower.as_str(),
                    "none" | "not specified" | "not set" | "default string" | "to be filled by o.e.m." | "system serial number"
                ) {
                    return None;
                }
                Some(t.to_string())
            };
            let from_profiler = || -> Option<String> {
                let output = Command::new("/usr/sbin/system_profiler")
                    .arg("SPHardwareDataType")
                    .arg("-json")
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
                let hardware = json
                    .get("SPHardwareDataType")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())?;
                for k in ["serial_number", "serial_number_key"] {
                    if let Some(s) = hardware.get(k).and_then(|v| v.as_str()).and_then(usable) {
                        return Some(s);
                    }
                }
                None
            };
            let from_ioreg = || -> Option<String> {
                let output = Command::new("/usr/sbin/ioreg")
                    .args(["-c", "IOPlatformExpertDevice", "-d", "2"])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if !line.contains("IOPlatformSerialNumber") {
                        continue;
                    }
                    // "IOPlatformSerialNumber" = "C02XXXXX"
                    if let Some(rest) = line.split('=').nth(1) {
                        let cleaned = rest.trim().trim_matches('"').trim();
                        if let Some(s) = usable(cleaned) {
                            return Some(s);
                        }
                    }
                }
                None
            };
            let from_dmi = || -> Option<String> {
                let path = std::path::Path::new("/sys/class/dmi/id/product_serial");
                let raw = std::fs::read_to_string(path).ok()?;
                usable(&raw)
            };
            let info = from_profiler()
                .or_else(from_ioreg)
                .or_else(from_dmi)
                .unwrap_or_default();
            if info.is_empty() {
                debug3!("serial: system_profiler/ioreg/dmi empty");
            } else {
                debug3!("serial cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Hardware UUID (platform UUID) — not serial, not provisioning UDID.
/// Cached; used by the Discord/operator instant `/uuid` · `/hw-uuid` lane.
pub fn get_hardware_uuid() -> String {
    HARDWARE_UUID_CACHE
        .get_or_init(|| {
            let usable = |s: &str| -> Option<String> {
                let t = s.trim();
                if t.is_empty() {
                    return None;
                }
                let lower = t.to_ascii_lowercase();
                if matches!(
                    lower.as_str(),
                    "none" | "not specified" | "not set" | "default string" | "to be filled by o.e.m."
                ) {
                    return None;
                }
                // Hardware UUID is a dashed hex form; reject obvious junk.
                if t.len() < 8 {
                    return None;
                }
                Some(t.to_string())
            };
            let from_profiler = || -> Option<String> {
                let output = Command::new("/usr/sbin/system_profiler")
                    .arg("SPHardwareDataType")
                    .arg("-json")
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
                let hardware = json
                    .get("SPHardwareDataType")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())?;
                for k in ["platform_UUID", "platform_uuid", "hardware_UUID", "hardware_uuid"] {
                    if let Some(s) = hardware.get(k).and_then(|v| v.as_str()).and_then(usable) {
                        return Some(s);
                    }
                }
                None
            };
            let from_ioreg = || -> Option<String> {
                let output = Command::new("/usr/sbin/ioreg")
                    .args(["-c", "IOPlatformExpertDevice", "-d", "2"])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if !line.contains("IOPlatformUUID") {
                        continue;
                    }
                    if let Some(rest) = line.split('=').nth(1) {
                        let cleaned = rest.trim().trim_matches('"').trim();
                        if let Some(s) = usable(cleaned) {
                            return Some(s);
                        }
                    }
                }
                None
            };
            let info = from_profiler().or_else(from_ioreg).unwrap_or_default();
            if info.is_empty() {
                debug3!("hardware uuid: system_profiler/ioreg empty");
            } else {
                debug3!("hardware uuid cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Boot volume / startup disk name — not last boot wall-clock, not SSD % / free / Disk Cleanup.
/// Cached; used by the Discord/operator instant `/boot-volume` · `/startup-disk` lane.
pub fn get_boot_volume() -> String {
    BOOT_VOLUME_CACHE
        .get_or_init(|| {
            let usable = |s: &str| -> Option<String> {
                let t = s.trim();
                if t.is_empty() {
                    return None;
                }
                let lower = t.to_ascii_lowercase();
                if matches!(
                    lower.as_str(),
                    "none" | "not specified" | "not set" | "n/a" | "unknown" | "-"
                ) {
                    return None;
                }
                Some(t.to_string())
            };
            // macOS: `diskutil info /` → Volume Name
            let from_diskutil = || -> Option<String> {
                let output = Command::new("/usr/sbin/diskutil")
                    .args(["info", "/"])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let t = line.trim();
                    if let Some(rest) = t.strip_prefix("Volume Name:") {
                        if let Some(s) = usable(rest) {
                            return Some(s);
                        }
                    }
                }
                None
            };
            // Linux / generic: findmnt LABEL on /
            let from_findmnt = || -> Option<String> {
                let output = Command::new("findmnt")
                    .args(["-n", "-o", "LABEL", "/"])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                usable(String::from_utf8_lossy(&output.stdout).trim())
            };
            // Linux fallback: lsblk LABEL for the / source device
            let from_lsblk = || -> Option<String> {
                let src = Command::new("findmnt")
                    .args(["-n", "-o", "SOURCE", "/"])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !src.status.success() {
                    return None;
                }
                let source = String::from_utf8_lossy(&src.stdout).trim().to_string();
                if source.is_empty() {
                    return None;
                }
                let output = Command::new("lsblk")
                    .args(["-no", "LABEL", &source])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                usable(String::from_utf8_lossy(&output.stdout).trim())
            };
            let info = from_diskutil()
                .or_else(from_findmnt)
                .or_else(from_lsblk)
                .unwrap_or_default();
            if info.is_empty() {
                debug3!("boot volume: diskutil/findmnt/lsblk empty");
            } else {
                debug3!("boot volume cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// FileVault full-disk encryption status from `fdesetup status` (live; not OnceLock).
/// Distinct from boot volume `/boot-volume`, SSD `/ssd` · `/free`, and Disk Cleanup.
/// Used by Discord/operator instant `/filevault` · `/fv` lane.
pub fn get_filevault_status() -> String {
    let output = Command::new("/usr/bin/fdesetup")
        .arg("status")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("filevault: fdesetup spawn failed");
        return String::new();
    };
    if !output.status.success() {
        debug3!("filevault: fdesetup non-zero");
        return String::new();
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = stdout.to_ascii_lowercase();
    // Prefer On before Off so "On." wins over any trailing Off chatter.
    let info = if lower.contains("filevault is on") || lower.contains("filevault is enabled") {
        "On".to_string()
    } else if lower.contains("filevault is off") || lower.contains("filevault is disabled") {
        "Off".to_string()
    } else {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            String::new()
        } else {
            t.to_string()
        }
    };
    if info.is_empty() {
        debug3!("filevault: empty status");
    } else {
        debug3!("filevault status: '{}'", info);
    }
    info
}

/// System Integrity Protection status from `csrutil status` (live; not OnceLock).
/// Distinct from FileVault `/filevault`, Gatekeeper, and boot volume.
/// Used by Discord/operator instant `/sip` · `/csrutil` lane.
pub fn get_sip_status() -> String {
    let output = Command::new("/usr/bin/csrutil")
        .arg("status")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("sip: csrutil spawn failed");
        return String::new();
    };
    // csrutil may exit non-zero when disabled; still parse stdout.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = stdout.to_ascii_lowercase();
    let info = if lower.contains("enabled") {
        "Enabled".to_string()
    } else if lower.contains("disabled") {
        "Disabled".to_string()
    } else {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            String::new()
        } else {
            t.to_string()
        }
    };
    if info.is_empty() {
        debug3!("sip: empty status (exit={})", output.status);
    } else {
        debug3!("sip status: '{}'", info);
    }
    info
}

/// Gatekeeper assessment status from `spctl --status` (live; not OnceLock).
/// Distinct from SIP `/sip`, FileVault `/filevault`, and boot volume.
/// Used by Discord/operator instant `/gatekeeper` · `/spctl` lane.
pub fn get_gatekeeper_status() -> String {
    let output = Command::new("/usr/sbin/spctl")
        .arg("--status")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("gatekeeper: spctl spawn failed");
        return String::new();
    };
    // spctl may exit non-zero when disabled; still parse stdout.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = stdout.to_ascii_lowercase();
    // Prefer Enabled before Disabled so "assessments enabled" wins.
    let info = if lower.contains("assessments enabled")
        || (lower.contains("enabled") && !lower.contains("disabled"))
    {
        "Enabled".to_string()
    } else if lower.contains("assessments disabled") || lower.contains("disabled") {
        "Disabled".to_string()
    } else {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            String::new()
        } else {
            t.to_string()
        }
    };
    if info.is_empty() {
        debug3!("gatekeeper: empty status (exit={})", output.status);
    } else {
        debug3!("gatekeeper status: '{}'", info);
    }
    info
}

/// Parse `mdutil -s <vol>` Spotlight indexing line(s) → Enabled / Disabled / raw.
/// Ignores volume path headers (`/:`). Prefers Enabled before Disabled.
pub(crate) fn parse_mdutil_spotlight_status(raw: &str) -> Option<String> {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("indexing enabled") {
        return Some("Enabled".to_string());
    }
    if lower.contains("indexing disabled")
        || lower.contains("indexing and searching disabled")
        || lower.contains("searching disabled")
    {
        return Some("Disabled".to_string());
    }
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.ends_with(':') {
            continue;
        }
        let tl = t.to_ascii_lowercase();
        if tl.starts_with("error") || tl.contains("unknown") {
            return Some(t.trim_end_matches('.').to_string());
        }
        if tl.contains("index") || tl.contains("search") {
            return Some(t.trim_end_matches('.').to_string());
        }
    }
    None
}

/// macOS Spotlight indexing from `mdutil -s /` (live; not OnceLock).
/// Distinct from Time Machine `/timemachine`, Disk Cleanup, SIP `/sip`, Gatekeeper `/gatekeeper`,
/// and boot volume `/boot-volume`.
/// Used by Discord/operator instant `/spotlight` · `/mdutil` lane.
pub fn get_spotlight_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/mdutil")
            .args(["-s", "/"])
            .stderr(std::process::Stdio::piped())
            .output();
        let Ok(output) = output else {
            debug3!("spotlight: mdutil spawn failed");
            return String::new();
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = if stdout.trim().is_empty() {
            stderr.to_string()
        } else if stderr.trim().is_empty() {
            stdout.to_string()
        } else {
            format!("{}\n{}", stdout, stderr)
        };
        let info = parse_mdutil_spotlight_status(&combined).unwrap_or_default();
        if info.is_empty() {
            debug3!("spotlight: empty status (exit={})", output.status);
        } else {
            debug3!("spotlight status: '{}'", info);
        }
        info
    }
}

/// macOS Application Firewall global state from `socketfilterfw --getglobalstate` (live; not OnceLock).
/// Distinct from Gatekeeper `/gatekeeper`, SIP `/sip`, and FileVault `/filevault`.
/// Used by Discord/operator instant `/firewall` · `/afw` lane.
pub fn get_firewall_status() -> String {
    let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--getglobalstate")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("firewall: socketfilterfw spawn failed");
        return String::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = stdout.to_ascii_lowercase();
    let info = if lower.contains("enabled") && !lower.contains("disabled") {
        "Enabled".to_string()
    } else if lower.contains("disabled") {
        "Disabled".to_string()
    } else if lower.contains("state = 1") || lower.contains("state=1") {
        "Enabled".to_string()
    } else if lower.contains("state = 0") || lower.contains("state=0") {
        "Disabled".to_string()
    } else if lower.contains("state = 2") || lower.contains("state=2") {
        // State 2 = block all / stealth-ish modes on some macOS builds.
        "Block all".to_string()
    } else {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            String::new()
        } else {
            t.to_string()
        }
    };
    if info.is_empty() {
        debug3!("firewall: empty status (exit={})", output.status);
    } else {
        debug3!("firewall status: '{}'", info);
    }
    info
}

/// macOS Application Firewall stealth mode from `socketfilterfw --getstealthmode` (live; not OnceLock).
/// Distinct from Firewall global `/firewall`, Gatekeeper `/gatekeeper`, SIP `/sip`, and FileVault `/filevault`.
/// Used by Discord/operator instant `/stealth` · `/firewall-stealth` lane.
pub fn get_firewall_stealth_status() -> String {
    let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--getstealthmode")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("firewall-stealth: socketfilterfw spawn failed");
        return String::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = stdout.to_ascii_lowercase();
    let info = if lower.contains("enabled") && !lower.contains("disabled") {
        "Enabled".to_string()
    } else if lower.contains("disabled") {
        "Disabled".to_string()
    } else if lower.contains("on") && !lower.contains("off") {
        "Enabled".to_string()
    } else if lower.contains("off") {
        "Disabled".to_string()
    } else {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            String::new()
        } else {
            t.to_string()
        }
    };
    if info.is_empty() {
        debug3!("firewall-stealth: empty status (exit={})", output.status);
    } else {
        debug3!("firewall-stealth status: '{}'", info);
    }
    info
}

/// Parse `socketfilterfw --getloggingmode` (On / Off).
pub(crate) fn parse_firewall_logging_mode(raw: &str) -> Option<&'static str> {
    let lower = raw.to_ascii_lowercase();
    if lower.trim().is_empty() {
        return None;
    }
    if lower.contains("disabled") || lower.contains("is off") || lower.contains("logging off") {
        return Some("Off");
    }
    if lower.contains("enabled") || lower.contains("is on") || lower.contains("logging on") {
        return Some("On");
    }
    if lower.contains(" off") || lower.ends_with("off") {
        return Some("Off");
    }
    if lower.contains(" on") || lower.ends_with("on") {
        return Some("On");
    }
    None
}

/// Parse `socketfilterfw --getloggingopt` (throttled / detail / brief).
pub(crate) fn parse_firewall_logging_opt(raw: &str) -> Option<&'static str> {
    let lower = raw.to_ascii_lowercase();
    if lower.trim().is_empty() {
        return None;
    }
    if lower.contains("throttl") {
        return Some("throttled");
    }
    if lower.contains("detail") {
        return Some("detail");
    }
    if lower.contains("brief") {
        return Some("brief");
    }
    None
}

/// Combine logging mode + optional logging option into a short status line.
pub(crate) fn format_firewall_logging_parts(
    mode: Option<&str>,
    opt: Option<&str>,
) -> String {
    match (mode, opt) {
        (Some("Off"), _) => "Off".to_string(),
        (Some("On"), Some(o)) => format!("On · {o}"),
        (Some("On"), None) => "On".to_string(),
        (None, Some(o)) => o.to_string(),
        (None, None) => String::new(),
        (Some(other), Some(o)) => format!("{other} · {o}"),
        (Some(other), None) => other.to_string(),
    }
}

/// macOS Application Firewall logging from `socketfilterfw --getloggingmode` (+ `--getloggingopt`).
/// Distinct from Firewall global `/firewall`, stealth `/stealth`, Gatekeeper `/gatekeeper`, SIP `/sip`.
/// Used by Discord/operator instant `/firewall-logging` · `/fw-logging` lane.
pub fn get_firewall_logging_status() -> String {
    let mode_raw = {
        let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
            .arg("--getloggingmode")
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => String::new(),
            Err(_) => {
                debug3!("firewall-logging: socketfilterfw --getloggingmode spawn failed");
                String::new()
            }
        }
    };
    let opt_raw = {
        let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
            .arg("--getloggingopt")
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => String::new(),
            Err(_) => {
                debug3!("firewall-logging: socketfilterfw --getloggingopt spawn failed");
                String::new()
            }
        }
    };
    let mode = parse_firewall_logging_mode(&mode_raw);
    let opt = parse_firewall_logging_opt(&opt_raw);
    let info = format_firewall_logging_parts(mode, opt);
    if info.is_empty() {
        // Fall back to first line of mode raw when parsers miss.
        let t = mode_raw.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            debug3!("firewall-logging: empty status");
            String::new()
        } else {
            debug3!("firewall-logging status (raw): '{}'", t);
            t.to_string()
        }
    } else {
        debug3!("firewall-logging status: '{}'", info);
        info
    }
}

/// Parse `socketfilterfw --getallowsigned` (built-in + downloaded lines → On / Off).
pub(crate) fn parse_firewall_allow_signed_line(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    if lower.trim().is_empty() {
        return None;
    }
    if lower.contains("disabled") || lower.contains("is off") {
        return Some("Off");
    }
    if lower.contains("enabled") || lower.contains("is on") {
        return Some("On");
    }
    if lower.contains(" off") || lower.ends_with("off") {
        return Some("Off");
    }
    if lower.contains(" on") || lower.ends_with("on") {
        return Some("On");
    }
    None
}

/// Split `--getallowsigned` stdout into built-in + downloaded On/Off.
pub(crate) fn parse_firewall_allow_signed(raw: &str) -> (Option<&'static str>, Option<&'static str>) {
    let mut built_in = None;
    let mut downloaded = None;
    for line in raw.lines() {
        let lower = line.to_ascii_lowercase();
        let state = parse_firewall_allow_signed_line(line);
        if lower.contains("built-in") || lower.contains("built in") {
            built_in = state;
        } else if lower.contains("downloaded") {
            downloaded = state;
        }
    }
    (built_in, downloaded)
}

/// Combine built-in + downloaded allow-signed into a short status line.
pub(crate) fn format_firewall_allow_signed_parts(
    built_in: Option<&str>,
    downloaded: Option<&str>,
) -> String {
    match (built_in, downloaded) {
        (Some(b), Some(d)) => format!("Built-in {b} · Downloaded {d}"),
        (Some(b), None) => format!("Built-in {b}"),
        (None, Some(d)) => format!("Downloaded {d}"),
        (None, None) => String::new(),
    }
}

/// macOS Application Firewall allow-signed from `socketfilterfw --getallowsigned`.
/// Distinct from Firewall global `/firewall`, stealth `/stealth`, logging `/firewall-logging`,
/// block-all `/block-all`.
/// Used by Discord/operator instant `/allow-signed` · `/fw-allow-signed` lane.
pub fn get_firewall_allow_signed_status() -> String {
    let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--getallowsigned")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("firewall-allow-signed: socketfilterfw spawn failed");
        return String::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let (built_in, downloaded) = parse_firewall_allow_signed(&stdout);
    let info = format_firewall_allow_signed_parts(built_in, downloaded);
    if info.is_empty() {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            debug3!("firewall-allow-signed: empty status (exit={})", output.status);
            String::new()
        } else {
            debug3!("firewall-allow-signed status (raw): '{}'", t);
            t.to_string()
        }
    } else {
        debug3!("firewall-allow-signed status: '{}'", info);
        info
    }
}

/// Parse `socketfilterfw --getblockall` (Enabled / Disabled).
pub(crate) fn parse_firewall_block_all(raw: &str) -> Option<&'static str> {
    let lower = raw.to_ascii_lowercase();
    if lower.trim().is_empty() {
        return None;
    }
    if lower.contains("disabled") || lower.contains("is off") {
        return Some("Disabled");
    }
    if lower.contains("enabled") || lower.contains("is on") {
        return Some("Enabled");
    }
    if lower.contains(" off") || lower.ends_with("off") {
        return Some("Disabled");
    }
    if lower.contains(" on") || lower.ends_with("on") {
        return Some("Enabled");
    }
    None
}

/// macOS Application Firewall block-all from `socketfilterfw --getblockall`.
/// Distinct from Firewall global `/firewall`, stealth `/stealth`, logging `/firewall-logging`,
/// allow-signed `/allow-signed`.
/// Used by Discord/operator instant `/block-all` · `/fw-block-all` lane.
pub fn get_firewall_block_all_status() -> String {
    let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--getblockall")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("firewall-block-all: socketfilterfw spawn failed");
        return String::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(info) = parse_firewall_block_all(&stdout) {
        debug3!("firewall-block-all status: '{}'", info);
        return info.to_string();
    }
    let t = stdout.lines().next().unwrap_or("").trim();
    if t.is_empty() {
        debug3!("firewall-block-all: empty status (exit={})", output.status);
        String::new()
    } else {
        debug3!("firewall-block-all status (raw): '{}'", t);
        t.to_string()
    }
}

/// One Application Firewall listed app from `socketfilterfw --listapps`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FirewallListedApp {
    pub path: String,
    pub state: String,
}

/// Short label for a listed firewall app path (prefer `Foo.app`, else basename).
pub(crate) fn firewall_listapps_label(path: &str) -> &str {
    if let Some(idx) = path.find(".app") {
        let through_app = &path[..idx + 4];
        if let Some(slash) = through_app.rfind('/') {
            return &through_app[slash + 1..];
        }
        return through_app;
    }
    if let Some(slash) = path.rfind('/') {
        return &path[slash + 1..];
    }
    path
}

/// Parse `socketfilterfw --listapps` into (total, rows).
pub(crate) fn parse_firewall_listapps(raw: &str) -> (usize, Vec<FirewallListedApp>) {
    let mut total = 0usize;
    let mut apps = Vec::new();
    let mut pending_path: Option<String> = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if lower.starts_with("total number of apps") {
            if let Some(n) = lower.rsplit('=').next() {
                total = n.trim().parse().unwrap_or(0);
            }
            continue;
        }
        if let Some((idx, rest)) = t.split_once(':') {
            let idx = idx.trim();
            if !idx.is_empty() && idx.chars().all(|c| c.is_ascii_digit()) {
                let path = rest.trim().to_string();
                if !path.is_empty() {
                    pending_path = Some(path);
                }
                continue;
            }
        }
        if t.starts_with('(') && t.ends_with(')') {
            if let Some(path) = pending_path.take() {
                let inner = t.trim_matches(|c| c == '(' || c == ')').trim();
                let inner_l = inner.to_ascii_lowercase();
                let state = if inner_l.contains("block") {
                    "Block".to_string()
                } else if inner_l.contains("allow") {
                    "Allow".to_string()
                } else {
                    inner.to_string()
                };
                apps.push(FirewallListedApp { path, state });
            }
        }
    }
    if total == 0 {
        total = apps.len();
    }
    (total, apps)
}

/// Format listapps rows for the instant lane (cap rows; `+N more` when truncated).
pub(crate) fn format_firewall_listapps_parts(
    total: usize,
    apps: &[FirewallListedApp],
    max_rows: usize,
) -> String {
    if apps.is_empty() && total == 0 {
        return String::new();
    }
    let mut lines = Vec::with_capacity(max_rows.saturating_add(2));
    lines.push(total.to_string());
    let show = apps.len().min(max_rows);
    for app in apps.iter().take(show) {
        lines.push(format!(
            "· {} — {}",
            firewall_listapps_label(&app.path),
            app.state
        ));
    }
    if apps.len() > show {
        lines.push(format!("_… +{} more_", apps.len() - show));
    }
    lines.join("\n")
}

/// macOS Application Firewall app list from `socketfilterfw --listapps`.
/// Distinct from Firewall global `/firewall`, stealth `/stealth`, logging `/firewall-logging`,
/// allow-signed `/allow-signed`, block-all `/block-all`.
/// Used by Discord/operator instant `/listapps` · `/firewall-apps` lane.
pub fn get_firewall_listapps_status() -> String {
    let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--listapps")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("firewall-listapps: socketfilterfw spawn failed");
        return String::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let (total, apps) = parse_firewall_listapps(&stdout);
    let info = format_firewall_listapps_parts(total, &apps, 12);
    if info.is_empty() {
        debug3!("firewall-listapps: empty list (exit={})", output.status);
        String::new()
    } else {
        debug3!("firewall-listapps: {} apps (showing up to 12)", total);
        info
    }
}


/// Time Machine session + destination from `tmutil status` / `destinationinfo` (live; not OnceLock).
/// Distinct from Disk Cleanup `view disk` / `tmutil-thin`, SSD `/ssd` · `/free`, and boot volume.
/// Used by Discord/operator instant `/timemachine` · `/tm` lane. Does not need Full Disk Access.
pub fn get_time_machine_status() -> String {
    let phase = {
        let output = Command::new("/usr/bin/tmutil")
            .arg("status")
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let running = stdout.lines().any(|l| {
                    let t = l.trim();
                    t.starts_with("Running")
                        && (t.contains("= 1") || t.contains("=1") || t.ends_with("1;"))
                });
                let percent = stdout.lines().find_map(|l| {
                    let t = l.trim();
                    if !t.starts_with("Percent") {
                        return None;
                    }
                    let raw = t
                        .split('=')
                        .nth(1)?
                        .trim()
                        .trim_matches(|c: char| c == '"' || c == ';')
                        .trim();
                    if raw.is_empty() || raw == "-1" {
                        return None;
                    }
                    // tmutil Percent is 0..1 float when running.
                    if let Ok(f) = raw.parse::<f64>() {
                        if (0.0..1.0).contains(&f) {
                            return Some(format!("{:.0}%", f * 100.0));
                        }
                        if (1.0..=100.0).contains(&f) {
                            return Some(format!("{:.0}%", f));
                        }
                    }
                    Some(raw.to_string())
                });
                if running {
                    match percent {
                        Some(p) => format!("Running ({p})"),
                        None => "Running".to_string(),
                    }
                } else if !stdout.trim().is_empty() {
                    "Idle".to_string()
                } else {
                    String::new()
                }
            }
            Err(_) => {
                debug3!("timemachine: tmutil status spawn failed");
                String::new()
            }
        }
    };

    let dest = {
        let output = Command::new("/usr/bin/tmutil")
            .arg("destinationinfo")
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut name: Option<String> = None;
                let mut mount: Option<String> = None;
                for line in stdout.lines() {
                    let t = line.trim();
                    if let Some(rest) = t.strip_prefix("Name") {
                        let v = rest
                            .trim_start_matches(|c: char| c == ':' || c.is_whitespace())
                            .trim();
                        if !v.is_empty() && name.is_none() {
                            name = Some(v.to_string());
                        }
                    } else if let Some(rest) = t.strip_prefix("Mount Point") {
                        let v = rest
                            .trim_start_matches(|c: char| c == ':' || c.is_whitespace())
                            .trim();
                        if !v.is_empty() && mount.is_none() {
                            mount = Some(v.to_string());
                        }
                    }
                }
                match (name, mount) {
                    (Some(n), Some(m)) => format!("{n} ({m})"),
                    (Some(n), None) => n,
                    (None, Some(m)) => m,
                    (None, None) => String::new(),
                }
            }
            Err(_) => {
                debug3!("timemachine: tmutil destinationinfo spawn failed");
                String::new()
            }
        }
    };

    let info = match (phase.is_empty(), dest.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!("{phase} · no destination"),
        (true, false) => format!("dest {dest}"),
        (false, false) => format!("{phase} · dest {dest}"),
    };
    if info.is_empty() {
        debug3!("timemachine: empty status");
    } else {
        debug3!("timemachine status: '{}'", info);
    }
    info
}

/// macOS Remote Login (SSH) from `systemsetup -getremotelogin` (live; not OnceLock).
/// Distinct from Firewall `/firewall`, stealth `/stealth`, VPN `/vpn`, and Time Machine `/timemachine`.
/// Used by Discord/operator instant `/remote-login` · `/ssh` lane.
pub fn get_remote_login_status() -> String {
    let output = Command::new("/usr/sbin/systemsetup")
        .arg("-getremotelogin")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("remote-login: systemsetup spawn failed");
        return String::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = stdout.to_ascii_lowercase();
    let info = if lower.contains("remote login: on")
        || lower.contains("remote login:on")
        || (lower.contains("on") && !lower.contains("off") && lower.contains("remote"))
    {
        "On".to_string()
    } else if lower.contains("remote login: off")
        || lower.contains("remote login:off")
        || (lower.contains("off") && lower.contains("remote"))
    {
        "Off".to_string()
    } else if lower.contains("enabled") && !lower.contains("disabled") {
        "On".to_string()
    } else if lower.contains("disabled") {
        "Off".to_string()
    } else {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            String::new()
        } else {
            // Strip the "Remote Login:" label when present.
            t.trim_start_matches("Remote Login:")
                .trim_start_matches("remote login:")
                .trim()
                .to_string()
        }
    };
    if info.is_empty() {
        debug3!("remote-login: empty status (exit={})", output.status);
    } else {
        debug3!("remote-login status: '{}'", info);
    }
    info
}

/// macOS Screen Sharing enablement from `launchctl print-disabled` + live state.
/// Distinct from Remote Login `/remote-login` · `/ssh`, Firewall `/firewall`, and display `/screen`.
/// Used by Discord/operator instant `/screen-sharing` · `/vnc` lane.
pub fn get_screen_sharing_status() -> String {
    let enabled = {
        let output = Command::new("/bin/launchctl")
            .args(["print-disabled", "system"])
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut found: Option<bool> = None;
                for line in stdout.lines() {
                    let l = line.trim();
                    if !l.contains("com.apple.screensharing") {
                        continue;
                    }
                    let lower = l.to_ascii_lowercase();
                    if lower.contains("=> disabled") || lower.ends_with("disabled") {
                        found = Some(false);
                    } else if lower.contains("=> enabled") || lower.ends_with("enabled") {
                        found = Some(true);
                    }
                    break;
                }
                found
            }
            Err(_) => {
                debug3!("screen-sharing: launchctl print-disabled spawn failed");
                None
            }
        }
    };
    let running = {
        let output = Command::new("/bin/launchctl")
            .args(["print", "system/com.apple.screensharing"])
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lower = stdout.to_ascii_lowercase();
                if lower.contains("state = running") {
                    Some(true)
                } else if lower.contains("state = not running")
                    || lower.contains("state = waiting")
                    || lower.contains("could not find")
                {
                    Some(false)
                } else if stdout.trim().is_empty() {
                    None
                } else {
                    Some(false)
                }
            }
            Err(_) => {
                debug3!("screen-sharing: launchctl print spawn failed");
                None
            }
        }
    };
    let info = match (enabled, running) {
        (Some(true), Some(true)) => "On · active".to_string(),
        (Some(true), Some(false)) => "On · idle".to_string(),
        (Some(true), None) => "On".to_string(),
        (Some(false), _) => "Off".to_string(),
        (None, Some(true)) => "active".to_string(),
        (None, Some(false)) => "idle".to_string(),
        (None, None) => String::new(),
    };
    if info.is_empty() {
        debug3!("screen-sharing: empty status");
    } else {
        debug3!("screen-sharing status: '{}'", info);
    }
    info
}

/// macOS File Sharing (SMB) enablement from `launchctl print-disabled` + live state.
/// Distinct from Screen Sharing `/screen-sharing` · `/vnc`, Remote Login `/remote-login` · `/ssh`,
/// and FileVault `/filevault`.
/// Used by Discord/operator instant `/file-sharing` · `/smb` lane.
pub fn get_file_sharing_status() -> String {
    let enabled = {
        let output = Command::new("/bin/launchctl")
            .args(["print-disabled", "system"])
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut found: Option<bool> = None;
                for line in stdout.lines() {
                    let l = line.trim();
                    // Prefer smbd; accept older com.apple.AppleFileServer when present.
                    if !(l.contains("com.apple.smbd") || l.contains("com.apple.AppleFileServer")) {
                        continue;
                    }
                    let lower = l.to_ascii_lowercase();
                    if lower.contains("=> disabled") || lower.ends_with("disabled") {
                        found = Some(false);
                    } else if lower.contains("=> enabled") || lower.ends_with("enabled") {
                        found = Some(true);
                    }
                    if l.contains("com.apple.smbd") {
                        break;
                    }
                }
                found
            }
            Err(_) => {
                debug3!("file-sharing: launchctl print-disabled spawn failed");
                None
            }
        }
    };
    let running = {
        let output = Command::new("/bin/launchctl")
            .args(["print", "system/com.apple.smbd"])
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lower = stdout.to_ascii_lowercase();
                if lower.contains("state = running") {
                    Some(true)
                } else if lower.contains("state = not running")
                    || lower.contains("state = waiting")
                    || lower.contains("could not find")
                {
                    Some(false)
                } else if stdout.trim().is_empty() {
                    None
                } else {
                    Some(false)
                }
            }
            Err(_) => {
                debug3!("file-sharing: launchctl print spawn failed");
                None
            }
        }
    };
    let info = match (enabled, running) {
        (Some(true), Some(true)) => "On · active".to_string(),
        (Some(true), Some(false)) => "On · idle".to_string(),
        (Some(true), None) => "On".to_string(),
        (Some(false), _) => "Off".to_string(),
        (None, Some(true)) => "active".to_string(),
        (None, Some(false)) => "idle".to_string(),
        (None, None) => String::new(),
    };
    if info.is_empty() {
        debug3!("file-sharing: empty status");
    } else {
        debug3!("file-sharing status: '{}'", info);
    }
    info
}

/// macOS Printer Sharing from `cupsctl` `_share_printers`.
/// Distinct from File Sharing `/file-sharing` · `/smb`, Screen Sharing `/screen-sharing` · `/vnc`,
/// and Remote Login `/remote-login` · `/ssh`.
/// Used by Discord/operator instant `/printer-sharing` · `/share-printers` lane.
pub fn get_printer_sharing_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/sbin/cupsctl")
            .stderr(std::process::Stdio::null())
            .output();
        let info = match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut found: Option<bool> = None;
                for line in stdout.lines() {
                    let l = line.trim();
                    if !l.starts_with("_share_printers=") {
                        continue;
                    }
                    let val = l.split_once('=').map(|(_, v)| v.trim()).unwrap_or("");
                    found = Some(val == "1" || val.eq_ignore_ascii_case("true") || val == "yes");
                    break;
                }
                match found {
                    Some(true) => "On".to_string(),
                    Some(false) => "Off".to_string(),
                    None => String::new(),
                }
            }
            Err(_) => {
                debug3!("printer-sharing: cupsctl spawn failed");
                String::new()
            }
        };
        if info.is_empty() {
            debug3!("printer-sharing: empty status");
        } else {
            debug3!("printer-sharing status: '{}'", info);
        }
        info
    }
}

/// macOS Remote Management (ARD) from `RemoteManagement.launchd` marker + `ARDAgent` process.
/// Distinct from Screen Sharing `/screen-sharing` · `/vnc`, Remote Login `/remote-login` · `/ssh`,
/// File Sharing `/file-sharing` · `/smb`, and Printer Sharing `/printer-sharing`.
/// Used by Discord/operator instant `/remote-management` · `/ard` lane.
pub fn get_remote_management_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let marker = [
            "/Library/Application Support/Apple/Remote Desktop/RemoteManagement.launchd",
            "/private/etc/RemoteManagement.launchd",
            "/etc/RemoteManagement.launchd",
        ]
        .iter()
        .any(|p| std::path::Path::new(p).is_file());

        let agent = {
            let output = Command::new("/bin/ps")
                .args(["-axc", "-o", "comm="])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().any(|l| {
                        let t = l.trim();
                        t == "ARDAgent" || t.ends_with("/ARDAgent")
                    })
                }
                Err(_) => {
                    debug3!("remote-management: ps spawn failed");
                    false
                }
            }
        };

        let launchd_hint = {
            let output = Command::new("/bin/launchctl")
                .args(["list"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().any(|l| {
                        let lower = l.to_ascii_lowercase();
                        lower.contains("remotedesktop")
                            || lower.contains("remote_desktop")
                            || lower.contains("ardagent")
                    })
                }
                Err(_) => {
                    debug3!("remote-management: launchctl list spawn failed");
                    false
                }
            }
        };

        let enabled = marker || launchd_hint || agent;
        let info = match (enabled, agent) {
            (true, true) if marker || launchd_hint => "On · active".to_string(),
            (true, false) if marker || launchd_hint => "On · idle".to_string(),
            (true, true) => "active".to_string(),
            (true, false) => "On".to_string(),
            (false, _) => "Off".to_string(),
        };
        debug3!("remote-management status: '{}'", info);
        info
    }
}

/// macOS Content Caching (AssetCache) from `AssetCacheManagerUtil` / prefs + `AssetCache` process.
/// Distinct from Remote Management `/remote-management` · `/ard`, File Sharing `/file-sharing` · `/smb`,
/// Printer Sharing `/printer-sharing`, Screen Sharing `/screen-sharing` · `/vnc`, and Remote Login
/// `/remote-login` · `/ssh`.
/// Used by Discord/operator instant `/content-caching` · `/asset-cache` lane.
pub fn get_content_caching_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let activated = {
            // Prefer AssetCacheManagerUtil status (Activated: YES/NO).
            let from_util = {
                let output = Command::new("/usr/bin/AssetCacheManagerUtil")
                    .args(["status"])
                    .stderr(std::process::Stdio::null())
                    .output();
                match output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let mut found: Option<bool> = None;
                        for line in stdout.lines() {
                            let lower = line.trim().to_ascii_lowercase();
                            if !(lower.contains("activated") || lower.contains("activation")) {
                                continue;
                            }
                            if lower.contains("yes")
                                || lower.contains("true")
                                || lower.ends_with(": 1")
                                || lower.contains("= 1")
                                || lower.contains("=1")
                            {
                                found = Some(true);
                                break;
                            }
                            if lower.contains("no")
                                || lower.contains("false")
                                || lower.ends_with(": 0")
                                || lower.contains("= 0")
                                || lower.contains("=0")
                            {
                                found = Some(false);
                                break;
                            }
                        }
                        found
                    }
                    Err(_) => {
                        debug3!("content-caching: AssetCacheManagerUtil spawn failed");
                        None
                    }
                }
            };
            if from_util.is_some() {
                from_util
            } else {
                // Fallback: defaults Activated on AssetCache plist.
                let output = Command::new("/usr/bin/defaults")
                    .args([
                        "read",
                        "/Library/Preferences/com.apple.AssetCache.plist",
                        "Activated",
                    ])
                    .stderr(std::process::Stdio::null())
                    .output();
                match output {
                    Ok(output) if output.status.success() => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let t = stdout.trim();
                        if t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
                        {
                            Some(true)
                        } else if t == "0"
                            || t.eq_ignore_ascii_case("false")
                            || t.eq_ignore_ascii_case("no")
                        {
                            Some(false)
                        } else {
                            None
                        }
                    }
                    Ok(_) => None,
                    Err(_) => {
                        debug3!("content-caching: defaults read spawn failed");
                        None
                    }
                }
            }
        };

        let agent = {
            let output = Command::new("/bin/ps")
                .args(["-axc", "-o", "comm="])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().any(|l| {
                        let t = l.trim();
                        t == "AssetCache"
                            || t.ends_with("/AssetCache")
                            || t == "AssetCacheTetherator"
                            || t.ends_with("/AssetCacheTetherator")
                    })
                }
                Err(_) => {
                    debug3!("content-caching: ps spawn failed");
                    false
                }
            }
        };

        let launchd_hint = {
            let output = Command::new("/bin/launchctl")
                .args(["list"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().any(|l| {
                        let lower = l.to_ascii_lowercase();
                        lower.contains("assetcache") || lower.contains("asset_cache")
                    })
                }
                Err(_) => {
                    debug3!("content-caching: launchctl list spawn failed");
                    false
                }
            }
        };

        let enabled = activated.unwrap_or(false) || launchd_hint || agent;
        let info = match (activated, enabled, agent) {
            (Some(true), _, true) => "On · active".to_string(),
            (Some(true), _, false) => "On · idle".to_string(),
            (Some(false), _, _) => "Off".to_string(),
            (None, true, true) if launchd_hint => "On · active".to_string(),
            (None, true, false) if launchd_hint => "On · idle".to_string(),
            (None, true, true) => "active".to_string(),
            (None, true, false) => "On".to_string(),
            (None, false, _) => "Off".to_string(),
        };
        debug3!("content-caching status: '{}'", info);
        info
    }
}

/// Parse PlistBuddy `Print :NAT:Enabled` stdout (0/1 / true/false).
pub(crate) fn parse_nat_enabled_plistbuddy(stdout: &str) -> Option<bool> {
    let t = stdout.trim();
    if t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes") {
        Some(true)
    } else if t == "0" || t.eq_ignore_ascii_case("false") || t.eq_ignore_ascii_case("no") {
        Some(false)
    } else {
        None
    }
}

/// Parse `launchctl print system/com.apple.NetworkSharing` for running state.
pub(crate) fn parse_network_sharing_running(stdout: &str) -> Option<bool> {
    for line in stdout.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("state = ") {
            let v = rest.trim();
            if v == "running" {
                return Some(true);
            }
            if v == "not running" || v.starts_with("not running") {
                return Some(false);
            }
        }
    }
    None
}

/// macOS Internet Sharing from NAT plist `Enabled` + `com.apple.NetworkSharing` daemon.
/// Reads only `:NAT:Enabled` via PlistBuddy (never dumps NetworkPassword).
/// Distinct from Content Caching `/content-caching`, File Sharing `/file-sharing` · `/smb`,
/// Remote Management `/remote-management` · `/ard`, and VPN `/vpn`.
/// Used by Discord/operator instant `/internet-sharing` · `/ics` lane.
pub fn get_internet_sharing_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let enabled = {
            let output = Command::new("/usr/libexec/PlistBuddy")
                .args([
                    "-c",
                    "Print :NAT:Enabled",
                    "/Library/Preferences/SystemConfiguration/com.apple.nat.plist",
                ])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_nat_enabled_plistbuddy(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("internet-sharing: PlistBuddy spawn failed");
                    None
                }
            }
        };

        let running = {
            let output = Command::new("/bin/launchctl")
                .args(["print", "system/com.apple.NetworkSharing"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_network_sharing_running(&stdout)
                }
                Err(_) => {
                    debug3!("internet-sharing: launchctl print spawn failed");
                    None
                }
            }
        };

        let info = match (enabled, running) {
            (Some(true), Some(true)) => "On · active".to_string(),
            (Some(true), Some(false)) => "On · idle".to_string(),
            (Some(true), None) => "On".to_string(),
            (Some(false), _) => "Off".to_string(),
            (None, Some(true)) => "active".to_string(),
            (None, Some(false)) => "idle".to_string(),
            (None, None) => String::new(),
        };
        if info.is_empty() {
            debug3!("internet-sharing: empty status");
        } else {
            debug3!("internet-sharing status: '{}'", info);
        }
        info
    }
}

/// macOS Bluetooth Sharing (OBEX file exchange) from Bluetooth prefs + `BluetoothFileExchange`.
/// Prefs: `FileTransferEnabledServer` / `BluetoothFileTransferEnabled` via PlistBuddy (0/1).
/// Distinct from Internet Sharing `/internet-sharing` · `/ics`, File Sharing `/file-sharing` · `/smb`,
/// Content Caching `/content-caching`, and Wi-Fi `/wifi`.
/// Used by Discord/operator instant `/bluetooth-sharing` · `/bt-sharing` lane.
pub fn get_bluetooth_sharing_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let enabled = {
            let mut found: Option<bool> = None;
            for key in [
                "Print :FileTransferEnabledServer",
                "Print :BluetoothFileTransferEnabled",
            ] {
                let output = Command::new("/usr/libexec/PlistBuddy")
                    .args([
                        "-c",
                        key,
                        "/Library/Preferences/com.apple.Bluetooth.plist",
                    ])
                    .stderr(std::process::Stdio::null())
                    .output();
                match output {
                    Ok(output) if output.status.success() => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if let Some(v) = parse_nat_enabled_plistbuddy(&stdout) {
                            found = Some(v);
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {
                        debug3!("bluetooth-sharing: PlistBuddy spawn failed");
                    }
                }
            }
            found
        };

        let exchange = {
            let output = Command::new("/bin/ps")
                .args(["-axc", "-o", "comm="])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().any(|l| {
                        let t = l.trim();
                        t == "BluetoothFileExchange"
                            || t.ends_with("/BluetoothFileExchange")
                    })
                }
                Err(_) => {
                    debug3!("bluetooth-sharing: ps spawn failed");
                    false
                }
            }
        };

        let info = match (enabled, exchange) {
            (Some(true), true) => "On · active".to_string(),
            (Some(true), false) => "On · idle".to_string(),
            (Some(false), _) => "Off".to_string(),
            (None, true) => "active".to_string(),
            (None, false) => String::new(),
        };
        if info.is_empty() {
            debug3!("bluetooth-sharing: empty status");
        } else {
            debug3!("bluetooth-sharing status: '{}'", info);
        }
        info
    }
}

/// Combine Home Sharing / Photos flags + `mediasharingd` into a short status line.
pub(crate) fn format_media_sharing_parts(
    home: Option<bool>,
    photos: Option<bool>,
    daemon: bool,
) -> String {
    match (home, photos, daemon) {
        (Some(true), Some(true), true) => "On · home · photos · active".to_string(),
        (Some(true), Some(true), false) => "On · home · photos".to_string(),
        (Some(true), _, true) => "On · home · active".to_string(),
        (Some(true), _, false) => "On · home".to_string(),
        (_, Some(true), true) => "On · photos · active".to_string(),
        (_, Some(true), false) => "On · photos".to_string(),
        (Some(false), Some(false), _) => "Off".to_string(),
        (Some(false), None, _) => "Off".to_string(),
        (None, Some(false), false) => "Off".to_string(),
        (None, None, true) => "active".to_string(),
        _ => String::new(),
    }
}

/// macOS Media Sharing (Home Sharing / Photos) from `com.apple.amp.mediasharingd` prefs.
/// Distinct from Bluetooth Sharing `/bluetooth-sharing` · `/bts`, File Sharing `/smb`,
/// and Internet Sharing `/internet-sharing` · `/ics`.
/// Used by Discord/operator instant `/media-sharing` · `/home-sharing` lane.
pub fn get_media_sharing_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let read_flag = |key: &str| -> Option<bool> {
            let output = Command::new("/usr/bin/defaults")
                .args(["read", "com.apple.amp.mediasharingd", key])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_nat_enabled_plistbuddy(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("media-sharing: defaults read spawn failed ({key})");
                    None
                }
            }
        };

        let home = read_flag("home-sharing-enabled");
        let photos = read_flag("photo-sharing-enabled");

        let daemon = {
            let output = Command::new("/bin/ps")
                .args(["-axc", "-o", "comm="])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().any(|l| {
                        let t = l.trim();
                        t == "mediasharingd" || t.ends_with("/mediasharingd")
                    })
                }
                Err(_) => {
                    debug3!("media-sharing: ps spawn failed");
                    false
                }
            }
        };

        let info = format_media_sharing_parts(home, photos, daemon);
        if info.is_empty() {
            debug3!("media-sharing: empty status");
        } else {
            debug3!("media-sharing status: '{}'", info);
        }
        info
    }
}

/// macOS Remote Apple Events from `systemsetup -getremoteappleevents` (live; not OnceLock).
/// Distinct from Remote Login `/remote-login` · `/ssh`, Remote Management `/remote-management` · `/ard`,
/// Screen Sharing `/screen-sharing` · `/vnc`, and Media Sharing `/media-sharing` · `/home-sharing`.
/// Used by Discord/operator instant `/remote-apple-events` · `/rae` · `/eppc` lane.
pub fn get_remote_apple_events_status() -> String {
    let output = Command::new("/usr/sbin/systemsetup")
        .arg("-getremoteappleevents")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("remote-apple-events: systemsetup spawn failed");
        return String::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = stdout.to_ascii_lowercase();
    let info = if lower.contains("remote apple events: on")
        || lower.contains("remote apple events:on")
        || (lower.contains("on")
            && !lower.contains("off")
            && lower.contains("apple")
            && lower.contains("event"))
    {
        "On".to_string()
    } else if lower.contains("remote apple events: off")
        || lower.contains("remote apple events:off")
        || (lower.contains("off") && lower.contains("apple") && lower.contains("event"))
    {
        "Off".to_string()
    } else if lower.contains("enabled") && !lower.contains("disabled") {
        "On".to_string()
    } else if lower.contains("disabled") {
        "Off".to_string()
    } else {
        let t = stdout.lines().next().unwrap_or("").trim();
        if t.is_empty() {
            String::new()
        } else {
            t.trim_start_matches("Remote Apple Events:")
                .trim_start_matches("remote apple events:")
                .trim()
                .to_string()
        }
    };
    if info.is_empty() {
        debug3!(
            "remote-apple-events: empty status (exit={})",
            output.status
        );
    } else {
        debug3!("remote-apple-events status: '{}'", info);
    }
    info
}

/// Combine AirPlay Receiver pref + advertising + listen ports into a short status line.
pub(crate) fn format_airplay_receiver_parts(
    enabled: Option<bool>,
    advertising: Option<i64>,
    listening: bool,
) -> String {
    match (enabled, listening) {
        (Some(false), _) => "Off".to_string(),
        (Some(true), true) => "On · listening".to_string(),
        (Some(true), false) => "On".to_string(),
        (None, true) => "On · listening".to_string(),
        (None, false) => match advertising {
            Some(0) => "Off".to_string(),
            Some(_) => "On".to_string(),
            None => String::new(),
        },
    }
}

/// True when localhost TCP 5000 or 7000 accepts (ControlCenter AirPlay Receiver).
pub(crate) fn airplay_receiver_ports_listening() -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let timeout = Duration::from_millis(80);
    for port in [5000_u16, 7000] {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if TcpStream::connect_timeout(&addr, timeout).is_ok() {
            return true;
        }
    }
    false
}

/// Parse `defaults -currentHost read …` AirPlay Receiver flag (0/1 / true/false / yes/no).
pub(crate) fn parse_airplay_receiver_enabled(raw: &str) -> Option<bool> {
    let t = raw.trim().to_ascii_lowercase();
    if t.is_empty() {
        return None;
    }
    if matches!(t.as_str(), "0" | "false" | "no" | "off") {
        return Some(false);
    }
    if matches!(t.as_str(), "1" | "true" | "yes" | "on") {
        return Some(true);
    }
    None
}

/// Parse AirplayReceiverAdvertising integer from defaults stdout.
pub(crate) fn parse_airplay_receiver_advertising(raw: &str) -> Option<i64> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<i64>().ok()
}

/// macOS AirPlay Receiver from Control Center host prefs + listen ports 5000/7000.
/// Distinct from Media Sharing `/media-sharing`, Remote Apple Events `/rae` · `/eppc`,
/// Screen Sharing `/vnc`, and display `/display`.
/// Used by Discord/operator instant `/airplay` · `/airplay-receiver` lane.
pub fn get_airplay_receiver_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let read_host = |key: &str| -> Option<String> {
            let output = Command::new("/usr/bin/defaults")
                .args(["-currentHost", "read", "com.apple.controlcenter", key])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if stdout.is_empty() {
                        None
                    } else {
                        Some(stdout)
                    }
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("airplay-receiver: defaults -currentHost read spawn failed ({key})");
                    None
                }
            }
        };

        let enabled = read_host("AirplayReceiverEnabled")
            .as_deref()
            .and_then(parse_airplay_receiver_enabled);
        let advertising = read_host("AirplayReceiverAdvertising")
            .as_deref()
            .and_then(parse_airplay_receiver_advertising);
        let listening = airplay_receiver_ports_listening();

        let info = format_airplay_receiver_parts(enabled, advertising, listening);
        if info.is_empty() {
            debug3!("airplay-receiver: empty status");
        } else {
            debug3!("airplay-receiver status: '{}'", info);
        }
        info
    }
}

/// macOS DVD or CD Sharing enablement from `launchctl print-disabled` + live `ODSAgent` state.
/// Distinct from Media Sharing `/media-sharing` · `/home-sharing`, AirPlay `/airplay`,
/// File Sharing `/file-sharing` · `/smb`, and Screen Sharing `/screen-sharing` · `/vnc`.
/// Used by Discord/operator instant `/dvd-sharing` · `/cd-sharing` · `/ods` lane.
pub fn get_dvd_cd_sharing_status() -> String {
    let enabled = {
        let output = Command::new("/bin/launchctl")
            .args(["print-disabled", "system"])
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut found: Option<bool> = None;
                for line in stdout.lines() {
                    let l = line.trim();
                    // Optical Disk Sharing agent (System Settings → Sharing → DVD or CD Sharing).
                    if !(l.contains("com.apple.ODSAgent") || l.contains("com.apple.odsagent")) {
                        continue;
                    }
                    let lower = l.to_ascii_lowercase();
                    if lower.contains("=> disabled") || lower.ends_with("disabled") {
                        found = Some(false);
                    } else if lower.contains("=> enabled") || lower.ends_with("enabled") {
                        found = Some(true);
                    }
                    break;
                }
                found
            }
            Err(_) => {
                debug3!("dvd-cd-sharing: launchctl print-disabled spawn failed");
                None
            }
        }
    };
    let running = {
        let output = Command::new("/bin/launchctl")
            .args(["print", "system/com.apple.ODSAgent"])
            .stderr(std::process::Stdio::null())
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lower = stdout.to_ascii_lowercase();
                if lower.contains("state = running") {
                    Some(true)
                } else if lower.contains("state = not running")
                    || lower.contains("state = waiting")
                    || lower.contains("could not find")
                {
                    Some(false)
                } else if stdout.trim().is_empty() {
                    None
                } else {
                    Some(false)
                }
            }
            Err(_) => {
                debug3!("dvd-cd-sharing: launchctl print spawn failed");
                None
            }
        }
    };
    let info = match (enabled, running) {
        (Some(true), Some(true)) => "On · active".to_string(),
        (Some(true), Some(false)) => "On · idle".to_string(),
        (Some(true), None) => "On".to_string(),
        (Some(false), _) => "Off".to_string(),
        (None, Some(true)) => "active".to_string(),
        (None, Some(false)) => "idle".to_string(),
        (None, None) => String::new(),
    };
    if info.is_empty() {
        debug3!("dvd-cd-sharing: empty status");
    } else {
        debug3!("dvd-cd-sharing status: '{}'", info);
    }
    info
}

/// Parse `defaults read com.apple.sharingd DiscoverableMode` (Off / Contacts Only / Everyone).
pub(crate) fn parse_airdrop_discoverable_mode(raw: &str) -> Option<&'static str> {
    let t = raw
        .trim()
        .trim_matches('"')
        .trim()
        .to_ascii_lowercase()
        .replace('_', " ")
        .replace('-', " ");
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.is_empty() {
        return None;
    }
    if matches!(
        t.as_str(),
        "off" | "0" | "disabled" | "no" | "false" | "none"
    ) {
        return Some("Off");
    }
    if matches!(
        t.as_str(),
        "1" | "contacts" | "contacts only" | "contacts only mode" | "contactsonly"
    ) || t.contains("contact")
    {
        return Some("Contacts Only");
    }
    if matches!(t.as_str(), "2" | "everyone" | "all" | "everybody")
        || t.contains("everyone")
        || t.contains("everybody")
    {
        return Some("Everyone");
    }
    None
}

/// Parse NetworkBrowser `DisableAirDrop` (1/true → forced Off).
pub(crate) fn parse_airdrop_disabled_flag(raw: &str) -> Option<bool> {
    let t = raw.trim().trim_matches('"').trim().to_ascii_lowercase();
    if t.is_empty() {
        return None;
    }
    if matches!(t.as_str(), "1" | "true" | "yes" | "on") {
        return Some(true);
    }
    if matches!(t.as_str(), "0" | "false" | "no" | "off") {
        return Some(false);
    }
    None
}

/// Combine DiscoverableMode + optional DisableAirDrop into a short status line.
pub(crate) fn format_airdrop_parts(
    mode: Option<&str>,
    disabled: Option<bool>,
) -> String {
    if disabled == Some(true) {
        return "Off".to_string();
    }
    mode.unwrap_or("").to_string()
}

/// macOS AirDrop discoverability from `com.apple.sharingd` DiscoverableMode (+ NetworkBrowser
/// DisableAirDrop). Distinct from AirPlay `/airplay`, Bluetooth Sharing `/bluetooth-sharing` · `/bts`,
/// Media Sharing `/media-sharing`, and File Sharing `/smb`.
/// Used by Discord/operator instant `/airdrop` · `/air-drop` lane.
pub fn get_airdrop_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let read_defaults = |domain: &str, key: &str| -> Option<String> {
            let output = Command::new("/usr/bin/defaults")
                .args(["read", domain, key])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if stdout.is_empty() {
                        None
                    } else {
                        Some(stdout)
                    }
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("airdrop: defaults read spawn failed ({domain} {key})");
                    None
                }
            }
        };

        let mode = read_defaults("com.apple.sharingd", "DiscoverableMode")
            .as_deref()
            .and_then(parse_airdrop_discoverable_mode);
        let disabled = read_defaults("com.apple.NetworkBrowser", "DisableAirDrop")
            .as_deref()
            .and_then(parse_airdrop_disabled_flag);

        let info = format_airdrop_parts(mode, disabled);
        if info.is_empty() {
            debug3!("airdrop: empty status");
        } else {
            debug3!("airdrop status: '{}'", info);
        }
        info
    }
}

/// Parse `defaults read … GuestEnabled` (0/1 / true/false).
pub(crate) fn parse_guest_enabled_defaults(raw: &str) -> Option<bool> {
    parse_nat_enabled_plistbuddy(raw)
}

/// Parse `sysadminctl -guestAccount status` stdout/stderr (“Guest account enabled/disabled”).
pub(crate) fn parse_guest_account_sysadminctl(raw: &str) -> Option<&'static str> {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("guest account enabled")
        || lower.contains("guest user enabled")
        || (lower.contains("enabled") && lower.contains("guest") && !lower.contains("disabled"))
    {
        return Some("On");
    }
    if lower.contains("guest account disabled")
        || lower.contains("guest user disabled")
        || (lower.contains("disabled") && lower.contains("guest"))
    {
        return Some("Off");
    }
    None
}

/// Combine loginwindow GuestEnabled + optional sysadminctl into a short On/Off line.
pub(crate) fn format_guest_account_parts(
    enabled: Option<bool>,
    sysadmin: Option<&str>,
) -> String {
    if let Some(on) = enabled {
        return if on {
            "On".to_string()
        } else {
            "Off".to_string()
        };
    }
    sysadmin.unwrap_or("").to_string()
}

/// macOS Guest Account from `/Library/Preferences/com.apple.loginwindow` GuestEnabled
/// (fallback: `sysadminctl -guestAccount status`). Distinct from login username `/username` ·
/// `/whoami`, FileVault `/filevault`, and SIP `/sip`.
/// Used by Discord/operator instant `/guest` · `/guest-account` lane.
pub fn get_guest_account_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let enabled = {
            let output = Command::new("/usr/bin/defaults")
                .args([
                    "read",
                    "/Library/Preferences/com.apple.loginwindow",
                    "GuestEnabled",
                ])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_guest_enabled_defaults(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("guest-account: defaults read spawn failed");
                    None
                }
            }
        };

        let sysadmin = {
            let output = Command::new("/usr/sbin/sysadminctl")
                .args(["-guestAccount", "status"])
                .output();
            match output {
                Ok(output) => {
                    let mut buf = String::from_utf8_lossy(&output.stdout).into_owned();
                    if !buf.is_empty() && !output.stderr.is_empty() {
                        buf.push('\n');
                    }
                    buf.push_str(&String::from_utf8_lossy(&output.stderr));
                    parse_guest_account_sysadminctl(&buf)
                }
                Err(_) => {
                    debug3!("guest-account: sysadminctl spawn failed");
                    None
                }
            }
        };

        let info = format_guest_account_parts(enabled, sysadmin);
        if info.is_empty() {
            debug3!("guest-account: empty status");
        } else {
            debug3!("guest-account status: '{}'", info);
        }
        info
    }
}

/// Parse loginwindow `autoLoginUser` (username string when Automatic Login is on).
pub(crate) fn parse_autologin_user_defaults(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("does not exist") {
        return None;
    }
    // `defaults` error text for missing key.
    if s.to_ascii_lowercase().contains("does not exist") {
        return None;
    }
    Some(s.to_string())
}

/// Parse `sysadminctl -autologin status` stdout/stderr into a short On/Off line.
pub(crate) fn parse_autologin_sysadminctl(raw: &str) -> Option<String> {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("filevault")
        && (lower.contains("disabled") || lower.contains("because"))
    {
        return Some("Off · FileVault".to_string());
    }
    if lower.contains("system administrator") && lower.contains("disabled") {
        return Some("Off · admin".to_string());
    }
    // “Automatic login user: name”
    if let Some(idx) = lower.find("automatic login user:") {
        let rest = raw[idx + "automatic login user:".len()..].trim();
        let name = rest
            .split(|c: char| c == '\n' || c == '\r')
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        if !name.is_empty() {
            return Some(format!("On · {name}"));
        }
    }
    if lower.contains("automatic login is off")
        || lower.contains("automatic login is disabled")
        || (lower.contains("autologin") && lower.contains("off") && !lower.contains("on"))
    {
        return Some("Off".to_string());
    }
    if lower.contains("automatic login is on")
        || lower.contains("automatic login is enabled")
        || lower.contains("automatic login enabled")
    {
        return Some("On".to_string());
    }
    None
}

/// Prefer defaults username when set; else sysadminctl On/Off (+ FileVault reason).
pub(crate) fn format_autologin_parts(
    user: Option<&str>,
    sysadmin: Option<&str>,
) -> String {
    if let Some(u) = user.map(str::trim).filter(|s| !s.is_empty()) {
        return format!("On · {u}");
    }
    sysadmin.unwrap_or("").to_string()
}

/// macOS Automatic Login from loginwindow `autoLoginUser` (+ `sysadminctl -autologin status`).
/// Distinct from Guest Account `/guest`, login username `/username` · `/whoami`, and
/// Remote Login `/remote-login` · `/ssh`.
/// Used by Discord/operator instant `/autologin` · `/auto-login` lane.
pub fn get_autologin_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let user = {
            let output = Command::new("/usr/bin/defaults")
                .args([
                    "read",
                    "/Library/Preferences/com.apple.loginwindow",
                    "autoLoginUser",
                ])
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_autologin_user_defaults(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("autologin: defaults read spawn failed");
                    None
                }
            }
        };

        let sysadmin = {
            let output = Command::new("/usr/sbin/sysadminctl")
                .args(["-autologin", "status"])
                .output();
            match output {
                Ok(output) => {
                    let mut buf = String::from_utf8_lossy(&output.stdout).into_owned();
                    if !buf.is_empty() && !output.stderr.is_empty() {
                        buf.push('\n');
                    }
                    buf.push_str(&String::from_utf8_lossy(&output.stderr));
                    parse_autologin_sysadminctl(&buf)
                }
                Err(_) => {
                    debug3!("autologin: sysadminctl spawn failed");
                    None
                }
            }
        };

        let info = format_autologin_parts(user.as_deref(), sysadmin.as_deref());
        if info.is_empty() {
            debug3!("autologin: empty status");
        } else {
            debug3!("autologin status: '{}'", info);
        }
        info
    }
}

/// Parse `defaults … askForPassword` (0/1 / true/false).
pub(crate) fn parse_screen_lock_ask_defaults(raw: &str) -> Option<bool> {
    parse_nat_enabled_plistbuddy(raw)
}

/// Parse `defaults … askForPasswordDelay` (seconds as int/float).
pub(crate) fn parse_screen_lock_delay_defaults(raw: &str) -> Option<f64> {
    let t = raw.trim();
    if t.is_empty() || t.to_ascii_lowercase().contains("does not exist") {
        return None;
    }
    t.parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.0)
}

/// Human delay for Screen Lock (0 → immediate; else Ns / Nm / Nh).
pub(crate) fn format_screen_lock_delay(secs: f64) -> String {
    if secs <= 0.0 {
        return "immediate".to_string();
    }
    if secs < 60.0 {
        return format!("{}s", secs.round() as i64);
    }
    if secs < 3600.0 {
        let m = (secs / 60.0).round() as i64;
        return format!("{m}m");
    }
    let h = (secs / 3600.0).round() as i64;
    format!("{h}h")
}

/// Combine askForPassword + optional delay into a short On/Off line.
pub(crate) fn format_screen_lock_parts(ask: Option<bool>, delay: Option<f64>) -> String {
    match ask {
        Some(false) => "Off".to_string(),
        Some(true) => match delay {
            Some(d) => format!("On · {}", format_screen_lock_delay(d)),
            None => "On".to_string(),
        },
        None => match delay {
            // Delay alone is a weak signal; prefer empty over guessing On.
            Some(_) | None => String::new(),
        },
    }
}

/// macOS Screen Lock from `com.apple.screensaver` askForPassword (+ askForPasswordDelay).
/// Prefer `-currentHost`, fall back to user domain. Distinct from Screen Sharing `/screen-sharing` ·
/// `/vnc`, display `/screen` · `/display`, and Automatic Login `/autologin`.
/// Used by Discord/operator instant `/screen-lock` · `/lock-screen` lane.
pub fn get_screen_lock_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let read_key = |current_host: bool, key: &str| -> Option<String> {
            let mut cmd = Command::new("/usr/bin/defaults");
            if current_host {
                cmd.arg("-currentHost");
            }
            cmd.args(["read", "com.apple.screensaver", key]);
            cmd.stderr(std::process::Stdio::null());
            match cmd.output() {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                    if stdout.trim().is_empty() {
                        None
                    } else {
                        Some(stdout)
                    }
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("screen-lock: defaults read spawn failed ({key})");
                    None
                }
            }
        };

        let ask_raw = read_key(true, "askForPassword").or_else(|| read_key(false, "askForPassword"));
        let delay_raw =
            read_key(true, "askForPasswordDelay").or_else(|| read_key(false, "askForPasswordDelay"));

        let ask = ask_raw.as_deref().and_then(parse_screen_lock_ask_defaults);
        let delay = delay_raw
            .as_deref()
            .and_then(parse_screen_lock_delay_defaults);

        let info = format_screen_lock_parts(ask, delay);
        if info.is_empty() {
            debug3!("screen-lock: empty status");
        } else {
            debug3!("screen-lock status: '{}'", info);
        }
        info
    }
}

/// Parse `sysadminctl -smbGuestAccess status` (“SMB guest access enabled/disabled”).
pub(crate) fn parse_smb_guest_sysadminctl(raw: &str) -> Option<&'static str> {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("smb guest access enabled")
        || lower.contains("smb guest access is enabled")
        || lower.contains("smb guest access on")
        || (lower.contains("smb")
            && lower.contains("guest")
            && lower.contains("enabled")
            && !lower.contains("disabled"))
    {
        return Some("On");
    }
    if lower.contains("smb guest access disabled")
        || lower.contains("smb guest access is disabled")
        || lower.contains("smb guest access off")
        || (lower.contains("smb")
            && lower.contains("guest")
            && lower.contains("disabled"))
    {
        return Some("Off");
    }
    None
}

/// Parse `defaults … AllowGuestAccess` (0/1 / true/false) on the SMB server prefs.
pub(crate) fn parse_smb_guest_defaults(raw: &str) -> Option<bool> {
    parse_nat_enabled_plistbuddy(raw)
}

/// Prefer defaults AllowGuestAccess when set; else sysadminctl On/Off.
pub(crate) fn format_smb_guest_parts(
    enabled: Option<bool>,
    sysadmin: Option<&str>,
) -> String {
    if let Some(on) = enabled {
        return if on {
            "On".to_string()
        } else {
            "Off".to_string()
        };
    }
    sysadmin.unwrap_or("").to_string()
}

/// macOS SMB Guest Access from `com.apple.smb.server` AllowGuestAccess
/// (fallback: `sysadminctl -smbGuestAccess status`). Distinct from Guest Account `/guest`,
/// File Sharing `/file-sharing` · `/smb`, and Screen Lock `/screen-lock`.
/// Used by Discord/operator instant `/smb-guest` · `/guest-access` lane.
pub fn get_smb_guest_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let enabled = {
            let output = Command::new("/usr/bin/defaults")
                .args([
                    "read",
                    "/Library/Preferences/SystemConfiguration/com.apple.smb.server",
                    "AllowGuestAccess",
                ])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_smb_guest_defaults(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("smb-guest: defaults read spawn failed");
                    None
                }
            }
        };

        let sysadmin = {
            let output = Command::new("/usr/sbin/sysadminctl")
                .args(["-smbGuestAccess", "status"])
                .output();
            match output {
                Ok(output) => {
                    let mut buf = String::from_utf8_lossy(&output.stdout).into_owned();
                    if !buf.is_empty() && !output.stderr.is_empty() {
                        buf.push('\n');
                    }
                    buf.push_str(&String::from_utf8_lossy(&output.stderr));
                    parse_smb_guest_sysadminctl(&buf)
                }
                Err(_) => {
                    debug3!("smb-guest: sysadminctl spawn failed");
                    None
                }
            }
        };

        let info = format_smb_guest_parts(enabled, sysadmin);
        if info.is_empty() {
            debug3!("smb-guest: empty status");
        } else {
            debug3!("smb-guest status: '{}'", info);
        }
        info
    }
}

/// Parse `defaults … idleTime` (seconds; 0 = Never).
pub(crate) fn parse_screensaver_idle_defaults(raw: &str) -> Option<f64> {
    parse_screen_lock_delay_defaults(raw)
}

/// Human idle for Screensaver (0 → Never; else Ns / Nm / Nh).
pub(crate) fn format_screensaver_idle_secs(secs: f64) -> String {
    if secs <= 0.0 {
        return "Never".to_string();
    }
    format_screen_lock_delay(secs)
}

/// Format optional idleTime into a short status line.
pub(crate) fn format_screensaver_idle_parts(idle: Option<f64>) -> String {
    match idle {
        Some(secs) => format_screensaver_idle_secs(secs),
        None => String::new(),
    }
}

/// macOS Screensaver idle from `com.apple.screensaver` `idleTime` (seconds; 0 = Never).
/// Prefer `-currentHost`, fall back to user domain. Distinct from Screen Lock `/screen-lock`
/// (askForPassword), Screen Sharing `/screen-sharing` · `/vnc`, and display `/screen` · `/display`.
/// Used by Discord/operator instant `/screensaver` · `/screensaver-idle` · `/idle-time` lane.
pub fn get_screensaver_idle_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let read_key = |current_host: bool| -> Option<String> {
            let mut cmd = Command::new("/usr/bin/defaults");
            if current_host {
                cmd.arg("-currentHost");
            }
            cmd.args(["read", "com.apple.screensaver", "idleTime"]);
            cmd.stderr(std::process::Stdio::null());
            match cmd.output() {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                    if stdout.trim().is_empty() {
                        None
                    } else {
                        Some(stdout)
                    }
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("screensaver-idle: defaults read spawn failed");
                    None
                }
            }
        };

        let raw = read_key(true).or_else(|| read_key(false));
        let idle = raw.as_deref().and_then(parse_screensaver_idle_defaults);
        let info = format_screensaver_idle_parts(idle);
        if info.is_empty() {
            debug3!("screensaver-idle: empty status");
        } else {
            debug3!("screensaver-idle status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `powernap` flags → (battery, ac).
pub(crate) fn parse_pmset_custom_powernap(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("powernap") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `powernap` flag (Currently in use).
pub(crate) fn parse_pmset_active_powernap(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("powernap") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_power_nap_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => {
            if b {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        }
        (Some(b), Some(a)) => {
            let bat = if b { "On" } else { "Off" };
            let acs = if a { "On" } else { "Off" };
            format!("AC {acs} · Battery {bat}")
        }
        (Some(b), None) => {
            format!("{} (Battery)", if b { "On" } else { "Off" })
        }
        (None, Some(a)) => {
            format!("{} (AC)", if a { "On" } else { "Off" })
        }
        (None, None) => match active {
            Some(true) => "On".to_string(),
            Some(false) => "Off".to_string(),
            None => String::new(),
        },
    }
}

/// macOS Power Nap from `pmset -g custom` Battery/AC `powernap` (fallback: active `pmset -g`).
/// Distinct from Low Power Mode `/lpm`, power strip `/power`, watts `/watts`, and Screensaver
/// `/screensaver`. Used by Discord/operator instant `/powernap` · `/power-nap` lane.
pub fn get_power_nap_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_powernap(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("power-nap: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_powernap(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("power-nap: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_power_nap_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("power-nap: empty status");
        } else {
            debug3!("power-nap status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `womp` flags → (battery, ac).
pub(crate) fn parse_pmset_custom_womp(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("womp") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `womp` flag (Currently in use).
pub(crate) fn parse_pmset_active_womp(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("womp") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_wake_for_network_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => {
            if b {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        }
        (Some(b), Some(a)) => {
            let bat = if b { "On" } else { "Off" };
            let acs = if a { "On" } else { "Off" };
            format!("AC {acs} · Battery {bat}")
        }
        (Some(b), None) => {
            format!("{} (Battery)", if b { "On" } else { "Off" })
        }
        (None, Some(a)) => {
            format!("{} (AC)", if a { "On" } else { "Off" })
        }
        (None, None) => match active {
            Some(true) => "On".to_string(),
            Some(false) => "Off".to_string(),
            None => String::new(),
        },
    }
}

/// macOS Wake for network access from `pmset -g custom` Battery/AC `womp`
/// (fallback: active `pmset -g`). Distinct from Power Nap `/powernap`, Low Power Mode `/lpm`,
/// power strip `/power`, and Wi-Fi `/wifi`. Used by Discord/operator instant `/womp` ·
/// `/wake-for-network` lane.
pub fn get_wake_for_network_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_womp(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("wake-for-network: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_womp(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("wake-for-network: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_wake_for_network_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("wake-for-network: empty status");
        } else {
            debug3!("wake-for-network status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `displaysleep` minutes → (battery, ac).
pub(crate) fn parse_pmset_custom_displaysleep(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("displaysleep") {
            continue;
        }
        let mins = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = mins.or(battery),
            Some("ac") => ac = mins.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `displaysleep` minutes (Currently in use).
pub(crate) fn parse_pmset_active_displaysleep(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("displaysleep") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Human Display Sleep minutes (0 → Never; else Nm / Nh).
pub(crate) fn format_displaysleep_mins(mins: u32) -> String {
    if mins == 0 {
        return "Never".to_string();
    }
    if mins >= 60 && mins % 60 == 0 {
        return format!("{}h", mins / 60);
    }
    format!("{mins}m")
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_displaysleep_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => format_displaysleep_mins(b),
        (Some(b), Some(a)) => {
            format!(
                "AC {} · Battery {}",
                format_displaysleep_mins(a),
                format_displaysleep_mins(b)
            )
        }
        (Some(b), None) => format!("{} (Battery)", format_displaysleep_mins(b)),
        (None, Some(a)) => format!("{} (AC)", format_displaysleep_mins(a)),
        (None, None) => match active {
            Some(m) => format_displaysleep_mins(m),
            None => String::new(),
        },
    }
}

/// macOS Display Sleep from `pmset -g custom` Battery/AC `displaysleep` minutes
/// (fallback: active `pmset -g`). Distinct from Screensaver `/screensaver`, display resolution
/// `/display` · `/screen`, Power Nap `/powernap`, and Wake for network `/womp`.
/// Used by Discord/operator instant `/displaysleep` · `/display-sleep` lane.
pub fn get_displaysleep_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_displaysleep(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("displaysleep: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_displaysleep(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("displaysleep: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_displaysleep_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("displaysleep: empty status");
        } else {
            debug3!("displaysleep status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `disksleep` minutes → (battery, ac).
pub(crate) fn parse_pmset_custom_disksleep(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("disksleep") {
            continue;
        }
        let mins = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = mins.or(battery),
            Some("ac") => ac = mins.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `disksleep` minutes (Currently in use).
pub(crate) fn parse_pmset_active_disksleep(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("disksleep") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Human Disk Sleep minutes (0 → Never; else Nm / Nh).
pub(crate) fn format_disksleep_mins(mins: u32) -> String {
    format_displaysleep_mins(mins)
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_disksleep_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    format_displaysleep_parts(battery, ac, active)
}

/// macOS Disk Sleep from `pmset -g custom` Battery/AC `disksleep` minutes
/// (fallback: active `pmset -g`). Distinct from Display Sleep `/displaysleep`, Screensaver
/// `/screensaver`, system `sleep`, and Disk Cleanup. Used by Discord/operator instant
/// `/disksleep` · `/disk-sleep` lane.
pub fn get_disksleep_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_disksleep(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("disksleep: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_disksleep(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("disksleep: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_disksleep_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("disksleep: empty status");
        } else {
            debug3!("disksleep status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `proximitywake` flag → (battery, ac).
pub(crate) fn parse_pmset_custom_proximitywake(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("proximitywake") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `proximitywake` flag (Currently in use).
pub(crate) fn parse_pmset_active_proximitywake(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("proximitywake") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_proximitywake_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Proximity Wake from `pmset -g custom` Battery/AC `proximitywake`
/// (fallback: active `pmset -g`). Distinct from Wake for network `/womp`, Power Nap
/// `/powernap`, Display Sleep `/displaysleep`, and Disk Sleep `/disksleep`.
/// Used by Discord/operator instant `/proximitywake` · `/proximity-wake` lane.
pub fn get_proximitywake_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_proximitywake(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("proximitywake: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_proximitywake(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("proximitywake: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_proximitywake_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("proximitywake: empty status");
        } else {
            debug3!("proximitywake status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `ttyskeepawake` flag → (battery, ac).
pub(crate) fn parse_pmset_custom_ttyskeepawake(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("ttyskeepawake") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `ttyskeepawake` flag (Currently in use).
pub(crate) fn parse_pmset_active_ttyskeepawake(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("ttyskeepawake") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_ttyskeepawake_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS TTY keep awake from `pmset -g custom` Battery/AC `ttyskeepawake`
/// (fallback: active `pmset -g`). Distinct from Proximity Wake `/proximitywake`,
/// Wake for network `/womp`, Power Nap `/powernap`, and system `sleep`.
/// Used by Discord/operator instant `/ttyskeepawake` · `/tty-keep-awake` lane.
pub fn get_ttyskeepawake_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_ttyskeepawake(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("ttyskeepawake: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_ttyskeepawake(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("ttyskeepawake: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_ttyskeepawake_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("ttyskeepawake: empty status");
        } else {
            debug3!("ttyskeepawake status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `tcpkeepalive` flag → (battery, ac).
pub(crate) fn parse_pmset_custom_tcpkeepalive(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("tcpkeepalive") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `tcpkeepalive` flag (Currently in use).
pub(crate) fn parse_pmset_active_tcpkeepalive(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("tcpkeepalive") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_tcpkeepalive_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS TCP Keep Alive from `pmset -g custom` Battery/AC `tcpkeepalive`
/// (fallback: active `pmset -g`). Distinct from TTY keep awake `/ttyskeepawake`,
/// Wake for network `/womp`, and Network over sleep `/networkoversleep`.
/// Used by Discord/operator instant `/tcpkeepalive` · `/tcp-keep-alive` lane.
pub fn get_tcpkeepalive_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_tcpkeepalive(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("tcpkeepalive: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_tcpkeepalive(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("tcpkeepalive: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_tcpkeepalive_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("tcpkeepalive: empty status");
        } else {
            debug3!("tcpkeepalive status: '{}'", info);
        }
        info
    }
}


/// Parse `pmset -g custom` Battery/AC `networkoversleep` flag → (battery, ac).
pub(crate) fn parse_pmset_custom_networkoversleep(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("networkoversleep") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `networkoversleep` flag (Currently in use).
pub(crate) fn parse_pmset_active_networkoversleep(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("networkoversleep") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_networkoversleep_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Network over sleep from `pmset -g custom` Battery/AC `networkoversleep`
/// (fallback: active `pmset -g`). Distinct from TCP Keep Alive `/tcpkeepalive`,
/// Wake for network `/womp`, System Sleep `/systemsleep`, and hibernatemode.
/// Used by Discord/operator instant `/networkoversleep` · `/network-over-sleep` lane.
pub fn get_networkoversleep_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_networkoversleep(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("networkoversleep: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_networkoversleep(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("networkoversleep: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_networkoversleep_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("networkoversleep: empty status");
        } else {
            debug3!("networkoversleep status: '{}'", info);
        }
        info
    }
}

/// Human hibernatemode label (common pmset codes).
pub(crate) fn format_hibernatemode_code(mode: u32) -> String {
    match mode {
        0 => "0 (Sleep only)".to_string(),
        3 => "3 (Safe Sleep)".to_string(),
        25 => "25 (Hibernate only)".to_string(),
        n => format!("{n}"),
    }
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_hibernatemode_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => format_hibernatemode_code(b),
        (Some(b), Some(a)) => {
            format!(
                "AC {} · Battery {}",
                format_hibernatemode_code(a),
                format_hibernatemode_code(b)
            )
        }
        (Some(b), None) => format!("{} (Battery)", format_hibernatemode_code(b)),
        (None, Some(a)) => format!("{} (AC)", format_hibernatemode_code(a)),
        (None, None) => match active {
            Some(m) => format_hibernatemode_code(m),
            None => String::new(),
        },
    }
}

/// Parse `pmset -g custom` Battery/AC `hibernatemode` → (battery, ac).
pub(crate) fn parse_pmset_custom_hibernatemode(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("hibernatemode") {
            continue;
        }
        let mode = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = mode.or(battery),
            Some("ac") => ac = mode.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `hibernatemode` (Currently in use).
pub(crate) fn parse_pmset_active_hibernatemode(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("hibernatemode") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// macOS Hibernate mode from `pmset -g custom` Battery/AC `hibernatemode`
/// (fallback: active `pmset -g`). Distinct from System Sleep `/systemsleep`,
/// Network over sleep `/networkoversleep`, and TCP Keep Alive `/tcpkeepalive`.
/// Used by Discord/operator instant `/hibernatemode` · `/hibernate` lane.
pub fn get_hibernatemode_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_hibernatemode(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("hibernatemode: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_hibernatemode(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("hibernatemode: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_hibernatemode_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("hibernatemode: empty status");
        } else {
            debug3!("hibernatemode status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `standby` flag → (battery, ac).
/// Exact key `standby` only (not `standbydelay` / `standbydelayhigh` / `standbydelaylow`).
pub(crate) fn parse_pmset_custom_standby(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standby") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `standby` flag (Currently in use).
pub(crate) fn parse_pmset_active_standby(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standby") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_standby_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Standby from `pmset -g custom` Battery/AC `standby`
/// (fallback: active `pmset -g`). Distinct from Hibernate mode `/hibernatemode`,
/// System Sleep `/systemsleep`, Network over sleep `/networkoversleep`, and
/// Power Nap `/powernap`. Used by Discord/operator instant `/standby` lane.
pub fn get_standby_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_standby(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("standby: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_standby(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("standby: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_standby_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("standby: empty status");
        } else {
            debug3!("standby status: '{}'", info);
        }
        info
    }
}

/// Parse one `Sleep On Power Button <0|1>` line (multi-word pmset key).
pub(crate) fn parse_sleep_on_power_button_flag_line(line: &str) -> Option<bool> {
    let t = line.trim();
    let lower = t.to_ascii_lowercase();
    const PREFIX: &str = "sleep on power button";
    if !lower.starts_with(PREFIX) {
        return None;
    }
    let rest = t[PREFIX.len()..].trim();
    match rest.split_whitespace().next() {
        Some("1") | Some("true") | Some("yes") => Some(true),
        Some("0") | Some("false") | Some("no") => Some(false),
        _ => None,
    }
}

/// Parse `pmset -g custom` Battery/AC `Sleep On Power Button` → (battery, ac).
pub(crate) fn parse_pmset_custom_sleep_on_power_button(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let on = match parse_sleep_on_power_button_flag_line(t) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `Sleep On Power Button` flag (Currently in use).
pub(crate) fn parse_pmset_active_sleep_on_power_button(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        if let Some(v) = parse_sleep_on_power_button_flag_line(line) {
            return Some(v);
        }
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_sleep_on_power_button_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Sleep On Power Button from `pmset -g custom` Battery/AC
/// `Sleep On Power Button` (fallback: active `pmset -g`). Distinct from System Sleep
/// `/systemsleep` · `/sleep`, Standby `/standby`, and Hibernate mode `/hibernatemode`.
/// Used by Discord/operator instant `/sleep-on-power-button` · `/sopb` lane.
pub fn get_sleep_on_power_button_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_sleep_on_power_button(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("sleep-on-power-button: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_sleep_on_power_button(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("sleep-on-power-button: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_sleep_on_power_button_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("sleep-on-power-button: empty status");
        } else {
            debug3!("sleep-on-power-button status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `lessbright` flag → (battery, ac).
/// Exact key `lessbright` only (battery-only on many Macs; AC often absent).
pub(crate) fn parse_pmset_custom_lessbright(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("lessbright") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `lessbright` flag (Currently in use).
pub(crate) fn parse_pmset_active_lessbright(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("lessbright") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_lessbright_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Less Bright from `pmset -g custom` Battery/AC `lessbright`
/// (fallback: active `pmset -g`). Distinct from Display Sleep `/displaysleep`,
/// display resolution `/display`, Sleep On Power Button `/sopb`, and Low Power Mode
/// `/lpm`. Used by Discord/operator instant `/lessbright` · `/less-bright` lane.
pub fn get_lessbright_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_lessbright(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("lessbright: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_lessbright(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("lessbright: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_lessbright_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("lessbright: empty status");
        } else {
            debug3!("lessbright status: '{}'", info);
        }
        info
    }
}

/// Human `powermode` label (Apple Silicon / recent macOS pmset codes).
/// 0 = Automatic · 1 = Low Power · 2 = High Power.
pub(crate) fn format_powermode_code(mode: u32) -> String {
    match mode {
        0 => "0 (Automatic)".to_string(),
        1 => "1 (Low Power)".to_string(),
        2 => "2 (High Power)".to_string(),
        n => format!("{n}"),
    }
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_powermode_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => format_powermode_code(b),
        (Some(b), Some(a)) => {
            format!(
                "AC {} · Battery {}",
                format_powermode_code(a),
                format_powermode_code(b)
            )
        }
        (Some(b), None) => format!("{} (Battery)", format_powermode_code(b)),
        (None, Some(a)) => format!("{} (AC)", format_powermode_code(a)),
        (None, None) => match active {
            Some(m) => format_powermode_code(m),
            None => String::new(),
        },
    }
}

/// Parse `pmset -g custom` Battery/AC `powermode` → (battery, ac).
/// Exact key `powermode` only (not `lowpowermode` / `powernap`).
pub(crate) fn parse_pmset_custom_powermode(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("powermode") {
            continue;
        }
        let mode = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = mode.or(battery),
            Some("ac") => ac = mode.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `powermode` (Currently in use).
pub(crate) fn parse_pmset_active_powermode(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("powermode") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// macOS Power Mode from `pmset -g custom` Battery/AC `powermode`
/// (fallback: active `pmset -g`). Distinct from Low Power Mode chip `/lpm`,
/// Power Nap `/powernap`, power strip `/power`, and Less Bright `/lessbright`.
/// Used by Discord/operator instant `/powermode` · `/power-mode` lane.
pub fn get_powermode_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_powermode(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("powermode: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_powermode(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("powermode: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_powermode_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("powermode: empty status");
        } else {
            debug3!("powermode status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `halfdim` flag → (battery, ac).
/// Exact key `halfdim` only (often absent on Apple Silicon; Intel/legacy may show it).
pub(crate) fn parse_pmset_custom_halfdim(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("halfdim") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `halfdim` flag (Currently in use).
pub(crate) fn parse_pmset_active_halfdim(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("halfdim") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_halfdim_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Half Dim from `pmset -g custom` Battery/AC `halfdim`
/// (fallback: active `pmset -g`). Distinct from Less Bright `/lessbright`,
/// Display Sleep `/displaysleep`, and Power Mode `/powermode`.
/// Used by Discord/operator instant `/halfdim` · `/half-dim` lane.
pub fn get_halfdim_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_halfdim(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("halfdim: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_halfdim(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("halfdim: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_halfdim_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("halfdim: empty status");
        } else {
            debug3!("halfdim status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `hibernatefile` path → (battery, ac).
/// Exact key `hibernatefile` only (not `hibernatemode`). Path is the rest of the line.
pub(crate) fn parse_pmset_custom_hibernatefile(
    raw: &str,
) -> (Option<String>, Option<String>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("hibernatefile") {
            continue;
        }
        let path = parts.collect::<Vec<_>>().join(" ");
        if path.is_empty() {
            continue;
        }
        let path = Some(path);
        match section {
            Some("battery") => battery = path.or(battery),
            Some("ac") => ac = path.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `hibernatefile` path (Currently in use).
pub(crate) fn parse_pmset_active_hibernatefile(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("hibernatefile") {
            continue;
        }
        let path = parts.collect::<Vec<_>>().join(" ");
        if path.is_empty() {
            return None;
        }
        return Some(path);
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_hibernatefile_parts(
    battery: Option<String>,
    ac: Option<String>,
    active: Option<String>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => b,
        (Some(b), Some(a)) => format!("AC {a} · Battery {b}"),
        (Some(b), None) => format!("{b} (Battery)"),
        (None, Some(a)) => format!("{a} (AC)"),
        (None, None) => active.unwrap_or_default(),
    }
}

/// macOS Hibernate file from `pmset -g custom` Battery/AC `hibernatefile`
/// (fallback: active `pmset -g`). Distinct from Hibernate mode `/hibernatemode` · `/hibernate`,
/// System Sleep `/systemsleep`, and Standby `/standby`.
/// Used by Discord/operator instant `/hibernatefile` · `/sleepimage` lane.
pub fn get_hibernatefile_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_hibernatefile(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("hibernatefile: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_hibernatefile(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("hibernatefile: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_hibernatefile_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("hibernatefile: empty status");
        } else {
            debug3!("hibernatefile status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `standbydelay` seconds → (battery, ac).
/// Exact key `standbydelay` only (not `standby` / `standbydelayhigh` / `standbydelaylow`).
pub(crate) fn parse_pmset_custom_standbydelay(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standbydelay") {
            continue;
        }
        let secs = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = secs.or(battery),
            Some("ac") => ac = secs.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `standbydelay` seconds (Currently in use).
pub(crate) fn parse_pmset_active_standbydelay(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standbydelay") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Human Standby delay seconds (0 → Never; else Ns / Nm / Nh).
pub(crate) fn format_standbydelay_secs(secs: u32) -> String {
    if secs == 0 {
        return "Never".to_string();
    }
    if secs >= 3600 && secs % 3600 == 0 {
        return format!("{}h", secs / 3600);
    }
    if secs >= 60 && secs % 60 == 0 {
        return format!("{}m", secs / 60);
    }
    if secs >= 60 {
        return format!("{}m{}s", secs / 60, secs % 60);
    }
    format!("{secs}s")
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_standbydelay_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => format_standbydelay_secs(b),
        (Some(b), Some(a)) => {
            format!(
                "AC {} · Battery {}",
                format_standbydelay_secs(a),
                format_standbydelay_secs(b)
            )
        }
        (Some(b), None) => format!("{} (Battery)", format_standbydelay_secs(b)),
        (None, Some(a)) => format!("{} (AC)", format_standbydelay_secs(a)),
        (None, None) => match active {
            Some(s) => format_standbydelay_secs(s),
            None => String::new(),
        },
    }
}

/// macOS Standby delay from `pmset -g custom` Battery/AC `standbydelay`
/// (fallback: active `pmset -g`; often absent on Apple Silicon). Distinct from
/// Standby `/standby`, Hibernate file `/hibernatefile`, and Auto Power Off `/autopoweroff`.
/// Used by Discord/operator instant `/standbydelay` · `/standby-delay` lane.
pub fn get_standbydelay_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_standbydelay(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("standbydelay: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_standbydelay(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("standbydelay: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_standbydelay_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("standbydelay: empty status");
        } else {
            debug3!("standbydelay status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `autopoweroff` flag → (battery, ac).
/// Exact key `autopoweroff` only (not `autopoweroffdelay`).
pub(crate) fn parse_pmset_custom_autopoweroff(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("autopoweroff") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `autopoweroff` flag (Currently in use).
pub(crate) fn parse_pmset_active_autopoweroff(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("autopoweroff") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_autopoweroff_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Auto Power Off from `pmset -g custom` Battery/AC `autopoweroff`
/// (fallback: active `pmset -g`; often absent on Apple Silicon). Distinct from
/// Standby `/standby`, Standby delay `/standbydelay`, and autopoweroffdelay.
/// Used by Discord/operator instant `/autopoweroff` · `/auto-power-off` lane.
pub fn get_autopoweroff_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_autopoweroff(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("autopoweroff: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_autopoweroff(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("autopoweroff: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_autopoweroff_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("autopoweroff: empty status");
        } else {
            debug3!("autopoweroff status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `autopoweroffdelay` seconds → (battery, ac).
/// Exact key `autopoweroffdelay` only (not `autopoweroff`).
pub(crate) fn parse_pmset_custom_autopoweroffdelay(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("autopoweroffdelay") {
            continue;
        }
        let secs = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = secs.or(battery),
            Some("ac") => ac = secs.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `autopoweroffdelay` seconds (Currently in use).
pub(crate) fn parse_pmset_active_autopoweroffdelay(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("autopoweroffdelay") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_autopoweroffdelay_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    format_standbydelay_parts(battery, ac, active)
}

/// macOS Autopower Off delay from `pmset -g custom` Battery/AC `autopoweroffdelay`
/// (fallback: active `pmset -g`; often absent on Apple Silicon). Distinct from
/// Auto Power Off `/autopoweroff`, Standby delay `/standbydelay`,
/// Standby delay high `/standbydelayhigh`, and standbydelaylow.
/// Used by Discord/operator instant `/autopoweroffdelay` · `/auto-power-off-delay` lane.
pub fn get_autopoweroffdelay_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_autopoweroffdelay(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("autopoweroffdelay: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_autopoweroffdelay(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("autopoweroffdelay: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_autopoweroffdelay_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("autopoweroffdelay: empty status");
        } else {
            debug3!("autopoweroffdelay status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `standbydelayhigh` seconds → (battery, ac).
/// Exact key `standbydelayhigh` only (not `standbydelay` / `standbydelaylow` / `standby`).
pub(crate) fn parse_pmset_custom_standbydelayhigh(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standbydelayhigh") {
            continue;
        }
        let secs = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = secs.or(battery),
            Some("ac") => ac = secs.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `standbydelayhigh` seconds (Currently in use).
pub(crate) fn parse_pmset_active_standbydelayhigh(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standbydelayhigh") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_standbydelayhigh_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    format_standbydelay_parts(battery, ac, active)
}

/// macOS Standby delay high from `pmset -g custom` Battery/AC `standbydelayhigh`
/// (fallback: active `pmset -g`; often absent on Apple Silicon). Distinct from
/// Standby delay `/standbydelay`, Standby `/standby`, Autopower Off delay
/// `/autopoweroffdelay`, and Standby delay low `/standbydelaylow`.
/// Used by Discord/operator instant `/standbydelayhigh` · `/standby-delay-high` lane.
pub fn get_standbydelayhigh_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_standbydelayhigh(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("standbydelayhigh: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_standbydelayhigh(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("standbydelayhigh: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_standbydelayhigh_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("standbydelayhigh: empty status");
        } else {
            debug3!("standbydelayhigh status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `standbydelaylow` seconds → (battery, ac).
/// Exact key `standbydelaylow` only (not `standbydelay` / `standbydelayhigh` / `standby`).
pub(crate) fn parse_pmset_custom_standbydelaylow(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standbydelaylow") {
            continue;
        }
        let secs = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = secs.or(battery),
            Some("ac") => ac = secs.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `standbydelaylow` seconds (Currently in use).
pub(crate) fn parse_pmset_active_standbydelaylow(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("standbydelaylow") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_standbydelaylow_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    format_standbydelay_parts(battery, ac, active)
}

/// macOS Standby delay low from `pmset -g custom` Battery/AC `standbydelaylow`
/// (fallback: active `pmset -g`; often absent on Apple Silicon). Distinct from
/// Standby delay `/standbydelay`, Standby delay high `/standbydelayhigh`,
/// Standby `/standby`, and High Standby Threshold `/highstandbythreshold`.
/// Used by Discord/operator instant `/standbydelaylow` · `/standby-delay-low` lane.
pub fn get_standbydelaylow_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_standbydelaylow(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("standbydelaylow: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_standbydelaylow(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("standbydelaylow: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_standbydelaylow_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("standbydelaylow: empty status");
        } else {
            debug3!("standbydelaylow status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `highstandbythreshold` percent → (battery, ac).
/// Exact key `highstandbythreshold` only (not `standbydelay` / `standbydelayhigh` / `standbydelaylow`).
pub(crate) fn parse_pmset_custom_highstandbythreshold(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("highstandbythreshold") {
            continue;
        }
        let pct = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = pct.or(battery),
            Some("ac") => ac = pct.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `highstandbythreshold` percent (Currently in use).
pub(crate) fn parse_pmset_active_highstandbythreshold(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("highstandbythreshold") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Human High Standby Threshold percent.
pub(crate) fn format_highstandbythreshold_pct(pct: u32) -> String {
    format!("{pct}%")
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_highstandbythreshold_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => format_highstandbythreshold_pct(b),
        (Some(b), Some(a)) => {
            format!(
                "AC {} · Battery {}",
                format_highstandbythreshold_pct(a),
                format_highstandbythreshold_pct(b)
            )
        }
        (Some(b), None) => format!("{} (Battery)", format_highstandbythreshold_pct(b)),
        (None, Some(a)) => format!("{} (AC)", format_highstandbythreshold_pct(a)),
        (None, None) => match active {
            Some(p) => format_highstandbythreshold_pct(p),
            None => String::new(),
        },
    }
}

/// macOS High Standby Threshold from `pmset -g custom` Battery/AC `highstandbythreshold`
/// (fallback: active `pmset -g`; battery % that switches standbydelayhigh vs low; often absent
/// on Apple Silicon). Distinct from Standby delay `/standbydelay`, Standby delay high
/// `/standbydelayhigh`, Standby delay low `/standbydelaylow`, and Standby `/standby`.
/// Used by Discord/operator instant `/highstandbythreshold` · `/high-standby-threshold` lane.
pub fn get_highstandbythreshold_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_highstandbythreshold(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("highstandbythreshold: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_highstandbythreshold(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("highstandbythreshold: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_highstandbythreshold_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("highstandbythreshold: empty status");
        } else {
            debug3!("highstandbythreshold status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `destroyfvkeyonstandby` flag → (battery, ac).
/// Exact key `destroyfvkeyonstandby` only (often absent on Apple Silicon; Intel/legacy may show it).
pub(crate) fn parse_pmset_custom_destroyfvkeyonstandby(
    raw: &str,
) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("destroyfvkeyonstandby") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `destroyfvkeyonstandby` flag (Currently in use).
pub(crate) fn parse_pmset_active_destroyfvkeyonstandby(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("destroyfvkeyonstandby") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_destroyfvkeyonstandby_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Destroy FileVault key on standby from `pmset -g custom` Battery/AC
/// `destroyfvkeyonstandby` (fallback: active `pmset -g`; often absent on Apple Silicon).
/// Distinct from FileVault `/filevault`, Standby `/standby`, High Standby Threshold
/// `/highstandbythreshold`, and Hibernate mode `/hibernatemode`.
/// Used by Discord/operator instant `/destroyfvkeyonstandby` · `/destroy-fv-key-on-standby` lane.
pub fn get_destroyfvkeyonstandby_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_destroyfvkeyonstandby(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("destroyfvkeyonstandby: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_destroyfvkeyonstandby(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("destroyfvkeyonstandby: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_destroyfvkeyonstandby_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("destroyfvkeyonstandby: empty status");
        } else {
            debug3!("destroyfvkeyonstandby status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `acwake` flag → (battery, ac).
/// Exact key `acwake` only (often absent on Apple Silicon; Intel/legacy may show it).
pub(crate) fn parse_pmset_custom_acwake(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("acwake") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `acwake` flag (Currently in use).
pub(crate) fn parse_pmset_active_acwake(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("acwake") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_acwake_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS AC Wake from `pmset -g custom` Battery/AC `acwake`
/// (fallback: active `pmset -g`). Distinct from Wake for network `/womp`,
/// Proximity Wake `/proximitywake`, Lid Wake `/lidwake`, and Standby `/standby`.
/// Used by Discord/operator instant `/acwake` · `/ac-wake` lane.
pub fn get_acwake_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_acwake(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("acwake: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_acwake(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("acwake: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_acwake_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("acwake: empty status");
        } else {
            debug3!("acwake status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `lidwake` flag → (battery, ac).
/// Exact key `lidwake` only (often absent on Apple Silicon; Intel/legacy may show it).
pub(crate) fn parse_pmset_custom_lidwake(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("lidwake") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `lidwake` flag (Currently in use).
pub(crate) fn parse_pmset_active_lidwake(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("lidwake") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_lidwake_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Lid Wake from `pmset -g custom` Battery/AC `lidwake`
/// (fallback: active `pmset -g`). Distinct from Wake for network `/womp`,
/// Proximity Wake `/proximitywake`, AC Wake `/acwake`, and Standby `/standby`.
/// Used by Discord/operator instant `/lidwake` · `/lid-wake` lane.
pub fn get_lidwake_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_lidwake(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("lidwake: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_lidwake(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("lidwake: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_lidwake_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("lidwake: empty status");
        } else {
            debug3!("lidwake status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `autorestart` flag → (battery, ac).
/// Exact key `autorestart` only (automatic restart on power loss; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_autorestart(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("autorestart") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `autorestart` flag (Currently in use).
pub(crate) fn parse_pmset_active_autorestart(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("autorestart") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_autorestart_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Auto Restart from `pmset -g custom` Battery/AC `autorestart`
/// (fallback: active `pmset -g`). Distinct from Auto Power Off `/autopoweroff`
/// and Autopower Off delay `/autopoweroffdelay`.
/// Used by Discord/operator instant `/autorestart` · `/auto-restart` lane.
pub fn get_autorestart_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_autorestart(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("autorestart: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_autorestart(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("autorestart: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_autorestart_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("autorestart: empty status");
        } else {
            debug3!("autorestart status: '{}'", info);
        }
        info
    }
}

/// Human `gpuswitch` label (Intel dual-GPU / graphics switching pmset codes).
/// 0 = Integrated · 1 = Discrete · 2 = Auto (often absent on Apple Silicon).
pub(crate) fn format_gpuswitch_code(mode: u32) -> String {
    match mode {
        0 => "0 (Integrated)".to_string(),
        1 => "1 (Discrete)".to_string(),
        2 => "2 (Auto)".to_string(),
        n => format!("{n}"),
    }
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_gpuswitch_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    match (battery, ac) {
        (Some(b), Some(a)) if b == a => format_gpuswitch_code(b),
        (Some(b), Some(a)) => {
            format!(
                "AC {} · Battery {}",
                format_gpuswitch_code(a),
                format_gpuswitch_code(b)
            )
        }
        (Some(b), None) => format!("{} (Battery)", format_gpuswitch_code(b)),
        (None, Some(a)) => format!("{} (AC)", format_gpuswitch_code(a)),
        (None, None) => match active {
            Some(m) => format_gpuswitch_code(m),
            None => String::new(),
        },
    }
}

/// Parse `pmset -g custom` Battery/AC `gpuswitch` → (battery, ac).
/// Exact key `gpuswitch` only (graphics switching; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_gpuswitch(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("gpuswitch") {
            continue;
        }
        let mode = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = mode.or(battery),
            Some("ac") => ac = mode.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `gpuswitch` (Currently in use).
pub(crate) fn parse_pmset_active_gpuswitch(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("gpuswitch") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// macOS GPU Switch from `pmset -g custom` Battery/AC `gpuswitch`
/// (fallback: active `pmset -g`). Distinct from GPU usage ring `/gpu`,
/// GPU identity `/gpu-name`, and Auto Restart `/autorestart`.
/// Used by Discord/operator instant `/gpuswitch` · `/gpu-switch` lane.
pub fn get_gpuswitch_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_gpuswitch(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("gpuswitch: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_gpuswitch(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("gpuswitch: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_gpuswitch_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("gpuswitch: empty status");
        } else {
            debug3!("gpuswitch status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `darkwakes` flag → (battery, ac).
/// Exact key `darkwakes` only (legacy dark wake; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_darkwakes(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("darkwakes") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `darkwakes` flag (Currently in use).
pub(crate) fn parse_pmset_active_darkwakes(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("darkwakes") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_darkwakes_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Dark Wakes from `pmset -g custom` Battery/AC `darkwakes`
/// (fallback: active `pmset -g`). Distinct from GPU Switch `/gpuswitch`
/// and Auto Restart `/autorestart`.
/// Used by Discord/operator instant `/darkwakes` · `/dark-wakes` lane.
pub fn get_darkwakes_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_darkwakes(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("darkwakes: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_darkwakes(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("darkwakes: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_darkwakes_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("darkwakes: empty status");
        } else {
            debug3!("darkwakes status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `ttysclobber` flag → (battery, ac).
/// Exact key `ttysclobber` only (prevent tty from clobbering sleep assertions; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_ttysclobber(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("ttysclobber") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `ttysclobber` flag (Currently in use).
pub(crate) fn parse_pmset_active_ttysclobber(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("ttysclobber") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_ttysclobber_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS TTY Clobber from `pmset -g custom` Battery/AC `ttysclobber`
/// (fallback: active `pmset -g`). Distinct from TTY keep awake `/ttyskeepawake`
/// and Dark Wakes `/darkwakes`.
/// Used by Discord/operator instant `/ttysclobber` · `/ttys-clobber` lane.
pub fn get_ttysclobber_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_ttysclobber(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("ttysclobber: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_ttysclobber(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("ttysclobber: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_ttysclobber_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("ttysclobber: empty status");
        } else {
            debug3!("ttysclobber status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `norestoresleep` flag → (battery, ac).
/// Exact key `norestoresleep` only (do not restore sleep settings after wake; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_norestoresleep(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("norestoresleep") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `norestoresleep` flag (Currently in use).
pub(crate) fn parse_pmset_active_norestoresleep(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("norestoresleep") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_norestoresleep_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS No Restore Sleep from `pmset -g custom` Battery/AC `norestoresleep`
/// (fallback: active `pmset -g`). Distinct from System Sleep `/systemsleep`
/// and TTY Clobber `/ttysclobber`.
/// Used by Discord/operator instant `/norestoresleep` · `/no-restore-sleep` lane.
pub fn get_norestoresleep_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_norestoresleep(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("norestoresleep: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_norestoresleep(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("norestoresleep: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_norestoresleep_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("norestoresleep: empty status");
        } else {
            debug3!("norestoresleep status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `reducebrightness` flag → (battery, ac).
/// Exact key `reducebrightness` only (legacy reduce brightness; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_reducebrightness(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("reducebrightness") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `reducebrightness` flag (Currently in use).
pub(crate) fn parse_pmset_active_reducebrightness(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("reducebrightness") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_reducebrightness_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Reduce Brightness from `pmset -g custom` Battery/AC `reducebrightness`
/// (fallback: active `pmset -g`). Distinct from Less Bright `/lessbright`
/// and Half Dim `/halfdim`.
/// Used by Discord/operator instant `/reducebrightness` · `/reduce-brightness` lane.
pub fn get_reducebrightness_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_reducebrightness(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("reducebrightness: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_reducebrightness(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("reducebrightness: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_reducebrightness_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("reducebrightness: empty status");
        } else {
            debug3!("reducebrightness status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `sms` flag → (battery, ac).
/// Exact key `sms` only (Sudden Motion Sensor; often absent on Apple Silicon / SSDs).
pub(crate) fn parse_pmset_custom_sms(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("sms") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `sms` flag (Currently in use).
pub(crate) fn parse_pmset_active_sms(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("sms") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_sms_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Sudden Motion Sensor from `pmset -g custom` Battery/AC `sms`
/// (fallback: active `pmset -g`). Distinct from System Sleep `/systemsleep`
/// and Reduce Brightness `/reducebrightness`.
/// Used by Discord/operator instant `/sms` · `/sudden-motion` lane.
pub fn get_sms_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_sms(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("sms: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_sms(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("sms: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_sms_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("sms: empty status");
        } else {
            debug3!("sms status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `ring` flag → (battery, ac).
/// Exact key `ring` only (wake on modem ring; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_modem_ring(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("ring") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `ring` flag (Currently in use).
pub(crate) fn parse_pmset_active_modem_ring(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("ring") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_modem_ring_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Wake on Modem Ring from `pmset -g custom` Battery/AC `ring`
/// (fallback: active `pmset -g`). Distinct from CPU `/rings` · `/cpu-ring`
/// and Sudden Motion Sensor `/sms`.
/// Used by Discord/operator instant `/ring` · `/modem-ring` lane.
pub fn get_modem_ring_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_modem_ring(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("modem_ring: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_modem_ring(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("modem_ring: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_modem_ring_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("modem_ring: empty status");
        } else {
            debug3!("modem_ring status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `SleepDisabled` flag → (battery, ac).
/// Exact key `SleepDisabled` / `sleepdisabled` only (disable sleep; often absent on Apple Silicon).
pub(crate) fn parse_pmset_custom_sleepdisabled(raw: &str) -> (Option<bool>, Option<bool>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("SleepDisabled") {
            continue;
        }
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
        match section {
            Some("battery") => battery = on.or(battery),
            Some("ac") => ac = on.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `SleepDisabled` flag (Currently in use).
pub(crate) fn parse_pmset_active_sleepdisabled(raw: &str) -> Option<bool> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("SleepDisabled") {
            continue;
        }
        return match parts.next() {
            Some("1") | Some("true") | Some("yes") => Some(true),
            Some("0") | Some("false") | Some("no") => Some(false),
            _ => None,
        };
    }
    None
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_sleepdisabled_parts(
    battery: Option<bool>,
    ac: Option<bool>,
    active: Option<bool>,
) -> String {
    format_wake_for_network_parts(battery, ac, active)
}

/// macOS Sleep Disabled from `pmset -g custom` Battery/AC `SleepDisabled`
/// (fallback: active `pmset -g`). Distinct from System Sleep `/systemsleep`
/// and Wake on Modem Ring `/ring`.
/// Used by Discord/operator instant `/sleepdisabled` · `/sleep-disabled` lane.
pub fn get_sleepdisabled_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_sleepdisabled(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("sleepdisabled: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_sleepdisabled(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("sleepdisabled: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_sleepdisabled_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("sleepdisabled: empty status");
        } else {
            debug3!("sleepdisabled status: '{}'", info);
        }
        info
    }
}

/// Sleep-relevant assertion type from a `pmset -g assertions` owning-process line.
pub(crate) fn assertion_line_prevent_kind(line: &str) -> Option<&'static str> {
    let l = line.to_ascii_lowercase();
    if l.contains("preventuseridlesystemsleep") {
        Some("System")
    } else if l.contains("preventuseridledisplaysleep") {
        Some("Display")
    } else if l.contains("preventsystemsleep") {
        Some("NoSleep")
    } else if l.contains("externalmedia") {
        Some("Media")
    } else {
        None
    }
}

/// Extract `Name` from `pid 123(Name): …` (owning-process assertion line).
pub(crate) fn assertion_line_process_name(line: &str) -> Option<String> {
    let t = line.trim();
    let after_pid = t.strip_prefix("pid ")?;
    let open = after_pid.find('(')?;
    let close = after_pid[open + 1..].find(')')?;
    let name = after_pid[open + 1..open + 1 + close].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Parse system-wide prevent flags that are on (`1` / true / yes).
pub(crate) fn parse_pmset_assertions_flags(raw: &str) -> Vec<&'static str> {
    let mut flags = Vec::new();
    let mut in_listed = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with("Listed by owning process") {
            in_listed = true;
            continue;
        }
        if t.starts_with("Kernel Assertions") {
            break;
        }
        if in_listed {
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        let on = match parts.next() {
            Some("1") | Some("true") | Some("yes") => true,
            _ => false,
        };
        if !on {
            continue;
        }
        let kind = match key {
            k if k.eq_ignore_ascii_case("PreventUserIdleSystemSleep") => Some("System"),
            k if k.eq_ignore_ascii_case("PreventUserIdleDisplaySleep") => Some("Display"),
            k if k.eq_ignore_ascii_case("PreventSystemSleep") => Some("NoSleep"),
            k if k.eq_ignore_ascii_case("ExternalMedia") => Some("Media"),
            _ => None,
        };
        if let Some(k) = kind {
            if !flags.contains(&k) {
                flags.push(k);
            }
        }
    }
    flags
}

/// Parse owning-process holders that keep the Mac awake (deduped, order preserved).
pub(crate) fn parse_pmset_assertions_holders(raw: &str) -> Vec<String> {
    let mut holders = Vec::new();
    let mut in_listed = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with("Listed by owning process") {
            in_listed = true;
            continue;
        }
        if t.starts_with("Kernel Assertions") {
            break;
        }
        if !in_listed {
            continue;
        }
        if assertion_line_prevent_kind(t).is_none() {
            continue;
        }
        if let Some(name) = assertion_line_process_name(t) {
            if !holders
                .iter()
                .any(|h: &String| h.eq_ignore_ascii_case(&name))
            {
                holders.push(name);
            }
        }
    }
    holders
}

/// Compact status: flags + up to 5 holder names (+N overflow).
pub(crate) fn format_assertions_parts(flags: &[&str], holders: &[String]) -> String {
    let mut bits: Vec<String> = flags.iter().map(|s| (*s).to_string()).collect();
    const MAX_HOLDERS: usize = 5;
    let show = holders.len().min(MAX_HOLDERS);
    for name in holders.iter().take(show) {
        bits.push(name.clone());
    }
    if holders.len() > MAX_HOLDERS {
        bits.push(format!("(+{})", holders.len() - MAX_HOLDERS));
    }
    bits.join(" · ")
}

/// macOS sleep assertions from `pmset -g assertions` (who keeps the Mac awake).
/// Distinct from System Sleep `/systemsleep` and Sleep Disabled `/sleepdisabled`.
/// Used by Discord/operator instant `/assertions` · `/sleep-preventers` lane.
pub fn get_assertions_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "assertions"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).into_owned(),
            Ok(_) => {
                debug3!("assertions: pmset -g assertions non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("assertions: pmset -g assertions spawn failed");
                return String::new();
            }
        };
        let flags = parse_pmset_assertions_flags(&stdout);
        let holders = parse_pmset_assertions_holders(&stdout);
        let info = format_assertions_parts(&flags, &holders);
        if info.is_empty() {
            debug3!("assertions: empty status");
        } else {
            debug3!("assertions status: '{}'", info);
        }
        info
    }
}

/// Compact one-shot `pmset -g sched` line body (`wake at DATE TIME by …` → `wake DATE HH:MM`).
pub(crate) fn compact_pmset_sched_oneshot(rest: &str) -> Option<String> {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() >= 4 && parts[1].eq_ignore_ascii_case("at") {
        let kind = parts[0].to_ascii_lowercase();
        let date = parts[2];
        let time = parts[3];
        let time_short = {
            let segs: Vec<&str> = time.split(':').collect();
            if segs.len() >= 2 {
                format!("{}:{}", segs[0], segs[1])
            } else {
                time.to_string()
            }
        };
        return Some(format!("{kind} {date} {time_short}"));
    }
    let t = rest.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Parse scheduled + repeating power events from `pmset -g sched` stdout.
pub(crate) fn parse_pmset_sched_events(raw: &str) -> Vec<String> {
    let mut events = Vec::new();
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let tl = t.to_ascii_lowercase();
        if tl.starts_with("scheduled power")
            || tl.starts_with("repeating")
            || tl.starts_with("no scheduled")
            || tl.starts_with("no repeating")
            || tl.contains("scheduled events:")
        {
            continue;
        }
        if let Some(close) = t.find(']') {
            if t.starts_with('[') {
                let rest = t[close + 1..].trim();
                if let Some(compact) = compact_pmset_sched_oneshot(rest) {
                    events.push(compact);
                }
                continue;
            }
        }
        // Repeating / free-form rows (wake every Monday…, wakemondays …).
        let interesting = tl.contains("wake")
            || tl.contains("sleep")
            || tl.contains("shutdown")
            || tl.contains("restart")
            || tl.contains("poweron")
            || tl.contains("poweroff")
            || tl.contains("power on")
            || tl.contains("power off");
        if !interesting {
            continue;
        }
        let compact = t.split_whitespace().collect::<Vec<_>>().join(" ");
        if compact.len() > 2 && compact.len() < 96 {
            events.push(compact);
        }
    }
    events
}

/// Compact status: up to 5 events (+N overflow).
pub(crate) fn format_powersched_parts(events: &[String]) -> String {
    const MAX: usize = 5;
    let show = events.len().min(MAX);
    let mut bits: Vec<String> = events.iter().take(show).cloned().collect();
    if events.len() > MAX {
        bits.push(format!("(+{})", events.len() - MAX));
    }
    bits.join(" · ")
}

/// macOS power schedule from `pmset -g sched` (one-shot + repeating wake/sleep).
/// Distinct from Agent Ops `/schedules` and Sleep assertions `/assertions`.
/// Used by Discord/operator instant `/powersched` · `/power-schedule` lane.
pub fn get_powersched_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "sched"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => {
                debug3!("powersched: pmset -g sched non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("powersched: pmset -g sched spawn failed");
                return String::new();
            }
        };
        let events = parse_pmset_sched_events(&stdout);
        let info = format_powersched_parts(&events);
        if info.is_empty() {
            debug3!("powersched: empty status");
        } else {
            debug3!("powersched status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g sysload` levels → (combined, user, battery, thermal).
/// Lines look like `combined level = OK` / `- user level = OK`.
pub(crate) fn parse_pmset_sysload_levels(
    raw: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut combined = None;
    let mut user = None;
    let mut battery = None;
    let mut thermal = None;
    for line in raw.lines() {
        let t = line.trim().trim_start_matches('-').trim();
        let lower = t.to_ascii_lowercase();
        let Some(eq) = lower.find(" level = ") else {
            continue;
        };
        let key = lower[..eq].trim();
        // Value keeps original casing from the raw line after `=`.
        let Some(raw_eq) = t.find('=') else {
            continue;
        };
        let val = t[raw_eq + 1..].trim();
        if val.is_empty() {
            continue;
        }
        match key {
            "combined" => combined = Some(val.to_string()),
            "user" => user = Some(val.to_string()),
            "battery" => battery = Some(val.to_string()),
            "thermal" => thermal = Some(val.to_string()),
            _ => {}
        }
    }
    (combined, user, battery, thermal)
}

/// Compact sysload status: `OK · user OK · bat Great · therm Great`.
pub(crate) fn format_sysload_parts(
    combined: Option<&str>,
    user: Option<&str>,
    battery: Option<&str>,
    thermal: Option<&str>,
) -> String {
    let mut bits: Vec<String> = Vec::new();
    if let Some(v) = combined {
        bits.push(v.to_string());
    }
    if let Some(v) = user {
        bits.push(format!("user {v}"));
    }
    if let Some(v) = battery {
        bits.push(format!("bat {v}"));
    }
    if let Some(v) = thermal {
        bits.push(format!("therm {v}"));
    }
    bits.join(" · ")
}

/// macOS system load levels from `pmset -g sysload` (combined / user / battery / thermal).
/// Distinct from Details load averages `/load` · `/load-1m` and Heat `/thermal` · `/heat`.
/// Used by Discord/operator instant `/sysload` · `/load-level` lane.
pub fn get_sysload_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "sysload"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => {
                debug3!("sysload: pmset -g sysload non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("sysload: pmset -g sysload spawn failed");
                return String::new();
            }
        };
        let (combined, user, battery, thermal) = parse_pmset_sysload_levels(&stdout);
        let info = format_sysload_parts(
            combined.as_deref(),
            user.as_deref(),
            battery.as_deref(),
            thermal.as_deref(),
        );
        if info.is_empty() {
            debug3!("sysload: empty status");
        } else {
            debug3!("sysload status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g ups` settings → (haltlevel %, haltafter mins-or-neg, haltremain mins).
/// Typical keys: `haltlevel` · `haltafter` · `haltremain` (absent when no UPS policy).
pub(crate) fn parse_pmset_ups_settings(
    raw: &str,
) -> (Option<i32>, Option<i32>, Option<i32>) {
    let mut haltlevel = None;
    let mut haltafter = None;
    let mut haltremain = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.to_ascii_lowercase().contains("ups settings") {
            continue;
        }
        if t.to_ascii_lowercase().contains("no ups") {
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        let val = match parts.next().and_then(|v| v.parse::<i32>().ok()) {
            Some(v) => v,
            None => continue,
        };
        match key.to_ascii_lowercase().as_str() {
            "haltlevel" => haltlevel = Some(val),
            "haltafter" => haltafter = Some(val),
            "haltremain" => haltremain = Some(val),
            _ => {}
        }
    }
    (haltlevel, haltafter, haltremain)
}

fn format_ups_mins(mins: i32) -> String {
    if mins < 0 {
        "never".to_string()
    } else if mins == 0 {
        "0m".to_string()
    } else if mins % 60 == 0 && mins >= 60 {
        format!("{}h", mins / 60)
    } else if mins >= 60 {
        format!("{}h{}m", mins / 60, mins % 60)
    } else {
        format!("{mins}m")
    }
}

/// Compact UPS status: `halt 5% · after never · remain 10m`.
pub(crate) fn format_ups_parts(
    haltlevel: Option<i32>,
    haltafter: Option<i32>,
    haltremain: Option<i32>,
) -> String {
    let mut bits: Vec<String> = Vec::new();
    if let Some(v) = haltlevel {
        bits.push(format!("halt {v}%"));
    }
    if let Some(v) = haltafter {
        bits.push(format!("after {}", format_ups_mins(v)));
    }
    if let Some(v) = haltremain {
        bits.push(format!("remain {}", format_ups_mins(v)));
    }
    bits.join(" · ")
}

/// macOS UPS settings from `pmset -g ups` (haltlevel / haltafter / haltremain).
/// Distinct from Bat `%` `/battery`, AC charger `/charger`, and sysload `/sysload`.
/// Used by Discord/operator instant `/ups` · `/ups-settings` lane.
pub fn get_ups_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "ups"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => {
                debug3!("ups: pmset -g ups non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("ups: pmset -g ups spawn failed");
                return String::new();
            }
        };
        let (haltlevel, haltafter, haltremain) = parse_pmset_ups_settings(&stdout);
        let info = format_ups_parts(haltlevel, haltafter, haltremain);
        if info.is_empty() {
            debug3!("ups: empty status");
        } else {
            debug3!("ups status: '{}'", info);
        }
        info
    }
}

/// Compact one Sleep / Wake / DarkWake / Notification row from `pmset -g log`.
/// Line shape: `YYYY-MM-DD HH:MM:SS ±ZZZZ Domain\trest…`
pub(crate) fn compact_pmset_pmlog_line(line: &str) -> Option<String> {
    let t = line.trim();
    if t.len() < 28 {
        return None;
    }
    let mut parts = t.split_whitespace();
    let date = parts.next()?;
    let time = parts.next()?;
    let tz = parts.next()?;
    if date.len() < 10 || !date.as_bytes().get(4).copied().is_some_and(|b| b == b'-') {
        return None;
    }
    if time.len() < 5 || !time.as_bytes().get(2).copied().is_some_and(|b| b == b':') {
        return None;
    }
    if !(tz.starts_with('+') || tz.starts_with('-')) || tz.len() < 5 {
        return None;
    }
    let after_tz = {
        let idx = t.find(tz)?;
        t[idx + tz.len()..].trim_start()
    };
    let (domain, rest_owned) = match after_tz.find('\t') {
        Some(i) => (
            after_tz[..i].trim().to_string(),
            after_tz[i + 1..].trim().to_string(),
        ),
        None => {
            let mut it = after_tz.split_whitespace();
            let d = it.next()?.trim().to_string();
            (d, it.collect::<Vec<_>>().join(" "))
        }
    };
    match domain.as_str() {
        "Sleep" | "Wake" | "DarkWake" | "Notification" => {}
        _ => return None,
    }
    let md = if date.len() >= 10 { &date[5..] } else { date };
    let hhmm = &time[..5];
    let reason_tokens: Vec<&str> = rest_owned.split_whitespace().take(5).collect();
    let mut reason = reason_tokens.join(" ");
    if reason.len() > 42 {
        reason.truncate(39);
        reason.push('…');
    }
    if reason.is_empty() {
        Some(format!("{md} {hhmm} {domain}"))
    } else {
        Some(format!("{md} {hhmm} {domain} {reason}"))
    }
}

/// Collect Sleep / Wake / DarkWake / Notification events from `pmset -g log` stdout (chronological).
pub(crate) fn parse_pmset_pmlog_events(raw: &str) -> Vec<String> {
    let mut events = Vec::new();
    for line in raw.lines() {
        if let Some(compact) = compact_pmset_pmlog_line(line) {
            events.push(compact);
        }
    }
    events
}

/// Compact status: last up to 5 events (+N older overflow).
pub(crate) fn format_pmlog_parts(events: &[String]) -> String {
    const MAX: usize = 5;
    if events.is_empty() {
        return String::new();
    }
    let start = events.len().saturating_sub(MAX);
    let mut bits: Vec<String> = events[start..].to_vec();
    if events.len() > MAX {
        bits.insert(0, format!("(+{} older)", events.len() - MAX));
    }
    bits.join(" · ")
}

/// macOS power-management log from `pmset -g log` (recent Sleep / Wake / DarkWake / Notification).
/// Distinct from Sleep assertions `/assertions` (current holders), debug `/logs`, and UPS `/ups`.
/// Used by Discord/operator instant `/pmlog` · `/powerlog` lane.
pub fn get_pmlog_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "log"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => {
                debug3!("pmlog: pmset -g log non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("pmlog: pmset -g log spawn failed");
                return String::new();
            }
        };
        let events = parse_pmset_pmlog_events(&stdout);
        let info = format_pmlog_parts(&events);
        if info.is_empty() {
            debug3!("pmlog: empty status");
        } else {
            debug3!("pmlog status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g therm` CPU limits → (scheduler_limit %, available_cpus, speed_limit %).
/// Typical keys: `CPU_Scheduler_Limit` · `CPU_Available_CPUs` · `CPU_Speed_Limit`.
pub(crate) fn parse_pmset_therm_limits(
    raw: &str,
) -> (Option<u32>, Option<u32>, Option<u32>) {
    let mut sched = None;
    let mut cpus = None;
    let mut speed = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.to_ascii_lowercase().starts_with("note:") {
            continue;
        }
        let (key, val_s) = if let Some((k, v)) = t.split_once('=') {
            (k.trim(), v.trim())
        } else {
            let mut parts = t.split_whitespace();
            let k = match parts.next() {
                Some(k) => k,
                None => continue,
            };
            let v = match parts.next() {
                Some(v) => v,
                None => continue,
            };
            (k, v)
        };
        let val = match val_s.parse::<u32>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        match key.to_ascii_lowercase().as_str() {
            "cpu_scheduler_limit" => sched = Some(val),
            "cpu_available_cpus" => cpus = Some(val),
            "cpu_speed_limit" => speed = Some(val),
            _ => {}
        }
    }
    (sched, cpus, speed)
}

/// Compact therm status: `sched 100% · speed 100% · 8 cpus`.
pub(crate) fn format_therm_parts(
    sched: Option<u32>,
    cpus: Option<u32>,
    speed: Option<u32>,
) -> String {
    let mut bits: Vec<String> = Vec::new();
    if let Some(v) = sched {
        bits.push(format!("sched {v}%"));
    }
    if let Some(v) = speed {
        bits.push(format!("speed {v}%"));
    }
    if let Some(v) = cpus {
        bits.push(format!("{v} cpus"));
    }
    bits.join(" · ")
}

/// macOS thermal / CPU speed limits from `pmset -g therm` (not the live `thermlog` stream).
/// Distinct from Heat chip `/thermal` · `/heat` and Power log `/pmlog`.
/// Used by Discord/operator instant `/therm` · `/cpu-speed-limit` lane.
pub fn get_therm_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "therm"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => {
                debug3!("therm: pmset -g therm non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("therm: pmset -g therm spawn failed");
                return String::new();
            }
        };
        let (sched, cpus, speed) = parse_pmset_therm_limits(&stdout);
        let info = format_therm_parts(sched, cpus, speed);
        if info.is_empty() {
            debug3!("therm: empty status");
        } else {
            debug3!("therm status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g stats` → (sleep_count, dark_wake_count, user_wake_count).
/// Line shape: `Sleep Count:52` / `Dark Wake Count:52` / `User Wake Count:6`.
pub(crate) fn parse_pmset_stats_counts(
    raw: &str,
) -> (Option<u64>, Option<u64>, Option<u64>) {
    let mut sleep = None;
    let mut dark_wake = None;
    let mut user_wake = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        let Some(colon) = lower.find(':') else {
            continue;
        };
        let key = lower[..colon].trim();
        let val = t[colon + 1..].trim();
        let Ok(n) = val.parse::<u64>() else {
            continue;
        };
        match key {
            "sleep count" => sleep = Some(n),
            "dark wake count" => dark_wake = Some(n),
            "user wake count" => user_wake = Some(n),
            _ => {}
        }
    }
    (sleep, dark_wake, user_wake)
}

/// Compact power stats: `sleep 52 · dark wake 52 · user wake 6`.
pub(crate) fn format_pmstats_parts(
    sleep: Option<u64>,
    dark_wake: Option<u64>,
    user_wake: Option<u64>,
) -> String {
    let mut bits: Vec<String> = Vec::new();
    if let Some(v) = sleep {
        bits.push(format!("sleep {v}"));
    }
    if let Some(v) = dark_wake {
        bits.push(format!("dark wake {v}"));
    }
    if let Some(v) = user_wake {
        bits.push(format!("user wake {v}"));
    }
    bits.join(" · ")
}

/// macOS power stats from `pmset -g stats` (Sleep / Dark Wake / User Wake counts).
/// Distinct from Power log `/pmlog` (event lines), Dark Wakes `/darkwakes` (setting),
/// and Therm `/therm` (CPU speed limits).
/// Used by Discord/operator instant `/pmstats` · `/power-stats` lane.
pub fn get_pmstats_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "stats"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => {
                debug3!("pmstats: pmset -g stats non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("pmstats: pmset -g stats spawn failed");
                return String::new();
            }
        };
        let (sleep, dark_wake, user_wake) = parse_pmset_stats_counts(&stdout);
        let info = format_pmstats_parts(sleep, dark_wake, user_wake);
        if info.is_empty() {
            debug3!("pmstats: empty status");
        } else {
            debug3!("pmstats status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g cap` → supported capability tokens (sleep, displaysleep, …).
/// Skips header / prose lines; keeps single alphanumeric identifiers.
pub(crate) fn parse_pmset_cap_list(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if lower.contains("capabilities")
            || lower.contains("depending")
            || lower.contains("currently")
            || lower.contains("may change")
            || lower.contains("hardware")
        {
            continue;
        }
        let mut parts = t.split_whitespace();
        let Some(tok) = parts.next() else {
            continue;
        };
        if parts.next().is_some() {
            continue;
        }
        if !tok
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            continue;
        }
        let key = tok.to_ascii_lowercase();
        if !out.iter().any(|e| e == &key) {
            out.push(key);
        }
    }
    out
}

/// Compact power capabilities: `15 · sleep · displaysleep · …` (cap list at 12 + remainder).
pub(crate) fn format_pmcap_parts(caps: &[String]) -> String {
    if caps.is_empty() {
        return String::new();
    }
    let n = caps.len();
    const SHOW: usize = 12;
    if n <= SHOW {
        format!("{n} · {}", caps.join(" · "))
    } else {
        format!(
            "{n} · {} · +{} more",
            caps[..SHOW].join(" · "),
            n - SHOW
        )
    }
}

/// macOS power capabilities from `pmset -g cap` (supported settings list).
/// Distinct from Power stats `/pmstats` (counts), Therm `/therm`, AC charger `/adapter`.
/// Used by Discord/operator instant `/pmcap` · `/power-cap` lane.
pub fn get_pmcap_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pmset")
            .args(["-g", "cap"])
            .stderr(std::process::Stdio::null())
            .output();
        let stdout = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
            Ok(_) => {
                debug3!("pmcap: pmset -g cap non-zero");
                return String::new();
            }
            Err(_) => {
                debug3!("pmcap: pmset -g cap spawn failed");
                return String::new();
            }
        };
        let caps = parse_pmset_cap_list(&stdout);
        let info = format_pmcap_parts(&caps);
        if info.is_empty() {
            debug3!("pmcap: empty status");
        } else {
            debug3!("pmcap status: '{}'", info);
        }
        info
    }
}

/// Parse `pmset -g custom` Battery/AC `sleep` minutes → (battery, ac).
/// Exact key `sleep` only (not `displaysleep` / `disksleep` / `networkoversleep`).
pub(crate) fn parse_pmset_custom_systemsleep(raw: &str) -> (Option<u32>, Option<u32>) {
    let mut section: Option<&str> = None;
    let mut battery = None;
    let mut ac = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("Battery Power:") {
            section = Some("battery");
            continue;
        }
        if t.eq_ignore_ascii_case("AC Power:") {
            section = Some("ac");
            continue;
        }
        let mut parts = t.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("sleep") {
            continue;
        }
        let mins = match parts.next().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => Some(v),
            None => continue,
        };
        match section {
            Some("battery") => battery = mins.or(battery),
            Some("ac") => ac = mins.or(ac),
            _ => {}
        }
    }
    (battery, ac)
}

/// Parse active `pmset -g` `sleep` minutes (Currently in use).
pub(crate) fn parse_pmset_active_systemsleep(raw: &str) -> Option<u32> {
    for line in raw.lines() {
        let mut parts = line.trim().split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        if !key.eq_ignore_ascii_case("sleep") {
            continue;
        }
        return parts.next().and_then(|v| v.parse::<u32>().ok());
    }
    None
}

/// Human System Sleep minutes (0 → Never; else Nm / Nh).
pub(crate) fn format_systemsleep_mins(mins: u32) -> String {
    format_displaysleep_mins(mins)
}

/// Prefer Battery+AC custom when both set; else active `pmset -g`.
pub(crate) fn format_systemsleep_parts(
    battery: Option<u32>,
    ac: Option<u32>,
    active: Option<u32>,
) -> String {
    format_displaysleep_parts(battery, ac, active)
}

/// macOS System Sleep from `pmset -g custom` Battery/AC `sleep` minutes
/// (fallback: active `pmset -g`). Distinct from Display Sleep `/displaysleep`, Disk Sleep
/// `/disksleep`, Screensaver `/screensaver`, and TTY keep awake `/ttyskeepawake`.
/// Used by Discord/operator instant `/systemsleep` · `/system-sleep` lane.
pub fn get_systemsleep_status() -> String {
    #[cfg(not(target_os = "macos"))]
    {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        let (battery, ac) = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g", "custom"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_custom_systemsleep(&stdout)
                }
                Ok(_) => (None, None),
                Err(_) => {
                    debug3!("systemsleep: pmset -g custom spawn failed");
                    (None, None)
                }
            }
        };

        let active = {
            let output = Command::new("/usr/bin/pmset")
                .args(["-g"])
                .stderr(std::process::Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    parse_pmset_active_systemsleep(&stdout)
                }
                Ok(_) => None,
                Err(_) => {
                    debug3!("systemsleep: pmset -g spawn failed");
                    None
                }
            }
        };

        let info = format_systemsleep_parts(battery, ac, active);
        if info.is_empty() {
            debug3!("systemsleep: empty status");
        } else {
            debug3!("systemsleep status: '{}'", info);
        }
        info
    }
}

/// Local timezone — IANA when `/etc/localtime` points at zoneinfo, else abbr · offset.
/// Live (not OnceLock): DST abbr/offset change while the process stays up.
/// Used by the Discord/operator instant `/timezone` · `/tz` lane.
pub fn get_timezone() -> String {
    let iana = std::fs::read_link("/etc/localtime").ok().and_then(|p| {
        let s = p.to_string_lossy().into_owned();
        s.split("zoneinfo/")
            .nth(1)
            .map(|z| z.trim_matches('/').to_string())
            .filter(|z| !z.is_empty())
    });
    let now = chrono::Local::now();
    let abbr = now.format("%Z").to_string();
    let off_raw = now.format("%z").to_string();
    let offset = if off_raw.len() == 5 {
        format!("{}:{}", &off_raw[..3], &off_raw[3..])
    } else {
        off_raw
    };
    let info = match (iana, !abbr.is_empty()) {
        (Some(z), true) if !abbr.eq_ignore_ascii_case(&z) => {
            format!("{z} · {abbr} ({offset})")
        }
        (Some(z), _) => {
            if offset.is_empty() {
                z
            } else {
                format!("{z} ({offset})")
            }
        }
        (None, true) => {
            if offset.is_empty() {
                abbr
            } else {
                format!("{abbr} ({offset})")
            }
        }
        (None, false) => offset,
    };
    if info.is_empty() {
        debug3!("timezone: localtime/chrono empty");
    } else {
        debug3!("timezone: '{}'", info);
    }
    info
}

/// Cached machine architecture (`uname -m`, else `std::env::consts::ARCH`).
/// Used by the Discord/operator instant `/arch` · `/architecture` lane.
pub fn get_architecture() -> String {
    ARCHITECTURE_CACHE
        .get_or_init(|| {
            let run_uname = |bin: &str| -> Option<String> {
                let output = Command::new(bin)
                    .arg("-m")
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            let info = run_uname("/usr/bin/uname")
                .or_else(|| run_uname("uname"))
                .unwrap_or_else(|| std::env::consts::ARCH.to_string());
            if info.is_empty() {
                debug3!("architecture: uname/consts empty");
            } else {
                debug3!("architecture cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Cached kernel release (`uname -r`).
/// Used by the Discord/operator instant `/kernel` · `/uname-r` lane.
/// Distinct from machine arch (`/arch` · `uname -m`) and macOS ProductVersion (`/macos`).
pub fn get_kernel_release() -> String {
    KERNEL_RELEASE_CACHE
        .get_or_init(|| {
            let run_uname = |bin: &str| -> Option<String> {
                let output = Command::new(bin)
                    .arg("-r")
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            let info = run_uname("/usr/bin/uname")
                .or_else(|| run_uname("uname"))
                .unwrap_or_default();
            if info.is_empty() {
                debug3!("kernel release: uname -r empty");
            } else {
                debug3!("kernel release cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Cached system locale (`defaults AppleLocale`, else `LANG` without encoding).
/// Used by the Discord/operator instant `/locale` · `/language` lane.
/// Distinct from timezone (`/tz`) — this is language · region, not wall-clock zone.
pub fn get_locale() -> String {
    LOCALE_CACHE
        .get_or_init(|| {
            let from_defaults = || -> Option<String> {
                let output = Command::new("/usr/bin/defaults")
                    .args(["read", "-g", "AppleLocale"])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            let from_lang_env = || -> Option<String> {
                let raw = std::env::var("LANG").ok()?;
                let t = raw.trim();
                if t.is_empty() {
                    return None;
                }
                // "en_US.UTF-8" → "en_US"; skip C / POSIX.
                let base = t.split('.').next().unwrap_or(t).trim();
                if base.is_empty()
                    || base.eq_ignore_ascii_case("C")
                    || base.eq_ignore_ascii_case("POSIX")
                {
                    None
                } else {
                    Some(base.to_string())
                }
            };
            let info = from_defaults()
                .or_else(from_lang_env)
                .unwrap_or_default();
            if info.is_empty() {
                debug3!("locale: AppleLocale/LANG empty");
            } else {
                debug3!("locale cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Cached CPU core count (`system_profiler` P/E breakdown, else `sysctl hw.logicalcpu`).
/// Used by the Discord/operator instant `/cores` · `/ncpu` lane.
/// Distinct from `/chip` (name · cores string) and `/p-core` / `/e-core` (cluster clocks).
pub fn get_cpu_cores() -> String {
    CPU_CORES_CACHE
        .get_or_init(|| {
            let from_profiler = || -> Option<String> {
                let output = Command::new("/usr/sbin/system_profiler")
                    .args(["SPHardwareDataType", "-json"])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
                let hardware = json
                    .get("SPHardwareDataType")?
                    .as_array()?
                    .first()?;
                let num_procs = hardware.get("number_processors")?.as_str()?;
                let clean = num_procs.strip_prefix("proc ").unwrap_or(num_procs);
                let parts: Vec<&str> = clean.split(':').collect();
                // "16:12:4" → total:P:E
                if parts.len() >= 3 {
                    let total = parts[0].parse::<u32>().ok()?;
                    let p = parts[1].parse::<u32>().ok().unwrap_or(0);
                    let e = parts[2].parse::<u32>().ok().unwrap_or(0);
                    if total == 0 {
                        return None;
                    }
                    if p > 0 || e > 0 {
                        return Some(format!("{total} ({p}P + {e}E)"));
                    }
                    return Some(total.to_string());
                }
                if parts.len() == 1 {
                    let total = parts[0].parse::<u32>().ok()?;
                    if total > 0 {
                        return Some(total.to_string());
                    }
                }
                None
            };
            let sysctl_u32 = |key: &str| -> Option<u32> {
                let output = Command::new("/usr/sbin/sysctl")
                    .args(["-n", key])
                    .stderr(std::process::Stdio::null())
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                let s = String::from_utf8_lossy(&output.stdout);
                let n = s.trim().parse::<u32>().ok()?;
                if n == 0 {
                    None
                } else {
                    Some(n)
                }
            };
            let from_sysctl = || -> Option<String> {
                let logical = sysctl_u32("hw.logicalcpu")?;
                let physical = sysctl_u32("hw.physicalcpu");
                match physical {
                    Some(p) if p != logical => Some(format!("{logical} logical · {p} physical")),
                    _ => Some(format!("{logical} logical")),
                }
            };
            let info = from_profiler()
                .or_else(from_sysctl)
                .unwrap_or_default();
            if info.is_empty() {
                debug3!("cpu cores: profiler/sysctl empty");
            } else {
                debug3!("cpu cores cached: '{}'", info);
            }
            info
        })
        .clone()
}

/// Wall-clock last boot time from `sysinfo` uptime + local clock.
/// Used by the Discord/operator instant `/boot` · `/boot-time` lane.
/// Distinct from Mac Up duration (`/uptime`) and app process uptime (`/app-uptime`).
pub fn get_boot_time() -> String {
    let uptime = System::uptime();
    let now = chrono::Local::now();
    let boot = now - chrono::Duration::seconds(uptime as i64);
    let formatted = boot.format("%Y-%m-%d %H:%M %Z").to_string();
    debug3!("boot time: '{}' (uptime {}s)", formatted, uptime);
    formatted
}

/// Primary local LAN IPv4 (default-route iface via `route` + `ipconfig getifaddr`).
/// Live (not OnceLock): Wi-Fi / DHCP / VPN can change while the process stays up.
/// Used by the Discord/operator instant `/ip` · `/local-ip` lane.
/// Distinct from public/WAN IP (agent) — this is the Mac's LAN address only.
pub fn get_local_ipv4() -> String {
    let default_iface = || -> Option<String> {
        let output = Command::new("/sbin/route")
            .args(["-n", "get", "default"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("interface:") {
                let iface = rest.trim();
                if !iface.is_empty() {
                    return Some(iface.to_string());
                }
            }
        }
        None
    };
    let ifaddr = |iface: &str| -> Option<String> {
        let output = Command::new("/usr/sbin/ipconfig")
            .args(["getifaddr", iface])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&output.stdout);
        let ip = s.trim();
        if ip.is_empty() || !ip.contains('.') {
            None
        } else {
            Some(ip.to_string())
        }
    };
    let from_default = || -> Option<String> {
        let iface = default_iface()?;
        let ip = ifaddr(&iface)?;
        Some(format!("{ip} ({iface})"))
    };
    let from_fallback = || -> Option<String> {
        for iface in ["en0", "en1", "en2"] {
            if let Some(ip) = ifaddr(iface) {
                return Some(format!("{ip} ({iface})"));
            }
        }
        None
    };
    let info = from_default().or_else(from_fallback).unwrap_or_default();
    if info.is_empty() {
        debug3!("local ipv4: route/ipconfig empty");
    } else {
        debug3!("local ipv4: '{}'", info);
    }
    info
}

/// Default IPv4 gateway from `route -n get default` (same probe as local IP iface).
/// Live (not OnceLock): Wi-Fi / DHCP / VPN can change while the process stays up.
/// Used by the Discord/operator instant `/gateway` · `/router` lane.
/// Distinct from Discord bot gateway (`/discord`) and local LAN IP (`/ip`).
pub fn get_default_gateway() -> String {
    let output = Command::new("/sbin/route")
        .args(["-n", "get", "default"])
        .stderr(std::process::Stdio::null())
        .output();
    let info = match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut gateway = None;
            let mut iface = None;
            for line in stdout.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("gateway:") {
                    let g = rest.trim();
                    if !g.is_empty() && g != "default" {
                        gateway = Some(g.to_string());
                    }
                } else if let Some(rest) = t.strip_prefix("interface:") {
                    let i = rest.trim();
                    if !i.is_empty() {
                        iface = Some(i.to_string());
                    }
                }
            }
            match (gateway, iface) {
                (Some(g), Some(i)) => format!("{g} ({i})"),
                (Some(g), None) => g,
                _ => String::new(),
            }
        }
        _ => String::new(),
    };
    if info.is_empty() {
        debug3!("default gateway: route empty");
    } else {
        debug3!("default gateway: '{}'", info);
    }
    info
}

/// DNS nameservers from the first `scutil --dns` resolver block.
/// Live (not OnceLock): Wi-Fi / DHCP / VPN can change while the process stays up.
/// Used by the Discord/operator instant `/dns` · `/nameserver` lane.
/// Distinct from default gateway (`/gateway`) and local LAN IP (`/ip`).
pub fn get_dns_servers() -> String {
    let output = Command::new("/usr/sbin/scutil")
        .arg("--dns")
        .stderr(std::process::Stdio::null())
        .output();
    let info = match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut servers: Vec<String> = Vec::new();
            let mut in_first = false;
            for line in stdout.lines() {
                if line.starts_with("resolver #") {
                    if !servers.is_empty() {
                        break;
                    }
                    in_first = true;
                    continue;
                }
                if !in_first {
                    continue;
                }
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("nameserver[") {
                    if let Some(colon) = rest.find(':') {
                        let addr = rest[colon + 1..].trim();
                        if !addr.is_empty() && !servers.iter().any(|s| s == addr) {
                            servers.push(addr.to_string());
                        }
                    }
                } else if !t.is_empty()
                    && !line.starts_with(' ')
                    && !line.starts_with('\t')
                    && !servers.is_empty()
                {
                    // Left the indented resolver body.
                    break;
                }
            }
            servers.join(", ")
        }
        _ => String::new(),
    };
    if info.is_empty() {
        debug3!("dns servers: scutil empty");
    } else {
        debug3!("dns servers: '{}'", info);
    }
    info
}

/// Current Wi-Fi SSID from `networksetup -getairportnetwork` (en0/en1/en2).
/// Live (not OnceLock): join/leave AP can change while the process stays up.
/// Used by the Discord/operator instant `/wifi` · `/ssid` lane.
/// Distinct from local LAN IP (`/ip`), default gateway (`/gateway`), and DNS (`/dns`).
pub fn get_wifi_ssid() -> String {
    let parse = |stdout: &str| -> Option<String> {
        for line in stdout.lines() {
            let t = line.trim();
            // networksetup: "Current Wi-Fi Network: MySSID"
            if let Some(rest) = t.strip_prefix("Current Wi-Fi Network:") {
                let ssid = rest.trim();
                if !ssid.is_empty()
                    && !ssid.eq_ignore_ascii_case("you are not associated with an airport network.")
                {
                    return Some(ssid.to_string());
                }
            }
            // Some locales / tools: "Current Network: MySSID"
            if let Some(rest) = t.strip_prefix("Current Network:") {
                let ssid = rest.trim();
                if !ssid.is_empty() {
                    return Some(ssid.to_string());
                }
            }
        }
        None
    };
    let probe = |iface: &str| -> Option<String> {
        let output = Command::new("/usr/sbin/networksetup")
            .args(["-getairportnetwork", iface])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse(&stdout)
    };
    let info = ["en0", "en1", "en2"]
        .into_iter()
        .find_map(probe)
        .unwrap_or_default();
    if info.is_empty() {
        debug3!("wifi ssid: networksetup empty");
    } else {
        debug3!("wifi ssid: '{}'", info);
    }
    info
}

/// Primary interface MAC / ethernet address (default-route iface via `route` + `ifconfig`).
/// Live (not OnceLock): Wi-Fi / USB / VPN iface can change while the process stays up.
/// Used by the Discord/operator instant `/mac-address` · `/ether` lane.
/// Distinct from Mac marketing model (`/mac-model`) and hardware serial (`/serial`).
pub fn get_primary_mac_address() -> String {
    let default_iface = || -> Option<String> {
        let output = Command::new("/sbin/route")
            .args(["-n", "get", "default"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("interface:") {
                let iface = rest.trim();
                if !iface.is_empty() {
                    return Some(iface.to_string());
                }
            }
        }
        None
    };
    let ether_of = |iface: &str| -> Option<String> {
        let output = Command::new("/sbin/ifconfig")
            .arg(iface)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("ether ") {
                let mac = rest.split_whitespace().next().unwrap_or("").trim();
                if mac.len() >= 11 && mac.contains(':') {
                    return Some(mac.to_string());
                }
            }
        }
        None
    };
    let from_default = || -> Option<String> {
        let iface = default_iface()?;
        let mac = ether_of(&iface)?;
        Some(format!("{mac} ({iface})"))
    };
    let from_fallback = || -> Option<String> {
        for iface in ["en0", "en1", "en2"] {
            if let Some(mac) = ether_of(iface) {
                return Some(format!("{mac} ({iface})"));
            }
        }
        None
    };
    let info = from_default().or_else(from_fallback).unwrap_or_default();
    if info.is_empty() {
        debug3!("primary mac address: ifconfig empty");
    } else {
        debug3!("primary mac address: '{}'", info);
    }
    info
}

/// Primary interface MTU (default-route iface via `route` + `ifconfig`).
/// Live (not OnceLock): Wi-Fi / USB / VPN iface can change while the process stays up.
/// Used by the Discord/operator instant `/mtu` · `/if-mtu` lane.
/// Distinct from MAC (`/mac-address`), LAN IP (`/ip`), and Wi-Fi SSID (`/wifi`).
pub fn get_primary_mtu() -> String {
    let default_iface = || -> Option<String> {
        let output = Command::new("/sbin/route")
            .args(["-n", "get", "default"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("interface:") {
                let iface = rest.trim();
                if !iface.is_empty() {
                    return Some(iface.to_string());
                }
            }
        }
        None
    };
    let mtu_of = |iface: &str| -> Option<String> {
        let output = Command::new("/sbin/ifconfig")
            .arg(iface)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            // macOS / BSD: `en0: flags=… mtu 1500` (often on the header line).
            if let Some(idx) = t.find("mtu ") {
                let rest = &t[idx + 4..];
                let mtu = rest
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .unwrap_or("")
                    .trim();
                if !mtu.is_empty() && mtu.chars().all(|c| c.is_ascii_digit()) {
                    return Some(mtu.to_string());
                }
            }
        }
        None
    };
    let from_default = || -> Option<String> {
        let iface = default_iface()?;
        let mtu = mtu_of(&iface)?;
        Some(format!("{mtu} ({iface})"))
    };
    let from_fallback = || -> Option<String> {
        for iface in ["en0", "en1", "en2"] {
            if let Some(mtu) = mtu_of(iface) {
                return Some(format!("{mtu} ({iface})"));
            }
        }
        None
    };
    let info = from_default().or_else(from_fallback).unwrap_or_default();
    if info.is_empty() {
        debug3!("primary mtu: ifconfig empty");
    } else {
        debug3!("primary mtu: '{}'", info);
    }
    info
}


/// Primary interface IPv4 netmask / subnet mask (default-route iface via `route` + `ifconfig`).
/// Live (not OnceLock): Wi-Fi / USB / VPN iface can change while the process stays up.
/// Used by the Discord/operator instant `/netmask` · `/subnet` lane.
/// Distinct from local LAN IP (`/ip`), gateway (`/gateway`), DNS (`/dns`), MAC (`/mac-address`), MTU (`/mtu`).
pub fn get_primary_netmask() -> String {
    let default_iface = || -> Option<String> {
        let output = Command::new("/sbin/route")
            .args(["-n", "get", "default"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("interface:") {
                let iface = rest.trim();
                if !iface.is_empty() {
                    return Some(iface.to_string());
                }
            }
        }
        None
    };
    let hex_to_dotted = |raw: &str| -> Option<String> {
        let hex = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"))?;
        if hex.len() != 8 {
            return None;
        }
        let value = u32::from_str_radix(hex, 16).ok()?;
        Some(format!(
            "{}.{}.{}.{}",
            (value >> 24) & 0xff,
            (value >> 16) & 0xff,
            (value >> 8) & 0xff,
            value & 0xff
        ))
    };
    let cidr_of = |dotted: &str| -> Option<u8> {
        let mut parts = dotted.split('.');
        let mut bits: u32 = 0;
        let mut seen_zero = false;
        for _ in 0..4 {
            let octet: u8 = parts.next()?.parse().ok()?;
            for shift in (0..8).rev() {
                let on = (octet >> shift) & 1 == 1;
                if on {
                    if seen_zero {
                        return None;
                    }
                    bits += 1;
                } else {
                    seen_zero = true;
                }
            }
        }
        if parts.next().is_some() {
            return None;
        }
        Some(bits as u8)
    };
    let netmask_of = |iface: &str| -> Option<String> {
        let output = Command::new("/sbin/ifconfig")
            .arg(iface)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if !t.starts_with("inet ") {
                continue;
            }
            let parts: Vec<&str> = t.split_whitespace().collect();
            let mut i = 0;
            while i + 1 < parts.len() {
                if parts[i] == "netmask" {
                    let raw = parts[i + 1];
                    let dotted = if raw.contains('.') {
                        raw.to_string()
                    } else {
                        hex_to_dotted(raw)?
                    };
                    let cidr = cidr_of(&dotted)
                        .map(|c| format!(" /{c}"))
                        .unwrap_or_default();
                    return Some(format!("{dotted}{cidr} ({iface})"));
                }
                i += 1;
            }
        }
        None
    };
    let from_default = || -> Option<String> {
        let iface = default_iface()?;
        netmask_of(&iface)
    };
    let from_fallback = || -> Option<String> {
        for iface in ["en0", "en1", "en2"] {
            if let Some(mask) = netmask_of(iface) {
                return Some(mask);
            }
        }
        None
    };
    let info = from_default().or_else(from_fallback).unwrap_or_default();
    if info.is_empty() {
        debug3!("primary netmask: ifconfig empty");
    } else {
        debug3!("primary netmask: '{}'", info);
    }
    info
}

/// Primary / default-route interface name (from `route -n get default`).
/// Live (not OnceLock): Wi-Fi / USB / VPN iface can change while the process stays up.
/// Used by the Discord/operator instant `/iface` · `/ifname` lane.
/// Distinct from LAN IP (`/ip`), MAC (`/mac-address`), MTU (`/mtu`), netmask (`/netmask`).
pub fn get_primary_iface() -> String {
    let default_iface = || -> Option<String> {
        let output = Command::new("/sbin/route")
            .args(["-n", "get", "default"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("interface:") {
                let iface = rest.trim();
                if !iface.is_empty() {
                    return Some(iface.to_string());
                }
            }
        }
        None
    };
    let exists = |iface: &str| -> bool {
        let output = Command::new("/sbin/ifconfig")
            .arg(iface)
            .stderr(std::process::Stdio::null())
            .output();
        matches!(output, Ok(o) if o.status.success())
    };
    let from_fallback = || -> Option<String> {
        for iface in ["en0", "en1", "en2"] {
            if exists(iface) {
                return Some(iface.to_string());
            }
        }
        None
    };
    let info = default_iface().or_else(from_fallback).unwrap_or_default();
    if info.is_empty() {
        debug3!("primary iface: route/ifconfig empty");
    } else {
        debug3!("primary iface: '{}'", info);
    }
    info
}

/// Primary interface IPv4 broadcast address (default-route iface via `route` + `ifconfig`).
/// Live (not OnceLock): Wi-Fi / USB / VPN iface can change while the process stays up.
/// Used by the Discord/operator instant `/broadcast` · `/bcast` lane.
/// Distinct from LAN IP (`/ip`), netmask (`/netmask`), gateway (`/gateway`), iface (`/iface`).
pub fn get_primary_broadcast() -> String {
    let default_iface = || -> Option<String> {
        let output = Command::new("/sbin/route")
            .args(["-n", "get", "default"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("interface:") {
                let iface = rest.trim();
                if !iface.is_empty() {
                    return Some(iface.to_string());
                }
            }
        }
        None
    };
    let broadcast_of = |iface: &str| -> Option<String> {
        let output = Command::new("/sbin/ifconfig")
            .arg(iface)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if !t.starts_with("inet ") {
                continue;
            }
            let parts: Vec<&str> = t.split_whitespace().collect();
            let mut i = 0;
            while i + 1 < parts.len() {
                if parts[i] == "broadcast" {
                    let bcast = parts[i + 1];
                    if bcast.contains('.') && bcast.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        return Some(format!("{bcast} ({iface})"));
                    }
                }
                i += 1;
            }
        }
        None
    };
    let from_default = || -> Option<String> {
        let iface = default_iface()?;
        broadcast_of(&iface)
    };
    let from_fallback = || -> Option<String> {
        for iface in ["en0", "en1", "en2"] {
            if let Some(bcast) = broadcast_of(iface) {
                return Some(bcast);
            }
        }
        None
    };
    let info = from_default().or_else(from_fallback).unwrap_or_default();
    if info.is_empty() {
        debug3!("primary broadcast: ifconfig empty");
    } else {
        debug3!("primary broadcast: '{}'", info);
    }
    info
}

/// Primary interface link status · media / speed (default-route iface via `route` + `ifconfig`).
/// Live (not OnceLock): Wi-Fi / USB / VPN iface can change while the process stays up.
/// Used by the Discord/operator instant `/link-status` · `/media` · `/link-speed` lane.
/// Distinct from MTU (`/link-mtu`), MAC (`/link-address`), iface name (`/iface`), broadcast (`/broadcast`).
pub fn get_primary_link_status() -> String {
    let default_iface = || -> Option<String> {
        let output = Command::new("/sbin/route")
            .args(["-n", "get", "default"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("interface:") {
                let iface = rest.trim();
                if !iface.is_empty() {
                    return Some(iface.to_string());
                }
            }
        }
        None
    };
    let link_of = |iface: &str| -> Option<String> {
        let output = Command::new("/sbin/ifconfig")
            .arg(iface)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut status: Option<String> = None;
        let mut media: Option<String> = None;
        for line in stdout.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("status:") {
                let s = rest.trim();
                if !s.is_empty() {
                    status = Some(s.to_string());
                }
            } else if let Some(rest) = t.strip_prefix("media:") {
                // Prefer negotiated speed in parens when present: "autoselect (1000baseT <full-duplex>)"
                let raw = rest.trim();
                if raw.is_empty() {
                    continue;
                }
                let pretty = if let (Some(open), Some(close)) = (raw.find('('), raw.rfind(')')) {
                    if open < close {
                        let inner = raw[open + 1..close].trim();
                        if !inner.is_empty() {
                            inner.to_string()
                        } else {
                            raw.to_string()
                        }
                    } else {
                        raw.to_string()
                    }
                } else {
                    raw.to_string()
                };
                media = Some(pretty);
            }
        }
        match (status, media) {
            (Some(s), Some(m)) => Some(format!("{s} · {m} ({iface})")),
            (Some(s), None) => Some(format!("{s} ({iface})")),
            (None, Some(m)) => Some(format!("{m} ({iface})")),
            (None, None) => None,
        }
    };
    let from_default = || -> Option<String> {
        let iface = default_iface()?;
        link_of(&iface)
    };
    let from_fallback = || -> Option<String> {
        for iface in ["en0", "en1", "en2"] {
            if let Some(link) = link_of(iface) {
                return Some(link);
            }
        }
        None
    };
    let info = from_default().or_else(from_fallback).unwrap_or_default();
    if info.is_empty() {
        debug3!("primary link status: ifconfig empty");
    } else {
        debug3!("primary link status: '{}'", info);
    }
    info
}

/// Format a battery crate `Time` as compact `Xh Ym` / `Ym` / `Zs`.
fn format_battery_eta_secs(secs: f32) -> String {
    let secs = secs.max(0.0).round() as u64;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        if m > 0 {
            format!("{h}h {m}m")
        } else {
            format!("{h}h")
        }
    } else if m > 0 {
        format!("{m}m")
    } else {
        format!("{s}s")
    }
}

/// Battery time-to-empty / time-to-full from the `battery` crate (short TTL cache).
/// Distinct from Bat % (`/battery` · `battery left`) and cycles (`/cycles`).
pub fn get_battery_time_remaining() -> String {
    static CACHE: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock() {
        if let Some((ref info, ts)) = *guard {
            if ts.elapsed() < Duration::from_secs(5) {
                return info.clone();
            }
        }
    }

    let info = match BatteryManager::new() {
        Ok(manager) => match manager.batteries() {
            Ok(mut batteries) => match batteries.next() {
                Some(Ok(battery)) => {
                    use battery::units::time::second;
                    match battery.state() {
                        State::Full => "full".to_string(),
                        State::Charging => battery
                            .time_to_full()
                            .map(|t| {
                                format!(
                                    "{} until full",
                                    format_battery_eta_secs(t.get::<second>())
                                )
                            })
                            .unwrap_or_default(),
                        State::Discharging => battery
                            .time_to_empty()
                            .map(|t| {
                                format!(
                                    "{} until empty",
                                    format_battery_eta_secs(t.get::<second>())
                                )
                            })
                            .unwrap_or_default(),
                        _ => String::new(),
                    }
                }
                _ => String::new(),
            },
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    if !info.is_empty() {
        debug3!("battery time remaining: '{}'", info);
        if let Ok(mut guard) = cache.lock() {
            *guard = Some((info.clone(), Instant::now()));
        }
    } else {
        debug3!("battery time remaining: no estimate");
    }
    info
}

/// Battery cycle count · health · max capacity from `system_profiler SPPowerDataType` (JSON).
/// Cached (OnceLock): cycle count changes slowly; Discord/operator instant `/cycles` lane.
/// Distinct from Bat % chip (`/battery`) and charging / left / low phrases.
pub fn get_battery_cycles() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let output = Command::new("/usr/sbin/system_profiler")
                .args(["SPPowerDataType", "-json"])
                .stderr(std::process::Stdio::null())
                .output();
            let Ok(output) = output else {
                debug3!("battery cycles: system_profiler spawn failed");
                return String::new();
            };
            if !output.status.success() {
                debug3!("battery cycles: system_profiler non-zero");
                return String::new();
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) else {
                debug3!("battery cycles: JSON parse failed");
                return String::new();
            };
            let Some(entries) = json.get("SPPowerDataType").and_then(|v| v.as_array()) else {
                return String::new();
            };
            for entry in entries {
                let Some(health) = entry.get("sppower_battery_health_info") else {
                    continue;
                };
                let cycles = health
                    .get("sppower_battery_cycle_count")
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n as u64)));
                let cond = health
                    .get("sppower_battery_health")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                let max_cap = health
                    .get("sppower_battery_health_maximum_capacity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                if let Some(n) = cycles {
                    let mut parts = vec![format!("{n} cycles")];
                    if !cond.is_empty() {
                        parts.push(cond.to_string());
                    }
                    if !max_cap.is_empty() {
                        parts.push(format!("{max_cap} max"));
                    }
                    let info = parts.join(" · ");
                    debug3!("battery cycles: '{}'", info);
                    return info;
                }
            }
            debug3!("battery cycles: no health_info / cycle_count");
            String::new()
        })
        .clone()
}

/// AC charger connected · wattage · charging from `system_profiler SPPowerDataType` (JSON).
/// Live (not OnceLock): plug / unplug / adapter swap can change while the process stays up.
/// Distinct from system CPU+GPU `/watts` and Bat % chip (`/battery` · `is it charging`).
pub fn get_ac_charger() -> String {
    let output = Command::new("/usr/sbin/system_profiler")
        .args(["SPPowerDataType", "-json"])
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("ac charger: system_profiler spawn failed");
        return String::new();
    };
    if !output.status.success() {
        debug3!("ac charger: system_profiler non-zero");
        return String::new();
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) else {
        debug3!("ac charger: JSON parse failed");
        return String::new();
    };
    let Some(entries) = json.get("SPPowerDataType").and_then(|v| v.as_array()) else {
        return String::new();
    };
    for entry in entries {
        let has_charger_keys = entry.get("sppower_battery_charger_connected").is_some()
            || entry.get("sppower_ac_charger_watts").is_some()
            || entry.get("sppower_battery_is_charging").is_some();
        if !has_charger_keys {
            continue;
        }
        let connected = entry
            .get("sppower_battery_charger_connected")
            .and_then(|v| match v {
                serde_json::Value::Bool(b) => Some(*b),
                serde_json::Value::String(s) => {
                    let t = s.trim();
                    Some(t.eq_ignore_ascii_case("TRUE") || t.eq_ignore_ascii_case("Yes") || t == "1")
                }
                _ => None,
            })
            .unwrap_or(false);
        let charging = entry
            .get("sppower_battery_is_charging")
            .and_then(|v| match v {
                serde_json::Value::Bool(b) => Some(*b),
                serde_json::Value::String(s) => {
                    let t = s.trim();
                    Some(t.eq_ignore_ascii_case("TRUE") || t.eq_ignore_ascii_case("Yes") || t == "1")
                }
                _ => None,
            })
            .unwrap_or(false);
        let watts = entry
            .get("sppower_ac_charger_watts")
            .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n as u64)).or_else(|| {
                v.as_str()
                    .and_then(|s| s.trim().parse::<u64>().ok())
            }));
        let mut parts: Vec<String> = Vec::new();
        if let Some(w) = watts {
            if w > 0 {
                parts.push(format!("{w} W"));
            }
        }
        if connected {
            parts.push("connected".to_string());
        } else {
            parts.push("disconnected".to_string());
        }
        if charging {
            parts.push("charging".to_string());
        } else if connected {
            parts.push("not charging".to_string());
        }
        let info = parts.join(" · ");
        debug3!("ac charger: '{}'", info);
        return info;
    }
    debug3!("ac charger: no charger keys");
    String::new()
}

/// Connected display names · resolutions from `system_profiler SPDisplaysDataType` (JSON).
/// Live (not OnceLock): plug / unplug / mirror / resolution change while the process stays up.
/// Distinct from GPU usage ring (`/gpu`) and screenshot folder path/size/age lanes.
pub fn get_display_resolution() -> String {
    let output = Command::new("/usr/sbin/system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("display resolution: system_profiler spawn failed");
        return String::new();
    };
    if !output.status.success() {
        debug3!("display resolution: system_profiler non-zero");
        return String::new();
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) else {
        debug3!("display resolution: JSON parse failed");
        return String::new();
    };
    let Some(gpus) = json.get("SPDisplaysDataType").and_then(|v| v.as_array()) else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for gpu in gpus {
        let Some(ndrvs) = gpu.get("spdisplays_ndrvs").and_then(|v| v.as_array()) else {
            continue;
        };
        for disp in ndrvs {
            let online = disp
                .get("spdisplays_online")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .contains("yes");
            if !online {
                // Still accept when the key is missing (older dumps); only skip explicit offline.
                let has_online = disp.get("spdisplays_online").is_some();
                if has_online {
                    continue;
                }
            }
            let name = disp
                .get("_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Display")
                .trim();
            let res = disp
                .get("_spdisplays_resolution")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    disp.get("_spdisplays_pixels")
                        .and_then(|v| v.as_str())
                })
                .unwrap_or("")
                .trim();
            if res.is_empty() {
                continue;
            }
            let main = disp
                .get("spdisplays_main")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .contains("yes");
            let label = if name.is_empty() {
                "Display".to_string()
            } else {
                name.to_string()
            };
            if main {
                parts.push(format!("{label} {res} (main)"));
            } else {
                parts.push(format!("{label} {res}"));
            }
        }
    }
    if parts.is_empty() {
        debug3!("display resolution: no online displays");
        return String::new();
    }
    let info = parts.join(" · ");
    debug3!("display resolution: '{}'", info);
    info
}

/// GPU chipset model · core count · Metal family from `system_profiler SPDisplaysDataType`.
/// Cached (OnceLock): discrete/built-in GPU identity does not change while the process stays up.
/// Distinct from GPU usage ring (`/gpu`) and display resolution (`/display`).
pub fn get_gpu_identity() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let output = Command::new("/usr/sbin/system_profiler")
                .args(["SPDisplaysDataType", "-json"])
                .stderr(std::process::Stdio::null())
                .output();
            let Ok(output) = output else {
                debug3!("gpu identity: system_profiler spawn failed");
                return String::new();
            };
            if !output.status.success() {
                debug3!("gpu identity: system_profiler non-zero");
                return String::new();
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) else {
                debug3!("gpu identity: JSON parse failed");
                return String::new();
            };
            let Some(gpus) = json.get("SPDisplaysDataType").and_then(|v| v.as_array()) else {
                return String::new();
            };
            let mut parts: Vec<String> = Vec::new();
            for gpu in gpus {
                let model = gpu
                    .get("sppci_model")
                    .and_then(|v| v.as_str())
                    .or_else(|| gpu.get("_name").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .trim();
                if model.is_empty() {
                    continue;
                }
                let mut piece = model.to_string();
                let cores = gpu.get("sppci_cores").and_then(|v| {
                    v.as_str()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .or_else(|| v.as_u64().map(|n| n.to_string()))
                });
                if let Some(cores) = cores {
                    piece.push_str(" · ");
                    piece.push_str(&cores);
                    piece.push_str(" cores");
                }
                if let Some(metal_raw) = gpu
                    .get("spdisplays_mtlgpufamilysupport")
                    .and_then(|v| v.as_str())
                {
                    let metal = metal_raw
                        .trim()
                        .strip_prefix("spdisplays_metal")
                        .unwrap_or("")
                        .trim();
                    if !metal.is_empty() {
                        piece.push_str(" · Metal ");
                        piece.push_str(metal);
                    }
                }
                parts.push(piece);
            }
            if parts.is_empty() {
                debug3!("gpu identity: no GPUs");
                return String::new();
            }
            let info = parts.join(" · ");
            debug3!("gpu identity: '{}'", info);
            info
        })
        .clone()
}
/// Parse `scutil --nc list` lines into a short VPN status string.
/// Prefers Connected services; falls back to Disconnected names when none are up.
pub(crate) fn parse_scutil_nc_vpn_list(stdout: &str) -> String {
    let mut connected: Vec<String> = Vec::new();
    let mut disconnected: Vec<String> = Vec::new();
    for line in stdout.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        // Prefer VPN / PPP / IPSec / IKEv2 rows; skip plain Ethernet/Wi-Fi services.
        let looks_vpn = lower.contains("vpn")
            || lower.contains("[ppp")
            || lower.contains("ppp:")
            || lower.contains("ipsec")
            || lower.contains("ikev2")
            || lower.contains("l2tp")
            || lower.contains("wireguard")
            || lower.contains("openvpn")
            || lower.contains("anyconnect");
        if !looks_vpn {
            continue;
        }
        let state = if lower.contains("(connected)") {
            "connected"
        } else if lower.contains("(connecting)") {
            "connecting"
        } else if lower.contains("(disconnecting)") {
            "disconnecting"
        } else if lower.contains("(disconnected)") {
            "disconnected"
        } else {
            continue;
        };
        let name = t
            .find('"')
            .and_then(|start| {
                let rest = &t[start + 1..];
                rest.find('"').map(|end| rest[..end].trim().to_string())
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "VPN".to_string());
        match state {
            "connected" | "connecting" | "disconnecting" => {
                if !connected.iter().any(|n| n == &name) {
                    connected.push(name);
                }
            }
            _ => {
                if !disconnected.iter().any(|n| n == &name) {
                    disconnected.push(name);
                }
            }
        }
    }
    if !connected.is_empty() {
        let names = connected.join(", ");
        return format!("connected · {names}");
    }
    if !disconnected.is_empty() {
        let names = disconnected.join(", ");
        return format!("disconnected · {names}");
    }
    String::new()
}

/// VPN connection status from `scutil --nc list` (live; not OnceLock).
/// Distinct from LAN `/ip`, Wi-Fi `/wifi`, and primary `/link-status`.
/// Used by Discord/operator instant `/vpn` · `/vpn-status` lane.
pub fn get_vpn_status() -> String {
    let output = Command::new("/usr/sbin/scutil")
        .args(["--nc", "list"])
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        debug3!("vpn status: scutil spawn failed");
        return String::new();
    };
    if !output.status.success() {
        debug3!("vpn status: scutil non-zero");
        return String::new();
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let info = parse_scutil_nc_vpn_list(&stdout);
    if info.is_empty() {
        // Fallback: a live utun with an IPv4 often means a tunnel is up (WireGuard etc.).
        if utun_has_ipv4() {
            debug3!("vpn status: utun fallback connected");
            return "connected · utun".to_string();
        }
        debug3!("vpn status: no VPN services");
        return String::new();
    }
    debug3!("vpn status: '{}'", info);
    info
}

fn utun_has_ipv4() -> bool {
    let output = Command::new("/sbin/ifconfig")
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut in_utun = false;
    for line in stdout.lines() {
        if line.starts_with('\t') || line.starts_with(' ') {
            if in_utun {
                let t = line.trim_start();
                if t.starts_with("inet ") {
                    return true;
                }
            }
        } else {
            in_utun = line.starts_with("utun");
        }
    }
    false
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
                    if let Some(hardware) = json.get("SPHardwareDataType").and_then(|v| v.as_array()).and_then(|a| a.first()) {
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
                                    parts.first().and_then(|s| s.parse::<u32>().ok())
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

    // Do NOT tick per-process GPU here: that runs a second `ioreg -l` (~20ms) and
    // stacked with the device-utilization dump when the window is open. Process
    // list / details advance the sampler on their own cadence.

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
                        if let Some(percent) =
                            extract_percentage_after_key(line, "Device Utilization %")
                        {
                            if (0.0..=100.0).contains(&percent) {
                                debug3!(
                                    "GPU usage from ioreg (Device Utilization %): {}%",
                                    percent
                                );
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
                        if let Some(percent) =
                            extract_percentage_after_key(line, "Renderer Utilization %")
                        {
                            if (0.0..=100.0).contains(&percent) {
                                debug3!(
                                    "GPU usage from ioreg (Renderer Utilization %): {}%",
                                    percent
                                );
                                return percent;
                            }
                        }
                    }
                    // Fallback to Tiler Utilization % if others not found
                    if line.contains("Tiler Utilization %") {
                        debug3!("Found 'Tiler Utilization %' in line: {}", line);
                        if let Some(percent) =
                            extract_percentage_after_key(line, "Tiler Utilization %")
                        {
                            if (0.0..=100.0).contains(&percent) {
                                debug3!("GPU usage from ioreg (Tiler Utilization %): {}%", percent);
                                return percent;
                            }
                        }
                    }
                }
                debug3!("ioreg AGXAccelerator: No utilization found in output");
            } else {
                debug3!(
                    "ioreg AGXAccelerator command failed with status: {:?}",
                    output.status
                );
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
                        if (0.0..=100.0).contains(&percent) {
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
        format!("\"{}\"", key), // "Device Utilization %"
        key.to_string(),        // Device Utilization %
    ];

    for key_variant in &key_variants {
        if let Some(key_pos) = line.find(key_variant) {
            // Find the = sign after the key
            let after_key = &line[key_pos + key_variant.len()..];
            if let Some(eq_pos) = after_key.find('=') {
                let after_eq = &after_key[eq_pos + 1..];
                // Extract the number after =
                // Remove any leading/trailing whitespace, quotes, commas
                let trimmed = after_eq
                    .trim()
                    .trim_start_matches('"')
                    .trim_start_matches(' ')
                    .trim_end_matches(',')
                    .trim_end_matches('"')
                    .trim_end_matches('}');

                debug3!("Extracting from '{}' after key '{}'", trimmed, key_variant);

                // Try to parse the first number (before any comma or closing brace)
                // Handle cases like "22," or "22}" or just "22"
                let num_str: String = trimmed
                    .chars()
                    .take_while(|c| c.is_numeric() || *c == '.')
                    .collect();

                if !num_str.is_empty() {
                    if let Ok(num) = num_str.parse::<f32>() {
                        if (0.0..=100.0).contains(&num) {
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
                    if (0.0..=100.0).contains(&num) {
                        debug3!("Successfully extracted {}% (fallback parse)", num);
                        return Some(num);
                    }
                }

                // Also try splitting by whitespace in case there's extra text
                for word in trimmed.split_whitespace() {
                    let cleaned = word.trim_end_matches(',').trim_end_matches('}');
                    if let Ok(num) = cleaned.parse::<f32>() {
                        if (0.0..=100.0).contains(&num) {
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
            if (0.0..=100.0).contains(&num) {
                return Some(num);
            }
        }
        // Also try splitting by whitespace in case there's extra text
        for word in after_eq.split_whitespace() {
            let cleaned = word.trim_end_matches(',');
            if let Ok(num) = cleaned.parse::<f32>() {
                if (0.0..=100.0).contains(&num) {
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
            if (0.0..=100.0).contains(&num) {
                return Some(num);
            }
        }
    }

    None
}

pub fn can_read_temperature() -> bool {
    use std::sync::atomic::Ordering;
    if crate::state::TEMPERATURE_READ_OK.load(Ordering::Relaxed) {
        return true;
    }
    TEMP_CACHE
        .lock()
        .ok()
        .and_then(|cache| {
            cache
                .as_ref()
                .map(|(temp, timestamp)| *temp > 0.0 && timestamp.elapsed() < TEMP_CACHE_MAX_AGE)
        })
        .unwrap_or(false)
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
                            debug3!(
                                "Nominal frequency computed: {:.2} GHz (tbfreq * clockrate.hz)",
                                freq_ghz
                            );
                            return freq_ghz;
                        } else {
                            debug3!(
                                "Computed frequency {:.2} GHz is out of range (0.1-10.0)",
                                freq_ghz
                            );
                        }
                    } else {
                        debug3!(
                            "tb_hz or clock_hz is zero: tb_hz={}, clock_hz={}",
                            tb_hz,
                            hz_value
                        );
                    }
                } else {
                    debug3!("Failed to parse tbfrequency as number");
                }
            } else {
                debug3!(
                    "sysctl commands failed: tb.status={:?}, clock.status={:?}",
                    tb.status,
                    clock.status
                );
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
                                debug3!(
                                    "Nominal frequency from sysctl (fallback): {:.2} GHz",
                                    freq_ghz
                                );
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
                debug3!(
                    "can_read_frequency: true (from FREQ_CACHE with freq={:.2} GHz)",
                    *freq
                );
                return true;
            }
        }
    }

    // OPTIMIZATION Phase 3: Use OnceLock for faster access (no locking required)
    *CAN_READ_FREQUENCY.get_or_init(|| {
        debug3!("can_read_frequency: First time check - trying nominal frequency computation...");
        let nominal = get_nominal_frequency();
        let can_read = nominal > 0.0;

        debug3!(
            "can_read_frequency: Cached result: {} (nominal={:.2} GHz)",
            can_read,
            nominal
        );
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
            // Focused + visible: visible-but-unfocused still woke IOKit (#14).
            let window_visible = crate::state::cpu_window_active_for_metrics();

            // If window is closed, always return cache (even if stale) to save CPU
            if !window_visible {
                debug3!(
                    "Battery info from cache (window closed): {:.1}%, charging={}, has_battery={}",
                    level,
                    charging,
                    *level >= 0.0
                );
                return (*level, *charging, *level >= 0.0);
            }

            // If window is visible, use cache if fresh (less than 1 second old)
            if timestamp.elapsed().as_secs() < 1 {
                debug3!(
                    "Battery info from cache: {:.1}%, charging={}, has_battery={}",
                    level,
                    charging,
                    *level >= 0.0
                );
                return (*level, *charging, *level >= 0.0);
            }
        } else {
            // No cache - require focused + visible before reading (#14).
            let window_visible = crate::state::cpu_window_active_for_metrics();

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
                                let percentage = battery
                                    .state_of_charge()
                                    .get::<battery::units::ratio::percent>();
                                let is_charging = matches!(battery.state(), State::Charging);

                                debug3!(
                                    "Battery read: {:.1}%, charging={}",
                                    percentage,
                                    is_charging
                                );

                                // Update cache
                                if let Ok(mut cache) = crate::state::BATTERY_CACHE.try_lock() {
                                    *cache =
                                        Some((percentage, is_charging, std::time::Instant::now()));
                                }

                                (percentage, is_charging, true)
                            }
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
                }
                Err(e) => {
                    debug3!("Failed to enumerate batteries: {:?}", e);
                    (-1.0, false, false)
                }
            }
        }
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
    // Focused + visible: visible-but-unfocused still woke IOReport (#14).
    let window_visible = crate::state::cpu_window_active_for_metrics();

    // Check cache first
    // IOReport power reading is expensive, so we cache longer
    if let Ok(cache) = crate::state::POWER_CACHE.try_lock() {
        if let Some((cpu_power, gpu_power, timestamp)) = cache.as_ref() {
            // If window is closed, always return cache (even if stale) to save CPU
            if !window_visible {
                debug3!(
                    "Power consumption from cache (window closed): CPU={:.2}W, GPU={:.2}W",
                    cpu_power,
                    gpu_power
                );
                return (*cpu_power, *gpu_power);
            }

            // If window is visible, use cache if fresh (less than 6 seconds old)
            // Background thread updates every 5 seconds
            if timestamp.elapsed().as_secs() < 6 {
                debug3!(
                    "Power consumption from cache: CPU={:.2}W, GPU={:.2}W",
                    cpu_power,
                    gpu_power
                );
                return (*cpu_power, *gpu_power);
            } else {
                // Cache is stale, but return last known values instead of 0.0 to prevent flickering
                // Background thread will update it soon
                debug3!(
                    "Power consumption from stale cache: CPU={:.2}W, GPU={:.2}W (age: {}s)",
                    cpu_power,
                    gpu_power,
                    timestamp.elapsed().as_secs()
                );
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

/// How often to re-read mount free space for the menu-bar SSD %.
/// Disk free space changes slowly; avoid refreshing every metrics tick.
const DISK_REFRESH_INTERVAL: Duration = Duration::from_secs(60);
static LAST_DISK_REFRESH: Mutex<Option<Instant>> = Mutex::new(None);

/// Prefer the macOS Data volume (real user capacity), then `/`, then largest local disk.
/// Avoids `.first()` picking a nearly-full external / Time Machine mount (e.g. x9pro ~95%).
fn pick_system_disk(disks: &Disks) -> Option<&sysinfo::Disk> {
    let list = disks.list();
    if list.is_empty() {
        return None;
    }
    let by_mount = |want: &str| {
        list.iter()
            .find(|d| d.mount_point() == std::path::Path::new(want))
    };
    if let Some(d) = by_mount("/System/Volumes/Data") {
        return Some(d);
    }
    if let Some(d) = by_mount("/") {
        return Some(d);
    }
    list.iter()
        .filter(|d| !d.is_removable() && d.total_space() > 0)
        .max_by_key(|d| d.total_space())
        .or_else(|| list.iter().max_by_key(|d| d.total_space()))
}

/// Refresh the cached disk list, then return `(total, available)` bytes.
/// `None` when the list is locked, empty, or the chosen disk has no size.
fn read_system_disk_bytes(refresh_if_new: bool) -> Option<(u64, u64)> {
    let mut disks = match DISKS.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            debug3!("WARNING: DISKS mutex is locked, using 0% for disk");
            return None;
        }
    };
    let now = Instant::now();
    let need_refresh = match LAST_DISK_REFRESH.lock() {
        Ok(mut last) => {
            let stale = last
                .map(|t| now.duration_since(t) >= DISK_REFRESH_INTERVAL)
                .unwrap_or(true);
            if stale {
                *last = Some(now);
            }
            stale
        }
        Err(_) => true,
    };

    if disks.is_none() {
        debug3!("Creating new Disks instance");
        let mut new_disks = Disks::new();
        if refresh_if_new || need_refresh {
            new_disks.refresh(true);
        }
        *disks = Some(new_disks);
    } else if need_refresh {
        // Re-read free space after cleanup / large deletes (was one-shot forever).
        if let Some(d) = disks.as_mut() {
            d.refresh(true);
            debug3!("Disk list refreshed (periodic)");
        }
    }

    let list = disks.as_ref()?;
    let disk = pick_system_disk(list)?;
    let total = disk.total_space();
    if total == 0 {
        return None;
    }
    let available = disk.available_space();
    debug3!(
        "Disk space: mount={:?} (total: {}, available: {})",
        disk.mount_point(),
        total,
        available
    );
    Some((total, available))
}

/// System disk used / total as percent (0–100). Same basis as menu-bar SSD.
fn get_disk_usage_percent(refresh_if_new: bool) -> f32 {
    match read_system_disk_bytes(refresh_if_new) {
        Some((total, available)) if total > 0 => {
            let disk_usage = ((total.saturating_sub(available)) as f32 / total as f32) * 100.0;
            debug3!("Disk usage: {:.1}%", disk_usage);
            disk_usage
        }
        _ => 0.0,
    }
}

/// System disk `(total, available)` bytes. Same volume as menu-bar SSD %.
pub fn get_system_disk_bytes() -> Option<(u64, u64)> {
    read_system_disk_bytes(false)
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
            let should = last_refresh
                .map(|lr| now.duration_since(lr).as_secs() >= 2)
                .unwrap_or(true);
            if should {
                *last_refresh = Some(now);
                debug3!("Refresh allowed (2 seconds passed)");
            } else {
                debug3!("Refresh skipped (less than 2 seconds since last)");
            }
            should
        }
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
                debug3!(
                    "Refreshing CPU usage and memory (is_new={}, should_refresh={})",
                    is_new_instance,
                    should_refresh
                );
                sys.refresh_cpu_usage();
                sys.refresh_memory();
            }

            let cpu = sys.global_cpu_usage();
            let ram = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;
            debug3!("CPU usage: {}%, RAM usage: {}%", cpu, ram);

            (cpu, ram)
        }
        Err(_) => {
            // Lock held - return zeros immediately, no retry to avoid CPU spinning
            // This will be caught by is_valid() check and update will be skipped
            debug3!("SYSTEM mutex locked, returning zeros (update will be skipped if invalid)");
            (0.0, 0.0)
        }
    };

    let disk_usage = get_disk_usage_percent(should_refresh);

    let gpu_usage = get_gpu_usage();
    debug3!("GPU usage: {}%", gpu_usage);

    let metrics = SystemMetrics {
        cpu: cpu_usage,
        gpu: gpu_usage,
        ram: ram_usage,
        disk: disk_usage,
    };

    debug3!(
        "Returning metrics: CPU={}%, GPU={}%, RAM={}%, DISK={}%",
        metrics.cpu,
        metrics.gpu,
        metrics.ram,
        metrics.disk
    );

    metrics
}

/// Format current system metrics for AI context (Ollama chat, agent router, Discord).
/// Use when the user may ask about CPU, GPU, RAM, disk, temperature, etc., so the model can answer accurately.
pub fn format_metrics_for_ai_context() -> String {
    let m = get_metrics();
    let c = get_cpu_details();
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "CPU: {:.1}%, GPU: {:.1}%, RAM: {:.1}%, Disk: {:.1}%",
        m.cpu, m.gpu, m.ram, m.disk
    ));
    if c.can_read_temperature && c.temperature > 0.0 {
        lines.push(format!("Temperature: {:.1}°C", c.temperature));
    } else if c.can_read_temperature {
        lines.push("Temperature: N/A".to_string());
    }
    if !c.thermal_state.is_empty() {
        lines.push(format!("Thermal pressure: {}", c.thermal_state));
    }
    if c.low_power_mode {
        lines.push("Low Power Mode: on".to_string());
    }
    if c.can_read_frequency {
        lines.push(format!(
            "Frequency: {:.2} GHz (P-core: {:.2}, E-core: {:.2})",
            c.frequency, c.p_core_frequency, c.e_core_frequency
        ));
    }
    lines.push(format!(
        "Load (1/5/15 min): {:.1} / {:.1} / {:.1}",
        c.load_1, c.load_5, c.load_15
    ));
    if !c.chip_info.is_empty() {
        lines.push(format!("Chip: {}", c.chip_info));
    }
    if c.has_battery {
        let charging = if c.is_charging { ", charging" } else { "" };
        lines.push(format!("Battery: {:.0}%{}", c.battery_level, charging));
    }
    if c.can_read_cpu_power || c.can_read_gpu_power {
        let parts: Vec<String> = [
            c.can_read_cpu_power
                .then(|| format!("CPU {:.1} W", c.cpu_power)),
            c.can_read_gpu_power
                .then(|| format!("GPU {:.1} W", c.gpu_power)),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !parts.is_empty() {
            lines.push(format!("Power: {}", parts.join(", ")));
        }
    }
    lines.push(format!("Uptime: {} s", c.uptime_secs));
    if !c.top_processes.is_empty() {
        let top: Vec<String> = c
            .top_processes
            .iter()
            .take(5)
            .map(|p| format!("{} ({:.1}%)", p.name, p.cpu))
            .collect();
        lines.push(format!("Top processes by CPU: {}", top.join(", ")));
    }
    format!("Current system metrics:\n{}", lines.join("\n"))
}

/// Get application version from Cargo.toml
/// This is called by the frontend to always display the correct version
#[tauri::command]
pub fn get_app_version() -> String {
    crate::config::Config::version()
}

/// Homebrew Caskroom roots that may hold a staged `mac-stats` cask.
const HOMEBREW_CASKROOM_MAC_STATS: &[&str] = &[
    "/opt/homebrew/Caskroom/mac-stats",
    "/usr/local/Caskroom/mac-stats",
];

fn homebrew_cask_staged_at(candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|p| std::path::Path::new(p).is_dir())
}

/// True when the Homebrew cask is staged under Caskroom (Apple Silicon or Intel prefix).
/// Used so the update banner only suggests `brew upgrade` for real brew installs (#21).
#[tauri::command]
pub fn is_homebrew_cask_install() -> bool {
    homebrew_cask_staged_at(HOMEBREW_CASKROOM_MAC_STATS)
}

#[cfg(test)]
mod media_sharing_parse_tests {
    use super::format_media_sharing_parts;

    #[test]
    fn formats_off_home_and_photos() {
        assert_eq!(
            format_media_sharing_parts(Some(false), Some(false), false),
            "Off"
        );
    }

    #[test]
    fn formats_home_on_idle() {
        assert_eq!(
            format_media_sharing_parts(Some(true), Some(false), false),
            "On · home"
        );
    }

    #[test]
    fn formats_home_and_photos_active() {
        assert_eq!(
            format_media_sharing_parts(Some(true), Some(true), true),
            "On · home · photos · active"
        );
    }
}

#[cfg(test)]
mod airplay_receiver_parse_tests {
    use super::{
        format_airplay_receiver_parts, parse_airplay_receiver_advertising,
        parse_airplay_receiver_enabled,
    };

    #[test]
    fn parses_enabled_flags() {
        assert_eq!(parse_airplay_receiver_enabled("0\n"), Some(false));
        assert_eq!(parse_airplay_receiver_enabled("1"), Some(true));
        assert_eq!(parse_airplay_receiver_enabled("true"), Some(true));
        assert_eq!(parse_airplay_receiver_enabled("false"), Some(false));
        assert_eq!(parse_airplay_receiver_enabled("nope"), None);
    }

    #[test]
    fn parses_advertising_ints() {
        assert_eq!(parse_airplay_receiver_advertising("2\n"), Some(2));
        assert_eq!(parse_airplay_receiver_advertising("0"), Some(0));
        assert_eq!(parse_airplay_receiver_advertising("x"), None);
    }

    #[test]
    fn formats_off_on_and_listening() {
        assert_eq!(
            format_airplay_receiver_parts(Some(false), Some(2), true),
            "Off"
        );
        assert_eq!(
            format_airplay_receiver_parts(Some(true), Some(2), false),
            "On"
        );
        assert_eq!(
            format_airplay_receiver_parts(Some(true), Some(3), true),
            "On · listening"
        );
        assert_eq!(
            format_airplay_receiver_parts(None, Some(2), true),
            "On · listening"
        );
        assert_eq!(
            format_airplay_receiver_parts(None, Some(0), false),
            "Off"
        );
        assert_eq!(
            format_airplay_receiver_parts(None, Some(2), false),
            "On"
        );
        assert_eq!(format_airplay_receiver_parts(None, None, false), "");
    }
}

#[cfg(test)]
mod firewall_block_all_parse_tests {
    use super::parse_firewall_block_all;

    #[test]
    fn parses_block_all_status() {
        assert_eq!(
            parse_firewall_block_all("Firewall has block all state set to disabled.\n"),
            Some("Disabled")
        );
        assert_eq!(
            parse_firewall_block_all("Firewall has block all state set to enabled."),
            Some("Enabled")
        );
        assert_eq!(parse_firewall_block_all("nope"), None);
    }
}

#[cfg(test)]
mod firewall_listapps_parse_tests {
    use super::{
        firewall_listapps_label, format_firewall_listapps_parts, parse_firewall_listapps,
        FirewallListedApp,
    };

    #[test]
    fn parses_listapps_rows_and_labels() {
        let raw = "Total number of apps = 2 \n\
1 : /Applications/Docker.app/Contents/MacOS/com.docker.backend \n\
             (Allow incoming connections)\n\
2 : /usr/sbin/cupsd \n\
             (Block incoming connections)\n";
        let (total, apps) = parse_firewall_listapps(raw);
        assert_eq!(total, 2);
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].state, "Allow");
        assert_eq!(apps[1].state, "Block");
        assert_eq!(
            firewall_listapps_label(&apps[0].path),
            "Docker.app"
        );
        assert_eq!(firewall_listapps_label(&apps[1].path), "cupsd");
        let body = format_firewall_listapps_parts(total, &apps, 12);
        assert!(body.starts_with("2\n"));
        assert!(body.contains("· Docker.app — Allow"));
        assert!(body.contains("· cupsd — Block"));
    }

    #[test]
    fn truncates_long_listapps() {
        let apps: Vec<FirewallListedApp> = (0..15)
            .map(|i| FirewallListedApp {
                path: format!("/tmp/app{i}"),
                state: "Allow".into(),
            })
            .collect();
        let body = format_firewall_listapps_parts(15, &apps, 3);
        assert!(body.starts_with("15\n"));
        assert!(body.contains("_… +12 more_"));
        assert_eq!(body.lines().filter(|l| l.starts_with('·')).count(), 3);
    }
}

#[cfg(test)]
mod firewall_allow_signed_parse_tests {
    use super::{
        format_firewall_allow_signed_parts, parse_firewall_allow_signed,
        parse_firewall_allow_signed_line,
    };

    #[test]
    fn parses_allow_signed_line() {
        assert_eq!(
            parse_firewall_allow_signed_line(
                "Automatically allow built-in signed software ENABLED.\n"
            ),
            Some("On")
        );
        assert_eq!(
            parse_firewall_allow_signed_line(
                "Automatically allow downloaded signed software DISABLED"
            ),
            Some("Off")
        );
        assert_eq!(parse_firewall_allow_signed_line("nope"), None);
    }

    #[test]
    fn parses_built_in_and_downloaded() {
        let raw = "Automatically allow built-in signed software ENABLED.\n\
Automatically allow downloaded signed software ENABLED.\n";
        assert_eq!(parse_firewall_allow_signed(raw), (Some("On"), Some("On")));
        let mixed = "Automatically allow built-in signed software DISABLED.\n\
Automatically allow downloaded signed software ENABLED.\n";
        assert_eq!(parse_firewall_allow_signed(mixed), (Some("Off"), Some("On")));
        assert_eq!(parse_firewall_allow_signed("nope"), (None, None));
    }

    #[test]
    fn formats_allow_signed_parts() {
        assert_eq!(
            format_firewall_allow_signed_parts(Some("On"), Some("On")),
            "Built-in On · Downloaded On"
        );
        assert_eq!(
            format_firewall_allow_signed_parts(Some("Off"), Some("On")),
            "Built-in Off · Downloaded On"
        );
        assert_eq!(
            format_firewall_allow_signed_parts(Some("On"), None),
            "Built-in On"
        );
        assert_eq!(
            format_firewall_allow_signed_parts(None, Some("Off")),
            "Downloaded Off"
        );
        assert_eq!(format_firewall_allow_signed_parts(None, None), "");
    }
}

#[cfg(test)]
mod firewall_logging_parse_tests {
    use super::{
        format_firewall_logging_parts, parse_firewall_logging_mode, parse_firewall_logging_opt,
    };

    #[test]
    fn parses_logging_mode() {
        assert_eq!(
            parse_firewall_logging_mode("Firewall logging is currently enabled\n"),
            Some("On")
        );
        assert_eq!(
            parse_firewall_logging_mode("Firewall logging is currently disabled"),
            Some("Off")
        );
        assert_eq!(parse_firewall_logging_mode("logging is on"), Some("On"));
        assert_eq!(parse_firewall_logging_mode("logging is off"), Some("Off"));
        assert_eq!(parse_firewall_logging_mode("nope"), None);
    }

    #[test]
    fn parses_logging_opt() {
        assert_eq!(
            parse_firewall_logging_opt("Firewall logging option is currently throttled\n"),
            Some("throttled")
        );
        assert_eq!(parse_firewall_logging_opt("detail"), Some("detail"));
        assert_eq!(parse_firewall_logging_opt("brief"), Some("brief"));
        assert_eq!(parse_firewall_logging_opt("nope"), None);
    }

    #[test]
    fn formats_off_on_and_opt() {
        assert_eq!(format_firewall_logging_parts(Some("Off"), Some("detail")), "Off");
        assert_eq!(
            format_firewall_logging_parts(Some("On"), Some("throttled")),
            "On · throttled"
        );
        assert_eq!(format_firewall_logging_parts(Some("On"), None), "On");
        assert_eq!(format_firewall_logging_parts(None, Some("brief")), "brief");
        assert_eq!(format_firewall_logging_parts(None, None), "");
    }
}

#[cfg(test)]
mod airdrop_parse_tests {
    use super::{
        format_airdrop_parts, parse_airdrop_disabled_flag, parse_airdrop_discoverable_mode,
    };

    #[test]
    fn parses_discoverable_modes() {
        assert_eq!(parse_airdrop_discoverable_mode("Off\n"), Some("Off"));
        assert_eq!(parse_airdrop_discoverable_mode("0"), Some("Off"));
        assert_eq!(parse_airdrop_discoverable_mode("Contacts Only"), Some("Contacts Only"));
        assert_eq!(parse_airdrop_discoverable_mode("1"), Some("Contacts Only"));
        assert_eq!(parse_airdrop_discoverable_mode("Everyone"), Some("Everyone"));
        assert_eq!(parse_airdrop_discoverable_mode("2"), Some("Everyone"));
        assert_eq!(parse_airdrop_discoverable_mode("nope"), None);
    }

    #[test]
    fn parses_disabled_flag() {
        assert_eq!(parse_airdrop_disabled_flag("1\n"), Some(true));
        assert_eq!(parse_airdrop_disabled_flag("0"), Some(false));
        assert_eq!(parse_airdrop_disabled_flag("true"), Some(true));
        assert_eq!(parse_airdrop_disabled_flag("x"), None);
    }

    #[test]
    fn formats_off_contacts_everyone() {
        assert_eq!(format_airdrop_parts(Some("Off"), None), "Off");
        assert_eq!(
            format_airdrop_parts(Some("Contacts Only"), None),
            "Contacts Only"
        );
        assert_eq!(format_airdrop_parts(Some("Everyone"), None), "Everyone");
        assert_eq!(format_airdrop_parts(Some("Everyone"), Some(true)), "Off");
        assert_eq!(format_airdrop_parts(None, Some(true)), "Off");
        assert_eq!(format_airdrop_parts(None, None), "");
        assert_eq!(format_airdrop_parts(None, Some(false)), "");
    }
}

#[cfg(test)]
mod guest_account_parse_tests {
    use super::{
        format_guest_account_parts, parse_guest_account_sysadminctl, parse_guest_enabled_defaults,
    };

    #[test]
    fn parses_guest_enabled_defaults() {
        assert_eq!(parse_guest_enabled_defaults("0\n"), Some(false));
        assert_eq!(parse_guest_enabled_defaults("1"), Some(true));
        assert_eq!(parse_guest_enabled_defaults("true"), Some(true));
        assert_eq!(parse_guest_enabled_defaults("nope"), None);
    }

    #[test]
    fn parses_sysadminctl_status() {
        assert_eq!(
            parse_guest_account_sysadminctl("Guest account disabled.\n"),
            Some("Off")
        );
        assert_eq!(
            parse_guest_account_sysadminctl("Guest account enabled.\n"),
            Some("On")
        );
        assert_eq!(parse_guest_account_sysadminctl("nope"), None);
    }

    #[test]
    fn formats_defaults_over_sysadmin() {
        assert_eq!(format_guest_account_parts(Some(true), Some("Off")), "On");
        assert_eq!(format_guest_account_parts(Some(false), Some("On")), "Off");
        assert_eq!(format_guest_account_parts(None, Some("On")), "On");
        assert_eq!(format_guest_account_parts(None, None), "");
    }
}

#[cfg(test)]
mod autologin_parse_tests {
    use super::{
        format_autologin_parts, parse_autologin_sysadminctl, parse_autologin_user_defaults,
    };

    #[test]
    fn parses_autologin_user_defaults() {
        assert_eq!(
            parse_autologin_user_defaults("raro42\n").as_deref(),
            Some("raro42")
        );
        assert_eq!(parse_autologin_user_defaults(""), None);
        assert_eq!(
            parse_autologin_user_defaults(
                "The domain/default pair of (/Library/Preferences/com.apple.loginwindow, autoLoginUser) does not exist\n"
            ),
            None
        );
    }

    #[test]
    fn parses_sysadminctl_autologin_status() {
        assert_eq!(
            parse_autologin_sysadminctl(
                "Automatic login is disabled because FileVault is enabled.\n"
            )
            .as_deref(),
            Some("Off · FileVault")
        );
        assert_eq!(
            parse_autologin_sysadminctl(
                "Automatic login is disabled by your system administrator.\n"
            )
            .as_deref(),
            Some("Off · admin")
        );
        assert_eq!(
            parse_autologin_sysadminctl("Automatic login is OFF.\n").as_deref(),
            Some("Off")
        );
        assert_eq!(
            parse_autologin_sysadminctl("Automatic login user: raro42\n").as_deref(),
            Some("On · raro42")
        );
        assert_eq!(parse_autologin_sysadminctl("nope"), None);
    }

    #[test]
    fn formats_user_over_sysadmin() {
        assert_eq!(
            format_autologin_parts(Some("raro42"), Some("Off · FileVault")),
            "On · raro42"
        );
        assert_eq!(
            format_autologin_parts(None, Some("Off · FileVault")),
            "Off · FileVault"
        );
        assert_eq!(format_autologin_parts(None, None), "");
    }
}

#[cfg(test)]
mod screen_lock_parse_tests {
    use super::{
        format_screen_lock_delay, format_screen_lock_parts, parse_screen_lock_ask_defaults,
        parse_screen_lock_delay_defaults,
    };

    #[test]
    fn parses_ask_and_delay() {
        assert_eq!(parse_screen_lock_ask_defaults("1\n"), Some(true));
        assert_eq!(parse_screen_lock_ask_defaults("0"), Some(false));
        assert_eq!(parse_screen_lock_ask_defaults("nope"), None);
        assert_eq!(parse_screen_lock_delay_defaults("0\n"), Some(0.0));
        assert_eq!(parse_screen_lock_delay_defaults("300"), Some(300.0));
        assert_eq!(parse_screen_lock_delay_defaults("5.0"), Some(5.0));
        assert_eq!(parse_screen_lock_delay_defaults(""), None);
        assert_eq!(
            parse_screen_lock_delay_defaults(
                "The domain/default pair of (com.apple.screensaver, askForPasswordDelay) does not exist\n"
            ),
            None
        );
    }

    #[test]
    fn formats_delay_and_parts() {
        assert_eq!(format_screen_lock_delay(0.0), "immediate");
        assert_eq!(format_screen_lock_delay(5.0), "5s");
        assert_eq!(format_screen_lock_delay(300.0), "5m");
        assert_eq!(format_screen_lock_delay(3600.0), "1h");
        assert_eq!(format_screen_lock_parts(Some(false), Some(0.0)), "Off");
        assert_eq!(
            format_screen_lock_parts(Some(true), Some(0.0)),
            "On · immediate"
        );
        assert_eq!(
            format_screen_lock_parts(Some(true), Some(300.0)),
            "On · 5m"
        );
        assert_eq!(format_screen_lock_parts(Some(true), None), "On");
        assert_eq!(format_screen_lock_parts(None, Some(0.0)), "");
        assert_eq!(format_screen_lock_parts(None, None), "");
    }
}

#[cfg(test)]
mod smb_guest_parse_tests {
    use super::{
        format_smb_guest_parts, parse_smb_guest_defaults, parse_smb_guest_sysadminctl,
    };

    #[test]
    fn parses_allow_guest_defaults() {
        assert_eq!(parse_smb_guest_defaults("0\n"), Some(false));
        assert_eq!(parse_smb_guest_defaults("1"), Some(true));
        assert_eq!(parse_smb_guest_defaults("true"), Some(true));
        assert_eq!(parse_smb_guest_defaults("nope"), None);
    }

    #[test]
    fn parses_sysadminctl_smb_guest_status() {
        assert_eq!(
            parse_smb_guest_sysadminctl("SMB guest access disabled.\n"),
            Some("Off")
        );
        assert_eq!(
            parse_smb_guest_sysadminctl("SMB guest access enabled.\n"),
            Some("On")
        );
        assert_eq!(parse_smb_guest_sysadminctl("nope"), None);
        assert_eq!(
            parse_smb_guest_sysadminctl("Guest account disabled.\n"),
            None
        );
    }

    #[test]
    fn formats_defaults_over_sysadmin() {
        assert_eq!(format_smb_guest_parts(Some(true), Some("Off")), "On");
        assert_eq!(format_smb_guest_parts(Some(false), Some("On")), "Off");
        assert_eq!(format_smb_guest_parts(None, Some("On")), "On");
        assert_eq!(format_smb_guest_parts(None, None), "");
    }
}

#[cfg(test)]
mod screensaver_idle_parse_tests {
    use super::{
        format_screensaver_idle_parts, format_screensaver_idle_secs, parse_screensaver_idle_defaults,
    };

    #[test]
    fn parses_idle_time_defaults() {
        assert_eq!(parse_screensaver_idle_defaults("0\n"), Some(0.0));
        assert_eq!(parse_screensaver_idle_defaults("120"), Some(120.0));
        assert_eq!(parse_screensaver_idle_defaults("nope"), None);
        assert_eq!(
            parse_screensaver_idle_defaults(
                "The domain/default pair of (com.apple.screensaver, idleTime) does not exist\n"
            ),
            None
        );
    }

    #[test]
    fn formats_never_and_durations() {
        assert_eq!(format_screensaver_idle_secs(0.0), "Never");
        assert_eq!(format_screensaver_idle_secs(30.0), "30s");
        assert_eq!(format_screensaver_idle_secs(120.0), "2m");
        assert_eq!(format_screensaver_idle_secs(7200.0), "2h");
        assert_eq!(format_screensaver_idle_parts(Some(0.0)), "Never");
        assert_eq!(format_screensaver_idle_parts(Some(300.0)), "5m");
        assert_eq!(format_screensaver_idle_parts(None), "");
    }
}

#[cfg(test)]
mod power_nap_parse_tests {
    use super::{
        format_power_nap_parts, parse_pmset_active_powernap, parse_pmset_custom_powernap,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 powernap             1
 womp                 0
AC Power:
 powernap             0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_powernap(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_powernap("nope"), (None, None));
    }

    #[test]
    fn parses_active_powernap() {
        assert_eq!(
            parse_pmset_active_powernap("Currently in use:\n powernap             1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_powernap(" powernap             0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_powernap("womp 1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_power_nap_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_power_nap_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_power_nap_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_power_nap_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(format_power_nap_parts(None, Some(true), None), "On (AC)");
        assert_eq!(format_power_nap_parts(None, None, Some(true)), "On");
        assert_eq!(format_power_nap_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod wake_for_network_parse_tests {
    use super::{
        format_wake_for_network_parts, parse_pmset_active_womp, parse_pmset_custom_womp,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 powernap             1
 womp                 0
AC Power:
 powernap             0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_womp(sample),
            (Some(false), Some(true))
        );
        assert_eq!(parse_pmset_custom_womp("nope"), (None, None));
    }

    #[test]
    fn parses_active_womp() {
        assert_eq!(
            parse_pmset_active_womp("Currently in use:\n womp                 1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_womp(" womp                 0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_womp("powernap 1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_wake_for_network_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_wake_for_network_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_wake_for_network_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_wake_for_network_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(
            format_wake_for_network_parts(None, Some(true), None),
            "On (AC)"
        );
        assert_eq!(
            format_wake_for_network_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(format_wake_for_network_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod proximitywake_parse_tests {
    use super::{
        format_proximitywake_parts, parse_pmset_active_proximitywake,
        parse_pmset_custom_proximitywake,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 proximitywake        0
 womp                 0
AC Power:
 proximitywake        1
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_proximitywake(sample),
            (Some(false), Some(true))
        );
        assert_eq!(parse_pmset_custom_proximitywake("nope"), (None, None));
    }

    #[test]
    fn parses_active_proximitywake() {
        assert_eq!(
            parse_pmset_active_proximitywake(
                "Currently in use:\n proximitywake        1\n"
            ),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_proximitywake(" proximitywake        0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_proximitywake("womp 1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_proximitywake_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_proximitywake_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_proximitywake_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_proximitywake_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(
            format_proximitywake_parts(None, Some(true), None),
            "On (AC)"
        );
        assert_eq!(
            format_proximitywake_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(format_proximitywake_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod ttyskeepawake_parse_tests {
    use super::{
        format_ttyskeepawake_parts, parse_pmset_active_ttyskeepawake,
        parse_pmset_custom_ttyskeepawake,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 ttyskeepawake        1
AC Power:
 ttyskeepawake        0
"#;
        assert_eq!(
            parse_pmset_custom_ttyskeepawake(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_ttyskeepawake("nope"), (None, None));
    }

    #[test]
    fn parses_active_ttyskeepawake() {
        assert_eq!(
            parse_pmset_active_ttyskeepawake(
                "Currently in use:\n ttyskeepawake        1\n"
            ),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_ttyskeepawake(" ttyskeepawake        0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_ttyskeepawake("womp 1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_ttyskeepawake_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_ttyskeepawake_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_ttyskeepawake_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_ttyskeepawake_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(
            format_ttyskeepawake_parts(None, Some(true), None),
            "On (AC)"
        );
        assert_eq!(
            format_ttyskeepawake_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(format_ttyskeepawake_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod tcpkeepalive_parse_tests {
    use super::{
        format_tcpkeepalive_parts, parse_pmset_active_tcpkeepalive,
        parse_pmset_custom_tcpkeepalive,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 tcpkeepalive         1
AC Power:
 tcpkeepalive         0
"#;
        assert_eq!(
            parse_pmset_custom_tcpkeepalive(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_tcpkeepalive("nope"), (None, None));
    }

    #[test]
    fn parses_active_tcpkeepalive() {
        assert_eq!(
            parse_pmset_active_tcpkeepalive(
                "Currently in use:\n tcpkeepalive         1\n"
            ),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_tcpkeepalive(" tcpkeepalive         0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_tcpkeepalive("ttyskeepawake 1\n"), None);
        assert_eq!(parse_pmset_active_tcpkeepalive("womp 1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_tcpkeepalive_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_tcpkeepalive_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_tcpkeepalive_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_tcpkeepalive_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(
            format_tcpkeepalive_parts(None, Some(true), None),
            "On (AC)"
        );
        assert_eq!(
            format_tcpkeepalive_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(format_tcpkeepalive_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod networkoversleep_parse_tests {
    use super::{
        format_networkoversleep_parts, parse_pmset_active_networkoversleep,
        parse_pmset_custom_networkoversleep,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 networkoversleep         1
AC Power:
 networkoversleep         0
"#;
        assert_eq!(
            parse_pmset_custom_networkoversleep(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_networkoversleep("nope"), (None, None));
    }

    #[test]
    fn parses_active_networkoversleep() {
        assert_eq!(
            parse_pmset_active_networkoversleep(
                "Currently in use:\n networkoversleep         1\n"
            ),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_networkoversleep(" networkoversleep         0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_networkoversleep("tcpkeepalive 1\n"), None);
        assert_eq!(parse_pmset_active_networkoversleep("womp 1\n"), None);
        assert_eq!(parse_pmset_active_networkoversleep("sleep 10\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_networkoversleep_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_networkoversleep_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_networkoversleep_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_networkoversleep_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(
            format_networkoversleep_parts(None, Some(true), None),
            "On (AC)"
        );
        assert_eq!(
            format_networkoversleep_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(format_networkoversleep_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod hibernatemode_parse_tests {
    use super::{
        format_hibernatemode_code, format_hibernatemode_parts, parse_pmset_active_hibernatemode,
        parse_pmset_custom_hibernatemode,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 hibernatemode        3
AC Power:
 hibernatemode        3
"#;
        assert_eq!(
            parse_pmset_custom_hibernatemode(sample),
            (Some(3), Some(3))
        );
        let split = r#"
Battery Power:
 hibernatemode        25
AC Power:
 hibernatemode        3
"#;
        assert_eq!(
            parse_pmset_custom_hibernatemode(split),
            (Some(25), Some(3))
        );
        assert_eq!(parse_pmset_custom_hibernatemode("nope"), (None, None));
    }

    #[test]
    fn parses_active_hibernatemode() {
        assert_eq!(
            parse_pmset_active_hibernatemode(
                "Currently in use:\n hibernatemode        3\n"
            ),
            Some(3)
        );
        assert_eq!(
            parse_pmset_active_hibernatemode(" hibernatemode        0\n"),
            Some(0)
        );
        assert_eq!(parse_pmset_active_hibernatemode("sleep 10\n"), None);
        assert_eq!(parse_pmset_active_hibernatemode("networkoversleep 1\n"), None);
    }

    #[test]
    fn formats_codes_matched_split_and_active() {
        assert_eq!(format_hibernatemode_code(0), "0 (Sleep only)");
        assert_eq!(format_hibernatemode_code(3), "3 (Safe Sleep)");
        assert_eq!(format_hibernatemode_code(25), "25 (Hibernate only)");
        assert_eq!(format_hibernatemode_code(7), "7");
        assert_eq!(
            format_hibernatemode_parts(Some(3), Some(3), Some(0)),
            "3 (Safe Sleep)"
        );
        assert_eq!(
            format_hibernatemode_parts(Some(25), Some(3), None),
            "AC 3 (Safe Sleep) · Battery 25 (Hibernate only)"
        );
        assert_eq!(
            format_hibernatemode_parts(Some(0), None, None),
            "0 (Sleep only) (Battery)"
        );
        assert_eq!(
            format_hibernatemode_parts(None, Some(3), None),
            "3 (Safe Sleep) (AC)"
        );
        assert_eq!(
            format_hibernatemode_parts(None, None, Some(25)),
            "25 (Hibernate only)"
        );
        assert_eq!(format_hibernatemode_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod standby_parse_tests {
    use super::{
        format_standby_parts, parse_pmset_active_standby, parse_pmset_custom_standby,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 standby              1
AC Power:
 standby              0
"#;
        assert_eq!(parse_pmset_custom_standby(sample), (Some(true), Some(false)));
        assert_eq!(parse_pmset_custom_standby("nope"), (None, None));
        // Exact key only — delay variants must not match.
        let delay = r#"
Battery Power:
 standbydelay         10800
AC Power:
 standbydelayhigh     86400
"#;
        assert_eq!(parse_pmset_custom_standby(delay), (None, None));
    }

    #[test]
    fn parses_active_standby() {
        assert_eq!(
            parse_pmset_active_standby("Currently in use:\n standby              1\n"),
            Some(true)
        );
        assert_eq!(parse_pmset_active_standby(" standby              0\n"), Some(false));
        assert_eq!(parse_pmset_active_standby("standbydelay 10800\n"), None);
        assert_eq!(parse_pmset_active_standby("hibernatemode 3\n"), None);
        assert_eq!(parse_pmset_active_standby("powernap 1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(format_standby_parts(Some(true), Some(true), Some(false)), "On");
        assert_eq!(format_standby_parts(Some(false), Some(false), Some(true)), "Off");
        assert_eq!(
            format_standby_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(format_standby_parts(Some(false), None, None), "Off (Battery)");
        assert_eq!(format_standby_parts(None, Some(true), None), "On (AC)");
        assert_eq!(format_standby_parts(None, None, Some(true)), "On");
        assert_eq!(format_standby_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod sleep_on_power_button_parse_tests {
    use super::{
        format_sleep_on_power_button_parts, parse_pmset_active_sleep_on_power_button,
        parse_pmset_custom_sleep_on_power_button, parse_sleep_on_power_button_flag_line,
    };

    #[test]
    fn parses_flag_line() {
        assert_eq!(
            parse_sleep_on_power_button_flag_line("Sleep On Power Button 1"),
            Some(true)
        );
        assert_eq!(
            parse_sleep_on_power_button_flag_line(" sleep on power button  0 "),
            Some(false)
        );
        assert_eq!(parse_sleep_on_power_button_flag_line("standby 1"), None);
        assert_eq!(parse_sleep_on_power_button_flag_line("sleep 10"), None);
    }

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 standby              1
AC Power:
 Sleep On Power Button 0
 standby              0
"#;
        assert_eq!(
            parse_pmset_custom_sleep_on_power_button(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_sleep_on_power_button("nope"), (None, None));
        let standby_only = r#"
Battery Power:
 standby              1
AC Power:
 standby              0
"#;
        assert_eq!(
            parse_pmset_custom_sleep_on_power_button(standby_only),
            (None, None)
        );
    }

    #[test]
    fn parses_active() {
        assert_eq!(
            parse_pmset_active_sleep_on_power_button(
                "Currently in use:\n Sleep On Power Button 1\n"
            ),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_sleep_on_power_button(" Sleep On Power Button 0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_sleep_on_power_button("standby 1\n"), None);
        assert_eq!(parse_pmset_active_sleep_on_power_button("sleep 10\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_sleep_on_power_button_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_sleep_on_power_button_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_sleep_on_power_button_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_sleep_on_power_button_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(
            format_sleep_on_power_button_parts(None, Some(true), None),
            "On (AC)"
        );
        assert_eq!(
            format_sleep_on_power_button_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(
            format_sleep_on_power_button_parts(None, None, None),
            ""
        );
    }
}

#[cfg(test)]
mod lessbright_parse_tests {
    use super::{
        format_lessbright_parts, parse_pmset_active_lessbright, parse_pmset_custom_lessbright,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 lessbright           1
 standby              1
AC Power:
 standby              1
 lessbright           0
"#;
        assert_eq!(
            parse_pmset_custom_lessbright(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_lessbright("nope"), (None, None));
        // Battery-only (common on Apple Silicon).
        let bat_only = r#"
Battery Power:
 lessbright           1
AC Power:
 standby              1
"#;
        assert_eq!(
            parse_pmset_custom_lessbright(bat_only),
            (Some(true), None)
        );
        let other = r#"
Battery Power:
 standby              1
AC Power:
 displaysleep         5
"#;
        assert_eq!(parse_pmset_custom_lessbright(other), (None, None));
    }

    #[test]
    fn parses_active_lessbright() {
        assert_eq!(
            parse_pmset_active_lessbright("Currently in use:\n lessbright           1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_lessbright(" lessbright           0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_lessbright(" standby              1\n"), None);
        assert_eq!(
            parse_pmset_active_lessbright(" Sleep On Power Button 1\n"),
            None
        );
        assert_eq!(parse_pmset_active_lessbright(" displaysleep         5\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_lessbright_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_lessbright_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_lessbright_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_lessbright_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_lessbright_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_lessbright_parts(None, None, Some(true)), "On");
        assert_eq!(format_lessbright_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod powermode_parse_tests {
    use super::{
        format_powermode_code, format_powermode_parts, parse_pmset_active_powermode,
        parse_pmset_custom_powermode,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 lessbright           1
 powermode            1
AC Power:
 Sleep On Power Button 0
 powermode            0
"#;
        assert_eq!(
            parse_pmset_custom_powermode(sample),
            (Some(1), Some(0))
        );
        assert_eq!(parse_pmset_custom_powermode("nope"), (None, None));
    }

    #[test]
    fn ignores_lowpowermode_and_powernap() {
        let other = r#"
Battery Power:
 lowpowermode         1
 powernap             1
AC Power:
 lowpowermode         0
"#;
        assert_eq!(parse_pmset_custom_powermode(other), (None, None));
    }

    #[test]
    fn parses_active_powermode() {
        assert_eq!(
            parse_pmset_active_powermode("Currently in use:\n powermode            0\n"),
            Some(0)
        );
        assert_eq!(
            parse_pmset_active_powermode(" powermode            2\n"),
            Some(2)
        );
        assert_eq!(parse_pmset_active_powermode(" lowpowermode 1\n"), None);
        assert_eq!(parse_pmset_active_powermode(" powernap 1\n"), None);
        assert_eq!(parse_pmset_active_powermode(" lessbright 1\n"), None);
    }

    #[test]
    fn formats_codes_and_parts() {
        assert_eq!(format_powermode_code(0), "0 (Automatic)");
        assert_eq!(format_powermode_code(1), "1 (Low Power)");
        assert_eq!(format_powermode_code(2), "2 (High Power)");
        assert_eq!(format_powermode_code(9), "9");
        assert_eq!(
            format_powermode_parts(Some(1), Some(1), Some(0)),
            "1 (Low Power)"
        );
        assert_eq!(
            format_powermode_parts(Some(1), Some(0), None),
            "AC 0 (Automatic) · Battery 1 (Low Power)"
        );
        assert_eq!(
            format_powermode_parts(Some(2), None, None),
            "2 (High Power) (Battery)"
        );
        assert_eq!(
            format_powermode_parts(None, Some(0), None),
            "0 (Automatic) (AC)"
        );
        assert_eq!(
            format_powermode_parts(None, None, Some(2)),
            "2 (High Power)"
        );
        assert_eq!(format_powermode_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod halfdim_parse_tests {
    use super::{
        format_halfdim_parts, parse_pmset_active_halfdim, parse_pmset_custom_halfdim,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 halfdim              1
 lessbright           1
AC Power:
 halfdim              0
 standby              1
"#;
        assert_eq!(
            parse_pmset_custom_halfdim(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_halfdim("nope"), (None, None));
        let other = r#"
Battery Power:
 lessbright           1
AC Power:
 displaysleep         5
"#;
        assert_eq!(parse_pmset_custom_halfdim(other), (None, None));
    }

    #[test]
    fn parses_active_halfdim() {
        assert_eq!(
            parse_pmset_active_halfdim("Currently in use:\n halfdim              1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_halfdim(" halfdim              0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_halfdim(" lessbright           1\n"), None);
        assert_eq!(parse_pmset_active_halfdim(" displaysleep         5\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_halfdim_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_halfdim_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_halfdim_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_halfdim_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_halfdim_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_halfdim_parts(None, None, Some(true)), "On");
        assert_eq!(format_halfdim_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod hibernatefile_parse_tests {
    use super::{
        format_hibernatefile_parts, parse_pmset_active_hibernatefile,
        parse_pmset_custom_hibernatefile,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 hibernatefile        /var/vm/sleepimage
 hibernatemode        3
AC Power:
 hibernatefile        /var/vm/sleepimage
 standby              1
"#;
        assert_eq!(
            parse_pmset_custom_hibernatefile(sample),
            (
                Some("/var/vm/sleepimage".to_string()),
                Some("/var/vm/sleepimage".to_string())
            )
        );
        assert_eq!(parse_pmset_custom_hibernatefile("nope"), (None, None));
        let split = r#"
Battery Power:
 hibernatefile        /private/var/vm/sleepimage
AC Power:
 hibernatefile        /Volumes/Other/sleepimage
"#;
        assert_eq!(
            parse_pmset_custom_hibernatefile(split),
            (
                Some("/private/var/vm/sleepimage".to_string()),
                Some("/Volumes/Other/sleepimage".to_string())
            )
        );
        let other = r#"
Battery Power:
 hibernatemode        3
AC Power:
 standby              1
"#;
        assert_eq!(parse_pmset_custom_hibernatefile(other), (None, None));
    }

    #[test]
    fn parses_active_hibernatefile() {
        assert_eq!(
            parse_pmset_active_hibernatefile(
                "Currently in use:\n hibernatefile        /var/vm/sleepimage\n"
            ),
            Some("/var/vm/sleepimage".to_string())
        );
        assert_eq!(
            parse_pmset_active_hibernatefile(" hibernatemode        3\n"),
            None
        );
        assert_eq!(parse_pmset_active_hibernatefile(" halfdim              1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_hibernatefile_parts(
                Some("/var/vm/sleepimage".into()),
                Some("/var/vm/sleepimage".into()),
                Some("/tmp/x".into())
            ),
            "/var/vm/sleepimage"
        );
        assert_eq!(
            format_hibernatefile_parts(
                Some("/a".into()),
                Some("/b".into()),
                None
            ),
            "AC /b · Battery /a"
        );
        assert_eq!(
            format_hibernatefile_parts(Some("/a".into()), None, None),
            "/a (Battery)"
        );
        assert_eq!(
            format_hibernatefile_parts(None, Some("/b".into()), None),
            "/b (AC)"
        );
        assert_eq!(
            format_hibernatefile_parts(None, None, Some("/c".into())),
            "/c"
        );
        assert_eq!(format_hibernatefile_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod standbydelay_parse_tests {
    use super::{
        format_standbydelay_parts, format_standbydelay_secs, parse_pmset_active_standbydelay,
        parse_pmset_custom_standbydelay,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 standbydelay         10800
 standby              1
AC Power:
 standbydelay         86400
 standbydelayhigh     86400
"#;
        assert_eq!(
            parse_pmset_custom_standbydelay(sample),
            (Some(10800), Some(86400))
        );
        assert_eq!(parse_pmset_custom_standbydelay("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 standbydelayhigh     86400
"#;
        assert_eq!(parse_pmset_custom_standbydelay(other), (None, None));
    }

    #[test]
    fn parses_active_standbydelay() {
        assert_eq!(
            parse_pmset_active_standbydelay("Currently in use:\n standbydelay         10800\n"),
            Some(10800)
        );
        assert_eq!(parse_pmset_active_standbydelay(" standby              1\n"), None);
        assert_eq!(
            parse_pmset_active_standbydelay(" standbydelayhigh     86400\n"),
            None
        );
    }

    #[test]
    fn formats_secs_and_parts() {
        assert_eq!(format_standbydelay_secs(0), "Never");
        assert_eq!(format_standbydelay_secs(45), "45s");
        assert_eq!(format_standbydelay_secs(90), "1m30s");
        assert_eq!(format_standbydelay_secs(120), "2m");
        assert_eq!(format_standbydelay_secs(3600), "1h");
        assert_eq!(format_standbydelay_secs(10800), "3h");
        assert_eq!(
            format_standbydelay_parts(Some(10800), Some(10800), Some(0)),
            "3h"
        );
        assert_eq!(
            format_standbydelay_parts(Some(10800), Some(86400), None),
            "AC 24h · Battery 3h"
        );
        assert_eq!(
            format_standbydelay_parts(Some(10800), None, None),
            "3h (Battery)"
        );
        assert_eq!(
            format_standbydelay_parts(None, Some(0), None),
            "Never (AC)"
        );
        assert_eq!(format_standbydelay_parts(None, None, Some(120)), "2m");
        assert_eq!(format_standbydelay_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod autopoweroff_parse_tests {
    use super::{
        format_autopoweroff_parts, parse_pmset_active_autopoweroff, parse_pmset_custom_autopoweroff,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 autopoweroff         1
 standby              1
AC Power:
 autopoweroff         0
 autopoweroffdelay    86400
"#;
        assert_eq!(
            parse_pmset_custom_autopoweroff(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_autopoweroff("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 autopoweroffdelay    86400
"#;
        assert_eq!(parse_pmset_custom_autopoweroff(other), (None, None));
    }

    #[test]
    fn parses_active_autopoweroff() {
        assert_eq!(
            parse_pmset_active_autopoweroff("Currently in use:
 autopoweroff         1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_autopoweroff(" autopoweroff         0
"),
            Some(false)
        );
        assert_eq!(
            parse_pmset_active_autopoweroff(" autopoweroffdelay    86400
"),
            None
        );
        assert_eq!(parse_pmset_active_autopoweroff(" standby              1\n"), None);
    }

    #[test]
    fn formats_parts() {
        assert_eq!(
            format_autopoweroff_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_autopoweroff_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_autopoweroff_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_autopoweroff_parts(Some(false), None, None),
            "Off (Battery)"
        );
        assert_eq!(
            format_autopoweroff_parts(None, Some(true), None),
            "On (AC)"
        );
        assert_eq!(format_autopoweroff_parts(None, None, Some(true)), "On");
        assert_eq!(format_autopoweroff_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod autopoweroffdelay_parse_tests {
    use super::{
        format_autopoweroffdelay_parts, parse_pmset_active_autopoweroffdelay,
        parse_pmset_custom_autopoweroffdelay,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 autopoweroffdelay    10800
 autopoweroff         1
AC Power:
 autopoweroffdelay    86400
 standbydelay         10800
"#;
        assert_eq!(
            parse_pmset_custom_autopoweroffdelay(sample),
            (Some(10800), Some(86400))
        );
        assert_eq!(parse_pmset_custom_autopoweroffdelay("nope"), (None, None));
        let other = r#"
Battery Power:
 autopoweroff         1
AC Power:
 standbydelay         10800
"#;
        assert_eq!(parse_pmset_custom_autopoweroffdelay(other), (None, None));
    }

    #[test]
    fn parses_active_autopoweroffdelay() {
        assert_eq!(
            parse_pmset_active_autopoweroffdelay(
                "Currently in use:\n autopoweroffdelay    10800\n"
            ),
            Some(10800)
        );
        assert_eq!(
            parse_pmset_active_autopoweroffdelay(" autopoweroff         1\n"),
            None
        );
        assert_eq!(
            parse_pmset_active_autopoweroffdelay(" standbydelay         10800\n"),
            None
        );
    }

    #[test]
    fn formats_parts() {
        assert_eq!(
            format_autopoweroffdelay_parts(Some(10800), Some(10800), Some(0)),
            "3h"
        );
        assert_eq!(
            format_autopoweroffdelay_parts(Some(10800), Some(86400), None),
            "AC 24h · Battery 3h"
        );
        assert_eq!(
            format_autopoweroffdelay_parts(Some(10800), None, None),
            "3h (Battery)"
        );
        assert_eq!(
            format_autopoweroffdelay_parts(None, Some(0), None),
            "Never (AC)"
        );
        assert_eq!(format_autopoweroffdelay_parts(None, None, Some(120)), "2m");
        assert_eq!(format_autopoweroffdelay_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod standbydelayhigh_parse_tests {
    use super::{
        format_standbydelayhigh_parts, parse_pmset_active_standbydelayhigh,
        parse_pmset_custom_standbydelayhigh,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 standbydelayhigh     86400
 standbydelay         10800
AC Power:
 standbydelayhigh     86400
 standbydelaylow      10800
"#;
        assert_eq!(
            parse_pmset_custom_standbydelayhigh(sample),
            (Some(86400), Some(86400))
        );
        assert_eq!(parse_pmset_custom_standbydelayhigh("nope"), (None, None));
        let other = r#"
Battery Power:
 standbydelay         10800
AC Power:
 standbydelaylow      10800
"#;
        assert_eq!(parse_pmset_custom_standbydelayhigh(other), (None, None));
    }

    #[test]
    fn parses_active_standbydelayhigh() {
        assert_eq!(
            parse_pmset_active_standbydelayhigh(
                "Currently in use:\n standbydelayhigh     86400\n"
            ),
            Some(86400)
        );
        assert_eq!(
            parse_pmset_active_standbydelayhigh(" standbydelay         10800\n"),
            None
        );
        assert_eq!(
            parse_pmset_active_standbydelayhigh(" standbydelaylow      10800\n"),
            None
        );
    }

    #[test]
    fn formats_parts() {
        assert_eq!(
            format_standbydelayhigh_parts(Some(86400), Some(86400), Some(0)),
            "24h"
        );
        assert_eq!(
            format_standbydelayhigh_parts(Some(10800), Some(86400), None),
            "AC 24h · Battery 3h"
        );
        assert_eq!(
            format_standbydelayhigh_parts(Some(86400), None, None),
            "24h (Battery)"
        );
        assert_eq!(
            format_standbydelayhigh_parts(None, Some(0), None),
            "Never (AC)"
        );
        assert_eq!(format_standbydelayhigh_parts(None, None, Some(120)), "2m");
        assert_eq!(format_standbydelayhigh_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod standbydelaylow_parse_tests {
    use super::{
        format_standbydelaylow_parts, parse_pmset_active_standbydelaylow,
        parse_pmset_custom_standbydelaylow,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 standbydelaylow      10800
 standbydelay         10800
AC Power:
 standbydelaylow      3600
 standbydelayhigh     86400
"#;
        assert_eq!(
            parse_pmset_custom_standbydelaylow(sample),
            (Some(10800), Some(3600))
        );
        assert_eq!(parse_pmset_custom_standbydelaylow("nope"), (None, None));
        let other = r#"
Battery Power:
 standbydelay         10800
AC Power:
 standbydelayhigh     86400
"#;
        assert_eq!(parse_pmset_custom_standbydelaylow(other), (None, None));
    }

    #[test]
    fn parses_active_standbydelaylow() {
        assert_eq!(
            parse_pmset_active_standbydelaylow(
                "Currently in use:\n standbydelaylow      10800\n"
            ),
            Some(10800)
        );
        assert_eq!(
            parse_pmset_active_standbydelaylow(" standbydelay         10800\n"),
            None
        );
        assert_eq!(
            parse_pmset_active_standbydelaylow(" standbydelayhigh     86400\n"),
            None
        );
    }

    #[test]
    fn formats_parts() {
        assert_eq!(
            format_standbydelaylow_parts(Some(10800), Some(10800), Some(0)),
            "3h"
        );
        assert_eq!(
            format_standbydelaylow_parts(Some(10800), Some(3600), None),
            "AC 1h · Battery 3h"
        );
        assert_eq!(
            format_standbydelaylow_parts(Some(10800), None, None),
            "3h (Battery)"
        );
        assert_eq!(
            format_standbydelaylow_parts(None, Some(0), None),
            "Never (AC)"
        );
        assert_eq!(format_standbydelaylow_parts(None, None, Some(120)), "2m");
        assert_eq!(format_standbydelaylow_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod highstandbythreshold_parse_tests {
    use super::{
        format_highstandbythreshold_parts, parse_pmset_active_highstandbythreshold,
        parse_pmset_custom_highstandbythreshold,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 highstandbythreshold 50
 standbydelaylow      10800
AC Power:
 highstandbythreshold 50
 standbydelayhigh     86400
"#;
        assert_eq!(
            parse_pmset_custom_highstandbythreshold(sample),
            (Some(50), Some(50))
        );
        assert_eq!(parse_pmset_custom_highstandbythreshold("nope"), (None, None));
        let other = r#"
Battery Power:
 standbydelaylow      10800
AC Power:
 standbydelayhigh     86400
"#;
        assert_eq!(parse_pmset_custom_highstandbythreshold(other), (None, None));
    }

    #[test]
    fn parses_active_highstandbythreshold() {
        assert_eq!(
            parse_pmset_active_highstandbythreshold(
                "Currently in use:\n highstandbythreshold 50\n"
            ),
            Some(50)
        );
        assert_eq!(
            parse_pmset_active_highstandbythreshold(" standbydelaylow      10800\n"),
            None
        );
        assert_eq!(
            parse_pmset_active_highstandbythreshold(" standbydelayhigh     86400\n"),
            None
        );
    }

    #[test]
    fn formats_parts() {
        assert_eq!(
            format_highstandbythreshold_parts(Some(50), Some(50), Some(40)),
            "50%"
        );
        assert_eq!(
            format_highstandbythreshold_parts(Some(40), Some(50), None),
            "AC 50% · Battery 40%"
        );
        assert_eq!(
            format_highstandbythreshold_parts(Some(50), None, None),
            "50% (Battery)"
        );
        assert_eq!(
            format_highstandbythreshold_parts(None, Some(50), None),
            "50% (AC)"
        );
        assert_eq!(format_highstandbythreshold_parts(None, None, Some(50)), "50%");
        assert_eq!(format_highstandbythreshold_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod destroyfvkeyonstandby_parse_tests {
    use super::{
        format_destroyfvkeyonstandby_parts, parse_pmset_active_destroyfvkeyonstandby,
        parse_pmset_custom_destroyfvkeyonstandby,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 destroyfvkeyonstandby 1
 standby                 1
AC Power:
 destroyfvkeyonstandby 0
 standby                 1
"#;
        assert_eq!(
            parse_pmset_custom_destroyfvkeyonstandby(sample),
            (Some(true), Some(false))
        );
        assert_eq!(
            parse_pmset_custom_destroyfvkeyonstandby("nope"),
            (None, None)
        );
        let other = r#"
Battery Power:
 standby                 1
AC Power:
 standby                 1
"#;
        assert_eq!(
            parse_pmset_custom_destroyfvkeyonstandby(other),
            (None, None)
        );
    }

    #[test]
    fn parses_active_destroyfvkeyonstandby() {
        assert_eq!(
            parse_pmset_active_destroyfvkeyonstandby(
                "Currently in use:\n destroyfvkeyonstandby 1\n"
            ),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_destroyfvkeyonstandby(" destroyfvkeyonstandby 0\n"),
            Some(false)
        );
        assert_eq!(
            parse_pmset_active_destroyfvkeyonstandby(" standby                 1\n"),
            None
        );
        assert_eq!(
            parse_pmset_active_destroyfvkeyonstandby(" highstandbythreshold 50\n"),
            None
        );
    }

    #[test]
    fn formats_parts() {
        assert_eq!(
            format_destroyfvkeyonstandby_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_destroyfvkeyonstandby_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_destroyfvkeyonstandby_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_destroyfvkeyonstandby_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_destroyfvkeyonstandby_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(
            format_destroyfvkeyonstandby_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(
            format_destroyfvkeyonstandby_parts(None, None, None),
            ""
        );
    }
}

#[cfg(test)]
mod acwake_parse_tests {
    use super::{
        format_acwake_parts, parse_pmset_active_acwake, parse_pmset_custom_acwake,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 acwake               1
 standby              1
AC Power:
 acwake               0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_acwake(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_acwake("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 womp                 1
"#;
        assert_eq!(parse_pmset_custom_acwake(other), (None, None));
    }

    #[test]
    fn parses_active_acwake() {
        assert_eq!(
            parse_pmset_active_acwake("Currently in use:\n acwake               1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_acwake(" acwake               0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_acwake(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_acwake(" lidwake              1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_acwake_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_acwake_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_acwake_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_acwake_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_acwake_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_acwake_parts(None, None, Some(true)), "On");
        assert_eq!(format_acwake_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod lidwake_parse_tests {
    use super::{
        format_lidwake_parts, parse_pmset_active_lidwake, parse_pmset_custom_lidwake,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 lidwake              1
 standby              1
AC Power:
 lidwake              0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_lidwake(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_lidwake("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 acwake               1
"#;
        assert_eq!(parse_pmset_custom_lidwake(other), (None, None));
    }

    #[test]
    fn parses_active_lidwake() {
        assert_eq!(
            parse_pmset_active_lidwake("Currently in use:\n lidwake              1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_lidwake(" lidwake              0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_lidwake(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_lidwake(" acwake               1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_lidwake_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_lidwake_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_lidwake_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_lidwake_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_lidwake_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_lidwake_parts(None, None, Some(true)), "On");
        assert_eq!(format_lidwake_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod autorestart_parse_tests {
    use super::{
        format_autorestart_parts, parse_pmset_active_autorestart, parse_pmset_custom_autorestart,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 autorestart          1
 standby              1
AC Power:
 autorestart          0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_autorestart(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_autorestart("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 autopoweroff         1
"#;
        assert_eq!(parse_pmset_custom_autorestart(other), (None, None));
    }

    #[test]
    fn parses_active_autorestart() {
        assert_eq!(
            parse_pmset_active_autorestart("Currently in use:\n autorestart          1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_autorestart(" autorestart          0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_autorestart(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_autorestart(" autopoweroff         1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_autorestart_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_autorestart_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_autorestart_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_autorestart_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_autorestart_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_autorestart_parts(None, None, Some(true)), "On");
        assert_eq!(format_autorestart_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod gpuswitch_parse_tests {
    use super::{
        format_gpuswitch_code, format_gpuswitch_parts, parse_pmset_active_gpuswitch,
        parse_pmset_custom_gpuswitch,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 gpuswitch            2
 standby              1
AC Power:
 gpuswitch            0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_gpuswitch(sample),
            (Some(2), Some(0))
        );
        assert_eq!(parse_pmset_custom_gpuswitch("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 autorestart          1
"#;
        assert_eq!(parse_pmset_custom_gpuswitch(other), (None, None));
    }

    #[test]
    fn parses_active_gpuswitch() {
        assert_eq!(
            parse_pmset_active_gpuswitch("Currently in use:\n gpuswitch            2\n"),
            Some(2)
        );
        assert_eq!(
            parse_pmset_active_gpuswitch(" gpuswitch            1\n"),
            Some(1)
        );
        assert_eq!(parse_pmset_active_gpuswitch(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_gpuswitch(" autorestart          1\n"), None);
    }

    #[test]
    fn formats_codes_matched_split_and_active() {
        assert_eq!(format_gpuswitch_code(0), "0 (Integrated)");
        assert_eq!(format_gpuswitch_code(1), "1 (Discrete)");
        assert_eq!(format_gpuswitch_code(2), "2 (Auto)");
        assert_eq!(format_gpuswitch_code(9), "9");
        assert_eq!(
            format_gpuswitch_parts(Some(2), Some(2), Some(0)),
            "2 (Auto)"
        );
        assert_eq!(
            format_gpuswitch_parts(Some(2), Some(0), None),
            "AC 0 (Integrated) · Battery 2 (Auto)"
        );
        assert_eq!(
            format_gpuswitch_parts(Some(1), None, None),
            "1 (Discrete) (Battery)"
        );
        assert_eq!(
            format_gpuswitch_parts(None, Some(0), None),
            "0 (Integrated) (AC)"
        );
        assert_eq!(
            format_gpuswitch_parts(None, None, Some(2)),
            "2 (Auto)"
        );
        assert_eq!(format_gpuswitch_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod darkwakes_parse_tests {
    use super::{
        format_darkwakes_parts, parse_pmset_active_darkwakes, parse_pmset_custom_darkwakes,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 darkwakes            1
 standby              1
AC Power:
 darkwakes            0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_darkwakes(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_darkwakes("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 gpuswitch            2
"#;
        assert_eq!(parse_pmset_custom_darkwakes(other), (None, None));
    }

    #[test]
    fn parses_active_darkwakes() {
        assert_eq!(
            parse_pmset_active_darkwakes("Currently in use:\n darkwakes            1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_darkwakes(" darkwakes            0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_darkwakes(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_darkwakes(" gpuswitch            2\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_darkwakes_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_darkwakes_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_darkwakes_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_darkwakes_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_darkwakes_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_darkwakes_parts(None, None, Some(true)), "On");
        assert_eq!(format_darkwakes_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod ttysclobber_parse_tests {
    use super::{
        format_ttysclobber_parts, parse_pmset_active_ttysclobber, parse_pmset_custom_ttysclobber,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 ttysclobber            1
 standby              1
AC Power:
 ttysclobber            0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_ttysclobber(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_ttysclobber("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 ttyskeepawake       1
"#;
        assert_eq!(parse_pmset_custom_ttysclobber(other), (None, None));
    }

    #[test]
    fn parses_active_ttysclobber() {
        assert_eq!(
            parse_pmset_active_ttysclobber("Currently in use:\n ttysclobber            1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_ttysclobber(" ttysclobber            0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_ttysclobber(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_ttysclobber(" ttyskeepawake       1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_ttysclobber_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_ttysclobber_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_ttysclobber_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_ttysclobber_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_ttysclobber_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_ttysclobber_parts(None, None, Some(true)), "On");
        assert_eq!(format_ttysclobber_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod spotlight_parse_tests {
    use super::parse_mdutil_spotlight_status;

    #[test]
    fn parses_indexing_enabled() {
        let sample = "/:\n\tIndexing enabled. \n";
        assert_eq!(
            parse_mdutil_spotlight_status(sample).as_deref(),
            Some("Enabled")
        );
    }

    #[test]
    fn parses_indexing_disabled() {
        assert_eq!(
            parse_mdutil_spotlight_status("/:\n\tIndexing disabled.\n").as_deref(),
            Some("Disabled")
        );
        assert_eq!(
            parse_mdutil_spotlight_status("Indexing and searching disabled.\n").as_deref(),
            Some("Disabled")
        );
    }

    #[test]
    fn prefers_enabled_when_mixed() {
        let sample = "/:\n\tIndexing enabled.\n/System/Volumes/Data:\n\tIndexing disabled.\n";
        assert_eq!(
            parse_mdutil_spotlight_status(sample).as_deref(),
            Some("Enabled")
        );
    }

    #[test]
    fn empty_on_noise() {
        assert_eq!(parse_mdutil_spotlight_status("nope"), None);
        assert_eq!(parse_mdutil_spotlight_status("/:\n"), None);
    }
}

#[cfg(test)]
mod norestoresleep_parse_tests {
    use super::{
        format_norestoresleep_parts, parse_pmset_active_norestoresleep, parse_pmset_custom_norestoresleep,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 norestoresleep            1
 standby              1
AC Power:
 norestoresleep            0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_norestoresleep(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_norestoresleep("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 ttysclobber            1
"#;
        assert_eq!(parse_pmset_custom_norestoresleep(other), (None, None));
    }

    #[test]
    fn parses_active_norestoresleep() {
        assert_eq!(
            parse_pmset_active_norestoresleep("Currently in use:\n norestoresleep            1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_norestoresleep(" norestoresleep            0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_norestoresleep(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_norestoresleep(" ttysclobber            1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_norestoresleep_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_norestoresleep_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_norestoresleep_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_norestoresleep_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_norestoresleep_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_norestoresleep_parts(None, None, Some(true)), "On");
        assert_eq!(format_norestoresleep_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod reducebrightness_parse_tests {
    use super::{
        format_reducebrightness_parts, parse_pmset_active_reducebrightness,
        parse_pmset_custom_reducebrightness,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 reducebrightness            1
 standby              1
AC Power:
 reducebrightness            0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_reducebrightness(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_reducebrightness("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 lessbright            1
"#;
        assert_eq!(parse_pmset_custom_reducebrightness(other), (None, None));
    }

    #[test]
    fn parses_active_reducebrightness() {
        assert_eq!(
            parse_pmset_active_reducebrightness(
                "Currently in use:\n reducebrightness            1\n"
            ),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_reducebrightness(" reducebrightness            0\n"),
            Some(false)
        );
        assert_eq!(
            parse_pmset_active_reducebrightness(" womp                 1\n"),
            None
        );
        assert_eq!(
            parse_pmset_active_reducebrightness(" lessbright            1\n"),
            None
        );
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_reducebrightness_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_reducebrightness_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_reducebrightness_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_reducebrightness_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_reducebrightness_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(
            format_reducebrightness_parts(None, None, Some(true)),
            "On"
        );
        assert_eq!(format_reducebrightness_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod sms_parse_tests {
    use super::{format_sms_parts, parse_pmset_active_sms, parse_pmset_custom_sms};

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 sms            1
 standby              1
AC Power:
 sms            0
 womp                 1
"#;
        assert_eq!(parse_pmset_custom_sms(sample), (Some(true), Some(false)));
        assert_eq!(parse_pmset_custom_sms("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 reducebrightness            1
"#;
        assert_eq!(parse_pmset_custom_sms(other), (None, None));
    }

    #[test]
    fn parses_active_sms() {
        assert_eq!(
            parse_pmset_active_sms("Currently in use:\n sms            1\n"),
            Some(true)
        );
        assert_eq!(parse_pmset_active_sms(" sms            0\n"), Some(false));
        assert_eq!(parse_pmset_active_sms(" womp                 1\n"), None);
        assert_eq!(
            parse_pmset_active_sms(" reducebrightness            1\n"),
            None
        );
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_sms_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_sms_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_sms_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(format_sms_parts(Some(true), None, None), "On (Battery)");
        assert_eq!(format_sms_parts(None, Some(false), None), "Off (AC)");
        assert_eq!(format_sms_parts(None, None, Some(true)), "On");
        assert_eq!(format_sms_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod modem_ring_parse_tests {
    use super::{
        format_modem_ring_parts, parse_pmset_active_modem_ring, parse_pmset_custom_modem_ring,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 ring            1
 standby              1
AC Power:
 ring            0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_modem_ring(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_modem_ring("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 sms            1
"#;
        assert_eq!(parse_pmset_custom_modem_ring(other), (None, None));
    }

    #[test]
    fn parses_active_modem_ring() {
        assert_eq!(
            parse_pmset_active_modem_ring("Currently in use:\n ring            1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_modem_ring(" ring            0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_modem_ring(" womp                 1\n"), None);
        assert_eq!(parse_pmset_active_modem_ring(" sms            1\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_modem_ring_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_modem_ring_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_modem_ring_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_modem_ring_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_modem_ring_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_modem_ring_parts(None, None, Some(true)), "On");
        assert_eq!(format_modem_ring_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod sleepdisabled_parse_tests {
    use super::{
        format_sleepdisabled_parts, parse_pmset_active_sleepdisabled, parse_pmset_custom_sleepdisabled,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 SleepDisabled            1
 standby              1
AC Power:
 SleepDisabled            0
 womp                 1
"#;
        assert_eq!(
            parse_pmset_custom_sleepdisabled(sample),
            (Some(true), Some(false))
        );
        assert_eq!(parse_pmset_custom_sleepdisabled("nope"), (None, None));
        let other = r#"
Battery Power:
 standby              1
AC Power:
 ring            1
"#;
        assert_eq!(parse_pmset_custom_sleepdisabled(other), (None, None));
        let lower = r#"
Battery Power:
 sleepdisabled            1
AC Power:
 sleepdisabled            1
"#;
        assert_eq!(
            parse_pmset_custom_sleepdisabled(lower),
            (Some(true), Some(true))
        );
    }

    #[test]
    fn parses_active_sleepdisabled() {
        assert_eq!(
            parse_pmset_active_sleepdisabled("Currently in use:\n SleepDisabled            1\n"),
            Some(true)
        );
        assert_eq!(
            parse_pmset_active_sleepdisabled(" sleepdisabled            0\n"),
            Some(false)
        );
        assert_eq!(parse_pmset_active_sleepdisabled(" ring            1\n"), None);
        assert_eq!(parse_pmset_active_sleepdisabled(" sleep                 10\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(
            format_sleepdisabled_parts(Some(true), Some(true), Some(false)),
            "On"
        );
        assert_eq!(
            format_sleepdisabled_parts(Some(false), Some(false), Some(true)),
            "Off"
        );
        assert_eq!(
            format_sleepdisabled_parts(Some(true), Some(false), None),
            "AC Off · Battery On"
        );
        assert_eq!(
            format_sleepdisabled_parts(Some(true), None, None),
            "On (Battery)"
        );
        assert_eq!(
            format_sleepdisabled_parts(None, Some(false), None),
            "Off (AC)"
        );
        assert_eq!(format_sleepdisabled_parts(None, None, Some(true)), "On");
        assert_eq!(format_sleepdisabled_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod assertions_parse_tests {
    use super::{
        assertion_line_prevent_kind, assertion_line_process_name, format_assertions_parts,
        parse_pmset_assertions_flags, parse_pmset_assertions_holders,
    };

    const SAMPLE: &str = r#"
Assertion status system-wide:
   UserIsActive                   1
   PreventUserIdleDisplaySleep    1
   PreventSystemSleep             0
   ExternalMedia                  1
   PreventUserIdleSystemSleep     1
   NetworkClientActive            0
Listed by owning process:
   pid 689(sharingd): [0x1] 00:01:04 PreventUserIdleSystemSleep named: "Handoff"
   pid 360(mds): [0x2] 170:18:33 BackgroundTask named: "com.apple.metadata.mds.power"
   pid 1492(AnyDesk): [0x3] 169:50:33 PreventUserIdleSystemSleep named: "AnyDesk frontend"
   pid 345(powerd): [0x4] 81:57:36 PreventUserIdleSystemSleep named: "Powerd - Prevent sleep"
   pid 345(powerd): [0x5] 81:57:33 ExternalMedia named: "externalmediamounted"
   pid 92966(WhatsApp): [0x6] 138:00:17 PreventUserIdleSystemSleep named: "camera"
   pid 21267(Jami): [0x7] 127:08:23 PreventUserIdleDisplaySleep named: "Active Call"
   pid 406(WindowServer): [0x8] 00:00:00 UserIsActive named: "tickle"
Kernel Assertions: 0x124=USB,THNDR,MAGICWAKE
   id=576  level=255 0x20=THNDR
"#;

    #[test]
    fn parses_flags_and_holders() {
        assert_eq!(
            parse_pmset_assertions_flags(SAMPLE),
            vec!["Display", "Media", "System"]
        );
        assert_eq!(
            parse_pmset_assertions_holders(SAMPLE),
            vec![
                "sharingd".to_string(),
                "AnyDesk".to_string(),
                "powerd".to_string(),
                "WhatsApp".to_string(),
                "Jami".to_string(),
            ]
        );
        assert_eq!(parse_pmset_assertions_flags("nope"), Vec::<&str>::new());
        assert_eq!(parse_pmset_assertions_holders("nope"), Vec::<String>::new());
    }

    #[test]
    fn parses_process_line_helpers() {
        let line =
            r#"pid 1492(AnyDesk): [0x3] 169:50:33 PreventUserIdleSystemSleep named: "AnyDesk""#;
        assert_eq!(assertion_line_process_name(line).as_deref(), Some("AnyDesk"));
        assert_eq!(assertion_line_prevent_kind(line), Some("System"));
        assert_eq!(
            assertion_line_prevent_kind(
                r#"pid 360(mds): [0x2] BackgroundTask named: "mds""#
            ),
            None
        );
    }

    #[test]
    fn formats_flags_holders_and_overflow() {
        let holders: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"]
            .into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(
            format_assertions_parts(&["System", "Display"], &holders),
            "System · Display · a · b · c · d · e · (+2)"
        );
        assert_eq!(format_assertions_parts(&[], &[]), "");
        assert_eq!(
            format_assertions_parts(&["Media"], &["powerd".into()]),
            "Media · powerd"
        );
    }
}

#[cfg(test)]
mod powersched_parse_tests {
    use super::{
        compact_pmset_sched_oneshot, format_powersched_parts, parse_pmset_sched_events,
    };

    const SAMPLE: &str = r#"
Scheduled power events:
 [0]  wake at 11/25/26 07:00:00 by 'com.apple.alarm.user.wake'
 [1]  sleep at 11/25/26 23:30:00
Repeating power events:
  wake every Monday at 8:00AM
"#;

    #[test]
    fn parses_oneshot_and_repeating() {
        assert_eq!(
            parse_pmset_sched_events(SAMPLE),
            vec![
                "wake 11/25/26 07:00".to_string(),
                "sleep 11/25/26 23:30".to_string(),
                "wake every Monday at 8:00AM".to_string(),
            ]
        );
        assert_eq!(parse_pmset_sched_events("No scheduled events."), Vec::<String>::new());
        assert_eq!(parse_pmset_sched_events("nope"), Vec::<String>::new());
    }

    #[test]
    fn compacts_oneshot_and_overflow() {
        assert_eq!(
            compact_pmset_sched_oneshot("wake at 07/04/25 06:00:00 by 'root'").as_deref(),
            Some("wake 07/04/25 06:00")
        );
        let events: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"]
            .into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(
            format_powersched_parts(&events),
            "a · b · c · d · e · (+2)"
        );
        assert_eq!(format_powersched_parts(&[]), "");
    }
}

#[cfg(test)]
mod sysload_parse_tests {
    use super::{format_sysload_parts, parse_pmset_sysload_levels};

    const SAMPLE: &str = r#"
2026-11-24 21:57:18 +0100
  combined level = OK
  - user level = OK
  - battery level = Great
  - thermal level = Great
"#;

    #[test]
    fn parses_levels_and_formats() {
        let (c, u, b, t) = parse_pmset_sysload_levels(SAMPLE);
        assert_eq!(c.as_deref(), Some("OK"));
        assert_eq!(u.as_deref(), Some("OK"));
        assert_eq!(b.as_deref(), Some("Great"));
        assert_eq!(t.as_deref(), Some("Great"));
        assert_eq!(
            format_sysload_parts(c.as_deref(), u.as_deref(), b.as_deref(), t.as_deref()),
            "OK · user OK · bat Great · therm Great"
        );
        assert_eq!(
            parse_pmset_sysload_levels("nope"),
            (None, None, None, None)
        );
        assert_eq!(format_sysload_parts(None, None, None, None), "");
        assert_eq!(
            format_sysload_parts(Some("OK"), None, None, Some("Fair")),
            "OK · therm Fair"
        );
    }
}

#[cfg(test)]
mod ups_parse_tests {
    use super::{format_ups_parts, parse_pmset_ups_settings};

    const SAMPLE: &str = r#"
UPS settings:
  haltlevel		5
  haltafter		-1
  haltremain		10
"#;

    #[test]
    fn parses_settings_and_formats() {
        let (level, after, remain) = parse_pmset_ups_settings(SAMPLE);
        assert_eq!(level, Some(5));
        assert_eq!(after, Some(-1));
        assert_eq!(remain, Some(10));
        assert_eq!(
            format_ups_parts(level, after, remain),
            "halt 5% · after never · remain 10m"
        );
        assert_eq!(parse_pmset_ups_settings("No UPS attached."), (None, None, None));
        assert_eq!(format_ups_parts(None, None, None), "");
        assert_eq!(
            format_ups_parts(Some(20), Some(120), None),
            "halt 20% · after 2h"
        );
    }
}

#[cfg(test)]
mod pmlog_parse_tests {
    use super::{compact_pmset_pmlog_line, format_pmlog_parts, parse_pmset_pmlog_events};

    const SAMPLE: &str = "\
2026-11-21 11:26:12 +0100 Sleep              \tEntering Sleep state due to 'Maintenance Sleep'\n\
2026-11-21 11:27:01 +0100 DarkWake           \tDarkWake from Deep Idle [CDNP] : using BATT\n\
2026-11-21 11:27:02 +0100 Notification       \tDisplay is turned on\n\
2026-11-21 11:27:05 +0100 Wake               \tDarkWake to FullWake from Deep Idle [CDNP]\n\
2026-11-21 11:27:10 +0100 Assertions         \tPID 1(launchd) Summary PreventUserIdleSystemSleep \"x\"\n\
2026-11-21 11:28:00 +0100 Sleep              \tEntering Sleep state due to 'Idle Sleep'\n\
";

    #[test]
    fn parses_sleep_wake_domains_skips_assertions_and_formats_tail() {
        assert!(compact_pmset_pmlog_line(
            "2026-11-21 11:27:10 +0100 Assertions         \tPID 1(launchd) Summary"
        )
        .is_none());
        let events = parse_pmset_pmlog_events(SAMPLE);
        assert_eq!(events.len(), 5);
        assert!(events[0].starts_with("11-21 11:26 Sleep"));
        assert!(events[1].starts_with("11-21 11:27 DarkWake"));
        assert!(events[2].starts_with("11-21 11:27 Notification"));
        assert!(events[3].starts_with("11-21 11:27 Wake"));
        assert!(events[4].starts_with("11-21 11:28 Sleep"));
        let formatted = format_pmlog_parts(&events);
        assert!(formatted.contains("11-21 11:28 Sleep"));
        assert!(!formatted.contains("(+"));
        let many: Vec<String> = (0..7).map(|i| format!("e{i}")).collect();
        let with_overflow = format_pmlog_parts(&many);
        assert!(with_overflow.starts_with("(+2 older)"));
        assert!(with_overflow.contains("e6"));
        assert_eq!(format_pmlog_parts(&[]), "");
    }
}

#[cfg(test)]
mod therm_parse_tests {
    use super::{format_therm_parts, parse_pmset_therm_limits};

    const SAMPLE: &str = r#"
Note: No thermal warning level has been recorded
Note: No performance warning level has been recorded
  CPU_Scheduler_Limit         = 100
  CPU_Available_CPUs          = 8
  CPU_Speed_Limit             = 100
"#;

    const THROTTLED: &str = r#"
2020-10-06 12:25:52 +0200 CPU Power notify
    CPU_Scheduler_Limit     = 40
    CPU_Available_CPUs     = 8
    CPU_Speed_Limit     = 21
"#;

    #[test]
    fn parses_limits_and_formats() {
        let (sched, cpus, speed) = parse_pmset_therm_limits(SAMPLE);
        assert_eq!(sched, Some(100));
        assert_eq!(cpus, Some(8));
        assert_eq!(speed, Some(100));
        assert_eq!(
            format_therm_parts(sched, cpus, speed),
            "sched 100% · speed 100% · 8 cpus"
        );
        let (s2, c2, sp2) = parse_pmset_therm_limits(THROTTLED);
        assert_eq!(s2, Some(40));
        assert_eq!(c2, Some(8));
        assert_eq!(sp2, Some(21));
        assert_eq!(
            format_therm_parts(s2, c2, sp2),
            "sched 40% · speed 21% · 8 cpus"
        );
        assert_eq!(
            parse_pmset_therm_limits("Note: No thermal warning level has been recorded"),
            (None, None, None)
        );
        assert_eq!(format_therm_parts(None, None, None), "");
        assert_eq!(
            format_therm_parts(Some(100), None, Some(80)),
            "sched 100% · speed 80%"
        );
    }
}

#[cfg(test)]
mod pmstats_parse_tests {
    use super::{format_pmstats_parts, parse_pmset_stats_counts};

    const SAMPLE: &str = "\
Sleep Count:52
Dark Wake Count:52
User Wake Count:6
";

    #[test]
    fn parses_counts_and_formats() {
        let (sleep, dark, user) = parse_pmset_stats_counts(SAMPLE);
        assert_eq!(sleep, Some(52));
        assert_eq!(dark, Some(52));
        assert_eq!(user, Some(6));
        assert_eq!(
            format_pmstats_parts(sleep, dark, user),
            "sleep 52 · dark wake 52 · user wake 6"
        );
        assert_eq!(parse_pmset_stats_counts(""), (None, None, None));
        assert_eq!(format_pmstats_parts(None, None, None), "");
        assert_eq!(
            format_pmstats_parts(Some(1), None, Some(2)),
            "sleep 1 · user wake 2"
        );
    }
}

#[cfg(test)]
mod pmcap_parse_tests {
    use super::{format_pmcap_parts, parse_pmset_cap_list};

    const SAMPLE: &str = "\
Capabilities may change depending on hardware configuration:
 sleep
 displaysleep
 disksleep
 powernap
 standby
 hibernatemode
 lowpowermode
";

    #[test]
    fn parses_cap_list_and_formats() {
        let caps = parse_pmset_cap_list(SAMPLE);
        assert_eq!(
            caps,
            vec![
                "sleep".to_string(),
                "displaysleep".to_string(),
                "disksleep".to_string(),
                "powernap".to_string(),
                "standby".to_string(),
                "hibernatemode".to_string(),
                "lowpowermode".to_string(),
            ]
        );
        assert_eq!(
            format_pmcap_parts(&caps),
            "7 · sleep · displaysleep · disksleep · powernap · standby · hibernatemode · lowpowermode"
        );
        assert!(parse_pmset_cap_list("").is_empty());
        assert_eq!(format_pmcap_parts(&[]), "");
        let many: Vec<String> = (0..14).map(|i| format!("c{i}")).collect();
        assert_eq!(
            format_pmcap_parts(&many),
            "14 · c0 · c1 · c2 · c3 · c4 · c5 · c6 · c7 · c8 · c9 · c10 · c11 · +2 more"
        );
    }
}

#[cfg(test)]
mod displaysleep_parse_tests {
    use super::{
        format_displaysleep_mins, format_displaysleep_parts, parse_pmset_active_displaysleep,
        parse_pmset_custom_displaysleep,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 displaysleep         2
 disksleep            10
AC Power:
 displaysleep         10
 disksleep            10
"#;
        assert_eq!(
            parse_pmset_custom_displaysleep(sample),
            (Some(2), Some(10))
        );
        assert_eq!(parse_pmset_custom_displaysleep("nope"), (None, None));
    }

    #[test]
    fn parses_active_displaysleep() {
        assert_eq!(
            parse_pmset_active_displaysleep("Currently in use:\n displaysleep         10\n"),
            Some(10)
        );
        assert_eq!(
            parse_pmset_active_displaysleep(" displaysleep         0\n"),
            Some(0)
        );
        assert_eq!(parse_pmset_active_displaysleep("disksleep 10\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(format_displaysleep_mins(0), "Never");
        assert_eq!(format_displaysleep_mins(10), "10m");
        assert_eq!(format_displaysleep_mins(60), "1h");
        assert_eq!(format_displaysleep_mins(120), "2h");
        assert_eq!(
            format_displaysleep_parts(Some(10), Some(10), Some(2)),
            "10m"
        );
        assert_eq!(
            format_displaysleep_parts(Some(0), Some(0), Some(10)),
            "Never"
        );
        assert_eq!(
            format_displaysleep_parts(Some(2), Some(10), None),
            "AC 10m · Battery 2m"
        );
        assert_eq!(
            format_displaysleep_parts(Some(2), None, None),
            "2m (Battery)"
        );
        assert_eq!(format_displaysleep_parts(None, Some(10), None), "10m (AC)");
        assert_eq!(format_displaysleep_parts(None, None, Some(0)), "Never");
        assert_eq!(format_displaysleep_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod disksleep_parse_tests {
    use super::{
        format_disksleep_mins, format_disksleep_parts, parse_pmset_active_disksleep,
        parse_pmset_custom_disksleep,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 displaysleep         2
 disksleep            10
AC Power:
 displaysleep         10
 disksleep            0
"#;
        assert_eq!(
            parse_pmset_custom_disksleep(sample),
            (Some(10), Some(0))
        );
        assert_eq!(parse_pmset_custom_disksleep("nope"), (None, None));
    }

    #[test]
    fn parses_active_disksleep() {
        assert_eq!(
            parse_pmset_active_disksleep("Currently in use:\n disksleep            10\n"),
            Some(10)
        );
        assert_eq!(
            parse_pmset_active_disksleep(" disksleep            0\n"),
            Some(0)
        );
        assert_eq!(parse_pmset_active_disksleep("displaysleep 10\n"), None);
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(format_disksleep_mins(0), "Never");
        assert_eq!(format_disksleep_mins(10), "10m");
        assert_eq!(format_disksleep_mins(60), "1h");
        assert_eq!(
            format_disksleep_parts(Some(10), Some(10), Some(2)),
            "10m"
        );
        assert_eq!(
            format_disksleep_parts(Some(0), Some(0), Some(10)),
            "Never"
        );
        assert_eq!(
            format_disksleep_parts(Some(10), Some(0), None),
            "AC Never · Battery 10m"
        );
        assert_eq!(
            format_disksleep_parts(Some(10), None, None),
            "10m (Battery)"
        );
        assert_eq!(format_disksleep_parts(None, Some(10), None), "10m (AC)");
        assert_eq!(format_disksleep_parts(None, None, Some(0)), "Never");
        assert_eq!(format_disksleep_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod systemsleep_parse_tests {
    use super::{
        format_systemsleep_mins, format_systemsleep_parts, parse_pmset_active_systemsleep,
        parse_pmset_custom_systemsleep,
    };

    #[test]
    fn parses_custom_battery_and_ac() {
        let sample = r#"
Battery Power:
 Sleep On Power Button 1
 sleep                10
 displaysleep         2
 disksleep            10
AC Power:
 sleep                0
 displaysleep         10
 disksleep            10
"#;
        assert_eq!(
            parse_pmset_custom_systemsleep(sample),
            (Some(10), Some(0))
        );
        assert_eq!(parse_pmset_custom_systemsleep("nope"), (None, None));
        // Exact key only — displaysleep / disksleep must not win.
        assert_eq!(
            parse_pmset_custom_systemsleep(
                "Battery Power:\n displaysleep 2\n disksleep 10\nAC Power:\n displaysleep 10\n"
            ),
            (None, None)
        );
    }

    #[test]
    fn parses_active_systemsleep() {
        assert_eq!(
            parse_pmset_active_systemsleep("Currently in use:\n sleep                10\n"),
            Some(10)
        );
        assert_eq!(
            parse_pmset_active_systemsleep(" sleep                0\n"),
            Some(0)
        );
        assert_eq!(parse_pmset_active_systemsleep("displaysleep 10\n"), None);
        assert_eq!(parse_pmset_active_systemsleep("disksleep 10\n"), None);
        // "Sleep On Power Button 1" has key Sleep but non-numeric value → skip.
        assert_eq!(
            parse_pmset_active_systemsleep(" Sleep On Power Button 1\n"),
            None
        );
    }

    #[test]
    fn formats_matched_split_and_active() {
        assert_eq!(format_systemsleep_mins(0), "Never");
        assert_eq!(format_systemsleep_mins(10), "10m");
        assert_eq!(format_systemsleep_mins(60), "1h");
        assert_eq!(
            format_systemsleep_parts(Some(10), Some(10), Some(2)),
            "10m"
        );
        assert_eq!(
            format_systemsleep_parts(Some(0), Some(0), Some(10)),
            "Never"
        );
        assert_eq!(
            format_systemsleep_parts(Some(10), Some(0), None),
            "AC Never · Battery 10m"
        );
        assert_eq!(
            format_systemsleep_parts(Some(10), None, None),
            "10m (Battery)"
        );
        assert_eq!(format_systemsleep_parts(None, Some(10), None), "10m (AC)");
        assert_eq!(format_systemsleep_parts(None, None, Some(0)), "Never");
        assert_eq!(format_systemsleep_parts(None, None, None), "");
    }
}

#[cfg(test)]
mod internet_sharing_parse_tests {
    use super::{parse_nat_enabled_plistbuddy, parse_network_sharing_running};

    #[test]
    fn parses_nat_enabled_flags() {
        assert_eq!(parse_nat_enabled_plistbuddy("0\n"), Some(false));
        assert_eq!(parse_nat_enabled_plistbuddy("1"), Some(true));
        assert_eq!(parse_nat_enabled_plistbuddy("true"), Some(true));
        assert_eq!(parse_nat_enabled_plistbuddy("nope"), None);
    }

    #[test]
    fn parses_network_sharing_running_state() {
        let idle = "active count = 0\n\tstate = not running\n\tprogram = /usr/libexec/InternetSharing\n";
        assert_eq!(parse_network_sharing_running(idle), Some(false));
        let active = "\tstate = running\n";
        assert_eq!(parse_network_sharing_running(active), Some(true));
        assert_eq!(parse_network_sharing_running("path = /tmp\n"), None);
    }
}

#[cfg(test)]
mod vpn_status_parse_tests {
    use super::parse_scutil_nc_vpn_list;

    #[test]
    fn prefers_connected_vpn_name() {
        let sample = r#"
Available network connection services in the current set (*=enabled):
* (Connected)      A1B2C3D4-E5F6-7890-ABCD-EF1234567890 VPN "NordLayer" [VPN:IKEv2]
* (Disconnected)   11111111-2222-3333-4444-555555555555 VPN "Work" [VPN:L2TP]
"#;
        assert_eq!(
            parse_scutil_nc_vpn_list(sample),
            "connected · NordLayer"
        );
    }

    #[test]
    fn reports_disconnected_when_none_up() {
        let sample = r#"
* (Disconnected)   11111111-2222-3333-4444-555555555555 VPN "Work" [VPN:L2TP]
* (Disconnected)   AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE PPP "Legacy" [PPP:PPTP]
"#;
        assert_eq!(
            parse_scutil_nc_vpn_list(sample),
            "disconnected · Work, Legacy"
        );
    }

    #[test]
    fn ignores_non_vpn_services() {
        let sample = r#"
* (Connected)      00000000-0000-0000-0000-000000000001 Ethernet "USB 10/100/1000 LAN"
* (Disconnected)   00000000-0000-0000-0000-000000000002 Wi-Fi "Wi-Fi"
"#;
        assert!(parse_scutil_nc_vpn_list(sample).is_empty());
    }
}

#[cfg(test)]
mod homebrew_cask_install_tests {
    use super::homebrew_cask_staged_at;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_staged_caskroom_dir() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("mac-stats-caskroom-test-{stamp}"));
        fs::create_dir_all(&dir).expect("temp caskroom");
        let path = dir.to_string_lossy().into_owned();
        assert!(homebrew_cask_staged_at(&[&path]));
        assert!(!homebrew_cask_staged_at(&[
            "/tmp/mac-stats-caskroom-missing-does-not-exist"
        ]));
        let _ = fs::remove_dir_all(&dir);
    }
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
            debug3!(
                "Reading changelog from current directory: {:?}",
                cwd_changelog
            );
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
                debug3!(
                    "Reading changelog from parent directory: {:?}",
                    parent_changelog
                );
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

#[tauri::command]
pub fn get_ai_agent_enabled() -> bool {
    crate::config::Config::ai_agent_enabled()
}

#[tauri::command]
pub fn set_ai_agent_enabled(enabled: bool) -> Result<bool, String> {
    crate::config::Config::set_ai_agent_enabled(enabled)?;
    if enabled {
        crate::ai_agent_stack::ensure_ai_agent_stack_started();
    } else {
        // Explicit off (Settings / Reset) — do not auto-enable on next launch when Ollama is up.
        let _ = crate::config::Config::set_ai_agent_ollama_auto_probe_done(true);
    }
    Ok(crate::config::Config::ai_agent_enabled())
}

#[tauri::command]
pub fn get_agent_judge_enabled() -> bool {
    crate::config::Config::agent_judge_enabled()
}

#[tauri::command]
pub fn set_agent_judge_enabled(enabled: bool) -> Result<bool, String> {
    crate::config::Config::set_agent_judge_enabled(enabled)?;
    Ok(crate::config::Config::agent_judge_enabled())
}

#[tauri::command]
pub fn get_agent_judge_on_failure_only() -> bool {
    crate::config::Config::agent_judge_on_failure_only()
}

#[tauri::command]
pub fn set_agent_judge_on_failure_only(on_failure_only: bool) -> Result<bool, String> {
    crate::config::Config::set_agent_judge_on_failure_only(on_failure_only)?;
    Ok(crate::config::Config::agent_judge_on_failure_only())
}

#[tauri::command]
pub fn get_ori_lifecycle_enabled() -> bool {
    crate::config::Config::ori_lifecycle_enabled()
}

#[tauri::command]
pub fn set_ori_lifecycle_enabled(enabled: bool) -> Result<bool, String> {
    crate::config::Config::set_ori_lifecycle_enabled(enabled)?;
    Ok(crate::config::Config::ori_lifecycle_enabled())
}

#[tauri::command]
pub fn get_having_fun_enabled() -> bool {
    crate::config::Config::having_fun_enabled()
}

#[tauri::command]
pub fn set_having_fun_enabled(enabled: bool) -> Result<bool, String> {
    crate::config::Config::set_having_fun_enabled(enabled)?;
    Ok(crate::config::Config::having_fun_enabled())
}

#[tauri::command]
pub fn get_discord_voice_stt_enabled() -> bool {
    crate::config::Config::discord_voice_stt_enabled()
}

#[tauri::command]
pub fn set_discord_voice_stt_enabled(enabled: bool) -> Result<bool, String> {
    crate::config::Config::set_discord_voice_stt_enabled(enabled)?;
    Ok(crate::config::Config::discord_voice_stt_enabled())
}

#[tauri::command]
pub fn get_menu_bar_compact() -> bool {
    crate::config::Config::menu_bar_compact()
}

#[tauri::command]
pub fn set_menu_bar_compact(compact: bool) -> Result<bool, String> {
    crate::config::Config::set_menu_bar_compact(compact)?;
    Ok(crate::config::Config::menu_bar_compact())
}

#[tauri::command]
pub fn get_cpu_window_compact() -> bool {
    crate::config::Config::cpu_window_compact()
}

#[tauri::command]
pub fn set_cpu_window_compact(compact: bool) -> Result<bool, String> {
    crate::config::Config::set_cpu_window_compact(compact)?;
    resize_cpu_window_for_compact(compact);
    Ok(crate::config::Config::cpu_window_compact())
}

#[tauri::command]
pub fn get_cpu_window_ui_state() -> serde_json::Value {
    crate::config::Config::cpu_window_ui_state()
}

#[tauri::command]
pub fn set_cpu_window_ui_state(state: serde_json::Value) -> Result<serde_json::Value, String> {
    crate::config::Config::set_cpu_window_ui_state(state)
}

/// One-shot: which CPU-window section to open (`agent-ops`, …). See `Config::take_open_ui_section`.
#[tauri::command]
pub fn take_open_ui_section() -> Option<String> {
    crate::config::Config::take_open_ui_section()
}

fn resize_cpu_window_for_compact(compact: bool) {
    use crate::state::APP_HANDLE;
    use tauri::Manager;
    let Some(app) = APP_HANDLE.get() else {
        return;
    };
    let Some(win) = app.get_webview_window("cpu") else {
        return;
    };
    let (w, h) = if compact {
        (520.0_f64, 560.0_f64)
    } else {
        (820.0_f64, 995.0_f64)
    };
    let _ = win.set_size(tauri::Size::Logical(tauri::LogicalSize::new(w, h)));
}

/// Cap — match `MAX_PINNED_PROCESSES` in cpu.js.
pub const MAX_PINNED_PROCESS_NAMES: usize = 6;

/// Clean + cap pinned process name list (UI / Discord shared).
pub fn normalize_pinned_process_names(names: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for n in names {
        let t = n.trim();
        if t.is_empty() {
            continue;
        }
        if out.iter().any(|e: &String| e == t) {
            continue;
        }
        out.push(t.to_string());
        if out.len() >= MAX_PINNED_PROCESS_NAMES {
            break;
        }
    }
    out
}

/// Load pinned favorites from `~/.mac-stats/pinned_processes.json` (JSON string array).
pub fn load_pinned_process_names() -> Vec<String> {
    let path = crate::config::Config::pinned_processes_file_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let parsed: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let arr = if let Some(a) = parsed.as_array() {
        a.clone()
    } else if let Some(a) = parsed.get("names").and_then(|v| v.as_array()) {
        a.clone()
    } else {
        return Vec::new();
    };
    normalize_pinned_process_names(arr.into_iter().filter_map(|v| {
        v.as_str().map(|s| s.to_string())
    }))
}

/// Persist pinned favorites for Discord `/processes pinned` + multi-WebView sync.
pub fn save_pinned_process_names(names: &[String]) -> Result<Vec<String>, String> {
    let cleaned = normalize_pinned_process_names(names.iter().cloned());
    let path = crate::config::Config::pinned_processes_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&cleaned).map_err(|e| e.to_string())?;
    crate::config::write_text_atomic(&path, &json)?;
    Ok(cleaned)
}

#[tauri::command]
pub fn get_pinned_process_names() -> Vec<String> {
    load_pinned_process_names()
}

#[tauri::command]
pub fn set_pinned_process_names(names: Vec<String>) -> Result<Vec<String>, String> {
    save_pinned_process_names(&names)
}

/// Look up current CPU/pid for pinned process names (highest-CPU match per name).
#[tauri::command]
pub fn get_processes_by_names(names: Vec<String>) -> Vec<ProcessUsage> {
    if names.is_empty() {
        return Vec::new();
    }
    let Ok(mut guard) = SYSTEM.try_lock() else {
        return Vec::new();
    };
    let Some(sys) = guard.as_mut() else {
        return Vec::new();
    };
    use sysinfo::ProcessesToUpdate;
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let gpu_map = gpu_processes::gpu_usage_by_pid();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let mut best: Option<ProcessUsage> = None;
        for (pid, proc) in sys.processes() {
            let n = proc.name().to_string_lossy().to_string();
            if n != name {
                continue;
            }
            let pid_u = pid.as_u32();
            let cand = ProcessUsage {
                name: n,
                cpu: proc.cpu_usage(),
                gpu: gpu_map.get(&pid_u).copied().unwrap_or(0.0),
                pid: pid_u,
                memory: proc.memory(),
            };
            if best
                .as_ref()
                .map(|b| cand.cpu > b.cpu)
                .unwrap_or(true)
            {
                best = Some(cand);
            }
        }
        if let Some(p) = best {
            out.push(p);
        }
    }
    out
}

/// Attach AGX GPU % and ensure high-GPU processes appear in the top list.
fn enrich_processes_with_gpu(mut processes: Vec<ProcessUsage>, sys: &System) -> Vec<ProcessUsage> {
    let gpu_map = gpu_processes::gpu_usage_by_pid();
    for p in &mut processes {
        if let Some(g) = gpu_map.get(&p.pid) {
            p.gpu = *g;
        }
    }
    let mut have: std::collections::HashSet<u32> = processes.iter().map(|p| p.pid).collect();
    let mut gpu_ranked: Vec<(u32, f32)> = gpu_map.into_iter().collect();
    gpu_ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (pid, gpu) in gpu_ranked.into_iter().take(8) {
        if gpu < 1.0 {
            break;
        }
        if have.contains(&pid) {
            continue;
        }
        let sys_pid = sysinfo::Pid::from_u32(pid);
        if let Some(proc) = sys.process(sys_pid) {
            processes.push(ProcessUsage {
                name: proc.name().to_string_lossy().to_string(),
                cpu: proc.cpu_usage(),
                gpu,
                pid,
                memory: proc.memory(),
            });
            have.insert(pid);
        }
    }
    processes.sort_by(|a, b| {
        let sa = a.gpu.max(a.cpu);
        let sb = b.gpu.max(b.cpu);
        sb.partial_cmp(&sa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.cpu
                    .partial_cmp(&a.cpu)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    processes.truncate(10);
    processes
}

#[tauri::command]
pub fn reset_config_to_monitor_defaults() -> Result<String, String> {
    crate::config::Config::reset_config_to_monitor_defaults()?;
    Ok("Monitor defaults applied (aiAgentEnabled=false, menuBarCompact=true). Restart recommended for Discord/scheduler.".into())
}

/// Set window decorations preference
#[tauri::command]
pub fn set_window_decorations(decorations: bool) -> Result<(), String> {
    use crate::config::Config;
    use serde_json::{json, Value};

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

    let before: Value = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    let mut after = before.clone();
    match after.as_object_mut() {
        Some(obj) => {
            obj.insert("windowDecorations".to_string(), json!(decorations));
        }
        None => {
            after = json!({ "windowDecorations": decorations });
        }
    }

    if let Err(e) = crate::config::reject_if_protected_config_json_changed(&before, &after) {
        tracing::warn!(
            "set_window_decorations: protected-config guard blocked merge: {}",
            e
        );
        return Err(e);
    }

    crate::config::write_text_atomic(
        &config_path,
        &serde_json::to_string_pretty(&after).unwrap_or_else(|_| after.to_string()),
    )
    .map_err(|e| format!("Failed to write config file: {}", e))?;

    crate::debug3!(
        "Window decorations preference set to: {} (saved to config file, merged with existing JSON)",
        decorations
    );
    Ok(())
}

/// How long a Top Processes snapshot stays fresh while the CPU window is open.
/// Must stay ≥ the CPU-window process-list poll (30s). A 5–10s TTL forced
/// `refresh_processes(All)` on almost every `get_cpu_details` while the window
/// was visible and kept Graphics and Media / host CPU hot (#14). #15 restores
/// a live 30s process list so the window is not stuck on "None yet".
const PROCESS_CACHE_TTL_SECS: u64 = 30;

#[tauri::command]
pub fn get_cpu_details() -> CpuDetails {
    // STEP 5: Rate limiting - prevent get_cpu_details from being called too frequently
    // Process list still refreshes when PROCESS_CACHE is older than PROCESS_CACHE_TTL_SECS.
    let should_allow_full_call = match crate::state::LAST_CPU_DETAILS_CALL.try_lock() {
        Ok(mut last_call) => {
            let now = std::time::Instant::now();
            // Align with CPU-window poll (~2s). A 600s floor left gauges at "None yet" (#15).
            let should = last_call
                .as_ref()
                .map(|lc| now.duration_since(*lc).as_secs_f64() >= 1.5)
                .unwrap_or(true);
            if should {
                *last_call = Some(now);
                true
            } else {
                false
            }
        }
        Err(_) => {
            // Lock held - allow call (non-blocking)
            true
        }
    };

    // Still honour process-cache TTL on the rate-limited path (without a full metrics rebuild).
    let should_check_process_cache = true;

    if !should_allow_full_call {
        debug3!("get_cpu_details() rate limited - returning cached values for most metrics");
        // Return cached values immediately without doing expensive work
        // BUT: Still check and refresh process cache if stale (>PROCESS_CACHE_TTL_SECS)
        let (usage, load, uptime_secs, ram_percent, ram_used_bytes, ram_total_bytes) =
            match crate::state::SYSTEM.try_lock() {
            Ok(sys) => {
                if let Some(sys) = sys.as_ref() {
                    let total = sys.total_memory();
                    let used = sys.used_memory();
                    let pct = if total > 0 {
                        (used as f32 / total as f32) * 100.0
                    } else {
                        0.0
                    };
                    (
                        sys.global_cpu_usage(),
                        sysinfo::System::load_average(),
                        sysinfo::System::uptime(),
                        pct,
                        used,
                        total,
                    )
                } else {
                    (
                        0.0,
                        sysinfo::LoadAvg {
                            one: 0.0,
                            five: 0.0,
                            fifteen: 0.0,
                        },
                        0,
                        0.0,
                        0,
                        0,
                    )
                }
            }
            Err(_) => (
                0.0,
                sysinfo::LoadAvg {
                    one: 0.0,
                    five: 0.0,
                    fifteen: 0.0,
                },
                0,
                0.0,
                0,
                0,
            ),
        };

        // Return cached values only
        let (temperature, frequency, p_core_frequency, e_core_frequency) = (
            crate::state::TEMP_CACHE
                .try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(t, _)| *t))
                .unwrap_or(0.0),
            crate::state::FREQ_CACHE
                .try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(f, _)| *f))
                .unwrap_or(crate::metrics::get_nominal_frequency()),
            crate::state::P_CORE_FREQ_CACHE
                .try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(f, _)| *f))
                .unwrap_or(0.0),
            crate::state::E_CORE_FREQ_CACHE
                .try_lock()
                .ok()
                .and_then(|c| c.as_ref().map(|(f, _)| *f))
                .unwrap_or(0.0),
        );

        // Check process cache age even when rate-limited.
        // If stale (>PROCESS_CACHE_TTL_SECS), refresh it now.
        let processes = if should_check_process_cache {
            // Focused + visible: visible-but-unfocused still refreshed processes (#14).
            let should_collect_processes = crate::state::cpu_window_active_for_metrics();

            if should_collect_processes {
                match crate::state::PROCESS_CACHE.try_lock() {
                    Ok(cache) => {
                        if let Some((procs, timestamp)) = cache.as_ref() {
                            let age_secs = timestamp.elapsed().as_secs();
                            if age_secs >= PROCESS_CACHE_TTL_SECS {
                                // Cache is stale - refresh now even if rate-limited
                                debug3!("Process cache is stale ({}s) - refreshing now (even though rate-limited)", age_secs);
                                // Need SYSTEM lock to refresh processes
                                match crate::state::SYSTEM.try_lock() {
                                    Ok(mut sys) => {
                                        if let Some(sys) = sys.as_mut() {
                                            use sysinfo::ProcessesToUpdate;
                                            sys.refresh_processes(ProcessesToUpdate::All, true);

                                            let mut processes: Vec<crate::metrics::ProcessUsage> =
                                                sys.processes()
                                                    .iter()
                                                    .map(|(pid, proc)| {
                                                        crate::metrics::ProcessUsage {
                                                            name: proc
                                                                .name()
                                                                .to_string_lossy()
                                                                .to_string(),
                                                            cpu: proc.cpu_usage(),
                                                            gpu: 0.0,
                                                            pid: pid.as_u32(),
                                                            memory: proc.memory(),
                                                        }
                                                    })
                                                    .collect();

                                            processes.sort_by(|a, b| {
                                                b.cpu
                                                    .partial_cmp(&a.cpu)
                                                    .unwrap_or(std::cmp::Ordering::Equal)
                                            });
                                            processes.truncate(12);
                                            let processes =
                                                enrich_processes_with_gpu(processes, sys);

                                            // Update cache
                                            if let Ok(mut process_cache) =
                                                crate::state::PROCESS_CACHE.try_lock()
                                            {
                                                *process_cache = Some((
                                                    processes.clone(),
                                                    std::time::Instant::now(),
                                                ));
                                                debug3!(
                                                    "Process cache refreshed (rate-limited call)"
                                                );
                                            }

                                            processes
                                        } else {
                                            procs.clone()
                                        }
                                    }
                                    Err(_) => procs.clone(), // SYSTEM locked, return cached
                                }
                            } else {
                                procs.clone()
                            }
                        } else {
                            Vec::new()
                        }
                    }
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Get cached battery and power info
        let (battery_level, is_charging, has_battery) = crate::state::BATTERY_CACHE
            .try_lock()
            .ok()
            .and_then(|c| {
                c.as_ref()
                    .map(|(level, charging, _)| (*level, *charging, *level >= 0.0))
            })
            .unwrap_or((-1.0, false, false));

        // Use get_power_consumption() for consistent cache handling
        // This ensures we always return cached values (even if stale) instead of 0.0
        let (cpu_power, gpu_power) = get_power_consumption();
        let gpu_usage = get_gpu_usage();

        // Check if we actually have power values (even if 0, if we have a cache entry, we can read power)
        // This is more reliable than checking the flags, which might not be set yet
        let has_power_cache = crate::state::POWER_CACHE
            .try_lock()
            .ok()
            .map(|c| c.is_some())
            .unwrap_or(false);

        // If we have power cache, we can read power (even if values are currently 0)
        // This prevents showing "Requires root privileges" when we're just waiting for the first read
        // OR if we have actual power values > 0, we definitely can read power
        let can_read_cpu_power =
            has_power_cache || cpu_power > 0.0 || crate::metrics::can_read_cpu_power();
        let can_read_gpu_power =
            has_power_cache || gpu_power > 0.0 || crate::metrics::can_read_gpu_power();

        let disk_percent = get_disk_usage_percent(false);
        let thermal_state = crate::ffi::objc::read_process_thermal_state()
            .unwrap_or("")
            .to_string();
        let low_power_mode = crate::ffi::objc::read_process_low_power_mode();

        return CpuDetails {
            usage,
            gpu_usage,
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
            ram_percent,
            ram_used_bytes,
            ram_total_bytes,
            disk_percent,
            top_processes: processes,
            chip_info: crate::metrics::get_chip_info(),
            can_read_temperature: crate::metrics::can_read_temperature(),
            can_read_frequency: crate::metrics::can_read_frequency(),
            can_read_cpu_power,
            can_read_gpu_power,
            battery_level,
            is_charging,
            has_battery,
            thermal_state,
            low_power_mode,
        };
    }

    debug3!("get_cpu_details() called");

    // Focused + visible: visible-but-unfocused still refreshed the process list (#14).
    // If window was closed (destroyed), get_webview_window returns None.
    let should_collect_processes = crate::state::cpu_window_active_for_metrics();

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
                let processes = crate::state::PROCESS_CACHE
                    .try_lock()
                    .ok()
                    .and_then(|c| c.as_ref().map(|(p, _)| p.clone()))
                    .unwrap_or_default();
                // Return 0.0 for usage (will be updated on next refresh)
                (0.0, load, uptime_secs, processes)
            } else {
                let sys = sys.as_mut().unwrap();
                // Refresh on the full-call path (~1.5s floor). The glass UI must
                // not depend on the menu-bar loop alone — that left rings at 0%
                // when the background thread was starved to 600s (#14 leftover).
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                let usage = sys.global_cpu_usage();
                let load = sysinfo::System::load_average();
                let uptime_secs = sysinfo::System::uptime();
                debug3!(
                    "System uptime: {} seconds ({} days, {} hours, {} minutes)",
                    uptime_secs,
                    uptime_secs / 86400,
                    (uptime_secs % 86400) / 3600,
                    (uptime_secs % 3600) / 60
                );

                // Only collect processes if window is visible (saves CPU when window is closed)
                let processes = if should_collect_processes {
                    // Cache process list for PROCESS_CACHE_TTL_SECS while the window is open (#14).
                    // Always check cache first and return immediately if available.
                    let cached_processes = match PROCESS_CACHE.try_lock() {
                        Ok(cache) => cache.as_ref().map(|(procs, timestamp)| {
                            let age_secs = timestamp.elapsed().as_secs();
                            (procs.clone(), age_secs)
                        }),
                        Err(_) => None, // Lock held, skip cache check
                    };

                    // Fresh cache: return immediately. Empty cache: refresh for first paint.
                    if let Some((cached_procs, age_secs)) = cached_processes {
                        if age_secs < PROCESS_CACHE_TTL_SECS {
                            debug3!(
                                "Returning cached process list (age: {}s) - refresh every {}s",
                                age_secs,
                                PROCESS_CACHE_TTL_SECS
                            );
                            cached_procs
                        } else {
                            debug3!(
                                "Process cache is stale ({}s), refreshing now ({}s interval)",
                                age_secs,
                                PROCESS_CACHE_TTL_SECS
                            );
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
                                    gpu: 0.0,
                                    pid: pid.as_u32(),
                                    memory: proc.memory(),
                                })
                                .collect();

                            // Sort by CPU usage (descending) to get actual top processes
                            processes.sort_by(|a, b| {
                                b.cpu
                                    .partial_cmp(&a.cpu)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });

                            // Take top candidates then merge high-GPU users
                            processes.truncate(12);
                            let processes = enrich_processes_with_gpu(processes, sys);

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
                                gpu: 0.0,
                                pid: pid.as_u32(),
                                memory: proc.memory(),
                            })
                            .collect();

                        // Sort by CPU usage (descending) to get actual top processes
                        processes.sort_by(|a, b| {
                            b.cpu
                                .partial_cmp(&a.cpu)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        // Take top candidates then merge high-GPU users
                        processes.truncate(12);
                        let processes = enrich_processes_with_gpu(processes, sys);

                        // Update cache
                        if let Ok(mut cache) = PROCESS_CACHE.try_lock() {
                            *cache = Some((processes.clone(), std::time::Instant::now()));
                            debug3!("Process cache updated (refreshed from system)");
                        }

                        processes
                    }
                } else {
                    // Window not visible - return empty process list to save CPU
                    debug3!("Window not visible, skipping process collection");
                    Vec::new()
                };

                (usage, load, uptime_secs, processes)
            }
        }
        Err(_) => {
            // Lock is held - return defaults immediately, don't retry
            debug3!(
                "WARNING: SYSTEM mutex locked in get_cpu_details, returning defaults immediately"
            );
            write_structured_log(
                "lib.rs:697",
                "SYSTEM locked, returning defaults",
                &serde_json::json!({}),
                "L",
            );
            (
                0.0,
                sysinfo::LoadAvg {
                    one: 0.0,
                    five: 0.0,
                    fifteen: 0.0,
                },
                0,
                Vec::new(),
            )
        }
    };

    // CRITICAL: Use cached values or defaults - don't call expensive functions
    // SMC calls and other operations can block the main thread
    // OPTIMIZATION Phase 3: Use OnceLock for fast capability flag access (no locking)
    let (
        temperature,
        frequency,
        p_core_frequency,
        e_core_frequency,
        cpu_power,
        gpu_power,
        chip_info,
        can_read_temperature,
        can_read_frequency,
        can_read_cpu_power,
        can_read_gpu_power,
        battery_level,
        is_charging,
        has_battery,
    ) = {
        // Get cached access flags (fast OnceLock access, no blocking)
        let can_read_freq = CAN_READ_FREQUENCY.get().copied().unwrap_or(false);
        let can_read_cpu_p = CAN_READ_CPU_POWER.get().copied().unwrap_or(false);
        let can_read_gpu_p = CAN_READ_GPU_POWER.get().copied().unwrap_or(false);

        // CRITICAL: Read temperature from cache (updated by background thread)
        // Non-blocking read - returns 0.0 if cache is locked or stale
        // Keep a scheduling margin beyond the background sampling interval.
        let temperature = match TEMP_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((temp, timestamp)) = cache.as_ref() {
                    if timestamp.elapsed() < TEMP_CACHE_MAX_AGE {
                        *temp
                    } else {
                        debug3!(
                            "Temperature cache is stale ({}s old), using 0.0",
                            timestamp.elapsed().as_secs()
                        );
                        0.0
                    }
                } else {
                    0.0
                }
            }
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
                        debug3!(
                            "Frequency cache is stale ({}s old), using nominal frequency",
                            timestamp.elapsed().as_secs()
                        );
                        get_nominal_frequency()
                    }
                } else {
                    // No cached value, use nominal
                    get_nominal_frequency()
                }
            }
            Err(_) => {
                // Cache locked - return nominal frequency (non-blocking)
                get_nominal_frequency()
            }
        };

        // Read P-core and E-core frequencies from cache
        let freq_logging = crate::state::FREQUENCY_LOGGING_ENABLED
            .lock()
            .map(|f| *f)
            .unwrap_or(false);

        let p_core_frequency = match P_CORE_FREQ_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((freq, timestamp)) = cache.as_ref() {
                    let age_secs = timestamp.elapsed().as_secs();
                    if age_secs < 35 {
                        if freq_logging {
                            debug3!(
                                "P-core frequency from cache: {:.2} GHz (age: {}s)",
                                *freq,
                                age_secs
                            );
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
            }
            Err(_) => {
                if freq_logging {
                    debug3!("P-core frequency cache is locked, falling back to nominal");
                }
                get_nominal_frequency()
            }
        };

        let e_core_frequency = match E_CORE_FREQ_CACHE.try_lock() {
            Ok(cache) => {
                if let Some((freq, timestamp)) = cache.as_ref() {
                    let age_secs = timestamp.elapsed().as_secs();
                    if age_secs < 35 {
                        if freq_logging {
                            debug3!(
                                "E-core frequency from cache: {:.2} GHz (age: {}s)",
                                *freq,
                                age_secs
                            );
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
            }
            Err(_) => {
                if freq_logging {
                    debug3!("E-core frequency cache is locked, falling back to nominal");
                }
                get_nominal_frequency()
            }
        };

        // Use cached chip info or default - ensure it's initialized by calling get_chip_info()
        let chip = get_chip_info();

        // Get power consumption (cached)
        let (cpu_power_val, gpu_power_val) = get_power_consumption();

        // Get battery info (cached)
        let (battery_level_val, is_charging_val, has_battery_val) = get_battery_info();

        // Return cached temperature, frequency, power, battery, and defaults for other expensive values
        (
            temperature,
            frequency,
            p_core_frequency,
            e_core_frequency,
            cpu_power_val,
            gpu_power_val,
            chip,
            can_read_temp,
            can_read_freq,
            can_read_cpu_p,
            can_read_gpu_p,
            battery_level_val,
            is_charging_val,
            has_battery_val,
        )
    };

    // Log data being sent to frontend for debugging
    let power_logging = crate::state::POWER_USAGE_LOGGING_ENABLED
        .lock()
        .map(|f| *f)
        .unwrap_or(false);

    if power_logging {
        debug3!("get_cpu_details returning: temperature={:.1}°C, frequency={:.2} GHz, cpu_power={:.2}W, gpu_power={:.2}W, battery={:.1}%, charging={}, has_battery={}",
            temperature, frequency, cpu_power, gpu_power, battery_level, is_charging, has_battery);
    } else {
        debug3!("get_cpu_details returning: temperature={:.1}°C, frequency={:.2} GHz, can_read_temperature={}, can_read_frequency={}", temperature, frequency, can_read_temperature, can_read_frequency);
    }

    let gpu_usage = get_gpu_usage();

    let (ram_percent, ram_used_bytes, ram_total_bytes) = match SYSTEM.try_lock() {
        Ok(sys) => {
            if let Some(sys) = sys.as_ref() {
                let total = sys.total_memory();
                let used = sys.used_memory();
                let pct = if total > 0 {
                    (used as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                (pct, used, total)
            } else {
                (0.0, 0, 0)
            }
        }
        Err(_) => (0.0, 0, 0),
    };

    let disk_percent = get_disk_usage_percent(false);
    let thermal_state = crate::ffi::objc::read_process_thermal_state()
        .unwrap_or("")
        .to_string();
    let low_power_mode = crate::ffi::objc::read_process_low_power_mode();

    CpuDetails {
        usage,
        gpu_usage,
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
        ram_percent,
        ram_used_bytes,
        ram_total_bytes,
        disk_percent,
        top_processes,
        chip_info,
        can_read_temperature,
        can_read_frequency,
        can_read_cpu_power,
        can_read_gpu_power,
        battery_level,
        is_charging,
        has_battery,
        thermal_state,
        low_power_mode,
    }
}

/// Get detailed information about a specific process by PID
#[tauri::command]
pub fn get_process_details(pid: u32) -> Result<ProcessDetails, String> {
    use sysinfo::Pid;

    debug3!("get_process_details() called for PID: {}", pid);

    // Focused + visible: Process Details is part of the CPU window (#14).
    let should_refresh_processes = crate::state::cpu_window_active_for_metrics();

    if !should_refresh_processes {
        debug3!("CPU window not focused/visible, skipping process refresh in get_process_details");
        // Still try to get the process from cache if available, but don't refresh
    }

    // Use try_lock to avoid blocking - collect all data while lock is held
    match SYSTEM.try_lock() {
        Ok(mut sys) => {
            if sys.is_none() {
                return Err("System not initialized".to_string());
            }
            let sys = sys.as_mut().unwrap();

            // Details modal refreshes every few seconds — never refresh *all*
            // processes here (~8ms+ each). Touch only this PID (~0.02ms).
            if should_refresh_processes {
                use sysinfo::ProcessesToUpdate;
                let target = Pid::from_u32(pid);
                sys.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
            }

            // Get the process while lock is held
            if let Some(proc) = sys.process(Pid::from_u32(pid)) {
                // Get parent process information while lock is still held
                let (parent_pid, parent_name) = if let Some(ppid) = proc.parent() {
                    let parent_proc = sys.process(ppid);
                    let parent_name = parent_proc.map(|p| p.name().to_string_lossy().to_string());
                    (Some(ppid.as_u32()), parent_name)
                } else {
                    (None, None)
                };

                // Get user ID and username information
                // Parse the UID string to get the numeric value for getpwuid
                let (user_id, user_name) = if let Some(uid) = proc.user_id() {
                    let uid_str = uid.to_string();
                    let username = uid_str.parse::<u32>().ok().and_then(get_username_from_uid);
                    (Some(uid_str), username)
                } else {
                    (None, None)
                };

                let (effective_user_id, effective_user_name) =
                    if let Some(euid) = proc.effective_user_id() {
                        let euid_str = euid.to_string();
                        let username = euid_str.parse::<u32>().ok().and_then(get_username_from_uid);
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
                    gpu: gpu_processes::gpu_usage_by_pid()
                        .get(&pid)
                        .copied()
                        .unwrap_or(0.0),
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

                debug3!(
                    "Process details retrieved for PID {}: {}",
                    pid,
                    details.name
                );
                Ok(details)
            } else {
                Err(format!("Process with PID {} not found", pid))
            }
        }
        Err(_) => Err("System lock unavailable".to_string()),
    }
}

/// Get username from UID using getpwuid
fn get_username_from_uid(uid: u32) -> Option<String> {
    unsafe {
        use libc::{c_char, getpwuid, passwd};
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
    let output = Command::new("kill").arg("-9").arg(pid.to_string()).output();

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
        }
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
    debug3!(
        "get_metrics_history() called with time_range_seconds={}, max_display_points={:?}",
        time_range_seconds,
        max_display_points
    );

    // Block briefly. try_lock dropped the open-window seed whenever the
    // sampler held the mutex (including the history.json write).
    let history_opt = match METRICS_HISTORY.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(history) = history_opt.as_ref() {
        let points = history.query(time_range_seconds, max_display_points);
        let oldest = history.oldest_timestamp();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        debug3!(
            "get_metrics_history: returning {} points, oldest_ts={:?}, newest_ts={}",
            points.len(),
            oldest,
            now
        );

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
}
