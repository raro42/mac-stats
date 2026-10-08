// Progress ring. Adapted from the `.ring-gauge` in the Mac app themes
// (src-tauri/dist/themes/*/cpu.html): same radius-42 circle in a 100-unit viewBox,
// without inline styles (the CSP blocks them). The theme decides how much of the circle
// the arc covers (`--ring-arc`: 240° on Glass, full circle on Dark, top half on Neon…)
// and CSS rotates it (`--ring-start`).

import { t } from "../i18n";

const SVG_NS = "http://www.w3.org/2000/svg";
const RADIUS = 42;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

/** Arc length for the active theme. Read once: a theme change reloads the page. */
let arcLength: number | null = null;
function arc(): number {
  if (arcLength == null) {
    const share = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--ring-arc"));
    arcLength = CIRCUMFERENCE * (share > 0 && share <= 1 ? share : 0.75);
  }
  return arcLength;
}

export type Level = "ok" | "warn" | "crit";

function circle(cls: string): SVGCircleElement {
  const c = document.createElementNS(SVG_NS, "circle");
  c.setAttribute("class", cls);
  c.setAttribute("cx", "50");
  c.setAttribute("cy", "50");
  c.setAttribute("r", String(RADIUS));
  return c;
}

/** Changes smaller than this jump without animating (saves repaints every second). */
const ANIMATE_FROM = 0.03;

export class RingGauge {
  private readonly root: HTMLDivElement;
  /** Last drawn fraction, rounded to whole percent; -1 before the first value. */
  private drawn = -1;
  private spoken = "";
  private readonly progress: SVGCircleElement;
  private readonly valueEl: HTMLSpanElement;
  private readonly subEl: HTMLDivElement;

  constructor(
    container: HTMLElement,
    private readonly label: string,
    accentClass: string,
  ) {
    this.root = document.createElement("div");
    this.root.className = `ring ${accentClass}`;
    this.root.setAttribute("role", "img");

    const svg = document.createElementNS(SVG_NS, "svg");
    svg.setAttribute("viewBox", "0 0 100 100");
    svg.setAttribute("aria-hidden", "true");
    const track = circle("ring-track");
    track.setAttribute("stroke-dasharray", `${arc()} ${CIRCUMFERENCE}`);
    this.progress = circle("ring-progress");
    this.progress.setAttribute("stroke-dasharray", `0 ${CIRCUMFERENCE}`);
    svg.append(track, this.progress);

    const center = document.createElement("div");
    center.className = "ring-center";
    this.valueEl = document.createElement("span");
    this.valueEl.className = "ring-value";
    this.valueEl.textContent = "—";
    const labelEl = document.createElement("span");
    labelEl.className = "ring-label";
    labelEl.textContent = label;
    center.append(this.valueEl, labelEl);

    this.subEl = document.createElement("div");
    this.subEl.className = "ring-sub";
    this.subEl.textContent = t("monitor.measuring");

    const dial = document.createElement("div");
    dial.className = "ring-dial";
    dial.append(svg, center);
    this.root.append(dial, this.subEl);
    this.root.dataset.empty = "true";
    container.append(this.root);
    this.root.setAttribute("aria-label", t("monitor.gaugeEmpty", { label }));
  }

  /** The ring's card, for extras such as the Data Poster mini charts. */
  get element(): HTMLDivElement {
    return this.root;
  }

  /** `fraction` between 0 and 1, or `null` when there is no data. */
  /**
   * Called every second, so it only touches the DOM when something visible changed: the
   * arc moves in whole-percent steps and only animates jumps of 3 points or more. Most of
   * the Monitor's CPU went into these SVG repaints (ios/docs/test-results.md).
   */
  set(fraction: number | null, value: string, sub: string, level: Level = "ok"): void {
    const f = fraction == null ? 0 : Math.round(Math.min(1, Math.max(0, fraction)) * 100) / 100;
    if (f !== this.drawn) {
      const jump = this.drawn < 0 || Math.abs(f - this.drawn) >= ANIMATE_FROM;
      this.progress.classList.toggle("ring-instant", !jump);
      this.progress.setAttribute("stroke-dasharray", `${arc() * f} ${CIRCUMFERENCE}`);
      this.drawn = f;
    }
    const levelState = fraction == null ? "none" : level;
    if (this.root.dataset.level !== levelState) this.root.dataset.level = levelState;
    const empty = String(f === 0); // at 0 the round line cap would draw a dot
    if (this.root.dataset.empty !== empty) this.root.dataset.empty = empty;
    if (this.valueEl.textContent !== value) this.valueEl.textContent = value;
    if (this.subEl.textContent !== sub) this.subEl.textContent = sub;
    const spoken = sub
      ? t("monitor.gaugeValueDetail", { label: this.label, value, detail: sub })
      : t("monitor.gaugeValue", { label: this.label, value });
    if (spoken !== this.spoken) {
      this.root.setAttribute("aria-label", spoken);
      this.spoken = spoken;
    }
  }
}
