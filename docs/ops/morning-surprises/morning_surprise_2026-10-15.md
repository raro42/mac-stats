# Morning surprise — 2026-10-15

## Latest ship — v0.1.1996 (~00:10)

**Keep:** Shared Settings header keyboard hint (`.settings-header-kb-hint`) — type mixes against an opaque wash (`color-mix` 63% `#0c0c10` on `#ffffff`; Data Poster / Dark 66% `#ffffff` on `#0e0e14`). No glass `opacity` on the title · Close move hint. cpu-ui.js inject matched.

**Why:** WebView idle cut (#14). Glass alpha on the Settings header keyboard hint keeps Graphics and Media busy (non-Apple themes; Apple shell already opaque). Also satisfies layout-daily data-poster (dark remap on `#0e0e14`).

**Fuel:** Digester empty; design-review due=false (grace; recommended feature-agent-ops ~29.17d); layout-daily due=true → data-poster (marked checked). Standing P2 #14 after v0.1.1995 (`.logs-toolbar-kb-hint`).

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `a740cd62` → push `main`.

**Next:** shared `.ollama-settings-header-kb-hint` (0.72), or Mac capture when TCC allows.

## Earlier — v0.1.1995 (~23:42)

**Keep:** Shared Debug Log toolbar keyboard hint (`.logs-toolbar-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the Refresh · Open in editor · Auto-refresh move hint.

**Verify:** keep @ `133fe571`.

## Earlier — v0.1.1994 (~23:20)

**Keep:** Shared Debug Log viewer prefix (`.logs-viewer-prefix`) — opaque wash (63% / 66%). No glass `opacity` on the path/prefix line.

**Verify:** keep @ `f4b98d34`.

## Earlier — v0.1.1993 (~22:55)

**Keep:** Shared Changelog body keyboard hint (`.changelog-body-toolbar-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the version-heading move hint.

**Verify:** keep @ `98cc671e`.

## Earlier — v0.1.1992 (~22:34)

**Keep:** Shared Changelog header keyboard hint (`.changelog-header-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the title · Close move hint.

**Verify:** keep @ `b583a2c0`.

## Earlier — v0.1.1991 (~22:10)

**Keep:** Shared Process Details header keyboard hint (`.process-details-header-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the title · Close move hint.

**Verify:** keep @ `344f8fda`.

## Earlier — v0.1.1990 (~21:45)

**Keep:** Shared Process Details hero toolbar keyboard hint (`.process-detail-hero-toolbar-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the name · PID move hint.

**Verify:** keep @ `4707d123`.

## Earlier — v0.1.1989 (~21:20)

**Keep:** Shared Debug Log keyboard hint (`.logs-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the log-line move hint.

**Verify:** keep @ `dad51607`.

## Earlier — v0.1.1988 (~20:55)

**Keep:** Shared AI Chat message-list keyboard hint (`.chat-kb-hint`) — opaque wash (63% / 66%). No glass `opacity` on the message-row move hint.

**Verify:** keep @ `001f199`.
