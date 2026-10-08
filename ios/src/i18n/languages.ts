// Languages the UI ships in. Rust (`src-tauri/src/language.rs`) keeps the same list and
// decides which one is active, so the UI and the chat always agree.

export const LANGUAGES = ["es", "en", "de", "fr", "pt-BR", "zh-Hans"] as const;

export type Language = (typeof LANGUAGES)[number];

/** Debug-only pseudo-locale: wraps and stretches every string to find hard-coded text. */
export const PSEUDO = "qps";

export type UiLanguage = Language | typeof PSEUDO;

/** Each language written in itself, for the language picker. Not translated. */
export const AUTONYMS: Record<UiLanguage, string> = {
  es: "Español",
  en: "English",
  de: "Deutsch",
  fr: "Français",
  "pt-BR": "Português (Brasil)",
  "zh-Hans": "简体中文",
  qps: "Pseudo",
};

export function isUiLanguage(value: unknown): value is UiLanguage {
  return value === PSEUDO || (LANGUAGES as readonly unknown[]).includes(value);
}
