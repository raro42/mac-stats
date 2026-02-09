# AI Tasks Roadmap: Web, Mail, WhatsApp, Google Docs

## Phase 1: Web navigation and extraction (implemented)

- **Backend fetch**: Tauri command `fetch_page(url)` in `src-tauri/src/commands/browser.rs`. Server-side GET with timeout (15s) and same SSL policy as website monitors. Returns body as text (max 500k chars).
- **Tool protocol**: Ollama can request a page fetch by replying with exactly one line: `FETCH_URL: <full URL>`. The app fetches the page and sends the content back to Ollama; the model then answers the user (e.g. "navigate to www.amvara.de and retrieve the phone number").
- **Flow**: Handled inside `ollama_chat_with_execution`: after each Ollama response, if `FETCH_URL:` is present, we fetch the URL, append the page content to the conversation, and call Ollama again (up to 3 fetch iterations).
- **Docs**: See agents.md and CLAUDE.md for where Ollama + fetch is documented.

## Phase 2 and beyond: Mail, WhatsApp, Google Docs (roadmap only)

These are separate integrations, not "browser" in the same sense.

- **Mail**: e.g. IMAP/OAuth (Apple Mail, Gmail) via a dedicated module; credentials in Keychain; Ollama could get tool actions like `READ_MAIL_FOLDER:inbox` and the app returns summarized content.
- **WhatsApp**: WhatsApp Business API (official) or unofficial APIs; same pattern: secure credentials, dedicated commands, optional tool protocol so Ollama can request "last N chats" or "send message" with user confirmation.
- **Google Docs**: Google APIs (OAuth), read/list documents; dedicated backend commands + tool protocol.
- **Common pattern**: Each integration = new Tauri commands + optional Keychain credentials + a convention for Ollama to request an action (e.g. `ACTION=READ_MAIL`, `ACTION=WHATSAPP_RECENT`, `ACTION=GDOCS_LIST`) and the app returns structured text/JSON to Ollama for the model to summarize or act.
- **No implementation yet**; add step by step when needed.

## Discord agent (implemented)

- **Module**: `src-tauri/src/discord/mod.rs` (Serenity client, EventHandler for DMs and @mentions).
- **Credentials**: Discord Bot Token in Keychain (`discord_bot_token`); configure via Settings in the app.
- **Reply pipeline**: Discord handler uses the shared "answer with Ollama + fetch" API so replies can use Ollama and FETCH_URL when answering (see docs/007_discord_agent.md).

## Ollama connection / session handling (improvement, deferred)

- **Current behaviour**: A single `OllamaClient` (config + one `reqwest::Client`) is stored when the user configures Ollama. For **chat** (`send_ollama_chat_messages` in `commands/ollama.rs`), only the **config** (endpoint, model, API key) is read from that client; a **new** `reqwest::Client` is created per request and used for the POST. So we do not reuse the stored HTTP client or its connection pool for chat. Other Ollama operations (list models, etc.) also often construct a new client per call.
- **Ollama API**: Stateless; each `POST /api/chat` is independent (full `messages` in the body). There is no server-side "session"; "session" on our side means reusing the same TCP/HTTP connections and config.
- **Intended improvement**: Use the stored `OllamaClient`'s `client` (or a single shared `reqwest::Client`) inside `send_ollama_chat_messages` and other Ollama call paths instead of building a new client each time, so connections to local (or remote) Ollama are reused. Deferred for now; no implementation planned yet.
