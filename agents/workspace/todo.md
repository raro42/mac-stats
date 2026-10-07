# WIP-14 / #14 — tauri://localhost CPU (v0.1.1718 modal-backdrop)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1717 (ring-track)
- [x] Opaque `#f2f2f6` mix for modal / popover backdrops (`--modal-backdrop`) (v0.1.1718)
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (do not close)

## Review
- Coder: `--modal-backdrop` was `rgba(0, 0, 0, 0.30)` glass under Settings · Monitors · Ollama full-screen dimmers. Now `color-mix(in srgb, #000000 30%, #f2f2f6)` so the wash mixes against opaque body fill.
- Tester next: open CPU window on macOS, open Settings / Monitors settings / AI Chat settings, warm ≥30s. Confirm dimmer stays solid (no glass alpha). Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: remaining text-color rgba (`--text` / `--muted` / section greys) is lower priority; screenshot feature-agent-ops when TCC allows.
