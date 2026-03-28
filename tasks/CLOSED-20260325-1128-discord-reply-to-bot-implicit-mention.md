# Discord: reply to bot counts as mention in MentionOnly

**Slug:** `20260325-1128-discord-reply-to-bot-implicit-mention`  
**Canonical task copy (reviewer workspace):** `mac-stats-reviewer/agents/tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md` (same slug; keep in sync when editing).

## Goal

In **MentionOnly** channels, a human message that **replies** to a message authored by the bot (Discord message reference) should activate the router as if the user had @mentioned the bot, without requiring a literal `<@bot>` mention.

## Acceptance criteria

1. **`discord_mentions_bot_effective`** returns true when the incoming message has a message reference to a message whose author is the bot (using `referenced_message` when the gateway provides it, else cache, else `get_message` fallback).
2. **Gateway `message` handler:** For non-DM, `ChannelMode::MentionOnly`, the early return that ignores non-mentions uses `mentions_bot_effective` (not only literal `mentions`), so reply-to-bot passes the gate.
3. **Observability:** Debug logs distinguish activation via reference vs literal mention (`MentionOnly activation via message reference` / `could not resolve referenced message for implicit mention`).
4. **Build:** `cargo check` in `src-tauri/` succeeds.

## Implementation (mac-stats)

- **`src-tauri/src/discord/mod.rs`:** `discord_mentions_bot_effective` (~1852); MentionOnly gate ~2814; router uses same helper ~1956.
- **`docs/007_discord_agent.md`:** `mention_only` reply-to-bot documented.

**Coder (2026-03-28 UTC):** Implementation already present; `cargo check` verified this run. No code changes.

---

## Testing instructions

**What to verify**

- In a guild channel configured **`mention_only`** in `~/.mac-stats/discord_channels.json`, a **Reply** to the bot’s **previous** message routes to the full Ollama/agent pipeline when the reply **does not** include a literal `@` mention of the bot.
- The bot **ignores** messages that neither mention it nor reply to a message it authored.
- **`HavingFun`** for humans unchanged.
- **`~/.mac-stats/debug.log`** at **`-vv`** includes a **DEBUG** line with **`MentionOnly activation via message reference`** (target **`mac_stats::discord`**) when activation is reference-only.

**How to test**

0. **Preflight:** `cd ~/projects/mac-stats/src-tauri && cargo check`; `cargo test outbound_attachment_path_allowlist -- --nocapture`; `rg -n 'discord_mentions_bot_effective|mentions_bot_effective' src-tauri/src/discord/mod.rs`; `rg -n 'MentionOnly activation via message reference|could not resolve referenced message for implicit mention' src-tauri/src/discord/mod.rs`.

1. Run mac-stats with Discord configured; test channel **`mention_only`** in `discord_channels.json`.
2. Start with **`-vv`**; confirm startup in `~/.mac-stats/debug.log`.
3. @mention the bot; wait for reply.
4. **Reply** to the bot’s last message with ping **off**; text without `@BotName`.
5. Bot should respond.
6. Plain message (no reply, no mention): bot should **not** respond in `mention_only`.
7. During step 4, `rg 'MentionOnly activation via message reference' ~/.mac-stats/debug.log`.
8. Optional: repeat in a **thread**.

**Pass/fail criteria**

- **Pass:** Steps 4–5 OK; step 6 no reply; step 7 shows debug line on reference-only activation.
- **Fail:** Reply-without-mention ignored in `mention_only`; spurious triggers; or missing debug line when reference-only activation occurs.

## Test report

**Date:** 2026-03-28 UTC (tester run)

**Commands run**

- `cd /Users/raro42/projects/mac-stats/src-tauri && cargo check` → **pass** (dev profile, 0 errors).
- `cargo test outbound_attachment_path_allowlist -- --nocapture` → **pass** (`discord::tests::outbound_attachment_path_allowlist`).
- `rg -n 'discord_mentions_bot_effective|mentions_bot_effective' src-tauri/src/discord/mod.rs` → hits at 1852, 1956–2016, 2787–2814 (router + MentionOnly gate use `mentions_bot_effective`).
- `rg -n 'MentionOnly activation via message reference|could not resolve referenced message for implicit mention' src-tauri/src/discord/mod.rs` → present with `target: "mac_stats::discord"` on `debug!` calls.

**Acceptance criteria**

1. **PASS** — `discord_mentions_bot_effective` (≈1852–1920): literal mention; else `referenced_message` author check + cache; else `get_message` fallback; failure path logs `could not resolve referenced message for implicit mention`.
2. **PASS** — Non-DM MentionOnly ignore path uses `!mentions_bot_effective` (≈2814–2815), not literal `mentions` only.
3. **PASS** — Reference-only activation and resolution-failure strings present; target `mac_stats::discord`.
4. **PASS** — `cargo check` succeeds.

**Manual Discord E2E** (task steps 1–8: reply without @, plain message, `debug.log` grep during live traffic): **not executed** in this run (no live Discord session). Operator smoke-test still recommended.

**Overall:** **PASS** (numbered acceptance criteria + preflight). Outcome: **CLOSED**.

---

## Test report

**Date:** 2026-03-28 UTC (tester run; Cursor agent)

**Path note:** Operator requested `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md`. That filename is **not present** in this workspace; the same slug exists as `tasks/CLOSED-20260325-1128-discord-reply-to-bot-implicit-mention.md`. Renaming `UNTESTED→TESTING` was **skipped** (no `UNTESTED-*` file to rename). Verification and this report were applied to the existing `CLOSED-*` task file.

**Commands run**

- `cd /Users/raro42/projects/mac-stats/src-tauri && cargo check` → **pass**
- `cargo test outbound_attachment_path_allowlist -- --nocapture` → **pass** (`discord::tests::outbound_attachment_path_allowlist`)
- `rg -n 'discord_mentions_bot_effective|mentions_bot_effective' src-tauri/src/discord/mod.rs` → hits at 1852, 1956–2016, 2796–2823
- `rg -n 'MentionOnly activation via message reference|could not resolve referenced message for implicit mention' src-tauri/src/discord/mod.rs` → present with `target: "mac_stats::discord"` on `debug!` calls (e.g. 1865–1917)

**Acceptance criteria (automated / code review)**

1. **PASS** — `discord_mentions_bot_effective`: literal mention; `referenced_message` path; cache; `get_message` fallback; failure logs `could not resolve referenced message for implicit mention`.
2. **PASS** — MentionOnly gate uses `mentions_bot_effective` (≈2823).
3. **PASS** — Distinct debug strings and `mac_stats::discord` target present.
4. **PASS** — `cargo check` succeeds.

**Manual Discord E2E** (steps 1–8 in task body): **not executed** (no live Discord in this environment).

**Outcome:** **PASS** on implementation + preflight. Filename already **CLOSED-**; no rename to `TESTED-` or `WIP-`.

---

## Test report

**Date:** 2026-03-28 UTC (tester run; operator path `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md`)

**Rename `UNTESTED→TESTING`:** **Skipped** — no `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md` in this workspace. Same slug is `tasks/CLOSED-20260325-1128-discord-reply-to-bot-implicit-mention.md`; verification applied here.

**Commands run**

- `cd /Users/raro42/projects/mac-stats/src-tauri && cargo check` → **pass** (dev profile, 0 errors).
- `cargo test outbound_attachment_path_allowlist -- --nocapture` → **pass** (`discord::tests::outbound_attachment_path_allowlist`).
- `rg -n 'discord_mentions_bot_effective|mentions_bot_effective' src-tauri/src/discord/mod.rs` → lines 1852, 1956, 2016, 2796–2797, 2823 (router + MentionOnly gate).
- `rg -n 'MentionOnly activation via message reference|could not resolve referenced message for implicit mention' src-tauri/src/discord/mod.rs` → lines 1867, 1888, 1901, 1915; `debug!` uses `target: "mac_stats::discord"`.

**Acceptance criteria**

1. **PASS** — `discord_mentions_bot_effective`: `referenced_message`, cache, `get_message` fallback, failure log string.
2. **PASS** — MentionOnly path uses `mentions_bot_effective` (≈2823).
3. **PASS** — Observability strings + `mac_stats::discord` target.
4. **PASS** — `cargo check` succeeds.

**Manual Discord E2E** (task steps 1–8): **not executed** in this environment.

**Outcome rename:** **CLOSED-** retained (all numbered acceptance criteria + preflight pass). No `TESTED-`/`WIP-` rename.

---

## Test report

**Date:** 2026-03-28 UTC (tester run; Cursor agent; operator path `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md`)

**Rename `UNTESTED→TESTING`:** **Skipped** — `tasks/UNTESTED-20260325-1128-discord-reply-to-bot-implicit-mention.md` does not exist in this workspace. The same slug is tracked as `tasks/CLOSED-20260325-1128-discord-reply-to-bot-implicit-mention.md`. Per TESTER.md, no other `UNTESTED-*` file was used.

**Commands run**

- `cd /Users/raro42/projects/mac-stats/src-tauri && cargo check` → **pass** (dev profile, 0 errors).
- `cargo test outbound_attachment_path_allowlist -- --nocapture` → **pass** (`discord::tests::outbound_attachment_path_allowlist`).
- `rg -n 'discord_mentions_bot_effective|mentions_bot_effective' src-tauri/src/discord/mod.rs` → lines 1852, 1956, 2016, 2796–2797, 2823.
- `rg -n 'MentionOnly activation via message reference|could not resolve referenced message for implicit mention' src-tauri/src/discord/mod.rs` → lines 1867, 1888, 1901, 1915; `debug!` uses `target: "mac_stats::discord"`.

**Acceptance criteria (code + preflight)**

1. **PASS** — `discord_mentions_bot_effective`: `referenced_message`, cache, `get_message` fallback, failure log string.
2. **PASS** — MentionOnly gate uses `!mentions_bot_effective` at ≈2823.
3. **PASS** — Observability strings + `mac_stats::discord` target.
4. **PASS** — `cargo check` succeeds.

**Manual Discord E2E** (task steps 1–8: live `mention_only` reply without @, plain message, `debug.log` grep): **not executed** in this environment.

**Outcome rename:** **CLOSED-** retained (preflight + numbered criteria pass). No rename to `TESTED-` (would apply only on implementation/preflight failure per operator convention).
