# Issue #11 — Enable Ollama / AI when Ollama is running

**Date:** 2026-09-11  
**Issue:** https://github.com/raro42/mac-stats/issues/11  
**Shipped:** v0.1.1008

## Problem

Fresh installs default to monitor-only (`aiAgentEnabled: false`). That dims the Ollama icon and sets `pointer-events: none`, so the chat section stays hidden even when local Ollama is already up. `install.sh` auto-enables AI when Ollama answers; DMG / Homebrew-only installs often skip that path.

## Fix

1. **Startup one-shot:** if AI is off, `MAC_STATS_NO_AI` is unset, `aiAgentOllamaAutoProbeDone` is false, and `GET {OLLAMA_HOST|/api/tags}` succeeds, set `aiAgentEnabled: true` and start the AI stack.
2. **Respect monitor-only:** Settings → AI off, or Reset to monitor defaults, sets `aiAgentOllamaAutoProbeDone: true` so the next launch does not re-enable.
3. **UI:** when AI is off, Ollama / Agent Ops / Perplexity icons stay clickable; click can enable AI (confirm) or open Settings.

## Opt out

- Env: `MAC_STATS_NO_AI=1`
- Config: `"aiAgentEnabled": false` + `"aiAgentOllamaAutoProbeDone": true`
- Settings Product → turn AI off, or Reset to monitor defaults
