// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mac_stats")]
#[command(about = "macOS system statistics menu bar app", long_about = None)]
struct Args {
    /// Verbose output (-v, -vv, -vvv)
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Open CPU window directly (for testing)
    #[arg(long = "cpu")]
    open_cpu: bool,
}

fn main() {
    let args = Args::parse();
    
    // Set verbosity level (0-3)
    let verbosity = if args.verbose > 3 { 3 } else { args.verbose };
    
    // Initialize tracing (structured logging)
    // TODO: Use config module for log path (Phase 3)
    let temp_log_path = std::env::var("HOME")
        .ok()
        .map(|home| std::path::PathBuf::from(home).join(".mac-stats").join("debug.log"))
        .or_else(|| Some(std::env::temp_dir().join("mac-stats-debug.log")));
    
    mac_stats_lib::init_tracing(verbosity, temp_log_path);
    
    // Also set legacy verbosity for compatibility during migration
    mac_stats_lib::set_verbosity(verbosity);
    
    // If -cpu flag is set, open window directly after a short delay
    if args.open_cpu {
        mac_stats_lib::run_with_cpu_window()
    } else {
        mac_stats_lib::run()
    }
}
