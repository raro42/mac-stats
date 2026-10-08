// "Lab" card: manual testing of the local models (phase A).
import { bytes, seconds, tokensPerSecond } from "../format";
import { t, tp, type TextKey } from "../i18n";
import { errorText } from "../i18n/errors";
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
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}

const STOP_LABEL: Record<string, TextKey> = {
  eos: "stop.eos",
  cancelled: "stop.cancelled",
  thermal: "stop.thermal",
  length: "stop.length",
};

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
    status.textContent = t("lab.error", { error: errorText(e) });
    setBusy(false);
  };

  models = await labModels();
  select.replaceChildren(
    ...models.map((m) => {
      const option = document.createElement("option");
      option.value = m.id;
      option.textContent = t(m.installed ? "model.option" : "lab.notInstalled", { name: m.name, size: bytes(m.size) });
      option.disabled = !m.installed;
      return option;
    }),
  );
  const firstInstalled = models.find((m) => m.installed);
  if (firstInstalled) select.value = firstInstalled.id;
  status.textContent = t(firstInstalled ? "lab.pickModel" : "lab.noModels");

  loadBtn.addEventListener("click", async () => {
    setBusy(true);
    status.textContent = t("lab.loading");
    try {
      const info = await labLoad(select.value);
      status.textContent = t("lab.loaded", {
        description: info.description || "—",
        seconds: seconds(info.loadMs / 1000),
        backend: info.gpu ? "GPU" : "CPU",
        headroom: bytes(info.availableMemory),
      });
    } catch (e) {
      fail(e);
    }
    setBusy(false);
  });

  unloadBtn.addEventListener("click", async () => {
    try {
      await labUnload();
      status.textContent = t("lab.unloaded");
    } catch (e) {
      fail(e);
    }
  });

  benchBtn.addEventListener("click", async () => {
    setBusy(true);
    status.textContent = t("lab.benching");
    try {
      const r = await labBench();
      status.textContent = t("lab.benchResult", {
        pp: tokensPerSecond(r.ppTps),
        tg: tokensPerSecond(r.tgTps),
        headroom: bytes(r.availableMemory),
      });
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
    sendBtn.textContent = t("chat.stop");
    output.textContent = "";
    try {
      const r = await labGenerate(text, selected()?.thinkPrefill ?? false, (delta) => {
        output.textContent += delta;
      });
      const reason = STOP_LABEL[r.stopReason];
      status.textContent = t("lab.generateResult", {
        tokens: tp("lab.tokens", r.nGen),
        tg: tokensPerSecond(r.tgTps),
        pp: tokensPerSecond(r.ppTps),
        reused: tp("lab.reused", r.nCached),
        reason: reason ? t(reason) : r.stopReason,
      });
    } catch (e) {
      fail(e);
    }
    generating = false;
    sendBtn.textContent = t("lab.generate");
  });
}
