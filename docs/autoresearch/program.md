# mac-stats overnight autoresearch (Track B — product code)

Karpathy-style ratchet for **mac-stats** (Rust/JS), not LLM `train.py`.
You program research by editing **this file**. The agent executes the loop; humans own the strategy.

Inspired by [karpathy/autoresearch](https://github.com/karpathy/autoresearch) and mac-stats-reviewer Track A (prompt MD). This track improves **shipped product behavior**.

## Goal (non-negotiable)

**Every overnight window must strengthen the product.** Quiet ticks are failure mode, not success.
A night that only appends “Quiet tick” to the backlog is a **loss** — same as shipping nothing while burning agent CapEx.

## Window

- **Active:** 20:00–06:00 local only.
- **Harness spawn:** `scripts/run_overnight_harness_loop.py` must **run** the Cursor `agent` CLI each tick (not only print `AGENT_LOOP_TICK_harness`). Print-only ticks are a quiet failure mode.
- **Quiet daytime:** do not notify or ship during focus hours unless the human explicitly asks.
- **NEVER STOP** inside the overnight window once a tick has started — finish the current experiment (keep or discard), then wait for the next harness tick.
- **No dirty leftovers:** when an experiment finishes, **commit + push** immediately. Around **23:00** local the harness loop runs `scripts/overnight_git_flush.py` once as a backstop (see `.cursor/rules/no-uncommitted-leftovers.mdc`).
- **Morning surprise (mandatory):** Before ~05:50 write/refresh `~/.mac-stats/improvements/morning_surprise_YYYY-MM-DD.md` with what **shipped** (or what was tried + discarded). “Digester empty / stayed on version X” alone is **not** a surprise — if that would be the whole note, you failed the night and must still have attempted a standing-backlog experiment.

## Nightly minimum (keep or discard)

Inside each 20:00–06:00 window:

1. **At least one** real experiment must land a row in `~/.mac-stats/improvements/autoresearch/results.tsv` (`keep` or `discard`).
2. Meta-only digester filter tweaks **do not** satisfy the nightly minimum by themselves — pair with a user-facing change, or treat filter-only as discard fuel and pick a product experiment next.
3. Individual ticks may prepare (digest, sibling scan, screenshot). The **night** must move the ratchet.
4. Cap: at most **one** quiet tick per night (backlog one-liner only). After that, pick from standing backlog / design review / Slowest even if digester “open” is empty.

## Immutable vs editable

**Do not modify (trust boundary):**

- `scripts/autoresearch_ratchet.py` — keep/discard gate + `results.tsv` logger
- Evaluation / safety: do not weaken `cargo check`, strip tests to fake green, or disable the ratchet
- Secrets / `.env` / Keychain material
- Force-push to `main` / `master`

**Prefer single-surface edits per experiment** (one coherent fix): one module family, one digester candidate, one design-review polish, or one sibling-harness port.

## Metric (binary keep/discard)

Primary gate — must pass:

```bash
python3 scripts/autoresearch_ratchet.py verify
```

That runs `cargo check` in `src-tauri/` plus a focused `cargo test --lib` smoke (override with `--test-filter`).

**Keep** only if:

1. `verify` exits 0, and
2. The change addresses a **real** candidate (see Idea priority), and
3. You can state the fitness in one line (what got better for the user).

**Discard** (revert) if verify fails, the change is speculative filler, or you cannot name the improvement.

```bash
# After noting START_SHA at experiment start:
python3 scripts/autoresearch_ratchet.py discard --start-sha "$START_SHA" --description "…"
# On success (after commit):
python3 scripts/autoresearch_ratchet.py keep --description "…"
```

Log path (untracked): `~/.mac-stats/improvements/autoresearch/results.tsv`

## Experiment loop (each harness tick)

1. Read this `program.md`, `docs/autoresearch/standing_backlog.md`, and `~/.mac-stats/improvements/{latest,sibling_harness,loop_backlog,standing_backlog}.md`.
2. Run `python3 scripts/digest_agent_runs.py` and `python3 scripts/watch_sibling_harnesses.py`.
3. Run `python3 scripts/overnight_design_review.py` — if **due**, prefer a design-review experiment this tick (screenshot + one visible polish).
4. Pick fuel in order (Idea priority). **Do not default to quiet** when standing backlog or design review has work.
5. Record `START_SHA=$(git rev-parse HEAD)`. Implement the smallest change that could fix it. Sync frontend with `./scripts/sync-dist.sh` when UI changes.
6. `python3 scripts/autoresearch_ratchet.py verify` (add `--test-filter <name>` when you know the module).
7. **Fail →** `discard --start-sha …`. Log and stop for this tick.
8. **Pass →** commit (no agent attribution), `keep --description …`, bump patch in `src-tauri/Cargo.toml` + user-facing `CHANGELOG.md` when behavior ships, install/restart when runtime changes, **`git push origin HEAD`** (do not leave the tree dirty).
9. Update `loop_backlog.md` with keep/discard outcome. Refresh morning surprise when something meaningful kept (and always before ~05:50).

## Idea priority

1. Digester **open** candidates (Slowest / Improve-task thrash / latency / errors — not already filtered noise)
2. **Overnight design review** when `overnight_design_review.py` says due (screenshot + one nicer view/function)
3. Standing backlog top item (`docs/autoresearch/standing_backlog.md` + `~/.mac-stats/improvements/standing_backlog.md`)
4. User-facing correctness (memory verbatim notes, menu bar metrics, Discord reliability)
5. Sibling harness ports (OpenClaw / Hermes) that clearly fit mac-stats
6. Prompt/tool guidance that prevents known failure modes
7. Skip as the night’s only win: docs-only churn, repeated clippy nits, version bumps without a fix, digester-filter-only meta

## Design review track

Periodic visual + UX ratchet (complements Wednesday `ui-weekly-review`, does **not** wait for it):

- Policy: `docs/043_overnight_design_review.md`
- Skill: `docs/skills/overnight-design-review.md`
- Helper: `python3 scripts/overnight_design_review.py` (which surface is stale; suggested screenshot target)
- One night = **one** surface: capture window-only screenshot → polish CSS/layout/copy → sync-dist → ship

## Simplicity criterion

Same as Karpathy: a tiny gain that adds ugly complexity → discard. Deleting code with equal/better verify → keep.

## Output expectations

- One experiment per tick max; ≥1 keep-or-discard per overnight window.
- `results.tsv` rows for every keep/discard/crash.
- Do not ask the human whether to continue overnight.
