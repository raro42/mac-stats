# Task: Temperature 0.0°C in system metrics (M3)

## Id: log-003
## Topic: temperature-unavailable
## Status: done
## Created: 2026-02-23T10:00:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Ollama request payloads include `Temperature: 0.0°C` in system metrics (e.g. lines 72, 98 in scanned log). On M3/M4, SMC temperature may be unavailable or key not yet discovered; 0.0 is misleading and suggests a sensor reading. CLAUDE.md notes: "Temperature reads may return 0.0 if SMC unavailable or if M3 key not discovered."

## Action

- When temperature is 0.0 or unavailable, either: (A) omit the temperature line from the metrics string sent to Ollama, or (B) show "Temperature: N/A" or "Temperature: unavailable" instead of "0.0°C".
- Optionally document in agent/system prompts that "N/A" means SMC not available (no user action needed).

Goal: Avoid implying a real 0°C reading; clarify when temp is unavailable.
