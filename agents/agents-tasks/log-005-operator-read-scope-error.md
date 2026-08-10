# Task: operator.read scope error echoed into conversation

## Id: log-005
## Topic: operator-read-scope-error
## Status: done
## Created: 2026-02-23T10:00:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Session send failure *"missing scope: operator.read"* was forwarded into the chat as user messages (log lines 128–134). The model then replied with generic permission/help text. Technical scope errors should not become conversation content.

## Action

- When "Session Send" fails with a scope or permission error, do not inject the raw error (or a long explanation) as user content into the channel.
- Options: (A) filter: log the error server-side, send a single short user-visible line e.g. "Message could not be sent (permission missing)." (B) Document: add operator.read (or required scope) to setup/docs so admins can fix; still avoid echoing full error into the thread.

Goal: No technical scope/permission errors as multi-line user messages; clear, short user-facing message if needed.
