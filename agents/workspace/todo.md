# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1717 shipped)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1716 (markdown table/hr)
- [x] Opaque `#ffffff` mix for ring gauge tracks (`.ring-track` / `--ring-track`) (v0.1.1717)
- [x] `cargo check` via autoresearch ratchet verify
- [ ] Rename / refresh UNTESTED-14 claim after bounce
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (#14 already closed on GitHub)

## Review
- Coder: `--ring-track` was `rgba(0, 0, 0, 0.06)` glass under always-visible CPU · GPU · Freq · Temp rings. Now `color-mix(in srgb, #000000 6%, #ffffff)` so the track mixes against opaque white (metric-card fill).
- Tester next: open CPU window, warm ≥30s. Confirm ring tracks stay solid (no glass alpha). Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: modal backdrop glass (`--modal-backdrop` on Settings / Monitors / Ollama popovers) only while open; remaining text-color rgba is lower priority; screenshot feature-agent-ops when TCC allows.
