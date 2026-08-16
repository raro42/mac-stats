# RAM details placement (2026-08-16)

## Problem

The menu bar shows **RAM %**, but the CPU window has no matching system RAM block. Process Details already shows per-process memory only.

## Recommendation

| Surface | Role | Content |
|--------|------|---------|
| **Metrics grid (rings)** | Primary glance (same weight as menu bar) | RAM % ring next to CPU / GPU |
| **Details section** | Numbers | Used GiB, Total GiB, Percent |
| **History sparklines** | Optional later | RAM % over time (like CPU) |
| **Top Processes** | Already exists | Per-process RSS |

Prefer the **metrics grid** for glance and **Details** for exact sizes. Do not put system RAM only under Agent Ops / Knowledge (that “memory” is agent notes, not hardware).

## Shipped first

- Details section: RAM %, used, total (from `get_cpu_details`).
- Window title: `mac-stats <version>` (was `CPU`).

## Follow-up

- Add a fifth **RAM** ring in the metrics grid (all themes).
- Optional RAM history sparkline.
