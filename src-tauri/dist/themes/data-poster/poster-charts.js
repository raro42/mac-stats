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

  // Initialize canvas contexts
  const contexts = {};
  Object.keys(canvases).forEach(metric => {
    if (canvases[metric].bar && canvases[metric].line) {
      contexts[metric] = {
        bar: canvases[metric].bar.getContext('2d'),
        line: canvases[metric].line.getContext('2d')
      };
      // Set canvas size for high DPI displays
      const dpr = window.devicePixelRatio || 1;
      [canvases[metric].bar, canvases[metric].line].forEach(canvas => {
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        const ctx = canvas.getContext('2d');
        ctx.scale(dpr, dpr);
        canvas.style.width = rect.width + 'px';
        canvas.style.height = rect.height + 'px';
      });
    }
  });

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
        drawBarChart(metric);
        drawLineChart(metric);
      }
      return true;
    },
    
    // Initialize charts (call on page load)
    init: () => {
      // Draw initial empty charts
      Object.keys(canvases).forEach(metric => {
        drawBarChart(metric);
        drawLineChart(metric);
      });
    }
  };

  // Initialize on load
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      window.posterCharts.init();
    });
  } else {
    window.posterCharts.init();
  }
})();
