//! Optional post-run agent judge: after an agent run completes, call an LLM to evaluate
//! whether the task was satisfied. Used for testing or quality logging when enabled via config.
//! Does not affect the agent loop or user-facing replies.

use std::path::PathBuf;
use tracing::{info, warn};

const TASK_TRUNCATE_CHARS: usize = 40_000;
const RESULT_TRUNCATE_CHARS: usize = 40_000;
const MAX_SCREENSHOT_PATHS: usize = 10;

const JUDGE_SYSTEM_PROMPT: &str = r#"You are an evaluator for an AI agent run. Given the original task, the agent's final reply, optional step summaries, and optional screenshot paths, decide whether the task was satisfied.

Output valid JSON only, with these fields:
- verdict (boolean): true if the task was satisfied, false otherwise
- score (number, optional): 0-1 quality score
- reasoning (string): brief explanation
- impossible_task (boolean, optional): true if the task was impossible or ill-defined
- reached_captcha (boolean, optional): true if the agent hit a captcha or block

If ground_truth is provided, treat it as the highest-priority success criteria. No other text outside the JSON."#;

/// Structured verdict from the judge LLM.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct JudgeVerdict {
    pub verdict: Option<bool>,
    pub score: Option<f64>,
    pub reasoning: Option<String>,
    pub impossible_task: Option<bool>,
    pub reached_captcha: Option<bool>,
}

fn truncate(s: &str, max_chars: usize) -> String {
    let t: String = s.chars().take(max_chars).collect();
    if s.chars().count() > max_chars {
        format!("{}… [truncated]", t)
    } else {
        t
    }
}

/// Build user message for the judge from task, result, step summaries, and screenshot paths.
fn build_judge_user_message(
    task: &str,
    result: &str,
    step_summaries: &[String],
    screenshot_paths: &[PathBuf],
    ground_truth: Option<&str>,
) -> String {
    let task_trunc = truncate(task, TASK_TRUNCATE_CHARS);
    let result_trunc = truncate(result, RESULT_TRUNCATE_CHARS);
    let steps: String = if step_summaries.is_empty() {
        "(no step-by-step history)".to_string()
    } else {
        step_summaries
            .iter()
            .enumerate()
            .map(|(i, s)| format!("Step {}: {}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let screens: String = screenshot_paths
        .iter()
        .take(MAX_SCREENSHOT_PATHS)
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let gt_block = match ground_truth {
        Some(gt) => format!(
            "\n\nGround truth (highest-priority criteria):\n{}",
            truncate(gt, 2000)
        ),
        None => String::new(),
    };
    format!(
        r#"Task:
{}

Agent result:
{}

Step summaries:
{}

Screenshot paths (last {}):
{}
{}"#,
        task_trunc,
        result_trunc,
        steps,
        MAX_SCREENSHOT_PATHS,
        if screens.is_empty() {
            "(none)"
        } else {
            &screens
        },
        gt_block
    )
}

/// Call the judge LLM and return a structured verdict. On parse failure returns a default verdict and logs.
pub async fn run_judge(
    task: &str,
    result: &str,
    step_summaries: &[String],
    screenshot_paths: &[PathBuf],
    ground_truth: Option<&str>,
) -> JudgeVerdict {
    let user_content =
        build_judge_user_message(task, result, step_summaries, screenshot_paths, ground_truth);
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: JUDGE_SYSTEM_PROMPT.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_content,
            images: None,
        },
    ];
    let response =
        match crate::commands::ollama::send_ollama_chat_messages(
            messages,
            None,
            None,
            crate::commands::ollama::OllamaHttpQueue::Acquire {
                key: "judge".to_string(),
                wait_hook: None,
            },
        )
        .await {
            Ok(r) => r,
            Err(e) => {
                warn!("Agent judge: LLM call failed: {}", e);
                return JudgeVerdict::default();
            }
        };
    let raw = response.message.content.trim();
    // Try to extract JSON from the response (model might wrap in markdown).
    let json_str = if let Some(start) = raw.find('{') {
        if let Some(end) = raw.rfind('}') {
            &raw[start..=end]
        } else {
            raw
        }
    } else {
        raw
    };
    match serde_json::from_str::<JudgeVerdict>(json_str) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "Agent judge: failed to parse verdict JSON: {}; raw: {}",
                e,
                truncate(json_str, 500)
            );
            JudgeVerdict::default()
        }
    }
}

/// If agent judge is enabled in config, run the judge and log verdict (and optional reasoning) to the debug log.
pub async fn run_judge_if_enabled(
    task: &str,
    result: &str,
    attachment_paths: &[PathBuf],
    ground_truth: Option<&str>,
) {
    if !crate::config::Config::agent_judge_enabled() {
        return;
    }
    let step_summaries: Vec<String> = vec![];
    let verdict = run_judge(
        task,
        result,
        &step_summaries,
        attachment_paths,
        ground_truth,
    )
    .await;
    let v = verdict.verdict.unwrap_or(false);
    let reason = verdict
        .reasoning
        .as_deref()
        .unwrap_or("(no reasoning)")
        .trim();
    let impossible = verdict.impossible_task.unwrap_or(false);
    let captcha = verdict.reached_captcha.unwrap_or(false);
    info!(
        "Agent judge: verdict={} reasoning={} impossible_task={} reached_captcha={}",
        v, reason, impossible, captcha
    );
    if let Some(s) = verdict.score {
        info!("Agent judge: score={}", s);
    }
}
