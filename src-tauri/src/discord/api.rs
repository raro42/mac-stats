//! Discord HTTP API for the agent router.
//!
//! Allows Ollama to call Discord's REST API (GET for read, POST only for sending messages)
//! when the request originates from Discord. Token and base URL are shared with the Gateway.

use std::sync::atomic::Ordering;
use tracing::debug;

const BASE_URL: &str = "https://discord.com/api/v10";
const MAX_RESPONSE_CHARS: usize = 8000;

/// If the error looks like a Discord scope/permission failure, return a short user-facing message
/// so we don't echo technical errors into the conversation (log-005).
pub fn sanitize_discord_api_error(err: &str) -> String {
    let lower = err.to_lowercase();
    if lower.contains("scope") || lower.contains("operator.read") || lower.contains("permission") || lower.contains("403") {
        return "Message could not be sent (permission missing). Check bot permissions (e.g. operator.read scope).".to_string();
    }
    err.to_string()
}

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

    // Do not log request/response headers or bodies that may contain credentials.
    let mut req = client
        .request(
            method_upper.parse().map_err(|e| format!("Invalid method: {}", e))?,
            &url,
        )
        .header("Authorization", format!("Bot {}", token))
        .header("User-Agent", &user_agent);

    if method_upper == "POST" && body.is_some() {
        let body_str = body.unwrap_or("{}").trim();
        if crate::logging::VERBOSITY.load(Ordering::Relaxed) >= 3 {
            debug!("Discord API request body (decoded): {}", body_str);
        }
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
            crate::logging::ellipse(&body_text, 500)
        ));
    }

    if body_text.chars().count() > MAX_RESPONSE_CHARS {
        Ok(crate::logging::ellipse(&body_text, MAX_RESPONSE_CHARS))
    } else {
        Ok(body_text)
    }
}

/// Fetch guild and channel metadata for the given channel via the Discord API.
/// Used when invoking the discord-expert agent from a Discord context so it has
/// current channel, guild, and channel list without an extra round-trip.
///
/// Returns a concise text summary (channel_id, channel name/type, guild_id, guild name,
/// and channels in the guild) or an error string. DM channels only get channel info (no guild).
pub async fn fetch_guild_channel_metadata(channel_id: u64) -> Result<String, String> {
    let channel_path = format!("/channels/{}", channel_id);
    let channel_body = discord_api_request("GET", &channel_path, None).await?;
    let channel_json: serde_json::Value =
        serde_json::from_str(&channel_body).map_err(|e| format!("Parse channel JSON: {}", e))?;

    let ch_id = channel_json.get("id").and_then(|v| v.as_str()).unwrap_or("?");
    let ch_name = channel_json.get("name").and_then(|v| v.as_str()).unwrap_or("(no name)");
    let ch_type = channel_json
        .get("type")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let channel_type_name = match ch_type {
        0 => "text",
        2 => "voice",
        4 => "category",
        5 => "announcement",
        13 => "stage",
        15 => "forum",
        _ => "other",
    };

    let mut lines = vec![
        format!("channel_id: {}", ch_id),
        format!("channel: #{} (type: {})", ch_name, channel_type_name),
    ];

    let guild_id = channel_json.get("guild_id").and_then(|v| v.as_str());
    if let Some(gid) = guild_id {
        lines.push(format!("guild_id: {}", gid));

        if let Ok(guild_body) = discord_api_request("GET", &format!("/guilds/{}", gid), None).await {
            if let Ok(guild_json) = serde_json::from_str::<serde_json::Value>(&guild_body) {
                let guild_name = guild_json
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no name)");
                lines.push(format!("guild: {}", guild_name));
            }
        }

        if let Ok(channels_body) =
            discord_api_request("GET", &format!("/guilds/{}/channels", gid), None).await
        {
            if let Ok(channels_arr) = serde_json::from_str::<Vec<serde_json::Value>>(&channels_body)
            {
                let mut channel_entries: Vec<String> = channels_arr
                    .iter()
                    .map(|c| {
                        let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("(no name)");
                        let ty = c
                            .get("type")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        let ty_name = match ty {
                            0 => "text",
                            2 => "voice",
                            4 => "category",
                            5 => "announcement",
                            13 => "stage",
                            15 => "forum",
                            _ => "other",
                        };
                        format!("  {} #{} ({})", id, name, ty_name)
                    })
                    .collect();
                channel_entries.sort();
                lines.push("channels in this guild:".to_string());
                lines.push(channel_entries.join("\n"));
            }
        }
    } else {
        lines.push("(DM channel — no guild)".to_string());
    }

    Ok(lines.join("\n"))
}
