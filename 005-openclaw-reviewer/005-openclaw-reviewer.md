# 005 — OpenClaw reviewer (mac-stats workspace)

Cross-check **OpenClaw** `AGENTS.md` against the **openclaw** repository (sibling: `../openclaw` from mac-stats). This file is the landing doc for the reviewer role; fix discrepancies in **OpenClaw**, not here.

## Latest verification — 2026-03-22 (re-check)

**OpenClaw root:** `../openclaw`  
**References:** `AGENTS.md` (repo guidelines), `package.json` (scripts), `vitest.config.ts` (coverage).

### §7-style checks (doc vs code)

| # | Claim (AGENTS.md) | Code truth | Verdict |
|---|-------------------|------------|---------|
| 1 | WhatsApp web surface: `src/channel-web.ts` (L9) | **`src/channel-web.ts`** present; no `provider-web.ts` | **Aligned** |
| 2 | Messaging: `docs/channels/`, `src/channels/`, `src/routing`, `src/channel-web.ts`, `src/plugins/…`, `extensions/*` (L17–20) | **`src/channels/`**, **`src/plugins/`** exist; no bogus top-level `src/telegram` etc. | **Aligned** |
| 3 | Format check vs write (L71–72) | **`pnpm format:check`** → `oxfmt --check`; **`pnpm format`** / **`format:fix`** → `oxfmt --write`; **`pnpm check`** includes `format:check` | **Aligned** |
| 4 | Vitest coverage (L113) | `vitest.config.ts`: lines/functions/statements **70**, branches **55** | **Aligned** |
| 5 | TypeScript: `pnpm tsgo` (L68–69) | Doc states `tsgo` comes from **`@typescript/native-preview`** and that **`pnpm check`** invokes it; no separate `"tsgo"` script in `package.json` | **Aligned** |

### Notes

- **`format:fix`** and **`format`** remain redundant aliases (`oxfmt --write`); harmless.
- No OpenClaw code issues surfaced; this pass is doc↔config/layout consistency only.

### Historical (2026-03-22 morning)

Earlier the same day, `AGENTS.md` had stale paths (`provider-web.ts`, old channel dirs), wrong format wording, and wrong branch threshold. Those were corrected upstream; the table above confirms the current tree matches the doc.

**Upstream status:** `../openclaw/AGENTS.md` matches `package.json`, `vitest.config.ts`, and `src/` as of this re-verification.

---

## Scope (standing)

- **In scope:** OpenClaw `AGENTS.md`, `package.json` scripts, Vitest/coverage config, high-level `src/` layout claims.
- **Out of scope for this repo:** Applying fixes inside OpenClaw unless a separate task says so; this file records **verification** for mac-stats agents.
