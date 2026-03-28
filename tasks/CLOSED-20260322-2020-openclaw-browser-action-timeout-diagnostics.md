---
## Triage summary (TOP)

- **Coder (UTC):** 2026-03-28 — Implementación ya presente en `browser_agent/mod.rs`, `commands/browser_tool_dispatch.rs`, `commands/browser_helpers.rs`, `browser_doctor.rs`: mensajes de timeout CDP con pista de proxy, `context:` compacto con `navchg=0|1`, omisión de HTTP fallback cuando `is_cdp_navigation_timeout_error`, y `mac_stats --browser-doctor` para sondas CDP. Verificación local: `cargo check` y `cargo test` en `src-tauri/`. *(En el árbol no existe `002-coder-backend/CODER.md`; backlog de features: `006-feature-coder/FEATURE-CODER.md`.)*
- **Next step:** Ninguno; última verificación tester: 2026-03-28 (automated §3 + rg §4).
---

# OpenClaw-style browser action timeout diagnostics

**Created (UTC):** 2026-03-22 20:20  
**Coder handoff (UTC):** 2026-03-28  
**Spec:** [docs/029_browser_automation.md](docs/029_browser_automation.md) (navigation timeout, `navchg`, proxy hint, `--browser-doctor`)

---

## 1. Goal

When **BROWSER_*** CDP work hits **navigation / action timeouts**, mac-stats surfaces **operator-actionable diagnostics**: clear timeout text, compact **`context:`** lines (including **`navchg=0|1`** when relevant), **dispatcher** behaviour that does not mask CDP timeouts with HTTP fallback, and **`--browser-doctor`** for CDP readiness — aligned with `docs/029_browser_automation.md` (OpenClaw-style visibility).

---

## 2. References

- `src-tauri/src/browser_doctor.rs` — `run_browser_doctor_stdio`, effective CDP timeouts / probe
- `src-tauri/src/commands/browser_helpers.rs` — `is_cdp_navigation_timeout_error`, unit test `cdp_navigation_timeout_detection_matches_tool_errors`
- `src-tauri/src/commands/browser_tool_dispatch.rs` — `nav_url_changed_hint_if_navigation_timeout`, `format_last_browser_error_context`, skip HTTP fallback on CDP nav timeout
- `src-tauri/src/browser_agent/mod.rs` — `navigation_timeout_error_with_proxy_hint`, `record_nav_timeout_url_changed_hint`, `format_last_browser_error_context`, `format_context_suffix_from_health`
- `docs/029_browser_automation.md` — navigation timeout, `navchg`, proxy hint, `mac_stats --browser-doctor`

---

## 3. Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds (including `browser_helpers` timeout detection test).
3. **Static verification:** Timeout diagnostics paths still present (`rg` spot-check in §4).

---

## 4. Testing instructions

Run from the **repository root** (or adjust paths).

### Automated (required)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Optional spot-check (symbols must match in the listed files):

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

Targeted unit test (optional, faster than full suite):

```bash
cd src-tauri && cargo test cdp_navigation_timeout_detection_matches_tool_errors --lib
```

### Manual / smoke (optional)

1. **CDP readiness:** With Chrome listening on the configured debug port and `browserToolsEnabled` true (see `docs/029_browser_automation.md`), run:
   ```bash
   ./src-tauri/target/release/mac_stats --browser-doctor -vv
   ```
   Confirm stdout describes CDP connectivity / timeouts (no silent failure).

2. **Navigation timeout path:** Trigger a **BROWSER_NAVIGATE** (or equivalent) to a URL that stalls beyond the navigation deadline (e.g. very slow host or blocked resource). Expect:
   - User/model-visible error mentioning **navigation timeout** (and proxy hint text when applicable).
   - A compact **`context:`** suffix including **`navchg=0`** or **`navchg=1`** when the dispatcher records URL-change hint for that timeout.
   - In `~/.mac-stats/debug.log`, an **INFO** `browser/tools` line stating that **HTTP fallback was skipped** on CDP navigation timeout (so the failure is not masked by fetch success).

3. **Contrast (non-timeout CDP failure):** After a non-timeout CDP error on navigate, behaviour may still attempt retry / HTTP fallback per existing logic — only **`is_cdp_navigation_timeout_error`** errors skip masking fallback.

---

## 5. Implementation summary

- `navigation_timeout_error_with_proxy_hint` builds stable timeout strings; `is_cdp_navigation_timeout_error` matches the `"Navigation failed: timeout after"` prefix so dispatch and tests stay aligned.
- `record_nav_timeout_url_changed_hint` + `format_last_browser_error_context` attach `navchg=` for operator triage.
- `BROWSER_NAVIGATE` in `browser_tool_dispatch.rs` logs and returns early on CDP nav timeout without HTTP fallback on first failure, after CDP retry failure, and preserves context lines on combined CDP+HTTP failure paths.

## Test report

- **Date:** 2026-03-28 (local, tester run)
- **Outcome:** Pass (automated acceptance criteria §3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Optional static spot-check (task §4):

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

### Results

- `cargo check`: succeeded (exit 0).
- `cargo test`: succeeded — `871` tests passed, `0` failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- `rg` spot-check: all listed symbols present in the expected files.

### Notes

- Manual / smoke steps in §4.3 were **not** executed (optional per task); automated criteria §3.1–§3.3 are satisfied.

---

## Test report (follow-up run)

- **Date:** 2026-03-28 (local, tester run; workspace: mac-stats)
- **Preflight:** El nombre pedido `UNTESTED-20260322-2020-…` no existía en el árbol; la tarea estaba en `CLOSED-…`. Se aplicó el ciclo TESTER renombrando `CLOSED-` → `TESTING-` para esta ejecución.
- **Outcome:** Pass (criterios automatizados §3.1–§3.3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Spot-check estático (§4):

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

### Results

- `cargo check`: exit 0.
- `cargo test`: exit 0 — `871` passed, `0` failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- `rg` spot-check: símbolos presentes en los cuatro archivos listados.

### Notes

- Pasos manuales §4.3 **no** ejecutados (opcionales). Resultado final del archivo: `CLOSED-` (todos los criterios de aceptación automatizados cumplidos).

---

## Test report

- **Date:** 2026-03-28 (local, America/Los_Angeles; tester run)
- **Preflight:** `tasks/UNTESTED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md` was **not present** in the workspace. The same task body lives at `tasks/CLOSED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md`. Per operator instruction, **no other** `UNTESTED-*` file was used. TESTER step “UNTESTED → TESTING” was **skipped** (missing source name); verification was run against this file’s §3–§4 only.
- **Outcome:** Pass (automated acceptance criteria §3.1–§3.3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Static spot-check (task §4):

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

### Results

- `cargo check`: exit 0.
- `cargo test`: exit 0 — `871` passed, `0` failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- `rg` spot-check: symbols present in the four listed files.

### Notes

- Manual / smoke steps in §4.3 **not** run (optional). Filename remains **`CLOSED-…`** (pass).

---

## Test report

- **Date:** 2026-03-28 (local, America/Los_Angeles; tester run)
- **Preflight:** `tasks/UNTESTED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md` **no existe** en el workspace; la tarea está en `tasks/CLOSED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md`. No se usó ningún otro `UNTESTED-*`. El paso TESTER «UNTESTED → TESTING» **no aplicó** (falta el nombre origen). Verificación según §3–§4 de este archivo.
- **Outcome:** Pass (criterios §3.1–§3.3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Spot-check estático (§4), desde la raíz del repo:

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

### Results

- `cargo check`: exit 0.
- `cargo test`: exit 0 — `871` passed, `0` failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- `rg` (vía búsqueda en workspace): símbolos presentes en los cuatro archivos listados.

### Notes

- Pasos manuales §4.3 **no** ejecutados (opcionales). El nombre del archivo permanece **`CLOSED-…`** (pass).

---

## Test report

- **Date:** 2026-03-28 (local, tester run)
- **Preflight:** `tasks/UNTESTED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md` **no estaba** en el workspace; la misma tarea estaba como `CLOSED-…`. Se renombró **`CLOSED-` → `TESTING-`** para esta ejecución (equivalente operativo al paso UNTESTED→TESTING). No se tocó ningún otro `UNTESTED-*`.
- **Outcome:** Pass (criterios automatizados §3.1–§3.3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Spot-check estático (§4), desde la raíz del repo:

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

(Verificación de símbolos también vía búsqueda en workspace en los cuatro archivos.)

### Results

- `cargo check`: exit 0.
- `cargo test`: exit 0 — `871` passed, `0` failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- Spot-check: símbolos presentes en `browser_agent/mod.rs`, `browser_tool_dispatch.rs`, `browser_helpers.rs`, `browser_doctor.rs`.

### Notes

- Pasos manuales §4.3 **no** ejecutados (opcionales). Resultado del archivo tras esta ronda: **`CLOSED-…`** (pass).

---

## Test report

- **Date:** 2026-03-28 (local, America/Los_Angeles; tester run)
- **Preflight:** `tasks/UNTESTED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md` **no existía**; la tarea estaba como `CLOSED-…`. Para seguir TESTER.md se renombró **`CLOSED-` → `TESTING-`** en esta ejecución (equivalente operativo a UNTESTED→TESTING). No se usó ningún otro `UNTESTED-*`.
- **Outcome:** Pass (criterios §3.1–§3.3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Spot-check estático (§4), desde la raíz del repo:

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

### Results

- `cargo check`: exit 0.
- `cargo test`: exit 0 — crate `mac_stats` (lib): **871** passed, **0** failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- Spot-check (`rg` / búsqueda en workspace): símbolos presentes en los cuatro archivos listados.

### Notes

- Pasos manuales §4.3 **no** ejecutados (opcionales). Archivo renombrado a **`CLOSED-…`** tras el informe (pass).

---

## Test report

- **Date:** 2026-03-28 (local, America/Los_Angeles; tester run)
- **Preflight:** `tasks/UNTESTED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md` **no existe** en el workspace; la tarea estaba como `CLOSED-…`. Se renombró **`CLOSED-` → `TESTING-`** para esta ejecución (equivalente al paso UNTESTED→TESTING de TESTER.md). No se usó ningún otro `UNTESTED-*`.
- **Outcome:** Pass (criterios de aceptación §3.1–§3.3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Spot-check estático (§4), desde la raíz del repo:

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

### Results

- `cargo check`: exit 0.
- `cargo test`: exit 0 — `871` passed, `0` failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- `rg` spot-check: símbolos presentes en los cuatro archivos listados.

### Notes

- Pasos manuales / smoke §4.3 **no** ejecutados (opcionales). Archivo renombrado a **`CLOSED-…`** tras este informe (pass).

---

## Test report

- **Date:** 2026-03-28 (local, America/Los_Angeles; tester run)
- **Preflight:** `tasks/UNTESTED-20260322-2020-openclaw-browser-action-timeout-diagnostics.md` **no existía**; la tarea estaba como `CLOSED-…`. Para aplicar TESTER.md se renombró **`CLOSED-` → `TESTING-`** antes de la verificación (equivalente operativo a UNTESTED→TESTING). No se usó ningún otro `UNTESTED-*`.
- **Outcome:** Pass (criterios de aceptación §3.1–§3.3)

### Commands run

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Spot-check estático (§4), desde la raíz del repo:

```bash
rg -n "format_last_browser_error_context|navchg=|navigation_timeout_error_with_proxy_hint|is_cdp_navigation_timeout_error|run_browser_doctor_stdio" \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/commands/browser_tool_dispatch.rs \
  src-tauri/src/commands/browser_helpers.rs \
  src-tauri/src/browser_doctor.rs
```

### Results

- `cargo check`: exit 0.
- `cargo test`: exit 0 — `871` passed, `0` failed; `commands::browser_helpers::tests::cdp_navigation_timeout_detection_matches_tool_errors` **ok**.
- `rg` spot-check: símbolos presentes en los cuatro archivos listados.

### Notes

- Pasos manuales / smoke §4.3 **no** ejecutados (opcionales). Archivo renombrado **`TESTING-` → `CLOSED-`** tras este informe (pass).
