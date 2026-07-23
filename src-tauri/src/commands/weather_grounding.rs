//! Ground weather answers with Open-Meteo so Brave snippets cannot invent temps.

use serde_json::Value;
use tracing::info;

/// True when the search query / user question is about current weather / forecast.
pub(crate) fn looks_like_weather_query(q: &str) -> bool {
    let n = normalize_weather_text(q).to_lowercase();
    // include common typo "wether"
    n.contains("weather")
        || n.contains("wether")
        || n.contains("forecast")
        || n.contains("temperature")
        || n.contains("humidit")
        || ((n.contains("rain") || n.contains("wind") || n.contains("cloudy"))
            && (n.contains("today") || n.contains("now") || n.contains("current")))
}

/// Extract a place name from a weather question/query.
pub(crate) fn extract_place(q: &str) -> Option<String> {
    let q = normalize_weather_text(q);
    let lower = q.to_lowercase();
    for sep in [" in ", " for ", " at "] {
        if let Some(idx) = lower.find(sep) {
            let rest = q[idx + sep.len()..].trim();
            // "temperature at noon" / "rain at midnight" — not a place.
            if sep == " at " && looks_like_time_of_day_rest(rest) {
                continue;
            }
            let end = rest.find(['?', '!', '.', '\n']).unwrap_or(rest.len());
            let mut tokens: Vec<&str> = rest[..end].split_whitespace().collect();
            while let Some(last) = tokens.last() {
                let l = last.to_lowercase().trim_matches(|c: char| {
                    matches!(c, '?' | '!' | '.' | ',' | ';' | ':')
                }).to_string();
                if matches!(
                    l.as_str(),
                    "right"
                        | "now"
                        | "today"
                        | "tonight"
                        | "currently"
                        | "please"
                        | "spain"
                        | "current"
                        | "conditions"
                        | "like"
                        | "and"
                        | "weather"
                        | "wether"
                        | "forecast"
                        | "date"
                        | "time"
                        | "datetime"
                        | "the"
                ) {
                    tokens.pop();
                } else {
                    break;
                }
            }
            while tokens.first().map(|t| t.eq_ignore_ascii_case("like")) == Some(true) {
                tokens.remove(0);
            }
            let place = tokens.join(" ").trim().to_string();
            if place.chars().count() >= 2 && !looks_like_place_garbage(&place) {
                return Some(place);
            }
        }
    }
    let stripped = lower
        .replace("current conditions", "")
        .replace("weather forecast", "")
        .replace("weather", "")
        .replace("wether", "")
        .replace("forecast", "")
        .replace("temperature", "")
        .replace("like", "")
        .replace("what's the", "")
        .replace("whats the", "")
        .replace("what is the", "")
        .replace("how's the", "")
        .replace("hows the", "")
        .replace("how is the", "")
        .replace("right now", "")
        .replace("today", "");
    let place = stripped
        .split_whitespace()
        .filter(|t| {
            !matches!(
                *t,
                "the"
                    | "a"
                    | "an"
                    | "in"
                    | "for"
                    | "at"
                    | "spain"
                    | "how"
                    | "how's"
                    | "hows"
                    | "what"
                    | "whats"
                    | "what's"
                    | "is"
                    | "it"
                    | "?"
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    if place.chars().count() >= 2 && !looks_like_place_garbage(&place) {
        if let Some(pos) = lower.find(&place) {
            return Some(q[pos..pos + place.len()].trim().to_string());
        }
        return Some(place);
    }
    None
}

fn looks_like_place_garbage(place: &str) -> bool {
    let n = place.to_lowercase();
    if n.starts_with("how")
        || n.starts_with("what")
        || n.starts_with("going")
        || n.starts_with("will ")
        || n.starts_with("would ")
    {
        return true;
    }
    let tokens: Vec<String> = n
        .split_whitespace()
        .map(|t| {
            t.trim_matches(|c: char| matches!(c, '?' | '!' | '.' | ',' | ';' | ':' | '"' | '\''))
                .to_string()
        })
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() || tokens.len() > 5 {
        return true;
    }
    // Clock-ish tokens are not places ("3pm", "14:00").
    if tokens.iter().any(|t| {
        t.chars().any(|c| c.is_ascii_digit()) || t.ends_with("am") || t.ends_with("pm")
    }) {
        return true;
    }
    // Phrase leftovers from "how's the weather going to be over time…" must not geocode.
    const STOP: &[&str] = &[
        "is",
        "it",
        "the",
        "a",
        "an",
        "please",
        "now",
        "today",
        "tonight",
        "going",
        "to",
        "be",
        "over",
        "time",
        "of",
        "day",
        "will",
        "would",
        "this",
        "week",
        "weekend",
        "tomorrow",
        "morning",
        "afternoon",
        "evening",
        "next",
        "few",
        "days",
        "hours",
        "minutes",
        "like",
        "look",
        "looking",
        "and",
        "or",
        "vs",
        "versus",
        "noon",
        "midnight",
        "sunrise",
        "sunset",
        "dusk",
        "dawn",
        "midday",
        "night",
        "nighttime",
        "nightfall",
        "lunch",
        "dinner",
        "breakfast",
        "brunch",
    ];
    if matches!(
        n.as_str(),
        "is" | "it" | "the" | "a" | "an" | "please" | "now" | "today" | "tonight"
    ) {
        return true;
    }
    let stop_count = tokens.iter().filter(|t| STOP.contains(&t.as_str())).count();
    stop_count * 2 >= tokens.len()
}

/// True when text after " at " is a time-of-day, not a place name.
fn looks_like_time_of_day_rest(rest: &str) -> bool {
    let first = rest
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase()
        .trim_matches(|c: char| matches!(c, '?' | '!' | '.' | ',' | ';' | ':'))
        .to_string();
    if first.is_empty() {
        return false;
    }
    if first.chars().any(|c| c.is_ascii_digit())
        || first.ends_with("am")
        || first.ends_with("pm")
    {
        return true;
    }
    matches!(
        first.as_str(),
        "noon"
            | "night"
            | "midnight"
            | "sunrise"
            | "sunset"
            | "dusk"
            | "dawn"
            | "lunch"
            | "dinner"
            | "breakfast"
            | "brunch"
            | "midday"
            | "nighttime"
            | "nightfall"
    )
}

/// True when Open-Meteo instant reply can answer without Brave/Perplexity.
pub(crate) fn can_instant_weather(q: &str) -> bool {
    looks_like_weather_query(q) && resolve_weather_place(q).is_some()
}

/// Place for Open-Meteo: explicit "in X", or configured default for short local asks.
pub(crate) fn resolve_weather_place(q: &str) -> Option<String> {
    if let Some(p) = extract_place(q) {
        return Some(p);
    }
    if should_use_default_weather_place(q) {
        return Some(crate::config::Config::weather_default_place());
    }
    None
}

fn should_use_default_weather_place(q: &str) -> bool {
    let n = normalize_weather_text(q).to_lowercase();
    if n.contains(" in ") || n.contains(" for ") {
        return false;
    }
    // "at Barcelona" is an explicit place; "at noon" is still a local ask.
    if let Some(idx) = n.find(" at ") {
        let rest = n[idx + " at ".len()..].trim();
        if !looks_like_time_of_day_rest(rest) {
            return false;
        }
    }
    if n.contains("search") || n.contains("google") || n.contains("http") {
        return false;
    }
    if n.chars().count() > 64 {
        return false;
    }
    // "how's the weather", "weather today", "is it raining", "temperature at noon", etc.
    let local = n.contains("today")
        || n.contains("tonight")
        || n.contains("right now")
        || n.contains(" outside")
        || n.contains(" here")
        || n.contains("current")
        || n.contains("temperature")
        || n.contains("weather")
        || n.contains("wether")
        || n.contains("forecast")
        || n.starts_with("how")
        || n == "weather"
        || n == "wether"
        || n.starts_with("weather?")
        || n.starts_with("wether?");
    local && looks_like_weather_query(q)
}

fn normalize_weather_text(q: &str) -> String {
    q.chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{00B4}' | '`' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            _ => c,
        })
        .collect()
}

/// Fetch current conditions from Open-Meteo for a weather query. Returns a grounded block
/// the model must prefer over search snippets.
pub(crate) async fn open_meteo_grounding_block(query: &str) -> Option<String> {
    if !looks_like_weather_query(query) {
        return None;
    }
    let place = resolve_weather_place(query)?;
    info!(
        "Weather grounding: geocoding place {:?}",
        crate::logging::ellipse(&place, 60)
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .ok()?;
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        urlencoding_encode(&place)
    );
    let geo_text = client.get(&geo_url).send().await.ok()?.text().await.ok()?;
    let geo: Value = serde_json::from_str(&geo_text).ok()?;
    let results = geo.get("results")?.as_array()?;
    let first = results.first()?;
    let lat = first.get("latitude")?.as_f64()?;
    let lon = first.get("longitude")?.as_f64()?;
    let name = first
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(place.as_str());
    let admin = first
        .get("admin1")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let country = first
        .get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let forecast_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m,precipitation&timezone=auto",
        lat, lon
    );
    let fc_text = client
        .get(&forecast_url)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let fc: Value = serde_json::from_str(&fc_text).ok()?;
    let current = fc.get("current")?;
    let temp = current.get("temperature_2m")?.as_f64()?;
    let feels = current
        .get("apparent_temperature")
        .and_then(|v| v.as_f64());
    let humidity = current
        .get("relative_humidity_2m")
        .and_then(|v| v.as_f64());
    let wind = current.get("wind_speed_10m").and_then(|v| v.as_f64());
    let precip = current.get("precipitation").and_then(|v| v.as_f64());
    let code = current.get("weather_code").and_then(|v| v.as_i64());
    let when = current
        .get("time")
        .and_then(|v| v.as_str())
        .unwrap_or("now");
    let tz = fc
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("local");
    let desc = weather_code_label(code.unwrap_or(-1));
    let mut lines = vec![
        "**Grounded current conditions (Open-Meteo — prefer these numbers over search snippets):**"
            .to_string(),
        format!(
            "- Place: {}{}{}",
            name,
            if admin.is_empty() {
                String::new()
            } else {
                format!(", {}", admin)
            },
            if country.is_empty() {
                String::new()
            } else {
                format!(", {}", country)
            }
        ),
        format!("- Observed at: {} ({})", when, tz),
        format!("- Temperature: {:.1} °C", temp),
    ];
    if let Some(f) = feels {
        lines.push(format!("- Feels like: {:.1} °C", f));
    }
    if let Some(h) = humidity {
        lines.push(format!("- Humidity: {:.0}%", h));
    }
    if let Some(w) = wind {
        lines.push(format!("- Wind: {:.1} km/h", w));
    }
    if let Some(p) = precip {
        lines.push(format!("- Precipitation: {:.1} mm", p));
    }
    lines.push(format!("- Conditions: {}", desc));
    lines.push(
        "Reply with these figures. Search snippets are links/context only — do not invent conflicting temps."
            .to_string(),
    );
    Some(lines.join("\n"))
}

/// User-facing weather reply (no LLM). Used for clear "weather in X" asks.
pub(crate) async fn format_instant_weather_reply(query: &str) -> Option<String> {
    if !looks_like_weather_query(query) {
        return None;
    }
    let place = resolve_weather_place(query)?;
    info!(
        "Weather instant: geocoding place {:?}",
        crate::logging::ellipse(&place, 60)
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .ok()?;
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        urlencoding_encode(&place)
    );
    let geo_text = client.get(&geo_url).send().await.ok()?.text().await.ok()?;
    let geo: Value = serde_json::from_str(&geo_text).ok()?;
    let first = geo.get("results")?.as_array()?.first()?;
    let lat = first.get("latitude")?.as_f64()?;
    let lon = first.get("longitude")?.as_f64()?;
    let name = first
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&place);
    let admin = first
        .get("admin1")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let country = first
        .get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let fc_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m,precipitation&timezone=auto",
        lat, lon
    );
    let fc_text = client.get(&fc_url).send().await.ok()?.text().await.ok()?;
    let fc: Value = serde_json::from_str(&fc_text).ok()?;
    let current = fc.get("current")?;
    let temp = current.get("temperature_2m")?.as_f64()?;
    let feels = current
        .get("apparent_temperature")
        .and_then(|v| v.as_f64());
    let humidity = current
        .get("relative_humidity_2m")
        .and_then(|v| v.as_f64());
    let wind = current.get("wind_speed_10m").and_then(|v| v.as_f64());
    let code = current.get("weather_code").and_then(|v| v.as_i64());
    let when = current
        .get("time")
        .and_then(|v| v.as_str())
        .unwrap_or("now");
    let tz = fc
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("local");
    let desc = weather_code_label(code.unwrap_or(-1));
    let where_s = format!(
        "{}{}{}",
        name,
        if admin.is_empty() {
            String::new()
        } else {
            format!(", {}", admin)
        },
        if country.is_empty() {
            String::new()
        } else {
            format!(", {}", country)
        }
    );
    let mut out = format!(
        "**{}**\n\n\
• **{:.1} °C** — {}\n",
        where_s, temp, desc
    );
    if let Some(f) = feels {
        out.push_str(&format!("• Feels like **{:.1} °C**\n", f));
    }
    if let Some(h) = humidity {
        out.push_str(&format!("• Humidity **{:.0}%**\n", h));
    }
    if let Some(w) = wind {
        out.push_str(&format!("• Wind **{:.1} km/h**\n", w));
    }
    out.push_str(&format!(
        "\nLocal time: **{}** ({})\n_Source: Open-Meteo_",
        when, tz
    ));
    Some(out)
}

fn weather_code_label(code: i64) -> &'static str {
    match code {
        0 => "Clear",
        1 | 2 => "Mainly clear / partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing rain",
        71 | 73 | 75 | 77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Unknown",
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_weather_typo() {
        assert!(looks_like_weather_query(
            "What´s the wether like in El Masnou right now?"
        ));
    }

    #[test]
    fn extracts_place_in() {
        let p = extract_place("What's the weather like in El Masnou right now?").unwrap();
        assert!(p.to_lowercase().contains("masnou"), "{p}");
    }

    #[test]
    fn extracts_place_date_time_and_weather() {
        let q = "What's date and time in El Masnou and weather?";
        assert!(looks_like_weather_query(q));
        let p = extract_place(q).unwrap();
        assert_eq!(p.to_lowercase(), "el masnou");
        assert!(can_instant_weather(q));
    }

    #[test]
    fn extracts_place_curly_apostrophe_typo() {
        let p = extract_place("What´s the wether like in El Masnou right now?").unwrap();
        assert!(p.to_lowercase().contains("masnou"), "{p}");
        assert!(can_instant_weather(
            "What´s the wether like in El Masnou right now?"
        ));
    }

    #[test]
    fn default_place_for_hows_the_weather() {
        assert!(should_use_default_weather_place("How's the weather?"));
        assert!(can_instant_weather("How's the weather today?"));
        assert!(!should_use_default_weather_place(
            "search the weather trends in Europe"
        ));
    }

    #[test]
    fn rejects_forecast_phrase_as_place() {
        let q = "How's the weather going to be over time?";
        assert!(looks_like_weather_query(q));
        assert!(extract_place(q).is_none());
        // Falls back to configured default place for short local asks.
        assert!(can_instant_weather(q));
        assert_ne!(
            resolve_weather_place(q).unwrap().to_lowercase(),
            "going to be over time"
        );
    }

    #[test]
    fn rejects_at_noon_as_place() {
        let q = "What's the temperature at noon?";
        assert!(looks_like_weather_query(q));
        assert!(extract_place(q).is_none());
        assert!(can_instant_weather(q)); // default place
    }

    #[test]
    fn still_extracts_place_after_at_city() {
        // Unusual phrasing but should still work when "at" introduces a place.
        let p = extract_place("weather at Barcelona today").unwrap();
        assert!(p.to_lowercase().contains("barcelona"), "{p}");
    }
}
