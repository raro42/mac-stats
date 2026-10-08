import { bytes, duration, joined, percent, rate } from "../format";
import { t, tp, type TextKey } from "../i18n";
import {
  deviceInfo,
  metricsHistory,
  type BatteryState,
  type Range,
  type Snapshot,
  type Thermal,
} from "../ipc";
import { onMetrics } from "../metrics-bus";
import { activeTheme } from "../themes";
import { LineChart } from "./line-chart";
import { PosterChart } from "./poster-chart";
import { RingGauge, type Level } from "./ring-gauge";
import { ThermalStrip } from "./thermal-strip";

const THERMAL_LABEL: Record<Thermal, TextKey> = {
  nominal: "thermal.nominal",
  fair: "thermal.fair",
  serious: "thermal.serious",
  critical: "thermal.critical",
  unknown: "thermal.unknown",
};

/** What each thermal state means, shown under the thermal scale. */
const THERMAL_MEANING: Record<Thermal, TextKey | null> = {
  nominal: "thermal.meaning.nominal",
  fair: "thermal.meaning.fair",
  serious: "thermal.meaning.serious",
  critical: "thermal.meaning.critical",
  unknown: null,
};

const THERMAL_ORDER: Thermal[] = ["nominal", "fair", "serious", "critical"];

const BATTERY_LABEL: Record<BatteryState, TextKey | null> = {
  charging: "battery.charging",
  full: "battery.full",
  unplugged: "battery.unplugged",
  unknown: null,
};

const WINDOWS: Record<Range, { windowMs: number; stepMs: number }> = {
  "5m": { windowMs: 5 * 60_000, stepMs: 1_000 },
  "1h": { windowMs: 60 * 60_000, stepMs: 60_000 },
};

const LIVE_POINTS = 300;
const HOUR_REFRESH_MS = 15_000;

/** Higher is worse (CPU, RAM, disk). */
function levelUp(value: number, warn: number, crit: number): Level {
  return value >= crit ? "crit" : value >= warn ? "warn" : "ok";
}

/** Lower is worse (battery). */
function levelDown(value: number, warn: number, crit: number): Level {
  return value <= crit ? "crit" : value <= warn ? "warn" : "ok";
}

/** Skips DOM writes when the text did not change (the monitor updates every second). */
function setText(el: HTMLElement, text: string): void {
  if (el.textContent !== text) el.textContent = text;
}

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}

export async function startMonitor(): Promise<void> {
  const ringsEl = byId("rings");
  const rings = {
    cpu: new RingGauge(ringsEl, t("metric.cpu"), "accent-cpu"),
    ram: new RingGauge(ringsEl, t("metric.ram"), "accent-ram"),
    disk: new RingGauge(ringsEl, t("metric.storage"), "accent-disk"),
    battery: new RingGauge(ringsEl, t("metric.battery"), "accent-battery"),
  };
  const thermalEl = byId("thermal");
  const thermalCard = byId("thermal-card");
  const thermalLevel = byId("thermal-level");
  const thermalMeaning = byId("thermal-meaning");
  const thermalSteps = Array.from(thermalCard.querySelectorAll<HTMLElement>(".thermal-step"));
  const lowPowerEl = byId("low-power");
  const appMemBar = byId("appmem-bar");
  const appMemFill = byId("appmem-fill");
  const appMemText = byId("appmem-text");
  const netDownEl = byId("net-down");
  const netUpEl = byId("net-up");
  const charts = {
    cpu: new LineChart(byId<HTMLCanvasElement>("chart-cpu"), "--cpu"),
    ram: new LineChart(byId<HTMLCanvasElement>("chart-ram"), "--ram"),
    thermal: new ThermalStrip(byId<HTMLCanvasElement>("chart-thermal")),
  };
  // Data Poster shows mini bar/line charts in the CPU and RAM tiles instead of rings.
  const posters = activeTheme()?.poster
    ? { cpu: new PosterChart(rings.cpu.element, "--cpu"), ram: new PosterChart(rings.ram.element, "--ram") }
    : null;

  let range: Range = "5m";
  let simulator = false;

  async function loadHistory(): Promise<void> {
    const points = await metricsHistory(range);
    const { windowMs, stepMs } = WINDOWS[range];
    charts.cpu.setWindow(windowMs, stepMs);
    charts.ram.setWindow(windowMs, stepMs);
    charts.thermal.setWindow(windowMs, stepMs);
    charts.cpu.setData(points.map((p) => ({ ts: p.ts, v: p.cpu, gap: p.gap })));
    charts.ram.setData(points.map((p) => ({ ts: p.ts, v: p.ram, gap: p.gap })));
    charts.thermal.setData(points.map((p) => ({ ts: p.ts, thermal: p.thermal, gap: p.gap })));
  }

  let shownThermal: Thermal | null = null;

  function renderThermal(state: Thermal): void {
    if (state === shownThermal) return; // updated every second; changes are rare
    shownThermal = state;
    const label = t(THERMAL_LABEL[state]);
    const meaningKey = THERMAL_MEANING[state];
    const meaning = meaningKey ? t(meaningKey) : "";
    const spoken = t("thermal.label", { level: label });
    thermalEl.textContent = label;
    thermalEl.dataset.state = state;
    thermalEl.setAttribute("aria-label", spoken);
    thermalCard.dataset.state = state;
    thermalCard.setAttribute("aria-label", meaning ? `${spoken}. ${meaning}` : spoken);
    thermalLevel.textContent = label;
    thermalMeaning.textContent = meaning;
    const current = THERMAL_ORDER.indexOf(state);
    thermalSteps.forEach((step, i) => {
      step.classList.toggle("is-current", i === current);
      step.classList.toggle("is-passed", current >= 0 && i < current);
    });
  }

  function render(s: Snapshot): void {
    if (s.cpu != null) {
      rings.cpu.set(s.cpu / 100, percent(s.cpu), t("monitor.cpuDetail"), levelUp(s.cpu, 50, 85));
    }

    const ramPct = s.ramUsed != null && s.ramTotal > 0 ? (s.ramUsed / s.ramTotal) * 100 : null;
    if (ramPct != null && s.ramUsed != null) {
      rings.ram.set(
        ramPct / 100,
        percent(ramPct),
        t("monitor.usedOfTotal", { used: bytes(s.ramUsed), total: bytes(s.ramTotal) }),
        levelUp(ramPct, 85, 95),
      );
    }

    if (s.storage) {
      const usedPct = ((s.storage.total - s.storage.available) / s.storage.total) * 100;
      rings.disk.set(
        usedPct / 100,
        percent(usedPct),
        t("monitor.free", { free: bytes(s.storage.available) }),
        levelUp(usedPct, 85, 95),
      );
    }

    if (s.battery) {
      const pct = s.battery.level * 100;
      const pluggedIn = s.battery.state === "charging" || s.battery.state === "full";
      const stateKey = BATTERY_LABEL[s.battery.state];
      rings.battery.set(
        s.battery.level,
        percent(pct),
        stateKey ? t(stateKey) : "",
        pluggedIn ? "ok" : levelDown(pct, 20, 10),
      );
    } else {
      rings.battery.set(null, "—", t(simulator ? "monitor.noBatterySimulator" : "monitor.unavailable"));
    }

    renderThermal(s.thermal);
    lowPowerEl.hidden = !s.lowPower;

    if (s.appFootprint != null) {
      if (s.appAvailable != null) {
        const share = s.appFootprint / (s.appFootprint + s.appAvailable);
        appMemFill.style.width = `${(share * 100).toFixed(1)}%`;
        appMemBar.setAttribute("aria-valuenow", (share * 100).toFixed(0));
        appMemBar.setAttribute("aria-valuetext", percent(share * 100));
        setText(
          appMemText,
          t("monitor.appMemoryUsed", { used: bytes(s.appFootprint), headroom: bytes(s.appAvailable) }),
        );
      } else {
        appMemFill.style.width = "0%";
        appMemText.textContent = t("monitor.appMemorySimulator", { used: bytes(s.appFootprint) });
      }
    }

    setText(netDownEl, s.netDown != null ? rate(s.netDown) : "—");
    setText(netUpEl, s.netUp != null ? rate(s.netUp) : "—");

    if (range === "5m") {
      charts.cpu.push({ ts: s.ts, v: s.cpu }, LIVE_POINTS);
      charts.ram.push({ ts: s.ts, v: ramPct }, LIVE_POINTS);
      charts.thermal.push({ ts: s.ts, thermal: s.thermal }, LIVE_POINTS);
    }
    posters?.cpu.push(s.cpu);
    posters?.ram.push(ramPct);
  }

  const rangeButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-range]"));
  for (const button of rangeButtons) {
    button.textContent = button.dataset.range === "1h" ? duration(1, "hour") : duration(5, "minute");
    button.addEventListener("click", () => {
      range = button.dataset.range as Range;
      for (const b of rangeButtons) b.setAttribute("aria-pressed", String(b === button));
      void loadHistory();
    });
  }

  const info = await deviceInfo();
  simulator = info.simulator;
  const model = info.modelKey ? t(`device.${info.modelKey}`) : (info.model ?? t("device.unknown"));
  byId("device").textContent = joined([
    info.simulator ? t("monitor.simulator", { model }) : model,
    t("monitor.osVersion", { version: info.osVersion }),
    tp("monitor.cores", info.cores),
    t("monitor.ramTotal", { size: bytes(info.ramTotal) }),
  ]);

  if (posters) {
    const recent = await metricsHistory("5m");
    posters.cpu.seed(recent.map((p) => p.cpu));
    posters.ram.seed(recent.map((p) => p.ram));
  }
  const latest = await onMetrics(render);
  await loadHistory();
  if (latest) render(latest);

  window.setInterval(() => {
    if (range === "1h" && !document.hidden) void loadHistory();
  }, HOUR_REFRESH_MS);
}
