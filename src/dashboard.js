// Dashboard JavaScript
const { invoke } = window.__TAURI__.core;

// Update interval (milliseconds)
const UPDATE_INTERVAL = 2000; // 2 seconds

// State
let updateInterval = null;
let monitorsCollapsed = false;
let ollamaCollapsed = false;

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    setupEventListeners();
    startUpdates();
    // Initialize Ollama after a short delay to ensure ollama.js is loaded
    setTimeout(() => {
        if (window.Ollama) {
            window.Ollama.checkConnection();
            window.Ollama.initListeners();
        } else {
            console.warn('[Dashboard] Ollama module not loaded yet');
        }
    }, 100);
});

function setupEventListeners() {
    // Section headers (clickable to toggle)
    document.getElementById('monitors-header').addEventListener('click', (e) => {
        // Don't toggle if clicking the button (button handles its own click)
        if (e.target.id !== 'monitors-collapse-btn' && !e.target.closest('.collapse-btn')) {
            toggleSection('monitors');
        }
    });
    
    // Collapse buttons
    document.getElementById('monitors-collapse-btn').addEventListener('click', (e) => {
        e.stopPropagation(); // Prevent header click from firing
        toggleSection('monitors');
    });
    
    document.getElementById('ollama-header').addEventListener('click', (e) => {
        // Don't toggle if clicking the button (button handles its own click)
        if (e.target.id !== 'ollama-collapse-btn' && !e.target.closest('.collapse-btn')) {
            toggleSection('ollama');
        }
    });
    
    document.getElementById('ollama-collapse-btn').addEventListener('click', (e) => {
        e.stopPropagation(); // Prevent header click from firing
        toggleSection('ollama');
    });

    // Add monitor button
    document.getElementById('add-monitor-btn').addEventListener('click', () => {
        showAddMonitorDialog();
    });

    // Chat input - handled by Ollama module
    // Ollama.initListeners() will set up the chat event listeners

    // Settings button
    document.getElementById('settings-btn').addEventListener('click', () => {
        showSettingsDialog();
    });
}

function toggleSection(section) {
    if (section === 'monitors') {
        monitorsCollapsed = !monitorsCollapsed;
        const content = document.getElementById('monitors-content');
        const btn = document.getElementById('monitors-collapse-btn');
        const list = document.getElementById('monitors-list');
        
        if (monitorsCollapsed) {
            content.classList.add('collapsed');
            btn.textContent = '+';
            list.style.display = 'none';
        } else {
            content.classList.remove('collapsed');
            btn.textContent = '−';
            list.style.display = 'block';
            loadMonitors();
        }
    } else if (section === 'ollama') {
        ollamaCollapsed = !ollamaCollapsed;
        const content = document.getElementById('ollama-content');
        const btn = document.getElementById('ollama-collapse-btn');
        const chat = document.getElementById('ollama-chat');
        
        if (ollamaCollapsed) {
            content.classList.add('collapsed');
            btn.textContent = '+';
            chat.style.display = 'none';
        } else {
            content.classList.remove('collapsed');
            btn.textContent = '−';
            chat.style.display = 'block';
        }
    }
}

function startUpdates() {
    updateMetrics();
    updateInterval = setInterval(updateMetrics, UPDATE_INTERVAL);
}

function updateMetrics() {
    // Get CPU details (includes temperature, frequency, battery, power)
    invoke('get_cpu_details')
        .then(cpuDetails => {
            updateGauges(cpuDetails);
            updateBatteryPower(cpuDetails);
        })
        .catch(err => {
            console.error('Failed to get CPU details:', err);
        });

    // Update monitors summary if section is expanded
    if (!monitorsCollapsed) {
        updateMonitorsSummary();
    }
}

function updateGauges(cpuDetails) {
    // Temperature gauge
    const temp = cpuDetails.temperature || 0;
    const tempValue = document.getElementById('temperature-value');
    const tempFill = document.getElementById('temperature-fill');
    
    tempValue.textContent = temp > 0 ? temp.toFixed(1) : '--';
    // Temperature range: 0-100°C (adjust as needed)
    const tempPercent = Math.min(100, (temp / 100) * 100);
    tempFill.style.width = `${tempPercent}%`;

    // CPU Usage gauge
    const cpuUsage = cpuDetails.usage || 0;
    const cpuValue = document.getElementById('cpu-usage-value');
    const cpuFill = document.getElementById('cpu-usage-fill');
    
    cpuValue.textContent = cpuUsage.toFixed(1);
    cpuFill.style.width = `${cpuUsage}%`;

    // CPU Frequency gauge
    const freq = cpuDetails.frequency || 0;
    const freqValue = document.getElementById('cpu-frequency-value');
    const freqFill = document.getElementById('cpu-frequency-fill');
    
    freqValue.textContent = freq > 0 ? freq.toFixed(2) : '--';
    // Frequency range: 0-4 GHz (adjust based on your CPU)
    const freqPercent = Math.min(100, (freq / 4.0) * 100);
    freqFill.style.width = `${freqPercent}%`;
}

function updateBatteryPower(cpuDetails) {
    const batteryLevel = document.getElementById('battery-level');
    const batteryStatus = document.getElementById('battery-status');
    const batteryIcon = document.getElementById('battery-icon');
    const powerValue = document.getElementById('power-value');
    const timeRemaining = document.getElementById('time-remaining');

    if (cpuDetails.has_battery) {
        const level = cpuDetails.battery_level || 0;
        const isCharging = cpuDetails.is_charging || false;
        
        batteryLevel.textContent = `${level.toFixed(0)}%`;
        batteryStatus.textContent = isCharging ? 'Charging' : 'Discharging';
        batteryIcon.textContent = isCharging ? '🔌' : '🔋';
        
        // Calculate total power (CPU + GPU)
        const totalPower = (cpuDetails.cpu_power || 0) + (cpuDetails.gpu_power || 0);
        powerValue.textContent = `${totalPower.toFixed(1)} W`;
        
        // Estimate time remaining (simplified calculation)
        if (!isCharging && level > 0 && totalPower > 0) {
            // Rough estimate: assume battery capacity and calculate hours
            // This is a placeholder - real calculation would need battery capacity
            const hours = (level / 100) * 10 / (totalPower / 20); // Simplified
            timeRemaining.textContent = `~${hours.toFixed(1)}h remaining`;
        } else {
            timeRemaining.textContent = '';
        }
    } else {
        batteryLevel.textContent = 'N/A';
        batteryStatus.textContent = 'No battery';
        batteryIcon.textContent = '⚡';
        powerValue.textContent = `${((cpuDetails.cpu_power || 0) + (cpuDetails.gpu_power || 0)).toFixed(1)} W`;
        timeRemaining.textContent = '';
    }
}

async function updateMonitorsSummary() {
    try {
        const monitorIds = await invoke('list_monitors');
        
        if (monitorIds.length === 0) {
            document.getElementById('monitors-summary-text').textContent = 'No monitors configured';
            return;
        }

        // Check all monitors and aggregate
        let upCount = 0;
        let totalResponseTime = 0;
        let responseTimeCount = 0;

        for (const monitorId of monitorIds) {
            try {
                const status = await invoke('check_monitor', { monitorId });
                if (status.is_up) {
                    upCount++;
                }
                if (status.response_time_ms) {
                    totalResponseTime += status.response_time_ms;
                    responseTimeCount++;
                }
            } catch (err) {
                console.error(`Failed to check monitor ${monitorId}:`, err);
            }
        }

        const avgResponseTime = responseTimeCount > 0 
            ? Math.round(totalResponseTime / responseTimeCount)
            : 0;

        document.getElementById('monitors-summary-text').textContent = 
            `${upCount} / ${monitorIds.length} sites up · Avg ${avgResponseTime} ms`;
    } catch (err) {
        console.error('Failed to update monitors summary:', err);
    }
}

async function loadMonitors() {
    const monitorsList = document.getElementById('monitors-list');
    monitorsList.innerHTML = '';

    try {
        const monitorIds = await invoke('list_monitors');
        
        for (const monitorId of monitorIds) {
            try {
                const status = await invoke('check_monitor', { monitorId });
                const monitorItem = createMonitorItem(monitorId, status);
                monitorsList.appendChild(monitorItem);
            } catch (err) {
                console.error(`Failed to load monitor ${monitorId}:`, err);
            }
        }
    } catch (err) {
        console.error('Failed to load monitors:', err);
    }
}

function createMonitorItem(monitorId, status) {
    const item = document.createElement('div');
    item.className = 'monitor-item';
    
    const statusIndicator = document.createElement('div');
    statusIndicator.className = 'status-indicator';
    if (!status.is_up) {
        statusIndicator.classList.add('down');
    }

    const info = document.createElement('div');
    info.className = 'monitor-info';
    info.innerHTML = `
        <div>${monitorId}</div>
        <div style="font-size: 12px; color: rgba(255,255,255,0.6);">
            ${status.response_time_ms ? `${status.response_time_ms}ms` : '--'}
            ${status.error ? ` · ${status.error}` : ''}
        </div>
    `;

    item.appendChild(statusIndicator);
    item.appendChild(info);
    
    return item;
}

// ============================================================================
// Ollama Functionality - Moved to src/ollama.js
// ============================================================================
// All Ollama chat, code execution, and connection management has been 
// extracted to src/ollama.js module.
// Use window.Ollama.* to access Ollama functionality.
// The ollama.js module is loaded in dashboard.html before this file.
// ============================================================================

function showAddMonitorDialog() {
    // Simple prompt for now - can be enhanced with a proper modal
    const url = prompt('Enter website URL to monitor:');
    if (url) {
        const id = `monitor_${Date.now()}`;
        const name = new URL(url).hostname;
        
        invoke('add_website_monitor', {
            request: {
                id,
                name,
                url,
                timeout_secs: 10,
                check_interval_secs: 60,
                verify_ssl: true
            }
        })
        .then(() => {
            loadMonitors();
            updateMonitorsSummary();
        })
        .catch(err => {
            alert(`Failed to add monitor: ${err}`);
        });
    }
}

function showSettingsDialog() {
    // Placeholder - can be enhanced with a proper settings modal
    alert('Settings dialog - To be implemented\n\nFeatures:\n- Configure monitors\n- Set up alert channels\n- Manage credentials\n- Configure Ollama');
}

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
    if (updateInterval) {
        clearInterval(updateInterval);
    }
});
