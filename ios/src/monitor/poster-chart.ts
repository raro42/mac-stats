// Data Poster mini charts: a 12-bar chart and a 60-point line chart per tile, ported from
// the desktop theme (src-tauri/dist/themes/data-poster/poster-charts.js): same buffer
// sizes, moving average of 5 for display, auto-scaling max with 10% headroom, and the
// tile color at 0.9 (bars) / 0.8 (line) / 0.15 (fill) opacity.

const BAR_COUNT = 12;
const LINE_POINTS = 60;
const SMOOTH_WINDOW = 5;

function movingAverage(values: number[], window: number): number[] {
  return values.map((_, i) => {
    const slice = values.slice(Math.max(0, i - window + 1), i + 1);
    return slice.reduce((a, b) => a + b, 0) / slice.length;
  });
}

function sized(canvas: HTMLCanvasElement): { ctx: CanvasRenderingContext2D; width: number; height: number } | null {
  const ctx = canvas.getContext("2d");
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  if (!ctx || width === 0 || height === 0) return null;
  const dpr = window.devicePixelRatio || 1;
  if (canvas.width !== Math.round(width * dpr)) canvas.width = Math.round(width * dpr);
  if (canvas.height !== Math.round(height * dpr)) canvas.height = Math.round(height * dpr);
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);
  return { ctx, width, height };
}

export class PosterChart {
  private readonly bars: HTMLCanvasElement;
  private readonly line: HTMLCanvasElement;
  private barValues = new Array<number>(BAR_COUNT).fill(0);
  private lineValues = new Array<number>(LINE_POINTS).fill(0);
  private max = 100;
  private min = 0;

  /** Adds the charts to a ring card; `colorVar` is the tile's metric color token. */
  constructor(
    tile: HTMLElement,
    private readonly colorVar: string,
  ) {
    const box = document.createElement("div");
    box.className = "poster-charts";
    box.setAttribute("aria-hidden", "true");
    this.bars = document.createElement("canvas");
    this.bars.className = "poster-bars";
    this.line = document.createElement("canvas");
    this.line.className = "poster-line";
    box.append(this.bars, this.line);
    tile.append(box);
    new ResizeObserver(() => this.draw()).observe(box);
  }

  /** Seeds the charts from history (oldest first), e.g. after the page loads. */
  seed(values: (number | null)[]): void {
    for (const v of values.slice(-LINE_POINTS)) this.add(v, false);
    this.draw();
  }

  push(value: number | null): void {
    this.add(value, true);
  }

  private add(value: number | null, redraw: boolean): void {
    if (value == null || Number.isNaN(value)) return;
    if (value > this.max) this.max = value * 1.1;
    if (value < this.min) this.min = Math.max(0, value * 0.9);
    this.barValues = [...this.barValues.slice(1), value];
    this.lineValues = [...this.lineValues.slice(1), value];
    if (redraw) this.draw();
  }

  private color(): string {
    return getComputedStyle(this.bars).getPropertyValue(this.colorVar).trim() || "#55c7ff";
  }

  private draw(): void {
    const bars = sized(this.bars);
    if (bars) {
      const { ctx, width, height } = bars;
      const barWidth = width / BAR_COUNT;
      ctx.fillStyle = this.color();
      ctx.globalAlpha = 0.9;
      movingAverage(this.barValues, SMOOTH_WINDOW).forEach((v, i) => {
        const h = (v / (this.max || 1)) * height;
        ctx.fillRect(i * barWidth + 1, height - h, barWidth - 2, h);
      });
      ctx.globalAlpha = 1;
    }

    const line = sized(this.line);
    if (line) {
      const { ctx, width, height } = line;
      const range = this.max - this.min || 1;
      const points = movingAverage(this.lineValues, SMOOTH_WINDOW).map((v, i) => ({
        x: (i / (LINE_POINTS - 1)) * width,
        y: height - ((v - this.min) / range) * height,
      }));
      ctx.beginPath();
      ctx.moveTo(points[0].x, height);
      for (const p of points) ctx.lineTo(p.x, p.y);
      ctx.lineTo(points[points.length - 1].x, height);
      ctx.closePath();
      const color = this.color();
      ctx.fillStyle = color;
      ctx.globalAlpha = 0.15;
      ctx.fill();
      ctx.beginPath();
      ctx.moveTo(points[0].x, points[0].y);
      for (const p of points) ctx.lineTo(p.x, p.y);
      ctx.strokeStyle = color;
      ctx.globalAlpha = 0.8;
      ctx.lineWidth = 2;
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      ctx.stroke();
      ctx.globalAlpha = 1;
    }
  }
}
