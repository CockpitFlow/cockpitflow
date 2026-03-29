<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Smartphone, Plus, Save, Trash2, Download, Upload, Copy, Eye, Grid3x3, Layers } from 'lucide-svelte';

  interface Threshold { below: number; color: string; }
  interface Detent { value: number; label: string; }
  interface Position { value: number; label: string; }
  interface SpringBack { from: number; to: number; }

  interface Component {
    id: string;
    type: string;
    col: number;
    row: number;
    w: number;
    h: number;
    label?: string;
    cmd?: string;
    cmd_ail?: string;
    cmd_elv?: string;
    color?: string;
    style?: string;
    default?: number;
    unit?: string;
    dataref?: string;
    format?: string;
    thresholds?: Threshold[];
    detents?: Detent[];
    positions?: Position[] | string[];
    spring_back?: SpringBack;
    step?: number;
    modes?: string[];
    sensitivity?: number;
    deadzone?: number;
    radio_type?: string;
    default_freq?: string;
  }

  interface Preset {
    name: string;
    author: string;
    aircraft?: string;
    version: number;
    grid: { cols: number; rows: number };
    theme: Record<string, string>;
    components: Component[];
  }

  let { lanIp = '127.0.0.1' }: { lanIp?: string } = $props();

  let presets = $state<string[]>([]);
  let activePreset = $state<string>('');
  let preset = $state<Preset | null>(null);
  let selectedComp = $state<Component | null>(null);
  let tab = $state<'list' | 'editor' | 'preview'>('list');
  let dirty = $state(false);
  let saveMsg = $state('');
  let newPresetName = $state('');

  // Component palette
  const componentTypes = [
    { type: 'lever', label: 'Lever', desc: 'Throttle/Mixture/etc', defaultW: 1, defaultH: 7 },
    { type: 'toggle', label: 'Toggle', desc: 'On/Off switch', defaultW: 1, defaultH: 1 },
    { type: 'magneto', label: 'Magneto', desc: 'Ignition switch', defaultW: 3, defaultH: 1 },
    { type: 'yoke', label: 'Yoke', desc: 'Flight control (touch/gyro)', defaultW: 5, defaultH: 7 },
    { type: 'gauge', label: 'Gauge', desc: 'Instrument readout', defaultW: 3, defaultH: 1 },
    { type: 'flaps', label: 'Flaps', desc: 'Flap detent selector', defaultW: 1, defaultH: 5 },
    { type: 'trim', label: 'Trim', desc: 'Elevator trim', defaultW: 1, defaultH: 2 },
    { type: 'fuel-selector', label: 'Fuel', desc: 'Fuel tank selector', defaultW: 3, defaultH: 1 },
    { type: 'radio', label: 'Radio', desc: 'COM/NAV/XPDR display', defaultW: 3, defaultH: 1 },
  ];

  onMount(loadPresets);

  async function loadPresets() {
    try {
      presets = await invoke('list_presets') as string[];
    } catch {
      presets = ['cessna-172'];
    }
  }

  async function loadPreset(name: string) {
    try {
      const json = await invoke('load_preset', { name }) as string;
      preset = JSON.parse(json);
      activePreset = name;
      selectedComp = null;
      dirty = false;
      tab = 'editor';
    } catch (e: any) {
      saveMsg = `Failed to load: ${e.message || e}`;
    }
  }

  async function savePreset() {
    if (!preset || !activePreset) return;
    try {
      await invoke('save_preset', { name: activePreset, data: JSON.stringify(preset, null, 2) });
      dirty = false;
      saveMsg = 'Saved!';
      setTimeout(() => saveMsg = '', 2000);
      loadPresets();
    } catch (e: any) {
      saveMsg = `Error: ${e.message || e}`;
    }
  }

  async function saveAs() {
    if (!preset || !newPresetName.trim()) return;
    const name = newPresetName.trim().toLowerCase().replace(/[^a-z0-9-]/g, '-');
    preset.name = newPresetName.trim();
    activePreset = name;
    await savePreset();
    newPresetName = '';
  }

  async function deletePreset(name: string) {
    // Just remove from list (can't delete via API yet, user can delete file manually)
    if (activePreset === name) { preset = null; activePreset = ''; tab = 'list'; }
  }

  function exportPreset() {
    if (!preset) return;
    const blob = new Blob([JSON.stringify(preset, null, 2)], { type: 'application/json' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${activePreset || 'preset'}.json`;
    a.click();
  }

  function importPreset() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        preset = JSON.parse(text);
        activePreset = file.name.replace('.json', '');
        dirty = true;
        tab = 'editor';
      } catch (e: any) {
        saveMsg = `Invalid JSON: ${e.message}`;
      }
    };
    input.click();
  }

  function addComponent(type: string) {
    if (!preset) return;
    const ct = componentTypes.find(c => c.type === type);
    const id = `${type}_${Date.now()}`;

    const comp: Component = {
      id,
      type,
      col: 1,
      row: 1,
      w: ct?.defaultW || 1,
      h: ct?.defaultH || 1,
      label: type.toUpperCase(),
    };

    // Set defaults per type
    if (type === 'lever') { comp.cmd = 'THROTTLE'; comp.color = 'default'; comp.default = 0; }
    if (type === 'toggle') { comp.cmd = 'MASTER_BATTERY'; comp.style = 'blue'; }
    if (type === 'gauge') { comp.dataref = 'AIRSPEED'; comp.unit = 'KTS'; comp.format = 'int'; comp.thresholds = []; }
    if (type === 'yoke') { comp.modes = ['touch', 'gyro']; comp.cmd_ail = 'AILERON'; comp.cmd_elv = 'ELEVATOR'; comp.sensitivity = 1; comp.deadzone = 0.05; }
    if (type === 'magneto') { comp.cmd = 'MAGNETO'; comp.positions = ['OFF', 'R', 'L', 'BOTH', 'START']; comp.spring_back = { from: 4, to: 3 }; comp.default = 3; }
    if (type === 'flaps') { comp.cmd = 'FLAPS'; comp.detents = [{ value: 0, label: 'UP' }, { value: 0.33, label: '10' }, { value: 0.66, label: '20' }, { value: 1, label: 'FULL' }]; }
    if (type === 'trim') { comp.cmd = 'TRIM'; comp.step = 0.03; }
    if (type === 'fuel-selector') { comp.cmd = 'FUEL_SEL'; comp.positions = [{ value: 1, label: 'L' }, { value: 2, label: 'BOTH' }, { value: 3, label: 'R' }]; comp.default = 2; }
    if (type === 'radio') { comp.radio_type = 'com'; comp.default_freq = '121.800'; }

    preset.components = [...preset.components, comp];
    selectedComp = comp;
    dirty = true;
  }

  function removeComponent(id: string) {
    if (!preset) return;
    preset.components = preset.components.filter(c => c.id !== id);
    if (selectedComp?.id === id) selectedComp = null;
    dirty = true;
  }

  function updateComp() {
    dirty = true;
    // Trigger reactivity
    if (preset) preset.components = [...preset.components];
  }

  function createNew() {
    preset = {
      name: 'New Preset',
      author: '',
      aircraft: '',
      version: 1,
      grid: { cols: 12, rows: 8 },
      theme: { bg: '#111317', panel: '#1c1e22', accent: '#4a9eff', green: '#44dd66', amber: '#ddaa44', red: '#ee4444' },
      components: [],
    };
    activePreset = '';
    dirty = true;
    tab = 'editor';
  }

  function cockpitUrl() {
    const name = activePreset || 'cessna-172';
    return `http://${lanIp}:8080/cockpit2.html?preset=${name}`;
  }

  function cockpitUrlHttps() {
    const name = activePreset || 'cessna-172';
    return `https://${lanIp}:8443/cockpit2.html?preset=${name}`;
  }

  // Color for component type in grid preview
  function typeColor(type: string): string {
    const colors: Record<string, string> = {
      lever: '#4a9eff33', toggle: '#ea4a5a33', magneto: '#e3b34133',
      yoke: '#34d05833', gauge: '#44dd6633', flaps: '#4a9eff22',
      trim: '#4a9eff22', 'fuel-selector': '#e3b34122', radio: '#34d05822',
    };
    return colors[type] || '#ffffff11';
  }
</script>

<div class="cb-wrap">
  <!-- Tab bar -->
  <div class="cb-tabs">
    <button class="cb-tab" class:on={tab === 'list'} onclick={() => tab = 'list'}>
      <Layers size={13} /> Presets
    </button>
    <button class="cb-tab" class:on={tab === 'editor'} onclick={() => tab = 'editor'} disabled={!preset}>
      <Grid3x3 size={13} /> Editor
    </button>
    <button class="cb-tab" class:on={tab === 'preview'} onclick={() => tab = 'preview'} disabled={!preset}>
      <Eye size={13} /> Preview
    </button>
    <div class="cb-tab-spacer"></div>
    {#if saveMsg}<span class="cb-msg">{saveMsg}</span>{/if}
    {#if dirty}<span class="cb-badge-dirty">UNSAVED</span>{/if}
  </div>

  <!-- LIST TAB -->
  {#if tab === 'list'}
    <div class="cb-content">
      <div class="cb-row" style="gap:8px;margin-bottom:12px">
        <button class="cb-btn primary" onclick={createNew}><Plus size={13} /> New Preset</button>
        <button class="cb-btn" onclick={importPreset}><Upload size={13} /> Import JSON</button>
      </div>

      {#if presets.length === 0}
        <p class="cb-empty">No presets found. Create one or import a JSON file.</p>
      {/if}

      <div class="cb-preset-list">
        {#each presets as name}
          <div class="cb-preset-card" class:active={activePreset === name}>
            <div class="cb-preset-info">
              <span class="cb-preset-name">{name}</span>
              <span class="cb-preset-file">{name}.json</span>
            </div>
            <div class="cb-preset-actions">
              <button class="cb-btn sm" onclick={() => loadPreset(name)}>Edit</button>
              <button class="cb-btn sm" onclick={() => { loadPreset(name).then(() => tab = 'preview'); }}>
                <Eye size={11} />
              </button>
            </div>
          </div>
        {/each}
      </div>

      {#if activePreset}
        <div class="cb-section" style="margin-top:16px">
          <div class="label">OPEN ON PHONE</div>
          <div class="cb-url-row">
            <Smartphone size={14} />
            <code class="cb-url">{cockpitUrl()}</code>
            <button class="cb-btn sm" onclick={() => navigator.clipboard.writeText(cockpitUrl())}><Copy size={11} /></button>
          </div>
          <div class="cb-url-row" style="margin-top:4px">
            <Smartphone size={14} />
            <code class="cb-url">{cockpitUrlHttps()}</code>
            <span class="cb-hint">HTTPS (for gyro)</span>
          </div>
        </div>
      {/if}
    </div>

  <!-- EDITOR TAB -->
  {:else if tab === 'editor' && preset}
    <div class="cb-editor">
      <!-- Left: Grid preview -->
      <div class="cb-grid-wrap">
        <div class="cb-grid"
          style="grid-template-columns:repeat({preset.grid.cols},1fr);grid-template-rows:repeat({preset.grid.rows},1fr)">
          <!-- Grid cells background -->
          {#each Array(preset.grid.cols * preset.grid.rows) as _, i}
            {@const col = (i % preset.grid.cols) + 1}
            {@const row = Math.floor(i / preset.grid.cols) + 1}
            <div class="cb-grid-cell"
              style="grid-column:{col};grid-row:{row}"
              onclick={() => {
                // Could add click-to-place later
              }}>
              <span class="cb-grid-coord">{col},{row}</span>
            </div>
          {/each}

          <!-- Components overlay -->
          {#each preset.components as comp}
            <div class="cb-grid-comp"
              class:selected={selectedComp?.id === comp.id}
              style="grid-column:{comp.col}/span {comp.w};grid-row:{comp.row}/span {comp.h};background:{typeColor(comp.type)}"
              onclick={() => selectedComp = comp}>
              <span class="cb-grid-comp-type">{comp.type}</span>
              <span class="cb-grid-comp-label">{comp.label || comp.id}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Right: Properties panel -->
      <div class="cb-props">
        <!-- Preset meta -->
        <div class="cb-section">
          <div class="label">PRESET</div>
          <div class="cb-field">
            <label>Name</label>
            <input type="text" bind:value={preset.name} onchange={updateComp} />
          </div>
          <div class="cb-field">
            <label>Author</label>
            <input type="text" bind:value={preset.author} onchange={updateComp} />
          </div>
          <div class="cb-field-row">
            <div class="cb-field">
              <label>Grid Cols</label>
              <input type="number" bind:value={preset.grid.cols} min="4" max="24" onchange={updateComp} />
            </div>
            <div class="cb-field">
              <label>Grid Rows</label>
              <input type="number" bind:value={preset.grid.rows} min="4" max="16" onchange={updateComp} />
            </div>
          </div>
        </div>

        <!-- Add component -->
        <div class="cb-section">
          <div class="label">ADD COMPONENT</div>
          <div class="cb-palette">
            {#each componentTypes as ct}
              <button class="cb-palette-btn" onclick={() => addComponent(ct.type)} title={ct.desc}>
                {ct.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Selected component props -->
        {#if selectedComp}
          <div class="cb-section">
            <div class="cb-section-header">
              <div class="label">{selectedComp.type.toUpperCase()}: {selectedComp.label || selectedComp.id}</div>
              <button class="cb-btn sm danger" onclick={() => removeComponent(selectedComp!.id)}>
                <Trash2 size={11} />
              </button>
            </div>

            <div class="cb-field">
              <label>ID</label>
              <input type="text" bind:value={selectedComp.id} onchange={updateComp} />
            </div>
            <div class="cb-field">
              <label>Label</label>
              <input type="text" bind:value={selectedComp.label} onchange={updateComp} />
            </div>
            <div class="cb-field-row">
              <div class="cb-field"><label>Col</label><input type="number" bind:value={selectedComp.col} min="1" max={preset.grid.cols} onchange={updateComp} /></div>
              <div class="cb-field"><label>Row</label><input type="number" bind:value={selectedComp.row} min="1" max={preset.grid.rows} onchange={updateComp} /></div>
              <div class="cb-field"><label>W</label><input type="number" bind:value={selectedComp.w} min="1" max={preset.grid.cols} onchange={updateComp} /></div>
              <div class="cb-field"><label>H</label><input type="number" bind:value={selectedComp.h} min="1" max={preset.grid.rows} onchange={updateComp} /></div>
            </div>

            {#if selectedComp.cmd !== undefined}
              <div class="cb-field"><label>Command</label><input type="text" bind:value={selectedComp.cmd} onchange={updateComp} /></div>
            {/if}
            {#if selectedComp.dataref !== undefined}
              <div class="cb-field"><label>Dataref</label><input type="text" bind:value={selectedComp.dataref} onchange={updateComp} /></div>
            {/if}
            {#if selectedComp.unit !== undefined}
              <div class="cb-field"><label>Unit</label><input type="text" bind:value={selectedComp.unit} onchange={updateComp} /></div>
            {/if}
            {#if selectedComp.color !== undefined}
              <div class="cb-field">
                <label>Color</label>
                <select bind:value={selectedComp.color} onchange={updateComp}>
                  <option value="default">Default (blue)</option>
                  <option value="red">Red</option>
                  <option value="amber">Amber</option>
                </select>
              </div>
            {/if}
            {#if selectedComp.style !== undefined}
              <div class="cb-field">
                <label>Style</label>
                <select bind:value={selectedComp.style} onchange={updateComp}>
                  <option value="blue">Blue</option>
                  <option value="red">Red</option>
                </select>
              </div>
            {/if}
            {#if selectedComp.format !== undefined}
              <div class="cb-field">
                <label>Format</label>
                <select bind:value={selectedComp.format} onchange={updateComp}>
                  <option value="int">Integer</option>
                  <option value="signed">Signed (+/-)</option>
                  <option value="heading">Heading (001-360)</option>
                  <option value="decimal">Decimal</option>
                </select>
              </div>
            {/if}
            {#if selectedComp.default !== undefined}
              <div class="cb-field"><label>Default</label><input type="number" bind:value={selectedComp.default} step="0.01" onchange={updateComp} /></div>
            {/if}
            {#if selectedComp.sensitivity !== undefined}
              <div class="cb-field"><label>Sensitivity</label><input type="number" bind:value={selectedComp.sensitivity} step="0.1" min="0.1" max="3" onchange={updateComp} /></div>
            {/if}
            {#if selectedComp.deadzone !== undefined}
              <div class="cb-field"><label>Deadzone</label><input type="number" bind:value={selectedComp.deadzone} step="0.01" min="0" max="0.3" onchange={updateComp} /></div>
            {/if}
          </div>
        {:else}
          <p class="cb-hint" style="margin-top:8px">Click a component in the grid to edit its properties.</p>
        {/if}

        <!-- Actions -->
        <div class="cb-section" style="margin-top:auto;padding-top:8px;border-top:1px solid var(--color-border)">
          <div class="cb-row" style="gap:4px;flex-wrap:wrap">
            <button class="cb-btn primary" onclick={savePreset} disabled={!dirty || !activePreset}><Save size={12} /> Save</button>
            <button class="cb-btn" onclick={exportPreset}><Download size={12} /> Export</button>
            <div style="flex:1"></div>
            <input type="text" class="cb-save-as-input" placeholder="new-name" bind:value={newPresetName} />
            <button class="cb-btn" onclick={saveAs} disabled={!newPresetName.trim()}>Save As</button>
          </div>
        </div>
      </div>
    </div>

  <!-- PREVIEW TAB -->
  {:else if tab === 'preview' && preset}
    <div class="cb-preview-wrap">
      <div class="cb-preview-bar">
        <span class="label" style="margin:0">PREVIEW: {preset.name}</span>
        <span class="cb-hint">{preset.grid.cols}x{preset.grid.rows} grid &middot; {preset.components.length} components</span>
        <div style="flex:1"></div>
        <a href={cockpitUrl()} target="_blank" class="cb-btn sm"><Smartphone size={11} /> Open in browser</a>
      </div>
      <div class="cb-preview-frame">
        <iframe src="/cockpit2.html?preset={activePreset}" title="Cockpit Preview"></iframe>
      </div>
    </div>
  {/if}
</div>

<style>
  .cb-wrap { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  /* Tabs */
  .cb-tabs { display: flex; border-bottom: 1px solid var(--color-border); flex-shrink: 0; align-items: center; }
  .cb-tab {
    padding: 7px 14px; border: none; background: none;
    color: var(--color-dim); font-family: 'Cascadia Code', monospace;
    font-size: 10px; font-weight: 600; letter-spacing: .5px;
    cursor: pointer; border-bottom: 2px solid transparent;
    display: flex; align-items: center; gap: 5px; transition: all .1s;
  }
  .cb-tab:hover { color: var(--color-fg); }
  .cb-tab.on { color: var(--color-accent); border-bottom-color: var(--color-accent); }
  .cb-tab:disabled { opacity: .3; cursor: default; }
  .cb-tab-spacer { flex: 1; }
  .cb-msg { font-size: 10px; color: var(--color-green); margin-right: 10px; }
  .cb-badge-dirty { font-family: 'Cascadia Code', monospace; font-size: 8px; font-weight: 700; padding: 2px 6px; border-radius: 3px; background: rgba(227,179,65,.1); color: var(--color-yellow); border: 1px solid rgba(227,179,65,.2); margin-right: 8px; }

  /* Content */
  .cb-content { flex: 1; overflow-y: auto; padding: 14px; }

  /* Buttons */
  .cb-btn {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 5px 12px; border: 1px solid var(--color-border); border-radius: 3px;
    background: var(--color-surface); color: var(--color-dim);
    font-size: 11px; font-weight: 600; cursor: pointer; transition: all .1s;
    font-family: inherit;
  }
  .cb-btn:hover { border-color: var(--color-dim); color: var(--color-fg); }
  .cb-btn:disabled { opacity: .3; cursor: default; }
  .cb-btn.primary { background: rgba(74,158,255,.08); border-color: rgba(74,158,255,.3); color: var(--color-accent); }
  .cb-btn.primary:hover { background: rgba(74,158,255,.15); }
  .cb-btn.danger { color: var(--color-red); border-color: rgba(234,74,90,.3); }
  .cb-btn.sm { padding: 3px 8px; font-size: 10px; }

  .cb-row { display: flex; align-items: center; }

  /* Preset list */
  .cb-preset-list { display: flex; flex-direction: column; gap: 4px; }
  .cb-preset-card {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 12px; background: var(--color-surface); border: 1px solid var(--color-border);
    border-radius: 4px; transition: all .1s;
  }
  .cb-preset-card:hover { border-color: var(--color-dim); }
  .cb-preset-card.active { border-color: var(--color-accent); }
  .cb-preset-info { display: flex; flex-direction: column; gap: 2px; }
  .cb-preset-name { font-weight: 600; font-size: 12px; }
  .cb-preset-file { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }
  .cb-preset-actions { display: flex; gap: 4px; }

  .cb-empty { color: var(--color-dim); font-size: 11px; }

  /* URL row */
  .cb-url-row { display: flex; align-items: center; gap: 6px; }
  .cb-url { font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-accent); background: var(--color-surface-2); padding: 4px 8px; border-radius: 3px; }
  .cb-hint { font-size: 9px; color: var(--color-dim); }

  /* Editor layout */
  .cb-editor { flex: 1; display: flex; overflow: hidden; }

  /* Grid preview */
  .cb-grid-wrap { flex: 1; padding: 10px; overflow: auto; display: flex; align-items: center; justify-content: center; }
  .cb-grid {
    display: grid;
    gap: 1px;
    width: 100%;
    max-width: 700px;
    aspect-ratio: 16/9;
    background: #0a0b0d;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    position: relative;
  }
  .cb-grid-cell {
    background: var(--color-surface);
    display: flex; align-items: center; justify-content: center;
    min-height: 0; min-width: 0; position: relative;
  }
  .cb-grid-coord { font-size: 7px; color: #1e2736; }
  .cb-grid-comp {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    border: 1px solid rgba(74,158,255,.15); border-radius: 2px;
    cursor: pointer; z-index: 1; transition: all .1s;
    min-height: 0; min-width: 0; overflow: hidden;
  }
  .cb-grid-comp:hover { border-color: rgba(74,158,255,.4); }
  .cb-grid-comp.selected { border-color: var(--color-accent); box-shadow: 0 0 8px rgba(74,158,255,.15); }
  .cb-grid-comp-type { font-family: 'Cascadia Code', monospace; font-size: 7px; color: var(--color-dim); letter-spacing: .5px; }
  .cb-grid-comp-label { font-size: 9px; font-weight: 600; color: var(--color-fg); }

  /* Properties panel */
  .cb-props {
    width: 260px; flex-shrink: 0;
    border-left: 1px solid var(--color-border);
    padding: 10px; overflow-y: auto;
    display: flex; flex-direction: column;
  }
  .cb-section { margin-bottom: 10px; }
  .cb-section-header { display: flex; align-items: center; justify-content: space-between; }
  .cb-field { margin-bottom: 6px; }
  .cb-field label { display: block; font-size: 9px; color: var(--color-dim); margin-bottom: 2px; font-weight: 600; }
  .cb-field input, .cb-field select {
    width: 100%; padding: 4px 8px;
    background: var(--color-surface-2); border: 1px solid var(--color-border);
    border-radius: 3px; color: var(--color-fg); font-size: 11px;
    font-family: inherit; outline: none;
  }
  .cb-field input:focus, .cb-field select:focus { border-color: var(--color-accent); }
  .cb-field input[type="number"] { width: 100%; }
  .cb-field-row { display: flex; gap: 6px; }
  .cb-field-row .cb-field { flex: 1; }

  .cb-palette { display: flex; flex-wrap: wrap; gap: 3px; }
  .cb-palette-btn {
    padding: 3px 8px; font-size: 9px; font-weight: 600;
    background: var(--color-surface-2); border: 1px solid var(--color-border);
    border-radius: 2px; color: var(--color-dim); cursor: pointer;
    font-family: inherit;
  }
  .cb-palette-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }

  .cb-save-as-input {
    padding: 4px 8px; width: 100px;
    background: var(--color-surface-2); border: 1px solid var(--color-border);
    border-radius: 3px; color: var(--color-fg); font-size: 10px;
    font-family: 'Cascadia Code', monospace; outline: none;
  }
  .cb-save-as-input:focus { border-color: var(--color-accent); }

  /* Preview */
  .cb-preview-wrap { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
  .cb-preview-bar { display: flex; align-items: center; gap: 10px; padding: 8px 14px; border-bottom: 1px solid var(--color-border); }
  .cb-preview-frame { flex: 1; }
  .cb-preview-frame iframe { width: 100%; height: 100%; border: none; background: #111; }
</style>
