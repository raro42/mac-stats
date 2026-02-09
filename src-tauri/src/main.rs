// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mac_stats")]
#[command(about = "macOS system statistics menu bar app")]
#[command(long_about = "A lightweight system monitor for macOS that displays real-time CPU, GPU, RAM, and disk usage in the menu bar with minimal CPU overhead.")]
struct Args {
    /// Verbose output level (use -v, -vv, or -vvv for increasing verbosity)
    #[arg(short = 'v', action = clap::ArgAction::Count, help = "Verbosity level: -v (minimal), -vv (moderate), -vvv (maximum)")]
    verbose: u8,
    
    /// Open CPU window directly at startup (for testing)
    #[arg(long = "cpu", help = "Open the CPU details window immediately when the app starts")]
    open_cpu: bool,
    
    /// Open CPU window directly at startup (alternative to --cpu)
    #[arg(long = "openwindow", help = "Open the CPU details window immediately when the app starts (same as --cpu)")]
    open_window: bool,
    
    /// Enable detailed frequency logging for debugging
    #[arg(long = "frequency", help = "Enable detailed logging of CPU frequency readings from IOReport")]
    frequency: bool,
    
    /// Enable detailed power usage logging for debugging
    #[arg(long = "power-usage", help = "Enable detailed logging of CPU and GPU power consumption")]
    power_usage: bool,
    
    /// Print changelog to console and exit
    #[arg(long = "changelog", help = "Display the application changelog and exit")]
    changelog: bool,
    
    /// Task operations (add, list, show, status, remove, assign, append). Run and exit without starting the app.
    #[command(subcommand)]
    task: Option<mac_stats::task::cli::TaskCmd>,
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
    
    // Set power usage logging flag
    mac_stats::set_power_usage_logging(args.power_usage);
    
    // If --changelog flag is set, test changelog functionality
    if args.changelog {
        use mac_stats::get_changelog;
        match get_changelog() {
            Ok(changelog) => {
                println!("Changelog ({} bytes):\n{}", changelog.len(), changelog);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error getting changelog: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    // If --task subcommand is used, run task CLI and exit
    if let Some(cmd) = args.task {
        let code = match mac_stats::task::cli::run(cmd) {
            Ok(()) => 0,
            Err(c) => c,
        };
        std::process::exit(code);
    }
    
    // If --cpu or --openwindow flag is set, open window directly after a short delay
    if args.open_cpu || args.open_window {
        mac_stats::run_with_cpu_window()
    } else {
        mac_stats::run()
    }
}
