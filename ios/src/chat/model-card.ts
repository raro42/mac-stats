// Model card: pick a model from the catalog, download it (with progress and SHA-256
// verification in Swift), cancel the download, or delete it.
import { bytes, percent } from "../format";
import { t } from "../i18n";
import { errorText } from "../i18n/errors";
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
  select.setAttribute("aria-label", t("model.title"));
  const meta = el("p", "card-text");
  const progress = el("div", "bar");
  progress.setAttribute("role", "progressbar");
  progress.setAttribute("aria-valuemin", "0");
  progress.setAttribute("aria-valuemax", "100");
  progress.setAttribute("aria-label", t("model.downloadProgress"));
  const fill = el("div", "bar-fill");
  progress.append(fill);
  progress.hidden = true;
  const actions = el("div", "model-actions");
  const cellular = el("label", "model-cellular");
  const cellularBox = el("input");
  cellularBox.type = "checkbox";
  cellular.append(cellularBox, el("span", "", t("model.cellular")));
  const primary = el("button", "lab-btn model-primary");
  primary.type = "button";
  const remove = el("button", "lab-btn");
  remove.type = "button";
  remove.textContent = t("common.delete");
  actions.append(primary, remove);
  root.replaceChildren(el("h2", "", t("model.title")), select, meta, progress, cellular, actions);

  const selected = () => models.find((m) => m.selected) ?? null;

  function render(): void {
    const model = selected();
    select.replaceChildren(
      ...models.map((m) => {
        const option = el(
          "option",
          "",
          t(m.recommended ? "model.optionRecommended" : "model.option", { name: m.name, size: bytes(m.size) }),
        );
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
        ? t("model.ready", { license: model.license })
        : t("model.notInstalled", { size: bytes(model.size), license: model.license });
    }
    cellular.hidden = installed || downloading;
    primary.hidden = installed && !downloading;
    primary.textContent = t(downloading ? "common.cancel" : "model.download");
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
    meta.textContent = t("model.preparing");
    render();
    try {
      await chatDownload(model.id, cellularBox.checked, (event) => {
        if (event.type === "progress" && event.total > 0) {
          const share = event.received / event.total;
          fill.style.width = `${(share * 100).toFixed(1)}%`;
          progress.setAttribute("aria-valuenow", (share * 100).toFixed(0));
          progress.setAttribute("aria-valuetext", percent(share * 100));
          meta.textContent = t("model.downloading", {
            percent: percent(share * 100),
            received: bytes(event.received),
            total: bytes(event.total),
          });
        } else if (event.type === "verifying") {
          meta.textContent = t("model.verifying");
        }
      });
      downloading = false;
      await refresh();
      onChange();
    } catch (error) {
      downloading = false;
      render();
      meta.textContent = errorText(error);
    }
  });

  // `confirm()` is not always shown in the iOS WebView, so deletion is confirmed with a second tap.
  let armed: number | null = null;
  remove.addEventListener("click", async () => {
    const model = selected();
    if (!model) return;
    if (armed === null) {
      remove.textContent = t("model.confirmDelete", { size: bytes(model.size) });
      armed = window.setTimeout(() => {
        armed = null;
        remove.textContent = t("common.delete");
      }, 4000);
      return;
    }
    window.clearTimeout(armed);
    armed = null;
    remove.textContent = t("common.delete");
    try {
      await chatDeleteModel(model.id);
    } catch (error) {
      meta.textContent = errorText(error);
    }
    await refresh();
    onChange();
  });

  return { selected, refresh };
}
