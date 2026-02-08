# Discord Agent (Gateway)

mac-stats can run a Discord bot that connects via the Gateway and responds to **DMs** and **@mentions** with replies (stub for now; Ollama/browser pipeline to be wired later).

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

## Tauri commands

- **`configure_discord(token: Option<string>)`**  
  Set token (store in Keychain) or clear it. Pass `null` to remove. When a token is saved, the gateway starts immediately. (Env and .config.env are read automatically; use this to persist from the UI.)
- **`is_discord_configured()`**  
  Returns `true` if a token is available (from env, .config.env, or Keychain). Does not reveal the token.

## Behavior

- Listens for **direct messages** to the bot and for **messages that @mention the bot** in guilds.
- Ignores the bot’s own messages and messages that don’t mention it (in guilds).
- Replies using the **“answer with Ollama + fetch”** pipeline: planning step (RECOMMEND) then execution with FETCH_URL, BRAVE_SEARCH, RUN_CMD, MCP, etc. (see `docs/100_all_agents.md`).

### Message prefixes (optional)

You can put **optional leading lines** in your message to override model, parameters, or skill for that request only. These lines are stripped before the question is sent to Ollama.

| Prefix | Example | Effect |
|--------|---------|--------|
| `model:` or `model=` | `model: llama3.2` | Use this model for this request (must be available in Ollama). |
| `temperature:` or `temperature=` | `temperature: 0.7` | Set temperature for this request. |
| `num_ctx:` or `num_ctx=` | `num_ctx: 8192` | Set context window size for this request. |
| `params:` | `params: temperature=0.7 num_ctx=8192` | Set multiple options in one line. |
| `skill:` or `skill=` | `skill: 2` or `skill: code` | Load `~/.mac-stats/skills/skill-<number>-<topic>.md` and prepend its content to the system prompt. |

Example message:

```
model: llama3.2
skill: code
Write a small Python function to compute factorial.
```

See **Ollama context, model/params, and skills** in `docs/012_ollama_context_skills.md` for details.

## Security

- Token is resolved from **DISCORD_BOT_TOKEN** env, then **.config.env** (cwd or `~/.mac-stats/.config.env`), then **Keychain** (service `com.raro42.mac-stats`, account `discord_bot_token`). Using env or .config.env avoids Keychain entirely.
- The token is never logged or exposed in the UI or in error messages.

## Using .config.env or DISCORD_BOT_TOKEN (no Keychain needed)

You can run the Discord gateway **without Keychain** by using the environment or a config file. The app checks in order: `DISCORD_BOT_TOKEN` env → `.config.env` in current directory → `~/.mac-stats/.config.env` → Keychain.

1. **Option A – .config.env in project (development)**  
   In `src-tauri/.config.env` add one line:
   ```bash
   DISCORD-USER1-TOKEN=your_bot_token_here
   ```
   Or:
   ```bash
   DISCORD_BOT_TOKEN=your_bot_token_here
   ```
   Run from **src-tauri** so the current directory contains `.config.env`:
   ```bash
   cd src-tauri && cargo run -- --cpu -v
   ```
   You should see logs: `Discord: Token from .config.env (current dir)`, then `Token found, spawning gateway thread`, then `Bot connected as …`.

2. **Option B – Run script (exports env from .config.env)**  
   ```bash
   ./scripts/run_with_discord_token.sh --dev
   ```
   The script reads `src-tauri/.config.env` and sets `DISCORD_BOT_TOKEN`; the app uses it and never touches Keychain.

3. **Option C – ~/.mac-stats/.config.env (any working directory)**  
   Create `~/.mac-stats/.config.env` with the same line format. The app will use it regardless of where you start the process.

**Keychain (optional):**  
If you use **Save token** in the app, the token is stored in Keychain. If the OS never prompts for Keychain access or Keychain blocks, use Option A, B, or C instead. For Keychain troubleshooting (unlock, allow prompt), see **Granting the app access to Keychain** below.

**Keychain test (store + read back):**
```bash
cd src-tauri && cargo run --bin test_discord_keychain -- .config.env
```
Only needed if you rely on Keychain. If it hangs after “Token length: N chars”, use env or .config.env instead.

## Granting the app access to Keychain (optional)

Only needed if you use **Save token** in the app. mac-stats stores that token in the **login** keychain. If Keychain is locked or the app was never allowed, storage can block or fail.

1. **Unlock the login keychain** in Keychain Access (select **login** → unlock with macOS password).
2. **Run the app from Terminal** so any “X wants to access keychain” dialog is visible; click **Always Allow**.
3. If the OS never shows the dialog, use **.config.env or DISCORD_BOT_TOKEN** (see above) and skip Keychain.

## Testing the Discord connection

To verify the token without running the full app, use the test binary (reads from `.config.env` or `DISCORD_BOT_TOKEN`):

```bash
cd src-tauri && cargo run --bin test_discord_connect
```

With a valid token you should see in the output:
- `Discord: Connecting to Discord Gateway (discord.com)…`
- `Discord: Gateway client built, starting connection…`
- `Discord: Bot connected as <YourBotName> (id: …)`

You can pass a custom env file path: `cargo run --bin test_discord_connect -- path/to/.config.env`.

## Debugging “Save token”

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
