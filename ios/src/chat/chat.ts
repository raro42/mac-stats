// Chat tab: conversation with the model running on the iPhone itself.
import {
  chatCancel,
  chatDelete,
  chatGet,
  chatList,
  chatSend,
  type Snapshot,
  type StoredMessage,
} from "../ipc";
import { joined, tokensPerSecond } from "../format";
import { t, type TextKey } from "../i18n";
import { errorText } from "../i18n/errors";
import { onMetrics } from "../metrics-bus";
import { disableLinks, renderMarkdown } from "./markdown";
import { createModelCard } from "./model-card";

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}

/** Why a reply stopped; `eos` (finished normally) shows no label in the chat. */
const STOP_LABEL: Record<string, TextKey> = {
  cancelled: "stop.cancelled",
  thermal: "stop.thermal",
  length: "stop.length",
  error: "stop.error",
};

function stats(tgTps: number, stopReason: string): string {
  const reason = STOP_LABEL[stopReason];
  return joined([tokensPerSecond(tgTps), reason && t(reason)]);
}

export interface ChatView {
  ask(question: string): Promise<void>;
  /** True while a reply is being written. */
  generating(): boolean;
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
    input.placeholder = t(ready ? "chat.placeholder" : "chat.placeholderNoModel");
    send.textContent = t(generating ? "chat.stop" : "chat.send");
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

  /** Placeholder in the reply bubble; VoiceOver hears "writing the reply" instead of "…". */
  function showPending(node: HTMLElement, visible: string): void {
    const shown = document.createElement("span");
    shown.setAttribute("aria-hidden", "true");
    shown.textContent = visible;
    const spoken = document.createElement("span");
    spoken.className = "sr-only";
    spoken.textContent = visible === "…" ? t("chat.pending") : visible;
    node.replaceChildren(shown, spoken);
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
      if (m.stats) footnote(node, stats(m.stats.tgTps, m.stats.stopReason));
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
        const title = c.title || t("chat.untitled");
        open.textContent = title;
        open.addEventListener("click", () => void openConversation(c.id));
        const del = document.createElement("button");
        del.type = "button";
        del.className = "conv-delete";
        del.textContent = t("common.delete");
        del.setAttribute("aria-label", t("chat.deleteConversation", { title }));
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
    if (snapshot.thermal === "critical") text = t("chat.bannerCritical");
    else if (snapshot.thermal === "serious") text = t("chat.bannerSerious");
    else if (snapshot.lowPower) text = t("chat.bannerLowPower");
    banner.textContent = text;
    banner.hidden = text === "";
  }

  async function ask(question: string): Promise<void> {
    generating = true;
    updateComposer();
    bubble("user", question);
    const answer = bubble("assistant");
    answer.classList.add("bubble-pending");
    showPending(answer, "…");
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
          showPending(answer, t("chat.loadingModel"));
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
      footnote(answer, stats(result.tgTps, result.stopReason));
    } catch (error) {
      answer.remove();
      bubble("error", errorText(error));
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
  return { ask, generating: () => generating };
}
