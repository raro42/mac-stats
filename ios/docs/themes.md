# Themes

iOS Stats and the desktop mac-stats are sister apps: someone who uses one should recognize the other at once. The iPhone app therefore ships the nine desktop themes with the same ids, names, colors, typography, ring style and backgrounds, adapted to the iPhone layout. The default theme, **System**, is the iOS look and follows iPhone light/dark.

## Theme list

| id | Name on iPhone | Name on desktop | Appearance |
|---|---|---|---|
| — | System | — | follows iOS |
| `apple` | Glass | Apple | light |
| `architect` | Architect | Architect | light |
| `dark` | Dark (TUI) | Dark (TUI) | dark |
| `data-poster` | Data Poster | Data Poster | dark |
| `futuristic` | Futuristic | Futuristic | dark |
| `light` | Light | Light | light |
| `material` | Material | Material | light |
| `neon` | Neon | Neon | dark |
| `swiss-minimalistic` | Swiss Minimalistic | Swiss Minimalistic | light |

The only difference is the `apple` theme. App Store guideline 5.2.5 forbids Apple trademarks, so on the iPhone it is called "Glass" and does not show the Apple logo. The id stays `apple`, the same key the desktop stores.

Theme names are proper names and are not translated, as on the desktop.

## What changes from the desktop, and why

- **Layout.** The desktop window has four gauges in a row. The iPhone has a 2×2 grid of rings followed by cards.
- **Rings.** The desktop rings are CPU, GPU, frequency and temperature. iOS does not expose GPU usage, frequency or °C, so the iPhone rings are CPU, RAM, storage and battery. Each takes the color of the matching desktop ring:

  | iPhone | Desktop color used |
  |---|---|
  | CPU | CPU (usage) color |
  | RAM | GPU color (`#c4a8d8`, the same in every desktop theme) |
  | Storage | Frequency color |
  | Battery | Desktop battery green `rgba(52, 199, 89, 0.8)` |

- **Thermal.** The desktop's temperature card shows "Thermal: Nominal / Fair / Serious / Critical" from the same OS thermal state. The iPhone thermal card uses the same words in English and the same state colors: nominal green, fair amber, serious amber pulsing. Critical is red on the iPhone because there it pauses the chat; the desktop has no separate critical color.
- **Status bar.** iOS draws the status bar. Fixed light or dark themes set the app windows' light/dark style (`src-tauri/src/theme.rs`) so the clock stays readable.

## How a theme is built

- `src/themes/system.css` defines every token with its default (System) value and documents the list.
- `src/themes/base.css` is the layout and components, and uses only tokens.
- `src/themes/<id>.css` overrides tokens under `:root[data-theme="<id>"]`, plus a few structural rules for the theme's look (glass hairlines, blueprint borders, the Neon frame, Material's dial faces…). Each block cites the desktop rule it comes from (`src-tauri/dist/themes/<id>/cpu.css`).
- Ring shape comes from tokens: `--ring-arc` (share of the circle) and `--ring-start` (rotation), read by `src/monitor/ring-gauge.ts`.
- Data Poster has no rings. Its CPU and RAM tiles draw the desktop's 12-bar and 60-point line charts (`src/monitor/poster-chart.ts`, ported from `data-poster/poster-charts.js`).
- The page background is painted on a fixed layer (`html::before`), because iOS ignores `background-attachment: fixed`.

## Adding or changing a theme

1. Change the desktop theme first, or at the same time: the two apps must stay in step.
2. Add the id to `THEMES` in `src/themes/index.ts` and in `src-tauri/src/theme.rs` (a Rust test checks the two lists match).
3. Create `src/themes/<id>.css`, import it in `src/themes/index.ts`, and add its picker swatch (`.swatch[data-swatch="<id>"]`).
4. Check it in the simulator in both iOS appearances:
   `xcrun simctl ui booted appearance light` / `dark`.
