# Ori Mnemos MCP with mac-stats

[Ori Mnemos](https://github.com/aayoawoyemi/Ori-Mnemos) (Recursive Memory Harness) is a **local-first** memory system: markdown vault, SQLite embeddings, wiki-link graph, and an **MCP server** that exposes tools such as `ori_orient`, `ori_query_ranked`, `ori_add`, `ori_promote`, and others. This guide explains how to use Ori **inside mac-stats** (menu bar app + Ollama + Discord / CPU chat / scheduler) using the same MCP path as any other server.

mac-stats **does not embed** Ori; it talks to Ori’s MCP process via **stdio**, exactly like the Airbnb `npx` example in [010_mcp_agent.md](010_mcp_agent.md).

---

## mac-stats vs Cursor

`ori bridge cursor --vault ~/brain` writes **Cursor IDE** `.cursor/mcp.json`. That **does not** configure mac-stats.

For mac-stats, set **`MCP_SERVER_STDIO`** or **`MCP_SERVER_URL`** in environment or in **`~/.mac-stats/.config.env`** (also searched: cwd `.config.env`, `src-tauri/.config.env`). Restart the app after changes.

---

## Single MCP server (important)

mac-stats supports **one** MCP configuration at a time: either **stdio** (`MCP_SERVER_STDIO`) or **HTTP/SSE** (`MCP_SERVER_URL`), whichever is found first per [mcp/mod.rs](../src-tauri/src/mcp/mod.rs) (`get_mcp_server_url`).

- Setting Ori as MCP **replaces** any previous `MCP_SERVER_STDIO` / `MCP_SERVER_URL` value for that install.
- **Workarounds:** run an external MCP **multiplexer** that exposes multiple backends on one URL; or **swap** `.config.env` when switching between servers; or use Ori from Cursor only for heavy memory work while mac-stats uses another MCP or none.

---

## Stdio behaviour (performance)

For **stdio** servers, mac-stats **starts a new subprocess** for each JSON-RPC round trip: **`tools/list`** (when building agent descriptions) and **each** `MCP:` tool call (`initialize` + one method, then exit). There is **no** persistent stdio connection yet.

- Large vaults or slow indexing can make **`tools/list`** or **`tools/call`** noticeably slow.
- If latency is unacceptable, consider vault maintenance (`ori index` per Ori docs), a smaller vault, or using Ori from Cursor until a pooling FEAT lands.

### Timeouts (current code)

See `src-tauri/src/mcp/mod.rs`: HTTP paths use **10s** / **15s** RPC waits; stdio uses **15s** for initialize read, **20s** per method read, and **30s** client timeout for HTTP client build paths. Long-running Ori tools (e.g. heavy **explore**) may hit timeouts in Discord; prefer narrower queries or increase limits only via a dedicated follow-up change.

---

## Prerequisites

1. **Node.js / npm** (if you install Ori via npm).
2. Install the **`ori-memory`** package globally (distribution name per [Ori Mnemos](https://github.com/aayoawoyemi/Ori-Mnemos); the CLI is typically invoked as **`ori`**):
   ```bash
   npm install -g ori-memory
   ori --version
   ```
3. **One-time vault scaffold** (example path; use your own absolute path):
   ```bash
   ori init /absolute/path/to/my-vault
   ```
4. Confirm MCP serve flags with your installed CLI (flags can change between releases):
   ```bash
   ori serve --help
   ```
   You want the mode that runs the **MCP server on stdio** (often `--mcp`).

---

## mac-stats configuration (`MCP_SERVER_STDIO`)

Use **pipe-separated** tokens: `command|arg1|arg2|...` — **no spaces around `|`** (same rules as the Airbnb example in [010_mcp_agent.md](010_mcp_agent.md)).

**Example** (verify `serve` / `--mcp` / `--vault` against `ori serve --help` on your machine):

```bash
# In ~/.mac-stats/.config.env
MCP_SERVER_STDIO=ori|serve|--mcp|--vault|/absolute/path/to/my-vault
```

If `ori` is not on PATH when mac-stats launches (e.g. GUI app), use the **full path** to the `ori` executable instead of `ori`.

**Alternate keys in `.config.env`:** `MCP-SERVER-STDIO=` is also read (see `mcp/mod.rs`).

Restart mac-stats after editing `.config.env`.

---

## Memory doctrine: Ori vs mac-stats markdown memory

Built-in memory files (`~/.mac-stats/agents/memory.md`, per-channel `memory-discord-*.md`, `memory-main.md`) are loaded into the system prompt and appended via **`MEMORY_APPEND`**; compaction can append lesson lines there. An Ori **vault is a separate directory tree** — mac-stats does **not** sync between them automatically.

| Doctrine | When to use | mac-stats `memory.md` / `MEMORY_APPEND` | Ori vault |
|----------|-------------|----------------------------------------|-----------|
| **A — Dual (default)** | General use | Quick bullets, compaction lessons, channel habits | Durable graph, linked notes, multi-hop retrieval |
| **B — Ori-primary** | Heavy knowledge work | Minimal or policy-disabled | Canonical long-term memory |
| **C — Markdown-primary** | No Node / simple install | Canonical | Optional / power-user |

**Compaction lessons:** By default, treat compaction output as **markdown-only** unless you explicitly want duplication. If you need the same lesson in Ori, add it with **`ori_add`** (or equivalent) in a separate step; automation is out of scope for this doc (see lifecycle FEAT in reviewer tasks).

---

## Model guidance (Ollama + `MCP:` lines)

- Call **`ori_orient`** at the **start of substantive work**, not on every one-liner.
- Prefer **`ori_query_ranked`** / **`ori_explore`** for multi-hop factual recall instead of guessing.
- Use **`ori_add`** / **`ori_promote`** for **durable** vault knowledge; use **`MEMORY_APPEND`** for short mac-stats lesson bullets — or adopt **B — Ori-primary** and keep markdown minimal.
- **JSON arguments:** mac-stats passes JSON to MCP when the model output after the tool name **starts with `{`**; otherwise it wraps as `{"input": "<rest>"}`. Tools that expect **structured parameters** need **valid JSON** on the `MCP:` line, e.g. `MCP: ori_query_ranked {"query":"..."}` (exact schema from live `tools/list`).

Optional **soul snippet** (paste into `~/.mac-stats/agents/soul.md` or a skill if you use Ori heavily):

```markdown
## Ori (MCP memory vault)
When MCP Ori tools are available: start substantive tasks with `ori_orient` if context is unclear. Prefer `ori_query_ranked` or `ori_explore` before inventing facts. For long-term vault knowledge use `ori_add` / `ori_promote`; use `MEMORY_APPEND` only for short mac-stats session bullets unless we agreed Ori-primary mode. Structured Ori tools need valid JSON after the tool name on the `MCP:` line.
```

---

## Security and backup

- The vault may contain **sensitive** notes; restrict filesystem permissions on the vault directory.
- **Untrusted channels** (public Discord, etc.) plus **write-capable** Ori tools create **poisoning / exfiltration** risk; treat tool availability as a trust decision.
- **Backup:** use **git** (with a remote) and/or **Time Machine** on the vault directory; SQLite / index files live alongside markdown — include them in backups or plan `ori index` rebuild per Ori docs.

---

## Sample `ori.config.yaml` fragment (local-only bias)

Ori’s config format is defined upstream. For **privacy-first** setups, prefer **disabling optional cloud LLM** features used for classification/promote if your build supports it — consult [Ori Mnemos](https://github.com/aayoawoyemi/Ori-Mnemos) for exact keys. Keep API keys out of mac-stats logs and Discord-visible errors.

---

## Troubleshooting

| Symptom | Likely cause | What to try |
|--------|----------------|-------------|
| MCP tools missing from agent list | `tools/list` failed or MCP not configured | Check `~/.mac-stats/debug.log` for `MCP list_tools failed`; verify `.config.env` and restart. |
| “MCP stdio spawn: …” / no such file | `ori` not on PATH for GUI app | Use full path to `ori` in `MCP_SERVER_STDIO`; test in Terminal: `which ori`. |
| Wrong vault / empty results | Wrong `--vault` path | Use absolute path; no spaces inside pipe tokens. |
| Timeouts | Large vault or slow tool | Narrow query; run `ori index`; avoid very long explore in Discord. |
| Spurious JSON errors | Model sent plain text for a structured tool | Instruct model to output valid JSON object after tool name. |
| Spaces around `\|` in stdio line | Parsing splits wrong | Use `cmd|a|b` with **no** spaces around pipes. |

---

## Benchmark checklist (operators / devs)

Record **p50 / p95** (wall clock) for:

1. **First message** of a session (includes `tools/list` + 0–2 typical Ori calls).
2. A message with **three sequential** Ori MCP calls.

Run on an **empty** vault and on a **large** vault; note model and machine. Paste results into your runbook or issue tracker.

**mac-stats maintainer note:** As of this doc, **no fixed p95 numbers** are checked in CI; treat the table above as the reporting template. If p95 is too high, prefer vault maintenance, smaller active vault, or Ori-in-Cursor until stdio pooling exists.

---

## Smoke tests (manual)

1. **Positive:** With Ori configured, send one Discord or CPU message that triggers **`ori_orient`** or a lightweight status-style tool from your server’s `tools/list`; confirm success in logs and a sensible reply.
2. **Negative:** Temporarily remove `ori` from PATH (or break the command) → expect a **clear** error, **no panic**.
3. **JSON tool:** Call one tool that **requires** JSON arguments (from live `tools/list`) via `MCP: ...` and confirm the server accepts the call.

---

## See also

- [010_mcp_agent.md](010_mcp_agent.md) — MCP setup, Airbnb example, error table.
- [100_all_agents.md](100_all_agents.md) — full tool list including **MCP**.
- [019_agent_session_and_memory.md](019_agent_session_and_memory.md) — session and markdown memory.
