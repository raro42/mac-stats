# agents/log-monitor/

Home for **reading logs**. Standing rule: always skim `~/.mac-stats/debug.log` after runtime changes or failed behaviour.

## Source

- Log: `~/.mac-stats/debug.log`
- **Read-only by default.** Do not truncate or copy the log to `debug.log-sic` from this monitor.

## How to scan

```bash
# Heuristic scan (no LLM)
python3 scripts/scan_debug_log_errors.py
python3 scripts/scan_debug_log_errors.py --minutes 60
python3 scripts/scan_debug_log_errors.py --write-finding
```

For agent-driven review, follow [PROMPT.md](PROMPT.md).

## Findings

Write new product-owned issues under [../agents-tasks/](../agents-tasks/) (`log-NNN-topic.md`). Update that folder’s README summary table. Deduplicate the same ERROR signature within 24 hours.

## Ignore (benign noise)

Document recurring false positives here as they appear (HTTP/2 GoAway, expected SSRF blocks, etc.). Prefer fixing product bugs over expanding the ignore list.
