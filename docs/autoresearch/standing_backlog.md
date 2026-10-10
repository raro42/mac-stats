# Overnight standing backlog (Track B fuel)

When digester **open** is empty, the overnight harness **must** pull from this list (top first). Cross out or move completed items down after a keep. Local overrides/merges live in `~/.mac-stats/improvements/standing_backlog.md` (merge, do not overwrite).

## P0 — latency / thrash

1. ~~**Improve/memory scheduled task thrash**~~ — done in **v0.1.260** (runner prompt compaction).
    2. **p50 direct latency** — digester excludes Improve-task thrash (**v0.1.270**); overnight / how-solved-task asks instant (**v0.1.271**); bare news + topic-dump pre-route (**v0.1.273**); exact saved-note reads instant (**v0.1.274**); dump-what-you-saved instant (**v0.1.275**); TASK_CREATE pre-route (**v0.1.276**); ship-version instant fix + lighthouse/pagespeed pre-route (**v0.1.277**); last-task / what-needs instant (**v0.1.278**); Perplexity NL + last-task clarifier (**v0.1.279**); exact-plan note extract (**v0.1.280**); Google SERP → search rewrite (**v0.1.281**); clear flight route asks pre-route (**v0.1.282**); vague follow-up clarifiers instant (**v0.1.283**); airport-hop + bare research pre-route (**v0.1.284**); itinerary-correction instant (**v0.1.286**); event-date reviews pre-route (**v0.1.292**); itinerary preference instant (**v0.1.293**); multi-city travel-plan pre-route (**v0.1.294**); digester Slowest filters for those three (**v0.1.295**); morning-surprise table highlights + product changelog instant (**v0.1.374**); scheduled `SKILL:` weekly reviews filtered from Slowest/p50 (**v0.1.377**); AI Chat operator instant lane (`/status` `/insights` `/schedules` `/digest` scrub `/help`) in **v0.1.628**; Agent Ops Runs Slow filter in **v0.1.693**; `/failed` **v0.1.695**; `/slow` **v0.1.696**; `/instant`+`/direct` **v0.1.697**; `/lite` **v0.1.704**; `/agents` On/Off **v0.1.705**; `/sessions` Live/Files **v0.1.706**; `/knowledge` Discord/Core **v0.1.707**; `/schedules` Jobs/Deliveries **v0.1.708**; `/monitors` Up/Down/Slow **v0.1.709**; `/disk` On/Off/Reclaim/Big/Clean **v0.1.710**; `/logs` **v0.1.711**; `/processes` **v0.1.712**; `/perplexity` **v0.1.713**; `/pinned` **v0.1.714**; `/rings` **v0.1.715**; `/strip` **v0.1.716**; `/details` **v0.1.718**; `/battery`·`/heat`·`/lpm` **v0.1.719**; `/cpu`·`/gpu`·`/freq`·`/temp` **v0.1.720**; `/ram`·`/ssd`·`/uptime` **v0.1.721**; `/discord` **v0.1.722**; `/ollama`·`/llm` **v0.1.723**; `/redmine` **v0.1.724**; `/brave` **v0.1.725**; `/perplexity key` **v0.1.726**; `/mastodon` **v0.1.727**; `/mcp` **v0.1.728**; `/cursor`·`/cursor-agent` **v0.1.729**; `/telegram`·`/slack`·`/signal`·`/alerts` **v0.1.730**; `/skills` **v0.1.731**; `/tasks` **v0.1.732**; `/plugins` On/Off **v0.1.733**; `/browser`·`/cdp` **v0.1.734**; `/judge` **v0.1.735**; `/ai`·`/ai-agent` **v0.1.736**; next schedule / next job instant (**v0.1.800**); last delivery instant (**v0.1.801**); schedule/delivery count instant (**v0.1.802**); operator inventory count instant (**v0.1.803**); runs count instant (**v0.1.804**); digest open read-only instant (**v0.1.805**); digest age read-only instant (**v0.1.806**); debug log error/warn count instant (**v0.1.807**); debug log file size instant (**v0.1.808**); debug log path instant (**v0.1.809**); debug log age instant (**v0.1.810**); config path / data home instant (**v0.1.811**); screenshots path instant (**v0.1.812**); runs.jsonl path instant (**v0.1.813**); task directory path instant (**v0.1.814**); memory/notes path instant (**v0.1.815**); session directory path instant (**v0.1.816**); agents directory path instant (**v0.1.817**); STT Elmasnow→El Masnou Open-Meteo (**v0.1.818**); skills directory path instant (**v0.1.819**); plugins/scripts directory path instant (**v0.1.820**); prompts directory path instant (**v0.1.821**); tmp directory path instant (**v0.1.822**); uploads directory path instant (**v0.1.824**); CDP traces directory path instant (**v0.1.825**); pdfs directory path instant (**v0.1.826**); browser credentials path instant (**v0.1.827**); browser storage-state / cookies path instant (**v0.1.828**); browser-downloads directory path instant (**v0.1.829**); cleanup-quarantine path instant (**v0.1.830**); pinned_processes.json path instant (**v0.1.831**); schedules.json path instant (**v0.1.832**); monitors.json path instant (**v0.1.833**); history.json path instant (**v0.1.834**); disk_cleanup.json path instant (**v0.1.835**); perplexity_last.json path instant (**v0.1.836**); discord_channels.json path instant (**v0.1.837**); scheduler_delivery_awareness.json path instant (**v0.1.838**); user-info.json path instant (**v0.1.839**); `.config.env` path instant (**v0.1.840**); improvements dir path instant (**v0.1.841**); credential_accounts.json path instant (**v0.1.845**); escalation_patterns.md path instant (**v0.1.846**); session_reset_phrases.md path instant (**v0.1.847**); cookie_reject_patterns.md path instant (**v0.1.848**); downloads-organizer-rules.md path instant (**v0.1.849**); downloads-organizer-state.json path instant (**v0.1.850**); soul.md path instant (**v0.1.851**); mood.md path instant (**v0.1.852**); skill.md path instant (**v0.1.853**); testing.md path instant (**v0.1.854**); planning_prompt.md path instant (**v0.1.855**); execution_prompt.md path instant (**v0.1.856**); agent.json path instant (**v0.1.857**); memory.md curated file path instant (**v0.1.858**); Ori vault path instant (**v0.1.859**); before-reset transcript path instant (**v0.1.860**); before-compaction transcript path instant (**v0.1.861**); Discord channel memory path instant (**v0.1.862**); session-memory path instant (**v0.1.863**); LaunchAgent plist path instant (**v0.1.864**); results.tsv / runs / digest / improvements / screenshots size lanes (**v0.1.866–874**); tmp directory size instant (**v0.1.875**); uploads directory size instant (**v0.1.876**); CDP traces directory size instant (**v0.1.877**); PDF exports directory size instant (**v0.1.878**); browser-downloads directory size instant (**v0.1.879**); agents directory size instant (**v0.1.880**); skills directory size instant (**v0.1.881**); plugins/scripts directory size instant (**v0.1.882**); prompts directory size instant (**v0.1.883**); session directory size instant (**v0.1.884**); task directory size instant (**v0.1.885**); cleanup-quarantine directory size instant (**v0.1.886**); notes/memory folder size instant (**v0.1.887**); `review logs` / `check logs` NL → `/logs` instant (**v0.1.888**); config.json size instant (**v0.1.889**); schedules.json size instant (**v0.1.890**); monitors.json size instant (**v0.1.891**); history.json size instant (**v0.1.892**); disk_cleanup.json size instant (**v0.1.893**); pinned_processes.json size instant (**v0.1.894**); discord_channels.json size instant (**v0.1.895**); perplexity_last.json size instant (**v0.1.896**); scheduler_delivery_awareness.json size instant (**v0.1.897**); user-info.json size instant (**v0.1.898**); credential_accounts.json size instant (**v0.1.899**); `.config.env` size instant (**v0.1.900**); soul.md size instant (**v0.1.901**); mood.md size instant (**v0.1.902**); skill.md size instant (**v0.1.903**); testing.md size instant (**v0.1.904**); planning_prompt.md size instant (**v0.1.905**); execution_prompt.md size instant (**v0.1.906**); agent.json size instant (**v0.1.907**); memory.md size instant (**v0.1.908**); escalation_patterns.md size (**v0.1.909**); session_reset_phrases.md size (**v0.1.910**); cookie_reject_patterns.md size (**v0.1.911**); downloads-organizer-rules.md size (**v0.1.912**); downloads-organizer-state.json size (**v0.1.913**); History sparkline Hot attention glance (**v0.1.914**); browser_storage_state.json size (**v0.1.915**); browser-credentials.toml size (**v0.1.916**); LaunchAgent plist size (**v0.1.917**); Discord channel memory size (**v0.1.918**); Overview Live idle calm (**v0.1.919**); session-memory size (**v0.1.920**); before-reset transcript size (**v0.1.921**); before-compaction transcript size (**v0.1.922**); Ori vault size (**v0.1.923**); config.json age (**v0.1.924**); schedules.json age (**v0.1.925**); monitors.json age (**v0.1.926**); history.json age (**v0.1.927**); disk_cleanup.json age (**v0.1.928**); pinned_processes.json age (**v0.1.929**); discord_channels.json age (**v0.1.930**); perplexity_last.json age (**v0.1.931**); scheduler_delivery_awareness.json age (**v0.1.932**); user-info.json age (**v0.1.933**); credential_accounts.json age (**v0.1.934**); `.config.env` age (**v0.1.935**); soul.md age (**v0.1.936**); mood.md age (**v0.1.937**); skill.md age (**v0.1.939**); testing.md age (**v0.1.940**); agent.json age (**v0.1.941**); planning_prompt.md age (**v0.1.943**); execution_prompt.md age (**v0.1.944**); memory.md age (**v0.1.945**); notes folder age (**v0.1.946**); session-memory age (**v0.1.947**); Discord channel memory age (**v0.1.948**); before-reset transcript age (**v0.1.949**); before-compaction transcript age (**v0.1.950**); Ori vault age (**v0.1.951**); improvements directory age (**v0.1.952**); screenshots directory age (**v0.1.953**); LaunchAgent plist age (**v0.1.954**); skills directory age (**v0.1.955**); agents directory age (**v0.1.956**); plugins/scripts directory age (**v0.1.957**); prompts directory age (**v0.1.958**); session directory age (**v0.1.959**); AI Chat filter Clear chip (**v0.1.960**); task directory age (**v0.1.961**); tmp directory age (**v0.1.962**); uploads directory age (**v0.1.963**); CDP traces directory age instant (**v0.1.964**); PDF exports directory age (**v0.1.965**); browser-downloads directory age (**v0.1.966**); cleanup-quarantine directory age (**v0.1.967**); cookie_reject_patterns.md age (**v0.1.968**); downloads-organizer-rules.md age (**v0.1.969**); downloads-organizer-state.json age (**v0.1.970**); browser-credentials.toml age (**v0.1.971**); browser_storage_state.json age (**v0.1.972**); escalation_patterns.md age (**v0.1.973**); session_reset_phrases.md age (**v0.1.974**); digest.md / latest.md age (**v0.1.975**); loop_backlog.md path (**v0.1.976**); loop_backlog.md size (**v0.1.977**); loop_backlog.md age (**v0.1.978**); sibling_harness.md path (**v0.1.979**); sibling_harness.md size (**v0.1.980**); sibling_harness.md age (**v0.1.981**); standing_backlog.md path (**v0.1.982**); standing_backlog.md size (**v0.1.983**); standing_backlog.md age (**v0.1.984**); morning surprise path (**v0.1.985**); morning surprise size (**v0.1.986**); morning surprise age (**v0.1.987**); overnight agent log path (**v0.1.990**); overnight agent log size (**v0.1.991**); overnight agent log age (**v0.1.992**); harness loop stdout path (**v0.1.993**); harness loop stdout size (**v0.1.994**); harness loop stdout age (**v0.1.995**); harness loop stderr path (**v0.1.999**); harness loop stderr size (**v0.1.1000**); harness loop stderr age (**v0.1.1001**); launchd stderr path (**v0.1.1002**); launchd stderr size (**v0.1.1003**); launchd stderr age (**v0.1.1004**); launchd stdout path (**v0.1.1005**); launchd stdout size (**v0.1.1006**); launchd stdout age (**v0.1.1007**); `/monitors` NL expand (**v0.1.1019**); `/disk` NL expand (**v0.1.1020**); `/schedules` NL expand (**v0.1.1021**); `/sessions` NL expand (**v0.1.1022**); `/knowledge` NL expand (**v0.1.1023**); `/agents` NL expand (**v0.1.1024**); `/skills` NL expand (**v0.1.1025**); `/tasks` NL expand (**v0.1.1026**); `/plugins` NL expand (**v0.1.1027**); `/browser`·`/cdp` NL expand (**v0.1.1028**); `/judge`·`/ai`·`/ai-agent` NL expand (**v0.1.1029**); `/compact`·`/menu-bar`·`/cpu-window` NL expand (**v0.1.1030**); `/downloads`·`/organizer` NL expand (**v0.1.1031**); `/ori`·`/mnemos` NL expand (**v0.1.1032**); `/having_fun`·`/fun`·`/idle` NL expand (**v0.1.1033**); `/voice`·`/stt` NL expand (**v0.1.1034**); `/telegram`·`/slack`·`/signal`·`/alerts` NL expand (**v0.1.1035**); `/redmine`·`/brave`·`/mastodon`·`/mcp` NL expand (**v0.1.1036**); `/discord`·`/ollama`·`/perplexity key`·`/cursor` NL expand (**v0.1.1037**); `/status`·`/health`·`/version` NL expand (**v0.1.1038**); `/insights`·`/help`·`/ops` NL expand (**v0.1.1039**); `/rings`·`/strip`·`/details` NL expand (**v0.1.1040**); `/cpu`·`/gpu`·`/freq`·`/temp` ring-chip NL expand (**v0.1.1041**); `/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/uptime` strip-chip NL expand (**v0.1.1042**); `/hot` · `/pinned` Hot/Pinned NL expand (**v0.1.1043**); `/failed` · `/slow` · `/instant` · `/lite` · `/direct` Runs-lane NL expand (**v0.1.1044**); `/perplexity` last-search NL expand (**v0.1.1045**); `/digest` open NL expand (**v0.1.1046**); `/keeps` ratchet counts (**v0.1.1047**); `/last-keep`·`/last-discard` newest row (**v0.1.1048**); `/recent-keeps`·`/recent-discards` tonight list (**v0.1.1049**); `/keep-rate` hit rate (**v0.1.1050**); `/keep-streak` consecutive (**v0.1.1051**); `/longest-streak` record (**v0.1.1052**); `/since-keep` age since last keep (**v0.1.1053**); `/first-keep` earliest tonight (**v0.1.1054**); `/keep-pace` avg gap (**v0.1.1055**); `/keep-median` median gap (**v0.1.1056**); `/keep-range` min–max gap (**v0.1.1057**); `/keep-p90` p90 gap (**v0.1.1058**); `/keep-iqr` IQR gap (**v0.1.1059**); `/keep-std` sample std gap (**v0.1.1060**); `/keep-mad` MAD gap (**v0.1.1061**); `/keep-cv` CV gap (**v0.1.1062**); `/keep-skew` skewness gap (**v0.1.1063**); `/keep-kurtosis` excess kurtosis gap (**v0.1.1064**); `/keep-entropy` Shannon entropy gap (**v0.1.1065**); `/keep-mode` modal gap (**v0.1.1066**); `/keep-gini` Gini gap (**v0.1.1068**); `/keep-histogram` gap histogram (**v0.1.1069**); `/keep-p95` p95 gap (**v0.1.1070**); `/keep-p10` p10 gap (**v0.1.1071**); `/keep-p25` p25 gap (**v0.1.1072**); which-model identity instant (**v0.1.1157**); Ollama URL / endpoint / “where is ollama” instant (**v0.1.1158**); how-hot / is-the-cpu-hot Temp ring instant (**v0.1.1159**); battery-left / is-the-battery-low Bat chip instant (**v0.1.1160**); RAM-used / how-much-memory / is-the-ram-high RAM chip instant (**v0.1.1161**); disk-used / how-much-storage / is-the-disk-full SSD chip instant (**v0.1.1162**); CPU-used / how-much-cpu / is-the-cpu-high / is-the-cpu-busy CPU ring instant (**v0.1.1163**); GPU-used / how-much-gpu / is-the-gpu-high / is-the-gpu-busy GPU ring instant (**v0.1.1164**); clock-speed / how-fast / is-the-frequency-high Freq ring instant (**v0.1.1165**); power-draw / how-much-power / is-the-power-high Power chip instant (**v0.1.1166**); load-high / how's-the-load / how-high-is-the-load Load chip instant (**v0.1.1167**); 5-minute / 15-minute load instant (**v0.1.1168**); P-core / E-core clock instant (**v0.1.1169**); heat-high / is-the-thermal-high / is-it-throttling Heat chip instant (**v0.1.1170**); charging / is-it-charging / is-it-plugged-in Bat chip instant (**v0.1.1171**); installed RAM / how-big-is-memory / how-many-gb-of-ram instant (**v0.1.1172**); chip name / what-chip / what-processor instant (**v0.1.1173**); disk free bytes / how-much-free-space / how-much-space-is-left instant (**v0.1.1175**); top CPU process / what's-using-the-most-cpu instant (**v0.1.1176**); top RAM process / what's-using-the-most-ram instant (**v0.1.1177**); top GPU process / what's-using-the-most-gpu instant (**v0.1.1178**); app uptime / how-long-have-you-been-running instant (**v0.1.1179**); “compact this session” no longer a session count (**v0.1.1180**); “reset this session” / “clear this session” / “new session” no longer a session count (**v0.1.1181**); “delete this session” / “remove this session” / “end this session” / “close this session” no longer a session count (**v0.1.1182**); “summarize this session” / “rename this session” / “session summary” / “title this session” no longer a session count (**v0.1.1183**); “resume this session” / “open this session” / “switch this session” / “continue this session” no longer a session count (**v0.1.1184**); “fork this session” / “duplicate this session” / “clone this session” no longer a session count (**v0.1.1185**); “export this session” / “share this session” / “archive this session” no longer a session count (**v0.1.1186**); “search this session” / “find this session” / “lookup this session” no longer a session count (**v0.1.1187**); “save this session” / “store this session” / “backup this session” no longer a session count (**v0.1.1188**); “restore this session” / “recover this session” / “reload this session” / “revert this session” no longer a session count (**v0.1.1189**); “import this session” / “load this session” / “merge this session” / “attach this session” no longer a session count (**v0.1.1190**); “pin this session” / “bookmark this session” / “star this session” / “favorite this session” no longer a session count (**v0.1.1191**); “start this session” / “begin this session” / “launch this session” / “restart this session” no longer a session count (**v0.1.1192**); “stop this session” / “halt this session” / “pause this session” / “quit this session” no longer a session count (**v0.1.1193**); “abort this session” / “cancel this session” / “terminate this session” no longer a session count (**v0.1.1194**); “kill this session” / “destroy this session” / “drop this session” no longer a session count (**v0.1.1195**, kill word-match so skill stays); “exit this session” / “leave this session” / “abandon this session” no longer a session count (**v0.1.1196**); session count is inventory-only (**v0.1.1197** — “edit” / “hide” / “move” / “copy” / “refresh” / “update this session” stay with the agent; verb list removed). operator inventory counts (agents/monitors/tasks/skills/plugins/knowledge) are inventory-only (**v0.1.1198** — “delete this agent”, “run this skill”, “check this monitor”, … stay with the agent); runs inventory counts are inventory-only (**v0.1.1199** — “delete these runs”, “clear the runs”, “export runs” stay with the agent). Schedule/delivery inventory counts are inventory-only (**v0.1.1201** — “delete these schedules”, “clear the jobs”, “export deliveries” stay with the agent). Keep/discard inventory counts are inventory-only (**v0.1.1202** — “delete these keeps”, “clear the discards”, “export keeps” stay with the agent).
    3. **Overnight design review** — Changelog body keyboard hint first paint in theme HTML (at the top of the changelog body, hidden until two versions exist, says how to move across those version headings, not injected after JS) in **v0.1.1385**; Agent Ops row-selection Tips keyboard hint first paint in theme HTML (under the tab-bar hint, says how to move across tabs / filters / list rows, not injected after JS) in **v0.1.1384**; AI Chat message-list keyboard hint first paint in theme HTML (above the message list, hidden until turns exist, says how to move across those messages, not injected after JS) in **v0.1.1383**; Perplexity setup keyboard hint first paint in theme HTML (under key · Save key, hidden until setup is on screen, says how to move across those controls, not injected after JS) in **0.1.1382**; Monitors detail toolbar keyboard hint first paint with Check now · Remove (same paint as the buttons, hidden until both are on screen, says how to move across those controls, not injected after the buttons) in **v0.1.1381**; Theme-list fallback keyboard hint first paint in theme HTML (at the end of the theme list, hidden until Settings is open and the Appearance section is missing, says how to move across those buttons, Appearance section hint stays in charge when that section exists, not injected after JS) in **v0.1.1380**; Product setting keyboard hint first paint in theme HTML (at the end of the Product setting, hidden until Settings is open, says how to move across those controls, Help sheet swaps the line to the copy keys, not injected after JS) in **v0.1.1379**; Appearance section keyboard hint first paint in theme HTML (at the end of the Appearance section, hidden until Settings is open, says how to move across those controls, first theme crosses to the Settings header, window frame last crosses to Product, not injected after JS) in **v0.1.1378**; Credentials section keyboard hint first paint in theme HTML (at the end of the Credentials section, hidden until Settings is open, says how to move across those controls, Discord token first crosses to the Settings header, not injected after JS) in **v0.1.1377**; Slack settings toolbar keyboard hint first paint in theme HTML (under webhook · Save · Clear, hidden until Settings is open, says how to move across those controls, webhook first crosses to Telegram, Clear last crosses to header, not injected after JS) in **v0.1.1376**; Telegram settings toolbar keyboard hint first paint in theme HTML (under token · chat id · Save · Clear, hidden until Settings is open, says how to move across those controls, token first crosses to Cursor, Clear last crosses to Slack, not injected after JS) in **v0.1.1375**; Cursor agent settings toolbar keyboard hint first paint in theme HTML (under workspace · executable · Save · Clear, hidden until Settings is open, says how to move across those controls, workspace first crosses to Browser, Clear last crosses to Telegram, not injected after JS) in **v0.1.1374**; Browser settings toolbar keyboard hint first paint in theme HTML (under path · port · Save · Clear, hidden until Settings is open, says how to move across those controls, path first crosses to MCP, Clear last crosses to Cursor, not injected after JS) in **v0.1.1373**; MCP settings toolbar keyboard hint first paint in theme HTML (under URL · stdio · Save · Clear, hidden until Settings is open, says how to move across those controls, URL first crosses to Mastodon, Clear last crosses to Browser, not injected after JS) in **v0.1.1372**; Mastodon settings toolbar keyboard hint first paint in theme HTML (under URL · token · Save · Clear, hidden until Settings is open, says how to move across those controls, URL first crosses to Redmine, Clear last crosses to MCP, not injected after JS) in **v0.1.1371**; Redmine settings toolbar keyboard hint first paint in theme HTML (under URL · key · Save · Clear, hidden until Settings is open, says how to move across those controls, URL first crosses to Brave, Clear last crosses to Mastodon, not injected after JS) in **v0.1.1370**; Brave settings toolbar keyboard hint first paint in theme HTML (under key · Save · Clear, hidden until Settings is open, says how to move across those controls, key first crosses to Perplexity, Clear last crosses to Redmine, not injected after JS) in **v0.1.1369**; Discord settings toolbar keyboard hint first paint in theme HTML (under token · Save · Clear · View logs, hidden until Settings is open, says how to move across those controls, not injected after JS) in **v0.1.1368**; Perplexity settings toolbar keyboard hint first paint in theme HTML (under key · Save · Clear, hidden until Settings is open, says how to move across those controls, not injected after JS) in **v0.1.1367**; Ollama settings toolbar keyboard hint first paint in theme HTML (under system prompt · Reset · Save, hidden until the popover is open, says how to move across those controls, not injected after JS) in **v0.1.1366**; Ollama settings header keyboard hint first paint in theme HTML (between the title and Close, hidden until the title and Close are both on screen, says how to move across those controls, not injected after JS) in **v0.1.1365**; Settings header keyboard hint first paint in theme HTML (between the title and Close, hidden until the title and Close are both on screen, says how to move across those controls, not injected after JS) in **v0.1.1364**; Changelog header keyboard hint first paint in theme HTML (between the title and Close, hidden until the title and Close are both on screen, says how to move across those controls, not injected after JS) in **v0.1.1363**; Process Details header keyboard hint first paint in theme HTML (between the title and Close, hidden until the title and Close are both on screen, says how to move across those controls, not injected after JS) in **v0.1.1362**; Monitors add-form toolbar keyboard hint first paint in theme HTML (under URL · Cancel · Add Monitor, hidden until the add form is open, says how to move across those controls, not injected after JS) in **v0.1.1361**; Process Details Force Quit keyboard hint stays hidden until Advanced is open (Force Quit is not on screen while that section is closed) in **v0.1.1360**; Process Details Force Quit keyboard hint first paint in theme HTML (inside the force-quit section, hidden until Advanced and Force Quit are both on screen, says how to move across those controls, not injected after JS) in **v0.1.1359**; Process Details name and PID keyboard hint first paint in theme HTML (inside the hero, hidden until name and PID are both on screen, says how to move across those controls, not injected after JS) in **v0.1.1358**; Details keyboard hint first paint in theme HTML (above the Details grid, says how to move across those values, a collapsed section still hides the line, not injected after JS) in **v0.1.1357**; Disk Cleanup scope list keyboard hint first paint in theme HTML (above the scope list, hidden until rows exist, says how to move across those rows, not injected after JS) in **v0.1.1354**; Disk Cleanup category list keyboard hint first paint in theme HTML (above the category list, hidden until rows exist, says how to move across those rows, not injected after JS) in **v0.1.1353**; Monitors list keyboard hint first paint in theme HTML (above the monitor list, hidden until rows exist, says how to move across those rows, not injected after JS) in **v0.1.1352**; Top Processes list keyboard hint first paint in theme HTML (above the process list, hidden until rows exist, says how to move across those rows, not injected after JS) in **v0.1.1351**; Debug Log viewer keyboard hint first paint in theme HTML (above the log viewer, hidden until lines exist, says how to move across those lines, last line points to Refresh, not injected after JS) in **v0.1.1350**; Perplexity results keyboard hint first paint in theme HTML (above the result list, hidden until a search has results, says how to move across those results, not injected after JS) in **v0.1.1349**; Perplexity search-box keyboard hint first paint in theme HTML (under query · Search, says how to move across those controls before a search, not injected after JS) in **v0.1.1348**; Debug Log toolbar keyboard hint first paint in theme HTML (under Refresh · Open in editor · Auto-refresh, says how to move across those controls before a log load, not injected after JS) in **v0.1.1347**; Disk Cleanup add-scope toolbar keyboard hint first paint in theme HTML (under label · path · days · Recursive · Add scope, says how to move across those fields, not injected after JS) in **v0.1.1346**; Disk Cleanup action toolbar keyboard hint first paint in theme HTML (under Clean now · Refresh · Save scopes, says how to move across those buttons, not injected after JS) in **v0.1.1345**; Disk Cleanup meta keyboard hint first paint in theme HTML (under Reclaimable now · Next automatic run · Runs when · Enabled scopes, says how to move across those cards, not injected after JS) in **v0.1.1344**; Header toolbar keyboard hint first paint in theme HTML (under Refresh · Settings, says how to move across those buttons, not injected after JS) in **v0.1.1343**; Footer toolbar keyboard hint first paint in theme HTML (under the version chip and GitHub link, closed-window line, open panels still replace the line, not injected after JS) in **v0.1.1342**; Section icon-line keyboard hint first paint in theme HTML (under Monitors · AI Chat · Perplexity · Debug Log · Discord · Disk Cleanup · Agent Ops, says how to move across those icons, not injected after JS) in **v0.1.1341**; History sparkline keyboard hint first paint in theme HTML (under CPU · GPU · Freq · Temp charts, says how to move across those charts, not injected after JS) in **v0.1.1340**; Ring gauge keyboard hint first paint in theme HTML (under CPU · GPU · Freq · Temp, says how to move across those rings, not injected after JS) in **v0.1.1339**; Power strip keyboard hint first paint in theme HTML (under Bat · LPM · Power, says how to move across those chips, not injected after JS) in **v0.1.1338**; Agent Ops refresh-row keyboard hint first paint in theme HTML (under Refresh · Refresh digest · Updated, says how to move across those controls, not injected after JS) in **v0.1.1337**; Agent Ops health-strip keyboard hint first paint in theme HTML (under Version · Discord · Redmine · Next schedule · Last delivery · Digest, says how to move across those cards, not injected after JS) in **v0.1.1336**; Agent Ops overview keyboard hint first paint in theme HTML (under the overview cards, says how to move across Agents · Schedules · Live · Knowledge · Recent chats · Runs · Digest, not injected after JS) in **v0.1.1335**; Agent Ops filter-row keyboard hint first paint in theme HTML (under search · match · Clear, hidden until a search shows the match count and Clear, not injected after JS) in **v0.1.1334**; Agent Ops edit-actions keyboard hint first paint in theme HTML (under Save · Load into AI Chat · Back, says how to move across those actions, not injected after JS) in **v0.1.1333**; Agent Ops file-tab keyboard hint first paint in theme HTML (under Soul · Skill · Mood, says how to move across those tabs, not injected after JS) in **v0.1.1332**; Agent Ops tab-bar keyboard hint first paint in theme HTML (under the tabs, says how to move across Overview · Agents · Sessions · Schedules · Knowledge · Runs, not injected after JS) in **v0.1.1330**; Agent Ops Insights keyboard hint first paint in theme HTML (under the Insights card, hidden until two clickable insight lines are on screen, kept across a runs refresh, not injected after JS) in **v0.1.1329**; Agent Ops Knowledge preview-row keyboard hint first paint in theme HTML (under the knowledge preview, hidden until Copy and Load into AI Chat are both on screen, not injected after JS) in **v0.1.1328**; Agent Ops Schedules preview-row keyboard hint first paint in theme HTML (under the schedule preview, hidden until Copy and Load into AI Chat are both on screen, not injected after JS) in **v0.1.1327**; Agent Ops Sessions preview-row keyboard hint first paint in theme HTML (under the session preview, hidden until Copy and Load into AI Chat are both on screen, not injected after JS) in **v0.1.1326**; Agent Ops Runs preview-row keyboard hint first paint in theme HTML (under the run preview, hidden until Copy and Load into AI Chat are both on screen, not injected after JS) in **v0.1.1325**; Agent Ops Runs filter-chip keyboard hint first paint in theme HTML (under All · Instant · Lite · Direct · Slow · Fail, at start ↑ → Agent Ops icon, at end → run list, not injected after JS) in **v0.1.1324**; Agent Ops Knowledge filter-chip keyboard hint first paint in theme HTML (under All · Discord · Core, at start ↑ → Agent Ops icon, at end → knowledge list, not injected after JS) in **v0.1.1323**; Agent Ops Schedules filter-chip keyboard hint first paint in theme HTML (under All · Jobs · Deliveries, at start ↑ → Agent Ops icon, at end → schedule list, not injected after JS) in **v0.1.1322**; Agent Ops Sessions filter-chip keyboard hint first paint in theme HTML (under All · Live · Files, at start ↑ → Agent Ops icon, at end → session list, not injected after JS) in **v0.1.1321**; Agent Ops Agents filter-chip keyboard hint first paint in theme HTML (under All · On · Off, at start ↑ → Agent Ops icon, at end → agent list, not injected after JS) in **v0.1.1320**; Disk Cleanup category filter-chip keyboard hint first paint in theme HTML (under All · Reclaim · Big · Clean, at start ↑ → Disk Cleanup icon, at end → category list, not injected after JS) in **v0.1.1319**; Disk Cleanup scope filter-chip keyboard hint first paint in theme HTML (under All · On · Off, at start ↑ → Disk Cleanup icon, at end → scope list, not injected after JS) in **v0.1.1318**; Debug Log filter-chip keyboard hint first paint in theme HTML (under All · Error · Warn, at start ↑ → Debug Log icon, at end → log viewer, not injected after JS) in **v0.1.1317**; Perplexity filter-chip keyboard hint first paint in theme HTML (under All · Top · Snippet, at start ↑ → Perplexity icon, at end → result list, not injected after JS) in **v0.1.1316**; Monitors filter-chip keyboard hint first paint in theme HTML (under All · Up · Down · Slow, at start ↑ → Monitors icon, at end → monitor list, not injected after JS) in **v0.1.1315**; Top Processes filter-chip keyboard hint first paint in theme HTML (under All · Pinned · Hot, not injected after JS) in **v0.1.1314**; AI Chat filter-chip keyboard hint first paint in theme HTML (under All · You · Assistant · Errors, not injected after JS) in **v0.1.1313**; AI Chat starter-chip keyboard hint first paint in theme HTML (under the starter chips, not injected after JS) in **v0.1.1312**; AI Chat composer keyboard hint first paint in theme HTML (starter-chip line, not injected after JS) in **v0.1.1311**; AI Chat errors glance first paint in theme HTML (None yet, hidden until a failed turn exists, not injected after JS) in **v0.1.1310**; AI Chat last-answer glance first paint in theme HTML (None yet, hidden until a reply exists, not injected after JS) in **v0.1.1309**; AI Chat turn glance first paint in theme HTML (None yet, hidden until a turn exists, not injected after JS) in **v0.1.1308**; AI Chat collapsed glance first paint in theme HTML (Not set · configure URL, hidden until the section is collapsed, not injected after JS) in **v0.1.1307**; AI Chat offline attention glance first paint in theme HTML (Chat · Not set · configure URL, not injected after JS) in **v0.1.1306**; AI Chat model glance first paint in theme HTML (Not set · configure URL, not injected after JS) in **v0.1.1305**; AI Chat All · You · Assistant · Errors chips first paint in theme HTML (not injected after JS) in **v0.1.1304**; Debug Log All · Error · Warn chips first paint in theme HTML (not injected after JS) in **v0.1.1303**; Perplexity All · Top · Snippet chips first paint in theme HTML (not injected after JS) in **v0.1.1302**; Monitors All · Up · Down · Slow chips first paint in theme HTML (not injected after JS) in **v0.1.1301**; Top Processes All · Pinned · Hot chips first paint in theme HTML (not injected after JS) in **v0.1.1300**; Disk Cleanup category All · Reclaim · Big · Clean chips first paint in theme HTML (not injected after JS) in **v0.1.1299**; Disk Cleanup scope All · On · Off chips first paint in theme HTML (not injected after JS) in **v0.1.1298**; Disk Cleanup scope On and Off counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1297**; AI Chat You, Assistant, and Errors counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1296**; Perplexity Top and Snippet counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1295**; Debug Log Error and Warn counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1294**; Disk Cleanup Reclaim/Big/Clean counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1293**; Monitors Up/Down/Slow counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1292**; Top Processes Pinned/Hot counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1291**; Agent Ops kind-filter counts stay None yet when the count is zero (not 0 until a load) in **v0.1.1290**; Agent Ops filter match chips (N/M) and Clear stay hidden in the theme HTML until a query (not injected after JS) in **v0.1.1289**; Agent Ops tab counts stay None yet when the count is zero (not 0 after refresh) in **v0.1.1288**; Agent Ops Updated stamp first paint in theme HTML (None yet, not injected after a refresh) in **v0.1.1287**; Agent Ops overview head count pills first paint in theme HTML (None yet / Quiet / Queue clear, not injected after JS) in **v0.1.1286**; Agent Ops tab count pills first paint in theme HTML (None yet, not injected after JS) in **v0.1.1285**; Agent Ops Sessions All · Live · Files chips first paint in theme HTML (not injected after JS) in **v0.1.1284**; Agent Ops filter rows first paint (Agents All·On·Off, Schedules, Knowledge, Runs) in theme HTML (not injected after JS) in **v0.1.1283**; Agent Ops Refresh row first paint under the health cards (not jumped up from the bottom after JS) in **v0.1.1282**; Agent Ops tab strip first paint Overview + digit keys (not injected after JS) in **v0.1.1281**; Agent Ops overview Agents · Runs · Digest cards first paint in theme HTML (not injected after JS) in **v0.1.1280**; Apple CPU · Freq ring arcs match the track (not a short arc that misses the gauge) in **v0.1.1279**; Low Power Mode chip first paint in theme HTML (not injected after JS) in **v0.1.1278**; Ring labels first paint Freq / Temp (not Frequency / Temperature until JS) in **v0.1.1277**; GPU sparkline first paint in theme HTML (CPU · GPU · Freq · Temp, not a three-column jump) in **v0.1.1276**; AI Chat message list first paint Nothing here yet — set an Ollama URL (not a blank list) in **v0.1.1275**; Perplexity results first paint Nothing here yet — search the web (not a blank region) in **v0.1.1274**; Debug Log viewer first paint Nothing here yet (not Expand to load log…) in **v0.1.1273**; Disk Cleanup last-run panel first paint Not yet this install (not a blank panel) in **v0.1.1272**; Disk Cleanup scopes list first paint No scopes yet (not a blank list) in **v0.1.1271**; Disk Cleanup category list first paint Nothing to reclaim yet (not a blank list) in **v0.1.1270**; Top Processes list first paint waiting line (not a blank list) in **v0.1.1269**; Monitors list first paint Nothing watching yet (not a blank list) in **v0.1.1268**; Monitors summary first paint None yet (not 0 / 0 sites up) in **v0.1.1267**; Footer theme label first paint None yet (not fake v0.0.3) in **v0.1.1266**; Insights card first-paint None yet (not blank) in **v0.1.1265**; Sessions / Schedules / Knowledge / Runs list first-paint calm empty in **v0.1.1264**; Agents tab first-paint No agents yet (not blank list) in **v0.1.1263**; Ollama model select + Changelog first-paint None yet (not Loading models… / Loading changelog…) in **v0.1.1262**; Agent Ops Overview first-paint None yet (not Loading…) in **v0.1.1261**; LPM strip first-paint None yet (not …) in **v0.1.1260**; Details load/power first-paint None yet in **v0.1.1259**; CPU/GPU ring first-paint None yet (not 0%) in **v0.1.1258**; Perplexity header + Ollama model first-paint Unknown (not —) in **v0.1.1257**; Settings credentials first-paint Unknown (not —) in **v0.1.1256**; Disk Cleanup summary/reclaim + Monitors Avg first-paint None yet in **v0.1.1255**; Temp ring first-paint None yet (not 0°C) in **v0.1.1253**; Chip/CPU·GPU subtext/Details RAM·Up first-paint Unknown/None yet in **v0.1.1252**; Bat strip no-battery None yet (not N/A) in **v0.1.1251**; Temp/Freq ring + Power empty None yet (not — / -- W) in **v0.1.1250**; Power strip empty Heat/Up/RAM/SSD None yet (not —) in **v0.1.1249**; Details collapsed glance empty Load/RAM/Up None yet (not —) in **v0.1.1248**; Top GPU empty metric None yet (not —) in **v0.1.1247**; Monitors summary Avg None yet + settings empty identity in **v0.1.1246**; Monitors empty identity Unknown / None yet in **v0.1.1245**; Disk Cleanup empty identity Unknown / None yet in **v0.1.1244**; Top Processes empty identity Unknown / None yet in **v0.1.1243**; AI Chat empty glance preview say None yet in **v0.1.1242**; Health Version empty uptime say None yet in **v0.1.1241**; Health Digest empty age say None yet in **v0.1.1240**; Health Last delivery empty summary preview say None yet in **v0.1.1239**; Health Next schedule empty task preview say None yet in **v0.1.1238**; Live/Sessions empty preview say None yet in **v0.1.1237**; Knowledge/Sessions/Live empty size/lines/msgs say Unknown in **v0.1.1236**; Schedule/delivery empty task/summary say None yet (lists) in **v0.1.1235**; Delivery empty schedule id say Unknown (not placeholder `schedule`) in **v0.1.1234**; Insights Slowest empty wall / lane / question say Unknown / None yet (not undefined ms / blank) in **v0.1.1233**; Insights Candidates empty kind / reason / question say Unknown / None yet (not blank) in **v0.1.1232**; Runs / Schedules empty question / id say None yet / Unknown (not `(empty)` / `(no id)`) in **v0.1.1231**; Knowledge / Sessions empty file title say Unknown (not blank) in **v0.1.1230**; Agents empty name / slug say Unknown (not blank) in **v0.1.1229**; Live empty source / session id say Unknown (not blank) in **v0.1.1228**; Sessions / Knowledge empty meta say Unknown (not blank) in **v0.1.1227**; Disk Cleanup empty meta say None yet (not em dash) in **v0.1.1226**; Delivery empty when say None yet (not em dash) in Overview / Schedules / preview in **v0.1.1225**; Runs empty meta say Unknown / None yet (not em dash) in Overview / Runs / Slowest / preview in **v0.1.1224**; Schedule empty next/when say None yet (not em dash) in Overview / Schedules / preview in **v0.1.1223**; Redmine unloaded health says Unknown (not em dash) in **v0.1.1222**; Health unloaded Version / Next schedule / Last delivery / Digest say Unknown (not em dash) in **v0.1.1221**; Discord empty health/glance say Unknown (not em dash) in **v0.1.1220**; Health Next schedule / Last delivery empty say None yet (overview head parity) in **v0.1.1219**; Overview empty head pills say None yet / Quiet (Digest Queue clear parity) in **v0.1.1218**; `/insights` empty sections say no lanes yet, no tools yet, nothing slow, nothing open in **v0.1.1217**; Insights fail zero says no fails (header, `/insights`, how many runs) in **v0.1.1216**; CPU ring labels say Freq and Temp (sparkline caption match) in **v0.1.1214**; Digest zero counts say queue clear / nothing stale (health, overview, Insights header) in **v0.1.1213**; Insights header mean/max n/a when the latency sample is empty in **v0.1.1212**; Insights Lanes empty calm in **v0.1.1211**; Insights Top tools empty calm in **v0.1.1210**; Insights Latency empty calm in **v0.1.1209**; Insights Stale empty calm in **v0.1.1208**; Insights Candidates empty calm in **v0.1.1207**; Insights Slowest empty calm in **v0.1.1206**; Digest open empty calm in Insights (**v0.1.1205**); Agent Ops true-empty calm in **v0.1.1204**; AI Chat last-answer glance calm in **v0.1.1103**; CPU · GPU · FREQ ring calm in **v0.1.1096**; Temp ring Nominal calm in **v0.1.1095**; Perplexity collapsed glance calm in **v0.1.1094**; Debug Log collapsed glance calm in **v0.1.1093**; Disk Cleanup collapsed glance calm in **v0.1.1092**; Top Processes glance calm in **v0.1.1091**; Details collapsed glance calm in **v0.1.1090**; Debug Log filter-miss calm in **v0.1.1088**; Follow `docs/043_overnight_design_review.md`. Prefer stale feature screens (`feature-agent-ops`, `feature-ai-chat`, `feature-processes`) before re-shooting CPU. Disk Cleanup filter-miss calm in **v0.1.1086**; Monitors filter-miss calm in **v0.1.1085**; Top Processes filter-miss calm in **v0.1.1084**; Agent Ops filter-miss calm (warm title + accent wash; Fail empty green) in **v0.1.1073**; CPU metrics Temp ring Fair thermal wash (Serious/Critical → hot under 70°C) in **v0.1.1067**; Agent Ops Schedules filter Clear chip beside All·Jobs·Deliveries in **v0.1.1016**; Agent Ops Knowledge filter Clear chip beside All·Discord·Core in **v0.1.1015**; Agent Ops Runs filter Clear chip beside All·Instant·Lite·Direct·Slow·Fail in **v0.1.1014**; Agent Ops Agents filter Clear chip beside All·On·Off in **v0.1.1013**; Agent Ops Sessions filter Clear chip beside All·Live·Files in **v0.1.1012**; Disk Cleanup scopes filter Clear chip beside All·On·Off in **v0.1.1011**; Perplexity Search filter Clear chip beside All·Top·Snippet in **v0.1.1010**; Debug Log filter Clear chip beside All·Error·Warn in **v0.1.1009**; Disk Cleanup filter Clear chip beside All·Reclaim·Big·Clean in **v0.1.998**; External / Monitors filter Clear chip beside All·Up·Down·Slow in **v0.1.997**; Top Processes filter Clear chip beside All·Pinned·Hot in **v0.1.996**; AI Chat filter Clear chip beside All·You·Assistant·Errors in **v0.1.960**; Ops chip Clear gaps remain: none (Schedules Clear in v0.1.1016); AI Chat filter-miss calm (You/Assistant/Errors empty wash + warm title) in **v0.1.942**; AI Chat empty Ready calm (connected+model → ok wash + warm empty copy) in **v0.1.938**; Overview Live idle calm (Ready → ok wash + warm empty copy) in **v0.1.919**; Digest empty-state polish in **v0.1.276**; refresh-button polish in **v0.1.278**; process-list polish in **v0.1.280** / keyboard+focus in **v0.1.298**; Agent Ops tab hover/focus in **v0.1.285**; Ops filter focus ring in **v0.1.287**; overview card hover in **v0.1.288** / focus-within in **v0.1.317**; list-row hover in **v0.1.296**; AI chat input in **v0.1.295** / composer glass + accents in **v0.1.370**; health-card keyboard nav in **v0.1.316**; overview active-tab + health wash in **v0.1.430**; active-tab accent wash + badge glass in **v0.1.431**; selected-row accent wash in **v0.1.447**; Sessions copy id/slug chip in **v0.1.451**; Schedules/deliveries click-to-preview in **v0.1.452**; Knowledge path copy chip in **v0.1.453**; Runs click-to-preview in **v0.1.468**; Agents copy id/slug chip in **v0.1.469**; Data Poster inactive-icon contrast in **v0.1.470**; Runs request-id copy chip in **v0.1.471**; Schedules/delivery id copy chip in **v0.1.472**; Runs Load into AI Chat in **v0.1.473**; Schedules Load into AI Chat in **v0.1.474**; Knowledge Load into AI Chat in **v0.1.475**; Agents Load into AI Chat in **v0.1.476**; overview Schedules click-to-preview in **v0.1.477**; overview Recent click-to-preview in **v0.1.478**; overview Live click-to-preview in **v0.1.479**; overview Knowledge click-to-preview (row select) in **v0.1.480**; overview Last delivery click-to-preview in **v0.1.481**; Runs Insights Slowest/Candidates click-to-preview in **v0.1.482**; health Next schedule / Last delivery click-to-preview in **v0.1.483**; Digest open hints + health Digest click-to-preview in **v0.1.484**; health Version → primary agent open in **v0.1.485**; overview Agents card in **v0.1.488**; overview Runs card in **v0.1.489**; overview Digest card in **v0.1.490**; health schedule/delivery ok/warn/bad wash in **v0.1.491**; health Version ok/warn/bad wash in **v0.1.492**; overview Schedules ok/warn/bad wash in **v0.1.493**; overview Agents ok/warn/bad wash in **v0.1.494**; overview Runs ok/warn/bad wash in **v0.1.495**; overview Digest ok/warn/bad wash in **v0.1.496**; overview Live ok/warn/bad wash in **v0.1.497**; overview Knowledge ok/warn/bad wash in **v0.1.498**; overview Recent ok/warn/bad wash in **v0.1.499**; overview cards click/keyboard open linked tab in **v0.1.500**; tab inventory count pills in **v0.1.503**; Refresh/Updated under health in **v0.1.504**; filter N/M chips in **v0.1.505**; filter-row Clear beside N/M in **v0.1.506**; overview head count pills in **v0.1.507**; 0 Overview jump in **v0.1.508**; Top Processes click-to-copy name in **v0.1.509**; Agent Ops `c` copy id in **v0.1.510**; Monitors `c` URL in **v0.1.511**; Disk Cleanup path copy in **v0.1.512**; CPU ring value copy in **v0.1.513** / CPU % Details toggle restore in **v0.1.516**; AI Chat starter chips in **v0.1.514** / starter chip In composer flash in **v0.1.527**; Debug Log Error/Warn filters in **v0.1.515**; Debug Log error/warn glance in **v0.1.533**; battery/power strip click-to-copy in **v0.1.517**; Monitors summary click + empty Add CTA in **v0.1.518**; Agent Ops empty Open AI Chat in **v0.1.519**; CPU metrics RAM strip in **v0.1.520**; CPU metrics GPU strip in **v0.1.534**; Temp °C strip in **v0.1.535**; frequency GHz strip in **v0.1.536**; SSD % strip in **v0.1.537**; CPU % strip in **v0.1.538**; AI Chat last-answer glance (copy) in **v0.1.539**; Monitors All/Up/Down filter chips in **v0.1.521**; Top Processes All/Pinned filter chips in **v0.1.522** / Hot in **v0.1.686**; Disk Cleanup empty Review scopes CTA in **v0.1.523**; Disk Cleanup Reclaimable now meta-card click in **v0.1.524**; Disk Cleanup Enabled scopes meta-card click in **v0.1.525**; Disk Cleanup Next automatic run meta-card click in **v0.1.526**; Disk Cleanup Runs when meta-card click in **v0.1.528**; Disk Cleanup Last run panel click in **v0.1.529**; Top Processes Top CPU glance in **v0.1.531**; Top Processes Top GPU glance in **v0.1.541**; Top Processes Top RAM glance in **v0.1.547**; AI Chat turn glance in **v0.1.532**; Agent Ops Knowledge All·Discord·Core chips in **v0.1.546**; Disk Cleanup All·Reclaim·Clean chips in **v0.1.548**; AI Chat All·You·Assistant chips in **v0.1.549**; CPU metrics uptime strip in **v0.1.550**; Perplexity last-search glance in **v0.1.551** / All·Top·Snippet filter in **v0.1.692**; Monitors collapsed glance in **v0.1.552**; Disk Cleanup collapsed glance in **v0.1.553**; AI Chat collapsed glance in **v0.1.554**; Agent Ops Discord Ready collapsed glance in **v0.1.555**; Perplexity collapsed keep-header in **v0.1.556**; Debug Log collapsed keep-header in **v0.1.557**; Top Processes collapsed keep-header in **v0.1.558**; Details collapsed keep-header in **v0.1.559**; CPU metrics Heat/thermal strip in **v0.1.560**; Heat prefers **NSProcessInfo.thermalState** in **v0.1.561**; menu-bar LPM in **v0.1.563**; menu-bar Heat Serious/Critical in **v0.1.564**; menu-bar SSD ≥85% amber in **v0.1.567**; menu-bar RAM ≥85% amber + strip hot wash in **v0.1.568**; menu-bar CPU ≥50% amber in **v0.1.569**; menu-bar GPU ≥15% amber in **v0.1.570**; menu-bar Temp ≥70°C amber in **v0.1.571**; Top Processes Filter attention glance in **v0.1.842**; Monitors Filter attention glance in **v0.1.843**; Disk Cleanup Filter attention glance in **v0.1.844**; Agent Ops Filter attention glance (On/Off · Live/Files · Jobs/Deliveries · Discord/Core · Runs lanes) in **v0.1.865**.
    4. ~~**README / landing**~~ — sharper vs-competitor framing (**v0.1.265**).

## P2 — reliability

5. **`debug.log` errors** — First recurring error/panic in the last 24h that is product-owned. Idle-thought Ollama timeout WARN is one line per 5 min even in a same-second burst (compare-exchange) in **v0.1.1439**. Ollama circuit-open WARN ≤1/5min + model-list fail cooldown 5m + shared waiter single-log in **v0.1.1397** (log-012); model-list fetch fail WARN rate-limit + 30s cooldown in **v0.1.1388** (log-012). WebView idle cut follow-up for GitHub **#14** in **v0.1.1683** (Apple Monitors settings popover shell `.monitors-settings-popover .popover-content` uses an opaque fill. No glass alpha on the popover panel or soft drop shadow. Close focus ring mixes against opaque. Perplexity search box opaque in v0.1.1682). Prior **v0.1.1682** (Apple Perplexity search box `.perplexity-search-box input` · `button` resting · hover · focus-visible mix against an opaque fill. No glass alpha on the query field, Search control, or focus rings. Model select opaque in v0.1.1681). Prior **v0.1.1681** (Apple AI Chat model select `.model-select` resting · hover · focus · focus-visible mix against an opaque fill. No glass alpha on the field, hover border, or focus ring. Send control opaque in v0.1.1680). Prior **v0.1.1680** (Apple AI Chat Send control `#chat-send-btn` resting · hover · focus-visible · active mix against an opaque fill. No glass alpha on the Send fill, soft hover shadow, or focus ring. Composer field opaque in v0.1.1679). Prior **v0.1.1679** (Apple AI Chat composer field `#chat-input` resting · hover · focus mix against an opaque fill. No glass alpha on the field, hover border, or focus ring. Composer shell opaque in v0.1.1678). Prior **v0.1.1678** (Apple AI Chat composer shell `.chat-input-container` resting · `:focus-within` mix against an opaque fill. No glass alpha on the composer shell or focus ring. Message bubbles opaque in v0.1.1677). Prior **v0.1.1677** (Apple AI Chat message bubbles `.chat-message.user` · `.chat-message.assistant` mix against an opaque fill. No glass alpha on the user/assistant bubbles or accent borders. Message list shell opaque in v0.1.1676). Prior **v0.1.1676** (Apple AI Chat message list `.chat-messages` resting · `:focus-within` mix against an opaque fill. No glass alpha on the list shell or focus ring. Empty shell opaque in v0.1.1666). Prior **v0.1.1675** (Top Processes usage bar fills `.process-bar-fill` mix against an opaque fill. No glass alpha on the bar fill. The shared sheet was already opaque; the Apple theme had put glass back). Prior **v0.1.1674** (Top Processes usage bar tracks `.process-bar` / `#process-list .process-bar` mix against an opaque fill. No glass alpha on the bar track. Shared sheet and Apple theme both updated). Prior **v0.1.1673** (Process Details metric row hairlines `.process-detail-row` border-bottom mix against an opaque fill. No glass alpha on the row dividers. Shared sheet and Apple theme both updated). Prior **v0.1.1672** (Force Quit control `.force-quit-btn` resting · hover · focus · active · `.is-confirming` (+ section hairline) mix the wash against an opaque fill. No glass alpha on the Force Quit control. Shared confirming wash and Apple theme both updated). Prior **v0.1.1671** (Process Details panel `.process-detail-hero` · `.process-detail-section` mix the wash against an opaque fill. No glass alpha on the hero or metric sections. Shared sheet and Apple theme both updated). Prior **v0.1.1670** (Monitors detail panel `.monitor-detail` · `.monitor-detail-log` mix the wash against an opaque fill. No glass alpha on the expanded detail shell or log pad). Prior **v0.1.1669** (Monitors row selected · focus `.monitor-item.is-selected` · `:focus-visible` mix the wash against an opaque fill. No glass alpha on the selection wash or focus ring). Prior **v0.1.1668** (Monitors Down · Slow rows `.monitor-item.is-down` · `.is-slow` resting · hover mix the wash against an opaque fill. No glass alpha on the status row). Prior **v0.1.1667** (Apple Monitors rows `.monitor-item` resting · hover mix the wash against an opaque fill. No glass alpha on the row. Hover drops the soft glass shadow). Prior **v0.1.1666** (Apple AI Chat empty shell `.chat-empty` resting · hover mixes the wash against an opaque fill. No glass alpha on the empty list. The shared sheet was already opaque; the Apple theme had put glass back). Prior **v0.1.1665** (Apple Monitors empty shell `.monitors-empty` resting · hover · error mix the wash against an opaque fill. No glass alpha on the empty list). Prior **v0.1.1664** (Monitors filter-miss `.monitors-filter-miss` · Down · Slow · Up empty + empty CTA `.monitors-empty-cta` resting · hover · focus mix the wash against an opaque fill. No glass alpha on the filter-miss shell or Add Monitor CTA). Prior **v0.1.1663** (Disk Cleanup soft-delete `.disk-cleanup-soft-delete` resting · hover · focus-within mixes the wash against an opaque fill. No glass alpha on the Move to Trash row). Prior **v0.1.1662** (Apple Top Processes empty shell `.process-empty` resting · hover mixes the wash against an opaque fill. No glass alpha on the empty list). Prior **v0.1.1661** (Apple Settings card `.settings-card` + `.settings-header` hairline use an opaque fill. No glass `--panel` alpha on the modal shell. Soft panel drop shadow dropped). Prior **v0.1.1660** (Apple Settings toggles `.setting-toggle input[type="checkbox"]` resting · checked · knob mix the track against an opaque fill. No glass alpha on the switch. Soft knob drop shadow dropped). Prior **v0.1.1659** (Apple Settings help sheet `.settings-help-sheet` resting · focus · Copied mixes the wash against an opaque fill. No glass alpha on the cheat-sheet panel, focus ring, or Copied flash). Prior **v0.1.1658** (Apple theme list `.theme-item` resting · hover · focus · current mixes the wash against an opaque fill. No glass alpha on the theme button. Hover drops the soft glass shadow). Prior **v0.1.1657** (Disk Cleanup primary toolbar `.disk-cleanup-toolbar .disk-cleanup-primary` resting · hover mix the fill against an opaque fill. No glass alpha on the Clean now / primary wash). Prior **v0.1.1656** (Apple Settings buttons `.settings-btn` · `.settings-btn-primary` resting · hover mix the fill against an opaque fill. No glass alpha on the button wash). Prior **v0.1.1655** (Disk Cleanup scope filter-miss `.disk-cleanup-scope-filter-miss` · On · Off empty opaque wash). Prior **v0.1.1654** (Apple Settings inputs `.settings-input` · `.discord-token-input` resting · hover mix the fill against an opaque fill. No glass alpha on the field or the hover border). Prior **v0.1.1653** (Disk Cleanup filter-miss `.disk-cleanup-filter-miss` · Reclaim · Big · Clean · empty CTA `.disk-cleanup-empty-cta` resting · hover · focus opaque wash). Prior **v0.1.1652** (Disk Cleanup empty shell `.disk-cleanup-empty` opaque wash). Prior **v0.1.1651** (Disk Cleanup last-run panel `.disk-cleanup-last` resting · hover · focus · has-skip · is-ok opaque wash). Prior **v0.1.1650** (Apple Settings inputs `.settings-input:focus` · `.discord-token-input:focus` mix the focus ring against an opaque fill). Prior **v0.1.1649** (Disk Cleanup meta cards `.disk-cleanup-meta-card` resting · hover · focus · Reclaim · Clean · scopes · due · periodic opaque wash; hover soft glass shadow dropped). Prior **v0.1.1648** (Disk Cleanup category and scope rows resting · hover · focus · selected · Reclaim · Big opaque wash; hover soft glass shadow dropped). Prior **v0.1.1647** (Disk Cleanup Copied badge `.disk-cleanup-item` / `.disk-cleanup-scope-row` `is-just-copied` `::after` opaque wash). Prior **v0.1.1646** (Apple history time-range `.time-range-dropdown:focus` opaque wash). Prior **v0.1.1645** (Monitors row Copied badge `.monitor-item.is-just-copied` `::after` opaque wash). Prior **v0.1.1644** (Apple power strip `.battery-power-strip:focus-within` + Bat/LPM attention flash rings opaque wash). Prior **v0.1.1643** (footer GitHub `#github-link` hover · focus-visible + Apple `.app-version:focus-visible` · `.apple-github-link:focus-visible` opaque wash). Prior **v0.1.1642** (Agent Ops row Copied badge `.ops-row.is-copied` `::after` opaque wash). Prior **v0.1.1641** (Apple section strip `.icon-line-item:focus-visible` opaque wash). Prior **v0.1.1640** (Apple icon strip `.icon-btn` hover · focus-visible · active opaque wash). Prior **v0.1.1639** (Details value Copied badge `.details-grid > .detail-value[role='option'].is-just-copied` `::after` opaque wash). Prior **v0.1.1638** (ring focus `#cpu-usage-card:focus-visible` · `.metric-card:focus-within` opaque wash; soft glass focus shadows dropped). Prior **v0.1.1637** (Details / Top Processes `.collapsible-header` hover · focus-visible opaque wash). Prior **v0.1.1636** (Top Processes row Copied badge `.process-row.is-just-copied` `::after` opaque wash). Prior **v0.1.1635** (collapsible section headers `.section-header-collapsible` hover · focus-visible opaque wash). Prior **v0.1.1634** (Debug Log lines `.logs-line` hover · selected opaque wash). Prior **v0.1.1633** (Perplexity result Copied badge `.perplexity-result-item` `::after` opaque wash). Prior **v0.1.1632** (Debug Log collapsed Error/Warn glance `.logs-error-glance` · Quiet · hover opaque wash; soft glass hover shadow dropped). Prior **v0.1.1631** (Debug Log filter-miss `.logs-viewer-empty.logs-filter-miss` · Error · Warn · Clear filter opaque wash). Prior **v0.1.1630** (Debug Log chrome `.logs-toolbar` · buttons · `.logs-viewer` · path focus opaque wash). Prior **v0.1.1629** (Perplexity weather card `.perplexity-weather-card` opaque wash). Prior **v0.1.1628** (Perplexity empty / filter-miss `.perplexity-empty` · error · Top · Snippet · Clear filter opaque wash). Prior **v0.1.1627** (Perplexity result rows `.perplexity-result-item` resting · hover · focus · selected · Top opaque wash). Prior **v0.1.1626** (Top Processes empty shell `.process-empty` opaque wash). Prior **v0.1.1625** (Top Processes filter-miss `.processes-filter-miss` · Hot · Pinned · Clear filter opaque wash). Prior **v0.1.1624** (Ring filter-miss `.rings-filter-miss` · CTA opaque wash). Prior **v0.1.1623** (AI Chat filter-miss / Clear opaque wash). Prior **v0.1.1622** (AI Chat error bubbles `.chat-message.assistant.is-error` opaque wash). Prior **v0.1.1621** (AI Chat exec / answer cards `.chat-exec-card` · `.chat-exec-code` · `.chat-answer-part` · final opaque wash). Prior **v0.1.1620** (AI Chat composer `#chat-input:focus` · `#chat-send-btn` resting · hover opaque wash / no soft glass Send shadow). Prior **v0.1.1619** (AI Chat message rows hover · focus · selected · Copied badge opaque wash). Prior **v0.1.1618** (AI Chat empty starter chips `.chat-empty-chip` resting · hover · focus-visible opaque wash). Prior **v0.1.1617** (Agent Ops refresh row Refresh · Updated stamp · top hairline opaque wash). Prior **v0.1.1616** (Agent Ops agent editor `textarea.ops-agent-editor` focus · dirty opaque wash). Prior **v0.1.1615** (AI Chat empty shell default · Ready · no model · offline · not set opaque wash). Prior **v0.1.1614** (Agent Ops detail preview `.ops-preview` opaque wash). Prior **v0.1.1613** (Agent Ops close `.ops-close-btn` resting · hover opaque wash). Prior **v0.1.1612** (Agent Ops loading shell opaque wash). Prior **v0.1.1611** (Agent Ops Overview Open `.ops-overview-link` resting · hover · focus-visible opaque wash). Prior **v0.1.1610** (Agent Ops copy-chip `.ops-session-copy-chip` resting · hover · focus-visible opaque wash). Prior **v0.1.1609** (Agent Ops empty panel + Clear filter button opaque wash). Prior **v0.1.1608** (Agent Ops On · Off `.ops-badge` resting · hover · focus-visible · off opaque wash). Prior **v0.1.1607** (Agent Ops base `.ops-row` resting · hover · focus-visible · selected opaque wash). Prior **v0.1.1606** (Agent Ops Runs list rows Lite · Slow · Fail opaque wash). Prior **v0.1.1605** (Agent Ops Runs lane All · Instant · Lite · Direct · Slow · Fail filter chips + Clear opaque wash). Prior **v0.1.1604** (Agent Ops Knowledge All · Discord · Core filter chips + Clear opaque wash). Prior **v0.1.1603** (Agent Ops Schedules All · Jobs · Deliveries filter chips + Clear opaque wash). Prior **v0.1.1602** (Agent Ops Agents All · On · Off filter chips + Clear opaque wash). Prior **v0.1.1601** (Agent Ops Sessions All · Live · Files filter chips + Clear opaque wash). Prior **v0.1.1600** (Agent Ops tab strip / count pills opaque wash). Prior **v0.1.1599** (ring, battery, and power Copied flashes sit above the value with left and right; no translate). Prior **v0.1.1598** (Agent Ops filter input, match chip, Clear, and just-cleared flash opaque wash). Prior **v0.1.1597** (Disk Cleanup scope filter chips All · On · Off + Clear opaque wash). Prior **v0.1.1596** (Disk Cleanup category filter chips All · Reclaim · Big · Clean + Clear opaque wash). Prior **v0.1.1595** (Debug Log filter chips All · Error · Warn + Clear opaque wash). Prior **v0.1.1594** (monitor history tick tips use left/top; opaque fill; no translate or shadow). Prior **v0.1.1593** (Perplexity filter chips All · Top · Snippet + Clear opaque wash). Prior **v0.1.1592** (AI Chat filter chips All · You · Assistant · Errors + Clear opaque wash). Prior **v0.1.1591** (Agent Ops overview cards opaque wash). Prior **v0.1.1590** (Rings filter chips All · Hot opaque wash). Prior **v0.1.1589** (External / Monitors filter chips + Clear opaque wash). Prior **v0.1.1588** (Agent Ops health cards opaque wash). Prior **v0.1.1587** (Top Processes filter chips + Clear opaque wash). Prior **v0.1.1586** (Top Processes pin hover/focus opaque wash). Prior **v0.1.1585** (Top Processes row pinned/hover/focus/active/selected opaque wash). Prior **v0.1.1584** (Details value hover/focus/selected opaque wash). Prior **v0.1.1583** (ring/power-strip copy hover/focus opaque wash). Prior **v0.1.1582** (ring/power-strip Copied flash opaque wash). Prior **v0.1.1581** (Agent Ops row Copied flash opaque wash). Prior **v0.1.1580** (AI Chat message Copied flash opaque wash). Prior **v0.1.1579** (Debug Log line Copied flash opaque wash). Prior **v0.1.1578** (Perplexity result Copied flash opaque wash). Prior **v0.1.1577** (Disk Cleanup row Copied flash opaque wash). Prior **v0.1.1576** (Monitors row Copied flash opaque wash). Prior **v0.1.1575** (Top Processes row Copied flash opaque wash). Prior **v0.1.1574** (Details value Copied flash opaque wash). Prior **v0.1.1573** (Settings product-toggle Saved flash opaque wash). Prior **v0.1.1572** (shared Save / secondary-button Saved flash opaque wash). Prior **v0.1.1529** (Settings Help Saved flash opaque wash). Prior **v0.1.1528** (footer GitHub Saved flash opaque wash). Prior **v0.1.1527** (header Refresh Saved flash opaque wash). Prior **v0.1.1526** (header Refresh button no longer spins while fetching). Prior **v0.1.1525** (Settings Having fun Off opaque wash). Prior **v0.1.1524** (Settings Ori Mnemos Off opaque wash). Prior **v0.1.1523** (Settings Downloads organizer Off opaque wash). Prior **v0.1.1522** (Settings Judge Off opaque wash). Prior **v0.1.1521** (Settings Voice STT glance opaque wash). Prior **v0.1.1520** (Settings Compact On opaque wash). Prior **v0.1.1519** (Settings AI Off opaque wash). Prior **v0.1.1518** (Settings Signal not-wired opaque wash). Prior **v0.1.1517** (Settings Help glance opaque wash). Prior **v0.1.1516** (Settings Slack not-set/partial opaque wash). Prior **v0.1.1515** (Settings Telegram not-set/partial opaque wash). Prior **v0.1.1514** (Settings Cursor agent not-set opaque wash). Prior **v0.1.1513** (Settings Browser / CDP not-set opaque wash). Prior **v0.1.1512** (Settings Discord token opaque wash). Prior **v0.1.1511** (Settings MCP not-set opaque wash). Prior **v0.1.1510** (Settings Mastodon not-set/partial opaque wash). Prior **v0.1.1509** (Settings Redmine not-set/partial opaque wash). Prior **v0.1.1508** (Settings Brave Key-not-set opaque wash). Prior **v0.1.1507** (Settings Perplexity key opaque wash). Prior **v0.1.1506** (Perplexity last-search opaque wash). Prior **v0.1.1505** (Perplexity Top/error/filter opaque wash). Prior **v0.1.1504** (Perplexity Key-not-set opaque wash). Prior **v0.1.1503** (External / Monitors summary opaque wash). Prior **v0.1.1502** (Agent Ops Signal Not wired/Partial opaque wash). Prior **v0.1.1501** (Agent Ops Slack Not set/Partial opaque wash). Prior **v0.1.1500** (Agent Ops Telegram Not set/Partial opaque wash). Prior **v0.1.1499** (Agent Ops Mastodon Not set/Partial opaque wash). Prior **v0.1.1498** (Agent Ops Perplexity Search Not set/Unavailable/Degraded opaque wash). Prior **v0.1.1497** (Agent Ops Cursor Not set/Unavailable/Degraded opaque wash). Prior **v0.1.1496** (Agent Ops MCP Not set/Unavailable/Degraded opaque wash). Prior **v0.1.1495** (Agent Ops Browser CDP opaque wash). Prior **v0.1.1494** (Agent Ops Brave Search Not set/Unavailable/Degraded opaque wash). Prior **v0.1.1493** (Agent Ops Ollama Not set/Offline/Degraded opaque wash). Prior **v0.1.1492** (Agent Ops Redmine Not set/Degraded/Unavailable opaque wash). Prior **v0.1.1491** (Agent Ops Discord Offline/Reconnect opaque wash). Prior **v0.1.1490** (Agent Ops Digest opaque wash). Prior **v0.1.1489** (Agent Ops Filter opaque wash). Prior **v0.1.1488** (Agent Ops Runs Fail/Slow opaque wash). Prior **v0.1.1487** (Debug Log Error/Warn opaque wash). Prior **v0.1.1408** (Tauri `Focused` park/resume; no sparkline history IPC seed on boot; ring skip ~70%; chart-line boot idle 180s; GPU warm 1800s; still open until macOS webview stays under ~1%). Prior **v0.1.1407** (`display: none` body park; ring ~60%; boot 120s; GPU warm 960s). Prior **v0.1.1400** (600s UI polls/TTL). WebView compositor follow-up in **v0.1.1387**. Discord idle-thought 503 safe retry in **v0.1.1107**; Having-fun idle Ollama timeout soft path in **v0.1.1083** (120s wall · no outer retry · WARN + backoff). Debug Log inventory counts in **v0.1.1203**. In-app Debug Log Error/Warn filter chips in **v0.1.515**. Brave health-ping quota burn mitigated in **v0.1.272**. Website monitor DNS/connect failures classify to short reasons in **v0.1.375** (UI + log). DOWN recheck backoff (DNS ≥5 min) in **v0.1.376**. DOWN next-check countdown in UI in **v0.1.377**. Unchanged-UP `monitors.json` rewrite throttle (~5 min) in **v0.1.378**. Unchanged UP/DOWN recheck logs → TRACE in **v0.1.379**. Install refuses stale release binary in **v0.1.380**. Idle task-review scan / no-open → DEBUG when empty in **v0.1.382**. Monitors summary names DOWN hosts + short reasons in **v0.1.383**. Monitor last-check age + DOWN-first list sort + slowest-host summary in **v0.1.384**. Monitors Arrow/Home/End + Enter check-now in **v0.1.385**; j/k + Esc clear selection in **v0.1.392**; `d` detail toggle + Esc closes detail first in **v0.1.402**. Disk Cleanup scope keyboard in **v0.1.386**; category keyboard + Enter Clean now in **v0.1.387**; Delete/Backspace removes custom scopes in **v0.1.388**; Enter-to-add + ⌘/Ctrl+S save in **v0.1.389**; `R` toggles Recurse in **v0.1.390**; `T` toggles Trash soft-delete in **v0.1.391**.
6. **Discord / LaunchAgent uptime** — Confirm process + Discord Ready after any install; fix silent downtime causes. Single-instance busy WARN rate-limit (**v0.1.381**) cuts KeepAlive thrash noise in `debug.log`.
## P3 — sibling ports

7. OpenClaw / Hermes ports that clearly map to mac-stats tools/sessions (not docs-only Related sections). Google SERP FETCH_URL→search rewrite shipped in **v0.1.281**. Insights/status/digest/schedules/scrub/`/help`/interrupt NL in **v0.1.306–315**. Discord voice STT harden in **v0.1.313**. Climate/clima/klima → Open-Meteo + Brave-weather→Perplexity redirect in **v0.1.319–321**.



## Overnight merge — v0.1.1817

- Apple Perplexity results keyboard hint (`.perplexity-kb-hint`) mixes type color against an opaque white results fill (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the results-list move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Chat kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1816

- Apple AI Chat message-list keyboard hint (`.chat-kb-hint`) mixes type color against an opaque white message-list fill (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the All · You · Assistant · list move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Composer kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1815

- Apple AI Chat composer keyboard hint (`.chat-composer-kb-hint`) mixes type color against an opaque white composer fill (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the input · Clear · Send move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Starter-chip kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1814

- Apple AI Chat empty starter keyboard hint (`.chat-empty-kb-hint`) mixes type color against an opaque white message-list fill (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the starter-chip move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Insights kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1813

- Apple Agent Ops Insights keyboard hint (`.ops-insights-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Insights move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Preview-row kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1812

- Apple Agent Ops preview-row keyboard hint (`.ops-preview-row-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Copy · Load into AI Chat move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Edit-actions kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1811

- Apple Agent Ops edit-actions keyboard hint (`.ops-agent-edit-actions-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Save · Load into AI Chat · Back move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. File-tab kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1810

- Apple Agent Ops file-tab keyboard hint (`.ops-file-tab-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Soul · Skill · Mood move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Refresh-row kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1809

- Apple Agent Ops refresh-row keyboard hint (`.ops-refresh-row-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Refresh · Refresh digest · Updated move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Filter-row kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1808

- Apple Agent Ops filter-row keyboard hint (`.ops-filter-row-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the search · match · Clear move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Overview kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1807

- Apple Agent Ops overview keyboard hint (`.ops-overview-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Agents · Schedules · Live · Knowledge · Recent chats · Runs · Digest move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Health-strip kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1806

- Apple Agent Ops health-strip keyboard hint (`.ops-health-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Version · Discord · Redmine · Next schedule · Last delivery · Digest move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Tab-bar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1805

- Apple Agent Ops tab-bar keyboard hint (`.ops-tab-bar-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Overview · tabs move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Changelog body toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1804

- Apple Changelog body toolbar keyboard hint (`.changelog-body-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the version-heading move hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Changelog header kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1803

- Data Poster / Dark: AI Chat exec · answer cards (`.chat-exec-card` · `.chat-exec-code` · `.chat-answer-part` · `.chat-answer-final`) remap Apple `#ffffff` washes onto `#0e0e14`. Layout daily review (Data Poster). No white slabs inside assistant turns.

## Overnight merge — v0.1.1802

- Apple Changelog header keyboard hint (`.changelog-header-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Changelog title · Close hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Force Quit toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1801

- Apple Force Quit toolbar keyboard hint (`.force-quit-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Force Quit hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Process Details hero toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1800

- Apple Process Details hero toolbar keyboard hint (`.process-detail-hero-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the name · PID hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Process Details header kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1799

- Apple Process Details header keyboard hint (`.process-details-header-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the title · Close hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Header toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1798

- Apple header toolbar keyboard hint (`.header-toolbar-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Refresh · Settings hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Footer toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1797

- Data Poster / Dark: AI Chat · Monitors filter-miss shells (`.chat-filter-miss` · `.monitors-filter-miss`) and empty CTAs (`.chat-filter-miss-cta` · `.monitors-empty-cta`) remap Apple `#ffffff` washes onto `#0e0e14`. Layout daily review (Data Poster). No white slabs when a filter has no rows.

## Overnight merge — v0.1.1796

- Apple footer toolbar keyboard hint (`.footer-toolbar-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the version chip · GitHub link hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Icon-line kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1793

- Apple section icon-line keyboard hint (`.icon-line-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the Monitors · AI Chat · Perplexity · Debug Log · Discord · Disk Cleanup · Agent Ops hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. History sparkline kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1792

- Apple history sparkline keyboard hint (`.history-sparkline-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the CPU · GPU · Freq · Temp chart hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Ring gauge kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1791

- Apple ring gauge keyboard hint (`.ring-gauge-kb-hint`) mixes type color against an opaque shell fill (`color-mix` 63% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the CPU · GPU · Freq · Temp hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Power strip kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1790

- Apple power strip keyboard hint (`.power-strip-kb-hint`) mixes type color against an opaque strip fill (`color-mix` 63% `#0c0c10` on `#ececf1`). No glass `opacity` on the Bat · LPM · Power hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Details kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1789

- Apple Details keyboard hint (`.details-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Details value hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Top Processes list kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1788

- Apple Top Processes list keyboard hint (`.processes-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Top Processes list hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Monitors list kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1787

- Apple Monitors list keyboard hint (`.monitors-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Monitors list hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Disk Cleanup scope kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1786

- Apple Disk Cleanup scope list keyboard hint (`.disk-cleanup-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the scope-list hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Disk Cleanup meta kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1785

- Apple Disk Cleanup meta keyboard hint (`.disk-cleanup-meta-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Reclaimable now · Next automatic run · Runs when · Enabled scopes hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Disk Cleanup category list kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1784

- Apple Disk Cleanup category list keyboard hint (`.disk-cleanup-list-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the category-list hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Disk Cleanup add-scope toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1783

- Apple Disk Cleanup add-scope toolbar keyboard hint (`.disk-cleanup-add-scope-toolbar-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the label · path · days · Recursive · Add scope hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Disk Cleanup action toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1782

- Apple Disk Cleanup action toolbar keyboard hint (`.disk-cleanup-toolbar-kb-hint`) mixes type color against an opaque panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Clean now · Refresh · Save scopes hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Monitors add-form toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1781

- Apple Monitors add-form toolbar keyboard hint (`.monitor-add-toolbar-kb-hint`) mixes type color against an opaque add-form panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Monitors add-form toolbar hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Ollama settings toolbar kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1780

- Apple Ollama settings toolbar keyboard hint (`.ollama-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Ollama settings toolbar hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Ollama settings header kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1779

- Apple Ollama settings header keyboard hint (`.ollama-settings-header-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Ollama settings header hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Settings header kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1778

- Apple Settings header keyboard hint (`.settings-header-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Settings header hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Credentials section kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1777

- Apple Credentials section keyboard hint (`.credentials-section-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Credentials section hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Product setting kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1776

- Apple Product setting keyboard hint (`.product-setting-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Product section hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Appearance setting kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1775

- Apple Appearance setting keyboard hint (`.appearance-setting-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Appearance section hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Theme-list kb-hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1773

- Apple theme-list keyboard hint (`.theme-list-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Appearance theme-list hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Slack settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1771

- Apple Slack settings toolbar keyboard hint (`.slack-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the webhook · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Telegram settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1768

- Apple Telegram settings toolbar keyboard hint (`.telegram-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the token · chat id · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Cursor agent settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1767

- Apple Cursor agent settings toolbar keyboard hint (`.cursor-agent-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the workspace · executable · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Browser settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1766

- Apple Browser settings toolbar keyboard hint (`.browser-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the path · port · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. MCP settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1765

- Apple MCP settings toolbar keyboard hint (`.mcp-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the URL · stdio · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Mastodon settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1764

- Apple Mastodon settings toolbar keyboard hint (`.mastodon-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the URL · token · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Redmine settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1763

- Apple power strip captions (`.power-label` · `.lpm-label`) mix type color against the opaque strip fill (`color-mix` 45% `#0c0c10` on `#ececf1`). No shell `--muted` on Power · LPM labels. Bat status / time-left already opaque. Design-review CPU metrics polish; screenshot deferred (Mac unreachable). P2 reliability / GitHub #14.

## Overnight merge — v0.1.1762

- Restore live CPU-window metrics after #14 idle ratchet broke gauges at "None yet" (GitHub #15). Focused polls ~2s; park only on blur / Focused(false) / hidden; drop hasFocus occlusion; process-cache TTL 30s; Data Poster dark washes for monitors summary + AI Chat glance.

## Overnight merge — v0.1.1761

- Apple Redmine settings toolbar keyboard hint (`.redmine-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the URL · key · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Brave settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1760

- Apple Brave settings toolbar keyboard hint (`.brave-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the key · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Perplexity settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1759

- Apple Perplexity settings toolbar keyboard hint (`.perplexity-settings-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the key · Save · Clear hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Discord settings toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1758

- Apple Discord settings toolbar keyboard hint (`.discord-toolbar-kb-hint`) mixes type color against an opaque settings card (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the token · Save · Clear · View logs hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Monitor detail toolbar hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1757

- Apple monitor detail toolbar keyboard hint (`.monitor-detail-toolbar-kb-hint`) mixes type color against an opaque detail panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the Check now · Remove hint. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Monitor detail notes already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1756

- Apple monitor detail notes (`.monitor-detail-note`) mix type color against an opaque detail panel (`color-mix` 63% `#0c0c10` on `#ffffff`). No glass `opacity` on the “no local check history” line. `.apple-shell` beats the later shared `opacity: 0.72` on inherited `--text`. Monitor detail labels already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1755

- Apple monitor detail labels (`.monitor-detail-k`) mix type color against an opaque detail panel (`color-mix` 39% `#0c0c10` on `#ffffff`). No glass `opacity` on those URL · status captions. `.apple-shell` beats the later shared `opacity: 0.78`. Monitor last-check age already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1754

- Apple monitor last-check age (`.monitor-checked-ago`) mixes type color against an opaque row fill (`color-mix` 36% `#0c0c10` on `#ffffff`; Down rows 43%). No glass `opacity` on that age line. `.apple-shell` beats the later shared `opacity: 0.72`. The Down selector beats shared `opacity: 0.85`. Monitor latency already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1753

- Apple monitor latency (`.monitor-latency`) mixes type color against an opaque row fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on that timing line. `.apple-shell` beats the later shared `opacity: 0.9`. Slow and Down rows keep their status color. Skipped `.rings-filter-miss-hint`: `removeRingsFilterChips()` drops that node, so the hint never paints. Disk Cleanup empty hints already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1752

- Apple Disk Cleanup empty and filter-miss hints (`.disk-cleanup-empty-hint` · `.disk-cleanup-filter-miss-hint`) mix type color against an opaque empty-shell fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on those hint lines. `.apple-shell` beats the later shared `opacity: 0.9`. Monitors empty hints already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1751

- Apple Monitors empty and filter-miss hints (`.monitors-empty-hint` · `.monitors-filter-miss-hint`) mix type color against an opaque empty-shell fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on those hint lines. `.apple-shell` beats the later shared `opacity: 0.9`. Debug Log filter-miss hints already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1750

- Apple Debug Log filter-miss hints (`.logs-filter-miss-hint`) mix type color against an opaque filter-miss fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on that hint line. `.apple-shell` beats the later shared `opacity: 0.9`. Top Processes filter-miss hints already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1749

- Apple Top Processes filter-miss hints (`.processes-filter-miss-hint`) mix type color against an opaque filter-miss fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on that hint line. `.apple-shell` beats the later shared `opacity: 0.9`. Perplexity empty hints already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1748

- Apple Perplexity empty hints (`.perplexity-empty-hint` · `.perplexity-filter-miss-hint`) mix type color against an opaque empty-shell fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on those hint lines. `.apple-shell` beats the later shared `opacity: 0.9`. AI Chat exec labels already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1747

- Apple AI Chat exec labels (`.chat-exec-label`) mix type color against an opaque message-list fill (`color-mix` 33% `#0c0c10` on `#ffffff`). No glass `opacity` on those Code executed / Result captions. `.apple-shell` beats the later shared `opacity: 0.65`. AI Chat thinking bubble already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1746

- Apple AI Chat thinking bubble (`.chat-message.thinking`) mixes type color against an opaque message-list fill (`color-mix` 75% `#0c0c10` on `#ffffff`). No glass `opacity` on that waiting bubble. AI Chat response time already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1745

- Apple AI Chat response time (`.response-time`) mixes type color against an opaque message-list fill (`color-mix` 36% `#0c0c10` on `#ffffff`). No glass `opacity` on that latency line. AI Chat status already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1744

- Apple AI Chat status (`.chat-status`) mixes type color against an opaque message-list fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on that status line. AI Chat empty shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1743

- Apple AI Chat empty shell (`.chat-empty`) mixes type color against an opaque message-list fill (`color-mix` 45% `#0c0c10` on `#ffffff`). No glass `opacity` on that empty state. Hover drops the `opacity: 1` restore. Perplexity result meta already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1742

- Apple Perplexity result meta (`.perplexity-result-meta`) mixes type color against an opaque row fill (`color-mix` 38% `#0c0c10` on `#ffffff`). No glass `opacity` on that source line. Debug Log path hint already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1741

- Apple Debug Log path hint (`.logs-path-hint`) mixes type color against an opaque toolbar fill (`color-mix` 43% `#0c0c10` on `#ffffff`). No glass `opacity` on that path line. Footer type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1740

- Apple footer (`.apple-footer`) mixes type color against an opaque shell fill (`color-mix` 28% `#0c0c10` on `#f7f7fa`; hover 44%). No glass `opacity` on the always-visible version · GitHub line. Time-left caption already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1739

- Apple time-left caption (`.time-remaining`) mixes type color against an opaque strip fill (`color-mix` 45% `#0c0c10` on `#ececf1`). No glass `opacity` on that always-visible line. Battery status already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1738

- Apple battery status (`.battery-status`) mixes type color against an opaque strip fill (`color-mix` 45% `#0c0c10` on `#ececf1`). No glass `opacity` on the always-visible Bat charging / AC caption. History range label already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1737

- Apple History range label (`.history-controls label`) mixes type color against an opaque fill (`color-mix` 46% `#0c0c10` on `#ffffff`). No glass `opacity` on the always-visible range caption. Section titles already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1736

- Apple section titles (`.section-title`) mix type color against an opaque panel fill (`color-mix` 75% `#010101` on `#ffffff`). No glass `rgb(1,1,1,0.75)` on the always-visible Details · Top Processes headings. Details label type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1735

- Apple Details labels (`.detail-label`) mix type color against an opaque panel fill (`color-mix` 63% `#3c3c43` on `#ffffff`). No glass `opacity` on the always-visible Load · RAM · Up captions. Chip glyph type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1734

- Apple chip glyph (`#chip-info::before`) mixes type color against an opaque shell fill (`color-mix` 32% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the always-visible  mark beside the chip line. Chip subtitle type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1733

- Apple chip subtitle (`.apple-subtitle` / `#chip-info`) mixes type color against an opaque shell fill (`color-mix` 46% `#0c0c10` on `#f7f7fa`). No glass `opacity` on the always-visible chip line under the title. History sparkline captions already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1732

- Apple History sparkline captions (`.history-chart-caption`) mix type color against an opaque chart fill (`color-mix` 36% `#0c0c10` on `#ffffff`). No glass `opacity` on the always-visible CPU · GPU · Freq · Temp chart labels. Metric-subtext type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1731

- Apple ring metric subtext (`.metric-subtext`) mixes type color against an opaque card fill (`color-mix` 40% `#0c0c10` on `#ffffff`). No glass `opacity` on the always-visible CPU · GPU · Freq · Temp secondary lines. Metric-label type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1730

- Apple ring metric labels (`.metric-label`) mix type color against an opaque card fill (`color-mix` 62% `#0c0c10` on `#ffffff`). No glass `opacity` on the always-visible CPU · GPU · Freq · Temp captions. Theme-item type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1729

- Apple Settings theme-list type (`.theme-item`) mixes against an opaque panel fill. No glass alpha on the theme button label fallback (`var(--text, #0c0c10)`). Markdown link type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1728

- Apple AI Chat markdown link type (`.chat-message .markdown a`) mixes against an opaque panel fill. No glass alpha on markdown link color. Force Quit type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1727

- Apple Force Quit control type (`.force-quit-btn` resting) mixes against an opaque panel fill. No glass alpha on Force Quit label type. Changelog body already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1726

- Apple Changelog body type (`.changelog-error` · `.changelog-h2` · `.changelog-version` · `.changelog-h3` · `.changelog-paragraph` · `.changelog-item` · bullet · `.changelog-code` · `strong`) mixes against an opaque panel fill. No glass alpha on Changelog copy or the error accent. Icon-line status already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1725

- Apple icon-line status washes (`.icon-line-item.status-good` · `.status-warning` · `.status-bad` resting · hover) mix type color against an opaque chip fill. No glass alpha on Ready / Slow / Down strip status type. Details / Top Processes body already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1724

- Apple Details / Top Processes body type (`.details-grid` · `.process-table`) mixes against an opaque panel fill. No glass alpha on Detail labels or process rows. Battery glyph already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1723

- Apple battery strip glyph (`.battery-icon` resting · charging) mixes type color against an opaque strip fill. No glass alpha on the always-visible Bat icon. Window title already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1722

- Apple window title (`.apple-title h1`) mixes type color against an opaque shell fill. No glass alpha on the always-visible product title. Icon-btn type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1721

- Apple section icon strip glyphs (`.icon-btn` resting · hover) mix type color against an opaque chip fill. No glass alpha on the always-visible Monitors · AI Chat · … strip icons. Icon-line type already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1720

- Apple icon-line strip glyphs (`.icon-line-item` resting · hover) mix type color against an opaque chip fill. No glass alpha on the always-visible Monitors · AI Chat · … strip icons. Primary / muted type tokens already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1719

- Apple primary / muted type tokens (`--text` · `--muted`) and leftover panel tokens (`--hairline` · `--panel` · `--panel-border` · `--panel-shadow`) mix against an opaque shell fill. No glass alpha on always-on labels. Modal dimmers already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1718

- Apple Settings / Monitors / AI Chat modal dimmers (`--modal-backdrop`) mix against an opaque fill. No glass alpha on the full-screen backdrop while those sheets are open. Ring tracks already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1717

- Apple ring gauge tracks (`.ring-track` / `--ring-track`) mix against an opaque fill. No glass alpha on the always-visible CPU · GPU · Freq · Temp ring tracks. Markdown table/hr hairlines already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1716

- Apple AI Chat markdown table cells and horizontal rules (`.chat-message .markdown table th` · `td` · `hr`) mix hairline borders against an opaque fill. No glass alpha on markdown table/hr dividers. Add form / row history already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1715

- Apple Monitors settings Add form and row history (`.add-monitor-form` · `.monitor-history`) mix hairline borders against an opaque fill. No glass alpha on the Add form divider or per-row history tick divider. Popover headers already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1714

- Apple Monitors / AI Chat settings popover headers (`.monitors-settings-popover .popover-header` · `.ollama-settings-popover .popover-header`) mix the hairline border against an opaque fill. No glass alpha on the popover title-row divider. Message-list border already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1713

- Apple AI Chat message list (`.chat-messages`) mixes the hairline border against an opaque fill. No glass alpha on the message-list border (panel fill already opaque). Icon-strip / section dividers already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1712

- Apple icon-strip dividers (`.icon-btn:not(:last-child)::after`) and section hairlines (`.apple-divider`) mix against an opaque fill. No glass alpha on the always-visible Monitors · AI Chat · … strip separators or section dividers. Outer shell border already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1711

- Apple outer window shell (`.apple-shell`) mixes the hairline border against an opaque fill. No glass alpha on the always-visible window chrome border. History chart shells already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1710

- Apple History sparkline shells (`.history-chart-container`) mix hairline borders against an opaque fill. No glass alpha on the always-visible CPU · GPU · Freq · Temp history chart borders. Metric cards already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1709

- Apple ring metric cards (`.metric-card`) mix hairline borders against an opaque fill. No glass alpha on the always-visible CPU · GPU · Freq · Temp card borders. Details / Top Processes shells already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1708

- Apple Details / Top Processes shells (`.apple-details` · `.apple-processes` resting · hover) mix hairline borders against an opaque fill. No glass alpha on the section panel borders. Changelog scrollbar already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1707

- Apple Changelog scrollbar (`.changelog-body` `::-webkit-scrollbar-track` · thumb resting · hover) mixes against an opaque fill. No glass alpha on the Changelog scroll chrome. Details / Top Processes thumbs already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1706

- Apple Details / Top Processes scrollbar thumbs (`.apple-details` · `.apple-processes` `::-webkit-scrollbar-thumb` resting · hover) mix against an opaque fill. No glass alpha on the thin scroll thumbs. Battery strip hairline already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1705

- Apple battery / power strip (`.battery-power-strip` resting · hover) mixes hairline borders against an opaque fill. No glass alpha on the strip border. Focus-within already opaque. History controls already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1704

- Apple History controls (`.history-controls` · `.time-range-dropdown` resting · hover) mix hairline borders against an opaque fill. No glass alpha on the History chrome or time-range border. Focus ring already opaque. Model-text hover already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1703

- Apple AI Chat model label (`.model-text` hover) and connection indicator (`.connection-indicator:focus-visible`) mix fills / focus ring against an opaque fill. No glass alpha on the model-name hover wash or connection focus ring. Model-text focus-visible already opaque. Popover Close hover already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1702

- Apple Monitors / AI Chat settings popover Close controls (`.monitors-settings-popover .popover-close` · `.ollama-settings-popover .popover-close` hover) mix fills against an opaque fill. No glass alpha on the Close hover wash. Overflow menu triggers already opaque. Popover shells already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1701

- Apple Monitors / AI Chat overflow menu triggers (`.monitors-menu-btn` · `.ollama-menu-btn` hover) mix fills against an opaque fill. No glass alpha on the ⋯ hover wash. Menu shells already opaque. Collapse control already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1700

- Apple section collapse control (`.collapse-btn` hover · focus-visible) mixes fills against an opaque fill. No glass alpha on the chevron hover wash or focus ring. Section headers already opaque. History tooltip already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1699

- Apple History sparkline tooltip (`.history-tooltip`) uses an opaque fill. No glass alpha on the tip panel, border, or soft drop shadow. Match monitor tick tips opaque in v0.1.1594. Markdown shells already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1683

- Apple Monitors settings popover shell (`.monitors-settings-popover .popover-content`) uses an opaque fill. No glass alpha on the popover panel or soft drop shadow. Close focus ring mixes against opaque. Perplexity search box already opaque. Model select already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1682

- Apple Perplexity search box (`.perplexity-search-box input` · `button` resting · hover · focus-visible) mixes the wash against an opaque fill. No glass alpha on the query field, Search control, or focus rings. Model select already opaque. Send control already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1681

- Apple AI Chat model select (`.model-select` resting · hover · focus · focus-visible) mixes the wash against an opaque fill. No glass alpha on the field, hover border, or focus ring. Send control already opaque. Composer field already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1680

- Apple AI Chat Send control (`#chat-send-btn` resting · hover · focus-visible · active) mixes the wash against an opaque fill. No glass alpha on the Send fill, soft hover shadow, or focus ring. Composer field already opaque. Composer shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1679

- Apple AI Chat composer field (`#chat-input` resting · hover · focus) mixes the wash against an opaque fill. No glass alpha on the field, hover border, or focus ring. Composer shell already opaque. Message bubbles already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1678

- Apple AI Chat composer shell (`.chat-input-container` resting · `:focus-within`) mixes the wash against an opaque fill. No glass alpha on the composer shell or focus ring. Message bubbles already opaque. Message list shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1677

- Apple AI Chat message bubbles (`.chat-message.user` · `.chat-message.assistant`) mix the wash against an opaque fill. No glass alpha on the user/assistant bubbles or accent borders. Message list shell already opaque. Empty shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1676

- Apple AI Chat message list (`.chat-messages` resting · `:focus-within`) mixes the wash against an opaque fill. No glass alpha on the list shell or focus ring. Empty shell already opaque. Top Processes bar fills already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1675

- Top Processes usage bar fills (`.process-bar-fill`) mix against an opaque fill. No glass alpha on the bar fill. Shared sheet already opaque; Apple had put glass back. Bar tracks already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1674

- Top Processes usage bar tracks (`.process-bar` / `#process-list .process-bar`) mix against an opaque fill. No glass alpha on the bar track. Process Details row hairlines already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1673

- Process Details metric row hairlines (`.process-detail-row` border-bottom) mix against an opaque fill. No glass alpha on the row dividers. Force Quit control already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1672

- Force Quit control (`.force-quit-btn` resting · hover · focus · active · `.is-confirming` + section hairline) mix the wash against an opaque fill. No glass alpha on the Force Quit control. Process Details panel already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1671

- Process Details panel (`.process-detail-hero` · `.process-detail-section`) mix the wash against an opaque fill. No glass alpha on the hero or metric sections. Shared sheet and Apple theme both updated. Monitors detail already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1670

- Monitors detail panel (`.monitor-detail` · `.monitor-detail-log`) mix the wash against an opaque fill. No glass alpha on the expanded detail shell or log pad. Selected · focus washes already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1669

- Monitors row selected · focus (`.monitor-item.is-selected` · `:focus-visible`) mix the wash against an opaque fill. No glass alpha on the selection wash or focus ring. Down · Slow status washes already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1668

- Monitors Down · Slow rows (`.monitor-item.is-down` · `.is-slow` resting · hover) mix the wash against an opaque fill. No glass alpha on the status row. Apple Monitors base row already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1667

- Apple Monitors rows (`.monitor-item` resting · hover) mix the wash against an opaque fill. No glass alpha on the row. Hover drops the soft glass shadow. AI Chat empty shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1666

- Apple AI Chat empty shell (`.chat-empty` resting · hover) mixes the wash against an opaque fill. No glass alpha on the empty list. The shared sheet was already opaque; the Apple theme had put glass back. Monitors empty shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1665

- Apple Monitors empty shell (`.monitors-empty` resting · hover · error) mixes the wash against an opaque fill. No glass alpha on the empty list. Filter-miss / empty CTA already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1664

- Monitors filter-miss (`.monitors-filter-miss` · Down · Slow · Up empty) and empty CTA (`.monitors-empty-cta` resting · hover · focus) mix the wash against an opaque fill. No glass alpha on the filter-miss shell or Add Monitor CTA. Visible when Monitors is expanded with a filter miss or empty list. Disk Cleanup soft-delete already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1663

- Disk Cleanup soft-delete (`.disk-cleanup-soft-delete` resting · hover · focus-within) mixes the wash against an opaque fill. No glass alpha on the Move to Trash row. Visible when Disk Cleanup is expanded. Top Processes empty shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1662

- Apple Top Processes empty shell (`.process-empty` resting · hover) mixes the wash against an opaque fill. No glass alpha on the empty list. The shared sheet was already opaque; the Apple theme had put glass back. Settings card already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1661

- Apple Settings card (`.settings-card` + `.settings-header` hairline) uses an opaque fill. No glass `--panel` alpha on the modal shell. Soft panel drop shadow dropped. Visible when Settings is open. Settings toggles already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1660

- Apple Settings toggles (`.setting-toggle input[type="checkbox"]` resting · checked · knob) mix the track against an opaque fill. No glass alpha on the switch. Soft knob drop shadow dropped. Visible when Settings is open. Help sheet already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1659

- Apple Settings help sheet (`.settings-help-sheet` resting · focus · Copied) mixes the wash against an opaque fill. No glass alpha on the cheat-sheet panel, focus ring, or Copied flash. Visible when Settings → Help is open. Theme list already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1658

- Apple theme list (`.theme-item` resting · hover · focus · current) mixes the wash against an opaque fill. No glass alpha on the theme button. Hover drops the soft glass shadow. Visible in Settings → Appearance. Disk Cleanup primary toolbar already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1657

- Disk Cleanup primary toolbar (`.disk-cleanup-toolbar .disk-cleanup-primary` resting · hover) mixes the fill against an opaque fill. No glass alpha on the Clean now / primary wash. Settings buttons already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1656

- Apple Settings buttons (`.settings-btn` · `.settings-btn-primary` resting · hover) mix the fill against an opaque fill. No glass alpha on the button wash. Disk Cleanup scope filter-miss already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1655

- Disk Cleanup scope filter-miss shell (`.disk-cleanup-scope-filter-miss` · On · Off empty) mixes the wash against an opaque fill. No glass alpha on the scope filter-miss shell. Settings input rest/hover already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1654

- Apple Settings inputs (`.settings-input` · `.discord-token-input` resting · hover) mix the fill against an opaque fill. No glass alpha on the field or the hover border. Visible when Settings is open. Focus ring and Disk Cleanup filter-miss already opaque. P2 reliability / GitHub #14.


## Overnight merge — v0.1.1653

- Disk Cleanup filter-miss shell and empty CTA (`.disk-cleanup-filter-miss` · Reclaim · Big · Clean · `.disk-cleanup-empty-cta` resting · hover · focus) mix the wash against an opaque fill. No glass alpha on the filter-miss shell or Clear/Review CTA. Empty shell already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1652

- Disk Cleanup empty shell (`.disk-cleanup-empty`) mixes the wash against an opaque fill. No glass alpha on the empty category/scope shell. Last-run panel already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1651

- Disk Cleanup last-run panel (`.disk-cleanup-last` resting · hover · focus · has-skip · is-ok) mixes the wash against an opaque fill. No glass alpha on the last-run shell. Meta cards already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1650

- Apple Settings inputs (`.settings-input:focus` · `.discord-token-input:focus`) mix the focus ring against an opaque fill. No glass alpha on the focus ring or the focused field. Visible when Settings is open. Disk Cleanup meta cards already opaque. P2 reliability / GitHub #14.


## Overnight merge — v0.1.1649

- Disk Cleanup meta cards (`.disk-cleanup-meta-card` resting · hover · focus · Reclaim · Clean · scopes · due · periodic) mix the wash against an opaque fill. No glass alpha on the card. Hover drops the soft glass shadow. Category/scope rows already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1648

- Disk Cleanup category and scope rows (resting · hover · focus · selected · Reclaim · Big) mix the wash against an opaque fill. No glass alpha on the row. Hover drops the soft glass shadow. Copied badge already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1647

- Disk Cleanup row Copied badge (`.disk-cleanup-item` / `.disk-cleanup-scope-row` `is-just-copied` `::after`) mixes the green wash against an opaque fill. No glass alpha on the badge. The row wash itself was already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1646

- Apple history time-range (`.time-range-dropdown:focus`) mixes the focus wash against an opaque fill. No glass alpha on the History dropdown focus ring or border. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1645

- Monitors row Copied badge (`.monitor-item.is-just-copied` `::after`) mixes the green wash against an opaque fill. No glass alpha on the badge. The row wash itself was already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1644

- Apple power strip (`.battery-power-strip:focus-within`) and Bat/LPM attention flash rings mix washes against an opaque strip fill (`#ececf1`). No glass alpha on the strip focus ring or Hot attention flash. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1643

- Footer GitHub (`#github-link` hover · focus-visible) and Apple version chip (`.app-version:focus-visible` · `.apple-github-link:focus-visible`) mix washes against an opaque fill. No glass alpha on the footer focus rings or GitHub hover wash. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1642

- Agent Ops row Copied badge (`.ops-row.is-copied` `::after`) mixes the green wash against an opaque fill. No glass alpha on the badge. The row wash itself was already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1641

- Apple section strip (`.icon-line-item:focus-visible`) mixes the focus outline against an opaque fill. No glass alpha on the Monitors · AI Chat · Perplexity · Debug Log · Discord · Disk Cleanup · Agent Ops focus ring. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1640

- Apple icon strip (`.icon-btn` hover · focus-visible · active) mixes washes against an opaque fill. No glass alpha on the section icon hover wash, focus ring, or active press. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1639

- Details value Copied badge (`.details-grid > .detail-value[role='option'].is-just-copied` `::after`) mixes the green wash against an opaque fill. No glass alpha on the badge. The value wash itself was already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1638

- Ring focus (`#cpu-usage-card:focus-visible` · `.metric-card:focus-within`) mixes the focus wash against an opaque fill and drops soft glass focus shadows. No glass alpha on the CPU ring focus ring or the GPU · Freq · Temp card focus outlines. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1637

- Details / Top Processes headers (`.collapsible-header` hover · focus-visible) mix washes against an opaque fill in the Apple theme. No glass alpha on the hover wash or focus ring. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1636

- Top Processes row Copied badge (`.process-row.is-just-copied` `::after`) mixes the green wash against an opaque fill. No glass alpha on the badge. The row wash itself was already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1635

- Collapsible section headers (`.section-header-collapsible` hover · focus-visible) mix washes against an opaque fill. No glass alpha on the hover wash, focus ring, or Apple theme border. Always-visible on the default collapsed layout. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1634

- Debug Log lines (`.logs-line` hover · selected) mix washes against an opaque fill. No glass alpha on the hover wash or selected inset ring. Copied flash already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1633

- Perplexity result Copied badge (`.perplexity-result-item` `::after`) mixes the green wash against an opaque fill. No glass alpha on the badge. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1632

- Debug Log collapsed Error/Warn glance (`.logs-error-glance` · Quiet · hover) mix washes against an opaque fill. Soft glass hover shadow dropped. Attention glance already opaque. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1631

- Debug Log filter-miss (`.logs-viewer-empty.logs-filter-miss` · Error · Warn · Clear filter) mix washes against an opaque fill. No glass alpha on the empty-filter shell or Clear filter control. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1630

- Debug Log chrome (`.logs-toolbar` · buttons · `.logs-viewer` · path focus) mix washes against an opaque fill. No glass alpha on the toolbar shell, Refresh / Open controls, viewer panel, or path focus ring. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1629

- Perplexity weather card (`.perplexity-weather-card`) mixes the blue wash against an opaque fill. No glass alpha on the card background, border, or accent edge. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1628

- Perplexity empty / filter-miss (`.perplexity-empty` · error · Top · Snippet · Clear filter) mix the wash against an opaque fill. No glass alpha on the empty panel, error shell, or Clear filter button. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1627

- Perplexity result rows (`.perplexity-result-item` resting · hover · focus · selected · Top) mix the wash against an opaque fill. No glass alpha on the row background, border, or focus ring. Hover drops the soft glass shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1626

- Top Processes empty shell (`.process-empty`) mixes its wash against an opaque fill. No glass alpha on the dashed empty-list panel. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1625

- Top Processes filter-miss (`.processes-filter-miss` · Hot · Pinned · Clear filter) mixes the wash against an opaque fill. No glass alpha on the empty panel or the Clear filter button. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1624

- Ring filter-miss (`.rings-filter-miss` · CTA) mix washes against an opaque fill. No glass alpha on the empty-filter shell or Clear filter control. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1623

- AI Chat filter-miss / Clear (`.chat-filter-miss` · CTA · `#chat-clear-btn`) mix washes against an opaque fill. No glass alpha on the empty-filter shell or Clear control. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1622

- AI Chat error bubbles (`.chat-message.assistant.is-error`) mix their wash against an opaque fill. No glass alpha on the error row background or border. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1621

- AI Chat exec / answer cards (`.chat-exec-card` · `.chat-exec-code` · `.chat-answer-part` · final) mix washes against an opaque fill. No glass alpha on the code-exec shell, code block, or answer-part panels. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1620

- AI Chat composer (`#chat-input:focus` · `#chat-send-btn` resting · hover) mixes the focus wash against an opaque fill, uses an opaque focus background, and drops soft glass Send shadows. No glass alpha on the composer focus ring or Send blur. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1619

- AI Chat message rows (hover · focus · selected · Copied badge) mix their wash against an opaque fill. No glass alpha on the row background, focus ring, or Copied badge. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1618

- AI Chat empty starter chips (`.chat-empty-chip` resting · hover · focus-visible) mix washes against an opaque fill. No glass alpha on the empty-shell suggestion chips. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1617

- Agent Ops refresh row (Refresh · Refresh digest · Updated stamp · top hairline) mixes washes against an opaque fill. No glass alpha on secondary refresh buttons, the Updated control, or the refresh-row divider. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1616

- Agent Ops agent editor (`textarea.ops-agent-editor` focus · dirty) mixes washes against an opaque fill. No glass alpha on the Agents edit textarea focus ring or dirty border. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1615

- AI Chat empty shell (default · Ready · no model · offline · not set) mixes its wash against an opaque fill. No glass alpha on the shell background or border. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1614

- Agent Ops detail preview (`.ops-preview`) mixes its wash against an opaque fill. No glass alpha on the preview panel background or border. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1613

- Agent Ops close button (`.ops-close-btn` resting · hover) mix washes against an opaque fill. No glass alpha on the section close control. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1612

- Agent Ops loading shell (`.ops-loading`) mixes its wash against an opaque fill. No glass alpha on the dashed border or the shell background. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1611

- Agent Ops Overview Open links (`.ops-overview-link` resting · hover · focus-visible) mix washes against an opaque fill. No glass alpha on card Open controls. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1610

- Agent Ops copy chips (`.ops-session-copy-chip` resting · hover · focus-visible) mix washes against an opaque fill. No glass alpha on id / slug / path / request-id chips before the Copied flash. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1609

- Agent Ops empty panels and the Clear filter button mix washes against an opaque fill. No glass alpha on the empty shell, filter-miss calm, fail-empty, true-empty tab, overview empty CTA, or Clear hover and focus. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1608

- Agent Ops On · Off badges (`.ops-badge` resting · hover · focus-visible · off) mix washes against an opaque fill. No glass alpha on list-row status badges. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1607

- Agent Ops list rows (`.ops-row` resting · hover · focus-visible · selected) mix washes against an opaque fill. No glass alpha on shared list-row states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1606

- Agent Ops Runs list rows (Lite · Slow · Fail) mix washes against an opaque fill. No glass alpha on resting or hover row states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1605

- Agent Ops Runs lane filter chips (All · Instant · Lite · Direct · Slow · Fail) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1604

- Agent Ops Knowledge filter chips (All · Discord · Core) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1603

- Agent Ops Schedules filter chips (All · Jobs · Deliveries) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1602

- Agent Ops Agents filter chips (All · On · Off) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1601

- Agent Ops Sessions filter chips (All · Live · Files) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1600

- Agent Ops tab strip (tabs, file tabs, count pills) mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, or active states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1599

- Ring, battery, and power Copied flashes sit above the value with left and right. No translate. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1598

- Agent Ops filter input, match chip (all · partial · zero), Clear, and just-cleared flash mix washes against an opaque fill. No glass alpha on resting, hover, focus, or match states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1597

- Disk Cleanup scope filter chips (All · On · Off) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1596

- Disk Cleanup category filter chips (All · Reclaim · Big · Clean) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1594

- Monitor history tick tips sit above the bar with left and top. Opaque fill. No translate or shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1592

- AI Chat filter chips (All · You · Assistant · Errors) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1591

- Agent Ops overview cards (Agents · Schedules · Sessions · Memory) mix washes against an opaque fill. No glass alpha on resting, hover, focus-within, focus-visible, ok/warn/bad, or active states. Soft hover drop shadows removed. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1590

- Rings filter chips (All · Hot) mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1589

- External / Monitors filter chips (All · Up · Down · Slow) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits chip states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1588

- Agent Ops health cards (Version · Discord · Redmine · Next schedule · Last delivery · Digest) mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, ok/warn/bad, or active states. Active hover drops the soft shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1587

- Top Processes filter chips (All · Pinned · Hot) and Clear mix washes against an opaque fill. No glass alpha on resting, hover, focus-visible, active, or has-hits states. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1586

- Top Processes pin hover and focus-visible mixes the accent wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1585

- Top Processes row pinned, hover, focus-visible, active, and selected mixes the accent wash against an opaque fill. No glass alpha or hover drop shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1584

- Details value hover, focus-visible, and selected mixes the accent wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1583

- Ring and power-strip copy hover and focus-visible mixes the accent wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1582

- Ring and power-strip Copied flash mixes the accent wash against an opaque fill. No glass alpha on the value or the Copied badge. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1581

- Agent Ops row Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1580

- AI Chat message Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1579

- Debug Log line Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1578

- Perplexity result row Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1577

- Disk Cleanup row Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1576

- Monitors row Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1575

- Top Processes row Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1574

- Details value Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1573

- Settings product-toggle Saved flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1572

- Shared Save / secondary-button Saved flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1571

- Disk Cleanup scope path Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1570

- Disk Cleanup category path Copied flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1529

- Settings Help Saved flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1528

- Footer GitHub Saved flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1527

- Header Refresh Saved flash mixes the green wash against an opaque fill. No glass alpha. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1526

- Header Refresh button stays dim while metrics load. No rotate animation. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1525

- Settings Having fun Off glance mixes the off wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1524

- Settings Ori Mnemos Off glance mixes the off wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1523

- Settings Downloads organizer Off glance mixes the off wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1522

- Settings Judge Off glance mixes the off wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1521

- Settings Voice STT glance mixes the off wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1520

- Settings Compact On glance mixes the on wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1519

- Settings AI Off glance mixes the off wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1518

- Settings Signal glance mixes the not-wired wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1517

- Settings Help glance mixes closed and open washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1516

- Settings Slack not-set/partial glance mixes the not-set and partial washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1515

- Settings Telegram not-set/partial glance mixes the not-set and partial washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1514

- Settings Cursor agent not-set glance mixes the not-set wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1513

- Settings Browser / CDP not-set glance mixes the not-set wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1512

- Settings Discord token glance mixes the not-set wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1511

- Settings MCP not-set attention glance mixes the not-set wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1510

- Settings Mastodon not-set/partial attention glance mixes the not-set and partial washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1509

- Settings Redmine not-set/partial attention glance mixes the not-set and partial washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1508

- Settings Brave Key-not-set attention glance mixes the not-set wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1507

- Settings Perplexity key glance mixes the not-set wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1506

- Perplexity last-search glance mixes results, searching, error, key-needed, and ready washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1505

- Perplexity Top/error/filter attention glance mixes error, top, and filter washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1504

- Perplexity Key-not-set attention glance mixes the not-set wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1503

- External / Monitors summary mixes down, all-up, and slow washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1502

- Agent Ops Signal Not wired/Partial attention glance mixes not-wired, not-set, partial, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1501

- Agent Ops Slack Not set/Partial attention glance mixes not-set, partial, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1500

- Agent Ops Telegram Not set/Partial attention glance mixes not-set, partial, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1499

- Agent Ops Mastodon Not set/Partial attention glance mixes not-set, partial, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1498

- Agent Ops Perplexity Search Not set/Unavailable/Degraded attention glance mixes not-set, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1497

- Agent Ops Cursor Not set/Unavailable/Degraded attention glance mixes not-set, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1496

- Agent Ops MCP Not set/Unavailable/Degraded attention glance mixes not-set, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1495

- Agent Ops Browser (CDP) Not set/Unavailable/Degraded attention glance mixes not-set, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1494

- Agent Ops Brave Search Not set/Unavailable/Degraded attention glance mixes not-set, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1493

- Agent Ops Ollama Not set/Offline/Degraded attention glance mixes not-set, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1492

- Agent Ops Redmine Not set/Degraded/Unavailable attention glance mixes not-set, warn, and bad washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1491

- Agent Ops Discord Offline/Reconnect attention glance mixes offline and reconnect washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1490

- Agent Ops Digest glance mixes the open-candidate wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1489

- Agent Ops Filter attention glance mixes All, On/Live/Jobs/Core/Instant/Lite/Direct, Off/Files/Deliveries/Discord, Slow, and Fail washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1488

- Agent Ops Runs Fail/Slow attention glance mixes fail and slow washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1487

- Debug Log Error/Warn attention glance mixes error and warn-only washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1486

- Disk Cleanup Reclaim/Due attention glance mixes Big, Reclaim, and Due washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1485

- Disk Cleanup filter glance mixes All, Reclaim, Big, and Clean washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14. Rebased past origin v0.1.1482–1484 (Monitors filter, Top Processes filter, Top Processes Hot).

## Overnight merge — v0.1.1484

- Top Processes Hot attention glance mixes the hot-count wash against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14. Ollama base-URL unit tests no longer mutate process-wide `OLLAMA_HOST`.

## Overnight merge — v0.1.1483

- Top Processes Filter attention glance mixes All, Pinned, and Hot washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1482

- External / Monitors Filter attention glance mixes All, Up, Down, and Slow washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1481

- External / Monitors Down/Slow attention glance mixes down and slow washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14. Rebased past origin v0.1.1477–1480.

## Overnight merge — v0.1.1476

- Agent Ops, process rows, logs, and Disk Cleanup drop hover lift and press scale. Copied badges center with margin, not `translateY(-50%)`. P2 reliability / GitHub #14. Rebased past origin v0.1.1473–1475.

## Overnight merge — v0.1.1475

- AI Chat model / connection glance mixes online, no-model, offline, and circuit washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1474

- CPU, GPU, Freq, and Temp history charts mix hot, calm, and Fair washes against an opaque fill. No glass alpha or ring shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1473

- AI Chat collapsed glance mixes online, offline, active, and error washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1472

- Press and hover no longer scale or lift. Add button, connection dot, and thinking-dots keyframes drop `scale`. Apple theme, result, log, and Send controls drop the one-pixel lift. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1471

- Agent Ops collapsed glance mixes ready, warn, and offline washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1470

- Disk Cleanup collapsed glance mixes reclaim, due, scopes-off, and clean washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1469

- External / Monitors collapsed glance mixes up, down, and slow washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1468

- Top Processes keep-header glances (CPU · GPU · RAM) mix calm and hot washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1467

- Ring progress strokes draw their start in the path. Dark, Futuristic, Neon, Material, and Swiss no longer use CSS rotate. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1466

- Details collapsed glance (Load · RAM · Up) mixes calm and hot washes against an opaque fill. No glass alpha or hover shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1465

- CPU, GPU, Freq, and Temp ring cards mix hot, calm, and Fair washes against an opaque fill. No glass alpha or ring shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1464

- Battery, power, Low Power Mode, and time-remaining status washes mix against an opaque fill. No glass alpha or ring shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1463

- Settings toggle knobs sit with `left`, not a translate. An on switch no longer keeps a transform layer. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1462

- Low Power Mode toggle uses an opaque track. No glass alpha, inset highlight, or knob drop shadow. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1461

- Section icon chips use opaque fills. No glass alpha, inset highlight, or hover drop shadow. Status washes mix against an opaque color. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1460

- Ring numbers and the line under them center without `transform: translate`. Those labels no longer keep a Graphics and Media layer on the open CPU window. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1459

- Low Power Mode knob sits with `left`, not a translate. The battery strip no longer keeps a transform layer while LPM is on. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1458

- Section icons skip transform layers. Hover and press do not lift or scale those chips. The Monitors status dot uses offset, not translate. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1457

- Header Refresh and Settings skip transform layers. The divider uses offset, not translate. Hover and press do not lift or scale those buttons. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1456

- Ring gauges center without `transform: translate`. The SVG no longer keeps a Graphics and Media layer on the open CPU window. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1455

- Sparkline and data-poster canvases skip bind and `canvas.width` on open. History containers stay out of the compositor until hover or Refresh. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1454

- Collapsed monitors, chat, logs, and Agent Ops skip `requestIdleCallback` wiring on open. Section chrome waits for a click or Tab. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1453

- Settings chrome and the changelog modal stay unwired until opened. Collapsed AI Chat, Debug Log, and Disk Cleanup skip filter wiring until expand. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1452

- Data-poster history charts skip `getComputedStyle` on open. Colors load on the first draw or tooltip. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1449

- Data-poster metric-card bar and line charts stay parked on open. First paint does not allocate canvas buffers or draw. The same idle unpark as the history charts draws them later. Canvas markup starts at 1×1. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1448

- CPU window: collapsed Agent Ops skips filter, overview, and keyboard wiring on monitoring idle. Expand still hydrates once. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1447

- CPU window: bake capture `?open=` at create; skip `take_open_ui_section` WebView IPC. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1445

- Data-poster history charts stay parked on open. First paint does not allocate canvas buffers or fetch history. The same idle unpark as the other themes draws them later. Focus does not unpark them on the event. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1444

- CPU window: collapsed Top Processes skips `get_pinned_process_names` on warm-up and focus resume. Pins paint from localStorage. Expand hydrates from disk. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1443

- CPU window: collapsed Monitors skips `list_monitor_statuses`; last-known icon wash from localStorage. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1442

- CPU window: Discord icon skips `is_discord_gateway_ready` on open/resume; localStorage last-known paint. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1441

- CPU window: Settings credential Save/Clear wiring deferred to Settings open (not monitoring idle). P2 reliability / GitHub #14.

## Overnight merge — v0.1.1440

- CPU window: collapsed Debug Log skips `read_debug_log` glance IPC. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1439

- Idle-thought Ollama timeout warnings: one WARN per five minutes even when several channels time out together. Later lines in that window stay debug. P2 reliability / debug.log.

## Overnight merge — v0.1.1438

- CPU window: one `list_monitor_statuses` IPC for Monitors summary/list/settings; defer 24h history availability probe until sparkline unpark/seed. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1437

- CPU window: collapsed Monitors skips history + full list IPC; icon-only summary walk (no per-host details); expand hydrates once. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1436

- CPU window: AI visibility from localStorage on open; defer Ollama `configure_ollama` until AI Chat expand / AI-on resume; Settings Product syncs `get_ai_agent_enabled` + cache. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1435

- CPU window: monitoring idle parks UI-state retry + pin hydrate; Compact from localStorage on open (backend sync on Settings Product); drop duplicate Ollama configure; Agent Ops wait loops bail while parked. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1434

- CPU window: Settings credential/decorations IPC deferred to Settings open; Process Details open + Monitors settings list skip IPC/DOM while parked; resume rebuilds list if popover still open. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1433

- CPU window: Settings Product toggles AI-only on open; Discord / decorations / changelog / footer version park while occluded. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1432

- CPU window: Settings credential status + chat stream + Agent Ops digest park while occluded. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1431

- Agent Ops: session / schedule / run / knowledge previews skip DOM while parked; mid-flight live/session/knowledge reads drop paint. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1430

- CPU window: AI Chat / Ollama connection + model-list and Perplexity key-status skip IPC/DOM while parked; monitor history Map rebuild drops mid-flight. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1427

- CPU window: mid-flight Discord / monitors / history / Process Details / Agent Ops skip IPC+DOM while parked (`__macStatsWindowWorkPaused`). Blur clears Process Details live refresh; focus re-arms if modal open. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1424

- CPU window: blur cancels open-path first-metrics + late-open idle; skip post-await DOM when occluded; chart-line cancels pending unpark on park. Focus re-schedules metrics if never armed. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1420

- CPU window focus resume idle-defers sparkline unpark + Discord/logs/history/Disk Cleanup/Agent Ops/Monitors polls (≤30s). Blur cancels pending resume. Fixes resumeIdleWindowPolls undoing chart-line deferred unpark. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1416

- CPU window focused-open restores `scheduleMonitoringFeaturesOnce` (idle ≤300s). v0.1.1415 left that to Focused/resume or the 10m late fallback; Focused can race past load. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1415

- CPU window first metrics idle ≤30s; no focus `_forceProcessUpdate`; version IPC idle ≤120s after metrics; monitoring/Agent Ops idle ≤300s. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1412

- CPU window deferred open wiring: always bind DOM + arm metrics interval even when occluded; refresh() still no-ops while occluded. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1408

- CPU window WebView idle cut (#14 follow-up): Tauri `WindowEvent::Focused` parks/resumes idle polls; sparkline boot skips history IPC seed; ring skip ~70%; chart-line boot idle 180s; GPU warm 1800s. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1404

- CPU window WebView idle cut (#14 follow-up): park sparkline canvases to 1×1 on blur / `document.hidden`; skip paints when `!document.hasFocus()`; history chart `content-visibility: auto` + `contain: paint`; data-poster DPR 1 + opaque canvas; ring skip ~30%. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1400

- CPU window WebView idle cut (#14 follow-up): 600s UI polls/TTL, 120s backend/`get_cpu_details`/temp read, process cache 600s, HISTORY_POINTS 2, ring skip ~10%, chart-line boot via `requestIdleCallback`, collapsed Top Processes `content-visibility`, GPU warm deferred 20s. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1399

- CPU window WebView idle cut (#14 follow-up): 300s UI polls/TTL, 90s backend/`get_cpu_details`, HISTORY_POINTS 4, skip ring/DOM rAF + zero-size canvas paints when hidden, collapsed keep-header `content-visibility`, GPU warm deferred 12s. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1398

- CPU window WebView idle cut (#14 follow-up): 180s UI polls/TTL, no Agent Ops glance IPC while icon-hidden, 60s backend/`get_cpu_details`, 2 sparkline points, skip canvas when hidden, closed settings `content-visibility`. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1397

- Ollama-down debug.log spam cut: Circuit opened WARN ≤1/5min; model-list fail cooldown 5m; shared `/api/tags` waiters log once. P2 reliability / log-012.
- CPU window WebView idle cut (#14 follow-up): 120s UI polls/TTL, 45s backend/`get_cpu_details`, 4 sparkline points, Debug Log 120s + blur pause. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1394

- CPU window WebView idle cut (#14 follow-up): 60s process-cache TTL (was 5–10s), 60s metrics/history/Discord/monitors/Agent Ops, 20s backend loop, 8 sparkline points, Process Details 60s, Debug Log auto-refresh 30s. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1393

- CPU window WebView idle cut (#14 follow-up): 45s metrics/history polls, 15s backend loop, 12 sparkline points, collapsed Top Processes glance-only, Debug Log auto-refresh 10s, opaque action/power-strip chrome. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1391

- CPU window WebView idle cut (#14 follow-up): 20s metrics/history polls, 8s backend loop, 24 sparkline points, process-details 15s, Light flat body/shell, Dark no glow. Native digest test isolated from shared latest.json race. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1390

- CPU window WebView idle cut (#14 follow-up): 15s metrics/history polls, 5s backend loop while open, 36 sparkline points, Apple/Light opaque flat chrome + CSS contain. Operator age routes use whole-token `age` (harness_ops false-positive fix). P2 reliability / GitHub #14.

## Overnight merge — v0.1.1388

- CPU window WebView idle cut (#14 follow-up): macOS `transparent(false)`, 8s metrics poll, pause more timers when hidden, opaque sparklines, slower history poll. Ollama model-list fetch: 30s fail cooldown + WARN ≤1/5min when Ollama is down (debug.log spam cut). P2 reliability / GitHub #14 + log-012.

## Overnight merge — v0.1.1386

- CPU window WebView compositor cut (#14 follow-up). Agent Ops loading and Force Quit confirm stay static (no infinite CSS pulse). Theme `cpu.css` drops live `backdrop-filter` blur; frosted rgba panels stay. Thinking dots stay static. P2 reliability / GitHub #14.

## Overnight merge — v0.1.1385

- Changelog body keyboard hint sits in the theme HTML at the top of the changelog body. It no longer pops in after JavaScript loads. It stays hidden until two versions are on screen. The line says how to move across those version headings. Keyboard tips stay out of the layout. Design review / Changelog.

## Overnight merge — v0.1.1384

- Agent Ops row-selection Tips keyboard hint sits in the theme HTML under the tab-bar hint. It no longer pops in after JavaScript loads. The line says how to move across tabs, filters, and list rows. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1383

- AI Chat message-list keyboard hint sits in the theme HTML above the message list. It no longer pops in after JavaScript loads. It stays hidden until turns exist. The line says how to move across those messages. Keyboard tips stay out of the layout. Design review / feature-ai-chat.

## Overnight merge — 0.1.1382

- Perplexity setup keyboard hint sits in the theme HTML under key and Save key. It no longer pops in after JavaScript loads. It stays hidden until the setup panel is on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / Perplexity.

## Overnight merge — v0.1.1381

- Monitors detail toolbar keyboard hint sits with Check now and Remove on first paint. It no longer pops in after those buttons. It stays hidden until both buttons are on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / feature-monitors.

## Overnight merge — v0.1.1380

- Theme-list fallback keyboard hint sits in the theme HTML at the end of the theme list. It no longer pops in after JavaScript loads. It stays hidden until Settings is open and the Appearance section is missing. The line says how to move across those buttons. When Appearance is present, that section hint stays in charge. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1379

- Product setting keyboard hint sits in the theme HTML at the end of the Product setting. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. An open Help sheet swaps the line to the copy keys. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1378

- Appearance section keyboard hint sits in the theme HTML at the end of the Appearance section. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. The first theme crosses to the Settings header. Window frame last crosses to Product. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1377

- Credentials section keyboard hint sits in the theme HTML at the end of the Credentials section. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Discord token first crosses to the Settings header. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1376

- Slack settings toolbar keyboard hint sits in the theme HTML under the webhook field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Webhook first crosses to Telegram. Clear last crosses to the Settings header. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1375

- Telegram settings toolbar keyboard hint sits in the theme HTML under the token field, the chat id field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Token first crosses to Cursor. Clear last crosses to Slack. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1374

- Cursor agent settings toolbar keyboard hint sits in the theme HTML under the workspace field, the executable field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Workspace first crosses to Browser. Clear last crosses to Telegram. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1373

- Browser settings toolbar keyboard hint sits in the theme HTML under the path field, the port field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Path first crosses to MCP. Clear last crosses to Cursor. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1372

- MCP settings toolbar keyboard hint sits in the theme HTML under the URL field, the stdio field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. URL first crosses to Mastodon. Clear last crosses to Browser. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1371

- Mastodon settings toolbar keyboard hint sits in the theme HTML under the URL field, the token field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. URL first crosses to Redmine. Clear last crosses to MCP. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1370

- Redmine settings toolbar keyboard hint sits in the theme HTML under the URL field, the key field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. URL first crosses to Brave. Clear last crosses to Mastodon. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1369

- Brave settings toolbar keyboard hint sits in the theme HTML under the key field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Key first crosses to Perplexity. Clear last crosses to Redmine. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1368

- Discord settings toolbar keyboard hint sits in the theme HTML under the token field, Save, Clear, and View logs. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1367

- Perplexity settings toolbar keyboard hint sits in the theme HTML under the key field, Save, and Clear. It no longer pops in after JavaScript loads. It stays hidden until Settings is open. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1366

- Ollama settings toolbar keyboard hint sits in the theme HTML under the system prompt, Reset, and Save. It no longer pops in after JavaScript loads. It stays hidden until the popover is open. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / AI Chat.

## Overnight merge — v0.1.1365

- Ollama settings header keyboard hint sits in the theme HTML between the title and Close. It no longer pops in after JavaScript loads. It stays hidden until the title and Close are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / AI Chat.

## Overnight merge — v0.1.1364

- Settings header keyboard hint sits in the theme HTML between the title and Close. It no longer pops in after JavaScript loads. It stays hidden until the title and Close are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / Settings.

## Overnight merge — v0.1.1363

- Changelog header keyboard hint sits in the theme HTML between the title and Close. It no longer pops in after JavaScript loads. It stays hidden until the title and Close are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / Changelog.

## Overnight merge — v0.1.1362

- Process Details header keyboard hint sits in the theme HTML between the title and Close. It no longer pops in after JavaScript loads. It stays hidden until the title and Close are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / feature-processes.

## Overnight merge — v0.1.1361

- Monitors add-form keyboard hint sits in the theme HTML under the URL field, Cancel, and Add Monitor. It no longer pops in after JavaScript loads. It stays hidden until the add form is open. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / feature-monitors.

## Overnight merge — v0.1.1360

- Process Details Force Quit keyboard hint stays hidden until Advanced is open. Force Quit is not on screen while that section is closed. Keyboard tips stay out of the layout. Design review / feature-processes.

## Overnight merge — v0.1.1359

- Process Details Force Quit keyboard hint sits in the theme HTML inside the force-quit section. It no longer pops in after JavaScript loads. It stays hidden until Advanced and Force Quit are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / feature-processes.

## Overnight merge — v0.1.1358

- Process Details name and PID keyboard hint sits in the theme HTML inside the process detail hero. It no longer pops in after JavaScript loads. It stays hidden until the name and the PID are both on screen. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / feature-processes.

## Overnight merge — v0.1.1357

- Details keyboard hint sits in the theme HTML above the Details grid. It no longer pops in after JavaScript loads. The line says how to move across those values. A collapsed Details section still hides the line. Keyboard tips stay out of the layout. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1354

- Disk Cleanup scope list keyboard hint sits in the theme HTML above the scope list. It no longer pops in after JavaScript loads. It stays hidden until the list has rows. The line says how to move across those rows. Keyboard tips stay out of the layout. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1353

- Disk Cleanup category list keyboard hint sits in the theme HTML above the category list. It no longer pops in after JavaScript loads. It stays hidden until the list has rows. The line says how to move across those rows. Keyboard tips stay out of the layout. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1352

- Monitors list keyboard hint sits in the theme HTML above the monitor list. It no longer pops in after JavaScript loads. It stays hidden until the list has rows. The line says how to move across those rows. Keyboard tips stay out of the layout. Design review / feature-monitors.

## Overnight merge — v0.1.1351

- Top Processes list keyboard hint sits in the theme HTML above the process list. It no longer pops in after JavaScript loads. It stays hidden until the list has rows. The line says how to move across those rows. Keyboard tips stay out of the layout. Design review / feature-processes.

## Overnight merge — v0.1.1350

- Debug Log viewer keyboard hint sits in the theme HTML above the log viewer. It no longer pops in after JavaScript loads. It stays hidden until the log has lines. The line says how to move across those lines. The last line points back to Refresh. Keyboard tips stay out of the layout. Design review / Debug Log.

## Overnight merge — v0.1.1349

- Perplexity results keyboard hint sits in the theme HTML above the result list. It no longer pops in after JavaScript loads. It stays hidden until a search has results. The line says how to move across those results. Keyboard tips stay out of the layout. Design review / Perplexity.

## Overnight merge — v0.1.1348

- Perplexity search-box keyboard hint sits in the theme HTML under the query and Search. It no longer pops in after JavaScript loads. Before a search, the line says how to move across those controls. After results exist, the line points the query back to the last result. Keyboard tips stay out of the layout. Design review / Perplexity.

## Overnight merge — v0.1.1347

- Debug Log toolbar keyboard hint sits in the theme HTML under Refresh, Open in editor, and Auto-refresh. It no longer pops in after JavaScript loads. Before a log load, the line says how to move across those controls. After lines exist, the line points Refresh back to the last line. Keyboard tips stay out of the layout. Design review / Debug Log.

## Overnight merge — v0.1.1346

- Disk Cleanup add-scope toolbar keyboard hint sits in the theme HTML under the label, path, days, Recursive, and Add scope fields. It no longer pops in after JavaScript loads. The line says how to move across those fields. Keyboard tips stay out of the layout. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1345

- Disk Cleanup action toolbar keyboard hint sits in the theme HTML under Clean now, Refresh, and Save scopes. It no longer pops in after JavaScript loads. The line says how to move across those buttons. Keyboard tips stay out of the layout. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1344

- Disk Cleanup meta keyboard hint sits in the theme HTML under Reclaimable now, Next automatic run, Runs when, and Enabled scopes. It no longer pops in after JavaScript loads. The line says how to move across those cards. Keyboard tips stay out of the layout. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1343

- Header toolbar keyboard hint sits in the theme HTML under Refresh and Settings. It no longer pops in after JavaScript loads. The line says how to move across those buttons. Keyboard tips stay out of the layout. Design review / CPU window.

## Overnight merge — v0.1.1342

- Footer toolbar keyboard hint sits in the theme HTML under the version chip and the GitHub link. It no longer pops in after JavaScript loads. Before a panel opens, the line says the version chip opens the changelog. An open panel still replaces that line. Keyboard tips stay out of the layout. Design review / CPU window.

## Overnight merge — v0.1.1341

- Section icon-line keyboard hint sits in the theme HTML under Monitors, AI Chat, Perplexity, Debug Log, Discord, Disk Cleanup, and Agent Ops. It no longer pops in after JavaScript loads. The line says how to move across those icons. Keyboard tips stay out of the layout. Design review / CPU window.

## Overnight merge — v0.1.1340

- History sparkline keyboard hint sits in the theme HTML under CPU, GPU, Freq, and Temp. It no longer pops in after JavaScript loads. The line says how to move across those charts. Keyboard tips stay out of the layout. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1339

- Ring gauge keyboard hint sits in the theme HTML under CPU, GPU, Freq, and Temp. It no longer pops in after JavaScript loads. The line says how to move across those rings. Keyboard tips stay out of the layout. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1338

- Power strip keyboard hint sits in the theme HTML under Bat, LPM, and Power. It no longer pops in after JavaScript loads. The line says how to move across those chips. Keyboard tips stay out of the layout. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1337

- Agent Ops refresh-row keyboard hint sits in the theme HTML under Refresh, Refresh digest, and Updated. It no longer pops in after JavaScript loads. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1336

- Agent Ops health-strip keyboard hint sits in the theme HTML under Version, Discord, Redmine, Next schedule, Last delivery, and Digest. It no longer pops in after JavaScript loads. The line says how to move across those cards. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1335

- Agent Ops overview keyboard hint sits in the theme HTML under the overview cards. It no longer pops in after JavaScript loads. The line says how to move across Agents, Schedules, Live, Knowledge, Recent chats, Runs, and Digest. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1334

- Agent Ops filter-row keyboard hint sits in the theme HTML under the search box, the match count, and Clear. It no longer pops in after JavaScript loads. It stays hidden until a search shows the match count and Clear. The line says how to move across those controls. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1333

- Agent Ops edit-actions keyboard hint sits in the theme HTML under Save, Load into AI Chat, and Back. It no longer pops in after JavaScript loads. The line says how to move across those actions. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1332

- Agent Ops file-tab keyboard hint sits in the theme HTML under Soul, Skill, and Mood. It no longer pops in after JavaScript loads. The line says how to move across those tabs. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1330

- Agent Ops tab-bar keyboard hint sits in the theme HTML under the tabs. It no longer pops in after JavaScript loads. The line says how to move across Overview, Agents, Sessions, Schedules, Knowledge, and Runs. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1329

- Agent Ops Insights keyboard hint sits in the theme HTML under the Insights card. It no longer pops in after JavaScript loads. It stays hidden until two clickable insight lines are on screen. The line says how to move across those lines. A runs refresh keeps the same hint. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1328

- Agent Ops Knowledge preview-row keyboard hint sits in the theme HTML under the knowledge preview. It no longer pops in after JavaScript loads. It stays hidden until Copy and Load into AI Chat are both on screen. The line says how to move across those actions. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1327

- Agent Ops Schedules preview-row keyboard hint sits in the theme HTML under the schedule preview. It no longer pops in after JavaScript loads. It stays hidden until Copy and Load into AI Chat are both on screen. The line says how to move across those actions. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1326

- Agent Ops Sessions preview-row keyboard hint sits in the theme HTML under the session preview. It no longer pops in after JavaScript loads. It stays hidden until Copy and Load into AI Chat are both on screen. The line says how to move across those actions. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1325

- Agent Ops Runs preview-row keyboard hint sits in the theme HTML under the run preview. It no longer pops in after JavaScript loads. It stays hidden until Copy and Load into AI Chat are both on screen. The line says how to move across those actions. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1324

- Agent Ops Runs filter-chip keyboard hint sits in the theme HTML under All · Instant · Lite · Direct · Slow · Fail. It no longer pops in after JavaScript loads. Before a load it says how to move across the chips. At the start, Up goes to the Agent Ops icon. At the end, the keys go to the run list. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1323

- Agent Ops Knowledge filter-chip keyboard hint sits in the theme HTML under All · Discord · Core. It no longer pops in after JavaScript loads. Before a load it says how to move across the chips. At the start, Up goes to the Agent Ops icon. At the end, the keys go to the knowledge list. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1322

- Agent Ops Schedules filter-chip keyboard hint sits in the theme HTML under All · Jobs · Deliveries. It no longer pops in after JavaScript loads. Before a load it says how to move across the chips. At the start, Up goes to the Agent Ops icon. At the end, the keys go to the schedule list. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1321

- Agent Ops Sessions filter-chip keyboard hint sits in the theme HTML under All · Live · Files. It no longer pops in after JavaScript loads. Before a load it says how to move across the chips. At the start, Up goes to the Agent Ops icon. At the end, the keys go to the session list. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1320

- Agent Ops Agents filter-chip keyboard hint sits in the theme HTML under All · On · Off. It no longer pops in after JavaScript loads. Before a load it says how to move across the chips. At the start, Up goes to the Agent Ops icon. At the end, the keys go to the agent list. Keyboard tips stay out of the layout. Design review / feature-agent-ops.

## Overnight merge — v0.1.1319

- Disk Cleanup category filter-chip keyboard hint sits in the theme HTML under All · Reclaim · Big · Clean. It no longer pops in after JavaScript loads. Before a load it says how to move across the chips. At the start, Up goes to the Disk Cleanup icon. At the end, the keys go to the category list. Keyboard tips stay out of the layout. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1318

- Disk Cleanup scope filter-chip keyboard hint sits in the theme HTML under All · On · Off. It no longer pops in after JavaScript loads. Before a load it says how to move across the chips. At the start, Up goes to the Disk Cleanup icon. At the end, the keys go to the scope list. Keyboard tips stay out of the layout. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1317

- Debug Log filter-chip keyboard hint sits in the theme HTML under All · Error · Warn. It no longer pops in after JavaScript loads. Before a log load it says how to move across the chips. At the start, Up goes to the Debug Log icon. At the end, the keys go to the log viewer. Keyboard tips stay out of the layout. Design review / Debug Log.

## Overnight merge — v0.1.1316

- Perplexity filter-chip keyboard hint sits in the theme HTML under All · Top · Snippet. It no longer pops in after JavaScript loads. Before a search it says how to move across the chips. At the start, Up goes to the Perplexity icon. At the end, the keys go to the result list. Keyboard tips stay out of the layout. Design review / Perplexity.

## Overnight merge — v0.1.1315

- Monitors filter-chip keyboard hint sits in the theme HTML under All · Up · Down · Slow. It no longer pops in after JavaScript loads. Before a check it says how to move across the chips. At the start, Up goes to the Monitors icon. At the end, the keys go to the monitor list. Keyboard tips stay out of the layout. Design review / feature-monitors.

## Overnight merge — v0.1.1314

- Top Processes filter-chip keyboard hint sits in the theme HTML under All · Pinned · Hot. It no longer pops in after JavaScript loads. Before a sample it says how to move across the chips. Keyboard tips stay out of the layout. Design review / feature-processes.

## Overnight merge — v0.1.1313

- AI Chat filter-chip keyboard hint sits in the theme HTML under All · You · Assistant · Errors. It no longer pops in after JavaScript loads. Before a turn it says how to move across the chips. Keyboard tips stay out of the layout. Design review / feature-ai-chat.

## Overnight merge — v0.1.1312

- AI Chat starter-chip keyboard hint sits in the theme HTML under the starter chips. It no longer pops in after JavaScript loads. Before a turn it says how to move across the chips and put a prompt in the composer. Design review / feature-ai-chat.

## Overnight merge — v0.1.1311

- AI Chat composer keyboard hint sits in the theme HTML under Send. It no longer pops in after JavaScript loads. Before a turn it matches the starter-chip line. Keyboard tips stay out of the layout. Design review / feature-ai-chat.

## Overnight merge — v0.1.1310

- AI Chat errors glance sits in the theme HTML under the last-answer glance. It no longer pops in after JavaScript loads. Before a failed turn it says None yet and stays hidden. Design review / feature-ai-chat.

## Overnight merge — v0.1.1309

- AI Chat last-answer glance sits in the theme HTML under the turn glance. It no longer pops in after JavaScript loads. Before a reply it says None yet and stays hidden. Design review / feature-ai-chat.

## Overnight merge — v0.1.1308

- AI Chat turn glance sits in the theme HTML under the model glance. It no longer pops in after JavaScript loads. Before a turn it says None yet and stays hidden. Design review / feature-ai-chat.

## Overnight merge — v0.1.1307

- AI Chat collapsed glance sits in the theme HTML under the header. It no longer pops in after JavaScript loads. Before a connection check it says Not set · configure URL. It stays hidden until the section is collapsed. Design review / feature-ai-chat.

## Overnight merge — v0.1.1306

- AI Chat offline attention glance sits in the theme HTML above the message filters. It no longer pops in after JavaScript loads. Before a connection check it says Chat · Not set · configure URL. Design review / feature-ai-chat.

## Overnight merge — v0.1.1305

- AI Chat model glance sits in the theme HTML under the header. It no longer pops in after JavaScript loads. Before a connection check it says Not set · configure URL. Design review / feature-ai-chat.

## Overnight merge — v0.1.1304

- AI Chat All, You, Assistant, and Errors chips sit in the theme HTML above the message list. They no longer pop in after JavaScript loads. A zero You, Assistant, or Errors count still says None yet. The row stays visible on first paint. Design review / feature-ai-chat.

## Overnight merge — v0.1.1303

- Debug Log All, Error, and Warn chips sit in the theme HTML on the log toolbar. They no longer pop in after JavaScript loads. A zero Error or Warn count still says None yet. Design review / Debug Log.

## Overnight merge — v0.1.1302

- Perplexity All, Top, and Snippet chips sit in the theme HTML above the result list. They no longer pop in after JavaScript loads. A zero Top or Snippet count still says None yet. Design review / Perplexity.

## Overnight merge — v0.1.1301

- Monitors All, Up, Down, and Slow chips sit in the theme HTML above the monitor list. They no longer pop in after JavaScript loads. A zero Up, Down, or Slow count still says None yet. Design review / feature-monitors.

## Overnight merge — v0.1.1300

- Top Processes All, Pinned, and Hot chips sit in the theme HTML above the process list. They no longer pop in after JavaScript loads. A zero Pinned or Hot count still says None yet. Design review / feature-processes.

## Overnight merge — v0.1.1299

- Disk Cleanup category All, Reclaim, Big, and Clean chips sit in the theme HTML above the category list. They no longer pop in after JavaScript loads. A zero Reclaim, Big, or Clean count still says None yet. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1298

- Disk Cleanup scope All, On, and Off chips sit in the theme HTML above the scope list. They no longer pop in after JavaScript loads. A zero On or Off count still says None yet. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1297

- Disk Cleanup scope On and Off counts stay None yet when the count is zero. The chips no longer flash 0 before a load. A positive count still replaces None yet. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1296

- AI Chat You, Assistant, and Errors counts stay None yet when the count is zero. The chips no longer flash 0 before a load. A positive count still replaces None yet. Design review / feature-ai-chat.

## Overnight merge — v0.1.1295

- Perplexity Top and Snippet counts stay None yet when the count is zero. The chips no longer flash 0 before a load. A positive count still replaces None yet. Design review / Perplexity.

## Overnight merge — v0.1.1294

- Debug Log Error and Warn counts stay None yet when the count is zero. The chips no longer flash 0 before a load. A positive count still replaces None yet. Design review / Debug Log.

## Overnight merge — v0.1.1293

- Disk Cleanup Reclaim, Big, and Clean counts stay None yet when the count is zero. The chips no longer flash 0 before a load. A positive count still replaces None yet. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1292

- Monitors Up, Down, and Slow counts stay None yet when the count is zero. The chips no longer flash 0 before a load. A positive count still replaces None yet. Design review / feature-monitors.

## Overnight merge — v0.1.1291

- Top Processes Pinned and Hot counts stay None yet when the count is zero. The chips no longer flash 0 before a load. A positive count still replaces None yet. Design review / feature-processes.

## Overnight merge — v0.1.1290

- Agent Ops kind-filter counts stay None yet when the count is zero. On/Off, Live/Files, Jobs/Deliveries, Discord/Core, and Runs lanes no longer flash 0 before a load. A positive count still replaces None yet. Design review / feature-agent-ops.

## Overnight merge — v0.1.1289

- Agent Ops filter match chips (N/M) and Clear stay hidden in the theme HTML until a query. They no longer pop in after JavaScript loads. A search still shows the count and Clear. Design review / feature-agent-ops.

## Overnight merge — v0.1.1288

- Agent Ops tab counts stay None yet when the count is zero. The pill no longer shrinks to 0 after a refresh. A positive count still replaces None yet. Design review / feature-agent-ops.

## Overnight merge — v0.1.1287

- Agent Ops Updated stamp first paint: the Refresh row shows None yet in the theme HTML. The stamp no longer pops in beside Refresh after the first refresh. A real age still replaces None yet (Updated just now). Design review / feature-agent-ops.

## Overnight merge — v0.1.1286

- Agent Ops overview head counts first paint: Agents, Schedules, Knowledge, Recent chats, and Runs show None yet. Live shows Quiet. Digest shows Queue clear. The card titles no longer grow when JavaScript loads the counts. A real count still replaces those words. Design review / feature-agent-ops.

## Overnight merge — v0.1.1285

- Agent Ops tab counts first paint: Agents, Sessions, Schedules, Knowledge, and Runs show a None yet pill in the theme HTML. The tabs no longer grow when JavaScript loads the counts. A real count still replaces None yet. Design review / feature-agent-ops.

## Overnight merge — v0.1.1284

- Agent Ops Sessions filter first paint: All · Live · Files chips are in the theme HTML. They no longer pop in after JavaScript loads. Counts still update after a load. Design review / feature-agent-ops.

## Overnight merge — v0.1.1283

- Agent Ops filter first paint: Agents (All · On · Off), Schedules (All · Jobs · Deliveries), Knowledge (All · Discord · Core), and Runs lanes are in the theme HTML. Those rows no longer pop in after JavaScript loads. Design review / feature-agent-ops.

## Overnight merge — v0.1.1282

- Agent Ops Refresh first paint: Refresh and Refresh digest sit under the health cards in the theme HTML. They no longer jump up from the bottom of Agent Ops after JavaScript loads. Design review / feature-agent-ops.

## Overnight merge — v0.1.1281

- Agent Ops tab strip first paint: Overview (press 0) and digit keys 1–5 are in the theme HTML. The strip no longer grows after JavaScript loads. Press 0 still jumps to the overview. Design review / feature-agent-ops.

## Overnight merge — v0.1.1280

- Agent Ops overview first paint: Agents, Runs, and Digest cards are in the theme HTML with None yet. The grid no longer jumps from four cards to seven after JavaScript loads. Design review / feature-agent-ops.

## Overnight merge — v0.1.1279

- Apple CPU and Freq rings: the progress arc matches the track (same path as GPU and Temp). The fill no longer rides a shorter curve that misses the gauge. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1278

- Low Power Mode first paint: the power strip shows Low Power Mode (LPM) and None yet in the theme HTML. The chip no longer pops in after JavaScript loads. On and Off still replace None yet after a sample. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1277

- Ring labels first paint: Freq and Temp (same words as the sparklines). The rings no longer flash Frequency and Temperature until JavaScript loads. Hover still shows the full words. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1276

- GPU history sparkline first paint: the GPU chart is in the theme HTML (CPU · GPU · Freq · Temp). The row no longer jumps from three columns to four after JS loads. Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1275

- AI Chat message list first paint: Nothing here yet — set an Ollama URL (not a blank list) until chat loads. Starter chips are already there. A real transcript still replaces that empty state. Design review / feature-ai-chat.

## Overnight merge — v0.1.1274

- Perplexity results first paint: Nothing here yet — search the web (not a blank region) until a search returns. A real search still replaces that line. Design review / Perplexity.

## Overnight merge — v0.1.1273

- Debug Log viewer first paint: Nothing here yet — loads when you open Debug Log (not Expand to load log…) until the tail loads. A real tail still replaces that line. Design review / Debug Log.

## Overnight merge — v0.1.1272

- Disk Cleanup last-run panel first paint: Not yet this install — will run on launch (not a blank panel) until status loads. A real last run still replaces that line. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1271

- Disk Cleanup scopes list first paint: No scopes yet + Add a scope (not a blank list) until scopes load. Real rows still replace that empty state. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1270

- Disk Cleanup category list first paint: Nothing to reclaim yet + Review scopes (not a blank list) until categories load. Real rows still replace that empty state. Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1269

- Top Processes list first paint: Waiting for process samples — opens with the CPU window (not a blank list) until samples load. Real rows still replace that line. Design review / feature-processes.

## Overnight merge — v0.1.1268

- Monitors list first paint: Nothing watching yet + Add a monitor (not a blank list) until sites load. Real rows still replace that empty state. Design review / feature-monitors.

## Overnight merge — v0.1.1267

- Monitors summary first paint: None yet (not 0 / 0 sites up) until a real check. Real up counts and DOWN lines still show. Design review / feature-monitors.

## Overnight merge — v0.1.1266

- Footer theme label first paint: None yet (not a fake v0.0.3). The theme name and real version still show after load. Design review / footer.

## Overnight merge — v0.1.1265

- Insights card first paint: calm None yet (not blank card). Design review / feature-agent-ops.

## Overnight merge — v0.1.1264

- Sessions / Schedules / Knowledge / Runs list first paint: calm empty titles (not blank lists). Design review / feature-agent-ops.

## Overnight merge — v0.1.1263

- Agents tab first paint: No agents yet (not blank list). Design review / feature-agent-ops.

## Overnight merge — v0.1.1262

- Ollama model select + Changelog first paint: None yet (not Loading models… / Loading changelog…). Design review / feature-ai-chat.

## Overnight merge — v0.1.1261

- Agent Ops Overview first paint: None yet (not Loading…). Design review / feature-agent-ops.

## Overnight merge — v0.1.1260

- Low Power Mode strip first paint: None yet (not …). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1259

- Details load averages + CPU/GPU Power first paint: None yet (not 0.0 / 0.0 W). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1258

- CPU / GPU ring first paint: None yet (not 0%). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1257

- Perplexity header config status + Ollama model text first paint: Unknown (not —). Model glance ignores Unknown/—/None yet placeholders. Design review / feature-ai-chat · Perplexity header.

## Overnight merge — v0.1.1256

- Settings credentials first paint: Unknown (not —) for Discord / Perplexity / Brave / Redmine / Mastodon / MCP / Browser / Cursor / Telegram / Slack. Design review / settings.

## Overnight merge — v0.1.1255

- Disk Cleanup summary / reclaim first paint: None yet (not —). Monitors summary Avg None yet (not Avg -- ms). Design review / feature-disk-cleanup · feature-monitors.

## Overnight merge — v0.1.1253

- Temp ring first paint: None yet (not 0°C). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1252

- Chip subtitle empty: Unknown (not —); CPU/GPU ring first-paint subtext + Details RAM/Up: None yet (not — / 0h). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1250

- Temp / Freq ring + Power empty: None yet (not — / -- W). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1249

- Power strip empty Heat / Up / RAM / SSD: None yet (not —). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1248

- Details collapsed glance empty Load / RAM / Up: None yet (not —). Design review / feature-cpu-metrics.

## Overnight merge — v0.1.1247

- Top Processes Top GPU empty metric: None yet (not —) on glance, list column, and Current GPU details. Design review / feature-processes.

## Overnight merge — v0.1.1246

- Monitors summary empty avg: Avg None yet (not Avg 0 ms); settings empty URL/latency Unknown / None yet; DOWN summary failure Unknown. Design review / feature-monitors.

## Overnight merge — v0.1.1245

- Monitors empty identity: missing URL/host say Unknown; empty latency / last-check age say None yet; empty DOWN failure say Unknown (Top Processes / Agents parity). Design review / feature-monitors.

## Overnight merge — v0.1.1244

- Disk Cleanup empty identity: missing scope label/kind, category title, last-run trigger or category label say Unknown; empty category policy say None yet (Top Processes / Agents parity). Design review / feature-disk-cleanup.

## Overnight merge — v0.1.1243

- Top Processes empty identity: missing name/parent/user say Unknown; empty Top CPU/RAM glance metrics say None yet (Agents/Live parity). Design review / feature-processes.

## Overnight merge — v0.1.1242

- AI Chat empty glance preview: blank turn / last-answer glance text say None yet (Live/Sessions preview parity). Real previews still show. Design review / feature-ai-chat.

## Overnight merge — v0.1.1241

- Health Version empty uptime: missing or zero process uptime say None yet after the version (Digest age parity). Real uptimes still show.

## Overnight merge — v0.1.1240

- Health Digest empty age: missing or unparseable `digest_generated_at` say None yet after open/stale (Last delivery / Next schedule parity). Real ages still show.

## Overnight merge — v0.1.1239

- Health Last delivery empty summary preview: missing summary say None yet after the age (Next schedule preview parity). Real delivery summaries still show.

## Overnight merge — v0.1.1238

- Health Next schedule empty task preview: missing nextTaskPreview say None yet after the ETA (Live / Sessions preview parity). Real next-task text still shows.

## Overnight merge — v0.1.1237

- Live / Sessions empty preview: missing preview snippet say None yet on Overview Live, Sessions live, and Sessions files (schedule task parity). Real previews still show.

## Overnight merge — v0.1.1236

- Knowledge / Sessions / Live empty size, line count, or message count say Unknown (not undefined / NaN). Real counts still show.

## Overnight merge — v0.1.1235

- Schedule / delivery empty body: missing task or summary say None yet on Overview and Schedules lists (preview already matched). Real text still show.

## Overnight merge — v0.1.1234

- Delivery empty schedule id: missing schedule_id say Unknown on Overview last delivery, Schedules deliveries list, and preview (schedule id parity). Real ids still show.

## Overnight merge — v0.1.1233

- Insights Slowest empty wall / lane / question: missing wall say Unknown; missing lane say Unknown; missing question say None yet (list). Candidate preview omits wall when missing so it says Unknown (not 0 ms). Real values still show.


## Overnight merge — v0.1.1232

- Insights Candidates empty kind / reason / question: missing kind or wall say Unknown; missing reason or question say None yet (list + preview). Real text still shows.


## Overnight merge — v0.1.1231

- Runs / Schedules empty question / id: missing question preview say None yet; missing schedule id say Unknown; empty task / delivery summary / session body say None yet (Overview, lists, Slowest, preview). Real text still shows.


## Overnight merge — v0.1.1230

- Knowledge / Sessions empty file title: missing name or slug say Unknown on Overview Knowledge, Overview Recent, Knowledge list, and Sessions files (Agents / Live identity parity). Real names still show.


## Overnight merge — v0.1.1229

- Agents empty identity: missing name or slug/id say Unknown on Overview Agents, Agents list, and agent detail (Live identity parity). Real names and ids still show.

## Overnight merge — v0.1.1228

- Live empty identity: missing source or session id say Unknown on Overview Live and Sessions live (run meta parity). Real source and id still show.

## Overnight merge — v0.1.1227

- Sessions / Knowledge empty meta: missing last activity, file age, kind, or source hint say Unknown (run meta parity). Real ages and labels still show.

## Overnight merge — v0.1.1226

- Disk Cleanup empty meta: Next automatic run, Runs when, Enabled scopes, and missing last-run times say None yet (schedule / delivery parity). Real labels still show.

## Overnight merge — v0.1.1225

- Delivery empty when: Overview Last delivery, Schedules deliveries list, and preview say None yet when the timestamp is missing (health Last delivery parity). Real ages still show.

## Overnight merge — v0.1.1224

- Runs empty meta: Overview, Runs list, Slowest, and preview say Unknown when lane, wall, when, or request id is missing. Empty tools say None yet (schedule / health empty calm parity). Real values still show.

## Overnight merge — v0.1.1223

- Schedule empty next/when: Overview, Schedules tab, and preview say None yet when next run or cron/at is missing (health Next schedule parity). Real next times still show.

## Overnight merge — v0.1.1222

- Redmine unloaded: health card says Unknown when the payload is missing (Discord / Version parity). Ok / Not configured / degraded still show.

## Overnight merge — v0.1.1221

- Health unloaded: Version, Next schedule, Last delivery, and Digest say Unknown when the payload is missing (Discord parity). Empty jobs still say None yet. Real values still show.

## Overnight merge — v0.1.1220

- Discord empty: health card and collapsed glance say Unknown when gateway string is empty. Ready / Offline / reconnect ages still show.

## Overnight merge — v0.1.1219

- Health Next schedule / Last delivery: empty jobs say None yet; empty deliveries say None yet (overview head parity). Real ETA and ages still show.

## Overnight merge — v0.1.1218

- Overview empty head pills: Agents / Runs / Schedules / Knowledge / Recent say None yet; Live says Quiet (Digest Queue clear parity). Real counts still show.

## Overnight merge — v0.1.1217

- `/insights` empty sections: when turns exist but Lanes, Top tools, Slowest, or Candidates are empty, the report says no lanes yet, no tools yet, nothing slow, and nothing open. Real rows still show. Zero turns stay “No turns”.

## Overnight merge — v0.1.1216

- Insights fail words: when no turn failed, the Insights header, `/insights`, and “how many runs” say no fails. A real fail count still shows.

## Overnight merge — v0.1.1215

- Digest status lines (`/status`, `/insights`, digest age, `digest open`) say queue clear and nothing stale when those counts are zero. “How many open” still returns a number.

## Overnight merge — v0.1.1214

- CPU ring labels: Frequency and Temperature say Freq and Temp so they line up with the sparklines. Hover still shows the long name.

## Overnight merge — v0.1.1213

- Agent Ops digest zeros: when the digester queue is clear, the health Digest line, overview card, and Insights header say queue clear and nothing stale (not 0 open / 0 stale). Overview empty copy matches Insights: quiet is a fail. Health says p50 n/a when turns exist but the latency sample is noise-filtered. Counts above zero still show.

## Overnight merge — v0.1.1212

- Agent Ops Insights header: when Runs Insights has turns but the latency sample is noise-filtered, mean and max say n/a (Latency “Nothing to measure” parity). Real mean and max still show when the sample has turns.

## Overnight merge — v0.1.1211

- Agent Ops Insights Lanes empty calm: when Runs Insights has turns but lane mix is empty, show warm “No lanes yet” + solid accent wash (Top tools / Digest Queue clear parity). Lane counts still surface when turns have a lane mix.

## Overnight merge — v0.1.1210

- Agent Ops Insights Top tools empty calm: when Runs Insights has turns but digester Top tools is empty, show warm “No tools yet” + solid accent wash (Latency / Slowest / Digest Queue clear parity). Tool counts still surface when turns used tools.

## Overnight merge — v0.1.1209

- Agent Ops Insights Latency empty calm: when Runs Insights has turns but latency sample is noise-filtered (p50 n/a), show warm “Nothing to measure” + solid accent wash (Slowest / Digest Queue clear parity). Real p50/mean/max still surface when the sample has turns.

## Overnight merge — v0.1.1208

- Agent Ops Insights Stale empty calm: when Runs Insights has turns but digester stale is empty, show warm “Nothing stale” + solid accent wash (Candidates / Slowest / Digest Queue clear parity). Stale hints surface when present.

## Overnight merge — v0.1.1207

- Agent Ops Insights Candidates empty calm: when Runs Insights has turns but Candidates is empty, show warm “Nothing open” + solid accent wash (Slowest / Digest Queue clear parity).

## Overnight merge — v0.1.1206

- Agent Ops Insights Slowest empty calm: when Runs Insights has turns but Slowest is empty, show warm “Nothing slow” + solid accent wash (Digest Queue clear parity).

## Overnight merge — v0.1.1205

- Agent Ops Digest open empty calm: when Insights has runs but open count is zero, Queue clear uses warm title + solid accent wash (true-empty / filter-miss parity).

## Overnight merge — v0.1.1204

- Agent Ops true-empty calm: empty Agents/Sessions/Schedules/Knowledge/Runs tabs use warm title + solid accent wash (filter-miss / overview Ready parity). Knowledge empty copy drops the raw home-path dump.

## Overnight merge — v0.1.1203

- Debug Log inventory counts answer only inventory asks (“how many log errors”, “error count”, “count the errors”, “number of warnings”). “Delete these errors”, “clear the log”, and “export warnings” stay with the agent.

## Overnight merge — v0.1.1202

- Keep / discard inventory counts answer only inventory asks (“how many keeps”, “keep count”, “count the keeps”, “number of discards”). “Delete these keeps”, “clear the discards”, and “export keeps” stay with the agent.

## Overnight merge — v0.1.1201

- Schedule / delivery inventory counts answer only inventory asks (“how many schedules”, “job count”, “count the schedules”, “number of deliveries”). “Delete these schedules”, “clear the jobs”, “export deliveries”, “pause this schedule”, and “run this job” stay with the agent.

## Overnight merge — v0.1.1200

- Digest open inventory counts answer only inventory asks (“how many open candidates”, “digest count”, “open count”). “Open digest” / “digest open” stay the read-only snapshot. “Delete open candidates” and other verbs stay with the agent.

## Overnight merge — v0.1.1199

- Runs inventory counts answer only inventory asks (“how many runs”, “run count”, “number of failed runs”). “Delete these runs”, “clear the runs”, “export runs”, and other verb phrases stay with the agent. Bare “runs” stays off the count lane. “How many runs” still counts.

## Overnight merge — v0.1.1198

- Operator inventory counts (agents, monitors, tasks, skills, plugins, knowledge) answer only inventory asks (“how many …”, “… count”, “number of …”). “Delete this agent”, “run this skill”, “check this monitor”, and other verbs stay with the agent. “How many agents” still counts. Session inventory-only from v0.1.1197 stays.

## Overnight merge — v0.1.1197

- Session count answers only inventory asks (“how many sessions”, “session count”, “number of sessions”, “count sessions”). “Edit this session”, “hide this session”, “move this session”, “copy this session”, “refresh this session”, and “update this session” stay with the agent. “How many sessions” still counts. “Open sessions” still lists. The verb-exclusion list is gone.

## Overnight merge — v0.1.1196

- Session count: “exit this session”, “leave this session”, and “abandon this session” no longer answer with the session count. Exit stays with the agent. “How many sessions” still counts. “Kill this session” and “stop this session” still stay with the agent.

## Overnight merge — v0.1.1195

- Session count: “kill this session”, “destroy this session”, and “drop this session” no longer answer with the session count. Kill stays with the agent. The match uses the kill word, so “how many skills” still counts skills. “How many sessions” still counts. “Stop this session” and “abort this session” still stay with the agent.

## Overnight merge — v0.1.1194

- Session count: “abort this session”, “cancel this session”, and “terminate this session” no longer answer with the session count. Abort stays with the agent. “How many sessions” still counts. “Stop this session” still stays with the agent. “Kill this session” stays for a later tick (the word “kill” sits inside “skill”).

## Overnight merge — v0.1.1193

- Session count: “stop this session”, “halt this session”, “pause this session”, and “quit this session” no longer answer with the session count. Stop stays with the agent. “How many sessions” still counts. “Start this session” still stays with the agent. “Abort this session” stays for a later tick.

## Overnight merge — v0.1.1192

- Session count: “start this session”, “begin this session”, “launch this session”, and “restart this session” no longer answer with the session count. Start stays with the agent. “How many sessions” still counts. “Star this session” still stays with the agent. “Stop this session” stays for a later tick.

## Overnight merge — v0.1.1191

- Session count: “pin this session”, “bookmark this session”, “star this session”, and “favorite this session” no longer answer with the session count. Pin stays with the agent. “How many sessions” still counts. “Saved sessions” still lists. “Start this session” stays for a later tick (the word “star” sits inside “start”).

## Overnight merge — v0.1.1190

- Session count: “import this session”, “load this session”, “merge this session”, and “attach this session” no longer answer with the session count. Import stays with the agent. “How many sessions” still counts. “Saved sessions” still lists.

## Overnight merge — v0.1.1189

- Session count: “restore this session”, “recover this session”, “reload this session”, and “revert this session” no longer answer with the session count. Restore stays with the agent. “How many sessions” still counts. “Saved sessions” still lists.

## Overnight merge — v0.1.1188

- Session count: “save this session”, “store this session”, and “backup this session” no longer answer with the session count. Save stays with the agent. “How many sessions” still counts. “Saved sessions” still lists.

## Overnight merge — v0.1.1187

- Session count: “search this session”, “find this session”, and “lookup this session” no longer answer with the session count. Search stays with the agent. “How many sessions” still counts.

## Overnight merge — v0.1.1186

- Session count: “export this session”, “share this session”, and “archive this session” no longer answer with the session count. Export stays with the agent. “How many sessions” still counts.

## Overnight merge — v0.1.1185

- Session count: “fork this session”, “duplicate this session”, and “clone this session” no longer answer with the session count. Fork stays with the agent. “How many sessions” still counts.

## Overnight merge — v0.1.1184

- Session count: “resume this session”, “open this session”, “switch this session”, and “continue this session” no longer answer with the session count. Resume, open, and switch stay with the agent. “Open sessions” still lists. “How many sessions” still counts.

## Overnight merge — v0.1.1183

- Session count: “summarize this session”, “rename this session”, “session summary”, and “title this session” no longer answer with the session count. Summarize and rename stay with the agent. “How many sessions” still counts.

## Overnight merge — v0.1.1182

- Session count: “delete this session”, “remove this session”, “end this session”, and “close this session” no longer answer with the session count. Delete and close stay with the agent. “How many sessions” still counts.

## Overnight merge — v0.1.1181

- Session count: “reset this session”, “clear this session”, and “new session” no longer answer with the session count. Reset stays with the agent. “How many sessions” still counts. Phrase-file path, size, and age stay on those lanes.

## Overnight merge — v0.1.1180

- Session count: “compact this session” no longer answers with the session count. Compaction stays with the agent. “How many sessions” still counts.

## Overnight merge — v0.1.1179

- App uptime instant: “how long have you been running”, “app uptime”, and “process uptime” answer with how long mac-stats has been up (no LLM). “What’s the uptime” and “how long has the Mac been up” stay the Up chip. “Why” stays with the agent.

## Overnight merge — v0.1.1178

- Top GPU instant: “what's using the most gpu”, “which process is using the most gpu”, and “what's eating the gpu” name that one process (no LLM). `/processes` stays the full list. “Hot processes” stays the Hot filter. “How much gpu” and “is the gpu hot” stay the GPU ring. “Why” stays with the agent.

## Overnight merge — v0.1.1177

- Top RAM instant: “what's using the most ram”, “which process is using the most memory”, and “what's eating the ram” name that one process (no LLM). `/processes` stays the full list. “Hot processes” stays the Hot filter. “How much ram” stays the percent chip. “How big is memory” stays installed RAM. “Why” stays with the agent.

## Overnight merge — v0.1.1176

- Top CPU instant: “what's using the most cpu”, “which process is using the most cpu”, and “what's eating the cpu” name that one process (no LLM). `/processes` stays the full list. “Hot processes” stays the Hot filter. “How much cpu” stays the CPU ring. “Why” stays with the agent. `CURSOR_AGENT:` tool calls no longer answer with the agent count.

## Overnight merge — v0.1.1175

- Disk free instant: “how much free space”, “how much space is left”, and “how many gb free” answer with free disk bytes (no LLM). “How much disk is used” and bare “free space” stay the percent chip. Disk Cleanup stays on `/disk`. “Why” stays with the agent.

## Overnight merge — v0.1.1173

- Chip instant: “what chip is this”, “what processor”, and “cpu name” answer with the Apple chip name and core count (no LLM). “What’s the CPU” stays the CPU ring. “Which model” stays Ollama. “Why” stays with the agent.

## Overnight merge — v0.1.1172

- RAM size instant: “how big is memory”, “memory size”, and “how many gb of ram” answer with installed RAM plus used (no LLM). “How much ram is used” stays the percent chip. Notes size and `memory.md` size stay on disk. “Why” stays with the agent.

## Overnight merge — v0.1.1171

- Charging instant: “is it charging”, “is the battery charging”, and “is it plugged in” use the Bat chip (percent and charging, no LLM). “Why is it charging” stays with the agent. “Is the battery low” stays the same chip.

## Overnight merge — v0.1.1170

- Heat-high instant: “is the heat high”, “is the thermal high”, and “is it throttling” use the Heat chip (thermal state, no LLM). “Why is the heat high” stays with the agent. “Is the cpu hot” stays the Temp ring. “Hot processes” stays the process list.

## Overnight merge — v0.1.1169

- P-core / E-core clock instant: “how fast are the p cores”, “p core frequency”, and “is the p core high” use the P-core clock (GHz, no LLM). “e core frequency” and “how fast are the e cores” use the E-core clock. “Why is the p core high” stays with the agent. “How fast is the cpu” stays the Freq ring. “How fast is the gpu” stays with the agent.

## Overnight merge — v0.1.1168

- Load 5m / 15m instant: “what’s the 5 minute load”, “15 minute load”, and “is the 5 minute load high” use the Load 5m or 15m chip (no LLM). “Why is the 5 minute load high” stays with the agent. `/load` stays the full Details panel. “Is the load high” stays the 1-minute Load chip.

## Overnight merge — v0.1.1167

- Load-high instant: “is the load high”, “how's the load”, and “how high is the load” use the Load chip (1-minute load, no LLM). “Why is the load high” stays with the agent. `/load` stays the full Details panel.

## Overnight merge — v0.1.1166

- Power-draw instant: “how much power is used”, “power draw”, and “is the power high” use the Power chip (CPU+GPU watts, no LLM). “Why is the power high” stays with the agent. `/power` stays the full strip. Low Power Mode stays on `/lpm`.

## Overnight merge — v0.1.1165

- Clock-speed instant: “how fast is the cpu”, “clock speed”, and “is the frequency high” use the Freq ring (GHz, no LLM). “Why is the frequency high” stays with the agent. “How much cpu” stays the CPU ring. “How fast is the gpu” stays with the agent.

## Overnight merge — v0.1.1164

- GPU-used instant: “how much gpu is used”, “is the gpu high”, and “is the gpu busy” use the GPU ring (%, no LLM). “Why is the gpu high” stays with the agent. “Is the gpu hot” stays the GPU ring. “Hot processes” stays the process list.

## Overnight merge — v0.1.1163

- CPU-used instant: “how much cpu is used”, “is the cpu high”, and “is the cpu busy” use the CPU ring (%, no LLM). “Why is the cpu high” stays with the agent. “Hot processes” stays the process list.

## Overnight merge — v0.1.1162

- Disk-used instant: “how much disk is used”, “how much storage”, and “is the disk full” use the SSD chip (%, no LLM). “Why is the disk full” stays with the agent. Disk Cleanup stays on `/disk`.

## Overnight merge — v0.1.1161

- RAM-used instant: “how much ram is used”, “how much memory”, and “is the ram high” use the RAM chip (%, no LLM). “Why is the ram high” stays with the agent. Notes path / size / age stay on those lanes.

## Overnight merge — v0.1.1160

- Battery-left instant: “how much battery is left”, “battery left”, and “is the battery low” use the Bat chip (%, charging, no LLM). “Why is the battery low” stays with the agent.

## Overnight merge — v0.1.1159

- How-hot instant: “how hot is the cpu”, “is the cpu hot”, and “how hot” use the Temp ring (°C, no LLM). “Is the gpu hot” uses the GPU ring. “Why is the cpu hot” stays with the agent. “Hot processes” stays the process list.

## Overnight merge — v0.1.1158

- Ollama URL instant: “what’s the ollama url”, “ollama endpoint”, and “where is ollama” use the Ollama Ready chip (host + model, no LLM). “Set the Ollama URL” stays a config change.

## Overnight merge — v0.1.1157

- Which-model instant: “which model are you” / `/model` uses the Ollama Ready chip (configured model name, no LLM). Plural “which models” stays a model list.

## Overnight merge — v0.1.1156

- CPU ring and sparkline Hot soft parity: metric card and history chart hot wash at 7% / 30% border — Monitors Slow. Fair stays quieter. Louder 12%/28% rest and 16%/44% pulse peak removed.

## Overnight merge — v0.1.1155

- Futuristic theme sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~12%/28% tint removed.

## Overnight merge — v0.1.1154

- Agent Ops active selection soft parity: selected tab, count pill, and overview card wash at 7% / 28% border — Ready calm. Louder 12%/40% tint removed.

## Overnight merge — v0.1.1153

- Architect theme sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~10%/22% tint removed.

## Overnight merge — v0.1.1152

- Neon theme sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~12%/28% tint removed.

## Overnight merge — v0.1.1151

- Dark theme sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~12%/28% tint removed.

## Overnight merge — v0.1.1150

- Data-poster sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~10%/22% tint removed.

## Overnight merge — v0.1.1149

- Material sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~10%/22% tint removed.

## Overnight merge — v0.1.1148

- Swiss-minimalistic sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~10%/22% tint removed.

## Overnight merge — v0.1.1147

- Light theme sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — apple Ready calm / Monitors Slow·Down. Louder ~10%/22% tint removed.

## Overnight merge — v0.1.1146

- Sidebar icon status soft parity: section icon good / warning / bad soft ok at 7% / 28% border and soft alert at 7% / 30% border — Ready calm / Monitors Slow·Down. Louder ~10%/22% tint removed.

## Overnight merge — v0.1.1145

- Ops health/overview ok·warn·bad soft parity: health cards + overview cards soft ok at 7% / 28% border and soft alert at 7% / 30% border — Ready calm / Runs Fail·Slow. Louder ~16%/55% tint removed.

## Overnight merge — v0.1.1144

- Changelog / Monitors empty-error soft parity: Changelog error pane + Monitors empty-error pane soft alert at 7% / 30% border — Monitors Down / Perplexity empty-error. Louder 8%/35% tint removed.

## Overnight merge — v0.1.1143

- Perplexity empty-error soft parity: empty-error pane soft alert at 7% / 30% border — Monitors Down / Perplexity has-error. Louder 8%/35% tint removed.

## Overnight merge — v0.1.1142

- AI Chat error-bubble soft parity: Assistant error message bubbles soft alert at 7% / 30% border — Errors glance / Monitors Down. Louder 8%/28% tint removed.

## Overnight merge — v0.1.1141

- Ops Runs Fail/Slow list-row soft parity: Fail and Slow run list rows soft alert at 7% / 30% border — Runs has-fail / has-slow. Louder 8%/22% tint removed.

## Overnight merge — v0.1.1140

- Disk Cleanup Big list-row soft parity: Big reclaim list rows soft alert at 7% / 30% border — Big attention / Monitors Slow. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1139

- Ollama collapsed has-errors + Settings Signal not-wired soft parity: collapsed has-errors + Signal not-wired soft alert at 7% / 30% border — Monitors Down / AI Chat Errors. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1138

- AI Chat Errors / last-answer has-errors soft parity: Errors glance + last-answer has-errors soft alert at 7% / 30% border — Monitors Down / Debug Log. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1137

- Debug Log has-errors soft parity: Error attention + collapsed error glances soft alert at 7% / 30% border — Monitors Down. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1136

- Ops MCP/Cursor/Perplexity/Mastodon/Telegram/Slack/Signal soft parity: Not-set / warn / bad attention glances soft alert at 7% / 30% border — Discord Offline / Reconnect. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1135

- Ops Redmine/Ollama/Brave/Browser soft parity: Not-set / warn / bad attention glances soft alert at 7% / 30% border — Discord Offline / Reconnect. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1134

- Ops Digest open soft parity: Digest open attention glance soft alert at 7% / 30% border — Monitors Slow / Discord Reconnect. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1133

- Ops Discord Offline/Reconnect soft parity: Offline / Reconnect attention glances soft alert at 7% / 30% border — Monitors Down / Slow. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1132

- Perplexity panel has-error soft parity: error attention glance + last-search has-error soft alert at 7% / 30% border — Monitors Down / Key-not-set. Louder 8%/34% (last-glance 8%/35%) tint removed.

## Overnight merge — v0.1.1131

- Perplexity panel key-not-set soft parity: Key-not-set attention glance soft alert at 7% / 30% border — Monitors Down / Settings credentials Not-set. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1130

- Settings credentials key-not-set soft parity: Discord / Perplexity / Brave / Redmine / Mastodon / MCP / Browser / Cursor / Telegram / Slack not-set (and partial) glances soft alert at 7% / 30% border — Monitors Down / Chat Offline Not-set. Louder 8%/34% tint removed.

## Overnight merge — v0.1.1129

- AI Chat offline/no-model soft parity: empty offline / no-model / circuit + model glance + Offline · No model · Errors · Not-set attention soft alert at 7% / 30% border — Monitors Slow / Down. Louder 8%/34% (circuit 10%/40%) tint removed.

## Overnight merge — v0.1.1128

- Settings product attention soft parity: AI Off / Compact On / Judge Off / Downloads Off / Ori Off / Having-fun Off / Voice STT Off soft alert at 7% / 30% border — Monitors Slow / Help open soft. Louder 10%/38% tint removed.

## Overnight merge — v0.1.1127

- Disk Cleanup last-run has-skip soft parity: Last run panel soft alert at 7% / 30% border — Monitors Slow / reclaim. Louder 10%/28% tint removed; clean last-run green stays soft.

## Overnight merge — v0.1.1126

- Disk Cleanup periodic-off soft parity: Runs when meta-card soft accent at 7% / 28% border — Ops Off / Files / Deliveries / Discord. Louder 8%/34% cyan tint removed; scopes-off already soft.

## Overnight merge — v0.1.1125

- Disk Cleanup reclaim soft parity: Reclaimable now + Next run due meta-cards soft alert at 7% / 30% border — Monitors Slow / Big attention. Louder 8%/34% tint removed; attention reclaim glances stay slightly softer.

## Overnight merge — v0.1.1124

- Ops Runs has-fail/has-slow soft parity: Fail · N / Slow · N attention glances soft alert at 7% / 30% border — Fail/Slow filter. Louder 8%/34% tint removed; still red/amber vs calm greens.

## Overnight merge — v0.1.1123

- Monitors summary/collapsed has-down soft parity: summary + collapsed Down glances soft alert at 7% / 30% border — Down attention / Down filter. Louder 8%/34% tint removed; still red vs all-up green.

## Overnight merge — v0.1.1122

- Monitors Down/Slow attention soft parity: Down · N / Slow · N glances soft alert at 7% / 30% border — Down/Slow filter. Louder 8%/34% tint removed; still red/amber vs calm greens.

## Overnight merge — v0.1.1121

- Disk Cleanup has-big attention soft parity: Disk · Big glance soft alert at 7% / 30% border — Monitors Slow / Big filter. Louder 8%/34% tint removed; reclaim stays slightly softer.

## Overnight merge — v0.1.1120

- Processes Hot attention soft parity: Hot · N hot glance soft alert at 7% / 30% border — Monitors Slow. Louder 8%/34% tint removed; Hot filter chip wash unchanged.

## Overnight merge — v0.1.1119

- Disk Cleanup Big filter attention soft parity: Disk · Big glance soft alert at 7% / 30% border — Monitors Slow. Louder 8%/34% tint removed; Reclaim stays slightly softer; Clean stays green.

## Overnight merge — v0.1.1118

- Ops Slow/Fail filter attention soft parity: Slow / Fail glances soft alert at 7% / 30% border — Monitors Slow / Down. Louder 8%/34% tint removed; still amber/red vs calm greens.

## Overnight merge — v0.1.1117

- Ops accent filter attention soft parity: Off / Files / Deliveries / Discord glances soft accent at 7% / 28% border — Ready / `.is-filter`. Louder 8%/34% tint removed; Fail / Slow stay stronger.

## Overnight merge — v0.1.1116

- Ops filter attention soft parity: On / Live / Jobs / Core / Instant / Lite / Direct glances soft green at 7% / 28% border — Ready / Monitors Up. Louder 8%/34% tint removed; Off / Files / Deliveries / Discord stay accent.

## Overnight merge — v0.1.1115

- Disk Cleanup due attention soft parity: Disk · Due glance soft green at 7% / 28% border — Ready calm / collapsed is-due. Louder 8%/34% tint removed; reclaim stays amber.

## Overnight merge — v0.1.1114

- Chat last-answer attention soft parity: Chat · Last answer glance soft green at 7% / 28% border — Ready calm / chat-answer glance. Louder 8%/32% tint removed; errors / sending stay stronger.

## Overnight merge — v0.1.1113

- Settings Help open calm soft parity: Help · Open glance soft green at 7% / 28% border — Ready calm / Monitors all-up. Louder 10%/38% tint removed; Help closed stays accent.

## Overnight merge — v0.1.1112

- Power-strip calm soft parity: battery healthy / LPM Off / Power low-draw / time-remaining (≥2h) soft green at 7% — Ready calm / Monitors all-up. Louder 10% tint removed.

## Overnight merge — v0.1.1111

- Accent filter-miss soft parity: Up / Pinned / Top / Snippet / Clean / On / You / Assistant / Ops calm empties soft accent at 7% — Ready calm / good-news soft parity. Louder 10% tint removed.

## Overnight merge — v0.1.1110

- Good-news filter-miss soft parity: Fail / Hot / Errors / Warn / Down / Slow / Reclaim / Big / Off-empty soft green at 7% — Ready calm / Monitors all-up parity. Stronger 10%/38% tint removed.

## Overnight merge — v0.1.1109

- AI Chat Ready calm soft parity: empty Ready pane + Chat · Ready attention glance soft green at 7% — model-online / turn / Monitors all-up parity. Stronger tint removed; offline / no-model / errors stay amber or red.

## Overnight merge — v0.1.1108

- AI Chat model online calm: connected model glance soft green at 7% — collapsed AI Chat / turn is-ok / Monitors all-up parity. Offline / no-model stays amber.

## Overnight merge — v0.1.1107

- Discord idle-thought 503 safe retry: treat Service Unavailable as safe to retry once (~1.5s backoff) so brief Discord outages do not drop Having-fun idle sends on the first failure.

## Overnight merge — v0.1.1106

- Time-remaining strip calm: soft green when estimate ≥2h — battery / LPM / Power parity. Under 1h amber; mid-range neutral.

## Overnight merge — v0.1.1105

- AI Chat turn glance calm: soft green wash when turns exist and nothing is sending — last-answer / Monitors all-up parity. Sending keeps accent.

## Overnight merge — v0.1.1104

- Disk Cleanup Last run calm: soft green wash when a last run exists without skips — Reclaimable is-clean / Details is-ok parity. Skips stay amber; not-yet-run stays neutral.

## Overnight merge — v0.1.1103

- AI Chat last-answer glance calm: soft green wash when a successful last answer is ready (Perplexity Ready / Debug Quiet / Monitors all-up parity). Failed turns stay red.

## Overnight merge — v0.1.1102

- Disk Cleanup Next run · Runs when calm: soft green wash when next run is ahead and when periodic is on — Reclaimable is-clean / Details is-ok parity. Due turns amber; periodic off stays cyan.

## Overnight merge — v0.1.1101

- Disk Cleanup Enabled scopes calm: soft green wash on the Enabled scopes meta-card when every scope is on — Reclaimable is-clean / Details is-ok parity. Scopes off stay amber.

## Overnight merge — v0.1.1100

- Disk Cleanup Reclaimable calm: soft green wash on the Reclaimable now meta-card when nothing is pending — collapsed is-clean / Details is-ok parity. Reclaimable stays amber.

## Overnight merge — v0.1.1099

- Power low-draw calm: soft green wash on the Power chip when combined CPU+GPU draw is below 20 W — battery healthy / LPM Off / Details is-ok parity. Elevated (≥20 W) stays amber.

## Overnight merge — v0.1.1098

- LPM Off calm: soft green wash on the LPM chip when Low Power Mode is Off — battery healthy / Details is-ok / ring calm parity. On keeps the stronger enabled green.

## Overnight merge — v0.1.1097

- Battery healthy calm: soft green wash on the battery chip when above 20% or charging — Details is-ok / Monitors all-up / ring calm parity. Low (≤20% discharging) stays amber.

## Overnight merge — v0.1.1096

- CPU · GPU · FREQ ring calm: soft green wash on rings + sparklines when below hot thresholds — Temp Nominal / Details is-ok parity.

## Overnight merge — v0.1.1095

- Temp ring Nominal calm: soft green wash on Temperature ring + TEMP sparkline when Heat is Nominal and Temp is below hot — Details is-ok / Monitors all-up parity.

## Overnight merge — v0.1.1094

- Perplexity collapsed glance calm: soft green wash when Ready · search — Monitors is-all-up / Disk is-clean / Debug Log quiet parity.

## Overnight merge — v0.1.1093

- Debug Log collapsed glance calm: soft green wash when Quiet · clean (no ERROR/WARN) — Monitors is-all-up / Disk is-clean / Details is-ok parity.

## Overnight merge — v0.1.1092

- Disk Cleanup collapsed glance calm: soft green wash when clean (nothing reclaimable, not due, scopes on) — Monitors is-all-up / Details is-ok parity.

## Overnight merge — v0.1.1091

- Top Processes glance calm: soft green wash on Top CPU · GPU · RAM when below hot thresholds (Details is-ok / Monitors all-up parity). Hot still amber.

## Overnight merge — v0.1.1090

- Details collapsed glance calm: soft green wash when Load and RAM are below hot thresholds (Monitors all-up / Disk clean parity). Hot still amber.

## Overnight merge — v0.1.1089

- Take note instant: `Take note:` / `note to self:` / `remember this:` / `make a note:` append curated memory on the instant lane (digester Slowest had a 23s BRAVE_SEARCH miss). Digester Slowest filter for historical take-note+Brave turns.

## Overnight merge — v0.1.1088

- Debug Log filter-miss calm: warm “Nothing here yet” title + solid soft-green wash (Error/Warn empty — no ERROR/WARN in the tail is good news). Perplexity / Disk / Monitors / Processes / AI Chat / Ops parity.

## Overnight merge — v0.1.1087

- Perplexity filter-miss calm: warm “Nothing here yet” title + solid accent wash (Top/Snippet). Disk Cleanup / Monitors / Top Processes / AI Chat / Agent Ops parity.

## Overnight merge — v0.1.1086

- Disk Cleanup filter-miss calm: warm “Nothing here yet” title + solid wash (Reclaim/Big and Off empty soft green; Clean/On accent). Monitors / Top Processes / AI Chat / Agent Ops parity.

## Overnight merge — v0.1.1085

- Monitors filter-miss calm: warm “Nothing here yet” title + solid wash (Down/Slow empty soft green; Up accent). Top Processes / AI Chat / Agent Ops parity.

## Overnight merge — v0.1.1084

- Top Processes filter-miss calm: warm “Nothing here yet” title + solid wash (Hot empty soft green; Pinned accent). AI Chat / Agent Ops filter-miss parity.

## Overnight merge — v0.1.1083

- Having-fun idle Ollama soft timeout: 120s wall budget, no outer Discord retry after chat timeout, rate-limited WARN (not ERROR), +15m next-idle backoff so the global Ollama queue recovers under load.

## Overnight merge — v0.1.1082

- Design review / Agent Ops Overview empty calm: warm “Nothing here yet” + solid accent wash on empty overview cards (filter-miss parity). Recaptured `feature-agent-ops.png` (window-only).

## Overnight merge — v0.1.1073

- Design review / Agent Ops: filter-miss calm (warm title + accent wash; Fail empty green; AI Chat Errors-empty parity). Recapture of `feature-agent-ops.png` deferred (Screen Recording TCC); polish grace marked.

## Overnight merge — v0.1.1072

- Instant `/keep-p25` nearest-rank p25 (floor quartile) gap between consecutive keep/discard rows (tonight + all-time; digester Slowest filters; p50).

## Done recently

- **v0.1.1111** — Accent filter-miss soft parity (Up/Pinned/Top/Snippet/Clean/On/You/Assistant/Ops calm empties soft accent at 7%).

- **v0.1.1109** — AI Chat Ready calm soft parity (empty Ready + Chat · Ready attention glance soft green at 7%; stronger tint removed).

- **v0.1.1104** — Disk Cleanup Last run calm (soft green when last run exists without skips; skips amber; not-yet-run neutral).

- **v0.1.1103** — AI Chat last-answer glance calm (soft green when successful reply ready; failed turns stay red).

- **v0.1.1102** — Disk Cleanup Next run · Runs when calm (soft green when not due / periodic on; due amber; periodic off cyan).

- **v0.1.1101** — Disk Cleanup Enabled scopes calm (soft green when every scope is on; scopes off stay amber).

- **v0.1.1100** — Disk Cleanup Reclaimable calm (soft green when nothing pending; reclaim stays amber).

- **v0.1.1099** — Power low-draw calm (soft green when combined CPU+GPU under 20 W; ≥20 W amber).

- **v0.1.1098** — LPM Off calm (soft green when Low Power Mode is Off; On keeps enabled green).

- **v0.1.1097** — Battery healthy calm (soft green when above 20% or charging; low stays amber).

- **v0.1.1096** — CPU · GPU · FREQ ring calm (soft green when below hot; Temp Nominal parity).

- **v0.1.1095** — Temp ring Nominal calm (soft green when Heat Nominal below hot).

- **v0.1.1094** — Perplexity collapsed glance calm (soft green when Ready · search).

- **v0.1.1093** — Debug Log collapsed glance calm (soft green when Quiet · clean; Monitors/Disk/Details parity).

- **v0.1.1092** — Disk Cleanup collapsed glance calm (soft green when clean; Monitors/Details parity).

- **v0.1.1091** — Top Processes Top CPU/GPU/RAM glance calm (soft green below hot; Details/Monitors parity).

- **v0.1.1090** — Details collapsed glance calm (soft green when Load/RAM fine; Monitors/Disk parity).

- **v0.1.1089** — Take note instant (`Take note:` / `note to self:` / `remember this:`) → curated MEMORY_APPEND; digester Slowest filter.

- **v0.1.1088** — Debug Log filter-miss calm (warm title + soft-green wash; Error/Warn empty).

- **v0.1.1087** — Perplexity filter-miss calm (warm title + solid accent wash; Top/Snippet).

- **v0.1.1086** — Disk Cleanup filter-miss calm (warm title + solid wash; Reclaim/Big/Off empty green; Clean/On accent).

- **v0.1.1085** — Monitors filter-miss calm (warm title + solid wash; Down/Slow empty green; Up accent).

- **v0.1.1084** — Top Processes filter-miss calm (warm title + solid wash; Hot empty green; Pinned accent).

- **v0.1.1083** — Having-fun idle Ollama soft timeout (120s wall · no outer Discord retry · rate-limited WARN · +15m idle backoff).

- **v0.1.1082** — Design review / Agent Ops Overview empty calm (warm title + accent wash; TCC grace).

- **v0.1.1073** — Design review / Agent Ops filter-miss calm: warm title (“Nothing here yet”) + solid accent wash on empty filter panes; Fail lane empty uses soft green wash (“No failed turns”) — AI Chat Errors-empty parity. TCC recapture of `feature-agent-ops.png` deferred; polish grace.

- **v0.1.1069** — Instant lane: `/keep-histogram` · `keep histogram` · `gap histogram` · `keep gap histogram` · `histogram keep gap` · `view keep histogram` / `open keep histogram` (+ discard / hist variants) — minute-bucket histogram (`<5m` · `5–10m` · `10–20m` · `20–30m` · `30–60m` · `≥60m`) of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters; Hermes-style ratchet glance; p50).

- **v0.1.1068** — Instant lane: `/keep-gini` · `keep gini` · `gini keep gap` · `gini coefficient keep gap` (+ discard) — Gini coefficient (0–1) of consecutive keep/discard gap lengths from `results.tsv` (tonight + all-time; digester Slowest filters; p50).

- **v0.1.1067** — Design review / CPU metrics: Temperature ring + TEMP sparkline Fair thermal wash when Apple thermal is Fair (below ≥70°C hot pulse); Serious/Critical thermal marks Temp hot under 70°C (power-strip Heat parity). Recapture of `feature-cpu-metrics.png` deferred (Screen Recording TCC); polish grace marked.

- **v0.1.1066** — Instant lane: `/keep-mode` · `keep mode` · `mode keep gap` · `modal keep gap` · `mode gap between keeps` · `view keep mode` / `open keep mode` (+ discard / modal variants) — modal (most frequent minute-bucketed) gap between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; mode only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/keep-entropy` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1065** — Instant lane: `/keep-entropy` · `keep entropy` · `entropy keep gap` · `shannon entropy keep gap` · `entropy gap between keeps` · `view keep entropy` / `open keep entropy` (+ discard / shannon variants) — Shannon entropy (bits) of minute-bucketed gaps between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; entropy only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/keep-kurtosis` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1064** — Instant lane: `/keep-kurtosis` · `keep kurtosis` · `kurtosis keep gap` · `excess kurtosis keep gap` · `kurtosis gap between keeps` · `view keep kurtosis` / `open keep kurtosis` (+ discard / kurt variants) — excess kurtosis (Fisher G2) of gaps between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; kurtosis only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/keep-skew` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1063** — Instant lane: `/keep-skew` · `keep skew` · `skew keep gap` · `skewness keep gap` · `skew gap between keeps` · `view keep skew` / `open keep skew` (+ discard / skewness variants) — skewness (Fisher–Pearson G1) of gaps between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; skew only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/keep-cv` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1062** — Instant lane: `/keep-cv` · `keep cv` · `cv keep gap` · `coefficient of variation keep gap` · `cv gap between keeps` · `view keep cv` / `open keep cv` (+ discard / CV variants) — CV (coefficient of variation = sample std/mean %) of gaps between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; CV only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/keep-mad` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1061** — Instant lane: `/keep-mad` · `keep mad` · `mad keep gap` · `median absolute deviation keep gap` · `mad gap between keeps` · `view keep mad` / `open keep mad` (+ discard / MAD variants) — MAD (median absolute deviation) of gaps between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; MAD only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/keep-std` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1060** — Instant lane: `/keep-std` · `keep std` · `std keep gap` · `standard deviation keep gap` · `std gap between keeps` · `view keep std` / `open keep std` (+ discard / stddev variants) — sample std gap between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; std only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/keep-iqr` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1059** — Instant lane: `/keep-iqr` · `keep iqr` · `iqr keep gap` · `interquartile keep gap` · `iqr gap between keeps` · `view keep iqr` / `open keep iqr` (+ discard / interquartile variants) — IQR (Q3−Q1) gap between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; IQR only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/keep-p90` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1058** — Instant lane: `/keep-p90` · `keep p90` · `p90 keep gap` · `90th percentile keep gap` · `p90 gap between keeps` · `view keep p90` / `open keep p90` (+ discard / percentile variants) — p90 gap between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; p90 only — no full dump; not `/keep-pace` / `/keep-median` / `/keep-range` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1057** — Instant lane: `/keep-range` · `keep range` · `keep gap range` · `min max keep gap` · `shortest and longest keep gap` · `view keep range` / `open keep range` (+ discard / spread variants) — min–max gap between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; range only — no full dump; not `/keep-pace` / `/keep-median` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1056** — Instant lane: `/keep-median` · `keep median` · `median keep gap` · `median gap between keeps` · `median time between keeps` · `view keep median` / `open keep median` (+ discard / median-gap variants) — median gap between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; median only — no full dump; not `/keep-pace` / `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1055** — Instant lane: `/keep-pace` · `keep pace` · `keep gap` · `time between keeps` · `average keep gap` · `average time between keeps` · `view keep pace` / `open keep pace` (+ discard / gap variants) — average gap between consecutive keep/discard rows from `results.tsv` (tonight since 20:00 + all-time; pace only — no full dump; not `/first-keep` / `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1054** — Instant lane: `/first-keep` · `first keep` · `first keep tonight` · `earliest keep` · `what was the first keep` · `when was the first keep` · `view first keep` / `open first keep` (+ discard / opening variants) — earliest keep/discard row tonight since 20:00 from `results.tsv` (one description only — no full dump; not `/last-keep` / `/since-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1053** — Instant lane: `/since-keep` · `since last keep` · `time since last keep` · `how long since last keep` · `how long ago was the last keep` · `view since keep` / `open since keep` (+ discard variants) — age since newest keep/discard row from `results.tsv` (age only — no description dump; not `/last-keep` / counts / rate / streak / recent / path·size·age / morning surprise; digester Slowest filters for ratchet glances; Hermes-style ratchet glance; p50).

- **v0.1.1052** — Instant lane: `/longest-streak` · `longest streak` · `best streak` · `record streak` · `view longest streak` / `open longest streak` (+ max-streak / longest-discard variants) — longest keep/discard streak from `results.tsv` (all-time + tonight since 20:00 only — no full dump; not current streak / counts / rate / last-row / recent list / path·size·age / morning surprise; Hermes-style ratchet glance; p50).

- **v0.1.1051** — Instant lane: `/keep-streak` · `keep streak` · `current streak` · `ratchet streak` · `view keep streak` / `open keep streak` (+ discard-streak / `/streak` variants) — consecutive keep/discard streak from `results.tsv` (current + tonight since 20:00 only — no full dump; not counts / rate / last-row / recent list / path·size·age / morning surprise; Hermes-style ratchet glance; p50).

- **v0.1.1050** — Instant lane: `/keep-rate` · `keep rate` · `hit rate` · `ratchet hit rate` · `keep percentage` · `view keep rate` / `open keep rate` (+ discard-rate variants) — keep/discard hit rate from `results.tsv` (tonight + all-time percentages only — no full dump; not counts / last-row / recent list / path·size·age / morning surprise; Hermes-style ratchet glance; p50).

- **v0.1.1049** — Instant lane: `/recent-keeps` · `recent keeps` · `list recent keeps` · `tonight keep list` · `view recent keeps` / `open recent keeps` (+ discard variants) — short tonight keep/discard list from `results.tsv` (newest first; capped at 5 — no full dump; not counts / last-row / path·size·age / morning surprise; Hermes-style ratchet glance; p50).

- **v0.1.1048** — Instant lane: `/last-keep` · `last keep` · `latest keep` · `what was the last keep` · `view last keep` / `open last keep` (+ discard variants) — newest keep or discard row from `results.tsv` (one description only — no full dump; not counts / path·size·age / morning surprise; Hermes-style ratchet glance; p50).

- **v0.1.1047** — Instant lane: `/keeps` · `keeps tonight` · `keep count` · `how many keeps` · `discard count` · `discards tonight` · `ratchet summary` · view/see/show me/open/list-the keeps (and discard variants) — keep/discard counts from `results.tsv` (tonight since 20:00 + all-time; counts only — not path·size·age / morning surprise; Hermes-style ratchet glance; p50).

- **v0.1.1046** — Instant lane: more Digest open NL (`view digest` / `see digest` / `show me the digest` / `open digest` / `list the digest` + open candidates variants) join `digest open` cached snapshot; refresh phrases (`/digest` · `rescan digest` · `show me digest`) still re-run digester (exact open only — not path·size·age; p50).

- **v0.1.1045** — Instant lane: more Perplexity last-search NL (`view perplexity` / `see perplexity` / `show me the perplexity` / `open perplexity` / `list the perplexity` + last search / top results / snippet results) join `/perplexity` · `/perplexity top` · `/perplexity snippet` (exact open only — not key / live search / path·size·age; p50).

- **v0.1.1044** — Instant lane: more Runs-lane NL (`view failed` / `see failed` / `show me the failed` / `open failed` / `list the failed` + slow / instant / lite / direct) join `/failed` · `/slow` · `/instant` · `/lite` · `/direct` (+ optional day window; exact open only — not why-did / why-is / make-it / monitor / ticket / path/size/age; p50).

- **v0.1.1043** — Instant lane: more Top Processes Hot/Pinned NL (`view hot` / `see hot` / `show me the hot` / `open hot` / `list the hot` + pinned / pinned processes) join `/hot` · `/pinned` Hot or Pinned lists (exact open only — not rings/strip/details Hot, not pinned path/size/age; p50).

- **v0.1.1042** — Instant lane: more power-strip chip NL (`view battery` / `see battery` / `show me the battery` / `open battery` / `list the battery` + heat / thermal / lpm / low power mode / ram / memory / ssd / uptime) join `/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/uptime` one-chip replies (exact open only — not `/strip`, Disk Cleanup `open disk`, `/details`, path/size/age; p50).

- **v0.1.1041** — Instant lane: more ring-chip NL (`view cpu` / `see cpu` / `show me the cpu` / `open cpu` / `list the cpu` + gpu / freq / frequency / temp / temperature) join `/cpu` · `/gpu` · `/freq` · `/temp` one-chip replies (exact open only — not `/rings`, `/details`, `cpu window`, path/size/age; p50).

- **v0.1.1040** — Instant lane: more metrics NL (`view rings` / `see rings` / `show me the rings` / `open rings` / `list the rings` + Hot; `view strip` / `open power` / same for strip; `view details` / `open load` / same for details) join `/rings` · `/strip` · `/details` (exact open only — not process details, disk cleanup, path/size/age; p50).

- **v0.1.1039** — Instant lane: more operator NL (`view insights` / `see insights` / `show me the insights` / `open insights` / `list the insights` + day window; `view help` / `see help` / `show me the help` / `open help` / `list the help` + ops / commands) join `/insights` · `/help` · `/ops` (exact open only — not insights on …, help me / help with …, path/size/age; p50).

- **v0.1.1038** — Instant lane: more health NL (`view status` / `see status` / `show me the status` / `open status` / `list the status` + health / version) join `/status` · `/health` · `/version` one-screen health (exact open only — not ticket status-of, changelog/ship/bump/release, path/size/age; p50).

- **v0.1.1037** — Instant lane: more gateway/config NL (`view discord` / `open discord` / `list the discord` + ollama/llm; `view perplexity key` / `open perplexity key` (key only — not bare perplexity last-search); `view cursor` / `open cursor-agent`) join Ready chips (exact open only; config/gateway only; p50).

- **v0.1.1036** — Instant lane: more integration NL (`view redmine`, `see redmine`, `show me the redmine`, `open redmine` / `open the redmine`, `list the redmine`, and the same for brave / brave search / mastodon / mcp / mcp server) join `/redmine` · `/brave` · `/mastodon` · `/mcp` Ready / Not set / Partial (exact open only — not tickets, web search, toot/post, or MCP tool calls; config only; p50).

- **v0.1.1035** — Instant lane: more alert-channel NL (`view telegram`, `see telegram`, `show me the telegram`, `open telegram` / `open the telegram`, `list the telegram`, and the same for slack / signal / alerts + telegram bot / slack webhook / signal app / alert channels) join `/telegram` · `/slack` · `/signal` · `/alerts` Ready / Not set / Partial (exact open only — not send/post/notify/trigger/create; Keychain + registry; p50).

- **v0.1.1034** — Instant lane: more Discord voice STT NL (`view voice`, `see voice`, `show me the voice`, `open voice` / `open the voice`, `list the voice`, and the same for stt / speech / speech to text / voice stt / discord voice / discord stt + close variants) join `/voice` · `/stt` Ready / Off / Partial / Not set (exact open only — not transcribe / voice notes / send voice / enable; config only; p50). Also fixes `/having_fun` · `/fun` · `/idle` `show me …` after normalizer strips `show me` / `show`.

- **v0.1.1033** — Instant lane: more Having fun / idle NL (`view having fun`, `see having fun`, `show me the having fun`, `open having fun` / `open the having fun`, `list the having fun`, and the same for fun / idle / idle thoughts + close variants) join `/having_fun` · `/fun` · `/idle` On/Off Ready (exact open only — not send/post, enable, or free-form “have fun …”; config only; p50).

- **v0.1.1032** — Instant lane: more Ori Mnemos NL (`view ori`, `see ori`, `show me the ori`, `open ori` / `open the ori`, `list the ori`, and the same for mnemos / ori mnemos / ori-mnemos + close variants) join `/ori` · `/mnemos` Ready / Off / Partial (exact open only — not MCP `ori_*` / MEMORY_APPEND / scrub / enable / vault path·size·age; config only; p50).

- **v0.1.1031** — Instant lane: more Downloads organizer NL (`view downloads`, `see downloads`, `show me the downloads`, `open downloads` / `open the downloads`, `list the downloads`, and the same for organizer / downloads organizer + close variants) join `/downloads` · `/organizer` On/Off Ready (exact open only — not run-now / enable / `/disk` / BROWSER_DOWNLOAD; path/size/age / rules/state safe; p50).

- **v0.1.1030** — Instant lane: more Compact NL (`view compact`, `see compact`, `show me the compact`, `open compact` / `open the compact`, `list the compact`, and the same for menu bar / menu-bar / cpu window / cpu-window + close variants) join `/compact` · `/menu-bar` · `/cpu-window` Menu bar / CPU window On/Off (exact open only — not compaction / enable / run compaction; config only; p50).

- **v0.1.1029** — Instant lane: more Judge / AI NL (`view judge`, `see judge`, `show me the judge`, `open judge` / `open the judge`, `list the judge`, and `view ai`, `see ai`, `show me the ai`, `open ai` / `open the ai`, `open ai agent`, `list the ai` + close variants) join `/judge` · `/ai` · `/ai-agent` Ready / Off / On (exact open only — not run/score/enable judge, OpenAI, ask/chat, `/agents`; config only; p50).

- **v0.1.1028** — Instant lane: more Browser / CDP NL (`view browser`, `see browser`, `show me the browser`, `open browser` / `open the browser`, `list the browser`, and the same for `cdp` + close variants) join `/browser` · `/cdp` Ready / Off / Not set (exact open only — not `open page` / `open url`; path/size/age / credentials / downloads / cookies safe; p50).

- **v0.1.1027** — Instant lane: more Plugins NL (`view plugins`, `see plugins`, `show me the plugins`, `open plugins` / `open the plugins`, `list the plugins` + close variants) join `list plugins` / On · Off (exact `open plugins` only — not `open plugin …`; path/size/age safe; p50).

- **v0.1.1026** — Instant lane: more Tasks NL (`view tasks`, `see tasks`, `show me the tasks`, `open tasks` / `open the tasks`, `list the tasks` + close variants) join `list tasks` / Active · All (exact `open tasks` only — not `open task …`; path/size/age safe; p50).

- **v0.1.1025** — Instant lane: more Skills NL (`view skills`, `see skills`, `show me the skills`, `open skills`, `list the skills` + close variants) join `list skills` / catalog (exact `open skills` only; path/size/age safe; p50).

- **v0.1.1024** — Instant lane: more Agents NL (`view agents`, `see agents`, `show me the agents`, `open agents`, `list the agents` + close variants) join `list agents` / On · Off (exact `open agents` only; path/size/age safe; p50).

- **v0.1.1023** — Instant lane: more Knowledge NL (`view knowledge`, `see knowledge`, `show me the knowledge`, `open knowledge`, `list the knowledge` + close variants) join `list knowledge` / Discord · Core (p50).

- **v0.1.1022** — Instant lane: more Sessions NL (`view sessions`, `see sessions`, `show me the sessions`, `open sessions`, `list the sessions` + close variants) join `list sessions` / Live · Files (exact `open sessions` only; p50).

- **v0.1.1021** — Instant lane: more Schedules NL (`view schedules`, `see schedules`, `show me the schedules`, `open schedules`, `list the schedules` + close variants) join `list schedules` / Jobs · Deliveries (p50).

- **v0.1.1020** — Instant lane: more Disk Cleanup NL (`view disk`, `see disk`, `show me the disk cleanup`, `open disk`, `list the disk cleanup` + close variants) join `disk cleanup` / On · Off · Reclaim · Big · Clean (p50).

- **v0.1.1019** — Instant lane: more External / Monitors NL (`view monitors`, `see monitors`, `show me the monitors`, `open monitors`, `list the monitors` + close variants) join `list monitors` / Up · Down · Slow (p50).

- **v0.1.1018** — Instant lane: more Top Processes NL (`view processes`, `see processes`, `show me the processes`, `open processes`, `list the processes` + close variants) join `top processes` / Hot · Pinned (p50).

- **v0.1.1017** — Instant lane: more Debug Log NL (`view logs`, `see logs`, `show me the logs`, `open logs`, `list the logs`) + digester Slowest filters for historical Review logs and Instant wake-ups (p50).

- **v0.1.1016** — Agent Ops Schedules filter Clear chip beside All · Jobs · Deliveries when a kind filter is active (Knowledge / Runs / Sessions Clear parity; Cleared flash; filter-miss Clear when Jobs/Deliveries was on; design review / Schedules).

- **v0.1.1015** — Agent Ops Knowledge filter Clear chip beside All · Discord · Core when a kind filter is active (Runs / Sessions / Agents Clear parity; Cleared flash; filter-miss Clear when Discord/Core was on; design review / Knowledge).

- **v0.1.1014** — Agent Ops Runs filter Clear chip beside All · Instant · Lite · Direct · Slow · Fail when a lane filter is active (Agents / Sessions / Processes Clear parity; Cleared flash; filter-miss Clear when a lane was on; design review / Runs).

- **v0.1.1013** — Agent Ops Agents filter Clear chip beside All · On · Off when a filter is active (Sessions / Processes / Disk Clear parity; Cleared flash; filter-miss Clear when On/Off was on; design review / Agents).

- **v0.1.1012** — Agent Ops Sessions filter Clear chip beside All · Live · Files when a kind filter is active (Processes / Disk / Debug Log / Perplexity Clear parity; Cleared flash; filter-miss Clear filter CTA uses same flash when Live/Files was on; design review / Agent Ops Sessions).

- **v0.1.1011** — Disk Cleanup scopes filter Clear chip beside All · On · Off when a filter is active (category Clear / Monitors / Processes / AI Chat Clear parity; Cleared flash; filter-miss Clear filter CTA; design review / Disk Cleanup scopes).

- **v0.1.1010** — Perplexity Search filter Clear chip beside All · Top · Snippet when a filter is active (Debug Log / Disk Cleanup / Monitors / Top Processes / AI Chat Clear parity; Cleared flash; filter-miss Clear filter CTA; design review / Perplexity).

- **v0.1.1009** — Debug Log filter Clear chip beside All · Error · Warn when a filter is active (Disk Cleanup / Monitors / Top Processes / AI Chat Clear parity; Cleared flash; filter-miss Clear filter CTA; design review / Debug Log).

- **v0.1.1007** — Instant lane: launchd stdout age (`launchd stdout age`, `how old is launchd.stdout.log`, `launchd.stdout.log age`, `mac-stats launchd stdout age`, `when was launchd stdout updated`; mtime only; no dump/tail; does not steal path / size / harness loop stdout / launchd stderr / LaunchAgent plist / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.1006** — Instant lane: launchd stdout size (`launchd stdout size`, `how big is launchd.stdout.log`, `launchd.stdout.log size`, `mac-stats launchd stdout size`; stat only; no dump/tail; does not steal path / age / harness loop stdout / launchd stderr / LaunchAgent plist / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.1005** — Instant lane: launchd stdout path (`launchd stdout path`, `where is launchd.stdout.log`, `launchd.stdout.log path`, `mac-stats launchd stdout path`; path only; no dump/tail; does not steal harness loop stdout / launchd stderr / LaunchAgent plist / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.1004** — Instant lane: launchd stderr age (`launchd stderr age`, `how old is launchd.stderr.log`, `launchd.stderr.log age`, `mac-stats launchd stderr age`, `when was launchd stderr updated`; mtime only; no dump/tail; does not steal path / size / harness loop stderr / LaunchAgent plist / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.1003** — Instant lane: launchd stderr size (`launchd stderr size`, `how big is launchd.stderr.log`, `launchd.stderr.log size`, `mac-stats launchd stderr size`; stat only; no dump/tail; does not steal path / age / harness loop stderr / LaunchAgent plist / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.1002** — Instant lane: launchd stderr path (`launchd stderr path`, `where is launchd.stderr.log`, `launchd.stderr.log path`, `mac-stats launchd stderr path`; path only; no dump/tail; does not steal harness loop stderr / LaunchAgent plist / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.1001** — Instant lane: harness loop stderr age (`harness loop stderr age`, `how old is overnight_harness_loop.stderr.log`, `overnight_harness_loop.stderr.log age`, `harness stderr age`, `when was harness loop stderr updated`; mtime only; no dump/tail; does not steal path / size / overnight_agent.log / stdout / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.1000** — Instant lane: harness loop stderr size (`harness loop stderr size`, `how big is overnight_harness_loop.stderr.log`, `overnight_harness_loop.stderr.log size`, `harness stderr size`; stat only; no dump/tail; does not steal path / age / overnight_agent.log / stdout / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.999** — Instant lane: harness loop stderr path (`harness loop stderr path`, `where is overnight_harness_loop.stderr.log`, `overnight_harness_loop.stderr.log path`, `harness stderr path`; path only; no dump/tail; does not steal overnight_agent.log / stdout / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.998** — Disk Cleanup filter Clear chip beside All · Reclaim · Big · Clean when a filter is active (Monitors / Top Processes / AI Chat / Ops Clear parity; Cleared flash; design review / `feature-disk-cleanup`).

- **v0.1.997** — External / Monitors filter Clear chip beside All · Up · Down · Slow when a filter is active (Top Processes / AI Chat / Ops Clear parity; Cleared flash; design review / `feature-monitors`).



- **v0.1.996** — Top Processes filter Clear chip beside All · Pinned · Hot when a filter is active (AI Chat / Ops Clear parity; Cleared flash; design review / `feature-processes`).

- **v0.1.995** — Instant lane: harness loop stdout age (`harness loop stdout age`, `how old is overnight_harness_loop.stdout.log`, `overnight_harness_loop.stdout.log age`, `harness stdout age`, `when was harness loop stdout updated`; mtime only; no dump/tail; does not steal path / size / overnight_agent.log / stderr / `debug.log age` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.994** — Instant lane: harness loop stdout size (`harness loop stdout size`, `how big is overnight_harness_loop.stdout.log`, `overnight_harness_loop.stdout.log size`, `harness stdout size`; stat only; no dump/tail; does not steal path / age / overnight_agent.log / stderr / `debug.log size` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.993** — Instant lane: harness loop stdout path (`harness loop stdout path`, `where is overnight_harness_loop.stdout.log`, `overnight_harness_loop.stdout.log path`, `harness stdout path`; path only; no dump/tail; does not steal overnight_agent.log / stderr / `debug.log path` / morning surprise / improvements / loop backlog / sibling / standing; p50).

- **v0.1.992** — Instant lane: overnight agent log age (`overnight agent log age`, `how old is overnight_agent.log`, `overnight_agent.log age`, `harness agent log age`, `when was overnight agent log updated`; mtime only; no dump/tail; does not steal path / size / `debug.log age` / morning surprise / improvements / loop / sibling / standing; p50).

- **v0.1.991** — Instant lane: overnight agent log size (`overnight agent log size`, `how big is overnight_agent.log`, `overnight_agent.log size`, `harness agent log size`; stat only; no dump/tail; does not steal path / age / `debug.log size` / morning surprise / improvements / loop / sibling / standing; p50).

- **v0.1.990** — Instant lane: overnight agent log path (`overnight agent log path`, `where is overnight_agent.log`, `overnight_agent.log path`, `harness agent log path`; path only; no dump/tail; does not steal `debug.log path` / morning surprise / improvements / loop / sibling / standing; p50).

- **v0.1.989** — LaunchAgent WorkingDirectory: `install-to-applications.sh` sets repo (else `$HOME`) on `com.raro42.mac-stats.plist` so launchd cwd is not `/` (defense in depth with RUN_CMD v0.1.988; `agents.md` example updated).

- **v0.1.988** — RUN_CMD working directory: children use Cursor Agent / config workspace (`~/projects/mac-stats`) so LaunchAgent cwd `/` no longer breaks relative `python3 scripts/…` (quality-weekly-review).

- **v0.1.987** — Instant lane: morning surprise age (`morning surprise age`, `how old is morning_surprise.md`, `overnight morning surprise age`, `when was morning surprise updated`, `today's morning surprise age`; today's note mtime; no dump; does not steal path / size / improvements / standing / sibling / loop; p50).

- **v0.1.986** — Instant lane: morning surprise size (`morning surprise size`, `how big is morning_surprise.md`, `overnight morning surprise size`, `today's morning surprise size`; today's note size on disk; no dump; does not steal path / age / improvements / standing / sibling / loop; p50).

- **v0.1.985** — Instant lane: morning surprise path (`morning surprise path`, `where is morning_surprise.md`, `where is the morning surprise`, `overnight morning surprise path`; today's dated file path only; no dump; does not steal improvements / standing / sibling / loop; content asks still use *morning surprise?*; p50).

- **v0.1.984** — Instant lane: standing_backlog.md age (`standing backlog age`, `how old is standing_backlog.md`, `overnight standing backlog age`, `when was standing backlog updated`, `track b backlog age`; mtime only; no dump; does not steal path / size / improvements / loop backlog / sibling harness; p50).

- **v0.1.983** — Instant lane: standing_backlog.md size (`standing backlog size`, `how big is standing_backlog.md`, `overnight standing backlog size`, `track b backlog size`; stat only; no dump; does not steal path / age / improvements / loop backlog / sibling harness; p50).

- **v0.1.982** — Instant lane: standing_backlog.md path (`standing backlog path`, `where is standing_backlog.md`, `overnight standing backlog path`, `track b backlog path`; path only; no dump; does not steal improvements / loop backlog / sibling harness; p50).

- **v0.1.981** — Instant lane: sibling_harness.md age (`sibling harness age`, `how old is sibling_harness.md`, `openclaw hermes scan age`, `when was sibling harness updated`, `overnight sibling harness age`; mtime only; no dump; does not steal path / size / improvements / loop backlog / standing backlog; p50).

- **v0.1.980** — Instant lane: sibling_harness.md size (`sibling harness size`, `how big is sibling_harness.md`, `openclaw hermes scan size`, `overnight sibling harness size`; stat only; no dump; does not steal path / age / improvements / loop backlog / standing backlog; p50).

- **v0.1.979** — Instant lane: sibling_harness.md path (`sibling harness path`, `where is sibling_harness.md`, `openclaw hermes scan path`, `overnight sibling harness path`; path only; no dump; does not steal improvements/loop backlog/standing backlog; p50).

- **v0.1.978** — Instant lane: loop_backlog.md age (`loop backlog age`, `how old is loop_backlog.md`, `harness tick log age`, `when was loop backlog updated`; mtime; no dump; path/size/improvements/results.tsv safe; p50).

- **v0.1.977** — Instant lane: loop_backlog.md size (`loop backlog size`, `how big is loop_backlog.md`, `harness tick log size`, `overnight loop backlog size`; file size on disk; no dump; does not steal path / age / `improvements size` / `results.tsv size`; p50 latency).

- **v0.1.976** — Instant lane: loop_backlog.md path (`loop backlog path`, `where is loop_backlog.md`, `harness tick log path`, `overnight loop backlog path`; path only; no dump; does not steal `improvements path` / `results.tsv path` / standing backlog / sibling harness; p50 latency).

- **v0.1.975** — Instant lane: digest.md / latest.md age (`digest.md age`, `latest.md age`, `how old is digest.md`, `when was latest.md updated`, `digest markdown age`; `latest.md` mtime only; no digester spawn; does not steal cache `digest age` / size / open / `/digest`; p50 latency).

- **v0.1.974** — Instant lane: session_reset_phrases.md age (`session reset age`, `session_reset_phrases.md age`, `how old is session reset phrases`, `when was session reset phrases updated`, `session reset phrases age`; mtime only; no dump; does not steal path / size / escalation / cookie reject; p50 latency).

- **v0.1.973** — Instant lane: escalation_patterns.md age (mtime; path/size/session-reset/cookie reject safe; p50).

- **v0.1.972** — Instant lane: browser_storage_state.json age (mtime; path/size/cookie reject/credentials safe; p50).

- **v0.1.971** — Instant lane: browser-credentials.toml age (mtime; path/size/storage state/credential accounts safe; p50).

- **v0.1.970** — Instant lane: downloads-organizer-state.json age (mtime; path/size/rules/downloads safe; p50).

- **v0.1.969** — Instant lane: downloads-organizer-rules.md age (`organizer rules age`, `downloads organizer rules age`, `how old is downloads organizer rules`, `when was downloads organizer rules updated`, `downloads-organizer-rules.md age`; mtime only; no dump; does not steal path / size / organizer state / `/downloads`; p50 latency).

- **v0.1.968** — Instant lane: cookie_reject_patterns.md age (`cookie reject age`, `cookie reject patterns age`, `how old is cookie reject patterns`, `when was cookie reject patterns updated`, `cookie_reject_patterns.md age`; mtime only; no dump; does not steal path / size / session-reset / escalation / browser cookies; p50 latency).

- **v0.1.967** — Instant lane: cleanup-quarantine directory age (`cleanup quarantine age`, `how old is quarantine`, `quarantine folder age`, `when was cleanup quarantine updated`, `disk quarantine age`; newest cleanup-quarantine/ mtime; no list; does not steal path / size / `/disk`; p50 latency).

- **v0.1.966** — Instant lane: browser-downloads directory age (`browser downloads age`, `how old are browser downloads`, `browser-downloads age`, `when was browser downloads updated`, `cdp downloads age`; newest browser-downloads/ mtime; no list; does not steal path / size / `/downloads`; p50 latency).

- **v0.1.965** — Instant lane: PDF exports directory age (`pdfs age`, `how old are pdfs`, `pdfs folder age`, `when was pdfs updated`, `pdf exports age`; newest pdfs/ mtime; no list; does not steal path / size / save; p50 latency).

- **v0.1.964** — Instant lane: CDP traces directory age (`traces age`, `how old are traces`, `traces folder age`, `when was traces updated`, `cdp traces age`; newest traces/ mtime; no list; does not steal path / size / prune; p50 latency).

- **v0.1.963** — Instant lane: uploads directory age (`uploads age`, `how old are uploads`, `uploads folder age`, `when was uploads updated`; newest uploads/ mtime; no list; does not steal path / size / upload; p50 latency).

- **v0.1.962** — Instant lane: tmp directory age (`tmp age`, `how old is tmp`, `tmp folder age`, `when was tmp updated`; newest tmp/ mtime; no list; does not steal path / size / prune / temperature; p50 latency).

- **v0.1.961** — Instant lane: task directory age (`task age`, `how old are tasks`, `task folder age`, `when was tasks updated`; newest task/ mtime; no list; does not steal path / size / `/tasks`; p50 latency).

- **v0.1.960** — AI Chat filter Clear chip beside All·You·Assistant·Errors when a role filter is active (Ops Clear parity; Cleared flash; design review / `feature-ai-chat`).

- **v0.1.959** — Instant lane: session directory age (`session age`, `how old are sessions`, `session folder age`, `when was sessions updated`; newest session/ mtime; no list; does not steal path / size / `session memory age` / `/sessions`; p50 latency).

- **v0.1.958** — Instant lane: prompts directory age (`prompts age`, `how old are prompts`, `prompts folder age`, `when was prompts updated`; newest agents/prompts/ mtime; no list; does not steal path / size / planning·execution ages; p50 latency).

- **v0.1.957** — Instant lane: plugins/scripts directory age (`plugins age`, `scripts age`, `how old are plugins`, `plugins folder age`, `when was plugins updated`; newest scripts/ mtime; no list; does not steal path / size / `/plugins`; p50 latency).

- **v0.1.956** — Instant lane: agents directory age (`agents age`, `how old are agents`, `agents folder age`, `when was agents updated`; newest agents/ mtime; no list; does not steal path / size / `agent.json age` / `/agents`; p50 latency).

- **v0.1.955** — Instant lane: skills directory age (`skills age`, `how old are skills`, `skills folder age`, `when was skills updated`; newest Hermes skills/ mtime; no list; does not steal path / size / `skill.md age` / `/skills`; p50 latency).

- **v0.1.954** — Instant lane: LaunchAgent plist age (`launchagent age`, `how old is the launchagent`, `mac-stats.plist age`, `harness plist age`; app KeepAlive + overnight harness mtimes; no dump; does not steal path / size / load/unload; p50 latency).

- **v0.1.953** — Instant lane: screenshots directory age (`screenshots age`, `how old are screenshots`, `screenshots folder age`, `when was screenshots updated`; newest file mtime; no list dump; does not steal path / size / take/list; p50 latency).

- **v0.1.952** — Instant lane: improvements directory age (`improvements age`, `how old is the improvements folder`, `improvements dir age`, `when was improvements updated`, `autoresearch folder age`; newest file mtime; no list dump; does not steal path / size / `results.tsv age` / overnight content; p50 latency).

- **v0.1.951** — Instant lane: Ori vault age (`ori vault age`, `mnemos vault age`, `how old is ori vault`, `when was ori vault updated`, `ori_vault age`; newest file mtime under vault; no list/MCP; does not steal path / size / `/ori` Ready; p50 latency).

- **v0.1.950** — Instant lane: before-compaction transcript age (`before compaction transcript age`, `before-compaction transcript age`, `how old is before compaction transcript`, `when was before compaction transcript updated`, `last_session_before_compaction.jsonl age`; mtime only; no dump/hook; does not steal path / size / before-reset / session reset phrases; p50 latency).

- **v0.1.949** — Instant lane: before-reset transcript age (`before reset transcript age`, `before-reset transcript age`, `how old is before reset transcript`, `when was before reset transcript updated`, `last_session_before_reset.jsonl age`; mtime only; no dump/hook; does not steal path / size / before-compaction / session reset phrases; p50 latency).

- **v0.1.948** — Instant lane: Discord channel memory age (`discord memory age`, `memory-discord age`, `channel memory age`, `how old is discord memory`, `when was discord memory updated`; newest `memory-discord-*.md` mtime; no dump; does not steal path / size / `/knowledge discord`; p50 latency).

- **v0.1.947** — Instant lane: session-memory age (`session memory age`, `session-memory age`, `how old is session memory`, `when was session memory updated`; newest `session-memory-*.md` mtime; no dump; does not steal path / size / session folder / `/sessions`; p50 latency).

- **v0.1.946** — Instant lane: notes folder age (`notes age`, `how old are notes`, `memory folder age`, `notes folder age`; newest file mtime; no dump; does not steal path / size / memory.md age / bare `memory age`; p50 latency). Session-memory size plural/`session-memory.md` detector fix.

- **v0.1.945** — Instant lane: memory.md age (`memory.md age`, `curated memory age`, `how old is memory.md`, `when was memory.md updated`; mtime only; no dump; does not steal path / size / notes folder / bare `memory age`; p50 latency).

- **v0.1.944** — Instant lane: execution_prompt.md age (`execution age`, `execution_prompt.md age`, `how old is execution`, `when was execution updated`; mtime only; no dump; does not steal path / size / prompts folder / planning; p50 latency).

- **v0.1.943** — Instant lane: planning_prompt.md age (`planning age`, `planning_prompt.md age`, `how old is planning`, `when was planning updated`; mtime only; no dump; does not steal path / size / prompts folder / execution; p50 latency).

- **v0.1.942** — AI Chat filter-miss calm (You/Assistant/Errors empty → titled warm hint + cue wash; Errors empty uses ok green; design review / `feature-ai-chat`).

- **v0.1.941** — Instant lane: agent.json age (`agent.json age`, `agent config age`, `how old is agent.json`, `when was agent.json updated`; newest mtime across per-agent files; no dump; does not steal path / size / agents folder / `config.json`; avoids bare `age` matching inside `agent`; p50 latency).

- **v0.1.940** — Instant lane: testing.md age (`testing age`, `testing.md age`, `how old is testing`, `when was testing updated`; newest mtime across per-agent files; no dump; does not steal path / size / skill / mood / soul; does not run tests; p50 latency).

- **v0.1.939** — Instant lane: skill.md age (`skill.md age`, `skill file age`, `how old is skill.md`, `when was skill.md updated`; newest mtime across per-agent files; no dump; does not steal path / size / bare `skill age` / skills folder / mood / soul; p50 latency).

- **v0.1.938** — AI Chat empty Ready calm (connected + model → soft ok wash + “Nothing here yet — glad you're here”; offline/not-set/no-model/circuit cue washes; design review / `feature-ai-chat`).

- **v0.1.937** — Instant lane: mood.md age (`mood age`, `mood.md age`, `how old is mood`, `when was mood updated`; newest mtime across per-agent files; no dump; does not steal path / size / soul / agents; p50 latency).


- **v0.1.936** — Instant lane: soul.md age (`soul age`, `soul.md age`, `how old is soul`, `when was soul updated`; mtime only; no dump; does not steal path / size / mood / agents; p50 latency).


- **v0.1.935** — Instant lane: `.config.env` age (`config.env age`, `.config.env age`, `how old is .config.env`, `when was config.env updated`; mtime only; no key dump; does not steal path / size / `config age`; p50 latency).

- **v0.1.934** — Instant lane: credential_accounts.json age (`credential accounts age`, `credential_accounts.json age`, `how old is credential accounts`, `when was credential accounts updated`; mtime only; no dump; does not steal path / size / browser credentials; p50 latency).

- **v0.1.933** — Instant lane: user-info.json age (`user info age`, `user-info.json age`, `how old is user info`, `when was user info updated`; mtime only; no dump; does not steal path / size / who-am-i; p50 latency).

- **v0.1.932** — Instant lane: scheduler_delivery_awareness.json age (`delivery awareness age`, `scheduler_delivery_awareness.json age`, `how old is delivery awareness`, `when was delivery awareness updated`; mtime only; no dump; does not steal path / size / `last delivery` / `/schedules`; p50 latency).

- **v0.1.931** — Instant lane: perplexity_last.json age (`perplexity last age`, `perplexity_last.json age`, `how old is perplexity last`, `when was perplexity last updated`; mtime only; no dump; does not steal path / size / `/perplexity`; p50 latency).

- **v0.1.930** — Instant lane: discord_channels.json age (`discord channels age`, `discord_channels.json age`, `how old is discord channels`, `when was discord channels updated`; mtime only; no dump; does not steal path / size / `/discord`; p50 latency).

- **v0.1.929** — Instant lane: pinned_processes.json age (`pinned processes age`, `pinned_processes.json age`, `how old is pinned processes`, `when was pinned processes updated`; mtime only; no dump; does not steal path / size / `/pinned`; p50 latency).

- **v0.1.928** — Instant lane: disk_cleanup.json age (`disk cleanup age`, `disk_cleanup.json age`, `how old is disk cleanup`, `when was disk cleanup updated`; mtime only; no dump; does not steal path / size / `/disk`; p50 latency).

- **v0.1.927** — Instant lane: history.json age (`history age`, `history.json age`, `how old is history`, `when was history updated`; mtime only; no dump; does not steal path / size / chat history; p50 latency).

- **v0.1.926** — Instant lane: monitors.json age (`monitors age`, `monitors.json age`, `how old is monitors`, `when was monitors updated`; mtime only; no dump; does not steal path / size / `/monitors`; p50 latency).

- **v0.1.925** — Instant lane: schedules.json age (`schedules age`, `schedules.json age`, `how old is schedules`, `when was schedules updated`; mtime only; no dump; does not steal path / size / `/schedules`; p50 latency).

- **v0.1.924** — Instant lane: config.json age (`config age`, `config.json age`, `how old is config`, `when was config updated`; mtime only; no dump; does not steal path / size / `.config.env`; p50 latency).

- **v0.1.923** — Instant lane: Ori vault size (`ori vault size`, `how big is ori vault`, `mnemos vault size`, `ori_vault size`; recursive file bytes under configured vault; no list/MCP; does not steal path / `/ori` Ready; p50 latency).

- **v0.1.922** — Instant lane: before-compaction transcript size (`before compaction transcript size`, `before-compaction transcript size`, `how big is before compaction transcript`, `last_session_before_compaction.jsonl size`; stat only; no dump; does not steal path / before-reset / session reset phrases; p50 latency).

- **v0.1.921** — Instant lane: before-reset transcript size (`before reset transcript size`, `before-reset transcript size`, `how big is before reset transcript`, `last_session_before_reset.jsonl size`; stat only; no dump; does not steal path / before-compaction / session reset phrases; p50 latency).

- **v0.1.920** — Instant lane: session-memory size (`session memory size`, `session-memory size`, `how big is session memory`; sum `session-memory-*.md`; stat only; no dump; does not steal path / `session size` / `/sessions`; p50 latency).

- **v0.1.919** — Agent Ops Overview Live idle calm (Discord Ready + empty → ok wash + warm empty copy; not amber warn; design review / `feature-agent-ops`).

- **v0.1.918** — Instant lane: Discord channel memory size (`discord memory size`, `memory-discord size`, `how big is discord memory`, `channel memory size`; sum `memory-discord-*.md`; stat only; no dump; does not steal path / `/knowledge discord` / `memory.md size`; p50 latency).

- **v0.1.917** — Instant lane: LaunchAgent plist size (`launchagent size`, `launchagent plist size`, `how big is the launchagent`, `mac-stats.plist size`, `harness plist size`; app KeepAlive + overnight harness; stat only; no dump; does not steal `launchagent path` / load/unload; p50 latency).

- **v0.1.916** — Instant lane: browser-credentials.toml size (`browser credentials size`, `browser-credentials.toml size`, `how big are browser credentials`, `browser credentials file size`; stat only; no dump; does not steal `browser credentials path` / storage state / credential accounts; p50 latency).

- **v0.1.915** — Instant lane: browser_storage_state.json size (`storage state size`, `browser_storage_state.json size`, `how big are browser cookies`, `browser cookies size`; stat only; no dump; does not steal `storage state path` / cookie reject / browser credentials; p50 latency).

- **v0.1.914** — History sparkline Hot attention glance (**Hot · CPU · Temp** above sparklines when amber; click → first hot chart + ring; amber wash on hot charts; rings keep pulse-only under gauges; design review / `feature-cpu-metrics`; PNG recaptured).


- **v0.1.913** — Instant lane: downloads-organizer-state.json size (`organizer state size`, `downloads-organizer-state.json size`, `how big is downloads organizer state`, `downloads organizer state size`; stat only; no dump; does not steal `downloads organizer state path` / rules / `/downloads`; p50 latency).

- **v0.1.912** — Instant lane: downloads-organizer-rules.md size (`organizer rules size`, `downloads-organizer-rules.md size`, `how big is downloads organizer rules`, `downloads organizer rules size`; stat only; no dump; does not steal `downloads organizer rules path` / organizer state / `/downloads`; p50 latency).

- **v0.1.911** — Instant lane: cookie_reject_patterns.md size (`cookie reject size`, `cookie_reject_patterns.md size`, `how big is cookie reject patterns`, `cookie reject patterns size`; stat only; no dump; does not steal `cookie reject patterns path` / session-reset / escalation; p50 latency).

- **v0.1.910** — Instant lane: session_reset_phrases.md size (`session reset size`, `session_reset_phrases.md size`, `how big is session reset phrases`, `session reset phrases size`; stat only; no dump; does not steal `session reset phrases path` / escalation / cookie reject; p50 latency).

- **v0.1.909** — Instant lane: escalation_patterns.md size (`escalation size`, `escalation_patterns.md size`, `how big is escalation patterns`, `escalation patterns size`; stat only; no dump; does not steal `escalation patterns path` / session-reset / cookie reject; p50 latency).

- **v0.1.908** — Instant lane: memory.md size (`memory.md size`, `curated memory size`, `how big is memory.md`, `memory file size`; curated `agents/memory.md` stat only; no dump; does not steal `memory.md path` / `notes size` / bare `memory size` RAM; p50 latency).

- **v0.1.907** — Instant lane: agent.json size (`agent.json size`, `agent config size`, `how big is agent.json`; stat only across per-agent `agent.json`; no dump; does not steal `agent.json path` / `agents size` / `config.json`; p50 latency).

- **v0.1.906** — Instant lane: execution_prompt.md size (`execution size`, `execution_prompt.md size`, `how big is execution_prompt.md`, `execution prompt size`; stat only; no dump; does not steal `execution_prompt.md path` / `prompts size`; p50 latency).

- **v0.1.905** — Instant lane: planning_prompt.md size (`planning size`, `planning_prompt.md size`, `how big is planning_prompt.md`, `planning prompt size`; stat only; no dump; does not steal `planning_prompt.md path` / `prompts size`; p50 latency).

- **v0.1.904** — Instant lane: testing.md size (`testing size`, `testing.md size`, `how big is testing.md`, `testing file size`; stat only across per-agent `testing.md`; no dump; does not steal `testing.md path` / run tests / `/agents`; p50 latency).

- **v0.1.903** — Instant lane: skill.md size (`skill.md size`, `skill file size`, `how big is skill.md`; stat only across per-agent `skill.md`; no dump; does not steal `skill.md path` / bare `skill size` / `skills size` / `/skills`; p50 latency).

- **v0.1.902** — Instant lane: mood.md size (`mood size`, `mood.md size`, `how big is mood`, `mood file size`; stat only across per-agent `mood.md`; no dump; does not steal `mood path` / soul / agents; p50 latency).

- **v0.1.901** — Instant lane: soul.md size (`soul size`, `soul.md size`, `how big is soul`, `soul file size`; stat only; no dump; does not steal `soul path` / mood / agents; p50 latency).

- **v0.1.900** — Instant lane: `.config.env` size (`config.env size`, `.config.env size`, `how big is .config.env`, `secrets env size`; stat only; no key dump; does not steal `config.env path` / `config size` / `config.json`; p50 latency).

- **v0.1.899** — Instant lane: credential_accounts.json size (`credential accounts size`, `credential_accounts.json size`, `how big is credential accounts`, `keychain accounts size`; stat only; no dump; does not steal `credential accounts path` / browser credentials; p50 latency).

- **v0.1.898** — Instant lane: user-info.json size (`user info size`, `user-info.json size`, `how big is user info`, `user details size`; stat only; no dump; does not steal `user info path` / who-am-i; p50 latency).

- **v0.1.897** — Instant lane: scheduler_delivery_awareness.json size (`delivery awareness size`, `scheduler_delivery_awareness.json size`, `how big is delivery awareness`, `awareness file size`; stat only; no dump; does not steal `delivery awareness path` / `last delivery` / `/schedules`; p50 latency).

- **v0.1.896** — Instant lane: perplexity_last.json size (`perplexity last size`, `perplexity_last.json size`, `how big is perplexity last`, `last search file size`; stat only; no dump; does not steal `perplexity last path` / `/perplexity`; p50 latency).

- **v0.1.895** — Instant lane: discord_channels.json size (`discord channels size`, `discord_channels.json size`, `how big is discord channels`, `channels.json size`; stat only; no dump; does not steal `discord channels path` / `/discord`; p50 latency).

- **v0.1.894** — Instant lane: pinned_processes.json size (`pinned processes size`, `pinned_processes.json size`, `how big is pinned processes`, `pin file size`; stat only; no dump; does not steal `pinned processes path` / `/pinned`; p50 latency).

- **v0.1.893** — Instant lane: disk_cleanup.json size (`disk cleanup size`, `disk_cleanup.json size`, `how big is disk cleanup`, `cleanup file size`; stat only; no dump; does not steal `disk cleanup path` / `/disk` / quarantine; p50 latency).

- **v0.1.892** — Instant lane: history.json size (`history size`, `history.json size`, `how big is history`, `metrics history size`; stat only; no dump; does not steal `history path` / chat history; p50 latency).

- **v0.1.891** — Instant lane: monitors.json size (`monitors size`, `monitors.json size`, `how big is monitors`; stat only; no dump; does not steal `monitors path` / `/monitors` / add/check; p50 latency).

- **v0.1.890** — Instant lane: schedules.json size (`schedules size`, `schedules.json size`, `how big is schedules`; stat only; no dump; does not steal `schedules path` / `/schedules` / schedule count; p50 latency).

- **v0.1.889** — Instant lane: config.json size (`config size`, `config.json size`, `how big is config`; stat only; no dump; does not steal `config path` / `.config.env` / `agent.json`; p50 latency).

- **v0.1.888** — Instant lane: `review logs` / `check logs` / `look at logs` / `read logs` (and close variants) → existing `/logs` Debug Log tail (no Ollama / Brave; digester Slowest 40s Brave waste; does not steal fix/explain/clear; p50 latency).

- **v0.1.887** — Instant lane: notes / memory folder size (`notes size`, `how big are notes`, `memory folder size`, `notes folder size`; recursive file bytes under `~/.mac-stats/agents/notes/`; no list dump; does not steal `memory path` / `notes path` / scrub / save; rejects bare `memory size` (RAM); p50 latency).

- **v0.1.886** — Instant lane: cleanup-quarantine directory size (`cleanup quarantine size`, `how big is quarantine`, `quarantine folder size`; recursive file bytes under `~/.mac-stats/cleanup-quarantine/`; no list dump; does not steal `cleanup quarantine path` / `/disk` / list/prune/restore; p50 latency).

- **v0.1.885** — Instant lane: task directory size (`task size`, `how big are tasks`, `task folder size`; recursive file bytes under `~/.mac-stats/task/`; no list dump; does not steal `task path` / `/tasks` / `TASK_CREATE:`; p50 latency).

- **v0.1.884** — Instant lane: session directory size (`session size`, `how big are sessions`, `session folder size`; recursive file bytes under `~/.mac-stats/session/`; no list dump; does not steal `session path` / `/sessions` Live/Files / session-memory path; p50 latency).

- **v0.1.883** — Instant lane: prompts directory size (`prompts size`, `how big are prompts`, `prompts folder size`; recursive file bytes under `~/.mac-stats/agents/prompts/`; no list dump; does not steal `prompts path` / planning·execution file paths / system prompt; p50 latency).

- **v0.1.882** — Instant lane: plugins/scripts directory size (`plugins size`, `scripts size`, `how big are plugins`, `plugins folder size`; recursive file bytes under `~/.mac-stats/scripts/`; no list dump; does not steal `plugins path` / `/plugins` On/Off; p50 latency).

- **v0.1.881** — Instant lane: skills directory size (`skills size`, `how big are skills`, `skills folder size`; recursive file bytes under Hermes skills dir; no list dump; does not steal `skills path` / `/skills` / skill.md / `SKILL:`; p50 latency).

- **v0.1.880** — Instant lane: agents directory size (`agents size`, `how big are agents`, `agents folder size`; recursive file bytes under `~/.mac-stats/agents/`; no list dump; does not steal `agents path` / `/agents` / agent.json; p50 latency).

- **v0.1.879** — Instant lane: browser-downloads directory size (`browser downloads size`, `how big are browser downloads`, `browser-downloads size`; recursive file bytes under `~/.mac-stats/browser-downloads/`; no list dump; does not steal `browser downloads path` / `/downloads` / download-now; p50 latency).

- **v0.1.878** — Instant lane: PDF exports directory size (`pdfs size`, `how big are pdfs`, `pdfs folder size`; recursive file bytes under `~/.mac-stats/pdfs/`; no list dump; does not steal `pdfs path` / save/list; p50 latency).

- **v0.1.877** — Instant lane: CDP traces directory size (`traces size`, `how big are traces`, `traces folder size`, `cdp traces size`; recursive file bytes under `~/.mac-stats/traces/`; no list dump; does not steal `traces path` / prune/list; p50 latency).

- **v0.1.876** — Instant lane: uploads directory size (`uploads size`, `how big are uploads`, `uploads folder size`; recursive file bytes under `~/.mac-stats/uploads/`; no list dump; does not steal `uploads path` / upload/list; p50 latency).

- **v0.1.875** — Instant lane: tmp directory size (`tmp size`, `how big is tmp`, `tmp folder size`; recursive file bytes under `~/.mac-stats/tmp/`; no list dump; does not steal `tmp path` / prune/clean / temperature; p50 latency).

- **v0.1.874** — Instant lane: screenshots directory size (`screenshots size`, `how big are screenshots`, `screenshots folder size`; recursive file bytes under BROWSER_SCREENSHOT dir; no list dump; does not steal `screenshot path` / take/list; p50 latency).

- **v0.1.873** — Instant lane: improvements directory size (`improvements size`, `how big is the improvements folder`, `improvements dir size`, `how large is the improvements directory`; recursive file bytes under `~/.mac-stats/improvements/`; no list dump; does not steal `improvements path` / overnight improvements asks / `results.tsv size` / digest size; p50 latency).

- **v0.1.872** — Instant lane: digest.md file size (`digest.md size`, `latest.md size`, `how big is digest.md`, `how big is latest.md`; `~/.mac-stats/improvements/latest.md` stat only; no digester spawn / open dump; does not steal `digest size` / `latest.json size` / `digest age` / `digest open` / `/digest`; p50 latency).

- **v0.1.871** — Instant lane: digest file size (`digest size`, `how big is the digest`, `latest.json size`; `~/.mac-stats/improvements/latest.json` stat only; no digester spawn / open dump; does not steal `digest age` / `digest open` / `/digest`; p50 latency).

- **v0.1.870** — Instant lane: runs.jsonl size (`runs size`, `how big is runs.jsonl`, `runs file size`, `how large is runs.jsonl`; stat only; no list/count/prune; does not steal `runs path` / `runs age` / `/insights`; p50 latency).

- **v0.1.869** — Instant lane: results.tsv size (`results.tsv size`, `how big is results.tsv`, `results file size`, `how large is autoresearch results`; stat only; no dump; does not steal `results.tsv path` / `results.tsv age` / `improvements path`; p50 latency).

- **v0.1.868** — Instant lane: results.tsv age (`results.tsv age`, `how old is results.tsv`, `when was results.tsv updated`, `results.tsv last modified`; mtime only; no dump; does not steal `results.tsv path` / `improvements path`; p50 latency).

- **v0.1.867** — Instant lane: runs.jsonl age (`runs age`, `how old is runs.jsonl`, `when was runs updated`, `runs.jsonl last modified`; mtime only; no list/count; does not steal `runs path` / `/insights` / how-many-runs; p50 latency).

- **v0.1.866** — Instant lane: results.tsv path (`results.tsv path`, `where is results.tsv`, `autoresearch results path`, `ratchet results path`; `~/.mac-stats/improvements/autoresearch/results.tsv`; config only; no dump; does not steal `improvements path` / bare `autoresearch path`; p50 latency).

- **v0.1.865** — Agent Ops Filter attention glance (**Filter · On/Off** / **Live/Files** / **Jobs/Deliveries** / **Discord/Core** / **Instant/Lite/Direct/Slow/Fail** when active; click → All; Runs Fail/Slow strip defers on Fail/Slow Filter; design review / `feature-agent-ops`).

- **v0.1.864** — Instant lane: LaunchAgent plist path (`launchagent path`, `where is launchagent`, `mac-stats.plist`, `harness plist`; `~/Library/LaunchAgents/com.raro42.mac-stats.plist` + overnight harness plist; config only; no load/unload; does not steal overnight-improvements / `improvements path`; p50 latency).

- **v0.1.863** — Instant lane: session-memory path (`session memory path`, `where is session memory`, `session-memory path`; `session/session-memory-<id>-<ts>-<topic>.md`; config only; no list/dump; does not steal `session path` / `/sessions` / discord memory / notes folder; p50 latency).

- **v0.1.862** — Instant lane: Discord channel memory path (`discord memory path`, `where is discord memory`, `memory-discord path`, `channel memory path`; agents/`memory-discord-<channelId>.md`; config only; no list/dump; does not steal bare `discord memory` / `/knowledge discord` / `memory.md` / notes folder; p50 latency).

- **v0.1.861** — Instant lane: before-compaction transcript path (`before compaction transcript path`, `where is before compaction transcript`, `last_session_before_compaction.jsonl`; config/env or default under agents/; no dump/hook; does not steal before-reset / session reset phrases / session dir; p50 latency).

- **v0.1.860** — Instant lane: before-reset transcript path (`before reset transcript path`, `where is before reset transcript`, `last_session_before_reset.jsonl`; config/env or default under agents/; no dump/hook; does not steal session reset phrases / before-compaction / session dir; p50 latency).

- **v0.1.859** — Instant lane: Ori vault path (`ori vault path`, `where is ori vault`, `mnemos vault path`, `ORI_VAULT path`; config/env only; no list/MCP; does not steal `/ori` / bare `ori vault` Ready; p50 latency).

- **v0.1.858** — Instant lane: memory.md path (`memory.md`, `where is memory.md`, `memory.md path`, `curated memory path`, `memory file path`; curated `agents/memory.md` only; config only; no dump/edit; does not steal `memory path` / notes folder / scrub / soul / agents path; p50 latency).

- **v0.1.857** — Instant lane: agent.json path (`agent.json`, `where is agent.json`, `agent.json path`, `agent config path`; per-agent `agent-<id>/agent.json`; config only; no dump/edit; does not steal `agents path` / app `config path` / soul / skill / testing path; p50 latency).

- **v0.1.856** — Instant lane: execution_prompt.md path (`execution_prompt.md`, `where is execution_prompt.md`, `execution prompt path`, `execution path`; config only; no dump/edit; does not steal `prompts path` / planning prompt / testing / agents path; bare `execution prompt` still goes to the model; p50 latency).

- **v0.1.855** — Instant lane: planning_prompt.md path (`planning_prompt.md`, `where is planning_prompt.md`, `planning prompt path`, `planning path`; config only; no dump/edit; does not steal `prompts path` / execution prompt / testing / agents path; bare `planning prompt` still goes to the model; p50 latency).

- **v0.1.854** — Instant lane: testing.md path (`testing.md`, `where is testing.md`, `testing path`, `testing file path`; per-agent `agent-<id>/testing.md`; config only; no dump/edit/run; does not steal `/agents` / agents dir / skill / mood / soul path; does not steal `agent test` / `run tests`; p50 latency).

- **v0.1.853** — Instant lane: skill.md path (`skill.md`, `where is skill.md`, `skill file path`; per-agent `agent-<id>/skill.md`; config only; no dump/edit; does not steal `/skills` / skills dir / mood / soul / agents path; bare `skill path` still means skills directory; p50 latency).

- **v0.1.852** — Instant lane: mood.md path (`mood path`, `where is mood.md`, `mood file path`; per-agent `agent-<id>/mood.md`; config only; no dump/edit; does not steal `/agents` / agents dir / soul / memory path; p50 latency).

- **v0.1.851** — Instant lane: soul.md path (`soul path`, `where is soul.md`, `soul file path`; config only; no dump/edit; does not steal `/agents` / agents dir / memory / mood path; p50 latency).

- **v0.1.850** — Instant lane: downloads-organizer-state.json path (`downloads organizer state path`, `where is downloads-organizer-state.json`, `organizer state path`; config only; no dump/run; does not steal `/downloads` / rules path / browser-downloads / agents path; p50 latency).

- **v0.1.849** — Instant lane: downloads-organizer-rules.md path (`downloads organizer rules path`, `where is downloads-organizer-rules.md`, `organizer rules path`; config only; no list/run; does not steal `/downloads` / browser-downloads / agents path; p50 latency).

- **v0.1.848** — Instant lane: cookie_reject_patterns.md path (`cookie reject patterns path`, `where is cookie_reject_patterns.md`, `cookie reject path`, `reject patterns path`; config only; no list/edit; does not steal browser cookies / session-reset / escalation / agents path; p50 latency).

- **v0.1.847** — Instant lane: session_reset_phrases.md path (`session reset phrases path`, `where is session_reset_phrases.md`, `reset phrases path`; config only; no list/clear; does not steal escalation / agents / session dir path; p50 latency).

- **v0.1.846** — Instant lane: escalation_patterns.md path (`escalation patterns path`, `where is escalation_patterns.md`, `escalation file path`; config only; no list/append; does not steal session-reset / agents path; p50 latency).

- **v0.1.845** — Instant lane: credential_accounts.json path (`credential accounts path`, `where is credential_accounts.json`, `keychain accounts path`; config only; no list/dump; does not steal browser credentials; p50 latency).

- **v0.1.844** — Disk Cleanup Filter attention glance (**Filter · Reclaim** / **Filter · Big** / **Filter · Clean** when active; click → All; Reclaim/Due strip defers; design review / `feature-disk-cleanup`).

- **v0.1.843** — External / Monitors Filter attention glance (**Filter · Up** / **Filter · Down** / **Filter · Slow** when active; click → All; Down/Slow strip defers; design review / `feature-monitors`).

- **v0.1.842** — Top Processes Filter attention glance (**Filter · Pinned** / **Filter · Hot** when active; click → All; Hot strip defers; design review / `feature-processes`).

- **v0.1.841** — Instant lane: improvements directory path (`improvements path`, `where is the improvements folder`, `autoresearch path`; config only; no list; does not steal overnight improvements asks; p50 latency).

- **v0.1.840** — Instant lane: `.config.env` path (`config.env path`, `where is .config.env`, `config env path`, `secrets env path`; path only; no key dump; does not steal `where is config`; p50 latency).

- **v0.1.839** — Instant lane: user-info.json path (`user info path`, `where is user-info.json`, `user-info path`; config only; no list/edit; p50 latency).

- **v0.1.838** — Instant lane: scheduler_delivery_awareness.json path (`delivery awareness path`, `where is scheduler_delivery_awareness.json`, `awareness file path`; config only; no list; does not steal `last delivery` / `/schedules`; p50 latency).

- **v0.1.837** — Instant lane: discord_channels.json path (`discord channels path`, `where is discord_channels.json`, `channels.json`; config only; no list/edit; does not steal `/discord`; p50 latency).

- **v0.1.836** — Instant lane: perplexity_last.json path (`perplexity last path`, `where is perplexity_last.json`, `last search file`; config only; no Top/Snippet dump / new search; does not steal `/perplexity`; p50 latency).

- **v0.1.835** — Instant lane: disk_cleanup.json path (`disk cleanup path`, `where is disk_cleanup.json`, `cleanup file path`; config only; no list/reclaim/clean; does not steal `/disk`; p50 latency).

- **v0.1.834** — Instant lane: history.json path (`history path`, `where is history.json`, `metrics history file`; config only; no sparkline dump / chat history; p50 latency).

- **v0.1.833** — Instant lane: monitors.json path (`monitors path`, `where is monitors.json`, `monitor file path`; config only; no list/add/check; does not steal `/monitors`; p50 latency).

- **v0.1.832** — Instant lane: schedules.json path (`schedules path`, `where is schedules.json`, `schedule file path`; config only; no list/count/create; does not steal `/schedules`; p50 latency).

- **v0.1.831** — Instant lane: pinned_processes.json path (`pinned processes path`, `where is pinned_processes.json`, `pin file path`; config only; no list/pin/unpin; does not steal `/pinned`; p50 latency).

- **v0.1.830** — Instant lane: cleanup-quarantine directory path (`cleanup quarantine path`, `where is cleanup-quarantine`, `quarantine folder`; config only; no list/prune/restore; does not steal `/disk`; p50 latency).

- **v0.1.829** — Instant lane: browser-downloads directory path (`browser downloads path`, `where are browser downloads`, `browser-downloads`; config only; no list/prune; does not steal `/downloads`; p50 latency).

- **v0.1.828** — Instant lane: browser storage-state / cookies path (`storage state path`, `where are browser cookies`, `browser_storage_state.json`; config only; no list/clear; p50 latency).

- **v0.1.827** — Instant lane: browser credentials path (`browser credentials path`, `where are browser credentials`, `browser-credentials.toml`; config only; no list/edit; p50 latency).

- **v0.1.826** — Instant lane: PDF exports directory path (`pdfs path`, `where is the pdfs folder`, `pdf directory`; config only; no list/save; p50 latency).

- **v0.1.825** — Instant lane: CDP traces directory path (`traces path`, `where is the traces folder`, `cdp traces`; config only; no list/prune; p50 latency).

- **v0.1.824** — Instant lane: uploads directory path (`uploads path`, `where is the uploads folder`, `upload directory`; config only; no list/upload; p50 latency).

- **v0.1.823** — Startup: skip macOS crash-restore modal (`ApplePersistenceIgnoreState`; LaunchAgent/overnight restarts no longer block Discord on `NSPersistentUIRestorer`).

- **v0.1.822** — Instant lane: tmp directory path (`tmp path`, `where is the tmp folder`, `temp directory`; config only; no list/prune; p50 latency).

- **v0.1.821** — Instant lane: prompts directory path (`prompts path`, `where is the prompts folder`, `prompts directory`; config only; no open/edit; p50 latency).

- **v0.1.820** — Instant lane: plugins/scripts directory path (`plugins path`, `scripts path`, `where is the plugins folder`; config only; no list/run; p50 latency).

- **v0.1.819** — Instant lane: skills directory path (`skills path`, `where is the skills folder`, `skills directory`; config only; no list/run; p50 latency).

- **v0.1.818** — Weather STT place: `Elmasnow` / `el masnow` → El Masnou (Open-Meteo instant; digester Brave weather open).

- **v0.1.817** — Instant lane: agents directory path (`agents path`, `where is the agents folder`, `agents directory`; config only; no list/create; p50 latency).

- **v0.1.816** — Instant lane: session directory path (`session path`, `where is the session folder`, `session directory`; config only; no list/resume; p50 latency).

- **v0.1.815** — Instant lane: memory / notes path (`memory path`, `notes path`, `where are notes`, `notes folder`; config only; no list/save/scrub; p50 latency).

- **v0.1.814** — Instant lane: task directory path (`task path`, `where is the task folder`, `task directory`; config only; no list/create; p50 latency).

- **v0.1.813** — Instant lane: runs.jsonl path (`runs path`, `where is runs.jsonl`, `runs file path`; config only; no list/count/prune; p50 latency).

- **v0.1.812** — Instant lane: screenshots path (`screenshot path`, `where are screenshots`, `screenshot folder`; config only; no take/list/prune; p50 latency).

- **v0.1.811** — Instant lane: config path / data home (`where is config`, `config path`, `mac-stats home`, `where is data directory`; config only; p50 latency).

- **v0.1.810** — Instant lane: debug log age (`log age`, `how old is the log`, `when was log updated`; mtime only; p50 latency).

- **v0.1.809** — Instant lane: debug log path (`where is the log`, `log file path`, `debug log path`; config only; p50 latency).

- **v0.1.808** — Instant lane: debug log file size (`log file size`, `how big is the log`, `debug log size`; stat only; p50 latency).

- **v0.1.807** — Instant lane: debug log error/warn count (`how many errors in the log`, `log error count`, `debug log count`; tail counts; p50 latency).

- **v0.1.806** — Instant lane: digest age read-only (`digest age`, `how old is the digest`, `when was digest updated`; cached `latest.json` without digester spawn; p50 latency).

- **v0.1.805** — Instant lane: digest open read-only (`digest open`, `open candidates`; cached `latest.json` without digester spawn; p50 latency).

- **v0.1.804** — Instant lane: runs count (`how many runs`, failed/slow/lane counts; p50 latency).

- **v0.1.803** — Instant lane: operator inventory counts (agents/monitors/tasks/sessions/skills/plugins/knowledge/digest open; p50 latency).

- **v0.1.802** — Instant lane: schedule / delivery count (p50 latency).

- **v0.1.801** — Instant lane: last delivery (p50 latency).

- **v0.1.800** — Instant lane: next schedule / next job (p50 latency).

- **v0.1.799** — Agent Ops Signal health attention glance (**Signal · Not wired · REST API pending** / **Partial · N channel(s) · …** from `get_feature_health` config probe under Slack; Settings Credentials + Signal note scroll; design review / `feature-agent-ops`).

- **v0.1.798** — Agent Ops Slack health attention glance (**Slack · Not set · add webhook URL** / **Partial · …** from `get_feature_health` config probe under Telegram; Settings Credentials + `slack-webhook-input`; design review / `feature-agent-ops`).

- **v0.1.797** — Agent Ops Telegram health attention glance (**Telegram · Not set · add bot token + chat id** / **Partial · …** from `get_feature_health` config probe under Mastodon; Settings Credentials + `telegram-bot-token-input` / `telegram-chat-id-input`; design review / `feature-agent-ops`).

- **v0.1.796** — Agent Ops Mastodon health attention glance (**Mastodon · Not set · add URL + token** / **Partial · …** from `get_feature_health` config probe under Perplexity; Settings Credentials + `mastodon-url-input` / `mastodon-token-input`; design review / `feature-agent-ops`).

- **v0.1.795** — Agent Ops Perplexity health attention glance (**Perplexity · Not set · add API key** from `get_feature_health` config probe under Cursor; Settings Credentials + `perplexity-api-key-input`; design review / `feature-agent-ops`).

- **v0.1.794** — Agent Ops Cursor health attention glance (**Cursor · Not set · add binary path** from `get_feature_health` config probe under MCP; Settings Credentials + `cursor-agent-executable-input`; design review / `feature-agent-ops`).

- **v0.1.793** — Agent Ops MCP health attention glance (**MCP · Not set · add URL or stdio** from `get_feature_health` config probe under Browser; Settings Credentials + `mcp-url-input`; design review / `feature-agent-ops`).

- **v0.1.792** — Agent Ops Browser health attention glance (**Browser · Not set / Unavailable / Degraded** from `get_feature_health` under Brave; Settings Credentials + `browser-chromium-path-input`; design review / `feature-agent-ops`).

- **v0.1.791** — Agent Ops Brave health attention glance (**Brave · Not set / Unavailable** from `get_feature_health` under Ollama; Settings Credentials + `brave-api-key-input`; design review / `feature-agent-ops`).

- **v0.1.790** — Agent Ops Ollama health attention glance (**Ollama · Not set / Offline / Degraded** from `get_feature_health` under Redmine; URL dialog or model picker; design review / `feature-agent-ops`).

- **v0.1.789** — Agent Ops Redmine health attention glance (**Redmine · Not set / Degraded / Unavailable** under Discord when probe not ok; Not set → Settings; else Redmine agent preview; design review / `feature-agent-ops`).

- **v0.1.788** — Perplexity Filter attention glance (Search · Filter · Top/Snippet when active; click → All; collapsed Filter · … parity; design review polish).

- **v0.1.787** — AI Chat Last answer attention glance (Chat · Last answer · preview when successful reply; click copies; collapsed parity; design review / `feature-ai-chat`).

- **v0.1.786** — AI Chat Errors attention glance (Chat · Errors · N failed when All; click → Errors filter; collapsed parity; design review / `feature-ai-chat`).

- **v0.1.785** — AI Chat Filter attention glance (You/Assistant/Errors active filter; click → All; design review / `feature-ai-chat`).

- **v0.1.784** — AI Chat Continue · ask another attention glance (history; design review / `feature-ai-chat`).

- **v0.1.783** — AI Chat Sending attention glance (in-flight; design review / `feature-ai-chat`).

- **v0.1.782** — AI Chat Ready · try a starter attention glance (empty Ready; design review / `feature-ai-chat`).

- **v0.1.781** — AI Chat Circuit-open attention glance (Offline · circuit open; `/ollama` parity; design review / `feature-ai-chat`).

- **v0.1.780** — AI Chat No-model attention glance (Offline parity; `/ollama` pick-one cue; design review / `feature-ai-chat`).

- **v0.1.779** — Settings Compact On attention glance (Product; `/compact` mentions expand in Settings).

- **v0.1.778** — Settings AI Off attention glance (Product; `/ai` mentions Settings).

- **v0.1.777** — Settings Discord voice STT toggle + Off attention glance (Product; `/voice` mentions Settings).

- **v0.1.776** — Settings Having fun / idle thoughts toggle + Off attention glance (Product; `/having_fun` mentions Settings).

- **v0.1.775** — Settings Ori Mnemos lifecycle toggle + Off attention glance (Product; `/ori` mentions Settings).

- **v0.1.774** — Settings Downloads organizer toggle + Off attention glance (Product; `/downloads` mentions Settings).

- **v0.1.773** — Settings Judge toggles + Off attention glance (Product; `/judge` mentions Settings).

- **v0.1.772** — Settings Signal alerts honest placeholder (**Signal · Not wired · REST API pending** in Credentials; `/signal` mentions Settings; Slack/Telegram parity glance; no fake Keychain until Signal REST is wired).

- **v0.1.771** — Settings Slack alerts webhook + not-set attention glance (**Slack · Not set · add webhook URL** in Credentials; Keychain `slack_webhook_slack_default`; channel restores at startup; `/slack` Not-set mentions Settings; Telegram parity).

- **v0.1.770** — Settings Telegram alerts bot token + chat id + not-set/partial attention glance (**Telegram · Not set / Partial** in Credentials; Keychain `telegram_bot_default` / `telegram_chat_default`; channel restores at startup; `/telegram` Not-set mentions Settings; Mastodon/Cursor parity).

- **v0.1.769** — Settings Cursor agent workspace + binary + not-set attention glance (**Cursor · Not set · add binary path** in Credentials; config.json `cursorAgentWorkspace` / `cursorAgentExecutable`; `/cursor` Not-set mentions Settings; Browser parity).

- **v0.1.768** — Settings Browser / CDP Chromium path + port + not-set attention glance (**Browser · Not set · add Chromium path** in Credentials; config.json `browserChromiumExecutable` / `browserCdpPort`; `/browser` Not-set mentions Settings; MCP parity).

- **v0.1.767** — Settings MCP server URL + stdio + not-set attention glance (**MCP · Not set · add URL or stdio** in Credentials; Keychain `mcp_server_url` / `mcp_server_stdio`; `/mcp` Not-set mentions Settings; Mastodon/Redmine/Brave/Perplexity/Discord parity).

- **v0.1.766** — Settings Mastodon instance URL + access token + not-set/partial attention glance (**Mastodon · Not set / Partial** in Credentials; Keychain `mastodon_instance_url` / `mastodon_access_token`; `/mastodon` Not-set mentions Settings; Redmine/Brave/Perplexity/Discord parity).

- **v0.1.765** — Settings Redmine URL + API key + not-set/partial attention glance (**Redmine · Not set / Partial** in Credentials; Keychain `redmine_url` / `redmine_api_key`; `/redmine` Not-set mentions Settings; Brave/Perplexity/Discord parity).

- **v0.1.764** — Settings Brave Search API key + key-not-set attention glance (**Brave · Not set · add API key** in Credentials; Keychain `brave_api_key`; `/brave` Not-set mentions Settings; Discord/Perplexity parity).

- **v0.1.763** — Settings Perplexity key-not-set attention glance (**Search · Not set · add API key** above Perplexity controls when Credentials open; click → key field; Discord token-not-set parity).

- **v0.1.762** — Hot ring gauges pulse (amber glow) instead of layout-shifting Hot bar under gauges.

- **v0.1.761** — CPU window blank UI fix (`cpu.js` missing `}` in `initLogsSection`).

- **v0.1.760** — Settings Discord token-not-set attention glance (**Discord · Not set · add bot token** above Discord bot controls when Credentials open; click → token field; Perplexity Key-not-set / Help parity).

- **v0.1.759** — Settings Help attention glance (**Help · cheat sheet · click Help** accent strip above Product actions when Settings open; green **Help · open · Enter/c copies** when sheet open; click opens or copies).

- **v0.1.758** — Perplexity Key-not-set attention glance (**Search · Not set · add API key** above setup when expanded; click → inline key; design review polish).

- **v0.1.757** — Agent Ops Discord Offline attention glance (**Discord · Offline · check gateway** / **Discord · Reconnect · disc×N** under Fail/Slow/Digest when expanded; click → Runs gateway preview; design review / `feature-agent-ops`).

- **v0.1.756** — AI Chat Offline attention glance (**Chat · Offline · check Ollama** / **Chat · Not set · configure URL** above All·You·Assistant when open; click → Ollama URL; design review / `feature-ai-chat`).

- **v0.1.755** — Agent Ops Digest open attention glance (**Digest · N open · …** under Fail/Slow when digester has open candidates; click → first hint preview; design review / `feature-agent-ops`).

- **v0.1.754** — Perplexity Top/error attention glance (**Search · error · …** / **Search · N results · Top** above All·Top·Snippet when open; click → focus query or Top filter; design review polish).

- **v0.1.753** — Details Load/RAM Hot attention glance (**Hot · Load · RAM** above Details grid when open; click → first hot row; `/details hot` parity; design review / `feature-cpu-metrics`).

- **v0.1.752** — Debug Log Error/Warn attention glance (**Logs · N errors · M warns** above toolbar when open; click → Error/Warn filter + first line; design review polish).

- **v0.1.751** — Power strip Hot attention glance (**Hot · Bat · Heat · …** under slim power row; click → first cue; `/strip hot` parity; design review / `feature-cpu-metrics`).

- **v0.1.750** — CPU rings Hot attention glance (**Hot · CPU · Temp** strip under gauges; click → first hot ring; design review / `feature-cpu-metrics`).

- **v0.1.749** — Disk Cleanup Reclaim/Due attention glance (amber Big/Reclaim / green Due strip; click → Big or Reclaim filter or Clean now; design review / `feature-disk-cleanup`).

- **v0.1.748** — External / Monitors Down/Slow attention glance (red Down / amber Slow strip; click → Down or Slow filter; design review / `feature-monitors`).

- **v0.1.747** — Top Processes Hot attention glance (amber strip; click → Hot filter; design review / `feature-processes`).


- **v0.1.746** — Agent Ops Runs Fail/Slow glance (attention strip under Refresh; click → Runs Fail/Slow filter; design review / `feature-agent-ops`).

- **v0.1.745** — AI Chat Errors glance (failed-turn strip + Last error wash; click → Errors filter; design review / `feature-ai-chat`).

- **v0.1.744** — `/voice` · `/stt` instant — Discord voice STT Ready / Partial / Not set (model · ffmpeg · Ollama config; config only, no transcribe; Discord + AI Chat; does not steal voice-note / send-voice / enable-disable).

- **v0.1.743** — `/having_fun` · `/fun` · `/idle` instant — Having fun / idle thoughts On/Off (channel count · idle · reply delays; config only, no send; Discord + AI Chat; does not steal send/post / enable-disable).

- **v0.1.742** — `/ori` · `/mnemos` instant — Ori Mnemos lifecycle Ready / Off / Partial (vault · orient · prefetch · capture · binary; config/env only, no subprocess; Discord + AI Chat; does not steal MCP `ori_*` / MEMORY_APPEND / scrub).

- **v0.1.741** — `/downloads` · `/organizer` instant — Downloads organizer On/Off (interval · dry-run · path · last run; config only, no run-now; Discord + AI Chat; does not steal `/disk` / BROWSER_DOWNLOAD / organize-now). Perplexity Ready `perplexity search status` reject fixed.

- **v0.1.740** — `/compact` · `/menu-bar` · `/cpu-window` instant — Compact Menu bar / CPU window On/Off (`menuBarCompact` · `cpuWindowCompact`; config only, no toggle; Discord + AI Chat; does not steal compaction / enable-disable).

- **v0.1.736** — `/ai` · `/ai-agent` instant — AI On / Off (`aiAgentEnabled`; config only, no toggle; Discord + AI Chat; does not steal `/agents` / enable-disable / chat-with-AI).

- **v0.1.735** — `/judge` instant — Judge Ready / Off (agentJudgeEnabled · failure-only vs every run; config only, no judge run; Discord + AI Chat; does not steal “judge this” / enable/disable / score).

- **v0.1.734** — `/browser` · `/cdp` instant — Browser / CDP Ready / Off / Not set (Chromium path + port; config only, no live probe; Discord + AI Chat; does not steal `BROWSER_*` / screenshot / navigate).

- **v0.1.733** — `/plugins` · `/plugins on` · `/plugins off` instant — registered script plugins On/Off list (Agents On/Off parity; Discord + AI Chat; no script run; does not steal add/run/remove or “search for tauri plugins”).

- **v0.1.732** — `/tasks` · `/tasks all` instant — Active (open·WIP) or All task files under `~/.mac-stats/task/` (TASK_LIST parity; Discord + AI Chat; does not steal `TASK_CREATE:` / `TASK_SHOW:` / create-append-status).

- **v0.1.731** — `/skills` instant — installed skills catalog (Hermes skills_list / SKILLS_LIST; Discord + AI Chat; does not steal `SKILL:` / `SKILL_VIEW:`).

- **v0.1.730** — `/telegram` · `/slack` · `/signal` · `/alerts` instant — alert channel Ready / Not set / Partial (Keychain + registry; no live send; Discord catch-all also covers prior Ready chips).

- **v0.1.729** — `/cursor` · `/cursor-agent` instant — Cursor agent Ready / Not set with PATH cue (no CLI probe; Discord + AI Chat; does not steal `CURSOR_AGENT:`).

- **v0.1.728** — `/mcp` instant — MCP Ready / Not set with stdio command · HTTP host cue (config only, no `tools/list` probe; Discord + AI Chat; does not steal `MCP: <tool>`).

- **v0.1.727** — `/mastodon` instant — Mastodon Ready / Not set / Partial with instance host · token cue (config only, no live probe; Discord + AI Chat; does not steal toot/post/timeline).

- **v0.1.726** — `/perplexity key` instant — Perplexity Ready / Not set with key cue (config only, no live probe; Discord + AI Chat; does not steal `/perplexity` last-search Top/Snippet or `perplexity search for …`).

- **v0.1.725** — `/brave` instant — Brave Search Ready / Not set with key cue (config only, no live probe / quota burn; Discord + AI Chat; does not steal web-search / bare `brave search`).

- **v0.1.724** — `/redmine` instant — Redmine Ready / Not set / Partial with URL host · key cue (Agent Ops health parity; config only, no live probe; Discord + AI Chat; does not steal ticket/issue/time/API).

- **v0.1.723** — `/ollama` · `/llm` instant — Ollama Ready / Offline with model · endpoint · circuit (menu-bar ✕ + AI Chat glance parity; Discord + AI Chat; does not steal pull/list/chat).

- **v0.1.722** — `/discord` instant — Discord Ready / Offline with reconnect cues (Agent Ops glance parity; Discord + AI Chat; does not steal `/knowledge discord`).

- **v0.1.721** — `/ram` · `/ssd` · `/uptime` instant — power-strip RAM · SSD · Up chips (RAM/SSD≥85% hot · Up≥7d long; Discord + AI Chat); does not steal `/strip` or `/disk` Disk Cleanup.

- **v0.1.720** — `/cpu` · `/gpu` · `/freq` · `/temp` instant — ring chips at menu-bar amber (CPU≥50% · GPU≥15% · Freq≥3.5 GHz · Temp≥70°C; Discord + AI Chat); does not steal `/rings`.

- **v0.1.719** — `/battery` · `/bat` · `/heat` · `/thermal` · `/lpm` instant — power-strip Bat · Heat · LPM chips (Bat≤20% · Heat Fair+ · LPM On hot; Discord + AI Chat); does not steal `/strip`.

- **v0.1.718** — `/details` · `/details hot` · `/load` instant — Details Load · RAM · Up (Load≥4 · RAM≥85% hot; Discord + AI Chat); ring keyboard hint no longer mentions removed All · Hot chips.

- **v0.1.716** — `/strip` · `/strip hot` · `/power` instant — power strip Hot list (menu-bar amber / attention; Discord + AI Chat); Discord `/rings` wired.

- **v0.1.715** — `/rings` · `/rings hot` instant — CPU · GPU · Freq · Temp Hot list (menu-bar amber thresholds; Discord + AI Chat; UI rings filter parity).

- **v0.1.714** — `/processes pinned` · `/pinned` instant + pin sync to `~/.mac-stats/pinned_processes.json` (Discord + AI Chat; UI Pinned filter parity).

- **v0.1.713** — `/perplexity` instant operator (top · snippet; Discord + AI Chat). Cache: perplexity_last.json.

- **v0.1.712** — `/processes` instant operator (hot; Discord + AI Chat). Pinned added in **v0.1.714**.

- **v0.1.711** — `/logs` instant operator (`/logs error` · `/logs warn`) — Debug Log Error/Warn list (Discord + AI Chat; p50 latency).

- **v0.1.710** — `/disk` instant operator (`/disk on` · `/disk off` · `/disk reclaim` · `/disk big` · `/disk clean`) — Disk Cleanup scopes/categories list (Discord + AI Chat; p50 latency).

- **v0.1.709** — `/monitors` instant operator (`/monitors up` · `/monitors down` · `/monitors slow`) — External / Monitors Up/Down/Slow list (Discord + AI Chat; p50 latency).

- **v0.1.708** — `/schedules` Jobs/Deliveries filter (`/schedules jobs` · `/schedules deliveries`) — Agent Ops Schedules parity (Discord + AI Chat; p50 latency).

- **v0.1.707** — `/knowledge` instant operator (`/knowledge discord` · `/knowledge core`) — Agent Ops Discord/Core list (Discord + AI Chat; p50 latency).

- **v0.1.706** — `/sessions` instant operator (`/sessions live` · `/sessions files`) — Agent Ops Live/Files list (Discord + AI Chat; p50 latency).

- **v0.1.705** — `/agents` instant operator (`/agents on` · `/agents off`) — Agent Ops On/Off list (Discord + AI Chat; p50 latency).

- **v0.1.704** — `/lite` instant operator (`lite runs`, `lite lane`) + Agent Ops Runs Lite filter (Instant/Direct parity; p50 latency).

- **v0.1.703** — Having_fun idle-thought Discord send retry + skip session memory on failed send + rate-limited timeout WARN (`debug.log` / Discord reliability).

- **v0.1.697** — `/instant` + `/direct` instant operators (`instant runs`, `direct runs`) — lane-filtered turns from runs.jsonl (Discord + AI Chat; Agent Ops Instant/Direct parity; p50 latency).

- **v0.1.696** — `/slow` instant operator (`what's slow`, `slow runs`) — ok turns ≥2000 ms from runs.jsonl (Discord + AI Chat; Agent Ops Slow parity; p50 latency).

- **v0.1.695** — `/failed` instant operator (`what failed`, `failed runs`) — ok=false turns from runs.jsonl with error text (Discord + AI Chat; Agent Ops Fail parity; p50 latency).

- **v0.1.694** — Agent Ops Runs Fail filter (ok=false; red row wash; overview Opens → Fail; AI Chat Errors parity).

- **v0.1.693** — Agent Ops Runs All · Instant · Direct · Slow filter (Slow ≥2000 ms; amber row wash; overview Opens → Slow; Monitors Slow / p50 parity).

- **v0.1.692** — Perplexity Search All · Top · Snippet filter (Top = first 3; Snippet = preview text; last-search glance opens Top when >3; design review grace).

- **v0.1.691** — Disk Cleanup categories All · Reclaim · Big · Clean filter (Big ≥50 MiB; Monitors Slow / Top Processes Hot parity; design review / `feature-disk-cleanup`).

- **v0.1.690** — Disk Cleanup scopes All · On · Off filter (Agents On/Off parity; design review / `feature-disk-cleanup`).

- **v0.1.689** — CPU rings All · Hot filter (menu-bar amber thresholds; history charts follow; design review / `feature-cpu-metrics`).

- **v0.1.688** — AI Chat All · You · Assistant · Errors filter (failed turns `Error: …`; Monitors Slow / Top Processes Hot parity; design review / `feature-ai-chat`).

- **v0.1.687** — Monitors All · Up · Down · Slow filter (UP ≥2000 ms menu-bar Mon amber; Top Processes Hot parity; design review / `feature-monitors`).

- **v0.1.686** — Top Processes All · Pinned · Hot filter (glance amber thresholds; design review / `feature-processes`).

- **v0.1.685** — AI Chat keep-header restored (collapsed Ready/turns glance stays visible; Monitors / Disk Cleanup / Debug Log / Perplexity / Agent Ops parity).

- **v0.1.684** — Monitors keep-header restored (collapsed up/down glance stays visible; Disk Cleanup / Debug Log / Perplexity / Agent Ops parity).

- **v0.1.683** — Agent Ops keep-header restored + Discord Ready glance↔icon↔footer toolbar chain (collapsed glance stays visible; Debug Log / Disk Cleanup / Perplexity parity).

- **v0.1.682** — Debug Log keep-header restored + collapsed glance↔icon↔footer toolbar chain (Quiet/error/warn glance stays visible; Perplexity / Disk Cleanup parity).

- **v0.1.681** — Disk Cleanup keep-header restored (collapsed glance stays visible; Perplexity parity; compact still full-hide).

- **v0.1.680** — Perplexity collapsed glance↔icon↔footer toolbar chain (keep-header restored; icon ↓ → glance when collapsed; glance ↑ → icon · ↓ → footer; footer ↑ → glance).

- **v0.1.679** — AI Chat collapsed glance↔icon↔footer toolbar chain (icon ↓ → glance when collapsed; glance ↑ → icon · ↓ → footer; footer ↑ → glance).

- **v0.1.678** — Disk Cleanup collapsed glance↔icon↔footer toolbar chain (icon ↓ → glance when collapsed; glance ↑ → icon · ↓ → footer; footer ↑ → glance).

- **v0.1.677** — Monitors collapsed glance↔icon↔footer toolbar chain (icon ↓ → glance when collapsed; glance ↑ → icon · ↓ → footer; footer ↑ → glance).

- **v0.1.676** — Top Processes collapsed glances↔Details↔footer toolbar chain (first glance ↑ → Details; last ↓ → footer; footer ↑ → last glance).

- **v0.1.675** — Power strip↔Details collapsed glance toolbar chain (strip last ↓ → glance; glance ↑ → strip · ↓ → processes; keep-header collapse shows glance).

- **v0.1.674** — History sparkline↔Top Processes filter toolbar chain (last chart ↓ → first filter chip; filter first ↑ → last chart).

- **v0.1.673** — Ring gauge↔Top Processes filter toolbar chain (temperature ring ↓ → first filter chip; filter first ↑ → temperature ring).

- **v0.1.672** — Details↔ring gauge toolbar chain (first value ↑ → temperature ring; ring first ↑ / last ↓ → Details first).

- **v0.1.671** — Settings Credentials↔header toolbar chain (CPU header Settings ↓ → Discord token; token ↑ → Settings; Perplexity key ↑ → icon) + Top Processes filter↔Refresh chain.

- **v0.1.670** — AI Chat icon-line↔Ollama settings toolbar chain (icon ↓ → system prompt when popover open; prompt first ↑ → icon).

- **v0.1.669** — Perplexity icon-line↔Settings toolbar chain (icon ↓ → API key when Settings open; key first ↑ → icon).

- **v0.1.668** — Discord icon-line↔Settings toolbar chain (icon ↓ → token when Settings open; token ↑ → icon).

- **v0.1.667** — Agent Ops icon-line↔section toolbar chain (icon ↓ → health / refresh / tabs; health first ↑ → icon).

- **v0.1.666** — Disk Cleanup icon-line↔section toolbar chain (icon ↓ → filter chips / meta / scopes / categories; first chip ↑ → icon).

- **v0.1.664** — Monitors icon-line↔section toolbar chain (icon ↓ → settings close / filter chips / list; first chip or settings close ↑ → Monitors icon).

- **v0.1.663** — Debug Log icon-line↔toolbar chain (icon ↓ → Refresh when open; Refresh first ↑ → icon when viewer empty).

- **v0.1.662** — Perplexity icon-line↔setup toolbar chain (icon ↓ → inline key when setup open; key first ↑ → icon).

- **v0.1.659** — Perplexity setup↔footer toolbar chain (Save key → footer; footer ← setup when inline API-key panel open).

- **v0.1.658** — Details section↔footer toolbar chain (last value ↓ → processes first / footer; processes first ↑ → Details last; footer ← Details last when processes collapsed).

- **v0.1.657** — Top Processes force-quit↔footer toolbar polish (footer ← Force Quit when modal open; list last ↓ → hero; section↔footer chain lands on Force Quit).

- **v0.1.656** — Top Processes list↔modal hero toolbar chain (selected row ↓ → name; name ← row; Force Quit → footer).

- **v0.1.655** — Monitors add-form↔settings-list toolbar chain (last Remove/CTA ↓ → URL; URL ← list; Add Monitor → footer; footer ← when Settings open).

- **v0.1.654** — Monitors list↔detail toolbar chain (open row ↓ → Check now; Check now ← row; Remove → footer; footer ← Remove when detail open).

- **v0.1.653** — Debug Log lines↔toolbar chain (last line ↓ → Refresh; Refresh ← last line; Auto-refresh → footer; footer ← Auto-refresh when logs open).

- **v0.1.652** — Perplexity results↔search toolbar chain (last result ↓ → query; query ← last result; Search → footer; footer ← Search).

- **v0.1.651** — AI Chat messages↔composer toolbar chain (last message ↓ → composer; composer ← last message; Send → footer; footer ← Send when chat open).

- **v0.1.650** — AI Chat empty starter chips toolbar (warm title + ←→/Enter → composer; composer ← chips).

- **v0.1.647** — Settings Help cheat sheet Product toolbar chain (open sheet between Help · Reset; Esc closes; Enter/c copies).

- **v0.1.646** — Discord settings full modal toolbar wrap (footer version ↔ Discord token toolbar when Settings open).

- **v0.1.645** — Perplexity settings full modal toolbar wrap (footer version ↔ API key toolbar when Settings open).

- **v0.1.644** — Ollama settings full modal toolbar wrap (footer version ↔ system-prompt when popover open).

- **v0.1.643** — Process Details full modal toolbar wrap (footer version ↔ force-quit when modal open).

- **v0.1.642** — Changelog full modal toolbar wrap (footer version ↔ changelog body when modal open).

- **v0.1.641** — Settings full modal toolbar wrap (CPU header Settings ↔ Appearance when modal open; ring chain when closed).

- **v0.1.640** — Section-content↔footer toolbar chain (last list row → footer; footer ← last row / Disk Cleanup toolbar).

- **v0.1.639** — Disk Cleanup scopes↔categories toolbar chain (last scope → first category; first category ↑ → scopes/filter chips; last category → action toolbar; empty scopes skip to categories from filter chips).

- **v0.1.638** — Filter-chip↔section-content toolbar chain (last chip → list; first row ↑ → chips; processes/monitors/chat/logs/disk/Agent Ops).

- **v0.1.637** — CPU metrics icon-line↔filter-chip toolbar chain (last icon → first filter chip; last chip → footer).

- **v0.1.636** — CPU metrics power-strip↔icon-line toolbar chain (last strip chip → first section icon; first icon ← last strip chip).

- **v0.1.635** — CPU metrics history sparkline↔power-strip toolbar chain (last chart → battery strip; first strip ← last chart).

- **v0.1.634** — CPU metrics history sparkline↔ring toolbar chain (last ring → first chart; first chart → temperature ring).

- **v0.1.633** — CPU window **header toolbar keyboard** (Refresh · Settings ←→/h l/Home/End; ring-gauge + footer wrap chain).

- **v0.1.632** — CPU window **footer toolbar keyboard** (version · GitHub ←→/h l/Home/End; icon-line wrap chain).

- **v0.1.627 bundle** — LPM strip toggle (`toggle_low_power_mode`); GPU history sparkline + live Y-scale; icon-line section persistence; Agent Ops fully hidden when closed; changelog body↔header toolbar wrap.

- Changelog **header↔body toolbar chain** — **v0.1.626**

- Process Details **hero↔force-quit toolbar chain** — **v0.1.624**

- Settings **Credentials↔header toolbar wrap** — **v0.1.625**

- Process Details **header↔hero toolbar chain** — **v0.1.623**

- Settings **header↔Appearance toolbar chain** — **v0.1.622**

- Settings **section toolbar chain** (Appearance·Product·Credentials) — **v0.1.621**

- Settings **Credentials section toolbar keyboard** — **v0.1.620**

- Changelog + Process Details **modal header toolbar keyboard** — **v0.1.619**

- Settings **Appearance toolbar keyboard** (theme list + window frame ← → / h l · Home/End; header toolbar parity) — **v0.1.618**

- Settings **close/header toolbar keyboard** (Settings title · Close ← → / h l · Home/End; Perplexity key toolbar parity) — **v0.1.617**

- Settings **Perplexity API key toolbar keyboard** (key input · Save · Clear ← → / h l · Home/End; Discord settings parity) — **v0.1.616**

- Ollama settings **toolbar keyboard** (close · system prompt · Reset · Save ← → / h l · Home/End; Discord settings parity) — **v0.1.615**

- Top Processes **force-quit toolbar keyboard** (Advanced summary · Force Quit ← → / h l · Home/End; detail hero parity) — **v0.1.614**

- Top Processes **detail hero toolbar keyboard** (← → / h l · Home/End across name · PID; Enter/Space copies; Disk Cleanup add-scope parity) — **v0.1.613**

- Disk Cleanup **add-scope toolbar keyboard** (← → / h l · Home/End across label · path · days · Recursive · Add scope; Monitors add-form parity) — **v0.1.612**

- Monitors **add-form toolbar keyboard** (← → / h l · Home/End across URL · Cancel · Add Monitor; Discord settings toolbar parity) — **v0.1.611**

- Discord settings **toolbar keyboard** (← → / h l · Home/End across token · Save · Clear · View logs; Monitors detail action toolbar parity) — **v0.1.610**

- Monitors **detail action toolbar keyboard** (← → / h l · Home/End across Check now · Remove; Disk Cleanup action toolbar parity) — **v0.1.609**

- Disk Cleanup **action toolbar keyboard** (← → / h l · Home/End across Clean now · Refresh · Save scopes; Debug Log / meta-card parity) — **v0.1.608**

- Debug Log **toolbar keyboard** (← → / h l · Home/End across Refresh · Open in editor · Auto-refresh; Space toggles; refresh-row parity) — **v0.1.607**

- Perplexity Search **toolbar keyboard** (← → / h l · Home/End across query · Search; setup key · Save key; composer parity) — **v0.1.606**

- AI Chat **composer toolbar keyboard** (← → / h l · Home/End across input · Clear · Send; arrows at text boundaries; filter-row parity) — **v0.1.605**

- Agent Ops **Runs Insights toolbar keyboard** (← → / h l · Home/End across Discord · Digest open · Slowest · Candidates; arrow previews run; Enter loads chat; filter-row / preview-row parity) — **v0.1.604**

- Agent Ops **filter-row toolbar keyboard** (← → / h l · Home/End across search input · N/M chip · Clear; arrows at text boundaries; preview-row / refresh-row parity) — **v0.1.603**

- Agent Ops **preview-row toolbar keyboard** (← → / h l · Home/End across copy chip · Load into AI Chat on Sessions / Runs / Schedules / Knowledge previews; edit-actions parity) — **v0.1.602**

- Agent Ops **agent edit-actions toolbar keyboard** (← → / h l · Home/End across Save · Load into AI Chat · Back; Enter/Space activates; file-tab / refresh-row parity) — **v0.1.601**

- Agent Ops **file-tab toolbar keyboard** (← → / h l · Home/End across Soul · Skill · Mood; Enter/Space activates; tab-bar / refresh-row parity) — **v0.1.600**

- Agent Ops **refresh-row toolbar keyboard** (← → / h l · Home/End across Refresh · Refresh digest · Updated; Enter/Space activates; Updated Enter triggers full refresh; tab-bar / health-strip parity) — **v0.1.599**

- Agent Ops **tab-bar toolbar keyboard** (← → / h l · Home/End across 0 Overview · Agents · Sessions · Schedules · Knowledge · Runs; Enter/Space opens tab; overview-card / health-strip parity) — **v0.1.598**

- Agent Ops **overview-card toolbar keyboard** (← → / h l · Home/End across Agents · Schedules · Live · Knowledge · Recent · Runs · Digest; Enter/Space opens linked tab; health-strip / power-strip parity) — **v0.1.597**

- Agent Ops **health-strip toolbar keyboard** (← → / h l · Home/End across Version · Discord · Redmine · Schedule · Delivery · Digest; Enter/Space opens tab/preview; Disk Cleanup meta-card / power-strip parity) — **v0.1.596**

- Disk Cleanup **meta-card toolbar keyboard** (← → / h l · Home/End across Reclaim · Next · Runs when · Scopes; Enter/Space activate; power-strip / filter-chip parity) — **v0.1.595**

- CPU metrics **history sparkline toolbar keyboard** (← → / h l · Home/End across CPU · Freq · Temp; Enter/Space/click → matching ring; ring-gauge / power-strip parity) — **v0.1.594**

- CPU metrics **ring-gauge toolbar keyboard** (← → / h l · Home/End across CPU · GPU · Frequency · Temperature; Enter/Space activates/copies; power-strip / filter-chip parity) — **v0.1.593**

- Settings Product **toolbar keyboard** (← → / h l · Home/End across AI · compact menu bar · compact CPU window · Help · Reset; Space toggles; theme-list / filter-chip parity) — **v0.1.592**

- Settings theme list **toolbar keyboard** (← → / h l · Home/End across themes; Enter/Space applies; filter-chip / power-strip parity) — **v0.1.591**

- Filter-chip **toolbar keyboard** (← → / h l · Home/End across All·… chips; Monitors · Processes · Logs · Disk · Chat · Ops; power-strip / icon-line parity) — **v0.1.590**

- Section **icon-line toolbar keyboard** (← → / h l · Home/End across Monitors · AI Chat · Perplexity · Debug Log · Discord · Disk Cleanup · Agent Ops; Enter/Space opens; AI-off skipped; power-strip parity) — **v0.1.589**

- Battery / power strip **toolbar keyboard** (← → / h l · Home/End across chips; Enter/Space activate/copy; Details / Monitors listbox chrome parity) — **v0.1.588**

- Details **click-to-copy + keyboard nav** (↑↓ / j k / Home / End · Enter/c copy · Esc; green Copied badge; listbox chrome first/last; Debug Log / Monitors / Top Processes parity) — **v0.1.587**

- AI Chat **listbox chrome keyboard** (focus messages → ↑↓ / j k / Home / End first/last; Perplexity / Monitors / Debug Log parity) — **v0.1.586**

- Perplexity Search **listbox chrome keyboard** (focus results → ↑↓ / j k / Home / End first/last; Monitors / Debug Log / Top Processes parity) — **v0.1.585**

- Agent Ops **row Copied badge + first/last chrome** (`c` / chip · green Copied badge; ↑/k no-selection → last; Monitors / Top Processes parity) — **v0.1.584**

- Top Processes **listbox chrome keyboard** (focus list → ↑↓ / j k / Home / End first/last; Monitors / Disk Cleanup / Debug Log chrome parity) — **v0.1.583**

- External / Monitors **row Copied wash + listbox chrome keyboard** (click URL / `c` · green Copied badge on row; ↑↓ / j k from listbox → first/last; Top Processes / Disk Cleanup parity) — **v0.1.582**

- Top Processes **row Copied wash** (click name / `c` · green Copied badge on row; name button still flashes; Disk Cleanup / Debug Log / Perplexity / AI Chat parity) — **v0.1.581**

- Disk Cleanup **row Copied flash + listbox chrome keyboard** (c / path click · green Copied wash; ↑↓ / j k from listbox → first/last; Debug Log / Perplexity parity) — **v0.1.580**

- Debug Log **line keyboard nav** (↑↓ / j k · Enter/c copy · Esc; ERROR/WARN tint; Monitors / Perplexity / AI Chat parity) — **v0.1.579**

- Perplexity Search **result keyboard nav** (↑↓ / j k · Enter opens · c copies URL · Esc; Monitors / AI Chat / Top Processes listbox parity) — **v0.1.578**


- AI Chat **message keyboard nav** (↑↓ / j k · selected wash · Esc · c copy; Monitors / Top Processes listbox parity) — **v0.1.577**
- AI Chat **click-to-copy on messages** (Copied flash; drag-select safe; last-answer / processes / monitors parity) — **v0.1.576**
- Menu bar **Mon** amber cue when any UP monitor responds ≥ 2000 ms (Monitors summary slowest / latency parity; Mon ✕ still wins on DOWN) — **v0.1.575**
- Menu bar **Bat** amber cue when cached battery ≤ 20% and not charging (power-strip battery is-low parity) — **v0.1.574**
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
