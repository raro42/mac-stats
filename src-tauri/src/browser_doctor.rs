//! Operator-facing CDP readiness diagnostics (OpenClaw-style `doctor-browser` analogue).
//!
//! Used by `mac_stats --browser-doctor` and optional rate-limited startup logging.

use crate::config::Config;
use serde_json::Value;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TRUNCATE_DISPLAY: usize = 72;
const STARTUP_HINT_COOLDOWN: Duration = Duration::from_secs(60 * 60);

static LAST_STARTUP_CDP_HINT_AT: Mutex<Option<Instant>> = Mutex::new(None);

fn truncate_display(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() <= TRUNCATE_DISPLAY {
        t.to_string()
    } else {
        let mut out: String = t.chars().take(TRUNCATE_DISPLAY).collect();
        out.push('…');
        out
    }
}

fn cdp_discovery_user_agent() -> String {
    format!("mac-stats/{}", env!("CARGO_PKG_VERSION"))
}

/// `GET /json/version` on loopback using the same HTTP timeout as production CDP discovery
/// ([`Config::browser_cdp_http_timeout_secs`], `no_proxy`, identifiable User-Agent).
pub fn probe_loopback_json_version() -> Result<Value, String> {
    let port = Config::browser_cdp_port();
    let timeout = Duration::from_secs(Config::browser_cdp_http_timeout_secs());
    let url = format!("http://127.0.0.1:{}/json/version", port);
    let user_agent = cdp_discovery_user_agent();
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .no_proxy()
        .user_agent(user_agent.as_str())
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("request to {}: {}", url, e))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("{} returned HTTP {}", url, status));
    }
    resp.json().map_err(|e| format!("JSON from {}: {}", url, e))
}

fn print_failure_hints(port: u16) {
    eprintln!();
    eprintln!("CDP probe failed. Try:");
    eprintln!(
        "  • Start Chromium with --remote-debugging-port={} (same as browserCdpPort).",
        port
    );
    eprintln!("  • Open chrome://inspect/#remote-debugging (Google Chrome).");
    eprintln!("  • Brave / Microsoft Edge: brave://inspect / edge://inspect — same remote-debugging idea.");
    eprintln!("  • The first DevTools or automation attach may require user consent if macOS or the browser shows a security prompt.");
    eprintln!("  • Config and env overrides: ~/.mac-stats/config.json — see docs/029_browser_automation.md.");
    eprintln!("  • SSRF and navigation rules are documented there (not duplicated here).");
}

/// Print diagnostics to stdout/stderr. Returns **0** if `/json/version` returns 2xx and parses as JSON; **1** otherwise.
pub fn run_browser_doctor_stdio() -> i32 {
    let port = Config::browser_cdp_port();
    let http_secs = Config::browser_cdp_http_timeout_secs();
    let ws_secs = Config::browser_cdp_ws_connect_timeout_secs();
    let vw = Config::browser_viewport_width();
    let vh = Config::browser_viewport_height();
    let tools_on = Config::browser_tools_enabled();
    let chrome_path = Config::browser_chromium_executable_path();
    let chrome_explicit = Config::browser_chromium_executable_configured();
    let user_data = Config::browser_chromium_user_data_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unset — default profile behaviour)".to_string());

    println!("mac-stats browser diagnostics (BROWSER_* / CDP)");
    println!("──────────────────────────────────────────────");
    println!("  browserToolsEnabled:            {}", tools_on);
    println!("  browserCdpPort:                 {}", port);
    println!("  browserCdpHttpTimeoutSecs:      {}", http_secs);
    println!("  browserCdpWsConnectTimeoutSecs: {}", ws_secs);
    println!("  viewport (width × height):      {} × {}", vw, vh);
    match Config::browser_cdp_emulate_viewport_dimensions() {
        Some((ew, eh)) => println!(
            "  CDP emulate device metrics:     {} × {} (dsf={:.3}, mobile={})",
            ew,
            eh,
            Config::browser_cdp_emulate_device_scale_factor(),
            Config::browser_cdp_emulate_mobile()
        ),
        None => println!("  CDP emulate device metrics:     (off — set both browserCdpEmulateViewportWidth and Height)"),
    }
    match Config::browser_cdp_emulate_geolocation() {
        Some((lat, lon, acc)) => println!(
            "  CDP emulate geolocation:        lat={:.5} lon={:.5} acc={}",
            lat,
            lon,
            acc.map(|a| format!("{:.1} m", a))
                .unwrap_or_else(|| "(default)".to_string())
        ),
        None => {
            println!("  CDP emulate geolocation:        (off — set both Latitude and Longitude)")
        }
    }
    println!(
        "  chromium executable (resolved): {}",
        chrome_path.display()
    );
    println!("  browserChromiumExecutable set: {}", chrome_explicit);
    println!("  browserChromiumUserDataDir:     {}", user_data);
    println!("  headless vs visible Chrome:     chosen per agent run (e.g. remote/Discord often headless; UI chat often visible). See docs/029_browser_automation.md.");
    println!();

    match probe_loopback_json_version() {
        Ok(json) => {
            let has_ws = json
                .get("webSocketDebuggerUrl")
                .and_then(|v| v.as_str())
                .is_some();
            let browser = json
                .get("Browser")
                .and_then(|v| v.as_str())
                .map(truncate_display);
            let ua_or_proto = json
                .get("User-Agent")
                .or_else(|| json.get("Protocol-Version"))
                .and_then(|v| v.as_str())
                .map(truncate_display);

            println!(
                "CDP HTTP probe: OK — http://127.0.0.1:{}/json/version",
                port
            );
            println!("  webSocketDebuggerUrl present: {}", has_ws);
            if let Some(b) = &browser {
                println!("  Browser: {}", b);
            } else {
                println!("  Browser: (missing in JSON)");
            }
            if let Some(u) = &ua_or_proto {
                println!("  User-Agent / Protocol-Version: {}", u);
            } else {
                println!("  User-Agent / Protocol-Version: (missing in JSON)");
            }
            0
        }
        Err(e) => {
            eprintln!("CDP HTTP probe: FAILED — {}", e);
            print_failure_hints(port);
            1
        }
    }
}

/// If browser tools are enabled and loopback CDP is not reachable, log a single **info** line
/// (rate-limited to at most once per hour per process) pointing operators at `--browser-doctor`.
pub fn maybe_log_startup_cdp_unreachable() {
    if !Config::browser_tools_enabled() {
        return;
    }
    if probe_loopback_json_version().is_ok() {
        return;
    }
    let now = Instant::now();
    let mut guard = match LAST_STARTUP_CDP_HINT_AT.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(prev) = *guard {
        if now.duration_since(prev) < STARTUP_HINT_COOLDOWN {
            return;
        }
    }
    *guard = Some(now);
    drop(guard);

    let port = Config::browser_cdp_port();
    tracing::info!(
        target: "browser/doctor",
        "Browser CDP not reachable on 127.0.0.1:{} while browser tools are enabled. Start Chromium with --remote-debugging-port={} or run `mac_stats --browser-doctor` for details.",
        port,
        port
    );
}
