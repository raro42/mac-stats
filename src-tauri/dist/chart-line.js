/**
 * Shared CPU window line charts (CPU / GPU / frequency / temperature).
 * Themes load this via ../../chart-line.js. Exposes window.themeHistory plus
 * legacy per-theme aliases (appleHistory, darkHistory, …) for cpu.js callers.
 */
(function () {
  "use strict";

  const LINE_CHART_POINTS = 60;
  const EMPTY_POINT = NaN;

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

  function canvasLayoutSize(canvas) {
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    const width = rect.width > 0 ? rect.width : canvas.offsetWidth || 200;
    const height = rect.height > 0 ? rect.height : canvas.offsetHeight || 40;
    return { dpr, width, height };
  }

  function setupCanvas(metric) {
    const canvas = canvases[metric];
    if (!canvas) return false;
    const { dpr, width, height } = canvasLayoutSize(canvas);
    if (width <= 0 || height <= 0) return false;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    const ctx = canvas.getContext("2d");
    if (!ctx) return false;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);
    contexts[metric] = ctx;
    canvas.style.width = width + "px";
    canvas.style.height = height + "px";
    canvas.style.backgroundColor = "transparent";
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
    ctx.clearRect(0, 0, width, height);
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

    ctx.beginPath();
    ctx.moveTo(points[firstIdx].x, height);
    for (let i = firstIdx; i <= lastIdx; i++) {
      if (!points[i].empty) ctx.lineTo(points[i].x, points[i].y);
    }
    ctx.lineTo(points[lastIdx].x, height);
    ctx.closePath();
    ctx.fillStyle = colors.fill;
    ctx.fill();

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

  function updateCharts(metric, value) {
    if (!canvases[metric] || !contexts[metric]) {
      initializeCanvases();
    }
    if (!contexts[metric] && canvases[metric]) {
      setupCanvas(metric);
    }
    if (!contexts[metric]) return;
    addValue(metric, value);
    drawLineChart(metric);
  }

  const api = {
    updateTemperature: (value) => updateCharts("temperature", value),
    updateUsage: (value) => updateCharts("usage", value),
    updateGpu: (value) => updateCharts("gpu", value),
    updateFrequency: (value) => updateCharts("frequency", value),
    init: () => {
      initializeCanvases();
      COLORS = getColors();
      Object.keys(canvases).forEach((metric) => {
        if (canvases[metric] && contexts[metric]) drawLineChart(metric);
      });
    },
    refreshLayout: () => {
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
    api.init();
    window.addEventListener("resize", () => api.refreshLayout());
    requestAnimationFrame(() => api.refreshLayout());
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
