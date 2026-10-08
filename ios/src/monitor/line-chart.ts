// Line chart on a <canvas>. Adapted from src/chart-line.js in the Mac app, which
// relied on fixed IDs and `window` globals; here it is a reusable class that places
// points by time and breaks the line at gaps.

import { prepareCanvas, runs, windowStart } from "./time-axis";

export interface Sample {
  ts: number;
  v: number | null;
  gap?: boolean;
}

export class LineChart {
  private readonly ctx: CanvasRenderingContext2D | null;
  private data: Sample[] = [];
  private windowMs = 5 * 60_000;
  private stepMs = 1_000;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly colorVar: string,
    private readonly max = 100,
  ) {
    this.ctx = canvas.getContext("2d");
    new ResizeObserver(() => this.draw()).observe(canvas);
  }

  /** Visible window and expected spacing between points (used to detect gaps). */
  setWindow(windowMs: number, stepMs: number): void {
    this.windowMs = windowMs;
    this.stepMs = stepMs;
  }

  setData(data: Sample[]): void {
    this.data = data;
    this.draw();
  }

  push(sample: Sample, cap: number): void {
    this.data.push(sample);
    if (this.data.length > cap) this.data.splice(0, this.data.length - cap);
    this.draw();
  }

  draw(): void {
    const canvas = prepareCanvas(this.canvas, this.ctx);
    if (!canvas) return;
    const { ctx, width, height } = canvas;

    const style = getComputedStyle(this.canvas);
    const color = style.getPropertyValue(this.colorVar).trim() || "#5ac8fa";
    const grid = style.getPropertyValue("--grid").trim() || "rgba(128,128,128,0.2)";

    ctx.strokeStyle = grid;
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 4]);
    ctx.beginPath();
    ctx.moveTo(0, Math.round(height / 2) + 0.5);
    ctx.lineTo(width, Math.round(height / 2) + 0.5);
    ctx.stroke();
    ctx.setLineDash([]);

    const start = windowStart(this.data, this.windowMs);
    const pad = 2;
    const x = (ts: number) => ((ts - start) / this.windowMs) * width;
    const y = (v: number) => pad + (1 - Math.min(this.max, Math.max(0, v)) / this.max) * (height - 2 * pad);

    const segments = runs(this.data, start, this.stepMs, (p) => p.v != null).map((run) =>
      run.map((p): [number, number] => [x(p.ts), y(p.v as number)]),
    );

    for (const seg of segments) {
      if (seg.length === 1) {
        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.arc(seg[0][0], seg[0][1], 1.5, 0, Math.PI * 2);
        ctx.fill();
        continue;
      }
      ctx.beginPath();
      ctx.moveTo(seg[0][0], height);
      for (const [px, py] of seg) ctx.lineTo(px, py);
      ctx.lineTo(seg[seg.length - 1][0], height);
      ctx.closePath();
      ctx.globalAlpha = 0.15;
      ctx.fillStyle = color;
      ctx.fill();
      ctx.globalAlpha = 1;

      ctx.beginPath();
      ctx.moveTo(seg[0][0], seg[0][1]);
      for (const [px, py] of seg.slice(1)) ctx.lineTo(px, py);
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.5;
      ctx.lineJoin = "round";
      ctx.stroke();
    }
  }
}
