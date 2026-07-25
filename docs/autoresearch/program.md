# mac-stats overnight autoresearch (Track B — product code)

Karpathy-style ratchet for **mac-stats** (Rust/JS), not LLM `train.py`.
You program research by editing **this file**. The agent executes the loop; humans own the strategy.

Inspired by [karpathy/autoresearch](https://github.com/karpathy/autoresearch) and mac-stats-reviewer Track A (prompt MD). This track improves **shipped product behavior**.

## Window

- **Active:** 20:00–06:00 local only.
- **Quiet:** do not notify or ship during daytime focus hours unless the human explicitly restarts a day loop.
- **NEVER STOP** inside the overnight window once a tick has started — finish the current experiment (keep or discard), then wait for the next harness tick.

## Immutable vs editable

**Do not modify (trust boundary):**

- `scripts/autoresearch_ratchet.py` — keep/discard gate + `results.tsv` logger
- Evaluation / safety: do not weaken `cargo check`, strip tests to fake green, or disable the ratchet
- Secrets / `.env` / Keychain material
- Force-push to `main` / `master`

**Prefer single-surface edits per experiment** (one coherent fix): one module family, one digester candidate, or one sibling-harness port. Avoid drive-by refactors and filler clippy/version churn when digester is empty.

## Metric (binary keep/discard)

Primary gate — must pass:

```bash
python3 scripts/autoresearch_ratchet.py verify
```

That runs `cargo check` in `src-tauri/` plus a focused `cargo test --lib` smoke (override with `--test-filter`).

**Keep** only if:

1. `verify` exits 0, and
2. The change addresses a **real** candidate (digester open item, sibling harness port, clear bug from `debug.log`, or user-facing gap like memory/menu-bar), and
3. You can state the fitness in one line (what got better).

**Discard** (revert) if verify fails, the change is speculative filler, or you cannot name the improvement.

```bash
# After noting START_SHA at experiment start:
python3 scripts/autoresearch_ratchet.py discard --start-sha "$START_SHA" --description "…"
# On success (after commit):
python3 scripts/autoresearch_ratchet.py keep --description "…"
```

Log path (untracked): `~/.mac-stats/improvements/autoresearch/results.tsv`

## Experiment loop (each harness tick)

1. Read this `program.md` and `~/.mac-stats/improvements/{latest,sibling_harness,loop_backlog}.md`.
2. Run `python3 scripts/digest_agent_runs.py` and `python3 scripts/watch_sibling_harnesses.py`.
3. If **no open digester candidates** and no clear high-impact bug/sibling port: **quiet tick** — append a one-liner to `loop_backlog.md`, do **not** invent filler ships.
4. Else pick **ONE** experiment. Record `START_SHA=$(git rev-parse HEAD)`.
5. Implement the smallest change that could fix it. Sync frontend with `./scripts/sync-dist.sh` when UI changes.
6. `python3 scripts/autoresearch_ratchet.py verify` (add `--test-filter <name>` when you know the module).
7. **Fail →** `discard --start-sha …` (hard reset working tree to start). Log and stop for this tick.
8. **Pass →** commit (no agent attribution), `keep --description …`, bump patch in `src-tauri/Cargo.toml` + user-facing `CHANGELOG.md` when behavior ships, install/restart when runtime changes (`install-to-applications.sh` + LaunchAgent kickstart + verify Discord). Push when reasonable.
9. Update `loop_backlog.md` with keep/discard outcome. Leave a morning surprise note when something meaningful kept.

## Idea priority

1. Digester **open** Slowest / Latency / error candidates (not already filtered noise)
2. User-facing correctness (memory verbatim notes, menu bar metrics, Discord reliability)
3. Sibling harness ports (OpenClaw / Hermes) that clearly fit mac-stats
4. Prompt/tool guidance that prevents known failure modes
5. Skip: docs-only churn, repeated clippy nits, version bumps without a fix

## Simplicity criterion

Same as Karpathy: a tiny gain that adds ugly complexity → discard. Deleting code with equal/better verify → keep.

## Output expectations

- One experiment per tick max.
- `results.tsv` rows for every keep/discard/crash.
- Do not ask the human whether to continue overnight.
