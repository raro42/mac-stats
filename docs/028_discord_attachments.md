# Discord attachments: Werner sending files

**Status: Implemented.** Werner can send screenshot attachments to Discord when BROWSER_SCREENSHOT is used.

## Implemented (summary)

- **OllamaReply** (`commands/ollama.rs`): `answer_with_ollama_and_fetch` returns `OllamaReply { text, attachment_paths }`. BROWSER_SCREENSHOT success pushes the PNG path into `attachment_paths`.
- **Gateway**: After sending the text reply, the Discord handler sends a follow-up message with the screenshot file(s) via Serenity `CreateMessage::new().content("Screenshot(s) as requested:").add_files(attachments)`.
- **REST**: `send_message_to_channel_with_attachments(channel_id, content, paths)` does multipart POST to Discord API; only paths under `~/.mac-stats/screenshots/` are allowed (`allowed_attachment_path`).
- **Config**: `Config::screenshots_dir()` returns `~/.mac-stats/screenshots/`; browser agent and attachment whitelist use it.

When a user asks Werner to take a screenshot of a URL, the bot posts the text reply and then attaches the PNG in the same channel (if the bot has permission to attach files).

---

## Original plan (reference)

### Previous behaviour

- **Sending text**: Two mechanisms (unchanged):
  1. **Gateway (Serenity)** in `discord/mod.rs`: `channel.say(&ctx, content)` for the main reply and “having fun” flows.
  2. **REST only**: `send_message_to_channel(channel_id, content)` in `discord/mod.rs` — used by CLI (`mac_stats discord send`) and scheduler. It does `POST /channels/{id}/messages` with JSON `{"content": "..."}`. No file support.

- **BROWSER_SCREENSHOT** (in `commands/ollama.rs`): Saves PNG under `~/.mac-stats/screenshots/`, returns the path to the model; the model’s reply is only text (e.g. “Screenshot saved to …”). The path is never passed back to the Discord layer to attach.

## What must be done

### 1. Return attachment path(s) from the agent

- **Where**: `answer_with_ollama_and_fetch` in `src-tauri/src/commands/ollama.rs`.
- **Change**: Return type today is `Result<String, String>` (reply text only). Extend it so the caller can get optional attachment path(s), e.g.:
  - New type: `pub struct OllamaReply { pub text: String, pub attachment_paths: Vec<PathBuf> }`, or
  - `Result<(String, Vec<PathBuf>), String>`.
- **In the tool loop**: When the tool is `BROWSER_SCREENSHOT` and the result is a success (path), push that path into a `Vec<PathBuf>` that is returned together with the final reply text. So the router returns both the model’s reply and any screenshot path(s) produced in that run.

### 2. Send messages with attachments in the Discord layer

Two call sites need to be able to send an attachment:

- **Gateway (main Discord reply)** in `discord/mod.rs` inside `EventHandler::message`:
  - After `answer_with_ollama_and_fetch` returns, if `attachment_paths` is non-empty, send the file(s) to the same channel.
  - Use Serenity’s API: e.g. `CreateMessage::new().content("…").add_file(CreateAttachment::path(&path).await?)` (exact types depend on Serenity 0.12). Send one message per file or one message with multiple attachments, depending on UX and Discord limits (e.g. 10MB per file, 25MB total per message).

- **REST path** (`send_message_to_channel` and any caller that doesn’t use the gateway):
  - Discord’s [Create Message](https://discord.com/developers/docs/resources/channel#create-message) endpoint accepts **multipart/form-data** when you send files: fields `content` (optional) and `files[n]` (file part with filename).
  - Add e.g. `send_message_to_channel_with_attachments(channel_id, content, paths: &[PathBuf])` that builds a multipart body (e.g. with `reqwest::multipart`) and POSTs to `https://discord.com/api/v10/channels/{id}/messages` with the same `Authorization: Bot <token>` header. Use this from the CLI/scheduler when you want to send a message plus file(s).

### 3. Optional: DISCORD_API tool

- Today the agent can use `DISCORD_API: POST /channels/{id}/messages` with a JSON body (text only). Supporting attachments from the agent would require either:
  - Allowing multipart in that helper (complex, and the agent would need to pass a file path or base64), or
  - Keeping attachment handling **only** in the app: when the router sees BROWSER_SCREENSHOT and returns attachment paths, the Discord handler sends them. No change to DISCORD_API is strictly required for “Werner sends the screenshot to the channel.”

### 4. Limits and safety

- **Discord**: Max 10MB per file, 25MB total per message; allowed types include PNG/JPEG/GIF. Screenshots are PNG and usually small.
- **Paths**: Only attach paths that the router explicitly returned (e.g. under `~/.mac-stats/screenshots/`). Validate path is under that dir (or a whitelist) so the app never sends arbitrary files.

## Summary

| Step | Where | What |
|------|--------|------|
| 1 | `commands/ollama.rs` | Return `(text, Vec<PathBuf>)` (or a struct) from `answer_with_ollama_and_fetch`; fill attachment list when BROWSER_SCREENSHOT succeeds. |
| 2a | `discord/mod.rs` (event handler) | After getting the reply, if attachment_paths non-empty, send message with file(s) via Serenity (e.g. `CreateMessage` + `CreateAttachment`). |
| 2b | `discord/mod.rs` | Add `send_message_to_channel_with_attachments(channel_id, content, paths)` using multipart POST for REST-only callers. |
| 3 | Optional | Restrict attachment paths to a whitelist (e.g. under `~/.mac-stats/screenshots/`). |

After this, when a user asks Werner to open a URL and take a screenshot, the bot can post the text reply **and** attach the PNG in the same channel.
