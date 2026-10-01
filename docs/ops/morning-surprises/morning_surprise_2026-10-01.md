# Morning surprise — 2026-10-01

Overnight Track B kept the CPU window calm before JavaScript paints.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1267** | External / Monitors summary first paint → **None yet** (not **0 / 0 sites up**). Real up counts and DOWN lines still show after a sample. All themes. |
| **v0.1.1268** | External / Monitors list first paint → **Nothing watching yet** and **Add a monitor** (not a blank list). Real site rows still replace that empty state after a load. All themes. |
| **v0.1.1269** | Top Processes list first paint → **Waiting for process samples — opens with the CPU window** (not a blank list). Real rows still replace that line after a sample. All themes. |
| **v0.1.1270** | Disk Cleanup category list first paint → **Nothing to reclaim yet** and **Review scopes** (not a blank list). Real category rows still replace that empty state after a load. All themes. |
| **v0.1.1271** | Disk Cleanup scopes list first paint → **No scopes yet** and **Add a scope** (not a blank list). Real scope rows still replace that empty state after a load. All themes. |
| **v0.1.1272** | Disk Cleanup last-run panel first paint → **Not yet this install — will run on launch** (not a blank panel). A real last run still replaces that line after a load. All themes. |
| **v0.1.1273** | Debug Log viewer first paint → **Nothing here yet — loads when you open Debug Log** (not **Expand to load log…**). A real tail still replaces that line after a load. All themes. |
| **v0.1.1274** | Perplexity results first paint → **Nothing here yet — search the web** (not a blank region). A real search still replaces that line. All themes. |
| **v0.1.1275** | AI Chat message list first paint → **Nothing here yet — set an Ollama URL** and starter chips (not a blank list). A real transcript still replaces that empty state. All themes. |
| **v0.1.1276** | GPU sparkline first paint → the history row shows **CPU · GPU · Freq · Temp** in the theme HTML. It no longer jumps from three charts to four after JavaScript loads. All themes. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~15.5d).
- Debug.log: no product ERROR/WARN/panic clusters in the scan window.
- ~20:05 tick: monitors summary first-paint keep (**v0.1.1267**); install/kickstart.
- ~20:35 tick: monitors list first-paint keep (**v0.1.1268**); install/kickstart.
- ~21:00 tick: Top Processes list first-paint keep (**v0.1.1269**); install/kickstart.
- ~21:23 tick: Disk Cleanup category list first-paint keep (**v0.1.1270**); install/kickstart.
- ~21:48 tick: Disk Cleanup scopes list first-paint keep (**v0.1.1271**); install/kickstart.
- ~22:18 tick: Disk Cleanup last-run first-paint keep (**v0.1.1272**); install/kickstart.
- ~22:43 tick: Debug Log viewer first-paint keep (**v0.1.1273**); install/kickstart.
- ~23:12 tick: Perplexity results first-paint keep (**v0.1.1274**); install/kickstart.
- ~23:35 tick: AI Chat message list first-paint keep (**v0.1.1275**); install/kickstart.
- ~00:05 tick: GPU sparkline first-paint keep (**v0.1.1276**); install/kickstart.

## Next fuel

- Agent Ops screenshot refresh when Screen Recording TCC allows (`feature-agent-ops.png`).
- Ring labels still say Frequency / Temperature until JavaScript shortens them to Freq / Temp.
- Sibling ports if digester stays empty.
