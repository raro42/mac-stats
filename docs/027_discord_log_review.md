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
