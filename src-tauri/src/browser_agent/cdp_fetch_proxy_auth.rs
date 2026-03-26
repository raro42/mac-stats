//! CDP `Fetch` domain: authenticated HTTP(S) proxies (`Fetch.authRequired` with source **Proxy**)
//! and safe defaults for server challenges, matching browser-use-style behaviour.
//!
//! headless_chrome already continues paused `Fetch.requestPaused` events via its default interceptor;
//! enabling `Fetch` here is scoped to `handleAuthRequests` so normal navigations are not stalled.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

use headless_chrome::protocol::cdp::types::Event;
use headless_chrome::protocol::cdp::Fetch;

use crate::{mac_stats_debug, mac_stats_info, mac_stats_warn};

static PROXY_AUTH_SETUP_TARGETS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn setup_targets() -> &'static Mutex<HashSet<String>> {
    PROXY_AUTH_SETUP_TARGETS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub(crate) fn clear_proxy_auth_setup_targets() {
    if let Ok(mut g) = setup_targets().lock() {
        g.clear();
    }
}

/// When `browserCdpProxyUsername` / `browserCdpProxyPassword` are both set, enable `Fetch` with
/// `handleAuthRequests` and route **Proxy** challenges to configured credentials; other challenges
/// use Chrome `Default` handling via [`headless_chrome::Tab::reset_fetch_auth_challenge_response_to_default`].
pub(crate) fn ensure_fetch_proxy_auth_on_tab(tab: &Arc<headless_chrome::Tab>) {
    if !crate::config::Config::browser_cdp_proxy_credentials_active() {
        return;
    }
    let tid = tab.get_target_id().to_string();
    {
        let g = setup_targets().lock().unwrap();
        if g.contains(&tid) {
            return;
        }
    }
    if let Err(e) = tab.enable_fetch(None, Some(true)) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: Fetch.enable (handleAuthRequests) failed: {} — CDP proxy auth handling unavailable on this tab",
            e
        );
        return;
    }
    let Some(u) = crate::config::Config::browser_cdp_proxy_username() else {
        return;
    };
    let Some(p) = crate::config::Config::browser_cdp_proxy_password() else {
        return;
    };
    if u.trim().is_empty() || p.trim().is_empty() {
        return;
    }
    let tab_l = Arc::clone(tab);
    let user = u;
    let pass = p;
    let listener = Arc::new(move |event: &Event| {
        let Event::FetchAuthRequired(ev) = event else {
            return;
        };
        let use_proxy_creds = matches!(
            ev.params.auth_challenge.source,
            Some(Fetch::AuthChallengeSource::Proxy)
        );
        if use_proxy_creds {
            if let Err(e) = tab_l.authenticate(Some(user.clone()), Some(pass.clone())) {
                mac_stats_debug!(
                    "browser/cdp",
                    "Browser agent [CDP]: authenticate before Fetch.authRequired (proxy) failed: {}",
                    e
                );
            }
        } else if let Err(e) = tab_l.reset_fetch_auth_challenge_response_to_default() {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: reset_fetch_auth_challenge_response_to_default before Fetch.authRequired failed: {}",
                e
            );
        }
    });
    if let Err(e) = tab.add_event_listener(listener) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: could not attach Fetch.authRequired listener: {}",
            e
        );
        return;
    }
    if let Ok(mut g) = setup_targets().lock() {
        g.insert(tid);
    }
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: Fetch proxy auth routing enabled (Proxy challenges use configured credentials; Server uses Default)"
    );
}
