// Thermal history strip: one colored block per point under the CPU/RAM charts, so it is
// easy to see when the iPhone was hot. Colors come from the theme's --thermal-* tokens
// (the desktop's thermal state colors).
import { t, type TextKey } from "../i18n";
import type { Thermal } from "../ipc";
import { prepareCanvas, runs, windowStart } from "./time-axis";

export interface ThermalSample {
  ts: number;
  thermal: Thermal | null;
  gap?: boolean;
}

type Known = Exclude<Thermal, "unknown">;

const RANK: Record<Known, number> = { nominal: 1, fair: 2, serious: 3, critical: 4 };

const COLOR_VAR: Record<Known, string> = {
  nominal: "--thermal-nominal",
  fair: "--thermal-fair",
  serious: "--thermal-serious",
  critical: "--thermal-critical",
};

const known = (t: Thermal | null): t is Known => t != null && t !== "unknown";

export class ThermalStrip {
  private readonly ctx: CanvasRenderingContext2D | null;
  private data: ThermalSample[] = [];
  private windowMs = 5 * 60_000;
  private stepMs = 1_000;

  constructor(private readonly canvas: HTMLCanvasElement) {
    this.ctx = canvas.getContext("2d");
    new ResizeObserver(() => this.draw()).observe(canvas);
  }

  setWindow(windowMs: number, stepMs: number): void {
    this.windowMs = windowMs;
    this.stepMs = stepMs;
  }

  setData(data: ThermalSample[]): void {
    this.data = data;
    this.draw();
  }

  push(sample: ThermalSample, cap: number): void {
    this.data.push(sample);
    if (this.data.length > cap) this.data.splice(0, this.data.length - cap);
    this.draw();
  }

  draw(): void {
    const start = windowStart(this.data, this.windowMs);
    const visible = runs(this.data, start, this.stepMs, (p) => known(p.thermal));
    this.describe(visible.flat());

    const canvas = prepareCanvas(this.canvas, this.ctx);
    if (!canvas) return;
    const { ctx, width, height } = canvas;
    const style = getComputedStyle(this.canvas);
    const x = (ts: number) => ((ts - start) / this.windowMs) * width;

    ctx.fillStyle = style.getPropertyValue("--track").trim() || "rgba(128,128,128,0.2)";
    ctx.fillRect(0, 0, width, height);

    for (const run of visible) {
      run.forEach((p, i) => {
        const next = run[i + 1]?.ts ?? p.ts + this.stepMs;
        const x0 = x(p.ts);
        ctx.fillStyle = style.getPropertyValue(COLOR_VAR[p.thermal as Known]).trim();
        ctx.fillRect(x0, 0, Math.max(1, x(next) - x0), height);
      });
    }
  }

  /** VoiceOver hears the worst state in the visible window. */
  private describe(points: ThermalSample[]): void {
    let worst: Known | null = null;
    for (const p of points) {
      if (known(p.thermal) && (!worst || RANK[p.thermal] > RANK[worst])) worst = p.thermal;
    }
    this.canvas.setAttribute(
      "aria-label",
      worst
        ? t("monitor.thermalHistory", { level: t(`thermal.${worst}` as TextKey) })
        : t("monitor.thermalHistoryEmpty"),
    );
  }
}
