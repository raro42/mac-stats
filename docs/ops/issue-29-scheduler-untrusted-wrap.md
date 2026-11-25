# Issue #29 — schedule replies “untrusted data will not be processed”

**Date:** 2026-11-25  
**Issue:** https://github.com/raro42/mac-stats/issues/29  
**Fix:** v0.1.3089

## Symptom

Discord posts from schedule `discord-layout-daily`:

`Understood. That block of text is noted as untrusted data and will not be processed.`

## Cause

`scheduler/mod.rs` wrapped the schedule task body with `wrap_untrusted_content("scheduler-task", …)`. The envelope tells the model not to follow instructions inside. The model complied and refused the skill.

Same misclassification also wrapped heartbeat checklists, local task-file content, and CLI questions.

## Fix

Pass operator-local instruction text plain. Keep `MS_UNTRUSTED` for external blobs only (Discord user messages, FETCH_URL, APIs, search, browser extract, RUN_CMD stdout). Suspicious-pattern scans may still log; they do not alter prompts.

## Verify

After restart, a due schedule must **not** log `label=scheduler-task` on `wrapped untrusted content for LLM prompt`. The question preview must not start with `<<<MS_UNTRUSTED_BEGIN`.
