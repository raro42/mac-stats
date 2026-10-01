import { invoke } from "@tauri-apps/api/core";

window.addEventListener("DOMContentLoaded", async () => {
  const estado = document.querySelector<HTMLParagraphElement>("#estado");
  if (!estado) return;
  try {
    estado.textContent = await invoke<string>("ping");
  } catch (e) {
    estado.textContent = `Error: ${String(e)}`;
  }
});
