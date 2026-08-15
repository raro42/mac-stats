# Disk ~93% full — 2026-08-15

## Container

| Item | Size |
|------|------|
| APFS container | 994.7 GB |
| In use | **934.0 GB (93.9%)** |
| Free | **~61 GB (6.1%)** |
| Data volume | ~892 GB |
| System volume | ~12.5 GB |
| VM volume | ~16 GB |
| Local TM snapshots | several today (purgeable under pressure) |

`df` on `/System/Volumes/Data` shows **~94%** / **~57–61 GiB free**. That matches the menu-bar Disk gauge.

## Where home space goes (~667 GB under `/Users/raro42`)

| Path | ~Size | Notes |
|------|-------|--------|
| `~/Library` | 351G | See breakdown |
| `~/projects` | 230G | **mac-stats alone ~159G** |
| `~/.ollama` | 54G | Models |
| `~/.cache` | 11G | CDP caches ~10G |
| `~/.cursor` | 5.8G | |
| `/Applications` | 34G | System-wide |

### Library highlights

| Path | ~Size |
|------|-------|
| `~/Library/CloudStorage` | 112G |
| `~/Library/Application Support` | 96G |
| → `…/com.apple.wallpaper` | **56G** |
| → Proton Mail | 10G |
| → Cursor | 9.9G |
| → Brave | 9.0G |
| `~/Library/Thunderbird` | **83G** |
| `~/Library/Caches` | 17G |

### Projects highlights

| Path | ~Size |
|------|-------|
| `~/projects/mac-stats` | **159G** |
| `cug-cognos-loadtest` | 30G |
| `llama-ori` | 18G |

## Feature note: size-ordered disk tree

Worth building as a **lazy, on-demand** tree under Disk Cleanup (not a always-on scan). Scope roots the user already trusts; sort children by size; cache results. Avoid scanning CloudStorage / Mail / full home on every open (CPU + TCC). DaisyDisk-class UX is valuable; keep it optional and scoped.

## Safe reclaim candidates (do not auto-delete)

1. Inspect `~/projects/mac-stats` for huge `target/`, caches, artifacts.
2. Ollama: `ollama list` / remove unused models (~54G).
3. Wallpaper support folder (56G) — unusual; verify before delete.
4. Thunderbird mail (83G) — archive or compress, do not wipe blindly.
5. CloudStorage (112G) — cloud client local cache / sync.
6. `~/.cache/pos-marketing-chrome-cdp` + `firewall-chrome-sso` (~10G).
7. Local Time Machine snapshots: macOS may purge when free space is low.
