# Morning surprise — 2026-10-15

## Latest ship — v0.1.1990 (~21:45)

**Keep:** Shared Process Details hero toolbar keyboard hint (`.process-detail-hero-toolbar-kb-hint`) — type mixes against an opaque wash (`color-mix` 63% `#0c0c10` on `#ffffff`; Data Poster / Dark 66% `#ffffff` on `#0e0e14`). No glass `opacity` on the name · PID move hint.

**Why:** WebView idle cut (#14). Glass alpha on the Process Details hero toolbar keyboard hint keeps Graphics and Media busy (non-Apple themes; Apple shell already opaque).

**Fuel:** Digester empty; design-review due=false (grace; recommended feature-agent-ops ~29.1d); layout-daily due=false (data-poster ok today; dark still [DUE] screen missing). Standing P2 #14 after v0.1.1989 (`.logs-kb-hint`).

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `4707d123` → push `main`.

**Next:** shared `.process-details-header-kb-hint` (0.72), or `.logs-viewer-prefix` (0.72), or Mac capture when TCC allows.

## Earlier — v0.1.1989 (~21:20)

**Keep:** Shared Debug Log keyboard hint (`.logs-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the log-line move hint.

**Verify:** keep @ `dad51607`.

## Earlier — v0.1.1988 (~20:55)

**Keep:** Shared AI Chat message-list keyboard hint (`.chat-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the message-row move hint.

**Verify:** keep @ `001f1999`.

## Earlier — v0.1.1987 (~20:32)

**Keep:** Shared Perplexity results keyboard hint (`.perplexity-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the result-row move hint.

**Verify:** keep @ `3d42657f`.

## Earlier — v0.1.1986 (~20:08)

**Keep:** Shared Details keyboard hint (`.details-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the Details value move hint.

**Verify:** keep @ `2e8132a6`.
