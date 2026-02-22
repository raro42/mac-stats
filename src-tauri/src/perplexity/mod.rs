//! Perplexity Search API client.
//!
//! Uses the official Search API: POST https://api.perplexity.ai/search
//! Returns ranked web search results (title, url, snippet). API key via Bearer auth.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const SEARCH_URL: &str = "https://api.perplexity.ai/search";

/// Request body for POST /search (minimal required fields).
#[derive(Debug, Clone, Serialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_recency_filter: Option<String>,
}

/// One search result page from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPage {
    pub title: String,
    pub url: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

/// Response from POST /search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchPage>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_time: Option<String>,
}

/// Run a web search via Perplexity Search API.
/// `api_key` must be a valid Perplexity API key (from Keychain).
pub async fn search(api_key: &str, query: &str, max_results: Option<u32>) -> Result<SearchResponse> {
    let body = SearchRequest {
        query: query.to_string(),
        max_results: max_results.or(Some(10)),
        search_recency_filter: None,
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Perplexity HTTP client")?;

    let response = client
        .post(SEARCH_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Perplexity search request")?;

    let status = response.status();
    let body_bytes = response
        .bytes()
        .await
        .context("Perplexity response body")?;

    if !status.is_success() {
        let msg = String::from_utf8_lossy(&body_bytes);
        anyhow::bail!("Perplexity API error {}: {}", status, msg);
    }

    let parsed: SearchResponse = serde_json::from_slice(&body_bytes).context("Perplexity response JSON")?;
    Ok(parsed)
}
