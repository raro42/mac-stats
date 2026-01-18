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
    
    /// Enable detailed frequency logging for debugging
    #[arg(long = "frequency")]
    frequency: bool,
}

fn main() {
    let args = Args::parse();
    
    // Set verbosity level (0-3)
    let verbosity = if args.verbose > 3 { 3 } else { args.verbose };
    
    // Initialize tracing (structured logging) using config module
    use mac_stats::config::Config;
    Config::ensure_log_directory().ok(); // Create log directory if needed
    let log_path = Config::log_file_path();
    mac_stats::init_tracing(verbosity, Some(log_path));
    
    // Also set legacy verbosity for compatibility during migration
    mac_stats::set_verbosity(verbosity);
    
    // Set frequency logging flag
    mac_stats::set_frequency_logging(args.frequency);
    
    // If -cpu flag is set, open window directly after a short delay
    if args.open_cpu {
        mac_stats::run_with_cpu_window()
    } else {
        mac_stats::run()
    }
}
