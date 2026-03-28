//! Repair assistant / tool-result pairing in chat history before Ollama requests.
//!
//! mac-stats uses text-based tool lines in assistant messages and user/system messages
//! for results. Interrupted tool loops or corrupted session state can leave unpaired
//! tool calls or orphan result blobs; this module applies a lightweight linear pass.

use crate::commands::tool_parsing::parse_all_tools_from_response;
use crate::debug2;
use crate::ollama::ChatMessage;

const SYNTHETIC_TOOL_RESULT: &str = "[tool result unavailable — execution was interrupted]";

/// True if the assistant content includes at least one parsed tool other than `DONE`.
fn assistant_has_executable_tools(content: &str) -> bool {
    parse_all_tools_from_response(content)
        .iter()
        .any(|(t, _)| t != "DONE")
}

fn tool_names_for_log(content: &str) -> String {
    let names: Vec<_> = parse_all_tools_from_response(content)
        .into_iter()
        .filter(|(t, _)| t != "DONE")
        .map(|(t, _)| t)
        .collect();
    if names.is_empty() {
        "(none)".to_string()
    } else {
        names.join(", ")
    }
}

/// Heuristic: segment looks like output from our tool handlers (not a normal short user reply).
fn segment_looks_like_tool_result(seg: &str) -> bool {
    let s = seg.trim_start();
    if s.is_empty() {
        return false;
    }
    let head = s.lines().next().map(str::trim).unwrap_or("").to_uppercase();

    const STARTS: &[&str] = &[
        "HERE IS THE COMMAND OUTPUT",
        "HERE IS THE PAGE CONTENT",
        "MCP TOOL",
        "SEARCH RESULTS",
        "DISCORD API",
        "FETCH_URL ERROR:",
        "PYTHON SCRIPT",
        "UNKNOWN TOOL",
        "MAXIMUM BROWSER",
        "SAME BROWSER ACTION",
        "PAGE CHANGED BY",
        "CONTEXT OVERFLOW",
        "[TRUNCATED FROM",
        "[TOOL RESULT UNAVAILABLE",
    ];
    for p in STARTS {
        if head.starts_with(p) {
            return true;
        }
    }
    if s.contains("FETCH_URL error:") {
        return true;
    }
    // Browser / API style errors or structured replies
    if head.starts_with("BROWSER_") || head.starts_with("REDMINE_") || head.starts_with("TASK_") {
        return true;
    }
    false
}

fn content_looks_like_tool_bundle(content: &str) -> bool {
    let t = content.trim();
    if t.contains("\n\n---\n\n") {
        return t.split("\n\n---\n\n").any(segment_looks_like_tool_result);
    }
    segment_looks_like_tool_result(t)
}

/// User/system message plausibly carries tool output for a preceding assistant tool turn.
fn next_message_is_tool_followup(next: &ChatMessage) -> bool {
    let t = next.content.trim();
    if t.is_empty() {
        return false;
    }
    if t == SYNTHETIC_TOOL_RESULT || t.starts_with("[tool result unavailable") {
        return true;
    }
    match next.role.as_str() {
        "system" => true,
        "user" => {
            content_looks_like_tool_bundle(&next.content)
                || t.contains("\n\n---\n\n")
                || t.chars().count() >= 800
        }
        _ => false,
    }
}

fn strip_tool_like_segments(content: &str) -> String {
    if content.contains("\n\n---\n\n") {
        let kept = content
            .split("\n\n---\n\n")
            .filter(|seg| !segment_looks_like_tool_result(seg))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");
        return kept.trim().to_string();
    }
    if segment_looks_like_tool_result(content) {
        String::new()
    } else {
        content.to_string()
    }
}

/// Best-effort sanitization: pair assistant tool lines with follow-up results, insert
/// synthetic results when missing, and drop orphan tool-like user/system segments.
pub(crate) fn sanitize_conversation_history(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return messages;
    }

    let mut out: Vec<ChatMessage> = Vec::with_capacity(messages.len().saturating_add(4));
    let mut i = 0usize;

    while i < messages.len() {
        let mut msg = messages[i].clone();
        msg.content =
            crate::commands::directive_tags::strip_inline_directive_tags_for_display(&msg.content);

        if matches!(msg.role.as_str(), "user" | "system") && i > 0 {
            let prev = &messages[i - 1];
            let prev_had_tools =
                prev.role == "assistant" && assistant_has_executable_tools(&prev.content);
            if !prev_had_tools && content_looks_like_tool_bundle(&msg.content) {
                let stripped = strip_tool_like_segments(&msg.content);
                if stripped.trim().is_empty() {
                    debug2!(
                        "Sanitized history: dropped user/system message at index {} (orphan tool-like content, no prior assistant tool call)",
                        i
                    );
                    i += 1;
                    continue;
                }
                if stripped != msg.content {
                    debug2!(
                        "Sanitized history: removed orphan tool-like segment(s) from message at index {}",
                        i
                    );
                    msg.content = stripped;
                }
            }
        }

        if msg.role == "assistant" && assistant_has_executable_tools(&msg.content) {
            let next_ok = messages
                .get(i + 1)
                .map(next_message_is_tool_followup)
                .unwrap_or(false);
            if !next_ok {
                debug2!(
                    "Sanitized history: unpaired tool call(s) [{}] at assistant index {} — inserting synthetic tool result",
                    tool_names_for_log(&msg.content),
                    i
                );
                out.push(msg);
                out.push(ChatMessage {
                    role: "user".to_string(),
                    content: SYNTHETIC_TOOL_RESULT.to_string(),
                    images: None,
                });
                i += 1;
                continue;
            }
        }

        out.push(msg);
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_synthetic_after_unpaired_fetch() {
        let hist = vec![
            ChatMessage {
                role: "assistant".into(),
                content: "Fetching.\nFETCH_URL: https://example.com".into(),
                images: None,
            },
            ChatMessage {
                role: "user".into(),
                content: "What is the capital of France?".into(),
                images: None,
            },
        ];
        let out = sanitize_conversation_history(hist);
        assert_eq!(out.len(), 3);
        assert_eq!(out[1].role, "user");
        assert_eq!(out[1].content, SYNTHETIC_TOOL_RESULT);
    }

    #[test]
    fn keeps_paired_fetch_and_result() {
        let page = format!("Here is the page content:\n\n{}", "x".repeat(900));
        let hist = vec![
            ChatMessage {
                role: "assistant".into(),
                content: "FETCH_URL: https://example.com".into(),
                images: None,
            },
            ChatMessage {
                role: "user".into(),
                content: page,
                images: None,
            },
        ];
        let out = sanitize_conversation_history(hist);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn drops_orphan_tool_blob_after_non_tool_assistant() {
        let hist = vec![
            ChatMessage {
                role: "assistant".into(),
                content: "Hello, how can I help?".into(),
                images: None,
            },
            ChatMessage {
                role: "user".into(),
                content: "Here is the page content:\n\ntest body".into(),
                images: None,
            },
        ];
        let out = sanitize_conversation_history(hist);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].role, "assistant");
    }
}
