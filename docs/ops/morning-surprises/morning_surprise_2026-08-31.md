# Morning surprise — 2026-08-31

Overnight Track B kept shipping operator Ready chips so config asks stay instant (no Ollama).

## Shipped tonight (so far)

| Version | What |
| --- | --- |
| **v0.1.744** | **`/voice` · `/stt` instant** — Discord voice STT Ready / Partial / Not set (model · ffmpeg · Ollama config; config only; Discord + AI Chat; does not steal voice-note / send-voice / enable-disable) |
| **v0.1.743** | **`/having_fun` · `/fun` · `/idle` instant** — Having fun / idle thoughts On/Off (channel count · idle · reply delays from `discord_channels.json`; config only; Discord + AI Chat; does not steal send/post / enable-disable) |
| **v0.1.742** | **`/ori` · `/mnemos` instant** — Ori Mnemos lifecycle Ready / Off / Partial (vault · orient · prefetch · capture · binary; config/env only; Discord + AI Chat; does not steal MCP `ori_*` / MEMORY_APPEND / scrub) |
| **v0.1.741** | **`/downloads` · `/organizer` instant** — Downloads organizer On/Off (interval · dry-run · path · last run; config only; Discord + AI Chat; Perplexity Ready `perplexity search status` reject fixed) |
| **v0.1.740** | **`/compact` · `/menu-bar` · `/cpu-window` instant** — Compact Menu bar / CPU window On/Off (`menuBarCompact` · `cpuWindowCompact`; config only; Discord + AI Chat) |

## Fuel / gate

- Digester **open** stayed empty; design review **due=false** (grace; `feature-ai-chat` still recommended).
- Fuel = standing backlog **p50** — after `/having_fun`, Discord voice STT Ready chip (sibling voice harden + operator gap).
- Latest keep: **v0.1.744** @ 7c26c0a8.

## Next for Ralf

- Design-review polish when grace ends (`feature-ai-chat` first), or more tool-heavy digester patterns.
- Watch idle-thought Discord timeout WARNs (retry from v0.1.703); `/having_fun` shows idle/reply windows without Ollama.
- `/voice` shows ffmpeg + Ollama set state without a live transcribe.
