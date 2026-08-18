# Morning surprise — 2026-08-18

Overnight Track B shipped Agent Ops Command Center polish, list copy-id/name/URL, Disk Cleanup path copy, CPU metrics ring-value copy (then restored CPU % Details toggle), AI Chat starter chips, Debug Log Error/Warn filters, battery/power strip click-to-copy, Monitors summary click + empty Add CTA, and Agent Ops empty Open AI Chat.

## Shipped (so far)

| Version | What got better |
|---------|-----------------|
| **v0.1.519** | Agent Ops **empty Open AI Chat** — Live, saved sessions, and Runs true-empty lists show **Open AI Chat** (expands the composer; Monitors empty Add CTA parity) |
| **v0.1.518** | External / Monitors **summary click + empty Add CTA** — click (or Enter/Space) the summary to open the first DOWN site (or first row); empty list shows “Nothing watching yet” with **Add a monitor** |
| **v0.1.517** | Battery / power strip **click-to-copy** — click (or Enter/Space) battery % or power watts; Copied overlay so live refresh keeps updating the number (GPU/GHz/°C parity) |
| **v0.1.516** | CPU gauge click again **toggles Details / Top Processes** — overnight ring copy on CPU % had stolen that control; copy stays on GPU, frequency, temperature, battery, and power |
| **v0.1.515** | Debug Log **Error / Warn filter chips** — All, Error, and Warn in the toolbar (live counts; indented follow-up lines stay with the match); empty filter says nothing is there yet |
| **v0.1.514** | AI Chat empty-state **starter chips** — click a prompt to fill the composer, then Send or Enter (Load into AI Chat parity; no auto-send) |
| **v0.1.513** | CPU metrics **click-to-copy value** — click (or Enter/Space) a ring value (GPU %, frequency, temperature) to copy; Copied overlay so live refresh keeps updating the number |
| **v0.1.512** | Disk Cleanup **click-to-copy path** — scope or category path copies on click or `c` (Top Processes / Monitors / Agent Ops parity); Copied flash |
| **v0.1.511** | Monitors **c copies URL** — selected row copies monitor URL (click-to-copy parity; Top Processes / Agent Ops); Copied flash |
| **v0.1.510** | Agent Ops **c copies id** — selected row or preview chip copies agent slug / session id / schedule id / knowledge path / run request id; Copied flash (Top Processes name-copy parity) |
| **v0.1.509** | Top Processes **click-to-copy name** — list name or `c` copies the process name (PID / monitor-URL parity); details hero name copies too; Copied flash |
| **v0.1.508** | Agent Ops **0 Overview** jump: tab-strip control + digit 0 scrolls health/overview into view after 1–5 detail tabs (accent flash) |
| **v0.1.507** | Agent Ops overview cards: glass **head count pills** (tab-count parity; inventory stays next to the title) |
| **v0.1.506** | Agent Ops list filters: glass **Clear** beside the N/M chip when a query is active (Esc parity; works when matches still show) |
| **v0.1.505** | Agent Ops list filters: live **N/M** match chip beside search (Sessions + Schedules combined; ok / partial / zero wash) |
| **v0.1.504** | Agent Ops **Refresh / Updated** under the health strip (stays in view with overview) |
| **v0.1.503** | Agent Ops **tab inventory counts** (glass pills after refresh) |
| **v0.1.502** | Agent Ops **Updated … ago** stamp beside Refresh |

## Prior night (2026-08-17) highlights

- **v0.1.501** tab digit badges · **v0.1.500** overview card click · **v0.1.493–499** overview washes

## Tried / deferred

- Recapture `docs/screens/feature-agent-ops.png` / `feature-cpu-metrics.png` / `feature-ai-chat.png` / `feature-disk-cleanup.png` / `feature-monitors.png` — no on-screen CPU window for Quartz/`screencapture -l` (or Screen Recording TCC). Prior assets kept until a permitted shot.

## Digester

- Open = design-review (stale feature-agent-ops ~6.5d). Latency empty after filters. This tick: Agent Ops empty Open AI Chat CTA (rotate off Monitors summary onto recommended Agent Ops surface).

Updated: 2026-08-18 ~21:00
