//! Exercises **BROWSER_GO_BACK**, **BROWSER_GO_FORWARD**, and **BROWSER_RELOAD** via the same
//! `browser_agent` entry points as the menu-bar app (including post-navigate tab reconciliation).
//!
//! Flow:
//! 1. `BROWSER_NAVIGATE`: https://example.com/
//! 2. `BROWSER_NAVIGATE`: second path on same host (builds history)
//! 3. `BROWSER_GO_BACK`
//! 4. `BROWSER_GO_FORWARD`
//! 5. `BROWSER_RELOAD` (normal cache)
//!
//! Run (from repo `src-tauri/`): `cargo run --example example_com_history_reload_smoke`
//!
//! Prereq: Chromium with CDP on the configured port (default **9222**), same as other `examples/`.
//! Use **`-vv`** on the mac-stats binary or `RUST_LOG=mac_stats=debug` when running this example
//! to see **`post-navigate tab reconciliation`** lines in the process output.

use mac_stats::browser_agent::{
    go_back, go_forward, navigate_and_get_state, reload_current_tab,
};

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new("warn,mac_stats=debug")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .try_init();

    println!("Step 1: BROWSER_NAVIGATE: https://example.com/");
    let s1 = navigate_and_get_state("https://example.com/").unwrap_or_else(|e| {
        eprintln!("navigate failed: {e}");
        std::process::exit(1);
    });
    println!("--- ok ---\n{s1}\n");

    println!("Step 2: BROWSER_NAVIGATE: https://example.com/history-smoke-path (history)");
    let s2 = navigate_and_get_state("https://example.com/history-smoke-path").unwrap_or_else(|e| {
        eprintln!("second navigate failed: {e}");
        std::process::exit(1);
    });
    println!("--- ok ---\n{s2}\n");

    println!("Step 3: BROWSER_GO_BACK");
    let s3 = go_back().unwrap_or_else(|e| {
        eprintln!("go_back failed: {e}");
        std::process::exit(1);
    });
    println!("--- ok ---\n{s3}\n");

    println!("Step 4: BROWSER_GO_FORWARD");
    let s4 = go_forward().unwrap_or_else(|e| {
        eprintln!("go_forward failed: {e}");
        std::process::exit(1);
    });
    println!("--- ok ---\n{s4}\n");

    println!("Step 5: BROWSER_RELOAD (ignore_cache=false)");
    let s5 = reload_current_tab(false).unwrap_or_else(|e| {
        eprintln!("reload failed: {e}");
        std::process::exit(1);
    });
    println!("--- ok ---\n{s5}\n");

    println!("DONE: history + reload smoke completed");
}
