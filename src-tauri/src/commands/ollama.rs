//! Ollama Tauri commands — orchestrator (`answer_with_ollama_and_fetch`) and re-exports.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;

pub(crate) use crate::commands::agent_session::run_agent_ollama_session;
pub use crate::commands::ollama_chat::send_ollama_chat_messages;
pub use crate::commands::ollama_config::{
    ensure_ollama_agent_ready_at_startup, get_default_ollama_model_name,
};
use crate::commands::ollama_config::{
    get_ollama_client, read_ollama_api_key_from_env_or_config,
    read_ollama_fast_model_from_env_or_config,
};
use crate::commands::ollama_memory::{
    format_router_soul_block, load_memory_block_for_request, load_soul_content,
    search_memory_for_request,
};
use crate::commands::ollama_models::list_ollama_models;
use crate::commands::perplexity_helpers::is_news_query;
use crate::commands::pre_routing::compute_pre_routed_recommendation;
use crate::commands::redmine_helpers::{
    is_redmine_review_or_summarize_only, is_redmine_time_entries_request,
};
use crate::commands::reply_helpers::{is_bare_done_plan, is_final_same_as_intermediate};
use crate::commands::tool_parsing::{
    normalize_inline_tool_sequences, parse_all_tools_from_response, parse_tool_from_response,
    MAX_BROWSER_TOOLS_PER_RUN,
};
pub use crate::commands::verification::OllamaReply;
use crate::commands::verification::{
    detect_new_topic, extract_success_criteria, original_request_for_retry,
    sanitize_success_criteria, summarize_last_turns, user_explicitly_asked_for_screenshot,
    verify_completion, RequestRunContext,
};

use crate::commands::agent_descriptions::{
    build_agent_descriptions, DISCORD_GROUP_CHANNEL_GUIDANCE, DISCORD_PLATFORM_FORMATTING,
};
use crate::commands::browser_helpers::wants_visible_browser;
use crate::commands::prompt_assembly::build_execution_system_content;
use crate::commands::session_history::{
    build_execution_message_stack, cap_tail_chronological, prepare_conversation_history,
    CONVERSATION_HISTORY_CAP,
};
use crate::commands::verification::build_verification_retry_hint;
use crate::{mac_stats_debug, mac_stats_info};

/// All parameters for an `answer_with_ollama_and_fetch` invocation.
///
/// Fields default to `None` / `false` / `0` via `Default`, so callers only
/// need to set the fields they care about:
///
/// ```ignore
/// let reply = answer_with_ollama_and_fetch(OllamaRequest {
///     question: "What time is it?".into(),
///     from_remote: true,
///     ..Default::default()
/// }).await?;
/// ```
#[derive(Default)]
pub struct OllamaRequest {
    pub question: String,
    pub status_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    pub discord_reply_channel_id: Option<u64>,
    pub discord_user_id: Option<u64>,
    pub discord_user_name: Option<String>,
    pub model_override: Option<String>,
    pub options_override: Option<crate::ollama::ChatOptions>,
    pub skill_content: Option<String>,
    pub agent_override: Option<crate::agents::Agent>,
    pub allow_schedule: bool,
    pub conversation_history: Option<Vec<crate::ollama::ChatMessage>>,
    pub escalation: bool,
    pub retry_on_verification_no: bool,
    /// When true (Discord, scheduler, task runner), browser defaults to headless.
    pub from_remote: bool,
    /// Base64-encoded images from Discord message attachments (vision models).
    pub attachment_images_base64: Option<Vec<String>>,
    /// When set (retry path from Discord), format reply as "Intermediate + Final".
    pub discord_intermediate: Option<String>,
    /// When true, keep conversation history and skip NEW_TOPIC check.
    pub is_verification_retry: bool,
    /// Original end-user request. When set, verification uses this instead of inferring.
    pub original_user_request: Option<String>,
    /// Request-local criteria extracted on the first pass; reused on retry.
    pub success_criteria_override: Option<Vec<String>>,
    /// Some(true) = Discord DM (load global memory), Some(false) = Discord guild (skip), None = not Discord (load).
    pub discord_is_dm: Option<bool>,
    /// Shared request id so retries correlate in logs.
    pub request_id_override: Option<String>,
    /// 0 = first run; 1 = verification retry.
    pub retry_count: u32,
    /// When set (Discord agent path), tool loop updates this message with throttled edits until flush.
    pub discord_draft: Option<crate::commands::discord_draft_stream::DiscordDraftHandle>,
}

/// Main orchestrator: plan → execute tools → verify → optionally retry.
/// Returns a boxed future to allow one level of async recursion (retry path).
pub fn answer_with_ollama_and_fetch(
    req: OllamaRequest,
) -> Pin<Box<dyn Future<Output = Result<OllamaReply, String>> + Send>> {
    let OllamaRequest {
        question,
        status_tx,
        discord_reply_channel_id,
        discord_user_id,
        discord_user_name,
        model_override,
        options_override,
        skill_content,
        agent_override,
        allow_schedule,
        mut conversation_history,
        escalation,
        retry_on_verification_no,
        from_remote,
        attachment_images_base64,
        discord_intermediate,
        is_verification_retry,
        original_user_request,
        success_criteria_override,
        discord_is_dm,
        request_id_override,
        retry_count,
        discord_draft,
    } = req;
    let load_global_memory = discord_is_dm.is_none_or(|dm| dm);
    if discord_is_dm == Some(false) {
        mac_stats_info!(
            "ollama/chat",
            "Agent router: Discord guild channel — not loading global memory (main-session only)"
        );
    }
    Box::pin(async move {
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
            mac_stats_info!("ollama/chat", 
                "Agent router [{}]: session start (verification retry {}, request-local criteria only) — {}",
                run_ctx.request_id,
                run_ctx.retry_count,
                crate::config::Config::version_display()
            );
        } else {
            mac_stats_info!(
                "ollama/chat",
                "Agent router [{}]: session start — {}",
                run_ctx.request_id,
                crate::config::Config::version_display()
            );
        }
        mac_stats_debug!("ollama/chat",
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
                && conversation_history.as_ref().is_some_and(|h| !h.is_empty())
            {
                mac_stats_info!("ollama/chat", 
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
            mac_stats_info!(
                "ollama/chat",
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
            mac_stats_info!(
                "ollama/chat",
                "Agent router [{}]: starting (question: {}... [{} chars])",
                request_id,
                q_preview,
                question.len()
            );
        } else {
            mac_stats_info!(
                "ollama/chat",
                "Agent router [{}]: starting (question: {})",
                request_id,
                q_preview
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
            mac_stats_info!(
                "ollama/chat",
                "Agent router [{}]: reusing {} request-local success criteria",
                request_id,
                criteria.len()
            );
            Some(sanitize_success_criteria(
                &request_for_verification,
                criteria,
            ))
        } else if is_redmine_review_or_summarize_only(&request_for_verification) {
            mac_stats_info!("ollama/chat", 
                "Agent router: review/summarize-only Redmine request — overriding success criteria to summary-only"
            );
            Some(vec![
                "Summary of ticket content provided to the user.".to_string()
            ])
        } else if is_redmine_time_entries_request(&request_for_verification) {
            mac_stats_info!("ollama/chat", 
                "Agent router: Redmine time-entries request — overriding success criteria to Redmine data-only"
            );
            Some(vec![
                "Actual Redmine time entries for the requested period were fetched.".to_string(),
                "A clear summary of hours or worked tickets was provided from that Redmine data."
                    .to_string(),
            ])
        } else if is_news_query(&request_for_verification) {
            mac_stats_info!(
                "ollama/chat",
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
                mac_stats_info!(
                    "ollama/chat",
                    "Agent router [{}]: extracted {} success criteria",
                    request_id,
                    c.len()
                );
                truncate_status(&c.join("; "), 200)
            }
            _ => {
                mac_stats_debug!("ollama/chat", 
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
            mac_stats_debug!("ollama/chat",
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
                        mac_stats_info!("ollama/chat", 
                            "Agent router [{}]: NEW_TOPIC — using no prior context; any verification retry will use only request-local criteria",
                            request_id
                        );
                        is_new_topic = true;
                    }
                    Ok(false) => {
                        mac_stats_info!(
                            "ollama/chat",
                            "Agent router [{}]: SAME_TOPIC — keeping prior context",
                            request_id
                        );
                    }
                    Err(e) => {
                        mac_stats_debug!("ollama/chat",
                            request_id = %request_id,
                            "new-topic check failed (keeping history): {}",
                            e
                        );
                    }
                }
            }
        }

        let raw_history: Vec<crate::ollama::ChatMessage> = cap_tail_chronological(
            conversation_history.unwrap_or_default(),
            CONVERSATION_HISTORY_CAP,
        );
        if raw_history.is_empty() {
            mac_stats_info!(
                "ollama/chat",
                "Agent router [{}]: no prior session",
                request_id
            );
        } else {
            mac_stats_info!(
                "ollama/chat",
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
            mac_stats_info!(
                "ollama/chat",
                "Agent router: using {} prior messages as context",
                conversation_history.len()
            );
        }

        let from_discord = discord_reply_channel_id.is_some();
        let discord_platform_formatting: String = if from_discord {
            mac_stats_info!("ollama/chat", "Agent router: Discord reply — injecting platform formatting (no tables, wrap links in <>)");
            if discord_is_dm == Some(false) {
                format!(
                    "{}{}",
                    DISCORD_PLATFORM_FORMATTING, DISCORD_GROUP_CHANNEL_GUIDANCE
                )
            } else {
                DISCORD_PLATFORM_FORMATTING.to_string()
            }
        } else {
            String::new()
        };
        let agent_descriptions = build_agent_descriptions(from_discord, Some(question)).await;
        mac_stats_info!(
            "ollama/chat",
            "Agent router: agent list built ({} chars)",
            agent_descriptions.len()
        );

        // When no agent/skill override, prepend soul (~/.mac-stats/agents/soul.md) so Ollama gets personality/vibe in context.
        let router_soul = skill_content.as_ref().map_or_else(
            || format_router_soul_block(&load_soul_content(), &crate::config::Config::version()),
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
            mac_stats_info!(
                "ollama/chat",
                "Agent router: skipping LLM planning (pre-routed)"
            );
            pre_routed
        } else {
            mac_stats_info!(
                "ollama/chat",
                "Agent router [{}]: planning step — asking Ollama for RECOMMEND",
                request_id
            );
            let planning_prompt = crate::config::Config::load_planning_prompt();
            let planning_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\n{}{}",
                    discord_user_context,
                    skill,
                    planning_prompt,
                    agent_descriptions,
                    discord_platform_formatting
                ),
                None => format!(
                    "{}{}{}\n\n{}{}",
                    router_soul,
                    discord_user_context,
                    planning_prompt,
                    agent_descriptions,
                    discord_platform_formatting
                ),
            };
            let mut planning_messages: Vec<crate::ollama::ChatMessage> =
                vec![crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: planning_system_content,
                    images: None,
                }];
            let planning_cap = crate::config::Config::planning_history_cap();
            let planning_history: &[crate::ollama::ChatMessage] =
                if planning_cap > 0 && conversation_history.len() > planning_cap {
                    mac_stats_info!(
                        "ollama/chat",
                        "Agent router [{}]: capping planning history from {} to {} messages",
                        request_id,
                        conversation_history.len(),
                        planning_cap
                    );
                    &conversation_history[conversation_history.len() - planning_cap..]
                } else if planning_cap == 0 && !conversation_history.is_empty() {
                    mac_stats_info!(
                    "ollama/chat",
                    "Agent router [{}]: planning history disabled (cap=0), skipping {} messages",
                    request_id,
                    conversation_history.len()
                );
                    &[]
                } else {
                    &conversation_history
                };
            for msg in planning_history {
                planning_messages.push(msg.clone());
            }
            let model_hint = model_override.as_ref().map(|m| format!("\n\nFor this request the user selected Ollama model: {}. The app will use that model for the reply; recommend answering the question (or using an agent) with that in mind.", m)).unwrap_or_default();
            let today_utc = chrono::Utc::now().format("%Y-%m-%d");
            mac_stats_info!(
                "ollama/chat",
                "Agent router [{}]: planning RECOMMEND with current date (UTC) {}",
                request_id,
                today_utc
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
                mac_stats_info!("ollama/chat", 
                    "Agent router [{}]: planner returned bare DONE plan but question indicates attachment failure; using synthetic summary instruction",
                    request_id
                );
                recommendation = "Reply with a brief summary of what was done and that the app could not attach the file(s) to Discord, then end with **DONE: no**. Do not run further tools.".to_string();
            }
        }
        mac_stats_info!(
            "ollama/chat",
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
            mac_stats_info!(
                "ollama/chat",
                "Agent router: clearing conversation history for fresh-session tool"
            );
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
        let (mut messages, response_content) = if let Some((ref tool, ref arg)) = direct_tool {
            let all_tools = parse_all_tools_from_response(&normalized_recommendation);
            if all_tools.len() > 1 {
                mac_stats_info!("ollama/chat", 
                    "Agent router: plan contains {} direct tools, will run in sequence: {} — skipping execution Ollama call",
                    all_tools.len(),
                    all_tools.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>().join(", ")
                );
            } else {
                mac_stats_info!("ollama/chat", 
                    "Agent router: plan contains direct tool call {}:{} — skipping execution Ollama call",
                    tool,
                    crate::logging::ellipse(arg, 60)
                );
            }
            let memory_block =
                load_memory_block_for_request(discord_reply_channel_id, load_global_memory);
            let prompt = build_execution_system_content(
                &router_soul,
                &memory_block,
                &discord_user_context,
                skill_content.as_deref(),
                &execution_prompt,
                &metrics_for_system,
                discord_screenshot_reminder,
                redmine_howto_reminder,
                news_format_reminder,
                &discord_platform_formatting,
                &model_identity,
                None,
            );
            let msgs = build_execution_message_stack(
                crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: prompt.content,
                    images: None,
                },
                &conversation_history,
                crate::ollama::ChatMessage {
                    role: "user".to_string(),
                    content: question_for_plan_and_exec.clone(),
                    images: attachment_images_base64.clone(),
                },
            );
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
            mac_stats_info!("ollama/chat", 
                "Agent router: execution step — sending plan + question, starting tool loop (max {} tools)",
                max_tool_iterations
            );
            let memory_block =
                load_memory_block_for_request(discord_reply_channel_id, load_global_memory);
            let prompt = build_execution_system_content(
                &router_soul,
                &memory_block,
                &discord_user_context,
                skill_content.as_deref(),
                &execution_prompt,
                &metrics_for_system,
                discord_screenshot_reminder,
                redmine_howto_reminder,
                news_format_reminder,
                &discord_platform_formatting,
                &model_identity,
                Some(&recommendation),
            );
            let mut msgs = build_execution_message_stack(
                crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: prompt.content,
                    images: None,
                },
                &conversation_history,
                crate::ollama::ChatMessage {
                    role: "user".to_string(),
                    content: question_for_plan_and_exec.clone(),
                    images: attachment_images_base64.clone(),
                },
            );
            let response = match send_ollama_chat_messages(
                msgs.clone(),
                model_override.clone(),
                options_override.clone(),
            )
            .await
            {
                Ok(r) => r,
                Err(e)
                    if crate::commands::content_reduction::is_context_overflow_error(&e)
                        && crate::config::Config::context_overflow_truncate_enabled() =>
                {
                    let max_chars = crate::config::Config::context_overflow_max_result_chars();
                    let n = crate::commands::content_reduction::truncate_oversized_tool_results(
                        &mut msgs, max_chars,
                    );
                    if n > 0 {
                        mac_stats_info!("ollama/chat", 
                            "Agent router: context overflow on first execution call — truncated {} tool result(s) to {} chars, retrying",
                            n, max_chars
                        );
                        send_ollama_chat_messages(
                            msgs.clone(),
                            model_override.clone(),
                            options_override.clone(),
                        )
                        .await?
                    } else {
                        return Err(
                            crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
                                .unwrap_or(e),
                        );
                    }
                }
                Err(e) => {
                    return Err(
                        crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
                            .unwrap_or(e),
                    );
                }
            };
            let content = response.message.content.clone();
            let n = content.chars().count();
            mac_stats_info!(
                "ollama/chat",
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
                    mac_stats_info!("ollama/chat", 
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

        // Browser mode: "headless" in question -> no visible window. From Discord/scheduler/task (from_remote) -> headless unless user explicitly asks to see the browser (so retries stay headless).
        let prefer_headless = if from_remote {
            !wants_visible_browser(question)
        } else {
            question.to_lowercase().contains("headless")
        };
        crate::browser_agent::set_prefer_headless_for_run(prefer_headless);

        let tool_loop_params = crate::commands::tool_loop::ToolLoopParams {
            question: question.to_string(),
            request_id: request_id.clone(),
            max_tool_iterations,
            model_override: model_override.clone(),
            options_override: options_override.clone(),
            status_tx: status_tx.clone(),
            discord_draft: discord_draft.clone(),
            discord_reply_channel_id,
            allow_schedule,
            load_global_memory,
            agent_descriptions_len: agent_descriptions.len(),
            model_context_size_tokens: model_info.context_size_tokens,
            budget_warning_ratio: crate::config::Config::tool_budget_warning_ratio(),
        };
        let tool_loop_result = crate::commands::tool_loop::run_tool_loop(
            &tool_loop_params,
            &mut messages,
            response_content,
        )
        .await?;
        let mut response_content = tool_loop_result.response_content;
        let tool_loop_state = tool_loop_result.state;
        let tool_count = tool_loop_state.tool_count;
        let attachment_paths = tool_loop_state.attachment_paths;
        let screenshot_requested_by_tool_run = tool_loop_state.screenshot_requested_by_tool_run;
        let agent_conversation = tool_loop_state.agent_conversation;
        let current_task_path = tool_loop_state.current_task_path;
        let last_browser_extract = tool_loop_state.last_browser_extract;
        let browser_tool_cap_reached = tool_loop_state.browser_tool_cap_reached;
        let last_news_search_was_hub_only = tool_loop_state.last_news_search_was_hub_only;

        if tool_loop_state.exited_via_done {
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
        mac_stats_info!(
            "ollama/chat",
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
                    mac_stats_info!(
                        "ollama/chat",
                        "Agent router: could not append conversation to task file: {}",
                        e
                    );
                } else {
                    mac_stats_info!(
                        "ollama/chat",
                        "Agent router: appended conversation to task {}",
                        crate::task::task_file_name(path)
                    );
                }
            } else {
                mac_stats_info!(
                    "ollama/chat",
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
            mac_stats_info!("ollama/chat", 
                "Agent router: heuristic guard — screenshot requested but no attachment, appended note"
            );
        }

        // User-facing note when browser action limit was reached (032_browser_loop_and_status_fix_plan).
        if browser_tool_cap_reached {
            response_content.push_str(&format!(
                "\n\nNote: Browser action limit ({} per run) was reached; some actions were skipped.",
                MAX_BROWSER_TOOLS_PER_RUN
            ));
            mac_stats_info!(
                "ollama/chat",
                "Agent router: browser tool cap was reached, appended user-facing note"
            );
        }

        // Completion verification: one short Ollama call; if not satisfied, retry once (A2) or append disclaimer
        let criteria_count = success_criteria.as_ref().map(|c| c.len()).unwrap_or(0);
        mac_stats_info!(
            "ollama/chat",
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
                    mac_stats_info!("ollama/chat", 
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
                    return answer_with_ollama_and_fetch(OllamaRequest {
                        question: retry_question,
                        status_tx,
                        discord_reply_channel_id,
                        discord_user_id,
                        discord_user_name,
                        model_override,
                        options_override,
                        skill_content,
                        agent_override,
                        allow_schedule,
                        conversation_history: Some(updated_history),
                        escalation,
                        retry_on_verification_no: false,
                        from_remote,
                        attachment_images_base64,
                        discord_intermediate: pass_intermediate,
                        is_verification_retry: true,
                        original_user_request: Some(request_for_verification.clone()),
                        success_criteria_override: success_criteria.clone(),
                        discord_is_dm,
                        request_id_override: Some(request_id.clone()),
                        retry_count: 1,
                        discord_draft,
                    })
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
                    mac_stats_info!("ollama/chat", 
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
                            mac_stats_info!(
                                "ollama/chat",
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
                            mac_stats_info!(
                                "ollama/chat",
                                "Agent router: cursor-agent handoff failed: {}",
                                e
                            );
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
                    mac_stats_info!(
                        "ollama/chat",
                        "Agent router: memory search found {} chars (not appended to reply)",
                        from_memory.len()
                    );
                }
                mac_stats_info!("ollama/chat", 
                    "Agent router: verification said not satisfied, appended disclaimer (reason: {}...)",
                    reason_preview
                );
            }
            Ok((true, _)) => {
                mac_stats_info!(
                    "ollama/chat",
                    "Agent router: verification passed (satisfied)"
                );
            }
            Err(e) => {
                mac_stats_debug!(
                    "ollama/chat",
                    "Agent router: verification failed (ignored): {}",
                    e
                );
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
