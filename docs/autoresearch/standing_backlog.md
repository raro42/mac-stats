# Overnight standing backlog (Track B fuel)

When digester **open** is empty, the overnight harness **must** pull from this list (top first). Cross out or move completed items down after a keep. Local overrides/merges live in `~/.mac-stats/improvements/standing_backlog.md` (merge, do not overwrite).

## P0 — latency / thrash

1. ~~**Improve/memory scheduled task thrash**~~ — done in **v0.1.260** (runner prompt compaction).
    2. **p50 direct latency** — digester excludes Improve-task thrash (**v0.1.270**); overnight / how-solved-task asks instant (**v0.1.271**); bare news + topic-dump pre-route (**v0.1.273**); exact saved-note reads instant (**v0.1.274**); dump-what-you-saved instant (**v0.1.275**); TASK_CREATE pre-route (**v0.1.276**); ship-version instant fix + lighthouse/pagespeed pre-route (**v0.1.277**); last-task / what-needs instant (**v0.1.278**); Perplexity NL + last-task clarifier (**v0.1.279**); exact-plan note extract (**v0.1.280**); Google SERP → search rewrite (**v0.1.281**); clear flight route asks pre-route (**v0.1.282**); vague follow-up clarifiers instant (**v0.1.283**); airport-hop + bare research pre-route (**v0.1.284**); itinerary-correction instant (**v0.1.286**); event-date reviews pre-route (**v0.1.292**); itinerary preference instant (**v0.1.293**); multi-city travel-plan pre-route (**v0.1.294**); digester Slowest filters for those three (**v0.1.295**); morning-surprise table highlights + product changelog instant (**v0.1.374**); scheduled `SKILL:` weekly reviews filtered from Slowest/p50 (**v0.1.377**); AI Chat operator instant lane (`/status` `/insights` `/schedules` `/digest` scrub `/help`) in **v0.1.628**; Agent Ops Runs Slow filter in **v0.1.693**; `/failed` **v0.1.695**; `/slow` **v0.1.696**; `/instant`+`/direct` **v0.1.697**; `/lite` **v0.1.704**; `/agents` On/Off **v0.1.705**; `/sessions` Live/Files **v0.1.706**; `/knowledge` Discord/Core **v0.1.707**; `/schedules` Jobs/Deliveries **v0.1.708**; `/monitors` Up/Down/Slow **v0.1.709**; `/disk` On/Off/Reclaim/Big/Clean **v0.1.710**; `/logs` **v0.1.711**; `/processes` **v0.1.712**; `/perplexity` **v0.1.713**; `/pinned` **v0.1.714**; `/rings` **v0.1.715**; `/strip` **v0.1.716**; `/details` **v0.1.718**; `/battery`·`/heat`·`/lpm` **v0.1.719**; `/cpu`·`/gpu`·`/freq`·`/temp` **v0.1.720**; `/ram`·`/ssd`·`/uptime` **v0.1.721**; `/discord` **v0.1.722**; `/ollama`·`/llm` **v0.1.723**; `/redmine` **v0.1.724**; `/brave` **v0.1.725**; `/perplexity key` **v0.1.726**; `/mastodon` **v0.1.727**; `/mcp` **v0.1.728**; `/cursor`·`/cursor-agent` **v0.1.729**; `/telegram`·`/slack`·`/signal`·`/alerts` **v0.1.730**; `/skills` **v0.1.731**; `/tasks` **v0.1.732**; `/plugins` On/Off **v0.1.733**; `/browser`·`/cdp` **v0.1.734**; `/judge` **v0.1.735**; `/ai`·`/ai-agent` **v0.1.736**; next schedule / next job instant (**v0.1.800**); last delivery instant (**v0.1.801**); schedule/delivery count instant (**v0.1.802**); operator inventory count instant (**v0.1.803**); runs count instant (**v0.1.804**); digest open read-only instant (**v0.1.805**); digest age read-only instant (**v0.1.806**); debug log error/warn count instant (**v0.1.807**); debug log file size instant (**v0.1.808**); debug log path instant (**v0.1.809**); debug log age instant (**v0.1.810**); config path / data home instant (**v0.1.811**); screenshots path instant (**v0.1.812**); more tool-heavy patterns remain.
    3. **Overnight design review** — Follow `docs/043_overnight_design_review.md`. Prefer stale feature screens (`feature-agent-ops`, `feature-ai-chat`, `feature-processes`) before re-shooting CPU. Digest empty-state polish in **v0.1.276**; refresh-button polish in **v0.1.278**; process-list polish in **v0.1.280** / keyboard+focus in **v0.1.298**; Agent Ops tab hover/focus in **v0.1.285**; Ops filter focus ring in **v0.1.287**; overview card hover in **v0.1.288** / focus-within in **v0.1.317**; list-row hover in **v0.1.296**; AI chat input in **v0.1.295** / composer glass + accents in **v0.1.370**; health-card keyboard nav in **v0.1.316**; overview active-tab + health wash in **v0.1.430**; active-tab accent wash + badge glass in **v0.1.431**; selected-row accent wash in **v0.1.447**; Sessions copy id/slug chip in **v0.1.451**; Schedules/deliveries click-to-preview in **v0.1.452**; Knowledge path copy chip in **v0.1.453**; Runs click-to-preview in **v0.1.468**; Agents copy id/slug chip in **v0.1.469**; Data Poster inactive-icon contrast in **v0.1.470**; Runs request-id copy chip in **v0.1.471**; Schedules/delivery id copy chip in **v0.1.472**; Runs Load into AI Chat in **v0.1.473**; Schedules Load into AI Chat in **v0.1.474**; Knowledge Load into AI Chat in **v0.1.475**; Agents Load into AI Chat in **v0.1.476**; overview Schedules click-to-preview in **v0.1.477**; overview Recent click-to-preview in **v0.1.478**; overview Live click-to-preview in **v0.1.479**; overview Knowledge click-to-preview (row select) in **v0.1.480**; overview Last delivery click-to-preview in **v0.1.481**; Runs Insights Slowest/Candidates click-to-preview in **v0.1.482**; health Next schedule / Last delivery click-to-preview in **v0.1.483**; Digest open hints + health Digest click-to-preview in **v0.1.484**; health Version → primary agent open in **v0.1.485**; overview Agents card in **v0.1.488**; overview Runs card in **v0.1.489**; overview Digest card in **v0.1.490**; health schedule/delivery ok/warn/bad wash in **v0.1.491**; health Version ok/warn/bad wash in **v0.1.492**; overview Schedules ok/warn/bad wash in **v0.1.493**; overview Agents ok/warn/bad wash in **v0.1.494**; overview Runs ok/warn/bad wash in **v0.1.495**; overview Digest ok/warn/bad wash in **v0.1.496**; overview Live ok/warn/bad wash in **v0.1.497**; overview Knowledge ok/warn/bad wash in **v0.1.498**; overview Recent ok/warn/bad wash in **v0.1.499**; overview cards click/keyboard open linked tab in **v0.1.500**; tab inventory count pills in **v0.1.503**; Refresh/Updated under health in **v0.1.504**; filter N/M chips in **v0.1.505**; filter-row Clear beside N/M in **v0.1.506**; overview head count pills in **v0.1.507**; 0 Overview jump in **v0.1.508**; Top Processes click-to-copy name in **v0.1.509**; Agent Ops `c` copy id in **v0.1.510**; Monitors `c` URL in **v0.1.511**; Disk Cleanup path copy in **v0.1.512**; CPU ring value copy in **v0.1.513** / CPU % Details toggle restore in **v0.1.516**; AI Chat starter chips in **v0.1.514** / starter chip In composer flash in **v0.1.527**; Debug Log Error/Warn filters in **v0.1.515**; Debug Log error/warn glance in **v0.1.533**; battery/power strip click-to-copy in **v0.1.517**; Monitors summary click + empty Add CTA in **v0.1.518**; Agent Ops empty Open AI Chat in **v0.1.519**; CPU metrics RAM strip in **v0.1.520**; CPU metrics GPU strip in **v0.1.534**; Temp °C strip in **v0.1.535**; frequency GHz strip in **v0.1.536**; SSD % strip in **v0.1.537**; CPU % strip in **v0.1.538**; AI Chat last-answer glance (copy) in **v0.1.539**; Monitors All/Up/Down filter chips in **v0.1.521**; Top Processes All/Pinned filter chips in **v0.1.522** / Hot in **v0.1.686**; Disk Cleanup empty Review scopes CTA in **v0.1.523**; Disk Cleanup Reclaimable now meta-card click in **v0.1.524**; Disk Cleanup Enabled scopes meta-card click in **v0.1.525**; Disk Cleanup Next automatic run meta-card click in **v0.1.526**; Disk Cleanup Runs when meta-card click in **v0.1.528**; Disk Cleanup Last run panel click in **v0.1.529**; Top Processes Top CPU glance in **v0.1.531**; Top Processes Top GPU glance in **v0.1.541**; Top Processes Top RAM glance in **v0.1.547**; AI Chat turn glance in **v0.1.532**; Agent Ops Knowledge All·Discord·Core chips in **v0.1.546**; Disk Cleanup All·Reclaim·Clean chips in **v0.1.548**; AI Chat All·You·Assistant chips in **v0.1.549**; CPU metrics uptime strip in **v0.1.550**; Perplexity last-search glance in **v0.1.551** / All·Top·Snippet filter in **v0.1.692**; Monitors collapsed glance in **v0.1.552**; Disk Cleanup collapsed glance in **v0.1.553**; AI Chat collapsed glance in **v0.1.554**; Agent Ops Discord Ready collapsed glance in **v0.1.555**; Perplexity collapsed keep-header in **v0.1.556**; Debug Log collapsed keep-header in **v0.1.557**; Top Processes collapsed keep-header in **v0.1.558**; Details collapsed keep-header in **v0.1.559**; CPU metrics Heat/thermal strip in **v0.1.560**; Heat prefers **NSProcessInfo.thermalState** in **v0.1.561**; menu-bar LPM in **v0.1.563**; menu-bar Heat Serious/Critical in **v0.1.564**; menu-bar SSD ≥85% amber in **v0.1.567**; menu-bar RAM ≥85% amber + strip hot wash in **v0.1.568**; menu-bar CPU ≥50% amber in **v0.1.569**; menu-bar GPU ≥15% amber in **v0.1.570**; menu-bar Temp ≥70°C amber in **v0.1.571**.
    4. ~~**README / landing**~~ — sharper vs-competitor framing (**v0.1.265**).

## P2 — reliability

5. **`debug.log` errors** — First recurring error/panic in the last 24h that is product-owned. In-app Debug Log Error/Warn filter chips in **v0.1.515**. Brave health-ping quota burn mitigated in **v0.1.272**. Website monitor DNS/connect failures classify to short reasons in **v0.1.375** (UI + log). DOWN recheck backoff (DNS ≥5 min) in **v0.1.376**. DOWN next-check countdown in UI in **v0.1.377**. Unchanged-UP `monitors.json` rewrite throttle (~5 min) in **v0.1.378**. Unchanged UP/DOWN recheck logs → TRACE in **v0.1.379**. Install refuses stale release binary in **v0.1.380**. Idle task-review scan / no-open → DEBUG when empty in **v0.1.382**. Monitors summary names DOWN hosts + short reasons in **v0.1.383**. Monitor last-check age + DOWN-first list sort + slowest-host summary in **v0.1.384**. Monitors Arrow/Home/End + Enter check-now in **v0.1.385**; j/k + Esc clear selection in **v0.1.392**; `d` detail toggle + Esc closes detail first in **v0.1.402**. Disk Cleanup scope keyboard in **v0.1.386**; category keyboard + Enter Clean now in **v0.1.387**; Delete/Backspace removes custom scopes in **v0.1.388**; Enter-to-add + ⌘/Ctrl+S save in **v0.1.389**; `R` toggles Recurse in **v0.1.390**; `T` toggles Trash soft-delete in **v0.1.391**.
6. **Discord / LaunchAgent uptime** — Confirm process + Discord Ready after any install; fix silent downtime causes. Single-instance busy WARN rate-limit (**v0.1.381**) cuts KeepAlive thrash noise in `debug.log`.

## P3 — sibling ports

7. OpenClaw / Hermes ports that clearly map to mac-stats tools/sessions (not docs-only Related sections). Google SERP FETCH_URL→search rewrite shipped in **v0.1.281**. Insights/status/digest/schedules/scrub/`/help`/interrupt NL in **v0.1.306–315**. Discord voice STT harden in **v0.1.313**. Climate/clima/klima → Open-Meteo + Brave-weather→Perplexity redirect in **v0.1.319–321**.

## Done recently

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
