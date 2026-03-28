//! Cached GET /api/tags with TTL, stale-while-revalidate, in-flight deduplication, and
//! poisoned-cache prevention: failed or empty responses do not replace a prior non-empty list.

use crate::ollama::ListResponse;
use crate::{mac_stats_info, mac_stats_warn};
use futures_util::future::FutureExt;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const MODEL_LIST_TTL: Duration = Duration::from_secs(5 * 60);

/// Visible in `~/.mac-stats/debug.log` message text: the file log layer omits tracing `target`, so
/// operators should `rg '\[ollama/model_cache\]'` (or the human-readable phrases) to audit stale serves.
const MCACHE_LOG_TAG: &str = "[ollama/model_cache]";

type FetchResult = Result<ListResponse, String>;
type BoxFetch = Pin<Box<dyn Future<Output = FetchResult> + Send>>;
type SharedFetch = futures_util::future::Shared<BoxFetch>;

#[derive(Default)]
struct EndpointEntry {
    last_success: Option<(Instant, ListResponse)>,
    bg_refreshing: bool,
}

#[derive(Default)]
struct CacheInner {
    endpoints: HashMap<String, EndpointEntry>,
    inflight: HashMap<String, SharedFetch>,
}

fn cache() -> &'static Mutex<CacheInner> {
    static C: OnceLock<Mutex<CacheInner>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(CacheInner::default()))
}

fn norm_endpoint(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

/// Apply poisoned-cache rules: empty `Ok` or `Err` does not replace a prior non-empty list.
fn merge_tags_fetch_result(fetched: FetchResult, prior: Option<ListResponse>) -> FetchResult {
    match fetched {
        Ok(list) if !list.models.is_empty() => Ok(list),
        Ok(list) => Ok(prior.filter(|p| !p.models.is_empty()).unwrap_or(list)),
        Err(e) => prior
            .filter(|p| !p.models.is_empty())
            .map(Ok)
            .unwrap_or(Err(e)),
    }
}

/// Drop cached entries (e.g. after Ollama endpoint change).
pub async fn clear_all() {
    let mut g = cache().lock().await;
    g.endpoints.clear();
    g.inflight.clear();
}

/// Drop cache for one endpoint only (e.g. cold-start retry without flushing other hosts).
pub async fn clear_endpoint(endpoint: &str) {
    let ep = norm_endpoint(endpoint);
    if ep.is_empty() {
        return;
    }
    let mut g = cache().lock().await;
    g.endpoints.remove(&ep);
    g.inflight.remove(&ep);
}

async fn fetch_tags_http(endpoint: &str, api_key: Option<&str>) -> FetchResult {
    crate::ollama::ollama_http_circuit_allow()?;
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let mut request = client.get(&url);
    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let http = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            crate::ollama::ollama_http_circuit_record_failure(true);
            return Err(format!("Failed to request models: {}", e));
        }
    };

    let status = http.status();
    if !status.is_success() {
        crate::ollama::ollama_http_circuit_record_failure(status.is_server_error());
        return Err(format!("Ollama /api/tags HTTP {}", status));
    }

    let list = match http.json::<ListResponse>().await {
        Ok(r) => r,
        Err(e) => {
            crate::ollama::ollama_http_circuit_record_failure(false);
            return Err(format!("Failed to parse models response: {}", e));
        }
    };
    crate::ollama::ollama_http_circuit_record_success();
    Ok(list)
}

async fn run_bg_refresh(endpoint: String, api_key: Option<String>) {
    let res = fetch_tags_http(&endpoint, api_key.as_deref()).await;
    let mut g = cache().lock().await;
    let ent = g.endpoints.entry(endpoint.clone()).or_default();
    ent.bg_refreshing = false;
    match res {
        Ok(list) if !list.models.is_empty() => {
            ent.last_success = Some((Instant::now(), list.clone()));
            mac_stats_info!(
                "ollama/model_cache",
                "{} Background model list refresh succeeded ({} models) for {}",
                MCACHE_LOG_TAG,
                list.models.len(),
                endpoint
            );
        }
        Ok(list) => {
            mac_stats_warn!(
                "ollama/model_cache",
                "{} Background model list refresh returned empty ({} models in response); cache not updated",
                MCACHE_LOG_TAG,
                list.models.len()
            );
        }
        Err(e) => {
            mac_stats_warn!(
                "ollama/model_cache",
                "{} Background model list refresh failed for {}: {}",
                MCACHE_LOG_TAG,
                endpoint,
                e
            );
        }
    }
}

/// Fetch `/api/tags` with caching. Concurrent callers share one in-flight request per endpoint.
pub async fn fetch_tags_cached(endpoint: &str, api_key: Option<&str>) -> FetchResult {
    let ep = norm_endpoint(endpoint);
    if ep.is_empty() {
        return Err("Ollama endpoint is empty".to_string());
    }
    let now = Instant::now();
    let api_key_owned = api_key.map(|s| s.to_string());

    {
        let g = cache().lock().await;
        if let Some(ent) = g.endpoints.get(&ep) {
            if let Some((t, list)) = &ent.last_success {
                if now.duration_since(*t) < MODEL_LIST_TTL {
                    return Ok(list.clone());
                }
            }
        }
    }

    {
        let mut g = cache().lock().await;
        if let Some(ent) = g.endpoints.get_mut(&ep) {
            if let Some((t, list)) = ent.last_success.clone() {
                if now.duration_since(t) >= MODEL_LIST_TTL {
                    let age = now.duration_since(t);
                    if !ent.bg_refreshing {
                        ent.bg_refreshing = true;
                        let ep_clone = ep.clone();
                        let ak = api_key_owned.clone();
                        drop(g);
                        tokio::spawn(run_bg_refresh(ep_clone, ak));
                        mac_stats_warn!(
                            "ollama/model_cache",
                            "{} Model list is stale (last successful refresh {}s ago); serving cached result ({} models) while refreshing in background",
                            MCACHE_LOG_TAG,
                            age.as_secs(),
                            list.models.len()
                        );
                        return Ok(list);
                    }
                    mac_stats_warn!(
                        "ollama/model_cache",
                        "{} Model list is stale ({}s since success); background refresh already in progress; serving cached result ({} models)",
                        MCACHE_LOG_TAG,
                        age.as_secs(),
                        list.models.len()
                    );
                    return Ok(list);
                }
            }
        }
    }

    let shared = {
        let mut g = cache().lock().await;
        if let Some(s) = g.inflight.get(&ep) {
            s.clone()
        } else {
            let ep_fetch = ep.clone();
            let ak = api_key_owned.clone();
            let fut = async move { fetch_tags_http(&ep_fetch, ak.as_deref()).await }
                .boxed()
                .shared();
            g.inflight.insert(ep.clone(), fut.clone());
            fut
        }
    };

    let result = shared.await;

    let out = {
        let mut g = cache().lock().await;
        g.inflight.remove(&ep);
        match result {
            Ok(list) if !list.models.is_empty() => {
                let ent = g.endpoints.entry(ep.clone()).or_default();
                ent.last_success = Some((Instant::now(), list.clone()));
                Ok(list)
            }
            Ok(list) => {
                let prior_snapshot = g.endpoints.get(&ep).and_then(|e| e.last_success.clone());
                let prior_list = prior_snapshot.as_ref().map(|(_, l)| l.clone());
                mac_stats_warn!(
                    "ollama/model_cache",
                    "{} Ollama returned empty model list ({} entries); not replacing cached data",
                    MCACHE_LOG_TAG,
                    list.models.len()
                );
                let merged = merge_tags_fetch_result(Ok(list), prior_list);
                if let Some((age_start, stale)) = prior_snapshot.as_ref() {
                    if !stale.models.is_empty() {
                        let age = Instant::now().duration_since(*age_start);
                        mac_stats_warn!(
                            "ollama/model_cache",
                            "{} Serving stale model list (last success {}s ago, {} models) after empty /api/tags",
                            MCACHE_LOG_TAG,
                            age.as_secs(),
                            stale.models.len()
                        );
                    }
                }
                merged
            }
            Err(e) => {
                let prior_snapshot = g.endpoints.get(&ep).and_then(|e| e.last_success.clone());
                let prior_list = prior_snapshot.as_ref().map(|(_, l)| l.clone());
                mac_stats_warn!(
                    "ollama/model_cache",
                    "{} Model list fetch failed: {}; not updating cache",
                    MCACHE_LOG_TAG,
                    e
                );
                let merged = merge_tags_fetch_result(Err(e), prior_list);
                if let Some((age_start, stale)) = prior_snapshot.as_ref() {
                    if !stale.models.is_empty() {
                        let age = Instant::now().duration_since(*age_start);
                        mac_stats_warn!(
                            "ollama/model_cache",
                            "{} Serving stale model list (last success {}s ago, {} models) after fetch error",
                            MCACHE_LOG_TAG,
                            age.as_secs(),
                            stale.models.len()
                        );
                    }
                }
                merged
            }
        }
    };
    out
}

#[cfg(test)]
mod merge_tests {
    use super::merge_tags_fetch_result;
    use crate::ollama::{ListResponse, ModelSummary};

    fn model(name: &str) -> ModelSummary {
        ModelSummary {
            name: name.to_string(),
            modified_at: None,
            size: None,
            digest: None,
            details: None,
        }
    }

    fn list_with(names: &[&str]) -> ListResponse {
        ListResponse {
            models: names.iter().copied().map(model).collect(),
        }
    }

    #[test]
    fn nonempty_ok_passthrough() {
        let fresh = list_with(&["a"]);
        let out = merge_tags_fetch_result(Ok(fresh.clone()), Some(list_with(&["b"]))).unwrap();
        assert_eq!(out.models.len(), 1);
        assert_eq!(out.models[0].name, "a");
    }

    #[test]
    fn empty_ok_keeps_prior_when_nonempty() {
        let prior = list_with(&["keep"]);
        let empty = ListResponse { models: vec![] };
        let out = merge_tags_fetch_result(Ok(empty), Some(prior.clone())).unwrap();
        assert_eq!(out.models.len(), 1);
        assert_eq!(out.models[0].name, "keep");
    }

    #[test]
    fn empty_ok_no_prior_returns_empty() {
        let empty = ListResponse { models: vec![] };
        let out = merge_tags_fetch_result(Ok(empty.clone()), None).unwrap();
        assert!(out.models.is_empty());
    }

    #[test]
    fn err_with_prior_non_empty_returns_ok_stale() {
        let prior = list_with(&["x"]);
        let out = merge_tags_fetch_result(Err("boom".to_string()), Some(prior.clone())).unwrap();
        assert_eq!(out.models[0].name, "x");
    }

    #[test]
    fn err_without_prior_returns_err() {
        let out = merge_tags_fetch_result(Err("nope".to_string()), None);
        assert_eq!(out.unwrap_err(), "nope");
    }

    #[test]
    fn prior_empty_does_not_block_empty_ok() {
        let prior = ListResponse { models: vec![] };
        let empty = ListResponse { models: vec![] };
        let out = merge_tags_fetch_result(Ok(empty.clone()), Some(prior)).unwrap();
        assert!(out.models.is_empty());
    }
}
