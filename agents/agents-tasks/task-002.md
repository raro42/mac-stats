# task-002: Validate and sanitize URLs for FETCH_URL / fetch_page

**Source:** `~/.mac-stats/debug.log` (monitoring run)

**Observed:**
- `ERROR Discord: Failed to generate reply: Fetch page failed: Invalid URL: invalid international domain name`
- `ERROR Scheduler: Ollama failed ... Fetch page failed: Request failed: ... for url (https://worldtimeapi.org/api/ip.%20Scheduling%20isn't%20supported%20here...)`
- URLs containing prompt text or instructions (e.g. `...api/ip.%20then%20AGENT:%20orchestrator...`) were passed to fetch, causing "connection closed" or invalid-URL errors.

**Problem:** FETCH_URL and fetch_page accept whatever string follows the prefix. No validation that the string is a single, well-formed URL. Model output or scheduler task text can include extra words or instructions that get concatenated to the URL, producing invalid URLs or wrong hosts.

**Required:**
1. **URL extraction:** When parsing `FETCH_URL: <arg>` (and scheduler FETCH_URL tasks), take only the first valid URL from `arg` (e.g. trim, then take first token that parses as URL, or use a strict regex for `https?://...` up to first space or newline).
2. **Validation before fetch:** Before calling the HTTP client, validate the URL with the same rules (e.g. `url::Url::parse` or equivalent). Reject with a clear error (e.g. "Invalid URL for FETCH_URL: ...") if invalid.
3. **International domain names:** Handle IDN explicitly: either reject with a clear message or normalize/convert to punycode so "invalid international domain name" is avoided or explained.

**Relevant code:**
- `src-tauri/src/commands/ollama.rs`: FETCH_URL parsing (e.g. `parse_fetch_url`), agent router FETCH_URL handling, scheduler call path.
- `src-tauri/src/scheduler/mod.rs`: FETCH_URL task execution (`task["FETCH_URL:".len()..].trim()`).
- `src-tauri/src/commands/browser.rs`: `fetch_page` / fetch implementation.

**Acceptance:** No fetch attempted for malformed or multi-token URLs; one clear error message to the user/model when URL is invalid; IDN either supported or rejected with a clear message.
