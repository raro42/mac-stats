# Downloads organizer

The **Downloads organizer** moves files from the top level of your Downloads folder into subfolders according to rules in a single markdown file. It runs in a background thread (checked every 60 seconds) when enabled.

## Where things live

| Item | Location |
|------|----------|
| Rules (markdown) | `~/.mac-stats/agents/downloads-organizer-rules.md` |
| Last-run summary (JSON) | `~/.mac-stats/downloads-organizer-state.json` |
| App config | `~/.mac-stats/config.json` keys below |
| Bundled default rules (first copy) | Shipped as `defaults/downloads-organizer-rules.md` in the repo; copied if the user file is missing |

## Config keys (`config.json`)

| Key | Values | Default |
|-----|--------|---------|
| `downloadsOrganizerEnabled` | boolean | `false` |
| `downloadsOrganizerInterval` | `"off"` \| `"hourly"` \| `"daily"` | `"off"` |
| `downloadsOrganizerDailyAtLocal` | `"HH:MM"` (24h, local) | `"09:00` if absent |
| `downloadsOrganizerPath` | string; empty = `~/Downloads` | empty |
| `downloadsOrganizerDryRun` | boolean | **`true`** (safe default) |

Config is re-read on each organizer tick and on each `Config::` accessor; **no app restart** is required for changes to take effect on the next run.

## Environment overrides

| Variable | Effect |
|----------|--------|
| `MAC_STATS_DOWNLOADS_ORGANIZER_ENABLED` | `true`/`false`/`1`/`0` |
| `MAC_STATS_DOWNLOADS_ORGANIZER_INTERVAL` | `hourly` / `daily` / `off` |
| `MAC_STATS_DOWNLOADS_ORGANIZER_PATH` | Overrides path |
| `MAC_STATS_DOWNLOADS_ORGANIZER_DRY_RUN` | `true`/`false` |

## Rules file format

- Human-readable markdown: optional `## Settings` (e.g. `catch_all: SubfolderName`), then repeated `## Rule` sections.
- Each rule lists bullet lines such as:
  - `match_extensions: png, jpg, pdf` (comma-separated, optional leading dots)
  - `match_glob: "*.dmg"` (simple glob; `*` supported)
  - `destination: Relative/Path` (under the Downloads root; no leading `/`)
- **First matching rule wins** (top to bottom).
- Files that match no rule are moved to **catch_all** if set; otherwise they stay in the Downloads root.
- Footer line `ruleset_version: 1` is optional metadata for future migrations.

## Safety

- **No deletes** in v1; only moves (with copy+delete fallback if rename crosses volumes).
- Always skipped: `.DS_Store`, `*.crdownload`, `*.part`.
- **Only top-level files** in the configured Downloads directory are considered (nothing inside existing subfolders).
- **Dry run:** when `downloadsOrganizerDryRun` is true, the engine logs planned moves at **debug** and records counts in state, but does not rename files.
- **Invalid rules:** the run is **skipped**; an error is logged and stored in state / shown in the Dashboard status.
- **Path override:** the resolved path must lie **under `$HOME`** (after canonicalization when the path exists).

## Collision policy

If the destination file already exists, the organizer uses `name (1).ext`, then `name (2).ext`, etc., until a free name is found.

## UI

Dashboard → **Settings** → **Downloads**: enable, interval, daily time, dry-run, optional path, last run summary, **Edit rules** (markdown editor modal), **Run now** (one immediate pass; requires organizer enabled).

## Tauri commands

- `read_downloads_organizer_rules` / `save_downloads_organizer_rules`
- `get_downloads_organizer_status` / `set_downloads_organizer_settings` (`patch` object, camelCase fields)
- `run_downloads_organizer_now`

## Security note

The organizer only operates under the configured Downloads root, and that root must stay inside the user’s home directory. Do not point it at system directories.
