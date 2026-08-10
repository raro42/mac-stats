# Show HN — preferred draft (2026-08-10)

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

Fresh measure tonight (v0.1.367, same top interval sampler): mac-stats idle ~0.17% CPU avg (window open ~0.20%). Same machine, exelban Stats menu-bar idle ~3.4% avg (2.4–6.9%). Built because my old monitor cost more CPU than half my actual work.

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

## Optional first comment

```text
Screens: https://github.com/raro42/mac-stats/tree/main/screens
Measure script: scripts/measure_performance.sh — happy to post before/after if useful.
AI stays off until Settings / aiAgentEnabled.
```
