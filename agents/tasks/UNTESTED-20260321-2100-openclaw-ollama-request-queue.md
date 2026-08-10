---
## Triage summary (TOP)

- **Coder (UTC):** 2026-03-28 — Added **`clamp_ollama_global_concurrency_n`** plus unit test **`ollama_global_concurrency_clamps_to_allowed_range`** in `src-tauri/src/config/mod.rs` (automated 1..=16 clamp coverage). Queue module (`ollama_queue.rs`) and router integration unchanged; §4 checklist remains complete.
- **Next step:** **UNTESTED** — tester runs **Testing instructions** below (including new `cargo test` filter); E2E Discord / overlap / contended logs / `ollamaGlobalConcurrency`: 2 after restart still require operator or staged environment.
- **Prior run:** Test report below records **Overall FAIL** when only unit + static checks ran; do not treat that as closure.
---

# UNTESTED: Keyed request queue to serialize Ollama calls per channel

**Created (UTC):** 2026-03-21 21:00  
**Coder handoff (UTC):** 2026-03-27 17:40  
**Source:** OpenClaw vs mac-stats review  
**Topic:** Serialize concurrent Ollama requests per channel to prevent GPU saturation

---

## 1. Summary

OpenClaw serializes LLM requests per session using a keyed async queue — only one request runs per session key at a time, with others queued FIFO. mac-stats had no concurrency control: simultaneous Discord messages, scheduled tasks, and CPU-window chat all hit Ollama in parallel, saturating the GPU. A lightweight per-channel queue plus a global semaphore prevents this.

---

## 2. OpenClaw behaviour / pattern

OpenClaw's gateway tracks active "chat runs" in a map keyed by session/run ID. When a new request arrives for a session that already has an active run, the gateway returns `{ status: "in_flight" }` instead of starting a duplicate. A `KeyedAsyncQueue` ensures only one operation runs per key at a time, queueing the rest FIFO.

---

## 3. Relevance for mac-stats

mac-stats has multiple concurrent Ollama request sources: Discord messages, scheduled tasks, and the CPU-window chat UI. A per-source-key queue (`discord:<channel id>`, `scheduler`, `cpu_ui`, …) serializes requests within each source; a global semaphore limits total concurrent `/api/chat` calls (default **1**).

---

## 4. High-level instructions for coder

- [x] Module `src-tauri/src/ollama_queue.rs`: keyed FIFO + global semaphore (`Config::ollama_global_concurrency()`).
- [x] Before `answer_with_ollama_and_fetch` / `send_ollama_chat_messages_streaming`, acquire via `OllamaHttpQueue::Acquire` with the right key; release after the full turn (router uses `Nested` for inner calls; retries use `skip_ollama_queue: true` where applicable).
- [x] Discord: `ollama_queue_wait_hook` triggers `broadcast_typing` when waiting on the per-key queue.
- [x] Config: `ollamaGlobalConcurrency` / `MAC_STATS_OLLAMA_GLOBAL_CONCURRENCY`, clamped 1–16, read once on first queue use (`OnceLock`).
- [x] `debug2` logs: `blocked waiting for key slot`, `acquired key slot` (`key_wait_ms`, `per_key_waiters_ahead`), `acquired global permit`.

---

## 5. Implementation notes (reference)

- **Router:** `answer_with_ollama_and_fetch` acquires once for the full turn (`turn_body` via `with_ollama_http_queue`); recursive retries set `skip_ollama_queue: true`. Inner `/api/chat` uses `OllamaHttpQueue::Nested`.
- **Keys:** `discord:<channel_id>`, `scheduler`, `cpu_ui`, `cli`, `judge`, `agent_cli` (see call sites).
- **Docs:** `docs/021_router_and_agents.md` (Ollama `/api/chat` queue paragraph).

---

## 6. Implementation summary

- **`ollama_queue.rs`:** Per-key FIFO (`oneshot` waiters) + `Semaphore` for global concurrency.
- **Unit tests:** `cargo test ollama_http_queue` — global serialization across keys, same-key FIFO, `wait_hook` on second waiter. `cargo test ollama_global_concurrency_clamps` — config clamp 1..=16 (env/JSON values use the same helper via `Config::ollama_global_concurrency()`).
- **Logs:** At `-vv`, `ollama/queue:` lines in `~/.mac-stats/debug.log` for contention and acquisition timing.

---

## Testing instructions

*(For the testing agent: verify end-to-end behaviour after this handoff.)*

### What to verify

- **Automated:** From `~/projects/mac-stats/src-tauri`: `cargo test ollama_http_queue` **and** `cargo test ollama_global_concurrency_clamps` both pass.
- With default **`ollamaGlobalConcurrency` = 1**, parallel workloads (Discord, CPU chat, scheduler, judge) do not overlap Ollama `/api/chat` in time; no dropped user/Discord messages.
- **Per-channel:** Two rapid messages in the **same** Discord channel are answered sequentially; two **different** channels can each hold a slot only up to the global limit.
- **`ollamaGlobalConcurrency` > 1:** After **app restart**, up to that many concurrent `/api/chat` calls are allowed (singleton reads limit once per process).
- **Logs (`-vv`):** Under contention, expect `blocked waiting for key slot` with `per_key_waiters_ahead` ≥ 1, then `acquired key slot` with `key_wait_ms` > 0 when a wait occurred; global permit lines follow.
- **Discord:** When a router request waits in the keyed queue, typing indicator may pulse (optional visual check); no panic from the wait hook.

### How to test

1. **Rust:** `cd ~/projects/mac-stats/src-tauri`; run `cargo test ollama_http_queue --lib` and `cargo test ollama_global_concurrency_clamps --lib` (cargo accepts one name filter per invocation).
2. Run mac-stats with **`-vv`** (menu bar or `target/release/mac_stats -vv`).
3. Ensure `~/.mac-stats/config.json` omits `ollamaGlobalConcurrency` or sets it to **1**.
4. **Discord:** In an agent / having_fun channel, send **three** short messages as fast as possible. Confirm three replies, one after another, none missing.
5. **Overlap:** While a long Discord turn runs, trigger scheduler Ollama work or CPU-window chat; with limit **1**, the second source should not overlap the first in wall-clock (subjective or correlate with log ordering / `ollama/queue` lines).
6. **Logs:** `grep ollama/queue ~/.mac-stats/debug.log` after forcing contention; confirm `blocked waiting for key slot` and/or `key_wait_ms` > 0 on acquire lines when queuing happened.
7. **Concurrency > 1:** Set `"ollamaGlobalConcurrency": 2`, **restart** mac-stats, repeat rapid traffic from two channels (or Discord + CPU chat) and confirm up to two generations can overlap (timing or logs).

### Pass / fail criteria

- **Pass:** Both Rust filters above pass; no lost Discord messages; with default limit 1, effective global serialization of `/api/chat`; queue diagnostics visible at `-vv` when contended; no panic/deadlock; invalid `ollamaGlobalConcurrency` outside 1–16 clamped (covered by clamp unit test + `Config::ollama_global_concurrency()`).
- **Fail:** Either automated test fails; dropped messages; parallel `/api/chat` storms with limit 1; deadlock; wait-hook panic.

---

## Test report

1. **Date and time (UTC):** 2026-03-28 13:08–13:10 UTC (run window). `debug.log` excerpts below are historical lines from the host log (not generated during this narrow window); `cargo test` does not write to `~/.mac-stats/debug.log`.

2. **Environment:** mac-stats repo `~/projects/mac-stats`; tests from `~/projects/mac-stats/src-tauri`. Command: `cargo test ollama_http_queue`. Read `~/.mac-stats/config.json` and `~/.mac-stats/debug.log` for config/log checks. mac-stats GUI/Discord/Ollama overlap flows were **not** executed in this run (no live Discord session, no long-running app instance under tester control).

3. **What was tested:** `ollama_http_queue` unit test; default config (no `ollamaGlobalConcurrency`); static review of `Config::ollama_global_concurrency()` clamping; `grep` of existing `ollama/queue` and `blocked waiting` strings in `~/.mac-stats/debug.log`.

4. **Results:**
   - **Automated `cargo test ollama_http_queue`:** **PASS** — `ollama_queue::tests::ollama_http_queue_serializes_and_fires_wait_hook` ok (1 passed).
   - **Default `ollamaGlobalConcurrency` = 1 (config omits key):** **PASS** — `~/.mac-stats/config.json` has no `ollamaGlobalConcurrency` entry; implementation defaults to 1.
   - **Parallel workloads / no dropped Discord / per-channel / global serialization (E2E):** **NOT VERIFIED** — steps 4–5 of “How to test” not run (Discord + overlap).
   - **`ollamaGlobalConcurrency` > 1 after restart:** **NOT VERIFIED** — step 7 not run.
   - **Logs (`-vv`) under contention (`blocked waiting…`, `key_wait_ms` > 0):** **NOT VERIFIED** — contention not forced in this run; full-file grep for `blocked waiting for key slot` in `~/.mac-stats/debug.log` returned no matches (only uncontended `acquired key slot` samples present).
   - **Discord wait hook / typing (optional) / no panic:** **NOT VERIFIED** — no Discord exercise in this run.
   - **Invalid `ollamaGlobalConcurrency` clamped 1–16:** **PASS** — `Config::ollama_global_concurrency()` uses `.clamp(MIN_N, MAX_N)` with `MIN_N=1`, `MAX_N=16` for env and config JSON (`as_u64` cast then clamp).

5. **Overall:** **FAIL** — Criteria not met: end-to-end Discord, overlap, live log contention, and concurrency>1 restart were not verified. Automated unit test and config clamp review passed.

6. **Product owner feedback:** The Rust unit test confirms global serialization across keys, same-key FIFO ordering, and the wait hook in isolation. Operator-facing checks (Discord triple-message, cross-source overlap, forced queue contention in `debug.log`, and `ollamaGlobalConcurrency`: 2 after restart) still need a human or staged run with Discord and Ollama. This task was renamed to **`TESTED-`** (verification complete for this agent run) so the **Closing Reviewer** can triage — typically **`TESTED-` → `WIP-`** if you want the coder to add lighter automated coverage or to re-queue after manual sign-off.

7. **URLs tested (browser flows only):** N/A — no browser

8. **Workflow:** untested → testing → tested (fail).

### `debug.log` (relevant lines) — last section

Existing host log lines showing `ollama/queue:` acquisition format (no `blocked waiting` lines matched in this file):

```
{"location":"ollama_queue.rs","message":"ollama/queue: acquired key slot key=discord:1474755809723285626 per_key_waiters_ahead=0 key_wait_ms=0 global_available_permits=1","data":{},"timestamp":1774692160136,"sessionId":"debug-session","runId":"run3","hypothesisId":""}
{"location":"ollama_queue.rs","message":"ollama/queue: acquired key slot key=cli per_key_waiters_ahead=0 key_wait_ms=0 global_available_permits=1","data":{},"timestamp":1774701635674,"sessionId":"debug-session","runId":"run3","hypothesisId":""}
{"location":"ollama_queue.rs","message":"ollama/queue: acquired global permit key=cli global_available_permits_after=0","data":{},"timestamp":1774701639550,"sessionId":"debug-session","runId":"run3","hypothesisId":""}
```
