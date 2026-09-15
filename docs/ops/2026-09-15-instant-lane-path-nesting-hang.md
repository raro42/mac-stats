# Instant-lane path matcher nesting hang (2026-09-15)

## Symptom

`mac_stats` pegs one core (~100% CPU) and the CPU window beachballs on open / Discord fast-lane. `sample` stacks sit in `looks_like_*_path_request` (often `looks_like_disk_cleanup_path_request`).

## Cause

Sibling path matchers in `harness_ops.rs` excluded each other by **calling** other `looks_like_*_path_request` functions. That graph is mutual. Short strings (≤72 chars) that miss the early length gate fan out exponentially.

## Fix (0.1.1081)

- Strip nested `looks_like_*_path_request(content)` excludes from path matchers.
- Keep **string-only** sibling excludes (`contains("history")`, `contains("monitors")`, …).
- Regression: `path_request_matchers_stay_fast_without_sibling_nesting`.

## Related

- Disk Cleanup UI hang (walks / `tmutil`): `2026-09-15-disk-cleanup-ui-hang.md` (0.1.1079)
- GPU `ioreg` main-thread hang: `2026-09-15-gpu-ioreg-ui-hang.md` (0.1.1080)
