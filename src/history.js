// Shared History Chart Visualization with Backend Integration
// Shows continuous line graphs for Temperature, Usage, and Frequency
// Fetches history from backend with adaptive downsampling
// Works with all themes by reading colors from CSS variables

(function() {
  'use strict';

  function tauriInvoke(cmd, payload) {
    if (window.__TAURI__?.core?.invoke) {
      return window.__TAURI__.core.invoke(cmd, payload);
    }
    const i = window.__TAURI_INTERNALS__;
    if (i && typeof i.invoke === 'function') {
      return i.invoke(cmd, payload);
    }
    throw new Error('Tauri invoke not available');
  }

  // Chart configuration
  const HISTORY_POINTS = 60; // Number of points in history graph
  // Chart-specific refresh: temperature redraw every 3s (changes slowly); usage/frequency every cycle
  const TEMPERATURE_REDRAW_INTERVAL_MS = 3000;
  let lastTemperatureDrawMs = 0;

  // Time range options (in seconds)
  const TIME_RANGES = {
    '5m': 300,
    '1h': 3600,
    '6h': 21600,
    '24h': 86400,
    '7d': 604800
  };

  let currentTimeRange = '5m'; // Default to last 5 minutes

  // Get colors from CSS variables or computed styles
  function getColors() {
    // Try to get a sample element to read CSS variables from
    const sampleElement = document.body || document.documentElement;
    const computedStyle = window.getComputedStyle(sampleElement);
    
    // Try to get theme color from CSS variables, fallback to computed text color
    let lineColor = '#8bb4e8'; // Default fallback
    try {
      // Try common CSS variable names
      lineColor = computedStyle.getPropertyValue('--ring-active')?.trim() ||
                  computedStyle.getPropertyValue('--text')?.trim() ||
                  computedStyle.getPropertyValue('--accent')?.trim() ||
                  computedStyle.color || lineColor;
      
      // Remove '#' if present and add it back, handle rgb/rgba
      if (lineColor.startsWith('rgb')) {
        // Keep as-is for rgba
      } else if (!lineColor.startsWith('#')) {
        lineColor = '#' + lineColor.replace('#', '');
      }
    } catch (e) {
      console.warn('[history] Could not read CSS color, using default');
    }
    
    // Convert hex to rgba for fill
    function hexToRgba(hex, alpha) {
      if (hex.startsWith('rgb')) return hex.replace(')', `, ${alpha})`).replace('rgb', 'rgba');
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }
    
    const fillColor = hexToRgba(lineColor, 0.1);
    
    return {
      temperature: {
        line: lineColor,
        fill: fillColor,
        text: lineColor
      },
      usage: {
        line: lineColor,
        fill: fillColor,
        text: lineColor
      },
      frequency: {
        line: lineColor,
        fill: fillColor,
        text: lineColor
      }
    };
  }

  // Color schemes (will be initialized from CSS)
  let COLORS = getColors();

  // Data buffers for each metric
  const dataBuffers = {
    temperature: {
      points: [],
      timestamps: [],
      max: 100,
      min: 0
    },
    usage: {
      points: [],
      timestamps: [],
      max: 100,
      min: 0
    },
    frequency: {
      points: [],
      timestamps: [],
      max: 4.0,
      min: 0
    }
  };

  // Canvas elements - get immediately when script loads (like poster-charts.js)
  const canvases = {
    temperature: document.getElementById('temperature-history-chart'),
    usage: document.getElementById('usage-history-chart'),
    frequency: document.getElementById('frequency-history-chart')
  };

  // Tooltip element
  let tooltipElement = null;

  // Canvas contexts - initialize immediately (like poster-charts.js)
  const contexts = {};
  
  // Initialize canvas contexts immediately (synchronously, like poster-charts.js)
  Object.keys(canvases).forEach(metric => {
    if (canvases[metric]) {
      const dpr = window.devicePixelRatio || 1;
      const rect = canvases[metric].getBoundingClientRect();
      // Use rect size or fallback to offsetWidth/Height or defaults
      let width = rect.width > 0 ? rect.width : canvases[metric].offsetWidth || 200;
      let height = rect.height > 0 ? rect.height : canvases[metric].offsetHeight || 40;
      
      // If still no size, try parent container
      if (width <= 0 || height <= 0) {
        const container = canvases[metric].parentElement;
        if (container) {
          const containerRect = container.getBoundingClientRect();
          width = containerRect.width > 0 ? containerRect.width : 200;
          height = containerRect.height > 0 ? containerRect.height : 40;
        } else {
          width = 200;
          height = 40;
        }
      }
      
      // Set physical pixel size (for high DPI) - this clears canvas and invalidates any existing context
      canvases[metric].width = width * dpr;
      canvases[metric].height = height * dpr;
      
      // Get context AFTER setting width/height (like poster-charts.js line 82)
      const ctx = canvases[metric].getContext('2d');
      if (ctx) {
        ctx.scale(dpr, dpr);
        contexts[metric] = ctx;
        // Set display size (CSS pixels)
        canvases[metric].style.width = width + 'px';
        canvases[metric].style.height = height + 'px';
        console.log(`[history] ${metric} canvas initialized synchronously: ${width}x${height} (${canvases[metric].width}x${canvases[metric].height} @ ${dpr}x)`);
      }
    }
  });

  // Create tooltip element
  function createTooltip() {
    if (!tooltipElement) {
      tooltipElement = document.createElement('div');
      tooltipElement.className = 'history-tooltip';
      tooltipElement.style.position = 'fixed';
      tooltipElement.style.backgroundColor = 'rgba(0, 0, 0, 0.9)';
      tooltipElement.style.color = COLORS.temperature.text;
      tooltipElement.style.padding = '8px 12px';
      tooltipElement.style.borderRadius = '4px';
      tooltipElement.style.fontSize = '12px';
      tooltipElement.style.fontFamily = 'monospace';
      tooltipElement.style.pointerEvents = 'none';
      tooltipElement.style.zIndex = '10000';
      tooltipElement.style.display = 'none';
      tooltipElement.style.border = `1px solid ${COLORS.temperature.text}`;
      tooltipElement.style.whiteSpace = 'nowrap';
      document.body.appendChild(tooltipElement);
    }
    return tooltipElement;
  }

  // Format timestamp to readable date/time
  function formatTimestamp(timestamp) {
    const date = new Date(timestamp * 1000);
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${month}/${day} ${hours}:${minutes}:${seconds}`;
  }

  // Show tooltip with value and timestamp
  function showTooltip(metric, x, y, value, timestamp) {
    const tooltip = createTooltip();
    const formattedTime = formatTimestamp(timestamp);
    const unit = metric === 'temperature' ? '°C' : (metric === 'frequency' ? 'GHz' : '%');

    tooltip.textContent = `${formattedTime}\n${value.toFixed(1)}${unit}`;
    tooltip.style.left = (x + 10) + 'px';
    tooltip.style.top = (y - 30) + 'px';
    tooltip.style.display = 'block';
  }

  // Hide tooltip
  function hideTooltip() {
    if (tooltipElement) {
      tooltipElement.style.display = 'none';
    }
  }

  // Draw line chart
  function drawLineChart(metric) {
    const canvas = canvases[metric];
    const ctx = contexts[metric];
    if (!canvas || !ctx) {
      console.warn(`[history] Canvas or context not available for ${metric}`, {
        canvas: !!canvas,
        ctx: !!ctx,
        canvasId: canvas?.id,
        allCanvases: Object.keys(canvases),
        allContexts: Object.keys(contexts)
      });
      return;
    }

    const buffer = dataBuffers[metric];
    const colors = COLORS[metric];
    
    // Get logical size (accounting for device pixel ratio scaling)
    const dpr = window.devicePixelRatio || 1;
    const width = canvas.width / dpr;
    const height = canvas.height / dpr;
    
    // Ensure we have valid dimensions
    if (width <= 0 || height <= 0) {
      console.warn(`[history] Invalid canvas size for ${metric}: ${width}x${height}`, {
        canvasWidth: canvas.width,
        canvasHeight: canvas.height,
        dpr: dpr,
        styleWidth: canvas.style.width,
        styleHeight: canvas.style.height,
        offsetWidth: canvas.offsetWidth,
        offsetHeight: canvas.offsetHeight,
        rect: canvas.getBoundingClientRect()
      });
      return;
    }
    
    const maxValue = buffer.max || 1;
    const minValue = buffer.min || 0;
    const range = maxValue - minValue || 1;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    if (buffer.points.length < 2) {
      console.log(`[history] Not enough data points for ${metric}: ${buffer.points.length} (need 2+)`);
      return;
    }
    
    console.log(`[history] Drawing ${metric} chart: ${buffer.points.length} points, size ${width}x${height}, range [${minValue}, ${maxValue}]`);

    // Calculate points
    const points = buffer.points.map((value, index) => {
      const x = (index / (buffer.points.length - 1)) * width;
      const y = height - ((value - minValue) / range) * height;
      return { x, y, value, timestamp: buffer.timestamps[index] };
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
    ctx.lineWidth = 1.5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.stroke();

    // Store points for hover detection
    canvas.pointsData = points;
  }

  // Add hover handler for tooltips
  function addCanvasHoverHandler(metric) {
    const canvas = canvases[metric];
    if (!canvas) return;

    canvas.addEventListener('mousemove', (e) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (canvas.pointsData && canvas.pointsData.length > 0) {
        // Find closest point to mouse
        let closestPoint = null;
        let closestDistance = 10; // 10px threshold

        canvas.pointsData.forEach(point => {
          const distance = Math.sqrt(
            Math.pow(x - point.x, 2) + Math.pow(y - point.y, 2)
          );
          if (distance < closestDistance) {
            closestDistance = distance;
            closestPoint = point;
          }
        });

        if (closestPoint) {
          showTooltip(metric, e.clientX, e.clientY, closestPoint.value, closestPoint.timestamp);
        } else {
          hideTooltip();
        }
      }
    });

    canvas.addEventListener('mouseleave', hideTooltip);
  }

  // Fetch history from backend
  async function fetchHistoryFromBackend(timeRangeSeconds, maxPoints) {
    try {
      const result = await tauriInvoke('get_metrics_history', {
        time_range_seconds: timeRangeSeconds,
        max_display_points: maxPoints
      });

      return result;
    } catch (error) {
      console.error('[history] Failed to fetch metrics history:', error);
      return null;
    }
  }

  // Update charts from backend data
  async function updateChartsFromBackend() {
    const timeRangeSeconds = TIME_RANGES[currentTimeRange] || 300;
    console.log(`[history] updateChartsFromBackend() called, timeRange=${currentTimeRange} (${timeRangeSeconds}s)`);
    
    const result = await fetchHistoryFromBackend(timeRangeSeconds, HISTORY_POINTS);

    if (!result || !result.points) {
      // History data not available yet (normal on startup) - silent return
      console.log(`[history] No history data available yet (result=${!!result}, points=${!!result?.points})`);
      return;
    }
    
    if (result.points.length === 0) {
      // No data points yet
      console.log(`[history] History data empty (0 points)`);
      return;
    }
    
    console.log(`[history] Received ${result.points.length} history points`, {
      oldest: result.oldest_available_timestamp,
      newest: result.newest_available_timestamp,
      firstPoint: result.points[0],
      lastPoint: result.points[result.points.length - 1]
    });

    // Extract data by metric
    const temperatureData = result.points.map(p => ({ value: p.temperature, timestamp: p.timestamp }));
    const usageData = result.points.map(p => ({ value: p.cpu, timestamp: p.timestamp })); // CPU usage
    const frequencyData = result.points.map(p => ({ value: p.frequency, timestamp: p.timestamp }));

    // Update buffers
    dataBuffers.temperature.points = temperatureData.map(d => d.value);
    dataBuffers.temperature.timestamps = temperatureData.map(d => d.timestamp);
    dataBuffers.temperature.max = Math.max(100, ...temperatureData.map(d => d.value || 0));
    dataBuffers.temperature.min = Math.min(0, ...temperatureData.map(d => d.value || 0));

    dataBuffers.usage.points = usageData.map(d => d.value);
    dataBuffers.usage.timestamps = usageData.map(d => d.timestamp);
    dataBuffers.usage.max = Math.max(100, ...usageData.map(d => d.value || 0));
    dataBuffers.usage.min = Math.min(0, ...usageData.map(d => d.value || 0));

    dataBuffers.frequency.points = frequencyData.map(d => d.value);
    dataBuffers.frequency.timestamps = frequencyData.map(d => d.timestamp);
    dataBuffers.frequency.max = Math.max(4.0, ...frequencyData.map(d => d.value || 0));
    dataBuffers.frequency.min = Math.min(0, ...frequencyData.map(d => d.value || 0));

    // Redraw charts (temperature only every 3s; usage and frequency every cycle)
    const nowMs = Date.now();
    const shouldRedrawTemperature = lastTemperatureDrawMs === 0 || (nowMs - lastTemperatureDrawMs >= TEMPERATURE_REDRAW_INTERVAL_MS);
    if (shouldRedrawTemperature) {
      lastTemperatureDrawMs = nowMs;
    }
    console.log(`[history] Redrawing charts with data:`, {
      temperaturePoints: dataBuffers.temperature.points.length,
      usagePoints: dataBuffers.usage.points.length,
      frequencyPoints: dataBuffers.frequency.points.length
    });
    
    Object.keys(canvases).forEach(metric => {
      if (metric === 'temperature' && !shouldRedrawTemperature) return;
      if (canvases[metric]) {
        drawLineChart(metric);
      } else {
        console.warn(`[history] Cannot draw ${metric}: canvas not found`);
      }
    });
  }

  // Set time range and update charts
  async function setTimeRange(timeRange) {
    if (TIME_RANGES[timeRange]) {
      currentTimeRange = timeRange;
      await updateChartsFromBackend();
    }
  }

  // Re-initialize canvas sizes (for window resize)
  function reinitializeCanvasSizes() {
    console.log('[history] reinitializeCanvasSizes() called');
    Object.keys(canvases).forEach(metric => {
      if (canvases[metric]) {
        const dpr = window.devicePixelRatio || 1;
        const rect = canvases[metric].getBoundingClientRect();
        let width = rect.width > 0 ? rect.width : canvases[metric].offsetWidth || 200;
        let height = rect.height > 0 ? rect.height : canvases[metric].offsetHeight || 40;
        
        if (width <= 0 || height <= 0) {
          const container = canvases[metric].parentElement;
          if (container) {
            const containerRect = container.getBoundingClientRect();
            width = containerRect.width > 0 ? containerRect.width : 200;
            height = containerRect.height > 0 ? containerRect.height : 40;
          } else {
            width = 200;
            height = 40;
          }
        }
        
        // Set physical pixel size (this invalidates the context, so we need to get a new one)
        canvases[metric].width = width * dpr;
        canvases[metric].height = height * dpr;
        
        // Get new context after setting size (like poster-charts.js)
        const ctx = canvases[metric].getContext('2d');
        if (ctx) {
          ctx.scale(dpr, dpr);
          contexts[metric] = ctx;
          canvases[metric].style.width = width + 'px';
          canvases[metric].style.height = height + 'px';
        }
      }
    });
  }

  // Public API
  window.historyCharts = {
    // Legacy API for backward compatibility
    updateTemperature: (value) => {},
    updateUsage: (value) => {},
    updateFrequency: (value) => {},

    // New backend-integrated API
    fetchAndUpdateHistory: updateChartsFromBackend,
    setTimeRange: setTimeRange,

    // Initialize charts (call on page load)
    init: () => {
      console.log('[history] init() called');
      console.log('[history] Canvas elements found:', {
        temperature: !!canvases.temperature,
        usage: !!canvases.usage,
        frequency: !!canvases.frequency,
        temperatureId: canvases.temperature?.id,
        usageId: canvases.usage?.id,
        frequencyId: canvases.frequency?.id,
        temperatureContext: !!contexts.temperature,
        usageContext: !!contexts.usage,
        frequencyContext: !!contexts.frequency
      });
      
      // Refresh colors from CSS (in case theme changed)
      COLORS = getColors();
      
      // Create tooltip
      createTooltip();

      // Add hover handlers to all canvases
      Object.keys(canvases).forEach(metric => {
        if (canvases[metric]) {
          addCanvasHoverHandler(metric);
        }
      });

      // Fetch initial history
      console.log('[history] Fetching initial history data');
      updateChartsFromBackend();

      // Refresh every 2 seconds
      setInterval(updateChartsFromBackend, 2000);
      
      // Handle window resize
      let resizeTimeout;
      window.addEventListener('resize', () => {
        clearTimeout(resizeTimeout);
        resizeTimeout = setTimeout(() => {
          reinitializeCanvasSizes();
          // Redraw charts after resize
          Object.keys(canvases).forEach(metric => {
            if (canvases[metric] && contexts[metric]) {
              drawLineChart(metric);
            }
          });
        }, 100);
      });
    }
  };

  // Initialize on load
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      window.historyCharts.init();
    });
  } else {
    window.historyCharts.init();
  }
})();
