<script lang="ts">
  import {
    getChecklists,
    getChecklist,
    loadCheckedState,
    saveCheckedState,
    type Checklist,
    type ChecklistPhase,
    type ChecklistItem,
    type CheckedState,
  } from './checklists';
  import {
    CheckCircle2,
    ChevronDown,
    ChevronRight,
    Plane,
    RotateCcw,
    Lock,
    Check,
  } from 'lucide-svelte';
  import { onMount } from 'svelte';

  // Props (optional, from URL params on mobile)
  // eslint-disable-next-line
  let { initialAircraft, initialMode }: { initialAircraft?: string; initialMode?: 'strict' | 'smart' } = $props();

  // State
  const _checklists = getChecklists();
  let checklists = $state(_checklists);
  // svelte-ignore state_referenced_locally
  let selectedId = $state(initialAircraft || (_checklists[0]?.id ?? ''));
  let checklist = $derived(getChecklist(selectedId));
  // svelte-ignore state_referenced_locally
  let mode: 'strict' | 'smart' = $state((initialMode || 'strict') as 'strict' | 'smart');
  let checkedState = $state<CheckedState>(loadCheckedState());
  let expandedItems = $state<Set<string>>(new Set());
  let activePhaseId = $state('');
  let darkMode = $state(true);
  let wsUrl = $state(localStorage.getItem('cf-ws-url') || '');
  let wsConnected = $state(false);
  let ws: WebSocket | null = $state(null);
  let simVars = $state<Record<string, number>>({});
  let detectedItems = $state<Record<string, boolean>>({});
  let utcTime = $state('');

  // Detect if served from the mobile HTTP server (port 8080)
  const isMobileServer = typeof window !== 'undefined' && window.location.port === '8080';
  let simConnected = $state(false);
  let dataMode = $state<'live' | 'manual'>('live');
  let autoCheckSetting = $state<'on' | 'off'>('on');
  let checkFeedbackSetting = $state<'on' | 'off'>('on');

  // UTC clock
  onMount(() => {
    function updateClock() {
      const now = new Date();
      utcTime = now.toISOString().slice(11, 16);
    }
    updateClock();
    const interval = setInterval(updateClock, 1000);

    // If on mobile server, poll /api/sim-state and /api/settings
    let simPollInterval: ReturnType<typeof setInterval> | null = null;
    let settingsPollInterval: ReturnType<typeof setInterval> | null = null;
    if (isMobileServer) {
      // Fetch settings periodically
      async function fetchSettings() {
        try {
          const res = await fetch('/api/settings');
          const data = await res.json();
          dataMode = data.data_mode === 'manual' ? 'manual' : 'live';
          autoCheckSetting = data.auto_check === 'off' ? 'off' : 'on';
          checkFeedbackSetting = data.check_feedback === 'off' ? 'off' : 'on';
          // Apply web theme from backend setting
          const theme = data.web_theme || 'dark';
          if (theme === 'light') {
            document.documentElement.setAttribute('data-theme', 'light');
          } else {
            document.documentElement.removeAttribute('data-theme');
          }
          darkMode = theme !== 'light';
        } catch {}
      }
      fetchSettings();
      settingsPollInterval = setInterval(fetchSettings, 5000);

      // Poll sim state for auto-detect
      async function pollSimState() {
        if (dataMode === 'manual') {
          simConnected = false;
          return;
        }
        try {
          const res = await fetch('/api/sim-state');
          const data = await res.json();
          simConnected = data.connected;
          if (data.connected && data.values) {
            simVars = { ...simVars, ...data.values };
            processAutoDetect();
          }
        } catch {
          simConnected = false;
        }
      }
      simPollInterval = setInterval(pollSimState, 500);
    }

    return () => {
      clearInterval(interval);
      if (simPollInterval) clearInterval(simPollInterval);
      if (settingsPollInterval) clearInterval(settingsPollInterval);
    };
  });

  // Initialize active phase
  $effect(() => {
    if (checklist && checklist.phases.length > 0) {
      const firstIncomplete = checklist.phases.find((p) => {
        const checked = getPhaseChecked(checklist!.id, p.id);
        return checked.length < p.items.length;
      });
      activePhaseId = firstIncomplete?.id ?? checklist.phases[0].id;
    }
  });

  // Theme effect
  $effect(() => {
    if (darkMode) {
      document.documentElement.removeAttribute('data-theme');
    } else {
      document.documentElement.setAttribute('data-theme', 'light');
    }
  });

  // Persist checked state
  $effect(() => {
    saveCheckedState(checkedState);
  });

  // WebSocket connection for auto-detect
  $effect(() => {
    if (wsUrl) {
      localStorage.setItem('cf-ws-url', wsUrl);
      connectWs();
    }
    return () => {
      if (ws) {
        ws.close();
        ws = null;
      }
    };
  });

  function connectWs() {
    if (ws) { ws.close(); ws = null; }
    if (!wsUrl) return;
    try {
      const socket = new WebSocket(wsUrl);
      socket.onopen = () => { wsConnected = true; ws = socket; };
      socket.onclose = () => { wsConnected = false; ws = null; };
      socket.onerror = () => { wsConnected = false; };
      socket.onmessage = (e) => {
        try {
          const data = JSON.parse(e.data);
          if (data.type === 'simvar_update' && data.vars) {
            simVars = { ...simVars, ...data.vars };
            processAutoDetect();
          }
        } catch {}
      };
      ws = socket;
    } catch {
      wsConnected = false;
    }
  }

  function processAutoDetect() {
    if (!checklist) return;
    for (const phase of checklist.phases) {
      for (const item of phase.items) {
        if (!item.auto_detect) continue;
        if (mode === 'strict') {
          const currentItem = getCurrentItem(phase);
          if (currentItem?.id !== item.id) continue;
        }
        const checked = getPhaseChecked(checklist.id, phase.id);
        if (checked.includes(item.id)) continue;
        const val = simVars[item.auto_detect.var_name];
        const condMet = val !== undefined && evaluateCondition(val, item.auto_detect.condition);
        // Store detection status for feedback
        const key = `${phase.id}::${item.id}`;
        detectedItems[key] = condMet;
        // Only auto-check if autoCheckSetting is ON
        if (condMet && autoCheckSetting === 'on') {
          toggleItem(checklist.id, phase.id, item.id);
        }
      }
    }
    detectedItems = { ...detectedItems }; // trigger reactivity
  }

  function evaluateCondition(value: number, condition: string): boolean {
    const match = condition.match(/^(==|!=|>=|<=|>|<)\s*(.+)$/);
    if (!match) return false;
    const op = match[1];
    const target = parseFloat(match[2]);
    if (isNaN(target)) return false;
    switch (op) {
      case '==': return value === target;
      case '!=': return value !== target;
      case '>=': return value >= target;
      case '<=': return value <= target;
      case '>':  return value > target;
      case '<':  return value < target;
      default: return false;
    }
  }

  function getPhaseChecked(clId: string, phaseId: string): string[] {
    return checkedState[clId]?.[phaseId] ?? [];
  }

  function isChecked(clId: string, phaseId: string, itemId: string): boolean {
    return getPhaseChecked(clId, phaseId).includes(itemId);
  }

  function toggleItem(clId: string, phaseId: string, itemId: string) {
    const current = { ...checkedState };
    if (!current[clId]) current[clId] = {};
    if (!current[clId][phaseId]) current[clId][phaseId] = [];
    const arr = [...current[clId][phaseId]];
    const idx = arr.indexOf(itemId);
    if (idx >= 0) {
      arr.splice(idx, 1);
    } else {
      arr.push(itemId);
    }
    current[clId][phaseId] = arr;
    checkedState = current;

    // Phase auto-advance
    if (checklist && idx < 0) {
      const phase = checklist.phases.find((p) => p.id === phaseId);
      if (phase && arr.length === phase.items.length) {
        const nextPhase = getNextIncompletePhase();
        if (nextPhase) {
          setTimeout(() => {
            activePhaseId = nextPhase.id;
            scrollToPhase(nextPhase.id);
          }, 300);
        }
      }
    }
  }

  function getNextIncompletePhase(): ChecklistPhase | undefined {
    if (!checklist) return undefined;
    const currentIdx = checklist.phases.findIndex((p) => p.id === activePhaseId);
    for (let i = currentIdx + 1; i < checklist.phases.length; i++) {
      const p = checklist.phases[i];
      const checked = getPhaseChecked(checklist.id, p.id);
      if (checked.length < p.items.length) return p;
    }
    return undefined;
  }

  function getCurrentItem(phase: ChecklistPhase): ChecklistItem | undefined {
    if (!checklist) return undefined;
    return phase.items.find((item) => !isChecked(checklist!.id, phase.id, item.id));
  }

  function isItemLocked(phase: ChecklistPhase, item: ChecklistItem): boolean {
    if (mode === 'smart') return false;
    const current = getCurrentItem(phase);
    return current?.id !== item.id && !isChecked(checklist!.id, phase.id, item.id);
  }

  function toggleExpand(itemKey: string) {
    const next = new Set(expandedItems);
    if (next.has(itemKey)) {
      next.delete(itemKey);
    } else {
      next.add(itemKey);
    }
    expandedItems = next;
  }

  function resetChecklist() {
    if (!checklist) return;
    const current = { ...checkedState };
    delete current[checklist.id];
    checkedState = current;
    expandedItems = new Set();
    if (checklist.phases.length > 0) {
      activePhaseId = checklist.phases[0].id;
    }
  }

  function scrollToPhase(phaseId: string) {
    const el = document.getElementById(`phase-sidebar-${phaseId}`);
    el?.scrollIntoView({ behavior: 'smooth', inline: 'center', block: 'nearest' });
  }

  function getPhaseProgress(phase: ChecklistPhase): { done: number; total: number } {
    if (!checklist) return { done: 0, total: 0 };
    const checked = getPhaseChecked(checklist.id, phase.id);
    return { done: checked.length, total: phase.items.length };
  }

  function getTotalProgress(): { done: number; total: number } {
    if (!checklist) return { done: 0, total: 0 };
    let done = 0, total = 0;
    for (const p of checklist.phases) {
      const checked = getPhaseChecked(checklist.id, p.id);
      done += checked.length;
      total += p.items.length;
    }
    return { done, total };
  }

  let totalProgress = $derived(getTotalProgress());
  let progressPct = $derived(totalProgress.total > 0 ? (totalProgress.done / totalProgress.total) * 100 : 0);
  let activePhase = $derived(checklist?.phases.find((p) => p.id === activePhaseId));
  let activePhaseProgress = $derived(activePhase ? getPhaseProgress(activePhase) : { done: 0, total: 0 });
  let activePhaseIdx = $derived(checklist ? checklist.phases.findIndex((p) => p.id === activePhaseId) : -1);
</script>

<div class="cf-mobile" style="background: var(--efb-bg); color: var(--efb-text);">
  {#if checklist && activePhase}
    <!-- ========== PHASE HEADER ========== -->
    <header class="cf-phase-header">
      <div class="cf-phase-title-row">
        <div class="cf-phase-title-left">
          <Plane size={18} style="color: var(--efb-accent); flex-shrink: 0;" />
          <h2 class="cf-phase-name">{activePhase.name}</h2>
        </div>
        <span class="cf-phase-count">{activePhaseProgress.done}/{activePhaseProgress.total}</span>
      </div>
      <!-- Prev / Next buttons -->
      <div class="cf-phase-nav">
        <button
          class="cf-nav-btn"
          disabled={activePhaseIdx <= 0}
          onclick={() => { if (activePhaseIdx > 0) activePhaseId = checklist!.phases[activePhaseIdx - 1].id; }}
        >
          &#9664; PREV
        </button>
        <span class="cf-phase-indicator">{activePhaseIdx + 1} / {checklist.phases.length}</span>
        <button
          class="cf-nav-btn"
          disabled={activePhaseIdx >= checklist.phases.length - 1}
          onclick={() => { if (activePhaseIdx < checklist!.phases.length - 1) activePhaseId = checklist!.phases[activePhaseIdx + 1].id; }}
        >
          NEXT &#9654;
        </button>
      </div>
    </header>

    <!-- ========== ITEMS LIST ========== -->
    <main class="cf-items-list">
      {#each activePhase.items as item, idx (item.id)}
        {@const checked = isChecked(checklist.id, activePhaseId, item.id)}
        {@const current = getCurrentItem(activePhase)?.id === item.id}
        {@const locked = isItemLocked(activePhase, item)}
        {@const expanded = expandedItems.has(`${activePhaseId}-${item.id}`)}
        {@const hasDetail = item.how_to || item.location}

        <div
          class="cf-item"
          style="opacity: {locked ? '0.35' : '1'}; pointer-events: {locked ? 'none' : 'auto'};
                 background: {current ? 'var(--efb-current)' : checked ? 'var(--efb-checked)' : 'transparent'};"
        >
          <!-- Main item row -->
          <button
            class="cf-item-row"
            onclick={() => { if (!locked) toggleItem(checklist!.id, activePhaseId, item.id); }}
            disabled={locked}
          >
            <!-- Check circle -->
            <span class="cf-item-check">
              {#if checked}
                <span class="cf-filled-circle efb-check-anim" style="background: var(--efb-success);">
                  <Check size={16} style="color: white;" />
                </span>
              {:else if locked}
                <Lock size={18} style="color: var(--efb-text-dim);" />
              {:else}
                <span class="cf-empty-circle" style="border-color: {current ? 'var(--efb-accent)' : 'var(--efb-text-dim)'};">
                  {#if current}
                    <span class="cf-inner-dot efb-pulse-ring" style="background: var(--efb-accent);"></span>
                  {/if}
                </span>
              {/if}
            </span>

            <!-- Label (sans-serif, big) -->
            <span class="cf-item-label" style="{checked ? 'opacity: 0.5; text-decoration: line-through;' : ''}">
              {item.label}
            </span>

            <!-- Feedback dot (no text) -->
            {#if checkFeedbackSetting === 'on' && item.auto_detect && !checked && dataMode === 'live'}
              {@const detected = detectedItems[`${activePhase.id}::${item.id}`]}
              <span class="cf-feedback-dot" style="background:{detected ? '#00c853' : '#ff1744'};box-shadow:0 0 5px {detected ? '#00c853' : '#ff1744'};"></span>
            {/if}

            <!-- Expected value (monospace, accent) -->
            <span class="cf-item-value" style="color: {checked ? 'var(--efb-success)' : 'var(--efb-accent)'};">
              {item.expected}
            </span>
          </button>

          <!-- Expand toggle for detail -->
          {#if hasDetail}
            <button class="cf-expand-btn" onclick={() => toggleExpand(`${activePhaseId}-${item.id}`)}>
              {#if expanded}
                <ChevronDown size={14} />
              {:else}
                <ChevronRight size={14} />
              {/if}
            </button>
          {/if}

          <!-- Expanded detail (plain text) -->
          {#if expanded && hasDetail}
            <div class="cf-item-detail efb-slide-down">
              {#if item.how_to}
                <p class="cf-detail-howto">{item.how_to}</p>
              {/if}
              {#if item.location}
                <p class="cf-detail-location">{item.location}</p>
              {/if}
            </div>
          {/if}
        </div>
      {/each}

      <!-- Phase complete message -->
      {#if activePhase && getPhaseProgress(activePhase).done === getPhaseProgress(activePhase).total}
        <div class="cf-phase-done efb-fade-in">
          <CheckCircle2 size={28} style="color: var(--efb-success);" />
          <span>{activePhase.name} Complete</span>
          {#if getNextIncompletePhase()}
            <button
              class="cf-next-phase-btn"
              onclick={() => { const next = getNextIncompletePhase(); if (next) { activePhaseId = next.id; } }}
            >
              Next: {getNextIncompletePhase()?.name}
            </button>
          {:else if progressPct === 100}
            <span class="cf-all-done">All phases complete.</span>
          {/if}
        </div>
      {/if}
    </main>

    <!-- ========== BOTTOM BAR ========== -->
    <footer class="cf-bottom-bar">
      <span
        class="cf-bar-mode"
        style="color: {mode === 'strict' ? 'var(--efb-accent)' : 'var(--efb-success)'};"
      >
        {mode === 'strict' ? 'STRICT' : 'SMART'}
      </span>
      <span class="cf-bar-progress" style="color: var(--efb-text-dim);">
        {totalProgress.done}/{totalProgress.total}
      </span>
      <button class="cf-bar-btn" onclick={resetChecklist} style="color: var(--efb-text-dim);">
        <RotateCcw size={14} />
        RESET
      </button>
    </footer>
  {:else}
    <!-- No checklist -->
    <div class="cf-empty">
      <Plane size={40} style="color: var(--efb-text-dim); opacity: 0.3;" />
      <p>No checklist loaded</p>
    </div>
  {/if}
</div>

<style>
  /* ====== CLEAN MOBILE CHECKLIST ====== */
  .cf-mobile {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    overflow: hidden;
    font-family: var(--efb-font, system-ui, -apple-system, sans-serif);
  }

  /* -- Phase header -- */
  .cf-phase-header {
    flex-shrink: 0;
    padding: 12px 16px 8px;
    border-bottom: 2px solid var(--efb-accent);
  }
  .cf-phase-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .cf-phase-title-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .cf-phase-name {
    font-size: 17px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0;
    color: var(--efb-text);
  }
  .cf-phase-count {
    font-family: var(--efb-mono);
    font-size: 14px;
    font-weight: 700;
    color: var(--efb-accent);
  }
  .cf-phase-nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .cf-nav-btn {
    font-family: var(--efb-font, system-ui, sans-serif);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    padding: 6px 14px;
    border-radius: 6px;
    border: 1px solid var(--efb-border);
    background: transparent;
    color: var(--efb-text-dim);
    cursor: pointer;
  }
  .cf-nav-btn:not(:disabled):active {
    background: var(--efb-current);
    color: var(--efb-accent);
  }
  .cf-nav-btn:disabled {
    opacity: 0.25;
    cursor: default;
  }
  .cf-phase-indicator {
    font-size: 11px;
    font-weight: 600;
    color: var(--efb-text-dim);
  }

  /* -- Items list -- */
  .cf-items-list {
    flex: 1;
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
  }

  .cf-item {
    position: relative;
    border-bottom: 1px solid var(--efb-border);
  }

  .cf-item-row {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    min-height: 52px;
    padding: 8px 16px;
    padding-right: 36px; /* room for expand btn */
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--efb-text);
  }

  .cf-item-check {
    flex-shrink: 0;
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cf-filled-circle {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cf-empty-circle {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 2px solid var(--efb-text-dim);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cf-inner-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .cf-item-label {
    flex: 1;
    font-size: 15px;
    font-weight: 500;
    line-height: 1.3;
    font-family: var(--efb-font, system-ui, sans-serif);
  }

  .cf-feedback-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .cf-item-value {
    font-family: var(--efb-mono);
    font-size: 14px;
    font-weight: 700;
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* Expand button (absolute right) */
  .cf-expand-btn {
    position: absolute;
    right: 8px;
    top: 14px;
    background: transparent;
    border: none;
    color: var(--efb-text-dim);
    cursor: pointer;
    padding: 4px;
  }

  /* -- Detail section -- */
  .cf-item-detail {
    padding: 0 16px 10px 54px;
  }
  .cf-detail-howto {
    font-size: 13px;
    line-height: 1.5;
    color: var(--efb-text-dim);
    margin: 0 0 4px;
  }
  .cf-detail-location {
    font-size: 12px;
    color: var(--efb-text-dim);
    opacity: 0.7;
    margin: 0;
  }

  /* -- Phase complete -- */
  .cf-phase-done {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 32px 16px;
    font-size: 16px;
    font-weight: 700;
    color: var(--efb-success);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .cf-next-phase-btn {
    margin-top: 8px;
    padding: 10px 20px;
    border-radius: 8px;
    border: none;
    background: var(--efb-accent);
    color: white;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }
  .cf-all-done {
    font-size: 13px;
    font-weight: 400;
    color: var(--efb-text-dim);
    text-transform: none;
  }

  /* -- Bottom bar -- */
  .cf-bottom-bar {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-top: 1px solid var(--efb-border);
    background: var(--efb-surface);
  }
  .cf-bar-btn {
    font-family: var(--efb-mono);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    background: transparent;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    border-radius: 4px;
  }
  .cf-bar-btn:active {
    background: var(--efb-current);
  }
  .cf-bar-mode {
    font-family: var(--efb-mono);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    padding: 6px 10px;
  }
  .cf-bar-progress {
    font-family: var(--efb-mono);
    font-size: 13px;
    font-weight: 600;
  }

  /* -- Empty state -- */
  .cf-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--efb-text-dim);
    font-size: 14px;
  }
</style>
