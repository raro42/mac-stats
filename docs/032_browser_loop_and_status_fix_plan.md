## mac-stats
### Overview

mac-stats is a local AI agent for macOS that provides a range of features, including Ollama chat, Discord bot, task runner, scheduler, and MCP. It is built with Rust and Tauri.

### Install

#### DMG (recommended)

[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

#### Build from source

```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```

Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

#### Gatekeeper workaround

If macOS blocks the app, Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

* **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
* **AI chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
* **Discord bot**: [Discord bot documentation](100_all_agents.md)

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Click Status

* **Current behaviour**: In `ollama.rs`, BROWSER_CLICK sends: `send_status("🖱️ Clicking element {}…", index)`.
* **Approach**: Cache last browser state in `browser_agent`:
  - Add a static (e.g. `LAST_BROWSER_STATE: Mutex<Option<BrowserState>>`).
  - After every successful `navigate_and_get_state`, `click_by_index_inner`, `input_by_index_inner` (and HTTP fallback equivalents), update this cache with the state we are about to return (or have just computed).
- Expose `pub fn get_last_element_label(index: u32) -> Option<String>` in `browser_agent`:
  - Lock `LAST_BROWSER_STATE`, find `Interactable` with `index == i.index`, then return a short label: `i.text` if non-empty, else `i.placeholder`, else `i.href`, else `"(no label)"`. Truncate to e.g. 40 chars for status.
- Use in ollama.rs for BROWSER_CLICK (and optionally BROWSER_INPUT):
  - Before `send_status`, call `crate::browser_agent::get_last_element_label(idx)`.
  - If `Some(label)`, send e.g. `🖱️ Clicking element {} ({})…`, else keep current `🖱️ Clicking element {}…`.
- Edge cases: First action in a run may have no cached state (e.g. first BROWSER_CLICK after BROWSER_NAVIGATE in same turn: we have state from NAVIGATE once it returns; but NAVIGATE runs before CLICK, so when CLICK runs we already have state from NAVIGATE).

## Browsers Popping Up Again and Again

* **Suspected cause**: Model keeps emitting BROWSER_NAVIGATE / BROWSER_CLICK (or multiple NAVIGATEs) without reaching DONE or a final answer.
* **Mitigations**:
  1. **Log and monitor**: Ensure we log each BROWSER_NAVIGATE (URL) and BROWSER_CLICK (index) in the agent router (already have `info!("BROWSER_CLICK: index {}", idx)` etc.).
  2. **Cap browser actions per run (safety limit)**: Maintain a counter for "consecutive or total browser tools" this run (e.g. `browser_tool_count` incremented for BROWSER_NAVIGATE, BROWSER_CLICK, BROWSER_INPUT, BROWSER_SCROLL, BROWSER_EXTRACT, BROWSER_SEARCH_PAGE). If `browser_tool_count` exceeds a limit (e.g. 15), either stop running more browser tools and return a message to the model: "Maximum browser actions per run reached (15). Reply with your answer or DONE: success / DONE: no." and break (or inject a user message and continue once).
  3. **Repetition detection (optional)**: Track last few (e.g. 3) (tool, arg) pairs for browser tools. If we see the same BROWSER_NAVIGATE URL twice in a row, or the same BROWSER_CLICK index twice in a row, optionally refuse the duplicate and tell the model: "Same browser action as previous step; use a different action or reply with DONE."
  4. **Encourage DONE after screenshot**: Reinforce: "After BROWSER_SCREENSHOT: current, reply with DONE: success and a short confirmation. Do not continue navigating or clicking unless the user asked for more."
  5. **Headless by default for non-interactive**: Consider defaulting to headless (no visible window) so "browsers popping up" is at least not visible.

## Implementation Status

* **Phase 1 (element label)**: Done. `browser_agent` has `LAST_ELEMENT_LABELS` cache, `set_last_element_labels()`, `get_last_element_label(index)`. CDP and HTTP fallback set the cache after navigate/click/input. In `ollama.rs`, BROWSER_CLICK and BROWSER_INPUT status messages show the label when available.
* **Phase 2 (browser tool cap)**: Done. In `ollama.rs` tool loop: `browser_tool_count` and `MAX_BROWSER_TOOLS_PER_RUN` (15). Before running any BROWSER_* tool we check the cap; if exceeded we push "Maximum browser actions per run reached (15). Reply with your answer or DONE: success / DONE: no." and skip that tool. Log line: "Agent router: browser tool #N/15 this run".
* **Phase 3 (Error short-circuiting)**: Done. Added logic in `ollama.rs` after tool execution to detect if a `BROWSER_` tool (or `BROWSER_SCREENSHOT`) returned an error. If an error is detected, the batched tool execution loop is aborted via `break`, preventing subsequent batched tools (like `BROWSER_SCREENSHOT` immediately after a failed `BROWSER_NAVIGATE`) from incorrectly executing on an invalid state.

## Open tasks:
- ~~Review and refine the `browser_agent` cache and `get_last_element_label` function for better performance and edge cases.~~ **Done:** Cache now uses `HashMap<u32, String>` for O(1) lookup; `set_last_element_labels` builds map from vec (duplicate indices: last wins). Edge cases documented in doc comment: lock poison → `None`, empty cache → `None`, index not in last state (e.g. first action before navigate) → `None`.
- ~~Investigate and implement a more robust repetition detection mechanism.~~ **Done:** In `ollama.rs` tool loop: `last_browser_tool_arg` stores last (tool, normalized_arg); `normalize_browser_tool_arg()` normalizes by tool (NAVIGATE → lowercase URL, CLICK/INPUT → index, etc.). If current (tool, normalized_arg) equals the previous, we skip execution and push "Same browser action as previous step; use a different action or reply with DONE." Duplicate does not consume the browser tool cap.
- ~~Consider adding a "browser tool limit" warning for users who exceed the limit.~~ **Done:** When the browser tool cap is reached, the reply now appends a user-facing note: "Note: Browser action limit (15 per run) was reached; some actions were skipped." (ollama.rs, after the tool loop).
- Review and optimize the `ollama.rs` tool loop for better performance and error handling.