# Issue #14 verify — 2026-10-08

Pulled `main` to `feac2394` (v0.1.1767). Release rebuild + restart with `--cpu`.

## Issue

`[bug] tauri://localhost eating CPU` — target below 1% with CPU window open.

## Method

- Binary: `/Users/raro42/projects/mac-stats/src-tauri/target/release/mac_stats` (Cargo version 0.1.1767)
- Detached start (`start_new_session`) with `--cpu`
- Identified **new** `com.apple.WebKit.WebContent` PID after launch (not unrelated long-lived WebKit PIDs)
- Window brought frontmost; 30×1s `ps` samples

## Results

| Process | PID | avg %CPU | max %CPU | min %CPU |
|---------|-----|----------|----------|----------|
| mac_stats | 48455 | 0.25 | 1.9 | 0.0 |
| WebContent (new with this launch) | 48485 | 0.35 | 1.6 | 0.1 |

## Verdict

**FIXED (<1% avg)**

Issue #14 is closed on GitHub after many Apple-theme opaque-type ratchets (through v0.1.1767). This run checks live CPU on the rebuilt tree.

## Notes

- Unrelated WebKit.WebContent PIDs on this Mac can sit near ~2–3% for days; do not attribute those to mac-stats.
- Main process may still show ~1–3% while the focused window polls metrics (#15 restored live updates).
- LaunchAgent `com.raro42.mac-stats` was bootout during verify to avoid single-instance exits; restore after verify if needed.
