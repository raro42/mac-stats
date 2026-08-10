---
name: overnight-design-review
description: Overnight Track B — screenshot one stale CPU-window surface and ship one visible polish.
---

# Overnight design review

Policy: `docs/043_overnight_design_review.md`. Helper: `python3 scripts/overnight_design_review.py`.

## Do

1. Run the helper; if `due=false` and digester has open work, prefer digester. If digester empty and standing backlog points here, proceed even when not “due” if screens are the best fuel.
2. Open the recommended surface in the **menu-bar CPU window** (not orphan dashboard).
3. Recapture `docs/screens/feature-*.png` window-only.
4. One CSS/layout/copy polish in the correct source files; `./scripts/sync-dist.sh`.
5. Ratchet verify → commit → keep → version + CHANGELOG → push when reasonable.
6. Morning surprise: name the surface + what got nicer.

## Do not

- Ship digester-only meta as the design-review keep
- Polish `dashboard.html` Settings as if it were shipped Command Center
- Ask the human whether to continue
