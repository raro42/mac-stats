// Settings tab: language picker. "Automatic" follows iOS (including the per-app language
// in iOS Settings). A change reloads the page so every string and number format switches
// at once; the chat picks up the new language on its next turn.
import { AUTONYMS, LANGUAGES, PSEUDO, t, type UiLanguage } from "../i18n";
import { errorText } from "../i18n/errors";
import { appLanguage, setAppLanguage } from "../ipc";

export interface SettingsOptions {
  /** Debug builds also offer the pseudo-locale. */
  debug: boolean;
  /** True while the chat is writing a reply; the language cannot change then. */
  busy: () => boolean;
}

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}

export async function startSettings({ debug, busy }: SettingsOptions): Promise<void> {
  const list = byId("language-list");
  const note = byId("language-note");
  const current = await appLanguage();
  const extra: UiLanguage[] = debug ? [PSEUDO] : [];
  const choices: (UiLanguage | null)[] = [null, ...LANGUAGES, ...extra];
  const automatic = AUTONYMS[current.automatic as UiLanguage] ?? current.automatic;

  const radios = choices.map((value) => {
    const label = document.createElement("label");
    label.className = "choice";
    const input = document.createElement("input");
    input.type = "radio";
    input.name = "language";
    input.checked = current.setting === value;
    const text = document.createElement("span");
    if (value === null) {
      text.textContent = t("settings.automatic", { language: automatic });
    } else {
      text.textContent = AUTONYMS[value];
      // VoiceOver reads each language name in its own language.
      if (value !== PSEUDO) text.lang = value;
    }
    input.addEventListener("change", async () => {
      if (busy()) {
        note.textContent = t("settings.busy");
        for (const r of radios) r.input.checked = r.value === current.setting;
        return;
      }
      try {
        await setAppLanguage(value);
        location.reload();
      } catch (error) {
        note.textContent = errorText(error);
      }
    });
    label.append(input, text);
    return { value, input, label };
  });
  list.replaceChildren(...radios.map((r) => r.label));
}
