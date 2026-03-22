# Feature coder — FEAT backlog

Agent-facing backlog for mac-stats. Pick an open row, implement, run `cargo check` / `cargo test` / `cargo clippy`, then mark done and note in `CHANGELOG.md` under `[Unreleased]` when behaviour changes.

## Recently closed

| ID | Item | Notes |
|----|------|--------|
| FEAT-D16 | Chat reserved words: no user bubble (022 §F8) | `sendChatMessage()` in `src/ollama.js` clears input then runs `--cpu` / `-v*` handling before `addChatMessage('user')` / `addToHistory`, so meta-commands are not shown as user turns (history was already skipped). Synced via `scripts/sync-dist.sh`. |
| FEAT-D15 | Agent soul vs shared soul helper + F3 tests | `agent_soul_or_shared()` in `agents/mod.rs` centralizes per-agent `soul.md` vs `Config::load_soul_content()` fallback; three unit tests lock 022 §F3 (non-empty per-agent wins; empty/missing → shared; both empty → None). `load_one_agent` unchanged behaviour; debug logs preserved. |
| FEAT-D14 | `MAC_STATS_SESSION_DIR` + session disk-resume tests | `Config::session_dir()` honors optional env (like `MAC_STATS_TASK_DIR`). Mutex-serialized tests call `load_messages_from_latest_session_file` against a temp dir: new layout, legacy `session-memory-{topic}-{id}-{ts}.md`, and newest `modified()` when two files match the id. Locks 022 §3 F1 “resume from disk” without writing under `~/.mac-stats/session`. |
| FEAT-D13 | Router soul prefix helper + F4 tests | `format_router_soul_block()` in `ollama_memory.rs` builds the planning-system soul + `You are mac-stats v…` prefix; `answer_with_ollama_and_fetch` calls it only when `skill_content` is `None` (agent `combined_prompt` still uses `skill_content` Some → empty soul). Three unit tests lock `docs/022_feature_review_plan.md` §F4 (no double voice when skill/agent prompt active; empty vs non-empty soul formatting). No API or runtime behaviour change. |
| FEAT-D12 | Execution message stack helper + F2 tests | `build_execution_message_stack()` in `session_history.rs` assembles system → capped history → current user (same order as both agent-router execution paths in `ollama.rs`); two unit tests lock `docs/022_feature_review_plan.md` §F2 ordering. No API or runtime behaviour change. |
| FEAT-E1 | Persist monitor stats after `check_monitor` | Each successful check updates `monitors.json` via `save_monitors()` so `last_check` / `last_status` survive restart and match background-thread behaviour (022 §3 F10). Inner block drops `get_monitors()` lock before save to avoid deadlock with `save_monitors`’ lock order; `save_monitors` errors ignored (busy locks). |
| FEAT-D11 | Chat verbosity command ↔ `VERBOSITY` atomic | Unit tests in `commands/logging.rs`: `set_chat_verbosity` updates the same `legacy::VERBOSITY` used by `ellipse()` / Ollama log truncation; values above 3 clamp. Mutex-serialized; restores prior level. Locks 022 §F8 checklist (reserved words path matches CLI atomic). No runtime behaviour change. |
| FEAT-D10 | TASK prompt F6 contract tests | `format_task_agent_description()` extracts the **TASK** paragraph from `build_agent_descriptions`; unit tests lock 022 §F6 strings (orchestrator for agent chats, `TASK_APPEND`/`TASK_STATUS` when topic+id exists). No runtime behaviour change. |
| FEAT-D9 | TASK_CREATE dedup tests + `MAC_STATS_TASK_DIR` | Optional env override in `Config::task_dir()` so unit tests use a temp dir (no writes under `~/.mac-stats/task`). `test_slug_deterministic` + `create_task_duplicate_topic_id_errors_with_task_append_hint` lock F5 slug/dedup/error-text contract (022 §3). Mutex-serialized tests restore env on drop. |
| FEAT-D8 | Conversation history cap helper + F1 ordering test | `cap_tail_chronological` in `commands/session_history.rs` (same semantics as `rev().take(N).rev()`); four unit tests. Used by `answer_with_ollama_and_fetch` and Discord having_fun (20 / 10 caps). `get_messages_before_add_user_excludes_current_turn` in `session_memory.rs` documents F1: fetch prior before `add_message` for the current user turn. Closes 022 §3 F1/F2 checklist automation. |
| FEAT-D7 | Session file parse: headings in body | `load_messages_from_latest_session_file` / `parse_session_markdown`: only full-line `## User` / `## Assistant` start a block (line-oriented parser). Lines like `## Notes` inside a message stay in the body; old `split("\n## ")` dropped those turns. Four unit tests. Closes 022 F1 checklist edge case (`## ` in content). |
| FEAT-D6 | Monitor check interval minimum | `check_interval_secs` clamped to ≥1 on add, load, and in `run_due_monitor_checks` so `0` cannot make a monitor due every pass (`elapsed >= 0`). Three unit tests. Closes review doc F10 edge case (022 §3). |
| FEAT-D5 | SSRF redirect DNS / empty resolve | `check_redirect_target_ssrf()`: redirect hop must resolve like the initial URL; DNS failure or zero addresses → `PermissionDenied` (no silent `follow`). Extracted from `ssrf_redirect_policy`; 4 unit tests (private IP, allowlist, public, `.invalid` DNS fail). Closes 022 §9 nit (redirect DNS-fail previously followed). |
| FEAT-D4 | FETCH_URL truncation marker | Oversized bodies: `truncate_fetch_body_if_needed()` ellipses with budget `MAX_BODY_CHARS - len(suffix)` and appends ` [content truncated]`; unit tests for short/long. Resolves review doc D3 (explicit LLM hint). |
| FEAT-D3 | SSRF: IPv4-mapped broadcast | `is_blocked_ip` IPv6 branch: `to_ipv4_mapped()` closure now includes `is_broadcast()` (parity with IPv4); test for `::ffff:255.255.255.255`. |
| FEAT-D2 | Clippy `items_after_test_module` | `logging/subsystem.rs`: `mod tests` moved after exported `mac_stats_*` macros. |
| FEAT-D1 | Session file resume: legacy filename layout | `load_messages_from_latest_session_file` matches new `session-memory-{id}-{ts}-{topic}.md` and legacy `session-memory-{topic}-{id}-{ts}.md` via `session_filename_matches_id` + tests. |

## Open / deferred (no owner)

Large integrations and vague items stay in **docs/006_roadmap_ai_tasks.md** and per-doc “Future/backlog” sections (Mail, WhatsApp, Google Docs, parallel tool loop, etc.) until scoped.

## When empty

Triage **docs/022_feature_review_plan.md** unchecked checklist items, **docs/035_memory_and_topic_handling.md**, or run `cargo clippy` for actionable warnings.
