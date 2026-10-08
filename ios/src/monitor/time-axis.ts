// Shared by the history canvases (line charts, thermal strip): canvas sizing and the
// split of time-ordered points into runs that the drawing must not join across.

export interface Timed {
  ts: number;
  gap?: boolean;
}

/** Sizes the canvas for its CSS size and the device pixel ratio; `null` if not laid out. */
export function prepareCanvas(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D | null,
): { ctx: CanvasRenderingContext2D; width: number; height: number } | null {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  if (!ctx || width === 0 || height === 0) return null;
  const dpr = window.devicePixelRatio || 1;
  if (canvas.width !== Math.round(width * dpr) || canvas.height !== Math.round(height * dpr)) {
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);
  return { ctx, width, height };
}

/**
 * Start of the visible window. It ends now, so time without data (the app closed) shows
 * as an empty stretch at the end instead of stretching the last points to the edge.
 */
export function windowStart(data: Timed[], windowMs: number): number {
  const newest = data.length ? data[data.length - 1].ts : 0;
  return Math.max(Date.now(), newest) - windowMs;
}

/**
 * Splits points into runs: a gap point, an invalid point or a jump of more than 2.5
 * steps (the app was frozen in the background) breaks the run. Points before `start`
 * are dropped.
 */
export function runs<T extends Timed>(data: T[], start: number, stepMs: number, valid: (p: T) => boolean): T[][] {
  const out: T[][] = [];
  let current: T[] = [];
  let prevTs: number | null = null;
  for (const p of data) {
    if (p.ts < start) continue;
    const ok = !p.gap && valid(p);
    const tooFar = prevTs != null && p.ts - prevTs > stepMs * 2.5;
    if ((!ok || tooFar) && current.length) {
      out.push(current);
      current = [];
    }
    if (ok) {
      current.push(p);
      prevTs = p.ts;
    } else {
      prevTs = null;
    }
  }
  if (current.length) out.push(current);
  return out;
}
