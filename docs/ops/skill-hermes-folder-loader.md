# Skill loader: Hermes folders (2026-11-10, v0.1.2555)

## Symptom

Discord (Werner schedules) sometimes replied that `changelog-weekly-review` (or `ui-weekly-review` / `layout-daily-review`) is unknown and cannot run. Tool result in logs:

```text
Unknown skill "changelog-weekly-review". Available skills: 1-summarize, 2-code.
```

The model then paraphrased that into chat.

## Cause

Weekly skills on disk use Hermes layout:

```text
~/.mac-stats/agents/skills/changelog-weekly-review/SKILL.md
```

`load_skills()` only accepted legacy flat names:

```text
skill-<number>-<topic>.md
```

So folder skills existed but never entered the catalog used by `SKILL:`.

## Fix

`src-tauri/src/skills.rs` loads both layouts. Folder skills get numbers after the max flat number. YAML frontmatter is stripped for the skill session prompt.

## Verify

After restart, `~/.mac-stats/debug.log` should show:

```text
Skills: loaded N from ".../agents/skills": 1-summarize, 2-code, … changelog-weekly-review …
```

Schedule task remains: `SKILL: changelog-weekly-review — …` in `~/.mac-stats/schedules.json`.
