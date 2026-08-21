# Overnight standing backlog (Track B fuel)

When digester **open** is empty, the overnight harness **must** pull from this list (top first). Cross out or move completed items down after a keep. Local overrides/merges live in `~/.mac-stats/improvements/standing_backlog.md` (merge, do not overwrite).

## P0 — latency / thrash

1. ~~**Improve/memory scheduled task thrash**~~ — done in **v0.1.260** (runner prompt compaction).
2. **p50 direct latency** — digester excludes Improve-task thrash (**v0.1.270**); overnight / how-solved-task asks instant (**v0.1.271**); bare news + topic-dump pre-route (**v0.1.273**); exact saved-note reads instant (**v0.1.274**); dump-what-you-saved instant (**v0.1.275**); TASK_CREATE pre-route (**v0.1.276**); ship-version instant fix + lighthouse/pagespeed pre-route (**v0.1.277**); last-task / what-needs instant (**v0.1.278**); Perplexity NL + last-task clarifier (**v0.1.279**); exact-plan note extract (**v0.1.280**); Google SERP → search rewrite (**v0.1.281**); clear flight route asks pre-route (**v0.1.282**); vague follow-up clarifiers instant (**v0.1.283**); airport-hop + bare research pre-route (**v0.1.284**); itinerary-correction instant (**v0.1.286**); event-date reviews pre-route (**v0.1.292**); itinerary preference instant (**v0.1.293**); multi-city travel-plan pre-route (**v0.1.294**); digester Slowest filters for those three (**v0.1.295**); morning-surprise table highlights + product changelog instant (**v0.1.374**); scheduled `SKILL:` weekly reviews filtered from Slowest/p50 (**v0.1.377**); more tool-heavy patterns remain.
    3. **Overnight design review** — Follow `docs/043_overnight_design_review.md`. Prefer stale feature screens (`feature-agent-ops`, `feature-ai-chat`, `feature-processes`) before re-shooting CPU. Digest empty-state polish in **v0.1.276**; refresh-button polish in **v0.1.278**; process-list polish in **v0.1.280** / keyboard+focus in **v0.1.298**; Agent Ops tab hover/focus in **v0.1.285**; Ops filter focus ring in **v0.1.287**; overview card hover in **v0.1.288** / focus-within in **v0.1.317**; list-row hover in **v0.1.296**; AI chat input in **v0.1.295** / composer glass + accents in **v0.1.370**; health-card keyboard nav in **v0.1.316**; overview active-tab + health wash in **v0.1.430**; active-tab accent wash + badge glass in **v0.1.431**; selected-row accent wash in **v0.1.447**; Sessions copy id/slug chip in **v0.1.451**; Schedules/deliveries click-to-preview in **v0.1.452**; Knowledge path copy chip in **v0.1.453**; Runs click-to-preview in **v0.1.468**; Agents copy id/slug chip in **v0.1.469**; Data Poster inactive-icon contrast in **v0.1.470**; Runs request-id copy chip in **v0.1.471**; Schedules/delivery id copy chip in **v0.1.472**; Runs Load into AI Chat in **v0.1.473**; Schedules Load into AI Chat in **v0.1.474**; Knowledge Load into AI Chat in **v0.1.475**; Agents Load into AI Chat in **v0.1.476**; overview Schedules click-to-preview in **v0.1.477**; overview Recent click-to-preview in **v0.1.478**; overview Live click-to-preview in **v0.1.479**; overview Knowledge click-to-preview (row select) in **v0.1.480**; overview Last delivery click-to-preview in **v0.1.481**; Runs Insights Slowest/Candidates click-to-preview in **v0.1.482**; health Next schedule / Last delivery click-to-preview in **v0.1.483**; Digest open hints + health Digest click-to-preview in **v0.1.484**; health Version → primary agent open in **v0.1.485**; overview Agents card in **v0.1.488**; overview Runs card in **v0.1.489**; overview Digest card in **v0.1.490**; health schedule/delivery ok/warn/bad wash in **v0.1.491**; health Version ok/warn/bad wash in **v0.1.492**; overview Schedules ok/warn/bad wash in **v0.1.493**; overview Agents ok/warn/bad wash in **v0.1.494**; overview Runs ok/warn/bad wash in **v0.1.495**; overview Digest ok/warn/bad wash in **v0.1.496**; overview Live ok/warn/bad wash in **v0.1.497**; overview Knowledge ok/warn/bad wash in **v0.1.498**; overview Recent ok/warn/bad wash in **v0.1.499**; overview cards click/keyboard open linked tab in **v0.1.500**; tab inventory count pills in **v0.1.503**; Refresh/Updated under health in **v0.1.504**; filter N/M chips in **v0.1.505**; filter-row Clear beside N/M in **v0.1.506**; overview head count pills in **v0.1.507**; 0 Overview jump in **v0.1.508**; Top Processes click-to-copy name in **v0.1.509**; Agent Ops `c` copy id in **v0.1.510**; Monitors `c` URL in **v0.1.511**; Disk Cleanup path copy in **v0.1.512**; CPU ring value copy in **v0.1.513** / CPU % Details toggle restore in **v0.1.516**; AI Chat starter chips in **v0.1.514** / starter chip In composer flash in **v0.1.527**; Debug Log Error/Warn filters in **v0.1.515**; Debug Log error/warn glance in **v0.1.533**; battery/power strip click-to-copy in **v0.1.517**; Monitors summary click + empty Add CTA in **v0.1.518**; Agent Ops empty Open AI Chat in **v0.1.519**; CPU metrics RAM strip in **v0.1.520**; CPU metrics GPU strip in **v0.1.534**; Temp °C strip in **v0.1.535**; frequency GHz strip in **v0.1.536**; SSD % strip in **v0.1.537**; CPU % strip in **v0.1.538**; AI Chat last-answer glance (copy) in **v0.1.539**; Monitors All/Up/Down filter chips in **v0.1.521**; Top Processes All/Pinned filter chips in **v0.1.522**; Disk Cleanup empty Review scopes CTA in **v0.1.523**; Disk Cleanup Reclaimable now meta-card click in **v0.1.524**; Disk Cleanup Enabled scopes meta-card click in **v0.1.525**; Disk Cleanup Next automatic run meta-card click in **v0.1.526**; Disk Cleanup Runs when meta-card click in **v0.1.528**; Disk Cleanup Last run panel click in **v0.1.529**; Top Processes Top CPU glance in **v0.1.531**; Top Processes Top GPU glance in **v0.1.541**; Top Processes Top RAM glance in **v0.1.547**; AI Chat turn glance in **v0.1.532**; Agent Ops Knowledge All·Discord·Core chips in **v0.1.546**; Disk Cleanup All·Reclaim·Clean chips in **v0.1.548**; AI Chat All·You·Assistant chips in **v0.1.549**; CPU metrics uptime strip in **v0.1.550**; Perplexity last-search glance in **v0.1.551**; Monitors collapsed glance in **v0.1.552**; Disk Cleanup collapsed glance in **v0.1.553**; AI Chat collapsed glance in **v0.1.554**; Agent Ops Discord Ready collapsed glance in **v0.1.555**; Perplexity collapsed keep-header in **v0.1.556**; Debug Log collapsed keep-header in **v0.1.557**; Top Processes collapsed keep-header in **v0.1.558**; Details collapsed keep-header in **v0.1.559**; CPU metrics Heat/thermal strip in **v0.1.560**; Heat prefers **NSProcessInfo.thermalState** in **v0.1.561**; menu-bar LPM in **v0.1.563**; menu-bar Heat Serious/Critical in **v0.1.564**; menu-bar SSD ≥85% amber in **v0.1.567**; menu-bar RAM ≥85% amber + strip hot wash in **v0.1.568**; menu-bar CPU ≥50% amber in **v0.1.569**; menu-bar GPU ≥15% amber in **v0.1.570**; menu-bar Temp ≥70°C amber in **v0.1.571**.
    4. ~~**README / landing**~~ — sharper vs-competitor framing (**v0.1.265**).

## P2 — reliability

5. **`debug.log` errors** — First recurring error/panic in the last 24h that is product-owned. In-app Debug Log Error/Warn filter chips in **v0.1.515**. Brave health-ping quota burn mitigated in **v0.1.272**. Website monitor DNS/connect failures classify to short reasons in **v0.1.375** (UI + log). DOWN recheck backoff (DNS ≥5 min) in **v0.1.376**. DOWN next-check countdown in UI in **v0.1.377**. Unchanged-UP `monitors.json` rewrite throttle (~5 min) in **v0.1.378**. Unchanged UP/DOWN recheck logs → TRACE in **v0.1.379**. Install refuses stale release binary in **v0.1.380**. Idle task-review scan / no-open → DEBUG when empty in **v0.1.382**. Monitors summary names DOWN hosts + short reasons in **v0.1.383**. Monitor last-check age + DOWN-first list sort + slowest-host summary in **v0.1.384**. Monitors Arrow/Home/End + Enter check-now in **v0.1.385**; j/k + Esc clear selection in **v0.1.392**; `d` detail toggle + Esc closes detail first in **v0.1.402**. Disk Cleanup scope keyboard in **v0.1.386**; category keyboard + Enter Clean now in **v0.1.387**; Delete/Backspace removes custom scopes in **v0.1.388**; Enter-to-add + ⌘/Ctrl+S save in **v0.1.389**; `R` toggles Recurse in **v0.1.390**; `T` toggles Trash soft-delete in **v0.1.391**.
6. **Discord / LaunchAgent uptime** — Confirm process + Discord Ready after any install; fix silent downtime causes. Single-instance busy WARN rate-limit (**v0.1.381**) cuts KeepAlive thrash noise in `debug.log`.

## P3 — sibling ports

7. OpenClaw / Hermes ports that clearly map to mac-stats tools/sessions (not docs-only Related sections). Google SERP FETCH_URL→search rewrite shipped in **v0.1.281**. Insights/status/digest/schedules/scrub/`/help`/interrupt NL in **v0.1.306–315**. Discord voice STT harden in **v0.1.313**. Climate/clima/klima → Open-Meteo + Brave-weather→Perplexity redirect in **v0.1.319–321**.

## Done recently (do not re-pick as filler)

- Menu bar **Up** amber cue when system uptime ≥ 7 days (power-strip Up is-long parity) — **v0.1.573**
- Menu bar **GHz** amber cue when cached frequency ≥ 3.5 GHz (power-strip freq is-hot parity) — **v0.1.572**
- Menu bar **Temp** amber cue when cached CPU ≥ 70°C (power-strip Temp is-hot parity) — **v0.1.571**
- Menu bar **GPU** amber cue when usage ≥ 15% (power-strip GPU is-hot parity) — **v0.1.570**
- Menu bar **CPU** amber cue when usage ≥ 50% (power-strip CPU is-hot parity) — **v0.1.569**
- Menu bar **RAM** amber cue when memory ≥ 85% + power-strip RAM hot wash — **v0.1.568**
- Menu bar **SSD** amber cue when disk ≥ 85% (power-strip hot parity) — **v0.1.567**
- Menu bar **Heat** Fair soft yellow + power-strip Fair soft wash — **v0.1.566**
- Menu bar **Ollama ✕** red semibold when circuit open (Mon ✕ color parity) — **v0.1.565**

- Menu bar **Heat** cue when thermal Serious (amber) or Critical (red); **Fair** soft yellow in **v0.1.566** — **v0.1.564**
- Menu bar **green LPM** cue when Low Power Mode is on (Mon ✕ cue style; hidden when off) — **v0.1.563**
- CPU metrics **Low Power Mode (LPM) on the battery/power strip** (On/Off from `isLowPowerModeEnabled`; click → Battery settings; green wash when On) — **v0.1.562**
- CPU metrics **Heat prefers NSProcessInfo.thermalState** (OS Nominal/Fair/Serious/Critical; °C-band fallback; AI Thermal pressure) — **v0.1.561**
- CPU metrics **Heat / thermal on the battery/power strip** (Nominal / Fair / Serious / Critical; click → temp ring; amber Serious / red Critical) — **v0.1.560**
- Details **collapsed keep-header** (header + Load · RAM · Up glance stay when collapsed; Waiting · details; amber wash Load≥4 / RAM≥85%) — **v0.1.559**
- Top Processes **collapsed keep-header** (header + Top CPU/GPU/RAM glances stay when collapsed; Waiting · processes; list/filters/hint hide) — **v0.1.558**
- Debug Log **collapsed keep-header** (Quiet · clean / error·warn glance stays under header when collapsed) — **v0.1.557**
- Debug Log collapsed keep-header in **v0.1.557**; Perplexity Search **collapsed keep-header** (header + last-search glance stay visible when collapsed; Ready · search / Last · query / Key · add API key; click expands) — **v0.1.556**
- Agent Ops **Discord Ready collapsed glance** (Discord · Ready · age / Offline / reconnect warn under header when collapsed; click → expand + gateway preview; 60s insights poll) — **v0.1.555**
- AI Chat **collapsed glance** (summary under header when collapsed; Offline / Ready · model / turns · last question; click → expand + composer or URL dialog) — **v0.1.554**
- Disk Cleanup **collapsed glance** (summary under header when collapsed; click → reclaim / due / scopes; 60s shallow poll; collapsed section keeps header + glance) — **v0.1.553**
- External / Monitors **collapsed glance** (summary under header when collapsed; click → expand + DOWN/slowest/Add; 30s summary poll) — **v0.1.552**
- Perplexity Search **last-search glance** (Last · query · N results / Key · add API key; click expands + focuses) — **v0.1.551**
- CPU metrics **uptime on the battery/power strip** (Up · formatted; click → Details Uptime flash; amber wash ≥ 7d) — **v0.1.550**
- AI Chat **All · You · Assistant** filter chips (counts; Clear → All; empty hides chips) — **v0.1.549**
- Disk Cleanup **All · Reclaim · Clean** filter chips (counts; Reclaimable-now → Reclaim; Clear → All) — **v0.1.548**
- Top Processes **Top RAM glance** (highest resident memory in list; click opens details; amber wash ≥ 1 GiB; `ProcessUsage.memory`) — **v0.1.547**
- Agent Ops Knowledge **All · Discord · Core** filter chips (counts; overview → Discord when any; Clear → All) — **v0.1.546**
- Agent Ops Schedules **All · Jobs · Deliveries** filter chips (counts; overview → Jobs when any; Clear → All) — **v0.1.545**
- Agent Ops Runs **All · Instant · Direct** filter chips (counts; overview → Direct when any; Clear → All) — **v0.1.544**
- Agent Ops Agents **All · On · Off** filter chips (counts; overview Agents opens On/Off; Clear → All) — **v0.1.543**
- Agent Ops Sessions **All · Live · Files** filter chips (counts; overview Live/Recent set kind; Clear → All) — **v0.1.542**
- Top Processes **Top GPU glance** (highest GPU in list; click opens details; amber wash ≥ 15%) — **v0.1.541**
- AI Chat **model / connection glance** (Model · name / Offline; click picker or URL; green/amber wash) — **v0.1.540**
- AI Chat **last-answer glance** (preview + click copies reply; Copied flash; accent wash) — **v0.1.539**
- CPU metrics **CPU % on the battery/power strip** (click scrolls to CPU ring; amber wash ≥ 50%) — **v0.1.538**
- CPU metrics **SSD % on the battery/power strip** (click opens Disk Cleanup; amber wash ≥ 85%) — **v0.1.537**
- CPU metrics **frequency GHz on the battery/power strip** (click scrolls to freq ring; amber wash ≥ 3.5 GHz) — **v0.1.536**
- CPU metrics **GPU % on the battery/power strip** (click scrolls to GPU ring; amber wash ≥ 15%) — **v0.1.534**
- Debug Log **error/warn glance** strip (ERROR/WARN counts from tail; 60s poll when collapsed; click expands + filters; red/amber wash) — **v0.1.533**
- AI Chat **turn glance** strip (turn count + last question; scroll to latest + focus composer; accent wash while sending) — **v0.1.532**
- Top Processes **Top CPU glance** click opens details (amber wash when CPU ≥ 15%) — **v0.1.531**
- Monitors summary click opens **slowest** site when all UP (latency hint; amber wash; DOWN still first DOWN) — **v0.1.530**
- Disk Cleanup **Last run panel click** (first cleaned category / reclaim / scopes; amber wash on Trash skips) — **v0.1.529**
- Disk Cleanup **Runs when meta-card click** (enabled scopes focus; periodic-off blue wash) — **v0.1.528**
- AI Chat empty-state starter chip **In composer** flash (Load into AI Chat parity; no auto-send) — **v0.1.527**
- Disk Cleanup **Next automatic run meta-card click** (due → Clean now; else last run; green wash when due) — **v0.1.526**
- Disk Cleanup **Enabled scopes meta-card click** (first off scope / Add form; soft wash when off) — **v0.1.525**
- Disk Cleanup **Reclaimable now meta-card click** (first reclaim category / scopes when empty) — **v0.1.524**
- Disk Cleanup **empty Review scopes CTA** (warm empty + focus first off scope / Add form) — **v0.1.523**
- Top Processes **All · Pinned filter chips** (pinned count + filter-miss Clear) — **v0.1.522**
- External / Monitors **All · Up · Down filter chips** (counts + filter-miss Clear) — **v0.1.521**
- CPU metrics **RAM on the battery/power strip** (click opens Details) — **v0.1.520**
- Agent Ops empty Live / session files / Runs **Open AI Chat** CTA — **v0.1.519**
- External / Monitors summary click + empty Add CTA (first DOWN / Add a monitor) — **v0.1.518**
- Battery / power strip click-to-copy (% · W; Copied overlay) — **v0.1.517**
- Debug Log Error / Warn filter chips (All · Error · Warn counts; continuation lines) — **v0.1.515**
- AI Chat empty-state starter chips (fill composer; Send/Enter; no auto-send) — **v0.1.514**
- CPU metrics click-to-copy ring values (GPU % · GHz · °C; Copied overlay) — **v0.1.513** (CPU % click restored to Details/Processes toggle in **v0.1.516**)

- Disk Cleanup click-to-copy path (scopes + categories; click / `c`; Copied flash) — **v0.1.512**
- Monitors `c` copies URL (click-to-copy parity; Top Processes / Agent Ops) — **v0.1.511**
- Agent Ops `c` copies selected id (list + preview chip; Copied flash) — **v0.1.510**
- Top Processes click-to-copy name (list + `c` + details hero) — **v0.1.509**
- Agent Ops 0 Overview jump — **v0.1.508**
- Agent Ops overview head count pills — **v0.1.507**
- Agent Ops filter-row Clear beside N/M chip — **v0.1.506**
- Agent Ops filter live N/M match chips — **v0.1.505**
- Agent Ops Refresh/Updated under health strip — **v0.1.504**
- Agent Ops tab inventory count pills — **v0.1.503**

- Agent Ops Updated … ago stamp beside Refresh — **v0.1.502**
- Agent Ops tab digit badges (1–5) — **v0.1.501**
- Agent Ops overview cards click/keyboard open linked tab — **v0.1.500**
- Agent Ops overview Recent ok/warn/bad wash — **v0.1.499**
- Agent Ops overview Knowledge ok/warn/bad wash — **v0.1.498**
- Agent Ops overview Live ok/warn/bad wash — **v0.1.497**
- Agent Ops overview Digest ok/warn/bad wash — **v0.1.496**
- Agent Ops overview Runs ok/warn/bad wash — **v0.1.495**
- Agent Ops overview Agents ok/warn/bad wash — **v0.1.494**
- Agent Ops overview Schedules ok/warn/bad wash — **v0.1.493**
- Agent Ops health Version ok/warn/bad wash — **v0.1.492**
- Agent Ops health Next schedule / Last delivery ok/warn/bad wash — **v0.1.491**
- Agent Ops overview Digest card (open-hint snapshot + click-to-preview) — **v0.1.490**
- Agent Ops overview Runs card (recent snapshot + click-to-preview) — **v0.1.489**
- Agent Ops overview Agents card (enabled snapshot + click-to-open) — **v0.1.488**
- Agent Ops health Redmine → Redmine agent open — **v0.1.487**
- Agent Ops health Discord → Runs gateway preview — **v0.1.486**
- Agent Ops health Version → primary agent open — **v0.1.485**
- Agent Ops Runs Insights Digest open hints + health Digest click-to-preview — **v0.1.484**
- Agent Ops health Next schedule / Last delivery click-to-preview — **v0.1.483**
- Agent Ops Runs Insights Slowest/Candidates click-to-preview — **v0.1.482**
- Agent Ops overview Last delivery click-to-preview — **v0.1.481**
- Agent Ops overview Knowledge click-to-preview (list row select) — **v0.1.480**
- Agent Ops overview Live click-to-preview — **v0.1.479**
- Agent Ops overview Recent click-to-preview — **v0.1.478**
- Agent Ops overview Schedules click-to-preview — **v0.1.477**
- Agent Ops Agents Load into AI Chat (soul/skill/mood → composer) — **v0.1.476**
- Agent Ops Knowledge Load into AI Chat (file → composer) — **v0.1.475**
- Agent Ops Schedules Load into AI Chat (task/summary → composer) — **v0.1.474**
- Agent Ops Runs Load into AI Chat (question → composer) — **v0.1.473**
- Agent Ops Schedules/deliveries click-to-copy id chip — **v0.1.472**
- Agent Ops Runs request-id click-to-copy chip — **v0.1.471**
- Agent Ops Agents click-to-copy slug/id chip — **v0.1.469**
- Data Poster inactive section icons near-white — **v0.1.470**
- Agent Ops Runs click-to-preview question/tools/request id — **v0.1.468**
- Agent Ops Knowledge click-to-copy path chip — **v0.1.453**
- Agent Ops Schedules/deliveries click-to-preview full task/summary — **v0.1.452**
- Agent Ops Sessions click-to-copy id/slug chip — **v0.1.451**
- Agent Ops health-card active-tab accent ring — **v0.1.450**
- Agent Ops tab true-empty title+hint — **v0.1.449**
- Monitors URL click-to-copy Copied flash — **v0.1.448**
- Agent Ops selected-row accent wash (↑/↓ · j/k · click) — **v0.1.447**
- Agent Ops overview empty Open-tab CTAs — **v0.1.446**
- Top Processes detail PID click-to-copy Copied flash — **v0.1.445**
- Disk Cleanup soft-delete (Move to Trash) Saved flash — **v0.1.444**
- Agent Ops filter-miss Clear filter — **v0.1.443**
- Settings product toggles Saved flash (AI / compact menu bar / compact CPU window) — **v0.1.442**
- Debug Log path hint click-to-copy + Copied flash — **v0.1.441**
- Theme / app version → changelog Opened flash — **v0.1.440**
- Footer GitHub Opening…/Opened flash — **v0.1.439**
- Settings Help / cheat sheet Opened flash — **v0.1.438**
- Top Processes pin/unpin ★ flash (Pinned/Unpinned) — **v0.1.437**
- CPU window header Refresh spinning + ✓ flash — **v0.1.436**
- Settings View logs Opening…/Opened flash — **v0.1.435**
- Settings Reset to monitor defaults Resetting…/Reset flash — **v0.1.433**
- AI Chat system-prompt Reset to Default Resetting…/Reset flash — **v0.1.434**
- Agent Ops active-tab accent wash + on/off badge glass — **v0.1.431** (screenshot recapture deferred TCC)
- Agent Ops overview active-tab highlight + health ok/warn/bad wash — **v0.1.430** (screenshot recapture deferred TCC)
- Agent Ops Load into AI Chat Loaded flash — **v0.1.432**
- Force Quit Quitting… busy-guard + Quit flash — **v0.1.424**
- Logs Open in editor busy-guard (Opening…) + Opened flash — **v0.1.423**
- Disk Cleanup Add scope busy-guard (Adding…) + Added flash — **v0.1.422**
- Monitors Delete/Backspace removes selected + detail Remove busy-guard — **v0.1.421**
- Disk Cleanup Save scopes busy-guard (Saving…) + Saved flash — **v0.1.420**
- Perplexity Save key / Clear key busy-guard (Saving…/Clearing…) + Saved/Cleared flash (Settings + inline) — **v0.1.419**
- Monitors Add Monitor busy-guard (Adding…) + Added flash; form stays open for confirmation — **v0.1.418**
- AI Chat system-prompt Save busy-guard + Saved flash — **v0.1.417**
- Agent Ops Save (soul/skill/mood) busy-guard + Saved flash — **v0.1.416**
- Discord Save token / Clear token busy-guard + Saved/Cleared flash — **v0.1.415**
- Logs Refresh busy-guard (Refreshing…) + Refreshed flash; auto-refresh silent — **v0.1.414**
- Monitors Check now busy-guard (Checking…) + Checked flash — **v0.1.413**
- Disk Cleanup Refresh busy-guard (Refreshing…) + Refreshed flash; Clean now Cleaned flash — **v0.1.412**
- Agent Ops Refresh / Refresh digest busy-guard (Refreshing…) + Refreshed flash — **v0.1.411**
- Perplexity Search busy-guard (Searching…) + Searched flash + Enter-to-search — **v0.1.410**
- AI Chat Clear button + Cleared flash (disabled while empty / Sending…) — **v0.1.409**
- AI Chat Send busy-guard (Sending…) + Sent flash — **v0.1.408**
- Disk Cleanup last-run soft-delete skip counts — **v0.1.407**
- Disk Cleanup soft-delete: skip when Trash move fails — no permanent fallback (**v0.1.406**)
- Disk Cleanup PageUp/PageDown (~5) on scopes + categories (**v0.1.405**). Soft-delete EPERM → skip (no permanent fallback) in **v0.1.406**
- Monitors PageUp/PageDown (~5) + hint/tooltips (**v0.1.404**)
- Top Processes PageUp/PageDown (~5) + `d` details + Esc closes modal first (**v0.1.403**)
- Monitors `d` detail toggle + Esc closes detail before clear (**v0.1.402**)
- Top Processes j/k + Esc clear + P pin/unpin + kb hint (**v0.1.401**)
- Disk Cleanup j/k + Esc clear selection + hints (**v0.1.393**)
- Monitors j/k + Esc clear selection + kb hint (**v0.1.392**)
- Disk Cleanup keyboard: scopes / categories / Delete custom / Enter-to-add + ⌘S / R recurse / T soft-delete / j/k + Esc (**v0.1.386–391, 393**)
- Instant lanes: version, thread clarifier, weather Open-Meteo, short ack, …
- Menu bar SSD + `MEMORY: save` verbatim notes
- Digester filters for travel/SEO and scheduled SKILL (meta — not a night’s sole win)
