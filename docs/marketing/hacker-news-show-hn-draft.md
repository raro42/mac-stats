# Show HN draft — mac-stats (2026-08-10)

HN does not want “another Stats clone.” Stats already owns that story.
Lead with a clear difference: Apple Silicon menu-bar monitor + **optional** local agent, AI off by default.

## Best angle (recommended)

**Monitor first. Agent optional.** Technical readers install for CPU/SSD glance + low idle cost. They stay for Disk Cleanup, themes, and opt-in Ollama/Discord.

## Title options (pick one)

1. **Show HN: mac-stats – Apple Silicon menu-bar monitor with optional local AI (MIT)**
2. **Show HN: mac-stats – Rust/Tauri system monitor; Ollama agent is opt-in, off by default**
3. **Show HN: I put a local LLM agent inside a menu-bar Mac monitor (and you can leave it off)**

Prefer **1** for breadth. Prefer **3** if you want debate / curiosity.

## Body (copy-ready)

Hi HN —

I built **mac-stats**, a free MIT menu-bar system monitor for **Apple Silicon**.

Core idea: glanceable CPU / SSD / temp in the menu bar, a glass window with themes and process list, plus Disk Cleanup and website monitors — without cloud telemetry.

What makes it different from Stats / iStat Menus / MenuMeters:

- **Optional local AI agent** (Ollama) — chat about machine state, schedules, Discord bot. **Off by default.** Most people can use it as a plain monitor.
- **Apple Silicon only** (arm64). Frequency via IOReport; temp via SMC where available. No fake `hw.cpufrequency` claims.
- **Rust + Tauri.** Idle target around **~0.5% CPU** with the menu bar only (window closed).
- **Disk Cleanup** with scoped reclaim and soft-delete to Trash by default.
- Install: Homebrew cask or DMG. Config lives under `~/.mac-stats/`.

Repo: https://github.com/raro42/mac-stats  
Install: `brew tap raro42/mac-stats https://github.com/raro42/mac-stats && brew install --cask mac-stats`

Honest gaps vs Stats / iStat: we are leaner on deep sensors, history graphs, network widgets, and weather. If you need every probe, keep those tools. If you want essentials + local agent workflows in one binary, this may fit.

Happy to answer questions about idle CPU tricks (lazy CPU window, selective SMC/IOReport), the agent tool loop, or notarization / Homebrew packaging.

## Alternate body — “engineering story” (if title 3)

I run a Mac that does Discord, schedules, website checks, and local Ollama tools. Those only work while a process is alive — so I put the agent **inside** the system monitor I already want in the menu bar.

mac-stats is that binary:

1. Menu-bar metrics (always).
2. Optional agent (Ollama / Discord / scheduler) when `aiAgentEnabled: true`.

Same process, MIT, no cloud for core metrics. Design constraint: the monitor must stay cheap when the agent is off and the CPU window is closed (~0.5% idle target).

https://github.com/raro42/mac-stats

## What to avoid on HN

- “Best Stats alternative” as the headline (invites pile-on).
- Leading with Discord / Redmine / agent ops (too niche; bury as “also”).
- AI hype without “off by default.”
- Fake benchmarks. Quote measured idle only if you can defend the script (`scripts/measure_performance.sh`).
- Walls of feature lists. Three bullets + honest gaps beat ten checkmarks.

## Timing / posting tips

- Use **Show HN** prefix.
- Submit **GitHub repo** (or landing `docs/site/index.html` if polished); put brew one-liner in the text.
- Post when you can reply for a few hours (US morning often works).
- First comment can add a screenshot album + short demo video link (`screens/mac-stats-features.mp4`).
- Expect “why not Stats?” — answer with Disk Cleanup + optional local agent + Apple Silicon focus + MIT, and concede sensor depth.

## One-line pitch (tweet / About)

Apple Silicon menu-bar monitor (MIT) with an optional local Ollama agent — AI off by default.
