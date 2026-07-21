---
name: changelog-weekly-review
description: Weekly Keep a Changelog hygiene for mac-stats CHANGELOG.md — strip agent/tester noise, one Unreleased, fold shipped versions.
---

# Changelog weekly review

Repo root: `~/projects/mac-stats`. File: `CHANGELOG.md`. Policy: `docs/040_changelog_hygiene.md`.

## Do

1. Open `CHANGELOG.md`. Confirm a single `## [Unreleased]` (if more than one, merge or archive extras into `docs/changelog-archive-agent-noise.md`).
2. Remove or archive bullets that are OpenClaw-reviewer ticks, repeated tester CLOSED/TESTING paperwork, or digests with `DIGEST_SHA256`.
3. Compare `src-tauri/Cargo.toml` version and recent `git log --oneline` to Unreleased; if versions shipped without headings, add concise `## [0.1.x] - date` sections (Added/Changed/Fixed).
4. Keep Unreleased short and user-facing.
5. **Ship via Cursor Agent** (Discord alone must not claim it committed). Emit:
   `CURSOR_AGENT: in ~/projects/mac-stats apply CHANGELOG hygiene if still dirty, then commit and push to origin`
6. Reply briefly with what you cleaned (lines removed, versions folded). If Cursor Agent was invoked, say so.

## Do not

- Dump full cargo test logs into the changelog
- Create a second `## [Unreleased]`
- Rewrite ancient version history unless fixing obvious corruption
- Reply with the Discord safety refusal about git — this skill is allowed to finish via `CURSOR_AGENT`
