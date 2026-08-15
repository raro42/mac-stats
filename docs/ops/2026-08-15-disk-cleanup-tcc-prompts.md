# 2026-08-15 — macOS Downloads / Trash access prompts

## Why the prompts appeared

Disk Cleanup had **Downloads** and **Trash** scopes enabled in `~/.mac-stats/config.json`. On every app launch, `run_now("startup")` walked those folders. Soft-delete also wrote into `~/.Trash`. macOS shows “access files in your Downloads folder” (and similar) for that.

Defaults ship those scopes **off**. This machine had them turned on.

## What we changed (v0.1.425)

1. **Auto runs** (startup + periodic): only the `mac-stats` scope (`~/.mac-stats`). Skip Downloads / Trash / Desktop / Documents path scopes.
2. **Auto soft-delete**: move into `~/.mac-stats/cleanup-quarantine` instead of `~/.Trash` (no Trash TCC).
3. **Status preview**: do not scan TCC folders until **Refresh** (`deep: true`) or **Clean now**.
4. CPU window no longer force-refreshes Disk Cleanup on every open when the section is collapsed.
5. Operator config: Downloads + Trash scopes disabled again on this Mac.

## Manual clean

**Clean now** still runs all enabled scopes and may prompt once if you re-enable Downloads/Trash — expected when you ask it to touch those folders.
