//! Discord HTTP API for the agent router.
//!
//! Allows Ollama to call Discord's REST API (GET for read, POST only for sending messages)
//! when the request originates from Discord. Token and base URL are shared with the Gateway.
//!
//! **Non-idempotent sends** (POST message, Gateway `say`): retries only on errors that likely
//! occurred *before* Discord accepted the message (pre-connect / DNS / connection refused) or on
//! explicit rate limits. We do **not** retry on request timeouts, connection reset, or ambiguous
//! network errors where the first attempt may already have delivered the message. The same rule
//! should apply to any future outbound channel (e.g. Telegram `sendMessage`).

use std::sync::atomic::Ordering;
use std::time::Duration;
use tracing::{debug, info, warn};

const BASE_URL: &str = "https://discord.com/api/v10";
const MAX_RESPONSE_CHARS: usize = 8000;
const RETRY_DELAY_SECS: u64 = 2;
const MAX_RATE_LIMIT_RETRIES: u32 = 3;

/// Map a reqwest::Error from a Discord API request to a short user-facing message when the API
/// is unavailable (connection refused, timeout). Used by discord_api_request and send_message_*.
pub fn user_message_for_discord_request_error(e: &reqwest::Error) -> String {
    if e.is_connect() || e.is_timeout() {
        return "Discord API is temporarily unavailable (connection or timeout). Try again in a moment.".to_string();
    }
    let s = e.to_string();
    let lower = s.to_lowercase();
    if lower.contains("connection refused")
        || lower.contains("timed out")
        || lower.contains("connection reset")
    {
        return "Discord API is temporarily unavailable (connection or timeout). Try again in a moment.".to_string();
    }
    format!("Request failed: {}", e)
}

/// If the error looks like a Discord scope/permission failure, return a short user-facing message
/// so we don't echo technical errors into the conversation (log-005).
pub fn sanitize_discord_api_error(err: &str) -> String {
    let lower = err.to_lowercase();
    if lower.contains("scope")
        || lower.contains("operator.read")
        || lower.contains("permission")
        || lower.contains("403")
    {
        return "Message could not be sent (permission missing). Check bot permissions (e.g. operator.read scope).".to_string();
    }
    err.to_string()
}

/// Whether a failed **non-idempotent** Discord outbound send (HTTP or Gateway) may be retried
/// without a material risk of posting the same user-visible message twice.
///
/// Safe: rate limit (429), DNS / name resolution failures, connection refused, unreachable host
/// (request likely never reached Discord). Unsafe: timeouts, connection reset / broken pipe, and
/// other ambiguous cases where the server may already have stored the message.
pub fn is_safe_to_retry_discord_outbound_error_message(err_str: &str) -> bool {
    let lower = err_str.to_lowercase();
    if lower.contains("429") || lower.contains("rate limit") || lower.contains("too many requests")
    {
        return true;
    }
    // Timeouts may fire after the request body was accepted — do not retry the same content.
    if lower.contains("timed out") || lower.contains("timeout") || lower.contains("deadline") {
        return false;
    }
    if lower.contains("connection reset")
        || lower.contains("connection aborted")
        || lower.contains("broken pipe")
        || lower.contains("econnreset")
        || lower.contains("epipe")
        || lower.contains("etimedout")
    {
        return false;
    }
    lower.contains("connection refused")
        || lower.contains("econnrefused")
        || lower.contains("could not resolve")
        || lower.contains("failed to resolve")
        || lower.contains("failed to lookup")
        || lower.contains("nodename nor servname")
        || lower.contains("name or service not known")
        || lower.contains("dns")
        || lower.contains("enotfound")
        || lower.contains("eai_again")
        || lower.contains("network is unreachable")
        || lower.contains("enetunreach")
        || lower.contains("no route to host")
        || lower.contains("ehostunreach")
        || lower.contains("host unreachable")
}

/// Sleep duration before a single safe retry for Gateway / string-classified errors.
/// Rate limits get a longer backoff; other safe errors keep a short delay.
pub fn discord_outbound_safe_retry_sleep_duration(err_str: &str) -> Duration {
    let lower = err_str.to_lowercase();
    if lower.contains("429") || lower.contains("rate limit") || lower.contains("too many requests")
    {
        return Duration::from_millis(2000 + jitter_millis());
    }
    Duration::from_millis(500)
}

/// `reqwest` transport errors for Discord HTTP sends: retry only when the failure was likely
/// before the API accepted the request (see module comment).
pub fn is_safe_to_retry_discord_send_transport_error(e: &reqwest::Error) -> bool {
    if e.is_timeout() {
        return false;
    }
    if e.is_connect() {
        return true;
    }
    is_safe_to_retry_discord_outbound_error_message(&e.to_string())
}

/// Whether a terminal (non-retrying) transport error should advance the outbound send circuit breaker.
/// Rate-limit style failures are excluded elsewhere; 429 does not use this path for tripping.
pub(crate) fn discord_outbound_transport_terminal_should_trip(e: &reqwest::Error) -> bool {
    if e.is_timeout() || e.is_connect() {
        return true;
    }
    let s = e.to_string().to_lowercase();
    s.contains("connection reset")
        || s.contains("broken pipe")
        || s.contains("unreachable")
        || s.contains("resolve")
        || s.contains("lookup")
        || s.contains("dns")
}

/// Extract retry-after seconds from the Retry-After header value and/or the JSON body's
/// `retry_after` field.  Falls back to 5 s when neither source provides a value.
fn parse_retry_after(header_value: Option<f64>, body: &str) -> f64 {
    if let Some(secs) = header_value {
        return secs;
    }
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(secs) = json.get("retry_after").and_then(|v| v.as_f64()) {
            return secs;
        }
    }
    5.0
}

/// Pseudo-random jitter (100–499 ms) derived from the system clock.
fn jitter_millis() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    (nanos % 400) + 100
}

/// Wait out a Discord 429 rate-limit with jitter, or return Err when retries are exhausted.
/// `retry_after_header` is the parsed Retry-After header (seconds, float) if present.
pub(crate) async fn wait_for_rate_limit(
    retry_after_header: Option<f64>,
    body: &str,
    rate_limit_retries: &mut u32,
    route: &str,
) -> Result<(), String> {
    if *rate_limit_retries >= MAX_RATE_LIMIT_RETRIES {
        warn!(
            "Discord rate limited after {} retries on {}",
            MAX_RATE_LIMIT_RETRIES, route
        );
        return Err("Discord API rate limited after max retries. Try again later.".to_string());
    }
    let retry_secs = parse_retry_after(retry_after_header, body);
    let jitter = Duration::from_millis(jitter_millis());
    let total = Duration::from_secs_f64(retry_secs) + jitter;
    warn!(
        "Discord 429 on {} (retry {}/{}), waiting {:.1}s",
        route,
        *rate_limit_retries + 1,
        MAX_RATE_LIMIT_RETRIES,
        total.as_secs_f64()
    );
    tokio::time::sleep(total).await;
    *rate_limit_retries += 1;
    Ok(())
}

/// Extract the Retry-After header as f64 seconds from a response's headers.
pub(crate) fn retry_after_from_headers(headers: &reqwest::header::HeaderMap) -> Option<f64> {
    headers
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
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

    let body_json_opt: Option<serde_json::Value> = if method_upper == "POST" && body.is_some() {
        let body_str = body.unwrap_or("{}").trim();
        if crate::logging::VERBOSITY.load(Ordering::Relaxed) >= 3 {
            debug!("Discord API request body (decoded): {}", body_str);
        }
        Some(serde_json::from_str(body_str).map_err(|e| format!("Invalid JSON body: {}", e))?)
    } else {
        None
    };

    let track_outbound_circuit = method_upper == "POST";
    if track_outbound_circuit {
        if let Err(e) = crate::discord::discord_http_send_allow() {
            warn!(
                "Discord API POST {}: outbound send skipped (circuit): {}",
                path, e
            );
            return Err(e);
        }
    }

    let mut conn_attempt: u32 = 0;
    let mut rate_limit_retries: u32 = 0;
    let route = format!("{} {}", method_upper, path);

    loop {
        let mut req = client
            .request(
                method_upper
                    .parse()
                    .map_err(|e| format!("Invalid method: {}", e))?,
                &url,
            )
            .header("Authorization", format!("Bot {}", token))
            .header("User-Agent", &user_agent);

        if let Some(ref body_json) = body_json_opt {
            req = req
                .header("Content-Type", "application/json")
                .json(body_json);
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                let retryable = is_safe_to_retry_discord_send_transport_error(&e);
                if retryable && conn_attempt < 1 {
                    conn_attempt += 1;
                    info!(
                        "Discord API request failed (safe-to-retry transport), retrying in {}s (attempt {})",
                        RETRY_DELAY_SECS, conn_attempt
                    );
                    tokio::time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    continue;
                }
                if !retryable {
                    warn!(
                        "Discord API request failed with unsafe-to-retry transport error, not retrying to avoid duplicate: {}",
                        e
                    );
                }
                if track_outbound_circuit {
                    crate::discord::discord_http_send_record_failure(
                        discord_outbound_transport_terminal_should_trip(&e),
                    );
                }
                return Err(user_message_for_discord_request_error(&e));
            }
        };

        let status = resp.status();
        let retry_after_hdr = retry_after_from_headers(resp.headers());
        let body_text = resp.text().await.unwrap_or_default();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            wait_for_rate_limit(retry_after_hdr, &body_text, &mut rate_limit_retries, &route)
                .await?;
            continue;
        }

        if status.is_success() {
            if track_outbound_circuit {
                crate::discord::discord_http_send_record_success();
            }
            return if body_text.chars().count() > MAX_RESPONSE_CHARS {
                Ok(crate::logging::ellipse(&body_text, MAX_RESPONSE_CHARS))
            } else {
                Ok(body_text)
            };
        }

        // 5xx after a full request/response may mean the message was processed; only retry for idempotent GETs.
        if status.is_server_error() {
            let retry_5xx = method_upper == "GET" && conn_attempt < 1;
            if retry_5xx {
                conn_attempt += 1;
                info!(
                    "Discord API {} (5xx, GET retry attempt {}), retrying in {}s",
                    route, conn_attempt, RETRY_DELAY_SECS
                );
                tokio::time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                continue;
            }
            if method_upper == "POST" {
                warn!(
                    "Discord API {} returned {} — unsafe-to-retry for POST, not retrying to avoid duplicate message",
                    route, status
                );
            }
        }

        debug!("Discord API {}: {}", route, status);
        if track_outbound_circuit {
            crate::discord::discord_http_send_record_failure(status.is_server_error());
        }
        return Err(format!(
            "Discord API {}: {}",
            status,
            crate::logging::ellipse(&body_text, 500)
        ));
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

    let ch_id = channel_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let ch_name = channel_json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("(no name)");
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

        if let Ok(guild_body) = discord_api_request("GET", &format!("/guilds/{}", gid), None).await
        {
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
                        let name = c
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("(no name)");
                        let ty = c.get("type").and_then(|v| v.as_u64()).unwrap_or(0);
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

#[cfg(test)]
mod outbound_retry_tests {
    use super::*;

    #[test]
    fn outbound_safe_errors_include_rate_limit_and_preconnect() {
        assert!(is_safe_to_retry_discord_outbound_error_message(
            "HTTP 429 Too Many Requests"
        ));
        assert!(is_safe_to_retry_discord_outbound_error_message(
            "We are being rate limited"
        ));
        assert!(is_safe_to_retry_discord_outbound_error_message(
            "error sending request: connection refused"
        ));
        assert!(is_safe_to_retry_discord_outbound_error_message(
            "failed to resolve host 'discord.com'"
        ));
    }

    #[test]
    fn outbound_unsafe_errors_reject_retry() {
        assert!(!is_safe_to_retry_discord_outbound_error_message(
            "operation timed out"
        ));
        assert!(!is_safe_to_retry_discord_outbound_error_message(
            "Connection reset by peer"
        ));
        assert!(!is_safe_to_retry_discord_outbound_error_message(
            "broken pipe"
        ));
    }

    #[test]
    fn rate_limit_retry_uses_longer_delay() {
        let d_short = discord_outbound_safe_retry_sleep_duration("connection refused");
        let d_rl = discord_outbound_safe_retry_sleep_duration("429 rate limited");
        assert!(d_rl >= Duration::from_millis(2000));
        assert!(d_short < Duration::from_millis(1500));
    }
}
