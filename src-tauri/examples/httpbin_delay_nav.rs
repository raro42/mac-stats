//! One-shot navigate to httpbin delay endpoint (same stack as BROWSER_NAVIGATE).
//! Run: `cd src-tauri && cargo run --release --example httpbin_delay_nav`

use mac_stats::browser_agent::navigate_and_get_state;

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new("warn,mac_stats=info").unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("warn")
            }),
        )
        .try_init();

    match navigate_and_get_state("https://httpbin.org/delay/5") {
        Ok(state) => println!("ok (state {} chars)", state.len()),
        Err(e) => {
            eprintln!("navigate failed: {e}");
            std::process::exit(1);
        }
    }
}
