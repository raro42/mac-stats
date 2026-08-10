# Show HN — preferred draft (2026-08-10)

**Posted:** 2026-08-10 · https://news.ycombinator.com/item?id=49248085

Tone: dry, sarcastic, nerdy. Short. Honest about early stage.
Real motivation: iStat Menus eating 6–15% CPU constantly.

---

## Title

```text
Show HN: I put a local LLM agent inside a menu-bar Mac monitor (and you can leave it off)
```

**URL:** `https://github.com/raro42/mac-stats`

---

## Body (short — use this)

```text
Hi HN —

iStat Menus was sitting at 6–15% CPU all day for the privilege of showing me… CPU. That felt rude.

So I built mac-stats: an MIT menu-bar monitor for Apple Silicon that tries not to be the workload. Optional local Ollama agent in the same binary — off by default. Never enable it and it’s just a monitor.

Menu bar: CPU / SSD / °C. Window: themes, processes, site monitors, Disk Cleanup. Agent (opt-in): local chat, schedules, Discord. Rust + Tauri. No cloud telemetry. Arm64 only.

Fresh measure (v0.1.368, same top interval sampler): mac-stats idle ~0.17% CPU avg (window open ~0.20%). Same machine, exelban Stats menu-bar idle ~3.4% avg (2.4–6.9%). Built because my old monitor cost more CPU than half my actual work.

Not a full iStat/Stats replacement if you want every sensor and graph. Early (~6 GitHub stars). Numbers: https://github.com/raro42/mac-stats/blob/main/docs/ops/2026-08-10-performance-baseline.md

  brew tap raro42/mac-stats https://github.com/raro42/mac-stats
  brew install --cask mac-stats

https://github.com/raro42/mac-stats
```

---

## Why this angle works

- Specific numbers (6–15%) beat vague “I wanted something better.”
- HN loves “the tool measuring X was the hot path.”
- Still fair to iStat: deep sensors cost something; you chose lean.

---

## First comment — paste this (2026-08-10)

Post as **raro43** on https://news.ycombinator.com/item?id=49248085  
(Comment box at the bottom → **add comment**.)

```text
A few pointers if you try it:

- Screens + short demo: https://github.com/raro42/mac-stats/tree/main/docs/screens
- Idle CPU numbers (mac-stats vs Stats, same host/sampler): https://github.com/raro42/mac-stats/blob/main/docs/ops/2026-08-10-performance-baseline.md
- Measure yourself: ./scripts/measure_performance.sh 60 1 idle

Install (Apple Silicon):

  brew tap raro42/mac-stats https://github.com/raro42/mac-stats
  brew install --cask mac-stats

The local LLM / Discord / scheduler stuff stays off until you turn on Settings → AI agent. Menu-bar monitor alone is the default path.

Happy to dig into methodology or weird SMC/freq edge cases if useful.
```
