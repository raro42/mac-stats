# Morning surprise — 2026-10-14

## Shipped — v0.1.1957 (~23:58 CEST)

**Keep:** Shared Agent Ops keyboard hint (`.ops-keyboard-hint`) — type mixes against an opaque near-white wash (`color-mix` 62% ink). No glass `opacity` on the soft keyboard tip line (flash keyframes stay opaque too).

**Why:** WebView idle cut (#14). Glass alpha on soft keyboard tip chrome keeps Graphics and Media busy.

**Fuel:** Digester empty; design review due=false (grace); layout daily due=false. Standing #14 after v0.1.1956 `.ops-filter-caption`.

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `18466d9c` → pushed `main`.

**Next:** `.ops-row-meta` (0.58), or layout-daily data-poster / Mac capture of stale CPU metrics screen.

## Shipped — v0.1.1956 (~23:43 CEST)

**Keep:** Shared Agent Ops filter caption (`.ops-filter-caption`) — type mixes against an opaque near-white wash (`color-mix` 66% ink). No glass `opacity` on the caption under search · match · Clear.

**Why:** WebView idle cut (#14). Glass alpha on filter caption chrome keeps Graphics and Media busy.

**Fuel:** Digester open design-review stale `feature-cpu-metrics.png`; Mac capture deferred (TCC). Standing #14 after parallel v0.1.1955 `.ops-filter-row-kb-hint`.

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `f3c90084` → pushed `main`.

**Next:** done in v0.1.1957 (`.ops-keyboard-hint`).

## Shipped — v0.1.1955 (~23:33 CEST)

**Keep:** Shared Agent Ops filter-row keyboard hint (`.ops-filter-row-kb-hint`) — type mixes against an opaque near-white wash (`color-mix` 63% ink). No glass `opacity` on the keyboard line under search · match · Clear.

**Why:** WebView idle cut (#14). Glass alpha on filter-row keyboard chrome keeps Graphics and Media busy.

**Fuel:** Digester empty; design review due=false (grace); layout daily due=false. Standing #14 after v0.1.1954 `.ops-overview-kb-hint`.

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `9d759f70` → pushed `main`.

**Next:** `.ops-filter-caption` (0.75) / `.ops-keyboard-hint` (0.7), or layout-daily data-poster / Mac capture of stale CPU metrics screen.

## Shipped — v0.1.1954 (~23:18 CEST)

**Keep:** Shared Agent Ops overview keyboard hint (`.ops-overview-kb-hint`) — type mixes against an opaque near-white wash (`color-mix` 63% ink). No glass `opacity` on the keyboard line under Overview cards.

**Why:** WebView idle cut (#14). Glass alpha on Overview keyboard chrome keeps Graphics and Media busy.

**Fuel:** Digester + design review due (stale `feature-cpu-metrics.png`); Mac capture deferred (TCC). Standing #14 after parallel v0.1.1953 `.ops-health-kb-hint`.

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `d2d28c25` → pushed `main`.

**Next:** done in v0.1.1955 (`.ops-filter-row-kb-hint`).

## Shipped — v0.1.1953 (~23:05 CEST)

**Keep:** Shared Agent Ops health-strip keyboard hint (`.ops-health-kb-hint`) — type mixes against an opaque near-white wash (`color-mix` 63% ink). No glass `opacity` on the keyboard line under Version · Discord · Redmine · Next schedule · Last delivery · Digest.

**Why:** WebView idle cut (#14). Glass alpha on health-strip chrome keeps Graphics and Media busy.

**Fuel:** Digester empty; design review due=false (grace); layout daily due=false. Standing #14 after v0.1.1952 `.ops-empty-tab-hint`.

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `0e6a163b` → pushed `main`.

**Next:** `.ops-overview-kb-hint` / filter kb-hints, or layout-daily data-poster / Mac capture of stale CPU metrics screen.

## Shipped — v0.1.1952 (~22:55 CEST)

**Keep:** Shared Agent Ops empty-tab hint (`.ops-empty-tab-hint`) — type mixes against an opaque near-white wash (`color-mix` 63% ink). No glass `opacity` on the hint under Agents · Sessions · Schedules · Knowledge · Runs empty panels.

**Why:** WebView idle cut (#14). Glass alpha on empty-tab chrome keeps Graphics and Media busy.

**Fuel:** Digester + design review due (stale `feature-cpu-metrics.png`); Mac capture deferred (TCC). Standing #14 after parallel v0.1.1951 `.ops-loading`.

**Verify:** `autoresearch_ratchet.py verify` OK → keep @ `baad03e5` → pushed `main`.

**Next:** done in v0.1.1953 (`.ops-health-kb-hint`).

## ~22:40 — v0.1.1951 Agent Ops loading shell opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-loading` uses opaque `color-mix` type (55% ink on near-white). No glass `opacity` on the Loading… panel under Overview · Agents · Sessions · Schedules · Knowledge · Runs.
- **Fuel:** standing P2 #14 after v0.1.1950 tab-bar keyboard hint; digester empty; design-review due=false (grace); layout daily due=false; Mac capture deferred.
- **Keep:** `d4903150`. Linux rack: CSS only (no install/kickstart).
- **Next:** `.ops-empty-tab-hint` (0.72), or layout-daily data-poster / Mac capture of stale `feature-cpu-metrics.png`.

## ~22:31 — v0.1.1950 Agent Ops tab-bar keyboard hint opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-tab-bar-kb-hint` uses opaque `color-mix` type (63% ink on near-white). No glass `opacity` on the keyboard line under Overview · Agents · Sessions · Schedules · Knowledge · Runs.
- **Fuel:** standing P2 #14 after v0.1.1949 overview hints; digester open design-review stale `feature-cpu-metrics.png` (~36.8d); design-review due=true; layout daily due (data-poster) deferred; Mac capture deferred (TCC / menu-bar pattern).
- **Keep:** `3854dfaa`. Install/kickstart done.
- **Next:** done in v0.1.1951 (`.ops-loading`).

## ~22:15 — v0.1.1949 Agent Ops overview hint type opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-hint` uses opaque `color-mix` type (40% ink on near-white). No glass `opacity` on the quiet helper line under Overview cards.
- **Fuel:** standing P2 #14 after v0.1.1948 overview body counts; digester open design-review stale `feature-cpu-metrics.png`; Mac capture deferred.
- **Keep:** `31e8bafc`.
- **Next:** done in v0.1.1950 (`.ops-tab-bar-kb-hint`).

## ~22:10 — v0.1.1948 Agent Ops overview body counts opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-overview-count` uses opaque `color-mix` type (48% ink on near-white). No glass `opacity` on the secondary count line under Overview card titles.
- **Fuel:** standing P2 #14 after v0.1.1947 empty shells; digester open design-review stale `feature-cpu-metrics.png` (~36.8d); design-review due=true; layout daily due (data-poster) deferred; Mac capture deferred (no on-screen CPU window / TCC pattern).
- **Keep:** `3ca93c4f`. Install/kickstart done.
- **Next:** `.ops-hint` (0.45), or `.ops-tab-bar-kb-hint` (0.72) / `.ops-loading` (0.62) / `.ops-empty-tab-hint` (0.72), or layout-daily data-poster / Mac capture of stale `feature-cpu-metrics.png`.

## ~21:50 — v0.1.1947 Agent Ops empty shell type opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-empty` resting · hover uses opaque `color-mix` type (69% / 81% ink on near-white). No glass `opacity` on Agents · Sessions · Schedules · Knowledge · Runs empty panels.
- **Fuel:** standing P2 #14 after v0.1.1946 tab digits; digester empty; design-review due=false (grace); layout daily due=false; Mac capture deferred.
- **Keep:** `48780e97`. Linux rack: CSS only (no install/kickstart).
- **Next:** `.ops-overview-count` (0.55), or `.ops-hint` / `.ops-tab-bar-kb-hint` / `.ops-loading` / `.ops-empty-tab-hint`.

## ~21:45 — v0.1.1946 Agent Ops tab digit keys opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-tab-digit` resting · hover · active uses opaque `color-mix` type (37% / 63% ink on near-white; active accent). No glass `opacity` on Overview · Agents · Sessions · Schedules · Knowledge · Runs digit keys.
- **Fuel:** standing P2 #14 after v0.1.1945 Overview Open; design-review due (stale feature-cpu-metrics); Mac capture deferred.
- **Keep:** `09a6dfd1`. Install/kickstart done.
- **Next:** `.ops-empty` (0.78 / hover 0.92), or `.ops-overview-count` / `.ops-hint` / `.ops-tab-bar-kb-hint` / `.ops-loading`.

## ~21:25 — v0.1.1945 Overview Open opaque (#14)

- **Shipped:** Shared Agent Ops Overview `.ops-overview-link` resting · hover · focus-visible · active-card uses opaque `color-mix` type (66% / 88% ink on near-white). No glass `opacity` on Open beside Agents · Schedules · Sessions · Knowledge · Runs · Digest.
- **Fuel:** standing P2 #14 after v0.1.1944 tab counts; digester empty; design-review due=false (grace); layout daily due=false; Mac capture deferred.
- **Keep:** `0669eb16`.
- **Next:** `.ops-tab-digit` — done in v0.1.1946.

## ~21:20 — v0.1.1944 Agent Ops tab count pills opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-tab-count` resting · zero · hover · active uses opaque `color-mix` type (69% / 37% / 81% ink on near-white). No glass `opacity` on Overview · Agents · Sessions · Schedules · Knowledge · Runs inventory pills.
- **Fuel:** standing P2 #14 after v0.1.1943 overview head counts; digester open design-review stale `feature-cpu-metrics.png` (~36.8d); design-review due=true; layout daily due (data-poster) deferred; Mac capture attempted (CPU window open) — TCC blocked (`could not create image from window`). Parallel-tick race on head-count discarded first.
- **Keep:** `fe6a1173`. Install/kickstart done.
- **Next:** `.ops-overview-link` — done in v0.1.1945. `.ops-tab-digit` — done in v0.1.1946.

## ~20:56 — v0.1.1943 Overview head counts opaque (#14)

- **Shipped:** Shared Agent Ops Overview `.ops-overview-head-count` resting · zero · hover · active-zero uses opaque `color-mix` type (69% / 37% / 81% ink on near-white; active-zero 55% accent). No glass `opacity` on Overview inventory pills.
- **Fuel:** standing P2 #14 after v0.1.1942 filter match; digester empty; design-review due=false (grace); layout daily due=false; Mac capture deferred.
- **Keep:** `3f27d964`.
- **Next:** `.ops-tab-count` — done in v0.1.1944.

## ~20:50 — v0.1.1942 Agent Ops filter match opaque (#14)

- **Shipped:** Shared Agent Ops `.ops-filter-match` resting · All · Partial · Zero uses opaque `color-mix` type (75% / 84% ink on near-white). No glass `opacity` on the N/M filter count chip beside search · Clear.
- **Fuel:** standing P2 #14 after v0.1.1941 Schedules counts; digester open design-review stale `feature-cpu-metrics.png` (~36.7d); design-review due=true; layout daily due (data-poster) deferred; Mac capture deferred (menu bar only).
- **Keep:** `405d44db`. Install/kickstart done.
- **Next:** `.ops-overview-head-count` (0.78 / zero 0.42 / hover 0.92), or `.ops-tab-count`, or layout-daily Data Poster / Mac capture of `feature-cpu-metrics.png`.

## ~20:30 — v0.1.1941 Schedules filter counts opaque (#14)

- **Shipped:** Shared Agent Ops Schedules `.ops-schedules-kind-count` resting · `.is-zero` uses opaque `color-mix` type (79% / 37% ink on near-white). No glass `opacity` on Jobs · Deliveries counts.
- **Fuel:** standing P2 #14 after v0.1.1940 Schedules chips; digester empty; design-review due=false (grace); layout daily due=false; Mac capture deferred. Parallel tick raced the same count wash; keep logged @ 7753eb7 (sibling `143d5e4b` won on rebase).
- **Next:** `.ops-filter-match` (0.85 / all·partial·zero 0.95) — done in v0.1.1942.

## ~20:10 — v0.1.1940 Schedules filter chips opaque (#14)

- **Shipped:** Shared Agent Ops Schedules `.ops-schedules-kind-chip` resting · hover · active uses opaque `color-mix` type (77% / 88% ink on near-white). No glass `opacity` on All · Jobs · Deliveries.
- **Fuel:** standing P2 #14 after v0.1.1939 Agents counts; digester empty; design-review due=false (grace); layout daily due=false; Mac capture deferred. Parallel tick raced the same chip wash; keep logged.
- **Next:** `.ops-schedules-kind-count` (0.85 / zero 0.42) — done in v0.1.1941.