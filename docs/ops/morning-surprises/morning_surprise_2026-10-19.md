# Morning surprise — 2026-10-19

Overnight Track B kept shipping WebView idle-cut (#14) opaque paints. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2095** | Architect section icon images: the bitmap paints at full strength on the white chip. Was a glass `brightness` / `opacity()` filter (resting `opacity(0.55)`, hover `opacity(0.7)`). Ready, Slow, and Down still tint and dim their own images |
| **v0.1.2094** | Swiss section icon images: the bitmap paints at full strength on the white chip. Was a glass `brightness` / `opacity()` filter (resting `opacity(0.55)`, hover `opacity(0.7)`). Ready, Slow, and Down still tint and dim their own images |
| **v0.1.2093** | Apple Ready, Slow, and Down section icons: those bitmaps paint at full strength on the white chip. Was glass `opacity` 0.85 / 0.8 (hover 0.95 / 0.9) |
| **v0.1.2092** | Light Ready, Slow, and Down section icons: those bitmaps paint at full strength on the white chip. Was glass `opacity` 0.85 / 0.8 (hover 0.95 / 0.9) |
| **v0.1.2091** | Apple section icon images: the bitmap paints at full strength on the white chip. Was glass `opacity` 0.55 (hover 0.75). Ready, Slow, and Down still dim their own images |
| **v0.1.2090** | Light section icon images: the bitmap paints at full strength on the white chip. Was glass `opacity` 0.55 (hover 0.75). Ready, Slow, and Down still dim their own images |
| **v0.1.2089** | Bat glyph with no battery: mixes the dim ink into the strip (20% ink on Apple and Light, 31% `#ebebf5` on Data Poster). Was glass `opacity` 0.4 |
| **v0.1.2088** | AI Chat, Perplexity, Discord, and Agent Ops icons while local AI is off: the chip mixes 55% of its fill into the shell. Ready, Slow, and Down stay full strength. Was glass `opacity` 0.55 |
| **v0.1.2087** | Settings "Judge on failure only" while Judge is off: 48% ink on white (41% white on Data Poster and Futuristic, 44% mint on Neon, 44% green on Dark). Was glass `opacity` 0.55 |
| **v0.1.2086** | Agent Ops insight emphasis: the question mixes 70% of the line color into the wash (53% white on the dark dashboard sheet). Was glass `opacity` 0.7 |
| **v0.1.2085** | iOS chat Send while disabled: fill mixes 40% of the accent into the composer bar, label mixes 40% of the on-accent color. Was glass `opacity` 0.4 |
| **v0.1.2084** | iOS lab buttons while disabled: fill mixes 50% of the track into the card, label mixes 50% of the text color (Generate uses the accent). Was glass `opacity` 0.5 |
| **v0.1.2083** | iOS thermal passed step: the scale bar mixes 35% of the step color into transparent (was glass `opacity` 0.35) |

## Why it matters

Glass `opacity` on those controls forces the compositor to blend every paint. Mixing the color into the wash (or into the bar, the card, or transparent for the thermal bar) keeps the same look and cuts idle work (#14).

## Still open

- Material section icon images still use `brightness` / `opacity()` filters. Architect, Material, and the dark shells still dim Ready, Slow, and Down bitmaps (0.85 / 0.8). Swiss Ready, Slow, and Down still tint with filters. The thermal pulse keyframe stays, because that is a 2.4s animation on the current step.
- Design-review screen `feature-cpu-metrics.png` is stale (~42d). Needs Mac TCC capture.
