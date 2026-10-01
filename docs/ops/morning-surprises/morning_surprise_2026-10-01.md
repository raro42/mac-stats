# Morning surprise — 2026-10-01

Overnight Track B kept Disk Cleanup calm before categories load.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1267** | External / Monitors summary first paint → **None yet** (not **0 / 0 sites up**). Real up counts and DOWN lines still show after a sample. All themes. |
| **v0.1.1268** | External / Monitors list first paint → **Nothing watching yet** and **Add a monitor** (not a blank list). Real site rows still replace that empty state after a load. All themes. |
| **v0.1.1269** | Top Processes list first paint → **Waiting for process samples — opens with the CPU window** (not a blank list). Real rows still replace that line after a sample. All themes. |
| **v0.1.1270** | Disk Cleanup category list first paint → **Nothing to reclaim yet** and **Review scopes** (not a blank list). Real category rows still replace that empty state after a load. All themes. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~15.4d).
- Debug.log: no product ERROR/WARN/panic clusters in the scan window.
- ~20:05 tick: monitors summary first-paint keep (**v0.1.1267**); install/kickstart.
- ~20:35 tick: monitors list first-paint keep (**v0.1.1268**); install/kickstart.
- ~21:00 tick: Top Processes list first-paint keep (**v0.1.1269**); install/kickstart.
- ~21:23 tick: Disk Cleanup category list first-paint keep (**v0.1.1270**); install/kickstart.

## Next fuel

- Disk Cleanup scopes list is still blank until JavaScript paints it.
- Agent Ops screenshot refresh when Screen Recording TCC allows (`feature-agent-ops.png`).
- Sibling ports if digester stays empty.
