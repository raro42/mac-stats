# agents/testing/

Home for **verification**. Standing rule: always test after runtime changes.

## Workflow

1. Pick one file under `agents/tasks/` named `UNTESTED-…` (only that file).
2. Move/rename to `agents/testing/active/TESTING-…` (same basename after the prefix).
3. Run verification from the task body (`cargo check` / `cargo test` / listed greps).
4. Append a `## Test report` section (date, commands, pass/fail).
5. Move outcome to `agents/tasks/CLOSED-…` (pass) or `agents/tasks/WIP-…` (blocked/fail).

Do not leave `TESTING-*` in `agents/tasks/`. Do not delete history inside the file; append reports.

## Files

- [TESTER.md](TESTER.md) — tester role prompt
- [active/](active/) — in-flight `TESTING-*` only
