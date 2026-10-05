/**
 * Shared CPU window line charts (CPU / GPU / frequency / temperature).
 * Themes load this via ../../chart-line.js. Exposes window.themeHistory plus
 * legacy per-theme aliases (appleHistory, darkHistory, …) for cpu.js callers.
 */
(function () {
  "use strict";

  // Fewer points = less canvas work per sample (#14).
  const LINE_CHART_POINTS = 2;
  const EMPTY_POINT = NaN;
  let cachedSparklineBackdrop = null;

  function metricColor(metric, computedStyle) {
    const keys = {
      usage: ["--accent-usage", "--accent"],
      gpu: ["--accent-gpu", "--accent"],
      frequency: ["--accent-freq", "--accent"],
      temperature: ["--accent", "--accent-freq"],
    }[metric] || ["--accent"];
    for (const key of keys) {
      const c = (computedStyle.getPropertyValue(key) || "").trim();
      if (c && (c.startsWith("#") || c.startsWith("rgb"))) return c;
    }
    return "#8bb4e8";
  }

  function getColors() {
    const sampleElement = document.body || document.documentElement;
    const computedStyle = window.getComputedStyle(sampleElement);

    function hexToRgba(hex, alpha) {
      if (hex.startsWith("rgb")) {
        return hex.replace(")", `, ${alpha})`).replace("rgb", "rgba");
      }
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }

    const out = {};
    for (const metric of ["usage", "gpu", "frequency", "temperature"]) {
      const line = metricColor(metric, computedStyle);
      out[metric] = { line, fill: hexToRgba(line, 0.12) };
    }
    return out;
  }

  let COLORS = getColors();

  const dataBuffers = {
    temperature: { line: new Array(LINE_CHART_POINTS).fill(EMPTY_POINT) },
    usage: { line: new Array(LINE_CHART_POINTS).fill(EMPTY_POINT) },
    gpu: { line: new Array(LINE_CHART_POINTS).fill(EMPTY_POINT) },
    frequency: { line: new Array(LINE_CHART_POINTS).fill(EMPTY_POINT) },
  };

  let canvases = {};
  let contexts = {};
  const canvasLayoutCache = {};
  let canvasesParked = false;
  const lastSample = {
    temperature: NaN,
    usage: NaN,
    gpu: NaN,
    frequency: NaN,
  };

  /** macOS often keeps visibilityState=visible when another app is frontmost (#14). */
  function windowOccluded() {
    if (typeof document === "undefined") return false;
    if (document.hidden) return true;
    try {
      if (typeof document.hasFocus === "function" && !document.hasFocus()) {
        return true;
      }
    } catch (_) {
      /* ignore */
    }
    return false;
  }

  /** Drop GPU backing stores while occluded — keeps last buffer for unpark redraw (#14). */
  function parkCanvases() {
    canvasesParked = true;
    Object.keys(canvases).forEach((metric) => {
      const canvas = canvases[metric];
      if (!canvas) return;
      try {
        canvas.width = 1;
        canvas.height = 1;
      } catch (_) {
        /* ignore */
      }
      try {
        canvas.style.visibility = "hidden";
        canvas.style.contentVisibility = "hidden";
      } catch (_) {
        /* ignore */
      }
      delete contexts[metric];
      delete canvasLayoutCache[metric];
    });
  }

  function unparkCanvases() {
    if (!canvasesParked && Object.keys(contexts).length) return;
    canvasesParked = false;
    Object.keys(canvases).forEach((metric) => {
      const canvas = canvases[metric];
      if (!canvas) return;
      try {
        canvas.style.visibility = "";
        canvas.style.contentVisibility = "";
      } catch (_) {
        /* ignore */
      }
    });
    initializeCanvases();
    COLORS = getColors();
    Object.keys(dataBuffers).forEach((metric) => {
      if (contexts[metric] && canvasIsPaintable(metric)) drawLineChart(metric);
    });
  }

  function canvasLayoutSize(canvas) {
    // Cap DPR: 2x/3x backing stores keep the compositor busy for 40px sparklines.
    const dpr = Math.min(1, window.devicePixelRatio || 1);
    const rect = canvas.getBoundingClientRect();
    const width = rect.width > 0 ? rect.width : canvas.offsetWidth || 200;
    const height = rect.height > 0 ? rect.height : canvas.offsetHeight || 40;
    return { dpr, width, height };
  }

  function sparklineBackdrop() {
    if (cachedSparklineBackdrop) return cachedSparklineBackdrop;
    try {
      const bg = window.getComputedStyle(document.body || document.documentElement)
        .backgroundColor;
      if (bg && bg !== "rgba(0, 0, 0, 0)" && bg !== "transparent") {
        cachedSparklineBackdrop = bg;
        return bg;
      }
    } catch (_) {
      /* ignore */
    }
    cachedSparklineBackdrop = "#f7f7fa";
    return cachedSparklineBackdrop;
  }

  function setupCanvas(metric) {
    const canvas = canvases[metric];
    if (!canvas) return false;
    const { dpr, width, height } = canvasLayoutSize(canvas);
    if (width <= 0 || height <= 0) return false;
    const prev = canvasLayoutCache[metric];
    const same =
      prev &&
      prev.dpr === dpr &&
      prev.width === width &&
      prev.height === height &&
      contexts[metric];
    if (same) return true;
    canvasLayoutCache[metric] = { dpr, width, height };
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    // Opaque canvas: avoid per-frame alpha blending with the shell (#14).
    const ctx = canvas.getContext("2d", { alpha: false });
    if (!ctx) return false;
    const backdrop = sparklineBackdrop();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = backdrop;
    ctx.fillRect(0, 0, width, height);
    contexts[metric] = ctx;
    canvas.style.width = width + "px";
    canvas.style.height = height + "px";
    canvas.style.backgroundColor = backdrop;
    return true;
  }

  function initializeCanvases() {
    canvases = {
      temperature: document.getElementById("temperature-history-chart"),
      usage: document.getElementById("usage-history-chart"),
      gpu: document.getElementById("gpu-history-chart"),
      frequency: document.getElementById("frequency-history-chart"),
    };
    contexts = {};
    Object.keys(canvases).forEach((metric) => {
      delete canvasLayoutCache[metric];
      if (canvases[metric]) setupCanvas(metric);
    });
  }

  function addValue(metric, value) {
    if (value === null || value === undefined || !Number.isFinite(value)) return;
    const buffer = dataBuffers[metric];
    buffer.line.shift();
    buffer.line.push(value);
  }

  function valueRange(finiteValues, metric) {
    let maxValue = Math.max(...finiteValues);
    let minValue = Math.min(...finiteValues);
    if (maxValue === minValue) {
      const pad = Math.max(Math.abs(maxValue) * 0.08, 1);
      maxValue += pad;
      minValue -= pad;
    }
    if (metric === "usage" || metric === "gpu") {
      minValue = Math.max(0, minValue);
      maxValue = Math.min(100, Math.max(maxValue, 1));
    } else if (metric === "temperature") {
      minValue = Math.max(0, minValue);
    } else if (metric === "frequency") {
      minValue = Math.max(0, minValue);
    }
    const range = maxValue - minValue || 1;
    return { minValue, maxValue, range };
  }

  function drawLineChart(metric) {
    const canvas = canvases[metric];
    const ctx = contexts[metric];
    if (!canvas || !ctx) return;
    const buffer = dataBuffers[metric];
    const colors = COLORS[metric] || COLORS.usage;
    const { width, height } = canvasLayoutSize(canvas);
    const finiteValues = buffer.line.filter((val) => Number.isFinite(val));
    const backdrop = sparklineBackdrop();
    ctx.fillStyle = backdrop;
    ctx.fillRect(0, 0, width, height);
    if (!finiteValues.length) return;

    const { minValue, maxValue, range } = valueRange(finiteValues, metric);
    const points = buffer.line.map((value, index) => {
      const x =
        buffer.line.length <= 1
          ? width / 2
          : (index / (buffer.line.length - 1)) * width;
      if (!Number.isFinite(value)) {
        return { x, y: height, empty: true };
      }
      const y = height - ((value - minValue) / range) * height;
      return { x, y, empty: false };
    });

    const plotPoints = points.filter((p) => !p.empty);
    if (!plotPoints.length) return;

    if (plotPoints.length === 1) {
      const p = plotPoints[0];
      ctx.beginPath();
      ctx.moveTo(0, p.y);
      ctx.lineTo(width, p.y);
      ctx.strokeStyle = colors.line;
      ctx.lineWidth = 1.5;
      ctx.stroke();
      return;
    }

    const firstIdx = points.findIndex((p) => !p.empty);
    let lastIdx = firstIdx;
    for (let i = points.length - 1; i >= 0; i--) {
      if (!points[i].empty) {
        lastIdx = i;
        break;
      }
    }

    // Stroke only — area fills force extra canvas blend work every sample (#14).
    ctx.beginPath();
    let started = false;
    for (let i = firstIdx; i <= lastIdx; i++) {
      const point = points[i];
      if (point.empty) continue;
      if (!started) {
        ctx.moveTo(point.x, point.y);
        started = true;
      } else {
        ctx.lineTo(point.x, point.y);
      }
    }
    ctx.strokeStyle = colors.line;
    ctx.lineWidth = 1.5;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.stroke();
  }

  function sampleEpsilon(metric) {
    // Wider deadband: skip canvas work when the sample barely moved (#14).
    if (metric === "frequency") return 0.25;
    if (metric === "temperature") return 4.0;
    return 5.0;
  }

  function canvasIsPaintable(metric) {
    const canvas = canvases[metric];
    if (!canvas) return false;
    // Zero-size / display:none charts still buffered; skip WebKit paints (#14).
    if (canvas.clientWidth < 2 || canvas.clientHeight < 2) return false;
    return true;
  }

  function updateCharts(metric, value) {
    if (value === null || value === undefined || !Number.isFinite(value)) return;
    const prev = lastSample[metric];
    if (Number.isFinite(prev) && Math.abs(prev - value) < sampleEpsilon(metric)) {
      return;
    }
    lastSample[metric] = value;
    // Buffer even if the canvas is not ready yet (zero-size layout, late GPU inject).
    addValue(metric, value);
    // Occluded / parked: keep the buffer, skip WebKit canvas invalidation (#14).
    if (canvasesParked || windowOccluded()) return;
    if (!canvases[metric] || !contexts[metric]) {
      initializeCanvases();
    }
    if (!contexts[metric] && canvases[metric]) {
      setupCanvas(metric);
    }
    if (!contexts[metric] || !canvasIsPaintable(metric)) return;
    drawLineChart(metric);
  }

  function seriesFromHistory(points, getter, metric) {
    const slice = points.slice(-LINE_CHART_POINTS);
    const values = [];
    for (let i = 0; i < slice.length; i++) {
      const v = getter(slice[i]);
      if (typeof v !== "number" || !Number.isFinite(v)) continue;
      // Temp/freq 0 means "no sample" in the backend cache path.
      if ((metric === "temperature" || metric === "frequency") && v <= 0) continue;
      values.push(v);
    }
    const line = new Array(LINE_CHART_POINTS).fill(EMPTY_POINT);
    if (!values.length) return line;
    if (values.length === 1) {
      line[LINE_CHART_POINTS - 1] = values[0];
      return line;
    }
    // Oldest on the left, newest on the right, across the full width.
    // A short stub on the right looks like the chart "starts in ticks".
    for (let i = 0; i < values.length; i++) {
      const idx = Math.round((i / (values.length - 1)) * (LINE_CHART_POINTS - 1));
      line[idx] = values[i];
    }
    return line;
  }

  function seedFromPoints(points) {
    if (!Array.isArray(points) || !points.length) return false;
    const getters = {
      usage: (p) => p && p.cpu,
      gpu: (p) => p && p.gpu,
      frequency: (p) => p && p.frequency,
      temperature: (p) => p && p.temperature,
    };
    for (const [metric, getter] of Object.entries(getters)) {
      dataBuffers[metric].line = seriesFromHistory(points, getter, metric);
      const line = dataBuffers[metric].line;
      let last = NaN;
      for (let i = line.length - 1; i >= 0; i--) {
        if (Number.isFinite(line[i])) {
          last = line[i];
          break;
        }
      }
      lastSample[metric] = last;
    }
    if (canvasesParked || windowOccluded()) return true;
    if (!canvases.usage) initializeCanvases();
    COLORS = getColors();
    Object.keys(dataBuffers).forEach((metric) => {
      if (!contexts[metric] && canvases[metric]) setupCanvas(metric);
      if (contexts[metric] && canvasIsPaintable(metric)) drawLineChart(metric);
    });
    return true;
  }

  const api = {
    updateTemperature: (value) => updateCharts("temperature", value),
    updateUsage: (value) => updateCharts("usage", value),
    updateGpu: (value) => updateCharts("gpu", value),
    updateFrequency: (value) => updateCharts("frequency", value),
    seedFromPoints,
    park: parkCanvases,
    unpark: unparkCanvases,
    init: () => {
      if (windowOccluded()) {
        canvasesParked = true;
        return;
      }
      canvasesParked = false;
      initializeCanvases();
      COLORS = getColors();
      Object.keys(canvases).forEach((metric) => {
        if (canvases[metric] && contexts[metric]) drawLineChart(metric);
      });
    },
    refreshLayout: () => {
      if (windowOccluded()) {
        parkCanvases();
        return;
      }
      canvasesParked = false;
      initializeCanvases();
      COLORS = getColors();
      Object.keys(canvases).forEach((metric) => {
        if (canvases[metric] && contexts[metric]) drawLineChart(metric);
      });
    },
  };

  window.themeHistory = api;
  [
    "appleHistory",
    "darkHistory",
    "lightHistory",
    "futuristicHistory",
    "materialHistory",
    "neonHistory",
    "swissHistory",
    "architectHistory",
  ].forEach((name) => {
    window[name] = api;
  });

  function boot() {
    // Defer canvas setup past first paint so open does not stack with
    // get_cpu_details IPC / ring DOM (#14).
    const start = () => {
      api.init();
      let resizeTimer = null;
      window.addEventListener("resize", () => {
        if (resizeTimer) clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => api.refreshLayout(), 200);
      });
      // Release sparkline GPU buffers when occluded; redraw on focus (#14).
      document.addEventListener("visibilitychange", () => {
        if (document.hidden) parkCanvases();
        else unparkCanvases();
      });
      window.addEventListener("blur", parkCanvases);
      window.addEventListener("focus", unparkCanvases);
      if (typeof window.seedThemeHistoryFromBackend === "function") {
        void window.seedThemeHistoryFromBackend();
      }
    };
    if (typeof window.requestIdleCallback === "function") {
      window.requestIdleCallback(start, { timeout: 30000 });
    } else {
      setTimeout(start, 0);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
