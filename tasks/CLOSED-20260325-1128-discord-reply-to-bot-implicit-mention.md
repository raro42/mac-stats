# Discord: reply to bot counts as mention in MentionOnly

## Goal

In **MentionOnly** channels, a human message that **replies** to a message authored by the bot (Discord message reference) should activate the router as if the user had @mentioned the bot, without requiring a literal `<@bot>` mention.

## Acceptance criteria

1. **`discord_mentions_bot_effective`** returns true when the incoming message has a message reference to a message whose author is the bot (using `referenced_message` when the gateway provides it, else cache, else `get_message` fallback).
2. **Gateway `message` handler:** For non-DM, `ChannelMode::MentionOnly`, the early return that ignores non-mentions uses `mentions_bot_effective` (not only literal `mentions`), so reply-to-bot passes the gate.
3. **Observability:** Debug logs distinguish activation via reference vs literal mention (existing `MentionOnly activation via message reference` / `could not resolve referenced message for implicit mention` strings).
4. **Build:** `cargo check` in `src-tauri/` succeeds.

## Verification

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test outbound_attachment_path_allowlist -- --nocapture
```

Wiring / presence:

```bash
rg -n "discord_mentions_bot_effective|mentions_bot_effective" src-tauri/src/discord/mod.rs
```

Optional: manual Discord — MentionOnly channel, reply to the bot’s last message without @mention; expect the bot to process the message (live token required; not required for this automated pass).

## Test report

**Preflight:** `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md` **no estaba** en el working tree al inicio del run; se escribió el cuerpo de la tarea en esa ruta y se renombró a `TESTING-20260325-1128-discord-reply-to-bot-implicit-mention.md` según `003-tester/TESTER.md`. No se usó ningún otro archivo `UNTESTED-*`.

**Date:** 2026-03-27 (local, macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit test (discord module) | `cd src-tauri && cargo test outbound_attachment_path_allowlist -- --nocapture` | **pass** (1 test) |
| Wiring | `rg -n discord_mentions_bot_effective src-tauri/src/discord/mod.rs` (y `mentions_bot_effective` en el mismo archivo) | **pass** — definición ~1852; gateway ~2787–2788; filtro MentionOnly ~2814; router ~1956 |

**Code review:** `discord_mentions_bot_effective` comprueba mención literal, luego `message_reference` + `referenced_message` o caché o `get_message`; logs de debug para activación por referencia y fallo de resolución (líneas ~1865–1917 en `discord/mod.rs`).

**Notes:** Prueba manual en Discord con bot real **no** ejecutada en esta corrida.

**Outcome:** **CLOSED** — criterios de aceptación cubiertos por revisión de código + `cargo check` + grep; verificación automatizada del task pasó.

### Test report — 2026-03-27 (re-run, local macOS)

**Preflight:** El operador pidió `UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md`; en el árbol solo existía `CLOSED-20260325-1128-discord-reply-to-bot-implicit-mention.md`. Se renombró ese archivo a `TESTING-…` para esta corrida (misma tarea, sin elegir otro `UNTESTED-*`).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit test | `cd src-tauri && cargo test outbound_attachment_path_allowlist -- --nocapture` | **pass** (1 test) |
| Wiring | `rg -n 'discord_mentions_bot_effective|mentions_bot_effective' src-tauri/src/discord/mod.rs` | **pass** (líneas 1852, 1956, 2016, 2787–2788, 2814) |

**Manual Discord:** no ejecutada (opcional).

**Outcome:** **CLOSED** — todos los criterios automatizados del task pasan.

### Test report — 2026-03-27 (corrida TESTER, hora local macOS)

**Preflight:** El operador indicó `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md`; ese nombre no existía (solo `CLOSED-…`). Se aplicó `003-tester/TESTER.md` sobre la misma tarea: `CLOSED-…` → `TESTING-…` para esta corrida; no se eligió otro `UNTESTED-*`.

| Paso | Comando | Resultado |
|------|---------|-----------|
| Compilación | `cd src-tauri && cargo check` | **pass** |
| Test unitario | `cd src-tauri && cargo test outbound_attachment_path_allowlist -- --nocapture` | **pass** (1 test en `mac_stats` lib) |
| Cableado | `rg -n "discord_mentions_bot_effective\|mentions_bot_effective" src-tauri/src/discord/mod.rs` | **pass** — líneas 1852, 1956, 2016, 2787–2788, 2814 |

**Manual Discord:** no ejecutada (opcional; requiere token).

**Outcome:** **CLOSED** — criterios 1–4 del task cumplen vía código + `cargo check` + test + grep; sin bloqueos.

### Test report — 2026-03-27 (corrida TESTER; hora local macOS)

**Preflight:** El operador pidió `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md`; ese path **no existía** (solo `CLOSED-…`). Se aplicó `003-tester/TESTER.md` sobre la misma tarea: `CLOSED-…` → `TESTING-…` para esta corrida; **no** se eligió otro archivo `UNTESTED-*`.

| Paso | Comando | Resultado |
|------|---------|-----------|
| Compilación | `cd src-tauri && cargo check` | **pass** |
| Test unitario | `cd src-tauri && cargo test outbound_attachment_path_allowlist -- --nocapture` | **pass** (1 test, `discord::tests::outbound_attachment_path_allowlist`) |
| Cableado | `rg -n "discord_mentions_bot_effective\|mentions_bot_effective" src-tauri/src/discord/mod.rs` | **pass** — líneas 1852, 1956, 2016, 2787–2788, 2814 |

**Manual Discord:** no ejecutada (opcional).

**Outcome:** **CLOSED** — criterios de aceptación del task cumplen; archivo renombrado de vuelta a `CLOSED-…` tras el informe.

