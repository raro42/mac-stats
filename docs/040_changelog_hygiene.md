# Changelog hygiene (weekly review)

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/). It is a **user-facing** product history, not an agent run log.

## What belongs

- User-visible **Added / Changed / Fixed / Removed** under `## [Unreleased]` or a version heading
- Short bullets (one idea each); link to docs/tasks only when helpful
- Version sections when cutting a release (`## [0.1.x] - YYYY-MM-DD`)

## What does **not** belong

- OpenClaw / Hermes **reviewer tick** dumps (`Last check`, `DIGEST_SHA256`, aligned/misaligned)
- Repeated **`003-tester`** / CLOSED↔TESTING rename paperwork
- Full `cargo test` pass counts every hour
- Duplicate `[Unreleased]` sections

Put that noise in `tasks/`, `005-openclaw-reviewer/`, or (if already dumped) `docs/changelog-archive-agent-noise.md`.

## Weekly review (owner: Werner + agent)

**Cadence:** every **Monday ~10:00** local (see `~/.mac-stats/schedules.json`).

**Checklist:**

1. Read `CHANGELOG.md` top → first version heading.
2. Ensure **exactly one** `## [Unreleased]`.
3. Strip or relocate any reviewer/tester spam from Unreleased.
4. Fold shipped-but-unversioned bullets into a new `## [0.1.x]` if a release shipped since last review (use `Cargo.toml` version + `git log`).
5. Keep bullets scannable; archive bulk noise rather than deleting history without a trail.
6. Commit + push when the file is cleaner; Discord a short “changelog review done” note (counts removed / versions folded).

**Skill:** `~/.mac-stats/agents/skills/changelog-weekly-review/SKILL.md` (also invoked via `SKILL: changelog-weekly-review`).
