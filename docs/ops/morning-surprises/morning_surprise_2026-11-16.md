# Morning surprise — 2026-11-16

Overnight Track B kept shipping soft-state accent parity while Mac TCC still blocks window-only feature screenshots (`screencapture -l` → could not create image / Control Center chip only).

## Shipped tonight (window so far)

- **v0.1.2816** — Neon / Futuristic design-review (AI Chat composer): `body.theme-neon` / `body.theme-futuristic #chat-input:focus` in `agent-ops.css` — Futuristic bare poster cyan `#55c7ff` on `#0b0b10` → theme `--accent` `#3bd6ff` on `#111123`; Neon onto `var(--accent, #6dff7b)` on `#08120c`. Matches theme `cpu.css` and `.chat-input-container:focus-within` from v0.1.2808. Polish-without-capture grace marked for `feature-ai-chat`.
- **v0.1.2815** — Swiss Minimalistic design-review (Monitors add form): `.add-monitor-form input:focus` Apple `#007aff` → theme `--accent` Swiss ink `#121216`, matching Light / Material / Architect after v0.1.2800. Polish-without-capture grace marked for `feature-monitors`.
- **v0.1.2814** — Dark design-review (AI Chat composer): `body.theme-dark #chat-input:focus` in `agent-ops.css` poster cyan `#55c7ff` → theme `--accent` Matrix green `#00cc66` on `#0a0a0a`.
- **v0.1.2813** — Swiss Minimalistic design-review (AI Chat composer): `body.theme-swiss-minimalistic #chat-input:focus` in `agent-ops.css` Apple `#0a84ff` → theme `--accent` Swiss ink `#121216`, plus Swiss `.chat-input-container:focus-within` fallbacks → `#121216` (cream agent-ops parity after v0.1.2812).
- **v0.1.2812** — Light / Material / Architect design-review (AI Chat composer): `body.theme-* #chat-input:focus` in `agent-ops.css` Apple `#0a84ff` / `#007aff` → theme `--accent` (soft blue / taupe / slate).
- **v0.1.2811** — Swiss Minimalistic design-review (AI Chat user bubbles): `.chat-message.user` Apple `#007aff` → theme `--accent` Swiss ink `#121216`.
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
- Digester Slowest empty after instant-lane filters; fuel came from design-review + standing soft-states.
- Race earlier: planned CRT chat-input as 2808 already on origin — discarded and re-picked cream composer focus as 2809.

## Why it matters

Futuristic AI Chat focus no longer flashes Data Poster cyan when `agent-ops.css` loads after theme `cpu.css`. Neon composer focus stays on theme `--accent` the same way. Swiss Monitors add-URL and Dark Matrix focus from earlier ticks stay intact.

## Next fuel

- Dark layout-daily on the next calendar day (helper recommends Dark).
- Mac capture for stale `feature-agent-ops.png` / `feature-monitors.png` when TCC allows (grace is temporary).
- Send white-on-blue intentional; Futuristic filter-miss / monitors empty CTA / chat-answer-final still poster cyan `#55c7ff` fallbacks if muddy; Dark early shared poster-cyan rules already overridden by late CRT remaps.
