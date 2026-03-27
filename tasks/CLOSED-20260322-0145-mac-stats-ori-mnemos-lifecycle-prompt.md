# mac-stats: Ori Mnemos lifecycle vs execution prompt

## Goal

Confirm that Ori lifecycle sections in the execution system prompt follow the order documented in `docs/ori-lifecycle.md` (markdown memory, then Ori briefing, then prefetch notes, then live metrics).

## Acceptance criteria

1. `cargo check` succeeds in `src-tauri/`.
2. `cargo test prompts::` succeeds (includes `ori_briefing_and_prefetch_follow_memory_before_metrics`).
3. Prompt assembly keeps memory block before `## Ori session briefing`, briefing before `## Possibly relevant vault notes`, and both before metrics (covered by the unit test above).

## Verification

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test prompts:: --no-fail-fast
```

## Test report

- **Date:** 2026-03-27, hora local del entorno donde se ejecutaron los comandos (no UTC fijada).
- **Note:** En el árbol de trabajo **no existía** `tasks/UNTESTED-20260322-0145-mac-stats-ori-mnemos-lifecycle-prompt.md`; se creó el cuerpo de la tarea y se aplicó **UNTESTED → TESTING** según `003-tester/TESTER.md`, sin usar otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Prompt tests | `cd src-tauri && cargo test prompts:: --no-fail-fast` | **pass** — 5 passed (incl. `ori_briefing_and_prefetch_follow_memory_before_metrics`), 0 failed |

- **Outcome:** Criterios de aceptación cumplidos → **CLOSED**.

## Test report

- **Date:** 2026-03-27, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador solicitó `tasks/UNTESTED-20260322-0145-mac-stats-ori-mnemos-lifecycle-prompt.md`; en el árbol **no existía** ese prefijo (la tarea estaba como `CLOSED-*`). Se aplicó el flujo de `003-tester/TESTER.md` renombrando `CLOSED-*` → `TESTING-*` para la corrida, sin elegir otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Prompt tests | `cd src-tauri && cargo test prompts:: --no-fail-fast` | **pass** — 5 passed (incl. `ori_briefing_and_prefetch_follow_memory_before_metrics`), 0 failed |

- **Outcome:** Criterios de aceptación cumplidos → **CLOSED**.

## Test report

- **Date:** 2026-03-27, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador nombró `tasks/UNTESTED-20260322-0145-mac-stats-ori-mnemos-lifecycle-prompt.md`; ese archivo **no existe** en el árbol. Se siguió `003-tester/TESTER.md` sobre la misma tarea renombrando `CLOSED-*` → `TESTING-*` para esta corrida, sin abrir otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Prompt tests | `cd src-tauri && cargo test prompts:: --no-fail-fast` | **pass** — 5 passed (incl. `ori_briefing_and_prefetch_follow_memory_before_metrics`), 0 failed |

- **Outcome:** Criterios de aceptación cumplidos → **CLOSED**.

## Test report

- **Date:** 2026-03-27, hora local del entorno de ejecución (macOS; no UTC fijada).
- **Note:** El operador pidió `tasks/UNTESTED-20260322-0145-mac-stats-ori-mnemos-lifecycle-prompt.md`; en el árbol **no existía** con prefijo `UNTESTED-`. Se aplicó `003-tester/TESTER.md` sobre la misma tarea: `CLOSED-*` → `TESTING-*` durante la corrida, verificación, informe y `TESTING-*` → `CLOSED-*` al cerrar. No se usó otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Prompt tests | `cd src-tauri && cargo test prompts:: --no-fail-fast` | **pass** — 5 passed (incl. `ori_briefing_and_prefetch_follow_memory_before_metrics`), 0 failed |

- **Outcome:** Criterios de aceptación cumplidos → **CLOSED**.

## Test report

- **Date:** 2026-03-27, hora local del entorno de ejecución (macOS; no UTC fijada).
- **Note:** El operador nombró explícitamente `tasks/UNTESTED-20260322-0145-mac-stats-ori-mnemos-lifecycle-prompt.md`; ese archivo **no existía** (la tarea ya estaba `CLOSED-*`). Se ejecutó el flujo de `003-tester/TESTER.md` solo sobre esta tarea: `CLOSED-*` → `TESTING-*`, comandos de verificación, este informe, luego `TESTING-*` → `CLOSED-*`. No se eligió otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Prompt tests | `cd src-tauri && cargo test prompts:: --no-fail-fast` | **pass** — 5 passed (incl. `ori_briefing_and_prefetch_follow_memory_before_metrics`), 0 failed |

- **Outcome:** Criterios de aceptación cumplidos → **CLOSED**.

## Test report

- **Date:** 2026-03-27, hora local del entorno de ejecución (macOS; no UTC fijada).
- **Note:** Nueva corrida del flujo `003-tester/TESTER.md`: el archivo nombrado como `UNTESTED-*` no estaba en el árbol; se usó la misma tarea (`CLOSED-*` → `TESTING-*` → verificación → informe → `CLOSED-*`). No se abrió ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Prompt tests | `cd src-tauri && cargo test prompts:: --no-fail-fast` | **pass** — 5 passed (incl. `ori_briefing_and_prefetch_follow_memory_before_metrics`), 0 failed |

- **Outcome:** Criterios de aceptación cumplidos → **CLOSED**.
