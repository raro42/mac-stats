//! Deterministic pre-routing: skip the LLM planning step for unambiguous patterns.
//!
//! Screenshot + URL → BROWSER_SCREENSHOT; "run <command>" → RUN_CMD; ticket → REDMINE_API.

use tracing::info;

use crate::commands::redmine_helpers::{
    extract_ticket_id, is_redmine_time_entries_request, redmine_request_for_routing,
    redmine_time_entries_range,
};
use crate::commands::reply_helpers::{extract_last_prefixed_argument, extract_screenshot_recommendation};

/// Try to pre-route the question to a tool without asking the LLM.
///
/// Returns `Some(recommendation_string)` when the question unambiguously maps to a tool,
/// or `None` when the LLM planning step is needed.
pub(crate) fn compute_pre_routed_recommendation(
    question: &str,
    request_for_verification: &str,
    is_verification_retry: bool,
) -> Option<String> {
    extract_screenshot_recommendation(question).or_else(|| {
        let run_cmd_rec = try_pre_route_run_cmd(question);
        if run_cmd_rec.is_some() {
            return run_cmd_rec;
        }
        try_pre_route_redmine(question, request_for_verification, is_verification_retry)
    })
}

/// "run <command>" / "RUN_CMD: <command>" → `RUN_CMD: <arg>`.
fn try_pre_route_run_cmd(question: &str) -> Option<String> {
    if !crate::commands::run_cmd::is_local_cmd_allowed() {
        return None;
    }
    let q = question.trim();
    let q_lower = q.to_lowercase();
    let cmd_rest = if let Some(cmd) = extract_last_prefixed_argument(q, "RUN_CMD:") {
        cmd
    } else if q_lower.starts_with("run command:") {
        q[12..].trim().to_string()
    } else if q_lower.starts_with("run ") {
        q[4..].trim().to_string()
    } else {
        String::new()
    };
    if cmd_rest.is_empty() {
        return None;
    }
    let rec = format!("RUN_CMD: {}", cmd_rest);
    info!(
        "Agent router: pre-routed to RUN_CMD (run command): {}",
        crate::logging::ellipse(&cmd_rest, 60)
    );
    Some(rec)
}

/// Ticket / time-entries patterns → `REDMINE_API: GET /...`.
fn try_pre_route_redmine(
    question: &str,
    request_for_verification: &str,
    is_verification_retry: bool,
) -> Option<String> {
    if !crate::redmine::is_configured() {
        return None;
    }
    let q = question.trim();
    let redmine_request =
        redmine_request_for_routing(q, request_for_verification, is_verification_retry);
    let redmine_request_lower = redmine_request.to_lowercase();

    if is_redmine_time_entries_request(redmine_request) {
        let (from, to) = redmine_time_entries_range(redmine_request);
        let rec = format!(
            "REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100",
            from, to
        );
        info!(
            "Agent router: pre-routed to REDMINE_API for time entries ({}..{})",
            from, to
        );
        return Some(rec);
    }

    let ticket_id = extract_ticket_id(&redmine_request_lower);
    let wants_update = redmine_request_lower.contains("update")
        || redmine_request_lower.contains("add comment")
        || redmine_request_lower.contains("with the next steps")
        || redmine_request_lower.contains("post a comment")
        || redmine_request_lower.contains("write ")
        || redmine_request_lower.contains("put ");
    ticket_id
        .filter(|_| {
            redmine_request_lower.contains("ticket")
                || redmine_request_lower.contains("issue")
                || redmine_request_lower.contains("redmine")
        })
        .filter(|_| !wants_update)
        .map(|id| {
            let rec = format!(
                "REDMINE_API: GET /issues/{}.json?include=journals,attachments",
                id
            );
            info!("Agent router: pre-routed to REDMINE_API for ticket #{}", id);
            rec
        })
}
