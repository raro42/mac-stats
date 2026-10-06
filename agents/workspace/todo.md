# Coder — #14 v0.1.1436

## Plan
- [x] AI agent visibility from localStorage on open (no `get_ai_agent_enabled` IPC)
- [x] Persist AI flag on toggle / Settings sync / enable-from-icon / event
- [x] Defer `initializeOllama` off DOMContentLoaded; arm on expand / AI-on need
- [x] Focus resume: Ollama recheck only when AI on (localStorage)
- [x] Bump to v0.1.1436; CHANGELOG; sync-dist; standing backlog; task notes
- [x] `cargo check`
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1436** for GitHub #14. AI visibility from localStorage on open; Ollama configure deferred until AI Chat expand / AI-on resume; Settings Product syncs backend + cache. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
