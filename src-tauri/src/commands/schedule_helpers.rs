//! Schedule argument parsing for the SCHEDULE tool.
//!
//! Extracted from `commands/ollama.rs` to keep modules small and cohesive.

use std::str::FromStr;

/// Result of parsing a SCHEDULE argument: either a recurring cron or a one-shot "at" datetime.
#[derive(Debug)]
pub(crate) enum ScheduleParseResult {
    Cron { cron_str: String, task: String },
    At { at_str: String, task: String },
}

/// Parse SCHEDULE argument. Supports:
/// - "every N minutes <task>"
/// - "at <datetime> <task>" (one-shot; datetime ISO or YYYY-MM-DD HH:MM)
/// - "<cron expression> <task>" (5- or 6-field; 5-field gets "0 " prepended)
pub(crate) fn parse_schedule_arg(arg: &str) -> Result<ScheduleParseResult, String> {
    let trimmed = arg.trim();
    let lower = trimmed.to_lowercase();
    let rest = lower.trim_start();

    // 1. "every N minutes <task>"
    if let Some(after_every) = rest.strip_prefix("every ") {
        let mut n_str = String::new();
        for c in after_every.chars() {
            if c.is_ascii_digit() {
                n_str.push(c);
            } else {
                break;
            }
        }
        let remainder = after_every[n_str.len()..].trim_start();
        if n_str.is_empty() {
            return Err("expected a number after 'every' (e.g. every 5 minutes)".to_string());
        }
        let n: u64 = n_str
            .parse()
            .map_err(|_| "expected integer after 'every'".to_string())?;
        if n == 0 {
            return Err("interval must be at least 1 minute".to_string());
        }
        if !remainder.to_lowercase().starts_with("minute") {
            return Err("expected 'minutes' after the number (e.g. every 5 minutes)".to_string());
        }
        let cron_str = format!("0 */{} * * * *", n);
        let task = trimmed.to_string();
        return Ok(ScheduleParseResult::Cron { cron_str, task });
    }

    // 2. "at <datetime> <task>" (one-shot)
    if let Some(after_at) = rest.strip_prefix("at ") {
        let after_at = after_at.trim_start();
        let tokens: Vec<&str> = after_at.split_whitespace().collect();
        if tokens.is_empty() {
            return Err(
                "at requires a datetime and task (e.g. at 2025-02-09T05:00:00 Remind me)"
                    .to_string(),
            );
        }
        // Try first token as ISO (2025-02-09T05:00:00)
        if tokens[0].contains('T') {
            if let Ok(dt) = parse_at_datetime(tokens[0]) {
                let task = tokens[1..].join(" ").trim().to_string();
                if task.is_empty() {
                    return Err("at requires a task description after the datetime".to_string());
                }
                return Ok(ScheduleParseResult::At { at_str: dt, task });
            }
        }
        // Try first two tokens as "YYYY-MM-DD HH:MM" or "YYYY-MM-DD HH:MM:SS"
        if tokens.len() >= 2 {
            let combined = format!("{} {}", tokens[0], tokens[1]);
            if let Ok(dt) = parse_at_datetime(&combined) {
                let task = tokens[2..].join(" ").trim().to_string();
                if task.is_empty() {
                    return Err("at requires a task description after the datetime".to_string());
                }
                return Ok(ScheduleParseResult::At { at_str: dt, task });
            }
        }
        return Err(
            "invalid at datetime: use YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM (local time)"
                .to_string(),
        );
    }

    // 3. Raw cron: first 5 or 6 space-separated tokens, then task
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for &n in &[6, 5] {
        if tokens.len() >= n {
            let cron_part: String = if n == 5 {
                format!("0 {}", tokens[..5].join(" "))
            } else {
                tokens[..6].join(" ")
            };
            if cron::Schedule::from_str(&cron_part).is_ok() {
                let task = tokens[n..].join(" ").trim().to_string();
                return Ok(ScheduleParseResult::Cron {
                    cron_str: cron_part,
                    task,
                });
            }
        }
    }

    Err("expected 'every N minutes <task>', 'at <datetime> <task>', or '<cron> <task>' (see SCHEDULE cron examples)".to_string())
}

/// Parse datetime for "at" one-shot. Returns ISO string for storage (local, no Z).
/// Rejects past times.
pub(crate) fn parse_at_datetime(s: &str) -> Result<String, String> {
    use chrono::{Local, TimeZone};
    let s = s.trim();
    let dt = chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Local))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").map(|n| {
                Local
                    .from_local_datetime(&n)
                    .single()
                    .unwrap_or_else(|| n.and_utc().with_timezone(&Local))
            })
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|n| {
                Local
                    .from_local_datetime(&n)
                    .single()
                    .unwrap_or_else(|| n.and_utc().with_timezone(&Local))
            })
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").map(|n| {
                Local
                    .from_local_datetime(&n)
                    .single()
                    .unwrap_or_else(|| n.and_utc().with_timezone(&Local))
            })
        })
        .map_err(|e| {
            format!(
                "invalid datetime: {} (use YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM)",
                e
            )
        })?;
    let now = Local::now();
    if dt < now {
        return Err("datetime must be in the future".to_string());
    }
    Ok(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
}
