//! Structured subsystem health probes for startup diagnostics and the CPU window.
//!
//! Probes run with short timeouts and in parallel where possible; results are cached and logged once after a short post-start delay so Discord/Ollama can initialize.

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serenity::gateway::ConnectionStage;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    Ok,
    Degraded,
    Unavailable,
    NotConfigured,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    /// RFC3339 UTC
    pub checked_at: String,
}

static FEATURE_HEALTH_REPORT: Mutex<Vec<FeatureHealth>> = Mutex::new(Vec::new());

/// When the in-memory health report was last collected (for stale-while-revalidate).
static FEATURE_HEALTH_COLLECTED_AT: Mutex<Option<Instant>> = Mutex::new(None);

const HEALTH_REPORT_TTL: Duration = Duration::from_secs(5 * 60);

/// Last known-good Brave Search probe (avoid flashing Unavailable on transient API errors).
static BRAVE_LAST_OK: Mutex<Option<(Instant, FeatureHealth)>> = Mutex::new(None);

fn now_rfc3339_utc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn entry(name: &str, status: HealthStatus, message: Option<String>) -> FeatureHealth {
    FeatureHealth {
        name: name.to_string(),
        status,
        message,
        checked_at: now_rfc3339_utc(),
    }
}

fn chrome_binary_present() -> bool {
    let p = crate::config::Config::browser_chromium_executable_path();
    if p.is_absolute() {
        p.is_file()
    } else {
        true
    }
}

async fn probe_cdp_http_reachable() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let port = crate::config::Config::browser_cdp_port();
    let url = format!("http://127.0.0.1:{}/json/version", port);
    match tokio::time::timeout(PROBE_TIMEOUT, client.get(url).send()).await {
        Ok(Ok(resp)) => resp.status().is_success(),
        _ => false,
    }
}

fn model_in_tags(configured: &str, tag_names: &[String]) -> bool {
    let base = configured.split(':').next().unwrap_or(configured);
    tag_names
        .iter()
        .any(|n| n == configured || n.starts_with(&format!("{base}:")) || n == base)
}

async fn probe_ollama() -> FeatureHealth {
    let client_info = {
        let guard = match crate::commands::ollama_config::get_ollama_client().lock() {
            Ok(g) => g,
            Err(_) => {
                return entry(
                    "Ollama",
                    HealthStatus::Unavailable,
                    Some("client lock poisoned".into()),
                );
            }
        };
        guard.as_ref().map(|c| {
            (
                c.config.endpoint.clone(),
                c.config.model.clone(),
                c.config.api_key.clone(),
            )
        })
    };
    let Some((endpoint, model, api_key_acct)) = client_info else {
        return entry(
            "Ollama",
            HealthStatus::NotConfigured,
            Some("not configured".into()),
        );
    };

    let http = match reqwest::Client::builder().timeout(PROBE_TIMEOUT).build() {
        Ok(c) => c,
        Err(e) => {
            return entry(
                "Ollama",
                HealthStatus::Unavailable,
                Some(format!("HTTP client: {e}")),
            );
        }
    };
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    let mut req = http.get(url);
    if let Some(acc) = &api_key_acct {
        if let Ok(Some(key)) = crate::security::get_credential(acc) {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
    }
    let resp = match tokio::time::timeout(PROBE_TIMEOUT, req.send()).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            return entry(
                "Ollama",
                HealthStatus::Unavailable,
                Some(format!("request failed: {e}")),
            );
        }
        Err(_) => {
            return entry(
                "Ollama",
                HealthStatus::Unavailable,
                Some("probe timeout".into()),
            );
        }
    };
    if !resp.status().is_success() {
        return entry(
            "Ollama",
            HealthStatus::Unavailable,
            Some(format!("HTTP {}", resp.status())),
        );
    }
    let body: serde_json::Value = match tokio::time::timeout(PROBE_TIMEOUT, resp.json()).await {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            return entry(
                "Ollama",
                HealthStatus::Degraded,
                Some(format!("invalid JSON: {e}")),
            );
        }
        Err(_) => {
            return entry(
                "Ollama",
                HealthStatus::Degraded,
                Some("JSON parse timeout".into()),
            );
        }
    };
    let names: Vec<String> = body
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    if model_in_tags(&model, &names) {
        return entry(
            "Ollama",
            HealthStatus::Ok,
            Some(format!("reachable; model '{model}' available")),
        );
    }
    if names.is_empty() {
        return entry(
            "Ollama",
            HealthStatus::Degraded,
            Some("reachable but no models in /api/tags".into()),
        );
    }
    entry(
        "Ollama",
        HealthStatus::Degraded,
        Some(format!(
            "reachable; configured model '{model}' not listed ({} models)",
            names.len()
        )),
    )
}

fn probe_discord() -> FeatureHealth {
    if !crate::discord::discord_bot_token_configured() {
        return entry(
            "Discord",
            HealthStatus::NotConfigured,
            Some("no bot token (env, .config.env, or Keychain)".into()),
        );
    }

    let had_ready = crate::discord::discord_bot_gateway_ready();
    let stage = crate::discord::discord_last_shard_stage();
    let started = crate::discord::discord_gateway_client_started_at();

    if had_ready {
        if let Some(s) = stage {
            if s.is_connecting() {
                return entry(
                    "Discord",
                    HealthStatus::Degraded,
                    Some(
                        "token present; gateway reconnecting (handshake or resume in progress)"
                            .into(),
                    ),
                );
            }
            if s == ConnectionStage::Disconnected {
                return entry(
                    "Discord",
                    HealthStatus::Degraded,
                    Some("token present; gateway disconnected (reconnect expected)".into()),
                );
            }
        }
        return entry(
            "Discord",
            HealthStatus::Ok,
            Some("token present; gateway ready".into()),
        );
    }

    match stage {
        Some(s) if s.is_connecting() => entry(
            "Discord",
            HealthStatus::Degraded,
            Some("token present; gateway handshake in progress (Ready not received yet)".into()),
        ),
        Some(ConnectionStage::Connected) => entry(
            "Discord",
            HealthStatus::Degraded,
            Some("token present; connected to gateway, awaiting Ready".into()),
        ),
        Some(ConnectionStage::Disconnected) => {
            let recent = started
                .map(|t| t.elapsed() < Duration::from_secs(45))
                .unwrap_or(false);
            if recent {
                entry(
                    "Discord",
                    HealthStatus::Degraded,
                    Some("token present; gateway connecting".into()),
                )
            } else {
                entry(
                    "Discord",
                    HealthStatus::Degraded,
                    Some(
                        "token present; gateway not ready (connection stalled, failed, or reconnecting)"
                            .into(),
                    ),
                )
            }
        }
        Some(_) => entry(
            "Discord",
            HealthStatus::Degraded,
            Some("token present; gateway state uncertain (awaiting Ready)".into()),
        ),
        None => {
            if started
                .map(|t| t.elapsed() < Duration::from_secs(120))
                .unwrap_or(false)
            {
                entry(
                    "Discord",
                    HealthStatus::Degraded,
                    Some("token present; gateway client starting (Ready not received yet)".into()),
                )
            } else {
                entry(
                    "Discord",
                    HealthStatus::Degraded,
                    Some("token present; gateway not ready yet or connection failed".into()),
                )
            }
        }
    }
}

async fn probe_browser() -> FeatureHealth {
    let port = crate::config::Config::browser_cdp_port();
    let chrome = chrome_binary_present();
    let cdp = probe_cdp_http_reachable().await;
    match (chrome, cdp) {
        (true, true) => entry(
            "Browser (CDP)",
            HealthStatus::Ok,
            Some(format!(
                "Chromium binary found; port {} responds (json/version)",
                port
            )),
        ),
        (true, false) => entry(
            "Browser (CDP)",
            HealthStatus::Degraded,
            Some(format!(
                "Chromium binary present but nothing on port {} (CDP tools may use HTTP fallback)",
                port
            )),
        ),
        (false, true) => entry(
            "Browser (CDP)",
            HealthStatus::Degraded,
            Some(format!(
                "Configured Chromium path missing or unresolved; port {} responds",
                port
            )),
        ),
        (false, false) => entry(
            "Browser (CDP)",
            HealthStatus::Unavailable,
            Some(format!(
                "Chromium binary path not available; port {} not reachable",
                port
            )),
        ),
    }
}

async fn probe_brave() -> FeatureHealth {
    let Some(api_key) = crate::commands::brave::get_brave_api_key() else {
        return entry(
            "Brave Search",
            HealthStatus::NotConfigured,
            Some("no BRAVE_API_KEY".into()),
        );
    };

    match crate::commands::brave::ping_brave_search_api(&api_key).await {
        Ok(()) => {
            let h = entry(
                "Brave Search",
                HealthStatus::Ok,
                Some("API reachable".into()),
            );
            if let Ok(mut g) = BRAVE_LAST_OK.lock() {
                *g = Some((Instant::now(), h.clone()));
            }
            h
        }
        Err(err) => {
            if let Ok(g) = BRAVE_LAST_OK.lock() {
                if let Some((t, h)) = g.as_ref() {
                    if t.elapsed() < Duration::from_secs(5 * 60) {
                        tracing::warn!(
                            "feature_health: Brave Search API check failed; serving stale OK from {}s ago: {}",
                            t.elapsed().as_secs(),
                            err
                        );
                        return FeatureHealth {
                            name: "Brave Search".to_string(),
                            status: HealthStatus::Ok,
                            message: Some(format!(
                                "{} (stale; transient API error, last live check {}s ago)",
                                h.message.as_deref().unwrap_or("API reachable"),
                                t.elapsed().as_secs()
                            )),
                            checked_at: now_rfc3339_utc(),
                        };
                    }
                }
            }
            entry("Brave Search", HealthStatus::Unavailable, Some(err))
        }
    }
}

async fn probe_redmine() -> FeatureHealth {
    if !crate::redmine::is_configured() {
        return entry("Redmine", HealthStatus::NotConfigured, None);
    }
    let base = crate::redmine::get_redmine_url().unwrap_or_default();
    let key = crate::redmine::get_redmine_api_key().unwrap_or_default();
    let client = match reqwest::Client::builder().timeout(PROBE_TIMEOUT).build() {
        Ok(c) => c,
        Err(e) => {
            return entry(
                "Redmine",
                HealthStatus::Unavailable,
                Some(format!("HTTP client: {e}")),
            );
        }
    };
    let url = format!("{}/users/current.json", base.trim_end_matches('/'));
    let req = client.get(&url).header("X-Redmine-API-Key", &key).header(
        "User-Agent",
        format!("mac-stats/{}", crate::config::Config::version()),
    );
    match tokio::time::timeout(PROBE_TIMEOUT, req.send()).await {
        Ok(Ok(r)) if r.status().is_success() => {
            entry("Redmine", HealthStatus::Ok, Some("reachable".into()))
        }
        Ok(Ok(r)) => entry(
            "Redmine",
            HealthStatus::Degraded,
            Some(format!("HTTP {}", r.status())),
        ),
        Ok(Err(e)) => entry("Redmine", HealthStatus::Unavailable, Some(e.to_string())),
        Err(_) => entry("Redmine", HealthStatus::Unavailable, Some("timeout".into())),
    }
}

async fn probe_smc_blocking() -> FeatureHealth {
    let (tx, rx) = oneshot::channel::<Result<(), String>>();
    std::thread::spawn(move || {
        let out = std::thread::spawn(|| {
            macsmc::Smc::connect()
                .map(|_| ())
                .map_err(|e| format!("{e:?}"))
        })
        .join()
        .unwrap_or_else(|_| Err("SMC thread panicked".into()));
        let _ = tx.send(out);
    });
    match tokio::time::timeout(PROBE_TIMEOUT, rx).await {
        Ok(Ok(Ok(()))) => entry(
            "SMC (temperature)",
            HealthStatus::Ok,
            Some("SMC driver reachable".into()),
        ),
        Ok(Ok(Err(e))) => entry("SMC (temperature)", HealthStatus::Unavailable, Some(e)),
        Ok(Err(_)) => entry(
            "SMC (temperature)",
            HealthStatus::Unavailable,
            Some("probe cancelled".into()),
        ),
        Err(_) => entry(
            "SMC (temperature)",
            HealthStatus::Degraded,
            Some("probe timeout".into()),
        ),
    }
}

async fn probe_ioreport_blocking() -> FeatureHealth {
    let (tx, rx) = oneshot::channel::<bool>();
    std::thread::spawn(move || {
        let ok =
            std::thread::spawn(|| crate::ffi::ioreport::probe_cpu_performance_channels_available())
                .join()
                .unwrap_or(false);
        let _ = tx.send(ok);
    });
    match tokio::time::timeout(PROBE_TIMEOUT, rx).await {
        Ok(Ok(true)) => entry(
            "IOReport (CPU frequency)",
            HealthStatus::Ok,
            Some("CPU performance channels available".into()),
        ),
        Ok(Ok(false)) => entry(
            "IOReport (CPU frequency)",
            HealthStatus::Unavailable,
            Some("no CPU performance-state channels (expected on some configs)".into()),
        ),
        Ok(Err(_)) => entry(
            "IOReport (CPU frequency)",
            HealthStatus::Unavailable,
            Some("probe cancelled".into()),
        ),
        Err(_) => entry(
            "IOReport (CPU frequency)",
            HealthStatus::Degraded,
            Some("probe timeout".into()),
        ),
    }
}

fn probe_scheduler() -> FeatureHealth {
    let n = crate::scheduler::schedule_entry_count();
    entry(
        "Scheduler",
        HealthStatus::Ok,
        Some(format!("{n} task(s) loaded from schedules.json")),
    )
}

/// Run all probes (parallel async where applicable).
pub async fn collect_feature_health() -> Vec<FeatureHealth> {
    let ollama = probe_ollama();
    let redmine = probe_redmine();
    let browser = probe_browser();
    let brave = probe_brave();
    let (o, r, b, br) = tokio::join!(ollama, redmine, browser, brave);

    let smc = probe_smc_blocking();
    let ioreport = probe_ioreport_blocking();
    let (s, i) = tokio::join!(smc, ioreport);

    vec![o, probe_discord(), b, br, r, s, i, probe_scheduler()]
}

pub fn store_report(report: &[FeatureHealth]) {
    if let Ok(mut g) = FEATURE_HEALTH_REPORT.lock() {
        *g = report.to_vec();
    }
    if let Ok(mut t) = FEATURE_HEALTH_COLLECTED_AT.lock() {
        *t = Some(Instant::now());
    }
}

/// Log one info line per feature for `debug.log` scanning.
pub fn log_feature_health_summary(report: &[FeatureHealth]) {
    tracing::info!(
        "feature_health: ─── subsystem health ({} entries) ───",
        report.len()
    );
    for h in report {
        let msg = h.message.as_deref().unwrap_or("-");
        tracing::info!(
            "feature_health: {:24} {:12} {}",
            h.name,
            format!("{:?}", h.status),
            msg
        );
    }
    tracing::info!("feature_health: ─── end subsystem health ───");
}

/// Spawn delayed health collection after startup (Discord gateway + Ollama need a moment).
///
/// Uses a dedicated thread + `current_thread` runtime so probes run reliably (Tauri's async
/// runtime may not execute tasks scheduled from `.setup()` until after the loop is up).
pub fn spawn_startup_feature_health_probe() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(2));
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!(
                    "feature_health: could not build runtime for startup probe: {}",
                    e
                );
                return;
            }
        };
        tracing::info!("feature_health: running startup probes (parallel, bounded timeout)");
        let report = rt.block_on(collect_feature_health());
        log_feature_health_summary(&report);
        store_report(&report);
    });
}

/// Last collected report, or fresh collection if cache empty or `refresh` is true.
#[tauri::command]
pub async fn get_feature_health(refresh: Option<bool>) -> Result<Vec<FeatureHealth>, String> {
    let need_refresh = refresh.unwrap_or(false);
    let cached_empty = FEATURE_HEALTH_REPORT
        .lock()
        .map(|g| g.is_empty())
        .unwrap_or(true);
    if need_refresh || cached_empty {
        tracing::info!("feature_health: collecting probes (refresh={need_refresh}, cache_empty={cached_empty})");
        let report = collect_feature_health().await;
        log_feature_health_summary(&report);
        store_report(&report);
        Ok(report)
    } else {
        let stale_age = FEATURE_HEALTH_COLLECTED_AT
            .lock()
            .ok()
            .and_then(|t| *t)
            .map(|at| at.elapsed());
        let need_bg_refresh = stale_age.map(|a| a > HEALTH_REPORT_TTL).unwrap_or(true);
        if need_bg_refresh {
            tracing::info!(
                "feature_health: in-memory report past TTL or unset (age={:?}); serving cached snapshot and refreshing in background",
                stale_age
            );
            tokio::spawn(async move {
                let report = collect_feature_health().await;
                log_feature_health_summary(&report);
                store_report(&report);
            });
        }
        FEATURE_HEALTH_REPORT
            .lock()
            .map(|g| g.clone())
            .map_err(|_| "feature health lock poisoned".to_string())
    }
}
