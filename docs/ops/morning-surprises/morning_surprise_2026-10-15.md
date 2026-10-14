# Morning surprise — 2026-10-15

## Latest ship — v0.1.1988 (~20:55)

**Keep:** Shared AI Chat message-list keyboard hint (`.chat-kb-hint`) — type mixes against an opaque wash (`color-mix` 63% `#0c0c10` on `#ffffff`; Data Poster / Dark 66% `#ffffff` on `#0e0e14`). No glass `opacity` on the message-row move hint.

**Why:** WebView idle cut (#14). Glass alpha on the AI Chat message-list keyboard hint keeps Graphics and Media busy (non-Apple themes; Apple shell already opaque).

**Fuel:** Digester empty; design-review due=false (grace; recommended feature-agent-ops ~29.0d); layout-daily due=false (data-poster ok today; dark still [DUE] screen missing). Standing P2 #14 after v0.1.1987 (`.perplexity-kb-hint`).

**Verify:** `autoresearch_ratchet.py verify` OK → keep → push `main`.

**Next:** shared `.logs-kb-hint` (0.72), or `.process-detail-hero-toolbar-kb-hint` (0.72), or Mac capture when TCC allows.

## Earlier — v0.1.1987 (~20:32)

**Keep:** Shared Perplexity results keyboard hint (`.perplexity-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the result-row move hint.

**Verify:** keep @ `3d42657f`.

## Earlier — v0.1.1986 (~20:08)

**Keep:** Shared Details keyboard hint (`.details-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the Details value move hint.

**Verify:** keep @ `2e8132a6`.
