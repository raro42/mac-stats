// Pestaña Chat: conversación con el modelo que corre dentro del iPhone.
import {
  chatCancel,
  chatDelete,
  chatGet,
  chatList,
  chatSend,
  type Snapshot,
  type StoredMessage,
} from "../ipc";
import { onMetrics } from "../metrics-bus";
import { disableLinks, renderMarkdown } from "./markdown";
import { createModelCard } from "./model-card";

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Falta el elemento #${id}`);
  return el as T;
}

const STOP_LABEL: Record<string, string> = {
  cancelled: "Detenida",
  thermal: "Detenida: el iPhone está muy caliente",
  length: "Cortada: llegó al límite de longitud",
  context: "Cortada: la conversación ya no cabe",
  error: "Interrumpida por un error",
};

export interface ChatView {
  ask(question: string): Promise<void>;
}

export async function startChat(): Promise<ChatView> {
  const history = byId("chat-history");
  const form = byId<HTMLFormElement>("chat-form");
  const input = byId<HTMLTextAreaElement>("chat-input");
  const send = byId<HTMLButtonElement>("chat-send");
  const banner = byId("chat-banner");
  const convList = byId("conv-list");
  const empty = byId("chat-empty");

  let conversationId: string | null = null;
  let generating = false;
  disableLinks(history);

  const card = createModelCard(byId("model-card"), () => updateComposer());

  function updateComposer(): void {
    const ready = card.selected()?.installed ?? false;
    input.disabled = !ready && !generating;
    input.placeholder = ready ? "Pregunta algo…" : "Descarga un modelo para empezar";
    send.textContent = generating ? "Detener" : "Enviar";
    send.disabled = !generating && (!ready || input.value.trim() === "");
  }

  function bubble(role: "user" | "assistant" | "error", text = ""): HTMLDivElement {
    const node = document.createElement("div");
    node.className = `bubble bubble-${role}`;
    if (role === "assistant") node.innerHTML = renderMarkdown(text);
    else node.textContent = text;
    history.append(node);
    empty.hidden = true;
    return node;
  }

  function footnote(node: HTMLElement, text: string): void {
    const small = document.createElement("div");
    small.className = "bubble-meta";
    small.textContent = text;
    node.after(small);
  }

  function scrollToEnd(): void {
    window.scrollTo({ top: document.body.scrollHeight });
  }

  function showMessages(messages: StoredMessage[]): void {
    history.replaceChildren();
    empty.hidden = messages.length > 0;
    for (const m of messages) {
      const node = bubble(m.role, m.content);
      if (m.stats) {
        const reason = STOP_LABEL[m.stats.stopReason];
        footnote(node, [`${m.stats.tgTps.toFixed(1)} tok/s`, reason].filter(Boolean).join(" · "));
      }
    }
  }

  async function refreshConversations(): Promise<void> {
    const list = await chatList();
    convList.replaceChildren(
      ...list.map((c) => {
        const item = document.createElement("li");
        const open = document.createElement("button");
        open.type = "button";
        open.className = "conv-open";
        open.textContent = c.title;
        open.addEventListener("click", () => void openConversation(c.id));
        const del = document.createElement("button");
        del.type = "button";
        del.className = "conv-delete";
        del.textContent = "Borrar";
        del.setAttribute("aria-label", `Borrar «${c.title}»`);
        del.addEventListener("click", async () => {
          await chatDelete(c.id);
          if (c.id === conversationId) newConversation();
          await refreshConversations();
        });
        item.append(open, del);
        return item;
      }),
    );
  }

  async function openConversation(id: string): Promise<void> {
    const conversation = await chatGet(id);
    if (!conversation) return;
    conversationId = conversation.id;
    showMessages(conversation.messages);
    scrollToEnd();
  }

  function newConversation(): void {
    conversationId = null;
    showMessages([]);
  }

  function showBanner(snapshot: Snapshot): void {
    let text = "";
    if (snapshot.thermal === "critical") text = "El iPhone está muy caliente: el chat se pausa hasta que se enfríe.";
    else if (snapshot.thermal === "serious") text = "El iPhone está caliente: las respuestas irán más lentas.";
    else if (snapshot.lowPower) text = "Modo de bajo consumo activado: las respuestas pueden ir más lentas.";
    banner.textContent = text;
    banner.hidden = text === "";
  }

  async function ask(question: string): Promise<void> {
    generating = true;
    updateComposer();
    bubble("user", question);
    const answer = bubble("assistant");
    answer.classList.add("bubble-pending");
    answer.textContent = "…";
    scrollToEnd();

    let text = "";
    let scheduled = false;
    const paint = () => {
      scheduled = false;
      answer.innerHTML = renderMarkdown(text);
      scrollToEnd();
    };

    try {
      const result = await chatSend(conversationId, question, (event) => {
        if (event.type === "status") {
          answer.textContent = event.text;
        } else if (event.type === "delta") {
          text += event.text;
          answer.classList.remove("bubble-pending");
          if (!scheduled) {
            scheduled = true;
            setTimeout(paint, 80);
          }
        }
      });
      conversationId = result.conversationId;
      paint();
      answer.classList.remove("bubble-pending");
      const reason = STOP_LABEL[result.stopReason];
      footnote(answer, [`${result.tgTps.toFixed(1)} tok/s`, reason].filter(Boolean).join(" · "));
    } catch (error) {
      answer.remove();
      bubble("error", String(error));
    }
    generating = false;
    updateComposer();
    await refreshConversations();
    scrollToEnd();
  }

  input.addEventListener("input", () => {
    input.style.height = "auto";
    input.style.height = `${Math.min(input.scrollHeight, 140)}px`;
    updateComposer();
  });

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (generating) {
      void chatCancel();
      return;
    }
    const question = input.value.trim();
    if (!question) return;
    input.value = "";
    input.style.height = "auto";
    void ask(question);
  });

  byId("chat-new").addEventListener("click", newConversation);

  await card.refresh();
  updateComposer();
  await refreshConversations();
  const [latest] = await chatList();
  if (latest) await openConversation(latest.id);

  const snapshot = await onMetrics(showBanner);
  if (snapshot) showBanner(snapshot);
  return { ask };
}
