# Weekly disk-full reclaim (mac-stats)

Date: 2026-09-15 (updated 2026-09-27)  
Source: `~/projects/general-knowledge-gathering/notes/ops/disk-cleanup-proposal-2026-08-26.md` plus earlier ops notes.

## What filled the disk

Repeated cleanups on this Mac (Data volume) followed the same pattern:

| When | Main reclaim | Result |
|------|----------------|--------|
| 2026-08-15 | `cargo clean` on mac-stats `target/` (~198 GiB) | ~94% → ~76% |
| 2026-08-26 | Thunderbird, 30 GB MP4, mac-stats `target/` (~229 GB) | ~97% → ~57% |
| 2026-09-01 | mac-stats `target/` (~162 GB) + HF/CDP/uv caches | ~84% → ~57% |
| 2026-09-08 | mac-stats `target/` (~227 GB) + other Rust `target/` | ~86% → ~52% |
| 2026-09-25 | `cargo clean` on mac-stats `target/` (**102 GiB**, mostly release) | ~274 → ~369 Gi free |

**The weekly spike is almost always `~/projects/mac-stats/src-tauri/target` (debug + release).** Overnight builds grow it again. Soft-delete to Trash does **not** free space for a 100+ GB tree. Release alone was **89 GiB** on 2026-09-25 — Disk Cleanup used to leave `target/release` alone.

## Overnight building loop (daily)

`scripts/run_overnight_harness_loop.py` runs **`scripts/overnight_rust_target_clean.py` once per overnight window** (first tick after 20:00):

- `cargo clean` in `src-tauri` (fallback: `rm -rf target`)
- Skip when cargo/rustc is busy, or `target/` &lt; 1 GiB, or already cleaned that night
- Then `tmutil thinlocalsnapshots` so APFS returns the bytes
- Stamp: `~/.mac-stats/improvements/overnight_rust_target_clean_date.txt`

Manual: `python3 scripts/overnight_rust_target_clean.py --force`

## What Disk Cleanup does (daytime safety net)

Disk Cleanup (launch + every 24h + **Clean now**) includes builtin scopes:

| Scope | Default | Action |
|-------|---------|--------|
| Rust debug (mac-stats) | On | Permanent wipe of `…/target/debug` when ≥ **20 GiB** |
| Rust release (mac-stats) | On | Permanent wipe of `…/target/release` when ≥ **10 GiB** |
| uv / npm / Chrome CDP caches | On | Permanent wipe when over a small size cap |
| Cursor `state.vscdb.backup` | On | Age > 1 day (keep the live DB) |
| Docker dangling images | On | `docker image prune -f` only (not volumes, not `prune -a`) |
| Local Time Machine snapshots | On | `tmutil thinlocalsnapshots` (APFS purgeable space) |
| Hugging Face cache | Off | Opt-in wipe if ≥ 10 GiB |
| Other Rust `target/debug` (listed projects) | Off | Same 20 GiB rule |

**Not auto-deleted:** Ollama models, Apple wallpaper aerials, mail, CloudStorage, Docker volumes.

Turn a scope off in the Disk Cleanup panel if a rebuild cost is too high that week.

**UI note (2026-09-15):** Shallow status / glance polls do **not** walk rebuild trees or call `tmutil`/`docker` (they beachballed the CPU window). Use **Refresh** or **Clean now** for a deep scan. Dir walks cap at 1.5s; `tmutil`/`docker` at 8s.

## Do not start with Docker logs

Container JSON logs were ~0.5 GB. Dangling images matter more. Do **not** run `docker system prune -a --volumes` without review.

## Menu bar still ~95%?

Internal Data can be fine while **`/Volumes/x9pro 1`** (Time Machine) is ~95%. The menu bar prefers `/System/Volumes/Data`.
