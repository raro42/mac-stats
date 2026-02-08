//! Discord HTTP API for the agent router.
//!
//! Allows Ollama to call Discord's REST API (GET for read, POST only for sending messages)
//! when the request originates from Discord. Token and base URL are shared with the Gateway.

use tracing::debug;

const BASE_URL: &str = "https://discord.com/api/v10";
const MAX_RESPONSE_CHARS: usize = 8000;

/// POST paths that are allowed (e.g. send message). All other POST/PATCH/DELETE are rejected.
fn is_allowed_post_path(path: &str) -> bool {
    let path = path.trim().trim_start_matches('/');
    // Allow: channels/{channel_id}/messages
    if let Some(rest) = path.strip_prefix("channels/") {
        if let Some(trailing) = rest.find('/') {
            let after_channel_id = &rest[trailing..];
            return after_channel_id == "/messages";
        }
    }
    false
}

/// Perform a Discord API request. Used by the DISCORD_API agent tool.
///
/// - method: GET, or POST only for allow-listed paths (e.g. POST /channels/{id}/messages).
/// - path: path relative to base (e.g. /users/@me/guilds). Must start with /.
/// - body: optional JSON body for POST; ignored for GET.
///
/// Returns the response body as string (truncated if very large), or an error string.
pub async fn discord_api_request(
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let token = match crate::discord::get_discord_token() {
        Some(t) => t,
        None => return Err("Discord not configured (no token)".to_string()),
    };

    let path = path.trim();
    if path.is_empty() || !path.starts_with('/') {
        return Err("Discord API path must start with /".to_string());
    }

    let method_upper = method.to_uppercase();
    let allowed = match method_upper.as_str() {
        "GET" => true,
        "POST" => is_allowed_post_path(path),
        _ => false,
    };
    if !allowed {
        return Err(format!(
            "Discord API: method {} not allowed (only GET, or POST to /channels/{{id}}/messages)",
            method
        ));
    }

    let url = format!("{}{}", BASE_URL, path);
    let version = crate::config::Config::version();
    let user_agent = format!("DiscordBot (mac-stats, {})", version);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let mut req = client
        .request(
            method_upper.parse().map_err(|e| format!("Invalid method: {}", e))?,
            &url,
        )
        .header("Authorization", format!("Bot {}", token))
        .header("User-Agent", &user_agent);

    if method_upper == "POST" && body.is_some() {
        let body_str = body.unwrap_or("{}").trim();
        let body_json: serde_json::Value = serde_json::from_str(body_str)
            .map_err(|e| format!("Invalid JSON body: {}", e))?;
        req = req.header("Content-Type", "application/json").json(&body_json);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        debug!("Discord API {} {}: {}", method_upper, path, status);
        return Err(format!(
            "Discord API {}: {}",
            status,
            body_text.chars().take(500).collect::<String>()
        ));
    }

    if body_text.len() > MAX_RESPONSE_CHARS {
        Ok(format!(
            "{}... [truncated, {} chars total]",
            body_text.chars().take(MAX_RESPONSE_CHARS).collect::<String>(),
            body_text.len()
        ))
    } else {
        Ok(body_text)
    }
}
