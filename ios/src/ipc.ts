// Contract with Rust (src-tauri/src/metrics/mod.rs). Field names arrive in camelCase.
import { Channel, invoke } from "@tauri-apps/api/core";

export type Thermal = "nominal" | "fair" | "serious" | "critical" | "unknown";
export type BatteryState = "unknown" | "unplugged" | "charging" | "full";
export type Range = "5m" | "1h";

export interface Snapshot {
  ts: number;
  cpu: number | null;
  ramUsed: number | null;
  ramTotal: number;
  appFootprint: number | null;
  appAvailable: number | null;
  netDown: number | null;
  netUp: number | null;
  battery: { level: number; state: BatteryState } | null;
  storage: { total: number; available: number } | null;
  thermal: Thermal;
  lowPower: boolean;
}

export interface HistoryPoint {
  ts: number;
  cpu: number | null;
  ram: number | null;
  appMb: number | null;
  /** In the 1 h view, the worst state of the minute. */
  thermal: Thermal | null;
  gap: boolean;
}

export interface DeviceInfo {
  /** Marketing name, or the raw identifier for unknown models; null if unreadable. */
  model: string | null;
  /** Translation key for names that change by language (`device.<modelKey>`). */
  modelKey: "iphoneSe3" | null;
  identifier: string | null;
  osVersion: string;
  cores: number;
  ramTotal: number;
  simulator: boolean;
}

/** Receives one reading per second; returns the latest one Rust already has. */
export async function subscribeMetrics(
  onSample: (snapshot: Snapshot) => void,
): Promise<Snapshot | null> {
  const channel = new Channel<Snapshot>();
  channel.onmessage = onSample;
  return invoke<Snapshot | null>("metrics_subscribe", { onSample: channel });
}

export function metricsHistory(range: Range): Promise<HistoryPoint[]> {
  return invoke<HistoryPoint[]>("metrics_history", { range });
}

export function deviceInfo(): Promise<DeviceInfo> {
  return invoke<DeviceInfo>("device_info");
}

// --- Model lab (phase A, debug only) ---

export interface LabModel {
  id: string;
  name: string;
  file: string;
  size: number;
  license: string;
  thinkPrefill: boolean;
  installed: boolean;
}

export interface LoadInfo {
  loadMs: number;
  description: string;
  nParams: number;
  sizeBytes: number;
  nCtx: number;
  gpu: boolean;
  availableMemory: number;
}

export interface BenchResult {
  ppTps: number;
  tgTps: number;
  availableMemory: number;
}

export interface GenerateResult {
  stopReason: string;
  nPrompt: number;
  nCached: number;
  nGen: number;
  ppTps: number;
  tgTps: number;
  availableMemory: number;
}

export const labModels = () => invoke<LabModel[]>("lab_models");
export const labLoad = (id: string) => invoke<LoadInfo>("lab_load", { id });
export const labUnload = () => invoke<void>("lab_unload");
export const labBench = () => invoke<BenchResult>("lab_bench");
export const labCancel = () => invoke<void>("lab_cancel");

export function labGenerate(
  prompt: string,
  thinkPrefill: boolean,
  onDelta: (text: string) => void,
): Promise<GenerateResult> {
  const channel = new Channel<{ type: string; text?: string }>();
  channel.onmessage = (event) => {
    if (event.type === "delta" && event.text) onDelta(event.text);
  };
  return invoke<GenerateResult>("lab_generate", { prompt, thinkPrefill, onEvent: channel });
}

// --- Chat (phase B) ---

export interface ChatModel {
  id: string;
  name: string;
  file: string;
  size: number;
  license: string;
  installed: boolean;
  selected: boolean;
  recommended: boolean;
}

export interface ChatStatus {
  loadedModel: string | null;
  selectedModel: string;
  engine: { loaded: boolean; busy: boolean; availableMemory: number };
}

export interface ReplyStats {
  modelId: string;
  nGen: number;
  tgTps: number;
  stopReason: string;
}

export interface StoredMessage {
  role: "user" | "assistant";
  content: string;
  ts: number;
  stats?: ReplyStats;
}

export interface Conversation {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
  messages: StoredMessage[];
}

export interface ConversationSummary {
  id: string;
  title: string;
  updatedAt: number;
  messageCount: number;
}

export interface SendResult {
  conversationId: string;
  stopReason: string;
  nGen: number;
  tgTps: number;
  /** Language the reply was asked for (BCP-47). */
  replyLanguage: string;
}

export type ChatEvent =
  | { type: "status"; code: "loading_model" }
  | { type: "delta"; text: string }
  | { type: "progress"; received: number; total: number }
  | { type: "verifying" };

function channelOf(onEvent: (event: ChatEvent) => void): Channel<ChatEvent> {
  const channel = new Channel<ChatEvent>();
  channel.onmessage = onEvent;
  return channel;
}

export const chatModels = () => invoke<ChatModel[]>("chat_models");
export const chatSelectModel = (id: string) => invoke<void>("chat_select_model", { id });
export const chatStatus = () => invoke<ChatStatus>("chat_status");
export const chatCancelDownload = () => invoke<void>("chat_cancel_download");
export const chatDeleteModel = (id: string) => invoke<void>("chat_delete_model", { id });
export const chatList = () => invoke<ConversationSummary[]>("chat_list");
export const chatGet = (id: string) => invoke<Conversation | null>("chat_get", { id });
export const chatDelete = (id: string) => invoke<void>("chat_delete", { id });
export const chatCancel = () => invoke<void>("chat_cancel");
export const debugBuild = () => invoke<boolean>("debug_build");

// --- Language ---

export interface AppLanguage {
  /** null = Automatic (follow iOS). */
  setting: string | null;
  /** Language the UI uses now. */
  resolved: string;
  /** What Automatic resolves to on this iPhone. */
  automatic: string;
  /** Full tag for number formats, e.g. `es-MX`. */
  locale: string | null;
}

export const appLanguage = () => invoke<AppLanguage>("app_language");

// --- Theme ---

/** `theme` is a desktop theme id, or null for System. */
export const appTheme = () => invoke<{ theme: string | null }>("app_theme");
export const setAppTheme = (theme: string | null) =>
  invoke<{ theme: string | null }>("set_app_theme", { theme });
export const setAppLanguage = (language: string | null) =>
  invoke<AppLanguage>("set_app_language", { language });
export const debugDemoPrompt = () => invoke<string | null>("debug_demo_prompt");
export const debugDemoView = () => invoke<string | null>("debug_demo_view");

export function chatDownload(
  id: string,
  allowCellular: boolean,
  onEvent: (event: ChatEvent) => void,
): Promise<void> {
  return invoke<void>("chat_download", { id, allowCellular, onEvent: channelOf(onEvent) });
}

export function chatSend(
  conversationId: string | null,
  text: string,
  onEvent: (event: ChatEvent) => void,
): Promise<SendResult> {
  return invoke<SendResult>("chat_send", { conversationId, text, onEvent: channelOf(onEvent) });
}
