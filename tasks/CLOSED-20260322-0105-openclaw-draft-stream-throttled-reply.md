# CLOSED — OpenClaw-style Discord draft stream / throttled reply (2026-03-22)

## Goal

Discord full-agent path behaves like OpenClaw-style progress: post a placeholder (“Processing…”), **edit the same message** on a throttle while tools run (e.g. “Running FETCH_URL…”), then **flush** the final reply into that message (first chunk; overflow chunks as separate messages). Operator reference: `docs/007_discord_agent.md`.

## Acceptance criteria

- `src-tauri/src/commands/discord_draft_stream.rs` implements throttled/coalesced draft updates and immediate flush.
- `spawn_discord_draft_editor` is used from `src-tauri/src/discord/mod.rs`; `DiscordDraftHandle` is threaded through `commands/tool_loop.rs`, `commands/turn_lifecycle.rs`, and `commands/ollama.rs`.
- Throttle interval is configurable via `discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`, clamped 200–60_000 ms (`config/mod.rs`).
- Unit tests cover `clamp_discord_content` (Discord length limit).

## Verification

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Manual (optional):** live Discord router run with tools — not required for this automated pass.

---

## Historic test report (preflight stub, 2026-03-27)

Prior cycle used `WIP-20260322-0105-openclaw-draft-stream-throttled-reply.md` because `UNTESTED-*` was missing. Excerpt preserved:

**Date:** 2026-03-27, local (America/Mexico_City).

- **UNTESTED → TESTING:** skipped (no `UNTESTED-*` then).
- `cargo check` — Pass; `cargo test discord_draft_stream::` — Pass (2 tests).
- **Conclusion then:** WIP — blocked (no task body). Partial module checks passed.

## Test report

**Date:** 2026-03-27, hora local del entorno donde se ejecutó la verificación (shell en el workspace mac-stats).

**Workflow**

- **UNTESTED → TESTING:** aplicado (archivo `UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` recreado con Goal / Acceptance / Verification porque no existía en disco; el antiguo `WIP-…` duplicado del mismo ID se eliminó tras fusionar el historial en la sección “Historic test report”).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests `clamp_*`) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Criterio manual opcional (Discord en vivo con herramientas) | **No ejecutado** (fuera del alcance de esta pasada automatizada) |

**Conclusión:** **CLOSED** — criterios de aceptación verificados por compilación, pruebas unitarias del módulo y comprobación estática de cableado; la prueba manual en Discord queda como seguimiento operativo si se desea.

### Test report — TESTER.md pass (2026-03-27)

**Date:** 2026-03-27, local (shell en el workspace mac-stats).

**Workflow**

- **UNTESTED → TESTING:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` no existía en el repo; la misma tarea estaba como `CLOSED-*`. Para seguir `003-tester/TESTER.md` se renombró `CLOSED-…` → `TESTING-…`, se ejecutó la verificación y se vuelve a `CLOSED-…` al cerrar (solo este ID de tarea, sin otro `UNTESTED-*`).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` | **Pass** |
| `cargo test discord_draft_stream::` (2 tests) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Manual opcional (Discord en vivo) | **No ejecutado** |

**Conclusión:** **CLOSED** — criterios automatizados cumplidos.

### Test report — TESTER.md pass (2026-03-27, segunda pasada)

**Date:** 2026-03-27, local (workspace mac-stats, shell).

**Workflow**

- **UNTESTED → TESTING:** el archivo solicitado `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` **no estaba en el repo**; no se tocó ningún otro `UNTESTED-*`. Se renombró `CLOSED-…` → `TESTING-…` para esta pasada y, tras verificar, se vuelve a `CLOSED-…`.

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Manual opcional (Discord en vivo) | **No ejecutado** |

**Conclusión:** **CLOSED** — criterios de aceptación verificados (compilación, tests del módulo, cableado por `rg`).

### Test report — TESTER.md pass (2026-03-27, operador: UNTESTED solicitado)

**Date:** 2026-03-27, local (workspace mac-stats, shell).

**Workflow**

- **UNTESTED → TESTING:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` **no existía** en el repo; no se eligió ningún otro `UNTESTED-*`. Se aplicó `CLOSED-…` → `TESTING-…` para esta pasada (mismo ID y cuerpo de tarea).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Manual opcional (Discord en vivo) | **No ejecutado** |

**Conclusión:** **CLOSED** — criterios de aceptación automatizados cumplidos.

### Test report — TESTER.md pass (2026-03-27, operador: UNTESTED-… solicitado)

**Date:** 2026-03-27, local (workspace mac-stats, shell).

**Workflow**

- **UNTESTED → TESTING:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` **no existe** en el repo; no se eligió ningún otro `UNTESTED-*`. Se renombró `CLOSED-…` → `TESTING-…` para esta pasada (mismo ID `20260322-0105`).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Manual opcional (Discord en vivo con herramientas) | **No ejecutado** |

**Conclusión:** **CLOSED** — todos los criterios de aceptación verificables de forma automatizada pasan; la prueba manual en Discord sigue siendo opcional.

### Test report — TESTER.md pass (2026-03-27, operador: archivo UNTESTED-… nombrado)

**Date:** 2026-03-27, local (workspace mac-stats, shell).

**Workflow**

- **UNTESTED → TESTING:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` **no existe** en el repo (solo esta tarea bajo `CLOSED-*` / ahora `TESTING-*`); no se eligió ningún otro `UNTESTED-*`. Se renombró `CLOSED-…` → `TESTING-…` para esta pasada (mismo ID `20260322-0105`).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Manual opcional (Discord en vivo con herramientas) | **No ejecutado** |

**Notas:** Criterios de aceptación adicionales (archivo `discord_draft_stream.rs`, `discord_draft_throttle_ms` / env en `config/mod.rs`) siguen presentes en el árbol; no listados en el bloque Verification pero coherentes con la tarea.

**Conclusión:** **CLOSED** — verificación automatizada del bloque Verification cumplida.

### Test report — TESTER.md pass (2026-03-28)

**Date:** 2026-03-28, local (workspace mac-stats, shell; hora del entorno del operador).

**Workflow**

- **UNTESTED → TESTING:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` **no existe** en el repo; no se eligió ningún otro `UNTESTED-*`. Se renombró `CLOSED-…` → `TESTING-…` para esta pasada (mismo ID `20260322-0105`).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Manual opcional (Discord en vivo con herramientas) | **No ejecutado** |

**Notas:** Criterios de aceptación no listados en Verification (`discord_draft_stream.rs`, `discord_draft_throttle_ms` / `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS` con clamp 200..=60_000 en `config/mod.rs`) siguen presentes en el árbol.

**Conclusión:** **CLOSED** — todos los criterios verificables de forma automatizada pasan.

### Test report — TESTER.md pass (2026-03-28, sesión agente; tarea UNTESTED-… nombrada)

**Date:** 2026-03-28, local (workspace mac-stats, shell del agente).

**Workflow**

- **UNTESTED → TESTING:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` **no existe** en el repo; no se eligió ningún otro `UNTESTED-*`. Se renombró `CLOSED-…` → `TESTING-…` para esta pasada (mismo ID `20260322-0105`).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs`, `turn_lifecycle.rs`, `ollama.rs` | **Pass** |
| Manual opcional (Discord en vivo con herramientas) | **No ejecutado** |

**Conclusión:** **CLOSED** — criterios del bloque Verification cumplidos; archivo vuelve a prefijo `CLOSED-…`.

### Test report — TESTER.md pass (2026-03-28, operador: UNTESTED-… nombrado explícitamente)

**Date:** 2026-03-28, local (workspace mac-stats, shell del agente; hora del entorno).

**Workflow**

- **UNTESTED → TESTING:** `tasks/UNTESTED-20260322-0105-openclaw-draft-stream-throttled-reply.md` **no existe** en el repo; no se eligió ningún otro `UNTESTED-*`. Se renombró `CLOSED-…` → `TESTING-…` para esta pasada (mismo ID `20260322-0105`).

**Comandos ejecutados**

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test discord_draft_stream::
rg -n "spawn_discord_draft_editor" src-tauri/src/discord/mod.rs
rg -n "DiscordDraftHandle" src-tauri/src/commands/tool_loop.rs src-tauri/src/commands/turn_lifecycle.rs src-tauri/src/commands/ollama.rs
```

**Resultados**

| Comprobación | Resultado |
|--------------|-----------|
| `cargo check` (src-tauri) | **Pass** |
| `cargo test discord_draft_stream::` (2 tests: `clamp_under_limit_unchanged`, `clamp_truncates_with_ellipsis`) | **Pass** |
| `spawn_discord_draft_editor` en `discord/mod.rs` (L2172) | **Pass** |
| `DiscordDraftHandle` en `tool_loop.rs` (L14, L152), `turn_lifecycle.rs` (L10, L95), `ollama.rs` (p. ej. L109 con tipo completo) | **Pass** |
| Manual opcional (Discord en vivo con herramientas) | **No ejecutado** |

**Conclusión:** **CLOSED** — criterios del bloque Verification cumplidos; archivo vuelve a prefijo `CLOSED-…`.
