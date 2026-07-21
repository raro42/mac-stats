/**
 * Agent Ops Command Center — shared by dashboard.html and theme cpu.html.
 */
(function () {
  'use strict';

  function opsInvoke(cmd, args) {
    const fn =
      window.__TAURI__?.core?.invoke ??
      window.__TAURI_INTERNALS__?.invoke ??
      (typeof window.invoke === 'function' ? window.invoke.bind(window) : null);
    if (!fn) throw new Error('Tauri invoke not available');
    return fn(cmd, args);
  }
  const invoke = (...a) => opsInvoke(...a);

  /** Escape clears the focused filter (Hermes-style: Escape skips / clears, does not leave the panel). */
  function bindOpsFilterEscape(input, onClear) {
    input.addEventListener('keydown', (e) => {
      if (e.key !== 'Escape') return;
      if (!(input.value || '').length) return;
      e.preventDefault();
      e.stopPropagation();
      input.value = '';
      onClear();
    });
  }

  const OPS_REFRESH_INTERVAL = 30000;
  let agentOpsInterval = null;
  let agentOpsCollapsed = true;
  let opsAgentCache = null;
  let opsAgentFileTab = 'soul';
  let opsRefreshInFlight = false;
  let opsSessionLoadRows = null;
  let opsSessionFilterQ = '';
  let opsLiveCache = [];
  let opsSessionFilesCache = [];
  let opsMemoryFilterQ = '';
  let opsMemoryCache = [];
  let opsRunsFilterQ = '';
  let opsRunsInsightsCache = null;
  let opsAgentsFilterQ = '';
  let opsAgentsCache = [];
  let opsSchedulesFilterQ = '';
  let opsSchedulesCache = [];
  let opsDeliveriesCache = [];

// --- Agent Ops (Command Center: overview + detail tabs) ---

function selectOpsTab(tab) {
    document.querySelectorAll('.agent-ops-tab').forEach((b) => {
        b.classList.toggle('active', b.dataset.opsTab === tab);
    });
    document.querySelectorAll('.agent-ops-panel').forEach((p) => {
        p.classList.toggle('active', p.id === `ops-panel-${tab}`);
    });
    const panel = document.getElementById(`ops-panel-${tab}`);
    const tabs = document.querySelector('.agent-ops-tabs');
    (panel || tabs)?.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
}

function setupAgentOps() {
    document.querySelectorAll('.agent-ops-tab').forEach((btn) => {
        btn.addEventListener('click', () => selectOpsTab(btn.dataset.opsTab));
    });
    document.querySelectorAll('.ops-overview-link').forEach((btn) => {
        btn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            const tab = btn.dataset.gotoTab;
            if (!tab) return;
            // Ensure pane is open (dashboard can leave it collapsed)
            if (agentOpsCollapsed) applyOpsCollapsed(false);
            selectOpsTab(tab);
        });
    });
    document.querySelectorAll('.ops-file-tab').forEach((btn) => {
        btn.addEventListener('click', () => {
            opsAgentFileTab = btn.dataset.file;
            document.querySelectorAll('.ops-file-tab').forEach((b) => b.classList.toggle('active', b === btn));
            renderOpsAgentPreview();
        });
    });
    document.getElementById('ops-agent-back')?.addEventListener('click', () => {
        document.getElementById('ops-agent-detail').hidden = true;
        document.getElementById('ops-agents-list').style.display = '';
    });
    document.getElementById('ops-refresh-btn')?.addEventListener('click', () => refreshAgentOps());
    document.getElementById('ops-digest-refresh-btn')?.addEventListener('click', () => refreshOpsDigest());
    document.getElementById('ops-session-load-chat')?.addEventListener('click', () => loadOpsSessionIntoChat());
    ensureOpsSessionFilter();
    ensureOpsMemoryFilter();
    ensureOpsRunsFilter();
    ensureOpsAgentsFilter();
    ensureOpsSchedulesFilter();
    if (!agentOpsCollapsed) {
      refreshAgentOps();
      startAgentOpsAutoRefresh();
    }
}

function ensureOpsSessionFilter() {
    const panel = document.getElementById('ops-panel-sessions');
    if (!panel) return;
    let input = document.getElementById('ops-session-filter');
    if (!input) {
        const row = document.createElement('div');
        row.className = 'ops-filter-row';
        input = document.createElement('input');
        input.type = 'search';
        input.id = 'ops-session-filter';
        input.className = 'ops-filter-input';
        input.placeholder = 'Filter live + files…';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        panel.insertBefore(row, panel.firstChild);
    }
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    input.addEventListener('input', () => {
        opsSessionFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsLive(opsLiveCache);
        renderOpsSessionFiles(opsSessionFilesCache);
    });
    bindOpsFilterEscape(input, () => {
        opsSessionFilterQ = '';
        renderOpsLive(opsLiveCache);
        renderOpsSessionFiles(opsSessionFilesCache);
    });
}

function sessionRowMatchesFilter(haystack) {
    if (!opsSessionFilterQ) return true;
    return String(haystack || '').toLowerCase().includes(opsSessionFilterQ);
}

function ensureOpsMemoryFilter() {
    const panel = document.getElementById('ops-panel-memory');
    if (!panel) return;
    let input = document.getElementById('ops-memory-filter');
    if (!input) {
        const row = document.createElement('div');
        row.className = 'ops-filter-row';
        input = document.createElement('input');
        input.type = 'search';
        input.id = 'ops-memory-filter';
        input.className = 'ops-filter-input';
        input.placeholder = 'Filter knowledge files…';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        panel.insertBefore(row, panel.firstChild);
    }
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    input.addEventListener('input', () => {
        opsMemoryFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsMemory(opsMemoryCache);
    });
    bindOpsFilterEscape(input, () => {
        opsMemoryFilterQ = '';
        renderOpsMemory(opsMemoryCache);
    });
}

function memoryRowMatchesFilter(haystack) {
    if (!opsMemoryFilterQ) return true;
    return String(haystack || '').toLowerCase().includes(opsMemoryFilterQ);
}

function ensureOpsRunsFilter() {
    const panel = document.getElementById('ops-panel-runs');
    if (!panel) return;
    let input = document.getElementById('ops-runs-filter');
    if (!input) {
        const row = document.createElement('div');
        row.className = 'ops-filter-row';
        input = document.createElement('input');
        input.type = 'search';
        input.id = 'ops-runs-filter';
        input.className = 'ops-filter-input';
        input.placeholder = 'Filter runs by lane, tool, question…';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        const insights = document.getElementById('ops-runs-insights');
        if (insights) panel.insertBefore(row, insights.nextSibling);
        else panel.insertBefore(row, panel.firstChild);
    }
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    input.addEventListener('input', () => {
        opsRunsFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsRuns(opsRunsInsightsCache);
    });
    bindOpsFilterEscape(input, () => {
        opsRunsFilterQ = '';
        renderOpsRuns(opsRunsInsightsCache);
    });
}

function runsRowMatchesFilter(haystack) {
    if (!opsRunsFilterQ) return true;
    return String(haystack || '').toLowerCase().includes(opsRunsFilterQ);
}

function ensureOpsAgentsFilter() {
    const panel = document.getElementById('ops-panel-agents');
    if (!panel) return;
    let input = document.getElementById('ops-agents-filter');
    if (!input) {
        const row = document.createElement('div');
        row.className = 'ops-filter-row';
        input = document.createElement('input');
        input.type = 'search';
        input.id = 'ops-agents-filter';
        input.className = 'ops-filter-input';
        input.placeholder = 'Filter agents by name, slug, model…';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        const list = document.getElementById('ops-agents-list');
        if (list) panel.insertBefore(row, list);
        else panel.insertBefore(row, panel.firstChild);
    }
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    input.addEventListener('input', () => {
        opsAgentsFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsAgents(opsAgentsCache);
    });
    bindOpsFilterEscape(input, () => {
        opsAgentsFilterQ = '';
        renderOpsAgents(opsAgentsCache);
    });
}

function agentsRowMatchesFilter(haystack) {
    if (!opsAgentsFilterQ) return true;
    return String(haystack || '').toLowerCase().includes(opsAgentsFilterQ);
}

function ensureOpsSchedulesFilter() {
    const panel = document.getElementById('ops-panel-schedules');
    if (!panel) return;
    let input = document.getElementById('ops-schedules-filter');
    if (!input) {
        const row = document.createElement('div');
        row.className = 'ops-filter-row';
        input = document.createElement('input');
        input.type = 'search';
        input.id = 'ops-schedules-filter';
        input.className = 'ops-filter-input';
        input.placeholder = 'Filter schedules + deliveries…';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        const list = document.getElementById('ops-schedules-list');
        const sub = panel.querySelector('.ops-subhead');
        if (sub) panel.insertBefore(row, sub);
        else if (list) panel.insertBefore(row, list);
        else panel.insertBefore(row, panel.firstChild);
    }
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    input.addEventListener('input', () => {
        opsSchedulesFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsSchedulesTab(opsSchedulesCache, opsDeliveriesCache);
    });
    bindOpsFilterEscape(input, () => {
        opsSchedulesFilterQ = '';
        renderOpsSchedulesTab(opsSchedulesCache, opsDeliveriesCache);
    });
}

function schedulesRowMatchesFilter(haystack) {
    if (!opsSchedulesFilterQ) return true;
    return String(haystack || '').toLowerCase().includes(opsSchedulesFilterQ);
}

function startAgentOpsAutoRefresh() {
    if (agentOpsInterval) return;
    agentOpsInterval = setInterval(() => {
        if (agentOpsCollapsed || opsRefreshInFlight) return;
        refreshAgentOps();
    }, OPS_REFRESH_INTERVAL);
}

function stopAgentOpsAutoRefresh() {
    if (agentOpsInterval) {
        clearInterval(agentOpsInterval);
        agentOpsInterval = null;
    }
}

async function refreshOpsDigest() {
    const btn = document.getElementById('ops-digest-refresh-btn');
    const digestEl = document.getElementById('ops-health-digest');
    if (btn) {
        btn.disabled = true;
        btn.textContent = 'Refreshing…';
    }
    try {
        const msg = await invoke('refresh_agent_digest');
        if (digestEl) digestEl.textContent = String(msg).slice(0, 80);
        await refreshAgentOps();
    } catch (err) {
        console.warn('[Agent Ops] digest refresh', err);
        if (digestEl) digestEl.textContent = `Refresh failed`;
    } finally {
        if (btn) {
            btn.disabled = false;
            btn.textContent = 'Refresh digest';
        }
    }
}

function fmtBytes(n) {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

function fmtAge(ms) {
    if (!ms) return '';
    const age = Date.now() - ms;
    if (age < 60_000) return 'just now';
    if (age < 3600_000) return `${Math.floor(age / 60_000)}m ago`;
    if (age < 86400_000) return `${Math.floor(age / 3600_000)}h ago`;
    return `${Math.floor(age / 86400_000)}d ago`;
}

function fmtScheduleEta(sched) {
    if (!sched || sched.totalEntries == null) return '—';
    if (sched.totalEntries === 0) return 'None';
    if (sched.secondsUntilNextFire == null) return `${sched.totalEntries} jobs`;
    const secs = Number(sched.secondsUntilNextFire);
    const when =
        secs < 3600
            ? `${Math.max(1, Math.round(secs / 60))}m`
            : `${Math.round(secs / 3600)}h`;
    const preview = sched.nextTaskPreview
        ? String(sched.nextTaskPreview).slice(0, 32)
        : '';
    return preview ? `${when} · ${preview}` : when;
}

function setText(id, text) {
    const el = document.getElementById(id);
    if (el) el.textContent = text;
}

function renderOpsHealth({ version, insights, sched, deliveries, agents, live, redmine }) {
    const enabled = (agents || []).filter((a) => a.enabled).length;
    setText(
        'ops-health-version',
        version ? `v${version}` : '—'
    );
    const agentsHint = document.getElementById('ops-health-version');
    if (agentsHint && version) {
        agentsHint.title = `${enabled}/${(agents || []).length} agents · ${(live || []).length} live`;
    }

    const dg = insights?.discord_gateway || '';
    const readyMatch = dg.match(/last Ready\s+([^·]+)/i);
    const discMatch = dg.match(/disconnect×(\d+)/i);
    const resumeMatch = dg.match(/resume×(\d+)/i);
    const stageMatch = dg.match(/stage=([^\s·]+)/i);
    const discN = discMatch ? Number(discMatch[1]) : 0;
    const resumeN = resumeMatch ? Number(resumeMatch[1]) : 0;
    const stage = (stageMatch ? stageMatch[1] : '').trim();
    let discordText = readyMatch ? readyMatch[1].trim() : dg ? 'see Runs' : '—';
    if (discN > 0) {
        discordText = `${discordText} · disc×${discN}`;
    }
    setText('ops-health-discord', discordText);
    const discordEl = document.getElementById('ops-health-discord');
    if (discordEl) {
        discordEl.title = dg || '';
        const card = discordEl.closest('.ops-health-card');
        if (card) {
            card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
            const stageLower = stage.toLowerCase();
            if (!dg || discordText === '—') {
                /* leave neutral */
            } else if (stageLower === 'disconnected') {
                card.classList.add('ops-health-bad');
            } else if (discN > 0 || resumeN > 0 || stageLower === 'resuming') {
                card.classList.add('ops-health-warn');
            } else if (stageLower === 'connected' || readyMatch) {
                card.classList.add('ops-health-ok');
            }
        }
    }

        if (redmine) {
        const st = String(redmine.status || '').toLowerCase();
        const msg = String(redmine.message || '').trim();
        let text = '—';
        if (st === 'ok') text = msg || 'Ok';
        else if (st === 'notconfigured') text = 'Not configured';
        else if (st) text = msg ? `${st}: ${msg}`.slice(0, 36) : st;
        setText('ops-health-redmine', text);
        const el = document.getElementById('ops-health-redmine');
        if (el) {
            el.title = msg || st || '';
            const card = el.closest('.ops-health-card');
            if (card) {
                card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
                if (st === 'ok') card.classList.add('ops-health-ok');
                else if (st === 'notconfigured' || st === 'degraded') card.classList.add('ops-health-warn');
                else if (st === 'unavailable') card.classList.add('ops-health-bad');
            }
        }
        if (typeof syncOpsIconHealth === 'function') syncOpsIconHealth(redmine);
    }

    setText('ops-health-schedule', fmtScheduleEta(sched));

    let deliveryText = '—';
    if (Array.isArray(deliveries) && deliveries.length) {
        const newest = deliveries[0];
        const t = newest?.utc ? Date.parse(newest.utc) : NaN;
        deliveryText = !Number.isNaN(t) ? fmtAge(t) : (newest.utc || '—');
    }
    setText('ops-health-delivery', deliveryText);

    let digestText = '—';
    if (insights) {
        const open = insights.digest_open_count ?? 0;
        const stale = insights.digest_stale_count ?? 0;
        let age = '';
        if (insights.digest_generated_at) {
            const t = Date.parse(insights.digest_generated_at);
            if (!Number.isNaN(t)) age = ` · ${fmtAge(t)}`;
        }
        digestText = `${open} open / ${stale} stale${age}`;
    }
    setText('ops-health-digest', digestText);
    const digestEl = document.getElementById('ops-health-digest');
    if (digestEl) {
        const openN = insights?.digest_open_count ?? 0;
        const hints = insights?.digest_open_hints || [];
        digestEl.title = hints.length
            ? hints.slice(0, 5).join('\n')
            : insights?.digest_generated_at
              ? `Generated ${insights.digest_generated_at}`
              : '';
        const card = digestEl.closest('.ops-health-card');
        if (card) {
            card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
            if (openN > 0) card.classList.add('ops-health-warn');
            else if (insights) card.classList.add('ops-health-ok');
            card.style.cursor = 'pointer';
            if (card.dataset.opsDigestClick !== '1') {
                card.dataset.opsDigestClick = '1';
                card.addEventListener('click', () => {
                    if (agentOpsCollapsed) applyOpsCollapsed(false);
                    selectOpsTab('runs');
                });
            }
        }
    }
}

function renderOverviewSchedules(schedules, deliveries) {
    const body = document.getElementById('ops-overview-schedules-body');
    if (!body) return;
    body.innerHTML = '';
    if (!schedules || !schedules.length) {
        body.innerHTML = '<div class="ops-empty">No schedules</div>';
        return;
    }
    const count = document.createElement('div');
    count.className = 'ops-overview-count';
    count.textContent = `${schedules.length} active`;
    body.appendChild(count);
    schedules.slice(0, 3).forEach((s) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const id = s.id || '(no id)';
        const next = s.next_run || s.nextRun || '—';
        const task = String(s.task || '').slice(0, 40);
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(id)}</div><div class="ops-row-meta">next ${escapeHtml(next)} · ${escapeHtml(task)}</div></div>`;
        btn.addEventListener('click', () => selectOpsTab('schedules'));
        body.appendChild(btn);
    });
    if (Array.isArray(deliveries) && deliveries.length) {
        const t = deliveries[0]?.utc ? Date.parse(deliveries[0].utc) : NaN;
        const meta = document.createElement('div');
        meta.className = 'ops-overview-count';
        meta.textContent = !Number.isNaN(t)
            ? `Last delivery ${fmtAge(t)}`
            : 'Last delivery recorded';
        body.appendChild(meta);
    }
}

function renderOverviewLive(rows) {
    const body = document.getElementById('ops-overview-live-body');
    if (!body) return;
    body.innerHTML = '';
    if (!rows || !rows.length) {
        body.innerHTML = '<div class="ops-empty">No live sessions</div>';
        return;
    }
    const count = document.createElement('div');
    count.className = 'ops-overview-count';
    count.textContent = `${rows.length} live`;
    body.appendChild(count);
    rows.slice(0, 3).forEach((r) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(r.source)} · ${r.session_id}</div><div class="ops-row-meta">${r.message_count} msgs${r.preview ? ` · ${escapeHtml(r.preview)}` : ''}</div></div>`;
        btn.addEventListener('click', async () => {
            selectOpsTab('sessions');
            try {
                const msgs = await invoke('read_live_session_messages', {
                    source: r.source,
                    sessionId: r.session_id,
                });
                showOpsSessionPreview(msgs, `Live ${r.source} · ${r.session_id}`);
                showOpsSessionStatus('Preview ready — use “Load into AI Chat” on the Sessions tab.', true);
            } catch (err) {
                showOpsSessionPreview([], String(err));
                showOpsSessionStatus(String(err), false);
            }
        });
        body.appendChild(btn);
    });
}

function renderOverviewKnowledge(files) {
    const body = document.getElementById('ops-overview-knowledge-body');
    if (!body) return;
    body.innerHTML = '';
    if (!files || !files.length) {
        body.innerHTML = '<div class="ops-empty">No knowledge files</div>';
        return;
    }
    const sorted = [...files].sort((a, b) => (b.modified_ms || 0) - (a.modified_ms || 0));
    const count = document.createElement('div');
    count.className = 'ops-overview-count';
    count.textContent = `${files.length} files`;
    body.appendChild(count);
    sorted.slice(0, 4).forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.name)}</div><div class="ops-row-meta">${escapeHtml(f.kind)} · ${fmtAge(f.modified_ms)}</div></div>`;
        btn.addEventListener('click', async () => {
            selectOpsTab('memory');
            const preview = document.getElementById('ops-memory-preview');
            try {
                const text = await invoke('read_memory_file', { path: f.path });
                if (preview) {
                    preview.hidden = false;
                    preview.textContent = text.slice(0, 12000);
                }
            } catch (err) {
                if (preview) {
                    preview.hidden = false;
                    preview.textContent = String(err);
                }
            }
        });
        body.appendChild(btn);
    });
}

function renderOverviewRecent(files) {
    const body = document.getElementById('ops-overview-recent-body');
    if (!body) return;
    body.innerHTML = '';
    if (!files || !files.length) {
        body.innerHTML = '<div class="ops-empty">No recent chats</div>';
        return;
    }
    const count = document.createElement('div');
    count.className = 'ops-overview-count';
    count.textContent = `${Math.min(5, files.length)} of ${files.length}`;
    body.appendChild(count);
    files.slice(0, 5).forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.slug || f.name)}</div><div class="ops-row-meta">${escapeHtml(f.source_hint)} · ${fmtAge(f.modified_ms)}</div></div>`;
        btn.addEventListener('click', async () => {
            selectOpsTab('sessions');
            try {
                const msgs = await invoke('read_session_file_messages', { path: f.path });
                if (msgs && msgs.length) {
                    showOpsSessionPreview(msgs, f.name);
                } else {
                    const text = await invoke('read_session_file', { path: f.path });
                    const preview = document.getElementById('ops-session-preview');
                    const loadBtn = document.getElementById('ops-session-load-chat');
                    if (preview) {
                        preview.hidden = false;
                        preview.textContent = text.slice(0, 12000);
                    }
                    opsSessionLoadRows = null;
                    if (loadBtn) loadBtn.hidden = true;
                }
            } catch (err) {
                showOpsSessionPreview([], String(err));
            }
        });
        body.appendChild(btn);
    });
}

function renderOpsSchedulesTab(schedules, deliveries) {
    const list = document.getElementById('ops-schedules-list');
    const delList = document.getElementById('ops-deliveries-list');
    if (list) {
        list.innerHTML = '';
        const all = schedules || [];
        const filtered = all.filter((s) => {
            const when = s.cron ? `cron ${s.cron}` : s.at ? `at ${s.at}` : '';
            return schedulesRowMatchesFilter(
                `${s.id || ''} ${when} ${s.next_run || s.nextRun || ''} ${s.task || ''}`
            );
        });
        if (!all.length) {
            list.innerHTML = '<div class="ops-empty">No schedules</div>';
        } else if (!filtered.length) {
            list.innerHTML = '<div class="ops-empty">No schedules match filter</div>';
        } else {
            filtered.forEach((s) => {
                const div = document.createElement('div');
                div.className = 'ops-row';
                const id = s.id || '(no id)';
                const when = s.cron ? `cron ${s.cron}` : s.at ? `at ${s.at}` : '—';
                const next = s.next_run || s.nextRun || '—';
                const task = String(s.task || '');
                div.innerHTML = `<div><div class="ops-row-title">${escapeHtml(id)}</div><div class="ops-row-meta">${escapeHtml(when)} · next ${escapeHtml(next)}</div><div class="ops-row-meta">${escapeHtml(task.slice(0, 80))}${task.length > 80 ? '…' : ''}</div></div>`;
                list.appendChild(div);
            });
        }
    }
    if (delList) {
        delList.innerHTML = '';
        const all = deliveries || [];
        const filtered = all.filter((d) =>
            schedulesRowMatchesFilter(`${d.schedule_id || ''} ${d.summary || ''} ${d.utc || ''}`)
        );
        if (!all.length) {
            delList.innerHTML = '<div class="ops-empty">No deliveries yet</div>';
        } else if (!filtered.length) {
            delList.innerHTML = '<div class="ops-empty">No deliveries match filter</div>';
        } else {
            filtered.slice(0, 8).forEach((d) => {
                const div = document.createElement('div');
                div.className = 'ops-row';
                const t = d.utc ? Date.parse(d.utc) : NaN;
                const age = !Number.isNaN(t) ? fmtAge(t) : d.utc || '';
                const summary = String(d.summary || '').slice(0, 72);
                div.innerHTML = `<div><div class="ops-row-title">${escapeHtml(d.schedule_id || 'schedule')}</div><div class="ops-row-meta">${escapeHtml(age)} · ${escapeHtml(summary)}</div></div>`;
                delList.appendChild(div);
            });
        }
    }
}

async function refreshAgentOps() {
    const healthRow = document.getElementById('ops-health-row');
    if (opsRefreshInFlight) return;
    opsRefreshInFlight = true;
    try {
        const [agents, live, files, memory, insights, version, sched, deliveries, schedules, features] =
            await Promise.all([
                invoke('list_agents'),
                invoke('list_live_sessions'),
                invoke('list_session_files', { limit: 40 }),
                invoke('list_memory_files'),
                invoke('get_runs_insights', { limit: 40 }),
                invoke('get_app_version').catch(() => null),
                invoke('get_scheduler_snapshot').catch(() => null),
                invoke('list_scheduler_delivery_awareness').catch(() => null),
                invoke('list_schedules').catch(() => []),
                invoke('get_feature_health', { refresh: false }).catch(() => []),
            ]);
        const redmine = (features || []).find(
            (h) => String(h.name || '').toLowerCase() === 'redmine'
        );
        renderOpsHealth({
            version,
            insights,
            sched,
            deliveries,
            agents,
            live,
            redmine,
        });
        renderOverviewSchedules(schedules || [], deliveries || []);
        renderOverviewLive(live || []);
        renderOverviewKnowledge(memory || []);
        renderOverviewRecent(files || []);
        opsSchedulesCache = schedules || [];
        opsDeliveriesCache = deliveries || [];
        renderOpsSchedulesTab(opsSchedulesCache, opsDeliveriesCache);
        opsAgentsCache = agents || [];
        renderOpsAgents(opsAgentsCache);
        opsLiveCache = live || [];
        opsSessionFilesCache = files || [];
        renderOpsLive(opsLiveCache);
        renderOpsSessionFiles(opsSessionFilesCache);
        opsMemoryCache = memory || [];
        renderOpsMemory(opsMemoryCache);
        opsRunsInsightsCache = insights;
        renderOpsRuns(opsRunsInsightsCache);
    } catch (err) {
        console.warn('[Agent Ops]', err);
        if (healthRow) {
            setText('ops-health-version', 'Unavailable');
            setText('ops-health-discord', String(err).slice(0, 40));
        }
    } finally {
        opsRefreshInFlight = false;
    }
}

function renderOpsAgents(agents) {
    const list = document.getElementById('ops-agents-list');
    list.innerHTML = '';
    const all = agents || [];
    const filtered = all.filter((a) =>
        agentsRowMatchesFilter(
            `${a.name || ''} ${a.slug || ''} ${a.id || ''} ${a.model || ''} ${a.enabled ? 'on' : 'off'} ${a.orchestrator ? 'orchestrator' : ''}`
        )
    );
    if (!all.length) {
        list.innerHTML = '<div class="ops-empty">No agents under ~/.mac-stats/agents</div>';
        return;
    }
    if (!filtered.length) {
        list.innerHTML = '<div class="ops-empty">No agents match filter</div>';
        return;
    }
    filtered.forEach((a) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const slug = a.slug || a.id;
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(a.name)} <span class="ops-row-meta">· ${escapeHtml(slug)}</span></div><div class="ops-row-meta">${escapeHtml(a.model || 'default model')}${a.orchestrator ? ' · orchestrator' : ''}</div></div><span class="ops-badge ${a.enabled ? '' : 'off'}">${a.enabled ? 'on' : 'off'}</span>`;
        btn.addEventListener('click', () => openOpsAgent(a.id));
        list.appendChild(btn);
    });
}

async function openOpsAgent(id) {
    try {
        opsAgentCache = await invoke('get_agent_details', { selector: id });
        document.getElementById('ops-agents-list').style.display = 'none';
        const detail = document.getElementById('ops-agent-detail');
        detail.hidden = false;
        document.getElementById('ops-agent-meta').textContent =
            `${opsAgentCache.name} · ${opsAgentCache.slug || opsAgentCache.id} · ${opsAgentCache.model || 'default'} · ${opsAgentCache.enabled ? 'enabled' : 'disabled'}`;
        opsAgentFileTab = 'soul';
        document.querySelectorAll('.ops-file-tab').forEach((b) => {
            b.classList.toggle('active', b.dataset.file === 'soul');
        });
        renderOpsAgentPreview();
    } catch (err) {
        alert(`Failed to load agent: ${err}`);
    }
}

function renderOpsAgentPreview() {
    if (!opsAgentCache) return;
    const pre = document.getElementById('ops-agent-preview');
    const map = {
        soul: opsAgentCache.soul || '(empty soul.md)',
        skill: opsAgentCache.skill || '(empty skill.md)',
        mood: opsAgentCache.mood || '(empty mood.md)',
    };
    pre.textContent = map[opsAgentFileTab] || '';
}

function formatSessionMessagesPreview(rows) {
    if (!rows || !rows.length) return '(empty session)';
    return rows
        .map((m) => `## ${m.role === 'user' ? 'User' : 'Assistant'}\n\n${m.content}`)
        .join('\n\n');
}

function showOpsSessionPreview(rows, label) {
    const preview = document.getElementById('ops-session-preview');
    const loadBtn = document.getElementById('ops-session-load-chat');
    opsSessionLoadRows = rows && rows.length ? rows : null;
    preview.hidden = false;
    preview.textContent = (label ? `${label}\n\n` : '') + formatSessionMessagesPreview(rows || []);
    if (loadBtn) loadBtn.hidden = !opsSessionLoadRows;
}

function showOpsSessionStatus(msg, ok) {
    let el = document.getElementById('ops-session-status');
    if (!el) {
        const loadBtn = document.getElementById('ops-session-load-chat');
        el = document.createElement('div');
        el.id = 'ops-session-status';
        el.className = 'ops-row-meta';
        el.style.margin = '6px 4px 0';
        loadBtn?.parentNode?.insertBefore(el, loadBtn.nextSibling);
    }
    el.textContent = msg || '';
    el.style.opacity = msg ? '0.9' : '0';
    el.style.color = ok === false ? 'rgba(200,60,60,0.95)' : '';
}

function loadOpsSessionIntoChat() {
    if (!opsSessionLoadRows || !opsSessionLoadRows.length) {
        showOpsSessionStatus('Select a session with messages first.', false);
        return;
    }
    if (!window.Ollama?.replaceHistory) {
        showOpsSessionStatus('AI Chat module not ready — open AI Chat once, then retry.', false);
        console.warn('[Agent Ops] Ollama.replaceHistory unavailable');
        return;
    }
    // Ensure AI agent UI is usable
    const aiOff =
      document.getElementById('icon-ollama')?.style.pointerEvents === 'none' ||
      document.getElementById('ollama-section')?.style.display === 'none';
    if (aiOff) {
        showOpsSessionStatus('Enable local AI agent in Settings to load into chat.', false);
        return;
    }
    window.Ollama.replaceHistory(opsSessionLoadRows);
    const section = document.querySelector('.ollama-section');
    const themeCollapsed =
      section?.classList.contains('collapsed') ||
      localStorage.getItem('ollama_collapsed') === 'true';
    if (themeCollapsed) {
      document.getElementById('ollama-header')?.click();
    }
    const content = document.getElementById('ollama-content');
    const btn = document.getElementById('ollama-collapse-btn');
    if (content) {
      content.classList.remove('collapsed');
      if (content.style.display === 'none') content.style.display = '';
    }
    if (section) section.classList.remove('collapsed');
    if (btn) btn.textContent = '−';
    section?.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
    setTimeout(() => document.getElementById('chat-input')?.focus(), 80);
    showOpsSessionStatus(
      `Loaded ${opsSessionLoadRows.length} message(s) into AI Chat.`,
      true
    );
}

function renderOpsLive(rows) {
    const el = document.getElementById('ops-live-sessions');
    el.innerHTML = '';
    const all = rows || [];
    const filtered = all.filter((r) =>
        sessionRowMatchesFilter(
            `${r.source} ${r.session_id} ${r.preview || ''} ${r.last_activity || ''}`
        )
    );
    if (!all.length) {
        el.innerHTML = '<div class="ops-empty">No live in-memory sessions</div>';
        return;
    }
    if (!filtered.length) {
        el.innerHTML = '<div class="ops-empty">No live sessions match filter</div>';
        return;
    }
    filtered.forEach((r) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(r.source)} · ${r.session_id}</div><div class="ops-row-meta">${r.message_count} msgs · ${escapeHtml(r.last_activity)}${r.preview ? ` · ${escapeHtml(r.preview)}` : ''}</div></div>`;
        const openLive = async () => {
            try {
                const msgs = await invoke('read_live_session_messages', {
                    source: r.source,
                    sessionId: r.session_id,
                });
                showOpsSessionPreview(msgs, `Live ${r.source} · ${r.session_id}`);
                showOpsSessionStatus('Preview ready — click “Load into AI Chat” or double-click again to load.', true);
            } catch (err) {
                showOpsSessionPreview([], String(err));
                showOpsSessionStatus(String(err), false);
            }
        };
        btn.addEventListener('click', openLive);
        btn.addEventListener('dblclick', async () => {
            await openLive();
            loadOpsSessionIntoChat();
        });
        btn.title = 'Click to preview · double-click to load into AI Chat';
        el.appendChild(btn);
    });
}

function renderOpsSessionFiles(files) {
    const el = document.getElementById('ops-session-files');
    const preview = document.getElementById('ops-session-preview');
    const loadBtn = document.getElementById('ops-session-load-chat');
    el.innerHTML = '';
    preview.hidden = true;
    if (loadBtn) loadBtn.hidden = true;
    opsSessionLoadRows = null;
    const all = files || [];
    const filtered = all.filter((f) =>
        sessionRowMatchesFilter(
            `${f.slug || ''} ${f.name || ''} ${f.source_hint || ''} ${f.preview || ''}`
        )
    );
    if (!all.length) {
        el.innerHTML = '<div class="ops-empty">No session-memory-*.md files</div>';
        return;
    }
    if (!filtered.length) {
        el.innerHTML = '<div class="ops-empty">No session files match filter</div>';
        return;
    }
    filtered.forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.slug || f.name)}</div><div class="ops-row-meta">${escapeHtml(f.source_hint)} · ${fmtBytes(f.size_bytes)} · ${fmtAge(f.modified_ms)}${f.preview ? ` · ${escapeHtml(f.preview)}` : ''}</div></div>`;
        const openFile = async () => {
            try {
                const msgs = await invoke('read_session_file_messages', { path: f.path });
                if (msgs && msgs.length) {
                    showOpsSessionPreview(msgs, f.name);
                    showOpsSessionStatus('Preview ready — click “Load into AI Chat” or double-click to load.', true);
                } else {
                    const text = await invoke('read_session_file', { path: f.path });
                    preview.hidden = false;
                    preview.textContent = text.slice(0, 12000);
                    opsSessionLoadRows = null;
                    if (loadBtn) loadBtn.hidden = true;
                    showOpsSessionStatus('No parseable turns — raw file shown.', false);
                }
            } catch (err) {
                preview.hidden = false;
                preview.textContent = String(err);
                opsSessionLoadRows = null;
                if (loadBtn) loadBtn.hidden = true;
                showOpsSessionStatus(String(err), false);
            }
        };
        btn.addEventListener('click', openFile);
        btn.addEventListener('dblclick', async () => {
            await openFile();
            loadOpsSessionIntoChat();
        });
        btn.title = 'Click to preview · double-click to load into AI Chat';
        el.appendChild(btn);
    });
}

function renderOpsMemory(files) {
    const el = document.getElementById('ops-memory-list');
    const preview = document.getElementById('ops-memory-preview');
    el.innerHTML = '';
    preview.hidden = true;
    const all = files || [];
    const filtered = all.filter((f) =>
        memoryRowMatchesFilter(`${f.name || ''} ${f.kind || ''} ${f.path || ''}`)
    );
    if (!all.length) {
        el.innerHTML = '<div class="ops-empty">No memory/soul files</div>';
        return;
    }
    if (!filtered.length) {
        el.innerHTML = '<div class="ops-empty">No knowledge files match filter</div>';
        return;
    }
    filtered.forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.name)}</div><div class="ops-row-meta">${escapeHtml(f.kind)} · ${f.line_count} lines · ${fmtBytes(f.size_bytes)}</div></div>`;
        btn.addEventListener('click', async () => {
            try {
                const text = await invoke('read_memory_file', { path: f.path });
                preview.hidden = false;
                preview.textContent = text.slice(0, 12000);
            } catch (err) {
                preview.hidden = false;
                preview.textContent = String(err);
            }
        });
        el.appendChild(btn);
    });
}

function renderOpsRuns(insights) {
    const card = document.getElementById('ops-runs-insights');
    const el = document.getElementById('ops-runs-list');
    el.innerHTML = '';
    if (card) card.innerHTML = '';
    const gateway = insights?.discord_gateway || '';
    if (!insights || !insights.turns) {
        if (card && gateway) {
            card.innerHTML = `
                <div class="ops-insight-title">Insights</div>
                <div class="ops-row-meta">${escapeHtml(gateway)}</div>
                <div class="ops-row-meta">Digest: ${insights.digest_open_count ?? 0} open · ${insights.digest_stale_count ?? 0} stale${insights.digest_source ? ` · ${escapeHtml(insights.digest_source)}` : ''}</div>
                <div class="ops-empty" style="padding:8px 0 0">No runs in ~/.mac-stats/runs.jsonl yet</div>
            `;
        } else {
            el.innerHTML = '<div class="ops-empty">No runs in ~/.mac-stats/runs.jsonl yet</div>';
        }
        return;
    }
    const lanes = (insights.by_lane || []).map(([k, v]) => `${k}:${v}`).join(' · ');
    const tools = (insights.by_tool || [])
        .slice(0, 6)
        .map(([k, v]) => `${k}×${v}`)
        .join(', ');
    if (card) {
        const cand = (insights.candidates || [])
            .slice(0, 4)
            .map(
                (c) =>
                    `<div class="ops-insight-line"><span class="ops-badge">${escapeHtml(c.kind)}</span> ${c.wall_ms} ms — ${escapeHtml(c.reason)} · <em>${escapeHtml(c.question_preview)}</em></div>`
            )
            .join('');
        const slow = (insights.slowest || [])
            .slice(0, 3)
            .map(
                (s) =>
                    `<div class="ops-insight-line">${s.wall_ms} ms · ${escapeHtml(s.lane)} · ${escapeHtml(s.question_preview || '(empty)')}</div>`
            )
            .join('');
        card.innerHTML = `
            <div class="ops-insight-title">Insights</div>
            <div class="ops-row-meta">${insights.ok_count}/${insights.turns} ok · fail ${insights.fail_count || 0} · mean ${insights.mean_ms} ms · max ${insights.max_ms} ms</div>
            ${gateway ? `<div class="ops-row-meta">${escapeHtml(gateway)}</div>` : ''}
            <div class="ops-row-meta">Digest: ${insights.digest_open_count ?? 0} open · ${insights.digest_stale_count ?? 0} stale${insights.digest_source ? ` · ${escapeHtml(insights.digest_source)}` : ''}${insights.digest_generated_at ? ` · ${escapeHtml(String(insights.digest_generated_at).slice(0, 19))}` : ''}</div>
            ${(insights.digest_open_hints || []).length ? `<div class="ops-insight-sub">Digest open</div>${(insights.digest_open_hints || []).slice(0, 3).map((h) => `<div class="ops-insight-line">${escapeHtml(h)}</div>`).join('')}` : ''}
            <div class="ops-row-meta">Lanes: ${escapeHtml(lanes) || '—'}</div>
            <div class="ops-row-meta">Top tools: ${escapeHtml(tools) || '—'}</div>
            ${slow ? `<div class="ops-insight-sub">Slowest</div>${slow}` : ''}
            ${cand ? `<div class="ops-insight-sub">Candidates</div>${cand}` : ''}
        `;
    }
    (insights.recent || []).forEach((r) => {
        const toolsJoined = (r.tools || []).join(', ') || '—';
        if (
            !runsRowMatchesFilter(
                `${r.question_preview || ''} ${r.lane || ''} ${toolsJoined} ${r.ok ? 'ok' : 'fail'}`
            )
        ) {
            return;
        }
        const div = document.createElement('div');
        div.className = 'ops-row';
        div.innerHTML = `<div><div class="ops-row-title">${escapeHtml(r.question_preview || '(empty)')}</div><div class="ops-row-meta">${escapeHtml(r.lane)} · ${r.wall_ms} ms · ${escapeHtml(toolsJoined)}${r.ok ? '' : ' · FAIL'}</div></div>`;
        el.appendChild(div);
    });
    if (opsRunsFilterQ && !el.children.length) {
        el.innerHTML = '<div class="ops-empty">No runs match filter</div>';
    }
}

function escapeHtml(s) {
    return String(s ?? '')
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}



  function syncOpsIcon() {
    const icon = document.getElementById('icon-agent-ops');
    if (!icon) return;
    const open = !agentOpsCollapsed;
    icon.classList.toggle('status-good', open);
    if (open) icon.classList.remove('status-warning');
    icon.setAttribute('aria-pressed', open ? 'true' : 'false');
    icon.title = open ? 'Hide Agent Ops' : 'Agent Ops';
  }

  function syncOpsIconHealth(redmine) {
    const icon = document.getElementById('icon-agent-ops');
    if (!icon) return;
    const st = String(redmine?.status || '').toLowerCase();
    const warn = st === 'notconfigured' || st === 'unavailable' || st === 'degraded';
    icon.classList.toggle('status-warning', warn && agentOpsCollapsed);
    if (warn && agentOpsCollapsed) {
      icon.title = `Agent Ops — Redmine ${st === 'notconfigured' ? 'not configured' : st}`;
    }
  }

  function applyOpsCollapsed(collapsed) {
    agentOpsCollapsed = collapsed;
    const section = document.getElementById('agent-ops-section') || document.querySelector('.agent-ops-section');
    const content = document.getElementById('agent-ops-content');
    const btn = document.getElementById('agent-ops-collapse-btn');
    if (section) {
      section.classList.toggle('collapsed', collapsed);
    }
    if (content) {
      content.classList.toggle('collapsed', collapsed);
      // Themes use .section-content-collapsible.collapsed { display:none }; dashboard uses inline display.
      if (content.classList.contains('section-content-collapsible')) {
        content.style.display = collapsed ? 'none' : 'block';
      } else {
        content.style.display = collapsed ? 'none' : '';
      }
    }
    if (btn) btn.textContent = collapsed ? '+' : '−';
    syncOpsIcon();
    if (collapsed) {
      stopAgentOpsAutoRefresh();
    } else {
      refreshAgentOps();
      startAgentOpsAutoRefresh();
      // Defer scroll until layout applies display:block
      requestAnimationFrame(() => {
        section?.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
      });
    }
  }

  function toggleAgentOpsSection() {
    applyOpsCollapsed(!agentOpsCollapsed);
  }

  function wireCollapse() {
    const header = document.getElementById('agent-ops-header');
    const btn = document.getElementById('agent-ops-collapse-btn');
    const icon = document.getElementById('icon-agent-ops');

    if (icon && !icon.dataset.opsWired) {
      icon.dataset.opsWired = '1';
      const onIcon = (e) => {
        e.preventDefault();
        e.stopPropagation();
        toggleAgentOpsSection();
      };
      icon.addEventListener('click', onIcon);
      // SVG child clicks still hit the button; keep hit target large enough
      icon.style.pointerEvents = 'auto';
    }

    const closeBtn = document.getElementById('agent-ops-close-btn');
    if (closeBtn) {
      closeBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        applyOpsCollapsed(true);
      });
    }

    if (header) {
      header.addEventListener('click', (e) => {
        if (e.target.id === 'agent-ops-collapse-btn' || e.target.closest('.collapse-btn')) return;
        if (e.target.closest('.ops-overview-link') || e.target.closest('.agent-ops-tab')) return;
        // With an icon present, header click collapses (hide) only — open via icon
        if (icon && !agentOpsCollapsed) {
          applyOpsCollapsed(true);
          return;
        }
        if (!icon) toggleAgentOpsSection();
      });
    }
    if (btn) {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        toggleAgentOpsSection();
      });
    }
    // Themes: start hidden (icon opens). Dashboard without icon: start expanded if not collapsed.
    const content = document.getElementById('agent-ops-content');
    const section = document.querySelector('.agent-ops-section');
    let startsCollapsed = true;
    if (icon) {
      startsCollapsed = true;
    } else {
      startsCollapsed =
        content?.classList.contains('collapsed') ||
        content?.style.display === 'none' ||
        (section?.classList.contains('collapsed') ?? false);
    }
    applyOpsCollapsed(!!startsCollapsed);
  }

  function initAgentOps() {
    if (!document.getElementById('ops-health-row')) return;
    wireCollapse();
    setupAgentOps();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initAgentOps);
  } else {
    initAgentOps();
  }

  window.addEventListener('beforeunload', () => stopAgentOpsAutoRefresh());

  window.AgentOps = {
    refresh: refreshAgentOps,
    selectTab: selectOpsTab,
    toggle: toggleAgentOpsSection,
  };
})();
