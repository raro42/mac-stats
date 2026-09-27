# Morning surprise — 2026-09-27

Overnight Track B kept shipping empty-metric calm on the CPU window, plus overnight disk hygiene.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1265** | Agent Ops Insights card first paint → calm **None yet** (not a blank card). Matches Overview / Runs list empty calm. Design review / `feature-agent-ops`. |
| **v0.1.1264** | Agent Ops Sessions · Schedules · Knowledge · Runs first paint → calm empty titles (Live / files / schedules / deliveries / knowledge / runs — not blank lists). Matches Agents tab. Design review / `feature-agent-ops`. |
| **v0.1.1263** | Agents tab (default) first paint → calm **No agents yet** (not a blank list). Matches Overview empty calm. Design review / `feature-agent-ops`. |
| **v0.1.1262** | Ollama model select + Changelog first paint → **None yet** (not **Loading models…** / **Loading changelog…**). Matches Overview empty calm. Design review / `feature-ai-chat`. |
| **v0.1.1261** | Agent Ops Overview first paint → **None yet** (not pulsing **Loading…**). Schedules / Live / Knowledge / Recent + injected Agents / Runs / Digest. Matches overview empty head pills. Design review / `feature-agent-ops`. |
| **v0.1.1260** | Low Power Mode strip first paint → **None yet** (not **…**). Matches Power / Heat empty-metric calm. Design review / `feature-cpu-metrics`. |
| **v0.1.1259** | Details Load 1m/5m/15m + CPU/GPU Power first paint → **None yet** (not **0.0** / **0.0 W**). Power rows stay calm until a real watt sample. Design review / `feature-cpu-metrics`. |
| **v0.1.1258** | CPU · GPU ring first paint → **None yet** (not **0%**). Matches Temp / Freq empty-metric calm. Design review / `feature-cpu-metrics`. |
| **v0.1.1257** | Perplexity header config + Ollama model label first paint → **Unknown** (not **—**). Model glance ignores Unknown/—/None yet placeholders. |
| **v0.1.1256** | Settings credentials first paint → **Unknown** (not **—**). Discord / Perplexity / Brave / Redmine / Mastodon / MCP / Browser / Cursor / Telegram / Slack. Matches Agent Ops health empty calm. |
| **v0.1.1255** | Disk Cleanup summary / reclaim first paint → **None yet** (not **—**). Monitors Avg → **None yet** (not **Avg -- ms**). Design review / `feature-disk-cleanup` · `feature-monitors`. |
| **v0.1.1254** | Overnight daily Rust `target/` clean (first post-20:00 tick) + Disk Cleanup reclaim of `target/release` at ≥10 GiB. |
| **v0.1.1253** | Temp ring first paint → **None yet** (not **0°C**). Matches Freq empty-metric calm. Design review / `feature-cpu-metrics`. |
| **v0.1.1252** | Chip subtitle empty → **Unknown**. CPU/GPU ring first-paint subtext + Details RAM / uptime → **None yet**. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~11d); polish grace remade after **v0.1.1265**.
- Debug.log: no ERROR/WARN/panic clusters in the scan window.
- ~06:00 tick: Insights card first-paint keep (**v0.1.1265**); install/kickstart.
- ~05:36 tick: Sessions / Schedules / Knowledge / Runs list first-paint keep (**v0.1.1264**).
- ~05:15 tick: Agents tab first-paint keep (**v0.1.1263**).
- ~04:50 tick: Ollama model select / Changelog first-paint keep (**v0.1.1262**).
- ~04:25 tick: Agent Ops Overview first-paint keep (**v0.1.1261**).
- ~03:50 tick: LPM strip first-paint keep (**v0.1.1260**).
- ~03:20 tick: Details load / power first-paint keep (**v0.1.1259**).
- ~02:40 tick: CPU/GPU ring first-paint keep (**v0.1.1258**).
- ~02:10 tick: Perplexity / Ollama model first-paint keep (**v0.1.1257**).
- ~01:45 tick: Settings credentials first-paint keep (**v0.1.1256**).
- ~01:20 tick: Disk Cleanup / Monitors first-paint keep (**v0.1.1255**).
- Earlier same night: Bat / Temp / Freq / Power / strip / Details glance / Top GPU / Monitors / Disk Cleanup / Top Processes / AI Chat empty calm through **v0.1.1242–1254**.

## Next fuel

- Remaining first-paint leftovers (if any) on Agent Ops / AI Chat.
- Agent Ops screenshot refresh when Screen Recording TCC allows (`feature-agent-ops.png`); polish grace remade after **v0.1.1265**.
- Sibling ports if digester stays empty.
