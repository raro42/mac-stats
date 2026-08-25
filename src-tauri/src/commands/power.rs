//! Battery / Low Power Mode controls for the CPU window.

use std::path::Path;
use std::process::Command;
use tracing::warn;

#[derive(serde::Serialize)]
pub struct LowPowerModeToggleResult {
    pub enabled: bool,
}

fn pmset_path() -> &'static str {
    if Path::new("/usr/bin/pmset").is_file() {
        "/usr/bin/pmset"
    } else {
        "/usr/sbin/pmset"
    }
}

/// Toggle Apple Low Power Mode. Uses `powermode` on recent macOS (0=auto, 1=low)
/// and `lowpowermode` on older laptops. macOS shows an admin password dialog.
#[tauri::command]
pub fn toggle_low_power_mode() -> Result<LowPowerModeToggleResult, String> {
    let current = crate::ffi::objc::read_process_low_power_mode();
    let enable = !current;
    let powermode = u8::from(enable);
    let lowpowermode = u8::from(enable);
    let pmset = pmset_path();
    let shell = format!(
        "{pmset} -a powermode {powermode}; {pmset} -a lowpowermode {lowpowermode} 2>/dev/null || true"
    );
    let script = format!("do shell script {shell:?} with administrator privileges");
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Could not run osascript: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        warn!("toggle_low_power_mode failed: {msg}");
        return Err(if msg.is_empty() {
            "Low Power Mode was not changed (admin password cancelled or denied)".to_string()
        } else {
            msg.to_string()
        });
    }
    let enabled = crate::ffi::objc::read_process_low_power_mode();
    Ok(LowPowerModeToggleResult { enabled })
}
