# Quality monitor (weekly)

Standing role: catch repo-root clutter and dead scaffolding **before** a human has to ask.

## Schedule

Friday ~10:00 local — `SKILL: quality-weekly-review` (see `~/.mac-stats/schedules.json` id `discord-quality-weekly`).

## Entrypoints

| Item | Path |
|------|------|
| Policy | [docs/044_repo_quality_hygiene.md](../../docs/044_repo_quality_hygiene.md) |
| Skill (repo) | [docs/skills/quality-weekly-review.md](../../docs/skills/quality-weekly-review.md) |
| Scanner | `python3 scripts/scan_repo_quality.py` |
| Live skill | `~/.mac-stats/agents/skills/quality-weekly-review/SKILL.md` |

## Prompt

Follow [PROMPT.md](PROMPT.md) when running as the quality-monitor agent.
