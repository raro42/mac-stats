# Overnight standing backlog (Track B fuel)

When digester **open** is empty, the overnight harness **must** pull from this list (top first). Cross out or move completed items down after a keep. Local overrides/merges live in `~/.mac-stats/improvements/standing_backlog.md` (merge, do not overwrite).

## P0 — latency / thrash

1. ~~**Improve/memory scheduled task thrash**~~ — done in **v0.1.260** (runner prompt compaction).
2. **p50 direct latency** — digester excludes Improve-task thrash (**v0.1.270**); overnight / how-solved-task asks instant (**v0.1.271**); bare news + topic-dump pre-route (**v0.1.273**); exact saved-note reads instant (**v0.1.274**); dump-what-you-saved instant (**v0.1.275**); TASK_CREATE pre-route (**v0.1.276**); ship-version instant fix + lighthouse/pagespeed pre-route (**v0.1.277**); last-task / what-needs instant (**v0.1.278**); Perplexity NL + last-task clarifier (**v0.1.279**); exact-plan note extract (**v0.1.280**); Google SERP → search rewrite (**v0.1.281**); clear flight route asks pre-route (**v0.1.282**); vague follow-up clarifiers instant (**v0.1.283**); airport-hop + bare research pre-route (**v0.1.284**); itinerary-correction instant (**v0.1.286**); event-date reviews pre-route (**v0.1.292**); itinerary preference instant (**v0.1.293**); multi-city travel-plan pre-route (**v0.1.294**); digester Slowest filters for those three (**v0.1.295**); morning-surprise table highlights + product changelog instant (**v0.1.374**); scheduled `SKILL:` weekly reviews filtered from Slowest/p50 (**v0.1.377**); more tool-heavy patterns remain.
3. **Overnight design review** — Follow `docs/043_overnight_design_review.md`. Prefer stale feature screens (`feature-agent-ops`, `feature-ai-chat`, `feature-processes`) before re-shooting CPU. Digest empty-state polish in **v0.1.276**; refresh-button polish in **v0.1.278**; process-list polish in **v0.1.280** / keyboard+focus in **v0.1.298**; Agent Ops tab hover/focus in **v0.1.285**; Ops filter focus ring in **v0.1.287**; overview card hover in **v0.1.288** / focus-within in **v0.1.317**; list-row hover in **v0.1.296**; AI chat input in **v0.1.295** / composer glass + accents in **v0.1.370**; health-card keyboard nav in **v0.1.316**.
4. ~~**README / landing**~~ — sharper vs-competitor framing (**v0.1.265**).

## P2 — reliability

5. **`debug.log` errors** — First recurring error/panic in the last 24h that is product-owned. Brave health-ping quota burn mitigated in **v0.1.272**. Website monitor DNS/connect failures classify to short reasons in **v0.1.375** (UI + log). DOWN recheck backoff (DNS ≥5 min) in **v0.1.376**. DOWN next-check countdown in UI in **v0.1.377**. Unchanged-UP `monitors.json` rewrite throttle (~5 min) in **v0.1.378**. Unchanged UP/DOWN recheck logs → TRACE in **v0.1.379**. Install refuses stale release binary in **v0.1.380**. Idle task-review scan / no-open → DEBUG when empty in **v0.1.382**. Monitors summary names DOWN hosts + short reasons in **v0.1.383**. Monitor last-check age + DOWN-first list sort + slowest-host summary in **v0.1.384**. Monitors Arrow/Home/End + Enter check-now in **v0.1.385**. Disk Cleanup scope keyboard in **v0.1.386**; category keyboard + Enter Clean now in **v0.1.387**; Delete/Backspace removes custom scopes in **v0.1.388**; Enter-to-add + ⌘/Ctrl+S save in **v0.1.389**; `R` toggles Recurse in **v0.1.390**; `T` toggles Trash soft-delete in **v0.1.391**.
6. **Discord / LaunchAgent uptime** — Confirm process + Discord Ready after any install; fix silent downtime causes. Single-instance busy WARN rate-limit (**v0.1.381**) cuts KeepAlive thrash noise in `debug.log`.

## P3 — sibling ports

7. OpenClaw / Hermes ports that clearly map to mac-stats tools/sessions (not docs-only Related sections). Google SERP FETCH_URL→search rewrite shipped in **v0.1.281**. Insights/status/digest/schedules/scrub/`/help`/interrupt NL in **v0.1.306–315**. Discord voice STT harden in **v0.1.313**. Climate/clima/klima → Open-Meteo + Brave-weather→Perplexity redirect in **v0.1.319–321**.

## Done recently (do not re-pick as filler)

- Disk Cleanup keyboard: scopes / categories / Delete custom / Enter-to-add + ⌘S / R recurse / T soft-delete (**v0.1.386–391**)
- Instant lanes: version, thread clarifier, weather Open-Meteo, short ack, …
- Menu bar SSD + `MEMORY: save` verbatim notes
- Digester filters for travel/SEO and scheduled SKILL (meta — not a night’s sole win)
