import { bytes, percent, rate } from "../format";
import {
  deviceInfo,
  metricsHistory,
  type BatteryState,
  type Range,
  type Snapshot,
  type Thermal,
} from "../ipc";
import { onMetrics } from "../metrics-bus";
import { LineChart } from "./line-chart";
import { RingGauge, type Level } from "./ring-gauge";

const THERMAL_LABEL: Record<Thermal, string> = {
  nominal: "Normal",
  fair: "Templado",
  serious: "Caliente",
  critical: "Crítico",
  unknown: "Desconocido",
};

const BATTERY_LABEL: Record<BatteryState, string> = {
  charging: "Cargando",
  full: "Llena",
  unplugged: "Desconectado",
  unknown: "",
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

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}

export async function startMonitor(): Promise<void> {
  const ringsEl = byId("rings");
  const rings = {
    cpu: new RingGauge(ringsEl, "CPU", "accent-cpu"),
    ram: new RingGauge(ringsEl, "RAM", "accent-ram"),
    disk: new RingGauge(ringsEl, "Espacio", "accent-disk"),
    battery: new RingGauge(ringsEl, "Batería", "accent-battery"),
  };
  const thermalEl = byId("thermal");
  const lowPowerEl = byId("low-power");
  const appMemFill = byId("appmem-fill");
  const appMemText = byId("appmem-text");
  const netDownEl = byId("net-down");
  const netUpEl = byId("net-up");
  const charts = {
    cpu: new LineChart(byId<HTMLCanvasElement>("chart-cpu"), "--cpu"),
    ram: new LineChart(byId<HTMLCanvasElement>("chart-ram"), "--ram"),
  };

  let range: Range = "5m";
  let simulator = false;

  async function loadHistory(): Promise<void> {
    const points = await metricsHistory(range);
    const { windowMs, stepMs } = WINDOWS[range];
    charts.cpu.setWindow(windowMs, stepMs);
    charts.ram.setWindow(windowMs, stepMs);
    charts.cpu.setData(points.map((p) => ({ ts: p.ts, v: p.cpu, gap: p.gap })));
    charts.ram.setData(points.map((p) => ({ ts: p.ts, v: p.ram, gap: p.gap })));
  }

  function render(s: Snapshot): void {
    if (s.cpu != null) {
      rings.cpu.set(s.cpu / 100, percent(s.cpu), "uso total", levelUp(s.cpu, 50, 85));
    }

    const ramPct = s.ramUsed != null && s.ramTotal > 0 ? (s.ramUsed / s.ramTotal) * 100 : null;
    if (ramPct != null && s.ramUsed != null) {
      rings.ram.set(
        ramPct / 100,
        percent(ramPct),
        `${bytes(s.ramUsed)} de ${bytes(s.ramTotal)}`,
        levelUp(ramPct, 85, 95),
      );
    }

    if (s.storage) {
      const usedPct = ((s.storage.total - s.storage.available) / s.storage.total) * 100;
      rings.disk.set(
        usedPct / 100,
        percent(usedPct),
        `${bytes(s.storage.available)} libres`,
        levelUp(usedPct, 85, 95),
      );
    }

    if (s.battery) {
      const pct = s.battery.level * 100;
      const pluggedIn = s.battery.state === "charging" || s.battery.state === "full";
      rings.battery.set(
        s.battery.level,
        percent(pct),
        BATTERY_LABEL[s.battery.state],
        pluggedIn ? "ok" : levelDown(pct, 20, 10),
      );
    } else {
      rings.battery.set(null, "—", simulator ? "No hay batería en el simulador" : "No disponible");
    }

    thermalEl.textContent = THERMAL_LABEL[s.thermal];
    thermalEl.dataset.state = s.thermal;
    lowPowerEl.hidden = !s.lowPower;

    if (s.appFootprint != null) {
      if (s.appAvailable != null) {
        const share = s.appFootprint / (s.appFootprint + s.appAvailable);
        appMemFill.style.width = `${(share * 100).toFixed(1)}%`;
        appMemText.textContent = `${bytes(s.appFootprint)} usados · margen ${bytes(s.appAvailable)}`;
      } else {
        appMemFill.style.width = "0%";
        appMemText.textContent = `${bytes(s.appFootprint)} usados · el margen solo se ve en un iPhone`;
      }
    }

    netDownEl.textContent = s.netDown != null ? rate(s.netDown) : "—";
    netUpEl.textContent = s.netUp != null ? rate(s.netUp) : "—";

    if (range === "5m") {
      charts.cpu.push({ ts: s.ts, v: s.cpu }, LIVE_POINTS);
      charts.ram.push({ ts: s.ts, v: ramPct }, LIVE_POINTS);
    }
  }

  const rangeButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-range]"));
  for (const button of rangeButtons) {
    button.addEventListener("click", () => {
      range = button.dataset.range as Range;
      for (const b of rangeButtons) b.setAttribute("aria-pressed", String(b === button));
      void loadHistory();
    });
  }

  const info = await deviceInfo();
  simulator = info.simulator;
  byId("device").textContent = [
    info.simulator ? `Simulador (${info.model})` : info.model,
    `iOS ${info.osVersion}`,
    `${info.cores} núcleos`,
    `${bytes(info.ramTotal)} de RAM`,
  ].join(" · ");

  const latest = await onMetrics(render);
  await loadHistory();
  if (latest) render(latest);

  window.setInterval(() => {
    if (range === "1h" && !document.hidden) void loadHistory();
  }, HOUR_REFRESH_MS);
}
