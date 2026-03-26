//! Browser smoke: navigate to example.com, then `search_page_text("Example")` twice (same as two
//! `BROWSER_SEARCH_PAGE: Example` calls after `BROWSER_NAVIGATE`).
//! Run: `cd src-tauri && cargo run --example example_com_search_twice`

use mac_stats::browser_agent::{self, navigate_and_get_state};

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new("warn,mac_stats=info").unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("warn")
            }),
        )
        .try_init();

    println!("(1) BROWSER_NAVIGATE: https://example.com");
    match navigate_and_get_state("https://example.com") {
        Ok(state) => println!("--- navigate ok (state {} chars) ---", state.len()),
        Err(e) => {
            eprintln!("navigate failed: {e}");
            std::process::exit(1);
        }
    }

    println!("\n(2) First: BROWSER_SEARCH_PAGE: Example");
    match browser_agent::search_page_text("Example", None) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("search failed: {e}");
            std::process::exit(1);
        }
    }

    println!("\n(3) Second: BROWSER_SEARCH_PAGE: Example");
    match browser_agent::search_page_text("Example", None) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("search failed: {e}");
            std::process::exit(1);
        }
    }
}
