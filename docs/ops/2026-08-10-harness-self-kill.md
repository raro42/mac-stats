# Overnight harness aborted (2026-08-10 ~19:07 CEST)

## What happened

- Overnight harness terminal (`run_overnight_harness_loop.py`, started 2026-08-08) ended with `status: aborted` after ~56h.
- Timing matches the “Anything going wild?” investigation (~19:07). High-CPU PID `25253` was under scrutiny; that process is gone.
- mac-stats / Werner stayed up (LaunchAgent `com.raro42.mac-stats`, PID running with `-vv`).

## Fix applied

- Restarted `python3 -u scripts/run_overnight_harness_loop.py` without touching `mac_stats`.
- Do not kill Cursor/`agent` or the harness when chasing “wild” CPU — that process is often the investigator.

## Rule

Kill chrome-devtools / Hermes when asked. Never kill `mac_stats` or the overnight harness without an immediate start+verify.
