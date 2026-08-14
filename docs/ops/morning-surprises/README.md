# Morning surprises (repo archive)

Overnight Track B writes the live note to:

`~/.mac-stats/improvements/morning_surprise_YYYY-MM-DD.md`

That path stays the **runtime** source for digester / instant “what shipped overnight?” answers.

**Also copy** the same file into this folder and commit it when the night finishes (or the next morning if the agent forgot). Do not put these tables in `CHANGELOG.md` — Keep a Changelog stays user-facing; this folder is the overnight ship log.

## Dual-write checklist

1. Write/refresh `~/.mac-stats/improvements/morning_surprise_YYYY-MM-DD.md` before ~05:50.
2. `cp` into `docs/ops/morning-surprises/morning_surprise_YYYY-MM-DD.md`.
3. Commit + push with the night’s work (or a small docs commit).

Helper: `python3 scripts/archive_morning_surprise.py` (copies today or `--date YYYY-MM-DD`).
