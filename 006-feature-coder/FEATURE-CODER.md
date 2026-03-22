# Feature coder — FEAT backlog

Agent-facing backlog for mac-stats. Pick an open row, implement, run `cargo check` / `cargo test` / `cargo clippy`, then mark done and note in `CHANGELOG.md` under `[Unreleased]` when behaviour changes.

## Recently closed

| ID | Item | Notes |
|----|------|--------|
| FEAT-D2 | Clippy `items_after_test_module` | `logging/subsystem.rs`: `mod tests` moved after exported `mac_stats_*` macros. |
| FEAT-D1 | Session file resume: legacy filename layout | `load_messages_from_latest_session_file` matches new `session-memory-{id}-{ts}-{topic}.md` and legacy `session-memory-{topic}-{id}-{ts}.md` via `session_filename_matches_id` + tests. |

## Open / deferred (no owner)

Large integrations and vague items stay in **docs/006_roadmap_ai_tasks.md** and per-doc “Future/backlog” sections (Mail, WhatsApp, Google Docs, parallel tool loop, etc.) until scoped.

## When empty

Triage **docs/022_feature_review_plan.md** unchecked checklist items, **docs/035_memory_and_topic_handling.md**, or run `cargo clippy` for actionable warnings.
