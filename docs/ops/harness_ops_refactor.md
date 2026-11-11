# harness_ops refactor (instant lane)

Date: 2026-11-11

## Why

`commands/harness_ops.rs` grew to **135k lines / 5.2MB**. Overnight shipped one
`/keep-pNN` glance per version by copy-pasting ~1k-line matchers (~98% identical).

## What changed

| Before | After |
| --- | --- |
| One 135k-line god file | `commands/harness_ops/` fragments (~24k lines) |
| 60× `/keep-pNN` clones | `parse_keep_percentile` + `format_keep_percentile_gateway` |
| Hand-rolled path/size/age matchers | `ResourceLane` table + shared `looks_like_resource_stat` |
| 5k-line digester noise laundry list | Compact `is_insights_slowest_noise` |

Fragments are `*.inc.rs` included into one `gateway` module so private helpers
stay in one scope (dense cross-calls). Edit the fragment that matches the domain.

## Layout

```text
commands/harness_ops/
  mod.rs              # re-exports + tests
  normalize.rs        # normalize_operator_command
  gateway.rs          # include! hub
  agent_ops.inc.rs    # Tauri Agent Ops + digest
  keep_stats.inc.rs   # results.tsv + /keep-p{N}
  resource_stat.inc.rs
  insights.inc.rs
  status_chips.inc.rs
  inventory.inc.rs
  router.inc.rs       # try_operator_instant_reply
```

## Rules (do not re-inflate)

1. **New keep percentiles** — use `/keep-p{N}` (1–99) via `parse_keep_percentile`.
   Do **not** add `looks_like_results_tsv_pNN_request` or
   `format_results_tsv_pNN_gateway`.
2. **New path/size/age file lanes** — add a `ResourceLane` (nouns + short phrases).
   Do **not** clone exclusion laundry lists. Sibling excludes stay string-only.
3. **Sibling path excludes** — never call other `looks_like_*` from a path matcher
   (see `2026-09-15-instant-lane-path-nesting-hang.md`).
4. **Line budget** — `python3 scripts/check_harness_ops_budget.py` must pass.

## Related

- Instant-lane path nesting hang: `docs/ops/2026-09-15-instant-lane-path-nesting-hang.md`
