// included by gateway.rs — try_operator_instant_reply router

pub fn try_operator_instant_reply(content: &str) -> Option<String> {
    if looks_like_ops_help_request(content) {
        return Some(format_ops_help_gateway());
    }
    if looks_like_status_request(content) {
        return Some(format_status_gateway());
    }
    if looks_like_discord_gateway_request(content) {
        return Some(format_discord_gateway_chip());
    }
    if looks_like_ollama_ready_request(content) {
        return Some(format_ollama_ready_chip());
    }
    if looks_like_redmine_ready_request(content) {
        return Some(format_redmine_ready_chip());
    }
    if looks_like_brave_ready_request(content) {
        return Some(format_brave_ready_chip());
    }
    if looks_like_perplexity_ready_request(content) {
        return Some(format_perplexity_ready_chip());
    }
    if looks_like_mastodon_ready_request(content) {
        return Some(format_mastodon_ready_chip());
    }
    if looks_like_mcp_ready_request(content) {
        return Some(format_mcp_ready_chip());
    }
    if looks_like_cursor_agent_ready_request(content) {
        return Some(format_cursor_agent_ready_chip());
    }
    if looks_like_browser_ready_request(content) {
        return Some(format_browser_ready_chip());
    }
    if looks_like_judge_ready_request(content) {
        return Some(format_judge_ready_chip());
    }
    if looks_like_ai_agent_ready_request(content) {
        return Some(format_ai_agent_ready_chip());
    }
    if looks_like_compact_ready_request(content) {
        return Some(format_compact_ready_chip());
    }
    // Installed RAM before resource-stat size lanes (execution_prompt / planning / soul / …
    // nouns include bare "size"/"big" and would steal `ram size` / `/ram-size` / `how big is ram`).
    if looks_like_ram_capacity_request(content) {
        return Some(format_ram_capacity_gateway());
    }
    // SSD capacity before resource-stat size lanes (disk cleanup / notes / … would steal
    // `disk size` / `/ssd-size` / `how big is the ssd`). Bare `/ssd` / `view disk` stay later.
    if looks_like_disk_capacity_request(content) {
        return Some(format_disk_capacity_gateway());
    }
    // downloads-organizer-rules.md size before age/path (stat only; no dump).
    if looks_like_downloads_organizer_rules_size_request(content) {
        return Some(format_downloads_organizer_rules_size_gateway());
    }
    // downloads-organizer-rules.md age before path (mtime only; no dump).
    if looks_like_downloads_organizer_rules_age_request(content) {
        return Some(format_downloads_organizer_rules_age_gateway());
    }
    // downloads-organizer-state.json size before age/path (stat only; no dump).
    if looks_like_downloads_organizer_state_size_request(content) {
        return Some(format_downloads_organizer_state_size_gateway());
    }
    // downloads-organizer-state.json age before path (mtime only; no dump).
    if looks_like_downloads_organizer_state_age_request(content) {
        return Some(format_downloads_organizer_state_age_gateway());
    }
    // downloads-organizer-state.json before rules / /downloads Ready (path-only asks).
    if looks_like_downloads_organizer_state_path_request(content) {
        return Some(format_downloads_organizer_state_path_gateway());
    }
    // downloads-organizer-rules.md before /downloads Ready (path-only asks).
    if looks_like_downloads_organizer_rules_path_request(content) {
        return Some(format_downloads_organizer_rules_path_gateway());
    }
    // agent.json size before age/path (stat only; no dump).
    if looks_like_agent_json_size_request(content) {
        return Some(format_agent_json_size_gateway());
    }
    // agent.json age before path (mtime only; no dump).
    if looks_like_agent_json_age_request(content) {
        return Some(format_agent_json_age_gateway());
    }
    // agent.json before testing / skill / mood / soul / agents-dir path / /agents catalog (path-only asks).
    if looks_like_agent_json_path_request(content) {
        return Some(format_agent_json_path_gateway());
    }
    // execution_prompt.md size before age/path (stat only; no dump).
    if looks_like_execution_prompt_size_request(content) {
        return Some(format_execution_prompt_size_gateway());
    }
    // execution_prompt.md age before path (mtime only; no dump).
    if looks_like_execution_prompt_age_request(content) {
        return Some(format_execution_prompt_age_gateway());
    }
    // execution_prompt.md before planning / prompts-dir path / testing / skill (path-only asks).
    if looks_like_execution_prompt_path_request(content) {
        return Some(format_execution_prompt_path_gateway());
    }
    // planning_prompt.md size before age/path (stat only; no dump).
    if looks_like_planning_prompt_size_request(content) {
        return Some(format_planning_prompt_size_gateway());
    }
    // planning_prompt.md age before path (mtime only; no dump).
    if looks_like_planning_prompt_age_request(content) {
        return Some(format_planning_prompt_age_gateway());
    }
    // planning_prompt.md before prompts-dir path / testing / skill (path-only asks).
    if looks_like_planning_prompt_path_request(content) {
        return Some(format_planning_prompt_path_gateway());
    }
    // testing.md size before age/path (stat only; no dump).
    if looks_like_testing_md_size_request(content) {
        return Some(format_testing_md_size_gateway());
    }
    // testing.md age before path (mtime only; no dump).
    if looks_like_testing_md_age_request(content) {
        return Some(format_testing_md_age_gateway());
    }
    // testing.md before skill / mood / soul / agents-dir path / /agents catalog (path-only asks).
    if looks_like_testing_md_path_request(content) {
        return Some(format_testing_md_path_gateway());
    }
    // skill.md size before age/path (stat only; no dump).
    if looks_like_skill_md_size_request(content) {
        return Some(format_skill_md_size_gateway());
    }
    // skill.md age before path (mtime only; no dump).
    if looks_like_skill_md_age_request(content) {
        return Some(format_skill_md_age_gateway());
    }
    // skill.md before mood / soul / skills-dir / agents-dir path / /agents catalog (path-only asks).
    if looks_like_skill_md_path_request(content) {
        return Some(format_skill_md_path_gateway());
    }
    // mood.md size before age/path (stat only; no dump).
    if looks_like_mood_size_request(content) {
        return Some(format_mood_size_gateway());
    }
    // mood.md age before path (mtime only; no dump).
    if looks_like_mood_age_request(content) {
        return Some(format_mood_age_gateway());
    }
    // mood.md before soul / agents-dir path / /agents catalog (path-only asks).
    if looks_like_mood_path_request(content) {
        return Some(format_mood_path_gateway());
    }
    // soul.md size before age/path (stat only; no dump).
    if looks_like_soul_size_request(content) {
        return Some(format_soul_size_gateway());
    }
    // soul.md age before path (mtime only; no dump).
    if looks_like_soul_age_request(content) {
        return Some(format_soul_age_gateway());
    }
    // soul.md before agents-dir path / /agents catalog (path-only asks).
    if looks_like_soul_path_request(content) {
        return Some(format_soul_path_gateway());
    }
    // memory.md size before age/path (stat only; no dump).
    if looks_like_memory_md_size_request(content) {
        return Some(format_memory_md_size_gateway());
    }
    // memory.md age before path (mtime only; no dump).
    if looks_like_memory_md_age_request(content) {
        return Some(format_memory_md_age_gateway());
    }
    // memory.md curated file before notes-folder memory path (path-only asks).
    if looks_like_memory_md_path_request(content) {
        return Some(format_memory_md_path_gateway());
    }
    // Discord channel memory size before age/path (stat only; no list/dump).
    if looks_like_discord_memory_size_request(content) {
        return Some(format_discord_memory_size_gateway());
    }
    // Discord channel memory age before path (newest mtime; no list/dump).
    if looks_like_discord_memory_age_request(content) {
        return Some(format_discord_memory_age_gateway());
    }
    // Discord channel memory path before notes-folder / Knowledge list (path-only asks).
    if looks_like_discord_memory_path_request(content) {
        return Some(format_discord_memory_path_gateway());
    }
    // Session-memory-*.md size before age/path (stat only; no list/dump).
    if looks_like_session_memory_size_request(content) {
        return Some(format_session_memory_size_gateway());
    }
    // Session-memory-*.md age before path (newest mtime; no list/dump).
    if looks_like_session_memory_age_request(content) {
        return Some(format_session_memory_age_gateway());
    }
    // Session-memory-*.md path before notes-folder / session-dir path (path-only asks).
    if looks_like_session_memory_path_request(content) {
        return Some(format_session_memory_path_gateway());
    }
    // LaunchAgent plist age before size/path (mtime only; no dump/load).
    if looks_like_launchagent_age_request(content) {
        return Some(format_launchagent_age_gateway());
    }
    // LaunchAgent plist size before path (stat only; no dump/load).
    if looks_like_launchagent_size_request(content) {
        return Some(format_launchagent_size_gateway());
    }
    // LaunchAgent plist paths before overnight-improvements content asks (path-only).
    if looks_like_launchagent_path_request(content) {
        return Some(format_launchagent_path_gateway());
    }
    // results.tsv age since last keep/discard before first-tonight / last-row / recent / streaks.
    if looks_like_results_tsv_since_request(content) {
        return Some(format_results_tsv_since_gateway(content));
    }
    // results.tsv first keep/discard tonight (one row) before last-row / recent / counts.
    if looks_like_results_tsv_first_request(content) {
        return Some(format_results_tsv_first_gateway(content));
    }
    // results.tsv keep/discard gap histogram before Gini / modal / Shannon entropy / excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_histogram_request(content) {
        return Some(format_results_tsv_histogram_gateway(content));
    }
    // results.tsv keep/discard Gini gap before modal / Shannon entropy / excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_gini_request(content) {
        return Some(format_results_tsv_gini_gateway(content));
    }
    // results.tsv keep/discard modal gap before Shannon entropy / excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_mode_request(content) {
        return Some(format_results_tsv_mode_gateway(content));
    }
    // results.tsv keep/discard Shannon entropy before excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_entropy_request(content) {
        return Some(format_results_tsv_entropy_gateway(content));
    }
    // results.tsv keep/discard excess kurtosis before skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_kurtosis_request(content) {
        return Some(format_results_tsv_kurtosis_gateway(content));
    }
    // results.tsv keep/discard skewness before CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_skew_request(content) {
        return Some(format_results_tsv_skew_gateway(content));
    }
    // results.tsv keep/discard CV (std/mean %) before MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_cv_request(content) {
        return Some(format_results_tsv_cv_gateway(content));
    }
    // results.tsv keep/discard MAD gap before sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_mad_request(content) {
        return Some(format_results_tsv_mad_gateway(content));
    }
    // results.tsv keep/discard sample-std gap before IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_std_request(content) {
        return Some(format_results_tsv_std_gateway(content));
    }
    // results.tsv keep/discard IQR gap before p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_iqr_request(content) {
        return Some(format_results_tsv_iqr_gateway(content));
    }
    // Parameterized keep/discard gap percentile (/keep-pN).
    if let Some(p) = parse_keep_percentile(content) {
        return Some(format_keep_percentile_gateway(p, content));
    }
    // results.tsv keep/discard min–max gap (range) before median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_range_request(content) {
        return Some(format_results_tsv_range_gateway(content));
    }
    // results.tsv keep/discard median gap before average pace / last-row / recent / counts.
    if looks_like_results_tsv_median_request(content) {
        return Some(format_results_tsv_median_gateway(content));
    }
    // results.tsv keep/discard average gap (pace) before last-row / recent / counts.
    if looks_like_results_tsv_pace_request(content) {
        return Some(format_results_tsv_pace_gateway(content));
    }
    // results.tsv last keep/discard (one row) before recent list / counts / size/age/path.
    if looks_like_results_tsv_last_request(content) {
        return Some(format_results_tsv_last_gateway(content));
    }
    // results.tsv recent tonight list (capped) before rate / streak / counts / size/age/path.
    if looks_like_results_tsv_recent_request(content) {
        return Some(format_results_tsv_recent_gateway(content));
    }
    // results.tsv longest keep/discard streak before current streak / rate / counts (record only).
    if looks_like_results_tsv_longest_streak_request(content) {
        return Some(format_results_tsv_longest_streak_gateway());
    }
    // results.tsv keep/discard streak before rate / counts / size/age/path (consecutive only).
    if looks_like_results_tsv_streak_request(content) {
        return Some(format_results_tsv_streak_gateway());
    }
    // results.tsv keep/discard hit rate before counts / size/age/path (percentages only).
    if looks_like_results_tsv_rate_request(content) {
        return Some(format_results_tsv_rate_gateway());
    }
    // results.tsv keep/discard counts before size/age/path (no row dump).
    if looks_like_results_tsv_count_request(content) {
        return Some(format_results_tsv_count_gateway());
    }
    // results.tsv size before age/path (stat only; no dump); path before improvements-dir.
    if looks_like_results_tsv_size_request(content) {
        return Some(format_results_tsv_size_gateway());
    }
    if looks_like_results_tsv_age_request(content) {
        return Some(format_results_tsv_age_gateway());
    }
    if looks_like_results_tsv_path_request(content) {
        return Some(format_results_tsv_path_gateway());
    }
    // loop_backlog.md size before age before path (stat only; no dump); path before improvements-dir.
    if looks_like_loop_backlog_size_request(content) {
        return Some(format_loop_backlog_size_gateway());
    }
    if looks_like_loop_backlog_age_request(content) {
        return Some(format_loop_backlog_age_gateway());
    }
    if looks_like_loop_backlog_path_request(content) {
        return Some(format_loop_backlog_path_gateway());
    }
    // sibling_harness.md size before age before path (stat only; no dump); path before improvements-dir / notes.
    if looks_like_sibling_harness_size_request(content) {
        return Some(format_sibling_harness_size_gateway());
    }
    if looks_like_sibling_harness_age_request(content) {
        return Some(format_sibling_harness_age_gateway());
    }
    if looks_like_sibling_harness_path_request(content) {
        return Some(format_sibling_harness_path_gateway());
    }
    // standing_backlog.md size before age before path (stat only; no dump); path before improvements-dir / notes.
    if looks_like_standing_backlog_size_request(content) {
        return Some(format_standing_backlog_size_gateway());
    }
    if looks_like_standing_backlog_age_request(content) {
        return Some(format_standing_backlog_age_gateway());
    }
    if looks_like_standing_backlog_path_request(content) {
        return Some(format_standing_backlog_path_gateway());
    }
    // morning_surprise_*.md size before age before path (stat only; no dump); path before improvements-dir / content.
    if looks_like_morning_surprise_size_request(content) {
        return Some(format_morning_surprise_size_gateway());
    }
    if looks_like_morning_surprise_age_request(content) {
        return Some(format_morning_surprise_age_gateway());
    }
    if looks_like_morning_surprise_path_request(content) {
        return Some(format_morning_surprise_path_gateway());
    }
    if looks_like_downloads_organizer_ready_request(content) {
        return Some(format_downloads_organizer_ready_chip());
    }
    // Ori vault size before age/path (recursive bytes; no list/MCP); age before path; path before `/ori` Ready.
    if looks_like_ori_vault_size_request(content) {
        return Some(format_ori_vault_size_gateway());
    }
    if looks_like_ori_vault_age_request(content) {
        return Some(format_ori_vault_age_gateway());
    }
    // Ori vault path before `/ori` Ready chip (path-only asks).
    if looks_like_ori_vault_path_request(content) {
        return Some(format_ori_vault_path_gateway());
    }
    if looks_like_ori_ready_request(content) {
        return Some(format_ori_ready_chip());
    }
    if looks_like_having_fun_ready_request(content) {
        return Some(format_having_fun_ready_chip());
    }
    if looks_like_voice_stt_ready_request(content) {
        return Some(format_voice_stt_ready_chip());
    }
    if looks_like_telegram_ready_request(content) {
        return Some(format_telegram_ready_chip());
    }
    if looks_like_slack_ready_request(content) {
        return Some(format_slack_ready_chip());
    }
    if looks_like_signal_ready_request(content) {
        return Some(format_signal_ready_chip());
    }
    if looks_like_alerts_ready_request(content) {
        return Some(format_alerts_ready_chip());
    }
    if looks_like_debug_log_count_request(content) {
        let kind =
            parse_debug_log_count_kind(content).expect("looks_like_debug_log_count_request implies kind");
        return Some(format_debug_log_count_gateway(kind));
    }
    // overnight_agent.log size before age before path before debug.log (stat only; no dump/tail).
    if looks_like_overnight_agent_log_size_request(content) {
        return Some(format_overnight_agent_log_size_gateway());
    }
    if looks_like_overnight_agent_log_age_request(content) {
        return Some(format_overnight_agent_log_age_gateway());
    }
    if looks_like_overnight_agent_log_path_request(content) {
        return Some(format_overnight_agent_log_path_gateway());
    }
    // harness loop stdout size before age before path before debug.log (stat only; no dump/tail).
    if looks_like_harness_loop_stdout_size_request(content) {
        return Some(format_harness_loop_stdout_size_gateway());
    }
    if looks_like_harness_loop_stdout_age_request(content) {
        return Some(format_harness_loop_stdout_age_gateway());
    }
    if looks_like_harness_loop_stdout_path_request(content) {
        return Some(format_harness_loop_stdout_path_gateway());
    }
    // harness loop stderr size before age before path (Track B loop capture; no dump/tail).
    if looks_like_harness_loop_stderr_size_request(content) {
        return Some(format_harness_loop_stderr_size_gateway());
    }
    if looks_like_harness_loop_stderr_age_request(content) {
        return Some(format_harness_loop_stderr_age_gateway());
    }
    if looks_like_harness_loop_stderr_path_request(content) {
        return Some(format_harness_loop_stderr_path_gateway());
    }
    // launchd.stderr.log size before age before path (app LaunchAgent stderr redirect; no dump/tail).
    if looks_like_launchd_stderr_size_request(content) {
        return Some(format_launchd_stderr_size_gateway());
    }
    if looks_like_launchd_stderr_age_request(content) {
        return Some(format_launchd_stderr_age_gateway());
    }
    if looks_like_launchd_stderr_path_request(content) {
        return Some(format_launchd_stderr_path_gateway());
    }
    // launchd.stdout.log size before age before path (app LaunchAgent stdout redirect; no dump/tail).
    if looks_like_launchd_stdout_size_request(content) {
        return Some(format_launchd_stdout_size_gateway());
    }
    if looks_like_launchd_stdout_age_request(content) {
        return Some(format_launchd_stdout_age_gateway());
    }
    if looks_like_launchd_stdout_path_request(content) {
        return Some(format_launchd_stdout_path_gateway());
    }
    if looks_like_debug_log_size_request(content) {
        return Some(format_debug_log_size_gateway());
    }
    if looks_like_debug_log_path_request(content) {
        return Some(format_debug_log_path_gateway());
    }
    if looks_like_debug_log_age_request(content) {
        return Some(format_debug_log_age_gateway());
    }
    // .config.env size before age/path (stat only; never dumps keys).
    if looks_like_config_env_size_request(content) {
        return Some(format_config_env_size_gateway());
    }
    // .config.env age before path (mtime only; never dumps keys).
    if looks_like_config_env_age_request(content) {
        return Some(format_config_env_age_gateway());
    }
    // .config.env before generic config.json / data-home path lane.
    if looks_like_config_env_path_request(content) {
        return Some(format_config_env_path_gateway());
    }
    // config.json size before age/path (stat only; no dump).
    if looks_like_config_size_request(content) {
        return Some(format_config_size_gateway());
    }
    // config.json age before path (mtime only; no dump).
    if looks_like_config_age_request(content) {
        return Some(format_config_age_gateway());
    }
    // Improvements dir size before age/path (recursive bytes; no list); age before path.
    if looks_like_improvements_size_request(content) {
        return Some(format_improvements_size_gateway());
    }
    if looks_like_improvements_age_request(content) {
        return Some(format_improvements_age_gateway());
    }
    // Improvements dir before overnight-content asks collide on the word “improvements”.
    if looks_like_improvements_path_request(content) {
        return Some(format_improvements_path_gateway());
    }
    // credential_accounts.json size before age/path (stat only; no dump).
    if looks_like_credential_accounts_size_request(content) {
        return Some(format_credential_accounts_size_gateway());
    }
    // credential_accounts.json age before path (mtime only; no dump).
    if looks_like_credential_accounts_age_request(content) {
        return Some(format_credential_accounts_age_gateway());
    }
    // credential_accounts.json before browser-credentials / generic “credentials” path lanes.
    if looks_like_credential_accounts_path_request(content) {
        return Some(format_credential_accounts_path_gateway());
    }
    // escalation_patterns.md size before age/path (stat only; no dump).
    if looks_like_escalation_patterns_size_request(content) {
        return Some(format_escalation_patterns_size_gateway());
    }
    // escalation_patterns.md age before path (mtime only; no dump).
    if looks_like_escalation_patterns_age_request(content) {
        return Some(format_escalation_patterns_age_gateway());
    }
    // escalation_patterns.md before agents-dir / session-reset path lanes.
    if looks_like_escalation_patterns_path_request(content) {
        return Some(format_escalation_patterns_path_gateway());
    }
    // before-compaction transcript size before age/path (stat only; no dump/hook).
    if looks_like_before_compaction_transcript_size_request(content) {
        return Some(format_before_compaction_transcript_size_gateway());
    }
    // before-compaction transcript age before path (mtime only; no dump/hook).
    if looks_like_before_compaction_transcript_age_request(content) {
        return Some(format_before_compaction_transcript_age_gateway());
    }
    // before-compaction transcript before before-reset / session_reset_phrases / session-dir path lanes.
    if looks_like_before_compaction_transcript_path_request(content) {
        return Some(format_before_compaction_transcript_path_gateway());
    }
    // before-reset transcript size before age/path (stat only; no dump/hook).
    if looks_like_before_reset_transcript_size_request(content) {
        return Some(format_before_reset_transcript_size_gateway());
    }
    // before-reset transcript age before path (mtime only; no dump/hook).
    if looks_like_before_reset_transcript_age_request(content) {
        return Some(format_before_reset_transcript_age_gateway());
    }
    // before-reset transcript before session_reset_phrases / session-dir path lanes.
    if looks_like_before_reset_transcript_path_request(content) {
        return Some(format_before_reset_transcript_path_gateway());
    }
    // session_reset_phrases.md size before age/path (stat only; no dump).
    if looks_like_session_reset_phrases_size_request(content) {
        return Some(format_session_reset_phrases_size_gateway());
    }
    // session_reset_phrases.md age before path (mtime only; no dump).
    if looks_like_session_reset_phrases_age_request(content) {
        return Some(format_session_reset_phrases_age_gateway());
    }
    // session_reset_phrases.md before agents-dir / generic session path lanes.
    if looks_like_session_reset_phrases_path_request(content) {
        return Some(format_session_reset_phrases_path_gateway());
    }
    // cookie_reject_patterns.md size before age/path (stat only; no dump).
    if looks_like_cookie_reject_patterns_size_request(content) {
        return Some(format_cookie_reject_patterns_size_gateway());
    }
    // cookie_reject_patterns.md age before path (mtime only; no dump).
    if looks_like_cookie_reject_patterns_age_request(content) {
        return Some(format_cookie_reject_patterns_age_gateway());
    }
    // cookie_reject_patterns.md before browser cookies / agents-dir path lanes.
    if looks_like_cookie_reject_patterns_path_request(content) {
        return Some(format_cookie_reject_patterns_path_gateway());
    }
    // downloads-organizer-rules.md size before age/path (stat only; no dump).
    if looks_like_downloads_organizer_rules_size_request(content) {
        return Some(format_downloads_organizer_rules_size_gateway());
    }
    // downloads-organizer-rules.md age before path (mtime only; no dump).
    if looks_like_downloads_organizer_rules_age_request(content) {
        return Some(format_downloads_organizer_rules_age_gateway());
    }
    // downloads-organizer-state.json size before age/path (stat only; no dump).
    if looks_like_downloads_organizer_state_size_request(content) {
        return Some(format_downloads_organizer_state_size_gateway());
    }
    // downloads-organizer-state.json age before path (mtime only; no dump).
    if looks_like_downloads_organizer_state_age_request(content) {
        return Some(format_downloads_organizer_state_age_gateway());
    }
    // downloads-organizer-state.json before rules / agents-dir / browser-downloads path lanes.
    if looks_like_downloads_organizer_state_path_request(content) {
        return Some(format_downloads_organizer_state_path_gateway());
    }
    // downloads-organizer-rules.md before agents-dir / browser-downloads path lanes.
    if looks_like_downloads_organizer_rules_path_request(content) {
        return Some(format_downloads_organizer_rules_path_gateway());
    }
    // agent.json size before age/path (stat only; no dump).
    if looks_like_agent_json_size_request(content) {
        return Some(format_agent_json_size_gateway());
    }
    // agent.json age before path (mtime only; no dump).
    if looks_like_agent_json_age_request(content) {
        return Some(format_agent_json_age_gateway());
    }
    // agent.json before testing / skill / mood / soul / agents-dir path lane.
    if looks_like_agent_json_path_request(content) {
        return Some(format_agent_json_path_gateway());
    }
    // execution_prompt.md size before age/path (stat only; no dump).
    if looks_like_execution_prompt_size_request(content) {
        return Some(format_execution_prompt_size_gateway());
    }
    // execution_prompt.md age before path (mtime only; no dump).
    if looks_like_execution_prompt_age_request(content) {
        return Some(format_execution_prompt_age_gateway());
    }
    // execution_prompt.md before planning / prompts-dir / testing / skill path lanes.
    if looks_like_execution_prompt_path_request(content) {
        return Some(format_execution_prompt_path_gateway());
    }
    // planning_prompt.md size before age/path (stat only; no dump).
    if looks_like_planning_prompt_size_request(content) {
        return Some(format_planning_prompt_size_gateway());
    }
    // planning_prompt.md age before path (mtime only; no dump).
    if looks_like_planning_prompt_age_request(content) {
        return Some(format_planning_prompt_age_gateway());
    }
    // planning_prompt.md before prompts-dir / testing / skill path lanes.
    if looks_like_planning_prompt_path_request(content) {
        return Some(format_planning_prompt_path_gateway());
    }
    // testing.md size before age/path (stat only; no dump).
    if looks_like_testing_md_size_request(content) {
        return Some(format_testing_md_size_gateway());
    }
    // testing.md age before path (mtime only; no dump).
    if looks_like_testing_md_age_request(content) {
        return Some(format_testing_md_age_gateway());
    }
    // testing.md before skill / mood / soul / agents-dir path lane.
    if looks_like_testing_md_path_request(content) {
        return Some(format_testing_md_path_gateway());
    }
    // skill.md size before age/path (stat only; no dump).
    if looks_like_skill_md_size_request(content) {
        return Some(format_skill_md_size_gateway());
    }
    // skill.md age before path (mtime only; no dump).
    if looks_like_skill_md_age_request(content) {
        return Some(format_skill_md_age_gateway());
    }
    // skill.md before mood / soul / skills-dir / agents-dir path lane.
    if looks_like_skill_md_path_request(content) {
        return Some(format_skill_md_path_gateway());
    }
    // mood.md size before age/path (stat only; no dump).
    if looks_like_mood_size_request(content) {
        return Some(format_mood_size_gateway());
    }
    // mood.md age before path (mtime only; no dump).
    if looks_like_mood_age_request(content) {
        return Some(format_mood_age_gateway());
    }
    // mood.md before soul / agents-dir path lane.
    if looks_like_mood_path_request(content) {
        return Some(format_mood_path_gateway());
    }
    // soul.md size before age/path (stat only; no dump).
    if looks_like_soul_size_request(content) {
        return Some(format_soul_size_gateway());
    }
    // soul.md age before path (mtime only; no dump).
    if looks_like_soul_age_request(content) {
        return Some(format_soul_age_gateway());
    }
    // soul.md before agents-dir path lane.
    if looks_like_soul_path_request(content) {
        return Some(format_soul_path_gateway());
    }
    // memory.md size before age/path (stat only; no dump).
    if looks_like_memory_md_size_request(content) {
        return Some(format_memory_md_size_gateway());
    }
    // memory.md age before path (mtime only; no dump).
    if looks_like_memory_md_age_request(content) {
        return Some(format_memory_md_age_gateway());
    }
    // memory.md curated file before notes-folder memory path lane.
    if looks_like_memory_md_path_request(content) {
        return Some(format_memory_md_path_gateway());
    }
    if looks_like_config_path_request(content) {
        return Some(format_config_path_gateway());
    }
    // user-info.json size before age/path (stat only; no dump).
    if looks_like_user_info_size_request(content) {
        return Some(format_user_info_size_gateway());
    }
    // user-info.json age before path (mtime only; no dump).
    if looks_like_user_info_age_request(content) {
        return Some(format_user_info_age_gateway());
    }
    // user-info before nested sibling path detectors (string-only; avoids exponential nest).
    if looks_like_user_info_path_request(content) {
        return Some(format_user_info_path_gateway());
    }
    // discord_channels.json size before age/path (stat only; no dump).
    if looks_like_discord_channels_size_request(content) {
        return Some(format_discord_channels_size_gateway());
    }
    // discord_channels.json age before path (mtime only; no dump).
    if looks_like_discord_channels_age_request(content) {
        return Some(format_discord_channels_age_gateway());
    }
    if looks_like_discord_channels_path_request(content) {
        return Some(format_discord_channels_path_gateway());
    }
    // scheduler_delivery_awareness.json size before age/path (stat only; no dump).
    if looks_like_scheduler_delivery_awareness_size_request(content) {
        return Some(format_scheduler_delivery_awareness_size_gateway());
    }
    // scheduler_delivery_awareness.json age before path (mtime only; no dump).
    if looks_like_scheduler_delivery_awareness_age_request(content) {
        return Some(format_scheduler_delivery_awareness_age_gateway());
    }
    if looks_like_scheduler_delivery_awareness_path_request(content) {
        return Some(format_scheduler_delivery_awareness_path_gateway());
    }
    // Screenshots dir age before size/path (newest mtime; no list); size before path; path before take/list.
    if looks_like_screenshots_age_request(content) {
        return Some(format_screenshots_age_gateway());
    }
    if looks_like_screenshots_size_request(content) {
        return Some(format_screenshots_size_gateway());
    }
    if looks_like_screenshots_path_request(content) {
        return Some(format_screenshots_path_gateway());
    }
    // runs.jsonl size before age/path (stat only; no list/count).
    if looks_like_runs_size_request(content) {
        return Some(format_runs_size_gateway());
    }
    // runs.jsonl age before path (mtime; no list/count).
    if looks_like_runs_age_request(content) {
        return Some(format_runs_age_gateway());
    }
    if looks_like_runs_path_request(content) {
        return Some(format_runs_path_gateway());
    }
    // Task dir age before size/path (newest mtime; no list); size before path; path before Active catalog.
    if looks_like_task_age_request(content) {
        return Some(format_task_age_gateway());
    }
    if looks_like_task_size_request(content) {
        return Some(format_task_size_gateway());
    }
    if looks_like_task_path_request(content) {
        return Some(format_task_path_gateway());
    }
    // memory.md size before age/path (stat only; no dump).
    if looks_like_memory_md_size_request(content) {
        return Some(format_memory_md_size_gateway());
    }
    // memory.md age before path (mtime only; no dump).
    if looks_like_memory_md_age_request(content) {
        return Some(format_memory_md_age_gateway());
    }
    // memory.md curated file before notes-folder combo path.
    if looks_like_memory_md_path_request(content) {
        return Some(format_memory_md_path_gateway());
    }
    // Discord channel memory size before age/path (stat only; no list/dump).
    if looks_like_discord_memory_size_request(content) {
        return Some(format_discord_memory_size_gateway());
    }
    // Discord channel memory age before path (newest mtime; no list/dump).
    if looks_like_discord_memory_age_request(content) {
        return Some(format_discord_memory_age_gateway());
    }
    // Discord channel memory path before notes-folder / Knowledge list.
    if looks_like_discord_memory_path_request(content) {
        return Some(format_discord_memory_path_gateway());
    }
    // Session-memory-*.md size before age/path (stat only; no list/dump).
    if looks_like_session_memory_size_request(content) {
        return Some(format_session_memory_size_gateway());
    }
    // Session-memory-*.md age before path (newest mtime; no list/dump).
    if looks_like_session_memory_age_request(content) {
        return Some(format_session_memory_age_gateway());
    }
    // Session-memory-*.md path before notes-folder / session-dir path.
    if looks_like_session_memory_path_request(content) {
        return Some(format_session_memory_path_gateway());
    }
    // LaunchAgent plist age before size/path (mtime only; no dump/load).
    if looks_like_launchagent_age_request(content) {
        return Some(format_launchagent_age_gateway());
    }
    // LaunchAgent plist size before path (stat only; no dump/load).
    if looks_like_launchagent_size_request(content) {
        return Some(format_launchagent_size_gateway());
    }
    // LaunchAgent plist paths (path-only; no load/unload).
    if looks_like_launchagent_path_request(content) {
        return Some(format_launchagent_path_gateway());
    }
    // results.tsv age since last keep/discard before first-tonight / last-row / recent / streaks.
    if looks_like_results_tsv_since_request(content) {
        return Some(format_results_tsv_since_gateway(content));
    }
    // results.tsv first keep/discard tonight (one row) before last-row / recent / counts.
    if looks_like_results_tsv_first_request(content) {
        return Some(format_results_tsv_first_gateway(content));
    }
    // results.tsv keep/discard gap histogram before Gini / modal / Shannon entropy / excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_histogram_request(content) {
        return Some(format_results_tsv_histogram_gateway(content));
    }
    // results.tsv keep/discard Gini gap before modal / Shannon entropy / excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_gini_request(content) {
        return Some(format_results_tsv_gini_gateway(content));
    }
    // results.tsv keep/discard modal gap before Shannon entropy / excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_mode_request(content) {
        return Some(format_results_tsv_mode_gateway(content));
    }
    // results.tsv keep/discard Shannon entropy before excess kurtosis / skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_entropy_request(content) {
        return Some(format_results_tsv_entropy_gateway(content));
    }
    // results.tsv keep/discard excess kurtosis before skewness / CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_kurtosis_request(content) {
        return Some(format_results_tsv_kurtosis_gateway(content));
    }
    // results.tsv keep/discard skewness before CV / MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_skew_request(content) {
        return Some(format_results_tsv_skew_gateway(content));
    }
    // results.tsv keep/discard CV (std/mean %) before MAD / sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_cv_request(content) {
        return Some(format_results_tsv_cv_gateway(content));
    }
    // results.tsv keep/discard MAD gap before sample-std / IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_mad_request(content) {
        return Some(format_results_tsv_mad_gateway(content));
    }
    // results.tsv keep/discard sample-std gap before IQR / p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_std_request(content) {
        return Some(format_results_tsv_std_gateway(content));
    }
    // results.tsv keep/discard IQR gap before p90 / range / median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_iqr_request(content) {
        return Some(format_results_tsv_iqr_gateway(content));
    }
    // results.tsv keep/discard min–max gap (range) before median / average pace / last-row / recent / counts.
    if looks_like_results_tsv_range_request(content) {
        return Some(format_results_tsv_range_gateway(content));
    }
    // results.tsv keep/discard median gap before average pace / last-row / recent / counts.
    if looks_like_results_tsv_median_request(content) {
        return Some(format_results_tsv_median_gateway(content));
    }
    // results.tsv keep/discard average gap (pace) before last-row / recent / counts.
    if looks_like_results_tsv_pace_request(content) {
        return Some(format_results_tsv_pace_gateway(content));
    }
    // results.tsv last keep/discard (one row) before recent list / counts / size/age/path.
    if looks_like_results_tsv_last_request(content) {
        return Some(format_results_tsv_last_gateway(content));
    }
    // results.tsv recent tonight list (capped) before rate / streak / counts / size/age/path.
    if looks_like_results_tsv_recent_request(content) {
        return Some(format_results_tsv_recent_gateway(content));
    }
    // results.tsv longest keep/discard streak before current streak / rate / counts (record only).
    if looks_like_results_tsv_longest_streak_request(content) {
        return Some(format_results_tsv_longest_streak_gateway());
    }
    // results.tsv keep/discard streak before rate / counts / size/age/path (consecutive only).
    if looks_like_results_tsv_streak_request(content) {
        return Some(format_results_tsv_streak_gateway());
    }
    // results.tsv keep/discard hit rate before counts / size/age/path (percentages only).
    if looks_like_results_tsv_rate_request(content) {
        return Some(format_results_tsv_rate_gateway());
    }
    // results.tsv keep/discard counts before size/age/path (no row dump).
    if looks_like_results_tsv_count_request(content) {
        return Some(format_results_tsv_count_gateway());
    }
    // results.tsv size before age/path (stat only; no dump); path before improvements-dir.
    if looks_like_results_tsv_size_request(content) {
        return Some(format_results_tsv_size_gateway());
    }
    if looks_like_results_tsv_age_request(content) {
        return Some(format_results_tsv_age_gateway());
    }
    if looks_like_results_tsv_path_request(content) {
        return Some(format_results_tsv_path_gateway());
    }
    // loop_backlog.md size before age before path (stat only; no dump); path before notes/memory.
    if looks_like_loop_backlog_size_request(content) {
        return Some(format_loop_backlog_size_gateway());
    }
    if looks_like_loop_backlog_age_request(content) {
        return Some(format_loop_backlog_age_gateway());
    }
    if looks_like_loop_backlog_path_request(content) {
        return Some(format_loop_backlog_path_gateway());
    }
    // sibling_harness.md size before age before path (stat only; no dump); path before notes/memory.
    if looks_like_sibling_harness_size_request(content) {
        return Some(format_sibling_harness_size_gateway());
    }
    if looks_like_sibling_harness_age_request(content) {
        return Some(format_sibling_harness_age_gateway());
    }
    if looks_like_sibling_harness_path_request(content) {
        return Some(format_sibling_harness_path_gateway());
    }
    // standing_backlog.md size before age before path (stat only; no dump); path before notes/memory.
    if looks_like_standing_backlog_size_request(content) {
        return Some(format_standing_backlog_size_gateway());
    }
    if looks_like_standing_backlog_age_request(content) {
        return Some(format_standing_backlog_age_gateway());
    }
    if looks_like_standing_backlog_path_request(content) {
        return Some(format_standing_backlog_path_gateway());
    }
    // morning_surprise_*.md size before age before path (stat only; no dump); path before notes/memory.
    if looks_like_morning_surprise_size_request(content) {
        return Some(format_morning_surprise_size_gateway());
    }
    if looks_like_morning_surprise_age_request(content) {
        return Some(format_morning_surprise_age_gateway());
    }
    if looks_like_morning_surprise_path_request(content) {
        return Some(format_morning_surprise_path_gateway());
    }
    // overnight_agent.log size before age before path (Track B harness transcript; no dump/tail).
    if looks_like_overnight_agent_log_size_request(content) {
        return Some(format_overnight_agent_log_size_gateway());
    }
    if looks_like_overnight_agent_log_age_request(content) {
        return Some(format_overnight_agent_log_age_gateway());
    }
    if looks_like_overnight_agent_log_path_request(content) {
        return Some(format_overnight_agent_log_path_gateway());
    }
    // harness loop stdout size before age before path (Track B loop capture; no dump/tail).
    if looks_like_harness_loop_stdout_size_request(content) {
        return Some(format_harness_loop_stdout_size_gateway());
    }
    if looks_like_harness_loop_stdout_age_request(content) {
        return Some(format_harness_loop_stdout_age_gateway());
    }
    if looks_like_harness_loop_stdout_path_request(content) {
        return Some(format_harness_loop_stdout_path_gateway());
    }
    // harness loop stderr size before age before path (Track B loop capture; no dump/tail).
    if looks_like_harness_loop_stderr_size_request(content) {
        return Some(format_harness_loop_stderr_size_gateway());
    }
    if looks_like_harness_loop_stderr_age_request(content) {
        return Some(format_harness_loop_stderr_age_gateway());
    }
    if looks_like_harness_loop_stderr_path_request(content) {
        return Some(format_harness_loop_stderr_path_gateway());
    }
    // launchd.stderr.log size before age before path (app LaunchAgent stderr redirect; no dump/tail).
    if looks_like_launchd_stderr_size_request(content) {
        return Some(format_launchd_stderr_size_gateway());
    }
    if looks_like_launchd_stderr_age_request(content) {
        return Some(format_launchd_stderr_age_gateway());
    }
    if looks_like_launchd_stderr_path_request(content) {
        return Some(format_launchd_stderr_path_gateway());
    }
    // launchd.stdout.log size before age before path (app LaunchAgent stdout redirect; no dump/tail).
    if looks_like_launchd_stdout_size_request(content) {
        return Some(format_launchd_stdout_size_gateway());
    }
    if looks_like_launchd_stdout_age_request(content) {
        return Some(format_launchd_stdout_age_gateway());
    }
    if looks_like_launchd_stdout_path_request(content) {
        return Some(format_launchd_stdout_path_gateway());
    }
    // Chip name before ring chips ("what's the cpu" stays usage).
    // (Installed RAM is routed early — before resource-stat size nouns steal `ram size`.)
    if looks_like_chip_identity_request(content) {
        return Some(format_chip_identity_gateway());
    }
    // macOS sw_vers before ring/strip ("what version" / bare `/version` stay app status).
    if looks_like_macos_version_request(content) {
        return Some(format_macos_version_gateway());
    }
    // ComputerName · LocalHostName (bare "host" / monitor host stay out).
    if looks_like_hostname_request(content) {
        return Some(format_hostname_gateway());
    }
    // Mac marketing name · machine model (bare `/model` · `which model` stay Ollama; `/chip` stays chip).
    if looks_like_mac_model_request(content) {
        return Some(format_mac_model_gateway());
    }
    // Login username (bare "user" / user-info.json stay out).
    if looks_like_username_request(content) {
        return Some(format_username_gateway());
    }
    // Hardware serial (bare "sn" stays with the agent; Mac model / chip stay their lanes).
    if looks_like_serial_request(content) {
        return Some(format_serial_gateway());
    }
    // Local timezone (bare `/time` / `what time` stay with the agent).
    if looks_like_timezone_request(content) {
        return Some(format_timezone_gateway());
    }
    // System locale / language (bare `lang` / programming language stay with the agent).
    if looks_like_locale_request(content) {
        return Some(format_locale_gateway());
    }
    // Kernel release (bare `uname` / `/uname` stay with the agent; `/uname-m` stays arch).
    if looks_like_kernel_request(content) {
        return Some(format_kernel_gateway());
    }
    // Machine arch (bare "architect" theme / chip name stay out).
    if looks_like_architecture_request(content) {
        return Some(format_architecture_gateway());
    }
    // CPU core count (P/E clocks stay `/p-core` · `/e-core`; `/chip` stays name; bare `core` stays out).
    if looks_like_cpu_cores_request(content) {
        return Some(format_cpu_cores_gateway());
    }
    // Boot volume / startup disk (bare `boot` / `volume` / `disk` stay with the agent; `/boot` stays boot time;
    // `view disk` stays Disk Cleanup; `/ssd` · `/free` stay capacity · free).
    if looks_like_boot_volume_request(content) {
        return Some(format_boot_volume_report());
    }
    // FileVault status (bare `filevault` / `fv` / `encryption` stay with the agent; `/boot-volume` stays volume;
    // `view disk` stays Disk Cleanup; `/ssd` · `/free` stay capacity · free).
    if looks_like_filevault_request(content) {
        return Some(format_filevault_report());
    }
    // SIP status (bare `sip` / `csrutil` stay with the agent; `/filevault` stays FileVault; `/gatekeeper` stays Gatekeeper).
    if looks_like_sip_request(content) {
        return Some(format_sip_report());
    }
    // Gatekeeper status (bare `gatekeeper` / `spctl` stay with the agent; `/sip` stays SIP; `/filevault` stays FileVault).
    if looks_like_gatekeeper_request(content) {
        return Some(format_gatekeeper_report());
    }
    // Firewall stealth (bare `stealth` stays with the agent; `/firewall` stays global state; `/gatekeeper` stays Gatekeeper).
    if looks_like_firewall_stealth_request(content) {
        return Some(format_firewall_stealth_report());
    }
    // Application Firewall (bare `firewall` / `afw` stay with the agent; `/gatekeeper` stays Gatekeeper; `/sip` stays SIP;
    // `/stealth` · `/firewall-stealth` stay stealth).
    if looks_like_firewall_request(content) {
        return Some(format_firewall_report());
    }
    // Time Machine (bare `backup` / `tm` / `time machine` stay with the agent; `view disk` stays Disk Cleanup;
    // `/ssd` · `/free` stay capacity · free; `/boot-volume` stays startup disk).
    if looks_like_time_machine_request(content) {
        return Some(format_time_machine_report());
    }
    // Remote Login / SSH (bare `ssh` / `sshd` / `remote login` stay with the agent; `/vpn` stays VPN;
    // `/firewall` stays Firewall; `/timemachine` stays Time Machine; Screen Sharing stays `/screen-sharing`).
    if looks_like_remote_login_request(content) {
        return Some(format_remote_login_report());
    }
    // Screen Sharing / VNC (bare `screen sharing` / `vnc` stay with the agent; `/remote-login` · `/ssh`
    // stay Remote Login; `/screen` · `/screens` stay display resolution; File Sharing stays `/file-sharing`).
    if looks_like_screen_sharing_request(content) {
        return Some(format_screen_sharing_report());
    }
    // File Sharing / SMB (bare `file sharing` / `smb` / `smbd` stay with the agent; `/screen-sharing` · `/vnc`
    // stay Screen Sharing; `/remote-login` · `/ssh` stay Remote Login; `/filevault` stays FileVault;
    // Printer Sharing stays `/printer-sharing`).
    if looks_like_file_sharing_request(content) {
        return Some(format_file_sharing_report());
    }
    // Printer Sharing (bare `printer` / `printers` / `cups` stay with the agent; `/file-sharing` · `/smb`
    // stay File Sharing; `/screen-sharing` · `/vnc` stay Screen Sharing; `/remote-login` · `/ssh` stay Remote Login).
    if looks_like_printer_sharing_request(content) {
        return Some(format_printer_sharing_report());
    }
    // Remote Management / ARD (bare `ard` / `remote management` / `remote desktop` stay with the agent;
    // `/screen-sharing` · `/vnc` stay Screen Sharing; `/remote-login` · `/ssh` stay Remote Login;
    // `/file-sharing` · `/smb` stay File Sharing; `/printer-sharing` stays Printer Sharing;
    // Content Caching stays `/content-caching` · `/asset-cache`).
    if looks_like_remote_management_request(content) {
        return Some(format_remote_management_report());
    }
    // Content Caching / AssetCache (bare `caching` / `content caching` / `asset cache` stay with the agent;
    // `/remote-management` · `/ard` stay Remote Management; `/file-sharing` · `/smb` stay File Sharing;
    // `/printer-sharing` stays Printer Sharing; `/screen-sharing` · `/vnc` stay Screen Sharing;
    // `/remote-login` · `/ssh` stay Remote Login; Internet Sharing stays `/internet-sharing` · `/ics`).
    if looks_like_content_caching_request(content) {
        return Some(format_content_caching_report());
    }
    // Internet Sharing / ICS / NAT (bare `internet` / `nat` / `ics` / `internet sharing` stay with the agent;
    // `/vpn` stays VPN; `/content-caching` · `/asset-cache` stay Content Caching; `/file-sharing` · `/smb`
    // stay File Sharing; `/remote-management` · `/ard` stay Remote Management; `/ip` stays LAN).
    if looks_like_internet_sharing_request(content) {
        return Some(format_internet_sharing_report());
    }
    // Last boot wall-clock (bare `/uptime` / `view uptime` stay Mac Up; `/app-uptime` stays process;
    // `/boot-volume` · `/startup-disk` stay boot volume).
    if looks_like_boot_time_request(content) {
        return Some(format_boot_time_gateway());
    }
    // Local LAN IPv4 (bare `ip` / `what's my ip` / public·wan stay with the agent).
    if looks_like_local_ip_request(content) {
        return Some(format_local_ip_gateway());
    }
    // Default gateway (Discord `/discord` · bare gateway/router stay out; `/ip` stays LAN).
    if looks_like_default_gateway_request(content) {
        return Some(format_default_gateway_report());
    }
    // DNS nameservers (bare `dns` / lookup·dig·flush stay with the agent; `/gateway` stays route).
    if looks_like_dns_servers_request(content) {
        return Some(format_dns_servers_report());
    }
    // Wi-Fi SSID (bare `wifi` / `ssid` / password·settings stay with the agent; `/ip` stays LAN).
    if looks_like_wifi_ssid_request(content) {
        return Some(format_wifi_ssid_report());
    }
    // Primary MAC / ethernet (bare `mac` / `ether` stay with the agent; `/mac-model` stays model; `/wifi` stays SSID).
    if looks_like_mac_address_request(content) {
        return Some(format_mac_address_report());
    }
    // Primary interface MTU (bare `mtu` stays with the agent; `/mac-address` stays ethernet; `/ip` stays LAN).
    if looks_like_mtu_request(content) {
        return Some(format_mtu_report());
    }
    // Primary IPv4 netmask (bare `netmask` / `subnet` / `mask` stay with the agent; `/ip` stays LAN; `/mtu` stays MTU).
    if looks_like_netmask_request(content) {
        return Some(format_netmask_report());
    }
    // Primary / default-route iface name (bare `iface` / `interface` stay with the agent; `/iface-mtu` stays MTU; `/en0` stays LAN).
    if looks_like_primary_iface_request(content) {
        return Some(format_primary_iface_report());
    }
    // Primary IPv4 broadcast (bare `broadcast` / `bcast` stay with the agent; `/ip` stays LAN; `/netmask` stays mask).
    if looks_like_broadcast_request(content) {
        return Some(format_broadcast_report());
    }
    // Primary link status · media / speed (bare `status` / `media` / `speed` / `link` stay with the agent; `/link-mtu` stays MTU; `/link-address` stays MAC).
    if looks_like_link_status_request(content) {
        return Some(format_link_status_report());
    }
    // Battery cycles · health · max capacity (bare `cycles` stay with the agent; `/battery` · view battery stay Bat %).
    if looks_like_battery_cycles_request(content) {
        return Some(format_battery_cycles_report());
    }
    // AC charger · adapter watts (bare `charger` / `adapter` stay with the agent; `/watts` stays CPU+GPU draw; `/battery` · is it charging stay Bat %).
    if looks_like_ac_charger_request(content) {
        return Some(format_ac_charger_report());
    }
    // Battery time-to-empty / time-to-full (bare `time` / `remaining` stay with the agent; `battery left` / `/battery` stay Bat %).
    if looks_like_battery_time_request(content) {
        return Some(format_battery_time_report());
    }
    // Connected displays · resolutions (bare `display` / `resolution` / `screen` stay with the agent; `/gpu` stays GPU ring; screenshots / view monitors stay elsewhere).
    if looks_like_display_resolution_request(content) {
        return Some(format_display_resolution_report());
    }
    // GPU chipset · cores · Metal (bare `gpu` / `view gpu` / `what's the gpu` stay usage ring; `/chip` stays CPU chip; `/display` stays displays).
    if looks_like_gpu_identity_request(content) {
        return Some(format_gpu_identity_report());
    }
    // VPN status (bare `vpn` stays with the agent; `/wifi` stays SSID; `/ip` stays LAN; `/signal` stays alerts).
    if looks_like_vpn_status_request(content) {
        return Some(format_vpn_status_report());
    }
    // Hardware UUID (bare `uuid` stays with the agent; `/serial` stays serial; `/udid` stays with the agent).
    if looks_like_hardware_uuid_request(content) {
        return Some(format_hardware_uuid_report());
    }
    // Notes/memory folder age before size/path (newest mtime; no list); size before path; path before scrub/save.
    if looks_like_memory_age_request(content) {
        return Some(format_memory_age_gateway());
    }
    if looks_like_memory_size_request(content) {
        return Some(format_memory_size_gateway());
    }
    if looks_like_memory_path_request(content) {
        return Some(format_memory_path_gateway());
    }
    // Session dir age before size/path (newest mtime; no list); size before path; path before Live/Files.
    if looks_like_session_age_request(content) {
        return Some(format_session_age_gateway());
    }
    if looks_like_session_size_request(content) {
        return Some(format_session_size_gateway());
    }
    if looks_like_session_path_request(content) {
        return Some(format_session_path_gateway());
    }
    // Agents dir age before size/path (newest mtime; no list); size before path; path before On/Off catalog.
    if looks_like_agents_age_request(content) {
        return Some(format_agents_age_gateway());
    }
    if looks_like_agents_size_request(content) {
        return Some(format_agents_size_gateway());
    }
    if looks_like_agents_path_request(content) {
        return Some(format_agents_path_gateway());
    }
    // Skills dir age before size/path (newest mtime; no list); size before path; path before catalog.
    if looks_like_skills_age_request(content) {
        return Some(format_skills_age_gateway());
    }
    if looks_like_skills_size_request(content) {
        return Some(format_skills_size_gateway());
    }
    if looks_like_skills_path_request(content) {
        return Some(format_skills_path_gateway());
    }
    // Plugins/scripts dir age before size/path (newest mtime; no list); size before path; path before On/Off.
    if looks_like_plugins_age_request(content) {
        return Some(format_plugins_age_gateway());
    }
    if looks_like_plugins_size_request(content) {
        return Some(format_plugins_size_gateway());
    }
    if looks_like_plugins_path_request(content) {
        return Some(format_plugins_path_gateway());
    }
    // Prompts dir age before size/path (newest mtime; no list); size before path; path before planning/execution files.
    if looks_like_prompts_age_request(content) {
        return Some(format_prompts_age_gateway());
    }
    if looks_like_prompts_size_request(content) {
        return Some(format_prompts_size_gateway());
    }
    if looks_like_prompts_path_request(content) {
        return Some(format_prompts_path_gateway());
    }
    // Tmp dir age before size/path (newest mtime; no list); size before path; path before prune/clean.
    if looks_like_tmp_age_request(content) {
        return Some(format_tmp_age_gateway());
    }
    if looks_like_tmp_size_request(content) {
        return Some(format_tmp_size_gateway());
    }
    if looks_like_tmp_path_request(content) {
        return Some(format_tmp_path_gateway());
    }
    // Uploads dir age before size/path (newest mtime; no list); size before path; path before upload/list.
    if looks_like_uploads_age_request(content) {
        return Some(format_uploads_age_gateway());
    }
    if looks_like_uploads_size_request(content) {
        return Some(format_uploads_size_gateway());
    }
    if looks_like_uploads_path_request(content) {
        return Some(format_uploads_path_gateway());
    }
    // Traces dir age before size/path (newest mtime; no list); size before path; path before prune/list.
    if looks_like_traces_age_request(content) {
        return Some(format_traces_age_gateway());
    }
    if looks_like_traces_size_request(content) {
        return Some(format_traces_size_gateway());
    }
    if looks_like_traces_path_request(content) {
        return Some(format_traces_path_gateway());
    }
    // PDFs dir age before size/path (newest mtime; no list); size before path; path before save/list.
    if looks_like_pdfs_age_request(content) {
        return Some(format_pdfs_age_gateway());
    }
    if looks_like_pdfs_size_request(content) {
        return Some(format_pdfs_size_gateway());
    }
    if looks_like_pdfs_path_request(content) {
        return Some(format_pdfs_path_gateway());
    }
    // browser-credentials.toml size before age/path (stat only; no dump).
    if looks_like_browser_credentials_size_request(content) {
        return Some(format_browser_credentials_size_gateway());
    }
    // browser-credentials.toml age before path (mtime only; no dump).
    if looks_like_browser_credentials_age_request(content) {
        return Some(format_browser_credentials_age_gateway());
    }
    if looks_like_browser_credentials_path_request(content) {
        return Some(format_browser_credentials_path_gateway());
    }
    // browser_storage_state.json size before age/path (stat only; no dump).
    if looks_like_browser_storage_state_size_request(content) {
        return Some(format_browser_storage_state_size_gateway());
    }
    // browser_storage_state.json age before path (mtime only; no dump).
    if looks_like_browser_storage_state_age_request(content) {
        return Some(format_browser_storage_state_age_gateway());
    }
    if looks_like_browser_storage_state_path_request(content) {
        return Some(format_browser_storage_state_path_gateway());
    }
    // Browser-downloads dir age before size/path (newest mtime; no list); size before path.
    if looks_like_browser_downloads_age_request(content) {
        return Some(format_browser_downloads_age_gateway());
    }
    // Browser-downloads dir size before path (recursive bytes; no list); path before download/list.
    if looks_like_browser_downloads_size_request(content) {
        return Some(format_browser_downloads_size_gateway());
    }
    if looks_like_browser_downloads_path_request(content) {
        return Some(format_browser_downloads_path_gateway());
    }
    // Cleanup-quarantine dir age before size/path (newest mtime; no list); size before path.
    if looks_like_cleanup_quarantine_age_request(content) {
        return Some(format_cleanup_quarantine_age_gateway());
    }
    // Cleanup-quarantine dir size before path (recursive bytes; no list); path before /disk.
    if looks_like_cleanup_quarantine_size_request(content) {
        return Some(format_cleanup_quarantine_size_gateway());
    }
    if looks_like_cleanup_quarantine_path_request(content) {
        return Some(format_cleanup_quarantine_path_gateway());
    }
    // pinned_processes.json size before age/path (stat only; no dump).
    if looks_like_pinned_processes_size_request(content) {
        return Some(format_pinned_processes_size_gateway());
    }
    // pinned_processes.json age before path (mtime only; no dump).
    if looks_like_pinned_processes_age_request(content) {
        return Some(format_pinned_processes_age_gateway());
    }
    if looks_like_pinned_processes_path_request(content) {
        return Some(format_pinned_processes_path_gateway());
    }
    // schedules.json size before age/path (stat only; no dump).
    if looks_like_schedules_size_request(content) {
        return Some(format_schedules_size_gateway());
    }
    // schedules.json age before path (mtime only; no dump).
    if looks_like_schedules_age_request(content) {
        return Some(format_schedules_age_gateway());
    }
    if looks_like_schedules_path_request(content) {
        return Some(format_schedules_path_gateway());
    }
    // monitors.json size before age/path (stat only; no dump).
    if looks_like_monitors_size_request(content) {
        return Some(format_monitors_size_gateway());
    }
    // monitors.json age before path (mtime only; no dump).
    if looks_like_monitors_age_request(content) {
        return Some(format_monitors_age_gateway());
    }
    if looks_like_monitors_path_request(content) {
        return Some(format_monitors_path_gateway());
    }
    // history.json size before age/path (stat only; no dump).
    if looks_like_history_size_request(content) {
        return Some(format_history_size_gateway());
    }
    // history.json age before path (mtime only; no dump).
    if looks_like_history_age_request(content) {
        return Some(format_history_age_gateway());
    }
    if looks_like_history_path_request(content) {
        return Some(format_history_path_gateway());
    }
    // disk_cleanup.json size before age/path (stat only; no dump).
    if looks_like_disk_cleanup_size_request(content) {
        return Some(format_disk_cleanup_size_gateway());
    }
    // disk_cleanup.json age before path (mtime only; no dump).
    if looks_like_disk_cleanup_age_request(content) {
        return Some(format_disk_cleanup_age_gateway());
    }
    if looks_like_disk_cleanup_path_request(content) {
        return Some(format_disk_cleanup_path_gateway());
    }
    // perplexity_last.json size before age/path (stat only; no dump).
    if looks_like_perplexity_last_size_request(content) {
        return Some(format_perplexity_last_size_gateway());
    }
    // perplexity_last.json age before path (mtime only; no dump).
    if looks_like_perplexity_last_age_request(content) {
        return Some(format_perplexity_last_age_gateway());
    }
    if looks_like_perplexity_last_path_request(content) {
        return Some(format_perplexity_last_path_gateway());
    }
    if looks_like_runs_count_request(content) {
        let kind = parse_runs_count_kind(content).expect("looks_like_runs_count_request implies kind");
        return Some(format_runs_count_gateway(kind));
    }
    if looks_like_insights_request(content) {
        let days = parse_insights_days(content);
        let insights = compute_runs_insights_for(80, days);
        return Some(format_runs_insights_gateway(&insights));
    }
    if looks_like_failed_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_failed_runs_gateway(days));
    }
    if looks_like_slow_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_slow_runs_gateway(days));
    }
    if looks_like_instant_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_instant_runs_gateway(days));
    }
    if looks_like_direct_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_direct_runs_gateway(days));
    }
    if looks_like_lite_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_lite_runs_gateway(days));
    }
    if looks_like_agents_request(content) {
        let filter = parse_agents_list_filter(content);
        return Some(format_agents_gateway(filter));
    }
    if looks_like_skills_request(content) {
        return Some(format_skills_gateway());
    }
    if looks_like_tasks_request(content) {
        let filter = parse_tasks_list_filter(content);
        return Some(format_tasks_gateway(filter));
    }
    if looks_like_plugins_request(content) {
        let filter = parse_plugins_list_filter(content);
        return Some(format_plugins_gateway(filter));
    }
    if looks_like_sessions_request(content) {
        let filter = parse_sessions_list_filter(content);
        return Some(format_sessions_gateway(filter));
    }
    if looks_like_knowledge_request(content) {
        let filter = parse_knowledge_list_filter(content);
        return Some(format_knowledge_gateway(filter));
    }
    if looks_like_next_schedule_request(content) {
        return Some(format_next_schedule_gateway());
    }
    if looks_like_last_delivery_request(content) {
        return Some(format_last_delivery_gateway());
    }
    if looks_like_schedule_count_request(content) {
        return Some(format_schedule_count_gateway(content));
    }
    if looks_like_operator_count_request(content) {
        let kind = parse_operator_count_kind(content)
            .expect("looks_like_operator_count_request implies kind");
        return Some(format_operator_count_gateway(kind));
    }
    if looks_like_schedules_request(content) {
        let filter = parse_schedules_list_filter(content);
        return Some(format_schedules_gateway(filter));
    }
    if looks_like_monitors_request(content) {
        let filter = parse_monitors_list_filter(content);
        return Some(format_monitors_gateway(filter));
    }
    if looks_like_disk_cleanup_request(content) {
        let filter = parse_disk_cleanup_list_filter(content);
        return Some(format_disk_cleanup_gateway(filter));
    }
    if looks_like_debug_log_request(content) {
        let filter = parse_debug_log_list_filter(content);
        return Some(format_debug_log_gateway(filter));
    }
    if looks_like_rings_request(content) {
        let filter = parse_rings_list_filter(content);
        return Some(format_rings_gateway(filter));
    }
    if let Some(cluster) = parse_core_cluster_freq(content) {
        return Some(format_core_cluster_freq_gateway(cluster));
    }
    if looks_like_ring_chip_request(content) {
        if let Some(ask) = parse_ring_chip_ask(content) {
            return Some(format_ring_chip_gateway(ask));
        }
    }
    // Free bytes before the SSD % chip ("how much disk is used" / bare "free space" stay %).
    if looks_like_disk_free_request(content) {
        return Some(format_disk_free_gateway());
    }
    // App clock before the Mac Up chip ("what's the uptime" stays system uptime).
    if looks_like_app_uptime_request(content) {
        return Some(format_app_uptime_gateway());
    }
    if looks_like_strip_chip_request(content) {
        if let Some(ask) = parse_strip_chip_ask(content) {
            return Some(format_strip_chip_gateway(ask));
        }
    }
    if looks_like_strip_request(content) {
        let filter = parse_strip_list_filter(content);
        return Some(format_strip_gateway(filter));
    }
    if looks_like_load_chip_request(content) {
        return Some(format_load_chip_gateway());
    }
    if let Some(window) = parse_load_window_chip(content) {
        return Some(format_load_window_chip_gateway(window));
    }
    if looks_like_details_request(content) {
        let filter = parse_details_list_filter(content);
        return Some(format_details_gateway(filter));
    }
    // One top-CPU / top-RAM / top-GPU process before the full Top Processes list.
    if looks_like_top_cpu_process_request(content) {
        return Some(format_top_cpu_process_gateway());
    }
    if looks_like_top_ram_process_request(content) {
        return Some(format_top_ram_process_gateway());
    }
    if looks_like_top_gpu_process_request(content) {
        return Some(format_top_gpu_process_gateway());
    }
    if looks_like_processes_request(content) {
        let filter = parse_processes_list_filter(content);
        return Some(format_processes_gateway(filter));
    }
    if looks_like_perplexity_request(content) {
        let filter = parse_perplexity_list_filter(content);
        return Some(format_perplexity_gateway(filter));
    }
    if looks_like_memory_scrub_request(content) {
        let (files, removed) = crate::commands::session_search::scrub_polluted_memory_files();
        return Some(if removed == 0 {
            "Memory scrub: nothing polluted to remove.".to_string()
        } else {
            format!(
                "Memory scrub: removed **{}** polluted line(s) from **{}** file(s).",
                removed, files
            )
        });
    }
    if let Some(reply) = try_digest_instant_reply(content) {
        return Some(reply);
    }
    None
}

