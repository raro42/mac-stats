// Line chart on a <canvas>. Adapted from src/chart-line.js in the Mac app, which
// relied on fixed IDs and `window` globals; here it is a reusable class that places
// points by time and breaks the line at gaps.

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
    const ctx = this.ctx;
    const width = this.canvas.clientWidth;
    const height = this.canvas.clientHeight;
    if (!ctx || width === 0 || height === 0) return;

    const dpr = window.devicePixelRatio || 1;
    if (this.canvas.width !== Math.round(width * dpr) || this.canvas.height !== Math.round(height * dpr)) {
      this.canvas.width = Math.round(width * dpr);
      this.canvas.height = Math.round(height * dpr);
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

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

    const end = this.data.length ? this.data[this.data.length - 1].ts : Date.now();
    const start = end - this.windowMs;
    const pad = 2;
    const x = (ts: number) => ((ts - start) / this.windowMs) * width;
    const y = (v: number) => pad + (1 - Math.min(this.max, Math.max(0, v)) / this.max) * (height - 2 * pad);

    const segments: Array<Array<[number, number]>> = [];
    let current: Array<[number, number]> = [];
    let prevTs: number | null = null;
    for (const p of this.data) {
      if (p.ts < start) continue;
      const valid = !p.gap && p.v != null;
      const tooFar = prevTs != null && p.ts - prevTs > this.stepMs * 2.5;
      if ((!valid || tooFar) && current.length) {
        segments.push(current);
        current = [];
      }
      if (valid) {
        current.push([x(p.ts), y(p.v as number)]);
        prevTs = p.ts;
      } else {
        prevTs = null;
      }
    }
    if (current.length) segments.push(current);

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
