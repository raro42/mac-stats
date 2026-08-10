# Repo quality hygiene (weekly review)

mac-stats is mostly **Rust + static frontend**. Keep the repo root small. Do not leave dead scaffolding for humans to discover later.

## Cadence

**Friday ~10:00** local — Werner schedule `discord-quality-weekly` + skill `quality-weekly-review`.

Complements Monday CHANGELOG hygiene and Wednesday UI polish. This pass is **repo health**, not product UI and not CHANGELOG spam cleanup.

## Checklist

1. Run `python3 scripts/scan_repo_quality.py` from the repo root. Fix every **fail** it reports (or document a deliberate exception in this file).
2. Scan the **repo root** for orphan dirs/files that belong under `agents/`, `docs/`, or `scripts/`.
3. Confirm there is **no** root `package.json` / `package-lock.json` unless a real Node frontend returns (build is Cargo / `cargo tauri`).
4. Confirm marketing assets live under `docs/screens/` (not root `screens/`). Runtime screenshots stay in `~/.mac-stats/screenshots/`.
5. Confirm agent queue/role homes live under `agents/` (not root `tasks/`, `003-tester/`, …).
6. Spot-check README / FEATURES links (demo video, gallery) still open.
7. If you change files: commit + push via  
   `CURSOR_AGENT: in ~/projects/mac-stats apply quality fixes if still dirty, then commit and push to origin`

## What to remove when found

| Smell | Action |
|-------|--------|
| Root npm scaffolding with no frontend bundler | Delete `package.json` / `package-lock.json`; `node_modules` is gitignored |
| Root `screens/` | Move to `docs/screens/` and update links |
| Root `tasks/` / role folders | Move under `agents/` |
| Stale README paths after a move | Fix links in the same change |
| Scripts that hard-code old paths | Update scripts |

## Skill

`~/.mac-stats/agents/skills/quality-weekly-review/SKILL.md`  
Also: `SKILL: quality-weekly-review`  
Repo mirror: `docs/skills/quality-weekly-review.md`  
Agent home: `agents/007-quality-monitor/`
