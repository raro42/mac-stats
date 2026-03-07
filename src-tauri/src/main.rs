// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mac_stats")]
#[command(about = "macOS system statistics menu bar app")]
#[command(
    long_about = "A lightweight system monitor for macOS that displays real-time CPU, GPU, RAM, and disk usage in the menu bar with minimal CPU overhead."
)]
struct Args {
    /// Verbose output level (default: 2 = -vv). Use -v for minimal, -vvv for maximum.
    #[arg(short = 'v', action = clap::ArgAction::Count, help = "Verbosity: default -vv (moderate). -v = minimal, -vvv = maximum")]
    verbose: u8,

    /// Open CPU window directly at startup (for testing)
    #[arg(
        long = "cpu",
        help = "Open the CPU details window immediately when the app starts"
    )]
    open_cpu: bool,

    /// Open CPU window directly at startup (alternative to --cpu)
    #[arg(
        long = "openwindow",
        help = "Open the CPU details window immediately when the app starts (same as --cpu)"
    )]
    open_window: bool,

    /// Enable detailed frequency logging for debugging
    #[arg(
        long = "frequency",
        help = "Enable detailed logging of CPU frequency readings from IOReport"
    )]
    frequency: bool,

    /// Enable detailed power usage logging for debugging
    #[arg(
        long = "power-usage",
        help = "Enable detailed logging of CPU and GPU power consumption"
    )]
    power_usage: bool,

    /// Print changelog to console and exit
    #[arg(
        long = "changelog",
        help = "Display the application changelog and exit"
    )]
    changelog: bool,

    /// Subcommands: task (add, list, show, ...) or agent (test). Run and exit without starting the app.
    #[command(subcommand)]
    cmd: Option<MainCmd>,
}

#[derive(clap::Subcommand, Debug)]
enum MainCmd {
    /// Task operations (add, list, show, status, remove, assign, append)
    #[command(subcommand)]
    Task(mac_stats::task::cli::TaskCmd),
    /// Agent operations (test with testing.md)
    #[command(subcommand)]
    Agent(AgentCmd),
    /// Discord: send a message to a channel (uses bot token from config)
    #[command(subcommand)]
    Discord(DiscordCmd),
}

#[derive(clap::Subcommand, Debug)]
enum DiscordCmd {
    /// Post a message to a Discord channel. Channel ID from Discord (e.g. right-click channel → Copy ID).
    Send {
        #[arg(help = "Discord channel ID (number)")]
        channel_id: u64,
        #[arg(help = "Message text to send")]
        message: String,
    },
    /// Run the same Ollama+tools pipeline as a Discord DM (headless browser, no Discord). For testing.
    RunOllama {
        #[arg(help = "Question to run (same flow as Discord DM)")]
        question: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum AgentCmd {
    /// Run agent with prompts from testing.md. Logs to ~/.mac-stats/debug.log; use -vv.
    Test {
        /// Agent selector: id, slug, or name (e.g. 001, senior-coder, General Assistant)
        selector: String,
        /// Path to a markdown file with test prompts. If omitted, uses ~/.mac-stats/agents/agent-<id>/testing.md
        path: Option<PathBuf>,
    },
}

fn main() {
    let args = Args::parse();

    // Set verbosity level (0-3). Default 2 (-vv) so logs are visible when no -v flags given.
    let verbosity = if args.verbose > 3 {
        3
    } else if args.verbose > 0 {
        args.verbose
    } else {
        2
    };

    // Initialize tracing (structured logging) using config module
    use mac_stats::config::Config;
    Config::ensure_log_directory().ok(); // Create log directory if needed
    let log_path = Config::log_file_path();
    mac_stats::init_tracing(verbosity, Some(log_path.clone()));

    // Also set legacy verbosity for compatibility during migration
    mac_stats::set_verbosity(verbosity);

    tracing::info!("mac-stats: verbosity {} (logs: {:?})", verbosity, log_path);

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

    // If a subcommand is used, run it and exit
    if let Some(cmd) = args.cmd {
        let code = match cmd {
            MainCmd::Task(task_cmd) => match mac_stats::task::cli::run(task_cmd) {
                Ok(()) => 0,
                Err(c) => c,
            },
            MainCmd::Agent(AgentCmd::Test { selector, path }) => {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async {
                    mac_stats::agents::cli::run_agent_test(&selector, path.as_deref())
                        .await
                        .map(|_| 0)
                        .unwrap_or_else(|c| c)
                })
            }
            MainCmd::Discord(DiscordCmd::Send {
                channel_id,
                message,
            }) => {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async {
                    match mac_stats::discord::send_message_to_channel(channel_id, &message).await {
                        Ok(()) => {
                            println!("Sent to channel {}", channel_id);
                            0
                        }
                        Err(e) => {
                            eprintln!("Discord send failed: {}", e);
                            1
                        }
                    }
                })
            }
            MainCmd::Discord(DiscordCmd::RunOllama { question }) => {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async {
                    mac_stats::config::Config::ensure_defaults();
                    mac_stats::ensure_ollama_agent_ready_at_startup().await;
                    match mac_stats::answer_with_ollama_and_fetch(
                        &question, None, None, None, None, None, None, None, None,
                        true, // allow_schedule
                        None, false, // escalation
                        true,  // retry_on_verification_no
                        true,  // from_remote: headless browser
                        None,  // attachment_images_base64
                        None,  // discord_intermediate
                        false, // is_verification_retry
                        None,  // original_user_request
                        None,  // success_criteria_override
                    )
                    .await
                    {
                        Ok(reply) => {
                            println!(
                                "Reply ({} chars):\n{}",
                                reply.text.chars().count(),
                                reply.text
                            );
                            for p in &reply.attachment_paths {
                                println!("Attachment: {}", p.display());
                            }
                            0
                        }
                        Err(e) => {
                            eprintln!("Run failed: {}", e);
                            1
                        }
                    }
                })
            }
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
