//! Browser task detection, navigation-target extraction, and retry-prompt helpers.
//!
//! Extracted from `commands/ollama.rs` to keep modules small and cohesive.

pub(crate) fn is_video_review_request(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("video") && (q.contains("review") || q.contains("check"))
}

pub(crate) fn explicit_no_playable_video_finding(response_summary: &str) -> bool {
    let lower = response_summary.to_lowercase();
    (lower.contains("no playable video")
        || lower.contains("videos aren't available")
        || lower.contains("videos are not available")
        || lower.contains("doesn't navigate anywhere")
        || lower.contains("do not lead to playable videos")
        || lower.contains("lacks embedded video content")
        || lower.contains("video availability"))
        && (lower.contains("video") || lower.contains("videos"))
}

pub(crate) fn is_browser_navigation_target_token(token: &str) -> bool {
    let candidate = token.trim().trim_end_matches(['.', ',', ';', ':']);
    if candidate.is_empty() || candidate.contains(char::is_whitespace) {
        return false;
    }
    let lower = candidate.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "to" | "the"
            | "a"
            | "an"
            | "current"
            | "current-page"
            | "page"
            | "url"
            | "link"
            | "video"
            | "videos"
    ) {
        return false;
    }
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("file://")
        || lower.starts_with("www.")
        || lower == "localhost"
        || lower.starts_with("localhost:")
        || lower.starts_with("127.0.0.1")
    {
        return true;
    }
    let host = lower
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(lower.as_str());
    host.contains('.')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':' | '[' | ']'))
}

pub(crate) fn extract_browser_navigation_target(arg: &str) -> Option<String> {
    let trimmed = arg.trim();
    let first = trimmed.split_whitespace().next()?;
    if is_browser_navigation_target_token(first) {
        Some(first.trim_end_matches(['.', ',', ';', ':']).to_string())
    } else {
        None
    }
}

pub(crate) fn append_latest_browser_state_guidance(message: &str) -> String {
    let base = format!(
        "{} Use the latest browser `Current page` / `Elements` output for the next step; do not reuse stale indices or invent a new URL.",
        message.trim_end()
    );
    if let Some(snapshot) = crate::browser_agent::get_last_browser_state_snapshot() {
        format!("{}\n\nLatest browser state:\n{}", base, snapshot.trim_end())
    } else {
        base
    }
}

pub(crate) fn browser_retry_grounding_prompt(request: &str, retry_base: &str) -> String {
    let mut prompt = format!(
        "Original request: \"{}\". Continue from the latest real browser state already returned in this conversation. Reuse the most recent `Current page` / `Elements` list, do not reuse stale indices, and do not invent a URL or claim a site error from an agent-generated browser argument. If the page already shows in-page content, inspect that content or click a real listed element. If no grounded browser action is available, reply with a brief limitation and **DONE: no**.\n\n{}",
        request.trim(),
        retry_base
    );
    if let Some(snapshot) = crate::browser_agent::get_last_browser_state_snapshot() {
        prompt.push_str("\n\nLatest browser state:\n");
        prompt.push_str(snapshot.trim_end());
    }
    prompt
}

pub(crate) fn is_browser_task_request(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("browser")
        || q.contains("screenshot")
        || q.contains("click ")
        || q.contains("navigate ")
        || q.contains("open ")
        || q.contains("cookie")
        || q.contains("consent")
        || q.contains("video")
}

/// True when the request looks like a coding task (implement, refactor, fix, write code, etc.).
/// Kept for tests and optional future use (e.g. logging or handoff hints); verification fallback no longer restricts by this.
#[allow(dead_code)]
pub(crate) fn is_coding_like_request(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("implement")
        || q.contains("refactor")
        || q.contains("fix ")
        || q.contains("fix bug")
        || q.contains("write code")
        || q.contains("create file")
        || q.contains("add feature")
        || q.contains("code change")
        || q.contains("organize")
        || (q.contains("folder") && (q.contains("organize") || q.contains("structure")))
        || (q.contains("make ")
            && (q.contains("change") || q.contains("edit"))
            && q.contains("project"))
        || q.contains("cursor-agent")
        || q.contains("cursor agent")
}

pub(crate) fn should_use_http_fallback_after_browser_action_error(tool: &str, err: &str) -> bool {
    let trimmed = err.trim();
    if trimmed.starts_with(tool)
        || trimmed.contains("index out of range")
        || trimmed.contains("requires a numeric index")
        || trimmed.contains("must be >=")
        || trimmed.contains("current page is a new tab")
    {
        return false;
    }
    true
}

/// True when CDP navigation exceeded the configured wait (`navigation_timeout_error_with_proxy_hint` in browser_agent).
/// In that case we must not run HTTP `navigate_http`, which would mask a real CDP stall with a separate fetch result.
pub(crate) fn is_cdp_navigation_timeout_error(err: &str) -> bool {
    err.contains("Navigation failed: timeout after")
}

/// True if the question explicitly asks for a visible browser (e.g. "show me the browser", "visible", "I want to see").
pub(crate) fn wants_visible_browser(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("visible")
        || q.contains("show me the browser")
        || q.contains("show me a browser")
        || q.contains("i want to see the browser")
        || q.contains("open a window")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_navigation_target_accepts_real_urls() {
        assert!(is_browser_navigation_target_token(
            "https://www.amvara.de/about"
        ));
        assert!(is_browser_navigation_target_token("www.example.com"));
        assert_eq!(
            extract_browser_navigation_target("https://www.amvara.de/about and inspect videos"),
            Some("https://www.amvara.de/about".to_string())
        );
    }

    #[test]
    fn browser_navigation_target_accepts_file_url() {
        assert!(is_browser_navigation_target_token(
            "file:///tmp/browser-input-routing.html"
        ));
        assert_eq!(
            extract_browser_navigation_target(
                "file:///Users/example/mac-stats/docs/fixtures/browser-input-routing.html then test"
            ),
            Some("file:///Users/example/mac-stats/docs/fixtures/browser-input-routing.html".to_string())
        );
    }

    #[test]
    fn browser_navigation_target_rejects_placeholder_words() {
        assert!(!is_browser_navigation_target_token("to"));
        assert!(!is_browser_navigation_target_token("video"));
        assert_eq!(extract_browser_navigation_target("to the video URL"), None);
    }

    #[test]
    fn browser_retry_prompt_keeps_browser_grounding_rules() {
        let prompt = browser_retry_grounding_prompt(
            "Use browser to review www.amvara.de, click on about and review videos.",
            "Verification said the task was incomplete.",
        );
        assert!(prompt.contains("latest real browser state"));
        assert!(prompt.contains("do not reuse stale indices"));
        assert!(prompt.contains("do not invent a URL"));
    }

    #[test]
    fn browser_task_request_detects_browser_style_questions() {
        assert!(is_browser_task_request(
            "Use browser to review www.amvara.de, click on about and review videos."
        ));
        assert!(!is_browser_task_request(
            "Provide me the list of redmine tickets work on today."
        ));
    }

    #[test]
    fn explicit_no_playable_video_finding_detects_grounded_browser_review_result() {
        assert!(explicit_no_playable_video_finding(
            "The \"Amvara's videos\" link is present but doesn't navigate anywhere. No playable videos were found on the page."
        ));
    }

    #[test]
    fn is_coding_like_request_detects_implement_refactor_fix() {
        assert!(is_coding_like_request("Implement a login form"));
        assert!(is_coding_like_request("Refactor the auth module"));
        assert!(is_coding_like_request("Fix the bug in parser"));
        assert!(is_coding_like_request("Add feature: dark mode"));
        assert!(is_coding_like_request("Use cursor-agent to add tests"));
        assert!(!is_coding_like_request("What's the weather today?"));
        assert!(!is_coding_like_request("List Redmine tickets"));
    }

    #[test]
    fn cdp_navigation_timeout_detection_matches_tool_errors() {
        assert!(is_cdp_navigation_timeout_error(
            "Navigation failed: timeout after 5s. If Chrome uses a proxy"
        ));
        assert!(!is_cdp_navigation_timeout_error(
            "Navigation failed: the target page did not load."
        ));
    }

    /// HTTP-fallback-only / combined-failure paths pass `cdp_used=false` so the model is not told a WS existed.
    #[test]
    fn http_only_browser_error_context_uses_cdp_not_used_label() {
        assert_eq!(
            crate::browser_agent::format_last_browser_error_context(false, None).as_deref(),
            Some("context: cdp=not_used")
        );
        assert_eq!(
            crate::browser_agent::format_last_browser_error_context(false, Some(false)).as_deref(),
            Some("context: cdp=not_used navchg=0")
        );
    }
}
