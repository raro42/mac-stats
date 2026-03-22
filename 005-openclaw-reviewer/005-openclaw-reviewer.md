# 005 — OpenClaw reviewer (mac-stats workspace)

Cross-check **OpenClaw** `AGENTS.md` against the **openclaw** repository (sibling: `../openclaw` from mac-stats). This file is the landing doc for the reviewer role; fix discrepancies in **OpenClaw**, not here.

## Latest verification — 2026-03-22

**OpenClaw root:** `../openclaw`  
**References:** `AGENTS.md` (repo guidelines), `package.json` (scripts), `vitest.config.ts` (coverage).

### §7-style checks (doc vs code)

| # | Claim (AGENTS.md) | Code truth | Verdict |
|---|-------------------|------------|---------|
| 1 | L8: web provider at `src/provider-web.ts` | File absent; **`src/channel-web.ts`** exists | **Doc wrong** — rename reference |
| 2 | L18–19: core channel code under `src/telegram`, `src/discord`, `src/slack`, `src/signal`, `src/imessage`, `src/web` | **`src/telegram`**, **`src/discord`**, etc. are **not** top-level dirs; **`src/channels`** exists; channel/plugin wiring under **`src/plugins/`** (e.g. `src/plugins/runtime/`) and **`extensions/*`** | **Doc wrong** — layout moved; update paths |
| 3 | L71: “Format check: `pnpm format` (oxfmt --check)” | **`pnpm format`** → `oxfmt --write`. **`pnpm format:check`** → `oxfmt --check`. **`pnpm check`** runs `format:check` | **Doc wrong** |
| 4 | L113: Vitest “70% lines/**branches**/functions/statements” | `vitest.config.ts` thresholds: **lines 70, functions 70, statements 70, branches 55** | **Doc wrong** — branches are 55% |
| 5 | L69: “TypeScript checks: `pnpm tsgo`” | No `"tsgo"` entry under `package.json` **scripts**; `pnpm check` still invokes `pnpm tsgo` → resolves via **`@typescript/native-preview`** (devDependency) **binary** after `pnpm install` | **Doc incomplete** — worth one line that `tsgo` is the dependency CLI, not a repo script |

### Unchanged / notes

- **`format:fix`** and **`format`** both run `oxfmt --write` in `package.json` (redundant aliases; not harmful).
- No OpenClaw **code** defects found in this pass; findings are **documentation accuracy** only.
- mac-stats **CHANGELOG** already recorded similar findings under §95/§96; this run **confirms** they still apply on 2026-03-22.

### Recommended edits (upstream OpenClaw `AGENTS.md`)

1. Replace `src/provider-web.ts` with `src/channel-web.ts` (or describe both if a split is intentional).
2. Replace the bullet list of fake top-level channel dirs with the real layout: `src/channels/`, `src/plugins/…`, `extensions/*`, and point to `docs/channels/` for detail.
3. Fix the format section: format **check** = `pnpm format:check`; format **write** = `pnpm format` or `pnpm format:fix`.
4. Fix testing thresholds: **branches 55%**, other metrics 70% (or reword to “70% with lower branch threshold”).

---

## Scope (standing)

- **In scope:** OpenClaw `AGENTS.md`, `package.json` scripts, Vitest/coverage config, high-level `src/` layout claims.
- **Out of scope for this repo:** Applying fixes inside OpenClaw unless a separate task says so; this file records **verification** for mac-stats agents.
