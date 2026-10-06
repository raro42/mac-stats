# Session todo — FEAT-D468 virtual orbitals overflow

No GitHub `FEAT-*` / `WIP-*` under `agents/tasks/` (only `UNTESTED-14-…`). Open/deferred FEAT-D empty after D467 → add and implement **FEAT-D468**.

- [x] Add `virtual orbitals exceed` / `virtual orbital exceed` arm in `content_reduction/mod.rs` (ident-boundary + context-slot guard, parallel to occupied/canonical)
- [x] Unit tests in `content_reduction/tests.rs` (positives, HTTP/no-slot negatives, micro/meta/sub compounds)
- [x] FEATURE-CODER.md: Recently closed + When empty → D468
- [x] CHANGELOG + bump `0.1.1417`
- [x] `cargo check` + targeted `cargo test` for overflow
- [ ] Commit + push `origin/main`

## Review
Shipped **FEAT-D468** / v0.1.1417: `is_context_overflow_error` recognizes `virtual orbitals exceed` / `virtual orbital exceed` with the same context-slot + ident-boundary rules as occupied/canonical. No GitHub issue to close. `UNTESTED-14-…` left for tester.
