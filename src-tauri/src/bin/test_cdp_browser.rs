//! Test binary: CDP browser agent — connect to Chrome, open amvara.de, extract and log telephone number.
//!
//! Prereq: Chrome running with --remote-debugging-port=9222, e.g.:
//!   /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222
//! Or run this binary; it will try to launch Chrome if not connected.

use std::process::{Child, Command, Stdio};
use std::time::Duration;

fn main() {
    mac_stats::init_tracing(2, None);
    tracing::info!("CDP browser test: starting");

    // Try to get WS URL; if failed, try launching Chrome
    let port = 9222u16;
    let ws_ok = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .and_then(|c| c.get(format!("http://127.0.0.1:{}/json/version", port)).send())
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let _chrome_guard = if !ws_ok {
        tracing::info!("CDP browser test: no Chrome on port {}, attempting to launch Chrome", port);
        #[cfg(target_os = "macos")]
        let chrome_path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
        #[cfg(not(target_os = "macos"))]
        let chrome_path = "google-chrome"; // Linux fallback
        let child = Command::new(chrome_path)
            .arg("--remote-debugging-port=9222")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        match child {
            Ok(c) => {
                std::thread::sleep(Duration::from_secs(3));
                Some(c)
            }
            Err(e) => {
                tracing::error!("CDP browser test: failed to launch Chrome: {}", e);
                tracing::error!("Please start Chrome manually: {} --remote-debugging-port=9222", chrome_path);
                std::process::exit(1);
            }
        }
    } else {
        None::<Child>
    };

    let url = "https://www.amvara.de/#contact";
    let result = if ws_ok {
        mac_stats::browser_agent::fetch_page_and_extract_phones(port, url)
    } else {
        tracing::info!("CDP browser test: using headless_chrome-launched Chrome");
        mac_stats::browser_agent::launch_browser_and_extract_phones(url)
    };

    match result {
        Ok(phones) => {
            if phones.is_empty() {
                tracing::warn!("CDP browser test: no telephone numbers extracted (try URL with #contact for SPA sites)");
            } else {
                tracing::info!("CDP browser test: SUCCESS — amvara.de telephone number(s): {:?}", phones);
            }
        }
        Err(e) => {
            tracing::error!("CDP browser test: FAILED — {}", e);
            std::process::exit(1);
        }
    }
}
