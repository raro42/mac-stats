# task-007: Read attachments in Discord message

**Goal:** When a user sends a Discord message with image attachment(s) (e.g. screenshot from spiegel.de), the bot should download the attachment(s), pass them to Ollama as vision input, and answer based on the image content (e.g. "what is on this screenshot?").

**Plan:**
1. In Discord message handler: collect image attachments (content_type image/* or filename .png/.jpg/.jpeg/.gif/.webp), download via Serenity `attachment.download().await`, base64-encode, pass to Ollama.
2. Add parameter `attachment_images_base64: Option<Vec<String>>` to `answer_with_ollama_and_fetch`; when building the first user message (question), set `images: attachment_images_base64`.
3. If message content is empty but there are image attachments, use a default prompt e.g. "What do you see in the attached image(s)? Describe the content."
4. Ensure we don't ignore messages that have only attachments (no text) — treat as valid request.

**Relevant code:**
- `src-tauri/src/discord/mod.rs`: `EventHandler::message`, where we read `new_message.content` and call `answer_with_ollama_and_fetch`. Add: read `new_message.attachments`, filter images, download, base64, pass to answer_with_ollama_and_fetch.
- `src-tauri/src/commands/ollama.rs`: `answer_with_ollama_and_fetch` — add param `attachment_images_base64: Option<Vec<String>>`; when pushing the user message that contains the question, set `images: attachment_images_base64`.

**Acceptance:** User can send a message in #agents-general with an attached screenshot (e.g. from https://www.spiegel.de/); bot replies describing or answering based on the image content (vision model used when images present).
