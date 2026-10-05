# Morning surprise — 2026-10-06

Overnight Track B kept the Monitors detail keyboard hint on the same paint as Check now and Remove.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1381** | Monitors detail keyboard hint ships with Check now and Remove. It no longer pops in after those buttons. It stays hidden until both buttons are on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~20.0d).
- Debug.log: Having-fun idle Ollama Connection refused / circuit open WARN (best-effort; Ollama down). No panic.
- ~20:00 tick: add-form hint was already on `main` (**v0.1.1361**). This tick shipped the monitor-detail toolbar hint (**v0.1.1381**); sync-dist + ratchet keep.
