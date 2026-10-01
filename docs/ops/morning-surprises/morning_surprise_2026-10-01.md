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
| **v0.1.1277** | Ring labels first paint → **Freq** and **Temp** (same words as the sparklines). They no longer flash **Frequency** and **Temperature** until JavaScript loads. Hover still shows the full words. All themes. |
| **v0.1.1278** | Low Power Mode first paint → the power strip shows **Low Power Mode (LPM)** and **None yet** in the theme HTML. The chip no longer pops in after JavaScript loads. On and Off still replace **None yet** after a sample. All themes. |
| **v0.1.1279** | Apple CPU and Freq rings → the fill follows the same arc as the track (and as GPU and Temp). It no longer rides a shorter curve that misses the gauge. |
| **v0.1.1280** | Agent Ops overview first paint → **Agents**, **Runs**, and **Digest** cards with **None yet** are in the theme HTML. The grid no longer jumps from four cards to seven after JavaScript loads. Live counts still replace **None yet**. All themes. |
| **v0.1.1281** | Agent Ops tab strip first paint → **Overview** (press 0) and digit keys **1–5** are in the theme HTML. The strip no longer grows after JavaScript loads. Press 0 still jumps to the overview. All themes. |
| **v0.1.1282** | Agent Ops Refresh first paint → **Refresh** and **Refresh digest** sit under the health cards in the theme HTML. They no longer jump up from the bottom of Agent Ops after JavaScript loads. All themes. |
| **v0.1.1283** | Agent Ops filter first paint → Agents (**All · On · Off**), Schedules (**All · Jobs · Deliveries**), Knowledge (**All · Discord · Core**), and Runs lanes are in the theme HTML. Those rows no longer pop in after JavaScript loads. Counts still update after a load. All themes. |
| **v0.1.1284** | Agent Ops Sessions filter first paint → **All · Live · Files** chips are in the theme HTML. They no longer pop in after JavaScript loads. Counts still update after a load. All themes. |
| **v0.1.1285** | Agent Ops tab counts first paint → Agents, Sessions, Schedules, Knowledge, and Runs show a **None yet** count pill in the theme HTML. The tabs no longer grow when JavaScript loads the counts. A real count still replaces **None yet**. All themes. |
| **v0.1.1286** | Agent Ops overview head counts first paint → Agents, Schedules, Knowledge, Recent chats, and Runs show **None yet**. Live shows **Quiet**. Digest shows **Queue clear**. The card titles no longer grow when JavaScript loads the counts. A real count still replaces those words. All themes. |
| **v0.1.1287** | Agent Ops Updated stamp first paint → the Refresh row shows **None yet** in the theme HTML. The stamp no longer pops in beside Refresh after the first refresh. A real age still replaces **None yet** (**Updated just now**). All themes. |
| **v0.1.1288** | Agent Ops tab counts stay **None yet** when the count is zero. The pill no longer shrinks to **0** after a refresh. A positive count still replaces **None yet**. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~15.7d).
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
- ~00:27 tick: ring label first-paint keep (**v0.1.1277**); install/kickstart.
- ~00:52 tick: Low Power Mode chip first-paint keep (**v0.1.1278**); install/kickstart.
- ~01:19 tick: Apple CPU and Freq ring arcs match the track (**v0.1.1279**); install/kickstart.
- ~01:46 tick: Agent Ops overview cards first-paint keep (**v0.1.1280**); install/kickstart.
- ~02:13 tick: Agent Ops tab strip first-paint keep (**v0.1.1281**); install/kickstart.
- ~02:39 tick: Agent Ops Refresh row first-paint keep (**v0.1.1282**); install/kickstart.
- ~03:08 tick: Agent Ops filter-row first-paint keep (**v0.1.1283**); install/kickstart.
- ~03:35 tick: Agent Ops Sessions All · Live · Files first-paint keep (**v0.1.1284**); install/kickstart.
- ~04:05 tick: Agent Ops tab count first-paint keep (**v0.1.1285**); install/kickstart.
- ~04:26 tick: Agent Ops overview head count first-paint keep (**v0.1.1286**); install/kickstart.
- ~04:53 tick: Agent Ops Updated stamp first-paint keep (**v0.1.1287**); install/kickstart.
- ~05:19 tick: Agent Ops tab counts stay None yet at zero (**v0.1.1288**); install/kickstart.

## Next fuel

- Agent Ops filter match chips (`N/M`) and Clear stay hidden until a query, so they do not move first paint.
- The keyboard hint (`#ops-keyboard-hint`) stays hidden in CSS, so it does not move the layout.
- Agent Ops screenshot refresh when Screen Recording TCC allows (`feature-agent-ops.png`).
- Sibling ports if digester stays empty.
