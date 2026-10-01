// Tarjeta «Laboratorio»: prueba manual de los modelos locales (fase A).
import { bytes } from "../format";
import {
  labBench,
  labCancel,
  labGenerate,
  labLoad,
  labModels,
  labUnload,
  type LabModel,
} from "../ipc";

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Falta el elemento #${id}`);
  return el as T;
}

const tps = (v: number) => `${v.toFixed(1)} tok/s`;

export async function startLab(): Promise<void> {
  const select = byId<HTMLSelectElement>("lab-model");
  const status = byId("lab-status");
  const output = byId("lab-output");
  const prompt = byId<HTMLTextAreaElement>("lab-prompt");
  const loadBtn = byId<HTMLButtonElement>("lab-load");
  const unloadBtn = byId<HTMLButtonElement>("lab-unload");
  const benchBtn = byId<HTMLButtonElement>("lab-bench");
  const sendBtn = byId<HTMLButtonElement>("lab-send");

  let models: LabModel[] = [];
  let generating = false;

  const selected = () => models.find((m) => m.id === select.value);
  const setBusy = (busy: boolean) => {
    for (const b of [loadBtn, unloadBtn, benchBtn]) b.disabled = busy;
  };
  const fail = (e: unknown) => {
    status.textContent = `Error: ${String(e)}`;
    setBusy(false);
  };

  models = await labModels();
  select.replaceChildren(
    ...models.map((m) => {
      const option = document.createElement("option");
      option.value = m.id;
      option.textContent = `${m.name} · ${bytes(m.size)}${m.installed ? "" : " (no está en el iPhone)"}`;
      option.disabled = !m.installed;
      return option;
    }),
  );
  const firstInstalled = models.find((m) => m.installed);
  if (firstInstalled) select.value = firstInstalled.id;
  status.textContent = firstInstalled
    ? "Elige un modelo y pulsa Cargar."
    : "No hay modelos en el iPhone todavía.";

  loadBtn.addEventListener("click", async () => {
    setBusy(true);
    status.textContent = "Cargando… (la primera vez compila los shaders de Metal)";
    try {
      const info = await labLoad(select.value);
      status.textContent = `${info.description} · ${(info.loadMs / 1000).toFixed(1)} s · ${
        info.gpu ? "GPU" : "CPU"
      } · margen ${bytes(info.availableMemory)}`;
    } catch (e) {
      fail(e);
    }
    setBusy(false);
  });

  unloadBtn.addEventListener("click", async () => {
    try {
      await labUnload();
      status.textContent = "Modelo descargado de la memoria.";
    } catch (e) {
      fail(e);
    }
  });

  benchBtn.addEventListener("click", async () => {
    setBusy(true);
    status.textContent = "Midiendo (pp512 / tg128 × 3)…";
    try {
      const r = await labBench();
      status.textContent = `Prompt ${tps(r.ppTps)} · generación ${tps(r.tgTps)} · margen ${bytes(r.availableMemory)}`;
    } catch (e) {
      fail(e);
    }
    setBusy(false);
  });

  sendBtn.addEventListener("click", async () => {
    if (generating) {
      void labCancel();
      return;
    }
    const text = prompt.value.trim();
    if (!text) return;
    generating = true;
    sendBtn.textContent = "Detener";
    output.textContent = "";
    try {
      const r = await labGenerate(text, selected()?.thinkPrefill ?? false, (delta) => {
        output.textContent += delta;
      });
      status.textContent = `${r.nGen} tokens · ${tps(r.tgTps)} · prompt ${tps(r.ppTps)} (${r.nCached} reutilizados) · fin: ${r.stopReason}`;
    } catch (e) {
      fail(e);
    }
    generating = false;
    sendBtn.textContent = "Generar";
  });
}
