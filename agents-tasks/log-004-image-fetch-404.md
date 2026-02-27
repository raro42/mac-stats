# Task: Image fetch 404 forwarded as user message to model

## Id: log-004
## Topic: image-fetch-404
## Status: done
## Created: 2026-02-23T10:00:00Z
## Source: ~/.mac-stats/debug.log scan (cron 2026-02-23)

## Summary

When an image URL returns 404, the raw error text is sent to the model as user content: *"Werni: The requested image could not be fetched due to a 404 error. Please verify the URL and try again with a valid image link..."* (see log ~line 116). This pollutes the conversation and triggers generic "help" responses.

## Action

- Handle image fetch 404 (and similar client errors) before building the user message: do not forward the full error blurb as the user message.
- Either: (A) send a short, natural line like "Image link returned 404 — could not load image", or (B) skip adding the message and reply once with a brief "That image link didn’t work (404). Try another?" so the model can respond conversationally without parsing error text.

Goal: Clean conversation context; no raw error paragraphs as user messages.
