import { startBenchBanner } from "./bench-banner";
import { startChat, type ChatView } from "./chat/chat";
import { startLab } from "./chat/lab";
import { applyTranslations, isUiLanguage, setLanguage, t } from "./i18n";
import { errorText } from "./i18n/errors";
import { appLanguage, debugBuild, debugDemoPrompt } from "./ipc";
import { startMonitor } from "./monitor/monitor";
import { startSettings } from "./settings/settings";

function setupTabs(): void {
  const tabs = Array.from(document.querySelectorAll<HTMLButtonElement>(".tab"));
  const views = Array.from(document.querySelectorAll<HTMLElement>(".view"));
  for (const tab of tabs) {
    tab.addEventListener("click", () => {
      for (const t of tabs) t.setAttribute("aria-selected", String(t === tab));
      for (const v of views) v.hidden = v.id !== `view-${tab.dataset.view}`;
    });
  }
}

/** Rust decides the language (in-app setting or iOS preferences) so the UI and the chat agree. */
async function loadLanguage(): Promise<void> {
  try {
    const lang = await appLanguage();
    setLanguage(isUiLanguage(lang.resolved) ? lang.resolved : "en", lang.locale ?? undefined);
  } catch (error) {
    console.error("could not read the app language", error);
    setLanguage("en");
  }
  applyTranslations();
  document.body.classList.remove("i18n-pending");
}

window.addEventListener("DOMContentLoaded", async () => {
  await loadLanguage();
  setupTabs();
  const debug = await debugBuild().catch(() => false);

  startMonitor().catch((error: unknown) => {
    const device = document.getElementById("device");
    if (device) device.textContent = t("monitor.startFailed", { error: errorText(error) });
  });

  let chat: ChatView | null = null;
  startSettings({ debug, busy: () => chat?.generating() ?? false }).catch((error: unknown) => {
    console.error("could not start settings", error);
  });

  startChat()
    .then(async (view) => {
      chat = view;
      const demo = await debugDemoPrompt();
      if (!demo) return;
      document.querySelector<HTMLButtonElement>('.tab[data-view="chat"]')?.click();
      await view.ask(demo);
    })
    .catch((error: unknown) => {
      const banner = document.getElementById("chat-banner");
      if (banner) {
        banner.textContent = t("chat.startFailed", { error: errorText(error) });
        banner.hidden = false;
      }
    });

  // The model lab only shows up in debug builds.
  if (debug) {
    const lab = document.getElementById("lab-details");
    if (lab) lab.hidden = false;
    void startBenchBanner();
    startLab().catch((error: unknown) => {
      const status = document.getElementById("lab-status");
      if (status) status.textContent = t("lab.startFailed", { error: errorText(error) });
    });
  }
});
