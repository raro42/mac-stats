# Morning surprise — 2026-11-15

Overnight Track B kept shipping. Digester open stayed empty on this rack; design-review stayed due for stale Agent Ops / processes / cpu-metrics screens. Mac Screen Recording TCC still blocks `screencapture -l` on the Linux rack, so polish shipped as polish-without-capture. **Layout-daily** already marked Data Poster + Dark earlier; Neon / Futuristic wait for a later calendar day.

## Shipped tonight (keep)

| Version | What |
|---|---|
| **v0.1.2769** | Neon / Futuristic design-review Agent Ops AI-off glyphs: `.icon-line-item.is-ai-off` resting/hover type drops extra 55% dim on `#08120c` / `#111123` → match Dark AI-off / resting icon-line 45%/85% (Dark remap was 2767; TCC polish-without-capture). |
| **v0.1.2768** | Dark / Neon / Futuristic / Data Poster Top Processes pinned pin: Apple `#0a84ff` → theme accent (parity with `.process-row.is-pinned`). |
| **v0.1.2767** | Dark design-review Agent Ops AI-off glyphs: `.icon-line-item.is-ai-off` resting/hover type drops extra 55% dim (~`#424244` on `#0a0a0a`) → match resting icon-line 45%/85% (border remap was 2760; TCC polish-without-capture). |
| **v0.1.2766** | Dark / Neon / Futuristic Top Processes name/detail focus: Apple `#0a84ff` → theme accent outlines (parity with `.process-row:focus-visible`). |
| **v0.1.2765** | Dark design-review CPU metric hints: `:root` `--hint` `#006600` → readable Matrix muted `#008800` (match `--muted` / Neon metric-hint; TCC polish-without-capture). |
| **v0.1.2764** | Dark design-review Top Processes: `.process-row` border muddy green 10% → Matrix green 18% (`--hairline` parity; TCC polish-without-capture). |
| **v0.1.2763** | Dark design-review CPU metric rings: `--ring-inactive` / `.ring-track` `#003300` → Matrix green 18% field mix (Neon ring-track parity). |
| **v0.1.2762** | Dark Agent Ops chrome tokens: `:root` `--hairline` / `--panel-border` / `--border` `#004400` → Matrix green field mixes 18%/22%/22%. |
| **v0.1.2761** | Dark Top Processes Force Quit: `.force-quit-section` border → Matrix green 28% hairline. |
| **v0.1.2760** | Dark Agent Ops AI-off chip: `.icon-line-item.is-ai-off` borders → Matrix green hairlines 28%/40%. |
| **v0.1.2759** | Dark AI Chat `#chat-input` resting border → Matrix green hairline 28%. |
| **v0.1.2758** | Dark icon-line (Agent Ops + toolbar): borders → Matrix green hairlines 28%/40%. |
| **v0.1.2757** | Dark Top Processes `.process-bar`: near-invisible `#003300` → Matrix green 12% tint. |
| **v0.1.2756** | Dark layout-daily: `.setting-toggle` resting track visible green/white hairline. |
| **v0.1.2755** | Data Poster layout-daily: base `.lpm-toggle` Apple `#ececf1` → poster shell `#0b0b10`. |
| earlier 20:00–23:45 | Cream opaque-wash ladder — see `results.tsv` / `loop_backlog.md`. |

## Tried / discarded

- Parallel-race discards when origin already held the local pick (including local `--hint` as **v0.1.2764** lost to origin process-row **v0.1.2764**; re-picked as **v0.1.2765**; local AI-off glyph as **v0.1.2766** lost to origin process-focus **v0.1.2766**; re-picked as **v0.1.2767**).
- Mac capture for feature screens still blocked (TCC / Linux rack) — polish-without-capture used instead.

## Why it matters

Dark Matrix chrome, gauge wells, process-row dividers, metric hint captions, and AI-off toolbar glyphs stop vanishing into `#0a0a0a`. Neon / Futuristic AI-off chips now match that readable glyph strength. Token remaps fix the glass shell; ring-track / process-row / hint / AI-off glyph remaps make gauges and Agent Ops chrome readable again.

## Next fuel

- Neon / Futuristic **layout-daily** on later calendar days.
- Mac capture for `feature-cpu-metrics.png` / `feature-processes.png` / `feature-agent-ops.png` when TCC allows.
- Leftover soft-states (Disk Cleanup `#004400` hairline fallbacks · Data Poster AI-off resting double-mix if muddy) · Send white-on-blue intentional — leave alone.

## Nightly minimum

Satisfied: multiple `keep` rows in `~/.mac-stats/improvements/autoresearch/results.tsv` for the 20:00–06:00 window (including **v0.1.2769**).
