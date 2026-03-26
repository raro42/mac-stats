## Install

- **DMG (recommended):** [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.
- **Build from source:**
  ```bash
  git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
  ```

  Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

- **If macOS blocks the app:** Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Single-instance behavior
`mac-stats` is designed to run as a single live process. If you launch it a second time (Finder, CLI, or a double-click race), the app will detect the existing instance, log a warning to `~/.mac-stats/debug.log`, and exit immediately to avoid split-brain state (Discord/scheduler/CDP).

## 1. Tool agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## 2. Discord Agent (Gateway + HTTP API)

mac-stats can run a Discord bot that connects via the **Gateway** and responds to **DMs** and **@mentions**. In guild channels configured as **`mention_only`**, a **reply** to the bot’s last message (without toggling the mention ping) also counts as addressing the bot—the handler treats a **message reference** to a message authored by the bot like an @mention. Replies use the Ollama agent pipeline and can call the **Discord HTTP API** (list servers, channels, members, get user info, send messages). The bot records the message author’s display name and passes it to Ollama so it can address the user by name.


### Bot functionality at a glance

- **Triggers:** Responds to direct messages; in guild channels, to messages that **@mention** the bot, or (for **`mention_only`** channels) to **replies** to the bot’s own messages even when the reply does not include a literal `<@id>` mention.
- **Reply pipeline:** Ollama + tools (FETCH_URL, BRAVE_SEARCH, RUN_CMD, BROWSER_SCREENSHOT, DISCORD_API, SCHEDULE, MCP, etc.); planning step then execution; platform formatting for Discord (bullets, link wrapping).
- **Personalization:** Records your display name per channel; tells Ollama who it is talking to.
- **Session and memory:** Per-channel session files; say a reset phrase (any language) to clear context and start fresh.
- **Scheduling:** SCHEDULE (cron or one-shot) and REMOVE_SCHEDULE; schedule ID returned so you can cancel later.
- **Optional:** having_fun channels (casual-only persona); DISCORD_API for listing servers/channels/members and sending messages; View logs in Settings.

## Setup

1. **Create a Discord application** at [Discord Developer Portal](https://discord.com/developers/applications) → New Application.
2. **Create a Bot** in the application: Bot → Add Bot → copy the **Token** (this is your "Discord API key").
3. **Enable intents** (Bot → Privileged Gateway Intents):
   - **Message Content Intent** must be ON (required to read message text).
4. **Provide the token** (checked in this order; first wins):
   - **DISCORD_BOT_TOKEN** environment variable (e.g. `export DISCORD_BOT_TOKEN=your_token` or use `./scripts/run_with_discord_token.sh`).
   - **.config.env** file in the current working directory or in `~/.mac-stats/.config.env`. Use a line like `DISCORD_BOT_TOKEN=...` or `DISCORD-USER1-TOKEN=...`.
   - **In-app (Keychain)**: Open the CPU window → Settings (gear) → under **Discord bot** paste your token and click **Save token**. Stored in macOS Keychain; gateway connects right away.
   - Or from devtools: `invoke('configure_discord', { token: 'YOUR_TOKEN' })`.
5. **Clearing the token**: Use **Clear token** in Settings (removes from Keychain). Env and .config.env are not cleared by the app. To fully disconnect, restart mac-stats.
6. **Bot permissions for sending replies**: In each channel where the bot should reply, it needs **Send Messages** and **View Channel**. When inviting the bot to a server, use the OAuth2 URL with scope `bot` and enable **Send Messages** and **View Channel** (and **Attach Files** if you use screenshot/attachment replies). If the bot lacks these permissions, you will see "Missing Permissions" in logs and the app will try to post a short fallback message ("Reply could not be sent to this channel (missing permissions)..."). See `~/.mac-stats/debug.log` for the exact permission hint (channel id and suggested scopes).

## 3. Tauri commands

- **`configure_discord(token: Option<string>)`**  
  Set token (store in Keychain) or clear it. Pass `null` to remove. When a token is saved, the gateway starts immediately. (Env and .config.env are read automatically; use this to persist from the UI.)
- **`is_discord_configured()`**  
  Returns `true` if a token is available (from env, .config.env, or Keychain). Does not reveal the token.

## 4. Behavior

- **Inbound debounce (full router only):** Rapid messages in the **same channel** that use the full Ollama agent router (not having_fun’s casual buffer) are merged into **one** Ollama run after a short quiet period. Default wait is **2000 ms** in `~/.mac-stats/config.json` as **`discord_debounce_ms`** (0 = disable debouncing). Override with env **`MAC_STATS_DISCORD_DEBOUNCE_MS`**. Per-channel: in `discord_channels.json`, on a channel object set **`debounce_ms`** (milliseconds, 0 = immediate) or **`immediate_ollama`:** `true`. **Bypass debouncing** (handled immediately): any message with **attachments**, content starting with **`/`**, **`new session:`** / session-reset phrases, or image payloads for vision. On app disconnect, any **still-queued** batch is **discarded** and logged (no flush).
- **Draft reply (progress while tools run):** For the full agent router path, the bot posts **“Processing…”** immediately, then **edits that same message** with throttled updates such as **“Running FETCH_URL…”** while tools execute. When the final answer is ready, the bot **replaces** the draft with the first chunk of the reply (up to Discord’s 2000-character limit); any additional chunks are sent as normal follow-up messages. Configure the minimum interval between draft edits with **`discord_draft_throttle_ms`** in `config.json` (default **1500** ms; clamped 200–60000) or env **`MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`**. Logs for edits use target **`discord/draft`**. If the placeholder message cannot be sent, the bot falls back to the previous behaviour (typing indicator only, then new messages for the full reply). Implementation: `commands/discord_draft_stream.rs`, `discord/mod.rs`, `commands/tool_loop.rs`, `commands/ollama.rs`.
- **Ordered outbound pipeline:** Final text is split in `commands/outbound_pipeline.rs` (Discord limit **2000** characters; optional paragraph-aware splitting when the model uses **`[[split_long]]`**). Chunks are sent **one after another** with a short pause between messages, a **per-send wall-clock timeout** (default **10** seconds per chunk; override with env **`MAC_STATS_OUTBOUND_SEND_TIMEOUT_SECS`**, clamped 1–120), and **deduplication** so identical chunk payloads are not sent twice in the same reply. On timeout, remaining chunks are skipped and a short user-visible line may be sent. The same splitter and timeouts apply to **having_fun** and **idle thought** multi-chunk replies. Logs may include tracing target **`outbound_pipeline`**.
- **Outbound file attachments:** File paths returned by the agent (e.g. **BROWSER_SCREENSHOT**) are attached only if they resolve under the **shared outbound allowlist** after canonicalization: **`~/.mac-stats/screenshots/`**, **`~/.mac-stats/pdfs/`**, and **`~/.mac-stats/browser-downloads/`** (each tree is honored only once that directory exists on disk). To widen sources deliberately, add **`extraAttachmentRoots`** to `config.json` — a JSON array of directory paths (`~/…`, absolute, or relative to `$HOME`). Each entry must resolve under canonical **`$HOME/.mac-stats`** or **`$HOME`** (symlinks are followed); anything else is ignored with a log line. Arbitrary paths from tool output cannot be attached. Code: `security/attachment_roots.rs`, `discord/mod.rs` (`send_message_to_channel_with_attachments` and the gateway path use the same check).
- **Outbound circuit breaker:** Discord **HTTP** sends (`send_message_to_channel`, `send_message_to_channel_with_attachments`, and **DISCORD_API** `POST /channels/{id}/messages`) share a per-service breaker (threshold **3**, reset **30s**): repeated **5xx** or connection-class transport failures open the circuit so attempts fail fast with a log; **429** still uses Retry-After retries and does **not** advance the breaker. Gateway/WebSocket `say` paths are unchanged. Transition lines use tracing target **`mac_stats::circuit`** (see `docs/039_mac_stats_log_subsystems.md`).
- **Platform formatting:** When the reply is sent to Discord, the system prompt includes **Platform formatting (Discord)** so the model avoids markdown tables (uses bullet lists instead) and wraps links in `<>` to suppress embeds (e.g. `<https://example.com>`). This keeps messages readable and reduces embed clutter in the channel.
- Listens for **direct messages** to the bot and for **messages that @mention the bot** in guilds (and, in **`mention_only`** mode, replies that reference the bot’s message).
- Ignores the bot’s own messages. In **`mention_only`** guild channels, ignores messages that neither @mention the bot nor reply to a message authored by the bot.
- Replies using the **“answer with Ollama + tools”** pipeline: planning step (RECOMMEND) then execution with FETCH_URL, BRAVE_SEARCH, RUN_CMD, MCP, **DISCORD_API**, etc. (see `docs/100_all_agents.md`).
- When you message the bot, it records your **display name** and tells Ollama “You are talking to **&lt;name&gt;** (user id: …)” so replies can be personalized. Names are cached for reuse in the session.
- **Having_fun channels:** Replies and idle thoughts always use a **casual-only** system prompt (no work/Redmine soul). If a channel is configured with an `agent` override in `discord_channels.json`, that override is **ignored** for having_fun so the persona stays consistent; the optional channel `prompt` and time-of-day guidance are still applied. On LLM timeout or failure (e.g. Ollama busy), the bot posts a short user-friendly message only (e.g. “Something went wrong on my side — try again in a bit.”). Technical errors and CLI hints are never sent to the channel; the real error is logged to `~/.mac-stats/debug.log`. Idle thoughts retry once on timeout before giving up. Agent failure notices (e.g. that message or "Agent failed before reply") are **not** stored in the channel's session memory and are **filtered out** when building the idle-thought or reply context, so the model is never asked to "reply" to an error line and the casual tone is preserved. **Group-chat guidance** is also included for having_fun: know when to speak; one response per message (no triple-tap); use **REACT: &lt;emoji&gt;** (e.g. `REACT: 👍`) when a full reply isn't needed — the bot will add that emoji as a reaction and not send text; participate without dominating.
- **Guild channels (all_messages / mention_only):** When the reply target is a guild channel (not a DM), the system prompt includes **group channel** guidance: reply when mentioned or when adding value; at most one substantive reply per message; do not expose the user's private context in the channel.

## 5. Faster model for Discord (optional)

When no channel or message overrides the model, the app uses a default. To get **faster replies** (e.g. for browser/screenshot flows), set **OLLAMA_FAST_MODEL** in env or `~/.mac-stats/.config.env` (e.g. `OLLAMA_FAST_MODEL=qwen2.5:1.5b` or `qwen2.5:latest`). The agent router will use that model for planning and tool runs. **OLLAMA_MODEL** still overrides the UI/default elsewhere when set.

## 6. Session reset and memory (clear context)

To **start a fresh conversation** in a channel (clear prior context), say a phrase that means "clear session" or "new topic" — the bot clears the session for that channel and replies without old context. **Phrases are matched in any language** (substring, case-insensitive).

## 7. Discord API (HTTP)

When a request comes **from Discord**, the agent router adds a **DISCORD_API** tool so Ollama can call Discord’s REST API. This is only available in the Discord context (not in the scheduler or CPU-window chat).

### Endpoints available to the agent

Base URL: `https://discord.com/api/v10`. Paths are relative to this base.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/users/@me` | Current bot user. |
| GET | `/users/@me/guilds` | List servers (guilds) the bot is in. Optional query: `?with_counts=true`. |
| GET | `/guilds/{guild_id}/channels` | List channels in a server. |
| GET | `/guilds/{guild_id}/members?limit=100` | List members (use `after=user_id` for pagination). |
| GET | `/guilds/{guild_id}/members/search?query=...` | Search members by nickname/username. |
| GET | `/users/{user_id}` | Get user by ID. |
| GET | `/channels/{channel_id}` | Get channel. |
| POST | `/channels/{channel_id}/messages` | Send message. Body: `{"content":"..."}`. |

### DISCORD_API tool

Ollama invokes the Discord API by replying with exactly one line:

- **Read**: `DISCORD_API: GET <path>`  
  Example: `DISCORD_API: GET /users/@me/guilds`
- **Send message**: `DISCORD_API: POST /channels/<channel_id>/messages {"content":"Hello"}`  
  The path and optional JSON body are on the same line (body after a space and `{`).

Only **GET** is allowed for general read endpoints. **POST** is allowed only for `/channels/{channel_id}/messages`. All Discord HTTP calls honour **429 rate limits**: on a 429 the app parses `Retry-After` (header or JSON body), waits that duration plus jitter, and retries up to 3 times. Each 429 is logged at warn level. Heavy use may still hit Discord’s rate limits; see [Discord rate limits](https://discord.com/developers/docs/topics/rate-limits).

**Safe vs unsafe retries (outbound messages):** Sending a message is not idempotent. The app retries only when failure likely happened *before* Discord accepted the message (e.g. connection refused, DNS failure) or on explicit **429** rate limits (Gateway replies use a longer backoff for rate-limit errors). It does **not** retry on request timeouts, connection reset, or similar ambiguous cases where the first attempt may already have been delivered—this avoids duplicate posts. The same principle applies to any future non-idempotent outbound channel (e.g. Telegram). Implementation: `discord/api.rs` (`is_safe_to_retry_discord_*`), Gateway send path in `discord/mod.rs`, and HTTP `send_message_to_channel*`.

## 8. SCHEDULE and REMOVE_SCHEDULE (reminders and recurring tasks)

The agent can add entries to `~/.mac-stats/schedules.json` via the **SCHEDULE** tool. When a schedule is added from Discord, the agent is given a **schedule ID** (e.g. `discord-1770648842`) and is instructed to tell the user so they can remove it later. When the scheduler runs a task and posts the result to Discord, the message includes the schedule ID (e.g. `[Schedule: discord-1770648842]`) for context.

To remove a schedule, the user can say e.g. **"Remove schedule: discord-1770648842"**. The agent will call **REMOVE_SCHEDULE: &lt;schedule-id&gt;** and the entry is removed from `~/.mac-stats/schedules.json`.

### SCHEDULE formats (add)

1. **Recurring (every N minutes):** `SCHEDULE: every N minutes <task>` — e.g. `every 5 minutes Check CPU and reply here`.
2. **Recurring (cron):** `SCHEDULE: <cron expression> <task>` — cron is 6-field (sec min hour day month dow) or 5-field (min hour day month dow; the app prepends `0` for seconds). For patterns (every day at 5am, weekdays at 9am, etc.) see [crontab.guru examples](https://crontab.guru/examples.html); use the 6-field form or the standard 5-field form.
3. **One-shot (at a specific time):** `SCHEDULE: at <datetime> <task>` — run once at the given local time. Datetime must be ISO format: `YYYY-MM-DDTHH:MM:SS` or `YYYY-MM-DD HH:MM` (e.g. `at 2025-02-09T05:00:00 Remind me of my flight`). For "tomorrow 5am" the model can use `RUN_CMD: date +%Y-%m-%d` to get today's date and compute the next day.

### REMOVE_SCHEDULE (remove by ID)

- **Remove:** `REMOVE_SCHEDULE: <schedule-id>` — e.g. `REMOVE_SCHEDULE: discord-1770648842`. The agent invokes this when the user asks to remove, delete, or cancel a schedule and provides the ID (or the user said e.g. "Remove schedule: discord-1770648842").

### Customizing SCHEDULE behavior

You can cap how many schedules are allowed by setting **maxSchedules** in `~/.mac-stats/config.json` (e.g. `"maxSchedules": 20`). When the number of entries in `~/.mac-stats/schedules.json` reaches this limit, new SCHEDULE requests are rejected with a message asking the user to remove some or increase the limit. If maxSchedules is omitted or 0, there is no limit. Value is clamped to 1–1000.

## 9. Understanding servers, users, channels, guilds

- **Servers (guilds)**  
  Use `GET /users/@me/guilds` to see which servers the bot is in. Each guild has an `id` and `name`.

- **Channels**  
  Use `GET /guilds/{guild_id}/channels` to list channels in a server (text, voice, categories, etc.).

- **Users on a server**  
  Use `GET /guilds/{guild_id}/members` to list members, or `GET /guilds/{guild_id}/members/search?query=...` to search by nickname/username.

- **User names**  
  The bot records the display name of users who message it and passes it to Ollama. Ollama can also call `GET /users/{user_id}` for full user details (username, global_name, etc.).

- **User details (user-info.json)**  
  You can add per-user details in `~/.mac-stats/user-info.json` (many users, keyed by Discord user id). The file is read when handling a message; if the author’s id is present, the bot adds "User details: …" to the context (notes, timezone, extra fields). When a user messages the bot, their **display name** from Discord is written into the file if it differs from the stored value (or a minimal entry is added if the user was not yet in the file). For full field reference and examples, see **docs/data_files_reference.md**. Example:

  ```json
  {
    "users": [
      {
        "id": "123456789012345678",
        "display_name": "Alice",
        "notes": "Prefers short answers.",
        "timezone": "Europe/Paris",
        "extra": { "language": "en" }
      }
    ]
  }
  ```

  `id` is the Discord user id (snowflake) as a string. Optional: `display_name` (override), `notes`, `timezone`, `extra` (key-value). Ollama can read the file via `RUN_CMD: cat ~/.mac-stats/user-info.json` if needed.

## 10. Message prefixes (optional)

You can put **optional leading lines** in your message to override model, parameters, or skill for that request only. These lines are stripped before the question is sent to Ollama.

| Prefix | Example | Effect |
|--------|---------|--------|
| `model:` or `model=` | `model: llama3.2` | Use this model for this request (must be available in Ollama). |
| `temperature:` or `temperature=` | `temperature: 0.7` | Set temperature for this request. |
| `num_ctx:` or `num_ctx=` | `num_ctx: 8192` | Set context window size for this request. |
| `params:` | `params: temperature=0.7 num_ctx=8192` | Set multiple options in one line. |
| `skill:` or `skill=` | `skill: 2` or `skill: code` | Load `~/.mac-stats/agents/skills/skill-<number>-<topic>.md` and prepend its content to the system prompt. |
| `verbose` or `verbose: true` | (on its own line) | Show status/thinking messages in the channel for this request. |
| `verbose: false` | (on its own line) | Hide status messages for this request (default in channels). |

## 11. Security

- Token is resolved from **DISCORD_BOT_TOKEN** env, then **.config.env** (cwd or `~/.mac-stats/.config.env`), then **Keychain** (service `com.raro42.mac-stats`, account `discord_bot_token`). Using env or .config.env avoids Keychain entirely.
- **Secure token storage (recommended):** For production or any machine where the token should not live in a file or process environment, store the token in **Keychain** via Settings (CPU window → Settings → Discord bot → Save token). Keychain is macOS’s secure credential store; the app never writes the token to disk. Use **DISCORD_BOT_TOKEN** or **.config.env** only for development, CI, or one-off runs (e.g. `test_discord_connect`).
- The token is never logged or exposed in the UI or in error messages.
- **DISCORD_API** uses the same bot token and only allows **GET** (read) and **POST** to `/channels/{id}/messages` (send message). Other write operations are not exposed.

## 12. Testing the Discord connection

The **test_discord_connect** binary checks that your Discord bot token works without starting the full app (no menu bar, no Keychain). It connects to the Gateway, prints connection status, then exits after a configurable duration (default 15 seconds).

### How to run

From the repo (so the default env path resolves):

```bash
cd src-tauri && cargo run --bin test_discord_connect
```

With a custom env file path:

```bash
cargo run --bin test_discord_connect -- path/to/.config.env
```

To customize how long the binary stays connected before exiting (default 15s, range 1–300):

- **Quick check:** `--quick` or `-q` — runs for 2 seconds (enough to see "Bot connected" then exit). Example: `cargo run --bin test_discord_connect -- --quick` or `cargo run --bin test_discord_connect -- .config.env --quick`.
- **Environment:** `TEST_DISCORD_CONNECT_SECS=30` (e.g. `TEST_DISCORD_CONNECT_SECS=30 cargo run --bin test_discord_connect`).
- **CLI:** Optional second argument (seconds), or a single numeric argument to use default path and that duration:
  - `cargo run --bin test_discord_connect -- .config.env 30` — env file `.config.env`, run 30 seconds.
  - `cargo run --bin test_discord_connect -- 30` — default path `.config.env`, run 30 seconds.

### Token resolution

The binary resolves the token in this order:

1. **Environment:** `DISCORD_BOT_TOKEN` (if set).
2. **Env file:** First line in the given file that starts with `DISCORD-USER1-TOKEN=` or `DISCORD-USER2-TOKEN=`; the value after `=` is used. Default file path is `.config.env` in the current working directory.

Keychain is **not** used; this binary is for quick token checks (e.g. from a `.config.env` or CI). The main app uses Keychain when no env token is set.

### Success output

With a valid token you should see on stderr:

- `Discord: Connecting to Discord Gateway (discord.com)…`
- `Discord: Gateway client built, starting connection…`
- `Discord: Bot connected as <YourBotName> (id: …)`

The process then runs for the configured duration (default 15s, see above) and exits (the binary deliberately aborts the connection after that).

### No token / failure

If no token is found, the binary prints to stderr:

`No token: set DISCORD_BOT_TOKEN or put DISCORD-USER1-TOKEN=... in <path>`

and exits with code 1. Ensure the env file exists and contains a line like `DISCORD-USER1-TOKEN=your_bot_token` (or use `DISCORD_BOT_TOKEN` in the environment).

## 13. Debugging “Save token”

If clicking **Save token** does nothing or the app stalls:

1. **Run with verbose logging** so backend and Keychain are visible:
   ```bash
   cd src-tauri && cargo run -- --cpu -v
   ```
   Or with the run script: `./run dev -- --cpu -v`

2. **Watch the log file** in another terminal:
   ```bash
   tail -f ~/.mac-stats/debug.log
   ```

3. **Open Settings** in the app (gear icon) → paste a token → click **Save token**.

4. **In the log you should see** (if the backend is hit):
   - `Discord: configure_discord invoked (has_token=true, len=...)`
   - `Discord: Token stored (restart app to connect)`
   If these lines never appear, the click is not reaching the Rust command (e.g. invoke not available or handler not attached). If they appear and the UI still hangs, the hang is after the command (e.g. alert or refresh).

## 14. Testing the pipeline without Discord

To run the same Ollama+tools pipeline as a Discord DM (planning, tool loop, **headless** browser) without sending a real message:

```bash
cd src-tauri
./target/release/mac_stats -vv discord run-ollama 'Your question here'
```

Requires **Ollama** running and configured (env `OLLAMA_HOST` or `~/.mac-stats/.config.env`). The app ensures defaults and initializes the Ollama client before running. Reply and any attachment paths (e.g. screenshots under `~/.mac-stats/screenshots/`) are printed to stdout. Logs: `~/.mac-stats/debug.log` (e.g. `grep -E "headless|BROWSER_NAVIGATE|screenshot" ~/.mac-stats/debug.log`).

## 15. Optional post-run agent judge

When enabled, after each agent run completes (Discord reply or scheduler task), the app calls an LLM once to evaluate whether the task was satisfied and logs the verdict (and optional reasoning) to `~/.mac-stats/debug.log`. This is for **testing or quality logging** only; it does not change the agent loop or user-facing replies.

- **Config:** `~/.mac-stats/config.json` — set `"agentJudgeEnabled": true`. Or set env `MAC_STATS_AGENT_JUDGE_ENABLED=true` (or `1` / `yes`). Default is **false** (judge off).
- **Behaviour:** After the run, the judge receives: original task (truncated), final reply (truncated), step summaries (if any), and paths to the last 10 screenshots. It returns a structured verdict (success/failure, optional score, reasoning, impossible_task, reached_captcha). Parse failures are logged and treated as "unknown".
- **Cost:** One extra LLM call per run when enabled. Leave disabled in production unless you need quality metrics or test assertions.

## 16. Agent test (regression path)

To run a single agent with prompts from its `testing.md` (no Discord, no router):

```bash
./target/release/mac_stats agent test <selector> [path]
```

Example: `./target/release/mac_stats agent test redmine` runs the Redmine agent’s test prompts from `~/.mac-stats/agents/agent-006-redmine/testing.md`. Each prompt is limited by a **per-prompt timeout** (default 45s); if the model doesn’t respond in time, the run fails with a clear message instead of hanging. Override: env `MAC_STATS_AGENT_TEST_TIMEOUT_SECS` or config.json `agentTestTimeoutSecs` (5–300). Useful as a regression check after changes to agents or Ollama integration.

## 17. Tool budget warning and last-iteration guidance

When the agent approaches the tool iteration cap (`max_tool_iterations`), the system can inject a warning message into the conversation so the model consolidates results instead of starting another tool chain that would be cut off. Inspired by browser-use's step budget warning.

- **Config:** `~/.mac-stats/config.json` — set `"toolBudgetWarningRatio": 0.75` (default). Or env `MAC_STATS_TOOL_BUDGET_WARNING_RATIO`. Value 0.0–1.0; set to 0.0 or 1.0 to disable.
- **Budget warning:** When `tool_count / max_tool_iterations >= ratio` (and it is not the last iteration), a system message is injected: "BUDGET WARNING: You have used X/Y tool iterations (Z%). N remaining. Consolidate results or call DONE. Partial results are far more valuable than exhausting all iterations with nothing saved."
- **Last-iteration guidance:** When `tool_count + 1 == max_tool_iterations`, a stronger system message is injected: "LAST ITERATION WARNING: This is your LAST tool iteration. Reply with your final answer now." The tool list/schema is unchanged; this is guidance only.
- **Scope:** Only applies in the agent tool loop (Discord, scheduler, run-ollama). No new tools are added.
- **Disabling:** Set ratio to 0.0 or 1.0 to preserve current behaviour (no warnings injected).

## 18. Optional tool-loop repeat detection (OpenClaw-style)

When **disabled** (default), the agent tool loop still uses the **legacy** guard: the same tool with the same argument is blocked after **3** invocations, and short **ping-pong** patterns (e.g. A→B→A→B with at least two distinct tool signatures) are blocked before the tool runs.

When **enabled**, the legacy same-call limit is **replaced** by hash-based tracking:

- **History:** Up to `historySize` recent invocations (default **25**, clamped 10–60), keyed by tool name and **SHA-256** of **normalized** arguments (for common URL tools: lowercase host, strip trailing slashes on the path, preserve query/fragment).
- **Warning:** If the same tool+argument signature appears at least `warningThreshold` times in that window (default **8**), a short **`[SYSTEM]`** note is appended to that tool’s result so the model sees it on the next turn.
- **Critical:** If the count reaches `criticalThreshold` (default **12**, always kept above the warning threshold), the run **stops** and the user sees: *“Same action was repeated too many times in this session. Try rephrasing your request or starting a new topic.”*
- **Ping-pong:** Before each tool runs, the same cycle detection as legacy applies, but **pure** repeats (AAAA…) are **not** treated as cycles; those are handled only by the repeat counter.

**Config:** `~/.mac-stats/config.json` — object `toolLoopDetection`:

```json
"toolLoopDetection": {
  "enabled": true,
  "historySize": 25,
  "warningThreshold": 8,
  "criticalThreshold": 12
}
```

**Env:** `MAC_STATS_TOOL_LOOP_DETECTION_ENABLED` — `true` / `1` / `yes` forces enable (still uses JSON sizes/thresholds when present); `false` / `0` / `no` forces **off** even if JSON has `"enabled": true`.

**Scope:** Discord bot, scheduler, and `run-ollama` agent path (`commands/tool_loop.rs`). No new tools are added.

## Open tasks

Discord-related open tasks and completed items are tracked in **006-feature-coder/FEATURE-CODER.md** (table “Remaining open” and “Current FEAT backlog”).