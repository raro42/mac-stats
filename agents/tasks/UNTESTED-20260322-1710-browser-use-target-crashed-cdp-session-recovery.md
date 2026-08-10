---
## Triage summary (TOP)

- **Coder (UTC):** 2026-03-28 — La implementación ya está en `cdp_target_crash_listener.rs`, `browser_agent/mod.rs` y `main.rs` (listener lateral CDP, `Target.setDiscoverTargets` sin `filter: null`, `notify_target_renderer_crashed_side`, limpieza de sesión, bandera one-shot para herramientas, CLI `--browser-debug-crash-tab` tras `browserToolsEnabled`). Verificación local: `cargo check` y `cargo test --lib` OK. *(En el árbol no existe `002-coder-backend/CODER.md`; el flujo de backlog de features está en `agents/006-feature-coder/FEATURE-CODER.md`.)*
- **Next step:** Tester ejecuta **§4 Testing instructions**. Informes de verificación anteriores (muy repetidos) siguen en `agents/tasks/CLOSED-20260322-1710-browser-use-target-crashed-cdp-session-recovery.md` solo como histórico.
---

# Browser use: Target.targetCrashed → recuperación de sesión CDP

**Created (UTC):** 2026-03-22 17:10  
**Coder handoff (UTC):** 2026-03-28  
**Spec:** [docs/029_browser_automation.md](docs/029_browser_automation.md) (Renderer crash)

---

## 1. Goal

Cuando el renderer de la pestaña de automatización cae, CDP emite `Target.targetCrashed`. mac-stats abre un WebSocket lateral (`cdp_target_crash_listener`) con `Target.setDiscoverTargets`, compara el `targetId` con la pestaña activa, invalida la sesión en caché, registra en `browser/cdp` y deja un mensaje one-shot para que la siguiente llamada a herramienta reconecte vía `with_connection_retry`.

---

## 2. References

- `src-tauri/src/browser_agent/cdp_target_crash_listener.rs` — `spawn_target_crash_side_listener`, bucle de lectura, `LISTENER_GEN`, envío `Target.setDiscoverTargets` **sin** clave `filter` inválida
- `src-tauri/src/browser_agent/mod.rs` — `notify_target_renderer_crashed_side`, `clear_cached_browser_session`, `debug_page_crash_current_automation_tab` / `_inner`, integración con `active_tab_target_id_store`
- `src-tauri/src/main.rs` — flag `--browser-debug-crash-tab` (requiere `browserToolsEnabled`)
- `docs/029_browser_automation.md` — comportamiento esperado ante crash del renderer

---

## 3. Acceptance criteria

1. `cdp_target_crash_listener.rs` arranca el listener lateral, envía `Target.setDiscoverTargets` **sin** `filter: null`, y reenvía `Target.targetCrashed` a `notify_target_renderer_crashed_side` cuando el `targetId` coincide con el seguimiento.
2. `browser_agent/mod.rs` implementa `notify_target_renderer_crashed_side`, `clear_cached_browser_session` ante crash, y `debug_page_crash_current_automation_tab` (smoke vía CLI).
3. `main.rs` expone `--browser-debug-crash-tab` condicionado a `browserToolsEnabled`.
4. `cargo check` y `cargo test --lib` en `src-tauri/` pasan.

---

## 4. Testing instructions

Ejecutar desde la **raíz del repositorio** (o ajustar rutas).

### Automated (required)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --lib
```

Comprobación estática (debe haber coincidencias en los **tres** archivos):

```bash
rg -n "targetCrashed|notify_target_renderer_crashed_side|spawn_target_crash_side_listener|debug_page_crash_current_automation_tab" \
  src-tauri/src/browser_agent/cdp_target_crash_listener.rs \
  src-tauri/src/browser_agent/mod.rs \
  src-tauri/src/main.rs
```

### Manual / smoke (optional)

1. En `~/.mac-stats/config.json`, `browserToolsEnabled` debe ser `true` y Chrome accesible en el puerto de depuración configurado (mismo flujo que herramientas BROWSER_*).
2. Tras `cargo build --release` en `src-tauri/`:
   ```bash
   ./src-tauri/target/release/mac_stats --browser-debug-crash-tab -vv
   ```
3. En consola debe imprimirse el mensaje de éxito de `Page.crash`; en `~/.mac-stats/debug.log` buscar:
   - `Target.setDiscoverTargets ok (listening for Target.targetCrashed)` (al conectar el listener),
   - `Target.targetCrashed for active automation tab` (WARN `browser/cdp`) tras el crash,
   - trazas de limpieza de sesión asociadas a `clear_cached_browser_session` / motivo `target renderer crashed`.
4. El binario **espera ~1,2 s** antes de salir para dar tiempo al hilo del WebSocket lateral; si se mata el proceso al instante, puede faltar el WARN en el log.
5. Si no aparece `Target.targetCrashed`, comprobar que la pestaña usada es la de automatización activa y que el listener lateral sigue vivo (misma generación de sesión); repetir con `-vvv` si hace falta más detalle.

---

## 5. Implementation summary

- WebSocket secundario con timeout de lectura corto para seguir en bucle sin morir en idle; invalidación por `LISTENER_GEN` al limpiar sesión.
- `Page.crash` en la pestaña actual vía `debug_page_crash_current_automation_tab_inner` para reproducir el evento CDP sin cliente externo.
- Mensaje one-shot a herramientas: `take_pending_renderer_crash_tool_error()` devuelve texto claro en la siguiente operación tras crash detectado.
