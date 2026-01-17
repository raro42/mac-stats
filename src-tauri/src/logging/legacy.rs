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

// Write structured log entry (JSON) to log file
pub fn write_structured_log(location: &str, message: &str, data: &serde_json::Value, hypothesis_id: &str) {
    // CRITICAL: Only write structured logs if verbosity is >= 2 (debug level)
    // This prevents excessive logging and CPU usage in normal operation
    if VERBOSITY.load(Ordering::Relaxed) < 2 {
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
    
    // Also write human-readable version to terminal (only if verbosity >= 2)
    eprintln!("[DEBUG] {}: {} (hypothesis: {})", location, message, hypothesis_id);
}

#[macro_export]
macro_rules! debug {
    ($level:expr, $($arg:tt)*) => {
        {
            use std::sync::atomic::Ordering;
            // Check legacy verbosity for backward compatibility
            if $crate::logging::VERBOSITY.load(Ordering::Relaxed) >= $level {
                // Use tracing macros directly
                let message = format!($($arg)*);
                match $level {
                    1 => tracing::info!("{}", message),
                    2 => tracing::debug!("{}", message),
                    3 => tracing::trace!("{}", message),
                    _ => {
                        let level_str = "LOG";
                        $crate::logging::write_log_entry(level_str, &message);
                    }
                }
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
