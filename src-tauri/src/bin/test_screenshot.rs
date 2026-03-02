//! Quick test: take_screenshot(URL) and print path. Run: cargo run --bin test_screenshot -- https://www.amvara.de

fn main() {
    mac_stats::init_tracing(2, None);
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://www.amvara.de".to_string());
    tracing::info!("Screenshot test: {}", url);
    match mac_stats::browser_agent::take_screenshot(&url) {
        Ok(path) => {
            println!("OK: {}", path.display());
            tracing::info!("Screenshot saved: {:?}", path);
        }
        Err(e) => {
            eprintln!("FAIL: {}", e);
            std::process::exit(1);
        }
    }
}
