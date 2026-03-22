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
    setupSettingsModalListeners();
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
        const detailsList = await invoke('list_monitors_with_details');
        for (const d of detailsList) {
            try {
                const status = await invoke('get_monitor_status', { monitorId: d.id });
                const s = status || { is_up: false, response_time_ms: null, error: 'Not checked yet' };
                const monitorItem = createMonitorItem(d.id, s, d.name);
                monitorsList.appendChild(monitorItem);
            } catch (err) {
                console.error(`Failed to load monitor ${d.id}:`, err);
            }
        }
    } catch (err) {
        console.error('Failed to load monitors:', err);
    }
}

function createMonitorItem(monitorId, status, displayName) {
    const name = displayName != null ? displayName : monitorId;
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
        <div>${escapeHtml(name)}</div>
        <div style="font-size: 12px; color: rgba(255,255,255,0.6);">
            ${status.response_time_ms ? `${status.response_time_ms}ms` : '--'}
            ${status.error ? ` · ${escapeHtml(status.error)}` : ''}
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
    // Open Settings modal on Monitors tab so user can add via the form
    showSettingsDialog('monitors');
}

function showSettingsDialog(openTab) {
    const modal = document.getElementById('settings-modal');
    modal.setAttribute('aria-hidden', 'false');
    const tab = openTab || 'monitors';
    switchSettingsTab(tab);
    loadSettingsMonitors();
    loadSettingsAlertChannels();
    loadSettingsSchedules();
    loadSettingsSkills();
    loadSettingsDownloads();
    loadSettingsOllama();
    setupAlertChannelTypeVisibility();
    setupScheduleTypeVisibility();
}

function hideSettingsDialog() {
    document.getElementById('settings-modal').setAttribute('aria-hidden', 'true');
}

function switchSettingsTab(tabName) {
    document.querySelectorAll('.settings-tab').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tabName);
    });
    document.querySelectorAll('.settings-panel').forEach(panel => {
        panel.classList.toggle('active', panel.id === `settings-${tabName}`);
    });
}

async function loadSettingsMonitors() {
    const listEl = document.getElementById('settings-monitors-list');
    listEl.innerHTML = '';
    try {
        const monitors = await invoke('list_monitors_with_details');
        if (monitors.length === 0) {
            listEl.innerHTML = '<p class="settings-empty">No monitors. Add one below.</p>';
            return;
        }
        for (const m of monitors) {
            const item = document.createElement('div');
            item.className = 'settings-list-item';
            item.innerHTML = `
                <div>
                    <span class="label">${escapeHtml(m.name)}</span>
                    <div class="sublabel">${escapeHtml(m.url || m.id)}</div>
                </div>
                <button type="button" class="btn-remove" data-monitor-id="${escapeHtml(m.id)}">Remove</button>
            `;
            item.querySelector('.btn-remove').addEventListener('click', () => removeMonitorFromSettings(m.id));
            listEl.appendChild(item);
        }
    } catch (err) {
        listEl.innerHTML = `<p class="settings-error">Failed to load: ${escapeHtml(String(err))}</p>`;
    }
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

async function removeMonitorFromSettings(monitorId) {
    try {
        await invoke('remove_monitor', { monitorId });
        loadSettingsMonitors();
        if (!monitorsCollapsed) {
            loadMonitors();
            updateMonitorsSummary();
        }
    } catch (err) {
        alert(`Failed to remove monitor: ${err}`);
    }
}

function setupSettingsModalListeners() {
    document.getElementById('settings-modal-close').addEventListener('click', hideSettingsDialog);
    document.getElementById('settings-modal').addEventListener('click', (e) => {
        if (e.target.id === 'settings-modal') hideSettingsDialog();
    });
    document.querySelectorAll('.settings-tab').forEach(btn => {
        btn.addEventListener('click', () => switchSettingsTab(btn.dataset.tab));
    });
    document.getElementById('settings-add-monitor-btn').addEventListener('click', addMonitorFromSettings);
    document.getElementById('settings-add-alert-btn').addEventListener('click', addAlertChannelFromSettings);
    document.getElementById('settings-add-schedule-btn').addEventListener('click', addScheduleFromSettings);
    document.getElementById('alert-channel-type').addEventListener('change', setupAlertChannelTypeVisibility);
    document.getElementById('schedule-type').addEventListener('change', setupScheduleTypeVisibility);
    const refreshModelsBtn = document.getElementById('ollama-refresh-models-btn');
    if (refreshModelsBtn) refreshModelsBtn.addEventListener('click', refreshOllamaModels);
    const ollamaApplyBtn = document.getElementById('ollama-apply-btn');
    if (ollamaApplyBtn) ollamaApplyBtn.addEventListener('click', applyOllamaConfig);

    const doApply = document.getElementById('do-apply-btn');
    if (doApply) doApply.addEventListener('click', applyDownloadsOrganizerSettings);
    const doRun = document.getElementById('do-run-now-btn');
    if (doRun) doRun.addEventListener('click', runDownloadsOrganizerNow);
    const doEdit = document.getElementById('do-edit-rules-btn');
    if (doEdit) doEdit.addEventListener('click', openDownloadsRulesEditor);
    const drClose = document.getElementById('downloads-rules-modal-close');
    if (drClose) drClose.addEventListener('click', closeDownloadsRulesModal);
    const drCancel = document.getElementById('downloads-rules-cancel-btn');
    if (drCancel) drCancel.addEventListener('click', closeDownloadsRulesModal);
    const drSave = document.getElementById('downloads-rules-save-btn');
    if (drSave) drSave.addEventListener('click', saveDownloadsRulesFromModal);
    const drModal = document.getElementById('downloads-rules-modal');
    if (drModal) {
        drModal.addEventListener('click', (e) => {
            if (e.target.id === 'downloads-rules-modal') closeDownloadsRulesModal();
        });
    }
}

function setupScheduleTypeVisibility() {
    const type = document.getElementById('schedule-type').value;
    document.getElementById('schedule-row-cron').style.display = type === 'cron' ? 'block' : 'none';
    document.getElementById('schedule-row-at').style.display = type === 'at' ? 'block' : 'none';
}

async function loadSettingsSkills() {
    const listEl = document.getElementById('settings-skills-list');
    if (!listEl) return;
    listEl.innerHTML = '';
    try {
        const skills = await invoke('list_skills');
        if (skills.length === 0) {
            listEl.innerHTML = '<p class="settings-empty">No skills loaded. Add <code>skill-&lt;number&gt;-&lt;topic&gt;.md</code> files to ~/.mac-stats/agents/skills/ (see docs/016_skill_agent.md).</p>';
            return;
        }
        for (const s of skills) {
            const item = document.createElement('div');
            item.className = 'settings-list-item';
            item.innerHTML = `
                <div>
                    <span class="label">${escapeHtml(String(s.number))}-${escapeHtml(s.topic)}</span>
                    <div class="sublabel">${escapeHtml(s.path)}</div>
                </div>
            `;
            listEl.appendChild(item);
        }
    } catch (err) {
        listEl.innerHTML = `<p class="settings-error">Failed to load: ${escapeHtml(String(err))}</p>`;
    }
}

async function loadSettingsDownloads() {
    const pathHint = document.getElementById('do-rules-path-hint');
    const lastRun = document.getElementById('do-last-run');
    const lastSum = document.getElementById('do-last-summary');
    const errRow = document.getElementById('do-rules-error-row');
    const errEl = document.getElementById('do-rules-error');
    const enabled = document.getElementById('do-enabled');
    const interval = document.getElementById('do-interval');
    const daily = document.getElementById('do-daily');
    const dry = document.getElementById('do-dry-run');
    const pathEl = document.getElementById('do-path');
    const statusEl = document.getElementById('do-apply-status');
    if (!enabled || !interval) return;
    if (statusEl) statusEl.textContent = '';
    try {
        const s = await invoke('get_downloads_organizer_status');
        enabled.checked = !!s.enabled;
        interval.value = s.interval || 'off';
        if (daily) daily.value = s.dailyAtLocal || '09:00';
        if (dry) dry.checked = !!s.dryRun;
        if (pathEl) pathEl.value = s.pathRaw || '';
        if (pathHint) {
            pathHint.innerHTML = `Rules file: <code>${escapeHtml(s.rulesPath || '')}</code>`;
        }
        if (lastRun) {
            lastRun.textContent = s.lastRunUtc
                ? `Last run (UTC): ${escapeHtml(s.lastRunUtc)}`
                : 'Last run: never';
        }
        if (lastSum) {
            const dr = s.lastDryRun ? 'yes' : 'no';
            lastSum.textContent = `Last result: moved ${s.moved}, skipped ${s.skipped}, failed ${s.failed} (dry-run was ${dr})`;
        }
        if (s.rulesError) {
            if (errRow) errRow.style.display = 'block';
            if (errEl) errEl.textContent = `Rules error: ${escapeHtml(s.rulesError)}`;
        } else {
            if (errRow) errRow.style.display = 'none';
            if (errEl) errEl.textContent = '';
        }
    } catch (err) {
        if (lastRun) lastRun.textContent = `Failed to load status: ${escapeHtml(String(err))}`;
    }
}

async function applyDownloadsOrganizerSettings() {
    const statusEl = document.getElementById('do-apply-status');
    const enabled = document.getElementById('do-enabled');
    const interval = document.getElementById('do-interval');
    const daily = document.getElementById('do-daily');
    const dry = document.getElementById('do-dry-run');
    const pathEl = document.getElementById('do-path');
    if (!enabled || !interval || !dry) return;
    const patch = {
        enabled: enabled.checked,
        interval: interval.value,
        dailyAtLocal: daily ? daily.value.trim() : undefined,
        dryRun: dry.checked
    };
    const pr = pathEl ? pathEl.value.trim() : '';
    if (pr) patch.path = pr;
    if (statusEl) statusEl.textContent = 'Saving…';
    try {
        await invoke('set_downloads_organizer_settings', { patch });
        if (statusEl) statusEl.textContent = 'Saved. Config is read on each run (no restart needed).';
        loadSettingsDownloads();
    } catch (err) {
        if (statusEl) statusEl.textContent = `Failed: ${escapeHtml(String(err))}`;
    }
}

async function runDownloadsOrganizerNow() {
    const statusEl = document.getElementById('do-apply-status');
    if (statusEl) statusEl.textContent = 'Running organizer…';
    try {
        const msg = await invoke('run_downloads_organizer_now');
        if (statusEl) statusEl.textContent = escapeHtml(String(msg));
        loadSettingsDownloads();
    } catch (err) {
        if (statusEl) statusEl.textContent = `Failed: ${escapeHtml(String(err))}`;
    }
}

function openDownloadsRulesEditor() {
    const modal = document.getElementById('downloads-rules-modal');
    const ta = document.getElementById('downloads-rules-textarea');
    const st = document.getElementById('downloads-rules-save-status');
    if (st) st.textContent = '';
    if (!modal || !ta) return;
    invoke('read_downloads_organizer_rules')
        .then((text) => {
            ta.value = text;
            modal.setAttribute('aria-hidden', 'false');
        })
        .catch((err) => {
            alert(`Could not load rules: ${err}`);
        });
}

function closeDownloadsRulesModal() {
    const modal = document.getElementById('downloads-rules-modal');
    if (modal) modal.setAttribute('aria-hidden', 'true');
}

async function saveDownloadsRulesFromModal() {
    const ta = document.getElementById('downloads-rules-textarea');
    const st = document.getElementById('downloads-rules-save-status');
    if (!ta) return;
    if (st) st.textContent = 'Saving…';
    try {
        await invoke('save_downloads_organizer_rules', { content: ta.value });
        if (st) st.textContent = 'Saved.';
        closeDownloadsRulesModal();
        loadSettingsDownloads();
    } catch (err) {
        if (st) st.textContent = `Failed: ${escapeHtml(String(err))}`;
    }
}

async function loadSettingsOllama() {
    const endpointEl = document.getElementById('ollama-endpoint');
    const modelEl = document.getElementById('ollama-model');
    const statusEl = document.getElementById('ollama-config-status');
    if (!endpointEl || !modelEl) return;
    try {
        const config = await invoke('get_ollama_config');
        if (config) {
            endpointEl.value = config.endpoint || 'http://localhost:11434';
            if (statusEl) statusEl.textContent = '';
            await refreshOllamaModels();
            if (config.model && modelEl.options.length > 0) {
                const found = Array.from(modelEl.options).find(o => o.value === config.model);
                if (found) modelEl.value = config.model; else modelEl.value = modelEl.options[0]?.value || '';
            }
        } else {
            endpointEl.value = localStorage.getItem('ollama_endpoint') || 'http://localhost:11434';
            modelEl.innerHTML = '<option value="">— Load models first —</option>';
            if (statusEl) statusEl.textContent = 'Not configured. Enter endpoint, refresh models, select model, then Apply.';
        }
    } catch (err) {
        endpointEl.value = 'http://localhost:11434';
        modelEl.innerHTML = '<option value="">— Load models first —</option>';
        if (statusEl) statusEl.textContent = '';
    }
}

async function refreshOllamaModels() {
    const endpointEl = document.getElementById('ollama-endpoint');
    const modelEl = document.getElementById('ollama-model');
    const statusEl = document.getElementById('ollama-config-status');
    if (!endpointEl || !modelEl) return;
    const endpoint = endpointEl.value.trim() || 'http://localhost:11434';
    if (statusEl) statusEl.textContent = 'Loading…';
    try {
        const models = await invoke('list_ollama_models_at_endpoint', { endpoint });
        const current = modelEl.value;
        modelEl.innerHTML = models.length === 0
            ? '<option value="">No models at this endpoint</option>'
            : '<option value="">— Select model —</option>' + models.map(m => `<option value="${escapeHtml(m)}">${escapeHtml(m)}</option>`).join('');
        if (current && models.includes(current)) modelEl.value = current;
        if (statusEl) statusEl.textContent = '';
    } catch (err) {
        modelEl.innerHTML = '<option value="">— Load failed —</option>';
        if (statusEl) statusEl.textContent = `Failed: ${escapeHtml(String(err))}`;
    }
}

async function applyOllamaConfig() {
    const endpointEl = document.getElementById('ollama-endpoint');
    const modelEl = document.getElementById('ollama-model');
    const statusEl = document.getElementById('ollama-config-status');
    if (!endpointEl || !modelEl) return;
    const endpoint = (endpointEl.value.trim() || 'http://localhost:11434').replace(/\/$/, '');
    const model = modelEl.value?.trim();
    if (!model) {
        if (statusEl) statusEl.textContent = 'Select a model first (use Refresh models if needed).';
        return;
    }
    if (statusEl) statusEl.textContent = 'Applying…';
    try {
        await invoke('configure_ollama', { endpoint, model });
        if (window.Ollama?.saveOllamaEndpoint) window.Ollama.saveOllamaEndpoint(endpoint);
        const ok = await invoke('check_ollama_connection');
        if (statusEl) statusEl.textContent = ok ? 'Saved and connected.' : 'Saved; connection check failed.';
        if (window.Ollama?.checkConnection) window.Ollama.checkConnection();
    } catch (err) {
        if (statusEl) statusEl.textContent = `Failed: ${escapeHtml(String(err))}`;
    }
}

async function loadSettingsSchedules() {
    const listEl = document.getElementById('settings-schedules-list');
    if (!listEl) return;
    listEl.innerHTML = '';
    try {
        const schedules = await invoke('list_schedules');
        if (schedules.length === 0) {
            listEl.innerHTML = '<p class="settings-empty">No schedules. Add one below.</p>';
            return;
        }
        for (const s of schedules) {
            const id = s.id || '(no id)';
            const when = s.cron ? `cron: ${s.cron}` : (s.at ? `at: ${s.at}` : '—');
            const next = s.next_run ? `Next: ${s.next_run}` : '';
            const item = document.createElement('div');
            item.className = 'settings-list-item';
            item.innerHTML = `
                <div>
                    <span class="label">${escapeHtml(id)}</span>
                    <div class="sublabel">${escapeHtml(when)} ${escapeHtml(next)}</div>
                    <div class="sublabel">${escapeHtml(s.task.slice(0, 60))}${s.task.length > 60 ? '…' : ''}</div>
                </div>
                <button type="button" class="btn-remove" data-schedule-id="${escapeHtml(id)}">Remove</button>
            `;
            item.querySelector('.btn-remove').addEventListener('click', () => removeScheduleFromSettings(id));
            listEl.appendChild(item);
        }
    } catch (err) {
        listEl.innerHTML = `<p class="settings-error">Failed to load: ${escapeHtml(String(err))}</p>`;
    }
}

async function removeScheduleFromSettings(scheduleId) {
    try {
        await invoke('remove_schedule', { scheduleId });
        loadSettingsSchedules();
    } catch (err) {
        alert(`Failed to remove schedule: ${err}`);
    }
}

async function addScheduleFromSettings() {
    const type = document.getElementById('schedule-type').value;
    const task = document.getElementById('schedule-task').value.trim();
    if (!task) {
        alert('Please enter a task.');
        return;
    }
    const replyToChannelId = document.getElementById('schedule-reply-channel').value.trim() || null;
    try {
        if (type === 'cron') {
            const cron = document.getElementById('schedule-cron').value.trim();
            if (!cron) {
                alert('Please enter a cron expression (e.g. 0 0 9 * * * for daily at 09:00).');
                return;
            }
            const result = await invoke('add_schedule', { cron, task, replyToChannelId });
            if (result.status === 'AlreadyExists') {
                alert('A schedule with the same cron and task already exists.');
                return;
            }
        } else {
            const at = document.getElementById('schedule-at').value.trim();
            if (!at) {
                alert('Please enter date and time (e.g. 2025-03-17T09:00:00).');
                return;
            }
            const result = await invoke('add_schedule_at', { at, task, replyToChannelId });
            if (result.status === 'AlreadyExists') {
                alert('A one-shot schedule with the same time and task already exists.');
                return;
            }
        }
        document.getElementById('schedule-task').value = '';
        document.getElementById('schedule-reply-channel').value = '';
        loadSettingsSchedules();
    } catch (err) {
        alert(`Failed to add schedule: ${err}`);
    }
}

function setupAlertChannelTypeVisibility() {
    const type = document.getElementById('alert-channel-type').value;
    document.getElementById('alert-row-chat-id').style.display = type === 'telegram' ? 'block' : 'none';
    document.getElementById('alert-row-instance').style.display = type === 'mastodon' ? 'block' : 'none';
}

async function addMonitorFromSettings() {
    const name = document.getElementById('monitor-name').value.trim();
    const url = document.getElementById('monitor-url').value.trim();
    if (!url) {
        alert('Please enter a URL.');
        return;
    }
    const id = `monitor_${Date.now()}`;
    const displayName = name || new URL(url).hostname || id;
    const timeoutSecs = parseInt(document.getElementById('monitor-timeout').value, 10) || 10;
    const intervalSecs = parseInt(document.getElementById('monitor-interval').value, 10) || 60;
    const verifySsl = document.getElementById('monitor-verify-ssl').checked;
    try {
        await invoke('add_website_monitor', {
            request: {
                id,
                name: displayName,
                url,
                timeout_secs: timeoutSecs,
                check_interval_secs: intervalSecs,
                verify_ssl: verifySsl
            }
        });
        document.getElementById('monitor-name').value = '';
        document.getElementById('monitor-url').value = '';
        loadSettingsMonitors();
        if (!monitorsCollapsed) {
            loadMonitors();
            updateMonitorsSummary();
        }
    } catch (err) {
        alert(`Failed to add monitor: ${err}`);
    }
}

async function loadSettingsAlertChannels() {
    const listEl = document.getElementById('settings-alerts-list');
    listEl.innerHTML = '';
    try {
        const channelIds = await invoke('list_alert_channels');
        if (channelIds.length === 0) {
            listEl.innerHTML = '<p class="settings-empty">No alert channels. Add one below.</p>';
            return;
        }
        for (const id of channelIds) {
            const item = document.createElement('div');
            item.className = 'settings-list-item';
            item.innerHTML = `
                <div><span class="label">${escapeHtml(id)}</span></div>
                <button type="button" class="btn-remove" data-channel-id="${escapeHtml(id)}">Remove</button>
            `;
            item.querySelector('.btn-remove').addEventListener('click', () => removeAlertChannelFromSettings(id));
            listEl.appendChild(item);
        }
    } catch (err) {
        listEl.innerHTML = `<p class="settings-error">Failed to load: ${escapeHtml(String(err))}</p>`;
    }
}

async function removeAlertChannelFromSettings(channelId) {
    try {
        await invoke('remove_alert_channel', { channelId });
        loadSettingsAlertChannels();
    } catch (err) {
        alert(`Failed to remove channel: ${err}`);
    }
}

async function addAlertChannelFromSettings() {
    const type = document.getElementById('alert-channel-type').value;
    const id = document.getElementById('alert-channel-id').value.trim();
    if (!id) {
        alert('Please enter a channel ID.');
        return;
    }
    try {
        if (type === 'telegram') {
            const chatId = document.getElementById('alert-telegram-chat-id').value.trim();
            if (!chatId) {
                alert('Please enter the Telegram chat ID.');
                return;
            }
            await invoke('register_telegram_channel', { id, chatId });
        } else if (type === 'slack') {
            await invoke('register_slack_channel', { id });
        } else if (type === 'mastodon') {
            const instanceUrl = document.getElementById('alert-mastodon-instance').value.trim();
            if (!instanceUrl) {
                alert('Please enter the Mastodon instance URL.');
                return;
            }
            await invoke('register_mastodon_channel', { id, instanceUrl });
        }
        document.getElementById('alert-channel-id').value = '';
        document.getElementById('alert-telegram-chat-id').value = '';
        document.getElementById('alert-mastodon-instance').value = '';
        loadSettingsAlertChannels();
    } catch (err) {
        alert(`Failed to add channel: ${err}`);
    }
}

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
    if (updateInterval) {
        clearInterval(updateInterval);
    }
});
