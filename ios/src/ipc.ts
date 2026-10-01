// Contrato con Rust (src-tauri/src/metrics/mod.rs). Los nombres llegan en camelCase.
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
  gap: boolean;
}

export interface DeviceInfo {
  model: string;
  identifier: string;
  osVersion: string;
  cores: number;
  ramTotal: number;
  simulator: boolean;
}

/** Recibe una lectura por segundo; devuelve la última que ya tenga Rust. */
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

// --- Laboratorio de modelos (fase A, solo depuración) ---

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
