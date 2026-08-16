# Session handoff — 2026-08-16 (evening)

Condensed notes for the next agent after a conversation restart.

## Theme of this session

Polish **user-facing warmth**: titles, taglines, README Privacy, and a **Star growth** banner — plus the standing rule **be nice to the user**.

## Shipped (on `main`)

| Item | Detail |
|------|--------|
| Window title | `mac-stats · glad you're here` (version stays in footer) — `status_bar.rs`, `src/cpu.js`, themes |
| Rule | `.cursor/rules/be-nice-to-the-user.mdc` (alwaysApply); noted in `agents.md` |
| Track rules | `.gitignore` keeps `.cursor/*` private but **tracks** `.cursor/rules/**` |
| README Privacy | Removed “(never commit)” — agent reminder lives in `agents.md` secrets bullet |
| Tagline | “Your Mac already knows how busy it is. Now you can too — from the menu bar.” (+ local-AI line) in README, Getting Started, landing, Features |
| Star growth | README + `docs/site/index.html` banner; chart `docs/screens/star-history.svg` |
| Nightly chart | `overnight_git_flush.py` (~23:00) runs `generate_star_history_svg.py`; rewrite only when star fingerprint changes |
| App install | `/Applications` was refreshed to **0.1.467** during title work |

## Key paths

- Title: `src-tauri/src/ui/status_bar.rs`, `src/cpu.js`
- Stars: `scripts/generate_star_history_svg.py`, `scripts/overnight_git_flush.py`, `docs/screens/star-history.svg`
- Copy tone: `.cursor/rules/be-nice-to-the-user.mdc`

## Do not regress

- Do **not** put `mac-stats 0.1.xxx` in the window title.
- Do **not** put agent-only reminders (“never commit”) in the public README Privacy blurb.
- Public Star History SVG embeds need a sealed token (GitHub 2026 API); prefer the local chart + nightly refresh.
- After `src/` JS edits: `./scripts/sync-dist.sh`, rebuild/install (frontend is embedded). Force-add theme HTML under `src-tauri/dist` when needed (`git add -f`).

## Optional follow-ups (not started)

- RAM **metrics-grid ring** (Details RAM rows shipped earlier; ring still optional).
- Alternate title/tagline phrases if the owner wants a different line.
- Sealed-token live Star History embed (owner PAT) — only if they prefer third-party over local SVG.

## Recent commits (this arc)

```
8008fd6 Document star-chart refresh in overnight_git_flush.
80c5405 Refresh the star chart nightly via overnight git flush.
cf698b0 Add a Star growth banner with a local cumulative chart.
d193ee0 / cfca970 Tagline + be-nice sarcasm note
f5769ac Move secret commit reminder from README into agent docs
6662fad / ae0a1aa / 44b8579 Warmer title + track .cursor/rules
```

## Version note

`Cargo.toml` was at **0.1.467** for the title ship. Docs-only follow-ups after that sit under **Unreleased** in `CHANGELOG.md` until the next version bump.
