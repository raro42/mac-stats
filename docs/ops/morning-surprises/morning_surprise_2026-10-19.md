# Morning surprise — 2026-10-19

Overnight Track B (20:00–06:00 local). Digester Slowest empty; design-review stale `feature-agent-ops.png` (~33d) and `feature-cpu-metrics.png` (~42d) stayed open. Mac capture deferred on the Linux rack. Standing P2 #14 (WebView glass opacity) fueled the keeps.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2106** | Apple Top Processes `.process-row:active` — drop glass opacity 0.92; quiet `color-mix` press wash at full opacity |
| **v0.1.2105** | Dark Ready · Slow · Down section icons — drop glass opacity; boot contrast CSS matched |
| **v0.1.2104** | Neon and Futuristic Ready · Slow · Down section icons — drop glass opacity + colorizing filters; boot contrast CSS matched |
| **v0.1.2103** | Dark resting section icons — Ollama mark paints in chip color; drop glass `opacity` 0.45 (hover 0.75); boot contrast CSS matched |
| **v0.1.2102** | Neon and Futuristic resting section icons — Ollama mark paints in chip color; drop glass `brightness` / `invert` / `opacity()` filter |
| **v0.1.2101** | Data Poster Ready · Slow · Down section icons — drop glass opacity + colorizing filters; boot contrast CSS matched |
| **v0.1.2100** | Swiss Ready · Slow · Down section icons — drop glass opacity + colorizing filters |
| **v0.1.2099** | Material Ready · Slow · Down section icons — drop glass opacity + colorizing filters |
| **v0.1.2098** | Data Poster resting section icons — Ollama mark paints in chip color (no `brightness` / `invert`) |
| **v0.1.2097** | Architect Ready · Slow · Down section icons — drop glass opacity + colorizing filters |

## This tick (~04:00)

- Fuel: digester empty → design review due (Mac capture deferred) → standing after Dark Ready/Slow/Down (origin at v0.1.2105). Named leftover: Apple `.process-row:active` glass opacity 0.92.
- Keep @ `cd5fa11c` — Apple Top Processes row press opaque; matches shared sheet wash.
- Layout daily due=false (data-poster already checked today).

## Why it matters

Glass `opacity` on the whole process row keeps Graphics and Media busy while you click. An opaque press wash cuts that idle work (#14).

## Next fuel

- Light/other theme `#cpu-usage-card:hover` / `.collapsible-header:hover` glass opacity
- Mac capture of stale `feature-cpu-metrics.png` / `feature-agent-ops.png` when TCC allows

## Note

Empty digester alone is not a surprise — the night moved the glass-opacity ratchet on section icon chrome and Top Processes press (#14).
