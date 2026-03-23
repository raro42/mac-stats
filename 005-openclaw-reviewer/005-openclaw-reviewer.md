# 005 — OpenClaw reviewer (mac-stats workspace)

Cross-check **OpenClaw** `AGENTS.md` against the **openclaw** repository (sibling: `../openclaw` from mac-stats). This file is the landing doc for the reviewer role; fix discrepancies in **OpenClaw**, not here.

## Latest verification — 2026-03-23

**OpenClaw root:** `../openclaw`  
**OpenClaw `HEAD`:** `95ae8aabb77a99bed6747698fe810f6b8e34490b` (short: `95ae8aa`)  
**References:** `AGENTS.md` (repo guidelines), `package.json` (scripts), `vitest.config.ts` (coverage; `vitest.unit.config.ts` extends it for `pnpm test:coverage`).

**Independent re-run:** `2026-03-23T11:35:43Z` — same `HEAD` (`95ae8aabb77a99bed6747698fe810f6b8e34490b`); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key in `scripts`; binary from `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `format` / `format:fix` / `format:check` match doc; `vitest.config.ts` thresholds still 70/70/55/70; `vitest.unit.config.ts` still extends base; table below unchanged (no drift vs `AGENTS.md`).

**Prior:** `2026-03-23T11:14:38Z`, `2026-03-23T10:57:09Z`, `2026-03-23T10:39:52Z`, `2026-03-23T10:23:17Z` — identical `HEAD` and conclusions.

**Earlier 2026-03-23:** `10:04:56Z`, `09:47:10Z`, `09:26:07Z`, `09:10:04Z`, `08:53:39Z` — identical `HEAD` and conclusions.

### §7-style checks (doc vs code)

| # | Claim (AGENTS.md) | Code truth | Verdict |
|---|-------------------|------------|---------|
| 1 | WhatsApp web surface: `src/channel-web.ts` (L9) | **`src/channel-web.ts`** present; no `provider-web.ts` | **Aligned** |
| 2 | Messaging: `docs/channels/`, `src/channels/`, `src/routing`, `src/channel-web.ts`, `src/plugins/…`, `extensions/*` (L17–20) | **`src/channels/`**, **`src/routing/`** (directory), **`src/plugins/`**, **`extensions/*`** present; channel code not misplaced as bogus top-level dirs (e.g. `src/telegram`) | **Aligned** |
| 3 | Format check vs write (L71–72) | **`pnpm format:check`** → `oxfmt --check`; **`pnpm format`** / **`format:fix`** → `oxfmt --write`; **`pnpm check`** includes `format:check` | **Aligned** |
| 4 | Vitest coverage (L113) | `vitest.config.ts`: lines/functions/statements **70**, branches **55** | **Aligned** |
| 5 | TypeScript: `pnpm tsgo` (L68–69) | Doc states `tsgo` comes from **`@typescript/native-preview`** and that **`pnpm check`** invokes it; no separate `"tsgo"` script in `package.json` | **Aligned** |

### Notes

- **`format:fix`** and **`format`** remain redundant aliases (`oxfmt --write`); harmless.
- No OpenClaw code issues surfaced; this pass is doc↔config/layout consistency only.
- Confirmed on disk: `src/cli`, `src/commands`, `src/channel-web.ts`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`; no `src/provider-web.ts`.
- **`@typescript/native-preview`** in root `package.json` is `7.0.0-dev.20260322.1` (devDependency; `pnpm tsgo` uses the binary from `.bin` after install). Not spelled out in `AGENTS.md`; no doc change required.

### Historical (2026-03-22 morning)

Earlier the same day, `AGENTS.md` had stale paths (`provider-web.ts`, old channel dirs), wrong format wording, and wrong branch threshold. Those were corrected upstream; the table above confirms the current tree matches the doc.

**Upstream status:** `../openclaw/AGENTS.md` matches `package.json`, `vitest.config.ts`, and `src/` as of 2026-03-23 (`95ae8aa`).

---

## Scope (standing)

- **In scope:** OpenClaw `AGENTS.md`, `package.json` scripts, Vitest/coverage config, high-level `src/` layout claims.
- **Out of scope for this repo:** Applying fixes inside OpenClaw unless a separate task says so; this file records **verification** for mac-stats agents.
