//! Persist Chrome cookie jar to `~/.mac-stats/browser_storage_state.json` (CDP `Network.getAllCookies` / `Network.setCookies`).
//! Merge on save: key `(name, domain, path)`; values from the live browser overwrite file entries for the same key.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use headless_chrome::protocol::cdp::Network::{
    self, ClearBrowserCookies, Cookie, CookieParam, GetAllCookies, SetCookies,
};
use headless_chrome::{Browser, Tab};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::mac_stats_debug;

/// Set when a new `Browser` is attached; first tab use runs restore once.
pub static COOKIE_RESTORE_PENDING: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Deserialize)]
struct BrowserStorageFile {
    version: u32,
    cookies: Vec<Cookie>,
}

fn storage_path() -> std::path::PathBuf {
    Config::browser_storage_state_json_path()
}

fn cookie_key(c: &Cookie) -> (String, String, String) {
    (c.name.clone(), c.domain.clone(), c.path.clone())
}

fn merge_cookie_vecs(from_file: Vec<Cookie>, from_browser: Vec<Cookie>) -> Vec<Cookie> {
    let mut map: HashMap<(String, String, String), Cookie> = HashMap::new();
    for c in from_file {
        map.insert(cookie_key(&c), c);
    }
    for c in from_browser {
        map.insert(cookie_key(&c), c);
    }
    map.into_values().collect()
}

fn load_cookies_vec(path: &Path) -> Result<Vec<Cookie>, String> {
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: BrowserStorageFile = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(parsed.cookies)
}

fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn cookie_to_param(c: &Cookie) -> CookieParam {
    CookieParam {
        name: c.name.clone(),
        value: c.value.clone(),
        url: None,
        domain: Some(c.domain.clone()),
        path: Some(c.path.clone()),
        secure: Some(c.secure),
        http_only: Some(c.http_only),
        same_site: c.same_site.clone(),
        expires: if c.session { None } else { Some(c.expires) },
        priority: Some(c.priority.clone()),
        same_party: Some(c.same_party),
        source_scheme: Some(c.source_scheme.clone()),
        source_port: Some(c.source_port),
        partition_key: c.partition_key.clone(),
    }
}

/// Call after inserting a new cached `Browser` session so the first tab enables restore.
pub fn mark_cookie_restore_pending() {
    COOKIE_RESTORE_PENDING.store(true, Ordering::SeqCst);
}

/// If a new browser was just attached, restore cookies from disk (once per session).
pub fn apply_pending_cookie_restore(tab: &Tab) {
    if !COOKIE_RESTORE_PENDING.swap(false, Ordering::SeqCst) {
        return;
    }
    if let Err(e) = restore_cookies_to_tab(tab) {
        mac_stats_debug!(
            "browser/cookies",
            "Browser agent [cookies]: restore after new session skipped: {}",
            e
        );
    }
}

/// Best-effort snapshot before dropping an idle or replaced browser session.
pub fn try_save_cookies_from_browser(browser: &Browser) {
    let tab = match browser.get_tabs().lock() {
        Ok(t) => t.first().cloned(),
        Err(_) => None,
    };
    let Some(tab) = tab else {
        return;
    };
    if let Err(e) = save_cookies_from_tab(&tab) {
        mac_stats_debug!(
            "browser/cookies",
            "Browser agent [cookies]: save before session drop skipped: {}",
            e
        );
    }
}

/// Merge live jar with file and write atomically.
pub fn save_cookies_from_tab(tab: &Tab) -> Result<(), String> {
    let path = storage_path();
    let from_browser = tab
        .call_method(GetAllCookies(None))
        .map_err(|e| e.to_string())?
        .cookies;
    let from_file = if path.exists() {
        load_cookies_vec(&path).unwrap_or_default()
    } else {
        Vec::new()
    };
    let merged = merge_cookie_vecs(from_file, from_browser);
    let n = merged.len();
    let file = BrowserStorageFile {
        version: 1,
        cookies: merged,
    };
    let json = serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?;
    atomic_write(&path, &json).map_err(|e| e.to_string())?;
    mac_stats_debug!(
        "browser/cookies",
        "Browser agent [cookies]: saved {} cookie(s) to disk (merged)",
        n
    );
    Ok(())
}

fn restore_cookies_to_tab(tab: &Tab) -> Result<(), String> {
    let path = storage_path();
    if !path.exists() {
        mac_stats_debug!(
            "browser/cookies",
            "Browser agent [cookies]: no storage file, skip restore"
        );
        return Ok(());
    }
    let cookies = load_cookies_vec(&path)?;
    if cookies.is_empty() {
        mac_stats_debug!(
            "browser/cookies",
            "Browser agent [cookies]: storage file empty, skip restore"
        );
        return Ok(());
    }
    let n = cookies.len();
    let _ = tab.call_method(Network::Enable {
        max_total_buffer_size: None,
        max_resource_buffer_size: None,
        max_post_data_size: None,
        report_direct_socket_traffic: None,
        enable_durable_messages: None,
    });
    let params: Vec<CookieParam> = cookies.iter().map(cookie_to_param).collect();
    tab.call_method(SetCookies { cookies: params })
        .map_err(|e| e.to_string())?;
    mac_stats_debug!(
        "browser/cookies",
        "Browser agent [cookies]: restored {} cookie(s) from disk",
        n
    );
    Ok(())
}

/// Delete `browser_storage_state.json` if present. Returns whether a file was removed.
pub fn remove_storage_file_if_present() -> Result<bool, String> {
    let path = storage_path();
    if !path.exists() {
        return Ok(false);
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn clear_browser_cookie_jar(tab: &Tab) -> Result<(), String> {
    tab.call_method(ClearBrowserCookies(None))
        .map_err(|e| e.to_string())?;
    Ok(())
}
