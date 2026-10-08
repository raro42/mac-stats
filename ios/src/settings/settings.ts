// Settings tab: appearance (theme) and language.
// - Themes: "System" follows iPhone light/dark; the others are the desktop mac-stats
//   themes with the same names (src/themes/index.ts).
// - Language: "Automatic" follows iOS (including the per-app language in iOS Settings).
// A change reloads the page so every string, number format and color switches at once;
// the chat picks up a new language on its next turn. Neither can change while the chat
// is writing a reply.
import { AUTONYMS, LANGUAGES, PSEUDO, t, type UiLanguage } from "../i18n";
import { errorText } from "../i18n/errors";
import {
  appLanguage,
  appTheme,
  historySettings,
  metricsClearHistory,
  setAppLanguage,
  setAppTheme,
  setBackgroundHistory,
  type HistorySettings,
} from "../ipc";
import { THEMES } from "../themes";

export interface SettingsOptions {
  /** Debug builds also offer the pseudo-locale. */
  debug: boolean;
  /** True while the chat is writing a reply; settings cannot change then. */
  busy: () => boolean;
}

interface Choice<V> {
  value: V;
  text: string;
  /** Language of `text`, so VoiceOver reads each language name in its own language. */
  lang?: string;
  /** Theme id for the color swatch. */
  swatch?: string;
}

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}

/** Radio list where picking an option saves it and reloads the page. */
function radioList<V>(
  list: HTMLElement,
  note: HTMLElement,
  name: string,
  choices: Choice<V>[],
  current: V,
  busy: () => boolean,
  save: (value: V) => Promise<unknown>,
): void {
  const radios = choices.map((choice) => {
    const label = document.createElement("label");
    label.className = "choice";
    const input = document.createElement("input");
    input.type = "radio";
    input.name = name;
    input.checked = choice.value === current;
    const text = document.createElement("span");
    text.textContent = choice.text;
    if (choice.lang) text.lang = choice.lang;
    input.addEventListener("change", async () => {
      if (busy()) {
        note.textContent = t("settings.busy");
        note.hidden = false;
        for (const r of radios) r.input.checked = r.value === current;
        return;
      }
      try {
        await save(choice.value);
        location.reload();
      } catch (error) {
        note.textContent = errorText(error);
        note.hidden = false;
      }
    });
    label.append(input);
    if (choice.swatch) {
      const swatch = document.createElement("span");
      swatch.className = "swatch";
      swatch.dataset.swatch = choice.swatch;
      swatch.setAttribute("aria-hidden", "true");
      label.append(swatch);
    }
    label.append(text);
    return { value: choice.value, input, label };
  });
  list.replaceChildren(...radios.map((r) => r.label));
}

/** History card: background samples switch, iOS status note and "Delete history". */
async function startHistory(): Promise<void> {
  const toggle = byId<HTMLInputElement>("background-history");
  const iosOff = byId("background-off");
  const remove = byId<HTMLButtonElement>("delete-history");
  const status = byId("history-status");

  const show = (s: HistorySettings) => {
    toggle.checked = s.background;
    // iOS lets the user turn Background App Refresh off per app; then only the time
    // with the app open is saved.
    iosOff.hidden = !(s.background && (s.iosStatus === "denied" || s.iosStatus === "restricted"));
  };
  show(await historySettings());

  toggle.addEventListener("change", async () => {
    try {
      show(await setBackgroundHistory(toggle.checked));
    } catch (error) {
      toggle.checked = !toggle.checked;
      status.textContent = errorText(error);
    }
  });

  // Deleting cannot be undone, so it takes a second tap (same pattern as the model card).
  let armed: number | null = null;
  remove.addEventListener("click", async () => {
    if (armed === null) {
      remove.textContent = t("settings.confirmDeleteHistory");
      armed = window.setTimeout(() => {
        armed = null;
        remove.textContent = t("settings.deleteHistory");
      }, 4000);
      return;
    }
    window.clearTimeout(armed);
    armed = null;
    remove.textContent = t("settings.deleteHistory");
    try {
      await metricsClearHistory();
      status.textContent = t("settings.historyDeleted");
    } catch (error) {
      status.textContent = errorText(error);
    }
  });
}

export async function startSettings({ debug, busy }: SettingsOptions): Promise<void> {
  void startHistory().catch((error: unknown) => console.error("history settings", error));
  const [language, theme] = await Promise.all([appLanguage(), appTheme()]);

  radioList<string | null>(
    byId("theme-list"),
    byId("theme-note"),
    "theme",
    [
      { value: null, text: t("settings.themeSystem"), swatch: "system" },
      ...THEMES.map((th) => ({ value: th.id, text: th.label, swatch: th.id })),
    ],
    theme.theme,
    busy,
    setAppTheme,
  );

  const extra: UiLanguage[] = debug ? [PSEUDO] : [];
  const automatic = AUTONYMS[language.automatic as UiLanguage] ?? language.automatic;
  radioList<UiLanguage | null>(
    byId("language-list"),
    byId("language-note"),
    "language",
    [
      { value: null, text: t("settings.automatic", { language: automatic }) },
      ...[...LANGUAGES, ...extra].map((l) => ({
        value: l,
        text: AUTONYMS[l],
        lang: l === PSEUDO ? undefined : l,
      })),
    ],
    language.setting as UiLanguage | null,
    busy,
    setAppLanguage,
  );
}
