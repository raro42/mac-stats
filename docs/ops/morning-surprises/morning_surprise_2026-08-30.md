# Morning surprise — 2026-08-30

Overnight Track B kept shipping operator Ready chips and a skills catalog so config and inventory asks stay instant (no Ollama).

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.731** | **`/skills` instant** — installed skills catalog from `~/.mac-stats/agents/skills/` (Hermes skills_list / SKILLS_LIST parity; Discord + AI Chat; does not steal `SKILL:` / `SKILL_VIEW:`) |
| **v0.1.730** | **`/telegram` · `/slack` · `/signal` · `/alerts` instant** — Ready / Not set / Partial (Keychain + registry; Signal REST not wired; Discord catch-all for prior Ready chips) |
| **v0.1.729** | **`/cursor` · `/cursor-agent` instant** — Ready / Not set with PATH cue (no CLI probe; Discord + AI Chat; does not steal `CURSOR_AGENT:`) |
| **v0.1.728** | **`/mcp` instant** — Ready / Not set with stdio command or HTTP host (config only; no `tools/list`; Discord + AI Chat; does not steal `MCP: <tool>`) |
| **v0.1.727** | **`/mastodon` instant** — Ready / Not set / Partial with instance host · token |
| **v0.1.726** | **`/perplexity key` instant** — Ready / Not set with key cue (keeps `/perplexity` Top/Snippet) |
| **v0.1.725** | **`/brave` instant** — Ready / Not set with key cue (no quota burn) |
| **v0.1.724** | **`/redmine` instant** — Ready / Not set / Partial (Agent Ops health parity) |
| **v0.1.723** | **`/ollama` · `/llm` instant** — Ready / Offline (model · endpoint · circuit) |
| **v0.1.722** | **`/discord` instant** — Ready / Offline with reconnect cues |

## Fuel / gate

- Digester **open** stayed empty; design review **due=false** (grace; `feature-ai-chat` still recommended).
- Fuel = standing backlog **p50** — after Ready chips through alerts, `/skills` catalog (Hermes skills_list).
- Latest keep: **v0.1.731** @ `b58c3a11`.

## Next for Ralf

- Design-review polish when grace ends (`feature-ai-chat` first), or more tool-heavy digester patterns (`/plugins`, task list).
- Watch idle-thought Discord timeout WARNs (retry from v0.1.703).
