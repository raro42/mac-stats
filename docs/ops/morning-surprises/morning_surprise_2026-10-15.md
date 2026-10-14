# Morning surprise — 2026-10-15

## Latest ship — v0.1.1986 (~20:08)

**Keep:** Shared Details keyboard hint (`.details-kb-hint`) — type mixes against an opaque wash (`color-mix` 63% `#0c0c10` on `#ffffff`; Data Poster / Dark 66% `#ffffff` on `#0e0e14`). No glass `opacity` on the Details value move hint.

**Why:** WebView idle cut (#14). Glass alpha on the Details keyboard hint keeps Graphics and Media busy (non-Apple themes; Apple shell already opaque).

**Fuel:** Digester empty; design-review due=false (grace; recommended feature-agent-ops ~29.0d); layout-daily due=false (data-poster ok today; dark still [DUE] screen missing). Standing P2 #14 after v0.1.1985 (`.processes-kb-hint`).

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `2e8132a6` → push `main`.

**Next:** shared `.perplexity-kb-hint` (0.72), or `.chat-kb-hint` (0.72), or `.logs-kb-hint` (0.72), or Mac capture when TCC allows.
