# Reviewing logs when asking Werner (Discord bot)

## Where logs are

- **Path:** `~/.mac-stats/debug.log`
- **View live:** `tail -f ~/.mac-stats/debug.log`
- **Verbosity:** Start mac-stats with `-v`, `-vv`, or `-vvv` for more detail.

## What to search for

When you ask Werner something (e.g. "go to www.amvara.de and take a screenshot"):

| Search term | Meaning |
|-------------|---------|
| `Discord/Ollama:` | Tool the agent chose (e.g. `FETCH_URL requested`, `BROWSER_SCREENSHOT requested`) |
| `BROWSER_SCREENSHOT: arg from parser` | Raw argument from the model (before URL cleaning) |
| `BROWSER_SCREENSHOT: URL sent to CDP` | Exact URL passed to the browser (after trimming trailing punctuation) |
| `Agent router: understood tool` | Which tool and argument were parsed from the model reply |
| `Agent router: running tool` | Which tool is being executed |
| `FETCH_URL` | Fetching page text (no screenshot) |
| `BROWSER_SCREENSHOT` | Opening URL in browser and saving a screenshot PNG |
| `Discord←Ollama: received` | Final reply text sent back to Discord |
| `Discord: sent N attachment(s)` | Screenshot file(s) were posted to the channel |

## CDP / browser agent logs (BROWSER_SCREENSHOT)

When a screenshot is taken, the browser agent logs with a `[CDP]` prefix:

| Log line | Meaning |
|----------|---------|
| `Browser agent [CDP]: take_screenshot called with url (raw):` | URL string as received by the browser module |
| `Browser agent [CDP]: normalized URL:` | URL after adding `https://` if missing |
| `Browser agent [CDP]: reusing Chrome on port 9222` / `no Chrome on port ..., launching` | Whether an existing Chrome was used or a new one started |
| `Browser agent [CDP]: navigating to:` | URL used for `navigate_to` |
| `Browser agent [CDP]: navigated; final tab URL:` | URL after navigation (redirects, error pages) |
| `Browser agent [CDP]: page title:` | Document title (e.g. "404 - GitHub") |
| `Browser agent [CDP]: page appears to be 404 or not found` | Warning when title suggests 404 |
| `Browser agent [CDP]: screenshot saved to` | Path of the saved PNG |

Use these to see whether the URL sent to CDP was wrong (e.g. trailing dot, truncated) or the server really returned 404.

## URL parsing (FETCH_URL and BROWSER_SCREENSHOT)

The agent takes only the first token (up to the first space) as the URL. The model sometimes appends a sentence after the URL, e.g.:

`BROWSER_SCREENSHOT: https://github.com/foo/bar. The screenshot will be saved...`

Without cleaning, the URL would be `https://github.com/foo/bar.` (trailing dot), which can cause 404s. The parser now **strips trailing sentence punctuation** (`.`, `,`, `;`, `:`) from the URL before calling CDP or fetch. So the URL sent is `https://github.com/foo/bar`.

## "Go to URL and take a screenshot"

- The agent uses **BROWSER_SCREENSHOT: &lt;URL&gt;**. For "go to www.amvara.de and take a screenshot" you should see:
  - `BROWSER_SCREENSHOT: arg from parser (raw): ...` and `BROWSER_SCREENSHOT: URL sent to CDP: https://www.amvara.de`
  - `Browser agent [CDP]:` lines (normalized URL, navigating, final tab URL, page title, screenshot saved)
  - The text reply to the user, then a follow-up message with the screenshot **attached** (if the bot has permission and the path is under `~/.mac-stats/screenshots/`).

Restart mac-stats after code changes so the new behaviour is loaded.

## PERPLEXITY_SEARCH (search → visit URLs → screenshot)

When the user asks to search the web (e.g. Perplexity) and to visit URLs or get screenshots "here" or "in Discord", the planner recommends **PERPLEXITY_SEARCH: &lt;query&gt;**. The app then:

1. **Truncates the search query** so the API gets only the query (e.g. "spanish newspaper websites"), not the rest of the plan (e.g. "then BROWSER_NAVIGATE: ..."). Truncation uses separators: ` then `, ` and then `, ` → `, `BROWSER_NAVIGATE:`, `BROWSER_SCREENSHOT:`, etc., and a 150-character cap.
2. **Runs Perplexity search** and gets results with URLs.
3. **If the question asked for screenshots** (e.g. "screenshot", "visit", "send me … in Discord"), the app **auto-visits** the first 5 result URLs and **takes a screenshot** of each, then attaches them in Discord.

### What to search for in the log

| Search term | Meaning |
|-------------|---------|
| `PERPLEXITY_SEARCH requested:` | Query sent to Perplexity (after truncation) |
| `Agent router: running tool … PERPLEXITY_SEARCH` | Tool and arg (arg is truncated in parser) |
| `Agent router: auto-visit and screenshot for N URLs` | User asked for screenshots; visiting first N result URLs |
| `Agent router: auto-screenshot N saved to` | Screenshot PNG path for the N-th visited page |
| `Discord: sent N attachment(s)` | Screenshot(s) posted to the channel |

If the model recommended a long line (e.g. `PERPLEXITY_SEARCH: spanish newspapers then BROWSER_NAVIGATE: https://...`), the log will show the truncated query only (e.g. `PERPLEXITY_SEARCH requested: spanish newspapers`).

### Log review

When debugging a Perplexity search + screenshot run, search the log for the terms in the table above in order: confirm `PERPLEXITY_SEARCH requested:` shows the truncated query, then `Agent router: auto-visit and screenshot for N URLs` if the user asked for screenshots (e.g. "send me screenshots in Discord"), then each `Agent router: auto-screenshot N saved to`, and finally `Discord: sent N attachment(s)`.

---

## Feedback

**TASK_APPEND:** Auto-visit + screenshot workflow for Perplexity search results: implemented in `commands/ollama.rs` (query truncation, `want_screenshots` detection including "send me", "in discord", "send the", " here "; first 5 result URLs visited via browser_agent, screenshots saved and attached in Discord). Log review section added above. `want_screenshots` broadened so "send me the screenshots in Discord" triggers the workflow without requiring "visit" or "url" in the question.

**TASK_STATUS:** finished
