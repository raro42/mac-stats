// UI translations. `initI18n()` asks Rust for the active language once at startup; a
// language change reloads the page, so formatters and dictionaries never switch live.
import { de } from "./de";
import { en } from "./en";
import { es } from "./es";
import { fr } from "./fr";
import { isUiLanguage, PSEUDO, type Language, type UiLanguage } from "./languages";
import type { Messages, Plural } from "./messages";
import { ptBR } from "./pt-BR";
import { zhHans } from "./zh-Hans";

export { AUTONYMS, LANGUAGES, PSEUDO, type Language, type UiLanguage } from "./languages";

const DICTIONARIES: Record<Language, Messages> = { es, en, de, fr, "pt-BR": ptBR, "zh-Hans": zhHans };

type KeysOf<V> = { [K in keyof Messages]: Messages[K] extends V ? K : never }[keyof Messages];
export type TextKey = KeysOf<string>;
export type PluralKey = KeysOf<Plural>;
export type Params = Record<string, string | number>;

let current: UiLanguage = "en";
let currentLocale = "en";
let messages: Messages = en;
let pluralRules: Intl.PluralRules | null = null;

/** Wraps every string as `[!! … ~~~ !!]`, about 40% longer, keeping `{placeholders}`. */
function pseudoize(dict: Messages): Messages {
  const wrap = (s: string) => `[!! ${s} ${"~".repeat(Math.ceil(s.length * 0.4))} !!]`;
  const out: Record<string, string | Plural> = {};
  for (const [key, value] of Object.entries(dict) as [string, string | Plural][]) {
    if (typeof value === "string") {
      out[key] = wrap(value);
    } else {
      const plural: Record<string, string> = {};
      for (const [category, text] of Object.entries(value)) plural[category] = wrap(text);
      out[key] = plural as Plural;
    }
  }
  return out as unknown as Messages;
}

/**
 * `regionLocale` is the iPhone's full tag for this language when it has one (for example
 * `es-MX`), so numbers follow the user's region. Invalid tags fall back to the language.
 */
export function setLanguage(language: UiLanguage, regionLocale?: string): void {
  current = language;
  messages = language === PSEUDO ? pseudoize(en) : DICTIONARIES[language];
  const base = language === PSEUDO ? "en" : language;
  currentLocale = base;
  if (regionLocale && language !== PSEUDO) {
    try {
      [currentLocale] = Intl.getCanonicalLocales(regionLocale);
    } catch {
      currentLocale = base;
    }
  }
  pluralRules = null;
  document.documentElement.lang = currentLocale;
}

export function language(): UiLanguage {
  return current;
}

/** BCP-47 tag for `Intl` and `<html lang>`. The pseudo-locale formats like English. */
export function locale(): string {
  return currentLocale;
}

function fill(template: string, params?: Params): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

export function t(key: TextKey, params?: Params): string {
  const template = messages[key] ?? en[key] ?? key;
  return fill(template, params);
}

/** Plural message for `count`; `{n}` is the count formatted for the current locale. */
export function tp(key: PluralKey, count: number, params?: Params): string {
  pluralRules ??= new Intl.PluralRules(locale());
  const forms = messages[key];
  const template = forms[pluralRules.select(count)] ?? forms.other;
  return fill(template, { n: new Intl.NumberFormat(locale()).format(count), ...params });
}

/** Untyped lookup for keys read from the DOM; unknown keys come back as the key itself. */
function lookup(key: string): string {
  const value = (messages as unknown as Record<string, unknown>)[key];
  return typeof value === "string" ? value : key;
}

const ATTRIBUTES = [
  ["aria-label", "i18nAriaLabel"],
  ["placeholder", "i18nPlaceholder"],
  ["title", "i18nTitle"],
] as const;

/**
 * Fills static text in the page from `data-i18n` (text content), `data-i18n-aria-label`,
 * `data-i18n-placeholder` and `data-i18n-title` attributes.
 */
export function applyTranslations(root: ParentNode = document): void {
  for (const el of root.querySelectorAll<HTMLElement>("[data-i18n]")) {
    el.textContent = lookup(el.dataset.i18n ?? "");
  }
  for (const [attr, prop] of ATTRIBUTES) {
    for (const el of root.querySelectorAll<HTMLElement>(`[data-i18n-${attr}]`)) {
      el.setAttribute(attr, lookup(el.dataset[prop] ?? ""));
    }
  }
}

export { isUiLanguage };
