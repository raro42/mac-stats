# Morning surprise — 2026-08-14

Overnight Track B (20:00–06:00 local, window opened 2026-08-13).

## Shipped

| Version | What |
|---------|------|
| **v0.1.393** | Disk Cleanup scopes + categories: j/k selection + Esc clears; hints updated |
| **v0.1.370** | Startup Disk Cleanup off main thread (no menu-bar hang); AI Chat composer glass + accents; `MAC_STATS_OPEN_SECTION=ai-chat` |
| **v0.1.371** | Top Processes accent CPU bars + pin ★ header; `OPEN_SECTION=processes` |
| **v0.1.372** | Disk Cleanup reclaim accent + scope/item hover/focus; `OPEN_SECTION=disk-cleanup` |
| **v0.1.373** | External / Monitors summary up/down accents, down/pending rows, clearer latency + history; `OPEN_SECTION=monitors` |
| **v0.1.374** | Instant overnight/wake-up highlights read morning-surprise **table** rows; product self-changelog asks (“Your changelog?”, “Latest enhancements…”) skip Brave/LLM |
| **v0.1.375** | Website monitors classify failures (DNS / timeout / TLS / refused) — short Monitors row + `debug.log` DOWN lines |
| **v0.1.376** | Background monitor recheck backoff while DOWN (DNS ≥5 min / other ≥3 min) — less DNS thrash + quieter `debug.log` |
| **v0.1.377** | DOWN rows show next auto-check countdown; digester/insights ignore scheduled `SKILL:` weekly reviews in Slowest/p50 |
| **v0.1.378** | Unchanged UP monitors rewrite `monitors.json` only on outcome change or ~5 min last_check checkpoint — fewer SSD writes + quieter logs |
| **v0.1.379** | Monitor UP/DOWN `debug.log` only on first check or outcome flip; unchanged rechecks → TRACE |
| **v0.1.380** | Install refuses stale `target/release` (Apps no longer stuck on older binary after source bumps); per-monitor save detail → TRACE |
| **v0.1.381** | Single-instance busy WARN ≤1 / 5 min (stamp file); LaunchAgent KeepAlive retries stay DEBUG |
| **v0.1.382** | Idle task-review `Task scan` / no-open-task → DEBUG (open/wip pressure stays INFO) — quieter per-minute `debug.log` |
| **v0.1.384** | Monitors last-check age on rows; DOWN-first list sort; all-up summary names slowest host + latency/age tooltips |
| **v0.1.385** | Monitors Arrow/Home/End keyboard nav + selected row; Enter/Space check now (bypass DOWN backoff) |
| **v0.1.386** | Disk Cleanup scope Arrow/Home/End keyboard nav + Space toggle enable + hint |
| **v0.1.387** | Disk Cleanup category Arrow/Home/End keyboard nav; Enter Clean now when reclaimable |
| **v0.1.388** | Disk Cleanup Delete/Backspace removes custom scopes (builtins stay) |
| **v0.1.389** | Disk Cleanup Enter adds custom scope + focuses new row; ⌘/Ctrl+S saves scopes |
| **v0.1.392** | Monitors j/k selection + Esc clears selection + keyboard hint |
| **v0.1.391** | Disk Cleanup `T` toggles Trash soft-delete (Move cleaned items to Trash) |
| **v0.1.390** | Disk Cleanup `R` toggles Recurse on the selected scope row |
| **v0.1.383** | Monitors summary names DOWN hosts + short failure reasons (e.g. `DOWN: econsultants.es (DNS)`); tooltip lists all |

## Design review

- Stale screens cleared via polish grace (Screen Recording TCC often blocks live `screencapture -l` from the agent).
- Next due surface: wait for age >3d or re-shoot when TCC allows.

## Digester

- Open candidates: none (Latency n/a after SKILL filter in **v0.1.377**).
- Fuel order: digester empty → design-review not due → standing backlog (p50 / debug.log).

## Tried / notes

- After Monitors j/k+Esc (**v0.1.392**), Disk Cleanup still Arrow-only — j/k + Esc clear in **v0.1.393**.
- After Monitors Arrow/Enter (**v0.1.385**), list lacked j/k + Esc clear — Agent Ops parity in **v0.1.392**.

- After Recurse `R` (**v0.1.390**), soft-delete still needed a mouse click — `T` in Disk Cleanup section in **v0.1.391**.

- After Enter-to-add + ⌘S (**v0.1.389**), Recurse still needed a mouse click on the checkbox — `R` on the selected scope row in **v0.1.390**.

- After Delete for custom scopes (**v0.1.388**), Add still needed a mouse click — Enter in the add form + focus new row + ⌘S save in **v0.1.389**.

- After category keyboard (**v0.1.387**), custom scopes still needed mouse Remove — Delete/Backspace in **v0.1.388**.

- Monitors summary previously showed only `N / M sites up` while econsultants.es stayed DOWN — named in **v0.1.383**.

- econsultants.es monitor DNS failures drove **v0.1.375**–**v0.1.377** (classify, backoff, UI countdown).
- After backoff, UP hosts still flushed disk every minute until **v0.1.378**.
- Single-instance WARN spam when install/kickstart races an already-running app — mitigated in **v0.1.381** (5 min rate limit).
- Bug found earlier tonight: `parse_morning_surprise_bullets` only matched `- **v…**` lists (**v0.1.374**).
- After disk throttle, unchanged DOWN still DEBUG every backoff until **v0.1.379** (TRACE on unchanged).
- Found Apps still on **0.1.376** after source keeps 377–379 — install copied stale release; fixed in **v0.1.380** with version guard + real `cargo build --release`.
- Idle task queue (open=0) still wrote two INFO lines every minute until **v0.1.382**.
- After DOWN naming (**v0.1.383**), rows still hid freshness after backoff/throttle — age + sort in **v0.1.384**.
- After age/sort (**v0.1.384**), list still lacked keyboard nav and UI never called `check_monitor` — fixed in **v0.1.385**.
- After Monitors keyboard (**v0.1.385**), Disk Cleanup scopes still mouse-only — keyboard parity in **v0.1.386**.
- After scope keyboard (**v0.1.386**), category/reclaim list still mouse-only — keyboard + Enter Clean now in **v0.1.387**.
