# Morning surprise — 2026-09-11

Overnight Track B (20:00–06:00 local) for Ralf.

## Shipped

| Version | What |
|---------|------|
| **v0.1.989** | **LaunchAgent WorkingDirectory** — `install-to-applications.sh` sets repo (else `$HOME`) on `com.raro42.mac-stats.plist` so launchd cwd is not `/`; defense in depth with RUN_CMD workspace |
| **v0.1.988** | **RUN_CMD working directory** — LaunchAgent cwd `/` no longer breaks relative skill scripts (`python3 scripts/scan_repo_quality.py`); children use Cursor Agent / config workspace (`~/projects/mac-stats`) |
| **v0.1.987** | Instant lane: **morning surprise age** (`morning surprise age`, `how old is morning_surprise.md`, `overnight morning surprise age`, `when was morning surprise updated`) — today's note mtime only; no dump; path/size/improvements/standing/sibling/loop safe |
| **v0.1.986** | Instant lane: morning surprise size |
| **v0.1.985** | Instant lane: morning surprise path |

## Fuel this window

- Digester open: empty
- Design review: not due (grace)
- Standing P2 / prior next-fuel → LaunchAgent WorkingDirectory (app KeepAlive lacked it; harness already had it)

## Notes

- Nightly keep landed in `results.tsv` (v0.1.989 + earlier keeps same night)
- Next fuel: digester open / sibling ports / overnight log instant lanes / design-review recapture when due
