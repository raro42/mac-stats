# 005 — OpenClaw reviewer (mac-stats workspace)

Cross-check **OpenClaw** `AGENTS.md` against the **openclaw** repository (sibling: `../openclaw` from mac-stats). This file is the landing doc for the reviewer role; fix discrepancies in **OpenClaw**, not here.

## Latest verification — 2026-03-26

**OpenClaw root:** `../openclaw`  
**OpenClaw `HEAD`:** `55dc6a8bb28dd9b6f4c4c1a8e8497fa48f7cc254` (short: `55dc6a8bb2`)  
**References:** `AGENTS.md` (repo guidelines), `package.json` (scripts), `vitest.config.ts` (coverage; `vitest.unit.config.ts` extends it for `pnpm test:coverage`).

**Independent re-run:** `2026-03-26T14:40:51Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T14:34:00Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T14:26:28Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T14:19:57Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T14:13:14Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T14:01:41Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T13:39:59Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T13:08:07Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md`; `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-26T12:59:14Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T12:47:17Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T12:40:16Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T12:28:23Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T12:17:14Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T12:05:53Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T11:59:31Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T11:52:42Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T11:46:35Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T10:31:18Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T10:09:26Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T09:51:28Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T09:39:53Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T09:07:50Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T08:41:03Z` — `HEAD` `55dc6a8bb2` (unchanged since prior check); identical conclusions (**Aligned**).

**Prior:** `2026-03-26T08:31:16Z` — `HEAD` `55dc6a8bb2`; identical conclusions (**Aligned**).

**Prior:** `2026-03-26T08:11:53Z` — `HEAD` `55dc6a8bb2`; identical conclusions (**Aligned**).

**Prior:** `2026-03-26T07:22:55Z` — `HEAD` `55dc6a8bb2`; identical conclusions (**Aligned**).

**Prior:** `2026-03-25T11:03:16Z` — `HEAD` `55dc6a8bb2`; same structural conclusions (**Aligned**).

**Prior:** `2026-03-25T03:37:03Z` — `HEAD` `d25b4a2`; same structural conclusions (**Aligned**).

**Prior:** `2026-03-25T03:03:07Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-25T02:34:05Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-25T01:57:44Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-25T01:16:52Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-25T00:40:23Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-25T00:12:11Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T23:40:37Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T23:11:50Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T22:45:25Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T22:16:48Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T21:47:31Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T21:20:02Z` — same `HEAD` `d25b4a2`; `src/channel-web.ts` present, no `src/provider-web.ts`; `src/cli`, `src/commands`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`, and `extensions/*` present; no bogus top-level `src/telegram`; `package.json` `check` chain still includes `format:check` → `pnpm tsgo` (no `"tsgo"` script key; binary from `@typescript/native-preview` `7.0.0-dev.20260322.1`) → …; `pnpm format` / `format:fix` / `format:check` match `AGENTS.md` (including `oxfmt --check --threads=1` for format check); `vitest.config.ts` thresholds 70/70/55/70; `vitest.unit.config.ts` still extends base; §7 table unchanged (**Aligned**).

**Prior:** `2026-03-24T20:51:28Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T20:12:10Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T19:43:54Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T19:20:31Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-24T18:50:06Z` — same `HEAD` `d25b4a2`; identical conclusions.

**Prior:** `2026-03-23T13:17:11Z` — `HEAD` `95ae8aabb77a99bed6747698fe810f6b8e34490b`; same structural conclusions before upstream advanced.

**Prior:** `2026-03-23T12:57:06Z`, `2026-03-23T12:13:46Z`, `2026-03-23T11:54:40Z`, `2026-03-23T11:35:43Z`, `2026-03-23T11:14:38Z`, `2026-03-23T10:57:09Z`, `2026-03-23T10:39:52Z`, `2026-03-23T10:23:17Z` — identical prior `HEAD` and conclusions.

**Earlier 2026-03-23:** `10:04:56Z`, `09:47:10Z`, `09:26:07Z`, `09:10:04Z`, `08:53:39Z` — identical `HEAD` and conclusions.

### §7-style checks (doc vs code)

| # | Claim (AGENTS.md) | Code truth | Verdict |
|---|-------------------|------------|---------|
| 1 | WhatsApp web surface: `src/channel-web.ts` (L9) | **`src/channel-web.ts`** present; no `provider-web.ts` | **Aligned** |
| 2 | Messaging: `docs/channels/`, `src/channels/`, `src/routing`, `src/channel-web.ts`, `src/plugins/…`, `extensions/*` (L18–21) | **`src/channels/`**, **`src/routing/`** (directory), **`src/plugins/`**, **`extensions/*`** present; channel code not misplaced as bogus top-level dirs (e.g. `src/telegram`) | **Aligned** |
| 3 | Format check vs write (L72–73) | **`pnpm format:check`** → `oxfmt --check --threads=1`; **`pnpm format`** / **`format:fix`** → `oxfmt --write`; **`pnpm check`** includes `format:check` | **Aligned** |
| 4 | Vitest coverage (L114) | `vitest.config.ts`: lines/functions/statements **70**, branches **55** | **Aligned** |
| 5 | TypeScript: `pnpm tsgo` (L70) | Doc states `tsgo` comes from **`@typescript/native-preview`** and that **`pnpm check`** invokes it; no separate `"tsgo"` script in `package.json` | **Aligned** |

### Notes

- **`format:fix`** and **`format`** remain redundant aliases (`oxfmt --write`); harmless.
- No OpenClaw code issues surfaced; this pass is doc↔config/layout consistency only.
- Confirmed on disk: `src/cli`, `src/commands`, `src/channel-web.ts`, `src/infra`, `src/media`, `src/channels/`, `src/routing/`, `src/plugins/`; no `src/provider-web.ts`.
- **`@typescript/native-preview`** in root `package.json` is `7.0.0-dev.20260322.1` (devDependency; `pnpm tsgo` uses the binary from `.bin` after install). Not spelled out in `AGENTS.md`; no doc change required.

### Historical (2026-03-22 morning)

Earlier the same day, `AGENTS.md` had stale paths (`provider-web.ts`, old channel dirs), wrong format wording, and wrong branch threshold. Those were corrected upstream; the table above confirms the current tree matches the doc.

**Upstream status:** `../openclaw/AGENTS.md` matches `package.json`, `vitest.config.ts`, and `src/` as of 2026-03-26 (`55dc6a8bb2`; last check `2026-03-26T14:40:51Z`).

---

## Scope (standing)

- **In scope:** OpenClaw `AGENTS.md`, `package.json` scripts, Vitest/coverage config, high-level `src/` layout claims.
- **Out of scope for this repo:** Applying fixes inside OpenClaw unless a separate task says so; this file records **verification** for mac-stats agents.
