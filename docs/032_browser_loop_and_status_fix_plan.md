# Bug fix plan: browser loop + click status context

## Context

- **025** (`docs/025_expectation_check_design_DONE.md`): Expectation check (criteria at start, verification at end, heuristic, escalation, retry on NO, DONE tool, status messages with reasoning + emojis).
- **Observed**: Many browser windows popping up again and again during a run; status shows "🖱️ Clicking element 7…" with no indication *what* element 7 is (e.g. "Accept all", "Search", link text).

## Goals

1. **Status context**: "Clicking element N" → include the element label (e.g. "Clicking element 7 (Accept all)…") so the user and logs show what is being clicked.
2. **Browser loop**: Reduce or prevent runaway browser actions (repeated NAVIGATE/CLICK opening many windows or looping).

---

## 1. Click status: add element label (e.g. "Clicking element 7 (Accept all)…")

### Current behaviour

- In `ollama.rs`, BROWSER_CLICK sends: `send_status("🖱️ Clicking element {}…", index)`.
- We only have the index; the element label (text, placeholder, or href) is in the page state returned *after* the previous browser action (NAVIGATE, CLICK, or INPUT). That state is sent to the model, not stored for the next status line.

### Approach

- **Cache last browser state** in `browser_agent`:
  - Add a static (e.g. `LAST_BROWSER_STATE: Mutex<Option<BrowserState>>`).
  - After every successful `navigate_and_get_state`, `click_by_index_inner`, `input_by_index_inner` (and HTTP fallback equivalents), update this cache with the state we are about to return (or have just computed).
- **Expose** `pub fn get_last_element_label(index: u32) -> Option<String>` in `browser_agent`:
  - Lock `LAST_BROWSER_STATE`, find `Interactable` with `index == i.index`, then return a short label: `i.text` if non-empty, else `i.placeholder`, else `i.href`, else `"(no label)"`. Truncate to e.g. 40 chars for status.
- **Use in ollama.rs** for BROWSER_CLICK (and optionally BROWSER_INPUT):
  - Before `send_status`, call `crate::browser_agent::get_last_element_label(idx)`.
  - If `Some(label)`, send e.g. `🖱️ Clicking element {} ({})…`, else keep current `🖱️ Clicking element {}…`.
- **Edge cases**: First action in a run may have no cached state (e.g. first BROWSER_CLICK after BROWSER_NAVIGATE in same turn: we have state from NAVIGATE once it returns; but NAVIGATE runs before CLICK, so when CLICK runs we already have state from NAVIGATE). So cache must be set when we return from NAVIGATE/CLICK/INPUT. When we're about to run CLICK, the cache holds the state from the *previous* tool result (same turn or previous turn), which is the page we're on. Correct.

### Optional

- Same for **BROWSER_INPUT**: "✍️ Typing into element 7 (Search box)…" using `get_last_element_label(7)`.

### Files

- `src-tauri/src/browser_agent/mod.rs`: add `LAST_BROWSER_STATE`, set it in navigate/click/input paths; add `get_last_element_label`.
- `src-tauri/src/browser_agent/http_fallback.rs`: set cache when returning state from navigate/click/input (and expose or use a shared cache – e.g. same `browser_agent::set_last_browser_state` from fallback).
- `src-tauri/src/commands/ollama.rs`: for BROWSER_CLICK (and optionally BROWSER_INPUT), call `get_last_element_label(idx)` and include label in `send_status`.

---

## 2. Browsers popping up again and again (loop prevention)

### Suspected cause

- Model keeps emitting BROWSER_NAVIGATE / BROWSER_CLICK (or multiple NAVIGATEs) without reaching DONE or a final answer.
- Each NAVIGATE might be opening a new window/tab, or the same window is reused but the user sees many "Navigating…" / "Clicking…" lines and perceives "lots of browsers".
- No guardrail to limit consecutive browser actions or to detect repetitive behaviour (same URL, same index, or too many browser tools in one run).

### Mitigations (plan)

1. **Log and monitor**
   - Ensure we log each BROWSER_NAVIGATE (URL) and BROWSER_CLICK (index) in the agent router (already have `info!("BROWSER_CLICK: index {}", idx)` etc.).
   - Add a short line when browser tools run: e.g. "Agent router: browser tool #N this run (NAVIGATE|CLICK|…)". Then in `~/.mac-stats/debug.log` we can see how many browser actions per run and spot loops.

2. **Cap browser actions per run (safety limit)**
   - In `ollama.rs` tool loop, maintain a counter for "consecutive or total browser tools" this run (e.g. `browser_tool_count` incremented for BROWSER_NAVIGATE, BROWSER_CLICK, BROWSER_INPUT, BROWSER_SCROLL, BROWSER_EXTRACT, BROWSER_SEARCH_PAGE).
   - If `browser_tool_count` exceeds a limit (e.g. 15 or 20), either:
     - Stop running more browser tools and return a message to the model: "Maximum browser actions per run reached (N). Reply with your answer or DONE: success / DONE: no." and break tool loop, or
     - Inject a system-like message into the conversation: "You have used many browser steps. If the task is done, reply with DONE: success; otherwise summarize what is left."
   - Prefer a hard cap so we never do e.g. 50 NAVIGATEs in one run.

3. **Repetition detection (optional)**
   - Track last few (e.g. 3) (tool, arg) pairs for browser tools. If we see the same BROWSER_NAVIGATE URL twice in a row, or the same BROWSER_CLICK index twice in a row, optionally refuse the duplicate and tell the model: "Same browser action as previous step; use a different action or reply with DONE."
   - Reduces accidental loops where the model retries the same click/nav.

4. **Encourage DONE after screenshot**
   - In agent base tools / planning prompt, reinforce: "After BROWSER_SCREENSHOT: current, reply with DONE: success and a short confirmation. Do not continue navigating or clicking unless the user asked for more."
   - Reduces unnecessary extra browser steps after the screenshot is taken.

5. **Headless by default for non-interactive**
   - If the request comes from Discord or the scheduler (no user in front of the CPU window), consider defaulting to headless (no visible window) so "browsers popping up" is at least not visible. Already have `set_prefer_headless_for_run(question.contains("headless"))`; could extend to "if from Discord/scheduler and question doesn't say 'browser' or 'visible', prefer headless". Optional and UX-dependent.

### Implementation order

1. **Phase 1 (quick)**: Add `get_last_element_label` + cache and use it in status ("Clicking element 7 (Accept all)…"). Improves observability immediately.
2. **Phase 2**: Add browser-tool counter and hard cap (e.g. 15) per run; when exceeded, stop browser tools and prompt model to reply or DONE. Log "browser tool #N" for each browser tool.
3. **Phase 3 (optional)**: Repetition detection; DONE reminder in prompts; headless default for Discord/scheduler.

### Files (Phase 2)

- `src-tauri/src/commands/ollama.rs`: in the tool loop, add `browser_tool_count: u32`, increment for each BROWSER_* tool; if `browser_tool_count >= MAX_BROWSER_TOOLS_PER_RUN`, return a message and break (or inject a user message and continue once). Define `MAX_BROWSER_TOOLS_PER_RUN` (e.g. 15).

---

## 3. 025 alignment

- **Status messages (reasoning + emojis)** (025 table): Already "Done" with URL, index, direction, etc. This plan adds *element label* to click (and optionally input), so status remains consistent with 025 and improves clarity.
- **DONE tool**: Encouraging DONE after screenshot and capping browser steps both reduce runaway runs without changing the expectation-check or verification logic.

---

## 4. Check the log

When reproducing:

- Truncate log: `: > ~/.mac-stats/debug.log`
- Trigger the run (ask the question).
- Inspect: `tail -100 ~/.mac-stats/debug.log` (or more) and look for:
  - Repeated "BROWSER_NAVIGATE: URL" / "BROWSER_CLICK: index" lines.
  - Whether the same URL or index appears many times.
  - How many browser tools run before the run ends or errors.

Use that to confirm loop pattern and tune the cap or repetition logic.

---

## Implementation status

- **Phase 1 (element label):** Done. `browser_agent` has `LAST_ELEMENT_LABELS` cache, `set_last_element_labels()`, `get_last_element_label(index)`. CDP and HTTP fallback set the cache after navigate/click/input. In `ollama.rs`, BROWSER_CLICK and BROWSER_INPUT status messages show the label when available (e.g. "🖱️ Clicking element 7 (Accept all)…", "✍️ Typing into element 4 (Search box)…").
- **Phase 2 (browser tool cap):** Done. In `ollama.rs` tool loop: `browser_tool_count` and `MAX_BROWSER_TOOLS_PER_RUN` (15). Before running any BROWSER_* tool we check the cap; if exceeded we push "Maximum browser actions per run reached (15). Reply with your answer or DONE: success / DONE: no." and skip that tool. Log line: "Agent router: browser tool #N/15 this run".
