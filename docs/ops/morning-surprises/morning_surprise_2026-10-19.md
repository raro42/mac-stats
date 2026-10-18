# Morning surprise — 2026-10-19

Overnight Track B kept shipping WebView idle-cut (#14) opaque paints. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2083** | iOS thermal passed step: the scale bar mixes 35% of the step color into transparent (was glass `opacity` 0.35) |

## Why it matters

Glass `opacity` on that mark forces the compositor to blend every paint. Mixing the step color into transparent keeps the same look and cuts idle work (#14).

## Still open

- iOS lab button while disabled (`.lab-btn:disabled`, 0.5) still uses glass opacity. Chat send while disabled is 0.4. The thermal pulse keyframe stays, because that is a 2.4s animation on the current step.
- Design-review screen `feature-cpu-metrics.png` is stale (~42d). Needs Mac TCC capture.
