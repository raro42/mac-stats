//! Ollama Tauri commands — orchestrator (`answer_with_ollama_and_fetch`) and re-exports.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::Ordering;

use crate::commands::ollama_config::{
    get_ollama_client, read_ollama_api_key_from_env_or_config,
    read_ollama_fast_model_from_env_or_config,
};
pub use crate::commands::ollama_config::{
    ensure_ollama_agent_ready_at_startup, get_default_ollama_model_name,
};
pub use crate::commands::ollama_chat::send_ollama_chat_messages;
use crate::commands::content_reduction::{
    reduce_fetched_content_to_fit, run_skill_ollama_session, run_js_via_node,
    CHARS_PER_TOKEN,
};
use crate::commands::ollama_models::{
    delete_ollama_model, get_ollama_version, list_ollama_models, list_ollama_models_full,
    list_ollama_running_models, load_ollama_model, ollama_embeddings, pull_ollama_model,
    unload_ollama_model,
};
use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply,
    grounded_redmine_time_entries_failure_reply,
    is_redmine_review_or_summarize_only, is_redmine_time_entries_request,
    question_explicitly_requests_json, redmine_direct_fallback_hint,
    redmine_time_entries_range,
};
use crate::commands::reply_helpers::{
    append_to_file, final_reply_from_tool_results, is_agent_unavailable_error,
    is_bare_done_plan, is_final_same_as_intermediate, looks_like_discord_401_confusion,
    mastodon_post,
};
use crate::commands::pre_routing::compute_pre_routed_recommendation;
use crate::commands::compaction::{
    compact_conversation_history, COMPACTION_THRESHOLD, MIN_CONVERSATIONAL_FOR_COMPACTION,
};
use crate::commands::ollama_memory::{
    load_memory_block_for_request, load_soul_content, search_memory_for_request,
};
use crate::commands::perplexity_helpers::is_news_query;
use crate::commands::tool_parsing::{
    normalize_browser_tool_arg, normalize_inline_tool_sequences, parse_all_tools_from_response,
    parse_python_script_from_response,
    parse_tool_from_response, truncate_search_query_arg, MAX_BROWSER_TOOLS_PER_RUN,
};
use crate::commands::verification::{
    detect_new_topic, extract_success_criteria, original_request_for_retry,
    sanitize_success_criteria, summarize_last_turns, user_explicitly_asked_for_screenshot,
    verify_completion, RequestRunContext,
};
pub use crate::commands::verification::OllamaReply;
pub(crate) use crate::commands::agent_session::{
    normalize_discord_api_path, run_agent_ollama_session,
};

use crate::commands::agent_descriptions::{
    build_agent_descriptions, DISCORD_GROUP_CHANNEL_GUIDANCE, DISCORD_PLATFORM_FORMATTING,
};
use crate::commands::browser_helpers::{
    append_latest_browser_state_guidance, browser_retry_grounding_prompt,
    extract_browser_navigation_target,
    is_browser_task_request,
    should_use_http_fallback_after_browser_action_error, wants_visible_browser,
};






/// When set, `skill_content` is prepended to system prompts (from ~/.mac-stats/agents/skills/skill-<n>-<topic>.md).
/// When set, `agent_override` uses that agent's model and combined_prompt (soul+mood+skill) for the main run (e.g. Discord "agent: 001").
/// When `allow_schedule` is false (e.g. when running from the scheduler), the SCHEDULE tool is disabled so a scheduled task cannot create more schedules.
/// When `conversation_history` is set (e.g. from Discord session memory), it is prepended so the model sees prior turns and can resolve "there", "it", etc.
/// When `escalation` is true (e.g. user said "think harder" or "get it done"), we inject a stronger "you MUST complete the task" instruction and allow more tool steps.
/// When `retry_on_verification_no` is true and verification says we didn't satisfy the request, we run one retry with a "complete the remaining steps" prompt; the retry run is called with `retry_on_verification_no: false` so we don't retry indefinitely.
/// When `from_remote` is true (Discord, scheduler, task runner), browser runs default to headless (no visible window) unless the question explicitly asks to see the browser (e.g. "visible", "show me").
/// Returns a boxed future to allow one level of async recursion (retry path).
#[allow(clippy::too_many_arguments)]
pub fn answer_with_ollama_and_fetch(
    question: &str,
    status_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    discord_reply_channel_id: Option<u64>,
    discord_user_id: Option<u64>,
    discord_user_name: Option<String>,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
    skill_content: Option<String>,
    agent_override: Option<crate::agents::Agent>,
    allow_schedule: bool,
    conversation_history: Option<Vec<crate::ollama::ChatMessage>>,
    escalation: bool,
    retry_on_verification_no: bool,
    from_remote: bool,
    // When set (e.g. from Discord message attachments), the first user message is sent with these base64-encoded images for vision models.
    attachment_images_base64: Option<Vec<String>>,
    // When set (retry path from Discord), we format the reply as "--- Intermediate answer:\n\n{this}\n\n---" + final or "Final answer is the same as intermediate."
    discord_intermediate: Option<String>,
    // When true (verification retry path), we keep conversation history and skip NEW_TOPIC check so the model retains context.
    is_verification_retry: bool,
    // Original end-user request for this run. When set, verification/criteria extraction use this
    // instead of inferring from retry prompts or conversation history.
    original_user_request: Option<String>,
    // Request-local criteria extracted on the first pass. Reused on retry to avoid context bleed.
    success_criteria_override: Option<Vec<String>>,
    // When Some(true) = Discord DM (main session, load global memory). When Some(false) = Discord guild (do not load global memory). When None = not Discord (load global memory).
    discord_is_dm: Option<bool>,
    // When Some, use this request id so retries share the same id for end-to-end log correlation (task-008 Phase 1).
    request_id_override: Option<String>,
    // 0 = first run; 1 = verification retry. Logged for request tracing.
    retry_count: u32,
) -> Pin<Box<dyn Future<Output = Result<OllamaReply, String>> + Send>> {
    let load_global_memory = discord_is_dm.is_none_or(|dm| dm);
    if discord_is_dm == Some(false) {
        tracing::info!(
            "Agent router: Discord guild channel — not loading global memory (main-session only)"
        );
    }
    let question = question.to_string();
    let mut conversation_history = conversation_history.map(|v| v.to_vec());
    let attachment_images_base64 = attachment_images_base64.map(|v| v.to_vec());
    let discord_intermediate = discord_intermediate.map(|s| s.to_string());
    let original_user_request = original_user_request.map(|s| s.to_string());
    let success_criteria_override = success_criteria_override.map(|v| v.to_vec());
    Box::pin(async move {
        use tracing::info;
        let question = question.as_str();
        let request_for_verification = original_user_request.clone().unwrap_or_else(|| {
            original_request_for_retry(
                question,
                conversation_history.as_deref(),
                is_verification_retry,
            )
        });
        let request_id: String = request_id_override.unwrap_or_else(|| {
            format!(
                "{:08x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
                    & 0xFFFF_FFFF
            )
        });

        let run_ctx = RequestRunContext {
            request_id: request_id.clone(),
            retry_count,
            original_user_question: request_for_verification.clone(),
            discord_channel_id: discord_reply_channel_id,
            discord_user_id,
            discord_user_name: discord_user_name.clone(),
        };

        if run_ctx.retry_count > 0 {
            info!(
                "Agent router [{}]: session start (verification retry {}, request-local criteria only) — {}",
                run_ctx.request_id,
                run_ctx.retry_count,
                crate::config::Config::version_display()
            );
        } else {
            info!(
                "Agent router [{}]: session start — {}",
                run_ctx.request_id,
                crate::config::Config::version_display()
            );
        }
        tracing::debug!(
            request_id = %run_ctx.request_id,
            retry_count = run_ctx.retry_count,
            channel_id = ?run_ctx.discord_channel_id,
            user_id = ?run_ctx.discord_user_id,
            user_name = ?run_ctx.discord_user_name.as_deref(),
            question_preview = %run_ctx.original_user_question.chars().take(60).collect::<String>(),
            "request run context"
        );

        // When Discord user asks for screenshots to be sent here, focus on current task only (no prior chat).
        // Skip clearing on verification retry so the model keeps context (e.g. original request, cookie consent retry).
        if !is_verification_retry && discord_reply_channel_id.is_some() {
            let q = question.to_lowercase();
            if q.contains("screenshot")
                && (q.contains("send") || q.contains("here") || q.contains("discord"))
                && conversation_history
                    .as_ref()
                    .is_some_and(|h| !h.is_empty())
                {
                    info!(
                        "Agent router: clearing history for Discord screenshot-send request (focus on current task)"
                    );
                    conversation_history = Some(vec![]);
                }
        }

        let (model_override, skill_content, mut max_tool_iterations) =
            if let Some(ref a) = agent_override {
                (
                    a.model.clone().or(model_override),
                    Some(a.combined_prompt.clone()),
                    a.max_tool_iterations,
                )
            } else {
                (
                    model_override,
                    skill_content,
                    15u32, // default when no agent override
                )
            };
        if escalation {
            max_tool_iterations = max_tool_iterations.saturating_add(10);
            info!(
                "Agent router: escalation mode — max_tool_iterations raised to {}",
                max_tool_iterations
            );
        }

        if let Some(ref model) = model_override {
            let available = list_ollama_models()
                .await
                .map_err(|e| format!("Could not list models: {}", e))?;
            let found = available
                .iter()
                .any(|m| m == model || m.starts_with(&format!("{}:", model)));
            if !found {
                return Err(format!(
                    "Model '{}' not found. Available: {}",
                    model,
                    available.join(", ")
                ));
            }
        }

        let (endpoint, effective_model, api_key) = {
            let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
            let client = guard
                .as_ref()
                .ok_or_else(|| "Ollama not configured".to_string())?;
            let effective = model_override.clone().unwrap_or_else(|| {
                read_ollama_fast_model_from_env_or_config()
                    .unwrap_or_else(|| client.config.model.clone())
            });
            // Prefer local model: if the chosen model is cloud (e.g. qwen3.5:cloud) and we have no
            // explicit override, use first local model so Discord/scheduler work without ollama.com auth.
            let effective = if model_override.is_some() {
                effective
            } else if crate::ollama::models::is_cloud_model(&effective) {
                crate::ollama::models::get_first_local_model_name().unwrap_or(effective)
            } else {
                effective
            };
            let api_key = client
                .config
                .api_key
                .as_ref()
                .and_then(|acc| crate::security::get_credential(acc).ok().flatten())
                .or_else(read_ollama_api_key_from_env_or_config);
            (client.config.endpoint.clone(), effective, api_key)
        };
        // Use resolved model (local when default was cloud) for all chat calls in this request.
        let model_override = Some(effective_model.clone());
        let model_info =
            crate::ollama::get_model_info(&endpoint, &effective_model, api_key.as_deref())
                .await
                .unwrap_or_else(|_| crate::ollama::ModelInfo::default());

        let send_status = |msg: &str| {
            if let Some(ref tx) = status_tx {
                let _ = tx.send(msg.to_string());
            }
        };

        let question_for_plan_and_exec = if escalation {
            format!(
                "[The user is not satisfied and wants the task actually completed. You MUST use tools to fulfill the request; do not reply with only text.]\n\n{}",
                question
            )
        } else {
            question.to_string()
        };

        let q_preview: String = question.chars().take(120).collect();
        if question.len() > 120 {
            info!(
                "Agent router [{}]: starting (question: {}... [{} chars])",
                request_id,
                q_preview,
                question.len()
            );
        } else {
            info!(
                "Agent router [{}]: starting (question: {})",
                request_id, q_preview
            );
        }
        // Discord message limit 2000; wrapper ~50 chars → leave room for full question
        const DISCORD_STATUS_QUESTION_MAX: usize = 1940;
        let truncate_status = |s: &str, max: usize| {
            let taken: String = s.chars().take(max).collect();
            if s.chars().count() > max {
                format!("{}…", taken)
            } else {
                taken
            }
        };
        send_status(&format!(
            "Asking Ollama for a plan (sending your question: \"{}\")…",
            truncate_status(question, DISCORD_STATUS_QUESTION_MAX)
        ));

        // Criteria at start: extract 1–3 success criteria to feed into end verification
        send_status("Extracting success criteria…");
        let success_criteria = if let Some(criteria) = success_criteria_override.clone() {
            info!(
                "Agent router [{}]: reusing {} request-local success criteria",
                request_id,
                criteria.len()
            );
            Some(sanitize_success_criteria(
                &request_for_verification,
                criteria,
            ))
        } else if is_redmine_review_or_summarize_only(&request_for_verification) {
            info!(
                "Agent router: review/summarize-only Redmine request — overriding success criteria to summary-only"
            );
            Some(vec![
                "Summary of ticket content provided to the user.".to_string()
            ])
        } else if is_redmine_time_entries_request(&request_for_verification) {
            info!(
                "Agent router: Redmine time-entries request — overriding success criteria to Redmine data-only"
            );
            Some(vec![
                "Actual Redmine time entries for the requested period were fetched.".to_string(),
                "A clear summary of hours or worked tickets was provided from that Redmine data."
                    .to_string(),
            ])
        } else if is_news_query(&request_for_verification) {
            info!(
                "Agent router: news/current-events request — using news success criteria"
            );
            Some(vec![
                "At least 2 named sources (publication or site name).".to_string(),
                "Dates included when available in the search results.".to_string(),
                "Short factual summary or bullet list (concise is OK).".to_string(),
            ])
        } else {
            extract_success_criteria(
                &request_for_verification,
                model_override.clone(),
                options_override.clone(),
            )
            .await
            .map(|criteria| sanitize_success_criteria(&request_for_verification, criteria))
        };
        let criteria_status = match &success_criteria {
            Some(c) if !c.is_empty() => {
                info!(
                    "Agent router [{}]: extracted {} success criteria",
                    request_id,
                    c.len()
                );
                truncate_status(&c.join("; "), 200)
            }
            _ => {
                tracing::debug!(
                    "Agent router: no success criteria extracted (verification will use request summary only)"
                );
                "(none)".to_string()
            }
        };
        send_status(&format!(
            "EDIT:Extracted success criteria: {}",
            criteria_status
        ));

        let mut is_new_topic = false;
        // 6.A New-topic check: one short LLM call when we have history and use a local model.
        // Skip when using a cloud model to minimize cost (only when cloud do we care about extra calls).
        // Skip on verification retry so the model keeps context (original request, cookie consent, etc.).
        if is_verification_retry || crate::ollama::models::is_cloud_model(&effective_model) {
            tracing::debug!(
                request_id = %request_id,
                verification_retry = is_verification_retry,
                "skipping new-topic check (retry or cloud model)"
            );
        } else if let Some(ref hist) = conversation_history {
            const NEW_TOPIC_MIN_HISTORY: usize = 2;
            if hist.len() >= NEW_TOPIC_MIN_HISTORY {
                send_status("Checking if new topic…");
                let summary = summarize_last_turns(hist, 3);
                match detect_new_topic(question, &summary, &effective_model).await {
                    Ok(true) => {
                        info!(
                            "Agent router [{}]: NEW_TOPIC — using no prior context; any verification retry will use only request-local criteria",
                            request_id
                        );
                        is_new_topic = true;
                    }
                    Ok(false) => {
                        info!(
                            "Agent router [{}]: SAME_TOPIC — keeping prior context",
                            request_id
                        );
                    }
                    Err(e) => {
                        tracing::debug!(
                            request_id = %request_id,
                            "new-topic check failed (keeping history): {}",
                            e
                        );
                    }
                }
            }
        }

        const CONVERSATION_HISTORY_CAP: usize = 20;
        let raw_history: Vec<crate::ollama::ChatMessage> = conversation_history
            .unwrap_or_default()
            .into_iter()
            .rev()
            .take(CONVERSATION_HISTORY_CAP)
            .rev()
            .collect();
        if raw_history.is_empty() {
            info!("Agent router [{}]: no prior session", request_id);
        } else {
            info!(
                "Agent router [{}]: prior session {} messages (capped at {})",
                request_id,
                raw_history.len(),
                CONVERSATION_HISTORY_CAP
            );
        }

        let conversation_history: Vec<crate::ollama::ChatMessage> = if raw_history.len()
            >= COMPACTION_THRESHOLD
        {
            // Skip compaction when session has no real conversational value (task-008 Phase 5).
            let pairs: Vec<(String, String)> = raw_history
                .iter()
                .map(|m| (m.role.clone(), m.content.clone()))
                .collect();
            let conversational = crate::session_memory::count_conversational_messages(&pairs);
            if conversational < MIN_CONVERSATIONAL_FOR_COMPACTION {
                info!(
                    "Session compaction [{}]: skipped (no real conversational value: {} conversational messages, need at least {}); keeping full history ({} messages)",
                    request_id, conversational, MIN_CONVERSATIONAL_FOR_COMPACTION, raw_history.len()
                );
                raw_history
                    .into_iter()
                    .map(|mut msg| {
                        if msg.role == "assistant" && looks_like_discord_401_confusion(&msg.content) {
                            msg.content.push_str("\n\n[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]");
                        }
                        msg
                    })
                    .collect()
            } else {
            send_status("Compacting session memory…");
            info!(
                "Session compaction [{}]: {} messages exceed threshold ({}), compacting",
                request_id,
                raw_history.len(),
                COMPACTION_THRESHOLD
            );
            match compact_conversation_history(&raw_history, question, discord_reply_channel_id).await {
                Ok((context, lessons)) => {
                    info!(
                        "Session compaction [{}]: produced context ({} chars), lessons: {}",
                        request_id,
                        context.len(),
                        lessons.as_ref().map(|_| "present").unwrap_or("none")
                    );
                    if let Some(ref lesson_text) = lessons {
                        let memory_path = discord_reply_channel_id
                            .map(crate::config::Config::memory_file_path_for_discord_channel)
                            .unwrap_or_else(crate::config::Config::memory_file_path);
                        for line in lesson_text.lines() {
                            let line = line.trim().trim_start_matches("- ").trim();
                            if !line.is_empty() && line.len() > 5 {
                                let entry = format!("- {}\n", line);
                                let _ = append_to_file(&memory_path, &entry);
                            }
                        }
                        info!("Session compaction [{}]: wrote lessons to {:?}", request_id, memory_path);
                    }
                    let context_lower = context.to_lowercase();
                    let not_needed = context_lower.contains("not needed for this request")
                        || context_lower.contains("covered different topics");
                    if let Some(channel_id) = discord_reply_channel_id {
                        let compacted = vec![
                            ("system".to_string(), context.clone()),
                            ("user".to_string(), question.to_string()),
                        ];
                        crate::session_memory::replace_session("discord", channel_id, compacted);
                    }
                    if is_new_topic || not_needed {
                        info!(
                            "Session compaction: prior context not relevant to current question (new topic), using no prior context"
                        );
                        vec![]
                    } else {
                        info!(
                            "Session compaction: replaced {} messages with summary ({} chars)",
                            raw_history.len(),
                            context.len()
                        );
                        vec![crate::ollama::ChatMessage {
                            role: "system".to_string(),
                            content: format!(
                                "Previous session context (compacted from {} messages):\n\n{}",
                                raw_history.len(),
                                context
                            ),
                            images: None,
                        }]
                    }
                }
                Err(e) => {
                    let n = raw_history.len();
                    let err_s = e.to_string();
                    let is_skip = err_s.contains("no real conversational value");
                    if is_skip {
                        info!(
                            "Session compaction skipped: {}; keeping full history ({} messages)",
                            err_s, n
                        );
                    } else {
                        let msg = if err_s.to_lowercase().contains("unauthorized")
                            || err_s.contains("401")
                        {
                            format!(
                                "Session compaction failed: {} (use a local model for compaction; cloud models may require auth). Keeping full history ({} messages) for this request.",
                                e, n
                            )
                        } else {
                            format!(
                                "Session compaction failed: {}; keeping full history ({} messages) for this request.",
                                e, n
                            )
                        };
                        tracing::warn!("{}", msg);
                    }
                    raw_history
                    .into_iter()
                    .map(|mut msg| {
                        if msg.role == "assistant" && looks_like_discord_401_confusion(&msg.content) {
                            msg.content.push_str("\n\n[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]");
                        }
                        msg
                    })
                    .collect()
                }
            }
            }
        } else if is_new_topic {
            vec![]
        } else {
            raw_history
            .into_iter()
            .map(|mut msg| {
                if msg.role == "assistant" && looks_like_discord_401_confusion(&msg.content) {
                    msg.content.push_str("\n\n[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]");
                }
                msg
            })
            .collect()
        };
        if !conversation_history.is_empty() {
            info!(
                "Agent router: using {} prior messages as context",
                conversation_history.len()
            );
        }

        let from_discord = discord_reply_channel_id.is_some();
        let discord_platform_formatting: String = if from_discord {
            info!("Agent router: Discord reply — injecting platform formatting (no tables, wrap links in <>)");
            if discord_is_dm == Some(false) {
                format!(
                    "{}{}",
                    DISCORD_PLATFORM_FORMATTING,
                    DISCORD_GROUP_CHANNEL_GUIDANCE
                )
            } else {
                DISCORD_PLATFORM_FORMATTING.to_string()
            }
        } else {
            String::new()
        };
        let agent_descriptions = build_agent_descriptions(from_discord, Some(question)).await;
        info!(
            "Agent router: agent list built ({} chars)",
            agent_descriptions.len()
        );

        // When no agent/skill override, prepend soul (~/.mac-stats/agents/soul.md) so Ollama gets personality/vibe in context.
        let router_soul = skill_content.as_ref().map_or_else(
            || {
                let s = load_soul_content();
                if s.is_empty() {
                    format!(
                        "You are mac-stats v{}.\n\n",
                        crate::config::Config::version()
                    )
                } else {
                    format!(
                        "{}\n\nYou are mac-stats v{}.\n\n",
                        s,
                        crate::config::Config::version()
                    )
                }
            },
            |_| String::new(),
        );

        let discord_user_context = match (discord_user_id, &discord_user_name) {
            (Some(id), name_opt) => {
                let name = name_opt.as_deref().unwrap_or("").to_string();
                let stored = crate::user_info::get_user_details(id);
                let display_name = stored
                    .as_ref()
                    .and_then(|d| d.display_name.as_deref())
                    .filter(|s| !s.is_empty())
                    .unwrap_or(name.as_str());
                let mut ctx = if !display_name.is_empty() {
                    format!(
                        "You are talking to Discord user **{}** (user id: {}). Use this when addressing the user or when calling Discord API with this user.",
                        display_name, id
                    )
                } else {
                    format!(
                        "You are talking to Discord user (user id: {}). Use this when calling Discord API with this user.",
                        id
                    )
                };
                if let Some(ref details) = stored {
                    let extra = crate::user_info::format_user_details_for_context(details);
                    if !extra.is_empty() {
                        ctx.push_str(&format!("\nUser details: {}.", extra));
                    }
                }
                ctx.push_str("\n\n");
                ctx
            }
            _ => String::new(),
        };

        // --- Pre-routing: deterministic tool dispatch for unambiguous patterns ---
        let pre_routed_recommendation = compute_pre_routed_recommendation(
            question,
            &request_for_verification,
            is_verification_retry,
        );

        // --- Planning step: ask Ollama how it would solve the question (skip if pre-routed) ---
        let mut recommendation = if let Some(pre_routed) = pre_routed_recommendation {
            info!("Agent router: skipping LLM planning (pre-routed)");
            pre_routed
        } else {
            info!(
                "Agent router [{}]: planning step — asking Ollama for RECOMMEND",
                request_id
            );
            let planning_prompt = crate::config::Config::load_planning_prompt();
            let planning_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\n{}{}",
                    discord_user_context, skill, planning_prompt, agent_descriptions,
                    discord_platform_formatting
                ),
                None => format!(
                    "{}{}{}\n\n{}{}",
                    router_soul, discord_user_context, planning_prompt, agent_descriptions,
                    discord_platform_formatting
                ),
            };
            let mut planning_messages: Vec<crate::ollama::ChatMessage> =
                vec![crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: planning_system_content,
                    images: None,
                }];
            for msg in &conversation_history {
                planning_messages.push(msg.clone());
            }
            let model_hint = model_override.as_ref().map(|m| format!("\n\nFor this request the user selected Ollama model: {}. The app will use that model for the reply; recommend answering the question (or using an agent) with that in mind.", m)).unwrap_or_default();
            let today_utc = chrono::Utc::now().format("%Y-%m-%d");
            info!(
                "Agent router [{}]: planning RECOMMEND with current date (UTC) {}",
                request_id, today_utc
            );
            planning_messages.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Current date (UTC): {}.\n\nCurrent user question: {}{}\n\nReply with RECOMMEND: your plan.",
                    today_utc, question_for_plan_and_exec, model_hint
                ),
                images: attachment_images_base64.clone(),
            });
            let plan_response = send_ollama_chat_messages(
                planning_messages,
                model_override.clone(),
                options_override.clone(),
            )
            .await?;
            let mut rec = plan_response.message.content.trim().to_string();
            while rec.to_uppercase().starts_with("RECOMMEND: ")
                || rec.to_uppercase().starts_with("RECOMMEND:")
            {
                let prefix_len = if rec.len() >= 11 && rec[..11].to_uppercase() == "RECOMMEND: " {
                    11
                } else {
                    10
                };
                rec = rec[prefix_len..].trim().to_string();
            }
            rec
        };
        // When planner returned only "DONE: no" or "DONE: success" and the question indicates attachment failure, use a synthetic reply-only instruction so the user gets a proper summary (task: avoid bare DONE as execution plan).
        if is_bare_done_plan(&recommendation) {
            let q = question.to_lowercase();
            if q.contains("could not be attached") || q.contains("could not attach") {
                info!(
                    "Agent router [{}]: planner returned bare DONE plan but question indicates attachment failure; using synthetic summary instruction",
                    request_id
                );
                recommendation = "Reply with a brief summary of what was done and that the app could not attach the file(s) to Discord, then end with **DONE: no**. Do not run further tools.".to_string();
            }
        }
        info!(
            "Agent router [{}]: understood plan — {}",
            request_id,
            recommendation.chars().take(200).collect::<String>()
        );
        send_status(&format!(
            "Executing plan: {}…",
            truncate_status(&recommendation, 72)
        ));

        // Tools that benefit from a clean session (no stale conversation context).
        // Redmine reviews must not be polluted by prior turns — the model hallucinates.
        let fresh_session_tools = ["REDMINE_API"];
        let rec_upper = recommendation.to_uppercase();
        let needs_fresh_session = fresh_session_tools.iter().any(|t| rec_upper.contains(t));
        let conversation_history = if needs_fresh_session && !conversation_history.is_empty() {
            info!("Agent router: clearing conversation history for fresh-session tool");
            Vec::new()
        } else {
            conversation_history
        };

        // --- Execution: system prompt with agents + plan, then tool loop ---
        let execution_prompt_raw = crate::config::Config::load_execution_prompt();
        let execution_prompt = execution_prompt_raw.replace("{{AGENTS}}", &agent_descriptions);

        /// Log content in full if ≤500 chars (or always in -vv), else ellipse (first half + "..." + last half).
        const LOG_CONTENT_MAX: usize = 500;
        let log_verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
        let log_content = |content: &str| {
            let n = content.chars().count();
            if log_verbosity >= 2 || n <= LOG_CONTENT_MAX {
                content.to_string()
            } else {
                crate::logging::ellipse(content, LOG_CONTENT_MAX)
            }
        };

        // Include current system metrics so the model can answer accurately when the user asks about CPU, RAM, disk, etc.
        let metrics_block = crate::metrics::format_metrics_for_ai_context();
        let metrics_for_system = format!("\n\n{}", metrics_block);
        let model_identity = format!(
            "\n\nYou are replying as the Ollama model: **{}**. If the user asks which model you are (or what model you run on), name this model.",
            effective_model
        );
        // When Discord user asked for screenshots to be sent here, remind executor to actually run BROWSER_NAVIGATE + BROWSER_SCREENSHOT per URL.
        let discord_screenshot_reminder = if discord_reply_channel_id.is_some() {
            let q = question.to_lowercase();
            if q.contains("screenshot")
                && (q.contains("send") || q.contains("here") || q.contains("discord"))
            {
                "\n\n**Discord:** The user asked for screenshot(s) to be sent here. You MUST call BROWSER_NAVIGATE then BROWSER_SCREENSHOT: current for each URL; the app will attach the images to the reply."
            } else {
                ""
            }
        } else {
            ""
        };
        // When user asks "how to query Redmine API" for time/hours, executor should run at least one REDMINE_API call (e.g. GET /time_entries.json with current month) so the reply includes a real example.
        let redmine_howto_reminder = {
            let q = question.to_lowercase();
            if (q.contains("how to") || q.contains("how do"))
                && (q.contains("redmine")
                    || q.contains("time entries")
                    || q.contains("spent time")
                    || q.contains("hours"))
            {
                "\n\n**Redmine:** For \"how to query\" time entries or spent hours, run at least one REDMINE_API: GET /time_entries.json with from= and to= for the current month so your reply includes a real example (not only placeholder IDs or wrong year)."
            } else {
                ""
            }
        };
        // News/current-events: instruct model to answer with short bullet list, at least 2 named sources, dates when available.
        let news_format_reminder = if is_news_query(question) {
            "\n\n**News/current events:** Reply with a short bullet list or compact summary. Include at least 2 named sources (publication or site) and dates when available in the search results. Concise answers are OK."
        } else {
            ""
        };

        let normalized_recommendation = normalize_inline_tool_sequences(&recommendation);

        // Fast path: if the recommendation already contains a parseable tool call, execute it
        // directly instead of asking Ollama a second time to regurgitate the same tool line.
        let direct_tool = parse_tool_from_response(&recommendation);
        let (mut messages, mut response_content) = if let Some((ref tool, ref arg)) = direct_tool {
            let all_tools = parse_all_tools_from_response(&normalized_recommendation);
            if all_tools.len() > 1 {
                info!(
                    "Agent router: plan contains {} direct tools, will run in sequence: {} — skipping execution Ollama call",
                    all_tools.len(),
                    all_tools.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>().join(", ")
                );
            } else {
                info!(
                    "Agent router: plan contains direct tool call {}:{} — skipping execution Ollama call",
                    tool,
                    crate::logging::ellipse(arg, 60)
                );
            }
            let memory_block =
                load_memory_block_for_request(discord_reply_channel_id, load_global_memory);
            let execution_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}{}{}{}{}{}{}",
                    discord_user_context,
                    skill,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    model_identity
                ),
                None => format!(
                    "{}{}{}{}{}{}{}{}{}{}",
                    router_soul,
                    memory_block,
                    discord_user_context,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    model_identity
                ),
            };
            let mut msgs: Vec<crate::ollama::ChatMessage> = vec![crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: execution_system_content,
                images: None,
            }];
            for msg in &conversation_history {
                msgs.push(msg.clone());
            }
            msgs.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: question_for_plan_and_exec.clone(),
                images: attachment_images_base64.clone(),
            });
            // Preserve multi-tool chains so the executor runs them step by step (not one RUN_CMD with the whole chain).
            let synthetic = if all_tools.len() > 1 {
                all_tools
                    .iter()
                    .map(|(t, a)| format!("{}: {}", t, a))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else if normalized_recommendation.contains('\n') {
                normalized_recommendation.clone()
            } else {
                format!("{}: {}", tool, arg)
            };
            (msgs, synthetic)
        } else {
            info!(
                "Agent router: execution step — sending plan + question, starting tool loop (max {} tools)",
                max_tool_iterations
            );
            let memory_block =
                load_memory_block_for_request(discord_reply_channel_id, load_global_memory);
            let execution_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}{}{}{}{}{}\n\nYour plan: {}{}",
                    discord_user_context,
                    skill,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    recommendation,
                    model_identity
                ),
                None => format!(
                    "{}{}{}{}{}{}{}{}{}\n\nYour plan: {}{}",
                    router_soul,
                    memory_block,
                    discord_user_context,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    recommendation,
                    model_identity
                ),
            };
            let mut msgs: Vec<crate::ollama::ChatMessage> = vec![crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: execution_system_content,
                images: None,
            }];
            for msg in &conversation_history {
                msgs.push(msg.clone());
            }
            msgs.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: question_for_plan_and_exec.clone(),
                images: attachment_images_base64.clone(),
            });
            let response = send_ollama_chat_messages(
                msgs.clone(),
                model_override.clone(),
                options_override.clone(),
            )
            .await?;
            let content = response.message.content.clone();
            let n = content.chars().count();
            info!(
                "Agent router: first response received ({} chars): {}",
                n,
                log_content(&content)
            );
            // Fallback: if Ollama returned empty but the recommendation contains a parseable tool,
            // synthesize the tool call so the tool loop can execute it.
            if n == 0 {
                if let Some((tool, arg)) = parse_tool_from_response(&recommendation) {
                    let synthetic = if normalized_recommendation.contains('\n') {
                        normalized_recommendation.clone()
                    } else {
                        format!("{}: {}", tool, arg)
                    };
                    info!(
                        "Agent router: empty response — falling back to tool from recommendation: {}",
                        crate::logging::ellipse(&synthetic, 80)
                    );
                    (msgs, synthetic)
                } else {
                    (msgs, content)
                }
            } else {
                (msgs, content)
            }
        };

        let mut tool_count: u32 = 0;
        // Browser mode: "headless" in question → no visible window. From Discord/scheduler/task (from_remote) → headless unless user explicitly asks to see the browser (so retries stay headless).
        let prefer_headless = if from_remote {
            !wants_visible_browser(question)
        } else {
            question.to_lowercase().contains("headless")
        };
        crate::browser_agent::set_prefer_headless_for_run(prefer_headless);
        // Paths to attach when replying on Discord (e.g. BROWSER_SCREENSHOT); only under ~/.mac-stats/screenshots/.
        let mut attachment_paths: Vec<PathBuf> = Vec::new();
        let mut screenshot_requested_by_tool_run = false;
        // Collect (agent_name, reply) for each AGENT call so we can append a conversation transcript when multiple agents participated.
        let mut agent_conversation: Vec<(String, String)> = Vec::new();
        // Dedupe repeated identical DISCORD_API calls so the model can't loop on the same request.
        let mut last_successful_discord_call: Option<(String, String)> = None;
        // Dedupe repeated identical RUN_CMD so the model can't loop (e.g. task says run once then TASK_APPEND/TASK_STATUS).
        let mut last_run_cmd_arg: Option<String> = None;
        // When the model does TASK_APPEND right after RUN_CMD, use this so the task file gets the full command output (not a summary).
        let mut last_run_cmd_raw_output: Option<String> = None;
        // Track the task file we're working on so we can append the full conversation at the end.
        let mut current_task_path: Option<std::path::PathBuf> = None;
        // When the model calls DONE we break the tool loop; strip the DONE line from the final reply.
        let mut exited_via_done: bool = false;
        // Last BROWSER_EXTRACT result (JS-rendered page text) for completion verification so we check against real content, not FETCH_URL HTML.
        let mut last_browser_extract: Option<String> = None;
        // Cap browser tools per run to prevent runaway NAVIGATE/CLICK loops (see docs/032_browser_loop_and_status_fix_plan.md).
        let mut browser_tool_count: u32 = 0;
        // Set when we skip a browser tool due to cap; used to append a user-facing note to the reply.
        let mut browser_tool_cap_reached: bool = false;
        // Repetition detection: refuse duplicate consecutive browser action (same NAVIGATE URL or same CLICK index).
        let mut last_browser_tool_arg: Option<(String, String)> = None;
        // For news requests: if the last PERPLEXITY_SEARCH returned only hub/landing/tag/standings pages, verification must not accept a confident news answer.
        let mut last_news_search_was_hub_only: Option<bool> = None;
        let budget_warning_ratio = crate::config::Config::tool_budget_warning_ratio();

        while tool_count < max_tool_iterations {
            let tools = parse_all_tools_from_response(&response_content);
            if tools.is_empty() {
                info!(
                    "Agent router: no tool call in response ({} chars), treating as final answer: {}",
                    response_content.chars().count(),
                    log_content(&response_content)
                );
                break;
            }
            if tools.len() > 1 {
                info!(
                    "Agent router: running {} tools in one turn: {}",
                    tools.len(),
                    tools
                        .iter()
                        .map(|(t, _)| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            let multi_tool_turn = tools.len() > 1;
            let mut done_claimed: Option<bool> = None;
            let mut tool_results: Vec<String> = Vec::with_capacity(tools.len());
            // Sequence-terminating: after a page-changing browser action (NAVIGATE, GO_BACK),
            // remaining browser tools in the same turn use stale indices and must be skipped.
            let mut page_changed_this_turn = false;
            for (tool, arg) in tools {
                if tool_count >= max_tool_iterations {
                    break;
                }
                tool_count += 1;
                // Set when we execute a browser tool this iteration; used to update last_browser_tool_arg after the match.
                let mut executed_browser_tool_arg: Option<(String, String)> = None;
                // When the plan puts the whole chain in one line (e.g. PERPLEXITY_SEARCH: spanish newspapers then BROWSER_NAVIGATE...), pass only the search query to PERPLEXITY/BRAVE.
                let arg = if tool == "PERPLEXITY_SEARCH" || tool == "BRAVE_SEARCH" {
                    truncate_search_query_arg(&arg)
                } else {
                    arg
                };
                let arg_preview: String = arg.chars().take(80).collect();
                if arg.chars().count() > 80 {
                    info!(
                        "Agent router: running tool {}/{} — {} (arg: {}...)",
                        tool_count, max_tool_iterations, tool, arg_preview
                    );
                } else {
                    info!(
                        "Agent router: running tool {}/{} — {} (arg: {})",
                        tool_count, max_tool_iterations, tool, arg_preview
                    );
                }

                // Cap browser tools per run to prevent runaway loops (032_browser_loop_and_status_fix_plan).
                let is_browser_tool = matches!(
                    tool.as_str(),
                    "BROWSER_NAVIGATE"
                        | "BROWSER_CLICK"
                        | "BROWSER_INPUT"
                        | "BROWSER_SCROLL"
                        | "BROWSER_EXTRACT"
                        | "BROWSER_SEARCH_PAGE"
                        | "BROWSER_SCREENSHOT"
                );
                if is_browser_tool {
                    let normalized_arg = normalize_browser_tool_arg(&tool, &arg);
                    // Repetition detection: same action as previous step → skip and tell model (032).
                    if last_browser_tool_arg.as_ref() == Some(&(tool.clone(), normalized_arg.clone())) {
                        let msg = "Same browser action as previous step; use a different action or reply with DONE.".to_string();
                        tool_results.push(msg);
                        info!(
                            "Agent router: duplicate browser action skipped ({} with same arg)",
                            tool
                        );
                        continue;
                    }
                    if browser_tool_count >= MAX_BROWSER_TOOLS_PER_RUN {
                        browser_tool_cap_reached = true;
                        let msg = format!(
                            "Maximum browser actions per run reached ({}). Reply with your answer or DONE: success / DONE: no.",
                            MAX_BROWSER_TOOLS_PER_RUN
                        );
                        tool_results.push(msg);
                        info!(
                            "Agent router: browser tool cap reached ({}), skipping {}",
                            MAX_BROWSER_TOOLS_PER_RUN, tool
                        );
                        continue;
                    }
                    if page_changed_this_turn {
                        tool_results.push(format!(
                            "Page changed by a previous navigation; {} was skipped because element indices are stale. Current page state is above — use it to plan new actions in the next turn.",
                            tool
                        ));
                        info!(
                            "Agent router: terminates-sequence — skipping {} after page-changing action this turn",
                            tool
                        );
                        continue;
                    }
                    browser_tool_count += 1;
                    executed_browser_tool_arg = Some((tool.clone(), normalized_arg));
                    info!(
                        "Agent router: browser tool #{}/{} this run",
                        browser_tool_count, MAX_BROWSER_TOOLS_PER_RUN
                    );
                }
                if tool == "BROWSER_SCREENSHOT" {
                    screenshot_requested_by_tool_run = true;
                }

                let user_message = match tool.as_str() {
                    "DONE" => {
                        let arg_lower = arg.trim().to_lowercase();
                        let claimed_success = arg_lower.is_empty()
                            || arg_lower.contains("success")
                            || arg_lower.contains("yes")
                            || arg_lower.contains("true");
                        done_claimed = Some(claimed_success);
                        send_status(if claimed_success {
                            "✅ Task marked done (success)."
                        } else {
                            "⏹️ Task marked done (could not complete)."
                        });
                        info!(
                            "Agent router: model called DONE (success={}), exiting tool loop",
                            claimed_success
                        );
                        String::new()
                    }
                    "FETCH_URL" if arg.contains("discord.com") => {
                        let path = if let Some(pos) = arg.find("/api/v10") {
                            arg[pos + "/api/v10".len()..].to_string()
                        } else if let Some(pos) = arg.find("/api/") {
                            arg[pos + "/api".len()..].to_string()
                        } else {
                            String::new()
                        };
                        if !path.is_empty() {
                            info!(
                                "Agent router: redirecting FETCH_URL discord.com -> DISCORD_API GET {}",
                                path
                            );
                            send_status(&format!("Discord API: GET {}", path));
                            match crate::discord::api::discord_api_request("GET", &path, None).await
                            {
                                Ok(result) => format!(
                                    "Discord API result (GET {}):\n\n{}\n\nUse this to answer the user's question.",
                                    path, result
                                ),
                                Err(e) => format!(
                                    "Discord API failed (GET {}): {}. Try DISCORD_API: GET {} or delegate to AGENT: discord-expert.",
                                    path, e, path
                                ),
                            }
                        } else {
                            info!(
                                "Agent router: blocked FETCH_URL for discord.com (no API path). Redirecting to discord-expert."
                            );
                            "Cannot fetch discord.com pages directly. Discord requires authenticated API access. Use AGENT: discord-expert for all Discord tasks, or use DISCORD_API: GET <path> with the correct API endpoint.".to_string()
                        }
                    }
                    "FETCH_URL" => {
                        send_status(&format!(
                            "🌐 Fetching page at {}…",
                            crate::logging::ellipse(&arg, 45)
                        ));
                        info!("Discord/Ollama: FETCH_URL requested: {}", arg);
                        let url = arg.to_string();
                        let fetch_result = tokio::task::spawn_blocking(move || {
                            crate::commands::browser::fetch_page_content(&url)
                        })
                        .await
                        .map_err(|e| format!("Fetch task: {}", e))?
                        .map_err(|e| format!("Fetch page failed: {}", e));
                        match fetch_result {
                            Ok(body) => {
                                let estimated_used =
                                    (messages.iter().map(|m| m.content.len()).sum::<usize>()
                                        + agent_descriptions.len())
                                        / CHARS_PER_TOKEN
                                        + 50;
                                let body_fit = reduce_fetched_content_to_fit(
                                    &body,
                                    model_info.context_size_tokens,
                                    estimated_used as u32,
                                    model_override.clone(),
                                    options_override.clone(),
                                )
                                .await?;
                                format!(
                                    "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                                    body_fit
                                )
                            }
                            Err(e) => {
                                if e.contains("401") {
                                    info!(
                                        "Discord/Ollama: Fetch returned 401 Unauthorized, stopping"
                                    );
                                    "That URL returned 401 Unauthorized. Do not try another URL. Answer based on what you know.".to_string()
                                } else {
                                    info!(
                                        "Discord/Ollama: FETCH_URL failed: {}",
                                        crate::logging::ellipse(&e, 300)
                                    );
                                    let url_lower = arg.to_lowercase();
                                    let redmine_hint = if url_lower.contains("redmine")
                                        || url_lower.contains("/issues/")
                                    {
                                        " For Redmine tickets use REDMINE_API or say \"review ticket <id>\"."
                                    } else {
                                        ""
                                    };
                                    format!(
                                        "That URL could not be fetched (connection or server error).{redmine_hint} Answer without that page."
                                    )
                                }
                            }
                        }
                    }
                    "BROWSER_SCREENSHOT" => {
                        let url_arg = arg.trim().to_string();
                        let is_current =
                            url_arg.is_empty() || url_arg.eq_ignore_ascii_case("current");
                        // Browser-use style: screenshot only works on current page. Reject BROWSER_SCREENSHOT: <url>.
                        if !is_current {
                            info!(
                                "Agent router: rejecting BROWSER_SCREENSHOT: {} — use NAVIGATE first, then SCREENSHOT: current",
                                crate::logging::ellipse(&url_arg, 60)
                            );
                            format!(
                                "BROWSER_SCREENSHOT only works on the current page. Use BROWSER_NAVIGATE: {} first, then BROWSER_SCREENSHOT: current. Never use BROWSER_SCREENSHOT: <url>.",
                                url_arg
                            )
                        } else {
                            send_status("📸 Taking screenshot of current page");
                            match tokio::task::spawn_blocking(
                                crate::browser_agent::take_screenshot_current_page,
                            )
                            .await
                            {
                                Ok(Ok(path)) => {
                                    attachment_paths.push(path.clone());
                                    if let Some(ref tx) = status_tx {
                                        let _ = tx.send(format!("ATTACH:{}", path.display()));
                                    }
                                    format!(
                                        "Screenshot of current page saved to: {}.\n\nTell the user the screenshot was taken; the app will attach it in Discord.",
                                        path.display()
                                    )
                                }
                                Ok(Err(e)) => {
                                    info!(
                                        "Agent router [{}]: BROWSER_SCREENSHOT (current) failed: {}",
                                        request_id,
                                        crate::logging::ellipse(&e, 200)
                                    );
                                    format!(
                                        "Screenshot of current page failed: {}. (Use BROWSER_NAVIGATE and BROWSER_CLICK first with CDP; then BROWSER_SCREENSHOT: current. Chrome may need to be on port 9222.)",
                                        e
                                    )
                                }
                                Err(e) => format!("Screenshot task error: {}", e),
                            }
                        }
                    }
                    "BROWSER_NAVIGATE" => {
                        let raw_arg = arg.trim().to_string();
                        if raw_arg.is_empty() {
                            "BROWSER_NAVIGATE requires a URL (e.g. BROWSER_NAVIGATE: https://www.example.com). Please try again with a URL.".to_string()
                        } else if let Some(url_arg) = extract_browser_navigation_target(&raw_arg) {
                            let new_tab = raw_arg
                                .split_whitespace()
                                .any(|w| w.eq_ignore_ascii_case("new_tab"));
                            send_status(&format!(
                                "🧭 Navigating to {}…{}",
                                url_arg,
                                if new_tab { " (new tab)" } else { "" }
                            ));
                            info!(
                                "Agent router [{}]: BROWSER_NAVIGATE: URL sent to CDP: {} new_tab={}",
                                request_id, url_arg, new_tab
                            );
                            match tokio::task::spawn_blocking({
                                let u = url_arg.clone();
                                move || crate::browser_agent::navigate_and_get_state_with_options(&u, new_tab)
                            })
                            .await
                            {
                                Ok(Ok(state_str)) => state_str,
                                Ok(Err(cdp_err)) => {
                                    info!(
                                        "BROWSER_NAVIGATE CDP failed, ensuring Chrome on 9222 and retrying: {}",
                                        crate::logging::ellipse(&cdp_err, 120)
                                    );
                                    tokio::task::spawn_blocking(|| {
                                        crate::browser_agent::ensure_chrome_on_port(9222)
                                    })
                                    .await
                                    .ok();
                                    match tokio::task::spawn_blocking({
                                        let u = url_arg.clone();
                                        move || crate::browser_agent::navigate_and_get_state_with_options(&u, new_tab)
                                    })
                                    .await
                                    {
                                        Ok(Ok(state_str)) => state_str,
                                        Ok(Err(cdp_err2)) => {
                                            info!(
                                                "BROWSER_NAVIGATE CDP retry failed, trying HTTP fallback: {}",
                                                crate::logging::ellipse(&cdp_err2, 120)
                                            );
                                            match tokio::task::spawn_blocking(move || {
                                                crate::browser_agent::navigate_http(&url_arg)
                                            })
                                            .await
                                            {
                                                Ok(Ok(state_str)) => state_str,
                                                Ok(Err(http_err)) => format!(
                                                    "BROWSER_NAVIGATE failed (CDP: {}). HTTP fallback also failed: {}",
                                                    crate::logging::ellipse(&cdp_err2, 80),
                                                    http_err
                                                ),
                                                Err(e) => format!(
                                                    "BROWSER_NAVIGATE HTTP fallback task error: {}",
                                                    e
                                                ),
                                            }
                                        }
                                        Err(e) => {
                                            format!("BROWSER_NAVIGATE CDP retry task error: {}", e)
                                        }
                                    }
                                }
                                Err(e) => format!("BROWSER_NAVIGATE task error: {}", e),
                            }
                        } else {
                            append_latest_browser_state_guidance(&format!(
                                "BROWSER_NAVIGATE requires a concrete URL. The step {:?} was not executed because it did not contain a grounded browser target. This was an agent planning/parsing issue, not evidence about the site.",
                                raw_arg
                            ))
                        }
                    }
                    "BROWSER_GO_BACK" => {
                        send_status("🔙 Going back…");
                        info!("Agent router [{}]: BROWSER_GO_BACK", request_id);
                        match tokio::task::spawn_blocking(crate::browser_agent::go_back).await {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(e)) => append_latest_browser_state_guidance(&format!(
                                "BROWSER_GO_BACK failed: {}",
                                e
                            )),
                            Err(e) => format!("BROWSER_GO_BACK task error: {}", e),
                        }
                    }
                    "BROWSER_CLICK" => {
                        let index_arg = arg.split_whitespace().next().unwrap_or("").trim();
                        let index = index_arg.parse::<u32>().ok();
                        let status_msg = match index {
                            Some(idx) => {
                                let label = crate::browser_agent::get_last_element_label(idx);
                                if let Some(l) = label {
                                    format!(
                                        "🖱️ Clicking element {} ({})",
                                        idx,
                                        crate::logging::ellipse(&l, 30)
                                    )
                                } else {
                                    format!("🖱️ Clicking element {}", idx)
                                }
                            }
                            None => format!(
                                "🖱️ Clicking element {}",
                                if index_arg.is_empty() { "?" } else { index_arg }
                            ),
                        };
                        send_status(&status_msg);
                        match index {
                    Some(idx) => {
                        info!("BROWSER_CLICK: index {}", idx);
                        match tokio::task::spawn_blocking(move || crate::browser_agent::click_by_index(idx)).await {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(cdp_err)) => {
                                if should_use_http_fallback_after_browser_action_error(
                                    "BROWSER_CLICK",
                                    &cdp_err,
                                ) {
                                    match tokio::task::spawn_blocking(move || crate::browser_agent::click_http(idx)).await {
                                        Ok(Ok(state_str)) => state_str,
                                        Ok(Err(e)) => append_latest_browser_state_guidance(&format!("BROWSER_CLICK failed: {}", e)),
                                        Err(e) => format!("BROWSER_CLICK task error: {}", e),
                                    }
                                } else {
                                    append_latest_browser_state_guidance(&format!(
                                        "BROWSER_CLICK failed: {}",
                                        cdp_err
                                    ))
                                }
                            }
                            Err(e) => append_latest_browser_state_guidance(&format!("BROWSER_CLICK task error: {}", e)),
                        }
                    }
                    None => append_latest_browser_state_guidance("BROWSER_CLICK requires a numeric index (e.g. BROWSER_CLICK: 3). Use the index from the Current page Elements list."),
                }
                    }
                    "BROWSER_INPUT" => {
                        let mut parts = arg.trim().splitn(2, |c: char| c.is_whitespace());
                        let index_arg = parts.next().unwrap_or("").trim();
                        let index_for_status = index_arg.parse::<u32>().ok();
                        let status_msg = match index_for_status {
                            Some(idx) => {
                                let label = crate::browser_agent::get_last_element_label(idx);
                                if let Some(l) = label {
                                    format!(
                                        "✍️ Typing into element {} ({})…",
                                        idx,
                                        crate::logging::ellipse(&l, 30)
                                    )
                                } else {
                                    format!("✍️ Typing into element {}…", idx)
                                }
                            }
                            None => format!(
                                "✍️ Typing into element {}…",
                                if index_arg.is_empty() { "?" } else { index_arg }
                            ),
                        };
                        send_status(&status_msg);
                        let text = parts.next().unwrap_or("").trim().to_string();
                        let index = index_arg.parse::<u32>().ok();
                        match index {
                    Some(idx) => {
                        info!("BROWSER_INPUT: index {} ({} chars)", idx, text.len());
                        let text_clone = text.clone();
                        match tokio::task::spawn_blocking(move || crate::browser_agent::input_by_index(idx, &text_clone)).await {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(cdp_err)) => {
                                if should_use_http_fallback_after_browser_action_error(
                                    "BROWSER_INPUT",
                                    &cdp_err,
                                ) {
                                    match tokio::task::spawn_blocking(move || crate::browser_agent::input_http(idx, &text)).await {
                                    Ok(Ok(state_str)) => state_str,
                                    Ok(Err(e)) => append_latest_browser_state_guidance(&format!("BROWSER_INPUT failed: {}", e)),
                                    Err(e) => format!("BROWSER_INPUT task error: {}", e),
                                }
                                } else {
                                    append_latest_browser_state_guidance(&format!(
                                        "BROWSER_INPUT failed: {}",
                                        cdp_err
                                    ))
                                }
                            }
                            Err(e) => append_latest_browser_state_guidance(&format!("BROWSER_INPUT task error: {}", e)),
                        }
                    }
                    None => append_latest_browser_state_guidance("BROWSER_INPUT requires a numeric index and text (e.g. BROWSER_INPUT: 4 search query). Use the index from the Current page Elements list."),
                }
                    }
                    "BROWSER_SCROLL" => {
                        let scroll_arg = if arg.trim().is_empty() {
                            "down".to_string()
                        } else {
                            arg.trim().to_string()
                        };
                        send_status(&format!(
                            "📜 Scrolling {}…",
                            crate::logging::ellipse(&scroll_arg, 20)
                        ));
                        match tokio::task::spawn_blocking(move || {
                            crate::browser_agent::scroll_page(&scroll_arg)
                        })
                        .await
                        {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(e)) => {
                                info!(
                                    "BROWSER_SCROLL failed: {}",
                                    crate::logging::ellipse(&e, 200)
                                );
                                format!("BROWSER_SCROLL failed: {}", e)
                            }
                            Err(e) => format!("BROWSER_SCROLL task error: {}", e),
                        }
                    }
                    "BROWSER_EXTRACT" => {
                        send_status("📄 Extracting page text…");
                        match tokio::task::spawn_blocking(crate::browser_agent::extract_page_text)
                            .await
                        {
                            Ok(Ok(text)) => text,
                            Ok(Err(_cdp_err)) => {
                                match tokio::task::spawn_blocking(
                                    crate::browser_agent::extract_http,
                                )
                                .await
                                {
                                    Ok(Ok(text)) => text,
                                    Ok(Err(e)) => format!(
                                        "BROWSER_EXTRACT failed: {}. (Navigate to a page first with BROWSER_NAVIGATE.)",
                                        e
                                    ),
                                    Err(e) => format!("BROWSER_EXTRACT task error: {}", e),
                                }
                            }
                            Err(e) => format!("BROWSER_EXTRACT task error: {}", e),
                        }
                    }
                    "BROWSER_SEARCH_PAGE" => {
                        let pattern = arg.trim().to_string();
                        if pattern.is_empty() {
                            "BROWSER_SEARCH_PAGE requires a search pattern (e.g. BROWSER_SEARCH_PAGE: Ralf Röber). Use to find specific text on the current page.".to_string()
                        } else {
                            send_status(&format!(
                                "🔍 Searching page for \"{}\"…",
                                crate::logging::ellipse(&pattern, 30)
                            ));
                            match tokio::task::spawn_blocking(move || {
                                crate::browser_agent::search_page_text(&pattern)
                            })
                            .await
                            {
                                Ok(Ok(result)) => result,
                                Ok(Err(e)) => {
                                    info!(
                                        "BROWSER_SEARCH_PAGE failed: {}",
                                        crate::logging::ellipse(&e, 200)
                                    );
                                    format!(
                                        "BROWSER_SEARCH_PAGE failed: {}. (Navigate to a page first with BROWSER_NAVIGATE.)",
                                        e
                                    )
                                }
                                Err(e) => format!("BROWSER_SEARCH_PAGE task error: {}", e),
                            }
                        }
                    }
                    "BRAVE_SEARCH" => {
                        send_status(&format!(
                            "🌐 Searching the web for \"{}\"…",
                            crate::logging::ellipse(&arg, 35)
                        ));
                        info!("Discord/Ollama: BRAVE_SEARCH requested: {}", arg);
                        match crate::commands::brave::get_brave_api_key() {
                    Some(api_key) => match crate::commands::brave::brave_web_search(&arg, &api_key).await {
                        Ok(results) => format!(
                            "Brave Search results:\n\n{}\n\nUse these to answer the user's question.",
                            results
                        ),
                        Err(e) => format!("Brave Search failed: {}. Answer without search results.", e),
                    },
                    None => "Brave Search is not configured (no BRAVE_API_KEY in env or .config.env). Answer without search results.".to_string(),
                }
                    }
                    "PERPLEXITY_SEARCH" => {
                        let result = crate::commands::perplexity_helpers::handle_perplexity_search(
                            question,
                            &arg,
                            status_tx.as_ref(),
                            &request_id,
                        )
                        .await;
                        attachment_paths.extend(result.new_attachment_paths);
                        if let Some(hub_only) = result.news_search_was_hub_only {
                            last_news_search_was_hub_only = Some(hub_only);
                        }
                        result.text
                    }
                    // RUN_JS: agent-triggered; runs with process privileges via run_js_via_node.
                    // Treat agent output as untrusted code. Audit log code length + hash only (no full body).
                    "RUN_JS" => {
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
                        send_status(&format!("Running code: {}…", code_ref));
                        // Audit: code length + content hash only (no full code in logs).
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
                        match run_js_via_node(&arg) {
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
                    "SKILL" => {
                        send_status("Using skill…");
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
                                send_status(&format!(
                                    "Using skill {}-{}…",
                                    skill.number, skill.topic
                                ));
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
                                    model_override.clone(),
                                    options_override.clone(),
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
                    "AGENT" => {
                        let arg = arg.trim();
                        let (selector, task_message) = if let Some(space_idx) = arg.find(' ') {
                            let (sel, rest) = arg.split_at(space_idx);
                            (sel.trim(), rest.trim())
                        } else {
                            (arg, "")
                        };
                        // Proxy agent: cursor-agent runs the CLI instead of Ollama (handoff when local model isn't enough).
                        let selector_lower = selector.to_lowercase();
                        if (selector_lower == "cursor-agent" || selector_lower == "cursor_agent")
                            && crate::commands::cursor_agent::is_cursor_agent_available()
                        {
                            let prompt = if task_message.is_empty() {
                                question.to_string()
                            } else {
                                task_message.to_string()
                            };
                            send_status("Running Cursor Agent…");
                            info!(
                                "Agent router: AGENT cursor-agent proxy (prompt {} chars)",
                                prompt.len()
                            );
                            let prompt_clone = prompt.clone();
                            match tokio::task::spawn_blocking(move || {
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
                            }
                        } else if selector_lower == "cursor-agent"
                            || selector_lower == "cursor_agent"
                        {
                            "Cursor Agent is not available (cursor-agent CLI not on PATH). Answer without it.".to_string()
                        } else {
                        let agents = crate::agents::load_agents();
                        match crate::agents::find_agent_by_id_or_name(&agents, selector) {
                            Some(agent) => {
                                // Break out of AGENT: orchestrator loop when the last message was already
                                // an orchestrator result (model keeps re-invoking instead of replying).
                                let is_orchestrator = agent.id == "000";
                                let last_is_orchestrator_result = messages
                                    .last()
                                    .filter(|m| m.role == "user")
                                    .map(|m| m.content.starts_with("Agent \"Orchestrator\""))
                                    .unwrap_or(false);
                                if is_orchestrator && last_is_orchestrator_result {
                                    info!(
                                        "Agent router: skipping repeated AGENT: orchestrator (loop breaker)"
                                    );
                                    "The orchestrator already replied above. Reply with a one-sentence summary for the user and **DONE: success** or **DONE: no**. Do not output AGENT: orchestrator again.".to_string()
                                } else {
                                    let mut user_msg: String = if task_message.is_empty() {
                                        question.to_string()
                                    } else {
                                        task_message.to_string()
                                    };
                                    // When invoking discord-expert from Discord, fetch guild/channel metadata via API and inject so the agent has current context.
                                    let is_discord_expert =
                                        agent.slug.as_deref().is_some_and(|s| {
                                            s.eq_ignore_ascii_case("discord-expert")
                                        }) || agent.id == "004";
                                    let is_redmine_agent = agent
                                        .slug
                                        .as_deref()
                                        .is_some_and(|s| s.eq_ignore_ascii_case("redmine"))
                                        || agent.id == "006";
                                    if is_discord_expert {
                                        if let Some(channel_id) = discord_reply_channel_id {
                                            send_status("Fetching Discord guild/channel context…");
                                            match crate::discord::api::fetch_guild_channel_metadata(
                                                channel_id,
                                            )
                                            .await
                                            {
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
                                    let preview: String =
                                        user_msg.chars().take(STATUS_MSG_MAX).collect();
                                    let status_text = if user_msg.chars().count() > STATUS_MSG_MAX {
                                        format!("{}…", preview)
                                    } else {
                                        preview
                                    };
                                    send_status(&format!(
                                        "{} -> Ollama: {}",
                                        agent.name, status_text
                                    ));
                                    match run_agent_ollama_session(
                                        agent,
                                        &user_msg,
                                        status_tx.as_ref(),
                                        load_global_memory,
                                    )
                                    .await
                                    {
                                        Ok(result) => {
                                            let label = format!("{} ({})", agent.name, agent.id);
                                            agent_conversation
                                                .push((label.clone(), result.trim().to_string()));
                                            format!(
                                                "Agent \"{}\" ({}) result:\n\n{}\n\nUse this to answer the user's question.",
                                                agent.name, agent.id, result
                                            )
                                        }
                                        Err(e) => {
                                            info!("Agent router: AGENT session failed: {}", e);
                                            if is_redmine_agent && is_agent_unavailable_error(&e) {
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
                                            }
                                        }
                                    }
                                }
                            }
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
                                format!(
                                    "Unknown agent \"{}\". Available agents: {}. Answer without using an agent.",
                                    selector, list
                                )
                            }
                        }
                        }
                    }
                    "SCHEDULE" => {
                        crate::commands::task_tool_handlers::handle_schedule(
                            &arg,
                            allow_schedule,
                            discord_reply_channel_id,
                            &status_tx,
                        )
                    }
                    "REMOVE_SCHEDULE" => {
                        crate::commands::task_tool_handlers::handle_remove_schedule(
                            &arg,
                            &status_tx,
                        )
                    }
                    "LIST_SCHEDULES" => {
                        crate::commands::task_tool_handlers::handle_list_schedules(
                            &status_tx,
                        )
                    }
                    "RUN_CMD" => {
                        info!(
                            "Agent router: RUN_CMD requested: {}",
                            crate::logging::ellipse(&arg, 120)
                        );
                        if !crate::commands::run_cmd::is_local_cmd_allowed() {
                            "RUN_CMD is not available (disabled by ALLOW_LOCAL_CMD=0). Answer without running local commands.".to_string()
                        } else if last_run_cmd_arg.as_deref() == Some(arg.as_str()) {
                            info!(
                                "Agent router: RUN_CMD duplicate (same arg as last run), skipping execution"
                            );
                            "You already ran this command; the result is in the message above. Do not run RUN_CMD again. Reply with TASK_APPEND then TASK_STATUS as the task instructs.".to_string()
                        } else {
                            const MAX_CMD_RETRIES: u32 = 3;
                            let mut current_cmd = arg.to_string();
                            let mut last_output = String::new();

                            for attempt in 0..=MAX_CMD_RETRIES {
                                send_status(&format!(
                                    "Running local command{}: {}",
                                    if attempt > 0 {
                                        format!(" (retry {})", attempt)
                                    } else {
                                        String::new()
                                    },
                                    current_cmd
                                ));
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
                                        last_run_cmd_raw_output = Some(output.clone());
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
                                        // Ask Ollama to fix the command (up to 2 attempts: full prompt, then format-only if parse fails)
                                        let allowed =
                                            crate::commands::run_cmd::allowed_commands().join(", ");
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
                                                            info!(
                                                                "Agent router: RUN_CMD fix suggestion not parseable as RUN_CMD (after format retry)"
                                                            );
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
                            last_output
                        }
                    }
                    "PYTHON_SCRIPT" => {
                        if !crate::commands::python_agent::is_python_script_allowed() {
                            "PYTHON_SCRIPT is not available (disabled by ALLOW_PYTHON_SCRIPT=0). Answer without running Python.".to_string()
                        } else {
                            match parse_python_script_from_response(&response_content) {
                        Some((id, topic, script_body)) => {
                            let script_label = format!("{} ({})", id, topic);
                            send_status(&format!("Running Python script '{}'…", script_label));
                            info!("Agent router: PYTHON_SCRIPT running script '{}' (id={}, topic={}, body {} chars)", script_label, id, topic, script_body.len());
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
                    }
                    "DISCORD_API" => {
                        let arg = arg.trim();
                        let (method, rest) = match arg.find(' ') {
                            Some(i) => (arg[..i].trim().to_string(), arg[i..].trim()),
                            None => ("GET".to_string(), arg),
                        };
                        let (path_raw, body) = if let Some(idx) = rest.find(" {") {
                            let (p, b) = rest.split_at(idx);
                            (p.trim().to_string(), Some(b.trim().to_string()))
                        } else {
                            (rest.to_string(), None)
                        };
                        let path = normalize_discord_api_path(&path_raw);
                        if path.is_empty() {
                            "DISCORD_API requires: DISCORD_API: <METHOD> <path> or DISCORD_API: POST <path> {\"content\":\"...\"}.".to_string()
                        } else if last_successful_discord_call
                            .as_ref()
                            .map(|(m, p)| m == &method && p == &path)
                            .unwrap_or(false)
                        {
                            "You already received the data for this endpoint above. Format it for the user and reply; do not call DISCORD_API again for the same path.".to_string()
                        } else {
                            let status_msg = format!("Calling Discord API: {} {}", method, path);
                            send_status(&status_msg);
                            info!("Discord API: {} {}", method, path);
                            match crate::discord::api::discord_api_request(
                                &method,
                                &path,
                                body.as_deref(),
                            )
                            .await
                            {
                                Ok(result) => {
                                    last_successful_discord_call =
                                        Some((method.clone(), path.clone()));
                                    format!(
                                        "Discord API result:\n\n{}\n\nUse this to answer the user's question.",
                                        result
                                    )
                                }
                                Err(e) => {
                                    let msg = crate::discord::api::sanitize_discord_api_error(&e);
                                    format!(
                                        "Discord API failed: {}. Answer without this result.",
                                        msg
                                    )
                                }
                            }
                        }
                    }
                    "OLLAMA_API" => {
                        let arg = arg.trim();
                        let (action, rest) = match arg.find(' ') {
                            Some(i) => (arg[..i].trim().to_lowercase(), arg[i..].trim()),
                            None => (arg.to_lowercase(), ""),
                        };
                        let status_detail = if rest.is_empty() {
                            format!("Ollama API: {}…", action)
                        } else {
                            let preview: String = rest.chars().take(40).collect();
                            format!("Ollama API: {} {}…", action, preview)
                        };
                        send_status(&status_detail);
                        info!(
                            "Agent router: OLLAMA_API requested: action={}, rest={} chars",
                            action,
                            rest.chars().count()
                        );
                        let result = match action.as_str() {
                            "list_models" => list_ollama_models_full().await.map(|r| {
                                serde_json::to_string_pretty(&r)
                                    .unwrap_or_else(|_| "[]".to_string())
                            }),
                            "version" => get_ollama_version().await.map(|r| r.version),
                            "running" => list_ollama_running_models().await.map(|r| {
                                serde_json::to_string_pretty(&r)
                                    .unwrap_or_else(|_| "[]".to_string())
                            }),
                            "pull" => {
                                let parts: Vec<&str> = rest.split_whitespace().collect();
                                let model =
                                    parts.first().map(|s| (*s).to_string()).unwrap_or_default();
                                let stream = parts.get(1).map(|s| *s == "true").unwrap_or(true);
                                if model.is_empty() {
                                    Err("OLLAMA_API pull requires a model name.".to_string())
                                } else {
                                    pull_ollama_model(model, stream)
                                        .await
                                        .map(|_| "Pull completed.".to_string())
                                }
                            }
                            "delete" => {
                                let model = rest.to_string();
                                if model.is_empty() {
                                    Err("OLLAMA_API delete requires a model name.".to_string())
                                } else {
                                    delete_ollama_model(model)
                                        .await
                                        .map(|_| "Model deleted.".to_string())
                                }
                            }
                            "embed" => {
                                let parts: Vec<&str> = rest.splitn(2, ' ').map(str::trim).collect();
                                if parts.len() < 2 || parts[1].is_empty() {
                                    Err("OLLAMA_API embed requires: embed <model> <text>."
                                        .to_string())
                                } else {
                                    let model = parts[0].to_string();
                                    let input = serde_json::Value::String(parts[1].to_string());
                                    ollama_embeddings(model, input, None).await.map(|r| {
                                        serde_json::to_string_pretty(&r)
                                            .unwrap_or_else(|_| "{}".to_string())
                                    })
                                }
                            }
                            "load" => {
                                let parts: Vec<&str> =
                                    rest.splitn(2, char::is_whitespace).map(str::trim).collect();
                                let model =
                                    parts.first().map(|s| (*s).to_string()).unwrap_or_default();
                                let keep_alive = parts
                                    .get(1)
                                    .filter(|s| !s.is_empty())
                                    .map(|s| (*s).to_string());
                                if model.is_empty() {
                                    Err("OLLAMA_API load requires a model name.".to_string())
                                } else {
                                    load_ollama_model(model, keep_alive)
                                        .await
                                        .map(|_| "Model loaded.".to_string())
                                }
                            }
                            "unload" => {
                                let model = rest.to_string();
                                if model.is_empty() {
                                    Err("OLLAMA_API unload requires a model name.".to_string())
                                } else {
                                    unload_ollama_model(model)
                                        .await
                                        .map(|_| "Model unloaded.".to_string())
                                }
                            }
                            _ => Err(format!(
                                "Unknown OLLAMA_API action: {}. Use list_models, version, running, pull, delete, embed, load, or unload.",
                                action
                            )),
                        };
                        match result {
                            Ok(msg) => format!(
                                "Ollama API result:\n\n{}\n\nUse this to answer the user's question.",
                                msg
                            ),
                            Err(e) => {
                                format!("OLLAMA_API failed: {}. Answer without this result.", e)
                            }
                        }
                    }
                    "TASK_APPEND" => {
                        crate::commands::task_tool_handlers::handle_task_append(
                            &arg,
                            &mut current_task_path,
                            &mut last_run_cmd_raw_output,
                            &status_tx,
                        )
                    }
                    "TASK_STATUS" => {
                        crate::commands::task_tool_handlers::handle_task_status(
                            &arg,
                            &mut current_task_path,
                        )
                    }
                    "TASK_CREATE" => {
                        crate::commands::task_tool_handlers::handle_task_create(
                            &arg,
                            discord_reply_channel_id,
                            &mut current_task_path,
                        )
                    }
                    "TASK_SHOW" => {
                        crate::commands::task_tool_handlers::handle_task_show(
                            &arg,
                            &mut current_task_path,
                            &status_tx,
                        )
                    }
                    "TASK_ASSIGN" => {
                        crate::commands::task_tool_handlers::handle_task_assign(
                            &arg,
                            &mut current_task_path,
                            &status_tx,
                        )
                    }
                    "TASK_SLEEP" => {
                        crate::commands::task_tool_handlers::handle_task_sleep(
                            &arg,
                            &mut current_task_path,
                            &status_tx,
                        )
                    }
                    "TASK_LIST" => {
                        crate::commands::task_tool_handlers::handle_task_list(
                            &arg,
                            &status_tx,
                        )
                    }
                    "MCP" => {
                        send_status("Calling MCP tool…");
                        info!(
                            "Agent router: MCP requested (arg len={})",
                            arg.chars().count()
                        );
                        match crate::mcp::get_mcp_server_url() {
                            Some(server_url) => {
                                let (mcp_tool_name, mcp_args) = if let Some(space) = arg.find(' ') {
                                    let (name, rest) = arg.split_at(space);
                                    let rest = rest.trim();
                                    let args = if rest.starts_with('{') {
                                        serde_json::from_str(rest).ok()
                                    } else {
                                        Some(serde_json::json!({ "input": rest }))
                                    };
                                    (name.to_string(), args)
                                } else {
                                    (arg.clone(), None)
                                };
                                match crate::mcp::call_tool(&server_url, &mcp_tool_name, mcp_args)
                                    .await
                                {
                                    Ok(result) => {
                                        info!(
                                            "Agent router: MCP tool {} completed ({} chars)",
                                            mcp_tool_name,
                                            result.len()
                                        );
                                        format!(
                                            "MCP tool \"{}\" result:\n\n{}\n\nUse this to answer the user's question.",
                                            mcp_tool_name, result
                                        )
                                    }
                                    Err(e) => {
                                        info!(
                                            "Agent router: MCP tool {} failed: {}",
                                            mcp_tool_name, e
                                        );
                                        format!(
                                            "MCP tool \"{}\" failed: {}. Answer the user without this result.",
                                            mcp_tool_name, e
                                        )
                                    }
                                }
                            }
                            None => {
                                info!("Agent router: MCP not configured (no MCP_SERVER_URL)");
                                "MCP is not configured (set MCP_SERVER_URL in env or .config.env). Answer without using MCP.".to_string()
                            }
                        }
                    }
                    "CURSOR_AGENT" => {
                        if !crate::commands::cursor_agent::is_cursor_agent_available() {
                            "CURSOR_AGENT is not available (cursor-agent CLI not found on PATH). Answer without it.".to_string()
                        } else {
                            let prompt = arg.trim().to_string();
                            if prompt.is_empty() {
                                "CURSOR_AGENT requires a prompt: CURSOR_AGENT: <detailed coding task>".to_string()
                            } else {
                                let preview: String = prompt.chars().take(80).collect();
                                send_status(&format!("Running Cursor Agent: {}…", preview));
                                info!(
                                    "Agent router: CURSOR_AGENT running prompt ({} chars)",
                                    prompt.len()
                                );
                                match tokio::task::spawn_blocking({
                                    let p = prompt.clone();
                                    move || crate::commands::cursor_agent::run_cursor_agent(&p)
                                })
                                .await
                                .map_err(|e| format!("CURSOR_AGENT task: {}", e))
                                .and_then(|r| r)
                                {
                                    Ok(output) => {
                                        info!(
                                            "Agent router: CURSOR_AGENT completed ({} chars output)",
                                            output.len()
                                        );
                                        let truncated = if output.chars().count() > 4000 {
                                            let half = 1800;
                                            let start: String = output.chars().take(half).collect();
                                            let end: String = output
                                                .chars()
                                                .rev()
                                                .take(half)
                                                .collect::<String>()
                                                .chars()
                                                .rev()
                                                .collect();
                                            format!("{}...\n[truncated]\n...{}", start, end)
                                        } else {
                                            output
                                        };
                                        format!(
                                            "Cursor Agent result:\n\n{}\n\nUse this to answer the user's question.",
                                            truncated
                                        )
                                    }
                                    Err(e) => {
                                        info!("Agent router: CURSOR_AGENT failed: {}", e);
                                        format!(
                                            "CURSOR_AGENT failed: {}. Answer the user without this result.",
                                            e
                                        )
                                    }
                                }
                            }
                        }
                    }
                    "REDMINE_API" => {
                        let arg = arg.trim();
                        let (method, rest) = match arg.find(' ') {
                            Some(i) => (arg[..i].trim().to_string(), arg[i..].trim()),
                            None => ("GET".to_string(), arg),
                        };
                        let (path, body) = if let Some(idx) = rest.find(" {") {
                            let (p, b) = rest.split_at(idx);
                            (p.trim().to_string(), Some(b.trim().to_string()))
                        } else {
                            (rest.to_string(), None)
                        };
                        if path.is_empty() {
                            "REDMINE_API requires: REDMINE_API: GET /issues/1234.json?include=journals,attachments".to_string()
                        } else {
                            send_status(&format!("Querying Redmine: {} {}", method, path));
                            info!(
                                "Agent router [{}]: REDMINE_API {} {}",
                                request_id, method, path
                            );
                            match crate::redmine::redmine_api_request(
                                &method,
                                &path,
                                body.as_deref(),
                            )
                            .await
                            {
                                Ok(result) => {
                                    let mut msg = if path.contains("time_entries") {
                                        format!(
                                            "Redmine API result:\n\n{}\n\nUse this data to answer the user's question. The derived summary above already lists the actual tickets worked (if any), totals, users, projects, and entry details. Use that instead of inventing ticket ids or subjects.",
                                            result
                                        )
                                    } else {
                                        format!(
                                            "Redmine API result:\n\n{}\n\nUse this data to answer the user's question. Summarize the issue clearly: subject, description quality, what's missing, status, assignee, and key comments.",
                                            result
                                        )
                                    };
                                    if method.to_uppercase() == "GET" {
                                        if is_redmine_review_or_summarize_only(question) {
                                            msg.push_str(
                                        "\n\nThe user asked only to review/summarize. Do NOT update the ticket or add a comment. Reply with your summary and DONE: success.",
                                    );
                                        } else if path.contains("time_entries") {
                                                msg.push_str(
                                            "\n\nUse this data to answer. If the user asked for tickets worked, list the actual issue ids and subjects from the derived summary. If the user asked for \"this month\", use from/to for the current month. If success criteria require JSON format, reply with valid JSON only (e.g. total hours, ticket list, project breakdown).",
                                        );
                                            } else {
                                                let id = path
                                                    .trim_start_matches('/')
                                                    .strip_prefix("issues/")
                                                    .map(|s| {
                                                        s.split(['.', '?'])
                                                            .next()
                                                            .unwrap_or("")
                                                            .to_string()
                                                    })
                                                    .unwrap_or_default();
                                                if !id.is_empty()
                                                    && id.chars().all(|c| c.is_ascii_digit())
                                                {
                                                    msg.push_str(&format!(
                                                "\n\nIf the user asked to **update** this ticket or **add a comment**, your next reply MUST be exactly one line: REDMINE_API: PUT /issues/{}.json {{\"issue\":{{\"notes\":\"<your comment text>\"}}}}. Do not reply with only a summary.",
                                                id
                                            ));
                                                }
                                            }
                                    }
                                    msg
                                }
                                Err(e) => format!(
                                    "Redmine API failed: {}. Answer without this result.",
                                    e
                                ),
                            }
                        }
                    }
                    "MASTODON_POST" => {
                        let arg = arg.trim();
                        if arg.is_empty() {
                            "MASTODON_POST requires text. Usage: MASTODON_POST: <text to post>. Optional visibility prefix: MASTODON_POST: unlisted: <text> (default: public).".to_string()
                        } else {
                            let (visibility, text) = if let Some(rest) = arg
                                .strip_prefix("unlisted:")
                                .or_else(|| arg.strip_prefix("unlisted "))
                            {
                                ("unlisted", rest.trim())
                            } else if let Some(rest) = arg
                                .strip_prefix("private:")
                                .or_else(|| arg.strip_prefix("private "))
                            {
                                ("private", rest.trim())
                            } else if let Some(rest) = arg
                                .strip_prefix("direct:")
                                .or_else(|| arg.strip_prefix("direct "))
                            {
                                ("direct", rest.trim())
                            } else if let Some(rest) = arg
                                .strip_prefix("public:")
                                .or_else(|| arg.strip_prefix("public "))
                            {
                                ("public", rest.trim())
                            } else {
                                ("public", arg)
                            };
                            send_status(&format!("Posting to Mastodon ({})…", visibility));
                            info!(
                                "Agent router: MASTODON_POST visibility={} text={}",
                                visibility,
                                crate::logging::ellipse(text, 100)
                            );
                            match mastodon_post(text, visibility).await {
                                Ok(msg) => msg,
                                Err(e) => format!("Mastodon post failed: {}", e),
                            }
                        }
                    }
                    "MEMORY_APPEND" => {
                        let arg = arg.trim();
                        if arg.is_empty() {
                            "MEMORY_APPEND requires content. Usage: MEMORY_APPEND: <lesson> or MEMORY_APPEND: agent:<slug-or-id> <lesson>".to_string()
                        } else {
                            let (target, lesson) = if arg.to_lowercase().starts_with("agent:") {
                                let rest = arg["agent:".len()..].trim();
                                if let Some(space_idx) = rest.find(' ') {
                                    let (sel, content) = rest.split_at(space_idx);
                                    (Some(sel.trim().to_string()), content.trim().to_string())
                                } else {
                                    (None, arg.to_string())
                                }
                            } else {
                                (None, arg.to_string())
                            };
                            let lesson_line = format!("- {}\n", lesson.trim_start_matches("- "));
                            let result = if let Some(selector) = target {
                                let agents = crate::agents::load_agents();
                                if let Some(agent) =
                                    crate::agents::find_agent_by_id_or_name(&agents, &selector)
                                {
                                    if let Some(dir) = crate::agents::get_agent_dir(&agent.id) {
                                        let path = dir.join("memory.md");
                                        append_to_file(&path, &lesson_line)
                                    } else {
                                        Err(format!("Agent directory not found for '{}'", selector))
                                    }
                                } else {
                                    Err(format!("Agent '{}' not found", selector))
                                }
                            } else {
                                let path = discord_reply_channel_id
                                    .map(
                                        crate::config::Config::memory_file_path_for_discord_channel,
                                    )
                                    .unwrap_or_else(crate::config::Config::memory_file_path);
                                append_to_file(&path, &lesson_line)
                            };
                            match result {
                                Ok(path) => {
                                    info!("Agent router: MEMORY_APPEND wrote to {:?}", path);
                                    format!(
                                        "Memory updated ({}). The lesson will be included in future prompts.",
                                        path.display()
                                    )
                                }
                                Err(e) => {
                                    info!("Agent router: MEMORY_APPEND failed: {}", e);
                                    format!("Failed to update memory: {}", e)
                                }
                            }
                        }
                    }
                    _ => {
                        let msg = format!(
                            "Unknown tool \"{}\". Use one of the available tools from the agent list (e.g. FETCH_URL, BRAVE_SEARCH, BROWSER_NAVIGATE, DONE) or reply with your answer.",
                            tool
                        );
                        tracing::warn!(
                            "Agent router: unknown tool \"{}\" (arg: {} chars), sending hint to model",
                            tool,
                            arg.chars().count()
                        );
                        msg
                    }
                };

                let result_len = user_message.chars().count();
                info!(
                    "Agent router: tool {} completed, sending result back to Ollama ({} chars): {}",
                    tool,
                    result_len,
                    log_content(&user_message)
                );

                if tool == "RUN_CMD" && user_message.starts_with("Here is the command output") {
                    last_run_cmd_arg = Some(arg.clone());
                } else if tool != "RUN_CMD" {
                    last_run_cmd_arg = None;
                }
                // Only clear stored RUN_CMD output when we run something other than RUN_CMD or TASK_APPEND
                // (so it stays set for the next TASK_APPEND after RUN_CMD).
                if tool != "TASK_APPEND" && tool != "RUN_CMD" {
                    last_run_cmd_raw_output = None;
                }
                if tool == "BROWSER_EXTRACT"
                    && !user_message.is_empty()
                    && !user_message.contains("BROWSER_EXTRACT failed")
                {
                    last_browser_extract = Some(user_message.clone());
                }

                let is_browser_error = if tool == "BROWSER_SCREENSHOT" {
                    user_message.starts_with("Screenshot of current page failed")
                        || user_message.starts_with("Screenshot task error")
                } else if tool.starts_with("BROWSER_") {
                    user_message.starts_with(&format!("{} failed", tool))
                        || user_message.starts_with(&format!("{} task error", tool))
                        || user_message.starts_with(&format!("{} HTTP fallback task error", tool))
                        || user_message.starts_with(&format!("{} CDP retry task error", tool))
                } else {
                    false
                };

                let is_multi_tool_run_cmd_error = tool == "RUN_CMD"
                    && user_message.starts_with("RUN_CMD failed in a multi-step plan");

                // Sequence-terminating: after a successful BROWSER_NAVIGATE or BROWSER_GO_BACK,
                // subsequent browser tools in this turn use stale indices and must be skipped.
                if multi_tool_turn
                    && !is_browser_error
                    && (tool == "BROWSER_NAVIGATE" || tool == "BROWSER_GO_BACK")
                    && !user_message.starts_with("BROWSER_NAVIGATE requires")
                {
                    page_changed_this_turn = true;
                    info!(
                        "Agent router: {} terminates sequence — remaining browser actions this turn will be skipped",
                        tool
                    );
                }

                tool_results.push(user_message);
                if let Some(pair) = executed_browser_tool_arg {
                    last_browser_tool_arg = Some(pair);
                }
                if is_browser_error || is_multi_tool_run_cmd_error {
                    info!(
                        "Agent router: {} returned an error, aborting remaining tools in this turn",
                        tool
                    );
                    break;
                }
            }

            let user_message = tool_results.join("\n\n---\n\n");
            if let Some(blocked_reply) = grounded_redmine_time_entries_failure_reply(
                &request_for_verification,
                &user_message,
            ) {
                info!("Agent router: returning grounded Redmine blocked-state reply");
                response_content = blocked_reply;
                break;
            }
            if !question_explicitly_requests_json(question) {
                if let Some(summary) = extract_redmine_time_entries_summary_for_reply(&user_message)
                {
                    info!("Agent router: returning direct Redmine time-entry summary");
                    response_content = summary;
                    if done_claimed.is_some() {
                        exited_via_done = true;
                    }
                    break;
                }
            }
            if done_claimed.is_some() {
                if !user_message.trim().is_empty() {
                    response_content = final_reply_from_tool_results(question, &user_message);
                }
                exited_via_done = true;
                break;
            }

            messages.push(crate::ollama::ChatMessage {
                role: "assistant".to_string(),
                content: response_content.clone(),
                images: None,
            });
            let tool_result_role = if user_message.starts_with("Here is the command output") {
                "system"
            } else {
                "user"
            };
            messages.push(crate::ollama::ChatMessage {
                role: tool_result_role.to_string(),
                content: user_message,
                images: None,
            });

            // Budget warning / last-iteration guidance (inspired by browser-use step budget).
            if max_tool_iterations > 1 && budget_warning_ratio > 0.0 && budget_warning_ratio < 1.0 {
                let ratio = (tool_count as f64) / (max_tool_iterations as f64);
                if tool_count + 1 == max_tool_iterations {
                    let msg = format!(
                        "LAST ITERATION WARNING: You have used {}/{} tool iterations. This is your LAST tool iteration. \
                         Reply with your final answer now. Summarize everything you have found so far. \
                         Do NOT start a new tool chain — respond with your best answer or call DONE with your results.",
                        tool_count, max_tool_iterations
                    );
                    info!("Agent router: injecting last-iteration guidance (tool_count={}/{})", tool_count, max_tool_iterations);
                    messages.push(crate::ollama::ChatMessage {
                        role: "system".to_string(),
                        content: msg,
                        images: None,
                    });
                } else if ratio >= budget_warning_ratio {
                    let remaining = max_tool_iterations - tool_count;
                    let pct = (ratio * 100.0) as u32;
                    let msg = format!(
                        "BUDGET WARNING: You have used {}/{} tool iterations ({}%). {} iterations remaining. \
                         If the task cannot be completed in the remaining iterations, prioritize: \
                         (1) consolidate your results so far, (2) reply with what you have or call DONE. \
                         Partial results are far more valuable than exhausting all iterations with nothing saved.",
                        tool_count, max_tool_iterations, pct, remaining
                    );
                    info!("Agent router: injecting budget warning (tool_count={}/{}, ratio={:.2}, threshold={:.2})", tool_count, max_tool_iterations, ratio, budget_warning_ratio);
                    messages.push(crate::ollama::ChatMessage {
                        role: "system".to_string(),
                        content: msg,
                        images: None,
                    });
                }
            }

            let follow_up = send_ollama_chat_messages(
                messages.clone(),
                model_override.clone(),
                options_override.clone(),
            )
            .await?;
            response_content = follow_up.message.content.clone();

            // Fallback: if Ollama returned empty after a successful tool result, use the raw
            // tool output directly so the user at least sees what the tool produced.
            if response_content.trim().is_empty() {
                if let Some(last_msg) = messages.last() {
                    let raw = &last_msg.content;
                    if raw.starts_with("Here is the command output")
                        || raw.starts_with("Here is the page content")
                        || raw.starts_with("MCP tool")
                        || raw.starts_with("Search results")
                        || raw.starts_with("Discord API")
                    {
                        info!(
                            "Agent router: Ollama returned empty after tool success — using raw tool output as response"
                        );
                        // Strip the instruction suffix we appended for the model
                        let cleaned = raw
                            .replace("\n\nUse this to answer the user's question.", "")
                            .replace("Here is the command output:\n\n", "")
                            .replace("Here is the page content:\n\n", "");
                        response_content = cleaned;
                    }
                }
            }

            if tool_count >= max_tool_iterations {
                info!(
                    "Agent router: max tool iterations reached ({}), using last response as final",
                    max_tool_iterations
                );
            }
        }

        if exited_via_done {
            let lines: Vec<&str> = response_content.lines().collect();
            if let Some(last) = lines.last() {
                let t = last.trim();
                if t.to_uppercase().starts_with("DONE") && t.contains(':') {
                    let keep = lines.len().saturating_sub(1);
                    response_content = lines[..keep].join("\n").trim_end().to_string();
                }
            }
        }

        let final_len = response_content.chars().count();
        info!(
            "Agent router: done after {} tool(s), returning final response ({} chars): {}",
            tool_count,
            final_len,
            log_content(&response_content)
        );

        // When multiple agents participated, ensure the user sees the conversation: append a transcript if we have 2+ agent turns and the final reply is short (so we don't hide a long model summary).
        if agent_conversation.len() >= 2 {
            const SHORT_REPLY_THRESHOLD: usize = 500;
            if response_content.chars().count() < SHORT_REPLY_THRESHOLD
                || response_content.contains("Thank you for providing")
                || response_content.contains("If you have any specific tasks")
            {
                let mut transcript = String::from("\n\n---\n**Conversation:**\n\n");
                for (label, reply) in &agent_conversation {
                    transcript.push_str("**");
                    transcript.push_str(label);
                    transcript.push_str(":**\n");
                    transcript.push_str(reply);
                    transcript.push_str("\n\n");
                }
                response_content.push_str(transcript.trim_end());
            }
        }

        // Log the full conversation (user question + assistant reply) into the task file when we touched a task this run.
        // Skip when the "user" message is the task runner's prompt (synthetic), so we don't log runner turns as User/Assistant.
        let is_runner_prompt = question
            .trim_start()
            .starts_with("Current task file content:");
        if let Some(ref path) = current_task_path {
            if !is_runner_prompt {
                if let Err(e) =
                    crate::task::append_conversation_block(path, question, &response_content)
                {
                    info!(
                        "Agent router: could not append conversation to task file: {}",
                        e
                    );
                } else {
                    info!(
                        "Agent router: appended conversation to task {}",
                        crate::task::task_file_name(path)
                    );
                }
            } else {
                info!(
                    "Agent router: skipped appending conversation (task runner turn) for {}",
                    crate::task::task_file_name(path)
                );
            }
        }

        // Heuristic guard: screenshot requested but no attachment
        let user_asked_screenshot = user_explicitly_asked_for_screenshot(&request_for_verification);
        if (user_asked_screenshot || screenshot_requested_by_tool_run)
            && attachment_paths.is_empty()
        {
            response_content
                .push_str("\n\nNote: A screenshot was requested but none was attached.");
            info!(
                "Agent router: heuristic guard — screenshot requested but no attachment, appended note"
            );
        }

        // User-facing note when browser action limit was reached (032_browser_loop_and_status_fix_plan).
        if browser_tool_cap_reached {
            response_content.push_str(&format!(
                "\n\nNote: Browser action limit ({} per run) was reached; some actions were skipped.",
                MAX_BROWSER_TOOLS_PER_RUN
            ));
            info!(
                "Agent router: browser tool cap was reached, appended user-facing note"
            );
        }

        // Completion verification: one short Ollama call; if not satisfied, retry once (A2) or append disclaimer
        let criteria_count = success_criteria.as_ref().map(|c| c.len()).unwrap_or(0);
        info!(
            "Agent router [{}]: running completion verification ({} criteria, {} attachment(s))",
            request_id,
            criteria_count,
            attachment_paths.len()
        );
        match verify_completion(
            &request_for_verification,
            &response_content,
            &attachment_paths,
            success_criteria.as_deref(),
            last_browser_extract.as_deref(),
            last_news_search_was_hub_only,
            model_override.clone(),
            options_override.clone(),
        )
        .await
        {
            Ok((false, reason)) => {
                // Use only the user's question for memory search — not the verification reason.
                // The reason contains generic words ("request", "verified", "assistant", "tools") that
                // match unrelated memory (e.g. GitLab/Redmine) and confuse the retry.
                let memory_snippet = search_memory_for_request(
                    &request_for_verification,
                    None,
                    discord_reply_channel_id,
                );
                if retry_on_verification_no {
                    let retry_base = reason
                    .as_deref()
                    .map(|r| format!("Verification said we didn't fully complete: {}. Complete the remaining steps now, then reply.", r.trim()))
                    .unwrap_or_else(|| "Verification said we didn't fully complete. Complete the remaining steps now, then reply.".to_string());
                    // When verifier said no and the reason mentions screenshots/attachments, steer the
                    // model to reply with a summary and DONE — do not invoke AGENT: orchestrator or
                    // create tasks, which can lead to unrelated actions (e.g. organize ~/tmp).
                    let reason_lower = reason
                        .as_deref()
                        .map(|r| r.to_lowercase())
                        .unwrap_or_default();
                    let reason_about_attachments = reason_lower.contains("screenshot")
                        || reason_lower.contains("attachment")
                        || (reason_lower.contains("missing")
                            && (reason_lower.contains("upload") || reason_lower.contains("sent")));
                    let reason_about_time_or_data = reason_lower.contains("time")
                        || reason_lower.contains("actual data")
                        || reason_lower.contains("spent time")
                        || reason_lower.contains("project parameter")
                        || (reason_lower.contains("missing")
                            && (reason_lower.contains("data")
                                || reason_lower.contains("parameter")));
                    let reason_about_json_format = reason_lower.contains("json format")
                        || reason_lower.contains("not in json")
                        || (reason_lower.contains("response") && reason_lower.contains("json"));
                    let reason_about_news_sourcing = reason_lower.contains("source")
                        || reason_lower.contains("date")
                        || reason_lower.contains("publication")
                        || reason_lower.contains("credible")
                        || reason_lower.contains("article");
                    let response_has_ticket_summary = response_content.len() > 150
                        && (response_content.contains("Subject")
                            || response_content.contains("Description")
                            || response_content.contains("Status")
                            || response_content.contains("Redmine"));
                    let retry_base_with_hint = if is_redmine_time_entries_request(
                        &request_for_verification,
                    ) {
                        let (from, to) = redmine_time_entries_range(&request_for_verification);
                        format!(
                            "This is a Redmine time-entry list/report request. Stay in that domain only. Do not use BROWSER_*, FETCH_URL, RUN_CMD, TASK_*, or single-issue endpoints like /issues/{{id}}.json unless the user explicitly asks for them. Base the answer only on the actual Redmine time-entry data already fetched, or re-fetch the same period with REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100 if needed. If the result is empty, say no time entries or worked tickets were found for that period. Do not mention screenshots or attachments. Do not return raw tool directives as the final user answer.\n\n{}",
                            from, to, retry_base
                        )
                    } else if is_redmine_review_or_summarize_only(&request_for_verification)
                        && response_has_ticket_summary
                    {
                        "The request was only to review/summarize. A summary was already provided. Reply with a brief confirmation and DONE: success; do not update or close the ticket.".to_string()
                    } else if reason_about_json_format {
                        format!(
                            "Success criteria require a response in JSON format. Reply with **valid JSON only** (e.g. total hours, project breakdown, user contributions); do not reply with prose or markdown lists.\n\n{}",
                            retry_base
                        )
                    } else if is_news_query(&request_for_verification) && reason_about_news_sourcing
                    {
                        info!(
                            "Agent router: verification NO for news (article-grade/sourcing); retrying with PERPLEXITY_SEARCH hint"
                        );
                        format!(
                            "This is a news-summary request. Stay in search-and-summary mode only. Re-run PERPLEXITY_SEARCH with a refined query if needed, but do **not** browse generic homepages, do **not** open BBC/CNN/NYTimes landing pages, and do **not** use screenshots or attachments. Reply with 3 concise bullet points. Each bullet must include: headline/topic, source name, publication date, and one-sentence factual summary. Prefer article-like results; if only hub/landing pages are available, say that clearly.\n\n{}",
                            retry_base
                        )
                    } else if reason_about_time_or_data
                        && (request_for_verification.to_lowercase().contains("redmine")
                            || request_for_verification.to_lowercase().contains("time")
                            || request_for_verification.to_lowercase().contains("spent"))
                    {
                        format!(
                            "Use the correct Redmine API for time entries: REDMINE_API: GET /time_entries.json with from= and to= for the requested period (for example 2026-03-01..2026-03-31 for this month, or the same day for today) and include limit=100 so the results are not truncated. Omit optional filters like user_id or project_id unless the user explicitly asked for them. Do not use /search.json for time entries. Then reply with the data or a clear summary.\n\n{}",
                            retry_base
                        )
                    } else if !attachment_paths.is_empty() && reason_about_attachments {
                        let n = attachment_paths.len();
                        format!(
                            "The app already attached {} file(s) to this reply. If the only missing item was screenshots/attachments, \
                         reply with a brief summary of what was done and end with **DONE: success**. \
                         Do not invoke AGENT: orchestrator and do not create new tasks.\n\n{}",
                            n, retry_base
                        )
                    } else if attachment_paths.is_empty()
                        && user_explicitly_asked_for_screenshot(&request_for_verification)
                        && reason_lower.contains("screenshot")
                    {
                        format!(
                            "Screenshots could not be attached to this reply. Reply with a brief summary of what was done (e.g. screenshot taken and saved), state that the app could not attach it to Discord, and end with **DONE: no**. \
                         Do not invoke AGENT: orchestrator and do not create new tasks.\n\n{}",
                            retry_base
                        )
                    } else if reason_lower.contains("cookie")
                        || (reason_lower.contains("consent") && reason_lower.contains("banner"))
                    {
                        format!(
                            "Original request: \"{}\". Verification said the cookie consent banner was not addressed. A screenshot may already have been taken. Complete the remaining step: dismiss the cookie banner (BROWSER_CLICK on the consent button using the Elements list) then BROWSER_SCREENSHOT: current if needed, or reply with a brief summary and **DONE: no** if the browser session is no longer available.\n\n{}",
                            request_for_verification.trim(),
                            retry_base
                        )
                    } else if is_browser_task_request(&request_for_verification)
                        && (last_browser_extract.is_some()
                            || response_content.contains("Current page:")
                            || reason_lower.contains("browser")
                            || reason_lower.contains("click")
                            || reason_lower.contains("video")
                            || reason_lower.contains("screenshot"))
                    {
                        browser_retry_grounding_prompt(&request_for_verification, &retry_base)
                    } else {
                        retry_base
                    };
                    let retry_question = if is_news_query(&request_for_verification) {
                        retry_base_with_hint
                    } else if let Some(ref from_memory) = memory_snippet {
                        format!(
                            "From memory (possibly relevant):\n{}\n\n{}",
                            from_memory, retry_base_with_hint
                        )
                    } else {
                        retry_base_with_hint
                    };
                    info!(
                        "Agent router [{}]: verification said not satisfied, retrying once with: {}...",
                        request_id,
                        retry_question.chars().take(60).collect::<String>()
                    );
                    let mut updated_history: Vec<crate::ollama::ChatMessage> =
                        conversation_history.clone();
                    updated_history.push(crate::ollama::ChatMessage {
                        role: "user".to_string(),
                        content: question.to_string(),
                        images: None,
                    });
                    updated_history.push(crate::ollama::ChatMessage {
                        role: "assistant".to_string(),
                        content: response_content.clone(),
                        images: None,
                    });
                    let pass_intermediate =
                        discord_reply_channel_id.map(|_| response_content.clone());
                    return answer_with_ollama_and_fetch(
                        retry_question.as_str(),
                        status_tx,
                        discord_reply_channel_id,
                        discord_user_id,
                        discord_user_name,
                        model_override,
                        options_override,
                        skill_content,
                        agent_override,
                        allow_schedule,
                        Some(updated_history),
                        escalation,
                        false, // don't retry again
                        from_remote,
                        None,              // don't re-send attachment images on retry
                        pass_intermediate, // format reply as Intermediate + Final when returning to Discord
                        true,              // is_verification_retry: keep context, skip NEW_TOPIC
                        Some(request_for_verification.clone()),
                        success_criteria.clone(),
                        discord_is_dm,
                        Some(request_id.clone()), // same request_id for end-to-end log correlation
                        1,                       // retry_count
                    )
                    .await;
                }
                let reason_preview = reason
                    .as_deref()
                    .map(|r| r.chars().take(80).collect::<String>())
                    .unwrap_or_default();
                // Handoff: when local model didn't satisfy, try cursor-agent for any task (coding or general, e.g. news/screenshot requests).
                let try_cursor_agent_handoff =
                    crate::commands::cursor_agent::is_cursor_agent_available();
                if try_cursor_agent_handoff {
                    info!(
                        "Agent router: verification not satisfied, handing off to Cursor Agent (general fallback)"
                    );
                    let request_clone = request_for_verification.clone();
                    match tokio::task::spawn_blocking(move || {
                        crate::commands::cursor_agent::run_cursor_agent(&request_clone)
                    })
                    .await
                    .map_err(|e| format!("Cursor Agent handoff task: {}", e))
                    .and_then(|r| r)
                    {
                        Ok(cursor_result) => {
                            info!(
                                "Agent router: cursor-agent handoff completed ({} chars)",
                                cursor_result.len()
                            );
                            response_content.clear();
                            response_content.push_str(
                                "The local model couldn't fully complete this, so I handed it off to Cursor Agent. Here's what it did:\n\n",
                            );
                            response_content.push_str(cursor_result.trim());
                        }
                        Err(e) => {
                            info!("Agent router: cursor-agent handoff failed: {}", e);
                            let disclaimer = reason
                                .map(|r| {
                                    format!(
                                        "\n\nNote: We may not have fully met your request: {}.",
                                        r.trim()
                                    )
                                })
                                .unwrap_or_else(|| {
                                    "\n\nNote: We may not have fully met your request.".to_string()
                                });
                            response_content.push_str(&disclaimer);
                        }
                    }
                } else {
                let disclaimer = reason
                    .map(|r| {
                        format!(
                            "\n\nNote: We may not have fully met your request: {}.",
                            r.trim()
                        )
                    })
                    .unwrap_or_else(|| {
                        "\n\nNote: We may not have fully met your request.".to_string()
                    });
                response_content.push_str(&disclaimer);
                }
                // Do not append "From past sessions" memory dump to the user-visible reply — it creates a messy mix and confuses Discord users.
                if let Some(ref from_memory) = memory_snippet {
                    info!(
                        "Agent router: memory search found {} chars (not appended to reply)",
                        from_memory.len()
                    );
                }
                info!(
                    "Agent router: verification said not satisfied, appended disclaimer (reason: {}...)",
                    reason_preview
                );
            }
            Ok((true, _)) => {
                info!("Agent router: verification passed (satisfied)");
            }
            Err(e) => {
                tracing::debug!("Agent router: verification failed (ignored): {}", e);
            }
        }

        let final_text = if discord_reply_channel_id.is_some()
            && discord_intermediate.as_ref().is_some()
        {
            let inter = discord_intermediate.as_ref().unwrap();
            let same = is_final_same_as_intermediate(inter, &response_content);
            if same {
                format!(
                    "--- Intermediate answer:\n\n{}\n\n---\n\nFinal answer is the same as intermediate.",
                    inter.trim()
                )
            } else {
                format!(
                    "--- Intermediate answer:\n\n{}\n\n---\n\n--- Final answer:\n\n{}\n\n---",
                    inter.trim(),
                    response_content.trim()
                )
            }
        } else {
            response_content
        };

        Ok(OllamaReply {
            text: final_text,
            attachment_paths,
        })
    })
}


