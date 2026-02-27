# Task: Discord FETCH_URL / web fetch user message

## Id: log-001
## Topic: discord-fetch-url
## Status: done
## Created: 2026-02-23T09:55:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Log shows repeated user-visible message from Discord agent (Werni): *"The web fetch operation failed... protect against potential social engineering..."*. This occurs when the orchestrator or another agent uses **FETCH_URL** for discord.com URLs. Backend correctly blocks or returns non-authenticated content (discord.com requires API token); the model then replies with this generic security message.

Evidence in log: FETCH_URL requested for `https://discord.com/channels/...` (e.g. 2026-02-21); skill.md already says "NEVER use FETCH_URL for discord.com URLs — it has no token and will fail with 401".

## Action

- **Option A**: In the agent router, when FETCH_URL is requested for a URL whose host is discord.com (or similar), reject immediately and return a short hint: "Use DISCORD_API or AGENT: discord-expert for Discord; FETCH_URL is not supported for discord.com."
- **Option B**: Strengthen orchestrator and discord-expert skill text so the model never attempts FETCH_URL for Discord; consider adding an explicit "Do not use FETCH_URL for discord.com" in the tool-list paragraph for FETCH_URL.
- **Option C**: Both: pre-block in router (clear UX) and keep skill text strong.

Goal: Users do not see "web fetch failed... social engineering" when the real issue is "use Discord API / agent instead."
