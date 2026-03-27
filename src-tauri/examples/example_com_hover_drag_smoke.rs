//! Multi-step browser smoke (maps to agent tools):
//! 1. BROWSER_NAVIGATE: https://example.com/
//! 2. BROWSER_HOVER: 1 (Learn more)
//! 3. BROWSER_DRAG: 1 1 (drag from element 1 to itself)
//!
//! Run: `cd src-tauri && cargo run --example example_com_hover_drag_smoke`
//!
//! Requires Chrome with CDP reachable per mac-stats browser config (same as other `examples/`).

use mac_stats::browser_agent::{drag_by_indices, hover_by_index, navigate_and_get_state};

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new("warn,mac_stats=info")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .try_init();

    println!("Step 1: BROWSER_NAVIGATE: https://example.com/");
    match navigate_and_get_state("https://example.com/") {
        Ok(state) => {
            println!("--- navigate ok ---\n{state}\n");
            if !state.contains("Learn more") && !state.contains("learn more") {
                eprintln!("warning: page state may not list expected link text");
            }
        }
        Err(e) => {
            eprintln!("navigate failed: {e}");
            std::process::exit(1);
        }
    }

    println!("Step 2: BROWSER_HOVER: 1");
    match hover_by_index(1) {
        Ok(state) => println!("--- hover ok ---\n{state}\n"),
        Err(e) => {
            eprintln!("hover failed: {e}");
            std::process::exit(1);
        }
    }

    println!("Step 3: BROWSER_DRAG: 1 1");
    match drag_by_indices(1, 1) {
        Ok(state) => println!("--- drag ok ---\n{state}\n"),
        Err(e) => {
            eprintln!("drag failed: {e}");
            std::process::exit(1);
        }
    }

    println!("Step 4: DONE: success");
}
