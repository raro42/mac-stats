import { startBenchBanner } from "./bench-banner";
import { startChat } from "./chat/chat";
import { startLab } from "./chat/lab";
import { debugBuild, debugDemoPrompt } from "./ipc";
import { startMonitor } from "./monitor/monitor";

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

window.addEventListener("DOMContentLoaded", () => {
  setupTabs();
  startMonitor().catch((error: unknown) => {
    const device = document.getElementById("device");
    if (device) device.textContent = `No se pudo iniciar el monitor: ${String(error)}`;
  });
  startChat()
    .then(async (chat) => {
      const demo = await debugDemoPrompt();
      if (!demo) return;
      document.querySelector<HTMLButtonElement>('.tab[data-view="chat"]')?.click();
      await chat.ask(demo);
    })
    .catch((error: unknown) => {
      const banner = document.getElementById("chat-banner");
      if (banner) {
        banner.textContent = `No se pudo iniciar el chat: ${String(error)}`;
        banner.hidden = false;
      }
    });
  // El laboratorio de modelos solo aparece en builds de depuración.
  void debugBuild().then((debug) => {
    if (!debug) return;
    const lab = document.getElementById("lab-details");
    if (lab) lab.hidden = false;
    void startBenchBanner();
    startLab().catch((error: unknown) => {
      const status = document.getElementById("lab-status");
      if (status) status.textContent = `No se pudo iniciar el laboratorio: ${String(error)}`;
    });
  });
});
