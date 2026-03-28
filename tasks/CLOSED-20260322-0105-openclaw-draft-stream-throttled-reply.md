# UNTESTED — OpenClaw-style Discord draft stream / throttled reply (2026-03-22)

## Goal

Discord full-agent path behaves like OpenClaw-style progress: post a placeholder (“Processing…”), **edit the same message** on a throttle while tools run (e.g. “Running FETCH_URL…”), then **flush** the final reply into that message (first chunk; overflow chunks as separate messages). Operator reference: `docs/007_discord_agent.md`.

## Acceptance criteria

- `src-tauri/src/commands/discord_draft_stream.rs` implements throttled/coalesced draft updates and immediate flush.
- `spawn_discord_draft_editor` is used from `src-tauri/src/discord/mod.rs`; `DiscordDraftHandle` is threaded through `commands/tool_loop.rs`, `commands/turn_lifecycle.rs`, and `commands/ollama.rs`.
- Throttle interval is configurable via `discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`, clamped 200–60_000 ms (`config/mod.rs`).
- Unit tests cover `clamp_discord_content` (Discord length limit).

## Implementation summary (coder, 2026-03-28 UTC)

No Rust changes required in this pass: the criteria above are already implemented on the current tree. Verified locally: `cargo check`, `cargo test discord_draft_stream::` (2 tests), and static wiring checks with `rg` (see **Verification**).

## Verification

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Manual (optional):** live Discord router with tools — see **Testing instructions**.

---

## Testing instructions

### What to verify

- Full-agent Discord path posts a **placeholder**, then **throttled in-place edits** (e.g. `Running FETCH_URL…`) while tools run, then **replaces** that message with the **first chunk** of the final reply; content beyond Discord’s per-message limit continues as **new** messages (existing outbound behaviour).
- Throttle comes from **`discord_draft_throttle_ms`** in `config.json` or **`MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`**, clamped to **200–60_000** ms (`Config::discord_draft_throttle_ms()`).
- **`clamp_discord_content`** enforces the Discord character cap (covered by unit tests in `discord_draft_stream.rs`).

### How to test

1. From repo root, run the **Verification** commands above; all must succeed.
2. **Optional — live Discord:** Run mac-stats with Discord agent/router enabled and verbosity at least **`-v`** so `discord/draft` logs appear in `~/.mac-stats/debug.log`. Send a full-agent message that runs at least one tool (e.g. a request that triggers `FETCH_URL` or another tool). Confirm: a “Processing…” (or equivalent) message appears, then edits show `Running <tool>…` no faster than the configured throttle, then that same message is replaced by the start of the final answer. Inspect the log for lines with target **`discord/draft`** (placeholder / draft update / draft flush).
3. **Optional — throttle override:** Set `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS` to a value below 200 or above 60000 and confirm effective delay stays within **200–60_000** ms (by log timestamps or perceived edit cadence).

### Pass/fail criteria

- **Pass:** `cargo check` and `cargo test discord_draft_stream::` pass; `rg` shows `spawn_discord_draft_editor` in `discord/mod.rs` and `DiscordDraftHandle` used from `tool_loop.rs`, `turn_lifecycle.rs`, and `ollama.rs`. Optional live run matches the Goal (single message edited, then flushed to final text).
- **Fail:** Any compile or test failure; missing wiring; placeholder never edited; final answer only as new messages with no in-place flush; throttle clearly outside **200–60_000** ms for a given config/env input.

## Test report

- **Date:** 2026-03-28 UTC (tester run).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files (import/struct/parameter as listed in grep output).
- **Result:** **Pass** — automated acceptance criteria satisfied per task **Pass/fail criteria** (optional live Discord / throttle-override checks not run this run).
- **Notes:** Throttle config/clamp in `config/mod.rs` and full draft behaviour were not re-validated end-to-end against Discord in this pass; only compile, unit tests, and static wiring as specified in **Verification**.

### Tester run (2026-03-28 UTC, follow-up)

- **Note:** On disk the task was already `CLOSED-*` (no `UNTESTED-*` with this slug); renamed `CLOSED` → `TESTING` to follow TESTER.md flow, then back to `CLOSED` after verification.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files.
- **Config:** `Config::discord_draft_throttle_ms()` in `config/mod.rs` documents clamp **200..=60_000** and env `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS` / key `discord_draft_throttle_ms`.
- **Result:** **Pass** — automated acceptance criteria satisfied; optional live Discord checks not run this pass.

### Tester run (2026-03-28 UTC, TESTER.md single-task)

- **Note:** El operador pidió `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md`; ese prefijo no existía en el árbol (la tarea estaba como `CLOSED-*`). Se aplicó el flujo renombrando `CLOSED` → `TESTING` → `CLOSED` sin tocar otro `UNTESTED-*`.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files.
- **Acceptance (automated):** `discord_draft_stream.rs` present; wiring y tests `clamp_discord_content`; throttle documentado/clamp **200..=60_000** en `config/mod.rs` (`discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`).
- **Result:** **Pass** — criterios automáticos OK; pruebas manuales Discord opcionales no ejecutadas en esta pasada.

### Tester run (2026-03-28 UTC, TESTER.md single-task)

- **Note:** El operador nombró `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md`; ese prefijo no existía (la tarea estaba como `CLOSED-*`). Se renombró `CLOSED` → `TESTING` para el paso 2 de TESTER.md, sin abrir otro `UNTESTED-*`.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files.
- **Acceptance:** `discord_draft_stream.rs` present; `Config::discord_draft_throttle_ms()` clamps **200..=60_000** with `discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS` (`config/mod.rs` lines 463–484).
- **Result:** **Pass** — automated criteria satisfied; optional live Discord steps not run.

### Tester run (2026-03-28 UTC, TESTER.md — slug UNTESTED-20260322-0105)

- **Note:** El archivo nombrado `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` no existía; la tarea estaba como `CLOSED-*`. Paso 2 de TESTER.md: `CLOSED` → `TESTING`, verificación, informe, `TESTING` → `CLOSED`. No se tocó ningún otro `UNTESTED-*`.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files.
- **Config:** `Config::discord_draft_throttle_ms()` (`config/mod.rs` ~463–484): clamp **200..=60_000**; env `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`; key `discord_draft_throttle_ms`.
- **Result:** **Pass** — criterios automáticos de la tarea cumplidos; pasos opcionales Discord en vivo no ejecutados.

### Tester run (2026-03-28 UTC, TESTER.md — operator-named UNTESTED slug)

- **Note:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` was not present; the task file was `CLOSED-*`. Per TESTER.md step 2, renamed `CLOSED` → `TESTING` for this run only; no other `UNTESTED-*` file was used.
- **Date:** 2026-03-28 UTC.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files.
- **Config:** `discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`, clamp **200..=60_000** — present in `config/mod.rs` (`discord_draft_throttle_ms()`).
- **Result:** **Pass** — automated acceptance criteria satisfied; optional live Discord / throttle-override steps not run.

### Tester run (2026-03-28 UTC, TESTER.md — operator request, solo esta tarea)

- **Date:** 2026-03-28 UTC (fecha del entorno del operador: sábado 28 mar 2026).
- **Note:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` no existía; la tarea estaba como `CLOSED-*`. Se aplicó el flujo TESTER.md renombrando `CLOSED` → `TESTING` → (tras el informe) `CLOSED`. No se usó ningún otro `UNTESTED-*`.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files.
- **Acceptance:** `src-tauri/src/commands/discord_draft_stream.rs` presente; cableado y tests de `clamp_discord_content`; `Config::discord_draft_throttle_ms()` en `config/mod.rs` con clamp **200..=60_000** y `discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`.
- **Result:** **Pass** — criterios automáticos cumplidos; pasos opcionales Discord en vivo / override de throttle no ejecutados. Archivo final: `CLOSED-*` (no `TESTED-*`).

### Tester run (2026-03-28, TESTER.md — UNTESTED slug solicitado)

- **Date:** 2026-03-28 (sábado), hora local del workspace; informe en UTC aproximado: mismo día 2026-03-28.
- **Note:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` no existía en el árbol; la tarea estaba como `CLOSED-*`. Paso 2 de TESTER.md: `CLOSED` → `TESTING`, verificación, informe, `TESTING` → `CLOSED`. No se usó ningún otro `UNTESTED-*`.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass.
  - `cd src-tauri && cargo test discord_draft_stream::` — pass (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`).
  - `rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs` — match at line 2172.
  - `rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs` — matches in all three files (import/struct/parameter).
- **Acceptance (automated):** Criterios de la tarea cumplidos: `discord_draft_stream.rs`, cableado, tests `clamp_discord_content`; throttle en `config/mod.rs` (`discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`, clamp 200–60_000 ms).
- **Result:** **Pass** — criterios automáticos OK; pasos opcionales Discord en vivo no ejecutados. Archivo renombrado a `CLOSED-*`.
