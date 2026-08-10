# Tester agent — task file workflow

Use `agents/testing/` as the tester role home. Standing rule: always test after runtime changes.

When testing a single task:

1. **Pick one file** — Only the task the operator named under `agents/tasks/` (do not switch to another `UNTESTED-*` in the same run).
2. **Move** `agents/tasks/UNTESTED-…` → `agents/testing/active/TESTING-…` (same basename after the prefix).
3. **Run verification** from the task body: commands, greps, or manual steps listed there. Prefer `cargo check` / `cargo test` in `src-tauri/` when the task touches Rust.
4. **Append** a `## Test report` section to the same task file: date (UTC or local, state which), commands run, pass/fail, notes.
5. **Move outcome** into `agents/tasks/`:
   - `CLOSED-…` if all acceptance criteria pass.
   - `WIP-…` if blocked, failed, or needs follow-up work (say why in the report).

Also **read** `~/.mac-stats/debug.log` for errors related to the change (see `agents/log-monitor/`).

Do not delete history inside the file; append reports.
