# task-005: Session compaction: clarify behavior when Ollama is unavailable

**Source:** `~/.mac-stats/debug.log` (monitoring run)

**Observed:**
- `WARN Session compaction failed: Failed to send chat request: ... operation timed out, using raw history with 401 annotations`

**Problem:** When session compaction fails (e.g. Ollama timeout), the code falls back to "raw history with 401 annotations". The number "401" here refers to the number of message annotations, not HTTP 401. This can be confused with "401 Unauthorized" (Discord/FETCH_URL). The warning is correct to emit, but the wording could be clearer so operators don't think every compaction failure is an auth issue.

**Required:**
1. Change the log message to avoid ambiguity with HTTP 401. For example:
   - "Session compaction failed: {error}, using raw history (N annotations)"
   - or "Session compaction failed: {error}; keeping full history (N messages) for this request."
2. Optionally document in code or docs: when compaction fails, the app continues with the full conversation history for that request, which may hit context limits; compaction will be retried next time.

**Relevant code:**
- `src-tauri/src/commands/ollama.rs`: search for "using raw history with 401 annotations" (around line 1331).

**Acceptance:** Log message no longer suggests HTTP 401 when it's the count of annotations; behavior unchanged.
