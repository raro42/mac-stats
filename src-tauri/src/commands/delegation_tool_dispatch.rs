//! Delegation and execution tool dispatch handlers for the agent router tool loop.
//!
//! Contains: AGENT, SKILL, RUN_JS, RUN_CMD, PYTHON_SCRIPT.
//! Extracted from `commands/ollama.rs` to keep modules small and cohesive.

use tracing::info;

use crate::commands::agent_session::run_agent_ollama_session;
use crate::commands::content_reduction::run_skill_ollama_session;
use crate::commands::ollama::send_ollama_chat_messages;
use crate::commands::redmine_helpers::redmine_direct_fallback_hint;
use crate::commands::reply_helpers::is_agent_unavailable_error;
use crate::commands::tool_parsing::{parse_python_script_from_response, parse_tool_from_response};

fn send_status(tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>, msg: &str) {
    if let Some(tx) = tx {
        let _ = tx.send(msg.to_string());
    }
}

// ── AGENT result struct ──────────────────────────────────────────────────

pub(crate) struct AgentResult {
    pub message: String,
    /// When an agent session produced a result, stores `(label, reply)` for conversation transcript.
    pub agent_conversation_entry: Option<(String, String)>,
}

// ── AGENT ────────────────────────────────────────────────────────────────

pub(crate) async fn handle_agent(
    arg: &str,
    question: &str,
    discord_reply_channel_id: Option<u64>,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
    load_global_memory: bool,
    last_message_content: Option<&str>,
) -> AgentResult {
    let arg = arg.trim();
    let (selector, task_message) = if let Some(space_idx) = arg.find(' ') {
        let (sel, rest) = arg.split_at(space_idx);
        (sel.trim(), rest.trim())
    } else {
        (arg, "")
    };

    let selector_lower = selector.to_lowercase();

    // Proxy agent: cursor-agent runs the CLI instead of Ollama.
    if (selector_lower == "cursor-agent" || selector_lower == "cursor_agent")
        && crate::commands::cursor_agent::is_cursor_agent_available()
    {
        let prompt = if task_message.is_empty() {
            question.to_string()
        } else {
            task_message.to_string()
        };
        send_status(status_tx, "Running Cursor Agent…");
        info!(
            "Agent router: AGENT cursor-agent proxy (prompt {} chars)",
            prompt.len()
        );
        let prompt_clone = prompt.clone();
        let message = match tokio::task::spawn_blocking(move || {
            crate::commands::cursor_agent::run_cursor_agent(&prompt_clone)
        })
        .await
        .map_err(|e| format!("Cursor Agent task: {}", e))
        .and_then(|r| r)
        {
            Ok(result) => {
                info!(
                    "Agent router: cursor-agent proxy completed ({} chars)",
                    result.len()
                );
                format!(
                    "Cursor Agent (proxy) result:\n\n{}\n\nUse this to answer the user's question.",
                    result.trim()
                )
            }
            Err(e) => {
                info!("Agent router: cursor-agent proxy failed: {}", e);
                format!(
                    "AGENT cursor-agent failed: {}. Answer without this result.",
                    e
                )
            }
        };
        return AgentResult {
            message,
            agent_conversation_entry: None,
        };
    }
    if selector_lower == "cursor-agent" || selector_lower == "cursor_agent" {
        return AgentResult {
            message: "Cursor Agent is not available (cursor-agent CLI not on PATH). Answer without it.".to_string(),
            agent_conversation_entry: None,
        };
    }

    let agents = crate::agents::load_agents();
    let agent = match crate::agents::find_agent_by_id_or_name(&agents, selector) {
        Some(a) => a,
        None => {
            let list: String = agents
                .iter()
                .map(|a| {
                    a.slug.as_deref().unwrap_or(a.name.as_str()).to_string()
                })
                .collect::<Vec<_>>()
                .join(", ");
            info!(
                "Agent router: AGENT unknown selector \"{}\" (available: {})",
                selector, list
            );
            return AgentResult {
                message: format!(
                    "Unknown agent \"{}\". Available agents: {}. Answer without using an agent.",
                    selector, list
                ),
                agent_conversation_entry: None,
            };
        }
    };

    // Break out of AGENT: orchestrator loop when the last message was already an orchestrator result.
    let is_orchestrator = agent.id == "000";
    let last_is_orchestrator_result = last_message_content
        .is_some_and(|c| c.starts_with("Agent \"Orchestrator\""));
    if is_orchestrator && last_is_orchestrator_result {
        info!("Agent router: skipping repeated AGENT: orchestrator (loop breaker)");
        return AgentResult {
            message: "The orchestrator already replied above. Reply with a one-sentence summary for the user and **DONE: success** or **DONE: no**. Do not output AGENT: orchestrator again.".to_string(),
            agent_conversation_entry: None,
        };
    }

    let mut user_msg: String = if task_message.is_empty() {
        question.to_string()
    } else {
        task_message.to_string()
    };

    // Inject Discord guild/channel metadata for discord-expert agent.
    let is_discord_expert = agent
        .slug
        .as_deref()
        .is_some_and(|s| s.eq_ignore_ascii_case("discord-expert"))
        || agent.id == "004";
    let is_redmine_agent = agent
        .slug
        .as_deref()
        .is_some_and(|s| s.eq_ignore_ascii_case("redmine"))
        || agent.id == "006";

    if is_discord_expert {
        if let Some(channel_id) = discord_reply_channel_id {
            send_status(status_tx, "Fetching Discord guild/channel context…");
            match crate::discord::api::fetch_guild_channel_metadata(channel_id).await {
                Ok(meta) => {
                    user_msg = format!(
                        "Current Discord context (use these IDs in DISCORD_API calls):\n{}\n\nUser request: {}",
                        meta, user_msg
                    );
                    info!(
                        "Agent router: injected Discord guild/channel metadata for discord-expert (channel {})",
                        channel_id
                    );
                }
                Err(e) => {
                    tracing::debug!(
                        "Agent router: Discord metadata fetch failed (channel {}): {}",
                        channel_id,
                        e
                    );
                }
            }
        }
    }

    const STATUS_MSG_MAX: usize = 120;
    let preview: String = user_msg.chars().take(STATUS_MSG_MAX).collect();
    let status_text = if user_msg.chars().count() > STATUS_MSG_MAX {
        format!("{}…", preview)
    } else {
        preview
    };
    send_status(
        status_tx,
        &format!("{} -> Ollama: {}", agent.name, status_text),
    );

    match run_agent_ollama_session(agent, &user_msg, status_tx, load_global_memory).await {
        Ok(result) => {
            let label = format!("{} ({})", agent.name, agent.id);
            let entry = Some((label.clone(), result.trim().to_string()));
            AgentResult {
                message: format!(
                    "Agent \"{}\" ({}) result:\n\n{}\n\nUse this to answer the user's question.",
                    agent.name, agent.id, result
                ),
                agent_conversation_entry: entry,
            }
        }
        Err(e) => {
            info!("Agent router: AGENT session failed: {}", e);
            let message = if is_redmine_agent && is_agent_unavailable_error(&e) {
                format!(
                    "Agent \"{}\" ({}) failed: {}.\n\nRe-plan this request without AGENT: redmine. {} Do not use FETCH_URL and do not reply with only another RUN_CMD.",
                    agent.name,
                    agent.id,
                    e,
                    redmine_direct_fallback_hint(question)
                )
            } else {
                format!(
                    "Agent \"{}\" ({}) failed: {}. Answer without this result.",
                    agent.name, agent.id, e
                )
            };
            AgentResult {
                message,
                agent_conversation_entry: None,
            }
        }
    }
}

// ── SKILL ────────────────────────────────────────────────────────────────

pub(crate) async fn handle_skill(
    arg: &str,
    question: &str,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    send_status(status_tx, "Using skill…");
    let arg = arg.trim();
    let (selector, task_message) = if let Some(space_idx) = arg.find(' ') {
        let (sel, rest) = arg.split_at(space_idx);
        (sel.trim(), rest.trim())
    } else {
        (arg, "")
    };

    let skills = crate::skills::load_skills();
    match crate::skills::find_skill_by_number_or_topic(&skills, selector) {
        Some(skill) => {
            send_status(
                status_tx,
                &format!("Using skill {}-{}…", skill.number, skill.topic),
            );
            info!(
                "Agent router: using skill {} ({}) — new session (no main context)",
                skill.number, skill.topic
            );
            let user_msg = if task_message.is_empty() {
                question
            } else {
                task_message
            };
            match run_skill_ollama_session(
                &skill.content,
                user_msg,
                model_override,
                options_override,
            )
            .await
            {
                Ok(result) => format!(
                    "Skill \"{}-{}\" result:\n\n{}\n\nUse this to answer the user's question.",
                    skill.number, skill.topic, result
                ),
                Err(e) => {
                    info!("Agent router: SKILL session failed: {}", e);
                    format!(
                        "Skill \"{}-{}\" failed: {}. Answer without this result.",
                        skill.number, skill.topic, e
                    )
                }
            }
        }
        None => {
            info!(
                "Agent router: SKILL unknown selector \"{}\" (available: {:?})",
                selector,
                skills
                    .iter()
                    .map(|s| format!("{}-{}", s.number, s.topic))
                    .collect::<Vec<_>>()
            );
            format!(
                "Unknown skill \"{}\". Available skills: {}. Answer without using a skill.",
                selector,
                skills
                    .iter()
                    .map(|s| format!("{}-{}", s.number, s.topic))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

// ── RUN_JS ───────────────────────────────────────────────────────────────

pub(crate) fn handle_run_js(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    use crate::commands::content_reduction::run_js_via_node;

    const CODE_PREVIEW_LEN: usize = 50;
    let code_preview: String = arg
        .trim()
        .lines()
        .next()
        .unwrap_or(arg.trim())
        .chars()
        .take(CODE_PREVIEW_LEN)
        .collect();
    let code_label = if arg.trim().chars().count() > CODE_PREVIEW_LEN {
        format!("{}…", code_preview.trim())
    } else {
        code_preview.trim().to_string()
    };
    let code_ref = if code_label.is_empty() {
        "…"
    } else {
        &code_label
    };
    send_status(status_tx, &format!("Running code: {}…", code_ref));

    let code_len = arg.chars().count();
    let code_hash = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        arg.hash(&mut h);
        h.finish()
    };
    info!(
        "RUN_JS audit: len={} hash_hex={:016x} preview={}",
        code_len, code_hash, code_ref
    );

    match run_js_via_node(arg) {
        Ok(result) => format!(
            "JavaScript result:\n\n{}\n\nUse this to answer the user's question.",
            result
        ),
        Err(e) => {
            info!("Discord/Ollama: RUN_JS failed: {}", e);
            format!(
                "JavaScript execution failed: {}. Answer the user's question without running code.",
                e
            )
        }
    }
}

// ── RUN_CMD result struct ────────────────────────────────────────────────

pub(crate) struct RunCmdResult {
    pub message: String,
    /// Raw stdout when the command succeeded; stored so the next TASK_APPEND gets the full output.
    pub raw_output: Option<String>,
}

// ── RUN_CMD ──────────────────────────────────────────────────────────────

pub(crate) async fn handle_run_cmd(
    arg: &str,
    last_run_cmd_arg: Option<&str>,
    multi_tool_turn: bool,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> RunCmdResult {
    info!(
        "Agent router: RUN_CMD requested: {}",
        crate::logging::ellipse(arg, 120)
    );

    if !crate::commands::run_cmd::is_local_cmd_allowed() {
        return RunCmdResult {
            message: "RUN_CMD is not available (disabled by ALLOW_LOCAL_CMD=0). Answer without running local commands.".to_string(),
            raw_output: None,
        };
    }
    if last_run_cmd_arg == Some(arg) {
        info!("Agent router: RUN_CMD duplicate (same arg as last run), skipping execution");
        return RunCmdResult {
            message: "You already ran this command; the result is in the message above. Do not run RUN_CMD again. Reply with TASK_APPEND then TASK_STATUS as the task instructs.".to_string(),
            raw_output: None,
        };
    }

    const MAX_CMD_RETRIES: u32 = 3;
    let mut current_cmd = arg.to_string();
    let mut last_output = String::new();
    let mut raw_output: Option<String> = None;

    for attempt in 0..=MAX_CMD_RETRIES {
        send_status(
            status_tx,
            &format!(
                "Running local command{}: {}",
                if attempt > 0 {
                    format!(" (retry {})", attempt)
                } else {
                    String::new()
                },
                current_cmd
            ),
        );
        info!("Agent router: RUN_CMD attempt {}: {}", attempt, current_cmd);

        match tokio::task::spawn_blocking({
            let cmd = current_cmd.clone();
            move || crate::commands::run_cmd::run_local_command(&cmd)
        })
        .await
        .map_err(|e| format!("RUN_CMD task: {}", e))
        .and_then(|r| r)
        {
            Ok(output) => {
                raw_output = Some(output.clone());
                info!(
                    "Agent router: RUN_CMD completed, stored output for next TASK_APPEND ({} chars)",
                    output.len()
                );
                last_output = format!(
                    "Here is the command output:\n\n{}\n\nUse this to answer the user's question.",
                    output
                );
                break;
            }
            Err(e) => {
                info!(
                    "Agent router: RUN_CMD failed (attempt {}): {}",
                    attempt, e
                );
                if multi_tool_turn {
                    last_output = format!(
                        "RUN_CMD failed in a multi-step plan: {}.\n\nRe-plan the full task from here. Keep the request in the correct tool domain. If Redmine data is still needed, use REDMINE_API directly with concrete parameters. Do not reply with only another RUN_CMD.",
                        e
                    );
                    break;
                }
                if attempt >= MAX_CMD_RETRIES {
                    last_output = format!(
                        "RUN_CMD failed after {} retries: {}.\n\nAnswer the user's question only (e.g. explain that the export or command failed). Do not include Redmine time entries, summaries, or other tool output that is unrelated to this request.",
                        MAX_CMD_RETRIES, e
                    );
                    break;
                }
                let allowed = crate::commands::run_cmd::allowed_commands().join(", ");
                let mut fix_prompt = format!(
                    "The command `{}` failed with error:\n{}\n\nReply with ONLY the corrected command on a single line, in this exact format:\nRUN_CMD: <corrected command>\n\nAllowed commands: {}. Paths must be under ~/.mac-stats.",
                    current_cmd, e, allowed
                );
                let mut fix_parse_retried = false;
                loop {
                    let fix_messages = vec![crate::ollama::ChatMessage {
                        role: "user".to_string(),
                        content: fix_prompt.clone(),
                        images: None,
                    }];
                    match send_ollama_chat_messages(
                        fix_messages,
                        model_override.clone(),
                        options_override.clone(),
                    )
                    .await
                    {
                        Ok(resp) => {
                            let fixed = resp.message.content.trim().to_string();
                            info!(
                                "Agent router: RUN_CMD fix suggestion: {}",
                                crate::logging::ellipse(&fixed, 120)
                            );
                            match parse_tool_from_response(&fixed) {
                                Some((tool, new_arg)) if tool == "RUN_CMD" => {
                                    current_cmd = new_arg;
                                    break;
                                }
                                _ => {
                                    if !fix_parse_retried {
                                        fix_parse_retried = true;
                                        fix_prompt = format!(
                                            "Your previous reply was not in the required format. Reply with exactly one line: RUN_CMD: <command>. No other text, no explanation.\n\nOriginal error: {}\nFailed command: {}",
                                            e, current_cmd
                                        );
                                        continue;
                                    }
                                    info!("Agent router: RUN_CMD fix suggestion not parseable as RUN_CMD (after format retry)");
                                    last_output = format!(
                                        "RUN_CMD failed: {}. The model's corrected command was not in the required format (exactly one line: RUN_CMD: <command>). Answer the user's question only; do not include Redmine or other unrelated tool output.",
                                        e
                                    );
                                    break;
                                }
                            }
                        }
                        Err(ollama_err) => {
                            info!(
                                "Agent router: RUN_CMD fix Ollama call failed: {}",
                                ollama_err
                            );
                            last_output = format!(
                                "RUN_CMD failed: {}. Could not get a corrected command from the model. Answer the user's question only; do not include Redmine or other unrelated tool output.",
                                e
                            );
                            break;
                        }
                    }
                }
                if !last_output.is_empty() {
                    break;
                }
            }
        }
    }

    RunCmdResult {
        message: last_output,
        raw_output,
    }
}

// ── PYTHON_SCRIPT ────────────────────────────────────────────────────────

pub(crate) async fn handle_python_script(
    _arg: &str,
    response_content: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if !crate::commands::python_agent::is_python_script_allowed() {
        return "PYTHON_SCRIPT is not available (disabled by ALLOW_PYTHON_SCRIPT=0). Answer without running Python.".to_string();
    }

    match parse_python_script_from_response(response_content) {
        Some((id, topic, script_body)) => {
            let script_label = format!("{} ({})", id, topic);
            send_status(
                status_tx,
                &format!("Running Python script '{}'…", script_label),
            );
            info!(
                "Agent router: PYTHON_SCRIPT running script '{}' (id={}, topic={}, body {} chars)",
                script_label,
                id,
                topic,
                script_body.len()
            );
            match tokio::task::spawn_blocking({
                let id = id.clone();
                let topic = topic.clone();
                let script_body = script_body.clone();
                move || crate::commands::python_agent::run_python_script(&id, &topic, &script_body)
            })
            .await
            .map_err(|e| format!("PYTHON_SCRIPT task: {}", e))
            .and_then(|r| r)
            {
                Ok(stdout) => format!(
                    "Python script result:\n\n{}\n\nUse this to answer the user's question.",
                    stdout
                ),
                Err(e) => format!(
                    "PYTHON_SCRIPT failed: {}. Answer without this result.",
                    e
                ),
            }
        }
        None => "PYTHON_SCRIPT requires: PYTHON_SCRIPT: <id> <topic> and then the Python code on the next lines or in a ```python block.".to_string(),
    }
}
