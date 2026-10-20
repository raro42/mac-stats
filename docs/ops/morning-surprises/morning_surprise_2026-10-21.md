# Morning surprise — 2026-10-21

Overnight autoresearch kept **v0.1.2172** (also **v0.1.2171**, **v0.1.2170**, and **v0.1.2169** earlier this window).

## Shipped

- **Swiss Ollama settings popover shell** — `.popover-content` no longer uses glass `--panel` / rgba or a soft panel shadow. Fill and border mix against the opaque shell fill `#ffffff` (Swiss body / `.swiss-shell`).
- Earlier tonight: **Swiss Ollama Reset** on `#ffffff` in **v0.1.2171**; **Swiss Ollama Save** on `#ffffff` in **v0.1.2170**; **Architect Ollama textarea / header / Close** on `#f1f1f1` in **v0.1.2169**.
- Why it matters: WebView alpha on always-openable AI Chat settings still shows up in Graphics and Media (#14). Opaque mixes keep the settings panel solid on Swiss.

## Tried / deferred

- Digester empty; sibling harness paths missing on this host.
- Design-review screenshot `feature-agent-ops.png` still deferred (`screencapture -l` / Screen Recording TCC / no on-screen CPU window on Linux rack).
- Layout-daily (Data Poster) still due — next fuel after Swiss Ollama textarea / header, or Data Poster Ollama primary, or a dedicated layout tick.
- Apple Ollama Save already opaque; skipped.

## Ratchet

- keep @ `b105a325` — Swiss Ollama settings popover shell mixes on #ffffff (no glass rgba/shadow). v0.1.2172
- keep @ `18a7d72c` — Swiss Ollama secondary (Reset) mixes on #ffffff (no glass rgba). v0.1.2171
- keep @ `257522e7` — Swiss Ollama Save mixes on #ffffff (no glass rgba). v0.1.2170
- keep @ `84d43358` — Architect Ollama textarea + header chrome mix on #f1f1f1 (no glass rgba). v0.1.2169
