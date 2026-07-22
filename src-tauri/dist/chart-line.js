/**
 * Shared CPU window line charts (temperature / usage / frequency).
 * Themes load this via ../../chart-line.js. Exposes window.themeHistory plus
 * legacy per-theme aliases (appleHistory, darkHistory, …) for cpu.js callers.
 */
(function () {
  "use strict";

  const LINE_CHART_POINTS = 60;

  function getColors() {
    const sampleElement = document.body || document.documentElement;
    const computedStyle = window.getComputedStyle(sampleElement);
    let lineColor = "#8bb4e8";
    try {
      const candidates = [
        computedStyle.getPropertyValue("--accent"),
        computedStyle.getPropertyValue("--accent-usage"),
        computedStyle.getPropertyValue("--accent-freq"),
        computedStyle.getPropertyValue("--ring-active"),
        computedStyle.getPropertyValue("--text"),
        computedStyle.color,
      ];
      for (const raw of candidates) {
        const c = (raw || "").trim();
        if (!c) continue;
        if (c.startsWith("#") || c.startsWith("rgb")) {
          lineColor = c;
          break;
        }
        lineColor = "#" + c.replace("#", "");
        break;
      }
    } catch (_) {
      /* keep default */
    }
    if (!lineColor || (!lineColor.startsWith("#") && !lineColor.startsWith("rgb"))) {
      lineColor = "#8bb4e8";
    }

    function hexToRgba(hex, alpha) {
      if (hex.startsWith("rgb")) {
        return hex.replace(")", `, ${alpha})`).replace("rgb", "rgba");
      }
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }

    const fillColor = hexToRgba(lineColor, 0.12);
    return {
      temperature: { line: lineColor, fill: fillColor },
      usage: { line: lineColor, fill: fillColor },
      frequency: { line: lineColor, fill: fillColor },
    };
  }

  let COLORS = getColors();

  const dataBuffers = {
    temperature: { line: new Array(LINE_CHART_POINTS).fill(0), max: 100, min: 0 },
    usage: { line: new Array(LINE_CHART_POINTS).fill(0), max: 100, min: 0 },
    frequency: { line: new Array(LINE_CHART_POINTS).fill(0), max: 4.0, min: 0 },
  };

  let canvases = {};
  let contexts = {};

  function initializeCanvases() {
    canvases = {
      temperature: document.getElementById("temperature-history-chart"),
      usage: document.getElementById("usage-history-chart"),
      frequency: document.getElementById("frequency-history-chart"),
    };
    contexts = {};
    Object.keys(canvases).forEach((metric) => {
      if (!canvases[metric]) return;
      const dpr = window.devicePixelRatio || 1;
      const rect = canvases[metric].getBoundingClientRect();
      const width = rect.width > 0 ? rect.width : canvases[metric].offsetWidth || 200;
      const height = rect.height > 0 ? rect.height : canvases[metric].offsetHeight || 40;
      canvases[metric].width = width * dpr;
      canvases[metric].height = height * dpr;
      const ctx = canvases[metric].getContext("2d");
      if (!ctx) return;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, width, height);
      contexts[metric] = ctx;
      canvases[metric].style.width = width + "px";
      canvases[metric].style.height = height + "px";
      canvases[metric].style.backgroundColor = "transparent";
    });
  }

  function addValue(metric, value) {
    if (value === null || value === undefined || isNaN(value)) return;
    const buffer = dataBuffers[metric];
    if (value > buffer.max) buffer.max = value * 1.1;
    if (value < buffer.min) buffer.min = Math.max(0, value * 0.9);
    buffer.line.shift();
    buffer.line.push(value);
  }

  function drawLineChart(metric) {
    const canvas = canvases[metric];
    const ctx = contexts[metric];
    if (!canvas || !ctx) return;
    const buffer = dataBuffers[metric];
    const colors = COLORS[metric];
    const width = canvas.width / (window.devicePixelRatio || 1);
    const height = canvas.height / (window.devicePixelRatio || 1);
    const maxValue = buffer.max || 1;
    const minValue = buffer.min || 0;
    const range = maxValue - minValue || 1;
    ctx.clearRect(0, 0, width, height);
    const hasValidData = buffer.line.some((val) => val !== 0 && !isNaN(val) && isFinite(val));
    if (buffer.line.length < 2 || !hasValidData) return;
    const points = buffer.line.map((value, index) => {
      const x = (index / (buffer.line.length - 1)) * width;
      const y = height - ((value - minValue) / range) * height;
      return { x, y };
    });
    ctx.beginPath();
    ctx.moveTo(points[0].x, height);
    points.forEach((point) => ctx.lineTo(point.x, point.y));
    ctx.lineTo(points[points.length - 1].x, height);
    ctx.closePath();
    ctx.fillStyle = colors.fill;
    ctx.fill();
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    points.forEach((point) => ctx.lineTo(point.x, point.y));
    ctx.strokeStyle = colors.line;
    ctx.lineWidth = 1;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.stroke();
  }

  function updateCharts(metric, value) {
    if (!canvases[metric] || !contexts[metric]) initializeCanvases();
    addValue(metric, value);
    drawLineChart(metric);
  }

  const api = {
    updateTemperature: (value) => updateCharts("temperature", value),
    updateUsage: (value) => updateCharts("usage", value),
    updateFrequency: (value) => updateCharts("frequency", value),
    init: () => {
      initializeCanvases();
      COLORS = getColors();
      Object.keys(canvases).forEach((metric) => {
        if (canvases[metric] && contexts[metric]) drawLineChart(metric);
      });
    },
  };

  window.themeHistory = api;
  // Legacy aliases used by src/cpu.js
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

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => api.init());
  } else {
    api.init();
  }
})();
