# Morning surprise — 2026-10-21

Overnight autoresearch (Track B) kept shipping opaque Data Poster chrome for GitHub #14.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| v0.1.2189 | Data Poster AI Chat `.ollama-menu` / `.ollama-menu-btn` / `.ollama-menu-item` opaque on `#0b0b10` (no glass panel/muted rgba / soft shadow) |
| v0.1.2188 | Data Poster AI Chat `.thinking-dots span` opaque on `#0b0b10` (no glass muted rgba) |
| v0.1.2187 | Data Poster AI Chat `.model-text` resting + hover opaque on `#0b0b10` (no glass muted/hairline rgba) |
| v0.1.2186 | Data Poster AI Chat markdown in bubbles (blockquote / code / pre / table / hr / link) opaque on `#0b0b10` |
| v0.1.2185 | Data Poster AI Chat `.model-select-dropdown` opaque on `#0b0b10` (no glass rgba / soft shadow) |
| v0.1.2184 | Data Poster AI Chat `.chat-message.user` / `.assistant` opaque on `#0b0b10` (no glass rgba) |
| v0.1.2183 | Data Poster AI Chat `.chat-messages` opaque on `#0b0b10` (no glass rgba) |
| v0.1.2182 | Data Poster AI Chat `.model-select` opaque on `#0b0b10` (no glass rgba) |
| v0.1.2181 | Data Poster Monitors `.add-btn-small` opaque |
| v0.1.2180 | Data Poster Monitors Remove + add-form opaque |
| v0.1.2179 | Data Poster Monitors header / Close / list-row opaque |
| v0.1.2178 | Data Poster Monitors settings `.popover-content` opaque |
| v0.1.2177 | Data Poster Ollama textarea + header chrome opaque |
| v0.1.2176 | Data Poster Ollama `.popover-content` opaque |
| v0.1.2175 | Data Poster Ollama Reset secondary opaque |
| v0.1.2174 | Data Poster Ollama Save primary opaque |

## Still open

- Data Poster `.settings-card` / Settings shell if still glass (`var(--panel)`)
- Stale design-review screens (`feature-agent-ops.png` ~36d) — Mac `screencapture -l` / Screen Recording TCC still blocked on this rack
- Digester empty (ask Werner something)

## Fitness

The Data Poster Ollama ⋯ menu no longer paints glass panel rgba through the shell. Menu shell, button type/hover, and item hover mix against the opaque `#0b0b10` field, same pattern as thinking-dots and model-text.
