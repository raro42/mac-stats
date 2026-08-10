# task-001: Improve Ollama timeout and session compaction handling

**Source:** `~/.mac-stats/debug.log` (monitoring run)

**Observed:**
- `ERROR Discord: Failed to generate reply: Failed to send chat request: ... operation timed out` (multiple occurrences)
- `WARN Session compaction failed: ... operation timed out, using raw history with 401 annotations`
- `WARN Periodic session compaction failed for discord ...: Ollama error: Service Temporarily Unavailable` / `operation timed out`

**Problem:** When Ollama is slow or unavailable, Discord reply generation and periodic session compaction fail with timeouts. The app does not retry, back off, or surface a clear user-visible state (e.g. "Ollama unavailable, retrying later").

**Required:**
1. **Discord reply generation:** On Ollama timeout or 503:
   - Option A: Retry with backoff (e.g. 1–2 retries, short delay) before returning error to user.
   - Option B: Return a user-friendly message (e.g. "Ollama is busy or unavailable; try again in a moment") instead of a raw error.
2. **Periodic session compaction:** On failure, log at WARN (already done) but consider:
   - Retrying once after a short delay, or
   - Skipping compaction for this cycle and retrying next cycle without treating it as a hard failure.
3. Ensure timeouts for Ollama HTTP calls are documented or configurable (e.g. in config or env) so operators can tune for slow models.

**Relevant code:**
- `src-tauri/src/commands/ollama.rs`: Discord reply flow, `compact_conversation_history`, periodic compaction loop (search for "Session compaction failed", "Periodic session compaction").
- Ollama HTTP client timeout configuration (e.g. reqwest client).

**Acceptance:** Fewer ERRORs for transient Ollama unavailability; user sees a clear message or retry when Ollama times out; compaction failures do not spam logs when Ollama is temporarily down.
