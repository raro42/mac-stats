# Morning surprise — 2026-10-24

Overnight Track B kept shipping Data Poster opaque washes (standing P2 / GitHub #14). Digester stayed empty; design review stayed due on stale feature screens.

## Shipped last night

| Version | What got nicer |
|---------|----------------|
| **v0.1.2249** | Disk Cleanup keyboard hints (`.disk-cleanup-meta-kb-hint` · `.disk-cleanup-toolbar-kb-hint` · `.disk-cleanup-list-kb-hint` · `.disk-cleanup-add-scope-toolbar-kb-hint` · `.disk-cleanup-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so tips under Clean now, scopes, and the meta cards match window chrome hints |
| **v0.1.2248** | Window chrome keyboard hints (`.icon-line-kb-hint` · `.footer-toolbar-kb-hint` · `.header-toolbar-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so tips under section icons, Refresh · Settings, and version · GitHub match metrics / Ollama kb-hints |
| **v0.1.2247** | Ollama settings keyboard hints (`.ollama-settings-header-kb-hint` · `.ollama-settings-toolbar-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so tips under the title and Reset · Save match AI Chat / metrics kb-hints |
| **v0.1.2246** | AI Chat response-time (`.response-time`) mixes on shell `#0b0b10` (no early `#0e0e14`) so latency stamps under assistant turns match the message / exec chrome |
| **v0.1.2245** | Metrics keyboard hints (`.power-strip-kb-hint` · `.ring-gauge-kb-hint` · `.history-sparkline-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so Bat / LPM / Power and ring tips match the opaque strip and gauge cards |
| **v0.1.2244** | AI Chat exec label (`.chat-exec-label`) mixes on shell `#0b0b10` (no early `#0e0e14`) so tool/exec headings match the exec / answer cards |
| **v0.1.2243** | AI Chat keyboard hints (`.chat-empty-kb-hint` · `.chat-composer-kb-hint` · `.chat-kb-hint` · `.filter-chip-kb-hint`) mix on shell `#0b0b10` (no early `#0e0e14`) so composer / chip tips match the empty shell |
| **v0.1.2242** | AI Chat filter-miss message type (`.chat-filter-miss-msg`) mixes on shell `#0b0b10` (no early `#0e0e14`) so empty You / Assistant / Errors copy matches the filter-miss shell |

## Design review

- Helper said **due** — recommended `feature-agent-ops.png` (~38d stale) on this host; `feature-ai-chat.png` and `feature-disk-cleanup.png` also stale.
- This tick polished Disk Cleanup keyboard hints (v0.1.2249) after window chrome kb-hints (v0.1.2248). Window-only Mac recapture still deferred when `screencapture -l` / Screen Recording TCC is unavailable on this host.

## Next fuel

- Monitors / Details / Processes / Settings / credential toolbar kb-hints still on early `#0e0e14`.
- Mac capture of `feature-agent-ops` / `feature-ai-chat` / `feature-disk-cleanup` when TCC allows.
