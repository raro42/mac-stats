# Scheduler → Discord delivery awareness for CPU (main) chat

## Goal

After a **scheduler-initiated** run posts text to Discord (`reply_to_channel_id`), the **in-app CPU window chat** should see a concise system block listing recent successful deliveries (OpenClaw-style “main session awareness” after isolated cron delivery), so the model can continue without blindly re-sending the same content.

## Acceptance criteria

1. **Persistence:** Successful scheduler Discord deliveries append to `scheduler_delivery_awareness.json` under the same directory as `schedules.json` (`~/.mac-stats/`), capped and de-duplicated by `context_key`.
2. **Recording:** The task/runner path calls `delivery_awareness::record_if_new` only after Discord accepts the message when scheduler delivery context is present.
3. **CPU chat injection:** Frontend Ollama chat path prepends `delivery_awareness::format_for_chat_context()` to the system prompt when non-empty (`augment_cpu_system_with_scheduler_awareness` in `commands/ollama_frontend_chat.rs`).
4. **API:** `list_scheduler_delivery_awareness` remains available for Settings/debug (newest-first).

## Verification

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test delivery_awareness -- --nocapture
```

Optional sanity (documentation / wiring):

```bash
rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs
```

## Test report

**Preflight:** `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` was **not present** in the working tree at run start. The task body was written to that path, then renamed to `TESTING-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used.

**Date:** 2026-03-27 (local macOS environment).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" …` | **pass** — hits `ollama_frontend_chat.rs`, `task/runner.rs`, `scheduler/mod.rs`, `delivery_awareness.rs` |

**Notes:** End-to-end Discord delivery was not exercised in this run (no live bot); acceptance is satisfied by code review + unit tests + grep wiring. Manual spot-check: trigger a scheduled task with `reply_to_channel_id`, confirm `~/.mac-stats/scheduler_delivery_awareness.json` grows and CPU chat debug log shows scheduler awareness prepended when block non-empty.

**Outcome:** **CLOSED** — all listed acceptance criteria and automated verification passed.

## Test report (re-run)

**Preflight:** `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` was **not** in the tree; the task file was already `CLOSED-*`. Per `003-tester/TESTER.md`, it was renamed `CLOSED-…` → `TESTING-…` for this run, verification executed, then renamed back to `CLOSED-…` on success. No other `UNTESTED-*` file was used.

**Date:** 2026-03-27 (local, America-friendly note: same calendar day as prior report).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src` | **pass** — `ollama_frontend_chat.rs`, `delivery_awareness.rs`, `scheduler/mod.rs`, `task/runner.rs` |

**Notes:** Live Discord / E2E not re-run; automated criteria unchanged.

**Outcome:** **CLOSED** — all verification steps passed.

## Test report (2026-03-27, Cursor tester run)

**Preflight:** El operador pidió `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md`; **no existía** en el árbol (el archivo estaba como `CLOSED-*`). Siguiendo `003-tester/TESTER.md` para esta tarea concreta: `CLOSED-…` → `TESTING-…`, verificación, informe, y tras éxito `TESTING-…` → `CLOSED-…`. No se tocó ningún otro `UNTESTED-*`.

**Date:** 2026-03-27, hora local (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación pasaron.

## Test report (2026-03-27, TESTER.md run)

**Preflight:** El operador indicó `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md`; **no existía** en el árbol (solo esta tarea, ya como `CLOSED-*`). Se renombró `CLOSED-…` → `TESTING-…`, se ejecutó la verificación del cuerpo de la tarea y se vuelve a `CLOSED-…` tras éxito. No se usó ningún otro `UNTESTED-*`.

**Date:** 2026-03-27, hora local del host (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src` | **pass** — `ollama_frontend_chat.rs`, `delivery_awareness.rs`, `scheduler/mod.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación pasaron.

## Test report (2026-03-27, TESTER.md — ejecución agente)

**Preflight:** Ruta pedida `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` **ausente**; el fichero estaba como `CLOSED-*` y se renombró a `TESTING-*` para esta pasada. Solo esta tarea; ningún otro `UNTESTED-*`.

**Date:** 2026-03-27, hora local (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado.

**Outcome:** **CLOSED** — verificación del cuerpo de la tarea completada con éxito.

## Test report (2026-03-27, TESTER.md — Cursor)

**Preflight:** La ruta indicada `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` **no existía**; el fichero estaba como `CLOSED-*`. Se aplicó `CLOSED-…` → `TESTING-…`, verificación según el cuerpo de la tarea, informe y `TESTING-…` → `CLOSED-…` al pasar todo. Solo esta tarea; ningún otro `UNTESTED-*`.

**Date:** 2026-03-27, hora local del host (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado (igual que en informes previos).

**Outcome:** **CLOSED** — criterios de verificación automatizados del cuerpo de la tarea cumplidos.

## Test report (2026-03-28, TESTER.md — Cursor)

**Preflight:** El operador pidió `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md`; **no existía** en el árbol (la tarea ya estaba como `CLOSED-*`). Para cumplir `003-tester/TESTER.md` sobre **esta** tarea sin usar otro `UNTESTED-*`: se renombró `CLOSED-…` → `TESTING-…`, se ejecutó la verificación del cuerpo de la tarea, se añade este informe y se vuelve a `CLOSED-…` si todo pasa.

**Date:** 2026-03-28, hora local del host (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación del cuerpo de la tarea pasaron.

## Test report (2026-03-28, TESTER.md — Cursor agent)

**Preflight:** El operador indicó `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md`; **no existía** en el árbol (la tarea ya estaba como `CLOSED-*`). Para aplicar `003-tester/TESTER.md` solo a esta tarea: `CLOSED-…` → `TESTING-…`, verificación, informe y `TESTING-…` → `CLOSED-…` si todo pasa. No se usó ningún otro `UNTESTED-*`.

**Date:** 2026-03-28, hora local del host (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación del cuerpo de la tarea pasaron.

## Test report

**Preflight:** La ruta pedida `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` **no existía**; el fichero estaba como `CLOSED-*` al inicio de esta pasada. Se renombró `CLOSED-…` → `TESTING-…`, se ejecutó la verificación del cuerpo de la tarea, se añade este informe y se volverá a `CLOSED-…` si todo pasa. No se usó ningún otro `UNTESTED-*`.

**Date:** 2026-03-28 00:14 UTC.

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación del cuerpo de la tarea pasaron.

## Test report (2026-03-28, TESTER.md — Cursor)

**Preflight:** `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` **no existía** en el árbol; la tarea ya estaba como `CLOSED-*`. Se renombró `CLOSED-…` → `TESTING-…`, se ejecutó la verificación del cuerpo de la tarea, se añade este informe y se vuelve a `CLOSED-…` al pasar todo. No se usó ningún otro `UNTESTED-*`.

**Date:** 2026-03-28, hora local del host (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación del cuerpo de la tarea pasaron.

## Test report (2026-03-28, TESTER.md — ejecución solicitada)

**Preflight:** El operador pidió `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md`; **no existía** (la tarea ya estaba `CLOSED-*`). Se aplicó `CLOSED-…` → `TESTING-…`, verificación del cuerpo de la tarea, este informe y `TESTING-…` → `CLOSED-…` al pasar todo. No se usó ningún otro `UNTESTED-*`.

**Date:** 2026-03-28, hora local del host (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación del cuerpo de la tarea pasaron.

## Test report (2026-03-28, TESTER.md — run del operador)

**Preflight:** `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` **no existía**; el fichero estaba como `CLOSED-*` al inicio. Para seguir `003-tester/TESTER.md` sobre **solo** esta tarea: `CLOSED-…` → `TESTING-…`, verificación, este informe y `TESTING-…` → `CLOSED-…` al pasar. No se usó ningún otro `UNTESTED-*`.

**Date:** 2026-03-28, hora local del host (macOS).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs` | **pass** — `ollama_frontend_chat.rs`, `scheduler/mod.rs`, `scheduler/delivery_awareness.rs`, `task/runner.rs` |

**Notes:** Discord / E2E no ejecutado; criterios automatizados y cableado verificados.

**Outcome:** **CLOSED** — todos los pasos de verificación del cuerpo de la tarea pasaron.
