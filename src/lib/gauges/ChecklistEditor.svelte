<script lang="ts">
  import {
    type Checklist,
    type ChecklistPhase,
    type ChecklistItem,
  } from './checklists';
  import {
    Plus,
    Trash2,
    Download,
    Upload,
    Eye,
    ChevronUp,
    ChevronDown,
    FileJson,
  } from 'lucide-svelte';

  let checklist = $state<Checklist>({
    id: '',
    name: '',
    author: '',
    version: '1.0',
    category: 'GA',
    simulator: 'both',
    phases: [],
  });

  let previewMode = $state(false);
  let importError = $state('');
  let fileInput: HTMLInputElement;

  const categories = ['GA', 'Airliner', 'Military', 'Helicopter', 'Glider', 'Other'];

  function generateId(name: string): string {
    return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
  }

  function addPhase() {
    checklist.phases = [...checklist.phases, {
      id: `phase-${Date.now()}`,
      name: '',
      items: [],
    }];
  }

  function removePhase(idx: number) {
    checklist.phases = checklist.phases.filter((_, i) => i !== idx);
  }

  function movePhase(idx: number, dir: -1 | 1) {
    const newIdx = idx + dir;
    if (newIdx < 0 || newIdx >= checklist.phases.length) return;
    const phases = [...checklist.phases];
    [phases[idx], phases[newIdx]] = [phases[newIdx], phases[idx]];
    checklist.phases = phases;
  }

  function addItem(phaseIdx: number) {
    const phases = [...checklist.phases];
    phases[phaseIdx] = {
      ...phases[phaseIdx],
      items: [...phases[phaseIdx].items, {
        id: `item-${Date.now()}`,
        label: '',
        expected: '',
        how_to: '',
        location: '',
        auto_detect: null,
      }],
    };
    checklist.phases = phases;
  }

  function removeItem(phaseIdx: number, itemIdx: number) {
    const phases = [...checklist.phases];
    phases[phaseIdx] = {
      ...phases[phaseIdx],
      items: phases[phaseIdx].items.filter((_, i) => i !== itemIdx),
    };
    checklist.phases = phases;
  }

  function toggleAutoDetect(phaseIdx: number, itemIdx: number) {
    const phases = [...checklist.phases];
    const item = { ...phases[phaseIdx].items[itemIdx] };
    if (item.auto_detect) {
      item.auto_detect = null;
    } else {
      item.auto_detect = { var_name: '', var_type: 'A', condition: '' };
    }
    phases[phaseIdx] = {
      ...phases[phaseIdx],
      items: phases[phaseIdx].items.map((it, i) => i === itemIdx ? item : it),
    };
    checklist.phases = phases;
  }

  function exportJson() {
    const output = {
      ...checklist,
      id: generateId(checklist.name) || 'untitled',
      phases: checklist.phases.map((p) => ({
        ...p,
        id: generateId(p.name) || p.id,
        items: p.items.map((item) => ({
          ...item,
          id: generateId(item.label) || item.id,
        })),
      })),
    };
    const blob = new Blob([JSON.stringify(output, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${output.id}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function importJson() {
    fileInput?.click();
  }

  function handleFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    importError = '';
    const reader = new FileReader();
    reader.onload = (ev) => {
      try {
        const data = JSON.parse(ev.target?.result as string);
        if (!data.name || !data.phases || !Array.isArray(data.phases)) {
          importError = 'Invalid checklist format.';
          return;
        }
        checklist = data as Checklist;
      } catch {
        importError = 'Invalid JSON file.';
      }
    };
    reader.readAsText(file);
    target.value = '';
  }

  function updateItemField(phaseIdx: number, itemIdx: number, field: string, value: string) {
    const phases = [...checklist.phases];
    const items = [...phases[phaseIdx].items];
    items[itemIdx] = { ...items[itemIdx], [field]: value };
    phases[phaseIdx] = { ...phases[phaseIdx], items };
    checklist.phases = phases;
  }

  function updateAutoDetectField(phaseIdx: number, itemIdx: number, field: string, value: string) {
    const phases = [...checklist.phases];
    const items = [...phases[phaseIdx].items];
    const ad = { ...items[itemIdx].auto_detect!, [field]: value };
    items[itemIdx] = { ...items[itemIdx], auto_detect: ad };
    phases[phaseIdx] = { ...phases[phaseIdx], items };
    checklist.phases = phases;
  }

  function updatePhaseName(phaseIdx: number, value: string) {
    const phases = [...checklist.phases];
    phases[phaseIdx] = { ...phases[phaseIdx], name: value };
    checklist.phases = phases;
  }
</script>

<div class="ce-root">
  <!-- Editor Header -->
  <header class="ce-header">
    <div class="ce-header-inner">
      <div class="ce-header-left">
        <FileJson size={16} class="ce-icon-accent" />
        <span class="ce-header-title">CHECKLIST EDITOR</span>
      </div>
      <div class="ce-header-actions">
        <button onclick={() => previewMode = !previewMode} class="ce-btn">
          <Eye size={13} />
          {previewMode ? 'EDIT' : 'PREVIEW'}
        </button>
        <button onclick={importJson} class="ce-btn">
          <Upload size={13} />
          IMPORT
        </button>
        <button onclick={exportJson} class="ce-btn ce-btn-primary">
          <Download size={13} />
          EXPORT
        </button>
      </div>
    </div>
    {#if importError}
      <p class="ce-error">{importError}</p>
    {/if}
    <input type="file" accept=".json" class="hidden" bind:this={fileInput} onchange={handleFileSelect} />
  </header>

  <main class="ce-main">
    {#if previewMode}
      <!-- Preview -->
      <div class="ce-panel">
        <h2 class="ce-preview-name">{checklist.name || '(UNTITLED)'}</h2>
        <p class="ce-preview-meta">
          BY {(checklist.author || 'UNKNOWN').toUpperCase()} | V{checklist.version} | {checklist.category}
        </p>
      </div>
      {#each checklist.phases as phase, pi}
        <div class="ce-panel ce-preview-phase">
          <h3 class="ce-phase-title">{(phase.name || `PHASE ${pi + 1}`).toUpperCase()}</h3>
          {#each phase.items as item}
            <div class="ce-preview-item">
              <span class="ce-preview-dot"></span>
              <span class="ce-preview-label">{item.label || '(No label)'}</span>
              {#if item.auto_detect}<span class="ce-preview-auto" title="Auto-detect">&#9889;</span>{/if}
              <span class="ce-preview-val">{item.expected || '?'}</span>
            </div>
          {/each}
          {#if phase.items.length === 0}
            <p class="ce-empty">NO ITEMS</p>
          {/if}
        </div>
      {/each}
      {#if checklist.phases.length === 0}
        <p class="ce-empty" style="padding: 24px 0;">NO PHASES ADDED YET</p>
      {/if}
    {:else}
      <!-- Edit Mode -->
      <!-- Checklist Meta -->
      <div class="ce-panel">
        <div class="ce-section-title"><span class="ce-section-line"></span>CHECKLIST INFO<span class="ce-section-line"></span></div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <span class="ce-label">NAME</span>
            <input
              type="text"
              bind:value={checklist.name}
              placeholder="e.g. Cessna 172 Skyhawk"
              class="ce-input"
            />
          </div>
          <div>
            <span class="ce-label">AUTHOR</span>
            <input
              type="text"
              bind:value={checklist.author}
              placeholder="Your name"
              class="ce-input"
            />
          </div>
          <div>
            <span class="ce-label">VERSION</span>
            <input
              type="text"
              bind:value={checklist.version}
              placeholder="1.0"
              class="ce-input"
            />
          </div>
          <div>
            <span class="ce-label">CATEGORY</span>
            <select bind:value={checklist.category} class="ce-input ce-select">
              {#each categories as cat}
                <option value={cat}>{cat}</option>
              {/each}
            </select>
          </div>
          <div>
            <span class="ce-label">SIMULATOR</span>
            <select bind:value={checklist.simulator} class="ce-input ce-select">
              <option value="both">Both (MSFS + X-Plane)</option>
              <option value="msfs">MSFS Only</option>
              <option value="xplane">X-Plane Only</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Phases -->
      {#each checklist.phases as phase, pi}
        <div class="ce-panel">
          <div class="ce-section-title"><span class="ce-section-line"></span>PHASE {pi + 1}<span class="ce-section-line"></span></div>
          <div class="ce-phase-row">
            <div class="ce-phase-arrows">
              <button
                onclick={() => movePhase(pi, -1)}
                class="ce-arrow-btn"
                disabled={pi === 0}
              >
                <ChevronUp size={14} />
              </button>
              <button
                onclick={() => movePhase(pi, 1)}
                class="ce-arrow-btn"
                disabled={pi === checklist.phases.length - 1}
              >
                <ChevronDown size={14} />
              </button>
            </div>
            <input
              type="text"
              value={phase.name}
              oninput={(e) => updatePhaseName(pi, (e.target as HTMLInputElement).value)}
              placeholder="Phase name (e.g. Pre-Flight)"
              class="ce-input ce-phase-input"
            />
            <button
              onclick={() => removePhase(pi)}
              class="ce-btn-danger"
              title="Remove phase"
            >
              <Trash2 size={14} />
            </button>
          </div>

          <!-- Items -->
          {#each phase.items as item, ii}
            <div class="ce-item-card">
              <div class="ce-item-row">
                <input
                  type="text"
                  value={item.label}
                  oninput={(e) => updateItemField(pi, ii, 'label', (e.target as HTMLInputElement).value)}
                  placeholder="Item label"
                  class="ce-input ce-item-label-input"
                />
                <input
                  type="text"
                  value={item.expected}
                  oninput={(e) => updateItemField(pi, ii, 'expected', (e.target as HTMLInputElement).value)}
                  placeholder="Expected"
                  class="ce-input ce-item-expected-input"
                />
                <button
                  onclick={() => removeItem(pi, ii)}
                  class="ce-btn-danger ce-btn-sm"
                >
                  <Trash2 size={12} />
                </button>
              </div>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                <input
                  type="text"
                  value={item.how_to || ''}
                  oninput={(e) => updateItemField(pi, ii, 'how_to', (e.target as HTMLInputElement).value)}
                  placeholder="How to (instructions)"
                  class="ce-input"
                />
                <input
                  type="text"
                  value={item.location || ''}
                  oninput={(e) => updateItemField(pi, ii, 'location', (e.target as HTMLInputElement).value)}
                  placeholder="Location in cockpit"
                  class="ce-input"
                />
              </div>
              <!-- Auto-detect -->
              <div>
                <button
                  onclick={() => toggleAutoDetect(pi, ii)}
                  class="ce-autodetect-toggle"
                >
                  {item.auto_detect ? 'REMOVE AUTO-DETECT' : '+ ADD AUTO-DETECT'}
                </button>
                {#if item.auto_detect}
                  <div class="grid grid-cols-3 gap-2 mt-1">
                    <input
                      type="text"
                      value={item.auto_detect.var_name}
                      oninput={(e) => updateAutoDetectField(pi, ii, 'var_name', (e.target as HTMLInputElement).value)}
                      placeholder="Var name"
                      class="ce-input ce-input-sm ce-input-mono"
                    />
                    <select
                      value={item.auto_detect.var_type}
                      onchange={(e) => updateAutoDetectField(pi, ii, 'var_type', (e.target as HTMLSelectElement).value)}
                      class="ce-input ce-input-sm ce-select"
                    >
                      <option value="A">A (SimVar)</option>
                      <option value="L">L (LVar)</option>
                    </select>
                    <input
                      type="text"
                      value={item.auto_detect.condition}
                      oninput={(e) => updateAutoDetectField(pi, ii, 'condition', (e.target as HTMLInputElement).value)}
                      placeholder="e.g. > 0"
                      class="ce-input ce-input-sm ce-input-mono"
                    />
                  </div>
                {/if}
              </div>
            </div>
          {/each}

          <button onclick={() => addItem(pi)} class="ce-add-item-btn">
            <Plus size={13} />
            ADD ITEM
          </button>
        </div>
      {/each}

      <button onclick={addPhase} class="ce-add-phase-btn">
        <Plus size={16} />
        ADD PHASE
      </button>
    {/if}

    <!-- Bottom spacer -->
    <div style="height: 16px;"></div>
  </main>
</div>

<style>
  /* ========================================
     CHECKLIST EDITOR - Avionics/EFB Theme
     ======================================== */

  .ce-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--efb-bg, var(--efb-bg));
    color: var(--efb-text, var(--efb-text));
    font-family: var(--efb-font, system-ui, sans-serif);
  }

  /* ---- Header ---- */
  .ce-header {
    flex-shrink: 0;
    background: var(--efb-header, #060a14);
    border-bottom: 1px solid var(--efb-panel-border);
    padding: 6px 12px;
  }
  .ce-header-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .ce-header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .ce-header-title {
    font-family: var(--efb-mono, monospace);
    font-weight: 700;
    font-size: 12px;
    letter-spacing: 0.15em;
    color: #ffffff;
    text-transform: uppercase;
  }
  .ce-header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  :global(.ce-icon-accent) {
    color: var(--efb-accent);
  }
  .ce-error {
    font-family: var(--efb-mono, monospace);
    font-size: 10px;
    color: #ff3355;
    margin-top: 4px;
    letter-spacing: 0.05em;
  }

  /* ---- Buttons ---- */
  .ce-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: var(--efb-mono, monospace);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.1em;
    padding: 4px 10px;
    border-radius: 3px;
    border: 1px solid var(--efb-panel-border);
    background: var(--efb-bg);
    color: var(--efb-text-dim);
    cursor: pointer;
    transition: all 0.12s;
    text-transform: uppercase;
  }
  .ce-btn:hover {
    color: var(--efb-text);
    border-color: var(--efb-accent);
    box-shadow: 0 0 8px rgba(0, 123, 255, 0.2);
  }
  .ce-btn-primary {
    background: rgba(0, 123, 255, 0.12);
    color: var(--efb-accent);
    border-color: var(--efb-accent);
  }
  .ce-btn-primary:hover {
    background: var(--efb-accent);
    color: #ffffff;
    box-shadow: 0 0 12px rgba(0, 123, 255, 0.35);
  }
  .ce-btn-danger {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: 3px;
    border: 1px solid rgba(255, 51, 85, 0.3);
    background: transparent;
    color: #ff3355;
    cursor: pointer;
    transition: all 0.12s;
  }
  .ce-btn-danger:hover {
    background: rgba(255, 51, 85, 0.1);
    border-color: #ff3355;
    box-shadow: 0 0 8px rgba(255, 51, 85, 0.2);
  }
  .ce-btn-sm {
    padding: 3px;
  }

  /* ---- Main scroll area ---- */
  .ce-main {
    flex: 1;
    overflow-y: auto;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
  }

  /* ---- Panel (card) ---- */
  .ce-panel {
    background: var(--efb-panel);
    border: 1px solid var(--efb-panel-border);
    border-radius: 5px;
    padding: 8px 12px;
  }

  /* ---- Section title (centered between lines) ---- */
  .ce-section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--efb-mono, monospace);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.2em;
    color: var(--efb-text-dim);
    text-transform: uppercase;
    margin-bottom: 8px;
  }
  .ce-section-line {
    flex: 1;
    height: 1px;
    background: var(--efb-panel-border);
  }

  /* ---- Labels ---- */
  .ce-label {
    display: block;
    font-family: var(--efb-mono, monospace);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.15em;
    color: var(--efb-text-dim);
    text-transform: uppercase;
    margin-bottom: 3px;
  }

  /* ---- Inputs & Selects ---- */
  .ce-input {
    width: 100%;
    height: 28px;
    font-family: var(--efb-mono, monospace);
    font-size: 11px;
    padding: 0 8px;
    border-radius: 3px;
    border: 1px solid var(--efb-panel-border);
    background: var(--efb-input-bg, var(--efb-bg));
    color: var(--efb-text);
    letter-spacing: 0.03em;
    transition: border-color 0.12s, box-shadow 0.12s;
  }
  .ce-input:focus {
    outline: none;
    border-color: var(--efb-accent);
    box-shadow: 0 0 8px rgba(0, 123, 255, 0.2);
  }
  .ce-input::placeholder {
    color: #2d3748;
    letter-spacing: 0.05em;
  }
  .ce-input-sm {
    height: 24px;
    font-size: 10px;
  }
  .ce-input-mono {
    font-family: var(--efb-mono, monospace);
  }
  .ce-select {
    cursor: pointer;
    appearance: none;
    -webkit-appearance: none;
  }

  /* ---- Phase editing ---- */
  .ce-phase-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
  }
  .ce-phase-arrows {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .ce-arrow-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1px;
    background: transparent;
    border: none;
    color: var(--efb-text-dim);
    cursor: pointer;
    transition: color 0.12s;
  }
  .ce-arrow-btn:hover:not(:disabled) {
    color: var(--efb-accent);
  }
  .ce-arrow-btn:disabled {
    opacity: 0.2;
    cursor: default;
  }
  .ce-phase-input {
    flex: 1;
    font-weight: 700;
    text-transform: uppercase;
  }

  /* ---- Item cards ---- */
  .ce-item-card {
    margin-left: 20px;
    padding: 8px 10px;
    background: var(--efb-bg);
    border: 1px solid var(--efb-panel-border);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 4px;
  }
  .ce-item-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .ce-item-label-input {
    flex: 1;
  }
  .ce-item-expected-input {
    width: 90px;
    color: var(--efb-success) !important;
    flex-shrink: 0;
  }

  /* ---- Auto-detect toggle ---- */
  .ce-autodetect-toggle {
    font-family: var(--efb-mono, monospace);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--efb-amber);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    transition: color 0.12s;
  }
  .ce-autodetect-toggle:hover {
    color: #ffc44d;
    text-decoration: underline;
  }

  /* ---- Add buttons ---- */
  .ce-add-item-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-left: 20px;
    margin-top: 4px;
    font-family: var(--efb-mono, monospace);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--efb-accent);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px 0;
    transition: color 0.12s;
  }
  .ce-add-item-btn:hover {
    color: #339dff;
    text-decoration: underline;
  }

  .ce-add-phase-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 100%;
    padding: 8px 0;
    border-radius: 4px;
    border: 1px dashed var(--efb-panel-border);
    background: transparent;
    font-family: var(--efb-mono, monospace);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    color: var(--efb-text-dim);
    cursor: pointer;
    transition: all 0.12s;
    text-transform: uppercase;
  }
  .ce-add-phase-btn:hover {
    border-color: var(--efb-accent);
    color: var(--efb-accent);
    box-shadow: 0 0 10px rgba(0, 123, 255, 0.1);
  }

  /* ---- Preview styles ---- */
  .ce-preview-name {
    font-family: var(--efb-mono, monospace);
    font-size: 16px;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin-bottom: 2px;
  }
  .ce-preview-meta {
    font-family: var(--efb-mono, monospace);
    font-size: 9px;
    color: var(--efb-text-dim);
    letter-spacing: 0.08em;
  }
  .ce-preview-phase {
    border-top: 2px solid var(--efb-panel-border);
  }
  .ce-phase-title {
    font-family: var(--efb-mono, monospace);
    font-size: 11px;
    font-weight: 700;
    color: var(--efb-accent);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 4px;
  }
  .ce-preview-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 0;
    border-bottom: 1px solid rgba(30, 58, 95, 0.4);
    font-size: 10px;
  }
  .ce-preview-item:last-child {
    border-bottom: none;
  }
  .ce-preview-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    border: 1px solid var(--efb-panel-border);
    flex-shrink: 0;
  }
  .ce-preview-label {
    flex: 1;
    font-family: var(--efb-mono, monospace);
    color: var(--efb-text);
    letter-spacing: 0.03em;
  }
  .ce-preview-auto {
    font-size: 10px;
    color: var(--efb-amber);
    flex-shrink: 0;
  }
  .ce-preview-val {
    font-family: var(--efb-mono, monospace);
    color: var(--efb-success);
    font-size: 10px;
    letter-spacing: 0.05em;
  }
  .ce-empty {
    font-family: var(--efb-mono, monospace);
    font-size: 10px;
    color: var(--efb-text-dim);
    text-align: center;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 8px 0;
  }
</style>
