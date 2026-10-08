// UI themes. "System" is the default and follows iPhone light/dark. The other nine are
// the desktop mac-stats themes, with the same ids and names as the Mac app
// (`src-tauri/dist/themes/<id>/` at the repo root) so both apps look like siblings.
// The one exception is the `apple` theme: App Store rules forbid Apple trademarks, so its
// label here is "Glass" and it has no Apple logo. Names are proper names: not translated.
//
// Same list as `src-tauri/src/theme.rs` (a Rust test compares them); keep one theme per line.
import "./base.css";
import "./system.css";
import "./apple.css";
import "./architect.css";
import "./dark.css";
import "./data-poster.css";
import "./futuristic.css";
import "./light.css";
import "./material.css";
import "./neon.css";
import "./swiss-minimalistic.css";

export interface Theme {
  id: string;
  label: string;
  appearance: "light" | "dark";
  /** Data Poster shows big numbers and mini charts instead of rings. */
  poster?: boolean;
}

export const THEMES: readonly Theme[] = [
  { id: "apple", label: "Glass", appearance: "light" },
  { id: "architect", label: "Architect", appearance: "light" },
  { id: "dark", label: "Dark (TUI)", appearance: "dark" },
  { id: "data-poster", label: "Data Poster", appearance: "dark", poster: true },
  { id: "futuristic", label: "Futuristic", appearance: "dark" },
  { id: "light", label: "Light", appearance: "light" },
  { id: "material", label: "Material", appearance: "light" },
  { id: "neon", label: "Neon", appearance: "dark" },
  { id: "swiss-minimalistic", label: "Swiss Minimalistic", appearance: "light" },
];

let current: Theme | null = null;

export function findTheme(id: string | null | undefined): Theme | null {
  return THEMES.find((t) => t.id === id) ?? null;
}

/** Active theme, or `null` for System. */
export function activeTheme(): Theme | null {
  return current;
}

/**
 * Applies a theme before the page is shown: `data-theme` selects the theme's CSS, and the
 * `color-scheme` meta makes native controls (select, radios, scrollbars) match it.
 */
export function applyTheme(id: string | null): void {
  current = findTheme(id);
  const root = document.documentElement;
  if (current) root.dataset.theme = current.id;
  else delete root.dataset.theme;
  const scheme = current ? current.appearance : "dark light";
  root.style.colorScheme = scheme;
  document.querySelector('meta[name="color-scheme"]')?.setAttribute("content", scheme);
}
