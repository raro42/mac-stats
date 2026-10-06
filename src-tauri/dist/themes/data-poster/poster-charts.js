// Data Poster Theme - Bar and Line Chart Visualization
// Maintains rolling buffers and renders charts for Temperature, Usage, and Frequency

(function() {
  'use strict';

  // Chart configuration
  const BAR_CHART_COUNT = 12; // Number of bars to show
  const LINE_CHART_POINTS = 60; // Number of points in line chart
  const SMOOTH_WINDOW = 5; // Moving-average window to reduce noise (display only; raw values still drive scale)
  
  // Color schemes for each metric
  const COLORS = {
    temperature: {
      bar: 'rgba(255, 184, 74, 0.9)',
      line: 'rgba(255, 184, 74, 0.8)',
      fill: 'rgba(255, 184, 74, 0.15)'
    },
    usage: {
      bar: 'rgba(85, 199, 255, 0.9)',
      line: 'rgba(85, 199, 255, 0.8)',
      fill: 'rgba(85, 199, 255, 0.15)'
    },
    frequency: {
      bar: 'rgba(93, 255, 176, 0.9)',
      line: 'rgba(93, 255, 176, 0.8)',
      fill: 'rgba(93, 255, 176, 0.15)'
    },
    gpu: {
      bar: 'rgba(196, 168, 216, 0.9)',
      line: 'rgba(196, 168, 216, 0.8)',
      fill: 'rgba(196, 168, 216, 0.15)'
    }
  };

  // Data buffers for each metric
  const dataBuffers = {
    temperature: {
      bars: new Array(BAR_CHART_COUNT).fill(0),
      line: new Array(LINE_CHART_POINTS).fill(0),
      max: 100, // Max value for scaling (will auto-adjust)
      min: 0
    },
    usage: {
      bars: new Array(BAR_CHART_COUNT).fill(0),
      line: new Array(LINE_CHART_POINTS).fill(0),
      max: 100,
      min: 0
    },
    frequency: {
      bars: new Array(BAR_CHART_COUNT).fill(0),
      line: new Array(LINE_CHART_POINTS).fill(0),
      max: 4.0, // GHz
      min: 0
    },
    gpu: {
      bars: new Array(BAR_CHART_COUNT).fill(0),
      line: new Array(LINE_CHART_POINTS).fill(0),
      max: 100,
      min: 0
    }
  };

  // Canvas elements
  const canvases = {
    temperature: {
      bar: document.getElementById('temperature-bar-chart'),
      line: document.getElementById('temperature-line-chart')
    },
    usage: {
      bar: document.getElementById('usage-bar-chart'),
      line: document.getElementById('usage-line-chart')
    },
    frequency: {
      bar: document.getElementById('frequency-bar-chart'),
      line: document.getElementById('frequency-line-chart')
    },
    gpu: {
      bar: document.getElementById('gpu-usage-bar-chart'),
      line: document.getElementById('gpu-usage-line-chart')
    }
  };

  // Contexts stay empty until idle unpark. Open must not allocate GPU (#14).
  const contexts = {};
  let canvasesParked = true;

  function posterWorkPaused() {
    try {
      if (typeof window.__macStatsWindowWorkPaused === 'function') {
        return !!window.__macStatsWindowWorkPaused();
      }
    } catch (_) { /* ignore */ }
    return false;
  }

  function setHistoryGpuUnparkedClass(on) {
    try {
      document.documentElement.classList.toggle('is-history-gpu-unparked', !!on);
    } catch (_) { /* ignore */ }
  }

  function parkPosterCanvases() {
    canvasesParked = true;
    setHistoryGpuUnparkedClass(false);
    Object.keys(canvases).forEach((metric) => {
      ['bar', 'line'].forEach((kind) => {
        const canvas = canvases[metric] && canvases[metric][kind];
        if (!canvas) return;
        try {
          canvas.width = 1;
          canvas.height = 1;
        } catch (_) { /* ignore */ }
        try {
          canvas.style.visibility = 'hidden';
          canvas.style.contentVisibility = 'hidden';
          canvas.style.display = 'none';
        } catch (_) { /* ignore */ }
      });
      delete contexts[metric];
    });
  }

  function sizePosterCanvas(canvas, fallbackW, fallbackH) {
    if (!canvas) return null;
    try {
      canvas.style.visibility = '';
      canvas.style.contentVisibility = '';
      canvas.style.display = '';
    } catch (_) { /* ignore */ }
    const dpr = Math.min(1, window.devicePixelRatio || 1);
    const rect = canvas.getBoundingClientRect();
    const width = rect.width > 0 ? rect.width : fallbackW;
    const height = rect.height > 0 ? rect.height : fallbackH;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    const ctx = canvas.getContext('2d', { alpha: false });
    if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    canvas.style.width = width + 'px';
    canvas.style.height = height + 'px';
    return ctx;
  }

  function unparkPosterCanvases() {
    if (posterWorkPaused()) return;
    canvasesParked = false;
    setHistoryGpuUnparkedClass(true);
    Object.keys(canvases).forEach((metric) => {
      const pair = canvases[metric];
      if (!pair || !pair.bar || !pair.line) return;
      const bar = sizePosterCanvas(pair.bar, 80, 60);
      const line = sizePosterCanvas(pair.line, 200, 40);
      if (!bar || !line) return;
      contexts[metric] = { bar, line };
      drawBarChart(metric);
      drawLineChart(metric);
    });
  }

  // Moving average for display (reduces chart noise; raw values still used for scale)
  function movingAverage(arr, windowSize) {
    if (!arr.length || windowSize <= 1) return arr.slice();
    const out = [];
    for (let i = 0; i < arr.length; i++) {
      const start = Math.max(0, i - windowSize + 1);
      const slice = arr.slice(start, i + 1);
      const sum = slice.reduce((a, b) => a + b, 0);
      out.push(sum / slice.length);
    }
    return out;
  }

  // Add new value to buffers
  function addValue(metric, value) {
    if (value === null || value === undefined || isNaN(value)) return;
    
    const buffer = dataBuffers[metric];
    
    // Update max/min for auto-scaling
    if (value > buffer.max) buffer.max = value * 1.1; // Add 10% padding
    if (value < buffer.min) buffer.min = Math.max(0, value * 0.9);
    
    // Add to bar chart buffer (rolling window)
    buffer.bars.shift();
    buffer.bars.push(value);
    
    // Add to line chart buffer (rolling window)
    buffer.line.shift();
    buffer.line.push(value);
  }

  // Draw bar chart
  function drawBarChart(metric) {
    if (canvasesParked || !contexts[metric]) return;
    const canvas = canvases[metric].bar;
    const ctx = contexts[metric].bar;
    if (!canvas || !ctx) return;
    
    const buffer = dataBuffers[metric];
    const colors = COLORS[metric];
    const width = canvas.width / (window.devicePixelRatio || 1);
    const height = canvas.height / (window.devicePixelRatio || 1);
    const barWidth = width / buffer.bars.length;
    const maxValue = buffer.max || 1;
    
    // Clear canvas
    ctx.clearRect(0, 0, width, height);
    
    const smoothed = movingAverage(buffer.bars, SMOOTH_WINDOW);
    // Draw bars (smoothed for display)
    smoothed.forEach((value, index) => {
      const barHeight = (value / maxValue) * height;
      const x = index * barWidth;
      const y = height - barHeight;
      
      ctx.fillStyle = colors.bar;
      ctx.fillRect(x + 1, y, barWidth - 2, barHeight);
    });
  }

  // Draw line chart
  function drawLineChart(metric) {
    if (canvasesParked || !contexts[metric]) return;
    const canvas = canvases[metric].line;
    const ctx = contexts[metric].line;
    if (!canvas || !ctx) return;
    
    const buffer = dataBuffers[metric];
    const colors = COLORS[metric];
    const width = canvas.width / (window.devicePixelRatio || 1);
    const height = canvas.height / (window.devicePixelRatio || 1);
    const maxValue = buffer.max || 1;
    const minValue = buffer.min || 0;
    const range = maxValue - minValue || 1;
    
    // Clear canvas
    ctx.clearRect(0, 0, width, height);
    
    if (buffer.line.length < 2) return;
    
    const smoothedLine = movingAverage(buffer.line, SMOOTH_WINDOW);
    // Calculate points (smoothed for display)
    const points = smoothedLine.map((value, index) => {
      const x = (index / (buffer.line.length - 1)) * width;
      const y = height - ((value - minValue) / range) * height;
      return { x, y };
    });
    
    // Draw filled area
    ctx.beginPath();
    ctx.moveTo(points[0].x, height);
    points.forEach(point => ctx.lineTo(point.x, point.y));
    ctx.lineTo(points[points.length - 1].x, height);
    ctx.closePath();
    ctx.fillStyle = colors.fill;
    ctx.fill();
    
    // Draw line
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    points.forEach(point => ctx.lineTo(point.x, point.y));
    ctx.strokeStyle = colors.line;
    ctx.lineWidth = 2;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.stroke();
  }

  // Update charts for a metric
  function updateCharts(metric, value) {
    addValue(metric, value);
    if (canvasesParked) return;
    drawBarChart(metric);
    drawLineChart(metric);
  }

  // Public API
  window.posterCharts = {
    updateTemperature: (value) => updateCharts('temperature', value),
    updateUsage: (value) => updateCharts('usage', value),
    updateFrequency: (value) => updateCharts('frequency', value),
    updateGpuUsage: (value) => updateCharts('gpu', value),
    seedFromPoints: (points) => {
      if (!Array.isArray(points) || !points.length) return false;
      const getters = {
        usage: (p) => p && p.cpu,
        gpu: (p) => p && p.gpu,
        frequency: (p) => p && p.frequency,
        temperature: (p) => p && p.temperature,
      };
      const n = 60;
      for (const [metric, getter] of Object.entries(getters)) {
        const values = [];
        const slice = points.slice(-n);
        for (let i = 0; i < slice.length; i++) {
          const v = getter(slice[i]);
          if (typeof v !== 'number' || !Number.isFinite(v)) continue;
          if ((metric === 'temperature' || metric === 'frequency') && v <= 0) continue;
          values.push(v);
        }
        const line = new Array(n).fill(0);
        const bars = new Array(12).fill(0);
        if (values.length === 1) {
          line[n - 1] = values[0];
          bars[11] = values[0];
        } else if (values.length > 1) {
          for (let i = 0; i < n; i++) {
            const src = Math.round((i / (n - 1)) * (values.length - 1));
            line[i] = values[src];
          }
          const barSlice = values.slice(-12);
          for (let i = 0; i < barSlice.length; i++) {
            bars[12 - barSlice.length + i] = barSlice[i];
          }
        }
        dataBuffers[metric].line = line;
        dataBuffers[metric].bars = bars;
        const peak = Math.max(...values, 1);
        dataBuffers[metric].max = peak * 1.1;
        if (!canvasesParked) {
          drawBarChart(metric);
          drawLineChart(metric);
        }
      }
      return true;
    },
    park: parkPosterCanvases,
    unpark: unparkPosterCanvases,
    // Stay parked on open. history.js idle unpark draws later (#14).
    init: () => {
      parkPosterCanvases();
    }
  };

  // Stay parked. Do not set canvas.width on parse (#14).
})();
