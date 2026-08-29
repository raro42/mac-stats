# Unattended GitHub Releases — 2026-08-29

## Decision

Ralf: the overnight loop should cut releases when we are getting things done — do not wait to be asked.

## Mechanism

- `scripts/maybe_cut_github_release.py` — gates (~20+ patches ahead of Latest, or ≥7 days with ≥5 patches; clean `main`; once/day), then annotated tag + `gh release create`.
- `scripts/run_overnight_harness_loop.py` — after ~23:00 `overnight_git_flush.py`, runs the release script.
- Cursor rule `.cursor/rules/push-when-reasonable.mdc` + `AGENTS.md` updated.

## Catch-up release

**v0.1.716** — first GitHub Release since **v0.1.458** (2026-08-16). CI attaches DMG; Homebrew cask SHA follows when the asset is ready.
