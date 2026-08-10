# Task: Redmine API 422 — invalid updated_on parameter

## Id: log-009
## Topic: redmine-api-422-updated-on
## Status: done
## Created: 2026-02-23T11:00:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Log shows: **Redmine API GET /issues.json?priority=urgent&updated_on=last_week** and **updated_on=2025-02-04..2025-02-09** both return **422 Unprocessable Entity: {"errors":["Updated is invalid"]}**. The model (or backend) is sending an `updated_on` value Redmine does not accept. Redmine typically expects a date in YYYY-MM-DD format or a range in a specific form; "last_week" and possibly that range format are invalid.

The failure is passed back to Ollama as: *"Redmine API failed: Redmine API 422 Unprocessable Entity: {\"errors\":[\"Updated is invalid\"]}. Answer without this result."* — so the user gets an unhelpful technical message.

## Action

- Document or implement valid Redmine `updated_on` (and related) query params: e.g. single date YYYY-MM-DD, or range format Redmine accepts (e.g. "&lt;=YYYY-MM-DD" / ">=YYYY-MM-DD" or "YYYY-MM-DD..YYYY-MM-DD" if supported).
- If the user asks for "issues updated last week", map that to a valid date range (e.g. compute last week's start/end in YYYY-MM-DD and send that).
- On 422, return a short user-facing message (e.g. "Redmine didn't accept the query; try a specific date or date range") and log the full 422 body server-side.

Goal: "Issues worked on this week" / "last week" style queries work via Redmine API, and 422 is not surfaced as raw JSON to the user.
