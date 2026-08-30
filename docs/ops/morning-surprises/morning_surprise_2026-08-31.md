# Morning surprise — 2026-08-31

Overnight Track B kept shipping: Ready chips early, then an AI Chat design-review polish.

## Shipped tonight (so far)

| Version | What |
| --- | --- |
| **v0.1.745** | **AI Chat Errors glance** — red **Errors · N failed turns** strip when the transcript has `Error: …` replies (Debug Log error/warn parity). Click opens the Errors filter. Last-answer glance shows **Last error · …** with the same wash (opens Errors instead of copying). Design review / `feature-ai-chat` (screenshot recapture deferred — no on-screen CPU window). |
| **v0.1.744** | **`/voice` · `/stt` instant** — Discord voice STT Ready / Partial / Not set (model · ffmpeg · Ollama config; config only; Discord + AI Chat; does not steal voice-note / send-voice / enable-disable) |
| **v0.1.743** | **`/having_fun` · `/fun` · `/idle` instant** — Having fun / idle thoughts On/Off (channel count · idle · reply delays from `discord_channels.json`; config only; Discord + AI Chat; does not steal send/post / enable-disable) |
| **v0.1.742** | **`/ori` · `/mnemos` instant** — Ori Mnemos lifecycle Ready / Off / Partial (vault · orient · prefetch · capture · binary; config/env only; Discord + AI Chat; does not steal MCP `ori_*` / MEMORY_APPEND / scrub) |
| **v0.1.741** | **`/downloads` · `/organizer` instant** — Downloads organizer On/Off (interval · dry-run · path · last run; config only; Discord + AI Chat; Perplexity Ready `perplexity search status` reject fixed) |
| **v0.1.740** | **`/compact` · `/menu-bar` · `/cpu-window` instant** — Compact Menu bar / CPU window On/Off (`menuBarCompact` · `cpuWindowCompact`; config only; Discord + AI Chat) |

## Fuel / gate

- Digester **open** stayed empty; design review **due=false** (grace; `feature-ai-chat` still recommended).
- Fuel = standing backlog **design-review polish** after Ready chips through `/voice`.
- Latest keep: **v0.1.745** @ f967bd38.

## Next for Ralf

- Recapture `feature-ai-chat.png` when the CPU window is on-screen (Problem Reporter was open during this tick).
- Watch idle-thought Discord timeout WARNs (retry from v0.1.703); `/having_fun` shows idle/reply windows without Ollama.
- Digester open / more tool-heavy patterns when they appear.
