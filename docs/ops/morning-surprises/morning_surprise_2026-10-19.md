# Morning surprise — 2026-10-19

## Latest ship — v0.1.2097 (~20:05 local 2026-10-18)

**Keep:** Architect Ready · Slow · Down section icon images (`.icon-line-item.status-good img`, `.status-warning img`, `.status-bad img` in Architect `cpu.css`) — drop glass `opacity` 0.85 / 0.8 (hover 0.95 / 0.9) and the colorizing `brightness` / `invert` filters. Those bitmaps paint at full strength on the white chip.

**Why:** WebView idle cut (#14). Glass alpha and multi-step CSS filters on Ready / Slow / Down icons keep Graphics and Media busy.

**Fuel:** Digester open → overnight design review (stale `feature-cpu-metrics.png`, 42.0d). Design review due=true; Mac capture deferred (menu-bar only / no on-screen CPU window). Layout-daily due=true (data-poster). First attempt discarded: duplicate `.processes-kb-hint` after lagging local tree vs origin at v0.1.2096. Standing next after Material resting icons (v0.1.2096): Architect Ready/Slow/Down.

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `5f27522a` → push `main`.

**Discard earlier this tick:** duplicate v0.1.1985 processes-kb-hint (origin already past that cut @ v0.1.2096).

**Next:** Data Poster section icon image filter (`brightness` / `invert`), or Material Ready/Slow/Down, or Swiss Ready/Slow/Down, or Neon/Futuristic resting `opacity()` filters, or Dark resting `opacity` 0.45, or Mac capture of stale `feature-cpu-metrics.png` when TCC allows.
