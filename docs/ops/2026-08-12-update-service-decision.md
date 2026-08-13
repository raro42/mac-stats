# In-app update check — product note (2026-08-12)

## Current behavior

CPU window already polls **once per calendar day** (`localStorage` day key) against GitHub `releases/latest`. If the tag is newer than `get_app_version()`, it shows a dismissible banner with release notes + `brew upgrade --cask mac-stats`. See `checkForAppUpdate` in `src/cpu.js`.

## Do we need a separate “update service”?

**No — not as a new daily binary poller.** That would duplicate the banner.

| Layer | Need? | Notes |
|-------|--------|--------|
| Detect newer release | Done | Once/day GitHub API from the CPU window |
| Tell the user | Done | Banner + brew hint |
| Download + replace `.app` silently | Later | Real auto-update (Sparkle / custom). Needs signed+notarized builds; ad-hoc DMGs already fight Gatekeeper |
| Homebrew users | Prefer brew | `brew upgrade --cask mac-stats` after tap sync; no custom updater required |

## Recommendation

1. **Keep** the lightweight banner (improve copy/opt-out if needed).
2. **Do not** add a background “update daemon” that only polls for a new binary.
3. **Before** auto-install: finish **notarized** CI DMGs (`docs/NOTARIZATION.md`, roadmap).
4. Then consider **Sparkle** (or Tauri updater) for DMG/`/Applications` installs; keep brew as the path for brew users.
5. Optional small enhancements (low cost): Agent Ops / Settings “Check for updates” button; show “up to date”; never auto-restart without consent (LaunchAgent + Discord uptime).

## Verdict

A daily **notify** check is a good enhancement and **already shipped**. A daily **download/replace service** is only worth it after notarization, and should be a proper updater — not a custom poller that reimplements the banner.
