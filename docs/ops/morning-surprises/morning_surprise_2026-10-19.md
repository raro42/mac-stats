# Morning surprise — 2026-10-19

Overnight Track B kept shipping WebView idle-cut (#14) opaque paints. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2084** | iOS lab buttons while disabled: fill mixes 50% of the track into the card, label mixes 50% of the text color (Generate uses the accent). Was glass `opacity` 0.5 |
| **v0.1.2083** | iOS thermal passed step: the scale bar mixes 35% of the step color into transparent (was glass `opacity` 0.35) |

## Why it matters

Glass `opacity` on those controls forces the compositor to blend every paint. Mixing the color into the card (or into transparent, for the thermal bar) keeps the same look and cuts idle work (#14).

## Still open

- iOS chat send while disabled (`.composer-send:disabled`, 0.4) still uses glass opacity. The thermal pulse keyframe stays, because that is a 2.4s animation on the current step.
- Design-review screen `feature-cpu-metrics.png` is stale (~42d). Needs Mac TCC capture.
