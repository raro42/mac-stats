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

  /** Filter input id → clearOpsFilter kind (row Clear beside N/M chip). */
  const OPS_FILTER_KIND_BY_INPUT = {
    'ops-session-filter': 'sessions',
    'ops-memory-filter': 'memory',
    'ops-runs-filter': 'runs',
    'ops-agents-filter': 'agents',
    'ops-schedules-filter': 'schedules',
  };

  /** Live match chip beside the filter input (stays visible while scrolling the list). */
  function ensureOpsFilterMatchChip(input) {
    if (!input || !input.parentElement) return null;
    let chip = input.parentElement.querySelector('.ops-filter-match');
    if (chip) return chip;
    chip = document.createElement('span');
    chip.className = 'ops-filter-match';
    chip.hidden = true;
    chip.setAttribute('aria-live', 'polite');
    input.parentElement.appendChild(chip);
    return chip;
  }

  /** Compact Clear control when a filter query is active (Esc parity; works with matches too). */
  function ensureOpsFilterClearBtn(input) {
    if (!input || !input.parentElement) return null;
    let btn = input.parentElement.querySelector('.ops-filter-clear');
    if (btn) return btn;
    const kind = OPS_FILTER_KIND_BY_INPUT[input.id];
    if (!kind) return null;
    btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'ops-filter-clear';
    btn.hidden = true;
    btn.setAttribute('aria-label', 'Clear filter');
    btn.title = 'Clear filter (Esc)';
    btn.textContent = 'Clear';
    btn.dataset.opsClearFilter = kind;
    input.parentElement.appendChild(btn);
    return btn;
  }

  function paintOpsFilterMatch(filterInputId, total, shown, q) {
    const input = document.getElementById(filterInputId);
    if (!input) return;
    const chip = ensureOpsFilterMatchChip(input);
    const clearBtn = ensureOpsFilterClearBtn(input);
    if (!chip) return;
    const query = String(q || '').trim();
    if (!query) {
      chip.hidden = true;
      chip.textContent = '';
      chip.removeAttribute('title');
      chip.classList.remove('is-zero', 'is-partial', 'is-all');
      if (clearBtn) clearBtn.hidden = true;
      refreshOpsFilterRowForInput(input);
      return;
    }
    const t = Math.max(0, Number(total) || 0);
    const s = Math.max(0, Math.min(t, Number(shown) || 0));
    chip.hidden = false;
    chip.textContent = `${s}/${t}`;
    chip.title = s === 1 ? `1 of ${t} match` : `${s} of ${t} match`;
    chip.classList.toggle('is-zero', s === 0);
    chip.classList.toggle('is-partial', s > 0 && s < t);
    chip.classList.toggle('is-all', s > 0 && s === t);
    if (clearBtn) clearBtn.hidden = false;
    refreshOpsFilterRowForInput(input);
  }

  /** Focusable filter-row items in DOM order (input · N/M chip · Clear when visible). */
  function getOpsFilterRowItems(row) {
    if (!row) return [];
    const items = [];
    const input = row.querySelector('.ops-filter-input');
    const chip = row.querySelector('.ops-filter-match');
    const clear = row.querySelector('.ops-filter-clear');
    if (input && !input.hidden) items.push(input);
    if (chip && !chip.hidden) {
      if (!chip.getAttribute('role')) chip.setAttribute('role', 'status');
      if (!chip.title) chip.title = 'Match count';
      items.push(chip);
    }
    if (clear && !clear.hidden) items.push(clear);
    return items.filter((el) => {
      if (!el || el.hidden) return false;
      return el.getClientRects().length > 0 || row.contains(el);
    });
  }

  function opsFilterInputAtMoveBoundary(input, direction) {
    if (!input || input.tagName !== 'INPUT') return true;
    if (direction > 0) {
      const len = (input.value || '').length;
      return input.selectionStart === len && input.selectionEnd === len;
    }
    return input.selectionStart === 0 && input.selectionEnd === 0;
  }

  function refreshOpsFilterRowRovingTabindex(row, preferred) {
    const items = getOpsFilterRowItems(row);
    if (!items.length) return;
    const focused = items.find((el) => el === document.activeElement);
    const current =
      (preferred && items.includes(preferred) && preferred) ||
      focused ||
      items.find((el) => el.tabIndex === 0) ||
      items[0];
    for (const el of items) {
      el.tabIndex = el === current ? 0 : -1;
    }
  }

  function refreshOpsFilterRowForInput(input) {
    const row = input && input.closest && input.closest('.ops-filter-row');
    if (!row) return;
    refreshOpsFilterRowRovingTabindex(row);
    const hint = row.querySelector('.ops-filter-row-kb-hint');
    if (hint) {
      const items = getOpsFilterRowItems(row);
      hint.hidden = items.length < 2;
    }
  }

  function ensureOpsFilterRowKbHint(row) {
    if (!row) return;
    let hint = row.querySelector('.ops-filter-row-kb-hint');
    if (!hint) {
      hint = document.createElement('div');
      hint.className = 'ops-filter-row-kb-hint';
      hint.setAttribute('aria-hidden', 'true');
      row.appendChild(hint);
    }
    const items = getOpsFilterRowItems(row);
    hint.hidden = items.length < 2;
    hint.textContent =
      '← → / h l · Home/End move · Enter / Space clears when on Clear';
  }

  /**
   * Filter-row toolbar keyboard — focus search input · N/M chip · Clear,
   * then ←→ / h l / Home/End (preview-row / refresh-row parity). Input keeps
   * normal typing; arrows move only at text start/end.
   */
  function wireOpsFilterRowToolbarKeyboard(row) {
    if (!row) return;
    ensureOpsFilterRowKbHint(row);
    refreshOpsFilterRowRovingTabindex(row);
    if (row.dataset.opsFilterRowKbWired === '1') return;
    row.dataset.opsFilterRowKbWired = '1';
    if (!row.getAttribute('role')) {
      row.setAttribute('role', 'toolbar');
    }
    if (!row.getAttribute('aria-label')) {
      row.setAttribute('aria-label', 'List filter');
    }
    row.addEventListener('focusin', (e) => {
      const items = getOpsFilterRowItems(row);
      if (items.includes(e.target)) {
        refreshOpsFilterRowRovingTabindex(row, e.target);
        ensureOpsFilterRowKbHint(row);
      }
    });
    row.addEventListener('keydown', (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const items = getOpsFilterRowItems(row);
      if (!items.length) return;
      const idx = items.indexOf(document.activeElement);
      if (idx < 0) return;
      const active = items[idx];
      if ((e.key === 'Enter' || e.key === ' ') && active?.classList?.contains('ops-filter-clear')) {
        return;
      }
      let next = -1;
      const forward =
        e.key === 'ArrowRight' ||
        e.key === 'l' ||
        e.key === 'ArrowDown' ||
        e.key === 'j';
      const back =
        e.key === 'ArrowLeft' ||
        e.key === 'h' ||
        e.key === 'ArrowUp' ||
        e.key === 'k';
      if (forward) {
        if (active?.classList?.contains('ops-filter-input') && !opsFilterInputAtMoveBoundary(active, 1)) {
          return;
        }
        next = Math.min(idx + 1, items.length - 1);
      } else if (back) {
        if (active?.classList?.contains('ops-filter-input') && !opsFilterInputAtMoveBoundary(active, -1)) {
          return;
        }
        next = Math.max(idx - 1, 0);
      } else if (e.key === 'Home') {
        next = 0;
      } else if (e.key === 'End') {
        next = items.length - 1;
      } else {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (next === idx) return;
      refreshOpsFilterRowRovingTabindex(row, items[next]);
      items[next].focus();
      if (items[next]?.classList?.contains('ops-filter-input') && typeof items[next].select === 'function') {
        const len = (items[next].value || '').length;
        items[next].setSelectionRange(len, len);
      }
    });
  }

  function ensureOpsFilterRowsToolbarKeyboard() {
    document.querySelectorAll('.ops-filter-row').forEach((row) => {
      wireOpsFilterRowToolbarKeyboard(row);
    });
  }

  /** @deprecated list captions replaced by filter-row chips — keep name for call-site clarity */
  function prependOpsFilterCaption(_el, total, shown, q, filterInputId) {
    if (filterInputId) paintOpsFilterMatch(filterInputId, total, shown, q);
  }

  /** Empty state when a filter is active but no rows match — includes Clear filter action. */
  function opsFilterMissHtml(message, filterKind) {
    const kind = String(filterKind || '').replace(/[^a-z]/gi, '');
    return (
      `<div class="ops-empty ops-empty-filter-miss">` +
      `<div class="ops-empty-filter-msg">${escapeHtml(message)}</div>` +
      `<button type="button" class="ops-clear-filter" data-ops-clear-filter="${kind}">Clear filter</button>` +
      `</div>`
    );
  }

  /** Overview card empty state with an Open-tab CTA (same affordance as Clear filter). */
  function opsOverviewEmptyHtml(message, tab, ctaLabel) {
    const safeTab = String(tab || '').replace(/[^a-z]/gi, '');
    const label = ctaLabel || (safeTab ? `Open ${safeTab}` : 'Open tab');
    return (
      `<div class="ops-empty ops-empty-filter-miss ops-empty-overview-cta">` +
      `<div class="ops-empty-filter-msg">${escapeHtml(message)}</div>` +
      `<button type="button" class="ops-clear-filter" data-ops-goto-tab="${safeTab}">${escapeHtml(label)}</button>` +
      `</div>`
    );
  }

  /** True-empty state on a list tab (already on the surface — title + short hint). */
  function opsTabEmptyHtml(title, hint, cta) {
    const hintHtml = hint
      ? `<div class="ops-empty-tab-hint">${escapeHtml(hint)}</div>`
      : '';
    let ctaHtml = '';
    if (cta && cta.action === 'ai-chat') {
      const label = cta.label || 'Open AI Chat';
      ctaHtml =
        `<button type="button" class="ops-clear-filter" data-ops-open-ai-chat="1">${escapeHtml(label)}</button>`;
    }
    const extra = ctaHtml ? ' ops-empty-filter-miss' : '';
    return (
      `<div class="ops-empty ops-empty-tab${extra}">` +
      `<div class="ops-empty-filter-msg">${escapeHtml(title)}</div>` +
      hintHtml +
      ctaHtml +
      `</div>`
    );
  }

  /** Empty Live / session / Runs CTA — expand AI Chat (Monitors empty Add parity). */
  function openOpsAiChat() {
    const input = document.getElementById('chat-input');
    const section = document.querySelector('.ollama-section');
    if (!input || !section || section.style.display === 'none') {
      return false;
    }
    applyOpsCollapsed(true);
    if (typeof window.setSectionCollapsed === 'function') {
      window.setSectionCollapsed('ollama_collapsed', false);
    } else {
      try {
        localStorage.setItem('ollama_collapsed', 'false');
      } catch (_) {
        /* ignore */
      }
    }
    const content = document.getElementById('ollama-content');
    const collapsed =
      section.classList.contains('collapsed') ||
      content?.classList.contains('collapsed') ||
      content?.style.display === 'none';
    if (collapsed) {
      document.getElementById('ollama-header')?.click();
    }
    if (content) {
      content.classList.remove('collapsed');
      if (content.style.display === 'none') content.style.display = '';
    }
    section.classList.remove('collapsed');
    const collapseBtn = document.getElementById('ollama-collapse-btn');
    if (collapseBtn) collapseBtn.textContent = '−';
    try {
      section.scrollIntoView?.({ behavior: 'smooth', block: 'start' });
    } catch (_) {
      section.scrollIntoView?.(true);
    }
    setTimeout(() => {
      input.focus();
    }, 80);
    return true;
  }

  function clearOpsFilter(kind) {
    const map = {
      sessions: {
        id: 'ops-session-filter',
        clear() {
          opsSessionFilterQ = '';
          setOpsSessionKindFilter('all');
        },
      },
      memory: {
        id: 'ops-memory-filter',
        clear() {
          opsMemoryFilterQ = '';
          setOpsMemoryKindFilter('all');
        },
      },
      runs: {
        id: 'ops-runs-filter',
        clear() {
          opsRunsFilterQ = '';
          setOpsRunsLaneFilter('all');
        },
      },
      agents: {
        id: 'ops-agents-filter',
        clear() {
          opsAgentsFilterQ = '';
          setOpsAgentsEnabledFilter('all');
        },
      },
      schedules: {
        id: 'ops-schedules-filter',
        clear() {
          opsSchedulesFilterQ = '';
          setOpsSchedulesKindFilter('all');
        },
      },
    };
    const entry = map[kind];
    if (!entry) return false;
    ensureOpsSessionFilter();
    ensureOpsMemoryFilter();
    ensureOpsRunsFilter();
    ensureOpsAgentsFilter();
    ensureOpsSchedulesFilter();
    const input = document.getElementById(entry.id);
    if (input) {
      input.value = '';
      input.focus();
    }
    entry.clear();
    if (input) {
      input.classList.add('ops-filter-just-cleared');
      setTimeout(() => input.classList.remove('ops-filter-just-cleared'), 900);
    }
    return true;
  }

  const OPS_REFRESH_INTERVAL = 30000;
  const OPS_GLANCE_POLL_INTERVAL = 60000;
  let agentOpsInterval = null;
  let agentOpsGlanceInterval = null;
  let agentOpsCollapsed = true;
  let opsAgentCache = null;
  let opsAgentFileTab = 'soul';
  let opsAgentDirty = { soul: false, skill: false, mood: false };
  let opsAgentLoadText = null;
  let opsAgentSaveStatusTimer = null;
  let opsAgentSaveBusy = false;
  let opsRefreshInFlight = false;
  let opsRefreshFlashTimer = null;
  let opsLastRefreshMs = 0;
  let opsUpdatedAgoTimer = null;
  let opsDigestRefreshInFlight = false;
  let opsDigestRefreshFlashTimer = null;
  let opsSessionLoadRows = null;
  let opsSessionFilterQ = '';
  /** Sessions panel kind chip: `all` | `live` | `files` (Monitors All/Up/Down parity). */
  let opsSessionKindFilter = 'all';
  let opsLiveCache = [];
  let opsSessionFilesCache = [];
  let opsMemoryFilterQ = '';
  /** Knowledge panel kind chip: `all` | `discord` | `core` (Sessions All/Live/Files parity). */
  let opsMemoryKindFilter = 'all';
  let opsMemoryCache = [];
  let opsMemoryLoadText = null;
  let opsRunsFilterQ = '';
  /** Runs panel lane chip: `all` | `instant` | `direct` (Agents All/On/Off parity). */
  let opsRunsLaneFilter = 'all';
  let opsRunsInsightsCache = null;
  /** Summary payload for Runs Insights lines (toolbar keyboard preview). */
  const opsInsightLineSummary = new WeakMap();
  let opsRunLoadQuestion = null;
  let opsScheduleLoadText = null;
  let opsAgentsFilterQ = '';
  /** Agents panel enabled chip: `all` | `on` | `off` (Sessions All/Live/Files parity). */
  let opsAgentsEnabledFilter = 'all';
  let opsAgentsCache = [];
  let opsSchedulesFilterQ = '';
  /** Schedules panel kind chip: `all` | `jobs` | `deliveries` (Sessions All/Live/Files parity). */
  let opsSchedulesKindFilter = 'all';
  let opsSchedulesCache = [];
  let opsDeliveriesCache = [];
  let opsActiveTab = 'agents';

// --- Agent Ops (Command Center: overview + detail tabs) ---

/** Health card → detail tab (same map as click navigation). */
const OPS_HEALTH_TAB_BY_KEY = {
    version: 'agents',
    discord: 'runs',
    redmine: 'agents',
    schedule: 'schedules',
    delivery: 'schedules',
    digest: 'runs',
};

/** Overview cards light up when their linked detail tab is active. */
function syncOpsOverviewCardActive(tab) {
    const active = tab || opsActiveTab || 'agents';
    document.querySelectorAll('.ops-overview-card').forEach((card) => {
        const link = card.querySelector('.ops-overview-link[data-goto-tab]');
        const goto = link?.dataset?.gotoTab || '';
        card.classList.toggle('is-active', !!goto && goto === active);
    });
}

/** Health cards light up when their linked detail tab is active (parity with overview). */
function syncOpsHealthCardActive(tab) {
    const active = tab || opsActiveTab || 'agents';
    document.querySelectorAll('.ops-health-card[data-health]').forEach((card) => {
        const goto = card.dataset.gotoTab || OPS_HEALTH_TAB_BY_KEY[card.dataset.health] || '';
        card.classList.toggle('is-active', !!goto && goto === active);
    });
}

function selectOpsTab(tab) {
    opsActiveTab = tab || 'agents';
    document.querySelectorAll('.agent-ops-tab').forEach((b) => {
        if (!b.dataset.opsTab) return;
        b.classList.toggle('active', b.dataset.opsTab === tab);
    });
    refreshOpsTabBarRovingTabindex();
    document.querySelectorAll('.agent-ops-panel').forEach((p) => {
        p.classList.toggle('active', p.id === `ops-panel-${tab}`);
    });
    syncOpsOverviewCardActive(opsActiveTab);
    syncOpsHealthCardActive(opsActiveTab);
    const panel = document.getElementById(`ops-panel-${tab}`);
    const tabs = document.querySelector('.agent-ops-tabs');
    (panel || tabs)?.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
    if (typeof window.setCpuUiSectionValue === 'function') {
        window.setCpuUiSectionValue('agent_ops_tab', tab);
    } else {
        try {
            localStorage.setItem('agent_ops_tab', tab);
        } catch (_) {}
    }
}

function focusActiveOpsFilter() {
    const idByTab = {
        sessions: 'ops-session-filter',
        memory: 'ops-memory-filter',
        runs: 'ops-runs-filter',
        agents: 'ops-agents-filter',
        schedules: 'ops-schedules-filter',
    };
    const id = idByTab[opsActiveTab];
    if (!id) return false;
    ensureOpsSessionFilter();
    ensureOpsMemoryFilter();
    ensureOpsRunsFilter();
    ensureOpsAgentsFilter();
    ensureOpsSchedulesFilter();
    const input = document.getElementById(id);
    if (!input) return false;
    input.focus();
    input.select?.();
    return true;
}

/** Scroll health + overview into view (1–5 jump into detail panels). */
function showOpsOverview() {
    const health = document.getElementById('ops-health-row');
    const grid = document.getElementById('ops-overview-grid');
    const target = health || grid;
    try {
        target?.scrollIntoView?.({ behavior: 'smooth', block: 'start' });
    } catch (_) {
        target?.scrollIntoView?.(true);
    }
    const flash = (el) => {
        if (!el) return;
        el.classList.remove('ops-overview-jump-flash');
        void el.offsetWidth;
        el.classList.add('ops-overview-jump-flash');
    };
    flash(document.getElementById('ops-overview-jump'));
    flash(grid);
    return !!target;
}

/** Visible 0 Overview control at the start of the tab strip. */
function ensureOpsOverviewJump() {
    const tabs = document.querySelector('.agent-ops-tabs');
    if (!tabs) return null;
    let btn = document.getElementById('ops-overview-jump');
    if (!btn) {
        btn = document.createElement('button');
        btn.type = 'button';
        btn.id = 'ops-overview-jump';
        btn.className = 'agent-ops-tab ops-overview-jump';
        btn.title = 'Overview · press 0';
        btn.setAttribute('aria-keyshortcuts', '0');
        btn.setAttribute('aria-label', 'Jump to overview');
        const digit = document.createElement('span');
        digit.className = 'ops-tab-digit';
        digit.setAttribute('aria-hidden', 'true');
        digit.textContent = '0';
        btn.appendChild(digit);
        btn.appendChild(document.createTextNode('Overview'));
        tabs.insertBefore(btn, tabs.firstChild);
        btn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            if (agentOpsCollapsed) applyOpsCollapsed(false);
            showOpsOverview();
        });
    }
    return btn;
}

/** Show digit keys on tab buttons (Hermes-style 1–5 already bound). */
function ensureOpsTabDigits() {
    const digits = {
        agents: '1',
        sessions: '2',
        schedules: '3',
        memory: '4',
        runs: '5',
    };
    document.querySelectorAll('.agent-ops-tab').forEach((btn) => {
        const tab = btn.dataset.opsTab || '';
        const d = digits[tab];
        if (!d || btn.querySelector('.ops-tab-digit')) return;
        const label = (btn.textContent || tab || '').trim() || tab;
        const span = document.createElement('span');
        span.className = 'ops-tab-digit';
        span.setAttribute('aria-hidden', 'true');
        span.textContent = d;
        btn.insertBefore(span, btn.firstChild);
        btn.dataset.opsTabLabel = label;
        btn.setAttribute('title', `${label} · press ${d}`);
        if (!btn.getAttribute('aria-keyshortcuts')) {
            btn.setAttribute('aria-keyshortcuts', d);
        }
    });
}

/** Focusable tab-bar buttons in DOM order (0 Overview · Agents · Sessions · …). */
function getOpsTabBarButtons() {
    const tabs = document.querySelector('.agent-ops-tabs');
    if (!tabs) return [];
    return Array.from(tabs.querySelectorAll(':scope > .agent-ops-tab')).filter((el) => {
        if (!el || el.hidden) return false;
        return el.getClientRects().length > 0 || el.offsetParent !== null || tabs.contains(el);
    });
}

function refreshOpsTabBarRovingTabindex(preferred) {
    const buttons = getOpsTabBarButtons();
    if (!buttons.length) return;
    const activeTab = opsActiveTab || 'agents';
    const activeBtn = buttons.find((el) => el.dataset.opsTab === activeTab);
    const focused = buttons.find((el) => el === document.activeElement);
    const current =
        (preferred && buttons.includes(preferred) && preferred) ||
        focused ||
        buttons.find((el) => el.tabIndex === 0) ||
        activeBtn ||
        buttons[0];
    for (const el of buttons) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function ensureOpsTabBarKbHint() {
    const tabs = document.querySelector('.agent-ops-tabs');
    if (!tabs) return;
    let hint = document.getElementById('ops-tab-bar-kb-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-tab-bar-kb-hint';
        hint.className = 'ops-tab-bar-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        tabs.insertAdjacentElement('afterend', hint);
    }
    hint.textContent =
        '← → / h l · Home/End move · Enter / Space opens tab or overview';
}

function getOpsFileTabButtons() {
    const tabs = document.querySelector('.ops-file-tabs');
    if (!tabs) return [];
    return Array.from(tabs.querySelectorAll(':scope > .ops-file-tab')).filter((el) => {
        if (!el || el.hidden) return false;
        const detail = document.getElementById('ops-agent-detail');
        if (detail?.hidden) return false;
        return el.getClientRects().length > 0 || el.offsetParent !== null || tabs.contains(el);
    });
}

function refreshOpsFileTabRovingTabindex(preferred) {
    const buttons = getOpsFileTabButtons();
    if (!buttons.length) return;
    const activeBtn = buttons.find((el) => el.classList.contains('active'));
    const focused = buttons.find((el) => el === document.activeElement);
    const current =
        (preferred && buttons.includes(preferred) && preferred) ||
        focused ||
        buttons.find((el) => el.tabIndex === 0) ||
        activeBtn ||
        buttons[0];
    for (const el of buttons) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function ensureOpsFileTabKbHint() {
    const tabs = document.querySelector('.ops-file-tabs');
    if (!tabs) return;
    let hint = document.getElementById('ops-file-tab-kb-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-file-tab-kb-hint';
        hint.className = 'ops-file-tab-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        tabs.insertAdjacentElement('afterend', hint);
    }
    hint.textContent =
        '← → / h l · Home/End move · Enter / Space switches soul / skill / mood';
}

/**
 * File-tab toolbar keyboard — focus Soul · Skill · Mood, then ←→ / h l / Home/End
 * (tab-bar / refresh-row parity). Enter/Space keeps existing tab activate.
 */
function ensureOpsFileTabToolbarKeyboard() {
    const tabs = document.querySelector('.ops-file-tabs');
    if (!tabs) return;
    ensureOpsFileTabKbHint();
    refreshOpsFileTabRovingTabindex();
    if (tabs.dataset.opsFileTabKbWired === '1') return;
    tabs.dataset.opsFileTabKbWired = '1';
    if (!tabs.getAttribute('role')) {
        tabs.setAttribute('role', 'toolbar');
    }
    if (!tabs.getAttribute('aria-label')) {
        tabs.setAttribute('aria-label', 'Agent soul, skill, and mood');
    }
    tabs.addEventListener('focusin', (e) => {
        const buttons = getOpsFileTabButtons();
        if (buttons.includes(e.target)) refreshOpsFileTabRovingTabindex(e.target);
    });
    tabs.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const buttons = getOpsFileTabButtons();
        if (!buttons.length) return;
        const idx = buttons.indexOf(document.activeElement);
        if (idx < 0) return;
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, buttons.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = buttons.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsFileTabRovingTabindex(buttons[next]);
        buttons[next].focus();
    });
}

/** Focusable agent edit-action items in DOM order (Save · Load into AI Chat · Back). */
function getOpsAgentEditActionItems() {
    const row = document.getElementById('ops-agent-edit-actions');
    if (!row) return [];
    const detail = document.getElementById('ops-agent-detail');
    if (detail?.hidden) return [];
    const items = [];
    const save = document.getElementById('ops-agent-save');
    const load = document.getElementById('ops-agent-load-chat');
    const back = document.getElementById('ops-agent-back');
    if (save && !save.hidden) items.push(save);
    if (load && !load.hidden) items.push(load);
    if (back && !back.hidden) items.push(back);
    return items.filter((el) => {
        if (!el || el.hidden) return false;
        return el.getClientRects().length > 0 || el.offsetParent !== null || row.contains(el);
    });
}

function refreshOpsAgentEditActionsRovingTabindex(preferred) {
    const items = getOpsAgentEditActionItems();
    if (!items.length) return;
    const focused = items.find((el) => el === document.activeElement);
    const current =
        (preferred && items.includes(preferred) && preferred) ||
        focused ||
        items.find((el) => el.tabIndex === 0) ||
        items[0];
    for (const el of items) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function ensureOpsAgentEditActionsKbHint() {
    const row = document.getElementById('ops-agent-edit-actions');
    if (!row) return;
    let hint = document.getElementById('ops-agent-edit-actions-kb-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-agent-edit-actions-kb-hint';
        hint.className = 'ops-agent-edit-actions-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        row.insertAdjacentElement('afterend', hint);
    }
    hint.textContent =
        '← → / h l · Home/End move · Enter / Space saves, loads chat, or goes back';
}

/**
 * Agent edit-actions toolbar keyboard — focus Save · Load into AI Chat · Back,
 * then ←→ / h l / Home/End (file-tab / refresh-row parity). Enter/Space keeps
 * existing button activate.
 */
function ensureOpsAgentEditActionsToolbarKeyboard() {
    const row = document.getElementById('ops-agent-edit-actions');
    if (!row) return;
    ensureOpsAgentEditActionsKbHint();
    refreshOpsAgentEditActionsRovingTabindex();
    if (row.dataset.opsAgentEditActionsKbWired === '1') return;
    row.dataset.opsAgentEditActionsKbWired = '1';
    if (!row.getAttribute('role')) {
        row.setAttribute('role', 'toolbar');
    }
    if (!row.getAttribute('aria-label')) {
        row.setAttribute('aria-label', 'Agent file save and navigation');
    }
    row.addEventListener('focusin', (e) => {
        const items = getOpsAgentEditActionItems();
        if (items.includes(e.target)) refreshOpsAgentEditActionsRovingTabindex(e.target);
    });
    row.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const items = getOpsAgentEditActionItems();
        if (!items.length) return;
        const idx = items.indexOf(document.activeElement);
        if (idx < 0) return;
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, items.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = items.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsAgentEditActionsRovingTabindex(items[next]);
        items[next].focus();
    });
}

/** Preview-row specs: copy chip · Load into AI Chat (Sessions / Runs / Schedules / Knowledge). */
const OPS_PREVIEW_ROW_SPECS = [
    {
        key: 'sessions',
        panelId: 'ops-panel-sessions',
        copyId: 'ops-session-copy-chip',
        loadId: 'ops-session-load-chat',
        hintId: 'ops-session-preview-kb-hint',
        anchorId: 'ops-session-preview',
        ariaLabel: 'Session preview actions',
    },
    {
        key: 'runs',
        panelId: 'ops-panel-runs',
        copyId: 'ops-runs-copy-chip',
        loadId: 'ops-runs-load-chat',
        hintId: 'ops-runs-preview-kb-hint',
        anchorId: 'ops-runs-preview',
        ariaLabel: 'Run preview actions',
    },
    {
        key: 'schedules',
        panelId: 'ops-panel-schedules',
        copyId: 'ops-schedule-copy-chip',
        loadId: 'ops-schedules-load-chat',
        hintId: 'ops-schedules-preview-kb-hint',
        anchorId: 'ops-schedule-preview',
        ariaLabel: 'Schedule preview actions',
    },
    {
        key: 'memory',
        panelId: 'ops-panel-memory',
        copyId: 'ops-memory-copy-chip',
        loadId: 'ops-memory-load-chat',
        hintId: 'ops-memory-preview-kb-hint',
        anchorId: 'ops-memory-preview',
        ariaLabel: 'Knowledge preview actions',
    },
];

function getOpsPreviewRowItems(spec) {
    if (!spec) return [];
    const panel = document.getElementById(spec.panelId);
    if (!panel || panel.hidden) return [];
    const items = [];
    const copy = document.getElementById(spec.copyId);
    const load = document.getElementById(spec.loadId);
    if (copy && !copy.hidden) items.push(copy);
    if (load && !load.hidden) items.push(load);
    return items.filter((el) => {
        if (!el || el.hidden) return false;
        return el.getClientRects().length > 0 || el.offsetParent !== null;
    });
}

function refreshOpsPreviewRowRovingTabindex(spec, preferred) {
    const items = getOpsPreviewRowItems(spec);
    if (!items.length) return;
    const focused = items.find((el) => el === document.activeElement);
    const current =
        (preferred && items.includes(preferred) && preferred) ||
        focused ||
        items.find((el) => el.tabIndex === 0) ||
        items[0];
    for (const el of items) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function refreshAllOpsPreviewRowRovingTabindex(preferredByKey) {
    for (const spec of OPS_PREVIEW_ROW_SPECS) {
        const preferred =
            preferredByKey && preferredByKey[spec.key]
                ? document.getElementById(preferredByKey[spec.key])
                : null;
        refreshOpsPreviewRowRovingTabindex(spec, preferred);
    }
}

function ensureOpsPreviewRowKbHint(spec) {
    if (!spec) return;
    const anchor = document.getElementById(spec.anchorId);
    if (!anchor || !anchor.parentNode) return;
    let hint = document.getElementById(spec.hintId);
    if (!hint) {
        hint = document.createElement('div');
        hint.id = spec.hintId;
        hint.className = 'ops-preview-row-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        const load = document.getElementById(spec.loadId);
        if (load && load.parentNode === anchor.parentNode) {
            load.parentNode.insertBefore(hint, load.nextSibling);
        } else {
            anchor.parentNode.insertBefore(hint, anchor.nextSibling);
        }
    }
    const items = getOpsPreviewRowItems(spec);
    hint.hidden = items.length < 2;
    hint.textContent =
        '← → / h l · Home/End move · Enter / Space copies or loads chat';
}

function wireOpsPreviewRowToolbarKeyboard(spec) {
    if (!spec) return;
    const anchor = document.getElementById(spec.anchorId);
    if (!anchor || !anchor.parentNode) return;
    const host = anchor.parentNode;
    ensureOpsPreviewRowKbHint(spec);
    refreshOpsPreviewRowRovingTabindex(spec);
    const wireKey = `opsPreviewRowKbWired_${spec.key}`;
    if (host.dataset[wireKey] === '1') return;
    host.dataset[wireKey] = '1';
    if (!host.dataset.opsPreviewRowToolbar) {
        host.dataset.opsPreviewRowToolbar = spec.key;
    }
    host.addEventListener('focusin', (e) => {
        const items = getOpsPreviewRowItems(spec);
        if (items.includes(e.target)) {
            refreshOpsPreviewRowRovingTabindex(spec, e.target);
            ensureOpsPreviewRowKbHint(spec);
        }
    });
    host.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const items = getOpsPreviewRowItems(spec);
        if (!items.length) return;
        const idx = items.indexOf(document.activeElement);
        if (idx < 0) return;
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, items.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = items.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsPreviewRowRovingTabindex(spec, items[next]);
        items[next].focus();
    });
}

/** Preview-row toolbar keyboard — copy chip · Load into AI Chat (edit-actions parity). */
function ensureOpsPreviewRowsToolbarKeyboard() {
    for (const spec of OPS_PREVIEW_ROW_SPECS) {
        wireOpsPreviewRowToolbarKeyboard(spec);
    }
}

/**
 * Tab-bar toolbar keyboard — focus 0 Overview · Agents · Sessions · Schedules ·
 * Knowledge · Runs, then ←→ / h l / Home/End (overview-card / health-strip parity).
 * Enter/Space keep existing tab activate.
 */
function ensureOpsTabBarToolbarKeyboard() {
    const tabs = document.querySelector('.agent-ops-tabs');
    if (!tabs) return;
    ensureOpsTabBarKbHint();
    refreshOpsTabBarRovingTabindex();
    if (tabs.dataset.opsTabBarKbWired === '1') return;
    tabs.dataset.opsTabBarKbWired = '1';
    if (!tabs.getAttribute('role')) {
        tabs.setAttribute('role', 'toolbar');
    }
    if (!tabs.getAttribute('aria-label')) {
        tabs.setAttribute('aria-label', 'Agent Ops tabs');
    }
    tabs.addEventListener('focusin', (e) => {
        const buttons = getOpsTabBarButtons();
        if (buttons.includes(e.target)) refreshOpsTabBarRovingTabindex(e.target);
    });
    tabs.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const buttons = getOpsTabBarButtons();
        if (!buttons.length) return;
        const idx = buttons.indexOf(document.activeElement);
        if (idx < 0) return;
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, buttons.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = buttons.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsTabBarRovingTabindex(buttons[next]);
        buttons[next].focus();
    });
}

/** Inventory counts on tabs (agents / sessions / schedules / knowledge / runs). */
function ensureOpsTabCountEl(btn) {
    let el = btn.querySelector('.ops-tab-count');
    if (el) return el;
    el = document.createElement('span');
    el.className = 'ops-tab-count';
    el.setAttribute('aria-hidden', 'true');
    el.hidden = true;
    btn.appendChild(el);
    return el;
}

function paintOpsTabCounts(counts) {
    ensureOpsTabDigits();
    const digits = {
        agents: '1',
        sessions: '2',
        schedules: '3',
        memory: '4',
        runs: '5',
    };
    const c = counts || {};
    document.querySelectorAll('.agent-ops-tab').forEach((btn) => {
        const tab = btn.dataset.opsTab || '';
        if (!(tab in c)) return;
        const n = Math.max(0, Number(c[tab]) || 0);
        const el = ensureOpsTabCountEl(btn);
        el.textContent = String(n);
        el.hidden = false;
        el.classList.toggle('is-zero', n === 0);
        const label =
            btn.dataset.opsTabLabel ||
            tab;
        const d = digits[tab] || '';
        btn.setAttribute(
            'title',
            d ? `${label} · ${n} · press ${d}` : `${label} · ${n}`
        );
    });
}

/**
 * Inventory/status pill in an overview card head (tab-count parity).
 * Keeps the glance number next to the title while rows scroll in the body.
 * @param {string} cardId
 * @param {string|null|undefined} text — empty/null hides the pill
 * @param {{ zero?: boolean }} [opts]
 */
function paintOpsOverviewHeadCount(cardId, text, opts) {
    const card = document.getElementById(cardId);
    if (!card) return;
    const head = card.querySelector('.ops-overview-head');
    if (!head) return;
    let el = head.querySelector('.ops-overview-head-count');
    const label = String(text || '').trim();
    if (!label) {
        if (el) {
            el.hidden = true;
            el.textContent = '';
            el.removeAttribute('title');
            el.classList.remove('is-zero');
        }
        return;
    }
    if (!el) {
        el = document.createElement('span');
        el.className = 'ops-overview-head-count';
        el.setAttribute('aria-hidden', 'true');
        const link = head.querySelector('.ops-overview-link');
        if (link) head.insertBefore(el, link);
        else head.appendChild(el);
    }
    el.hidden = false;
    el.textContent = label;
    el.title = label;
    const zero = !!(opts && opts.zero);
    el.classList.toggle('is-zero', zero);
}

function ensureOpsKeyboardHint() {
    const tabs = document.querySelector('.agent-ops-tabs');
    if (!tabs) return;
    let hint = document.getElementById('ops-keyboard-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-keyboard-hint';
        hint.className = 'ops-row-meta ops-keyboard-hint';
        tabs.insertAdjacentElement('afterend', hint);
    }
    hint.textContent =
        'Tips: 0 overview · digits + counts on tabs · overview head counts · Sessions All/Live/Files · filter N/M + Clear · ←/→ · ↑/↓ j/k (no selection → first/last) · PgUp/PgDn Home/End · Space/Enter · c copy id (Copied) · / Esc · r refresh · R digest · ?';
}

/** Hermes-style: ? flashes the keyboard tips row when not typing. */
function flashOpsKeyboardHint() {
    ensureOpsKeyboardHint();
    const hint = document.getElementById('ops-keyboard-hint');
    if (!hint) return false;
    hint.classList.remove('ops-keyboard-hint-flash');
    // Retrigger CSS animation.
    void hint.offsetWidth;
    hint.classList.add('ops-keyboard-hint-flash');
    hint.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
    return true;
}

/** Inject Agents overview card (first) so Agents tab gets active-card parity. */
function ensureOpsOverviewAgentsCard() {
    const grid = document.getElementById('ops-overview-grid');
    if (!grid || document.getElementById('ops-overview-agents')) return;
    const card = document.createElement('div');
    card.className = 'ops-overview-card';
    card.id = 'ops-overview-agents';
    card.innerHTML =
        `<div class="ops-overview-head">` +
        `<h3>Agents</h3>` +
        `<button type="button" class="ops-overview-link" data-goto-tab="agents">Open</button>` +
        `</div>` +
        `<div class="ops-overview-body" id="ops-overview-agents-body">` +
        `<div class="ops-loading" role="status">Loading…</div>` +
        `</div>`;
    grid.insertBefore(card, grid.firstChild);
}

/** Inject Runs overview card (end) so Runs tab gets active-card parity. */
function ensureOpsOverviewRunsCard() {
    const grid = document.getElementById('ops-overview-grid');
    if (!grid || document.getElementById('ops-overview-runs')) return;
    const card = document.createElement('div');
    card.className = 'ops-overview-card';
    card.id = 'ops-overview-runs';
    card.innerHTML =
        `<div class="ops-overview-head">` +
        `<h3>Runs</h3>` +
        `<button type="button" class="ops-overview-link" data-goto-tab="runs">Open</button>` +
        `</div>` +
        `<div class="ops-overview-body" id="ops-overview-runs-body">` +
        `<div class="ops-loading" role="status">Loading…</div>` +
        `</div>`;
    grid.appendChild(card);
}

/** Inject Digest overview card (after Runs) — digester open hints on the command center. */
function ensureOpsOverviewDigestCard() {
    const grid = document.getElementById('ops-overview-grid');
    if (!grid || document.getElementById('ops-overview-digest')) return;
    ensureOpsOverviewRunsCard();
    const card = document.createElement('div');
    card.className = 'ops-overview-card';
    card.id = 'ops-overview-digest';
    card.innerHTML =
        `<div class="ops-overview-head">` +
        `<h3>Digest</h3>` +
        `<button type="button" class="ops-overview-link" data-goto-tab="runs">Open</button>` +
        `</div>` +
        `<div class="ops-overview-body" id="ops-overview-digest-body">` +
        `<div class="ops-loading" role="status">Loading…</div>` +
        `</div>`;
    const runs = document.getElementById('ops-overview-runs');
    if (runs && runs.nextSibling) {
        grid.insertBefore(card, runs.nextSibling);
    } else if (runs) {
        grid.appendChild(card);
    } else {
        grid.appendChild(card);
    }
}

function setupAgentOps() {
    const activeBtn = document.querySelector('.agent-ops-tab.active');
    if (activeBtn?.dataset?.opsTab) opsActiveTab = activeBtn.dataset.opsTab;
    ensureOpsRefreshRowPlacement();
    ensureOpsOverviewJump();
    ensureOpsTabDigits();
    ensureOpsTabBarToolbarKeyboard();
    ensureOpsFileTabToolbarKeyboard();
    ensureOpsAgentEditActionsToolbarKeyboard();
    ensureOpsPreviewRowsToolbarKeyboard();
    ensureOpsRefreshRowToolbarKeyboard();
    ensureOpsFilterRowsToolbarKeyboard();
    ensureOpsInsightsToolbarKeyboard();
    ensureOpsKeyboardHint();
    ensureOpsUpdatedAgo();
    ensureOpsOverviewAgentsCard();
    ensureOpsOverviewRunsCard();
    ensureOpsOverviewDigestCard();
    wireOpsOverviewCardNavigation();
    syncOpsOverviewCardActive(opsActiveTab);
    syncOpsHealthCardActive(opsActiveTab);
    document.querySelectorAll('.agent-ops-tab').forEach((btn) => {
        btn.addEventListener('click', () => {
            const tab = btn.dataset.opsTab;
            if (!tab) return;
            selectOpsTab(tab);
        });
    });
    document.querySelectorAll('.ops-overview-link').forEach((btn) => {
        btn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            const tab = btn.dataset.gotoTab;
            if (!tab) return;
            // Ensure pane is open (dashboard can leave it collapsed)
            if (agentOpsCollapsed) applyOpsCollapsed(false);
            if (tab === 'agents') preferOpsAgentsEnabledFromOverview();
            if (tab === 'runs') preferOpsRunsLaneFromOverview();
            if (tab === 'schedules') preferOpsSchedulesKindFromOverview();
            if (tab === 'memory') preferOpsMemoryKindFromOverview();
            selectOpsTab(tab);
        });
    });
    const opsRoot =
      document.getElementById('agent-ops-section') ||
      document.querySelector('.agent-ops-section');
    if (opsRoot && opsRoot.dataset.opsClearFilterBound !== '1') {
      opsRoot.dataset.opsClearFilterBound = '1';
      opsRoot.addEventListener('click', (e) => {
        const btn = e.target?.closest?.('.ops-clear-filter, .ops-filter-clear');
        if (!btn || !opsRoot.contains(btn)) return;
        e.preventDefault();
        e.stopPropagation();
        if (btn.dataset.opsOpenAiChat) {
          openOpsAiChat();
          return;
        }
        const gotoTab = btn.dataset.opsGotoTab || '';
        if (gotoTab) {
          if (agentOpsCollapsed) applyOpsCollapsed(false);
          selectOpsTab(gotoTab);
          return;
        }
        clearOpsFilter(btn.dataset.opsClearFilter || '');
      });
    }
    document.querySelectorAll('.ops-file-tab').forEach((btn) => {
        btn.addEventListener('click', () => {
            if (opsAgentDirty[opsAgentFileTab] && opsAgentCache) {
                syncOpsAgentEditorToCache();
            }
            opsAgentFileTab = btn.dataset.file;
            document.querySelectorAll('.ops-file-tab').forEach((b) => b.classList.toggle('active', b === btn));
            renderOpsAgentPreview();
            refreshOpsAgentLoadText();
        });
    });
    document.getElementById('ops-agent-back')?.addEventListener('click', () => {
        if (Object.values(opsAgentDirty).some(Boolean)) {
            const ok = window.confirm('Discard unsaved soul/skill/mood changes?');
            if (!ok) return;
        }
        closeOpsAgentDetail();
    });
    document.getElementById('ops-agent-save')?.addEventListener('click', () => saveOpsAgentFile());
    ensureOpsAgentEditor();
    document.getElementById('ops-refresh-btn')?.addEventListener('click', () =>
        refreshAgentOps({ userTriggered: true })
    );
    document.getElementById('ops-digest-refresh-btn')?.addEventListener('click', () => refreshOpsDigest());
    const loadChatBtn = document.getElementById('ops-session-load-chat');
    loadChatBtn?.addEventListener('click', () => loadOpsSessionIntoChat());
    if (loadChatBtn) {
        loadChatBtn.title = 'Load preview into AI Chat (Enter)';
        if (!loadChatBtn.dataset.opsEnterHint) {
            loadChatBtn.dataset.opsEnterHint = '1';
            loadChatBtn.textContent = 'Load into AI Chat ↵';
        }
    }
    ensureOpsSessionFilter();
    ensureOpsMemoryFilter();
    ensureOpsRunsFilter();
    ensureOpsAgentsFilter();
    ensureOpsSchedulesFilter();
    if (!window.__opsFilterSlashBound) {
        window.__opsFilterSlashBound = true;
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && !e.metaKey && !e.ctrlKey && !e.altKey) {
                if (tryOpsAgentDetailEscape(e)) return;
                if (tryOpsPreviewEscape(e)) return;
                if (tryOpsClearSelectionEscape(e)) return;
            }
            if (
                (e.key === 'Enter' || e.key === ' ')
                && !e.metaKey
                && !e.ctrlKey
                && !e.altKey
            ) {
                const t = e.target;
                const tag = (t && t.tagName) || '';
                // Space in inputs must type a space; Enter in filters still opens rows.
                if (e.key === ' ' && (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable)) {
                    return;
                }
                if (tryOpsSessionEnterLoad(e)) return;
                if (tryOpsRunsEnterLoad(e)) return;
                if (tryOpsSchedulesEnterLoad(e)) return;
                if (tryOpsMemoryEnterLoad(e)) return;
                if (tryOpsAgentsEnterLoad(e)) return;
                if (tryOpsMemoryEnter(e)) return;
                if (tryOpsRunsEnter(e)) return;
                if (tryOpsAgentsEnter(e)) return;
                if (tryOpsSchedulesEnter(e)) return;
            }
            // Hermes-style: 0 jumps to overview; 1–5 jump detail tabs (when not typing).
            if (!e.metaKey && !e.ctrlKey && !e.altKey && /^[0-5]$/.test(e.key)) {
                if (agentOpsCollapsed) return;
                const t = e.target;
                const tag = (t && t.tagName) || '';
                if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return;
                if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
                    return;
                }
                if (e.key === '0') {
                    e.preventDefault();
                    showOpsOverview();
                    return;
                }
                const byDigit = {
                    1: 'agents',
                    2: 'sessions',
                    3: 'schedules',
                    4: 'memory',
                    5: 'runs',
                };
                const tab = byDigit[e.key];
                if (tab) {
                    e.preventDefault();
                    selectOpsTab(tab);
                    return;
                }
            }
            if (!e.metaKey && !e.ctrlKey && !e.altKey && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
                if (agentOpsCollapsed) return;
                const t = e.target;
                const tag = (t && t.tagName) || '';
                if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return;
                if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
                    return;
                }
                const order = ['agents', 'sessions', 'schedules', 'memory', 'runs'];
                let idx = order.indexOf(opsActiveTab);
                if (idx < 0) idx = 0;
                idx = e.key === 'ArrowRight'
                    ? (idx + 1) % order.length
                    : (idx - 1 + order.length) % order.length;
                e.preventDefault();
                selectOpsTab(order[idx]);
                return;
            }
            if (!e.metaKey && !e.ctrlKey && !e.altKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
                if (tryOpsArrowMoveSelection(e)) return;
            }
            if (!e.metaKey && !e.ctrlKey && !e.altKey && (e.key === 'Home' || e.key === 'End')) {
                if (tryOpsArrowMoveSelection(e)) return;
            }
            if (!e.metaKey && !e.ctrlKey && !e.altKey && (e.key === 'PageUp' || e.key === 'PageDown')) {
                if (tryOpsArrowMoveSelection(e)) return;
            }
            if (!e.metaKey && !e.ctrlKey && !e.altKey && (e.key === 'j' || e.key === 'k')) {
                if (tryOpsArrowMoveSelection(e)) return;
            }
            if (!e.metaKey && !e.ctrlKey && !e.altKey && (e.key === 'c' || e.key === 'C')) {
                if (tryOpsCopySelected(e)) return;
            }
            if (e.key === '?' && !e.metaKey && !e.ctrlKey && !e.altKey) {
                if (agentOpsCollapsed) return;
                const t = e.target;
                const tag = (t && t.tagName) || '';
                if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return;
                if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
                    return;
                }
                if (flashOpsKeyboardHint()) {
                    e.preventDefault();
                }
                return;
            }
            if (e.key === 'R' && !e.metaKey && !e.ctrlKey && !e.altKey) {
                if (agentOpsCollapsed) return;
                const t = e.target;
                const tag = (t && t.tagName) || '';
                if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return;
                if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
                    return;
                }
                e.preventDefault();
                refreshOpsDigest();
                return;
            }
            if (e.key === 'r' && !e.metaKey && !e.ctrlKey && !e.altKey) {
                if (agentOpsCollapsed) return;
                const t = e.target;
                const tag = (t && t.tagName) || '';
                if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return;
                if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
                    return;
                }
                e.preventDefault();
                refreshAgentOps({ userTriggered: true });
                return;
            }
            if (e.key !== '/' || e.metaKey || e.ctrlKey || e.altKey) return;
            if (agentOpsCollapsed) return;
            const t = e.target;
            const tag = (t && t.tagName) || '';
            if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return;
            if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
                return;
            }
            if (focusActiveOpsFilter()) {
                e.preventDefault();
            }
        });
    }
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
        input.placeholder = 'Filter live + files… (/ focus, Esc clears)';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        ensureOpsFilterMatchChip(input);
        ensureOpsFilterClearBtn(input);
        panel.insertBefore(row, panel.firstChild);
        wireOpsFilterRowToolbarKeyboard(row);
    }
    ensureOpsSessionKindChips();
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    ensureOpsFilterMatchChip(input);
    ensureOpsFilterClearBtn(input);
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

/** All · Live · Files chips (Monitors / Top Processes filter parity). */
function ensureOpsSessionKindChips() {
    const panel = document.getElementById('ops-panel-sessions');
    const filterRow = panel && panel.querySelector('.ops-filter-row');
    if (!panel || !filterRow) return;
    let wrap = document.getElementById('ops-session-kind-chips');
    if (!wrap) {
        wrap = document.createElement('div');
        wrap.id = 'ops-session-kind-chips';
        wrap.className = 'ops-session-kind-chips';
        wrap.setAttribute('role', 'group');
        wrap.setAttribute('aria-label', 'Session kind filter');
        wrap.innerHTML =
            '<button type="button" class="ops-session-kind-chip is-active" data-ops-session-kind="all" aria-pressed="true" title="Show live sessions and saved files">All</button>' +
            '<button type="button" class="ops-session-kind-chip" data-ops-session-kind="live" aria-pressed="false" title="Show live sessions only">Live <span class="ops-session-kind-count" data-ops-session-kind-count="live">0</span></button>' +
            '<button type="button" class="ops-session-kind-chip" data-ops-session-kind="files" aria-pressed="false" title="Show saved session files only">Files <span class="ops-session-kind-count" data-ops-session-kind-count="files">0</span></button>';
        filterRow.insertAdjacentElement('afterend', wrap);
        wrap.addEventListener('click', (e) => {
            const btn =
                e.target && e.target.closest && e.target.closest('[data-ops-session-kind]');
            if (!btn || !wrap.contains(btn)) return;
            setOpsSessionKindFilter(btn.getAttribute('data-ops-session-kind') || 'all');
        });
    }
    if (typeof window.wireFilterChipToolbarKeyboard === 'function') {
        window.wireFilterChipToolbarKeyboard(wrap);
    }
    paintOpsSessionKindChips();
    applyOpsSessionKindVisibility();
}

function setOpsSessionKindFilter(mode) {
    const next =
        mode === 'live' || mode === 'files' || mode === 'all' ? mode : 'all';
    opsSessionKindFilter = next;
    paintOpsSessionKindChips();
    applyOpsSessionKindVisibility();
    renderOpsLive(opsLiveCache);
    renderOpsSessionFiles(opsSessionFilesCache);
}

function paintOpsSessionKindChips() {
    const wrap = document.getElementById('ops-session-kind-chips');
    if (!wrap) return;
    const liveAll = opsLiveCache || [];
    const filesAll = opsSessionFilesCache || [];
    const liveEl = wrap.querySelector('[data-ops-session-kind-count="live"]');
    const filesEl = wrap.querySelector('[data-ops-session-kind-count="files"]');
    if (liveEl) liveEl.textContent = String(liveAll.length);
    if (filesEl) filesEl.textContent = String(filesAll.length);
    wrap.querySelectorAll('[data-ops-session-kind]').forEach((btn) => {
        const key = btn.getAttribute('data-ops-session-kind');
        const on = key === opsSessionKindFilter;
        btn.classList.toggle('is-active', on);
        btn.setAttribute('aria-pressed', on ? 'true' : 'false');
        if (key === 'live') {
            btn.classList.toggle('has-hits', liveAll.length > 0);
        } else if (key === 'files') {
            btn.classList.toggle('has-hits', filesAll.length > 0);
        }
    });
}

/** Hide Live or Files blocks when a kind chip narrows the Sessions tab. */
function applyOpsSessionKindVisibility() {
    const liveList = document.getElementById('ops-live-sessions');
    const fileList = document.getElementById('ops-session-files');
    if (!liveList || !fileList) return;
    const liveHead = liveList.previousElementSibling;
    const fileHead = fileList.previousElementSibling;
    const showLive = opsSessionKindFilter === 'all' || opsSessionKindFilter === 'live';
    const showFiles = opsSessionKindFilter === 'all' || opsSessionKindFilter === 'files';
    if (liveHead && liveHead.classList && liveHead.classList.contains('ops-subhead')) {
        liveHead.hidden = !showLive;
    }
    liveList.hidden = !showLive;
    if (fileHead && fileHead.classList && fileHead.classList.contains('ops-subhead')) {
        fileHead.hidden = !showFiles;
    }
    fileList.hidden = !showFiles;
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
        input.placeholder = 'Filter knowledge files… (/ focus, Esc clears)';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        ensureOpsFilterMatchChip(input);
        ensureOpsFilterClearBtn(input);
        panel.insertBefore(row, panel.firstChild);
        wireOpsFilterRowToolbarKeyboard(row);
    }
    ensureOpsMemoryKindChips();
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    ensureOpsFilterMatchChip(input);
    ensureOpsFilterClearBtn(input);
    input.addEventListener('input', () => {
        opsMemoryFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsMemory(opsMemoryCache);
    });
    bindOpsFilterEscape(input, () => {
        opsMemoryFilterQ = '';
        renderOpsMemory(opsMemoryCache);
    });
}

/** All · Discord · Core chips (Sessions Live/Files parity). */
function ensureOpsMemoryKindChips() {
    const panel = document.getElementById('ops-panel-memory');
    const filterRow = panel && panel.querySelector('.ops-filter-row');
    if (!panel || !filterRow) return;
    let wrap = document.getElementById('ops-memory-kind-chips');
    if (!wrap) {
        wrap = document.createElement('div');
        wrap.id = 'ops-memory-kind-chips';
        wrap.className = 'ops-memory-kind-chips';
        wrap.setAttribute('role', 'group');
        wrap.setAttribute('aria-label', 'Knowledge kind filter');
        wrap.innerHTML =
            '<button type="button" class="ops-memory-kind-chip is-active" data-ops-memory-kind="all" aria-pressed="true" title="Show every knowledge file">All</button>' +
            '<button type="button" class="ops-memory-kind-chip" data-ops-memory-kind="discord" aria-pressed="false" title="Show Discord channel memory files only">Discord <span class="ops-memory-kind-count" data-ops-memory-kind-count="discord">0</span></button>' +
            '<button type="button" class="ops-memory-kind-chip" data-ops-memory-kind="core" aria-pressed="false" title="Show soul / global / main files only">Core <span class="ops-memory-kind-count" data-ops-memory-kind-count="core">0</span></button>';
        filterRow.insertAdjacentElement('afterend', wrap);
        wrap.addEventListener('click', (e) => {
            const btn =
                e.target && e.target.closest && e.target.closest('[data-ops-memory-kind]');
            if (!btn || !wrap.contains(btn)) return;
            setOpsMemoryKindFilter(btn.getAttribute('data-ops-memory-kind') || 'all');
        });
    }
    if (typeof window.wireFilterChipToolbarKeyboard === 'function') {
        window.wireFilterChipToolbarKeyboard(wrap);
    }
    paintOpsMemoryKindChips();
}

function setOpsMemoryKindFilter(mode) {
    const next =
        mode === 'discord' || mode === 'core' || mode === 'all' ? mode : 'all';
    opsMemoryKindFilter = next;
    paintOpsMemoryKindChips();
    renderOpsMemory(opsMemoryCache);
}

function paintOpsMemoryKindChips() {
    const wrap = document.getElementById('ops-memory-kind-chips');
    if (!wrap) return;
    const all = opsMemoryCache || [];
    const discordN = all.filter((f) => memoryRowMatchesKind(f, 'discord')).length;
    const coreN = all.filter((f) => memoryRowMatchesKind(f, 'core')).length;
    const discordEl = wrap.querySelector('[data-ops-memory-kind-count="discord"]');
    const coreEl = wrap.querySelector('[data-ops-memory-kind-count="core"]');
    if (discordEl) discordEl.textContent = String(discordN);
    if (coreEl) coreEl.textContent = String(coreN);
    wrap.querySelectorAll('[data-ops-memory-kind]').forEach((btn) => {
        const key = btn.getAttribute('data-ops-memory-kind');
        const on = key === opsMemoryKindFilter;
        btn.classList.toggle('is-active', on);
        btn.setAttribute('aria-pressed', on ? 'true' : 'false');
        if (key === 'discord') {
            btn.classList.toggle('has-hits', discordN > 0);
        } else if (key === 'core') {
            btn.classList.toggle('has-hits', coreN > 0);
        }
    });
}

/** Overview Knowledge open → Discord when any else Core (Sessions Live/Files parity). */
function preferOpsMemoryKindFromOverview() {
    const rows = opsMemoryCache || [];
    if (!rows.length) {
        setOpsMemoryKindFilter('all');
        return;
    }
    const discordN = rows.filter((f) => memoryRowMatchesKind(f, 'discord')).length;
    if (discordN > 0) {
        setOpsMemoryKindFilter('discord');
        return;
    }
    const coreN = rows.filter((f) => memoryRowMatchesKind(f, 'core')).length;
    setOpsMemoryKindFilter(coreN > 0 ? 'core' : 'all');
}

function memoryRowMatchesKind(f, mode) {
    const kind = String(f?.kind || '').toLowerCase();
    const want = mode || opsMemoryKindFilter;
    if (want === 'discord') return kind === 'discord';
    if (want === 'core') return kind === 'soul' || kind === 'global' || kind === 'main';
    return true;
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
        input.placeholder = 'Filter runs by lane, tool, question… (/ focus, Esc clears)';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        ensureOpsFilterMatchChip(input);
        ensureOpsFilterClearBtn(input);
        const insights = document.getElementById('ops-runs-insights');
        if (insights) panel.insertBefore(row, insights.nextSibling);
        else panel.insertBefore(row, panel.firstChild);
        wireOpsFilterRowToolbarKeyboard(row);
    }
    ensureOpsRunsLaneChips();
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    ensureOpsFilterMatchChip(input);
    ensureOpsFilterClearBtn(input);
    input.addEventListener('input', () => {
        opsRunsFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsRuns(opsRunsInsightsCache);
    });
    bindOpsFilterEscape(input, () => {
        opsRunsFilterQ = '';
        renderOpsRuns(opsRunsInsightsCache);
    });
}

/** All · Instant · Direct chips (Agents / Sessions filter parity). */
function ensureOpsRunsLaneChips() {
    const panel = document.getElementById('ops-panel-runs');
    const filterRow = panel && panel.querySelector('.ops-filter-row');
    if (!panel || !filterRow) return;
    let wrap = document.getElementById('ops-runs-lane-chips');
    if (!wrap) {
        wrap = document.createElement('div');
        wrap.id = 'ops-runs-lane-chips';
        wrap.className = 'ops-runs-lane-chips';
        wrap.setAttribute('role', 'group');
        wrap.setAttribute('aria-label', 'Run lane filter');
        wrap.innerHTML =
            '<button type="button" class="ops-runs-lane-chip is-active" data-ops-runs-lane="all" aria-pressed="true" title="Show every run lane">All</button>' +
            '<button type="button" class="ops-runs-lane-chip" data-ops-runs-lane="instant" aria-pressed="false" title="Show instant-lane runs only">Instant <span class="ops-runs-lane-count" data-ops-runs-lane-count="instant">0</span></button>' +
            '<button type="button" class="ops-runs-lane-chip" data-ops-runs-lane="direct" aria-pressed="false" title="Show direct-lane runs only">Direct <span class="ops-runs-lane-count" data-ops-runs-lane-count="direct">0</span></button>';
        filterRow.insertAdjacentElement('afterend', wrap);
        wrap.addEventListener('click', (e) => {
            const btn =
                e.target && e.target.closest && e.target.closest('[data-ops-runs-lane]');
            if (!btn || !wrap.contains(btn)) return;
            setOpsRunsLaneFilter(btn.getAttribute('data-ops-runs-lane') || 'all');
        });
    }
    if (typeof window.wireFilterChipToolbarKeyboard === 'function') {
        window.wireFilterChipToolbarKeyboard(wrap);
    }
    paintOpsRunsLaneChips();
}

function setOpsRunsLaneFilter(mode) {
    const next =
        mode === 'instant' || mode === 'direct' || mode === 'all' ? mode : 'all';
    opsRunsLaneFilter = next;
    paintOpsRunsLaneChips();
    renderOpsRuns(opsRunsInsightsCache);
}

function paintOpsRunsLaneChips() {
    const wrap = document.getElementById('ops-runs-lane-chips');
    if (!wrap) return;
    const recent = Array.isArray(opsRunsInsightsCache?.recent)
        ? opsRunsInsightsCache.recent
        : [];
    const instantN = recent.filter((r) => runsRowMatchesLane(r, 'instant')).length;
    const directN = recent.filter((r) => runsRowMatchesLane(r, 'direct')).length;
    const instantEl = wrap.querySelector('[data-ops-runs-lane-count="instant"]');
    const directEl = wrap.querySelector('[data-ops-runs-lane-count="direct"]');
    if (instantEl) instantEl.textContent = String(instantN);
    if (directEl) directEl.textContent = String(directN);
    wrap.querySelectorAll('[data-ops-runs-lane]').forEach((btn) => {
        const key = btn.getAttribute('data-ops-runs-lane');
        const active = key === opsRunsLaneFilter;
        btn.classList.toggle('is-active', active);
        btn.setAttribute('aria-pressed', active ? 'true' : 'false');
        if (key === 'instant') {
            btn.classList.toggle('has-hits', instantN > 0);
        } else if (key === 'direct') {
            btn.classList.toggle('has-hits', directN > 0);
        }
    });
}

/** Overview Runs open → Direct when any direct else Instant when any instant (Agents On/Off parity). */
function preferOpsRunsLaneFromOverview() {
    const recent = Array.isArray(opsRunsInsightsCache?.recent)
        ? opsRunsInsightsCache.recent
        : [];
    if (!recent.length) {
        setOpsRunsLaneFilter('all');
        return;
    }
    const directN = recent.filter((r) => runsRowMatchesLane(r, 'direct')).length;
    if (directN > 0) {
        setOpsRunsLaneFilter('direct');
        return;
    }
    const instantN = recent.filter((r) => runsRowMatchesLane(r, 'instant')).length;
    setOpsRunsLaneFilter(instantN > 0 ? 'instant' : 'all');
}

function runsRowMatchesLane(r, mode) {
    const lane = String(r?.lane || '').toLowerCase();
    const want = mode || opsRunsLaneFilter;
    if (want === 'instant') return lane === 'instant';
    if (want === 'direct') return lane === 'direct';
    return true;
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
        input.placeholder = 'Filter agents by name, slug, model… (/ focus, Esc clears)';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        ensureOpsFilterMatchChip(input);
        ensureOpsFilterClearBtn(input);
        const list = document.getElementById('ops-agents-list');
        if (list) panel.insertBefore(row, list);
        else panel.insertBefore(row, panel.firstChild);
        wireOpsFilterRowToolbarKeyboard(row);
    }
    ensureOpsAgentsEnabledChips();
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    ensureOpsFilterMatchChip(input);
    ensureOpsFilterClearBtn(input);
    input.addEventListener('input', () => {
        opsAgentsFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsAgents(opsAgentsCache);
    });
    bindOpsFilterEscape(input, () => {
        opsAgentsFilterQ = '';
        renderOpsAgents(opsAgentsCache);
    });
}

/** All · On · Off chips (Sessions / Monitors filter parity). */
function ensureOpsAgentsEnabledChips() {
    const panel = document.getElementById('ops-panel-agents');
    const filterRow = panel && panel.querySelector('.ops-filter-row');
    if (!panel || !filterRow) return;
    let wrap = document.getElementById('ops-agents-enabled-chips');
    if (!wrap) {
        wrap = document.createElement('div');
        wrap.id = 'ops-agents-enabled-chips';
        wrap.className = 'ops-agents-enabled-chips';
        wrap.setAttribute('role', 'group');
        wrap.setAttribute('aria-label', 'Agent enabled filter');
        wrap.innerHTML =
            '<button type="button" class="ops-agents-enabled-chip is-active" data-ops-agents-enabled="all" aria-pressed="true" title="Show every agent">All</button>' +
            '<button type="button" class="ops-agents-enabled-chip" data-ops-agents-enabled="on" aria-pressed="false" title="Show enabled agents only">On <span class="ops-agents-enabled-count" data-ops-agents-enabled-count="on">0</span></button>' +
            '<button type="button" class="ops-agents-enabled-chip" data-ops-agents-enabled="off" aria-pressed="false" title="Show disabled agents only">Off <span class="ops-agents-enabled-count" data-ops-agents-enabled-count="off">0</span></button>';
        filterRow.insertAdjacentElement('afterend', wrap);
        wrap.addEventListener('click', (e) => {
            const btn =
                e.target && e.target.closest && e.target.closest('[data-ops-agents-enabled]');
            if (!btn || !wrap.contains(btn)) return;
            setOpsAgentsEnabledFilter(btn.getAttribute('data-ops-agents-enabled') || 'all');
        });
    }
    if (typeof window.wireFilterChipToolbarKeyboard === 'function') {
        window.wireFilterChipToolbarKeyboard(wrap);
    }
    paintOpsAgentsEnabledChips();
}

function setOpsAgentsEnabledFilter(mode) {
    const next = mode === 'on' || mode === 'off' || mode === 'all' ? mode : 'all';
    opsAgentsEnabledFilter = next;
    paintOpsAgentsEnabledChips();
    renderOpsAgents(opsAgentsCache);
}

function paintOpsAgentsEnabledChips() {
    const wrap = document.getElementById('ops-agents-enabled-chips');
    if (!wrap) return;
    const all = opsAgentsCache || [];
    const onN = all.filter((a) => a.enabled).length;
    const offN = all.length - onN;
    const onEl = wrap.querySelector('[data-ops-agents-enabled-count="on"]');
    const offEl = wrap.querySelector('[data-ops-agents-enabled-count="off"]');
    if (onEl) onEl.textContent = String(onN);
    if (offEl) offEl.textContent = String(offN);
    wrap.querySelectorAll('[data-ops-agents-enabled]').forEach((btn) => {
        const key = btn.getAttribute('data-ops-agents-enabled');
        const active = key === opsAgentsEnabledFilter;
        btn.classList.toggle('is-active', active);
        btn.setAttribute('aria-pressed', active ? 'true' : 'false');
        if (key === 'on') {
            btn.classList.toggle('has-hits', onN > 0);
        } else if (key === 'off') {
            btn.classList.toggle('has-hits', offN > 0);
        }
    });
}

/** Overview Agents open → On when any enabled, else Off when agents exist (Sessions Live/Files parity). */
function preferOpsAgentsEnabledFromOverview() {
    const rows = opsAgentsCache || [];
    if (!rows.length) {
        setOpsAgentsEnabledFilter('all');
        return;
    }
    const onN = rows.filter((a) => a.enabled).length;
    setOpsAgentsEnabledFilter(onN === 0 ? 'off' : 'on');
}

function agentsRowMatchesEnabled(a) {
    if (opsAgentsEnabledFilter === 'on') return !!a.enabled;
    if (opsAgentsEnabledFilter === 'off') return !a.enabled;
    return true;
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
        input.placeholder = 'Filter schedules + deliveries… (/ focus, Esc clears)';
        input.autocomplete = 'off';
        input.spellcheck = false;
        row.appendChild(input);
        ensureOpsFilterMatchChip(input);
        ensureOpsFilterClearBtn(input);
        const list = document.getElementById('ops-schedules-list');
        const sub = panel.querySelector('.ops-subhead');
        if (sub) panel.insertBefore(row, sub);
        else if (list) panel.insertBefore(row, list);
        else panel.insertBefore(row, panel.firstChild);
        wireOpsFilterRowToolbarKeyboard(row);
    }
    ensureOpsSchedulesKindChips();
    if (input.dataset.opsBound === '1') return;
    input.dataset.opsBound = '1';
    ensureOpsFilterMatchChip(input);
    ensureOpsFilterClearBtn(input);
    input.addEventListener('input', () => {
        opsSchedulesFilterQ = (input.value || '').trim().toLowerCase();
        renderOpsSchedulesTab(opsSchedulesCache, opsDeliveriesCache);
    });
    bindOpsFilterEscape(input, () => {
        opsSchedulesFilterQ = '';
        renderOpsSchedulesTab(opsSchedulesCache, opsDeliveriesCache);
    });
}

/** All · Jobs · Deliveries chips (Sessions Live/Files parity). */
function ensureOpsSchedulesKindChips() {
    const panel = document.getElementById('ops-panel-schedules');
    const filterRow = panel && panel.querySelector('.ops-filter-row');
    if (!panel || !filterRow) return;
    let wrap = document.getElementById('ops-schedules-kind-chips');
    if (!wrap) {
        wrap = document.createElement('div');
        wrap.id = 'ops-schedules-kind-chips';
        wrap.className = 'ops-schedules-kind-chips';
        wrap.setAttribute('role', 'group');
        wrap.setAttribute('aria-label', 'Schedule kind filter');
        wrap.innerHTML =
            '<button type="button" class="ops-schedules-kind-chip is-active" data-ops-schedules-kind="all" aria-pressed="true" title="Show active schedules and recent deliveries">All</button>' +
            '<button type="button" class="ops-schedules-kind-chip" data-ops-schedules-kind="jobs" aria-pressed="false" title="Show active schedules only">Jobs <span class="ops-schedules-kind-count" data-ops-schedules-kind-count="jobs">0</span></button>' +
            '<button type="button" class="ops-schedules-kind-chip" data-ops-schedules-kind="deliveries" aria-pressed="false" title="Show recent deliveries only">Deliveries <span class="ops-schedules-kind-count" data-ops-schedules-kind-count="deliveries">0</span></button>';
        filterRow.insertAdjacentElement('afterend', wrap);
        wrap.addEventListener('click', (e) => {
            const btn =
                e.target && e.target.closest && e.target.closest('[data-ops-schedules-kind]');
            if (!btn || !wrap.contains(btn)) return;
            setOpsSchedulesKindFilter(btn.getAttribute('data-ops-schedules-kind') || 'all');
        });
    }
    if (typeof window.wireFilterChipToolbarKeyboard === 'function') {
        window.wireFilterChipToolbarKeyboard(wrap);
    }
    paintOpsSchedulesKindChips();
    applyOpsSchedulesKindVisibility();
}

function setOpsSchedulesKindFilter(mode) {
    const next =
        mode === 'jobs' || mode === 'deliveries' || mode === 'all' ? mode : 'all';
    opsSchedulesKindFilter = next;
    paintOpsSchedulesKindChips();
    applyOpsSchedulesKindVisibility();
    renderOpsSchedulesTab(opsSchedulesCache, opsDeliveriesCache);
}

function paintOpsSchedulesKindChips() {
    const wrap = document.getElementById('ops-schedules-kind-chips');
    if (!wrap) return;
    const jobsAll = opsSchedulesCache || [];
    const delAll = opsDeliveriesCache || [];
    const jobsEl = wrap.querySelector('[data-ops-schedules-kind-count="jobs"]');
    const delEl = wrap.querySelector('[data-ops-schedules-kind-count="deliveries"]');
    if (jobsEl) jobsEl.textContent = String(jobsAll.length);
    if (delEl) delEl.textContent = String(delAll.length);
    wrap.querySelectorAll('[data-ops-schedules-kind]').forEach((btn) => {
        const key = btn.getAttribute('data-ops-schedules-kind');
        const on = key === opsSchedulesKindFilter;
        btn.classList.toggle('is-active', on);
        btn.setAttribute('aria-pressed', on ? 'true' : 'false');
        if (key === 'jobs') {
            btn.classList.toggle('has-hits', jobsAll.length > 0);
        } else if (key === 'deliveries') {
            btn.classList.toggle('has-hits', delAll.length > 0);
        }
    });
}

/** Hide Jobs or Deliveries blocks when a kind chip narrows the Schedules tab. */
function applyOpsSchedulesKindVisibility() {
    const jobList = document.getElementById('ops-schedules-list');
    const delList = document.getElementById('ops-deliveries-list');
    if (!jobList || !delList) return;
    const jobHead = jobList.previousElementSibling;
    const delHead = delList.previousElementSibling;
    const showJobs = opsSchedulesKindFilter === 'all' || opsSchedulesKindFilter === 'jobs';
    const showDel = opsSchedulesKindFilter === 'all' || opsSchedulesKindFilter === 'deliveries';
    if (jobHead && jobHead.classList && jobHead.classList.contains('ops-subhead')) {
        jobHead.hidden = !showJobs;
    }
    jobList.hidden = !showJobs;
    if (delHead && delHead.classList && delHead.classList.contains('ops-subhead')) {
        delHead.hidden = !showDel;
    }
    delList.hidden = !showDel;
}

/** Overview Schedules open → Jobs when any else Deliveries (Sessions Live/Files parity). */
function preferOpsSchedulesKindFromOverview() {
    const jobs = opsSchedulesCache || [];
    const dels = opsDeliveriesCache || [];
    if (jobs.length) {
        setOpsSchedulesKindFilter('jobs');
        return;
    }
    if (dels.length) {
        setOpsSchedulesKindFilter('deliveries');
        return;
    }
    setOpsSchedulesKindFilter('all');
}

function schedulesRowMatchesFilter(haystack) {
    if (!opsSchedulesFilterQ) return true;
    return String(haystack || '').toLowerCase().includes(opsSchedulesFilterQ);
}

/** Combined live + files match count for the shared Sessions filter chip. */
function paintOpsSessionFilterFromCaches() {
    ensureOpsSessionKindChips();
    const liveAll = opsLiveCache || [];
    const liveShown = liveAll.filter((r) =>
        sessionRowMatchesFilter(
            `${r.source} ${r.session_id} ${r.preview || ''} ${r.last_activity || ''}`
        )
    ).length;
    const filesAll = opsSessionFilesCache || [];
    const filesShown = filesAll.filter((f) =>
        sessionRowMatchesFilter(
            `${f.slug || ''} ${f.name || ''} ${f.source_hint || ''} ${f.preview || ''}`
        )
    ).length;
    let total = liveAll.length + filesAll.length;
    let shown = liveShown + filesShown;
    if (opsSessionKindFilter === 'live') {
        total = liveAll.length;
        shown = liveShown;
    } else if (opsSessionKindFilter === 'files') {
        total = filesAll.length;
        shown = filesShown;
    }
    paintOpsFilterMatch(
        'ops-session-filter',
        total,
        shown,
        opsSessionFilterQ
    );
    paintOpsSessionKindChips();
    applyOpsSessionKindVisibility();
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

/** Parse Discord gateway insight string (health card + collapsed glance). */
function parseOpsDiscordGateway(dgRaw) {
    const dg = String(dgRaw || '').trim();
    const readyMatch = dg.match(/last Ready\s+([^·]+)/i);
    const discMatch = dg.match(/disconnect×(\d+)/i);
    const resumeMatch = dg.match(/resume×(\d+)/i);
    const stageMatch = dg.match(/stage=([^\s·]+)/i);
    const lastDiscMatch = dg.match(/last disc\s+([^·]+)/i);
    const lastResumeMatch = dg.match(/last resume\s+([^·]+)/i);
    const discN = discMatch ? Number(discMatch[1]) : 0;
    const resumeN = resumeMatch ? Number(resumeMatch[1]) : 0;
    const stage = (stageMatch ? stageMatch[1] : '').trim();
    const lastDisc = lastDiscMatch ? lastDiscMatch[1].trim() : '';
    const lastResume = lastResumeMatch ? lastResumeMatch[1].trim() : '';
    let healthText = readyMatch ? readyMatch[1].trim() : dg ? 'see Runs' : '—';
    if (discN > 0) {
        healthText = lastDisc
            ? `${healthText} · disc×${discN} (${lastDisc})`
            : `${healthText} · disc×${discN}`;
    }
    if (resumeN > 0) {
        healthText = lastResume
            ? `${healthText} · res×${resumeN} (${lastResume})`
            : `${healthText} · res×${resumeN}`;
    }
    const stageLower = stage.toLowerCase();
    let wash = 'empty';
    if (!dg || healthText === '—') {
        wash = 'empty';
    } else if (stageLower === 'disconnected') {
        wash = 'offline';
    } else if (discN > 0 || resumeN > 0 || stageLower === 'resuming') {
        wash = 'warn';
    } else if (stageLower === 'connected' || readyMatch) {
        wash = 'ready';
    }
    let glanceLine = 'Discord · —';
    if (wash === 'offline') {
        glanceLine = 'Discord · Offline';
    } else if (readyMatch) {
        const age = readyMatch[1].trim();
        glanceLine =
            discN > 0
                ? `Discord · Ready · ${age} · disc×${discN}`
                : `Discord · Ready · ${age}`;
        if (wash === 'warn' && discN <= 0 && resumeN > 0) {
            glanceLine = `Discord · Resuming · ${age}`;
        }
    } else if (dg) {
        glanceLine = `Discord · ${healthText}`;
    }
    return { dg, healthText, wash, glanceLine, stage, discN, resumeN, readyMatch };
}

/** Collapsed-section glance under Agent Ops header (Discord Ready / Monitors parity). */
function ensureOpsCollapsedGlance() {
    const header = document.getElementById('agent-ops-header');
    if (!header) return null;
    let glance = document.getElementById('agent-ops-collapsed-glance');
    if (!glance) {
        glance = document.createElement('div');
        glance.id = 'agent-ops-collapsed-glance';
        glance.className = 'agent-ops-collapsed-glance';
        glance.hidden = true;
        glance.innerHTML = '<span id="agent-ops-collapsed-glance-text"></span>';
        header.insertAdjacentElement('afterend', glance);
        wireOpsCollapsedGlanceClick(glance);
    }
    return glance;
}

function syncOpsCollapsedGlance() {
    const glance = document.getElementById('agent-ops-collapsed-glance');
    if (glance) glance.hidden = true;
}

function activateOpsCollapsedGlance() {
    const gw = String(opsRunsInsightsCache?.discord_gateway || '').trim();
    if (gw && openOpsDiscordGatewayPreviewNavigate(gw)) {
        syncOpsCollapsedGlance();
        return;
    }
    if (agentOpsCollapsed) applyOpsCollapsed(false);
    selectOpsTab('runs');
    syncOpsCollapsedGlance();
}

function wireOpsCollapsedGlanceClick(glance) {
    if (!glance || glance.dataset.opsCollapsedGlanceWired === '1') return;
    glance.dataset.opsCollapsedGlanceWired = '1';
    const activate = () => {
        activateOpsCollapsedGlance();
    };
    glance.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        activate();
    });
    glance.addEventListener('keydown', (e) => {
        if (e.key !== 'Enter' && e.key !== ' ') return;
        e.preventDefault();
        e.stopPropagation();
        activate();
    });
}

function stopOpsGlancePoll() {
    if (agentOpsGlanceInterval) {
        clearInterval(agentOpsGlanceInterval);
        agentOpsGlanceInterval = null;
    }
}

async function pollOpsCollapsedGlance() {
    if (!agentOpsCollapsed) return;
    try {
        const insights = await invoke('get_runs_insights', { limit: 8 });
        if (insights && typeof insights === 'object') {
            opsRunsInsightsCache = {
                ...(opsRunsInsightsCache || {}),
                ...insights,
            };
        }
        syncOpsCollapsedGlance();
    } catch (_) {
        /* glance poll is best-effort */
        syncOpsCollapsedGlance();
    }
}

function startOpsGlancePoll() {
    stopOpsGlancePoll();
    ensureOpsCollapsedGlance();
    void pollOpsCollapsedGlance();
    agentOpsGlanceInterval = setInterval(() => {
        if (!agentOpsCollapsed) {
            stopOpsGlancePoll();
            return;
        }
        void pollOpsCollapsedGlance();
    }, OPS_GLANCE_POLL_INTERVAL);
}

function setOpsDigestRefreshBusy(busy) {
    const btn = document.getElementById('ops-digest-refresh-btn');
    if (!btn) return;
    if (opsDigestRefreshFlashTimer) {
        clearTimeout(opsDigestRefreshFlashTimer);
        opsDigestRefreshFlashTimer = null;
    }
    btn.classList.remove('is-just-saved');
    if (!btn.dataset.idleLabel) {
        btn.dataset.idleLabel = btn.textContent || 'Refresh digest';
    }
    if (busy) {
        btn.disabled = true;
        btn.textContent = 'Refreshing…';
        btn.title = 'Digest refresh in progress';
    } else {
        btn.disabled = false;
        btn.textContent = btn.dataset.idleLabel || 'Refresh digest';
        btn.title = 'Refresh agent digest';
    }
}

function flashOpsDigestRefreshed() {
    const btn = document.getElementById('ops-digest-refresh-btn');
    if (!btn) return;
    if (opsDigestRefreshFlashTimer) {
        clearTimeout(opsDigestRefreshFlashTimer);
        opsDigestRefreshFlashTimer = null;
    }
    const idle = btn.dataset.idleLabel || 'Refresh digest';
    btn.disabled = false;
    btn.classList.add('is-just-saved');
    btn.textContent = 'Refreshed';
    btn.title = 'Digest refresh complete';
    opsDigestRefreshFlashTimer = setTimeout(() => {
        btn.classList.remove('is-just-saved');
        btn.textContent = idle;
        btn.title = 'Refresh agent digest';
        opsDigestRefreshFlashTimer = null;
    }, 1600);
}

async function refreshOpsDigest() {
    if (opsDigestRefreshInFlight) return;
    opsDigestRefreshInFlight = true;
    const digestEl = document.getElementById('ops-health-digest');
    setOpsDigestRefreshBusy(true);
    let ok = false;
    try {
        const msg = await invoke('refresh_agent_digest');
        if (digestEl) digestEl.textContent = String(msg).slice(0, 80);
        await refreshAgentOps();
        ok = true;
    } catch (err) {
        console.warn('[Agent Ops] digest refresh', err);
        if (digestEl) digestEl.textContent = `Refresh failed`;
    } finally {
        opsDigestRefreshInFlight = false;
        setOpsDigestRefreshBusy(false);
        if (ok) flashOpsDigestRefreshed();
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

function fmtUptimeSecs(secs) {
    const s = Math.max(0, Math.floor(Number(secs) || 0));
    if (s < 60) return `${s}s`;
    const m = Math.floor(s / 60);
    if (m < 60) return `${m}m`;
    const h = Math.floor(m / 60);
    const rm = m % 60;
    if (h < 48) return rm ? `${h}h ${rm}m` : `${h}h`;
    return `${Math.floor(h / 24)}d`;
}

function fmtProcessUptime(secs) {
    const s = Number(secs) || 0;
    if (s <= 0) return '';
    return ` · ${fmtUptimeSecs(s)}`;
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

function renderOpsHealth({ version, insights, sched, deliveries, agents, live, redmine, sessionFiles }) {
    const enabled = (agents || []).filter((a) => a.enabled).length;
    const sessionN = (sessionFiles || []).length;
    setText(
        'ops-health-version',
        version ? `v${version}${fmtProcessUptime(insights?.process_uptime_secs)}` : '—'
    );
    const agentsHint = document.getElementById('ops-health-version');
    if (agentsHint) {
        if (version) {
            const sessLabel =
                sessionN >= 40 ? `${sessionN}+ session files` : `${sessionN} session file${sessionN === 1 ? '' : 's'}`;
            const up = Number(insights?.process_uptime_secs) || 0;
            const upBit = up > 0 ? ` · up ${fmtUptimeSecs(up)}` : '';
            agentsHint.title = `${enabled}/${(agents || []).length} agents · ${(live || []).length} live · ${sessLabel}${upBit}`;
        }
        const card = agentsHint.closest('.ops-health-card');
        if (card) {
            card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
            if (!version) {
                card.classList.add('ops-health-bad');
            } else if (enabled === 0 || sessionN >= 40) {
                card.classList.add('ops-health-warn');
            } else {
                card.classList.add('ops-health-ok');
            }
        }
    }
    wireOpsHealthCardNavigation();

    const parsedDiscord = parseOpsDiscordGateway(insights?.discord_gateway);
    setText('ops-health-discord', parsedDiscord.healthText);
    const discordEl = document.getElementById('ops-health-discord');
    if (discordEl) {
        discordEl.title = parsedDiscord.dg || '';
        const card = discordEl.closest('.ops-health-card');
        if (card) {
            card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
            if (parsedDiscord.wash === 'offline') {
                card.classList.add('ops-health-bad');
            } else if (parsedDiscord.wash === 'warn') {
                card.classList.add('ops-health-warn');
            } else if (parsedDiscord.wash === 'ready') {
                card.classList.add('ops-health-ok');
            }
        }
    }
    if (insights?.discord_gateway != null) {
        opsRunsInsightsCache = {
            ...(opsRunsInsightsCache || {}),
            discord_gateway: insights.discord_gateway,
        };
    }
    syncOpsCollapsedGlance();

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
    const scheduleEl = document.getElementById('ops-health-schedule');
    if (scheduleEl) {
        const card = scheduleEl.closest('.ops-health-card');
        if (card) {
            card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
            if (sched && sched.totalEntries != null) {
                if (Number(sched.totalEntries) === 0) {
                    card.classList.add('ops-health-warn');
                } else if (sched.secondsUntilNextFire != null) {
                    card.classList.add('ops-health-ok');
                } else {
                    card.classList.add('ops-health-warn');
                }
            }
        }
    }

    let deliveryText = '—';
    let deliveryAgeMs = NaN;
    if (Array.isArray(deliveries) && deliveries.length) {
        const newest = deliveries[0];
        const t = newest?.utc ? Date.parse(newest.utc) : NaN;
        deliveryAgeMs = !Number.isNaN(t) ? Date.now() - t : NaN;
        deliveryText = !Number.isNaN(t) ? fmtAge(t) : (newest.utc || '—');
    }
    setText('ops-health-delivery', deliveryText);
    const deliveryEl = document.getElementById('ops-health-delivery');
    if (deliveryEl) {
        const card = deliveryEl.closest('.ops-health-card');
        if (card) {
            card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
            if (Array.isArray(deliveries) && deliveries.length) {
                const dayMs = 24 * 60 * 60 * 1000;
                if (!Number.isNaN(deliveryAgeMs) && deliveryAgeMs >= 0 && deliveryAgeMs < dayMs) {
                    card.classList.add('ops-health-ok');
                } else if (!Number.isNaN(deliveryAgeMs) && deliveryAgeMs >= 7 * dayMs) {
                    card.classList.add('ops-health-bad');
                } else {
                    card.classList.add('ops-health-warn');
                }
            } else if (Array.isArray(deliveries)) {
                card.classList.add('ops-health-warn');
            }
        }
    }

    let digestText = '—';
    if (insights) {
        const open = insights.digest_open_count ?? 0;
        const stale = insights.digest_stale_count ?? 0;
        let age = '';
        if (insights.digest_generated_at) {
            const t = Date.parse(insights.digest_generated_at);
            if (!Number.isNaN(t)) age = ` · ${fmtAge(t)}`;
        }
        let p50 = '';
        const p50Ms = Number(insights.p50_ms);
        const latSample = Number(insights.latency_sample);
        const hasLatency =
            insights.turns > 0
            && Number.isFinite(p50Ms)
            && p50Ms > 0
            && (Number.isNaN(latSample) || latSample > 0);
        if (hasLatency) {
            p50 =
                p50Ms >= 1000
                    ? ` · p50 ${(p50Ms / 1000).toFixed(1)}s`
                    : ` · p50 ${Math.round(p50Ms)}ms`;
        }
        let fails = '';
        const failN = Number(insights.fail_count) || 0;
        if (failN > 0) fails = ` · ${failN} fail`;
        digestText = `${open} open / ${stale} stale${p50}${fails}${age}`;
    }
    setText('ops-health-digest', digestText);
    const digestEl = document.getElementById('ops-health-digest');
    if (digestEl) {
        const openN = insights?.digest_open_count ?? 0;
        const hints = insights?.digest_open_hints || [];
        const latBits = [];
        if (insights?.turns > 0) {
            if (Number(insights.latency_sample) > 0 || (insights.p50_ms > 0 && insights.latency_sample == null)) {
                if (insights.p50_ms != null) latBits.push(`p50 ${insights.p50_ms} ms`);
                if (insights.mean_ms != null) latBits.push(`mean ${insights.mean_ms} ms`);
                if (insights.max_ms != null) latBits.push(`max ${insights.max_ms} ms`);
            } else {
                latBits.push('p50 n/a (noise filtered)');
            }
            if (insights.latency_sample != null) {
                latBits.push(`latency ${insights.latency_sample}/${insights.turns}`);
            } else {
                latBits.push(`${insights.turns} turns`);
            }
            const lanes = (insights.by_lane || [])
                .map((pair) => (Array.isArray(pair) ? `${pair[0]}:${pair[1]}` : String(pair)))
                .join(' · ');
            if (lanes) latBits.push(lanes);
        }
        const hintLines = hints.length ? hints.slice(0, 5) : [];
        digestEl.title = [...latBits, ...hintLines].join('\n')
            || (insights?.digest_generated_at ? `Generated ${insights.digest_generated_at}` : '');
        const card = digestEl.closest('.ops-health-card');
        if (card) {
            card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
            const failN = Number(insights?.fail_count) || 0;
            if (openN > 0 || failN > 0) card.classList.add('ops-health-warn');
            else if (insights) card.classList.add('ops-health-ok');
        }
    }
}

/** Open Runs + preview a digest-open hint (Slowest/Candidates parity). */
function openOpsDigestHintPreviewNavigate(hint) {
    const text = String(hint || '').trim();
    if (!text) return false;
    if (agentOpsCollapsed) applyOpsCollapsed(false);
    selectOpsTab('runs');
    const card = document.getElementById('ops-runs-insights');
    let line = null;
    card?.querySelectorAll('.ops-insight-line[data-digest-hint]').forEach((el) => {
        if (line) return;
        if (String(el.dataset.digestHint || '') === text) line = el;
    });
    previewOpsRunFromInsight(formatOpsDigestHintAsSummary(text), line);
    if (line) {
        try {
            line.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        } catch (_) {
            /* ignore */
        }
    }
    return true;
}

/** Open Schedules + select/preview a schedule (overview / health parity). */
function openOpsSchedulePreviewNavigate(s) {
    if (!s) return false;
    if (agentOpsCollapsed) applyOpsCollapsed(false);
    selectOpsTab('schedules');
    setOpsSchedulesKindFilter('jobs');
    const id = s.id || '(no id)';
    const fullTask = String(s?.task || '').trim();
    showOpsSchedulePreview(formatOpsSchedulePreview(s), s.id || '', fullTask);
    const list = document.getElementById('ops-schedules-list');
    const delList = document.getElementById('ops-deliveries-list');
    list?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
    delList?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
    list?.querySelectorAll('.ops-row').forEach((row) => {
        const title = row.querySelector('.ops-row-title');
        if (title && title.textContent === id) {
            row.classList.add('is-selected');
            row.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
        }
    });
    return true;
}

/** Open Schedules + select/preview a delivery (overview / health parity). */
function openOpsDeliveryPreviewNavigate(d) {
    if (!d) return false;
    if (agentOpsCollapsed) applyOpsCollapsed(false);
    selectOpsTab('schedules');
    setOpsSchedulesKindFilter('deliveries');
    const id = d.schedule_id || 'schedule';
    const summary = String(d.summary || '').trim();
    showOpsSchedulePreview(formatOpsDeliveryPreview(d), d.schedule_id || '', summary);
    const list = document.getElementById('ops-schedules-list');
    const delList = document.getElementById('ops-deliveries-list');
    list?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
    delList?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
    let matched = false;
    delList?.querySelectorAll('.ops-row').forEach((row) => {
        if (matched) return;
        const title = row.querySelector('.ops-row-title');
        if (title && title.textContent === id) {
            matched = true;
            row.classList.add('is-selected');
            row.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
        }
    });
    return true;
}

/** Soonest schedule from cache (for health Next schedule preview). */
function findOpsNextSchedule() {
    const rows = opsSchedulesCache || [];
    if (!rows.length) return null;
    let best = null;
    let bestTs = Infinity;
    rows.forEach((s) => {
        const raw = s?.next_run || s?.nextRun || '';
        const t = raw ? Date.parse(raw) : NaN;
        if (!Number.isNaN(t) && t < bestTs) {
            bestTs = t;
            best = s;
        }
    });
    return best || rows[0];
}

/** Prefer enabled orchestrator, else first enabled, else first agent (health Version). */
function findOpsPrimaryAgent() {
    const rows = opsAgentsCache || [];
    if (!rows.length) return null;
    const orch = rows.find((a) => a.enabled && a.orchestrator);
    if (orch) return orch;
    const on = rows.find((a) => a.enabled);
    if (on) return on;
    return rows[0];
}

/** Prefer slug/name/id “redmine” for health Redmine (Version uses primary). */
function findOpsRedmineAgent() {
    const rows = opsAgentsCache || [];
    if (!rows.length) return null;
    const match = (a) => {
        const slug = String(a?.slug || '').toLowerCase();
        const id = String(a?.id || '').toLowerCase();
        const name = String(a?.name || '').toLowerCase();
        return slug === 'redmine' || name === 'redmine' || id.includes('redmine');
    };
    const on = rows.find((a) => a.enabled && match(a));
    if (on) return on;
    return rows.find(match) || null;
}

/** Open Agents + select/open a row (health Version parity with schedule/delivery). */
function openOpsAgentPreviewNavigate(a) {
    if (!a || !(a.id || a.slug)) return false;
    if (agentOpsCollapsed) applyOpsCollapsed(false);
    selectOpsTab('agents');
    const list = document.getElementById('ops-agents-list');
    const detail = document.getElementById('ops-agent-detail');
    if (detail && !detail.hidden && list?.style.display === 'none') {
        /* keep detail open; openOpsAgent replaces contents */
    } else if (list) {
        list.style.display = '';
    }
    const id = String(a.id || '');
    const slug = String(a.slug || a.id || '');
    const name = String(a.name || '');
    list?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
    let matched = null;
    list?.querySelectorAll('.ops-row').forEach((row) => {
        if (matched) return;
        const title = row.querySelector('.ops-row-title');
        const text = title ? title.textContent || '' : '';
        if (
            (slug && text.includes(slug)) ||
            (name && text.includes(name)) ||
            (id && text.includes(id))
        ) {
            matched = row;
            row.classList.add('is-selected');
            try {
                row.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            } catch (_) {
                /* ignore */
            }
        }
    });
    const selector = id || slug;
    if (selector) void openOpsAgent(selector);
    return true;
}

/** Click overview cards (chrome / empty body) to open the linked tab — health-card parity. */
function wireOpsOverviewCardNavigation() {
    const grid = document.getElementById('ops-overview-grid');
    if (!grid || grid.dataset.opsOverviewNav === '1') return;
    grid.dataset.opsOverviewNav = '1';
    grid.querySelectorAll('.ops-overview-card').forEach((card) => {
        const link = card.querySelector('.ops-overview-link[data-goto-tab]');
        const tab = link?.dataset?.gotoTab || '';
        if (!tab) return;
        card.classList.add('ops-overview-clickable');
        card.setAttribute('role', 'button');
        if (!card.getAttribute('aria-label')) {
            card.setAttribute('aria-label', `Open ${tab} tab`);
        }
        const openTab = () => {
            if (agentOpsCollapsed) applyOpsCollapsed(false);
            if (tab === 'agents') preferOpsAgentsEnabledFromOverview();
            if (tab === 'runs') preferOpsRunsLaneFromOverview();
            if (tab === 'schedules') preferOpsSchedulesKindFromOverview();
            if (tab === 'memory') preferOpsMemoryKindFromOverview();
            selectOpsTab(tab);
        };
        card.addEventListener('click', (e) => {
            if (
                e.target.closest(
                    '.ops-row, .ops-overview-link, .ops-clear-filter, button, a, input, textarea, select'
                )
            ) {
                return;
            }
            openTab();
        });
        card.addEventListener('keydown', (e) => {
            if (e.key !== 'Enter' && e.key !== ' ') return;
            if (e.target !== card) return;
            e.preventDefault();
            openTab();
        });
    });
    ensureOpsOverviewToolbarKeyboard();
}

/** Focusable overview cards in DOM order (Agents · Schedules · Live · Knowledge · Recent · Runs · Digest). */
function getOpsOverviewCards() {
    const grid = document.getElementById('ops-overview-grid');
    if (!grid) return [];
    return Array.from(grid.querySelectorAll(':scope > .ops-overview-card.ops-overview-clickable')).filter(
        (el) => {
            if (!el || el.hidden) return false;
            return el.getClientRects().length > 0 || el.offsetParent !== null || grid.contains(el);
        }
    );
}

function refreshOpsOverviewRovingTabindex(preferred) {
    const cards = getOpsOverviewCards();
    if (!cards.length) return;
    const focused = cards.find((el) => el === document.activeElement);
    const current =
        (preferred && cards.includes(preferred) && preferred) ||
        focused ||
        cards.find((el) => el.tabIndex === 0) ||
        cards[0];
    for (const el of cards) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function ensureOpsOverviewKbHint() {
    const grid = document.getElementById('ops-overview-grid');
    if (!grid) return;
    let hint = document.getElementById('ops-overview-kb-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-overview-kb-hint';
        hint.className = 'ops-overview-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        grid.appendChild(hint);
    }
    hint.textContent =
        '← → / h l · Home/End move · Enter / Space opens linked tab';
}

/**
 * Overview-card toolbar keyboard — focus Agents · Schedules · Live · Knowledge ·
 * Recent · Runs · Digest, then ←→ / h l / Home/End (health-strip / power-strip parity).
 * Enter/Space keep existing card activate.
 */
function ensureOpsOverviewToolbarKeyboard() {
    const grid = document.getElementById('ops-overview-grid');
    if (!grid) return;
    ensureOpsOverviewKbHint();
    refreshOpsOverviewRovingTabindex();
    if (grid.dataset.opsOverviewKbWired === '1') return;
    grid.dataset.opsOverviewKbWired = '1';
    if (!grid.getAttribute('role')) {
        grid.setAttribute('role', 'toolbar');
    }
    if (!grid.getAttribute('aria-label')) {
        grid.setAttribute('aria-label', 'Agent Ops overview cards');
    }
    grid.addEventListener('focusin', (e) => {
        const cards = getOpsOverviewCards();
        if (cards.includes(e.target)) refreshOpsOverviewRovingTabindex(e.target);
    });
    grid.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const cards = getOpsOverviewCards();
        if (!cards.length) return;
        const idx = cards.indexOf(document.activeElement);
        if (idx < 0) return;
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, cards.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = cards.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsOverviewRovingTabindex(cards[next]);
        cards[next].focus();
    });
}

/** Focusable health cards in DOM order (Version · Discord · Redmine · Schedule · Delivery · Digest). */
function getOpsHealthCards() {
    const row = document.getElementById('ops-health-row');
    if (!row) return [];
    return Array.from(row.querySelectorAll(':scope > .ops-health-card[data-health]')).filter(
        (el) => {
            if (!el || el.hidden) return false;
            return el.getClientRects().length > 0 || el.offsetParent !== null || row.contains(el);
        }
    );
}

function refreshOpsHealthRovingTabindex(preferred) {
    const cards = getOpsHealthCards();
    if (!cards.length) return;
    const focused = cards.find((el) => el === document.activeElement);
    const current =
        (preferred && cards.includes(preferred) && preferred) ||
        focused ||
        cards.find((el) => el.tabIndex === 0) ||
        cards[0];
    for (const el of cards) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function ensureOpsHealthKbHint() {
    const row = document.getElementById('ops-health-row');
    if (!row) return;
    let hint = document.getElementById('ops-health-kb-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-health-kb-hint';
        hint.className = 'ops-health-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        row.appendChild(hint);
    }
    hint.textContent =
        '← → / h l · Home/End move · Enter / Space opens linked tab or preview';
}

/**
 * Health-strip toolbar keyboard — focus Version · Discord · Redmine · Schedule ·
 * Delivery · Digest, then ←→ / h l / Home/End (Disk Cleanup meta-card / power-strip parity).
 * Enter/Space keep existing card activate.
 */
function ensureOpsHealthToolbarKeyboard() {
    const row = document.getElementById('ops-health-row');
    if (!row) return;
    ensureOpsHealthKbHint();
    refreshOpsHealthRovingTabindex();
    if (row.dataset.opsHealthKbWired === '1') return;
    row.dataset.opsHealthKbWired = '1';
    if (!row.getAttribute('role')) {
        row.setAttribute('role', 'toolbar');
    }
    if (!row.getAttribute('aria-label')) {
        row.setAttribute('aria-label', 'Agent Ops health summary');
    }
    row.addEventListener('focusin', (e) => {
        const cards = getOpsHealthCards();
        if (cards.includes(e.target)) refreshOpsHealthRovingTabindex(e.target);
    });
    row.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const cards = getOpsHealthCards();
        if (!cards.length) return;
        const idx = cards.indexOf(document.activeElement);
        if (idx < 0) return;
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, cards.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = cards.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsHealthRovingTabindex(cards[next]);
        cards[next].focus();
    });
}

/** Click health cards to jump to the related Agent Ops tab (once). */
function wireOpsHealthCardNavigation() {
    const row = document.getElementById('ops-health-row');
    if (!row || row.dataset.opsHealthNav === '1') return;
    row.dataset.opsHealthNav = '1';
    row.querySelectorAll('.ops-health-card[data-health]').forEach((card) => {
        const key = card.dataset.health;
        const tab = OPS_HEALTH_TAB_BY_KEY[key];
        if (!tab) return;
        card.dataset.gotoTab = tab;
        card.classList.add('ops-health-clickable');
        card.setAttribute('role', 'button');
        if (key === 'schedule') {
            card.title = 'Open Schedules · preview next job · load into AI Chat from that tab';
        } else if (key === 'delivery') {
            card.title =
                'Open Schedules · preview last delivery · load into AI Chat from that tab';
        } else if (key === 'digest') {
            card.title =
                'Open Runs · preview first digest-open hint · load into AI Chat from that tab';
        } else if (key === 'version') {
            card.title =
                'Open Agents · open primary agent · load soul/skill/mood into AI Chat from that tab';
        } else if (key === 'discord') {
            card.title =
                'Open Runs · preview Discord gateway status · load into AI Chat from that tab';
        } else if (key === 'redmine') {
            card.title =
                'Open Agents · open Redmine agent · load soul/skill/mood into AI Chat from that tab';
        } else {
            card.title = card.title || `Open ${tab}`;
        }
        const openTab = () => {
            if (agentOpsCollapsed) applyOpsCollapsed(false);
            if (key === 'schedule') {
                const next = findOpsNextSchedule();
                if (next && openOpsSchedulePreviewNavigate(next)) return;
            }
            if (key === 'delivery') {
                const newest =
                    Array.isArray(opsDeliveriesCache) && opsDeliveriesCache.length
                        ? opsDeliveriesCache[0]
                        : null;
                if (newest && openOpsDeliveryPreviewNavigate(newest)) return;
            }
            if (key === 'digest') {
                const hints = opsRunsInsightsCache?.digest_open_hints || [];
                const first = hints.length ? String(hints[0] || '').trim() : '';
                if (first && openOpsDigestHintPreviewNavigate(first)) return;
            }
            if (key === 'version') {
                const primary = findOpsPrimaryAgent();
                if (primary && openOpsAgentPreviewNavigate(primary)) return;
            }
            if (key === 'discord') {
                const gw = String(opsRunsInsightsCache?.discord_gateway || '').trim();
                if (gw && openOpsDiscordGatewayPreviewNavigate(gw)) return;
            }
            if (key === 'redmine') {
                const redmineAgent = findOpsRedmineAgent();
                if (redmineAgent && openOpsAgentPreviewNavigate(redmineAgent)) return;
            }
            selectOpsTab(tab);
        };
        card.addEventListener('click', openTab);
        card.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                openTab();
            }
        });
    });
    syncOpsHealthCardActive(opsActiveTab);
    ensureOpsHealthToolbarKeyboard();
}

/** Overview Agents card: ok/warn/bad wash (health Version agent-count parity). */
function setOverviewAgentsStatus(agents) {
    const card = document.getElementById('ops-overview-agents');
    if (!card) return;
    card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
    const rows = Array.isArray(agents) ? agents : [];
    if (!rows.length) {
        card.classList.add('ops-health-warn');
        card.title = 'No agents yet';
        return;
    }
    const enabledN = rows.filter((a) => a.enabled).length;
    if (enabledN === 0) {
        card.classList.add('ops-health-warn');
        card.title = 'Agents present but none enabled';
        return;
    }
    const hasOrch = rows.some((a) => a.enabled && a.orchestrator);
    card.classList.add('ops-health-ok');
    card.title = hasOrch
        ? `${enabledN}/${rows.length} agents on · orchestrator ready`
        : `${enabledN}/${rows.length} agents on`;
}

/** Overview Agents card: enabled/orchestrator first; click → Agents + Load into AI Chat. */
function renderOverviewAgents(agents) {
    ensureOpsOverviewAgentsCard();
    const body = document.getElementById('ops-overview-agents-body');
    if (!body) return;
    body.innerHTML = '';
    const rows = Array.isArray(agents) ? agents.slice() : [];
    setOverviewAgentsStatus(rows);
    if (!rows.length) {
        paintOpsOverviewHeadCount('ops-overview-agents', '0', { zero: true });
        body.innerHTML = opsOverviewEmptyHtml(
            'No agents yet — add folders under ~/.mac-stats/agents',
            'agents',
            'Open Agents'
        );
        return;
    }
    rows.sort((a, b) => {
        const ae = a.enabled ? 1 : 0;
        const be = b.enabled ? 1 : 0;
        if (be !== ae) return be - ae;
        const ao = a.orchestrator ? 1 : 0;
        const bo = b.orchestrator ? 1 : 0;
        if (bo !== ao) return bo - ao;
        return String(a.name || '').localeCompare(String(b.name || ''));
    });
    const enabledN = rows.filter((a) => a.enabled).length;
    paintOpsOverviewHeadCount(
        'ops-overview-agents',
        `${enabledN}/${rows.length} on`,
        { zero: enabledN === 0 }
    );
    rows.slice(0, 4).forEach((a) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const slug = a.slug || a.id || '';
        const metaBits = [
            a.model || 'default model',
            a.orchestrator ? 'orchestrator' : '',
            a.enabled ? 'on' : 'off',
        ].filter(Boolean);
        btn.innerHTML =
            `<div><div class="ops-row-title">${escapeHtml(a.name || slug)}` +
            (slug ? ` <span class="ops-row-meta">· ${escapeHtml(slug)}</span>` : '') +
            `</div><div class="ops-row-meta">${escapeHtml(metaBits.join(' · '))}</div></div>` +
            `<span class="ops-badge ${a.enabled ? '' : 'off'}">${a.enabled ? 'on' : 'off'}</span>`;
        btn.addEventListener('click', () => {
            body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            openOpsAgentPreviewNavigate(a);
        });
        btn.title = 'Open in Agents · load soul/skill/mood into AI Chat from that tab';
        body.appendChild(btn);
    });
}

/** Open Runs + preview a turn (overview / Insights Slowest parity). */
function openOpsRunPreviewNavigate(summary) {
    if (!summary) return false;
    if (agentOpsCollapsed) applyOpsCollapsed(false);
    const lane = String(summary?.lane || '').toLowerCase();
    if (lane === 'instant' || lane === 'direct') {
        setOpsRunsLaneFilter(lane);
    }
    selectOpsTab('runs');
    previewOpsRunFromInsight(summary, null);
    return true;
}

/** Overview Runs card: ok/warn/bad wash (health Digest fail/open parity). */
function setOverviewRunsStatus(insights) {
    const card = document.getElementById('ops-overview-runs');
    if (!card) return;
    card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
    const recent = Array.isArray(insights?.recent) ? insights.recent : [];
    const failN = Number(insights?.fail_count) || 0;
    const openN = Number(insights?.digest_open_count) || 0;
    if (!insights || (!recent.length && !(Number(insights?.turns) > 0))) {
        card.classList.add('ops-health-warn');
        card.title = 'No runs yet';
        return;
    }
    if (failN > 0) {
        card.classList.add('ops-health-warn');
        card.title = `${failN} failed turn${failN === 1 ? '' : 's'} in the window`;
        return;
    }
    if (openN > 0) {
        card.classList.add('ops-health-warn');
        card.title = `${openN} digester open candidate${openN === 1 ? '' : 's'}`;
        return;
    }
    card.classList.add('ops-health-ok');
    const turns = Number(insights?.turns) || recent.length;
    card.title = turns > 0 ? `${turns} turn${turns === 1 ? '' : 's'} · no fails` : 'Runs healthy';
}

/** Overview Runs card: recent turns snapshot; click → Runs + Load into AI Chat. */
function renderOverviewRuns(insights) {
    ensureOpsOverviewRunsCard();
    const body = document.getElementById('ops-overview-runs-body');
    if (!body) return;
    body.innerHTML = '';
    const recent = Array.isArray(insights?.recent) ? insights.recent : [];
    setOverviewRunsStatus(insights);
    if (!recent.length) {
        paintOpsOverviewHeadCount('ops-overview-runs', '0', { zero: true });
        body.innerHTML = opsOverviewEmptyHtml(
            'No runs yet — turns land after Discord or chat',
            'runs',
            'Open Runs'
        );
        return;
    }
    const turns = Number(insights?.turns) || recent.length;
    const okN = Number(insights?.ok_count);
    const failN = Number(insights?.fail_count) || 0;
    const okLabel = Number.isFinite(okN) ? okN : recent.filter((r) => r.ok).length;
    paintOpsOverviewHeadCount(
        'ops-overview-runs',
        failN > 0 ? `${okLabel}/${turns} ok · ${failN} fail` : `${okLabel}/${turns} ok`,
        { zero: turns === 0 }
    );
    recent.slice(0, 3).forEach((r) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const q = String(r?.question_preview || '').trim() || '(empty)';
        const toolsJoined = (r?.tools || []).slice(0, 3).join(', ') || '—';
        const wall = typeof r?.wall_ms === 'number' ? `${r.wall_ms} ms` : '—';
        btn.innerHTML =
            `<div><div class="ops-row-title">${escapeHtml(q.slice(0, 72))}</div>` +
            `<div class="ops-row-meta">${escapeHtml(r?.lane || '—')} · ${escapeHtml(wall)} · ${escapeHtml(toolsJoined)}` +
            `${r?.ok === false ? ' · FAIL' : ''}</div></div>`;
        btn.addEventListener('click', () => {
            body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            openOpsRunPreviewNavigate(r);
        });
        btn.title =
            'Open in Runs · preview question/tools · load into AI Chat from that tab';
        body.appendChild(btn);
    });
}

/** Overview Digest card: ok/warn/bad wash (health Digest fail/open parity). */
function setOverviewDigestStatus(insights) {
    const card = document.getElementById('ops-overview-digest');
    if (!card) return;
    card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
    if (!insights) {
        card.classList.add('ops-health-warn');
        card.title = 'Digest not loaded yet';
        return;
    }
    const openN = Number(insights.digest_open_count) || 0;
    const failN = Number(insights.fail_count) || 0;
    const staleN = Number(insights.digest_stale_count) || 0;
    if (openN > 0 || failN > 0) {
        card.classList.add('ops-health-warn');
        const bits = [];
        if (openN > 0) bits.push(`${openN} open candidate${openN === 1 ? '' : 's'}`);
        if (failN > 0) bits.push(`${failN} fail${failN === 1 ? '' : 's'}`);
        card.title = bits.join(' · ');
        return;
    }
    card.classList.add('ops-health-ok');
    card.title =
        staleN > 0
            ? `Digest clear · ${staleN} stale ignored`
            : 'Digest clear — no open candidates';
}

/** Overview Digest card: digester open hints; click → Runs + Load into AI Chat. */
function renderOverviewDigest(insights) {
    ensureOpsOverviewDigestCard();
    const body = document.getElementById('ops-overview-digest-body');
    if (!body) return;
    body.innerHTML = '';
    setOverviewDigestStatus(insights);
    const hints = (insights?.digest_open_hints || [])
        .map((h) => String(h || '').trim())
        .filter(Boolean);
    const openN = Number(insights?.digest_open_count);
    const staleN = Number(insights?.digest_stale_count) || 0;
    const openLabel = Number.isFinite(openN) ? openN : hints.length;
    if (!hints.length && openLabel === 0) {
        paintOpsOverviewHeadCount('ops-overview-digest', '0 open', { zero: true });
        body.innerHTML = opsOverviewEmptyHtml(
            'No open digester candidates — overnight is quiet for now',
            'runs',
            'Open Runs'
        );
        return;
    }
    if (!hints.length && openLabel > 0) {
        paintOpsOverviewHeadCount(
            'ops-overview-digest',
            staleN > 0 ? `${openLabel} open · ${staleN} stale` : `${openLabel} open`
        );
        body.innerHTML = opsOverviewEmptyHtml(
            `${openLabel} open in digest — open Runs Insights for the list`,
            'runs',
            'Open Runs'
        );
        return;
    }
    paintOpsOverviewHeadCount(
        'ops-overview-digest',
        staleN > 0 ? `${openLabel} open · ${staleN} stale` : `${openLabel} open`,
        { zero: openLabel === 0 }
    );
    hints.slice(0, 4).forEach((text) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const title = text.length > 72 ? `${text.slice(0, 72)}…` : text;
        btn.innerHTML =
            `<div><div class="ops-row-title">${escapeHtml(title)}</div>` +
            `<div class="ops-row-meta">Digester open candidate</div></div>`;
        btn.addEventListener('click', () => {
            body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            openOpsDigestHintPreviewNavigate(text);
        });
        btn.title =
            'Open in Runs · preview digester hint · load into AI Chat from that tab';
        body.appendChild(btn);
    });
}

/** Overview Schedules card: ok/warn/bad wash (health schedule + delivery parity). */
function setOverviewSchedulesStatus(schedules, deliveries) {
    const card = document.getElementById('ops-overview-schedules');
    if (!card) return;
    card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
    if (!schedules || !schedules.length) {
        card.classList.add('ops-health-warn');
        card.title = 'No schedules yet';
        return;
    }
    let deliveryAgeMs = NaN;
    if (Array.isArray(deliveries) && deliveries.length) {
        const t = deliveries[0]?.utc ? Date.parse(deliveries[0].utc) : NaN;
        deliveryAgeMs = !Number.isNaN(t) ? Date.now() - t : NaN;
    }
    const dayMs = 24 * 60 * 60 * 1000;
    const hasNext = schedules.some(
        (s) => s.next_run || s.nextRun || s.secondsUntilNextFire != null
    );
    if (!Number.isNaN(deliveryAgeMs) && deliveryAgeMs >= 7 * dayMs) {
        card.classList.add('ops-health-bad');
        card.title = 'Last delivery is a week or older';
        return;
    }
    if (hasNext && !Number.isNaN(deliveryAgeMs) && deliveryAgeMs >= 0 && deliveryAgeMs < dayMs) {
        card.classList.add('ops-health-ok');
        card.title = 'Schedules armed · last delivery under 24h';
        return;
    }
    card.classList.add('ops-health-warn');
    if (!hasNext) {
        card.title = 'Schedules present but no next fire time';
    } else if (!Array.isArray(deliveries) || !deliveries.length) {
        card.title = 'Schedules armed · no deliveries yet';
    } else {
        card.title = 'Schedules armed · last delivery older than 24h';
    }
}

function renderOverviewSchedules(schedules, deliveries) {
    const body = document.getElementById('ops-overview-schedules-body');
    if (!body) return;
    body.innerHTML = '';
    setOverviewSchedulesStatus(schedules, deliveries);
    if (!schedules || !schedules.length) {
        paintOpsOverviewHeadCount('ops-overview-schedules', '0', { zero: true });
        body.innerHTML = opsOverviewEmptyHtml(
          'No schedules yet — add one on the Schedules tab',
          'schedules',
          'Open Schedules'
        );
        return;
    }
    paintOpsOverviewHeadCount(
        'ops-overview-schedules',
        `${schedules.length} active`,
        { zero: false }
    );
    schedules.slice(0, 3).forEach((s) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const id = s.id || '(no id)';
        const next = s.next_run || s.nextRun || '—';
        const task = String(s.task || '').slice(0, 40);
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(id)}</div><div class="ops-row-meta">next ${escapeHtml(next)} · ${escapeHtml(task)}</div></div>`;
        btn.addEventListener('click', () => {
            body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            openOpsSchedulePreviewNavigate(s);
        });
        btn.title = 'Open in Schedules · preview task · load into AI Chat from that tab';
        body.appendChild(btn);
    });
    if (Array.isArray(deliveries) && deliveries.length) {
        appendOverviewLastDeliveryRow(body, deliveries[0]);
    }
}

/** Overview Schedules: last delivery as a clickable row (schedule-row preview parity). */
function appendOverviewLastDeliveryRow(body, d) {
    if (!body || !d) return;
    const t = d.utc ? Date.parse(d.utc) : NaN;
    const id = d.schedule_id || 'schedule';
    const ageLabel = !Number.isNaN(t) ? fmtAge(t) : '';
    const summary = String(d.summary || '').trim();
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'ops-row';
    const metaBits = [id, ageLabel, summary.slice(0, 40)].filter(Boolean);
    btn.innerHTML =
        `<div><div class="ops-row-title">Last delivery</div>` +
        `<div class="ops-row-meta">${escapeHtml(metaBits.join(' · '))}${summary.length > 40 ? '…' : ''}</div></div>`;
    btn.addEventListener('click', () => {
        body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
        btn.classList.add('is-selected');
        openOpsDeliveryPreviewNavigate(d);
    });
    btn.title = 'Open in Schedules · preview last delivery · load into AI Chat from that tab';
    body.appendChild(btn);
}

/** Overview Live card: ok/warn/bad wash (health Discord gateway parity). */
function setOverviewLiveStatus(rows, insights) {
    const card = document.getElementById('ops-overview-live');
    if (!card) return;
    card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
    const live = Array.isArray(rows) ? rows : [];
    const dg = insights?.discord_gateway || '';
    const stageMatch = dg.match(/stage=([^\s·]+)/i);
    const stage = (stageMatch ? stageMatch[1] : '').trim().toLowerCase();
    const discMatch = dg.match(/disconnect×(\d+)/i);
    const discN = discMatch ? Number(discMatch[1]) : 0;
    const resumeMatch = dg.match(/resume×(\d+)/i);
    const resumeN = resumeMatch ? Number(resumeMatch[1]) : 0;
    const readyMatch = /last Ready/i.test(dg);
    const liveLabel =
        live.length === 1 ? '1 live session' : `${live.length} live sessions`;

    if (stage === 'disconnected') {
        card.classList.add('ops-health-bad');
        card.title = live.length
            ? `Discord disconnected · ${liveLabel} still listed`
            : 'Discord disconnected · no live sessions';
        return;
    }
    if (discN > 0 || resumeN > 0 || stage === 'resuming') {
        card.classList.add('ops-health-warn');
        card.title = live.length
            ? `Gateway reconnect noise · ${liveLabel}`
            : 'Gateway reconnect noise · no live sessions';
        return;
    }
    if (!live.length) {
        card.classList.add('ops-health-warn');
        card.title =
            stage === 'connected' || readyMatch
                ? 'No live sessions — gateway ready'
                : 'No live sessions';
        return;
    }
    card.classList.add('ops-health-ok');
    card.title =
        stage === 'connected' || readyMatch
            ? `${liveLabel} · gateway ready`
            : liveLabel;
}

function renderOverviewLive(rows, insights) {
    const body = document.getElementById('ops-overview-live-body');
    if (!body) return;
    body.innerHTML = '';
    setOverviewLiveStatus(rows, insights);
    if (!rows || !rows.length) {
        paintOpsOverviewHeadCount('ops-overview-live', '0', { zero: true });
        body.innerHTML = opsOverviewEmptyHtml(
          'No live sessions — chats appear here while agents run',
          'sessions',
          'Open Sessions'
        );
        return;
    }
    paintOpsOverviewHeadCount('ops-overview-live', `${rows.length} live`);
    rows.slice(0, 3).forEach((r) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(r.source)} · ${r.session_id}</div><div class="ops-row-meta">${r.message_count} msgs${r.preview ? ` · ${escapeHtml(r.preview)}` : ''}</div></div>`;
        btn.addEventListener('click', async () => {
            body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            selectOpsTab('sessions');
            setOpsSessionKindFilter('live');
            const matchTitle = `${r.source} · ${r.session_id}`;
            const liveList = document.getElementById('ops-live-sessions');
            const fileList = document.getElementById('ops-session-files');
            fileList?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            liveList?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            let matchedRow = null;
            liveList?.querySelectorAll('.ops-row').forEach((row) => {
                const title = row.querySelector('.ops-row-title');
                if (title && title.textContent === matchTitle) {
                    matchedRow = row;
                    row.classList.add('is-selected');
                    row.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
                }
            });
            try {
                const msgs = await invoke('read_live_session_messages', {
                    source: r.source,
                    sessionId: r.session_id,
                });
                if (matchedRow) markOpsSessionRowSelected(matchedRow);
                showOpsSessionPreview(msgs, `Live ${r.source} · ${r.session_id}`, r.session_id);
                showOpsSessionStatus(
                    'Preview ready — Enter or “Load into AI Chat” · double-click also loads.',
                    true
                );
            } catch (err) {
                showOpsSessionPreview([], String(err), null);
                showOpsSessionStatus(String(err), false);
            }
        });
        btn.title = 'Open in Sessions · preview live chat · load into AI Chat from that tab';
        body.appendChild(btn);
    });
}

/** Overview Knowledge card: ok/warn/bad wash (health Version session-file count parity). */
function setOverviewKnowledgeStatus(files) {
    const card = document.getElementById('ops-overview-knowledge');
    if (!card) return;
    card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
    const rows = Array.isArray(files) ? files : [];
    if (!rows.length) {
        card.classList.add('ops-health-warn');
        card.title = 'No knowledge files yet under ~/.mac-stats';
        return;
    }
    const n = rows.length;
    const fileLabel = n === 1 ? '1 knowledge file' : `${n} knowledge files`;
    if (n >= 40) {
        card.classList.add('ops-health-warn');
        card.title = `${fileLabel} — vault is getting crowded`;
        return;
    }
    card.classList.add('ops-health-ok');
    card.title = fileLabel;
}

function renderOverviewKnowledge(files) {
    const body = document.getElementById('ops-overview-knowledge-body');
    if (!body) return;
    body.innerHTML = '';
    setOverviewKnowledgeStatus(files);
    if (!files || !files.length) {
        paintOpsOverviewHeadCount('ops-overview-knowledge', '0', { zero: true });
        body.innerHTML = opsOverviewEmptyHtml(
          'No knowledge files yet under ~/.mac-stats',
          'memory',
          'Open Knowledge'
        );
        return;
    }
    const sorted = [...files].sort((a, b) => (b.modified_ms || 0) - (a.modified_ms || 0));
    paintOpsOverviewHeadCount('ops-overview-knowledge', `${files.length} files`);
    sorted.slice(0, 4).forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.name)}</div><div class="ops-row-meta">${escapeHtml(f.kind)} · ${fmtAge(f.modified_ms)}</div></div>`;
        btn.addEventListener('click', async () => {
            body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            const kind = String(f.kind || '').toLowerCase();
            setOpsMemoryKindFilter(kind === 'discord' ? 'discord' : 'core');
            selectOpsTab('memory');
            const matchTitle = f.name || '';
            const list = document.getElementById('ops-memory-list');
            list?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            list?.querySelectorAll('.ops-row').forEach((row) => {
                const title = row.querySelector('.ops-row-title');
                if (title && title.textContent === matchTitle) {
                    row.classList.add('is-selected');
                    row.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
                }
            });
            const preview = document.getElementById('ops-memory-preview');
            const copyPath = f.path || f.name || '';
            const label = f.name || f.path || 'knowledge';
            try {
                const text = await invoke('read_memory_file', { path: f.path });
                const bodyText = String(text || '').slice(0, 12000);
                if (preview) {
                    preview.hidden = false;
                    preview.textContent = bodyText;
                }
                setOpsMemoryCopyChip(copyPath);
                const loadBody = bodyText.trim();
                if (loadBody) {
                    opsMemoryLoadText = `Knowledge: ${label}\n\n${loadBody}`;
                    setOpsMemoryLoadChatVisible(true);
                    showOpsMemoryLoadStatus(
                        'Preview ready — Enter or “Load into AI Chat” · double-click also loads.',
                        true
                    );
                } else {
                    opsMemoryLoadText = null;
                    setOpsMemoryLoadChatVisible(false);
                    showOpsMemoryLoadStatus('File is empty.', false);
                }
            } catch (err) {
                if (preview) {
                    preview.hidden = false;
                    preview.textContent = String(err);
                }
                opsMemoryLoadText = null;
                setOpsMemoryCopyChip(null);
                setOpsMemoryLoadChatVisible(false);
                showOpsMemoryLoadStatus(String(err), false);
            }
        });
        btn.title = 'Open in Knowledge · preview file · load into AI Chat from that tab';
        body.appendChild(btn);
    });
}

/** Overview Recent card: ok/warn/bad wash (health Last delivery age parity). */
function setOverviewRecentStatus(files) {
    const card = document.getElementById('ops-overview-recent');
    if (!card) return;
    card.classList.remove('ops-health-ok', 'ops-health-warn', 'ops-health-bad');
    const rows = Array.isArray(files) ? files : [];
    if (!rows.length) {
        card.classList.add('ops-health-warn');
        card.title = 'No recent chats — session memory shows up here';
        return;
    }
    const newestMs = rows.reduce((max, f) => {
        const m = Number(f?.modified_ms) || 0;
        return m > max ? m : max;
    }, 0);
    const n = rows.length;
    const chatLabel = n === 1 ? '1 recent chat' : `${n} recent chats`;
    if (!newestMs) {
        card.classList.add('ops-health-warn');
        card.title = `${chatLabel} — age unknown`;
        return;
    }
    const ageMs = Date.now() - newestMs;
    const dayMs = 24 * 60 * 60 * 1000;
    const ageLabel = fmtAge(newestMs);
    if (ageMs >= 0 && ageMs < dayMs) {
        card.classList.add('ops-health-ok');
        card.title = `${chatLabel} · newest ${ageLabel}`;
        return;
    }
    if (ageMs >= 7 * dayMs) {
        card.classList.add('ops-health-bad');
        card.title = `${chatLabel} — newest is ${ageLabel} (stale)`;
        return;
    }
    card.classList.add('ops-health-warn');
    card.title = `${chatLabel} · newest ${ageLabel}`;
}

function renderOverviewRecent(files) {
    const body = document.getElementById('ops-overview-recent-body');
    if (!body) return;
    body.innerHTML = '';
    setOverviewRecentStatus(files);
    if (!files || !files.length) {
        paintOpsOverviewHeadCount('ops-overview-recent', '0', { zero: true });
        body.innerHTML = opsOverviewEmptyHtml(
          'No recent chats — session memory shows up here',
          'sessions',
          'Open Sessions'
        );
        return;
    }
    paintOpsOverviewHeadCount(
        'ops-overview-recent',
        `${Math.min(5, files.length)} of ${files.length}`
    );
    files.slice(0, 5).forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.slug || f.name)}</div><div class="ops-row-meta">${escapeHtml(f.source_hint)} · ${fmtAge(f.modified_ms)}</div></div>`;
        btn.addEventListener('click', async () => {
            body.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            selectOpsTab('sessions');
            setOpsSessionKindFilter('files');
            const matchTitle = f.slug || f.name || '';
            const list = document.getElementById('ops-session-files');
            const liveList = document.getElementById('ops-live-sessions');
            liveList?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            list?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            let matchedRow = null;
            list?.querySelectorAll('.ops-row').forEach((row) => {
                const title = row.querySelector('.ops-row-title');
                if (title && title.textContent === matchTitle) {
                    matchedRow = row;
                    row.classList.add('is-selected');
                    row.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
                }
            });
            try {
                const msgs = await invoke('read_session_file_messages', { path: f.path });
                const copyId = f.slug || f.name || '';
                if (msgs && msgs.length) {
                    if (matchedRow) markOpsSessionRowSelected(matchedRow);
                    showOpsSessionPreview(msgs, f.name, copyId);
                    showOpsSessionStatus(
                        'Preview ready — Enter or “Load into AI Chat” · double-click also loads.',
                        true
                    );
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
                    setOpsSessionCopyChip(copyId);
                    showOpsSessionStatus('No messages to load — raw file preview only.', false);
                }
            } catch (err) {
                showOpsSessionPreview([], String(err), null);
                showOpsSessionStatus(String(err), false);
            }
        });
        btn.title = 'Open in Sessions · preview chat · load into AI Chat from that tab';
        body.appendChild(btn);
    });
}

/** Show full schedule or delivery text (rows truncate task/summary). */
function showOpsSchedulePreview(text, copyValue, loadText) {
    const preview = document.getElementById('ops-schedule-preview');
    if (!preview) return;
    const body = String(text || '').trim();
    if (!body) {
        preview.hidden = true;
        preview.textContent = '';
        opsScheduleLoadText = null;
        setOpsScheduleCopyChip(null);
        setOpsScheduleLoadChatVisible(false);
        showOpsScheduleLoadStatus('', true);
        return;
    }
    preview.hidden = false;
    preview.textContent = body.slice(0, 12000);
    setOpsScheduleCopyChip(copyValue);
    const q = String(loadText || '').trim();
    opsScheduleLoadText = q && q !== '(empty task)' && q !== '(empty summary)' ? q : null;
    setOpsScheduleLoadChatVisible(!!opsScheduleLoadText);
    if (opsScheduleLoadText) {
        showOpsScheduleLoadStatus('Preview ready — Enter or “Load into AI Chat” · double-click also loads.', true);
    } else {
        showOpsScheduleLoadStatus('', true);
    }
}

/** Load-into-chat control under the Schedules preview (Sessions/Runs parity). */
function ensureOpsScheduleLoadChatBtn() {
    let el = document.getElementById('ops-schedules-load-chat');
    if (el) return el;
    const preview = document.getElementById('ops-schedule-preview');
    if (!preview || !preview.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-schedules-load-chat';
    el.className = 'btn-secondary ops-schedules-load-chat';
    el.hidden = true;
    el.textContent = 'Load into AI Chat ↵';
    el.title = 'Put this schedule task or delivery summary into AI Chat (Enter)';
    el.setAttribute('aria-label', 'Load schedule task into AI Chat');
    preview.parentNode.insertBefore(el, preview.nextSibling);
    el.addEventListener('click', () => loadOpsScheduleIntoChat());
    return el;
}

function setOpsScheduleLoadChatVisible(visible) {
    const el = ensureOpsScheduleLoadChatBtn();
    if (!el) return;
    el.hidden = !visible;
    if (!visible) {
        el.classList.remove('is-just-saved');
        if (!el._saveFlashOriginalLabel) {
            el.textContent = 'Load into AI Chat ↵';
        }
    }
    refreshAllOpsPreviewRowRovingTabindex({ schedules: 'ops-schedules-load-chat' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[2]);
}

function showOpsScheduleLoadStatus(msg, ok) {
    let el = document.getElementById('ops-schedules-load-status');
    const loadBtn = ensureOpsScheduleLoadChatBtn();
    if (!el) {
        el = document.createElement('div');
        el.id = 'ops-schedules-load-status';
        el.className = 'ops-row-meta';
        el.style.margin = '6px 4px 0';
        if (loadBtn?.parentNode) {
            loadBtn.parentNode.insertBefore(el, loadBtn.nextSibling);
        }
    }
    el.textContent = msg || '';
    el.style.opacity = msg ? '0.9' : '0';
    el.style.color = ok === false ? 'rgba(200,60,60,0.95)' : '';
}

/** Put the previewed schedule task or delivery summary into AI Chat. */
function loadOpsScheduleIntoChat() {
    const loadBtn = ensureOpsScheduleLoadChatBtn();
    if (loadBtn?.classList.contains('is-just-saved')) return;
    const q = String(opsScheduleLoadText || '').trim();
    if (!q) {
        showOpsScheduleLoadStatus('Select a schedule or delivery with text first.', false);
        return;
    }
    const aiOff =
        document.getElementById('icon-ollama')?.style.pointerEvents === 'none' ||
        document.getElementById('ollama-section')?.style.display === 'none';
    if (aiOff) {
        showOpsScheduleLoadStatus('Enable local AI agent in Settings to load into chat.', false);
        return;
    }
    const input = document.getElementById('chat-input');
    if (!input) {
        showOpsScheduleLoadStatus('AI Chat input not ready — open AI Chat once, then retry.', false);
        return;
    }
    input.value = q;
    try {
        input.dispatchEvent(new Event('input', { bubbles: true }));
    } catch (_) {
        /* ignore */
    }
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
    setTimeout(() => {
        input.focus();
        try {
            const len = input.value.length;
            input.setSelectionRange(len, len);
        } catch (_) {
            /* ignore */
        }
    }, 80);
    showOpsScheduleLoadStatus('Loaded into AI Chat.', true);
    if (loadBtn && !loadBtn.hidden) {
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(loadBtn, { savedLabel: 'Loaded', durationMs: 1600 });
        } else {
            const idle = loadBtn.textContent || 'Load into AI Chat ↵';
            loadBtn.classList.add('is-just-saved');
            loadBtn.textContent = 'Loaded';
            setTimeout(() => {
                loadBtn.classList.remove('is-just-saved');
                loadBtn.textContent = idle;
            }, 1600);
        }
    }
}

/** Click-to-copy schedule id above the Schedules preview (Copied flash). */
function ensureOpsScheduleCopyChip() {
    let el = document.getElementById('ops-schedule-copy-chip');
    if (el) return el;
    const preview = document.getElementById('ops-schedule-preview');
    if (!preview || !preview.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-schedule-copy-chip';
    el.className = 'ops-session-copy-chip ops-schedule-copy-chip';
    el.hidden = true;
    el.setAttribute('aria-label', 'Copy schedule id');
    preview.parentNode.insertBefore(el, preview);
    el.addEventListener('click', async (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (el.classList.contains('is-just-saved')) return;
        const value = el.dataset.copyValue || '';
        if (!value) return;
        const ok = await copyOpsTextToClipboard(value);
        if (!ok) return;
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(el, { savedLabel: 'Copied', durationMs: 1600 });
        } else {
            const idle = el._saveFlashOriginalLabel || value;
            el._saveFlashOriginalLabel = idle;
            el.classList.add('is-just-saved');
            el.textContent = 'Copied';
            clearTimeout(el._saveFlashTimer);
            el._saveFlashTimer = setTimeout(() => {
                el.classList.remove('is-just-saved');
                el.textContent = idle;
                el._saveFlashOriginalLabel = null;
                el._saveFlashTimer = null;
            }, 1600);
        }
    });
    el.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            el.click();
        }
    });
    return el;
}

function setOpsScheduleCopyChip(copyValue) {
    const el = ensureOpsScheduleCopyChip();
    if (!el) return;
    const value = String(copyValue || '').trim();
    if (!value || value === '—') {
        el.hidden = true;
        el.dataset.copyValue = '';
        el.classList.remove('is-just-saved');
        el.textContent = '';
        refreshAllOpsPreviewRowRovingTabindex({ schedules: 'ops-schedule-copy-chip' });
        ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[2]);
        return;
    }
    el.hidden = false;
    el.dataset.copyValue = value;
    el.title = 'Click to copy schedule id (c)';
    el.setAttribute('aria-label', `Copy ${value}`);
    if (!el.classList.contains('is-just-saved')) {
        el.textContent = value;
        el._saveFlashOriginalLabel = value;
    }
    refreshAllOpsPreviewRowRovingTabindex({ schedules: 'ops-schedule-copy-chip' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[2]);
}

function formatOpsSchedulePreview(s) {
    const id = s?.id || '(no id)';
    const when = s?.cron ? `cron ${s.cron}` : s?.at ? `at ${s.at}` : '—';
    const next = s?.next_run || s?.nextRun || '—';
    const task = String(s?.task || '').trim() || '(empty task)';
    return `Schedule: ${id}\nWhen: ${when}\nNext: ${next}\n\nTask:\n${task}`;
}

function formatOpsDeliveryPreview(d) {
    const id = d?.schedule_id || 'schedule';
    const utc = d?.utc || '—';
    const t = d?.utc ? Date.parse(d.utc) : NaN;
    const age = !Number.isNaN(t) ? fmtAge(t) : '';
    const whenLine = age ? `${utc} (${age})` : utc;
    const summary = String(d?.summary || '').trim() || '(empty summary)';
    return `Delivery: ${id}\nWhen: ${whenLine}\n\nSummary:\n${summary}`;
}

function renderOpsSchedulesTab(schedules, deliveries) {
    const list = document.getElementById('ops-schedules-list');
    const delList = document.getElementById('ops-deliveries-list');
    showOpsSchedulePreview('');
    const schedAll = schedules || [];
    const schedFiltered = schedAll.filter((s) => {
        const when = s.cron ? `cron ${s.cron}` : s.at ? `at ${s.at}` : '';
        return schedulesRowMatchesFilter(
            `${s.id || ''} ${when} ${s.next_run || s.nextRun || ''} ${s.task || ''}`
        );
    });
    const delAll = deliveries || [];
    const delFiltered = delAll.filter((d) =>
        schedulesRowMatchesFilter(`${d.schedule_id || ''} ${d.summary || ''} ${d.utc || ''}`)
    );
    if (list) {
        list.innerHTML = '';
        if (!schedAll.length) {
            list.innerHTML = opsTabEmptyHtml(
              'No schedules yet',
              'Create one via Discord SCHEDULE tools or the scheduler API'
            );
        } else if (!schedFiltered.length) {
            list.innerHTML = opsFilterMissHtml('No schedules match filter', 'schedules');
        } else {
            schedFiltered.forEach((s) => {
                const btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'ops-row';
                const id = s.id || '(no id)';
                const when = s.cron ? `cron ${s.cron}` : s.at ? `at ${s.at}` : '—';
                const next = s.next_run || s.nextRun || '—';
                const task = String(s.task || '');
                btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(id)}</div><div class="ops-row-meta">${escapeHtml(when)} · next ${escapeHtml(next)}</div><div class="ops-row-meta">${escapeHtml(task.slice(0, 80))}${task.length > 80 ? '…' : ''}</div></div>`;
                setOpsRowCopyValue(btn, s.id);
                btn.title = 'Click to preview · c copies id · Enter / double-click to load task into AI Chat';
                const openPreview = () => {
                    list.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
                    delList?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
                    btn.classList.add('is-selected');
                    const task = String(s?.task || '').trim();
                    showOpsSchedulePreview(formatOpsSchedulePreview(s), s.id || '', task);
                };
                btn.addEventListener('click', openPreview);
                btn.addEventListener('dblclick', (e) => {
                    e.preventDefault();
                    openPreview();
                    loadOpsScheduleIntoChat();
                });
                list.appendChild(btn);
            });
        }
    }
    if (delList) {
        delList.innerHTML = '';
        if (!delAll.length) {
            delList.innerHTML = opsTabEmptyHtml(
              'No deliveries yet',
              'Results appear here after a schedule runs'
            );
        } else if (!delFiltered.length) {
            delList.innerHTML = opsFilterMissHtml('No deliveries match filter', 'schedules');
        } else {
            delFiltered.slice(0, 8).forEach((d) => {
                const btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'ops-row';
                const t = d.utc ? Date.parse(d.utc) : NaN;
                const age = !Number.isNaN(t) ? fmtAge(t) : d.utc || '';
                const summary = String(d.summary || '');
                btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(d.schedule_id || 'schedule')}</div><div class="ops-row-meta">${escapeHtml(age)} · ${escapeHtml(summary.slice(0, 72))}${summary.length > 72 ? '…' : ''}</div></div>`;
                setOpsRowCopyValue(btn, d.schedule_id);
                btn.title = 'Click to preview · c copies id · Enter / double-click to load summary into AI Chat';
                const openPreview = () => {
                    delList.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
                    list?.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
                    btn.classList.add('is-selected');
                    const summary = String(d?.summary || '').trim();
                    showOpsSchedulePreview(formatOpsDeliveryPreview(d), d.schedule_id || '', summary);
                };
                btn.addEventListener('click', openPreview);
                btn.addEventListener('dblclick', (e) => {
                    e.preventDefault();
                    openPreview();
                    loadOpsScheduleIntoChat();
                });
                delList.appendChild(btn);
            });
        }
    }
    paintOpsFilterMatch(
        'ops-schedules-filter',
        (() => {
            if (opsSchedulesKindFilter === 'jobs') return schedAll.length;
            if (opsSchedulesKindFilter === 'deliveries') return delAll.length;
            return schedAll.length + delAll.length;
        })(),
        (() => {
            if (opsSchedulesKindFilter === 'jobs') return schedFiltered.length;
            if (opsSchedulesKindFilter === 'deliveries') return delFiltered.length;
            return schedFiltered.length + delFiltered.length;
        })(),
        opsSchedulesFilterQ
    );
    paintOpsSchedulesKindChips();
    applyOpsSchedulesKindVisibility();
}

/** Focusable refresh-row items in DOM order (Refresh · Refresh digest · Updated when visible). */
function getOpsRefreshRowItems() {
    const row = document.querySelector('.ops-refresh-row');
    if (!row) return [];
    const items = [];
    const refresh = document.getElementById('ops-refresh-btn');
    const digest = document.getElementById('ops-digest-refresh-btn');
    const updated = document.getElementById('ops-updated-ago');
    if (refresh && !refresh.hidden) items.push(refresh);
    if (digest && !digest.hidden) items.push(digest);
    if (updated && !updated.hidden && (updated.textContent || '').trim()) {
        if (!updated.getAttribute('role')) updated.setAttribute('role', 'button');
        if (!updated.getAttribute('tabindex')) updated.tabIndex = -1;
        if (!updated.title || updated.title.indexOf('Enter') < 0) {
            updated.title = `${updated.title || 'Last refresh'} · Enter refreshes`;
        }
        items.push(updated);
    }
    return items.filter((el) => {
        if (!el || el.hidden) return false;
        return el.getClientRects().length > 0 || el.offsetParent !== null || row.contains(el);
    });
}

function refreshOpsRefreshRowRovingTabindex(preferred) {
    const items = getOpsRefreshRowItems();
    if (!items.length) return;
    const focused = items.find((el) => el === document.activeElement);
    const current =
        (preferred && items.includes(preferred) && preferred) ||
        focused ||
        items.find((el) => el.tabIndex === 0) ||
        items[0];
    for (const el of items) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function ensureOpsRefreshRowKbHint() {
    const row = document.querySelector('.ops-refresh-row');
    if (!row) return;
    let hint = document.getElementById('ops-refresh-row-kb-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-refresh-row-kb-hint';
        hint.className = 'ops-refresh-row-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        row.appendChild(hint);
    }
    hint.textContent =
        '← → / h l · Home/End move · Enter / Space refreshes or runs digest';
}

/**
 * Refresh-row toolbar keyboard — focus Refresh · Refresh digest · Updated,
 * then ←→ / h l / Home/End (tab-bar / health-strip parity). Enter/Space keeps
 * existing button activate; Updated stamp triggers full refresh.
 */
function ensureOpsRefreshRowToolbarKeyboard() {
    const row = document.querySelector('.ops-refresh-row');
    if (!row) return;
    ensureOpsRefreshRowKbHint();
    refreshOpsRefreshRowRovingTabindex();
    if (row.dataset.opsRefreshRowKbWired === '1') return;
    row.dataset.opsRefreshRowKbWired = '1';
    if (!row.getAttribute('role')) {
        row.setAttribute('role', 'toolbar');
    }
    if (!row.getAttribute('aria-label')) {
        row.setAttribute('aria-label', 'Agent Ops refresh controls');
    }
    row.addEventListener('focusin', (e) => {
        const items = getOpsRefreshRowItems();
        if (items.includes(e.target)) refreshOpsRefreshRowRovingTabindex(e.target);
    });
    row.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const items = getOpsRefreshRowItems();
        if (!items.length) return;
        const idx = items.indexOf(document.activeElement);
        if (idx < 0) return;
        if (e.key === 'Enter' || e.key === ' ') {
            const el = items[idx];
            if (el?.id === 'ops-updated-ago') {
                e.preventDefault();
                e.stopPropagation();
                refreshAgentOps({ userTriggered: true });
            }
            return;
        }
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, items.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = items.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsRefreshRowRovingTabindex(items[next]);
        items[next].focus();
    });
}

/**
 * Keep Refresh / Updated under the health strip (not buried under tab panels).
 * Themes still ship the row at the bottom of agent-ops-content; we re-home it once.
 */
function ensureOpsRefreshRowPlacement() {
    const row = document.querySelector('.ops-refresh-row');
    const health = document.getElementById('ops-health-row');
    if (!row || !health) return null;
    if (health.nextElementSibling === row) return row;
    health.insertAdjacentElement('afterend', row);
    row.classList.add('ops-refresh-row-top');
    return row;
}

/** Relative age stamp beside Refresh so operators see Command Center freshness. */
function ensureOpsUpdatedAgo() {
    ensureOpsRefreshRowPlacement();
    const row = document.querySelector('.ops-refresh-row');
    if (!row) return null;
    let el = document.getElementById('ops-updated-ago');
    if (el) return el;
    el = document.createElement('span');
    el.id = 'ops-updated-ago';
    el.className = 'ops-row-meta ops-updated-ago';
    el.setAttribute('aria-live', 'polite');
    el.hidden = true;
    row.appendChild(el);
    return el;
}

function paintOpsUpdatedAgo() {
    const el = ensureOpsUpdatedAgo();
    if (!el) return;
    if (!opsLastRefreshMs) {
        el.hidden = true;
        el.textContent = '';
        el.removeAttribute('title');
        el.removeAttribute('role');
        el.tabIndex = -1;
        refreshOpsRefreshRowRovingTabindex();
        return;
    }
    const age = fmtAge(opsLastRefreshMs) || 'just now';
    el.hidden = false;
    el.textContent = `Updated ${age}`;
    const when = new Date(opsLastRefreshMs).toLocaleTimeString();
    el.title = `Last refresh ${when} · Enter refreshes`;
    refreshOpsRefreshRowRovingTabindex();
}

function markOpsRefreshedAt(ms) {
    opsLastRefreshMs = ms || Date.now();
    paintOpsUpdatedAgo();
    if (opsUpdatedAgoTimer) return;
    opsUpdatedAgoTimer = setInterval(() => {
        if (!opsLastRefreshMs) return;
        paintOpsUpdatedAgo();
    }, 15_000);
}

function setOpsRefreshBusy(busy) {
    const btn = document.getElementById('ops-refresh-btn');
    if (!btn) return;
    if (opsRefreshFlashTimer) {
        clearTimeout(opsRefreshFlashTimer);
        opsRefreshFlashTimer = null;
    }
    btn.classList.remove('is-just-saved');
    if (!btn.dataset.idleLabel) {
        btn.dataset.idleLabel = btn.textContent || 'Refresh';
    }
    if (busy) {
        btn.disabled = true;
        btn.textContent = 'Refreshing…';
        btn.title = 'Refresh in progress';
    } else {
        btn.disabled = false;
        btn.textContent = btn.dataset.idleLabel || 'Refresh';
        btn.title = 'Refresh Agent Ops';
    }
}

function flashOpsRefreshed() {
    const btn = document.getElementById('ops-refresh-btn');
    if (!btn) return;
    if (opsRefreshFlashTimer) {
        clearTimeout(opsRefreshFlashTimer);
        opsRefreshFlashTimer = null;
    }
    const idle = btn.dataset.idleLabel || 'Refresh';
    btn.disabled = false;
    btn.classList.add('is-just-saved');
    btn.textContent = 'Refreshed';
    btn.title = 'Refresh complete';
    opsRefreshFlashTimer = setTimeout(() => {
        btn.classList.remove('is-just-saved');
        btn.textContent = idle;
        btn.title = 'Refresh Agent Ops';
        opsRefreshFlashTimer = null;
    }, 1600);
}

async function refreshAgentOps(opts = {}) {
    const userTriggered = !!opts.userTriggered;
    const healthRow = document.getElementById('ops-health-row');
    if (opsRefreshInFlight) return;
    opsRefreshInFlight = true;
    if (userTriggered) setOpsRefreshBusy(true);
    let ok = false;
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
            sessionFiles: files || [],
        });
        opsAgentsCache = agents || [];
        renderOverviewAgents(opsAgentsCache);
        renderOverviewSchedules(schedules || [], deliveries || []);
        renderOverviewLive(live || [], insights);
        renderOverviewKnowledge(memory || []);
        renderOverviewRecent(files || []);
        renderOverviewRuns(insights);
        renderOverviewDigest(insights);
        opsSchedulesCache = schedules || [];
        opsDeliveriesCache = deliveries || [];
        renderOpsSchedulesTab(opsSchedulesCache, opsDeliveriesCache);
        renderOpsAgents(opsAgentsCache);
        opsLiveCache = live || [];
        opsSessionFilesCache = files || [];
        renderOpsLive(opsLiveCache);
        renderOpsSessionFiles(opsSessionFilesCache);
        opsMemoryCache = memory || [];
        renderOpsMemory(opsMemoryCache);
        opsRunsInsightsCache = insights;
        renderOpsRuns(opsRunsInsightsCache);
        paintOpsTabCounts({
            agents: (opsAgentsCache || []).length,
            sessions: (opsLiveCache || []).length + (opsSessionFilesCache || []).length,
            schedules: (opsSchedulesCache || []).length,
            memory: (opsMemoryCache || []).length,
            runs: Array.isArray(insights?.recent)
                ? insights.recent.length
                : Number(insights?.turns) || 0,
        });
        ok = true;
    } catch (err) {
        console.warn('[Agent Ops]', err);
        if (healthRow) {
            setText('ops-health-version', 'Unavailable');
            setText('ops-health-discord', String(err).slice(0, 40));
        }
    } finally {
        opsRefreshInFlight = false;
        if (ok) markOpsRefreshedAt(Date.now());
        if (userTriggered) {
            setOpsRefreshBusy(false);
            if (ok) flashOpsRefreshed();
        }
    }
}

function renderOpsAgents(agents) {
    const list = document.getElementById('ops-agents-list');
    list.innerHTML = '';
    const all = agents || [];
    ensureOpsAgentsEnabledChips();
    const kindPool = all.filter(agentsRowMatchesEnabled);
    const filtered = kindPool.filter((a) =>
        agentsRowMatchesFilter(
            `${a.name || ''} ${a.slug || ''} ${a.id || ''} ${a.model || ''} ${a.enabled ? 'on' : 'off'} ${a.orchestrator ? 'orchestrator' : ''}`
        )
    );
    if (!all.length) {
        list.innerHTML = opsTabEmptyHtml(
          'No agents yet',
          'Add agent folders under ~/.mac-stats/agents'
        );
        paintOpsFilterMatch('ops-agents-filter', 0, 0, opsAgentsFilterQ);
        paintOpsAgentsEnabledChips();
        return;
    }
    if (!filtered.length) {
        list.innerHTML = opsFilterMissHtml('No agents match filter', 'agents');
        paintOpsFilterMatch('ops-agents-filter', kindPool.length, 0, opsAgentsFilterQ);
        paintOpsAgentsEnabledChips();
        return;
    }
    filtered.forEach((a) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const slug = a.slug || a.id;
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(a.name)} <span class="ops-row-meta">· ${escapeHtml(slug)}</span></div><div class="ops-row-meta">${escapeHtml(a.model || 'default model')}${a.orchestrator ? ' · orchestrator' : ''}</div></div><span class="ops-badge ${a.enabled ? '' : 'off'}">${a.enabled ? 'on' : 'off'}</span>`;
        setOpsRowCopyValue(btn, slug);
        btn.addEventListener('click', () => {
            list.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            openOpsAgent(a.id);
        });
        btn.addEventListener('dblclick', async (e) => {
            e.preventDefault();
            e.stopPropagation();
            list.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            await openOpsAgent(a.id);
            loadOpsAgentIntoChat();
        });
        btn.title = 'Open agent · c copies id · Enter / double-click to load soul/skill/mood into AI Chat';
        list.appendChild(btn);
    });
    paintOpsFilterMatch('ops-agents-filter', kindPool.length, filtered.length, opsAgentsFilterQ);
    paintOpsAgentsEnabledChips();
}

async function openOpsAgent(id) {
    try {
        opsAgentCache = await invoke('get_agent_details', { selector: id });
        document.getElementById('ops-agents-list').style.display = 'none';
        const detail = document.getElementById('ops-agent-detail');
        detail.hidden = false;
        document.getElementById('ops-agent-meta').textContent =
            `${opsAgentCache.name} · ${opsAgentCache.slug || opsAgentCache.id} · ${opsAgentCache.model || 'default'} · ${opsAgentCache.enabled ? 'enabled' : 'disabled'}`;
        setOpsAgentCopyChip(opsAgentCache.slug || opsAgentCache.id || '');
        opsAgentFileTab = 'soul';
        opsAgentDirty = { soul: false, skill: false, mood: false };
        document.querySelectorAll('.ops-file-tab').forEach((b) => {
            b.classList.toggle('active', b.dataset.file === 'soul');
        });
        ensureOpsAgentEditor();
        ensureOpsFileTabToolbarKeyboard();
        ensureOpsAgentEditActionsToolbarKeyboard();
        refreshOpsFileTabRovingTabindex();
        refreshOpsAgentEditActionsRovingTabindex();
        renderOpsAgentPreview();
        setOpsAgentSaveStatus('');
        refreshOpsAgentLoadText();
    } catch (err) {
        opsAgentLoadText = null;
        setOpsAgentLoadChatVisible(false);
        showOpsAgentLoadStatus(String(err), false);
        alert(`Failed to load agent: ${err}`);
    }
}

function closeOpsAgentDetail() {
    const detail = document.getElementById('ops-agent-detail');
    if (detail) detail.hidden = true;
    const list = document.getElementById('ops-agents-list');
    if (list) list.style.display = '';
    opsAgentDirty = { soul: false, skill: false, mood: false };
    opsAgentLoadText = null;
    setOpsAgentSaveStatus('');
    setOpsAgentCopyChip(null);
    setOpsAgentLoadChatVisible(false);
    showOpsAgentLoadStatus('', true);
    const editor = document.getElementById('ops-agent-preview');
    if (editor) editor.classList.remove('is-dirty');
    const saveBtn = document.getElementById('ops-agent-save');
    if (saveBtn) saveBtn.disabled = true;
}

/** Ensure agent detail uses an editable textarea + Save (themes may still ship a read-only <pre>). */
function ensureOpsAgentEditor() {
    const detail = document.getElementById('ops-agent-detail');
    if (!detail) return;
    let editor = document.getElementById('ops-agent-preview');
    if (editor && editor.tagName === 'PRE') {
        const ta = document.createElement('textarea');
        ta.className = 'ops-preview ops-agent-editor';
        ta.id = 'ops-agent-preview';
        ta.spellcheck = false;
        ta.setAttribute('aria-label', 'Agent soul, skill, or mood');
        editor.replaceWith(ta);
        editor = ta;
    } else if (!editor) {
        editor = document.createElement('textarea');
        editor.className = 'ops-preview ops-agent-editor';
        editor.id = 'ops-agent-preview';
        editor.spellcheck = false;
        editor.setAttribute('aria-label', 'Agent soul, skill, or mood');
        const tabs = detail.querySelector('.ops-file-tabs');
        if (tabs) tabs.after(editor);
        else detail.prepend(editor);
    }
    if (!document.getElementById('ops-agent-edit-actions') && !document.querySelector('.ops-agent-edit-actions')) {
        const row = document.createElement('div');
        row.className = 'ops-agent-edit-actions';
        row.id = 'ops-agent-edit-actions';
        const save = document.createElement('button');
        save.type = 'button';
        save.className = 'btn-primary';
        save.id = 'ops-agent-save';
        save.textContent = 'Save';
        save.disabled = true;
        const status = document.createElement('span');
        status.className = 'ops-agent-save-status';
        status.id = 'ops-agent-save-status';
        status.setAttribute('aria-live', 'polite');
        let back = document.getElementById('ops-agent-back');
        if (!back) {
            back = document.createElement('button');
            back.type = 'button';
            back.className = 'btn-secondary';
            back.id = 'ops-agent-back';
            back.textContent = '← Back';
            back.addEventListener('click', () => {
                if (Object.values(opsAgentDirty).some(Boolean)) {
                    const ok = window.confirm('Discard unsaved soul/skill/mood changes?');
                    if (!ok) return;
                }
                closeOpsAgentDetail();
            });
        } else {
            back.remove();
        }
        row.appendChild(save);
        row.appendChild(status);
        row.appendChild(back);
        editor.after(row);
        save.addEventListener('click', () => saveOpsAgentFile());
    }
    ensureOpsAgentLoadChatBtn();
    ensureOpsAgentEditActionsToolbarKeyboard();
    if (editor && editor.dataset.opsEditBound !== '1') {
        editor.dataset.opsEditBound = '1';
        editor.addEventListener('input', () => {
            if (!opsAgentCache) return;
            syncOpsAgentEditorToCache();
            opsAgentDirty[opsAgentFileTab] = true;
            editor.classList.add('is-dirty');
            const saveBtn = document.getElementById('ops-agent-save');
            if (saveBtn) saveBtn.disabled = false;
            setOpsAgentSaveStatus('Unsaved changes');
            refreshOpsAgentLoadText({ quiet: true });
        });
        editor.addEventListener('keydown', (e) => {
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 's') {
                e.preventDefault();
                saveOpsAgentFile();
            }
        });
    }
}

function opsAgentFileContent(kind) {
    if (!opsAgentCache) return '';
    if (kind === 'skill') return opsAgentCache.skill || '';
    if (kind === 'soul') return opsAgentCache.soul || '';
    if (kind === 'mood') return opsAgentCache.mood || '';
    return '';
}

function syncOpsAgentEditorToCache() {
    const editor = document.getElementById('ops-agent-preview');
    if (!editor || !opsAgentCache || editor.tagName === 'PRE') return;
    const text = editor.value;
    if (opsAgentFileTab === 'skill') opsAgentCache.skill = text;
    else if (opsAgentFileTab === 'soul') opsAgentCache.soul = text;
    else if (opsAgentFileTab === 'mood') opsAgentCache.mood = text;
}

function setOpsAgentSaveStatus(msg) {
    const el = document.getElementById('ops-agent-save-status');
    if (el) el.textContent = msg || '';
    if (opsAgentSaveStatusTimer) {
        clearTimeout(opsAgentSaveStatusTimer);
        opsAgentSaveStatusTimer = null;
    }
    if (msg && msg.startsWith('Saved')) {
        opsAgentSaveStatusTimer = setTimeout(() => {
            const cur = document.getElementById('ops-agent-save-status');
            if (cur && cur.textContent === msg) cur.textContent = '';
        }, 2500);
    }
}

function setOpsAgentSaveBusy(busy) {
    opsAgentSaveBusy = !!busy;
    const saveBtn = document.getElementById('ops-agent-save');
    if (!saveBtn) return;
    if (busy) {
        saveBtn.disabled = true;
        saveBtn.classList.remove('is-just-saved');
        if (saveBtn._saveFlashOriginalLabel == null) {
            saveBtn._saveFlashOriginalLabel = saveBtn.textContent || 'Save';
        }
        saveBtn.textContent = 'Saving…';
        return;
    }
    if (!saveBtn.classList.contains('is-just-saved')) {
        saveBtn.textContent = saveBtn._saveFlashOriginalLabel || 'Save';
        saveBtn._saveFlashOriginalLabel = null;
    }
}

function flashOpsAgentSaveBtn(saveBtn) {
    if (!saveBtn) return;
    if (typeof window.flashSaveButton === 'function') {
        window.flashSaveButton(saveBtn, { savedLabel: 'Saved', durationMs: 1600 });
        return;
    }
    const prev = saveBtn._saveFlashOriginalLabel || saveBtn.textContent || 'Save';
    saveBtn.classList.add('is-just-saved');
    saveBtn.textContent = 'Saved';
    setTimeout(() => {
        saveBtn.classList.remove('is-just-saved');
        saveBtn.textContent = prev;
        saveBtn._saveFlashOriginalLabel = null;
    }, 1600);
}

async function saveOpsAgentFile() {
    if (opsAgentSaveBusy) return;
    if (!opsAgentCache?.id) return;
    ensureOpsAgentEditor();
    syncOpsAgentEditorToCache();
    const kind = opsAgentFileTab;
    const content = opsAgentFileContent(kind);
    const cmd =
        kind === 'soul'
            ? 'update_agent_soul'
            : kind === 'mood'
              ? 'update_agent_mood'
              : 'update_agent_skill';
    const saveBtn = document.getElementById('ops-agent-save');
    setOpsAgentSaveBusy(true);
    setOpsAgentSaveStatus('Saving…');
    try {
        await invoke(cmd, { agentId: opsAgentCache.id, content });
        opsAgentDirty[kind] = false;
        const editor = document.getElementById('ops-agent-preview');
        if (editor) editor.classList.remove('is-dirty');
        const stillDirty = Object.values(opsAgentDirty).some(Boolean);
        setOpsAgentSaveBusy(false);
        if (saveBtn) saveBtn.disabled = !stillDirty;
        flashOpsAgentSaveBtn(saveBtn);
        setOpsAgentSaveStatus(`Saved ${kind}.md`);
    } catch (err) {
        setOpsAgentSaveBusy(false);
        if (saveBtn) saveBtn.disabled = false;
        setOpsAgentSaveStatus(`Save failed: ${err}`);
        alert(`Failed to save ${kind}.md: ${err}`);
    }
}

function renderOpsAgentPreview() {
    if (!opsAgentCache) return;
    ensureOpsAgentEditor();
    const editor = document.getElementById('ops-agent-preview');
    if (!editor) return;
    const text = opsAgentFileContent(opsAgentFileTab);
    if (editor.tagName === 'TEXTAREA') {
        editor.value = text;
        editor.placeholder = `Edit ${opsAgentFileTab}.md for this agent…`;
        editor.classList.toggle('is-dirty', !!opsAgentDirty[opsAgentFileTab]);
    } else {
        editor.textContent = text || `(empty ${opsAgentFileTab}.md)`;
    }
    const saveBtn = document.getElementById('ops-agent-save');
    if (saveBtn && !opsAgentSaveBusy && !saveBtn.classList.contains('is-just-saved')) {
        saveBtn.disabled = !Object.values(opsAgentDirty).some(Boolean);
        saveBtn.textContent = 'Save';
    }
    if (opsAgentSaveBusy) {
        setOpsAgentSaveStatus('Saving…');
    } else if (!opsAgentDirty[opsAgentFileTab]) {
        setOpsAgentSaveStatus('');
    } else {
        setOpsAgentSaveStatus('Unsaved changes');
    }
}

/** Esc closes agent detail when open (Agents tab). Filter Esc still clears first when focused. */
function tryOpsAgentDetailEscape(e) {
    if (agentOpsCollapsed) return false;
    const detail = document.getElementById('ops-agent-detail');
    if (!detail || detail.hidden) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return false;
    if (Object.values(opsAgentDirty).some(Boolean)) {
        const ok = window.confirm('Discard unsaved soul/skill/mood changes?');
        if (!ok) return true;
    }
    e.preventDefault();
    closeOpsAgentDetail();
    return true;
}

/** Esc hides session / knowledge / schedule / runs preview panes (Hermes-style dismiss). */
function tryOpsPreviewEscape(e) {
    if (agentOpsCollapsed) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return false;
    const sessionPreview = document.getElementById('ops-session-preview');
    const memoryPreview = document.getElementById('ops-memory-preview');
    const schedulePreview = document.getElementById('ops-schedule-preview');
    const runsPreview = document.getElementById('ops-runs-preview');
    const loadBtn = document.getElementById('ops-session-load-chat');
    let closed = false;
    if (sessionPreview && !sessionPreview.hidden) {
        sessionPreview.hidden = true;
        sessionPreview.textContent = '';
        opsSessionLoadRows = null;
        if (loadBtn) loadBtn.hidden = true;
        showOpsSessionStatus('', true);
        setOpsSessionCopyChip(null);
        closed = true;
    }
    if (memoryPreview && !memoryPreview.hidden) {
        memoryPreview.hidden = true;
        memoryPreview.textContent = '';
        opsMemoryLoadText = null;
        setOpsMemoryCopyChip(null);
        setOpsMemoryLoadChatVisible(false);
        showOpsMemoryLoadStatus('', true);
        closed = true;
    }
    if (schedulePreview && !schedulePreview.hidden) {
        showOpsSchedulePreview('');
        closed = true;
    }
    if (runsPreview && !runsPreview.hidden) {
        showOpsRunPreview('');
        closed = true;
    }
    if (closed) {
        e.preventDefault();
        return true;
    }
    return false;
}

/** Esc clears list row selection when nothing else to dismiss (Hermes Escape-skips). */
function tryOpsClearSelectionEscape(e) {
    if (agentOpsCollapsed) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return false;
    if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
        return false;
    }
    const selected = document.querySelectorAll('.ops-row.is-selected');
    const insightSel = document.querySelectorAll('.ops-insight-line.is-selected');
    if (!selected.length && !insightSel.length) return false;
    selected.forEach((el) => el.classList.remove('is-selected'));
    insightSel.forEach((el) => el.classList.remove('is-selected'));
    e.preventDefault();
    return true;
}

function formatSessionMessagesPreview(rows) {
    if (!rows || !rows.length) return '(empty session)';
    return rows
        .map((m) => `## ${m.role === 'user' ? 'User' : 'Assistant'}\n\n${m.content}`)
        .join('\n\n');
}

function markOpsSessionRowSelected(btn) {
    document
        .querySelectorAll('#ops-live-sessions .ops-row.is-selected, #ops-session-files .ops-row.is-selected')
        .forEach((el) => el.classList.remove('is-selected'));
    btn?.classList?.add('is-selected');
}

/** Enter loads the previewed session when Sessions tab is active and a load is ready. */
function tryOpsSessionEnterLoad(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-sessions');
    if (!panel || !panel.classList.contains('active')) return false;
    const loadBtn = document.getElementById('ops-session-load-chat');
    if (!loadBtn || loadBtn.hidden || !opsSessionLoadRows?.length) return false;
    if (loadBtn.classList.contains('is-just-saved')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    // Allow Enter from filter / list / body; skip unrelated text fields.
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-session-filter') return false;
    e.preventDefault();
    loadOpsSessionIntoChat();
    return true;
}

/** Enter loads the previewed knowledge file into AI Chat when ready (Sessions/Runs parity). */
function tryOpsMemoryEnterLoad(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-memory');
    if (!panel || !panel.classList.contains('active')) return false;
    const loadBtn = document.getElementById('ops-memory-load-chat');
    if (!loadBtn || loadBtn.hidden || !opsMemoryLoadText) return false;
    if (loadBtn.classList.contains('is-just-saved')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-memory-filter') return false;
    e.preventDefault();
    loadOpsMemoryIntoChat();
    return true;
}

/** Enter opens the selected Knowledge row (or first visible row) into the preview pane. */
function tryOpsMemoryEnter(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-memory');
    if (!panel || !panel.classList.contains('active')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-memory-filter') return false;
    const list = document.getElementById('ops-memory-list');
    if (!list) return false;
    const selected =
        list.querySelector('.ops-row.is-selected') || list.querySelector('.ops-row');
    if (!selected) return false;
    e.preventDefault();
    selected.click();
    return true;
}

/** Enter loads the previewed run question into AI Chat when ready (Sessions parity). */
function tryOpsRunsEnterLoad(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-runs');
    if (!panel || !panel.classList.contains('active')) return false;
    const loadBtn = document.getElementById('ops-runs-load-chat');
    if (!loadBtn || loadBtn.hidden || !opsRunLoadQuestion) return false;
    if (loadBtn.classList.contains('is-just-saved')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-runs-filter') return false;
    e.preventDefault();
    loadOpsRunIntoChat();
    return true;
}

/** Enter activates the selected (or first) Runs row. */
function tryOpsRunsEnter(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-runs');
    if (!panel || !panel.classList.contains('active')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-runs-filter') return false;
    const list = document.getElementById('ops-runs-list') || panel.querySelector('.ops-list');
    if (!list) return false;
    const selected =
        list.querySelector('.ops-row.is-selected') || list.querySelector('.ops-row');
    if (!selected) return false;
    e.preventDefault();
    selected.click();
    return true;
}

/** Enter opens the selected (or first) Agents row detail. */
function tryOpsAgentsEnter(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-agents');
    if (!panel || !panel.classList.contains('active')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-agents-filter') return false;
    const list = document.getElementById('ops-agents-list');
    if (!list || list.style.display === 'none') return false;
    const selected =
        list.querySelector('.ops-row.is-selected') || list.querySelector('.ops-row');
    if (!selected) return false;
    e.preventDefault();
    selected.click();
    return true;
}

/** Enter loads the open agent soul/skill/mood into AI Chat when ready (Sessions/Knowledge parity). */
function tryOpsAgentsEnterLoad(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-agents');
    if (!panel || !panel.classList.contains('active')) return false;
    const detail = document.getElementById('ops-agent-detail');
    if (!detail || detail.hidden) return false;
    const loadBtn = document.getElementById('ops-agent-load-chat');
    if (!loadBtn || loadBtn.hidden || !opsAgentLoadText) return false;
    if (loadBtn.classList.contains('is-just-saved')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-agents-filter') return false;
    e.preventDefault();
    loadOpsAgentIntoChat();
    return true;
}

/** Enter loads the previewed schedule task / delivery summary when ready (Sessions/Runs parity). */
function tryOpsSchedulesEnterLoad(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-schedules');
    if (!panel || !panel.classList.contains('active')) return false;
    const loadBtn = document.getElementById('ops-schedules-load-chat');
    if (!loadBtn || loadBtn.hidden || !opsScheduleLoadText) return false;
    if (loadBtn.classList.contains('is-just-saved')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-schedules-filter') return false;
    e.preventDefault();
    loadOpsScheduleIntoChat();
    return true;
}

/** Enter activates the selected (or first) Schedules/delivery row. */
function tryOpsSchedulesEnter(e) {
    if (agentOpsCollapsed) return false;
    const panel = document.getElementById('ops-panel-schedules');
    if (!panel || !panel.classList.contains('active')) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA') return false;
    if (tag === 'INPUT' && t.id && t.id !== 'ops-schedules-filter') return false;
    const selected =
        panel.querySelector('.ops-row.is-selected') || panel.querySelector('.ops-row');
    if (!selected) return false;
    e.preventDefault();
    selected.click();
    return true;
}

/** ↑/↓ or j/k move selection; PageUp/PageDown jump ~5; Home/End first/last (Enter opens). */
function tryOpsArrowMoveSelection(e) {
    if (agentOpsCollapsed) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'TEXTAREA' || t?.isContentEditable) return false;
    if (tag === 'INPUT') {
        // Allow arrows in filters only when empty / not navigating text mid-word — keep simple: skip inputs.
        return false;
    }
    if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
        return false;
    }
    const panelIdByTab = {
        agents: 'ops-panel-agents',
        sessions: 'ops-panel-sessions',
        schedules: 'ops-panel-schedules',
        memory: 'ops-panel-memory',
        runs: 'ops-panel-runs',
    };
    const panel = document.getElementById(panelIdByTab[opsActiveTab] || '');
    if (!panel || !panel.classList.contains('active')) return false;
    const rows = Array.from(panel.querySelectorAll('.ops-row')).filter((el) => {
        if (el.offsetParent === null && el.style.display === 'none') return false;
        const list = el.closest('.ops-list, .ops-detail');
        if (list && list.style.display === 'none') return false;
        return true;
    });
    if (!rows.length) return false;
    let idx = rows.findIndex((r) => r.classList.contains('is-selected'));
    const page = 5;
    // No selection: ↓/j/Home/PgDn → first; ↑/k/End/PgUp → last (Monitors listbox chrome).
    if (e.key === 'Home') {
        idx = 0;
    } else if (e.key === 'End') {
        idx = rows.length - 1;
    } else if (e.key === 'PageDown') {
        idx = idx < 0 ? 0 : Math.min(idx + page, rows.length - 1);
    } else if (e.key === 'PageUp') {
        idx = idx < 0 ? rows.length - 1 : Math.max(idx - page, 0);
    } else if (e.key === 'ArrowDown' || e.key === 'j') {
        idx = idx < 0 ? 0 : Math.min(idx + 1, rows.length - 1);
    } else {
        // ArrowUp or k
        idx = idx < 0 ? rows.length - 1 : Math.max(idx - 1, 0);
    }
    panel.querySelectorAll('.ops-row.is-selected').forEach((el) => el.classList.remove('is-selected'));
    const next = rows[idx];
    next.classList.add('is-selected');
    if (typeof next.scrollIntoView === 'function') {
        next.scrollIntoView({ block: 'nearest' });
    }
    e.preventDefault();
    return true;
}

const OPS_COPY_CHIP_BY_TAB = {
    agents: 'ops-agent-copy-chip',
    sessions: 'ops-session-copy-chip',
    schedules: 'ops-schedule-copy-chip',
    memory: 'ops-memory-copy-chip',
    runs: 'ops-runs-copy-chip',
};

function setOpsRowCopyValue(btn, value) {
    if (!btn) return;
    const v = String(value || '').trim();
    if (!v || v === '—' || v === '(no id)') {
        delete btn.dataset.copyValue;
        return;
    }
    btn.dataset.copyValue = v;
}

function flashOpsRowCopied(row) {
    if (!row) return;
    document.querySelectorAll('.ops-row.is-copied').forEach((el) => {
        if (el === row) return;
        el.classList.remove('is-copied');
        if (el._opsCopiedTimer) {
            clearTimeout(el._opsCopiedTimer);
            el._opsCopiedTimer = null;
        }
    });
    row.classList.add('is-copied');
    if (row._opsCopiedTimer) clearTimeout(row._opsCopiedTimer);
    row._opsCopiedTimer = setTimeout(() => {
        row.classList.remove('is-copied');
        row._opsCopiedTimer = null;
    }, 1600);
}

function activeOpsPanelEl() {
    const panelIdByTab = {
        agents: 'ops-panel-agents',
        sessions: 'ops-panel-sessions',
        schedules: 'ops-panel-schedules',
        memory: 'ops-panel-memory',
        runs: 'ops-panel-runs',
    };
    return document.getElementById(panelIdByTab[opsActiveTab] || '');
}

/** c copies selected/previewed id (Top Processes name-copy parity). */
function tryOpsCopySelected(e) {
    if (agentOpsCollapsed) return false;
    if (e.metaKey || e.ctrlKey || e.altKey) return false;
    const t = e.target;
    const tag = (t && t.tagName) || '';
    if (tag === 'INPUT' || tag === 'TEXTAREA' || t?.isContentEditable) return false;
    if (!document.getElementById('agent-ops') && !document.querySelector('.agent-ops-tabs')) {
        return false;
    }
    const chip = document.getElementById(OPS_COPY_CHIP_BY_TAB[opsActiveTab] || '');
    const chipValue = String(chip?.dataset?.copyValue || '').trim();
    if (chip && !chip.hidden && chipValue) {
        e.preventDefault();
        chip.click();
        flashOpsRowCopied(activeOpsPanelEl()?.querySelector('.ops-row.is-selected'));
        return true;
    }
    const panel = activeOpsPanelEl();
    if (!panel || !panel.classList.contains('active')) return false;
    let row = panel.querySelector('.ops-row.is-selected');
    if (!row && t?.closest) {
        const maybe = t.closest('.ops-row');
        if (maybe && panel.contains(maybe)) row = maybe;
    }
    const value = String(row?.dataset?.copyValue || '').trim();
    if (!value) return false;
    e.preventDefault();
    void (async () => {
        const ok = await copyOpsTextToClipboard(value);
        if (!ok) return;
        flashOpsRowCopied(row);
    })();
    return true;
}

async function copyOpsTextToClipboard(text) {
    if (typeof copyTextToClipboard === 'function') {
        return copyTextToClipboard(text);
    }
    const value = String(text || '').trim();
    if (!value) return false;
    try {
        if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
            await navigator.clipboard.writeText(value);
            return true;
        }
    } catch (_) {
        /* fall through */
    }
    try {
        const ta = document.createElement('textarea');
        ta.value = value;
        ta.setAttribute('readonly', '');
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        const ok = document.execCommand('copy');
        document.body.removeChild(ta);
        return !!ok;
    } catch (_) {
        return false;
    }
}

/** Click-to-copy agent slug/id under Agents detail meta (Copied flash). */
function ensureOpsAgentCopyChip() {
    let el = document.getElementById('ops-agent-copy-chip');
    if (el) return el;
    const meta = document.getElementById('ops-agent-meta');
    if (!meta || !meta.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-agent-copy-chip';
    el.className = 'ops-session-copy-chip ops-agent-copy-chip';
    el.hidden = true;
    el.setAttribute('aria-label', 'Copy agent id');
    meta.parentNode.insertBefore(el, meta.nextSibling);
    el.addEventListener('click', async (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (el.classList.contains('is-just-saved')) return;
        const value = el.dataset.copyValue || '';
        if (!value) return;
        const ok = await copyOpsTextToClipboard(value);
        if (!ok) return;
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(el, { savedLabel: 'Copied', durationMs: 1600 });
        } else {
            const idle = el._saveFlashOriginalLabel || value;
            el._saveFlashOriginalLabel = idle;
            el.classList.add('is-just-saved');
            el.textContent = 'Copied';
            clearTimeout(el._saveFlashTimer);
            el._saveFlashTimer = setTimeout(() => {
                el.classList.remove('is-just-saved');
                el.textContent = idle;
                el._saveFlashOriginalLabel = null;
                el._saveFlashTimer = null;
            }, 1600);
        }
    });
    el.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            el.click();
        }
    });
    return el;
}

function setOpsAgentCopyChip(copyValue) {
    const el = ensureOpsAgentCopyChip();
    if (!el) return;
    const value = String(copyValue || '').trim();
    if (!value) {
        el.hidden = true;
        el.dataset.copyValue = '';
        el.classList.remove('is-just-saved');
        el.textContent = '';
        return;
    }
    el.hidden = false;
    el.dataset.copyValue = value;
    el.title = 'Click to copy agent id (c)';
    el.setAttribute('aria-label', `Copy ${value}`);
    if (!el.classList.contains('is-just-saved')) {
        el.textContent = value;
        el._saveFlashOriginalLabel = value;
    }
}

/** Refresh Load-into-chat payload from the open agent file (current tab). */
function refreshOpsAgentLoadText(opts) {
    const quiet = !!(opts && opts.quiet);
    if (!opsAgentCache) {
        opsAgentLoadText = null;
        setOpsAgentLoadChatVisible(false);
        if (!quiet) showOpsAgentLoadStatus('', true);
        return;
    }
    syncOpsAgentEditorToCache();
    const kind = opsAgentFileTab || 'soul';
    const body = String(opsAgentFileContent(kind) || '')
        .trim()
        .slice(0, 12000);
    const label = opsAgentCache.name || opsAgentCache.slug || opsAgentCache.id || 'agent';
    if (body) {
        opsAgentLoadText = `Agent ${label} (${kind}.md)\n\n${body}`;
        setOpsAgentLoadChatVisible(true);
        if (!quiet) {
            showOpsAgentLoadStatus(
                'Detail ready — Enter or “Load into AI Chat” · double-click a row also loads.',
                true
            );
        }
    } else {
        opsAgentLoadText = null;
        setOpsAgentLoadChatVisible(false);
        if (!quiet) showOpsAgentLoadStatus(`${kind}.md is empty.`, false);
    }
}

/** Load-into-chat control on Agents detail (Sessions/Knowledge parity). */
function ensureOpsAgentLoadChatBtn() {
    let el = document.getElementById('ops-agent-load-chat');
    if (el) return el;
    const actions = document.getElementById('ops-agent-edit-actions');
    const editor = document.getElementById('ops-agent-preview');
    const parent = actions || editor?.parentNode;
    if (!parent) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-agent-load-chat';
    el.className = 'btn-secondary ops-agent-load-chat';
    el.hidden = true;
    el.textContent = 'Load into AI Chat ↵';
    el.title = 'Put this agent file into AI Chat (Enter)';
    el.setAttribute('aria-label', 'Load agent file into AI Chat');
    if (actions) {
        const back = document.getElementById('ops-agent-back');
        if (back) actions.insertBefore(el, back);
        else actions.appendChild(el);
    } else {
        parent.insertBefore(el, editor?.nextSibling || null);
    }
    el.addEventListener('click', () => loadOpsAgentIntoChat());
    return el;
}

function setOpsAgentLoadChatVisible(visible) {
    const el = ensureOpsAgentLoadChatBtn();
    if (!el) return;
    el.hidden = !visible;
    refreshOpsAgentEditActionsRovingTabindex();
    if (!visible) {
        el.classList.remove('is-just-saved');
        if (!el._saveFlashOriginalLabel) {
            el.textContent = 'Load into AI Chat ↵';
        }
    }
}

function showOpsAgentLoadStatus(msg, ok) {
    let el = document.getElementById('ops-agent-load-status');
    const loadBtn = ensureOpsAgentLoadChatBtn();
    if (!el) {
        el = document.createElement('div');
        el.id = 'ops-agent-load-status';
        el.className = 'ops-row-meta';
        el.style.margin = '6px 4px 0';
        const actions = document.getElementById('ops-agent-edit-actions');
        if (actions?.parentNode) {
            actions.parentNode.insertBefore(el, actions.nextSibling);
        } else if (loadBtn?.parentNode) {
            loadBtn.parentNode.insertBefore(el, loadBtn.nextSibling);
        }
    }
    el.textContent = msg || '';
    el.style.opacity = msg ? '0.9' : '0';
    el.style.color = ok === false ? 'rgba(200,60,60,0.95)' : '';
}

/** Put the open agent soul/skill/mood into AI Chat for a quick follow-up. */
function loadOpsAgentIntoChat() {
    const loadBtn = ensureOpsAgentLoadChatBtn();
    if (loadBtn?.classList.contains('is-just-saved')) return;
    refreshOpsAgentLoadText({ quiet: true });
    const q = String(opsAgentLoadText || '').trim();
    if (!q) {
        showOpsAgentLoadStatus('Open an agent file first.', false);
        return;
    }
    const aiOff =
        document.getElementById('icon-ollama')?.style.pointerEvents === 'none' ||
        document.getElementById('ollama-section')?.style.display === 'none';
    if (aiOff) {
        showOpsAgentLoadStatus('Enable local AI agent in Settings to load into chat.', false);
        return;
    }
    const input = document.getElementById('chat-input');
    if (!input) {
        showOpsAgentLoadStatus('AI Chat input not ready — open AI Chat once, then retry.', false);
        return;
    }
    input.value = q;
    try {
        input.dispatchEvent(new Event('input', { bubbles: true }));
    } catch (_) {
        /* ignore */
    }
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
    setTimeout(() => {
        input.focus();
        try {
            const len = input.value.length;
            input.setSelectionRange(len, len);
        } catch (_) {
            /* ignore */
        }
    }, 80);
    showOpsAgentLoadStatus('Agent file loaded into AI Chat.', true);
    if (loadBtn && !loadBtn.hidden) {
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(loadBtn, { savedLabel: 'Loaded', durationMs: 1600 });
        } else {
            const idle = loadBtn.textContent || 'Load into AI Chat ↵';
            loadBtn.classList.add('is-just-saved');
            loadBtn.textContent = 'Loaded';
            setTimeout(() => {
                loadBtn.classList.remove('is-just-saved');
                loadBtn.textContent = idle;
            }, 1600);
        }
    }
}

/** Click-to-copy knowledge file path above the Knowledge preview (Copied flash). */
function ensureOpsMemoryCopyChip() {
    let el = document.getElementById('ops-memory-copy-chip');
    if (el) return el;
    const preview = document.getElementById('ops-memory-preview');
    if (!preview || !preview.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-memory-copy-chip';
    el.className = 'ops-session-copy-chip';
    el.hidden = true;
    el.setAttribute('aria-label', 'Copy knowledge file path');
    preview.parentNode.insertBefore(el, preview);
    el.addEventListener('click', async (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (el.classList.contains('is-just-saved')) return;
        const value = el.dataset.copyValue || '';
        if (!value) return;
        const ok = await copyOpsTextToClipboard(value);
        if (!ok) return;
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(el, { savedLabel: 'Copied', durationMs: 1600 });
        } else {
            const idle = el._saveFlashOriginalLabel || value;
            el._saveFlashOriginalLabel = idle;
            el.classList.add('is-just-saved');
            el.textContent = 'Copied';
            clearTimeout(el._saveFlashTimer);
            el._saveFlashTimer = setTimeout(() => {
                el.classList.remove('is-just-saved');
                el.textContent = idle;
                el._saveFlashOriginalLabel = null;
                el._saveFlashTimer = null;
            }, 1600);
        }
    });
    el.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            el.click();
        }
    });
    return el;
}

function setOpsMemoryCopyChip(copyValue) {
    const el = ensureOpsMemoryCopyChip();
    if (!el) return;
    const value = String(copyValue || '').trim();
    if (!value) {
        el.hidden = true;
        el.dataset.copyValue = '';
        el.classList.remove('is-just-saved');
        el.textContent = '';
        refreshAllOpsPreviewRowRovingTabindex({ memory: 'ops-memory-copy-chip' });
        ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[3]);
        return;
    }
    el.hidden = false;
    el.dataset.copyValue = value;
    el.title = 'Click to copy path (c)';
    el.setAttribute('aria-label', `Copy ${value}`);
    if (!el.classList.contains('is-just-saved')) {
        el.textContent = value;
        el._saveFlashOriginalLabel = value;
    }
    refreshAllOpsPreviewRowRovingTabindex({ memory: 'ops-memory-copy-chip' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[3]);
}

/** Load-into-chat control under the Knowledge preview (Sessions/Runs/Schedules parity). */
function ensureOpsMemoryLoadChatBtn() {
    let el = document.getElementById('ops-memory-load-chat');
    if (el) return el;
    const preview = document.getElementById('ops-memory-preview');
    if (!preview || !preview.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-memory-load-chat';
    el.className = 'btn-secondary ops-memory-load-chat';
    el.hidden = true;
    el.textContent = 'Load into AI Chat ↵';
    el.title = 'Put this knowledge file into AI Chat (Enter)';
    el.setAttribute('aria-label', 'Load knowledge file into AI Chat');
    preview.parentNode.insertBefore(el, preview.nextSibling);
    el.addEventListener('click', () => loadOpsMemoryIntoChat());
    return el;
}

function setOpsMemoryLoadChatVisible(visible) {
    const el = ensureOpsMemoryLoadChatBtn();
    if (!el) return;
    el.hidden = !visible;
    if (!visible) {
        el.classList.remove('is-just-saved');
        if (!el._saveFlashOriginalLabel) {
            el.textContent = 'Load into AI Chat ↵';
        }
    }
    refreshAllOpsPreviewRowRovingTabindex({ memory: 'ops-memory-load-chat' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[3]);
}

function showOpsMemoryLoadStatus(msg, ok) {
    let el = document.getElementById('ops-memory-load-status');
    const loadBtn = ensureOpsMemoryLoadChatBtn();
    if (!el) {
        el = document.createElement('div');
        el.id = 'ops-memory-load-status';
        el.className = 'ops-row-meta';
        el.style.margin = '6px 4px 0';
        if (loadBtn?.parentNode) {
            loadBtn.parentNode.insertBefore(el, loadBtn.nextSibling);
        }
    }
    el.textContent = msg || '';
    el.style.opacity = msg ? '0.9' : '0';
    el.style.color = ok === false ? 'rgba(200,60,60,0.95)' : '';
}

/** Put the previewed knowledge file into AI Chat for a quick follow-up. */
function loadOpsMemoryIntoChat() {
    const loadBtn = ensureOpsMemoryLoadChatBtn();
    if (loadBtn?.classList.contains('is-just-saved')) return;
    const q = String(opsMemoryLoadText || '').trim();
    if (!q) {
        showOpsMemoryLoadStatus('Select a knowledge file first.', false);
        return;
    }
    const aiOff =
        document.getElementById('icon-ollama')?.style.pointerEvents === 'none' ||
        document.getElementById('ollama-section')?.style.display === 'none';
    if (aiOff) {
        showOpsMemoryLoadStatus('Enable local AI agent in Settings to load into chat.', false);
        return;
    }
    const input = document.getElementById('chat-input');
    if (!input) {
        showOpsMemoryLoadStatus('AI Chat input not ready — open AI Chat once, then retry.', false);
        return;
    }
    input.value = q;
    try {
        input.dispatchEvent(new Event('input', { bubbles: true }));
    } catch (_) {
        /* ignore */
    }
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
    setTimeout(() => {
        input.focus();
        try {
            const len = input.value.length;
            input.setSelectionRange(len, len);
        } catch (_) {
            /* ignore */
        }
    }, 80);
    showOpsMemoryLoadStatus('Knowledge loaded into AI Chat.', true);
    if (loadBtn && !loadBtn.hidden) {
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(loadBtn, { savedLabel: 'Loaded', durationMs: 1600 });
        } else {
            const idle = loadBtn.textContent || 'Load into AI Chat ↵';
            loadBtn.classList.add('is-just-saved');
            loadBtn.textContent = 'Loaded';
            setTimeout(() => {
                loadBtn.classList.remove('is-just-saved');
                loadBtn.textContent = idle;
            }, 1600);
        }
    }
}

/** Click-to-copy session id / file slug above the Sessions preview (Copied flash). */
function ensureOpsSessionCopyChip() {
    let el = document.getElementById('ops-session-copy-chip');
    if (el) return el;
    const preview = document.getElementById('ops-session-preview');
    if (!preview || !preview.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-session-copy-chip';
    el.className = 'ops-session-copy-chip';
    el.hidden = true;
    el.setAttribute('aria-label', 'Copy session id');
    preview.parentNode.insertBefore(el, preview);
    el.addEventListener('click', async (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (el.classList.contains('is-just-saved')) return;
        const value = el.dataset.copyValue || '';
        if (!value) return;
        const ok = await copyOpsTextToClipboard(value);
        if (!ok) {
            showOpsSessionStatus('Could not copy.', false);
            return;
        }
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(el, { savedLabel: 'Copied', durationMs: 1600 });
        } else {
            const idle = el._saveFlashOriginalLabel || value;
            el._saveFlashOriginalLabel = idle;
            el.classList.add('is-just-saved');
            el.textContent = 'Copied';
            clearTimeout(el._saveFlashTimer);
            el._saveFlashTimer = setTimeout(() => {
                el.classList.remove('is-just-saved');
                el.textContent = idle;
                el._saveFlashOriginalLabel = null;
                el._saveFlashTimer = null;
            }, 1600);
        }
    });
    el.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            el.click();
        }
    });
    return el;
}

function setOpsSessionCopyChip(copyValue) {
    const el = ensureOpsSessionCopyChip();
    if (!el) return;
    const value = String(copyValue || '').trim();
    if (!value) {
        el.hidden = true;
        el.dataset.copyValue = '';
        el.classList.remove('is-just-saved');
        el.textContent = '';
        refreshAllOpsPreviewRowRovingTabindex({ sessions: 'ops-session-copy-chip' });
        ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[0]);
        return;
    }
    el.hidden = false;
    el.dataset.copyValue = value;
    el.title = 'Click to copy (c)';
    el.setAttribute('aria-label', `Copy ${value}`);
    if (!el.classList.contains('is-just-saved')) {
        el.textContent = value;
        el._saveFlashOriginalLabel = value;
    }
    refreshAllOpsPreviewRowRovingTabindex({ sessions: 'ops-session-copy-chip' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[0]);
}

function showOpsSessionPreview(rows, label, copyValue) {
    const preview = document.getElementById('ops-session-preview');
    const loadBtn = document.getElementById('ops-session-load-chat');
    opsSessionLoadRows = rows && rows.length ? rows : null;
    preview.hidden = false;
    preview.textContent = (label ? `${label}\n\n` : '') + formatSessionMessagesPreview(rows || []);
    if (loadBtn) loadBtn.hidden = !opsSessionLoadRows;
    setOpsSessionCopyChip(copyValue);
    refreshAllOpsPreviewRowRovingTabindex({ sessions: 'ops-session-load-chat' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[0]);
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
    const loadBtn = document.getElementById('ops-session-load-chat');
    if (loadBtn?.classList.contains('is-just-saved')) return;
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
    const msgCount = opsSessionLoadRows.length;
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
    showOpsSessionStatus(`Loaded ${msgCount} message(s) into AI Chat.`, true);
    // Control feedback (status line alone is easy to miss) — block double Enter/click/dblclick.
    if (loadBtn && !loadBtn.hidden) {
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(loadBtn, { savedLabel: 'Loaded', durationMs: 1600 });
        } else {
            const idle = loadBtn.textContent || 'Load into AI Chat ↵';
            loadBtn.classList.add('is-just-saved');
            loadBtn.textContent = 'Loaded';
            setTimeout(() => {
                loadBtn.classList.remove('is-just-saved');
                loadBtn.textContent = idle;
            }, 1600);
        }
    }
}

function renderOpsLive(rows) {
    const el = document.getElementById('ops-live-sessions');
    if (!el) return;
    el.innerHTML = '';
    const all = rows || [];
    opsLiveCache = all;
    if (opsSessionKindFilter === 'files') {
        paintOpsSessionFilterFromCaches();
        return;
    }
    const filtered = all.filter((r) =>
        sessionRowMatchesFilter(
            `${r.source} ${r.session_id} ${r.preview || ''} ${r.last_activity || ''}`
        )
    );
    if (!all.length) {
        el.innerHTML = opsTabEmptyHtml(
          'No live sessions',
          'A chat appears here while an agent runs',
          { action: 'ai-chat', label: 'Open AI Chat' }
        );
        paintOpsSessionFilterFromCaches();
        return;
    }
    if (!filtered.length) {
        el.innerHTML = opsFilterMissHtml('No live sessions match filter', 'sessions');
        paintOpsSessionFilterFromCaches();
        return;
    }
    filtered.forEach((r) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(r.source)} · ${r.session_id}</div><div class="ops-row-meta">${r.message_count} msgs · ${escapeHtml(r.last_activity)}${r.preview ? ` · ${escapeHtml(r.preview)}` : ''}</div></div>`;
        setOpsRowCopyValue(btn, r.session_id);
        const openLive = async () => {
            try {
                const msgs = await invoke('read_live_session_messages', {
                    source: r.source,
                    sessionId: r.session_id,
                });
                markOpsSessionRowSelected(btn);
                showOpsSessionPreview(msgs, `Live ${r.source} · ${r.session_id}`, r.session_id);
                showOpsSessionStatus('Preview ready — Enter or “Load into AI Chat” · double-click also loads.', true);
            } catch (err) {
                showOpsSessionPreview([], String(err), null);
                showOpsSessionStatus(String(err), false);
            }
        };
        btn.addEventListener('click', openLive);
        btn.addEventListener('dblclick', async () => {
            await openLive();
            loadOpsSessionIntoChat();
        });
        btn.title = 'Click to preview · c copies id · Enter / double-click to load into AI Chat';
        el.appendChild(btn);
    });
    paintOpsSessionFilterFromCaches();
}

function renderOpsSessionFiles(files) {
    const el = document.getElementById('ops-session-files');
    const preview = document.getElementById('ops-session-preview');
    const loadBtn = document.getElementById('ops-session-load-chat');
    if (!el) return;
    el.innerHTML = '';
    preview.hidden = true;
    if (loadBtn) loadBtn.hidden = true;
    opsSessionLoadRows = null;
    setOpsSessionCopyChip(null);
    const all = files || [];
    opsSessionFilesCache = all;
    if (opsSessionKindFilter === 'live') {
        paintOpsSessionFilterFromCaches();
        return;
    }
    const filtered = all.filter((f) =>
        sessionRowMatchesFilter(
            `${f.slug || ''} ${f.name || ''} ${f.source_hint || ''} ${f.preview || ''}`
        )
    );
    if (!all.length) {
        el.innerHTML = opsTabEmptyHtml(
          'No saved session files',
          'session-memory-*.md appears after Discord chats land on disk',
          { action: 'ai-chat', label: 'Open AI Chat' }
        );
        paintOpsSessionFilterFromCaches();
        return;
    }
    if (!filtered.length) {
        el.innerHTML = opsFilterMissHtml('No session files match filter', 'sessions');
        paintOpsSessionFilterFromCaches();
        return;
    }
    filtered.forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.slug || f.name)}</div><div class="ops-row-meta">${escapeHtml(f.source_hint)} · ${fmtBytes(f.size_bytes)} · ${fmtAge(f.modified_ms)}${f.preview ? ` · ${escapeHtml(f.preview)}` : ''}</div></div>`;
        setOpsRowCopyValue(btn, f.slug || f.name);
        const openFile = async () => {
            try {
                const copyId = f.slug || f.name || '';
                const msgs = await invoke('read_session_file_messages', { path: f.path });
                if (msgs && msgs.length) {
                    markOpsSessionRowSelected(btn);
                    showOpsSessionPreview(msgs, f.name, copyId);
                    showOpsSessionStatus('Preview ready — Enter or “Load into AI Chat” · double-click also loads.', true);
                } else {
                    markOpsSessionRowSelected(btn);
                    const text = await invoke('read_session_file', { path: f.path });
                    preview.hidden = false;
                    preview.textContent = text.slice(0, 12000);
                    opsSessionLoadRows = null;
                    if (loadBtn) loadBtn.hidden = true;
                    setOpsSessionCopyChip(copyId);
                    showOpsSessionStatus('No parseable turns — raw file shown.', false);
                }
            } catch (err) {
                preview.hidden = false;
                preview.textContent = String(err);
                opsSessionLoadRows = null;
                if (loadBtn) loadBtn.hidden = true;
                setOpsSessionCopyChip(null);
                showOpsSessionStatus(String(err), false);
            }
        };
        btn.addEventListener('click', openFile);
        btn.addEventListener('dblclick', async () => {
            await openFile();
            loadOpsSessionIntoChat();
        });
        btn.title = 'Click to preview · c copies id · Enter / double-click to load into AI Chat';
        el.appendChild(btn);
    });
    paintOpsSessionFilterFromCaches();
}

function renderOpsMemory(files) {
    const el = document.getElementById('ops-memory-list');
    const preview = document.getElementById('ops-memory-preview');
    el.innerHTML = '';
    preview.hidden = true;
    opsMemoryLoadText = null;
    setOpsMemoryCopyChip(null);
    setOpsMemoryLoadChatVisible(false);
    showOpsMemoryLoadStatus('', true);
    ensureOpsMemoryKindChips();
    const all = files || [];
    const kindPool = all.filter((f) => memoryRowMatchesKind(f));
    const filtered = kindPool.filter((f) =>
        memoryRowMatchesFilter(`${f.name || ''} ${f.kind || ''} ${f.path || ''}`)
    );
    if (!all.length) {
        el.innerHTML = opsTabEmptyHtml(
          'No knowledge files yet',
          'soul.md and memory/*.md live under ~/.mac-stats'
        );
        paintOpsFilterMatch('ops-memory-filter', 0, 0, opsMemoryFilterQ);
        return;
    }
    if (!filtered.length) {
        el.innerHTML = opsFilterMissHtml('No knowledge files match filter', 'memory');
        paintOpsFilterMatch('ops-memory-filter', kindPool.length, 0, opsMemoryFilterQ);
        return;
    }
    filtered.forEach((f) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(f.name)}</div><div class="ops-row-meta">${escapeHtml(f.kind)} · ${f.line_count} lines · ${fmtBytes(f.size_bytes)}</div></div>`;
        setOpsRowCopyValue(btn, f.path || f.name);
        const openFile = async () => {
            document
                .querySelectorAll('#ops-memory-list .ops-row.is-selected')
                .forEach((row) => row.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            const copyPath = f.path || f.name || '';
            const label = f.name || f.path || 'knowledge';
            try {
                const text = await invoke('read_memory_file', { path: f.path });
                const body = String(text || '').slice(0, 12000);
                preview.hidden = false;
                preview.textContent = body;
                setOpsMemoryCopyChip(copyPath);
                const loadBody = body.trim();
                if (loadBody) {
                    opsMemoryLoadText = `Knowledge: ${label}\n\n${loadBody}`;
                    setOpsMemoryLoadChatVisible(true);
                    showOpsMemoryLoadStatus(
                        'Preview ready — Enter or “Load into AI Chat” · double-click also loads.',
                        true
                    );
                } else {
                    opsMemoryLoadText = null;
                    setOpsMemoryLoadChatVisible(false);
                    showOpsMemoryLoadStatus('File is empty.', false);
                }
            } catch (err) {
                preview.hidden = false;
                preview.textContent = String(err);
                opsMemoryLoadText = null;
                setOpsMemoryCopyChip(null);
                setOpsMemoryLoadChatVisible(false);
                showOpsMemoryLoadStatus(String(err), false);
            }
        };
        btn.addEventListener('click', openFile);
        btn.addEventListener('dblclick', async () => {
            await openFile();
            loadOpsMemoryIntoChat();
        });
        btn.title = 'Click to preview · c copies path · Enter / double-click to load into AI Chat';
        el.appendChild(btn);
    });
    paintOpsFilterMatch('ops-memory-filter', kindPool.length, filtered.length, opsMemoryFilterQ);
    if ((opsMemoryFilterQ || opsMemoryKindFilter !== 'all') && !el.querySelector('.ops-row')) {
        el.innerHTML = opsFilterMissHtml('No knowledge files match filter', 'memory');
    }
}

/** Ensure Runs preview pane exists (themes + dashboard; create if HTML is stale). */
function ensureOpsRunsPreview() {
    let preview = document.getElementById('ops-runs-preview');
    if (preview) return preview;
    const panel = document.getElementById('ops-panel-runs');
    const list = document.getElementById('ops-runs-list');
    if (!panel && !list) return null;
    preview = document.createElement('pre');
    preview.id = 'ops-runs-preview';
    preview.className = 'ops-preview';
    preview.hidden = true;
    if (list && list.parentNode) {
        list.parentNode.insertBefore(preview, list.nextSibling);
    } else if (panel) {
        panel.appendChild(preview);
    }
    return preview;
}

/** Click-to-copy run request id above the Runs preview (Copied flash). */
function ensureOpsRunsCopyChip() {
    let el = document.getElementById('ops-runs-copy-chip');
    if (el) return el;
    const preview = ensureOpsRunsPreview();
    if (!preview || !preview.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-runs-copy-chip';
    el.className = 'ops-session-copy-chip ops-runs-copy-chip';
    el.hidden = true;
    el.setAttribute('aria-label', 'Copy run request id');
    preview.parentNode.insertBefore(el, preview);
    el.addEventListener('click', async (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (el.classList.contains('is-just-saved')) return;
        const value = el.dataset.copyValue || '';
        if (!value) return;
        const ok = await copyOpsTextToClipboard(value);
        if (!ok) return;
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(el, { savedLabel: 'Copied', durationMs: 1600 });
        } else {
            const idle = el._saveFlashOriginalLabel || value;
            el._saveFlashOriginalLabel = idle;
            el.classList.add('is-just-saved');
            el.textContent = 'Copied';
            clearTimeout(el._saveFlashTimer);
            el._saveFlashTimer = setTimeout(() => {
                el.classList.remove('is-just-saved');
                el.textContent = idle;
                el._saveFlashOriginalLabel = null;
                el._saveFlashTimer = null;
            }, 1600);
        }
    });
    el.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            el.click();
        }
    });
    return el;
}

function setOpsRunsCopyChip(copyValue) {
    const el = ensureOpsRunsCopyChip();
    if (!el) return;
    const value = String(copyValue || '').trim();
    if (!value || value === '—') {
        el.hidden = true;
        el.dataset.copyValue = '';
        el.classList.remove('is-just-saved');
        el.textContent = '';
        refreshAllOpsPreviewRowRovingTabindex({ runs: 'ops-runs-copy-chip' });
        ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[1]);
        return;
    }
    el.hidden = false;
    el.dataset.copyValue = value;
    el.title = 'Click to copy request id (c)';
    el.setAttribute('aria-label', `Copy ${value}`);
    if (!el.classList.contains('is-just-saved')) {
        el.textContent = value;
        el._saveFlashOriginalLabel = value;
    }
    refreshAllOpsPreviewRowRovingTabindex({ runs: 'ops-runs-copy-chip' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[1]);
}

/** Load-into-chat control under the Runs preview (Sessions parity). */
function ensureOpsRunsLoadChatBtn() {
    let el = document.getElementById('ops-runs-load-chat');
    if (el) return el;
    const preview = ensureOpsRunsPreview();
    if (!preview || !preview.parentNode) return null;
    el = document.createElement('button');
    el.type = 'button';
    el.id = 'ops-runs-load-chat';
    el.className = 'btn-secondary ops-runs-load-chat';
    el.hidden = true;
    el.textContent = 'Load into AI Chat ↵';
    el.title = 'Put this run’s question into AI Chat (Enter)';
    el.setAttribute('aria-label', 'Load run question into AI Chat');
    preview.parentNode.insertBefore(el, preview.nextSibling);
    el.addEventListener('click', () => loadOpsRunIntoChat());
    return el;
}

function setOpsRunsLoadChatVisible(visible) {
    const el = ensureOpsRunsLoadChatBtn();
    if (!el) return;
    el.hidden = !visible;
    if (!visible) {
        el.classList.remove('is-just-saved');
        if (!el._saveFlashOriginalLabel) {
            el.textContent = 'Load into AI Chat ↵';
        }
    }
    refreshAllOpsPreviewRowRovingTabindex({ runs: 'ops-runs-load-chat' });
    ensureOpsPreviewRowKbHint(OPS_PREVIEW_ROW_SPECS[1]);
}

function showOpsRunLoadStatus(msg, ok) {
    let el = document.getElementById('ops-runs-load-status');
    const loadBtn = ensureOpsRunsLoadChatBtn();
    if (!el) {
        el = document.createElement('div');
        el.id = 'ops-runs-load-status';
        el.className = 'ops-row-meta';
        el.style.margin = '6px 4px 0';
        if (loadBtn?.parentNode) {
            loadBtn.parentNode.insertBefore(el, loadBtn.nextSibling);
        }
    }
    el.textContent = msg || '';
    el.style.opacity = msg ? '0.9' : '0';
    el.style.color = ok === false ? 'rgba(200,60,60,0.95)' : '';
}

/** Put the previewed run question into AI Chat for a quick retry. */
function loadOpsRunIntoChat() {
    const loadBtn = ensureOpsRunsLoadChatBtn();
    if (loadBtn?.classList.contains('is-just-saved')) return;
    const q = String(opsRunLoadQuestion || '').trim();
    if (!q) {
        showOpsRunLoadStatus('Select a run with a question first.', false);
        return;
    }
    const aiOff =
        document.getElementById('icon-ollama')?.style.pointerEvents === 'none' ||
        document.getElementById('ollama-section')?.style.display === 'none';
    if (aiOff) {
        showOpsRunLoadStatus('Enable local AI agent in Settings to load into chat.', false);
        return;
    }
    const input = document.getElementById('chat-input');
    if (!input) {
        showOpsRunLoadStatus('AI Chat input not ready — open AI Chat once, then retry.', false);
        return;
    }
    input.value = q;
    try {
        input.dispatchEvent(new Event('input', { bubbles: true }));
    } catch (_) {
        /* ignore */
    }
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
    setTimeout(() => {
        input.focus();
        try {
            const len = input.value.length;
            input.setSelectionRange(len, len);
        } catch (_) {
            /* ignore */
        }
    }, 80);
    showOpsRunLoadStatus('Question loaded into AI Chat.', true);
    if (loadBtn && !loadBtn.hidden) {
        if (typeof window.flashSaveButton === 'function') {
            window.flashSaveButton(loadBtn, { savedLabel: 'Loaded', durationMs: 1600 });
        } else {
            const idle = loadBtn.textContent || 'Load into AI Chat ↵';
            loadBtn.classList.add('is-just-saved');
            loadBtn.textContent = 'Loaded';
            setTimeout(() => {
                loadBtn.classList.remove('is-just-saved');
                loadBtn.textContent = idle;
            }, 1600);
        }
    }
}

/** Clickable Runs Insights lines in DOM order (Discord · Digest open · Slowest · Candidates). */
function getOpsInsightsToolbarItems() {
    const card = document.getElementById('ops-runs-insights');
    if (!card || card.hidden) return [];
    return Array.from(card.querySelectorAll('.ops-insight-line.is-clickable')).filter((el) => {
        if (!el || el.hidden) return false;
        return el.getClientRects().length > 0 || el.offsetParent !== null;
    });
}

function refreshOpsInsightsRovingTabindex(preferred) {
    const items = getOpsInsightsToolbarItems();
    if (!items.length) return;
    const focused = items.find((el) => el === document.activeElement);
    const current =
        (preferred && items.includes(preferred) && preferred) ||
        focused ||
        items.find((el) => el.tabIndex === 0) ||
        items[0];
    for (const el of items) {
        el.tabIndex = el === current ? 0 : -1;
    }
}

function ensureOpsInsightsKbHint() {
    const card = document.getElementById('ops-runs-insights');
    if (!card) return;
    let hint = document.getElementById('ops-insights-kb-hint');
    if (!hint) {
        hint = document.createElement('div');
        hint.id = 'ops-insights-kb-hint';
        hint.className = 'ops-insights-kb-hint';
        hint.setAttribute('aria-hidden', 'true');
        card.appendChild(hint);
    }
    const items = getOpsInsightsToolbarItems();
    hint.hidden = items.length < 2;
    hint.textContent =
        '← → / h l · Home/End move · Enter loads chat · click previews';
}

/**
 * Runs Insights toolbar keyboard — focus Discord · Digest open · Slowest ·
 * Candidates lines, then ←→ / h l / Home/End (filter-row / preview-row parity).
 * Enter/Space keeps line preview; Enter also loads into AI Chat.
 */
function ensureOpsInsightsToolbarKeyboard() {
    const card = document.getElementById('ops-runs-insights');
    if (!card) return;
    ensureOpsInsightsKbHint();
    refreshOpsInsightsRovingTabindex();
    if (card.dataset.opsInsightsKbWired === '1') return;
    card.dataset.opsInsightsKbWired = '1';
    card.addEventListener('focusin', (e) => {
        const items = getOpsInsightsToolbarItems();
        if (items.includes(e.target)) {
            refreshOpsInsightsRovingTabindex(e.target);
            ensureOpsInsightsKbHint();
            const summary = opsInsightLineSummary.get(e.target);
            if (summary) previewOpsRunFromInsight(summary, e.target);
        }
    });
    card.addEventListener('keydown', (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const items = getOpsInsightsToolbarItems();
        if (!items.length) return;
        const idx = items.indexOf(document.activeElement);
        if (idx < 0) return;
        if (e.key === 'Enter' || e.key === ' ') return;
        let next = -1;
        if (
            e.key === 'ArrowRight' ||
            e.key === 'l' ||
            e.key === 'ArrowDown' ||
            e.key === 'j'
        ) {
            next = Math.min(idx + 1, items.length - 1);
        } else if (
            e.key === 'ArrowLeft' ||
            e.key === 'h' ||
            e.key === 'ArrowUp' ||
            e.key === 'k'
        ) {
            next = Math.max(idx - 1, 0);
        } else if (e.key === 'Home') {
            next = 0;
        } else if (e.key === 'End') {
            next = items.length - 1;
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshOpsInsightsRovingTabindex(items[next]);
        const summary = opsInsightLineSummary.get(items[next]);
        if (summary) previewOpsRunFromInsight(summary, items[next]);
        items[next].focus();
    });
    const items = getOpsInsightsToolbarItems();
    if (items.length >= 2) {
        if (!card.getAttribute('role')) card.setAttribute('role', 'toolbar');
        if (!card.getAttribute('aria-label')) {
            card.setAttribute('aria-label', 'Runs Insights');
        }
    }
}

/** Preview a Slowest / Candidate insight line (select matching Runs row when present). */
function previewOpsRunFromInsight(summary, insightLine) {
    const card = document.getElementById('ops-runs-insights');
    card?.querySelectorAll('.ops-insight-line.is-selected').forEach((node) => {
        node.classList.remove('is-selected');
    });
    if (insightLine) insightLine.classList.add('is-selected');

    const el = document.getElementById('ops-runs-list');
    const rid = String(summary?.request_id || '').trim();
    const qPreview = String(summary?.question_preview || '').trim();
    let matched = null;
    if (el) {
        el.querySelectorAll('.ops-row.is-selected').forEach((node) => node.classList.remove('is-selected'));
        if (rid) {
            matched =
                Array.from(el.querySelectorAll('.ops-row')).find((btn) => btn.dataset.requestId === rid) ||
                null;
        }
        if (!matched && qPreview) {
            const needle = qPreview.slice(0, 48);
            el.querySelectorAll('.ops-row[data-question-preview]').forEach((btn) => {
                if (matched) return;
                const hay = String(btn.dataset.questionPreview || '');
                if (hay.startsWith(needle) || needle.startsWith(hay.slice(0, 48))) {
                    matched = btn;
                }
            });
        }
        if (matched) {
            matched.classList.add('is-selected');
            try {
                matched.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
            } catch (_) {
                /* ignore */
            }
        }
    }

    const q = qPreview && qPreview !== '(empty)' ? qPreview : '';
    showOpsRunPreview(formatOpsRunPreview(summary), summary?.request_id, q);
}

/** Wire Slowest / Candidate lines: click preview, Enter/dblclick load (list-row parity). */
function wireOpsInsightRunLine(lineEl, summary) {
    if (!lineEl || !summary) return;
    opsInsightLineSummary.set(lineEl, summary);
    lineEl.classList.add('is-clickable');
    lineEl.setAttribute('role', 'button');
    lineEl.tabIndex = -1;
    lineEl.title = 'Click to preview · Enter / double-click to load question into AI Chat';
    const open = () => previewOpsRunFromInsight(summary, lineEl);
    lineEl.addEventListener('click', open);
    lineEl.addEventListener('dblclick', (e) => {
        e.preventDefault();
        open();
        loadOpsRunIntoChat();
    });
    lineEl.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            open();
            if (e.key === 'Enter') loadOpsRunIntoChat();
        }
    });
}

function formatOpsCandidateAsSummary(c) {
    return {
        ts: '',
        lane: c?.lane || '—',
        wall_ms: typeof c?.wall_ms === 'number' ? c.wall_ms : 0,
        tools: [],
        question_preview: c?.question_preview || '',
        ok: true,
        request_id: c?.request_id || '',
        _candidateKind: c?.kind || '',
        _candidateReason: c?.reason || '',
    };
}

/** Digest-open hint → run preview shape (Load into AI Chat). */
function formatOpsDigestHintAsSummary(hint) {
    const text = String(hint || '').trim();
    return {
        ts: '',
        lane: 'digest',
        wall_ms: 0,
        tools: [],
        question_preview: text,
        ok: true,
        request_id: '',
        _candidateKind: 'digest-open',
        _candidateReason: 'Digester open candidate',
    };
}

/** Discord gateway status → run preview shape (Load into AI Chat). */
function formatOpsDiscordGatewayAsSummary(gateway) {
    const text = String(gateway || '').trim();
    return {
        ts: '',
        lane: 'discord',
        wall_ms: 0,
        tools: [],
        question_preview: text
            ? `Discord gateway status:\n${text}`
            : 'Discord gateway status unknown',
        ok: true,
        request_id: '',
        _candidateKind: 'discord-gateway',
        _candidateReason: 'Agent Ops Discord health',
    };
}

/** Open Runs + preview Discord gateway insight (health Discord parity with Digest). */
function openOpsDiscordGatewayPreviewNavigate(gateway) {
    const text = String(gateway || '').trim();
    if (!text) return false;
    if (agentOpsCollapsed) applyOpsCollapsed(false);
    selectOpsTab('runs');
    const card = document.getElementById('ops-runs-insights');
    let line = card?.querySelector('.ops-insight-line[data-discord-gateway="1"]') || null;
    previewOpsRunFromInsight(formatOpsDiscordGatewayAsSummary(text), line);
    if (line) {
        try {
            line.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        } catch (_) {
            /* ignore */
        }
    }
    return true;
}

/** Show full run turn details (list rows truncate question / tools). */
function showOpsRunPreview(text, requestId, question) {
    const preview = ensureOpsRunsPreview();
    if (!preview) return;
    const body = String(text || '').trim();
    if (!body) {
        preview.hidden = true;
        preview.textContent = '';
        opsRunLoadQuestion = null;
        setOpsRunsCopyChip(null);
        setOpsRunsLoadChatVisible(false);
        showOpsRunLoadStatus('', true);
        return;
    }
    preview.hidden = false;
    preview.textContent = body.slice(0, 12000);
    setOpsRunsCopyChip(requestId);
    const q = String(question || '').trim();
    opsRunLoadQuestion = q && q !== '(empty)' ? q : null;
    setOpsRunsLoadChatVisible(!!opsRunLoadQuestion);
    if (opsRunLoadQuestion) {
        showOpsRunLoadStatus('Preview ready — Enter or “Load into AI Chat” · double-click also loads.', true);
    } else {
        showOpsRunLoadStatus('', true);
    }
}

function formatOpsRunPreview(r) {
    const q = String(r?.question_preview || '').trim() || '(empty)';
    const lane = r?.lane || '—';
    const wall = typeof r?.wall_ms === 'number' ? `${r.wall_ms} ms` : '—';
    const tools = (r?.tools || []).length ? (r.tools || []).join(', ') : '—';
    const ok = r?.ok ? 'ok' : 'FAIL';
    const ts = r?.ts || '—';
    const rid = String(r?.request_id || '').trim() || '—';
    const kind = String(r?._candidateKind || '').trim();
    const reason = String(r?._candidateReason || '').trim();
    const head = kind
        ? `Candidate (${kind})${reason ? `\nWhy: ${reason}` : ''}\n`
        : `Run (${ok})\n`;
    return `${head}Lane: ${lane}\nWall: ${wall}\nWhen: ${ts}\nRequest: ${rid}\nTools: ${tools}\n\nQuestion:\n${q}`;
}

function renderOpsRuns(insights) {
    const card = document.getElementById('ops-runs-insights');
    const el = document.getElementById('ops-runs-list');
    el.innerHTML = '';
    if (card) card.innerHTML = '';
    showOpsRunPreview('');
    const gateway = insights?.discord_gateway || '';
    function appendOpsDiscordGatewayLine(parent, gwText) {
        const text = String(gwText || '').trim();
        if (!parent || !text) return;
        const sub = document.createElement('div');
        sub.className = 'ops-insight-sub';
        sub.textContent = 'Discord';
        parent.appendChild(sub);
        const line = document.createElement('div');
        line.className = 'ops-insight-line';
        line.textContent = text;
        line.dataset.discordGateway = '1';
        wireOpsInsightRunLine(line, formatOpsDiscordGatewayAsSummary(text));
        parent.appendChild(line);
    }
    if (!insights || !insights.turns) {
        if (card && gateway) {
            card.innerHTML = `<div class="ops-insight-title">Insights</div>`;
            appendOpsDiscordGatewayLine(card, gateway);
            const digestMeta = document.createElement('div');
            digestMeta.className = 'ops-row-meta';
            digestMeta.textContent = `Digest: ${insights.digest_open_count ?? 0} open · ${insights.digest_stale_count ?? 0} stale${insights.digest_source ? ` · ${insights.digest_source}` : ''}`;
            card.appendChild(digestMeta);
            const empty = document.createElement('div');
            empty.className = 'ops-empty ops-empty-compact ops-empty-tab ops-empty-filter-miss';
            empty.innerHTML =
                `<div class="ops-empty-filter-msg">No runs yet</div>` +
                `<div class="ops-empty-tab-hint">Turns land after Discord or chat</div>` +
                `<button type="button" class="ops-clear-filter" data-ops-open-ai-chat="1">Open AI Chat</button>`;
            card.appendChild(empty);
        } else {
            el.innerHTML = opsTabEmptyHtml(
              'No runs yet',
              'Turns land in ~/.mac-stats/runs.jsonl after Discord or chat',
              { action: 'ai-chat', label: 'Open AI Chat' }
            );
        }
        paintOpsFilterMatch('ops-runs-filter', 0, 0, opsRunsFilterQ);
        ensureOpsRunsLaneChips();
        return;
    }
    const lanes = (insights.by_lane || []).map(([k, v]) => `${k}:${v}`).join(' · ');
    const tools = (insights.by_tool || [])
        .slice(0, 6)
        .map(([k, v]) => `${k}×${v}`)
        .join(', ');
    if (card) {
        card.innerHTML = `
            <div class="ops-insight-title">Insights</div>
            <div class="ops-row-meta">${insights.ok_count}/${insights.turns} ok · fail ${insights.fail_count || 0} · mean ${insights.mean_ms} ms · max ${insights.max_ms} ms</div>
            <div class="ops-row-meta">Digest: ${insights.digest_open_count ?? 0} open · ${insights.digest_stale_count ?? 0} stale${insights.digest_source ? ` · ${escapeHtml(insights.digest_source)}` : ''}${insights.digest_generated_at ? ` · ${escapeHtml(String(insights.digest_generated_at).slice(0, 19))}` : ''}</div>
        `;
        appendOpsDiscordGatewayLine(card, gateway);
        const digestHints = insights.digest_open_hints || [];
        if (digestHints.length) {
            const sub = document.createElement('div');
            sub.className = 'ops-insight-sub';
            sub.textContent = 'Digest open';
            card.appendChild(sub);
            digestHints.slice(0, 3).forEach((h) => {
                const text = String(h || '').trim();
                if (!text) return;
                const line = document.createElement('div');
                line.className = 'ops-insight-line';
                line.textContent = text;
                line.dataset.digestHint = text;
                wireOpsInsightRunLine(line, formatOpsDigestHintAsSummary(text));
                card.appendChild(line);
            });
        } else if (Number(insights.digest_open_count) === 0) {
            const sub = document.createElement('div');
            sub.className = 'ops-insight-sub';
            sub.textContent = 'Digest open';
            card.appendChild(sub);
            const empty = document.createElement('div');
            empty.className = 'ops-empty ops-empty-compact';
            empty.textContent =
                'Queue clear — overnight must still ship design review / standing backlog (quiet is a fail)';
            card.appendChild(empty);
        }
        const lanesEl = document.createElement('div');
        lanesEl.className = 'ops-row-meta';
        lanesEl.textContent = `Lanes: ${lanes || '—'}`;
        card.appendChild(lanesEl);
        const toolsEl = document.createElement('div');
        toolsEl.className = 'ops-row-meta';
        toolsEl.textContent = `Top tools: ${tools || '—'}`;
        card.appendChild(toolsEl);
        const slowRows = insights.slowest || [];
        if (slowRows.length) {
            const sub = document.createElement('div');
            sub.className = 'ops-insight-sub';
            sub.textContent = 'Slowest';
            card.appendChild(sub);
            slowRows.slice(0, 3).forEach((s) => {
                const line = document.createElement('div');
                line.className = 'ops-insight-line';
                line.textContent = `${s.wall_ms} ms · ${s.lane || '—'} · ${s.question_preview || '(empty)'}`;
                wireOpsInsightRunLine(line, s);
                card.appendChild(line);
            });
        }
        const candRows = insights.candidates || [];
        if (candRows.length) {
            const sub = document.createElement('div');
            sub.className = 'ops-insight-sub';
            sub.textContent = 'Candidates';
            card.appendChild(sub);
            candRows.slice(0, 4).forEach((c) => {
                const line = document.createElement('div');
                line.className = 'ops-insight-line';
                line.innerHTML = `<span class="ops-badge">${escapeHtml(c.kind)}</span> ${c.wall_ms} ms — ${escapeHtml(c.reason)} · <em>${escapeHtml(c.question_preview)}</em>`;
                wireOpsInsightRunLine(line, formatOpsCandidateAsSummary(c));
                card.appendChild(line);
            });
        }
    }
    ensureOpsRunsLaneChips();
    const recentAll = insights.recent || [];
    const kindPool = recentAll.filter((r) => runsRowMatchesLane(r));
    let shown = 0;
    kindPool.forEach((r) => {
        const toolsJoined = (r.tools || []).join(', ') || '—';
        if (
            !runsRowMatchesFilter(
                `${r.question_preview || ''} ${r.lane || ''} ${toolsJoined} ${r.ok ? 'ok' : 'fail'}`
            )
        ) {
            return;
        }
        shown += 1;
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ops-row';
        const rid = String(r?.request_id || '').trim();
        if (rid) btn.dataset.requestId = rid;
        btn.dataset.questionPreview = String(r?.question_preview || '');
        setOpsRowCopyValue(btn, rid);
        btn.innerHTML = `<div><div class="ops-row-title">${escapeHtml(r.question_preview || '(empty)')}</div><div class="ops-row-meta">${escapeHtml(r.lane)} · ${r.wall_ms} ms · ${escapeHtml(toolsJoined)}${r.ok ? '' : ' · FAIL'}</div></div>`;
        btn.title = 'Click to preview · c copies id · Enter / double-click to load question into AI Chat';
        const openPreview = () => {
            document
                .getElementById('ops-runs-insights')
                ?.querySelectorAll('.ops-insight-line.is-selected')
                .forEach((node) => node.classList.remove('is-selected'));
            el.querySelectorAll('.ops-row.is-selected').forEach((node) => node.classList.remove('is-selected'));
            btn.classList.add('is-selected');
            const q = String(r?.question_preview || '').trim();
            showOpsRunPreview(formatOpsRunPreview(r), r?.request_id, q);
        };
        btn.addEventListener('click', openPreview);
        btn.addEventListener('dblclick', (e) => {
            e.preventDefault();
            openPreview();
            loadOpsRunIntoChat();
        });
        el.appendChild(btn);
    });
    paintOpsFilterMatch('ops-runs-filter', kindPool.length, shown, opsRunsFilterQ);
    paintOpsRunsLaneChips();
    if ((opsRunsFilterQ || opsRunsLaneFilter !== 'all') && !el.querySelector('.ops-row')) {
        el.innerHTML = opsFilterMissHtml('No runs match filter', 'runs');
        showOpsRunPreview('');
    }
    ensureOpsInsightsToolbarKeyboard();
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
    if (typeof window.syncSectionIcon === 'function') {
      window.syncSectionIcon('icon-agent-ops', open);
    } else {
      icon.classList.toggle('section-open', open);
      icon.classList.toggle('status-good', open);
      icon.setAttribute('aria-pressed', open ? 'true' : 'false');
    }
    if (open) icon.classList.remove('status-warning');
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

  function restoreAgentOpsTab() {
    let tab = 'agents';
    if (typeof window.getCpuUiSectionValue === 'function') {
      tab = window.getCpuUiSectionValue('agent_ops_tab', 'agents') || 'agents';
    } else {
      try {
        tab = localStorage.getItem('agent_ops_tab') || 'agents';
      } catch (_) {}
    }
    if (document.getElementById(`ops-panel-${tab}`)) {
      selectOpsTab(tab);
    }
  }

  function applyOpsCollapsed(collapsed) {
    agentOpsCollapsed = collapsed;
    const section = document.getElementById('agent-ops-section') || document.querySelector('.agent-ops-section');
    const content = document.getElementById('agent-ops-content');
    const btn = document.getElementById('agent-ops-collapse-btn');
    if (typeof window.setIconPaneVisibility === 'function') {
      window.setIconPaneVisibility(section, content, collapsed, null);
    } else if (section) {
      section.classList.toggle('collapsed', collapsed);
      section.style.display = collapsed ? 'none' : '';
      if (collapsed) section.setAttribute('aria-hidden', 'true');
      else section.removeAttribute('aria-hidden');
      if (content) {
        content.classList.toggle('collapsed', collapsed);
        if (content.classList.contains('section-content-collapsible')) {
          content.style.display = collapsed ? 'none' : 'block';
        } else {
          content.style.display = collapsed ? 'none' : '';
        }
      }
    }
    if (btn) btn.textContent = collapsed ? '+' : '−';
    const header = document.getElementById('agent-ops-header');
    if (header) header.setAttribute('aria-expanded', String(!collapsed));
    stopOpsGlancePoll();
    syncOpsCollapsedGlance();
    syncOpsIcon();
    if (typeof window.setSectionCollapsed === 'function') {
      window.setSectionCollapsed('agent_ops_collapsed', collapsed);
    } else {
      try {
        localStorage.setItem('agent_ops_collapsed', collapsed ? 'true' : 'false');
      } catch (_) {}
    }
    if (collapsed) {
      stopAgentOpsAutoRefresh();
    } else {
      restoreAgentOpsTab();
      refreshAgentOps();
      startAgentOpsAutoRefresh();
      requestAnimationFrame(() => {
        section?.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
      });
    }
  }
  window.applyOpsCollapsed = applyOpsCollapsed;

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

    if (header && !header.dataset.collapseA11y) {
      header.dataset.collapseA11y = '1';
      header.setAttribute('role', 'button');
      header.setAttribute('tabindex', '0');
      header.setAttribute('aria-controls', 'agent-ops-content');
      header.setAttribute('aria-expanded', String(!agentOpsCollapsed));
      header.addEventListener('keydown', (e) => {
        if (e.key !== 'Enter' && e.key !== ' ') return;
        if (e.target.closest?.('.collapse-btn, .ops-overview-link, .agent-ops-tab')) return;
        e.preventDefault();
        // Match click behavior: with icon present, keyboard collapses when open; toggles when closed/no icon
        if (icon && !agentOpsCollapsed) {
          applyOpsCollapsed(true);
          return;
        }
        if (icon && agentOpsCollapsed) {
          applyOpsCollapsed(false);
          return;
        }
        toggleAgentOpsSection();
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
    // Restore last open/closed state after config.json load (WebView is destroyed on close).
    void (async () => {
      try {
        for (let i = 0; i < 40; i++) {
          if (typeof window.loadCpuUiSections === 'function') break;
          await new Promise((r) => setTimeout(r, 25));
        }
        if (typeof window.loadCpuUiSections === 'function') {
          await window.loadCpuUiSections();
        } else if (window.cpuUiSectionsReady) {
          await window.cpuUiSectionsReady;
        }
      } catch (_) {}
      let startsCollapsed = true;
      if (typeof window.getSectionCollapsed === 'function') {
        startsCollapsed = window.getSectionCollapsed('agent_ops_collapsed');
      } else {
        try {
          const saved = localStorage.getItem('agent_ops_collapsed');
          if (saved !== null) startsCollapsed = saved === 'true';
        } catch (_) {
          startsCollapsed = true;
        }
      }
      applyOpsCollapsed(!!startsCollapsed);
      // Design-review / capture: MAC_STATS_OPEN_SECTION or one-shot config openUiSection.
      const scrollStart = (el) => {
        try {
          el?.scrollIntoView?.({ behavior: 'auto', block: 'start' });
        } catch (_) {
          el?.scrollIntoView?.(true);
        }
      };
      let section = null;
      for (let i = 0; i < 25; i++) {
        try {
          section = await invoke('take_open_ui_section');
          break;
        } catch (_) {
          await new Promise((r) => setTimeout(r, 100));
        }
      }
      if (!section) return;
      const key = String(section).trim().toLowerCase();
      if (key === 'agent-ops' || key === 'agent_ops' || key === 'ops') {
        applyOpsCollapsed(false);
        selectOpsTab('runs');
      } else if (
        key === 'ai-chat' ||
        key === 'ai_chat' ||
        key === 'ollama' ||
        key === 'chat'
      ) {
        applyOpsCollapsed(true);
        if (typeof window.setSectionCollapsed === 'function') {
          window.setSectionCollapsed('ollama_collapsed', false);
        } else {
          localStorage.setItem('ollama_collapsed', 'false');
        }
        const openOllama = () => {
          const ollamaSection = document.querySelector('.ollama-section');
          const ollamaContent = document.getElementById('ollama-content');
          const collapsed =
            ollamaSection?.classList.contains('collapsed') ||
            ollamaContent?.classList.contains('collapsed') ||
            ollamaContent?.style.display === 'none';
          // Expand only — icon-ollama toggles and would collapse an open section.
          if (collapsed) {
            document.getElementById('ollama-header')?.click();
          }
          try {
            ollamaSection?.scrollIntoView?.({ behavior: 'auto', block: 'start' });
          } catch (_) {
            ollamaSection?.scrollIntoView?.(true);
          }
          document.getElementById('chat-input')?.focus();
        };
        openOllama();
        setTimeout(openOllama, 400);
        setTimeout(openOllama, 1200);
      } else if (
        key === 'processes' ||
        key === 'process' ||
        key === 'process-list' ||
        key === 'process_list' ||
        key === 'top-processes'
      ) {
        applyOpsCollapsed(true);
        if (typeof window.showDetailsProcessesSections === 'function') {
          window.showDetailsProcessesSections();
        } else if (typeof window.setSectionCollapsed === 'function') {
          window.setSectionCollapsed('details_processes_collapsed', false);
        } else {
          localStorage.setItem('details_processes_collapsed', 'false');
        }
        const processesSection =
          document.getElementById('processes-section') ||
          document.querySelector(
            '.apple-processes, .arch-processes, .swiss-processes, .mat-processes, .cpu-processes, .processes-section'
          );
        setTimeout(() => {
          scrollStart(processesSection);
          const row =
            document.querySelector('#process-list .process-row[tabindex="0"]') ||
            document.querySelector('#process-list .process-row');
          row?.focus?.();
        }, 160);
      } else if (
        key === 'disk-cleanup' ||
        key === 'disk_cleanup' ||
        key === 'cleanup' ||
        key === 'disk'
      ) {
        applyOpsCollapsed(true);
        if (typeof window.setSectionCollapsed === 'function') {
          window.setSectionCollapsed('disk_cleanup_collapsed', false);
        } else {
          localStorage.setItem('disk_cleanup_collapsed', 'false');
        }
        const diskSection = document.querySelector('.disk-cleanup-section');
        const diskContent = document.getElementById('disk-cleanup-content');
        const isCollapsed =
          diskSection?.classList.contains('collapsed') ||
          diskContent?.classList.contains('collapsed');
        if (isCollapsed) {
          document.getElementById('icon-disk-cleanup')?.click();
        } else if (typeof window.refreshDiskCleanupPanel === 'function') {
          void window.refreshDiskCleanupPanel();
        }
        setTimeout(() => {
          scrollStart(document.querySelector('.disk-cleanup-section'));
          document.getElementById('disk-cleanup-run-btn')?.focus?.();
        }, 160);
      } else if (
        key === 'monitors' ||
        key === 'monitor' ||
        key === 'external' ||
        key === 'external-monitors' ||
        key === 'external_monitors'
      ) {
        applyOpsCollapsed(true);
        if (typeof window.setSectionCollapsed === 'function') {
          window.setSectionCollapsed('monitors_collapsed', false);
        } else {
          localStorage.setItem('monitors_collapsed', 'false');
        }
        const monitorsSection = document.querySelector('.monitors-section');
        const monitorsContent = document.getElementById('monitors-content');
        const isCollapsed =
          monitorsSection?.classList.contains('collapsed') ||
          monitorsContent?.classList.contains('collapsed');
        if (isCollapsed) {
          document.getElementById('icon-monitors')?.click();
        }
        setTimeout(() => {
          scrollStart(document.querySelector('.monitors-section'));
          const row =
            document.querySelector('#monitors-list .monitor-item[tabindex="0"]') ||
            document.querySelector('#monitors-list .monitor-item');
          (row || document.getElementById('monitors-menu-btn'))?.focus?.();
        }, 160);
      }
    })();
  }

  function readQuickAgentOpsCollapsed() {
    if (typeof window.getSectionCollapsed === 'function') {
      return window.getSectionCollapsed('agent_ops_collapsed');
    }
    try {
      const saved = localStorage.getItem('agent_ops_collapsed');
      if (saved !== null) return saved === 'true';
    } catch (_) {}
    return true;
  }

  function initAgentOps() {
    if (!document.getElementById('ops-health-row')) return;
    applyOpsCollapsed(readQuickAgentOpsCollapsed());
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
