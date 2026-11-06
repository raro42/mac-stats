# Morning surprise — 2026-11-07

Overnight Track B (design-review due + standing P2 #14 opaque remaps). Mac `screencapture -l` still blocked (Screen Recording TCC / no Quartz CPU window).

## Shipped

- **v0.1.2403** — Dark / Neon / Futuristic Disk Cleanup filter Clear (`.disk-cleanup-filter-clear` / `.disk-cleanup-scope-filter-clear`) opaque shells `#0a0a0a` / `#08120c` / `#0b0b10`.
- **v0.1.2402** — Dark / Neon / Futuristic Monitors · Processes · Logs filter Clear opaque.
- **v0.1.2401** — Dark / Neon / Futuristic Settings Save flashes opaque.
- **v0.1.2400** — Dark / Neon / Futuristic Perplexity filter Clear opaque + brace repair.

## Tried / deferred

- Recapture of stale `docs/screens/feature-agent-ops.png` (52d+) — deferred (TCC / no window id).
- First Monitors/Processes/Logs Clear attempt raced a sibling tick (`bfe39e1c`); reset and shipped Disk Cleanup Clear instead.

## Next fuel

- AI Chat `.chat-filter-clear` remaps if still Apple `#ffffff` on Dark/Neon/Futuristic.
- Other Settings Save leftovers (`#ollama-settings-save` / `#ops-agent-save` / `#monitors-add-save`).
- Mac capture when Screen Recording TCC allows `screencapture -l`.
