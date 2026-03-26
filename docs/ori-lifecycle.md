# Ori Mnemos lifecycle (optional)

mac-stats can integrate with a local [Ori Mnemos](https://github.com/aayoawoyemi/Ori-Mnemos) vault **without** relying on the model to call MCP tools. Everything is **opt-in** via environment variables; with the master flag off, behaviour matches a build without this feature (no extra processes, no extra prompt sections).

## Vault scope

- One **global** vault path (`MAC_STATS_ORI_VAULT` or `ORI_VAULT`). It must be the same root you pass to MCP (`ori serve --mcp --vault …`). Per-channel vaults are **not** implemented; in shared Discord guilds, captures and prefetch use the same brain for all channels—same caveat as the MCP bridge.

## Relationship to markdown memory (`MEMORY_APPEND`, compaction)

- Compaction still appends lesson bullets to `memory.md` / `memory-discord-*.md` as today.
- Ori capture is **separate** and policy-driven (`MAC_STATS_ORI_COMPACTION_CAPTURE_MODE`):
  - `off` — no Ori writes from compaction (default).
  - `excerpt_to_ori` — one inbox note with a capped excerpt; **24h dedupe** on identical excerpt hash per `(hook_source, session_id)` to limit vault spam.
  - `full_lessons_duplicate` — writes the full lesson text to Ori (operator opt-in duplication vs markdown files).

## Prompt section order (execution step, router soul path)

When skill/agent overlay is **not** active, dynamic sections are ordered:

1. mac-stats markdown memory (`load_memory_block_for_request`)
2. **Ori session briefing** (vault `self/` + `ops/` markdown excerpts)
3. **Possibly relevant vault notes** (`ori query similar` JSON → titles)
4. Live metrics, reminders, plan

Skill/agent overlay **omits** memory and Ori sections (unchanged historical behaviour).

## Session briefing vs MCP `ori_orient`

The packaged `ori` CLI (as of ori-memory 0.5.x) does **not** expose `ori orient`. mac-stats builds a bounded briefing from:

- `self/identity.md`, `self/goals.md`, `self/methodology.md`, `ops/daily.md`, `ops/reminders.md`

If those files are missing or empty, no briefing section is injected.

## Capture implementation

`ori add` does not accept a body on the CLI; mac-stats writes **Ori-compatible inbox markdown** (YAML frontmatter + body) under `inbox/`.

## Hook matrix

| Event | Condition | Action | Primary env flags | Module / call site |
|-------|-----------|--------|-------------------|---------------------|
| New session (empty prepared history) | Orient hook on, not automation source | Read vault excerpts → `## Ori session briefing` | `MAC_STATS_ORI_LIFECYCLE_ENABLED`, `MAC_STATS_ORI_HOOK_ORIENT`, vault path | `commands/ori_lifecycle.rs` → `commands/ollama.rs` after `prepare_conversation_history` |
| User message | Prefetch on, cooldown OK | `ori query similar` (cwd = vault) | `MAC_STATS_ORI_PREFETCH`, limits/timeouts | `commands/ori_lifecycle.rs` → `commands/ollama.rs` before execution prompt |
| Post-compaction | Success + lessons + mode ≠ off | Inbox note (background thread) | `MAC_STATS_ORI_HOOK_CAPTURE_COMPACTION`, `MAC_STATS_ORI_COMPACTION_CAPTURE_MODE` | `session_history.rs`, `compaction.rs` |
| Session reset | `clear_session` with messages | Inbox note (background thread) | `MAC_STATS_ORI_HOOK_BEFORE_RESET` | `session_memory.rs` → `ori_lifecycle` |
| Scheduler / heartbeat / task_runner | Default | Skip orient + prefetch | `MAC_STATS_ORI_ALLOW_ON_SCHEDULER=true` to opt in | `ori_skip_automation_sources` |

## Environment reference

| Variable | Purpose |
|----------|---------|
| `MAC_STATS_ORI_LIFECYCLE_ENABLED` / `ORI_LIFECYCLE_ENABLED` | Master gate |
| `MAC_STATS_ORI_VAULT` / `ORI_VAULT` | Vault root (`.ori` marker required) |
| `MAC_STATS_ORI_BINARY` | `ori` binary (default `ori`) |
| `MAC_STATS_ORI_HOOK_ORIENT` / `ORI_HOOK_ORIENT_ON_SESSION_START` | Session briefing |
| `MAC_STATS_ORI_HOOK_CAPTURE_COMPACTION` / `ORI_HOOK_CAPTURE_ON_COMPACTION` | Compaction → inbox |
| `MAC_STATS_ORI_HOOK_BEFORE_RESET` / `ORI_HOOK_BEFORE_SESSION_RESET` | Reset → inbox |
| `MAC_STATS_ORI_PREFETCH` / `ORI_PREFETCH_ENABLED` | Query similar prefetch |
| `MAC_STATS_ORI_COMPACTION_CAPTURE_MODE` / `ORI_COMPACTION_CAPTURE_MODE` | `off` / `excerpt_to_ori` / `full_lessons_duplicate` |
| `MAC_STATS_ORI_ORIENT_MAX_CHARS` | Briefing cap (default 8000) |
| `MAC_STATS_ORI_PREFETCH_MAX_CHARS` | Prefetch section cap (default 6000) |
| `MAC_STATS_ORI_PREFETCH_TOP_K` | `--limit` for similar (default 5) |
| `MAC_STATS_ORI_PREFETCH_TIMEOUT_SECS` | Subprocess timeout (default 12) |
| `MAC_STATS_ORI_PREFETCH_COOLDOWN_SECS` | Min seconds between prefetch per session (default 5) |
| `MAC_STATS_ORI_RESET_CAPTURE_MAX_CHARS` | Reset transcript cap (default 12000) |
| `MAC_STATS_ORI_ALLOW_ON_SCHEDULER` | When `true`, allow orient+prefetch for scheduler sources |

Logging: orient/prefetch **bodies** are not logged at info; use target `mac_stats::ori` and `-vv` for diagnostics.
