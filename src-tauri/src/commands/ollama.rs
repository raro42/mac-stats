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
use crate::commands::ollama_models::list_ollama_models;
use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply,
    grounded_redmine_time_entries_failure_reply,
    is_redmine_review_or_summarize_only, is_redmine_time_entries_request,
    question_explicitly_requests_json,
};
use crate::commands::reply_helpers::{
    final_reply_from_tool_results,
    is_bare_done_plan, is_final_same_as_intermediate,
};
use crate::commands::pre_routing::compute_pre_routed_recommendation;
use crate::commands::ollama_memory::{
    load_memory_block_for_request, load_soul_content, search_memory_for_request,
};
use crate::commands::perplexity_helpers::is_news_query;
use crate::commands::tool_parsing::{
    normalize_browser_tool_arg, normalize_inline_tool_sequences, parse_all_tools_from_response,
    parse_tool_from_response, truncate_search_query_arg, MAX_BROWSER_TOOLS_PER_RUN,
};
use crate::commands::verification::{
    detect_new_topic, extract_success_criteria, original_request_for_retry,
    sanitize_success_criteria, summarize_last_turns, user_explicitly_asked_for_screenshot,
    verify_completion, RequestRunContext,
};
pub use crate::commands::verification::OllamaReply;
pub(crate) use crate::commands::agent_session::run_agent_ollama_session;

use crate::commands::agent_descriptions::{
    build_agent_descriptions, DISCORD_GROUP_CHANNEL_GUIDANCE, DISCORD_PLATFORM_FORMATTING,
};
use crate::commands::browser_helpers::wants_visible_browser;
use crate::commands::prompt_assembly::build_execution_system_content;
use crate::commands::session_history::{prepare_conversation_history, CONVERSATION_HISTORY_CAP};
use crate::commands::verification::build_verification_retry_hint;



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

        send_status("Compacting session memory…");
        let conversation_history = prepare_conversation_history(
            raw_history,
            question,
            is_new_topic,
            discord_reply_channel_id,
            &request_id,
        )
        .await;
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
            let prompt = build_execution_system_content(
                &router_soul, &memory_block, &discord_user_context,
                skill_content.as_deref(), &execution_prompt, &metrics_for_system,
                discord_screenshot_reminder, redmine_howto_reminder,
                news_format_reminder, &discord_platform_formatting,
                &model_identity, None,
            );
            let mut msgs: Vec<crate::ollama::ChatMessage> = vec![crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: prompt.content,
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
            let prompt = build_execution_system_content(
                &router_soul, &memory_block, &discord_user_context,
                skill_content.as_deref(), &execution_prompt, &metrics_for_system,
                discord_screenshot_reminder, redmine_howto_reminder,
                news_format_reminder, &discord_platform_formatting,
                &model_identity, Some(&recommendation),
            );
            let mut msgs: Vec<crate::ollama::ChatMessage> = vec![crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: prompt.content,
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
                        crate::commands::network_tool_dispatch::handle_fetch_url_discord_redirect(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "FETCH_URL" => {
                        let estimated_used =
                            messages.iter().map(|m| m.content.len()).sum::<usize>()
                                + agent_descriptions.len();
                        crate::commands::network_tool_dispatch::handle_fetch_url(
                            &arg,
                            estimated_used,
                            model_info.context_size_tokens,
                            model_override.clone(),
                            options_override.clone(),
                            status_tx.as_ref(),
                        ).await?
                    }
                    "BROWSER_SCREENSHOT" => {
                        let result = crate::commands::browser_tool_dispatch::handle_browser_screenshot(
                            &arg, &request_id, status_tx.as_ref(),
                        ).await;
                        if let Some(path) = result.attachment_path {
                            attachment_paths.push(path);
                        }
                        result.message
                    }
                    "BROWSER_NAVIGATE" => {
                        crate::commands::browser_tool_dispatch::handle_browser_navigate(
                            &arg, &request_id, status_tx.as_ref(),
                        ).await
                    }
                    "BROWSER_GO_BACK" => {
                        crate::commands::browser_tool_dispatch::handle_browser_go_back(
                            &request_id, status_tx.as_ref(),
                        ).await
                    }
                    "BROWSER_CLICK" => {
                        crate::commands::browser_tool_dispatch::handle_browser_click(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "BROWSER_INPUT" => {
                        crate::commands::browser_tool_dispatch::handle_browser_input(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "BROWSER_SCROLL" => {
                        crate::commands::browser_tool_dispatch::handle_browser_scroll(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "BROWSER_EXTRACT" => {
                        send_status("📄 Extracting page text…");
                        crate::commands::browser_tool_dispatch::handle_browser_extract().await
                    }
                    "BROWSER_SEARCH_PAGE" => {
                        crate::commands::browser_tool_dispatch::handle_browser_search_page(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "BRAVE_SEARCH" => {
                        crate::commands::network_tool_dispatch::handle_brave_search(
                            &arg, status_tx.as_ref(),
                        ).await
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
                    "RUN_JS" => {
                        crate::commands::delegation_tool_dispatch::handle_run_js(
                            &arg, status_tx.as_ref(),
                        )
                    }
                    "SKILL" => {
                        crate::commands::delegation_tool_dispatch::handle_skill(
                            &arg, question, model_override.clone(),
                            options_override.clone(), status_tx.as_ref(),
                        ).await
                    }
                    "AGENT" => {
                        let last_user_content = messages
                            .last()
                            .filter(|m| m.role == "user")
                            .map(|m| m.content.as_str());
                        let result = crate::commands::delegation_tool_dispatch::handle_agent(
                            &arg, question, discord_reply_channel_id,
                            status_tx.as_ref(), load_global_memory,
                            last_user_content,
                        ).await;
                        if let Some(entry) = result.agent_conversation_entry {
                            agent_conversation.push(entry);
                        }
                        result.message
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
                        let result = crate::commands::delegation_tool_dispatch::handle_run_cmd(
                            &arg, last_run_cmd_arg.as_deref(), multi_tool_turn,
                            model_override.clone(), options_override.clone(),
                            status_tx.as_ref(),
                        ).await;
                        if let Some(raw) = result.raw_output {
                            last_run_cmd_raw_output = Some(raw);
                        }
                        result.message
                    }
                    "PYTHON_SCRIPT" => {
                        crate::commands::delegation_tool_dispatch::handle_python_script(
                            &arg, &response_content, status_tx.as_ref(),
                        ).await
                    }
                    "DISCORD_API" => {
                        let result = crate::commands::network_tool_dispatch::handle_discord_api(
                            &arg, last_successful_discord_call.as_ref(), status_tx.as_ref(),
                        ).await;
                        if let Some(call) = result.successful_call {
                            last_successful_discord_call = Some(call);
                        }
                        result.message
                    }
                    "OLLAMA_API" => {
                        crate::commands::misc_tool_dispatch::handle_ollama_api(
                            &arg, status_tx.as_ref(),
                        ).await
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
                        crate::commands::misc_tool_dispatch::handle_mcp(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "CURSOR_AGENT" => {
                        crate::commands::misc_tool_dispatch::handle_cursor_agent(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "REDMINE_API" => {
                        crate::commands::network_tool_dispatch::handle_redmine_api(
                            &arg, question, &request_id, status_tx.as_ref(),
                        ).await
                    }
                    "MASTODON_POST" => {
                        crate::commands::misc_tool_dispatch::handle_mastodon_post(
                            &arg, status_tx.as_ref(),
                        ).await
                    }
                    "MEMORY_APPEND" => {
                        crate::commands::misc_tool_dispatch::handle_memory_append(
                            &arg, discord_reply_channel_id,
                        )
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
                    let retry_base_with_hint = build_verification_retry_hint(
                        &request_for_verification,
                        reason.as_deref(),
                        &retry_base,
                        &response_content,
                        &attachment_paths,
                        last_browser_extract.as_deref(),
                    );
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


