---
## Triage summary (TOP)

- **Coder (UTC):** 2026-03-28 — Implementation already in `message_debounce.rs` / `discord/mod.rs` / `config`. Added call-site comment in `discord/mod.rs` naming `message_debounce::discord_message_bypasses_debounce` so static `rg` over `mod.rs` finds all three debounce entry points (`enqueue_or_run_router`, `discord_message_bypasses_debounce`, `discard_pending_batches_on_shutdown`). *(Repo has no `002-coder-backend/CODER.md`; FEAT backlog lives in `agents/006-feature-coder/FEATURE-CODER.md`.)*
- **Next step:** Tester runs **§4 Testing instructions** below; live Discord multi-message merge and shutdown discard still need operator or staging. Older closure notes remain in `agents/tasks/CLOSED-20260322-0100-openclaw-inbound-message-debounce.md`.
---

# UNTESTED: OpenClaw-style Discord inbound message debounce

**Created (UTC):** 2026-03-22 01:00  
**Coder handoff (UTC):** 2026-03-28  
**Source:** OpenClaw vs mac-stats review  
**Topic:** Debounce rapid full-router Discord messages per channel into one Ollama run after quiet period

---

## 1. Goal

**Full-router** Discord messages in the same channel are **debounced** into a single Ollama run after a quiet period; bypass rules (attachments, `/`, session reset, vision); merged text for same vs mixed authors; **discard** of pending batches on shutdown.

---

## 2. References

- `src-tauri/src/discord/message_debounce.rs` — `enqueue_or_run_router`, `discord_message_bypasses_debounce`, `merge_debounced_string_parts`, `discard_pending_batches_on_shutdown`, merge unit tests
- `src-tauri/src/discord/mod.rs` — `effective_discord_debounce_ms`, channel `debounce_ms` / `immediate_ollama`, wiring to `message_debounce`
- `src-tauri/src/config/mod.rs` — `discord_debounce_ms`, env `MAC_STATS_DISCORD_DEBOUNCE_MS`
- `docs/007_discord_agent.md` — operator-facing debounce behaviour

---

## 3. Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds (includes `message_debounce` merge unit tests).
3. **Static verification:** `enqueue_or_run_router`, `discord_message_bypasses_debounce`, and `discard_pending_batches_on_shutdown` are findable via `rg` in `src-tauri/src/discord/mod.rs` (call site / comment / shutdown hook).

---

## 4. Testing instructions

Run from the **repository root** (or adjust paths).

### Automated (required)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test merge_empty
cd src-tauri && cargo test message_debounce
cd src-tauri && cargo test
```

Optional spot-check (expect **three** matches in `mod.rs`: `enqueue_or_run_router`, `discord_message_bypasses_debounce` in the debounce comment, `discard_pending_batches_on_shutdown`):

```bash
rg -n "enqueue_or_run_router|discord_message_bypasses_debounce|discard_pending_batches_on_shutdown" src-tauri/src/discord/mod.rs
```

### Manual / staging (optional)

1. Set `discord_debounce_ms` in `~/.mac-stats/config.json` (e.g. **2000**) or `MAC_STATS_DISCORD_DEBOUNCE_MS`; restart mac-stats with Discord enabled.
2. In a **full-router** channel, send two short text messages within the debounce window; confirm a **single** Ollama run after quiet period and merged body (same author: newline-joined; mixed: `Name: text` lines). Watch `~/.mac-stats/debug.log` for `Discord debounce:` lines.
3. Confirm **bypass** paths trigger an immediate router run: message with **attachment**, content starting with **`/`**, session-reset phrasing, or **`immediate_ollama`** / **`debounce_ms: 0`** on the channel.
4. With a pending debounce batch, **quit** the app; log should show `Discord debounce: discarded … pending channel batch(es) on shutdown` when applicable.

---

## 5. Implementation summary

- Per-channel queue with generation counter; `tokio::sleep(debounce_ms)` then flush if generation unchanged.
- Non-text bypass and vision: `discord_message_bypasses_debounce` or non-empty `attachment_images_base64` → immediate `run_discord_ollama_router`.
- Shutdown: `disconnect_discord` → `message_debounce::discard_pending_batches_on_shutdown()` before gateway shutdown.
