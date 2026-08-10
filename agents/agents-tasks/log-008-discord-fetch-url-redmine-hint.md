# Task: Discord FETCH_URL for Redmine/example URL — user sees raw error

## Id: log-008
## Topic: discord-fetch-url-redmine-hint
## Status: done
## Created: 2026-02-23T11:00:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Log shows: model used **FETCH_URL** for `https://redmine.example.com/issues/1234.json?include=journals,attachments`. That URL does not resolve (example.com), so fetch failed with DNS error. User received: *"Sorry, I couldn't generate a reply: Fetch page failed: Request failed: error sending request for url (...): error trying to connect: dns error: failed to lookup address information: nodename nor servname provided, or not known. (Is Ollama configured?)"*

Two problems: (1) For Redmine, the model should use **REDMINE_API**, not FETCH_URL (no auth on FETCH_URL). (2) When FETCH_URL fails (DNS, 4xx, etc.), the reply to the user should be a short, friendly message and optionally a hint (e.g. "For Redmine tickets use REDMINE_API or say 'review ticket &lt;id&gt;'") instead of the raw error string.

## Action

- When the agent router or Discord flow uses FETCH_URL and the request fails (DNS, connection, 4xx): return a short user-facing message (e.g. "That URL couldn't be fetched. If this is a Redmine ticket, try: review ticket 1234") and log the full error server-side only.
- Optionally: when the URL host is redmine.* or path looks like /issues/*.json, reject FETCH_URL and hint to use REDMINE_API (same spirit as log-001 for discord.com).

Goal: Users do not see raw "dns error" / "nodename nor servname" when the fix is "use Redmine API" or "use a real Redmine URL."
