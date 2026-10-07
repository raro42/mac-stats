// Model card: pick a model from the catalog, download it (with progress and SHA-256
// verification in Swift), cancel the download, or delete it.
import { bytes } from "../format";
import {
  chatCancelDownload,
  chatDeleteModel,
  chatDownload,
  chatModels,
  chatSelectModel,
  type ChatModel,
} from "../ipc";

export interface ModelCard {
  /** Selected model, or `null` if the catalog has not been read yet. */
  selected(): ChatModel | null;
  refresh(): Promise<void>;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, className?: string, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}

export function createModelCard(root: HTMLElement, onChange: () => void): ModelCard {
  let models: ChatModel[] = [];
  let downloading = false;

  const select = el("select", "model-select");
  select.setAttribute("aria-label", "Modelo");
  const meta = el("p", "card-text");
  const progress = el("div", "bar");
  const fill = el("div", "bar-fill");
  progress.append(fill);
  progress.hidden = true;
  const actions = el("div", "model-actions");
  const cellular = el("label", "model-cellular");
  const cellularBox = el("input");
  cellularBox.type = "checkbox";
  cellular.append(cellularBox, " Usar datos móviles");
  const primary = el("button", "lab-btn model-primary");
  primary.type = "button";
  const remove = el("button", "lab-btn");
  remove.type = "button";
  remove.textContent = "Borrar";
  actions.append(primary, remove);
  root.replaceChildren(el("h2", "", "Modelo"), select, meta, progress, cellular, actions);

  const selected = () => models.find((m) => m.selected) ?? null;

  function render(): void {
    const model = selected();
    select.replaceChildren(
      ...models.map((m) => {
        const option = el("option", "", `${m.name} · ${bytes(m.size)}${m.recommended ? " · recomendado" : ""}`);
        option.value = m.id;
        option.selected = m.selected;
        return option;
      }),
    );
    select.disabled = downloading;
    if (!model) return;

    const installed = model.installed;
    if (!downloading) {
      progress.hidden = true;
      meta.textContent = installed
        ? `Listo en el iPhone · ${model.license}. Funciona sin conexión.`
        : `Se descarga una vez (${bytes(model.size)}) y luego funciona sin conexión · ${model.license}.`;
    }
    cellular.hidden = installed || downloading;
    primary.hidden = installed && !downloading;
    primary.textContent = downloading ? "Cancelar" : "Descargar";
    remove.hidden = !installed || downloading;
  }

  async function refresh(): Promise<void> {
    models = await chatModels();
    render();
  }

  select.addEventListener("change", async () => {
    await chatSelectModel(select.value);
    await refresh();
    onChange();
  });

  primary.addEventListener("click", async () => {
    const model = selected();
    if (!model) return;
    if (downloading) {
      await chatCancelDownload();
      return;
    }
    downloading = true;
    progress.hidden = false;
    fill.style.width = "0%";
    meta.textContent = "Preparando la descarga…";
    render();
    try {
      await chatDownload(model.id, cellularBox.checked, (event) => {
        if (event.type === "progress" && event.total > 0) {
          const share = event.received / event.total;
          fill.style.width = `${(share * 100).toFixed(1)}%`;
          meta.textContent = `Descargando ${(share * 100).toFixed(0)}% · ${bytes(event.received)} de ${bytes(event.total)}`;
        } else if (event.type === "verifying") {
          meta.textContent = "Verificando el archivo (SHA-256)…";
        }
      });
      downloading = false;
      await refresh();
      onChange();
    } catch (error) {
      downloading = false;
      render();
      meta.textContent = String(error);
    }
  });

  // `confirm()` is not always shown in the iOS WebView, so deletion is confirmed with a second tap.
  let armed: number | null = null;
  remove.addEventListener("click", async () => {
    const model = selected();
    if (!model) return;
    if (armed === null) {
      remove.textContent = `¿Borrar ${bytes(model.size)}? Toca otra vez`;
      armed = window.setTimeout(() => {
        armed = null;
        remove.textContent = "Borrar";
      }, 4000);
      return;
    }
    window.clearTimeout(armed);
    armed = null;
    remove.textContent = "Borrar";
    try {
      await chatDeleteModel(model.id);
    } catch (error) {
      meta.textContent = String(error);
    }
    await refresh();
    onChange();
  });

  return { selected, refresh };
}
