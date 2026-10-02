# Morning surprise — 2026-10-02

Overnight Track B kept filter counts calm before a load.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1295** | Perplexity Top and Snippet counts stay **None yet** when the count is zero. The chips no longer flash **0** before a load. A positive count still replaces **None yet**. |
| **v0.1.1294** | Debug Log Error and Warn counts stay **None yet** when the count is zero. The chips no longer flash **0** before a load. A positive count still replaces **None yet**. |
| **v0.1.1293** | Disk Cleanup Reclaim, Big, and Clean counts stay **None yet** when the count is zero. The chips no longer flash **0** before a load. A positive count still replaces **None yet**. |
| **v0.1.1292** | Monitors Up, Down, and Slow counts stay **None yet** when the count is zero. The chips no longer flash **0** before a load. A positive count still replaces **None yet**. All themes. |
| **v0.1.1291** | Top Processes Pinned and Hot counts stay **None yet** when the count is zero. The chips no longer flash **0** before a load. A positive count still replaces **None yet**. All themes. |
| **v0.1.1290** | Agent Ops kind-filter counts stay **None yet** when the count is zero. On/Off, Live/Files, Jobs/Deliveries, Discord/Core, and Runs lanes no longer flash **0** before a load. A positive count still replaces **None yet**. All themes. |

## Context

- Digester open stayed empty (instant/direct noise filtered).
- Design review `due=false` (grace); recommended surface still `feature-agent-ops` (~16.6d).
- Debug.log: no product ERROR/WARN/panic clusters in the scan window.
- ~20:40 tick: kind-filter counts stay None yet at zero (**v0.1.1290**); install/kickstart.
- ~23:15 tick: Top Processes Pinned/Hot counts stay None yet at zero (**v0.1.1291**); install/kickstart.
- ~23:40 tick: Monitors Up/Down/Slow counts stay None yet at zero (**v0.1.1292**); install/kickstart.
- ~00:05 tick: Disk Cleanup Reclaim/Big/Clean counts stay None yet at zero (**v0.1.1293**); install/kickstart.
- ~00:32 tick: Debug Log Error and Warn counts stay None yet at zero (**v0.1.1294**); install/kickstart.
- ~01:02 tick: Perplexity Top and Snippet counts stay None yet at zero (**v0.1.1295**); install/kickstart.
- GitHub Release **v0.1.1293** cut (20 patches since v0.1.1273). CI attaches the DMG.

## Next fuel

- AI Chat You, Assistant, and Errors filter counts still say **0** until a load. Disk Cleanup scope On/Off counts do the same.
- Disk Cleanup screenshot refresh when Screen Recording TCC allows (`feature-disk-cleanup.png`).
- Sibling ports if digester stays empty.
