# Morning surprise — 2026-08-30

Overnight Track B kept shipping operator Ready chips and catalogs so config and inventory asks stay instant (no Ollama).

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.735** | **`/judge` instant** — Judge Ready / Off (`agentJudgeEnabled` · failure-only vs every run; config only, no judge run; Discord + AI Chat; does not steal “judge this” / enable/disable / score) |
| **v0.1.734** | **`/browser` · `/cdp` instant** — Browser / CDP Ready / Off / Not set (Chromium path + port; config only, no live probe; Discord + AI Chat; does not steal `BROWSER_*` / screenshot / navigate) |
| **v0.1.733** | **`/plugins` · `/plugins on` · `/plugins off` instant** — registered script plugins On/Off list (Agents On/Off parity; Discord + AI Chat; no script run; does not steal add/run/remove) |
| **v0.1.732** | **`/tasks` · `/tasks all` instant** — Active (open·WIP) or All task files from `~/.mac-stats/task/` (TASK_LIST parity; Discord + AI Chat; does not steal `TASK_CREATE:` / `TASK_SHOW:`) |
| **v0.1.731** | **`/skills` instant** — installed skills catalog from `~/.mac-stats/agents/skills/` (Hermes skills_list / SKILLS_LIST parity; Discord + AI Chat; does not steal `SKILL:` / `SKILL_VIEW:`) |
| **v0.1.730** | **`/telegram` · `/slack` · `/signal` · `/alerts` instant** — Ready / Not set / Partial (Keychain + registry; Signal REST not wired; Discord catch-all for prior Ready chips) |
| **v0.1.729** | **`/cursor` · `/cursor-agent` instant** — Ready / Not set with PATH cue (no CLI probe; Discord + AI Chat; does not steal `CURSOR_AGENT:`) |
| **v0.1.728** | **`/mcp` instant** — Ready / Not set with stdio command or HTTP host (config only; no `tools/list`; Discord + AI Chat; does not steal `MCP: <tool>`) |
| **v0.1.727** | **`/mastodon` instant** — Ready / Not set / Partial with instance host · token |
| **v0.1.726** | **`/perplexity key` instant** — Ready / Not set with key cue (keeps `/perplexity` Top/Snippet) |
| **v0.1.725** | **`/brave` instant** — Ready / Not set with key cue (no quota burn) |
| **v0.1.724** | **`/redmine` instant** — Ready / Not set / Partial (Agent Ops health parity) |
| **v0.1.723** | **`/ollama` · `/llm` instant** — Ready / Offline (model · endpoint · circuit) |
| **v0.1.722** | **`/discord` instant** — Discord Ready / Offline with reconnect cues |

## Fuel / gate

- Digester **open** stayed empty; design review **due=false** (grace; `feature-ai-chat` still recommended).
- Fuel = standing backlog **p50** — after `/browser`, Judge Ready chip (`/judge`).
- Latest keep: **v0.1.735**.

## Next for Ralf

- Design-review polish when grace ends (`feature-ai-chat` first), or more tool-heavy digester patterns.
- Watch idle-thought Discord timeout WARNs (retry from v0.1.703).
