# Closing reviewer (mac-stats)

Use this file as the **entry point** when an agent is told to run the closing reviewer.

## What to do

1. Open **`docs/022_feature_review_plan.md`**:
   - **Section 9 — Closing review (2026-03-08)** — integration checklist (handler registration, monitor wiring, frontend sync when JS changes, etc.).
   - Under **Open tasks**, find the **latest** `### Closing reviewer smoke test …` block and match or exceed that bar: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`, and optional short `./target/release/mac_stats -vv` smoke (monitors/agents init in log).
2. Reconcile **`git diff`** with **`CHANGELOG.md`** (current version section) and **`agents/006-feature-coder/FEATURE-CODER.md`** for any **FEAT-D\*** items touched.
3. **Process uptime:** Do not `pkill -f mac_stats` without **immediately** starting again and verifying a PID if the operator depends on Discord/scheduler/monitors — see root **`AGENTS.md`**.

## Scope

This reviewer validates build health, aligns docs/backlog with code, and records a dated smoke block in **`docs/022_feature_review_plan.md`**. Deep feature review of historical F1–F10 remains in the same doc (sections 3–5).
