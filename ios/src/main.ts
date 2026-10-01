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
});
