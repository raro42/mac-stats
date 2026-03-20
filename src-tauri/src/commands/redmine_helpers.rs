//! Redmine helper functions extracted from ollama.rs.
//!
//! Pure-logic helpers for Redmine ticket detection, time-entry parsing,
//! failure-message extraction, and date-range computation. No Ollama or
//! network dependencies — every function operates on strings/dates.

/// Extract a numeric ticket/issue ID from text like "ticket #1234", "#1234", "issue 1234", "review redmine 7209".
pub(crate) fn extract_ticket_id(text: &str) -> Option<u64> {
    if let Some(pos) = text.find('#') {
        let after = &text[pos + 1..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }
    for keyword in &["ticket ", "issue ", "redmine "] {
        if let Some(pos) = text.find(keyword) {
            let after = &text[pos + keyword.len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                return digits.parse().ok();
            }
        }
    }
    None
}

pub(crate) fn question_explicitly_requests_json(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("json")
        || q.contains("machine readable")
        || q.contains("structured output")
        || q.contains("structured data")
}

pub(crate) fn extract_redmine_time_entries_summary_for_reply(tool_result: &str) -> Option<String> {
    let start = tool_result.find("Derived Redmine time-entry summary")?;
    let mut summary = &tool_result[start..];
    for marker in [
        "\n\nUse this data to answer",
        "\n\nUse only this Redmine data to continue or answer",
    ] {
        if let Some(idx) = summary.find(marker) {
            summary = &summary[..idx];
            break;
        }
    }
    let summary = if let Some(idx) = summary.find("\n\nEntry details:\n") {
        &summary[..idx]
    } else {
        summary
    };
    let cleaned = summary.trim().to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

pub(crate) fn extract_redmine_failure_message(text: &str) -> Option<String> {
    for prefix in [
        "Redmine API failed:",
        "Redmine GET failed:",
        "Redmine request failed:",
    ] {
        if let Some(start) = text.find(prefix) {
            let rest = text[start + prefix.len()..].trim();
            let first_block = rest.split("\n\n---\n\n").next().unwrap_or(rest).trim();
            let first_block = first_block
                .split("\n\nUse ")
                .next()
                .unwrap_or(first_block)
                .trim();
            let without_instruction = first_block
                .strip_suffix("Answer without this result.")
                .unwrap_or(first_block)
                .trim();
            if !without_instruction.is_empty() {
                return Some(without_instruction.trim_end_matches('.').trim().to_string());
            }
        }
    }
    None
}

pub(crate) fn is_redmine_infrastructure_failure_text(text: &str) -> bool {
    let t = text.to_lowercase();
    t.contains("redmine not configured")
        || t.contains("redmine_url missing")
        || t.contains("redmine_api_key missing")
        || t.contains("invalid url")
        || t.contains("dns")
        || t.contains("failed to lookup address")
        || t.contains("failed to lookup address information")
        || t.contains("name or service not known")
        || t.contains("nodename nor servname provided")
        || t.contains("no address associated with hostname")
        || t.contains("could not resolve host")
        || t.contains("connection refused")
        || t.contains("unreachable")
}

pub(crate) fn format_redmine_time_entries_period(question: &str) -> String {
    let (from, to) = redmine_time_entries_range(question);
    if from == to {
        from
    } else {
        format!("{}..{}", from, to)
    }
}

pub(crate) fn grounded_redmine_time_entries_failure_reply(
    question: &str,
    text: &str,
) -> Option<String> {
    if !is_redmine_time_entries_request(question) {
        return None;
    }

    let failure = extract_redmine_failure_message(text)?;
    if !is_redmine_infrastructure_failure_text(&failure) {
        return None;
    }

    let failure_lower = failure.to_lowercase();
    if failure_lower.contains("no time entries")
        || failure_lower.contains("no worked tickets")
        || failure_lower.contains("tickets were found")
    {
        return None;
    }

    let blocker = if failure_lower.contains("redmine not configured")
        || failure_lower.contains("redmine_url missing")
        || failure_lower.contains("redmine_api_key missing")
    {
        "Redmine is not configured on this machine."
    } else if failure_lower.contains("invalid url") {
        "the configured Redmine URL is invalid."
    } else if failure_lower.contains("dns")
        || failure_lower.contains("failed to lookup address")
        || failure_lower.contains("failed to lookup address information")
        || failure_lower.contains("name or service not known")
        || failure_lower.contains("nodename nor servname provided")
        || failure_lower.contains("no address associated with hostname")
        || failure_lower.contains("could not resolve host")
    {
        "the configured Redmine host could not be resolved."
    } else {
        "the configured Redmine host could not be reached."
    };

    Some(format!(
        "Could not retrieve Redmine time entries for {} because {} No Redmine data was fetched. Fix the Redmine configuration or connectivity, then retry.",
        format_redmine_time_entries_period(question),
        blocker
    ))
}

pub(crate) fn is_grounded_redmine_time_entries_blocked_reply(
    question: &str,
    response_content: &str,
) -> bool {
    if !is_redmine_time_entries_request(question) {
        return false;
    }

    let t = response_content.to_lowercase();
    let mentions_blocked_fetch = t.contains("could not retrieve redmine time entries")
        || (t.contains("redmine api failed")
            && is_redmine_infrastructure_failure_text(response_content));
    let mentions_infra_blocker = is_redmine_infrastructure_failure_text(response_content)
        || t.contains("no redmine data was fetched");
    let invents_empty_result = t.contains("no time entries or tickets were found")
        || t.contains("no time entries were found")
        || t.contains("no worked tickets were found")
        || t.contains("tickets were found for that period");

    mentions_blocked_fetch && mentions_infra_blocker && !invents_empty_result
}

/// True when the user asked only to review or summarize a Redmine ticket (no update, add comment, or close).
/// Used to avoid injecting PUT hint and to narrow success criteria so verification does not require ticket changes.
pub(crate) fn is_redmine_review_or_summarize_only(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    let has_redmine_ticket = (q.contains("redmine") || q.contains("ticket") || q.contains("issue"))
        && extract_ticket_id(question).is_some();
    let review_or_summarize =
        q.contains("review") || q.contains("summarize") || q.contains("summarise");
    let no_mutate = !q.contains("update")
        && !q.contains("add comment")
        && !q.contains("post a comment")
        && !q.contains("close")
        && !q.contains("resolve")
        && !q.contains("write ");
    has_redmine_ticket && review_or_summarize && no_mutate
}

pub(crate) fn is_redmine_relative_day_request(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    q.contains("today") || q.contains("yesterday") || q.contains("yestaerday")
}

pub(crate) fn is_redmine_yesterday_request(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    q.contains("yesterday") || q.contains("yestaerday")
}

pub(crate) fn is_redmine_time_entries_request(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    let mentions_redmine = q.contains("redmine");
    let mentions_time_entries = q.contains("time entries")
        || q.contains("spent time")
        || q.contains("hours this month")
        || q.contains("hours worked")
        || q.contains("time logs")
        || q.contains("tickets worked")
        || q.contains("worked tickets")
        || (q.contains("worked on") && q.contains("month"))
        || (q.contains("worked on") && is_redmine_relative_day_request(&q))
        || (q.contains("work on") && is_redmine_relative_day_request(&q))
        || (q.contains("work today") && q.contains("ticket"))
        || (q.contains("work yesterday") && q.contains("ticket"))
        || (q.contains("worked") && is_redmine_relative_day_request(&q) && q.contains("ticket"));
    mentions_redmine && mentions_time_entries
}

pub(crate) fn redmine_time_entries_range_for_date(
    question: &str,
    today: chrono::NaiveDate,
) -> (String, String) {
    use chrono::Datelike;

    let q = question.trim().to_lowercase();
    if is_redmine_yesterday_request(&q) {
        let day = today
            .pred_opt()
            .unwrap_or(today)
            .format("%Y-%m-%d")
            .to_string();
        return (day.clone(), day);
    }
    if q.contains("today") {
        let day = today.format("%Y-%m-%d").to_string();
        return (day.clone(), day);
    }
    let from = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .unwrap_or(today)
        .format("%Y-%m-%d")
        .to_string();
    let (next_year, next_month) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let next_month_start =
        chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap_or(today);
    let to = next_month_start
        .pred_opt()
        .unwrap_or(today)
        .format("%Y-%m-%d")
        .to_string();
    (from, to)
}

pub(crate) fn redmine_time_entries_range(question: &str) -> (String, String) {
    redmine_time_entries_range_for_date(question, chrono::Utc::now().date_naive())
}

pub(crate) fn redmine_request_for_routing<'a>(
    question: &'a str,
    request_for_verification: &'a str,
    is_verification_retry: bool,
) -> &'a str {
    if is_verification_retry
        && (is_redmine_time_entries_request(request_for_verification)
            || is_redmine_review_or_summarize_only(request_for_verification))
    {
        request_for_verification
    } else {
        question
    }
}

pub(crate) fn redmine_direct_fallback_hint(question: &str) -> String {
    if is_redmine_time_entries_request(question) {
        let (from, to) = redmine_time_entries_range(question);
        format!(
            "Use REDMINE_API directly with concrete dates: REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100.",
            from, to
        )
    } else if let Some(id) = extract_ticket_id(question) {
        format!(
            "Use REDMINE_API directly: REDMINE_API: GET /issues/{}.json?include=journals,attachments.",
            id
        )
    } else {
        "Use REDMINE_API directly with the correct concrete endpoint for this request.".to_string()
    }
}

// ── tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_redmine_time_entries_failure_reply_for_dns_blocker_is_user_facing() {
        let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let text = format!(
            "Redmine API result:\n\nRedmine GET failed: error sending request for url (https://example.invalid/time_entries.json?from={d}&to={d}&offset=0&limit=100): error trying to connect: dns error: failed to lookup address information: nodename nor servname provided, or not known\n\nUse this data to answer the user's question.",
            d = today
        );
        let reply = grounded_redmine_time_entries_failure_reply(
            "Provide me the list of redmine tickets work on today.",
            &text,
        )
        .expect("expected grounded failure reply");

        assert!(reply.contains(&format!("Could not retrieve Redmine time entries for {}", today)));
        assert!(reply.contains("configured Redmine host could not be resolved"));
        assert!(reply.contains("No Redmine data was fetched"));
        assert!(!reply.contains("no time entries were found"));
    }

    #[test]
    fn verification_accepts_grounded_redmine_blocked_reply() {
        let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let reply = format!(
            "Could not retrieve Redmine time entries for {} because the configured Redmine host could not be resolved. No Redmine data was fetched. Fix the Redmine configuration or connectivity, then retry.",
            today
        );
        assert!(is_grounded_redmine_time_entries_blocked_reply(
            "Provide me the list of redmine tickets work on today.",
            &reply
        ));
    }

    #[test]
    fn redmine_direct_fallback_hint_for_today_avoids_user_id_me() {
        let hint =
            redmine_direct_fallback_hint("List redmine tickets that have been worked on today");
        assert!(hint.contains("/time_entries.json?from="));
        assert!(hint.contains("&to="));
        assert!(hint.contains("&limit=100"));
        assert!(!hint.contains("user_id=me"));
    }

    #[test]
    fn redmine_request_for_routing_prefers_original_request_on_retry() {
        assert_eq!(
            redmine_request_for_routing(
                "Verification said we didn't fully complete. Re-fetch the same period if needed.",
                "Provide me the list of redmine tickets work on today.",
                true,
            ),
            "Provide me the list of redmine tickets work on today."
        );
    }

    #[test]
    fn redmine_request_for_routing_keeps_today_window_on_retry() {
        let routed = redmine_request_for_routing(
            "Verification said we didn't fully complete. Re-fetch the same period if needed.",
            "Provide me the list of redmine tickets work on today.",
            true,
        );
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
        assert_eq!(
            redmine_time_entries_range_for_date(routed, today),
            ("2026-03-07".to_string(), "2026-03-07".to_string())
        );
    }

    #[test]
    fn redmine_time_entries_range_for_date_uses_utc_day_for_today_queries() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        assert_eq!(
            redmine_time_entries_range_for_date(
                "Provide me the list of redmine tickets work on today.",
                today
            ),
            ("2026-03-06".to_string(), "2026-03-06".to_string())
        );
    }

    #[test]
    fn redmine_time_entries_range_for_date_uses_previous_utc_day_for_yesterday_queries() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        assert_eq!(
            redmine_time_entries_range_for_date(
                "Give me a list of Redmine tickets worked on yestaerday.",
                today
            ),
            ("2026-03-05".to_string(), "2026-03-05".to_string())
        );
    }

    #[test]
    fn extract_ticket_id_from_hash_notation() {
        assert_eq!(extract_ticket_id("review #7209"), Some(7209));
    }

    #[test]
    fn extract_ticket_id_from_keyword() {
        assert_eq!(extract_ticket_id("review redmine 7209"), Some(7209));
        assert_eq!(extract_ticket_id("ticket 1234 details"), Some(1234));
    }

    #[test]
    fn extract_ticket_id_none_when_absent() {
        assert_eq!(extract_ticket_id("what is the weather"), None);
    }

    #[test]
    fn question_explicitly_requests_json_positive() {
        assert!(question_explicitly_requests_json("give me json output"));
        assert!(question_explicitly_requests_json("structured data please"));
    }

    #[test]
    fn question_explicitly_requests_json_negative() {
        assert!(!question_explicitly_requests_json("review redmine ticket 7209"));
    }
}
