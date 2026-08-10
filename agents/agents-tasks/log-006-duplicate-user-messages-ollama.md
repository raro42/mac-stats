# Task: Duplicate user messages in Ollama request payload

## Id: log-006
## Topic: duplicate-user-messages-ollama
## Status: done
## Created: 2026-02-23T10:00:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

Ollama request payload contained the same user message twice in a row (log lines 80–86): identical "Werni: Die Kaffeemaschine hat bereits gewacht...". Duplicates waste tokens and can confuse the model.

## Action

- Before building the messages array for Ollama, deduplicate consecutive identical user (or assistant) messages: if the last message has the same role and content as the one being appended, skip appending.
- Apply in the path that builds conversation history for having_fun / Discord → Ollama (and any other callers that send history).

Goal: No consecutive duplicate messages in Ollama API payloads.
