# Task: Loop protection — visibility of dropped messages

## Id: log-007
## Topic: loop-protection-visibility
## Status: done
## Created: 2026-02-23T10:00:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Log shows: *"dropping bot message in having_fun channel 1475074452164317267 (loop protection)"* (line 143). Loop protection is working as intended; there is no summary of how often this happens per channel or per period.

## Action

- Optional enhancement: maintain a counter (per channel or global) of messages dropped due to loop protection and either: (A) log a periodic summary (e.g. every N minutes: "loop protection: 2 drops in channel X this period"), or (B) expose in a small admin/diagnostic view. Keep as DEBUG or optional so it does not add noise by default.

Goal: Visibility into loop-protection activity for tuning and support, without cluttering default logs.

## Implementation (2026-02-27)
- Added `loop_protection_drops: u64` to `HavingFunState`; incremented when a bot message is dropped.
- In having_fun heartbeat (every 60s when configured), for each channel with `loop_protection_drops > 0`: log `DEBUG Discord: loop protection: channel {} dropped {} message(s) this period`, then reset counter to 0.
- Log level DEBUG so default runs stay quiet; use `-vv` to see summaries.
