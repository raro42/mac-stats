use std::sync::atomic::{AtomicU8, Ordering};

// Debug verbosity level: 0 = none, 1 = -v, 2 = -vv, 3 = -vvv
// Make VERBOSITY accessible to macros
pub static VERBOSITY: AtomicU8 = AtomicU8::new(0);

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
// Internal function for write_log_entry - needs to be accessible to macros
#[allow(dead_code)]
pub fn write_log_entry(level_str: &str, message: &str) {
    let timestamp = format_timestamp();
    let log_line = format!("[{}] [{}] {}", timestamp, level_str, message);
    
    // Write to terminal (stderr)
    eprintln!("{}", log_line);
    
    // Write to log file using config module
    use crate::config::Config;
    let log_path = Config::log_file_path();
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", log_line);
    }
}

// Helper function to convert full file path to short format (e.g., "src-tauri/src/ui/status_bar.rs" -> "ui/status_bar.rs")
fn shorten_file_path(file_path: &str) -> String {
    // Remove common prefixes
    if let Some(stripped) = file_path.strip_prefix("src-tauri/src/") {
        stripped.to_string()
    } else if let Some(stripped) = file_path.strip_prefix("src/") {
        stripped.to_string()
    } else {
        // If no prefix matches, try to find the last "src/" and strip everything before it
        if let Some(pos) = file_path.rfind("src/") {
            file_path[pos + 4..].to_string()
        } else {
            file_path.to_string()
        }
    }
}

// Write structured log entry (JSON) to log file
// min_verbosity: minimum verbosity level required (1, 2, or 3)
pub fn write_structured_log_with_verbosity(location: &str, message: &str, data: &serde_json::Value, hypothesis_id: &str, min_verbosity: u8) {
    // Check verbosity level
    if VERBOSITY.load(Ordering::Relaxed) < min_verbosity {
        return;
    }
    
    let log_data = serde_json::json!({
        "location": location,
        "message": message,
        "data": data,
        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
        "sessionId": "debug-session",
        "runId": "run3",
        "hypothesisId": hypothesis_id
    });
    
    // Use config module for log file path
    use crate::config::Config;
    let log_path = Config::log_file_path();
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        if let Ok(json_str) = serde_json::to_string(&log_data) {
            let _ = writeln!(file, "{}", json_str);
        }
    }
    
    // Also write human-readable version to terminal
    let timestamp = format_timestamp();
    if hypothesis_id.is_empty() {
        eprintln!("[{}] [DEBUG] {}: {}", timestamp, location, message);
    } else {
        eprintln!("[{}] [DEBUG] {}: {} (hypothesis: {})", timestamp, location, message, hypothesis_id);
    }
}

// Write structured log entry (JSON) to log file
// Defaults to verbosity >= 2 for backward compatibility
pub fn write_structured_log(location: &str, message: &str, data: &serde_json::Value, hypothesis_id: &str) {
    write_structured_log_with_verbosity(location, message, data, hypothesis_id, 2);
}
    

// Internal helper function for debug macros to convert file path
pub fn shorten_file_path_internal(file_path: &str) -> String {
    shorten_file_path(file_path)
}

#[macro_export]
macro_rules! debug {
    ($level:expr, $($arg:tt)*) => {
        {
            use std::sync::atomic::Ordering;
            // Check legacy verbosity for backward compatibility
            if $crate::logging::VERBOSITY.load(Ordering::Relaxed) >= $level {
                let message = format!($($arg)*);
                // Extract file location from macro call site
                let file_path = file!();
                let location = $crate::logging::shorten_file_path_internal(file_path);
                // Use write_structured_log_with_verbosity with the appropriate level
                // For debug macros, use empty hypothesis_id
                $crate::logging::write_structured_log_with_verbosity(&location, &message, &serde_json::json!({}), "", $level);
            }
        }
    };
}

#[macro_export]
macro_rules! debug1 {
    ($($arg:tt)*) => { $crate::debug!(1, $($arg)*); };
}

#[macro_export]
macro_rules! debug2 {
    ($($arg:tt)*) => { $crate::debug!(2, $($arg)*); };
}

#[macro_export]
macro_rules! debug3 {
    ($($arg:tt)*) => { $crate::debug!(3, $($arg)*); };
}

pub fn set_verbosity(level: u8) {
    VERBOSITY.store(level, Ordering::Relaxed);
    if level > 0 {
        eprintln!("Debug verbosity level set to: {}", level);
    }
}
