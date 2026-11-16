# Morning surprise — 2026-11-16

Overnight Track B kept shipping soft-state accent parity while Mac TCC still blocks window-only feature screenshots (`screencapture -l` → could not create image / Control Center chip only).

## Shipped tonight (window so far)

- **v0.1.2815** — Swiss Minimalistic design-review (Monitors add form): `.add-monitor-form input:focus` Apple `#007aff` → theme `--accent` Swiss ink `#121216`, matching Light / Material / Architect after v0.1.2800. Polish-without-capture grace marked for `feature-monitors`.
- **v0.1.2814** — Dark design-review (AI Chat composer): `body.theme-dark #chat-input:focus` in `agent-ops.css` poster cyan `#55c7ff` → theme `--accent` Matrix green `#00cc66` on `#0a0a0a`.
- **v0.1.2813** — Swiss Minimalistic design-review (AI Chat composer): `body.theme-swiss-minimalistic #chat-input:focus` in `agent-ops.css` Apple `#0a84ff` → theme `--accent` Swiss ink `#121216`, plus Swiss `.chat-input-container:focus-within` fallbacks → `#121216` (cream agent-ops parity after v0.1.2812). Polish-without-capture grace marked for `feature-disk-cleanup`.
- **v0.1.2812** — Light / Material / Architect design-review (AI Chat composer): `body.theme-* #chat-input:focus` in `agent-ops.css` Apple `#0a84ff` / `#007aff` → theme `--accent` (soft blue / taupe / slate).
- **v0.1.2811** — Swiss Minimalistic design-review (AI Chat user bubbles): `.chat-message.user` Apple `#007aff` → theme `--accent` Swiss ink `#121216`, matching Light / Material / Architect after v0.1.2799.
- **v0.1.2810** — Swiss Minimalistic design-review (AI Chat Ollama settings): `.ollama-settings-popover` primary + textarea focus Apple `#007aff` → theme `--accent` Swiss ink `#121216`.
- **v0.1.2809** — Light / Material / Architect design-review (AI Chat composer): `.chat-input-container:focus-within` Apple `#007aff` → theme `--accent`.
- **v0.1.2808** — Dark / Neon / Futuristic design-review (AI Chat composer): `.chat-input-container:focus-within` Apple `#007aff` → theme `--accent`.
- **v0.1.2807** — Light / Material / Architect / Swiss design-review (Top Processes section chrome): collapsible / theme-version / collapse-btn focus leftovers → theme `--accent`.
- **v0.1.2806** — Dark / Neon / Futuristic collapse + theme-version focus → theme `--accent`.
- **v0.1.2805** — Data Poster layout-daily: collapsible / theme-version / Monitors+Ollama menu + popover-close focus leftovers → poster cyan. Layout-daily **data-poster marked** for 2026-11-16.
- Earlier in the window: cream Force Quit / changelog / add-monitor / Ollama / scopes-off / CRT focus remaps (see loop_backlog).

## Tried / blocked

- Race: local Swiss Monitors-as-2814 collided with origin Dark chat-input **v0.1.2814** — discarded; re-shipped as **v0.1.2815**.
- Fresh `docs/screens/feature-monitors.png`: only Control Center chip on-screen (no CPU window id) — polish-without-capture grace marked; prior asset kept.
- Fresh `docs/screens/feature-disk-cleanup.png`: only Control Center chip on-screen (no CPU window id) — polish-without-capture grace marked; prior asset kept.
- Digester Slowest empty after instant-lane filters; fuel came from design-review + standing soft-states.
- Race earlier: planned CRT chat-input as 2808 already on origin — discarded and re-picked cream composer focus as 2809.

## Why it matters

Swiss Monitors add-URL focus no longer flashes Apple blue on cream paper. Dark AI Chat composer focus no longer shares Data Poster cyan when `agent-ops.css` loads after theme `cpu.css`.

## Next fuel

- Dark layout-daily on the next calendar day (helper recommends Dark).
- Mac capture for stale `feature-monitors.png` when TCC allows (grace is temporary).
- Send white-on-blue intentional; Dark filter-miss / monitors shells still poster cyan `#55c7ff` fallbacks if muddy; Neon/Futuristic agent-ops `#chat-input:focus` if muddy.
