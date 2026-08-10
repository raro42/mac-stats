# Task: debug.log rotation / size control

## Id: log-002
## Topic: log-rotation
## Status: done
## Created: 2026-02-23T09:55:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Scan found debug.log at **126,934 lines** (~20MB). No rotation or size limit is applied; file grows unbounded. Most lines are DEBUG (HTTP/Discord/Ollama tracing). Large logs make scanning slow and can fill disk over time.

## Action

- Add log rotation or size-based truncation for `~/.mac-stats/debug.log` (e.g. when size exceeds N MB, rotate to debug.log.1 and truncate, or use tracing-appender with rolling file).
- Alternatively/additionally: reduce default log level so DEBUG is only written when `-vv` or `--verbose` is used, keeping INFO as default for normal runs.

Goal: Bounded disk usage and faster log scans without losing ability to enable verbose logging when needed.

## Implementation
- `logging/mod.rs`: if `debug.log` size exceeds 10 MB at startup, truncate file before opening for append. DEBUG level unchanged (use `-vv` for verbose).
