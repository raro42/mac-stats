# Morning surprise — 2026-11-15

Overnight Track B kept shipping. Digester opened **design-review** for stale `feature-processes.png` (~93d); `feature-cpu-metrics.png` also stale (~69d). Mac Screen Recording TCC still blocks `screencapture -l`, so polish shipped as polish-without-capture. **Layout-daily** took Data Poster + Dark earlier; Neon / Futuristic wait for a later calendar day.

## Shipped tonight (keep)

| Version | What |
|---|---|
| **v0.1.2764** | Dark design-review Top Processes: `.process-row` border muddy green 10% → Matrix green 18% (`--hairline` parity; TCC polish-without-capture). |
| **v0.1.2763** | Dark design-review CPU metric rings: `--ring-inactive` / `.ring-track` `#003300` → Matrix green 18% field mix (Neon ring-track parity; TCC polish-without-capture). |
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

- Parallel-race discards when origin already held the local pick (including a token-remap race against origin **v0.1.2762**). Re-picked next leftover each time.
- Mac capture for feature screens still blocked (TCC) — polish-without-capture used instead.

## Why it matters

Dark Matrix chrome, gauge wells, and Top Processes row dividers stop vanishing into `#0a0a0a`. Token remaps fix the glass shell; ring-track and process-row remaps make gauges and the process list readable again.

## Next fuel

- Neon / Futuristic **layout-daily** on later calendar days.
- Mac capture for `feature-cpu-metrics.png` / `feature-processes.png` when TCC allows.
- Leftover dark soft-states (`--hint` `#006600` if muddy) · Send white-on-blue intentional — leave alone.

## Nightly minimum

Satisfied: multiple `keep` rows in `~/.mac-stats/improvements/autoresearch/results.tsv` for the 20:00–06:00 window (including **v0.1.2764**).
