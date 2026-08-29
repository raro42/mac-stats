# Morning surprise — 2026-08-30

Overnight Track B kept shipping operator Ready chips so config checks stay instant (no Ollama).

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.728** | **`/mcp` instant** — Ready / Not set with stdio command or HTTP host (config only; no `tools/list`; Discord + AI Chat; does not steal `MCP: <tool>`) |
| **v0.1.727** | **`/mastodon` instant** — Ready / Not set / Partial with instance host · token |
| **v0.1.726** | **`/perplexity key` instant** — Ready / Not set with key cue (keeps `/perplexity` Top/Snippet) |
| **v0.1.725** | **`/brave` instant** — Ready / Not set with key cue (no quota burn) |
| **v0.1.724** | **`/redmine` instant** — Ready / Not set / Partial (Agent Ops health parity) |
| **v0.1.723** | **`/ollama` · `/llm` instant** — Ready / Offline (model · endpoint · circuit) |
| **v0.1.722** | **`/discord` instant** — Ready / Offline with reconnect cues |

## Fuel / gate

- Digester **open** stayed empty; design review **due=false** (grace; `feature-ai-chat` still recommended).
- Fuel = standing backlog **p50** Ready chips (Discord → … → Mastodon → MCP).
- Latest keep: **v0.1.728** @ `44b39d34`.

## Next for Ralf

- Design-review polish when grace ends (`feature-ai-chat` first), or Signal/Telegram key chips / tool-heavy digester patterns.
- Watch idle-thought Discord timeout WARNs (retry from v0.1.703).
