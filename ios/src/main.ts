import { startBenchBanner } from "./bench-banner";
import { startChat, type ChatView } from "./chat/chat";
import { startLab } from "./chat/lab";
import { applyTranslations, isUiLanguage, setLanguage, t } from "./i18n";
import { errorText } from "./i18n/errors";
import { appLanguage, appTheme, debugBuild, debugDemoPrompt, debugDemoView } from "./ipc";
import { startMonitor } from "./monitor/monitor";
import { startSettings } from "./settings/settings";
import { applyTheme } from "./themes";

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

/**
 * Language and theme, before the page is shown. Rust decides the language (in-app
 * setting or iOS preferences) so the UI and the chat agree, and stores the theme.
 */
async function loadAppearance(): Promise<void> {
  try {
    const lang = await appLanguage();
    setLanguage(isUiLanguage(lang.resolved) ? lang.resolved : "en", lang.locale ?? undefined);
  } catch (error) {
    console.error("could not read the app language", error);
    setLanguage("en");
  }
  try {
    applyTheme((await appTheme()).theme);
  } catch (error) {
    console.error("could not read the app theme", error);
    applyTheme(null);
  }
  applyTranslations();
  document.body.classList.remove("i18n-pending");
}

/**
 * Debug builds, for screenshots: `IOS_STATS_DEMO_VIEW=<tab>[-bottom][-<range>]`, e.g.
 * `settings`, `monitor-bottom` or `monitor-bottom-24h` (opens that history range).
 */
async function showDemoView(): Promise<void> {
  const view = await debugDemoView();
  if (!view) return;
  const [tab, ...options] = view.split("-");
  document.querySelector<HTMLButtonElement>(`.tab[data-view="${tab}"]`)?.click();
  const range = options.find((o) => o !== "bottom");
  window.setTimeout(() => {
    if (range) document.querySelector<HTMLButtonElement>(`[data-range="${range}"]`)?.click();
    if (options.includes("bottom")) window.scrollTo({ top: document.body.scrollHeight });
  }, 1500);
}

window.addEventListener("DOMContentLoaded", async () => {
  // iOS only applies :active (tap feedback in some themes) when a touch listener exists.
  document.addEventListener("touchstart", () => {}, { passive: true });
  await loadAppearance();
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
    void showDemoView();
    const lab = document.getElementById("lab-details");
    if (lab) lab.hidden = false;
    void startBenchBanner();
    startLab().catch((error: unknown) => {
      const status = document.getElementById("lab-status");
      if (status) status.textContent = t("lab.startFailed", { error: errorText(error) });
    });
  }
});
