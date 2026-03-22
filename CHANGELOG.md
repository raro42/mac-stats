# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **FETCH_URL truncation contract test (022 §F7)** — `truncate_fetch_body_ellipse_then_explicit_suffix_for_llm` documents that oversized bodies use `ellipse()` (middle `...`) plus the explicit ` [content truncated]` suffix inside `MAX_BODY_CHARS`. `022_feature_review_plan.md` F7 checklist row closed for the automated part. No runtime behaviour change. (`commands/browser.rs`; FEAT-D32.)
- **Context overflow user messaging tests (022 §F2)** — `sanitize_ollama_error_for_user` unit tests for additional Ollama-style strings (`maximum context length … tokens`, `context length exceeded`, `exceeds the model's context window`) alongside existing overflow cases. No runtime behaviour change. (`commands/content_reduction.rs`; FEAT-D28.)
- **`run_due_monitor_checks` lib.rs wiring test (022 §F10)** — `lib_rs_invokes_run_due_monitor_checks_in_background_loop` in `commands/monitors.rs` fails if the 30s background monitor loop stops calling `run_due_monitor_checks()`. (`commands/monitors.rs`; FEAT-D26.)
- **`prepare_conversation_history` integration tests (022 §F1)** — Below `COMPACTION_THRESHOLD`, a new topic clears prior turns; the normal path runs `annotate_discord_401` so assistant text that matches the Discord-401 confusion heuristic gets the FETCH_URL vs `DISCORD_API` system correction appended. (`commands/session_history.rs`; FEAT-D24.)
- **022 §F1 / §F7 contract tests** — `cap_tail_keeps_last_n_in_chronological_order` now drives length and expected tail from `CONVERSATION_HISTORY_CAP` (router + Discord share the same cap). `ellipse_max_len_two_clamped` asserts `max_len` below `sep_len + 1` clamps for long strings. No runtime behaviour change. (`commands/session_history.rs`, `logging/mod.rs`; FEAT-D23.)
- **`shorten_file_path_internal` unit tests (022 §F7)** — Five tests lock debug-log location shortening: `src-tauri/src/…` and `src/…` prefix strip, fallback to the last `src/` segment for absolute-ish paths, passthrough when no match, empty input. Same helper used by `debug!` / `debug1!` macros via `file!()`. No runtime behaviour change. (`logging/legacy.rs`; FEAT-D22.)
- **`looks_like_discord_401_confusion` unit tests** — Four tests document when assistant text is treated as FETCH_URL-vs-Discord API confusion (401/unauthorized + token/credential/auth + discord/guild/channel) before `annotate_discord_401` appends the system correction in `prepare_conversation_history`. No runtime behaviour change. (`commands/reply_helpers.rs`; FEAT-D21.)
- **Discord history caps in `session_history` (022 §F1)** — `HAVING_FUN_IDLE_HISTORY_CAP` (10) for having_fun idle thoughts; normal having_fun replies use `CONVERSATION_HISTORY_CAP` (20) instead of a duplicate literal so the agent router and Discord stay aligned. Contract test in `session_history.rs`. (`discord/mod.rs`, `commands/session_history.rs`; FEAT-D19.)
- **Background monitor due-ness tests (022 §F10)** — `is_monitor_due_for_background()` in `commands/monitors.rs` centralizes the `last_check` + clamped interval predicate used by `run_due_monitor_checks`; five unit tests cover due / not-due, default 60s when config is absent, and interval `0` clamped to 1. Caller unchanged: `lib.rs` wakes every 30s. (FEAT-D18.)
- **`extract_first_url` tests (FETCH_URL)** — Unit tests for first-token URL extraction, trailing punctuation strip, and `https://` vs `http://` search order (later `https` wins over earlier `http`). (`commands/browser.rs`; FEAT-D17.)
- **Agent soul vs shared soul (022 §F3)** — `agent_soul_or_shared()` in `agents/mod.rs` implements per-agent `soul.md` vs shared `agents/soul.md` selection; three unit tests document no double soul when per-agent text is present and fallback when it is empty or absent. (`agents/mod.rs`; FEAT-D15.)
- **`MAC_STATS_SESSION_DIR`** — Optional override for the persisted session directory (defaults unchanged: `$HOME/.mac-stats/session/`). Used by `session_memory` unit tests for `load_messages_from_latest_session_file` without touching the real home layout; same pattern as `MAC_STATS_TASK_DIR`. Three tests: new filename layout, legacy layout, newest mtime wins when two files match the session id. (`config/mod.rs`, `session_memory.rs`; FEAT-D14, 022 §3 F1 disk resume.)
- **Router soul prefix + F4 tests (022 §F4)** — `format_router_soul_block()` in `ollama_memory.rs` centralizes shared-soul + app identity lines for agent-router planning; three unit tests cover empty/non-empty soul text and empty prefix when `skill_content` is present (same branch as agent `combined_prompt`). Call site unchanged aside from helper. (`commands/ollama_memory.rs`, `commands/ollama.rs`; FEAT-D13.)
- **Chat verbosity ↔ legacy atomic (022 §F8)** — Unit tests assert `set_chat_verbosity` updates `logging::VERBOSITY` (same atomic as `-v`/`-vv`/`-vvv` and `ellipse`-gated request logs) and clamps levels above 3. Mutex-serialized with restore. (`commands/logging.rs`)
- **TASK prompt contract tests (022 §F6)** — `format_task_agent_description()` holds the **TASK** tool paragraph for the dynamic agent list; three unit tests assert orchestrator-vs-TASK_CREATE guidance and duplicate-task → TASK_APPEND/TASK_STATUS wording. (`commands/agent_descriptions.rs`)
- **`MAC_STATS_TASK_DIR`** — Optional override for the task file directory (defaults unchanged: `$HOME/.mac-stats/task/`). Used by unit tests; also available for isolated runs. (`config/mod.rs`)
- **TASK_CREATE deduplication tests** — `test_slug_deterministic` and `create_task_duplicate_topic_id_errors_with_task_append_hint` in `task/mod.rs` document 022 §3 F5 (slug stability, duplicate topic+id error mentions `TASK_APPEND` / alternate id). (`task/mod.rs`)

### Changed
- **HTML cleaning: Arabic Extended-A U+0890–U+0891 (FETCH_URL)** — `collapse_whitespace()` maps Arabic pound mark above and piastre mark above (Cf) to ASCII space; they are not Rust whitespace, so mixed Arabic financial or RTL HTML can otherwise glue Latin tokens. `arabic_extended_a_currency_format_marks_separate_words` in `commands/html_cleaning.rs`. (FEAT-D49.)
- **HTML cleaning: Ethiopic wordspace, Braille blank, Duployan format (FETCH_URL)** — `collapse_whitespace()` maps U+1361 (Ethiopic wordspace, Po), U+2800 (Braille pattern blank, So), and U+1BCA0–U+1BCA3 (Duployan shorthand format overlap/step, Cf) to ASCII space; none are Rust whitespace, so mixed-script or pasted text could otherwise glue Latin tokens. Tests: `ethiopic_wordspace_separates_words`, `braille_pattern_blank_separates_words`, `duployan_shorthand_format_separates_words` in `commands/html_cleaning.rs`. (FEAT-D48.)
- **HTML cleaning: `<img>` alt / title (FETCH_URL)** — Non-empty `alt` (trimmed) is emitted as inline `[Image: …]`; if `alt` is empty, non-empty `title` is used. Decorative images with neither stay silent (no placeholder). Addresses 022 §HTML noise review observation (images previously dropped with no LLM-visible description). (`commands/html_cleaning.rs`; FEAT-D47.)
- **HTML cleaning: Mongolian U+1806 / U+180A (FETCH_URL)** — `collapse_whitespace()` maps U+1806 (Mongolian TODO SOFT HYPHEN) and U+180A (NIRUGU) to ASCII space; both are Cf and not Rust whitespace, and they fall outside the existing U+180B–U+180E (FVS / vowel separator) range. `invisible_fillers_separate_words` in `commands/html_cleaning.rs`. (FEAT-D46.)
- **HTML cleaning: Arabic / Syriac edition format controls (FETCH_URL)** — `collapse_whitespace()` maps U+0600–U+0605 (Arabic number sign and related edition marks), U+06DD / U+08E2 (end of ayah), and U+070F (Syriac abbreviation mark) to ASCII space; they are Cf and not Rust whitespace, so copied RTL or scholarly HTML can otherwise glue Latin tokens. `arabic_and_syriac_edition_format_separate_words` in `commands/html_cleaning.rs`. (FEAT-D45.)
- **HTML cleaning: Khmer inherent vowels U+17B4 / U+17B5 (FETCH_URL)** — `collapse_whitespace()` maps Khmer vowel inherent AQ and AA (Cf) to ASCII space with other invisible format controls; Rust `split_whitespace()` does not treat them as whitespace, so Khmer-layout or mixed-script HTML can otherwise glue Latin tokens. `khmer_inherent_vowel_format_separates_words` in `commands/html_cleaning.rs`. (FEAT-D44.)
- **HTML cleaning: Unicode Tags block U+E0000–U+E007F (FETCH_URL)** — `collapse_whitespace()` maps deprecated language-tag / tag-id characters (Tags block) to ASCII space with other format controls; Rust `split_whitespace()` does not treat them as whitespace, so rare pasted text could otherwise glue Latin tokens. `unicode_language_tag_characters_separate_words` in `commands/html_cleaning.rs`. (FEAT-D43.)
- **HTML cleaning: variation selectors + Mongolian VS (FETCH_URL)** — `collapse_whitespace()` maps U+FE00–U+FE0F and U+E0100–U+E01EF (emoji / IVS presentation controls) and U+180B–U+180E (Mongolian free variation selectors + vowel separator) to ASCII space with existing invisible controls; Rust `split_whitespace()` does not treat these code points as whitespace. `variation_selectors_separate_words` and `invisible_fillers_separate_words` in `commands/html_cleaning.rs`. (FEAT-D42.)
- **HTML cleaning: U+202A–U+202E bidi embedding / override (FETCH_URL)** — `collapse_whitespace()` maps U+202A–U+202E (LRE/RLE/PDF/LRO/RLO) to ASCII space with existing bidi format controls; Rust `split_whitespace()` does not treat these Cf code points as whitespace. `bidi_embedding_and_override_separate_words` in `commands/html_cleaning.rs`. (FEAT-D41.)
- **HTML cleaning: U+2061 + interlinear / object replacement (FETCH_URL)** — `collapse_whitespace()` maps U+2061 (invisible function application, MathML) by extending the existing U+2062–U+206F range to U+2061..=U+206F, and maps U+FFF9–U+FFFC (interlinear annotation markers + object replacement character) to ASCII space; Rust `split_whitespace()` does not treat these Cf code points as whitespace. `invisible_math_and_bidi_format_separate_words` / `interlinear_annotation_and_object_replacement_separate_words` in `commands/html_cleaning.rs`. (FEAT-D40.)
- **HTML cleaning: Mongolian / Hangul invisible fillers (FETCH_URL)** — `collapse_whitespace()` maps U+180E (Mongolian vowel separator), U+115F/U+1160 (Hangul choseong/jungseong fillers), U+3164 (Hangul filler), and U+FFA0 (halfwidth Hangul filler) to ASCII space with existing ZWSP/bidi controls; Rust `split_whitespace()` does not treat these code points as whitespace. `invisible_fillers_separate_words` in `commands/html_cleaning.rs`. (FEAT-D39.)
- **HTML cleaning: U+2062–U+206F invisible math + bidi/shaping (FETCH_URL)** — `collapse_whitespace()` maps the full block (invisible times/separator/plus through nominal digit shapes, including directional isolates U+2066–U+2069) to ASCII space with existing ZWSP/NBSP/bidi controls so MathML- or RTL-heavy pages do not glue tokens. `invisible_math_and_bidi_format_separate_words` exercises U+2062..=U+206F. (`commands/html_cleaning.rs`; FEAT-D38.)
- **HTML cleaning: bidi / CGJ / isolates (FETCH_URL)** — `collapse_whitespace()` maps U+034F (combining grapheme joiner), U+061C (Arabic letter mark), U+200E/U+200F (LRM/RLM), and U+2066–U+2069 (directional isolates) to a normal space before tokenizing, same family as ZWSP/ZWNJ: Rust’s `split_whitespace()` does not treat those code points as whitespace, so without mapping, `hello\u{200E}world` (and similar) would stay one token. (`commands/html_cleaning.rs`; `bidi_and_grapheme_joiner_separate_words` test; FEAT-D37.)
- **HTML cleaning: ZWNJ / ZWJ / word joiner (FETCH_URL)** — `collapse_whitespace()` maps U+200C (ZWNJ), U+200D (ZWJ), and U+2060 (word joiner) to a normal space before tokenizing, same family as ZWSP/NBSP: Rust’s `split_whitespace()` does not treat those code points as whitespace, so without mapping, `hello\u{200C}world` (and similar) would stay one token. (`commands/html_cleaning.rs`; `zero_width_joiners_separate_words` test; FEAT-D36.)
- **HTML cleaning: NBSP word breaks (FETCH_URL)** — `collapse_whitespace()` maps U+00A0 (no-break space, HTML `&nbsp;`) to a normal space before tokenizing, same family as ZWSP/SHY: Rust’s `split_whitespace()` does not treat NBSP as whitespace, so without mapping, `hello\u{00A0}world` would stay one token. (`commands/html_cleaning.rs`; `nbsp_separates_words` test; FEAT-D35.)
- **HTML cleaning: soft hyphen word breaks (FETCH_URL)** — `collapse_whitespace()` maps U+00AD (soft hyphen, common from HTML `&shy;`) to a normal space before tokenizing, same family as ZWSP/BOM: Rust’s `split_whitespace()` does not treat SHY as whitespace, so without mapping, `hello\u{00AD}world` would stay one token. (`commands/html_cleaning.rs`; `soft_hyphen_separates_words` test; FEAT-D34.)
- **HTML cleaning: zero-width word breaks (FETCH_URL)** — `collapse_whitespace()` maps U+200B (ZWSP) and U+FEFF (BOM as ZWNBSP when it appears in text) to a normal space before tokenizing, so glued tokens like `hello\u{200B}world` become `hello world` for the model. Rust’s `split_whitespace()` does not treat those code points as whitespace. (`commands/html_cleaning.rs`; `zero_width_space_separates_words` test; FEAT-D33.)
- **`load_agents` shared-soul log (022 §F3)** — `info!` line for shared `soul.md` presence now runs whenever `load_agents` finishes a scan: agents dir missing, no enabled agents, or successful load (previously only when at least one agent was enabled). Pure helper `shared_soul_file_nonempty()` + `log_shared_soul_presence()`; three HOME-override unit tests. (`agents/mod.rs`; FEAT-D27.)
- **CPU window chat: reserved words (022 §F8)** — `--cpu` and `-v`/`-vv`/`-vvv` are handled before any user chat bubble is added, so meta-commands stay out of the visible transcript as well as conversation history. Input is cleared immediately; only the assistant status line is shown. (`src/ollama.js`, synced to `src-tauri/dist/ollama.js`; FEAT-D16.)
- **Agent-router execution messages (022 §F2)** — Both execution paths in `answer_with_ollama_and_fetch` now use `build_execution_message_stack()` (`session_history.rs`) for system → history → current user; two unit tests lock the ordering contract. No change to wire format or model payloads. (`commands/session_history.rs`, `commands/ollama.rs`)
- **TASK agent paragraph helper** — **TASK** block in `build_agent_descriptions` moved to `format_task_agent_description()` for unit testing; wording unchanged. (`commands/agent_descriptions.rs`)

### Documentation
- **FEATURE-CODER backlog (FEAT-D41–D49)** — Table rows for completed FETCH_URL `clean_html` work: U+202A–U+202E bidi embedding/override, variation selectors (U+FE00–U+FE0F, U+E0100–U+E01EF) plus Mongolian U+1806/U+180A/U+180B–U+180E, Unicode Tags U+E0000–U+E007F, Khmer inherent vowels U+17B4/U+17B5, Arabic/Syriac edition format controls U+0600–U+0605 / U+06DD / U+070F / U+08E2, FEAT-D47 (`<img>` alt/title → `[Image: …]`), FEAT-D48 (U+1361 / U+2800 / U+1BCA0–U+1BCA3), and FEAT-D49 (Arabic Extended-A U+0890–U+0891). (`006-feature-coder/FEATURE-CODER.md`.)
- **`toggle_cpu_window` from CPU window chat (022 §F8)** — Rustdoc clarifies that reserved word `--cpu` from inside the CPU window uses the same close-then-recreate path as the menu bar (WebView destroyed and recreated, not an in-place hide). `docs/022_feature_review_plan.md` F8 checklist item closed. (`commands/window.rs`, `ui/status_bar.rs`; FEAT-D31.)
- **`toggle_cpu_window` + `run_on_main_thread` (022 §F9)** — Rustdoc on `commands/window.rs` explains Tauri 1 threading: the command runs off the AppKit main thread, `run_on_main_thread` blocks the command thread until a short main-thread window close/create completes; checklist item in `docs/022_feature_review_plan.md` F9 marked complete. No runtime behaviour change. (FEAT-D30.)
- **`toggle_cpu_window` API docs (022 §F9)** — Rustdoc on `commands/window.rs` and `ui/status_bar.rs` matches implementation: existing CPU window is closed then recreated so the user always ends with a visible window; clarifies this is not a strict “close and stay closed” toggle. (FEAT-D29.)
- **022 §9 closing review snapshot** — Integration code-review bullets for F1 (session file legacy layout) and F7 (`ellipse` small-`max_len` clamp) updated to match current behaviour and tests.
- **README product overview** — New **What mac-stats ships with** section up front (menu bar metrics, Ollama toolbelt, Discord/tasks/scheduler/monitors, privacy/ops); trimmed duplicate “At a glance” bullets; cross-link to `docs/006_roadmap_ai_tasks.md`. **`docs/README.md`** Global Context now defers to root README; **`docs/006_roadmap_ai_tasks.md`** title clarifies roadmap vs product overview.
- **022 feature review checklist** — Per-section review items in `docs/022_feature_review_plan.md` marked complete where automated tests or code paths already cover them (F1–F4, F6, F7, F8, F10); integration table updated for session file compat and `run_due_monitor_checks` caller. Manual Discord/smoke rows unchanged. (FEAT-D26.)
- **Autoresearch snapshot plotter** — `scripts/plot_autoresearch_snapshot.py` reads mac-stats-reviewer Track A `results.tsv` and writes PNG/SVG (optional `state.json` subtitle). Generate outputs locally when needed; large samples are not kept in-tree.
- **022 review & FEATURE-CODER backlog** — F5 checklist items marked complete with test pointers; new FEAT-D9–D11 rows in the coder backlog. (`docs/022_feature_review_plan.md`, `006-feature-coder/FEATURE-CODER.md`)

### Removed
- **Autoresearch sample artifacts** — Dropped checked-in `docs/autoresearch-snapshots/amvara8-005-openclaw-24h_*` PNG/SVG/TSV/state JSON (~2.4k lines + binary) to reduce repo weight; recreate with `scripts/plot_autoresearch_snapshot.py` if you want the same plots.

### Fixed
- **Clippy `assertions_on_constants` (`prepare_conversation_history` tests)** — The below-threshold tokio tests no longer use runtime `assert!` on `COMPACTION_THRESHOLD`; a single `const` assert (`COMPACTION_THRESHOLD > 2`) in the test module documents the assumption (FEAT-D25). No runtime behaviour change. (`commands/session_history.rs`)
- **Clippy `assertions_on_constants` (session history caps)** — Ordering `HAVING_FUN_IDLE_HISTORY_CAP` < `CONVERSATION_HISTORY_CAP` is now a module-level `const` assert; the unit test only checks the literal values (20 / 10). (`commands/session_history.rs`; FEAT-D20.)
- **Monitor stats persistence after each check** — `check_monitor` now calls `save_monitors()` after updating `last_check` / `last_status`, so background `run_due_monitor_checks` and manual checks write `monitors.json` instead of keeping stats only in memory until add/remove. The monitors mutex is released before save to avoid deadlock with `save_monitors`’ lock order. Failures from `save_monitors` (e.g. busy `try_lock`) are ignored; in-memory stats remain authoritative for the running process. (`commands/monitors.rs`; FEAT-E1, 022 §3 F10.)

## [0.1.54] - 2026-03-22

### Added
- **`cap_tail_chronological`** — Shared “last N items, chronological order” helper for conversation caps (agent router + Discord having_fun); unit tests lock the `rev().take(N).rev()` contract. Session memory test documents loading prior turns before recording the current user message (Discord pipeline). (`commands/session_history.rs`, `commands/ollama.rs`, `discord/mod.rs`, `session_memory.rs`)

### Changed
- **FETCH_URL oversized body** — When the response exceeds the char cap, the ellipsed text now ends with ` [content truncated]` (after reserving suffix length inside the cap) so the model explicitly knows content was cut, not only middle `...`. (`commands/browser.rs`)
- **`logging/subsystem.rs` test placement** — `#[cfg(test)] mod tests` now follows the exported `mac_stats_*` macros so `cargo clippy` stays warning-free (`items_after_test_module`). (`logging/subsystem.rs`)
- **Tracing targets** — Broad use of `mac_stats_*!` subsystem macros across agent commands, Discord, browser agent, MCP, plugins, scheduler, alerts, and related modules so logs align with `MAC_STATS_LOG` filtering.

### Fixed
- **Session resume: `## ` inside message body** — Persisted session markdown is parsed line-by-line; only lines that trim to exactly `## User` or `## Assistant` start a new turn. In-message text such as `## Notes` no longer splits the file incorrectly (previously `\n## ` truncation could drop content). Four unit tests. (`session_memory.rs`)
- **Website monitor `check_interval_secs = 0`** — Interval is clamped to at least 1 second when loading from disk, adding a monitor, and when deciding if a monitor is due in `run_due_monitor_checks`. Zero previously made `elapsed >= interval` true even with zero elapsed seconds after a check, so a monitor could be treated as due on every pass. (`commands/monitors.rs`)
- **SSRF redirect validation** — HTTP redirect targets are checked with the same rigor as the initial URL: DNS resolution failure or an empty address list stops the redirect instead of following blindly (extracted `check_redirect_target_ssrf`, used by `ssrf_redirect_policy`). Four new unit tests. (`commands/browser.rs`)
- **SSRF IPv4-mapped broadcast** — IPv6 addresses that embed IPv4 (e.g. `::ffff:255.255.255.255`) now treat mapped IPv4 broadcast the same as native IPv4 (`is_broadcast()` in the `to_ipv4_mapped()` guard). Unit test added. (`commands/browser.rs`)
- **Session resume after restart (legacy filenames)** — `load_messages_from_latest_session_file` now picks up both the current layout `session-memory-{id}-{timestamp}-{topic}.md` and the older `session-memory-{topic}-{id}-{timestamp}.md` files, so Discord/session context can load after upgrading from pre-reorder naming. Filename matching uses explicit new/legacy patterns (with unit tests) instead of a simple `session-memory-{id}-` prefix. (`session_memory.rs`)

## [0.1.53] - 2026-03-22

### Added
- **Discord draft message while tools run** — Full agent router posts a placeholder (“Processing…”), then throttled in-place edits (e.g. `Running FETCH_URL…`) until the final reply; the first chunk replaces the placeholder, extra chunks use new messages as before. Config `discord_draft_throttle_ms` or env `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS` (default 1500 ms, clamped 200–60_000). If the placeholder send fails, falls back to reply-only mode. (`discord/mod.rs`, `commands/discord_draft_stream.rs`, `commands/tool_loop.rs`, `commands/ollama.rs`, `config/mod.rs`, `docs/007_discord_agent.md`)
- **`MAC_STATS_LOG` stderr filter** — Comma-separated subsystem allowlist (e.g. `browser,ollama`); console shows only events whose `tracing` target is `mac_stats::<name>` or a child path. File log (`~/.mac-stats/debug.log`) is unchanged. (`logging/subsystem.rs`, `logging/mod.rs`, `docs/039_mac_stats_log_subsystems.md`)
- **Subsystem `tracing` targets** — Ollama HTTP client, agent-router chat logs, and browser agent (CDP + HTTP fallback) use `mac_stats_*!` macros so they participate in `MAC_STATS_LOG`. (`ollama/mod.rs`, `commands/ollama.rs`, `browser_agent/mod.rs`, `browser_agent/http_fallback.rs`)

### Changed
- **MCP stdio errors** — Tool failures append short troubleshooting hints (PATH, timeouts, init/tools errors) pointing to `docs/038_ori_mnemos_mcp.md`; “not configured” mentions `MCP_SERVER_STDIO`. (`misc_tool_dispatch.rs`)
- **Docs** — Ori Mnemos MCP setup (`docs/038_ori_mnemos_mcp.md`); log subsystems reference (`docs/039_mac_stats_log_subsystems.md`); MCP agent doc, agent index/workflow, `README.md`, and `agents.md` cross-links.

## [0.1.52] - 2026-03-22

### Added
- **DISCORD_API pre-routing** — Discord API requests now skip the LLM planning step when the user's intent is unambiguous and a Discord bot token is configured. Explicit prefix (`DISCORD_API: <path>`) always pre-routes. Keyword patterns: "list servers" / "show servers" / "my servers" / "what servers am i in" → `DISCORD_API: GET /users/@me/guilds` (direct API call, no guild context needed). "list channels" / "show channels" / "list channels in …" → `AGENT: discord-expert` (needs guild discovery). "list members" / "show members" / "who's in …" → `AGENT: discord-expert` (needs guild context). Multi-step requests ("and then …", "after that …") are excluded and go through normal LLM planning. 22 new tests; 292 total pass, zero clippy warnings. (`commands/pre_routing.rs`)
- **Context-overflow auto-recovery** — When the Ollama API returns a context-window overflow error during the agent router or tool loop, oversized tool-result messages are automatically truncated and the request is retried instead of failing immediately. `is_context_overflow_error()` detects overflow patterns; `truncate_oversized_tool_results()` cuts messages exceeding the configured limit at word boundaries with a `[truncated]` marker. Applied in both `answer_with_ollama_and_fetch` (first execution call) and `run_tool_loop` (follow-up calls). If retry also fails, returns a clear message suggesting a new topic or larger context model. 12 new tests (truncation, overflow detection, skip system prompt, multiple results). (`commands/content_reduction.rs`, `commands/ollama.rs`, `commands/tool_loop.rs`)
- **Context-overflow config options** — `contextOverflowTruncateEnabled` (bool, default true) controls whether overflow recovery is attempted; `contextOverflowMaxResultChars` (number, default 4096) sets the per-message truncation limit. Both configurable via config.json or env vars (`MAC_STATS_CTX_OVERFLOW_TRUNCATE`, `MAC_STATS_CTX_OVERFLOW_MAX_RESULT_CHARS`). (`config/mod.rs`)
- **Discord full-router message debounce** — Rapid full-agent-router messages in the same channel merge into one Ollama run after a quiet period (default **2000 ms** via `discord_debounce_ms` in `~/.mac-stats/config.json`, env `MAC_STATS_DISCORD_DEBOUNCE_MS`, clamped 0–60s). Per-channel overrides in `discord_channels.json`: `debounce_ms` or `immediate_ollama: true`. Bypass: attachments, `/` commands, session-reset phrases, vision payloads; having_fun unchanged. Queued batches discarded on app disconnect (logged). Refactor: `run_discord_ollama_router` extracted. (`discord/message_debounce.rs`, `discord/mod.rs`, `config/mod.rs`, `defaults/discord_channels.json`, `docs/007_discord_agent.md`)

### Changed
- **Refactor `truncate_at_boundary`** — Replaced manual counter with `enumerate()` and a `broke_early` flag for clarity. No behavioral change. (`commands/content_reduction.rs`)
- **Compaction context cap** — The CONTEXT section produced by the session compaction LLM is now hard-capped at 12 000 bytes (~3 000 tokens) to prevent a verbose summary from consuming too much of the 32 K–40 K context window. Truncation cuts at the last sentence boundary (`.` / `!` / `?`) within budget, appends `[summary truncated]`, and is logged at warn level. Uses `floor_char_boundary` for safe UTF-8 handling. 7 new `cap_context` tests (short, exact, sentence boundary, no boundary, preserves punctuation, multibyte UTF-8, emoji) and 6 new `parse_compaction_output` tests (basic, no lessons, lessons-none, no headers, mixed-case, empty). 13 new tests; compaction module fully covered. (`commands/compaction.rs`)
- **Planning history cap** — The LLM planning step now receives only the last N conversation messages (default 6, ~3 user/assistant turns) instead of the full session history (up to 20). This reduces noise from past tool outputs that can bias the planner toward AGENT when a direct tool would suffice. Configurable via `planningHistoryCap` in config.json or env `MAC_STATS_PLANNING_HISTORY_CAP` (0 disables planning history entirely; max 40). The full history is still sent to the execution step. When capping is applied, logged at info level. (`commands/ollama.rs`, `config/mod.rs`)
- **Unit tests for `ellipse()` utility** — 12 new tests for the `ellipse()` string truncation function (widely used in 30+ call sites for log/status messages): short/exact/long strings, Unicode support, edge cases (empty, zero/one/three max_len), result length invariant, odd/even splits. (`logging/mod.rs`)
- **Unit tests for `merge_prompt_content` and `paragraph_key`** — 12 new tests for the prompt merge logic used at startup to add new default paragraphs to user-customized prompts without overwriting edits: empty existing, identical content, missing block appended, user edits preserved, no duplicates, exact first-line key matching, whitespace tolerance, planning_history_cap default. (`config/mod.rs`)
- **HTML noise stripping for FETCH_URL** — Fetched page content is now cleaned before being sent to the LLM: `script`, `style`, `head`, `meta`, `link`, `noscript`, `svg`, `iframe`, `object`, and `embed` tags are stripped entirely. Semantic structure is preserved: headings as `# …`, links as `[text](href)`, list items as `- …`, table rows as pipe-separated values, and block elements as newlines. Typical pages shrink 40–70%, saving context tokens and often avoiding the summarization Ollama round-trip. When cleaning produces empty output (all-JS pages), a helpful message suggests BROWSER_NAVIGATE instead. Compression ratio logged at info level. Applied to FETCH_URL in Discord/agent tool loop and CPU-window chat; HTTP fallback (which needs raw HTML for its own parser) is unaffected. 11 new tests; 221 total pass, zero clippy warnings. (`commands/html_cleaning.rs`, `commands/network_tool_dispatch.rs`, `commands/ollama_frontend_chat.rs`)
- **Scheduler per-task wall-clock timeout** — Each scheduler task is now wrapped in `tokio::time::timeout` to prevent a single stuck task from blocking the scheduler loop indefinitely. Default 600 s (10 min), configurable via `schedulerTaskTimeoutSecs` in config.json or env `MAC_STATS_SCHEDULER_TASK_TIMEOUT_SECS` (clamped 30–3600). Timed-out tasks are logged at error level and reported as failures. (`config/mod.rs`, `scheduler/mod.rs`)
- **Management command pre-routing** — LIST_SCHEDULES, TASK_LIST, TASK_SHOW, and OLLAMA_API management commands now skip the LLM planning step when the user's intent is unambiguous. Explicit prefixes (`LIST_SCHEDULES:`, `TASK_LIST:`, `TASK_SHOW: <id>`, `OLLAMA_API: <action>`) always pre-route. Keyword patterns: "list schedules" / "show schedules" / "what's scheduled" → LIST_SCHEDULES; "list tasks" / "show tasks" / "tasks" / "open tasks" / "all tasks" → TASK_LIST; "show task <id>" → TASK_SHOW; "list models" / "show models" / "what models are installed" / "available models" → OLLAMA_API: list_models; "pull model <name>" → OLLAMA_API: pull; "unload model <name>" → OLLAMA_API: unload; "running models" / "what models are running" → OLLAMA_API: running. Multi-step requests ("and then …", "after that …") are excluded and go through normal LLM planning. 33 new tests; 210 total pass, zero clippy warnings. (`commands/pre_routing.rs`)
- **BRAVE_SEARCH / PERPLEXITY_SEARCH pre-routing** — Requests with clear web search intent now skip the LLM planning step and route directly to the configured search tool. Explicit prefixes (`BRAVE_SEARCH: <query>`, `PERPLEXITY_SEARCH: <query>`) always pre-route. Keyword patterns ("search for …", "google …", "look up …", "web search …", "search the web for …", "search online for …", "research …") detect search intent and route to BRAVE_SEARCH (default) or PERPLEXITY_SEARCH ("research …" prefers Perplexity when configured; falls back to Brave). Multi-step requests ("and then …", "send to …", browser actions) are excluded and go through normal LLM planning. Only routes when the respective API key is configured. 21 new tests for `extract_search_query`; 177 total pass, zero clippy warnings. (`commands/pre_routing.rs`)
- **User-friendly Ollama API errors** — Known failures (context overflow, message role ordering, corrupted tool history) map to short actionable text via `sanitize_ollama_error_for_user()`; applied when surfacing errors from `answer_with_ollama_and_fetch`, CPU/chat `ollama_chat_with_execution` / continue, and `run_tool_loop`. Unknown errors keep prior formatting. New unit tests. (`commands/content_reduction.rs`, `commands/ollama.rs`, `commands/ollama_frontend_chat.rs`, `commands/tool_loop.rs`)
- **Docs: OpenClaw §96 re-verification** — All §7 checks re-run against OpenClaw AGENTS.md (23.9 KB, last modified 2026-03-21) vs code (`005-openclaw-reviewer`). Persistent findings from §95 confirmed still present: (1) `src/provider-web.ts` does not exist (actual: `src/channel-web.ts`); (2) `src/telegram`, `src/discord`, `src/slack`, `src/signal`, `src/imessage`, `src/web` do not exist as top-level dirs — channel runtimes live under `src/plugins/runtime/` and `src/channels/`, extensions under `extensions/`; (3) `pnpm format` runs `oxfmt --write` not `--check` (format check is `pnpm format:check`); (4) Vitest branch coverage threshold is 55%, not 70% as AGENTS.md line 109 claims (lines/functions/statements are 70%). New findings: (5) `pnpm tsgo` has no `scripts` entry in `package.json` — it relies on `@typescript/native-preview` (v7.0.0-dev.20260317.1) providing a `tsgo` binary via `.bin`; `pnpm check` calls `pnpm tsgo` in its chain, which works after `pnpm install` but is not a declared script; AGENTS.md line 69 documents `pnpm tsgo` as a standalone command. (6) AGENTS.md line 71 says `pnpm format` is "oxfmt --check" but actual `pnpm format` is `oxfmt --write`; the check command is `pnpm format:check`. (7) 82 extension dirs under `extensions/` (was 80 at §95); `anthropic-vertex`, `chutes`, `fal` still lack dedicated English provider pages; `phone-control` and `thread-ownership` still only in zh-CN plugin list. (8) SSRF test coverage in OpenClaw has grown significantly: 54 `it()` cases in 7 dedicated `*ssrf*.test.ts` files, plus ~14 SSRF-labeled tests across 11 other files (~68 total). (9) Recent commits: Discord routed through plugin SDK, plugin split runtime state, Claude bundle commands, context compaction notification, Matrix agentId mention fix, webchat image persistence, GitHub Copilot dynamic model IDs. No code bugs found; discrepancies are documentation-only.

## [0.1.51] - 2026-03-21

### Added
- **FETCH_URL pre-routing** — Requests with clear fetch/read intent and a URL (e.g. "fetch https://example.com", "get the page at https://…", "summarize this url https://…", or explicit "FETCH_URL: <url>") now skip the LLM planning step and route directly to `FETCH_URL`. Browser/navigate/screenshot patterns are excluded (handled by existing pre-routes). 17 new tests; 156 total pass, zero clippy warnings. (`commands/pre_routing.rs`)
- **General tool loop guard** — `ToolLoopGuard` in `commands/loop_guard.rs` detects repeated tool invocations and cycles within a single agent-router request. Tracks all (tool, arg) calls: blocks the same exact call after 3 invocations and detects repeating patterns of length 2–4 (e.g. A→B→A→B). Complements existing per-tool dedup (browser, Discord API, RUN_CMD) with cross-tool cycle detection. Blocked calls return an instructive message to the model ("reply with DONE or try a different approach"). 10 new tests; 139 total pass, zero clippy warnings. (`commands/loop_guard.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Auto-dismiss JS dialogs in CDP browser agent** — Registered a `PageJavascriptDialogOpening` event listener that automatically dismisses `alert`, `confirm`, `prompt`, and `beforeunload` dialogs on CDP tabs. Prevents the browser agent from hanging when a page triggers a JS dialog. Handler is idempotent (tracked per tab pointer in a global `HashSet`), clears on browser session reset. Applied on `get_current_tab`, new-tab navigation, and screenshot flows. (`browser_agent/mod.rs`)
- **`scripts/screenshot-url.sh`** — Standalone headless Chrome screenshot utility for quick URL captures outside the app. Saves PNGs to `~/.mac-stats/screenshots/`.
- **Discord 429 rate-limit handling** — All Discord HTTP calls (`discord_api_request`, `send_message_to_channel`, `send_message_to_channel_with_attachments`) now honour 429 responses: parse `Retry-After` from header or JSON body, wait that duration plus pseudo-random jitter (100–499 ms), and retry up to 3 times. Each 429 is logged at warn level. Shared helpers `wait_for_rate_limit`, `retry_after_from_headers`, `parse_retry_after`, `jitter_millis` in `discord/api.rs`; `discord_api_request` refactored from fixed-iteration loop to an open loop with separate connection-retry and rate-limit-retry counters. (`discord/api.rs`, `discord/mod.rs`, `docs/007_discord_agent.md`)

### Changed
- **`OllamaRequest` struct replaces 24 positional parameters** — `answer_with_ollama_and_fetch` now takes a single `OllamaRequest` struct instead of 24 positional arguments. All call sites (Discord, CLI `run-ollama`, scheduler, task runner, verification retry) updated to use struct initialization with `..Default::default()`, making each call self-documenting and new parameters trivial to add. `OllamaRequest` re-exported from `lib.rs`. No behavioral changes; zero clippy warnings, 139 tests pass. (`commands/ollama.rs`, `discord/mod.rs`, `main.rs`, `scheduler/mod.rs`, `task/runner.rs`, `lib.rs`)
- **Extract session history, prompt assembly, and verification retry hint from `ollama.rs`** — Moved session history preparation (cap, compaction, new-topic clearing, Discord-401 annotation) into `commands/session_history.rs` (143 lines). Consolidated duplicated execution system prompt assembly (skill-path and router-soul-path) into `commands/prompt_assembly.rs` (63 lines) via `build_execution_system_content()`. Moved 100-line verification retry hint builder (domain-specific retry prompts for Redmine, news, screenshots, browser, cookies, JSON format) into `verification.rs` as `build_verification_retry_hint()`. Removed 4 unused imports from `ollama.rs` (`append_to_file`, `looks_like_discord_401_confusion`, `browser_retry_grounding_prompt`, `is_browser_task_request`, `redmine_time_entries_range`). `ollama.rs` 1941→1671 lines (270 extracted). No behavioral changes; zero clippy warnings, 129 tests pass. (`commands/session_history.rs`, `commands/prompt_assembly.rs`, `commands/verification.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Extract network + delegation tool handlers from `ollama.rs`** — Moved 10 remaining inline tool handler match arms into two new modules: `commands/network_tool_dispatch.rs` (302 lines: FETCH_URL with discord.com redirect, BRAVE_SEARCH, DISCORD_API, REDMINE_API) and `commands/delegation_tool_dispatch.rs` (579 lines: AGENT with cursor-agent proxy and orchestrator loop-breaker, SKILL, RUN_JS, RUN_CMD with retry loop, PYTHON_SCRIPT). Each handler is a standalone function; `DiscordApiResult`, `AgentResult`, `RunCmdResult` structs communicate mutable state changes back to the caller. Removed 6 unused imports from `ollama.rs` (`reduce_fetched_content_to_fit`, `run_skill_ollama_session`, `run_js_via_node`, `CHARS_PER_TOKEN`, `parse_python_script_from_response`, `redmine_direct_fallback_hint`, `is_agent_unavailable_error`, `normalize_discord_api_path`). `ollama.rs` 2579→1941 lines (638 extracted). No behavioral changes; zero clippy warnings, 129 tests pass. (`commands/network_tool_dispatch.rs`, `commands/delegation_tool_dispatch.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Extract browser tool dispatch from `ollama.rs`** — Moved 8 browser tool handler match arms (BROWSER_SCREENSHOT, BROWSER_NAVIGATE, BROWSER_GO_BACK, BROWSER_CLICK, BROWSER_INPUT, BROWSER_SCROLL, BROWSER_EXTRACT, BROWSER_SEARCH_PAGE) into `commands/browser_tool_dispatch.rs` (422 lines). Each handler is an async function taking only its required parameters. `BrowserScreenshotResult` struct returns attachment paths. No behavioral changes. (`commands/browser_tool_dispatch.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Extract misc tool dispatch from `ollama.rs`** — Moved 5 tool handler match arms (OLLAMA_API, MCP, CURSOR_AGENT, MASTODON_POST, MEMORY_APPEND) into `commands/misc_tool_dispatch.rs` (346 lines). Removed unused imports from `ollama.rs` (`mastodon_post`, 8 ollama_models functions, 3 browser_helpers functions). `ollama.rs` 3145→2579 lines (566 extracted). No behavioral changes; zero clippy warnings, 129 tests pass. (`commands/misc_tool_dispatch.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Extract tool execution loop from `ollama.rs`** — Moved the ~600-line `while tool_count < max_tool_iterations` loop (tool parsing, dispatch, dedup guards, budget warnings, Redmine/DONE handling, follow-up Ollama calls) into `commands/tool_loop.rs` (748 lines). Exposes `ToolLoopParams` (immutable config), `ToolLoopState` (accumulated mutable state), `ToolLoopResult`, and `run_tool_loop()`. The orchestrator now calls `run_tool_loop()` and destructures the result. Removed unused imports from `ollama.rs` (`PathBuf`, `normalize_browser_tool_arg`, `truncate_search_query_arg`, `final_reply_from_tool_results`, `extract_redmine_time_entries_summary_for_reply`, `grounded_redmine_time_entries_failure_reply`, `question_explicitly_requests_json`). `ollama.rs` 1671→1064 lines (607 extracted). No behavioral changes; zero clippy warnings, code compiles. (`commands/tool_loop.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Fix clippy `unnecessary_map_or` warning in SSRF check** — `is_some_and` instead of `map_or(false, ...)` for IPv4-mapped IPv6 blocklist check. (`commands/browser.rs`)
- **Extract task and schedule tool handlers from `ollama.rs`** — Moved 10 tool handler match arms (TASK_APPEND, TASK_STATUS, TASK_CREATE, TASK_SHOW, TASK_ASSIGN, TASK_SLEEP, TASK_LIST, SCHEDULE, REMOVE_SCHEDULE, LIST_SCHEDULES) into `commands/task_tool_handlers.rs` (505 lines). Each handler is a standalone function taking only its required parameters. `ollama.rs` 3502→3145 lines (357 extracted). Removed unused `schedule_helpers` import from `ollama.rs`. No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/task_tool_handlers.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Extract pre-routing into `commands/pre_routing.rs`** — Moved deterministic pre-routing logic (screenshot→BROWSER_SCREENSHOT, "run …"→RUN_CMD, ticket→REDMINE_API) from `ollama.rs` into `commands/pre_routing.rs` (107 lines). Deduplicated Redmine pre-routing code that was copy-pasted in two branches. No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/pre_routing.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Extract PERPLEXITY_SEARCH handler into `perplexity_helpers.rs`** — Moved the entire PERPLEXITY_SEARCH tool handler (~200 lines: search, result formatting, auto-screenshot) from `ollama.rs` into `perplexity_helpers.rs` as `handle_perplexity_search()`, `format_search_results_markdown()`, and `auto_screenshot_urls()`. Returns `PerplexitySearchHandlerResult` struct. `ollama.rs` 3809→3502 lines (307 extracted). No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/perplexity_helpers.rs`, `commands/ollama.rs`)
- **Docs: agents.md commands/ directory structure** — Updated AGENTS.md `commands/` listing from 13 files to 45 files, reflecting all extracted modules (tool_loop, verification, agent_session, network/delegation/browser/misc_tool_dispatch, ollama_config/chat/frontend_chat/memory, compaction, session_history, prompt_assembly, reply_helpers, redmine_helpers, perplexity_helpers, loop_guard, etc.). (`AGENTS.md`)
- **Docs: OpenClaw §95 re-verification** — All §7 checks re-run against OpenClaw AGENTS.md vs code (`005-openclaw-reviewer`). Findings: (1) `src/provider-web.ts` does not exist (actual: `src/channel-web.ts`); (2) `src/telegram`, `src/discord`, `src/slack`, `src/signal`, `src/imessage`, `src/web` do not exist as top-level dirs — channels live under `extensions/` and `src/channels/`; (3) `pnpm format` is `oxfmt --write` not `--check` (the check command is `pnpm format:check`); (4) Vitest branch coverage threshold is 55%, not 70% as AGENTS claims; (5) SSRF test count: 15 tests, not 14 (CHANGELOG 0.1.50); (6) `OllamaRequest` has 22 fields, CHANGELOG says "24 positional parameters" (minor). Extensions docs: `anthropic-vertex`, `chutes`, `fal` lack dedicated provider pages; `phone-control` and `thread-ownership` only in zh-CN plugin list. No code bugs found; discrepancies are documentation-only.

## [0.1.50] - 2026-03-21

### Added
- **SSRF protection for all server-side URL fetches and browser navigations** — All model-triggered HTTP requests (`FETCH_URL`) and CDP navigations (`BROWSER_NAVIGATE`, `BROWSER_SCREENSHOT`) are now validated against a blocklist before execution: loopback (127.0.0.0/8, ::1), RFC 1918 private (10/8, 172.16/12, 192.168/16), link-local (169.254.0.0/16, fe80::/10), cloud metadata (169.254.169.254), IPv6 unique-local (fc00::/7), unspecified, broadcast, and IPv4-mapped IPv6 variants are all blocked. URLs with embedded credentials (userinfo) are rejected. DNS resolution is checked against the blocklist (catches hostnames resolving to private IPs). HTTP redirects are validated per-hop via a custom reqwest redirect policy. Configurable allowlist via `ssrfAllowedHosts` in `~/.mac-stats/config.json`. 14 new tests. (`commands/browser.rs`, `browser_agent/mod.rs`, `config/mod.rs`)

### Changed
- **agents.md: uptime section** — Added "Keep mac-stats running (uptime)" section with LaunchAgent recipe, operator checklist, lightweight watchdog, and coding-agent discipline notes. Updated restart coding principle to reference the new section.
- **Docs: SSRF protection** — Documented blocked targets, userinfo rejection, DNS resolution check, redirect protection, and allowlist in `docs/029_browser_automation.md`.

## [0.1.49] - 2026-03-21

### Added
- **Tool budget warning / last-iteration guidance** — When the agent approaches its tool iteration cap, budget warnings and last-iteration guidance are injected into the conversation to encourage result consolidation instead of starting new tool chains that would be cut off. Configurable via `toolBudgetWarningRatio` in config.json or env `MAC_STATS_TOOL_BUDGET_WARNING_RATIO` (0.0–1.0, default 0.75; 0.0 or 1.0 disables). (`commands/ollama.rs`, `config/mod.rs`)
- **Sequence-terminating navigation** — After a page-changing browser action (`BROWSER_NAVIGATE`, `BROWSER_GO_BACK`) in a multi-tool turn, remaining browser tools are skipped because element indices are stale. Non-browser tools still execute. The model receives the new page state and plans new actions in the next turn. (`commands/ollama.rs`)
- **Judge hook for CLI `--run-ollama`** — `run_judge_if_enabled()` now runs after `--run-ollama` completions. (`main.rs`, `lib.rs`)
- **Redmine query utility scripts** — `scripts/redmine_query.py` (Python, grouped time entry reports by ticket and day) and `scripts/redmine_query.sh` (curl wrapper). Both read credentials from `.config.env`.

### Removed
- **CLAUDE.md deleted** — Standalone `CLAUDE.md` removed; all content consolidated into `agents.md` as the single project instructions file.
- **Cleanup: old agent definition files** — Removed `005-openclaw-reviewer/005-openclaw-reviewer.md` and `006-feature-coder/FEATURE-CODER.md` (agent definitions now live in `~/.mac-stats/agents/`).
- **Cleanup: stale release notes** — Removed `release_notes_0.1.18.md`.
- **Cleanup: repetitive testing sections in `docs/022_feature_review_plan.md`** — Trimmed ~460 lines of duplicate closing-reviewer smoke test logs (content preserved in git history).

### Changed
- **agents.md expanded** — Added audience note, project overview at a glance, build/run section, backend runtime and performance summary (why CPU stays low, key technical choices, development notes, testing/debugging quick commands), and version management section. Now the single authoritative instructions file for Cursor, Claude Code, and similar tools.
- **Doc references updated** — All references to `CLAUDE.md` replaced with `agents.md` across `docs/README.md`, `scripts/README.md`, `docs/033_docs_vs_code_review.md`.
- **Docs: tool budget warning** — Documented in `007_discord_agent.md` §17 (config, behavior, disabling).
- **Docs: sequence-terminating navigation** — Documented in `029_browser_automation.md`.
- **Extract verification pipeline and agent session runner from `ollama.rs`** — Moved verification pipeline (`OllamaReply`, `RequestRunContext`, `verify_completion`, `extract_success_criteria`, `sanitize_success_criteria`, `detect_new_topic`, `summarize_last_turns`, `first_image_as_base64`, `original_request_for_retry`, `user_explicitly_asked_for_screenshot`, `truncate_text_on_line_boundaries`, `summarize_response_for_verification` + 12 tests) into `commands/verification.rs` (770 lines); agent session runner (`run_agent_ollama_session`, `execute_agent_tool_call`, `parse_agent_tool_from_response`, `build_agent_runtime_context`, `normalize_discord_api_path` + 4 tests) into `commands/agent_session.rs` (291 lines). `ollama.rs` 6543→5523 lines (1020 extracted). No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/verification.rs`, `commands/agent_session.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Extract Ollama config/startup and reply helpers from `ollama.rs`** — Moved Ollama configuration, startup, and env-variable resolution (`get_ollama_client`, `configure_ollama`, `get_ollama_config`, `list_ollama_models_at_endpoint`, `check_ollama_connection`, `ensure_ollama_agent_ready_at_startup`, `default_non_agent_system_prompt`, `get_default_ollama_system_prompt`, `ChatRequest`, `OllamaConfigRequest`, `OllamaConfigResponse` + env helpers) into `commands/ollama_config.rs` (513 lines); reply-routing helpers (`final_reply_from_tool_results`, `get_mastodon_config`, `mastodon_post`, `append_to_file`, `looks_like_discord_401_confusion`, `extract_url_from_question`, `extract_screenshot_recommendation`, `extract_last_prefixed_argument`, `is_bare_done_plan`, `is_final_same_as_intermediate`, `is_agent_unavailable_error` + tests) into `commands/reply_helpers.rs` (375 lines). `ollama.rs` 5523→4634 lines (889 extracted). No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/ollama_config.rs`, `commands/reply_helpers.rs`, `commands/mod.rs`, `commands/ollama.rs`, `commands/agent_descriptions.rs`, `commands/compaction.rs`, `commands/ollama_models.rs`, `lib.rs`)
- **Extract chat transport, frontend chat commands, and content reduction from `ollama.rs`** — Moved chat transport (`merge_chat_options`, `deduplicate_consecutive_messages`, `send_ollama_chat_messages`, streaming variant, `ollama_chat` Tauri command + 2 stream structs) into `commands/ollama_chat.rs` (351 lines); frontend chat Tauri commands (`ollama_chat_with_execution`, `ollama_chat_continue_with_result`, `ensure_cpu_window_open` + 3 structs) into `commands/ollama_frontend_chat.rs` (372 lines); content reduction + skill/JS execution (`CHARS_PER_TOKEN`, `truncate_at_boundary`, `reduce_fetched_content_to_fit`, `run_skill_ollama_session`, `run_js_via_node`) into `commands/content_reduction.rs` (190 lines). `ollama.rs` 4634→3744 lines (890 extracted). No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/content_reduction.rs`, `commands/ollama_chat.rs`, `commands/ollama_frontend_chat.rs`, `commands/mod.rs`, `commands/ollama.rs`, `lib.rs`)

## [0.1.48] - 2026-03-21

### Added
- **NewMentions alert rule implementation** — `NewMentions::evaluate` now filters Mastodon mention timestamps by configured `hours` window and fires when recent count >= threshold. `MonitorStatus` gains `extra: HashMap<String, Value>` (`#[serde(default)]`) for monitor-specific data. `MastodonMonitor::check_mentions()` returns timestamps + count; Mastodon API query now filters `types[]=mention&limit=40`. Website monitor satisfies new field with `Default::default()`. Previously the NewMentions rule always returned false. (`alerts/rules.rs`, `monitors/mod.rs`, `monitors/social.rs`, `monitors/website.rs`)
- **Redmine time entry creation (POST /time_entries.json)** — `is_allowed_post_path` extended to accept `time_entries.json`/`time_entries.xml`; new `parse_time_entry_activities()` fetches activity IDs into Redmine create context; agent description updated with POST /time_entries.json syntax and "log time" documentation; `wants_create_or_update` triggers on "log time/hours/book time/book hours/time entry"; date-sensitive tests fixed to use `chrono::Utc::now()` instead of hardcoded dates; tests for path allowlist and activity parsing. (`redmine/mod.rs`, `commands/ollama.rs`)

### Changed
- **Chrome lean flags (visible + headless)** — Visible Chrome (`launch_chrome_on_port()`, both macOS and non-macOS) now adds 6 lean flags: `--disable-extensions`, `--disable-background-networking`, `--disable-sync`, `--disable-default-apps`, `--disable-background-timer-throttling`, `--disable-renderer-backgrounding`. Headless Chrome (`launch_via_headless_chrome()`) adds `--disable-software-rasterizer`, `--mute-audio` via `LaunchOptions::args()`. Reduces helper process CPU when Chrome is launched for automation. (`browser_agent/mod.rs`)
- **Configurable browser idle timeout** — `Config::browser_idle_timeout_secs()` default lowered from 3600 (1 hour) to 300 (5 minutes). Now reads from env `MAC_STATS_BROWSER_IDLE_TIMEOUT_SECS` and config.json `browserIdleTimeoutSecs`, clamped to 30..=3600. (`config/mod.rs`)
- **Docs 032: Chrome helper processes plan — implementation complete** — All plan items marked implemented: lean flags, headless args, configurable idle timeout, documentation. Sign-off section replaced with implementation status checklist.
- **Zero clippy warnings** — Fixed all 44 clippy warnings across 12 source files: `strip_prefix()` instead of manual `starts_with()` + slice indexing (`task/mod.rs`, `session_memory.rs`, `mcp/mod.rs`); `.values()` instead of `.iter().map(|(_, v)| …)` (`monitors.rs`); `.rfind()` instead of `.filter().next_back()` (`ollama/models.rs`); `&Path` instead of `&PathBuf` (`task/runner.rs`); collapsed `if` blocks (`logging/mod.rs`, `commands/ollama.rs`); removed redundant variable rebindings; `#[allow(clippy::too_many_arguments)]` on two functions; function pointer instead of closure (`commands/ollama.rs`); `.is_none_or()` instead of `.map_or()` (`commands/ollama.rs`); `.trim()` before `.split_whitespace()` removed (`browser_agent/mod.rs`); doc-comment continuation lines indented (`commands/ollama.rs`). No behavioral changes.
- **Extract tool parsing into `commands/tool_parsing.rs`** — Moved 12 functions + 3 constants + tests from `ollama.rs` (9408→8923 lines) into `commands/tool_parsing.rs` (553 lines): `TOOL_LINE_PREFIXES`, `MAX_BROWSER_TOOLS_PER_RUN`, `MAX_TOOLS_PER_RESPONSE`, `line_starts_with_tool_prefix`, `parse_one_tool_at_line`, `truncate_search_query_arg`, `normalize_inline_tool_sequences`, `parse_tool_from_response`, `normalize_browser_tool_arg`, `parse_all_tools_from_response`, `parse_python_script_from_response`, `parse_fetch_url_from_response`. No behavioral changes; zero clippy warnings, all tests pass. (`commands/tool_parsing.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Docs: OpenClaw §87–§90 re-verification** — All §7 checks re-run; no discrepancies found (`005-openclaw-reviewer`).
- **FEATURE-CODER backlog** — Clippy clean builds, Chrome lean flags, Redmine time entry creation, and tool parsing extraction rows marked done (`006-feature-coder`).
- **Docs 025: Redmine API skill** — POST time entry documented; create context description updated; open task marked done.
- **agents.md** — Directory structure updated to include `tool_parsing.rs` under `commands/`.
- **Tighten JS code detection to reduce spurious execution rounds** — Replaced over-broad keyword-based fallback (`"function"`, `"=>"`, `"console.log"` anywhere in text) with fenced-code-block detection: only `ROLE=code-assistant` prefix or a markdown ` ```javascript`/` ```js`/` ``` ` block with executable JS patterns triggers code execution. Prose that merely *mentions* code no longer fires. Shared helper `detect_and_extract_js_code()` in `tool_parsing.rs` (DRY); both `ollama_chat_with_execution` and `ollama_chat_continue_with_result` use it. 12 new tests, 109 total pass, zero clippy warnings. (`commands/tool_parsing.rs`, `commands/ollama.rs`)
- **Docs: OpenClaw §91 re-verification** — All §7 checks re-run; no discrepancies found (`005-openclaw-reviewer`).
- **022 testing note** — Closing reviewer smoke tests 2026-03-20 (code detection tightening, tool_parsing extraction; cargo build, debug.log, agents, monitors UP).
- **037 follow-up marked done** — Code detection tightening follow-up noted as implemented (`docs/037`).
- **Extract model management + JS logging from `ollama.rs`** — Moved 9 Ollama model management Tauri commands (`list_ollama_models`, `list_ollama_models_full`, `get_ollama_version`, `list_ollama_running_models`, `pull_ollama_model`, `delete_ollama_model`, `ollama_embeddings`, `unload_ollama_model`, `load_ollama_model`) into `commands/ollama_models.rs` (237 lines) and 4 JS execution logging commands (`log_ollama_js_execution`, `log_ollama_js_check`, `log_ollama_js_extraction`, `log_ollama_js_no_blocks`) into `commands/ollama_logging.rs` (116 lines). `ollama.rs` shrinks by 344 lines. `get_ollama_client` made `pub(crate)` for cross-module access. No behavioral changes; zero clippy warnings, all tests pass. (`commands/ollama_models.rs`, `commands/ollama_logging.rs`, `commands/mod.rs`, `commands/ollama.rs`, `lib.rs`)
- **Extract Redmine helpers from `ollama.rs` into `commands/redmine_helpers.rs`** — Moved 16 Redmine helper functions (`extract_ticket_id`, `question_explicitly_requests_json`, `extract_redmine_time_entries_summary_for_reply`, `extract_redmine_failure_message`, `is_redmine_infrastructure_failure_text`, `format_redmine_time_entries_period`, `grounded_redmine_time_entries_failure_reply`, `is_grounded_redmine_time_entries_blocked_reply`, `is_redmine_review_or_summarize_only`, `is_redmine_relative_day_request`, `is_redmine_yesterday_request`, `is_redmine_time_entries_request`, `redmine_time_entries_range_for_date`, `redmine_time_entries_range`, `redmine_request_for_routing`, `redmine_direct_fallback_hint`) + 12 tests into `commands/redmine_helpers.rs` (427 lines). `ollama.rs` 8391→8016 lines (375 extracted). No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/redmine_helpers.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **Docs: OpenClaw §92–§93 re-verification** — All §7 checks re-run; no discrepancies found (`005-openclaw-reviewer`).
- **FEATURE-CODER backlog** — Model management + JS logging extraction and Redmine helpers extraction rows marked done (`006-feature-coder`).
- **agents.md** — Directory structure updated to include `ollama_models.rs` and `ollama_logging.rs` under `commands/`.
- **022 testing note** — Closing reviewer smoke test 2026-03-20 (redmine_helpers extraction; cargo build, 114 tests pass, debug.log, agents, monitors UP).
- **Extract Perplexity helpers, memory loading, and session compaction from `ollama.rs`** — Moved Perplexity/news search helpers (8 functions + 12 tests) into `commands/perplexity_helpers.rs` (454 lines); memory/soul loading (7 functions) into `commands/ollama_memory.rs` (158 lines); session compaction (3 functions + 5 constants) into `commands/compaction.rs` (289 lines). `ollama.rs` 8016→7136 lines (880 extracted). `lib.rs` updated to call `commands::compaction::run_periodic_session_compaction`. No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/compaction.rs`, `commands/ollama_memory.rs`, `commands/perplexity_helpers.rs`, `commands/mod.rs`, `commands/ollama.rs`, `lib.rs`)
- **Docs: OpenClaw §94 re-verification** — All §7 checks re-run; no discrepancies found (`005-openclaw-reviewer`).
- **FEATURE-CODER backlog** — Perplexity/memory/compaction extraction row marked done (`006-feature-coder`).
- **022 testing note** — Closing reviewer smoke test 2026-03-21 (Perplexity/memory/compaction extraction; cargo build, 114 tests pass, debug.log, agents, monitors UP).
- **Extract agent descriptions, browser helpers, and schedule parsing from `ollama.rs`** — Moved agent/tool description building (9 constants + 5 functions: `AGENT_DESCRIPTIONS_BASE`, `SCHEDULE_CRON_EXAMPLES`, `format_run_cmd_description`, `build_skill_agent_description`, `build_agent_agent_description`, etc.) into `commands/agent_descriptions.rs` (246 lines); browser task helpers (10 functions + 6 tests) into `commands/browser_helpers.rs` (213 lines); schedule parsing (1 enum + 2 functions) into `commands/schedule_helpers.rs` (152 lines). `ollama.rs` 7136→6543 lines (593 extracted). No behavioral changes; zero clippy warnings, 114 tests pass. (`commands/agent_descriptions.rs`, `commands/browser_helpers.rs`, `commands/schedule_helpers.rs`, `commands/mod.rs`, `commands/ollama.rs`)
- **FEATURE-CODER backlog** — Agent descriptions/browser helpers/schedule parsing extraction row marked done (`006-feature-coder`).

## [0.1.47] - 2026-03-20

### Fixed
- **Alert sustained-duration enforcement (TemperatureHigh, CpuHigh)** — `AlertManager` now tracks `condition_since` per alert and only fires when the threshold is exceeded for >= `duration_secs` consecutive seconds. Previously `duration_secs` was ignored and alerts fired on any single reading. New `required_duration_secs()` method on `AlertRule`. (`alerts/mod.rs`, `alerts/rules.rs`)

### Added
- **CLI: `agent reset-defaults [id]`** — New subcommand to force-overwrite bundled default agent files (agent.json, skill.md, testing.md, soul.md). Optional id filter to reset a single agent. `Config::reset_agent_defaults()` in config/mod.rs.

### Changed
- **Cloud model role resolution warning** — `resolve_agent_models` in agents/mod.rs: when role resolution fails because all catalog models are cloud, the warning now says "cloud default will be used at chat time (no local models available)" instead of the generic message.
- **Docs 017: cloud model fallback + agent reset** — New §§ "Cloud model as default — fallback behavior" (scenario table, entry-point vs sub-agent, local-preference override, warning log) and "Agent Reset" (CLI usage, overwrite semantics, user-file safety). Two open tasks marked done.
- **Docs housekeeping (004, 012, 015, 020, 100)** — Stale/vague open tasks marked deferred or done across five docs; each now points to 006-feature-coder/FEATURE-CODER.md for the active backlog.
- **FEATURE-CODER backlog** — Rows for cloud model roles, agent reset CLI, and docs open-task trim marked done; new "Trim stale open tasks" row added and closed.
- **022 testing note** — Closing reviewer smoke test 2026-03-20 (cargo build, debug.log, 8 agents, 15 models, 4 monitors UP).
- **Docs 005 §85, 006, 017, 022** — OpenClaw re-verification §85 (005); docs/017_llm_agents.md new §§ "testing.md format" (file structure, parsing rules, conventions, timeout, examples) and "Orchestrator routing examples" (routing table, multi-step, fallback), two open tasks marked done; FEATURE-CODER backlog rows for 017 testing.md and orchestrator routing done, two new open items added (006); 022 closing reviewer testing note 2026-03-20 (smoke).
- **Docs 005 §84, 006, 017, 022** — OpenClaw re-verification §84 (005); FEATURE-CODER backlog row for 017 "AGENT: <selector> [task] syntax" done (006); docs/017_llm_agents.md new § "AGENT: <selector> [task] syntax" (invocation, selector resolution order, optional task, cursor-agent proxy, behaviour) and open task marked done; 022 testing note 2026-03-20 (closing reviewer).
- **Docs 005 §83, 006, 017, 022** — OpenClaw re-verification §83 (005); FEATURE-CODER backlog row for 017 model_role resolution done (006); docs/017_llm_agents.md new § "model_role resolution logic" and open task marked done; 022 testing note 2026-03-20 (closing reviewer).
- **Docs 005 §82, 006, 022, 100** — OpenClaw re-verification §82 (005); FEATURE-CODER backlog row for 100 "Improve the user interface for scheduling tasks" done (006): scheduler UI already in Settings → Schedules tab; 100_all_agents open task marked done; 022 testing note 2026-03-20 (closing reviewer).
- **Docs 005 §81, 006, 014, 022** — OpenClaw re-verification §81 (005); FEATURE-CODER backlog row for 014 Python agent security review done (006); docs/014_python_agent.md new § "Security review (measures in place)" and open task marked done; 022 testing note 2026-03-20 (closing reviewer).
- **Docs 005 §80, 006, 022; get_cpu_details() API contract** — OpenClaw re-verification §80 (005); FEATURE-CODER backlog row for data-poster-charts-backend "Review and refactor get_cpu_details() API response" done (006): API contract documented in docs/data-poster-charts-backend.md (§ get_cpu_details() API contract); `CpuDetails` struct doc comment in metrics/mod.rs points to it. 022 testing note 2026-03-20 (closing reviewer).
- **Docs 005 §77, 006, 016, 022** — OpenClaw re-verification §77 (005); FEATURE-CODER backlog row for 016 "Clarify advanced skill features" done (006): open task in docs/016_skill_agent.md labeled "Future/backlog" and pointed to FEATURE-CODER; new "When backlog is empty" section in FEATURE-CODER. 022 testing note 2026-03-20 (closing reviewer).
- **Data Poster theme history charts (005 §76, 006, 022)** — Data Poster CPU theme now loads `history.js` so the history section uses the backend buffer (`get_metrics_history`); previously had history canvases but did not load the script. OpenClaw re-verification §76 (005); FEATURE-CODER backlog row for data-poster-charts-backend "frontend not utilizing historical data buffer" done (006); docs/data-poster-charts-backend.md and 022 testing note 2026-03-20.
- **Keychain credential list via persisted file (security)** — `list_credentials()` no longer relies on Keychain attribute enumeration (security_framework does not expose it for generic password items). Account names are persisted in `~/.mac-stats/credential_accounts.json`; `store_credential`/`delete_credential` update the file. New `Config::credential_accounts_file_path()`; docs data_files_reference § credential_accounts.json, 022 testing note 2026-03-19, 005/006.
- **Docs 005 §74, 006, 020, 022, README** — OpenClaw re-verification §74 (005); FEATURE-CODER backlog row for 020 "Documentation: Update for clarity and completeness" done (006); docs/020 tool table completed (RUN_JS, PERPLEXITY_SEARCH, RUN_CMD implementation details), See also for full list; RUN_JS row in docs/README.md fixed (was truncated); 022 testing note 2026-03-19 (closing reviewer).

### Added
- **Settings → Ollama tab** — Dashboard Settings: new Ollama tab to set endpoint URL and model (dropdown populated via "Refresh models", Apply); backend `get_ollama_config`, `list_ollama_models_at_endpoint`. Same config as CPU window; docs 005 §73, 006, 015, 022.
- **Settings → Skills tab** — Dashboard Settings: new Skills tab lists loaded skills (number, topic, path) via `list_skills` Tauri command; hint to ~/.mac-stats/agents/skills/ and docs/016. Backend: `commands/skills.rs`, `SkillForUi`, `list_skills_for_ui()` in skills.rs. Docs: 005 OpenClaw §72 re-verification, 006 FEATURE-CODER and 016 open task "Improve the user interface for managing skills" done, 022 testing note 2026-03-19.

### Changed
- **Docs 005 §71, 006, 022, 033** — OpenClaw re-verification §71 (005); FEATURE-CODER backlog row for 033 RUN_CMD allowlist note done (006); 033 resolution: full allowlist in 011 and 100, no further change; 022 testing note wording (smoke log details).
- **Docs 005 §70, 006, 011, 022, 100** — OpenClaw re-verification §70 (005); FEATURE-CODER backlog row for 011 shell-injection review done (006); docs/011_local_cmd_agent.md new § "Shell injection considerations" (full stage to `sh -c`, first token allowlisted, path validation, trust boundary and mitigations, strict-mode option as future); 100 open task run_local_command hardening review done; 022 testing note 2026-03-19 (closing reviewer).
- **Skills load logging and docs (016, 005, 006, 007, 012, 022, 100)** — `skills.rs`: warn when skipping files with no valid stem or invalid name format (skill-&lt;number&gt;-&lt;topic&gt;.md); info when skipping empty files; summary line when any files skipped (invalid name / empty) with pointer to docs/016. Docs: 005 OpenClaw, 006 FEATURE-CODER, 007 Discord, 012 skills, 016 skill agent (path/naming), 022 feature review, 100 all agents.
- **MCP error handling and retry (010, 006, 005 §68, 022, 100)** — docs/010_mcp_agent.md new § "Error handling" (list_tools/call_tool failure behavior, user/model message); one retry for transient errors (timeout, connection refused, etc.) in mcp/mod.rs (`list_tools_once`, `call_tool_once`, `is_transient_mcp_error`). OpenClaw re-verification §68 (005); FEATURE-CODER and docs/100_all_agents.md MCP error handling task done; 022 smoke note update.
- **Docs 005 §67, 006, 014, 022; PYTHON_SCRIPT diagnostics** — OpenClaw re-verification §67 (005); FEATURE-CODER backlog row for 014 Python agent diagnostics done (006); docs/014 open task done. PYTHON_SCRIPT: script path in user-facing error; `tracing::warn!` on spawn failure and on non-zero exit (script path, exit code, stderr preview 500 chars) to `~/.mac-stats/debug.log`. 022 smoke note (executable path, pid).
- **Docs 006, 022, 030** — Planning memory: new § "Planning memory — current behavior and considerations" in docs/030_screenshot_request_log_analysis.md (what planning receives, session vs global memory, recommendations); open tasks marked done. FEATURE-CODER backlog row for 030 session/global memory investigation done (006). 022 smoke note: executable path in example fixed to `./src-tauri/target/release/mac_stats`.
- **Docs 007, 020, 022, 006, README** — Discord bot "Bot functionality at a glance" in docs/007_discord_agent.md §2 (triggers, reply pipeline, personalization, session/memory, scheduling, optional); docs/README At a Glance one-line Discord summary with link to 007; 020 and FEATURE-CODER backlog task "Complete description of Discord bot functionality" done; 022 testing note 2026-03-19 (closing reviewer).
- **Redmine API error handling and docs (025, 006, 005 §66, 022)** — `redmine_api_request` returns clear user-facing messages for 401 (check API key), 404 (check ID/path), 422 (date-format hint); generic status and body snippet unchanged. docs/025: Configuration, Error handling (table), Implementation sections; open tasks moved to FEATURE-CODER. FEATURE-CODER backlog rows for 025 done (006). OpenClaw re-verification §66 (005); 022 testing note 2026-03-19. Removed duplicate §66 blocks from 005.
- **Docs 005 §65, 006, 011, 022; Cargo.lock** — OpenClaw re-verification §65 (005); FEATURE-CODER backlog row for 011 "Consider more RUN_CMD features" done — design only (006); docs/011_local_cmd_agent.md new § "More RUN_CMD features (design only)" (candidate commands table, path validation current + possible improvements); 022 testing note 2026-03-19 (closing reviewer). Cargo.lock version synced to 0.1.45.
- **Docs 005 §63, 006, 011, 022** — OpenClaw re-verification §63 (005); FEATURE-CODER backlog: 011 security review done, retry loop and more RUN_CMD features as optional/open (006); docs/011_local_cmd_agent.md new § "Security review (measures in place)" and open tasks moved to FEATURE-CODER (011); 022 testing note 2026-03-19 (closing reviewer).
- **Docs 005 §62, 006, 011, 022** — OpenClaw re-verification §62 (005); FEATURE-CODER backlog row for 011 RUN_CMD docs done (006); docs/011_local_cmd_agent.md updated (shell execution, allowlist case-insensitive, pipelines, duplicate detection, TASK_APPEND full output, RUN_CMD naming, retry count, tool iterations); 022 testing note 2026-03-19 (closing reviewer).
- **Docs 005 §61, 006, 019, 022** — OpenClaw re-verification §61 (005); FEATURE-CODER backlog row for 019 manual-edit long-term memory done (006); 019 new § "Manual edit of long-term memory (future)" and open task marked done; 022 testing note 2026-03-19 (closing reviewer).
- **Docs 005 §60, 006, 022** — OpenClaw re-verification §60 (005); FEATURE-CODER backlog row for 022 toggle_cpu_window verification done (006): verified in status_bar.rs that always-recreate after close is intentional; 022 F9 checklist and smoke note (build, run, debug.log) (2026-03-19).
- **Docs 005 §59, 006, 019, 022** — OpenClaw re-verification §59 (005); FEATURE-CODER backlog row for 019 conversation-history storage structure review done (006); 019 new § "Conversation-history storage structure (review)" (in-memory HashMap+Vec, persistence one file per persist, when to revisit; no code change) and open task closed; 022 testing note (2026-03-19).
- **Docs 005 §58, 006, 014, 022** — OpenClaw re-verification §58 (005); FEATURE-CODER row for 014 Python agent docs done (006); docs/014_python_agent.md expanded (when to use, config precedence, invocation examples, behaviour, security, troubleshooting table, PYTHON_SCRIPT in tool table); 022 testing note 2026-03-19.
- **Browser tool limit user-facing note (032)** — When the browser action cap (15 per run) is reached, the reply now appends: "Note: Browser action limit (15 per run) was reached; some actions were skipped." (`browser_tool_cap_reached` in `commands/ollama.rs`). Docs 005 §53 re-verification, 006 FEATURE-CODER and 032 open task marked done, 022 testing note.
- **Duplicate browser action refusal (032)** — Consecutive identical browser actions (same BROWSER_NAVIGATE URL or same BROWSER_CLICK index) are skipped; reply gets "Same browser action as previous step; use a different action or reply with DONE." `normalize_browser_tool_arg`, `last_browser_tool_arg` in `commands/ollama.rs`. Docs 005, 006, 022, 032.
- **Unknown-tool handling in tool loop (032)** — In `ollama.rs` tool loop, the catch-all for unknown tools no longer silently skips; unknown tools now produce a user-facing hint ("Unknown tool \"X\". Use one of the available tools...") and `tracing::warn!` so the model gets feedback and logs are traceable. Docs 005 §56 re-verification, 006 FEATURE-CODER and 032 open task marked done, 022 smoke note.
- **Session memory parser fix (019)** — `parse_session_file` in `session_memory.rs` now trims leading `## ` from each block so the first User/Assistant block is recognized when loading session files. Docs 019 implementation review done, 005 §57 re-verification, 006 FEATURE-CODER backlog, 022 testing note (2026-03-18).

## [0.1.46] - 2026-03-20

### Changed
- **Data Poster CPU: temperature cadence + chart backlog closed (005 §78–§79, 006, 022)** — `cpu.js`: temperature DOM, ring gauge, `posterCharts`, and per-theme `*History.updateTemperature` calls run only on the 3s temperature tick (usage/frequency remain 1s). OpenClaw re-verification §78–§79 (005); FEATURE-CODER rows done for chart-specific refresh rates and display smoothing (`poster-charts.js` moving average); `docs/data-poster-charts-backend.md` open tasks closed; 022 testing notes 2026-03-20.

## [0.1.45] - 2026-03-19

### Changed
- **RUN_CMD fix retry and docs 005 §64, 006, 011, 022** — Agent router RUN_CMD: only accept RUN_CMD in fix suggestion; one format-only retry when parse fails; clearer messages (format required, could not get corrected command). OpenClaw re-verification §64 (005); FEATURE-CODER backlog row RUN_CMD retry loop done (006); docs/011 retry steps and open task done; 022 testing note 2026-03-19.

## [0.1.44] - 2026-03-18

### Changed
- **Browser agent element label cache (032)** — `LAST_ELEMENT_LABELS` now uses `HashMap<u32, String>` for O(1) lookup when resolving labels for BROWSER_CLICK/BROWSER_INPUT status messages; `set_last_element_labels` builds map from vec (duplicate indices: last wins); `get_last_element_label` doc comment documents edge cases (lock poison, empty cache, index not in last state). Docs: 005 §54 re-verification, 006 FEATURE-CODER and 032 open task marked done, 022 testing note.

## [Unreleased]

### Changed
- **Docs 005 §52, 006, 021, 022, agents-tasks** — OpenClaw re-verification §52 (005); task-008 Phase 6 done: new § "Retry and failover taxonomy" in docs/021 (retry table: Ollama, verification, Discord API, CDP, BROWSER_NAVIGATE failover, compaction, having-fun; no-retry cases; summary); FEATURE-CODER and agents-tasks Phase 6 done; 022 testing note (2026-03-18).
- **Agent router observability (task-008 Phase 7)** — request_id on all agent-router logs (criteria, new-topic, prior session, compaction); SAME_TOPIC log when keeping history; prior session message count and cap; compaction decision/result with request_id and context/lessons; Brave and Perplexity search result count and blob size in logs. Docs 005 §51, 006 Phase 7 done, agents-tasks task-008 Phase 7 done, 022 testing note (2026-03-18).

### Added
- **Optional post-run agent judge** — When enabled (`agentJudgeEnabled` in config.json or `MAC_STATS_AGENT_JUDGE_ENABLED`), after each agent run (Discord reply or scheduler task) the app calls an LLM to evaluate whether the task was satisfied and logs the verdict (and optional reasoning) to `~/.mac-stats/debug.log`. For testing or quality logging only; does not change the agent loop or user-facing replies. New module `commands/judge.rs`; config `Config::agent_judge_enabled()`; docs/007_discord_agent.md §15.

### Changed
- **Session compaction hardening (task-008 Phase 5)** — Skip compaction when session has no real conversational value: `count_conversational_messages()` in `session_memory.rs`; compactor and periodic job require at least 2 conversational messages; compactor prompt preserves first system/task instructions and most recent assistant/tool outcome; clear logs for skipped vs failed; periodic job does not retry on skip. Docs 005 §50, 006, 022, agents-tasks Phase 5 done.
- **News/current-events format and verification** — `is_news_query` extended with "today" and "this week"; new `verification_news_format_note()` so verifier accepts concise/bullet answers and requires 2+ named sources and dates when available; success criteria and system reminder for news requests (short bullet list, 2 sources, dates). Docs 005, 006, 022.
- **Redmine create-context only when create/update (034)** — In `build_agent_descriptions`, `wants_create_or_update` aligned with pre-route: added "with the next steps", "put "; when `question` is None no create-context (`unwrap_or(false)`). Docs 005 §48, 006, 022, 034.
- **Search result shaping for Brave (task-008 Phase 3)** — New `search_result_shaping.rs`: shared `ShapableSearchResult`, `shape_search_results()` (snippet truncation, domain dedup, result cap), `format_search_results_blob()` with head+tail truncation. Brave Search now uses it: results have title, URL, snippet (280 chars), date when API provides `age`; max 10 results, 2 per domain; blob capped at 12k chars. Perplexity keeps existing news-specific shaping. FEATURE-CODER and task-008 Phase 3 done.
- **Session memory: internal artifacts not persisted (task-008 Phase 2)** — `session_memory.rs`: `is_internal_artifact()` filters completion-verifier prompts, criteria extraction, tool dumps, escalation prompts; `add_message` skips them; `get_messages`, `parse_session_file`, `replace_session` exclude internal when loading. Unit test `internal_artifacts_not_persisted`. Docs 005 §46, 006, 022, agents-tasks task-008 Phase 2 done.
- **Request-local execution state (task-008 Phase 1)** — `RequestRunContext` in `commands/ollama.rs` holds request_id, retry_count, original question, and Discord context; `answer_with_ollama_and_fetch` accepts `request_id_override` and `retry_count` so verification retries use the same request_id for log correlation and request-local criteria only. NEW_TOPIC log clarifies retries use request-local criteria. Call sites (Discord, main, scheduler, task runner) pass `None, 0` for first run. FEATURE-CODER and agents-tasks task-008: Phase 1 done.
- **Docs 005 §45, 006, 022** — OpenClaw §45 re-verification (005); FEATURE-CODER backlog row for task-008 Phase 1 done (006); 022 testing note (2026-03-18 closing reviewer).
- **Docs 005 §44, 006, 021, 022** — OpenClaw §44 re-verification (005); data_files_reference row in table (005). FEATURE-CODER: backlog row "more advanced tool commands" done (006); 021 new § "More advanced tool commands (future)" (structured args, streaming, compound/batch, tool schema; no code); 022 testing note (2026-03-18 closing reviewer).
- **Docs 005, 006, 021, 022; agents init** — OpenClaw §43 re-verification (005); FEATURE-CODER backlog row for 021 agent initialization investigation done (006); 021 new § "Agent initialization and model resolution" (load from disk, ModelCatalog, startup order, failure modes), open task marked done; 022 testing note (2026-03-18 closing reviewer). agents/mod.rs: log when model catalog not yet available (Ollama may still be starting).
- **Docs 005, 006, 021, 022** — OpenClaw §42 re-verification (005); FEATURE-CODER backlog row for 021 specialist agents docs done (006); 021 new § "Specialist agents" (definition, invocation, what they receive, where they live, default table, limitation), open task marked done; 022 testing note (2026-03-18 closing reviewer).
- **Docs 005, 006, 022** — OpenClaw §40 re-verification (005); FEATURE-CODER: design § "More robust patching system (Coder agent)" (dry-run, atomic apply, patch files, audit trail; current choice in-place), backlog row done (006); 022 testing note (2026-03-17 closing reviewer).
- **Docs 005, 006, 009, 022** — OpenClaw §39 re-verification (005); FEATURE-CODER backlog: scheduler multiple API keys design done (006); 009 new § "Multiple API keys / endpoints (design)" (current behaviour, options; no code); 022 testing note (2026-03-17 closing reviewer).

### Added
- **Scheduler UI** — Settings → Schedules tab: list schedules (id, cron/at, task preview, next run); add recurring (cron) or one-shot (at datetime) with optional Discord reply channel; remove by id. Backend: `list_schedules`, `add_schedule`, `add_schedule_at`, `remove_schedule`; scheduler `list_schedules_for_ui`, `ScheduleForUi`. See `commands/scheduler.rs`, `src/dashboard.html`, `src/dashboard.js`; FEATURE-CODER backlog done.
- **Dashboard Settings modal** — Settings modal (Monitors / Alert channels tabs): list monitors with name, URL, type via `list_monitors_with_details`; add website monitor (name, URL, timeout, interval, verify SSL); list and add alert channels (Telegram/Slack/Mastodon). Backend: `list_monitors_with_details`, `list_alert_channels`; `get_monitor_details` returns name and monitor_type from config. "Add monitor" opens Settings on Monitors tab. See `src/dashboard.html`, `src/dashboard.js`, `commands/monitors.rs`, `commands/alerts.rs`.
- **Periodic alert evaluation** — Background thread in lib.rs runs every 60s; `run_periodic_alert_evaluation()` in commands/alerts.rs builds context from metrics and monitor statuses and evaluates all alerts; `get_monitor_statuses_snapshot()` in commands/monitors.rs. SiteDown, BatteryLow, TemperatureHigh, CpuHigh etc. fire without user action. Docs 004 Known Issues and FEATURE-CODER backlog updated.

### Changed
- **Docs 004, 005, 006, 022** — 004: Alert evaluation periodic task marked done (Known Issues §2). OpenClaw §38 re-verification (005). FEATURE-CODER backlog row for periodic alert evaluation done (006). 022 testing note (2026-03-17 closing reviewer).
- **Docs 005, 006, 022; task duplicate error** — OpenClaw §37 re-verification (005); FEATURE-CODER D2 done — TASK_CREATE duplicate error suggests "or use a different id to create a new task" (006, 022); task/mod.rs error message updated; 022 testing note (2026-03-17).
- **Docs 005, 006, 022, 035** — OpenClaw §36 re-verification (005); FEATURE-CODER memory pruning docs done (006); 022 testing note (2026-03-17 closing reviewer); 035 new § "Memory pruning and compaction" (caps, on-request/periodic, having_fun, refs).
- **Docs 005, 006, 022, data_files_reference** — OpenClaw §35 re-verification (005); FEATURE-CODER multi-language reset phrases done (006); 022 closing testing note (2026-03-17); data_files_reference § session_reset_phrases.md.
- **Session reset phrases (docs)** — `docs/data_files_reference.md`: new § session_reset_phrases.md (path, format, multi-language default, fallback). FEATURE-CODER backlog: multi-language reset phrases marked done (035).
- **FETCH_URL content reduction** — `reduce_fetched_content_to_fit`: fast path via byte-length heuristic when body fits; when over limit by ≤25%, truncate only (no summarization) to avoid extra Ollama call; truncation at last newline/space via `truncate_at_boundary` for readability. See `commands/ollama.rs`; FEATURE-CODER and docs/012 open task marked done.
- **Discord skill-not-found** — When user requests a missing skill (e.g. `skill: 99`), Discord replies with "Skill \"X\" not found. Available: 1-summarize, 2-code." and returns early; `parse_discord_ollama_overrides` returns `requested_skill_selector` so handler can detect not-found. FEATURE-CODER and docs/012 open task marked done. See `discord/mod.rs`.
- **Docs 005, 006, 012, 022** — OpenClaw §32 re-verification (005); FEATURE-CODER backlog row for skill-not-found done (006); docs/012 open tasks note updated; 022 testing note (2026-03-17) and closing reviewer run (FETCH_URL content reduction).
- **Docs backlog centralization** — OpenClaw §30 re-verification (005); FEATURE-CODER backlog: active open tasks centralized in 006-feature-coder/FEATURE-CODER.md; 007, 008, 012, 022, 029, 002, 035, agent_workflow, docs/README now point to it; 022 testing note 2026-03-17 (closing reviewer).
- **Docs 005, 022** — OpenClaw §29 re-verification (005); 022 testing note (2026-03-17, closing reviewer).
- **Docs 004, 005, 006, 009, 022** — OpenClaw §28 re-verification (005); FEATURE-CODER "Remaining open" table (006); open tasks in 004 and 009 consolidated to FEATURE-CODER; 022 testing note (2026-03-17).
- **Docs 004, 005, 006, 022** — Known Issues §2 Alert System: channel registration items marked done (004); FEATURE-CODER backlog row for sync (006). OpenClaw §27 re-verification (005); 022 closing testing note (2026-03-16).

### Added
- **Alert channel commands** — Tauri commands to register/unregister alert channels: `register_telegram_channel(id, chat_id)`, `register_slack_channel(id)`, `register_mastodon_channel(id, instance_url)`, `remove_alert_channel(channel_id)`. Credentials via Keychain (telegram_bot_{id}, slack_webhook_{id}, mastodon_alert_{id}). See `commands/alerts.rs`; docs/004_notes.md and FEATURE-CODER backlog updated.

### Changed
- **Discord token storage (docs)** — `docs/007_discord_agent.md` §11: added "Secure token storage (recommended)" (Keychain via Settings for production; env/.config.env for dev/CI). Open task and FEATURE-CODER backlog marked done.
- **Docs 006, 007, 022, data_files_reference** — FEATURE-CODER backlog: schedules.json data-structure investigation done (006); 007 open task marked done, linked to data_files_reference § "Data structure and performance" (array kept, O(n) acceptable); 022 closing testing note (2026-03-16 reviewer run); data_files_reference: new § "Data structure and performance" for schedules.json.
- **Docs 005, 006, 022, 029** — OpenClaw re-verification §25 (005); FEATURE-CODER backlog row for 029 Chrome 9222 troubleshooting done (006); 022 closing testing note (2026-03-16); 029 new § "Troubleshooting: Chrome won't start or connect on 9222" (default path, port in use, spawn failures, connection timing, firewall, headless fallback, debug log); open task marked done.
- **Plugin execution diagnostics** — Plugin errors and warnings now include plugin id and script path; script-not-found, spawn failure, timeout, wait failure, and JSON parse errors are clearer; failed runs log exit code and trimmed stderr; parse errors include stdout snippet. See `plugins/mod.rs`. Docs 004, 022, 005 (§), 006 backlog updated.
- **Plugin script timeout** — Plugin execution now respects `timeout_secs`: script runs in a thread, main thread waits with `recv_timeout`; on timeout the process is killed (Unix) and a clear error is returned. See `plugins/mod.rs`; docs/004_notes.md and FEATURE-CODER backlog updated.
- **test_discord_connect --quick / -q** — `--quick` or `-q` runs for 2 seconds (enough to see "Bot connected" then exit). Docs: §12 in `docs/007_discord_agent.md`; FEATURE-CODER backlog and 007 open task marked done. OpenClaw re-verification §22 (005); 022 testing note 2026-03-16 (closing reviewer).
- **Process list DOM (CPU window)** — In `dist/cpu.js`: use `replaceChildren()` instead of `innerHTML = ""`; single click listener on list (event delegation) instead of per-row listeners; skip DOM update when process list data unchanged (`lastProcessListKey`). Docs 002 task and FEATURE-CODER backlog marked done; OpenClaw §21 (005); 022 testing note 2026-03-16.
- **Theme switch animation** — 200ms fade-out on body before theme navigation in `cpu-ui.js` (ensureThemeSwitchStyle + transitionend/250ms fallback); no extra ongoing CPU. Open task in docs/002 and FEATURE-CODER backlog marked done; OpenClaw re-verification §20 (005); 022 closing testing note (2026-03-16).
- **Docs 005, 006, 002, 022** — OpenClaw re-verification §19 (005); FEATURE-CODER backlog row for 002 fetch_page_content verification done (006); 002 § on fetch_page_content/main-thread blocking verified (frontend uses fetch_page + spawn_blocking); 022 testing note 2026-03-16 (integration/smoke closing).
- **Brave Search API** — API compliance and error-handling/edge-cases documented in `docs/008_brave_agent.md`; empty-query guard in `brave_web_search` (trim, reject empty/whitespace); FEATURE-CODER and agent_workflow open tasks marked done. OpenClaw §18 re-verification (005) and 022 testing note (2026-03-16) added.
- **Docs 005 (§17), 006, 022** — OpenClaw re-verification §17 (005); FEATURE-CODER backlog row for 022/023 merge done (006); 022 §8 "Externalized prompts (F11) — summary from 023" and open task closed.
- **Docs 005, 006, 022, 033** — OpenClaw re-verification §16 (005-openclaw-reviewer); FEATURE-CODER backlog row for 033 prefer_headless verification done (006); 022 testing note 2026-03-16 (integration/smoke); 033 prefer_headless edge cases and verification section plus open task closed.
- **user-info.json caching** — In-memory cache with file mtime invalidation in `user_info/mod.rs`: reads use cache when file unchanged; writes refresh cache so next read sees new data; external edits to the file trigger reload. Open task in `docs/007_discord_agent.md` and FEATURE-CODER backlog marked done; `docs/data_files_reference.md` and 022 testing note (2026-03-16) updated. OpenClaw re-verification (§15) added in `005-openclaw-reviewer/005-openclaw-reviewer.md`.
- **test_discord_connect duration** — Run duration configurable via env `TEST_DISCORD_CONNECT_SECS` (1–300) or CLI (second arg, or single numeric arg for default path + duration); default 15s. Docs: §12 in `docs/007_discord_agent.md`; FEATURE-CODER backlog and 007 open task marked done. OpenClaw re-verification (§14) and 022 testing note (2026-03-16) added.

### Added
- **005-openclaw-reviewer** — OpenClaw docs/code/defaults review (`005-openclaw-reviewer/005-openclaw-reviewer.md`): scope, doc/code/defaults verdicts, recommendations.
- **Heise schedule script** — `scripts/add-heise-schedule.sh` and `scripts/heise-schedule-entry.json` to add a daily Heise.de summary schedule to `~/.mac-stats/schedules.json`.
- **Scheduler failure → Discord** — When a scheduled task fails (FETCH_URL, BRAVE_SEARCH, Ollama, or TASK run), the scheduler sends a short failure message to the schedule’s Discord channel when `reply_to_channel_id` is set. `execute_task` now returns `Result<Option<(String, bool)>, String>`; loop handles `Err(msg)` and posts to Discord. See `docs/009_scheduler_agent.md`, `scheduler/mod.rs`.
- **View logs in Settings** — Discord/Settings section has a **View logs** button that opens `~/.mac-stats/debug.log` in the default app (macOS). Tauri commands: `get_debug_log_path`, `open_debug_log`. See `docs/007_discord_agent.md` and FEATURE-CODER backlog.
- **maxSchedules config** — Optional cap on number of schedule entries via `maxSchedules` in `~/.mac-stats/config.json` (1–1000; omit or 0 = no limit). When at cap, new SCHEDULE adds are rejected with a message to remove some or increase the limit. See `Config::max_schedules()`, `docs/007_discord_agent.md` (§ Customizing SCHEDULE behavior).
- **user-info.json display_name auto-sync** — When a user messages in Discord, the app updates (or adds) their `display_name` in `~/.mac-stats/user-info.json` so the file stays in sync with Discord; new users get a minimal entry. See `docs/007_discord_agent.md` and `user_info::maybe_update_display_name_from_discord`.
- **006-feature-coder** — Feature-coder workflow and FEAT backlog notes (`006-feature-coder/FEATURE-CODER.md`).
- **Discord platform formatting** — When replying in Discord, the system prompt includes "Platform formatting (Discord)": no markdown tables (use bullet lists), wrap links in `<>` to suppress embeds. Keeps messages readable and reduces embed clutter.
- **Discord group channel guidance** — For guild channels (having_fun, all_messages, mention_only): when to speak, at most one substantive reply per message (no triple-tap), and do not expose the user's private context in the channel. Documented in `docs/007_discord_agent.md`.
- **REACT: emoji in having_fun** — When the model replies with only `REACT: <emoji>` (e.g. `REACT: 👍`), the bot adds that emoji as a reaction to the last user message and does not send text. One reaction per message; group-chat guidance explains when to use it.
- **Cookie banner auto-dismiss** — After `BROWSER_NAVIGATE`, the browser agent looks for a button/link whose text matches patterns in `~/.mac-stats/agents/cookie_reject_patterns.md` (user-editable, one pattern per line; default includes "reject all", "ablehnen", "only necessary", etc.) and clicks it to dismiss the cookie banner. New default file `src-tauri/defaults/cookie_reject_patterns.md`.
- **Lean Chrome processes** — Serialized browser creation via `LAUNCH_MUTEX` so only one thread can launch headless Chrome at a time (avoids multiple Chrome PIDs from races). On startup, orphaned headless Chrome processes (from previous runs or races) are killed via `kill_orphaned_browser_processes()`. Plan doc: `docs/032_chrome_helper_processes_plan.md`.
- **Daily log rotation** — Once per calendar day (UTC), `debug.log` is copied to `debug.log_sic` and truncated. Last rotation date stored in `~/.mac-stats/.debug_log_last_rotated`. Config paths: `Config::debug_log_sic_path()`, `Config::debug_log_last_rotated_path()`.

### Changed
- **Agent workflow docs** — `docs/agent_workflow.md`: "How invocations work" section, full tool table (invocation, purpose, implementation), See also links (README, 007, 100_all_agents). FEATURE-CODER backlog row marked done.
- **022 feature review** — `docs/022_feature_review_plan.md`: closing testing note (2026-03-16) with integration and smoke check summary.
- **Scheduler deduplication** — One-shot schedules (`at` + task) now deduplicate like cron: adding the same `at` and same task (normalized) returns `AlreadyExists` and is not added. See `add_schedule_at`, `docs/data_files_reference.md`, `docs/009_scheduler_agent.md`.
- **Docs 007 and FEATURE-CODER** — §12 test_discord_connect expanded (token resolution, env file format DISCORD-USER1/USER2-TOKEN, success/failure output); open task and FEATURE-CODER backlog row marked done.
- **Docs 033** — Mark "Stale Branch" open task as done in `docs/033_docs_vs_code_review.md`.
- **Docs 033 / 006-feature-coder** — RUN_CMD allowlist documented in 033 Fixes; open tasks cleaned (stale branch, docs sync done); FEATURE-CODER backlog table: removed completed "Stale Branch" row.
- **Ollama HTTP client reuse** — `send_ollama_chat_messages` now uses the stored `OllamaClient`'s HTTP client (with app timeout from `Config::ollama_chat_timeout_secs()`) instead of creating a new `reqwest::Client` per request. `OllamaConfig` supports optional `timeout_secs`; configure_ollama passes it when building the client. See `docs/006_roadmap_ai_tasks.md`.
- **Session reset instruction** — Session startup text now says "greet the user briefly" instead of "respond to the user" for a shorter first reply.
- **Having_fun group-chat guidance** — Having_fun (and idle thoughts) now include explicit guidance: know when to speak, one response per message, use REACT when a full reply isn't needed, participate without dominating.
- **Docs 007, 022, 006** — 007: user-info auto-update and maxSchedules customization described; open tasks marked done. 022: closing review (§9) with integration checklist, F1–F10 notes, smoke test, D1/D4. FEATURE-CODER: user-info auto-update and SCHEDULE/REMOVE_SCHEDULE customization backlog rows marked done.
- **Docs backlog trim** — Trimmed completed open tasks from 007; FEATURE-CODER backlog: "Trim stale Open tasks" done, 006 points to FEATURE-CODER; docs README notes trim and single backlog location.
- **006 roadmap and FEATURE-CODER** — Open tasks in `docs/006_roadmap_ai_tasks.md` point to single FEAT backlog in `006-feature-coder/FEATURE-CODER.md`; backlog table and remaining items (Mail, WhatsApp, Google Docs) updated.
- **Data files reference** — New `docs/data_files_reference.md` documents `schedules.json` and `user-info.json` (paths, JSON structure, fields, local-time interpretation for cron/at). 007 and 009 open tasks for docs and cron timezone marked done.
- **Docs 029 and FEATURE-CODER** — New § "Connection process (step-by-step)" in `docs/029_browser_automation.md` (session lookup, port check, connect/launch, session clear on error, idle timeout). Open task and FEATURE-CODER backlog row for BROWSER_* connection docs marked done.

### Fixed
- **ellipse() edge case** — `logging::ellipse()` enforces `max_len >= sep_len + 1` so first_count/last_count never go negative for very small `max_len`.

## [0.1.43] - 2026-03-18

### Added
- **Main-session memory (in-app)** — `~/.mac-stats/agents/memory-main.md` for the CPU window chat (no Discord channel). Loaded and searched like per-channel Discord memory so the main session has persistent context. `Config::memory_file_path_for_main_session()`; `load_main_session_memory_block()` and integration in `load_memory_block_for_request` and `search_memory_for_request` in `commands/ollama.rs`. Docs: 035 memory injection §, data_files_reference § "Memory files (agents)".

### Changed
- **Docs 005, 006, 022, 035, data_files_reference** — OpenClaw §41 re-verification (005); FEATURE-CODER backlog: per-channel memory in non-Discord contexts and new-topic/compaction items done (006); 022 testing note (2026-03-18); 035 main-session memory in memory injection §; data_files_reference new § "Memory files (agents)".

## [0.1.42] - 2026-03-17

### Added
- **Ollama chat streaming** — CPU window Ollama chat streams response chunks to the UI: backend `send_ollama_chat_messages_streaming` (NDJSON stream, `stream: true`); frontend listens for `ollama-chat-chunk` and appends to the last assistant message for incremental display. Request supports `stream: true` (default). See `commands/ollama.rs`, `src/ollama.js`.

### Changed
- **Docs 004, 005, 006, 022** — Notes and backlog updates; OpenClaw re-verification (005); FEATURE-CODER and 022 feature review plan (2026-03-17).

## [0.1.41] - 2026-03-16

### Changed
- **Discord API error handling** — When the Discord API is unavailable (connection/timeout/5xx), the app returns a short user-facing message ("Discord API is temporarily unavailable (connection or timeout). Try again in a moment.") and retries once after 2s in `discord_api_request`; `send_message_to_channel` and multipart send use the same message. See `discord/api.rs`, `discord/mod.rs`, and `docs/007_discord_agent.md`.

## [0.1.40] - 2026-03-15

### Added
- **Same-domain navigation timeout (optional)** — When the navigation target is on the same domain as the current page (e.g. in-site link or SPA), a shorter wait can be used. Config: `config.json` key `browserSameDomainNavigationTimeoutSecs` or env `MAC_STATS_BROWSER_SAME_DOMAIN_NAVIGATION_TIMEOUT_SECS`. When set, same-domain BROWSER_NAVIGATE uses this timeout; cross-domain and BROWSER_GO_BACK use `browserNavigationTimeoutSecs`. Range 1–120s; when not set, all use the single navigation timeout.

### Changed
- **Docs 029** — Same-domain shorter timeout for BROWSER_NAVIGATE documented in `docs/029_browser_automation.md`.

## [0.1.39] - 2026-03-10

### Added
- **Browser navigation timeout** — Maximum wait for BROWSER_NAVIGATE and BROWSER_GO_BACK is configurable: `config.json` key `browserNavigationTimeoutSecs` (default 30, range 5–120) or env `MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS`. Slow or stuck navigations fail with a clear message (e.g. "Navigation failed: timeout after 30s") instead of hanging.
- **BROWSER_NAVIGATE new_tab** — Add `new_tab` after the URL (e.g. `BROWSER_NAVIGATE: https://example.com new_tab`) to open the URL in a new tab and switch focus to it; subsequent BROWSER_CLICK / BROWSER_SCREENSHOT apply to that tab.
- **BROWSER_GO_BACK** — New agent tool: go back one step in the current tab's history and return the new page state. Use when returning to the previous page without re-entering the URL.

### Changed
- **Docs 029** — Navigation timeout, new tab, and BROWSER_GO_BACK documented in `docs/029_browser_automation.md`.

## [0.1.38] - 2026-03-08

### Added
- **Cursor-agent handoff** — When completion verification fails (local model didn’t satisfy the request), the router hands off to the cursor-agent CLI with the original user request and returns that result instead of only appending a disclaimer. Applies to any task type (e.g. news, La Vanguardia / lavanguardia.es, browser/screenshot, coding). See `docs/031_cursor_agent_handoff.md`.
- **AGENT: cursor-agent proxy** — When cursor-agent is on PATH, it is listed as an available agent; the model can reply `AGENT: cursor-agent <task>` and the router runs the CLI (no Ollama) and injects the result.

### Changed
- **Session memory in Discord** — Global (personal) memory is loaded only for main session (in-app chat or Discord DM). In Discord guild channels and having_fun, only per-channel memory is loaded to avoid leaking personal context into server channels. Agents use `combined_prompt_without_memory` when `include_global_memory` is false.

## [0.1.37] - 2026-02-28

### Changed
- **Perplexity news tool suffix** — Extracted news-result suffix logic into `build_perplexity_news_tool_suffix()` (hub-only warning, article preference guidance, refined-query/filtered hints). Unit tests added for hub-only vs article-like behavior.

## [0.1.36] - 2026-03-07

### Added
- **Discord having_fun: casual-only prompt** — Having_fun channels always use the casual-only system prompt; channel `agent` override in `discord_channels.json` is ignored for having_fun so the persona stays consistent (no work/Redmine soul). Optional channel `prompt` and time-of-day guidance still apply.
- **Session compaction for having_fun** — For Discord having_fun channels, compaction skips the LLM and returns a fixed minimal context so we never invent task or platform themes (e.g. "language learning") from casual chat. Exposes `is_discord_channel_having_fun(channel_id)` for the compactor.
- **Planning: current date and multi-tool sequence** — Planning prompt now includes current date (UTC). Plans like `RUN_CMD: date then REDMINE_API GET /time_entries.json?...` are normalized and executed as separate steps in sequence (not one RUN_CMD with the whole chain).
- **Discord: filter failure notices from history** — Agent/LLM failure notices (e.g. "Agent failed before reply", "Something went wrong on my side") are filtered out of having_fun channel history and idle-thought context so the model is never asked to "reply" to an error line.

### Changed
- **Discord docs** — Bot permissions (Send Messages, View Channel, Attach Files) and having_fun behavior (casual-only, error filtering, no agent override) documented in `docs/007_discord_agent.md`. Tool loop and multi-tool sequencing in `docs/021_router_and_agents.md`. Planning prompt and session compaction docs updated.

## [0.1.35] - 2026-03-07

### Changed
- Release 0.1.35.

## [0.1.34] - 2026-03-07

### Added
- **Agent test per-prompt timeout** — `mac_stats agent test` now enforces a 45s (configurable) timeout per prompt so a stuck or overloaded model fails fast instead of hanging. Config: `agentTestTimeoutSecs` in config.json or env `MAC_STATS_AGENT_TEST_TIMEOUT_SECS`. Regression tests added for timeout behavior and `testing.md` prompt parsing.
- **Agent test regression path in docs** — Documented how to run `mac_stats agent test <selector>` as a regression path in `docs/README.md` (Testing & Validation) and `docs/007_discord_agent.md` (§15), including timeout and override.
- **News hub-only verification tests** — Unit tests for `verification_news_hub_only_block`: hub-only block included when search was hub-only and question is news-like; empty when not news query or when not hub-only.

### Fixed
- **Agent test hang** — The Redmine (and any other) agent test no longer blocks indefinitely on the first Ollama call; the harness aborts the prompt task and returns a clear timeout error with override instructions.
- **News verification when search returns only hubs** — When a news-style PERPLEXITY_SEARCH returns only hub/landing/tag/standings pages (no article-like results), completion verification now instructs the verifier not to accept an answer that presents them as complete news; the model is told article-grade results were not found and may retry or state so.

### Changed
- **Clippy cleanups (ollama)** — Removed redundant local, use `is_some_and` for conversation check, replace `ticket_id.is_some() + unwrap()` with `if let Some(id) = ticket_id.filter(...)` in Redmine pre-route.

## [0.1.33] - 2026-03-07

### Added
- **Grounded browser retry coverage** — Added focused tests for browser navigation target parsing, browser-task detection, and retry prompt grounding so browser regressions around invented URLs and stale element indices are easier to catch.

### Changed
- **Browser retry grounding** — Browser retries now carry the latest real `Current page` / `Elements` snapshot back into the prompt so follow-up browser steps stay grounded in actual page state instead of drifting into invented navigation targets.
- **Documentation refresh** — Updated active docs to better reflect the current Redmine, browser automation, Ollama context, session memory, and defaults-merge behavior while trimming older stale backlog notes.

### Fixed
- **Browser action fallback behavior** — `BROWSER_CLICK` and `BROWSER_INPUT` no longer fall through to weaker HTTP fallbacks for agent-generated argument mistakes like stale indices or missing numeric targets; those errors now return grounded guidance tied to the latest browser state.
- **Browser navigation argument parsing** — `BROWSER_NAVIGATE` handling now rejects placeholder tokens like `to` or `video` and only accepts concrete URL-like targets, which avoids fake site failures caused by model-invented navigation arguments.

## [0.1.32] - 2026-03-06

### Changed
- **Docs backlog cleanup** — Normalized active `Open tasks` sections into concrete backlog bullets, removed stale placeholder TODOs from historical docs, and cleaned completed docs so `_DONE` files no longer advertise unfinished work.

### Fixed
- **Completion criteria sanitization** — Generic news requests and browser-based video review requests now reject invented verification criteria more aggressively, which reduces retries that drift into unrelated football/source requirements or fake “playable video” expectations.
- **Redmine failure parsing** — Grounded Redmine error handling now recognizes more backend failure text forms (`Redmine API failed`, `Redmine GET failed`, `Redmine request failed`) so blocked-state replies stay user-facing even when the raw error wording differs.

### Added
- **Request-local retry guards for Discord/Ollama runs** — Verification retries now carry the original user request and sanitized success criteria explicitly so unrelated prior task context does not leak into fresh requests.
- **News/search result shaping** — News-style Perplexity results are now ranked, deduplicated by domain, annotated as `article-like` vs `hub/landing page`, and retried with a refined query when the first pass only returns weak landing pages.
- **Browser search fallback tests** — Added focused coverage for plain-text fallback matching so `BROWSER_SEARCH_PAGE` can return useful results or a clean “no matches found” response instead of failing internally.

### Changed
- **Session memory normalization** — Discord/session history now stores only conversational user/final-assistant content, filtering out intermediate answer wrappers and other internal execution artifacts before persistence or reload.
- **News verification behavior** — News completion checks now avoid inventing source-brand requirements or attachment requirements that the user never asked for, and retry prompts stay in search-and-summary mode instead of drifting into unrelated browser work.
- **Documentation refresh** — Updated the current plan/docs set to reflect the request-isolation work, browser/search behavior, and recent agent/router changes.

### Fixed
- **Barcelona/news retry contamination** — Generic news requests no longer reuse stale Redmine-style success criteria during verification retries, which removes the earlier cross-topic retry failure mode.
- **`BROWSER_SEARCH_PAGE` no-value failure** — When the JS walker returns no structured payload, the browser agent now falls back to plain page text search and returns either contextual matches or a normal “no matches found” result rather than aborting the browser run.
- **Amvara browser review flow** — Live testing against `www.amvara.de` now reaches the `About` page reliably and reports the actual current finding: the “videos” entry is present, but no confirmed playable video content is exposed there.

## [0.1.31] - 2026-03-06

### Fixed
- **Redmine worked-today ticket listing** — `time_entries` queries now use date-ranged Redmine API calls without the broken implicit `user_id=me` filter, so “tickets worked on today” returns the real entries from Redmine instead of false-empty results on this server.
- **Redmine time-entry parsing** — The backend now parses paginated `/time_entries.json` responses, groups entries by issue, backfills missing issue subjects via `/issues/{id}.json`, and produces deterministic ticket summaries from Redmine data instead of relying on the model to infer issue lists from raw JSON.
- **Redmine router follow-up handling** — For normal ticket-list questions, the Ollama router now returns the derived Redmine time-entry summary directly instead of doing an unnecessary second LLM summarization pass, which removes another source of wrong or slow worked-ticket replies.

## [0.1.30] - 2026-03-06

### Added
- **Redmine time-entry prompts for “today”** — The Redmine agent, planner prompt, and Ollama router now support “worked on today / tickets worked today” with direct same-day `REDMINE_API: GET /time_entries.json?from=YYYY-MM-DD&to=YYYY-MM-DD` calls.
- **Docs for recent backend behavior** — Added follow-up docs for Redmine review hallucination fixes, prompt/tool scaling, review-only Redmine behavior, project rename planning, and sending finished task summaries back to Discord.

### Changed
- **Redmine time-entry execution** — Time-entry calls no longer default to `user_id=me`; optional filters are only added when explicitly needed, and the planner/router now prefer directly executable date-ranged Redmine calls instead of chaining `RUN_CMD` just to derive dates.
- **Agent-safe tool parsing** — Specialist agent tool parsing now reuses the main router’s normalization so inline chains and `RECOMMEND:` wrappers are handled more reliably for agent-safe tools like `REDMINE_API`.
- **Documentation refresh** — Large cleanup and rewrite across README and docs to better match current code, defaults, session/memory behavior, browser automation, Redmine flows, and agent capabilities.

## [0.1.29] - 2026-03-06

### Added
- **Redmine specialist agent** — New default agent `agent-006-redmine` for Redmine ticket review/search/create/update via `REDMINE_API` only. The orchestrator now routes Redmine work to this agent by default.
- **Redmine time-entry flow** — Time-entry requests are recognized explicitly and routed to `GET /time_entries.json` with current-month date ranges instead of generic search endpoints.
- **Task finished summary to Discord** — When a task run has a reply-to Discord channel, the finished summary is sent back to that channel automatically.
- **Session reset phrases** — New bundled `session_reset_phrases.md` supports “clear session / new topic” style resets so Discord sessions can start fresh on request.

### Changed
- **Memory and session handling** — Global and per-channel memory are loaded separately, searched for relevant lines, and injected more selectively. Session compaction, new-topic detection, and retry prompts now avoid polluting replies with unrelated prior context.
- **Redmine review safety** — Review-only Redmine requests are handled separately from update flows so ticket summaries don’t accidentally drift into modification behavior. Redmine responses are summarized from API data only.
- **Discord / Ollama routing** — Image-only Discord messages now use a default vision prompt, criteria/status handling is cleaner, and Discord context/session flow is more consistent across retries and topic changes.
- **Browser and command flow** — Browser/session status, fallback handling, and command execution paths were tightened across `browser_agent`, `run_cmd`, monitors, scheduler, and task runner.
- **Backend maintenance** — Broad cleanup/refactor across config, metrics, FFI, logging, MCP, plugins, alerts, monitors, and agent/task plumbing; release includes the tested `src-tauri` backend changes only.

## [0.1.28] - 2026-03-04

### Added
- **Prompt merge on startup** — `planning_prompt.md` and `execution_prompt.md` under `~/.mac-stats/prompts/` are now merged with bundled defaults when they already exist: new paragraphs from defaults (identified by first-line key) are appended so new sections (e.g. "Search → screenshot → Discord") propagate without overwriting user edits. See `docs/024_mac_stats_merge_defaults.md`.
- **Discord guild/channel metadata for discord-expert** — When routing to the discord-expert agent from Discord, the app fetches current guild and channel info via the Discord API and injects it into the prompt (channel_id, guild_id, guild name, channel list) so the agent can use correct IDs in DISCORD_API calls without an extra round-trip. New `fetch_guild_channel_metadata()` in `discord/api.rs`.
- **PERPLEXITY + auto-screenshot flow** — If the user asks for screenshots (e.g. "screenshot", "visit", "send me in Discord"), after a Perplexity search the app auto-visits the first 5 result URLs, takes a screenshot of each, attaches them in Discord (ATTACH protocol), and tells the model they were attached. Perplexity max_results increased to 15 for search.
- **Search query truncation for chained tools** — When the plan puts multiple tools on one line (e.g. `PERPLEXITY_SEARCH: spanish newspapers then BROWSER_NAVIGATE...`), only the search query is passed to PERPLEXITY_SEARCH and BRAVE_SEARCH via `truncate_search_query_arg()` so the query is not truncated incorrectly.

### Changed
- **Session compaction uses actual user question** — Periodic session compaction now uses the last user message in the session as the "question" for the compaction call instead of a generic "Periodic session compaction." string, improving summary relevance.
- **New-topic session handling** — When the new-topic check returns true, we set `is_new_topic` and clear prior context; on compaction skip we also replace the session with system + current user message so the next turn starts clean. Compaction "not needed" and new-topic both clear history consistently.
- **Discord API context text** — Agent context for Discord tasks now describes guild/channel data endpoints (GET /users/@me/guilds, GET /guilds/{id}/channels) and prefers AGENT: discord-expert for fetching guild/channel data autonomously.
- **Docs** — Agent task flow (020), Discord log review (027), browser loop/status plan (032), discord-expert skill (agent-004), planning prompt wording.

## [0.1.27] - 2026-03-03

### Added
- **Browser viewport configurable** — `config.json` keys `browserViewportWidth` and `browserViewportHeight` (defaults 1800, 2400; clamped 800–3840 and 600–2160). Used for headless launch, visible Chrome `--window-size`, and tab `set_bounds` when connecting to existing Chrome.
- **Discord status: edit criteria message** — Send "Extracting success criteria…" then when done edit that message to "Extracted success criteria: &lt;text&gt;" (EDIT: protocol) so the channel shows one updated message instead of two.
- **Discord status: attach screenshot immediately** — In verbose mode, when a screenshot is taken we send it to the channel right away (ATTACH: protocol); final reply no longer re-attaches the same screenshots.
- **Discord: image-only messages** — If the user sends only image attachment(s) and no text, we use a default prompt ("What do you see…") and pass images to Ollama vision.
- **Discord: session reset by request** — When the user asks to clear or start a new session (phrases in `session_reset_phrases.md`), we clear that channel's conversation and start fresh. See docs/035.
- **Memory search for requests** — Global and per-channel memory are searched for lines relevant to the question; up to 5 matching lines are injected into the prompt when useful.
- **MEMORY_APPEND in Discord** — In Discord, plain `MEMORY_APPEND: &lt;lesson&gt;` now saves to that channel's memory file (`memory-discord-{id}.md`); non-Discord still uses global memory.

### Changed
- **Status messages: no trailing ellipsis** — "Clicking element N (label)" and "Taking screenshot of current page" no longer end with "…".
- **Browser viewport size** — Default 1800×2400 for headless, visible launch, and when connecting to existing Chrome (set_bounds on first tab). Configurable via config.json (see above).
- **README** — Perplexity (PERPLEXITY_SEARCH, optional network, config tree, Chat). Optimized: single Install section, deduplicated CPU/stats, Commands/Dev tightened. Menu bar copy tightened.

## [0.1.26] - 2026-02-27

### Changed
- **Headless when from_remote** — For Discord, scheduler, and `discord run-ollama`, browser runs use headless unless the question explicitly asks to see the browser (`wants_visible_browser`). When `from_remote` is true, `prefer_headless = !wants_visible_browser(question)`; `ensure_chrome_on_port` skips launching visible Chrome when headless was requested for the run.
- **Docs** — CLAUDE, README, agents, 007_discord_agent, 100_all_agents, docs/README.

### Added
- **Ollama timeout/503 retry and user message (task-001)** — `send_ollama_chat_messages` retries once after 2s on timeout or HTTP 503; after retry still failing returns "Ollama is busy or unavailable; try again in a moment." Periodic session compaction retries once after 3s on failure before logging WARN.
- **FETCH_URL URL validation (task-002)** — `extract_first_url()` in browser.rs; `validate_fetch_url()` enforces http/https and clear IDN error. Used in `fetch_page_content`, `parse_fetch_url_from_response`, and scheduler FETCH_URL.
- **Browser tool cap** — Max 15 browser tools per run; BROWSER_INPUT status shows element label when available. See docs/032.
- **Scheduler log (task-004)** — "Scheduler: loaded N entries" at DEBUG. **Session compaction log (task-005)** — "keeping full history (N messages)" on failure.
- **Clippy** — thread_local const, div_ceil/first/range contains, casts, closures, needless borrows, collapsible else-if; `cargo clippy --fix` batch; drop unused CStr, unnecessary unsafe, unused var. ModelCatalog: removed unused `eligible()`.

## [0.1.25] - 2026-03-02

### Changed
- **Completion verification uses browser-rendered content** — When the model ran BROWSER_EXTRACT, the last extracted page text (JS-rendered) is now passed into completion verification so the verifier can check requested text (e.g. "rhythem") against real page content instead of FETCH_URL HTML (SPA shell). Fixes false "text not found" on SPAs like amvara.de.
- **CDP navigation wait non-fatal for SPAs** — If `wait_until_navigated` fails (e.g. hash-only or in-app navigation), we log a warning, sleep 2s, and continue instead of failing. Avoids "Wait navigated: The event waited for never came" on hash-routed sites; BROWSER_NAVIGATE no longer falls back to HTTP unnecessarily.
- **Session reset recovery** — When the CDP session is lost (timeout, Transport loop, TargetDestroyed) the next action may run in a new browser on `chrome://newtab/`. We now detect new-tab/blank and return a clear error: "Browser session was reset; current page is a new tab. Use BROWSER_NAVIGATE: <your target URL> first to reopen the page, then retry." so the model can re-navigate instead of clicking/screenshotting the wrong page. Applied to BROWSER_CLICK, BROWSER_INPUT, BROWSER_SCREENSHOT (current), BROWSER_EXTRACT, BROWSER_SCROLL, BROWSER_SEARCH_PAGE. Also treat "Transport loop" timeout as a connection error so we clear and retry.

## [0.1.24] - 2026-03-02

### Added
- **DONE tool (browser-use style)** — Model can end a reply with **DONE: success** or **DONE: no**; we exit the tool loop (no further tool runs), strip the DONE line from the final reply, then run completion verification as usual. Described in agent base tools and planning prompt. See `docs/025_expectation_check_design_DONE.md`.
- **Completion verification** — At the start of each agent run we extract 1–3 success criteria from the user request; at the end we ask Ollama “Did we fully satisfy the request?” and, if not, retry once then append a short disclaimer if still not satisfied. Heuristic: if a screenshot was requested but none was attached, we add a note. See `docs/025_expectation_check_design_DONE.md`.
- **Escalation patterns (user-editable)** — Phrases that trigger “user is not satisfied” (stronger completion run, +10 tool steps) are now read from **~/.mac-stats/escalation_patterns.md**. One phrase per line; lines starting with `#` are comments. Edit the file to add your own triggers (e.g. “I don’t like your answer”, “You are stupid”) so the bot actually tries harder instead of just apologising. Default list includes “think harder”, “get it done”, “try again”, “no”, “nope”, etc. No restart needed — the file is read on each message. When we detect escalation, we append the user's phrase to the file if it's not already there (auto-add).
- **BROWSER_SCROLL** — Agent tool: scroll the current CDP page. Reply with `BROWSER_SCROLL: down|up|bottom|top` or `BROWSER_SCROLL: <pixels>`.
- **BROWSER_EXTRACT** — Agent tool: return visible text of the current CDP page (body innerText, truncated to 30k chars). Use after BROWSER_NAVIGATE/CLICK to get page content for the LLM.
- **HTTP-only browser fallback** — When Chrome/CDP is not available (e.g. port 9222), BROWSER_NAVIGATE / BROWSER_CLICK / BROWSER_INPUT / BROWSER_EXTRACT use HTTP fetch + HTML parsing; CLICK follows links or submits forms, INPUT fills form fields. No JavaScript execution.

### Changed
- **Status messages (emojis + context)** — Tool-run status in Discord/UI now includes emojis (🧭 🌐 🖱️ ✍️ 📜 📸 🔍 📄) and full context (e.g. "Navigating to \<url\>", "Clicking element N", "Typing into element N", "Scrolling direction", "Fetching page at \<url\>", "Searching page for pattern").
- **README** — Mastodon: optional network, .config.env, MASTODON_POST in Chat, Monitoring & alerts (Mastodon mentions/channels), Usage bullet (MASTODON_INSTANCE_URL, MASTODON_ACCESS_TOKEN). X.com note: "No X.com yet ;-) — let's see who implements it first."
- **Browser agent retry on connection error** — When CDP connection is stale (connection closed, timeout, "Unable to make method calls"), the app clears the cached session and retries once. All CDP entry points use this retry wrapper.
- **Browser-use style browser tools** — (1) **BROWSER_SCREENSHOT** only on current page — BROWSER_NAVIGATE first, then BROWSER_SCREENSHOT: current. (2) **BROWSER_SEARCH_PAGE: \<pattern\>** to search page text. (3) Pre-route "screenshot + URL" runs BROWSER_NAVIGATE + BROWSER_SCREENSHOT: current in sequence.
- **Logging for expectation check flow** — Added info/debug logs so `tail -f ~/.mac-stats/debug.log` shows: criteria extraction (count or “no criteria”), completion verification run (criteria + attachment count), verification result (passed / not satisfied with reason), retry-on-NO, disclaimer with reason, heuristic guard, escalation mode. Use `-vvv` for debug (extraction failure, raw verifier response, duplicate escalation pattern skip).
- **Task runner prompt** — Explicit hint to use CURSOR_AGENT for implement/refactor/add-feature/code tasks, then TASK_APPEND and TASK_STATUS.
- **Tool-first routing** — Pre-route "screenshot + URL" to BROWSER_SCREENSHOT (skip planner). Planning prompt: when one base tool fits, recommend that tool instead of AGENT. See `docs/031_orchestrator_tool_first_proposal_DONE.md`.

## [0.1.23] - 2026-03-02

### Added
- **Vision verification (screenshots)** — When a run has image attachment(s) (e.g. BROWSER_SCREENSHOT) and a local vision model is available, completion verification sends the first image (base64) to the vision model and asks "Does this image satisfy the request?"; fallback to text-only verification if no vision model or on vision call failure. See `docs/025_expectation_check_design_DONE.md`.

### Changed
- **Browser status messages** — "Navigating…" now shows the URL (e.g. "Navigating to https://…"); "Clicking…" now shows the element index (e.g. "Clicking element 3…").

## [0.1.22] - 2026-02-28

### Added
- **BROWSER_SCREENSHOT** — New agent tool: open a URL in a headless browser (CDP), take a screenshot, save to `~/.mac-stats/screenshots/`. Reply with `BROWSER_SCREENSHOT: <URL>`. In Discord, screenshot paths are sent as file attachments (only paths under `~/.mac-stats/screenshots/` are allowed). Requires Chrome with `--remote-debugging-port=9222` or use of headless Chrome via `browser_agent` module.
- **browser_agent** — CDP (Chrome DevTools Protocol) module: connect to Chrome, navigate, capture screenshot. Config: `Config::screenshots_dir()`, `Config::browser_idle_timeout_secs()` (default 1 hour).
- **Discord reply attachments** — `answer_with_ollama_and_fetch` returns `OllamaReply { text, attachment_paths }`. Discord sends allowed attachment paths (e.g. BROWSER_SCREENSHOT outputs) as message files. `send_message_to_channel_with_attachments` for CLI/API. Paths outside `~/.mac-stats/screenshots/` are rejected.
- **Security and secrets** — No-logging rule for credentials and `.config.env`: do not log file content or path. Security module docs: `get_credential`/`list_credentials` backend-only, never expose to frontend. Config doc on storing secrets in `~/.mac-stats/.config.env` or Keychain. RUN_CMD: cursor-agent documented as privileged (user/agent-controlled prompts).
- **README** — AI-first positioning: "The AI agent that just gets it done. All local." Features reordered: AI & agents first, UI, then system monitoring (background). Usage: Chat, Discord, Monitoring. Shorter copy and inspiration note.

### Changed
- **Agent tool list** — Base tools described as "7 base tools"; BROWSER_SCREENSHOT added between FETCH_URL and BRAVE_SEARCH. Session compaction: clearer error for 401/unauthorized (suggest local model for compaction).
- **Cargo** — reqwest `multipart` feature; deps `headless_chrome`, `regex`. Package description updated for AI-first wording.

## [0.1.21] - 2026-02-28

### Added
- **Discord having_fun per-channel model/agent**: In `discord_channels.json`, channels can set `model` (Ollama model override) and `agent` (agent id, e.g. `abliterated`) so having_fun uses that agent's soul+skill and model. When `agent` is set, the channel uses the agent's combined prompt and model; otherwise soul + optional channel `prompt` and `model` as before.
- **Discord having_fun configurable loop protection**: `having_fun.max_consecutive_bot_replies` in config (default 0). 0 = do not reply to bot messages; 1–20 = max consecutive bot messages before dropping (loop protection). Replaces hardcoded limit of 5.
- **Ollama chat timeout config**: `config.json` key `ollamaChatTimeoutSecs` (default 300, range 15–900). Env override `MAC_STATS_OLLAMA_CHAT_TIMEOUT_SECS`. Used for all Ollama /api/chat requests (UI, Discord, session compaction).
- **Model identity in prompts**: Agent router and having_fun system prompts now include "You are replying as the Ollama model: **<name>**" so the bot can answer "which model are you?" accurately. `get_default_ollama_model_name()` exposed for Discord/UI.
- **Default agent with soul**: New macro `default_agent_entry_with_soul!("id")` and default agent **abliterated** (`agent-abliterated/`: agent.json, skill.md, soul.md, testing.md) for having_fun channels that want a distinct persona.
- **docs/012_cursor_agent_tasks.md**: Cursor agent tasks documentation.

### Changed
- **having_fun**: Uses agent's soul+skill+model when channel has `agent`; otherwise unchanged (soul + channel prompt/model). Default `max_consecutive_bot_replies` 0 to avoid replying to other bots unless explicitly configured.
- **agents-tasks**: README clarifies log-NNN vs task-NNN, log path `~/.mac-stats/debug.log`; review docs and .gitignore use `agents-tasks` (fixed typo).

### Removed
- **OPTIMIZATION_PROGRESS.md** and **docs/OPTIMIZE_CHECKLIST.md**: Obsolete optimization checklists removed.

## [0.1.20] - 2026-02-27

### Added
- **Loop-protection visibility (log-007)**: Per-channel `loop_protection_drops` counter in having_fun state; incremented when a bot message is dropped; every 60s heartbeat logs `DEBUG Discord: loop protection: channel <id> dropped N message(s) this period` and resets counter. Use `-vv` to see summaries.

### Changed
- **Agent-tasks**: All log-001 through log-009 verified implemented; README and task files updated to status **done**. Log-002 (log rotation), log-003 (temperature N/A), log-004 (image 404), log-005 (Discord scope sanitize), log-006 (Ollama dedupe), log-007 (loop-protection visibility), log-008 (FETCH_URL redmine hint), log-009 (Redmine 422) confirmed in code.
- **Release**: Version 0.1.20; release build and app restart with `-vv` for verification.

## [0.1.19] - 2026-02-23

### Added
- **Redmine search API**: Keyword search for issues via `GET /search.json?q=<keyword>&issues=1&limit=100`. Documented in REDMINE_API tool description and in `docs/025_redmine_api_skill.md`. Use for "search/list tickets about X"; the issues list endpoint has no full-text search param.
- **Redmine create context**: When Redmine is configured, the app fetches projects, trackers, issue statuses, and priorities from the API, caches them for 5 minutes, and injects the list into the agent description so the model can resolve "Create in AMVARA" (or similar) to the correct `project_id`. See `docs/025_redmine_api_skill.md`.
- **Default agent macro**: New `default_agent_entry!("id")` macro in config; default agents are built from `DEFAULT_AGENT_IDS` so adding agent-004/005 (or more) is a single line. `Config::tmp_dir()` and `Config::tmp_js_dir()` for runtime scratch paths.
- **AGENTS.md restart-and-test rule**: After changes that affect runtime behavior (Redmine, tasks, agent prompts, scheduler, Discord, Ollama tools), restart mac-stats and test; do not assume it works without verification.
- **Merge-defaults doc**: `docs/024_mac_stats_merge_defaults.md` and agents.md section on updating `~/.mac-stats` from defaults (merge, do not overwrite).

### Changed
- **RUN_CMD logging**: Logs the exact command string (e.g. `RUN_CMD: executing: cat ~/.mac-stats/schedules.json`) and at entry the full argument line for debugging.
- **cargo default binary**: `default-run = "mac_stats"` in Cargo.toml so `cargo run -- -vv` works without `--bin mac_stats`.
- **Discord having_fun**: Casual-context constant for having_fun channels; channel config log moved to after having_fun state init; log line includes next response and next idle thought timing when having_fun channels exist.
- **Orchestrator skill**: REDMINE_API bullet now includes search endpoint and create-context note; task-create-and-assign flow documented for delegated coding tasks; RUN_CMD allowlist in skill.

## [0.1.18] - 2026-02-22

### Added
- **Task file naming**: New convention `task-<date-time>-<status>.md` (e.g. `task-20260222-140215-open.md`). Topic and id are stored in-file as `## Topic:` and `## Id:` for listing and resolution.
- **Task conversation logging**: When the agent touches a task (TASK_CREATE, TASK_APPEND, TASK_STATUS, etc.), the full user question and assistant reply are appended to the task file as a `## Conversation` block. Runner turns (synthetic "Current task file content..." prompts) are skipped.
- **Having_fun ASAP**: In having_fun channels, messages that are a mention or from a human trigger an immediate response (next tick) instead of the random delay.
- **Having_fun idle timer log**: The periodic "Having fun: idle timer" log now includes time until next response and next idle thought (e.g. `next response in 45s, next idle thought in 120s`). Logged about once a minute when there are having_fun channels.
- **Perplexity Search**: Optional web search via Perplexity API. Tauri commands `perplexity_search` and `is_perplexity_configured`; API key stored in Keychain (Settings). Use from Ollama/agents for real-time web search.

### Changed
- **Task resolution**: Resolve by full task filename (with or without `.md`), by short id (from `## Id:` in file), or by topic (from `## Topic:` in file). Legacy filenames (task-topic-id-datetime-status) still supported.
- **TASK_CREATE**: Rejects when topic looks like an existing task filename; sanitizes id (strips quotes/slashes). Deduplication checks `## Topic:` and `## Id:` in existing files.
- **TASK_APPEND / TASK_CREATE parsing**: Multi-line content is preserved (all lines until the next tool line), so research and long text are stored completely in the task file.
- **Having_fun flow**: Before replying, the app fetches the latest messages from Discord (after the bot's last response) and uses those as context for better flow. Falls back to the in-memory buffer if the API fetch fails.
- **Docs and memory**: All MD files and `~/.mac-stats/agents/memory.md` updated to document the new task naming (`task-<date>-<status>.md`, topic/id in-file). See `docs/013_task_agent.md`, `docs/021_task_system_fix_feb2026_DONE.md`, `docs/022_feature_review_plan.md`.

## [0.1.17] - 2026-02-22

### Added
- **Periodic session compaction**: Every 30 minutes a background thread compacts all in-memory sessions (≥ 4 messages) into long-term memory. Lessons are appended to global `memory.md`. Active sessions (activity within 30 min) are replaced with the summary; inactive sessions are cleared after compacting.
- **Session memory `last_activity` and `list_sessions()`**: Sessions now track last activity time. `list_sessions()` returns all in-memory sessions for the periodic compaction loop.
- **Having_fun configurable delays**: `discord_channels.json` supports a top-level `having_fun` block with `response_delay_secs_min/max` and `idle_thought_secs_min/max` (seconds). Each response or idle thought uses a random value in that range (e.g. 5–60 min). Default 300–3600. Config is reloaded when the file changes.
- **Having_fun time-of-day awareness**: The having_fun system prompt now includes current local time and period-aware guidance (night / morning / afternoon / evening) so the bot can behave differently by time of day (e.g. shorter and calmer at night, more energetic in the morning).
- **Discord channels config upgrade**: If `~/.mac-stats/discord_channels.json` exists but has no `having_fun` block, the app adds the default block on load and writes the file back.
- **Chatbot avatar assets**: SVG (and optional PNG) avatar icon for mac-stats chatbot in `src/assets/`.
- **Discord send CLI**: Subcommand `mac_stats discord send <channel_id> <message>` to post a message to a Discord channel (uses bot token from config). Useful for scripts and wrap-up notifications.

### Changed
- **Session compaction**: On-request compaction unchanged (≥ 8 messages); periodic compaction uses a lower threshold (4 messages) so more conversations are flushed to long-term memory.
- **docs/session_compaction_and_memory_plan_DONE.md**: Updated to document implemented behavior (30-min loop, last_activity, time-of-day).

## [0.1.16] - 2026-02-21

### Added
- **Discord channel modes** (`~/.mac-stats/discord_channels.json`): Per-channel listen configuration with three modes:
  - `mention_only` (default) — respond only to @mentions and DMs
  - `all_messages` — respond to every human message, no @mention required
  - `having_fun` — respond to everyone including other bots, with 30s buffered responses, idle thoughts after 5min silence, and loop protection (max 5 consecutive bot exchanges)
- **Per-channel prompt injection**: Channels support an optional `prompt` field that shapes response style (e.g. "be casual, no bullet points, never offer help"). Injected into the system context for that channel only.
- **Discord typing indicator**: Werner_Amvara now shows "is typing..." while processing a message. Fires immediately and refreshes every 8s until the reply is ready.
- **Verbose mode for Discord**: Status/thinking messages (e.g. "Asking Ollama for a plan...") are suppressed by default to keep channels clean. Add `verbose` as a header line to see them.
- **Bot mention stripping**: The `<@BOT_ID>` tag is now removed from message content before processing, so Ollama receives a clean question.
- **Session compaction**: When conversation history exceeds 8 messages, it is automatically compacted using a fast model (small role). Extracts verified facts and successful outcomes, drops failed attempts and hallucinations. Lessons learned are appended to global `memory.md`.
- **Session memory `replace_session()`**: Persists old session to disk and replaces in-memory history with compacted summary.
- **Discord Expert agent** (agent-004): Specialized agent for Discord API operations with its own tool loop and memory.
- **Persistent memory system**: Global (`memory.md`) and per-agent memory files loaded into every agent's prompt. `MEMORY_APPEND` tool for agents to write lessons learned.
- **Default `discord_channels.json`**: Shipped with the app via `ensure_defaults()`, with documentation and examples for all three modes.

### Changed
- **Discord bot ignores other bots** in `mention_only` and `all_messages` channels (prevents accidental bot-to-bot loops).
- **`having_fun` uses direct Ollama chat**: Bypasses the full planning/tools pipeline for faster, more conversational responses. Soul + channel prompt + history only.
- **FETCH_URL Discord intercept widened**: All `discord.com` URLs (not just `/api/`) are now intercepted and redirected to `DISCORD_API` or rejected with guidance to use the discord-expert agent.
- **Orchestrator skill.md**: Updated with Discord Expert delegation rules and DISCORD_API critical rules.

### Dependencies
- Added `tokio-util` (CancellationToken for typing indicator lifecycle).

## [0.1.15] - 2026-02-21

### Added
- **Dynamic model resolution for agents**: Agents now declare a `model_role` ("general", "code", "small") instead of hardcoding a model name. At startup, the app queries Ollama `/api/tags`, classifies all installed models by capability (Code vs General) and size tier (Small <4B, Medium 4-15B, Large >15B), and resolves each agent to the best available model. Models above 15B are excluded from auto-selection. Resolution is logged at startup for full visibility. The `model` field remains as an optional explicit override.
  - New module: `ollama/models.rs` with `ModelCatalog`, classification logic, and 7 unit tests
  - New field: `model_role` in `AgentConfig` / `Agent` structs and all CRUD commands
  - Default agent configs updated: orchestrator=general, coder=code, generalist=small
- **Redmine API agent**: Ollama can access Redmine issues, projects, and time entries via `REDMINE_API: GET /issues/1234.json`. Pre-routes ticket/issue questions directly to Redmine when configured. Configure via `REDMINE_URL` and `REDMINE_API_KEY` in env or `~/.mac-stats/.config.env`.
- **Discord "new session" command**: Type `new session: <question>` in Discord to clear conversation history and start fresh. Prior messages are persisted to disk before clearing.
- **Session memory `clear_session()`**: New function to flush and clear in-memory conversation history for a source/channel.
- **RUN_CMD dynamic allowlist**: The command allowlist is now read from the orchestrator agent's `skill.md` (section `## RUN_CMD allowlist`). Falls back to the default list if not configured. Added `cursor-agent` to default allowlist.
- **RUN_CMD pipe support**: Commands now support `cmd1 | cmd2 | cmd3` pipelines; each stage is validated against the allowlist independently.

### Changed
- **Agent default configs**: Shipped agent.json files use `model_role` instead of hardcoded `model` names. Existing user configs with explicit `model` continue to work (explicit model takes priority when available, falls back to `model_role` if the model is removed).

## [0.1.14] - 2026-02-19

### Added
- **Externalized prompts**: System prompts (`planning_prompt.md`, `execution_prompt.md`) and soul (`soul.md`) are now editable files under `~/.mac-stats/prompts/` and `~/.mac-stats/agents/`. Previously hardcoded as Rust constants. The execution prompt supports a `{{AGENTS}}` placeholder that is replaced at runtime with the dynamically generated tool list.
- **Default agents shipped**: Four default agents (orchestrator, general assistant, coder, generalist) are embedded in the binary via `include_str!` from `src-tauri/defaults/`. On first launch, `ensure_defaults()` writes all missing files (`agent.json`, `skill.md`, `testing.md` per agent, plus `soul.md` and prompts). Existing user files are never overwritten.
- **Tauri commands for prompt editing**: `list_prompt_files` returns name, path, and content of all prompt files; `save_prompt_file(name, content)` writes to a named prompt file. Available for frontend integration.
- **RUN_CMD retry loop**: When a local command fails (non-zero exit), the app sends the error to Ollama in a focused prompt asking for a corrected command. Retries up to 3 times. Handles cases where the model appends plan commentary to the command arg (e.g. `cat file.json then do X`).
- **Empty response fallback**: When Ollama returns an empty response after a successful tool execution, the raw tool output is returned directly to the user instead of showing nothing. Covers RUN_CMD, FETCH_URL, DISCORD_API, MCP, and search results.

### Fixed
- **Tool parsing: numbered list plans**: `parse_tool_from_response` now strips leading list numbering (`1. `, `2) `, `- `, `* `) and multiple nested `RECOMMEND:` prefixes. Previously, plans like `1. RUN_CMD: cat file.json 2. Extract...` were not recognized as tool calls.
- **Tool arg truncation**: When Ollama concatenates multiple plan steps on one line, the arg is now truncated at the next numbered step boundary (e.g. ` 2. `) so commands receive clean arguments.
- **RECOMMEND prefix stripping**: The recommendation from the planning step now has all `RECOMMEND:` prefixes stripped before being used in the execution system prompt and before tool parsing. Previously, the raw `RECOMMEND: RUN_CMD: ...` was passed to Ollama's execution step as `Your plan: RECOMMEND: RUN_CMD: ...`, which confused the model into returning empty responses.
- **Stale binary**: Ensured all code changes (fast-path tool execution, RECOMMEND stripping) are compiled into the running binary. Previous session's changes were only in source but not rebuilt.

### Changed
- **Prompts loaded from files**: `EXECUTION_PROMPT` and `PLANNING_PROMPT` are no longer Rust `const` strings. They are read from `~/.mac-stats/prompts/` at each request, so edits take effect immediately without rebuild.
- **`DEFAULT_SOUL` uses `include_str!`**: The default soul content is now a real Markdown file at `src-tauri/defaults/agents/soul.md`, embedded at compile time. Easier to read and diff than an inline Rust string literal.
- **`src-tauri/defaults/` directory**: All default content (soul, prompts, agents) lives as real `.md`/`.json` files in the repo, embedded via `include_str!`. Clean Markdown diffs in PRs.

## [0.1.13] - 2026-02-19

### Added
- **Task module and CLI**: All task logic centralized in `task/` (mod, runner, review, cli). Ollama and scheduler only call into the task module.
  - **CLI**: `mac_stats add|list|show|status|remove|assign|append` for testing and scripting (e.g. `mac_stats add foo 1 "Content"`, `mac_stats list --all`, `mac_stats assign 1 scheduler`).
  - **TASK_SHOW**: Show one task's status, assignee, and content to the user in the message channel (Discord/UI).
  - **Assignee**: Every task has `## Assigned: agent_id` (default `default`). **TASK_ASSIGN** reassigns to scheduler|discord|cpu|default. Review loop only picks tasks assigned to **scheduler** or **default**.
  - **TASK_STATUS** allows **unsuccessful** and **paused**. **TASK_SLEEP: &lt;id&gt; until &lt;ISO datetime&gt;** pauses until that time; review loop auto-resumes when time has passed.
  - **Dependencies**: `## Depends: id1, id2` in task file; review loop only picks tasks whose dependencies are finished or unsuccessful (**is_ready**).
  - **Sub-tasks**: `## Sub-tasks: id1, id2`; parent cannot be set to **finished** until all sub-tasks are finished or unsuccessful.
  - **Review loop**: Up to 3 open tasks per cycle, 20 iterations per task; auto-close as unsuccessful on max iterations; resume due paused tasks each cycle.
  - **task/runner.rs**: `run_task_until_finished` moved from ollama to task module; scheduler and review call `task::runner::run_task_until_finished`.
- **delete_task**: Remove all status files for a task (CLI `remove`, used by CLI only).
- **Discord session memory**: Discord bot now maintains short-term conversation context (last 20 messages per channel). The model can resolve references like "there", "it", etc. from prior turns. After app restart, context is resumed from the latest session file on disk.
- **Conversation history in agent router**: `answer_with_ollama_and_fetch` accepts optional `conversation_history` so Discord (and future entry points) can pass prior turns into planning and execution prompts.
- **Chat reserved words**: Type `--cpu` in chat to toggle the CPU window, or `-v`/`-vv`/`-vvv` to change log verbosity at runtime without restarting. New Tauri commands: `toggle_cpu_window`, `set_chat_verbosity`.
- **Background monitor checks**: Website monitors are now checked in a background thread every 30 seconds (by their configured interval), even when the CPU window is closed.
- **TASK_CREATE deduplication**: Creating a task with the same topic and id as an existing task now returns an error instead of silently creating duplicates.

### Fixed
- **Ollama model auto-detection at startup**: The app no longer hardcodes `llama2` as the default model. At startup, it queries `GET /api/tags` and picks the first available model. Frontend `getDefaultModel()` also queries installed models via `list_ollama_models`. Fallback is `llama3.2`.
- **Native tool-call parsing errors**: Models with built-in tool support (qwen3, command-r, etc.) caused Ollama to fail with "error parsing tool call" because Ollama tried to parse text tool invocations as JSON. Fixed by sending `"tools": []` in all chat requests, which disables Ollama's native tool-call parser.
- **Direct tool execution from plan**: When the planning step returns a recommendation that already contains a parseable tool call (e.g. `DISCORD_API: GET /users/@me/guilds`), the router now executes it directly instead of asking Ollama a second time. Saves one full Ollama round-trip per request.
- **Logging `ellipse()` helper**: Truncated text now shows first half + `...` + last half instead of hard truncation. Applied to Ollama request/response logs, FETCH_URL content, and Discord API responses.
- **Verbosity-aware logging**: At `-vv` or higher, Ollama request/response logs are never truncated.
- **Char-count vs byte-count**: Fixed Discord API response truncation to use `.chars().count()` instead of `.len()` for correct Unicode handling.

### Changed
- **Unified soul path**: Consolidated `~/.mac-stats/agent/soul.md` (router) and `~/.mac-stats/agents/soul.md` (agent fallback) into a single path: `~/.mac-stats/agents/soul.md`. Used by all agents (as fallback) and by the router (non-agent chat). The old `~/.mac-stats/agent/` directory is no longer used. **Migration**: move any customized `~/.mac-stats/agent/soul.md` to `~/.mac-stats/agents/soul.md`.
- **Task prompt guidance**: Agent descriptions now instruct the model to invoke `AGENT: orchestrator` (not just `TASK_CREATE`) when users want agents to chat, and to use `TASK_APPEND`/`TASK_STATUS` when a task with the same topic+id already exists.
- **Toggle CPU window refactored**: Extracted inline window toggle logic from the click handler into `toggle_cpu_window()` function, reusable from both menu bar clicks and the new `--cpu` chat command.
- **Task docs**: `docs/013_task_agent.md` rewritten — CLI, TASK_SHOW, assignee, TASK_ASSIGN, pause/sleep, dependencies, sub-tasks, module layout, review behaviour.

## [0.1.11] - 2026-02-09

### Added
- **SKILL agent documentation**: `docs/016_skill_agent.md` — SKILL tool invocation, logging, and future modify path. Agent descriptions sent to Ollama include enriched SKILL info for better recommendation; skills load is logged (info: available list; warn: read errors). See `docs/100_all_agents.md` (tool table, subsection 2.9).
- **SCHEDULE tool (one-shot and cron)**: The agent can add one-shot reminders and recurring tasks via SCHEDULE in three formats:
  - **Every N minutes**: `SCHEDULE: every N minutes <task>` (unchanged).
  - **Cron**: `SCHEDULE: <cron expression> <task>` — 6-field (sec min hour day month dow) or 5-field (min hour day month dow; app prepends `0` for seconds). Cron examples are injected into the agent context (e.g. every day at 5am, weekdays at 9am). See `docs/007_discord_agent.md`.
  - **One-shot (at datetime)**: `SCHEDULE: at <datetime> <task>` — run once at local time; datetime ISO `YYYY-MM-DDTHH:MM:SS` or `YYYY-MM-DD HH:MM`. Scheduler supports `add_schedule_at()` for one-shot entries in `~/.mac-stats/schedules.json`.

### Changed
- **SCHEDULE status**: Status line now shows a short preview of the schedule text while adding (e.g. "Scheduling: every 5 minutes…").

## [0.1.10] - 2026-02-09

### Added
- **Full Ollama API coverage**: List models with details, get version, list running models, pull/update/delete models, generate embeddings, load/unload models from memory.
  - Tauri commands: `list_ollama_models_full`, `get_ollama_version`, `list_ollama_running_models`, `pull_ollama_model`, `delete_ollama_model`, `ollama_embeddings`, `unload_ollama_model`, `load_ollama_model`. All use the configured Ollama endpoint (same as chat/Discord/scheduler).
  - Backend: `ollama/mod.rs` types and `OllamaClient` methods for GET /api/tags (full), GET /api/version, GET /api/ps, POST /api/pull, DELETE /api/delete, POST /api/embed, and load/unload via keep_alive on generate/chat.
  - Documentation: `docs/015_ollama_api.md`.
- **User info (user-info.json)**: Per-user details from `~/.mac-stats/user-info.json` (keyed by Discord user id) are merged into the agent context (display_name, notes, timezone, extra). See `docs/007_discord_agent.md`.
- **Task review loop**: Background loop every 10 minutes: lists open/wip tasks, closes WIP tasks older than 30 minutes as **unsuccessful** (appends note), then runs `run_task_until_finished` on one open task. Started at app startup. See `docs/013_task_agent.md`.
- **TASK_LIST tool**: Ollama can invoke `TASK_LIST` or `TASK_LIST:` to get the list of open and WIP task filenames under `~/.mac-stats/task/` (naming: `task-<date-time>-<status>.md`; topic and id are stored in-file).
- **Task status "unsuccessful"**: Task filenames can use status `unsuccessful`; review loop uses it for stale WIP timeouts.

### Changed
- **Agent status messages**: When the agent uses a skill or the Ollama API, the status line now shows details: "Using skill &lt;number&gt;-&lt;topic&gt;…" and "Ollama API: &lt;action&gt; [args]…".
- **README**: Features and Current Features sections updated to include all agents (Discord, MCP, Task, PYTHON_SCRIPT, Scheduler, Skills) and grouped by system monitoring, website & monitoring, AI & agents, UI.

## [0.1.9] - 2026-02-09

### Added
- **Discord API agent**: When a request comes from Discord, Ollama can call the Discord HTTP API via the DISCORD_API tool (e.g. list guilds, channels, members, get user). Endpoint list is documented in `docs/007_discord_agent.md` and injected into the agent context. Only GET and POST to `/channels/{id}/messages` are allowed.
- **Discord user names**: The bot records the message author's display name and passes it to Ollama so it can address the user by name; names are cached for reuse in the session.
- **MCP Agent (Model Context Protocol)**: Ollama can use tools from any MCP server
  - Configure via `MCP_SERVER_URL` (HTTP/SSE) or `MCP_SERVER_STDIO` (e.g. `npx|-y|@openbnb/mcp-server-airbnb`) in env or `~/.mac-stats/.config.env`
  - When configured, the app fetches the tool list and adds it to the agent descriptions; Ollama invokes tools by replying `MCP: <tool_name> <arguments>`
  - Supported in Discord bot, scheduler, and CPU window chat (same tool loop)
  - Documentation: `docs/010_mcp_agent.md`
- **Task agent**: Task files under `~/.mac-stats/task/` with TASK_APPEND, TASK_STATUS, TASK_CREATE. Scheduler supports `TASK: <path or id>` / `TASK_RUN: <path or id>` to run a task loop until status is `finished`; optional `reply_to_channel_id` sends start and result to Discord. Documentation: `docs/013_task_agent.md`.
- **PYTHON_SCRIPT agent**: Ollama can create and run Python scripts; scripts are written to `~/.mac-stats/scripts/` and executed with `python3`. Disable with `ALLOW_PYTHON_SCRIPT=0`. Documentation: `docs/014_python_agent.md`.

## [0.1.8] - 2026-02-08

### Added
- **Ollama context window and model/params**: Per-model context size via `POST /api/show`, cached; Discord can override model (`model: llama3.2`), temperature and num_ctx (`temperature: 0.7`, `num_ctx: 8192` or `params: ...`). Config supports optional default temperature/num_ctx. See `docs/012_ollama_context_skills.md`.
- **Context-aware FETCH_URL**: When fetched page content would exceed the model context, the app summarizes it via one Ollama call or truncates with a note. Uses heuristic token estimate (chars/4) and reserved space for the reply.
- **Skills**: Markdown files in `~/.mac-stats/skills/skill-<number>-<topic>.md` can be selected in Discord with `skill: 2` or `skill: code`; content is prepended to the system prompt so different “agents” respond differently.
- **Ollama agent at startup**: The app configures and checks the default Ollama endpoint at startup so the agent is available for Discord, scheduler, and CPU window without opening the CPU window first.

### Changed
- **Discord agent**: Reply uses full Ollama + tools pipeline (planning + execution). Message prefixes for model, temperature, num_ctx, and skill documented in `docs/007_discord_agent.md` and `docs/012_ollama_context_skills.md`.

## [0.1.7] - 2026-02-06

### Added
- **Discord Agent (Gateway)**: Optional Discord bot that connects via the Gateway and responds to DMs and @mentions
  - Bot token stored in macOS Keychain (account `discord_bot_token`); never logged or exposed
  - Listens for direct messages and guild messages that mention the bot; ignores own messages
  - Requires Message Content Intent enabled in Discord Developer Portal
  - Tauri commands: `configure_discord(token?)` to set/clear token, `is_discord_configured()` to check
  - Reply is currently a stub; Ollama/browser pipeline to be wired in a follow-up
  - Documentation: `docs/007_discord_agent.md`

## [0.1.6] - 2026-01-22

### Fixed
- **DMG Asset Bundling**: Fixed missing assets (Ollama icon, JavaScript/Tauri icons) in DMG builds
  - Added explicit `resources` configuration in `tauri.conf.json` to bundle `dist/assets/` files
  - Assets are now properly included in production DMG builds
- **Ollama Icon Path**: Fixed Ollama icon not displaying in DMG builds
  - Changed icon paths from relative (`../../assets/ollama.svg`) to absolute (`/assets/ollama.svg`) in all theme HTML files
  - Icons now resolve correctly in Tauri production builds
- **History Chart Initialization**: Fixed history charts not drawing in DMG builds
  - Moved canvas element lookup and context initialization to `initializeCanvases()` function
  - Added defensive initialization in `updateCharts()` to handle delayed DOM loading
  - Charts now properly initialize in production builds

### Added
- **DMG Testing Script**: Added `scripts/test-dmg.sh` for automated DMG verification before release
  - Mounts DMG and verifies app structure
  - Checks for required assets and theme files
  - Provides installation and testing instructions
  - Validates bundle correctness before distribution

### Changed
- **Test Script Path Detection**: Updated test script to check correct asset path (`dist/assets/` instead of `assets/`)

## [0.1.5] - 2026-01-22

### Changed
- **Release**: Version bump for release build

## [0.1.4] - 2026-01-22

### Added
- **Welcome Message**: Added a friendly welcome message that displays when the application starts and the menu bar is ready
  - Always visible in console (not dependent on verbosity flags)
  - Includes app version, warm greetings, and encouragement to share on GitHub and Mastodon
  - Encourages community contributions and feedback

## [0.1.3] - 2026-01-19

### Added
- **CLI Parameter Support**: Added support for passing CLI arguments through the `run` script
  - `./run --help` or `./run -h` to show help
  - `./run --openwindow` flag to optionally open CPU window at startup
  - All CLI flags (`-v`, `-vv`, `-vvv`, `--cpu`, `--frequency`, `--power-usage`, `--changelog`) now work through the `run` script
  - Development mode (`./run dev`) also passes arguments to `cargo run`

### Fixed
- **Window Opening at Startup**: Fixed issue where CPU window was automatically opening at startup
  - Removed manual `sendAction` test code that was triggering the click handler during setup
  - All windows are now properly hidden at startup (menu bar only mode)
  - Window only opens when explicitly requested via `--cpu` or `--openwindow` flags or when menu bar is clicked
- **Compilation Warnings**: Suppressed dead code warnings for utility methods
  - Added `#[allow(dead_code)]` to `total_points()`, `estimate_memory_bytes()`, `save_to_disk()`, and `load_from_disk()` methods
  - These methods are reserved for future use or used in tests
- **Power Consumption Flickering**: Fixed power consumption values flickering to 0.0W when background thread updates cache
  - Added `LAST_SUCCESSFUL_POWER` fallback cache to prevent flickering when main lock is unavailable
  - Power values now persist across lock contention scenarios
  - Improved power cache update logic to always maintain last successful reading
- **Power Display Precision**: Fixed power values < 1W showing as "0 W" causing visual flicker
  - Changed from `Math.round()` to `.toFixed(1)` to show 1 decimal place (e.g., "0.3 W" instead of "0 W")
  - Applied to both CPU and GPU power displays
  - Total power calculation now uses cached values to prevent flickering
- **Ollama Logging Safety**: Enhanced JavaScript execution logging with comprehensive sanitization
  - Added `sanitizeForLogging()` function to prevent dangerous characters from breaking logs
  - Safe logging wrapper that never throws errors, ensuring logging failures don't break execution flow
  - Truncates long strings, removes control characters, and sanitizes quotes/backticks
  - Prevents log injection and system breakage from malformed execution results

### Changed
- **Startup Behavior**: App now starts in menu bar only mode by default
  - No windows are visible at startup
  - CPU window is created on-demand when menu bar is clicked
  - Improved startup logging to indicate menu bar only mode
- **History Chart Styling**: Improved visual design of history chart container
  - Enhanced glass effect with backdrop blur and subtle shadows
  - Removed border, added inset highlights for depth
  - Better visual consistency with macOS glass aesthetic
- **Power Capability Detection**: Improved `can_read_cpu_power()` and `can_read_gpu_power()` functions
  - Now checks power cache existence as fallback when capability flags aren't set yet
  - Handles edge cases where power reading works but flags haven't been initialized
- **Development Logging**: Added verbose logging (`-vvv`) to release build script for easier debugging

### Technical
- **State Management**: Added `LAST_SUCCESSFUL_POWER` static state for power reading fallback
- **Error Handling**: Enhanced error handling in power consumption reading with graceful fallbacks
- **Logging Infrastructure**: Improved Ollama JavaScript execution logging with sanitization and error isolation

## [0.1.2] - 2026-01-19

### Added
- **Universal Collapsible Sections**: Replicated Apple theme's USAGE card click behavior across all themes
  - Clicking the USAGE card toggles both Details and Processes sections
  - Clicking section headers individually hides respective sections
  - Sections are hidden by default (collapsed state)
  - State persists in localStorage across sessions
  - Added universal IDs (`cpu-usage-card`, `details-section`, `processes-section`, `details-header`, `processes-header`) to all themes
  - Added clickable cursor and hover effects for better UX

### Fixed
- **Ollama Icon Visibility**: Fixed Ollama icon not being visible/green in themes other than Apple
  - Added default gray filter and opacity to all themes for icon visibility
  - Fixed green status filter to properly override default styling using `!important`
  - Icon now correctly displays green when Ollama is connected, yellow/amber when unavailable
  - Applied fixes to all 9 themes (apple, dark, architect, data-poster, futuristic, light, material, neon, swiss-minimalistic)
- **Data-Poster Theme Layout**: Fixed battery/power strip layout alignment with Apple theme
  - Removed unwanted grey background box around "Power:" label
  - Fixed battery icon color for dark theme visibility
  - Added missing `--hairline` CSS variable
  - Aligned spacing, padding, and styling to match Apple theme exactly
  - Fixed charging indicator to display green when charging

## [0.1.1] - 2026-01-19

### Fixed
- **Monitor Stats Persistence**: Fixed issue where external monitor stats (last_check, last_status) were not persisting after host reboot
  - Monitor stats are now saved to disk after each check
  - Stats are restored when monitors are loaded on app startup
  - Added `get_monitor_status()` command to retrieve cached stats without performing a new check
  - Stats persist across reboots in the monitors configuration file

## [0.1.0] - 2026-01-19

### Added
- **Monitoring System**: Comprehensive website and social media monitoring
  - Website uptime monitoring with response time tracking
  - Social media platform monitoring (Twitter/X, Facebook, Instagram, LinkedIn, YouTube)
  - Monitor status indicators (up/down) with response time display
  - Configurable monitor intervals and timeout settings
  - Monitor health summary with up/down counts
- **Alert System**: Multi-channel alerting infrastructure
  - Alert rules engine for monitor status changes
  - Alert channel support (prepared for future integrations)
  - Alert history and management
- **Ollama AI Chat Integration**: AI-powered chat assistant
  - Integration with local Ollama instance
  - Chat interface for system metrics queries
  - Model selection and connection status indicators
  - System prompt customization
  - Code execution support for JavaScript
  - Markdown rendering with syntax highlighting
- **Status Icon Line**: Quick access icon bar with status indicators
  - Monitors icon with green status when all monitors are up
  - Ollama icon with green status when connected, yellow when unavailable
  - 15-icon layout with placeholders for future features
  - Click-to-toggle section visibility
- **Dashboard UI**: New dashboard view for monitoring overview
  - Centralized monitoring status display
  - Quick access to all monitoring features
- **Security Infrastructure**: Keychain integration for secure credential storage
  - API key storage in macOS Keychain
  - Secure credential management for monitors and services
- **Plugin System**: Extensible plugin architecture
  - Plugin loading and management infrastructure
  - Prepared for future plugin integrations

### Changed
- **UI Layout**: Added collapsible sections for Monitors and AI Chat
  - Sections can be toggled via header clicks or icon clicks
  - Smooth expand/collapse animations
  - State persistence across sessions
- **Icon Styling**: Enhanced icon display with status-based color coding
  - Green for healthy/connected status
  - Yellow/amber for warnings/unavailable status
  - CSS filters for external SVG icons
- **Connection Status**: Real-time connection status updates
  - Visual indicators for Ollama connection state
  - Automatic connection checking on section expansion

### Technical
- **Backend Commands**: New Tauri commands for monitoring and Ollama
  - `list_monitors`, `add_monitor`, `remove_monitor`, `check_monitor`
  - `check_ollama_connection`, `ollama_chat`, `configure_ollama`
  - `list_alerts`, `add_alert_rule`, `remove_alert_rule`
- **State Management**: Enhanced application state with monitoring and Ollama state
- **Error Handling**: Comprehensive error handling for network requests and API calls
- **Logging**: Structured logging for monitoring and Ollama operations
- **Cross-Theme Support**: All new features (monitoring, Ollama chat, status icons) are available across all 9 themes
- **CSS Architecture**: Universal CSS with cascading variable fallbacks for cross-theme compatibility

## [0.0.6] - 2026-01-18

### Added
- **Power Consumption Monitoring**: Real-time CPU and GPU power consumption monitoring via IOReport Energy Model API
  - CPU power consumption in watts (W)
  - GPU power consumption in watts (W)
  - Power readings only when CPU window is visible (optimized for low CPU usage)
  - `--power-usage` command-line flag for detailed power debugging logs
- **Battery Monitoring**: Battery level and charging status display
  - Battery percentage display
  - Charging status indicator
  - Battery information only read when CPU window is visible
- **Process Details Modal**: Click any process in the list to view comprehensive details including:
  - Process name, PID, and current CPU usage
  - Total CPU time, parent process information
  - Start time with relative time display
  - User and effective user information
  - Memory usage (physical and virtual)
  - Disk I/O statistics (read/written)
- **Force Quit Functionality**: Force quit processes directly from the process details modal
- **Process List Interactivity**: Process rows are now clickable and show cursor pointer
- **Auto-refresh Process Details**: Process details modal automatically refreshes every 2 seconds while open
- **Scrollable Sections**: Added scrollable containers for Details and Processes sections with custom scrollbar styling
- **Process PID Display**: Process list now includes PID information in data attributes
- **Embedded Changelog**: Changelog is now embedded in the binary for reliable access
- **Changelog CLI Flag**: Added `--changelog` flag to test changelog functionality

### Changed
- **Process List Refresh**: Increased refresh interval from 5 seconds to 15 seconds for better CPU efficiency
- **Process Cache**: Improved process cache handling with immediate refresh on window open
- **UI Layout**: Improved flex layout with proper min-height and overflow handling for scrollable sections
- **Process Data Structure**: Added PID field to ProcessUsage struct for better process identification
- **Changelog Reading**: Improved changelog reading with multiple fallback strategies (current directory, executable directory, embedded)

### Performance
- **Smart Process Refresh**: Process details only refresh when CPU window is visible (saves CPU when window is hidden)
- **Conditional Process Updates**: Process list updates immediately on initial load and when window becomes visible
- **Efficient Modal Updates**: Process details modal only refreshes when actually visible
- **Power Reading Optimization**: Power consumption and battery readings only occur when CPU window is visible, maintaining <0.1% CPU usage when window is closed
- **IOReport Power Subscription**: Power subscription is created on-demand and cleaned up when window closes

### Technical
- **IOReport Power Integration**: Implemented IOReport Energy Model API integration for power monitoring
- **Array Channel Support**: Added support for IOReportChannels as arrays (Energy Model format)
- **Memory Management**: Proper CoreFoundation memory management for power channel dictionaries
- **Error Handling**: Graceful handling when power channels are not available on certain Mac models

## [0.0.5] - 2026-01-18

### Performance Improvements
- **Access Flags Optimization**: Replaced `Mutex<Option<_>>` with `OnceLock<bool>` for capability flags (temperature, frequency, power reading) - eliminates locking overhead on every access
- **Process Cache TTL**: Increased process list cache from 5 seconds to 10 seconds to reduce CPU overhead
- **Temperature Update Interval**: Increased from 15 seconds to 20 seconds for better efficiency
- **Frequency Read Interval**: Increased from 20 seconds to 30 seconds to reduce IOReport overhead
- **DOM Update Optimization**: Changed from `innerHTML` rebuilds to direct text node updates for metric values (reduces WebKit rendering overhead)
- **Ring Gauge Thresholds**: Increased update thresholds from 2% to 5% (visual) and 15% to 20% (animation) to reduce unnecessary animations
- **Window Cleanup**: Added cleanup handlers on window unload to clear animation state and pending updates

### Fixed
- **GitHub Actions Workflow**: Fixed workflow to properly handle missing code signing secrets (builds successfully without secrets)
- **Code Signing**: Made code signing optional - workflow builds unsigned DMG when secrets are not available
- **Legacy Code**: Removed outdated ACCESS_CACHE comment references

### Changed
- **Theme Gallery**: Updated README with comprehensive theme gallery showing all 9 themes
- **Screenshot Organization**: Removed old screenshot folders (screen_actual, screen-what-i-see), consolidated to screens/ folder

## [0.0.4] - 2026-01-18

### Added
- **App Version Display**: Added version number display in footer of all HTML templates
- **Version API**: Added `get_app_version` Tauri command to fetch version at runtime
- **Window Decorations Toggle**: Added window frame toggle in settings (affects new windows)
- **Config File Support**: Added persistent configuration file (`~/.mac-stats/config.json`) for window decorations preference
- **Toggle Switch Component**: Added modern toggle switch styling to all themes
- **GitHub Actions Workflow**: Automated DMG build and release on GitHub
- **Build Script**: Added `scripts/build-dmg.sh` for local DMG creation
- **DMG Download Section**: Added download instructions to README with Gatekeeper bypass steps

### Changed
- **Theme Improvements**: Massively improved all themes with better styling and visual consistency
- **Data Poster Theme**: Improved Details section styling to match Processes section (flex layout, consistent font sizes and weights)
- **Metric Unit Styling**: Improved metric unit display (%, GHz) with better font sizing and positioning
- **CPU Usage Display**: Fixed CPU usage value updates to properly maintain HTML structure with unit spans
- **Frequency Display**: Enhanced frequency display to include unit (GHz) with proper formatting
- **HTTPS Support**: Changed git clone URLs from SSH to HTTPS for better accessibility
- **Window Creation**: CPU window now respects window decorations preference from config file

### Fixed
- **Build Configuration**: Fixed Tauri build configuration (custom-protocol feature, bundle settings)
- **Binary Naming**: Fixed binary name from `mac-stats-backend` to `mac_stats` to match package name
- **DMG Detection**: Fixed build-dmg.sh script to properly detect DMG files using zsh array expansion
- **Release Workflow**: Fixed GitHub Actions workflow to properly upload DMG files to releases
- **Version Fetching**: Fixed duplicate command definition by moving `get_app_version` to metrics module

### Documentation
- **README Updates**: Added comprehensive DMG download instructions with Gatekeeper bypass methods
- **Known Limitations**: Added note about window frame toggle behavior (affects new windows only)
- **Installation Guide**: Improved installation section with multiple options and troubleshooting

## [0.0.3] - 2026-01-18

### Added
- **DMG Build Support**: Added DMG bundle creation for macOS distribution
- **GitHub Actions**: Added automated release workflow for building and publishing DMG files

### Changed
- **Version**: Bumped to 0.0.3

## [0.0.2] - 2026-01-18

### Fixed
- **CPU Frequency Reading**: Fixed frequency reading from IOReport to use delta samples instead of absolute counters, providing accurate recent frequency values instead of long-term averages
- **Memory Leaks**: Fixed CoreFoundation object leaks by properly retaining and releasing CF objects (channels_dict, subscription_dict, samples)
- **Crash Safety**: Added validation for IOReport channel dictionaries before calling IOReport functions to prevent crashes from invalid data
- **Channel Filtering**: Made `is_performance_channel()` more restrictive to only match actual CPU performance channels (ECPU*, PCPU*), reducing unnecessary processing

### Changed
- **Delta Sampling**: Frequency calculation now uses `IOReportCreateSamplesDelta()` to compute recent frequency over the sampling interval (20s) instead of since boot
- **Channel Classification**: Improved channel classification to correctly identify E-core (ECPU*) and P-core (PCPU*) channels
- **Frequency Extraction**: Enhanced frequency extraction to handle VxPy voltage/performance state format (e.g., V0P5, V19P0)
- **Command Execution**: Replaced fragile `sh -c` commands with direct binary calls using full paths (`/usr/sbin/sysctl`, `/usr/sbin/system_profiler`)
- **Code Organization**: Removed large redundant comment blocks from refactoring

### Refactored
- **Frequency Reading Logic**: Extracted complex nested frequency reading code from `lib.rs` into modular functions in `ffi/ioreport.rs`, reducing nesting from 5+ levels to max 2-3 levels
- **Array Processing**: Added support for IOReportChannels as an array (type_id=19) in addition to dictionary format
- **Logging**: Refactored `debug1/2/3` macros to use `write_structured_log` with timestamps for consistent logging format

### Added
- **Frequency Logging**: Added `--frequency` command-line flag for detailed frequency debugging logs
- **Validation**: Added validation checks for IOReport channel dictionaries (channel name, state count) before processing
- **Memory Management**: Added proper CFRetain/CFRelease cycles for all stored CoreFoundation objects
- **Cleanup**: Added cleanup path to release all CF objects when CPU window closes

### Security
- **FFI Safety**: Improved FFI safety by validating CoreFoundation types and null pointers before use
- **Memory Safety**: Fixed use-after-free risks by properly managing CF object lifetimes with guards

## [0.1.0] - Initial Release

### Added
- Basic system monitoring (CPU, RAM, Disk, GPU)
- Temperature monitoring via SMC
- CPU frequency monitoring via IOReport
- Process list with top CPU consumers
- Menu bar integration
- Multiple UI themes
- Low CPU usage optimizations
