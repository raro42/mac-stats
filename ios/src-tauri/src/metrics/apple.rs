//! Foundation and UIKit APIs: thermal state, Low Power Mode, storage,
//! battery and device info.
//!
//! Thermal state and Low Power Mode are adapted from the Mac app
//! (`src-tauri/src/ffi/objc.rs` at the repo root).

use objc2_foundation::{
    NSArray, NSNumber, NSProcessInfo, NSString, NSURLVolumeAvailableCapacityForImportantUsageKey,
    NSURLVolumeTotalCapacityKey, NSURL,
};
use serde::Serialize;

/// `NSProcessInfo.thermalState`. iOS does not expose temperatures in °C.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Thermal {
    Nominal,
    Fair,
    Serious,
    Critical,
    Unknown,
}

pub fn thermal_state() -> Thermal {
    // NSProcessInfoThermalState is a newtype over NSInteger, not a Rust enum.
    match NSProcessInfo::processInfo().thermalState().0 {
        0 => Thermal::Nominal,
        1 => Thermal::Fair,
        2 => Thermal::Serious,
        3 => Thermal::Critical,
        _ => Thermal::Unknown,
    }
}

pub fn low_power_mode() -> bool {
    NSProcessInfo::processInfo().isLowPowerModeEnabled()
}

pub fn physical_memory() -> u64 {
    NSProcessInfo::processInfo().physicalMemory()
}

pub fn active_cores() -> usize {
    NSProcessInfo::processInfo().activeProcessorCount()
}

pub fn os_version() -> String {
    let v = NSProcessInfo::processInfo().operatingSystemVersion();
    if v.patchVersion > 0 {
        format!("{}.{}.{}", v.majorVersion, v.minorVersion, v.patchVersion)
    } else {
        format!("{}.{}", v.majorVersion, v.minorVersion)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    pub total: u64,
    /// Space for "important usage": includes purgeable data iOS can free,
    /// which is what Settings shows.
    pub available: u64,
}

/// Capacity of the volume that contains `path` (the app container on iOS).
pub fn storage(path: &str) -> Option<Storage> {
    let url = NSURL::fileURLWithPath(&NSString::from_str(path));
    let (total_key, available_key) = unsafe {
        (
            NSURLVolumeTotalCapacityKey,
            NSURLVolumeAvailableCapacityForImportantUsageKey,
        )
    };
    let keys = NSArray::from_slice(&[total_key, available_key]);
    let values = url.resourceValuesForKeys_error(&keys).ok()?;
    let number = |key| -> Option<i64> {
        values
            .objectForKey(key)?
            .downcast::<NSNumber>()
            .ok()
            .map(|n| n.longLongValue())
    };
    let total = number(total_key)?;
    let available = number(available_key)?;
    if total <= 0 {
        return None;
    }
    Some(Storage {
        total: total as u64,
        available: available.clamp(0, total) as u64,
    })
}

#[cfg_attr(not(target_os = "ios"), allow(dead_code))] // only constructed on iOS
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BatteryState {
    Unknown,
    Unplugged,
    Charging,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    /// 0.0–1.0
    pub level: f32,
    pub state: BatteryState,
}

/// Reads the battery with `UIDevice`. Only works on the main thread; returns
/// `None` on the simulator (level −1) or when called from another thread.
#[cfg(target_os = "ios")]
pub fn battery_on_main_thread() -> Option<Battery> {
    use objc2_ui_kit::{UIDevice, UIDeviceBatteryState};

    let mtm = objc2::MainThreadMarker::new()?;
    let device = UIDevice::currentDevice(mtm);
    if !device.isBatteryMonitoringEnabled() {
        device.setBatteryMonitoringEnabled(true);
    }
    let level = device.batteryLevel();
    if level < 0.0 {
        return None;
    }
    let state = match device.batteryState() {
        UIDeviceBatteryState::Unplugged => BatteryState::Unplugged,
        UIDeviceBatteryState::Charging => BatteryState::Charging,
        UIDeviceBatteryState::Full => BatteryState::Full,
        _ => BatteryState::Unknown,
    };
    Some(Battery { level, state })
}

fn sysctl_string(name: &str) -> Option<String> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut len: libc::size_t = 0;
    let rc = unsafe {
        libc::sysctlbyname(cname.as_ptr(), std::ptr::null_mut(), &mut len, std::ptr::null_mut(), 0)
    };
    if rc != 0 || len == 0 {
        return None;
    }
    let mut buf = vec![0u8; len];
    let rc = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            buf.as_mut_ptr().cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if rc != 0 {
        return None;
    }
    buf.truncate(len);
    let text = String::from_utf8_lossy(&buf);
    Some(text.trim_end_matches('\0').to_string())
}

/// Model identifier (e.g. `iPhone13,3`) and whether it is the simulator.
pub fn model_identifier() -> (String, bool) {
    if let Ok(sim) = std::env::var("SIMULATOR_MODEL_IDENTIFIER") {
        return (sim, true);
    }
    let id = sysctl_string("hw.machine").unwrap_or_else(|| "desconocido".into());
    (id, false)
}

/// Marketing name of known iPhones; the identifier if it is not in the table.
pub fn marketing_name(identifier: &str) -> String {
    let name = match identifier {
        "iPhone13,1" => "iPhone 12 mini",
        "iPhone13,2" => "iPhone 12",
        "iPhone13,3" => "iPhone 12 Pro",
        "iPhone13,4" => "iPhone 12 Pro Max",
        "iPhone14,4" => "iPhone 13 mini",
        "iPhone14,5" => "iPhone 13",
        "iPhone14,2" => "iPhone 13 Pro",
        "iPhone14,3" => "iPhone 13 Pro Max",
        "iPhone14,6" => "iPhone SE (3.ª gen.)",
        "iPhone14,7" => "iPhone 14",
        "iPhone14,8" => "iPhone 14 Plus",
        "iPhone15,2" => "iPhone 14 Pro",
        "iPhone15,3" => "iPhone 14 Pro Max",
        "iPhone15,4" => "iPhone 15",
        "iPhone15,5" => "iPhone 15 Plus",
        "iPhone16,1" => "iPhone 15 Pro",
        "iPhone16,2" => "iPhone 15 Pro Max",
        "iPhone17,1" => "iPhone 16 Pro",
        "iPhone17,2" => "iPhone 16 Pro Max",
        "iPhone17,3" => "iPhone 16",
        "iPhone17,4" => "iPhone 16 Plus",
        "iPhone17,5" => "iPhone 16e",
        other => return other.to_string(),
    };
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_and_unknown_models() {
        assert_eq!(marketing_name("iPhone13,3"), "iPhone 12 Pro");
        assert_eq!(marketing_name("iPhone99,9"), "iPhone99,9");
    }

    #[test]
    fn foundation_readings_work_on_the_host() {
        assert!(physical_memory() > 0);
        assert!(active_cores() > 0);
        assert_ne!(thermal_state(), Thermal::Unknown);
        let home = std::env::var("HOME").unwrap();
        let s = storage(&home).expect("NSURL resource values");
        assert!(s.total > 0 && s.available <= s.total);
    }
}
