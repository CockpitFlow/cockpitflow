<script lang="ts">
  import { invoke } from '../mock';
  import { toast } from './Toast.svelte';

  let mode = $state<'select' | 'aircraft' | 'device' | 'mapping'>('select');
  let existingAircraft: any[] = $state([]);
  let existingDevices: any[] = $state([]);

  // ─── Aircraft Editor ───
  let aircraft = $state({
    id: '', name: '', author: '', description: '', category: 'GA', simulator: 'both',
    variables: [] as any[], events: [] as any[],
  });
  let newVar = $state({ name: '', unit: '', writable: false, category: 'flight', label: '' });
  let newEvent = $state({ name: '', category: 'flight', label: '' });

  const categories = ['flight', 'engine', 'autopilot', 'electrical', 'fuel', 'radio', 'lights', 'misc'];
  const acCategories = ['GA', 'airliner', 'military', 'helicopter', 'glider'];
  const commonUnits = ['feet', 'knots', 'degrees', 'percent', 'bool', 'number', 'rpm', 'mhz', 'khz', 'volts', 'amps', 'psi', 'celsius', 'gallons', 'pounds', 'feet per minute', 'inHg', 'mach', 'enum', 'position'];

  // ─── Device Editor ───
  let device = $state({
    id: '', name: '', author: '', description: '', board: 'esp32-c6', transport: 'serial',
    pins: [] as any[],
  });
  let newPin = $state({ pin: '', type: 'button', label: '', options: {} as any });
  const pinTypes = ['button', 'toggle', 'encoder', 'potentiometer', 'led', 'servo', 'display', 'stepper'];
  const boardChoices = ['esp32-c6', 'esp32-s3', 'esp32', 'arduino-mega', 'arduino-uno', 'arduino-nano', 'custom'];

  // ─── Mapping Editor ───
  let mapping = $state({
    id: '', name: '', author: '', description: '',
    device_profile: '', aircraft_profile: '',
    mappings: [] as any[],
  });
  let newMap = $state({ pin: '', action: 'event', target: '', options: {} as any });
  let selectedAircraftVars: any[] = $state([]);
  let selectedAircraftEvents: any[] = $state([]);

  async function loadExisting() {
    try {
      existingAircraft = await invoke('get_aircraft_profiles');
      existingDevices = await invoke('get_device_profiles');
    } catch {}
  }

  function autoId(name: string): string {
    return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  }

  // ─── Aircraft Actions ───
  function addVar() {
    if (!newVar.name) return;
    aircraft.variables = [...aircraft.variables, { ...newVar }];
    newVar = { name: '', unit: '', writable: false, category: newVar.category, label: '' };
    toast('Variable added', 'success');
  }

  function addEvent() {
    if (!newEvent.name) return;
    aircraft.events = [...aircraft.events, { ...newEvent }];
    newEvent = { name: '', category: newEvent.category, label: '' };
    toast('Event added', 'success');
  }

  function removeVar(i: number) { aircraft.variables = aircraft.variables.filter((_: any, j: number) => j !== i); }
  function removeEvent(i: number) { aircraft.events = aircraft.events.filter((_: any, j: number) => j !== i); }

  // ─── Device Actions ───
  function addPin() {
    if (!newPin.pin) return;
    const opts = newPin.type === 'encoder' ? { pin_b: 0, steps_per_detent: 4 } :
                 newPin.type === 'potentiometer' ? { deadzone: 3 } :
                 newPin.type === 'led' ? { pwm: false } : {};
    device.pins = [...device.pins, { ...newPin, options: { ...opts, ...newPin.options } }];
    newPin = { pin: '', type: newPin.type, label: '', options: {} };
    toast('Pin added', 'success');
  }

  function removePin(i: number) { device.pins = device.pins.filter((_: any, j: number) => j !== i); }

  // ─── Mapping Actions ───
  async function onAircraftSelect() {
    const ac = existingAircraft.find((a: any) => a.id === mapping.aircraft_profile);
    if (ac) {
      selectedAircraftVars = ac.variables || [];
      selectedAircraftEvents = ac.events || [];
    }
  }

  function addMapping() {
    if (!newMap.pin || !newMap.target) return;
    mapping.mappings = [...mapping.mappings, { ...newMap }];
    newMap = { pin: '', action: newMap.action, target: '', options: {} };
    toast('Mapping added', 'success');
  }

  function removeMapping(i: number) { mapping.mappings = mapping.mappings.filter((_: any, j: number) => j !== i); }

  // ─── Save / Export ───
  function exportAsJson(type: string) {
    let data: any;
    if (type === 'aircraft') {
      data = { $schema: 'aircraft', version: 1, ...aircraft, id: aircraft.id || autoId(aircraft.name) };
    } else if (type === 'device') {
      data = { $schema: 'device', version: 1, ...device, id: device.id || autoId(device.name) };
    } else {
      data = { $schema: 'mapping', version: 1, ...mapping, id: mapping.id || autoId(mapping.name) };
    }

    const json = JSON.stringify(data, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${data.id}.json`;
    a.click();
    URL.revokeObjectURL(url);
    toast(`Profile "${data.name}" exported`, 'success');
  }

  async function saveProfile(type: string) {
    let data: any;
    if (type === 'aircraft') {
      data = { $schema: 'aircraft', version: 1, ...aircraft, id: aircraft.id || autoId(aircraft.name) };
    } else if (type === 'device') {
      data = { $schema: 'device', version: 1, ...device, id: device.id || autoId(device.name) };
    } else {
      data = { $schema: 'mapping', version: 1, ...mapping, id: mapping.id || autoId(mapping.name) };
    }

    try {
      await invoke('import_profile', { json: JSON.stringify(data) });
      toast(`Profile "${data.name}" saved!`, 'success');
    } catch (e: any) {
      toast(`Save failed: ${e}`, 'error');
    }
  }

  $effect(() => { if (mode !== 'select') loadExisting(); });
</script>

<!-- Mode Select -->
{#if mode === 'select'}
  <div class="card">
    <h3>Create New Profile</h3>
    <p style="font-size: 12px; color: var(--text-secondary); margin-top: -8px; margin-bottom: 16px">
      Build profiles for your cockpit setup. Share them with the community.
    </p>
    <div class="creator-grid">
      <button class="creator-card" onclick={() => mode = 'aircraft'}>
        <div class="creator-icon">&#x2708;</div>
        <div class="creator-title">Aircraft Profile</div>
        <div class="creator-desc">Define SimConnect variables and events for a specific aircraft</div>
      </button>
      <button class="creator-card" onclick={() => mode = 'device'}>
        <div class="creator-icon">&#x1F527;</div>
        <div class="creator-title">Device Profile</div>
        <div class="creator-desc">Describe the pin layout of your hardware panel</div>
      </button>
      <button class="creator-card" onclick={() => mode = 'mapping'}>
        <div class="creator-icon">&#x1F517;</div>
        <div class="creator-title">Mapping Profile</div>
        <div class="creator-desc">Connect device pins to aircraft variables and events</div>
      </button>
    </div>
  </div>

<!-- ═══ Aircraft Editor ═══ -->
{:else if mode === 'aircraft'}
  <div class="card">
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px">
      <h3 style="margin: 0">&#x2708; Aircraft Profile Editor</h3>
      <button class="btn btn-sm" onclick={() => mode = 'select'}>Back</button>
    </div>

    <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; margin-bottom: 16px">
      <div><label class="field-label">Aircraft Name</label>
        <input type="text" bind:value={aircraft.name} placeholder="e.g. Piper PA-28 Cherokee" style="width: 100%" oninput={() => aircraft.id = autoId(aircraft.name)} /></div>
      <div><label class="field-label">Category</label>
        <select bind:value={aircraft.category} style="width: 100%">
          {#each acCategories as c}<option value={c}>{c}</option>{/each}
        </select></div>
      <div><label class="field-label">Author</label>
        <input type="text" bind:value={aircraft.author} placeholder="Your name" style="width: 100%" /></div>
    </div>
    <div style="margin-bottom: 16px"><label class="field-label">Description</label>
      <input type="text" bind:value={aircraft.description} placeholder="Short description..." style="width: 100%" /></div>

    <!-- Add Variable -->
    <div style="padding: 12px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius); margin-bottom: 12px">
      <div style="font-size: 12px; font-weight: 600; margin-bottom: 8px">Add SimConnect Variable</div>
      <div style="display: grid; grid-template-columns: 2fr 1fr 1fr 1fr auto; gap: 8px; align-items: end">
        <div><label class="field-label">SimVar Name</label>
          <input type="text" bind:value={newVar.name} placeholder="e.g. INDICATED ALTITUDE" style="width: 100%" /></div>
        <div><label class="field-label">Unit</label>
          <select bind:value={newVar.unit} style="width: 100%">
            <option value="">select...</option>
            {#each commonUnits as u}<option value={u}>{u}</option>{/each}
          </select></div>
        <div><label class="field-label">Category</label>
          <select bind:value={newVar.category} style="width: 100%">
            {#each categories as c}<option value={c}>{c}</option>{/each}
          </select></div>
        <div><label class="field-label">Label</label>
          <input type="text" bind:value={newVar.label} placeholder="Altitude" style="width: 100%" /></div>
        <button class="btn btn-primary btn-sm" onclick={addVar}>Add</button>
      </div>
      <label style="display: flex; align-items: center; gap: 6px; margin-top: 6px; font-size: 11px; color: var(--text-dim); cursor: pointer">
        <input type="checkbox" bind:checked={newVar.writable} style="accent-color: var(--accent)" /> Writable
      </label>
    </div>

    <!-- Add Event -->
    <div style="padding: 12px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius); margin-bottom: 12px">
      <div style="font-size: 12px; font-weight: 600; margin-bottom: 8px">Add Key Event</div>
      <div style="display: grid; grid-template-columns: 2fr 1fr 1fr auto; gap: 8px; align-items: end">
        <div><label class="field-label">Event Name</label>
          <input type="text" bind:value={newEvent.name} placeholder="e.g. TOGGLE_MASTER_BATTERY" style="width: 100%" /></div>
        <div><label class="field-label">Category</label>
          <select bind:value={newEvent.category} style="width: 100%">
            {#each categories as c}<option value={c}>{c}</option>{/each}
          </select></div>
        <div><label class="field-label">Label</label>
          <input type="text" bind:value={newEvent.label} placeholder="Toggle Battery" style="width: 100%" /></div>
        <button class="btn btn-primary btn-sm" onclick={addEvent}>Add</button>
      </div>
    </div>

    <!-- Variable List -->
    {#if aircraft.variables.length > 0}
      <div style="margin-bottom: 12px">
        <div style="font-size: 11px; font-weight: 600; color: var(--text-dim); margin-bottom: 6px">
          VARIABLES ({aircraft.variables.length})
        </div>
        <div style="display: flex; flex-wrap: wrap; gap: 4px">
          {#each aircraft.variables as v, i}
            <span class="tag-item">
              <span class="tag-cat" style="color: var(--accent)">{v.category}</span>
              {v.label || v.name}
              <span style="color: var(--text-dim)">({v.unit})</span>
              {#if v.writable}<span style="color: var(--orange); font-size: 9px">RW</span>{/if}
              <button class="tag-remove" onclick={() => removeVar(i)}>&times;</button>
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Event List -->
    {#if aircraft.events.length > 0}
      <div style="margin-bottom: 12px">
        <div style="font-size: 11px; font-weight: 600; color: var(--text-dim); margin-bottom: 6px">
          EVENTS ({aircraft.events.length})
        </div>
        <div style="display: flex; flex-wrap: wrap; gap: 4px">
          {#each aircraft.events as e, i}
            <span class="tag-item tag-event">
              <span class="tag-cat">{e.category}</span>
              {e.label || e.name}
              <button class="tag-remove" onclick={() => removeEvent(i)}>&times;</button>
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Save -->
    <div style="display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--border)">
      <button class="btn btn-sm" onclick={() => exportAsJson('aircraft')}>Export JSON</button>
      <button class="btn btn-primary" onclick={() => saveProfile('aircraft')}
        disabled={!aircraft.name || aircraft.variables.length + aircraft.events.length === 0}>
        Save Aircraft Profile
      </button>
    </div>
  </div>

<!-- ═══ Device Editor ═══ -->
{:else if mode === 'device'}
  <div class="card">
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px">
      <h3 style="margin: 0">&#x1F527; Device Profile Editor</h3>
      <button class="btn btn-sm" onclick={() => mode = 'select'}>Back</button>
    </div>

    <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; margin-bottom: 16px">
      <div><label class="field-label">Device Name</label>
        <input type="text" bind:value={device.name} placeholder="e.g. MCP Panel v2" style="width: 100%" oninput={() => device.id = autoId(device.name)} /></div>
      <div><label class="field-label">Board</label>
        <select bind:value={device.board} style="width: 100%">
          {#each boardChoices as b}<option value={b}>{b}</option>{/each}
        </select></div>
      <div><label class="field-label">Transport</label>
        <select bind:value={device.transport} style="width: 100%">
          <option value="serial">Serial (USB)</option>
          <option value="udp">UDP (WiFi)</option>
        </select></div>
    </div>

    <!-- Add Pin -->
    <div style="padding: 12px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius); margin-bottom: 12px">
      <div style="font-size: 12px; font-weight: 600; margin-bottom: 8px">Add Pin</div>
      <div style="display: grid; grid-template-columns: 80px 1fr 2fr auto; gap: 8px; align-items: end">
        <div><label class="field-label">Pin</label>
          <input type="text" bind:value={newPin.pin} placeholder="2 or A0" style="width: 100%" /></div>
        <div><label class="field-label">Type</label>
          <select bind:value={newPin.type} style="width: 100%">
            {#each pinTypes as t}<option value={t}>{t}</option>{/each}
          </select></div>
        <div><label class="field-label">Label</label>
          <input type="text" bind:value={newPin.label} placeholder="e.g. Master Battery Switch" style="width: 100%" /></div>
        <button class="btn btn-primary btn-sm" onclick={addPin}>Add</button>
      </div>
    </div>

    <!-- Pin List -->
    {#if device.pins.length > 0}
      <table class="table">
        <thead><tr><th>Pin</th><th>Type</th><th>Label</th><th></th></tr></thead>
        <tbody>
          {#each device.pins as p, i}
            <tr>
              <td style="font-family: var(--mono); font-weight: 600">{p.pin}</td>
              <td><span class="pin-type-badge">{p.type}</span></td>
              <td>{p.label}</td>
              <td><button class="btn btn-danger btn-sm" onclick={() => removePin(i)}>&times;</button></td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <div style="display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--border)">
      <button class="btn btn-sm" onclick={() => exportAsJson('device')}>Export JSON</button>
      <button class="btn btn-primary" onclick={() => saveProfile('device')}
        disabled={!device.name || device.pins.length === 0}>
        Save Device Profile
      </button>
    </div>
  </div>

<!-- ═══ Mapping Editor ═══ -->
{:else if mode === 'mapping'}
  <div class="card">
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px">
      <h3 style="margin: 0">&#x1F517; Mapping Profile Editor</h3>
      <button class="btn btn-sm" onclick={() => mode = 'select'}>Back</button>
    </div>

    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 12px">
      <div><label class="field-label">Mapping Name</label>
        <input type="text" bind:value={mapping.name} placeholder="e.g. My 737 MCP Setup" style="width: 100%" oninput={() => mapping.id = autoId(mapping.name)} /></div>
      <div><label class="field-label">Author</label>
        <input type="text" bind:value={mapping.author} placeholder="Your name" style="width: 100%" /></div>
    </div>

    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 16px">
      <div><label class="field-label">Device Profile</label>
        <select bind:value={mapping.device_profile} style="width: 100%">
          <option value="">Select device...</option>
          {#each existingDevices as d}<option value={d.id}>{d.name}</option>{/each}
        </select></div>
      <div><label class="field-label">Aircraft Profile</label>
        <select bind:value={mapping.aircraft_profile} style="width: 100%" onchange={onAircraftSelect}>
          <option value="">Select aircraft...</option>
          {#each existingAircraft as a}<option value={a.id}>{a.name}</option>{/each}
        </select></div>
    </div>

    <!-- Add Mapping -->
    {#if mapping.device_profile && mapping.aircraft_profile}
      <div style="padding: 12px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius); margin-bottom: 12px">
        <div style="font-size: 12px; font-weight: 600; margin-bottom: 8px">Add Pin Mapping</div>
        <div style="display: grid; grid-template-columns: 80px 100px 1fr auto; gap: 8px; align-items: end">
          <div><label class="field-label">Pin</label>
            <input type="text" bind:value={newMap.pin} placeholder="2" style="width: 100%" /></div>
          <div><label class="field-label">Action</label>
            <select bind:value={newMap.action} style="width: 100%">
              <option value="event">Event</option>
              <option value="set_var">Set Var</option>
              <option value="read_var">Read Var</option>
            </select></div>
          <div><label class="field-label">Target</label>
            <select bind:value={newMap.target} style="width: 100%">
              <option value="">Select target...</option>
              {#if newMap.action === 'event'}
                {#each selectedAircraftEvents as e}<option value={e.name}>{e.label || e.name}</option>{/each}
              {:else}
                {#each selectedAircraftVars as v}<option value={v.name}>{v.label || v.name} ({v.unit})</option>{/each}
              {/if}
            </select></div>
          <button class="btn btn-primary btn-sm" onclick={addMapping}>Add</button>
        </div>
      </div>
    {/if}

    <!-- Mapping List -->
    {#if mapping.mappings.length > 0}
      <table class="table">
        <thead><tr><th>Pin</th><th>Action</th><th>Target</th><th></th></tr></thead>
        <tbody>
          {#each mapping.mappings as m, i}
            <tr>
              <td style="font-family: var(--mono); font-weight: 600">{m.pin}</td>
              <td><span class="pin-type-badge" style="background: {m.action === 'event' ? 'rgba(79,143,234,0.15)' : 'rgba(167,139,250,0.15)'}; color: {m.action === 'event' ? 'var(--accent)' : 'var(--purple)'}">{m.action}</span></td>
              <td style="font-family: var(--mono); font-size: 11px">{m.target}</td>
              <td><button class="btn btn-danger btn-sm" onclick={() => removeMapping(i)}>&times;</button></td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <div style="display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--border)">
      <button class="btn btn-sm" onclick={() => exportAsJson('mapping')}>Export JSON</button>
      <button class="btn btn-primary" onclick={() => saveProfile('mapping')}
        disabled={!mapping.name || mapping.mappings.length === 0}>
        Save Mapping Profile
      </button>
    </div>
  </div>
{/if}

<style>
  .creator-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }
  .creator-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 24px 16px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
    font-family: var(--font);
    color: var(--text);
    width: 100%;
  }
  .creator-card:hover {
    border-color: var(--accent);
    background: var(--accent-soft);
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }
  .creator-icon { font-size: 32px; margin-bottom: 8px; }
  .creator-title { font-size: 14px; font-weight: 600; margin-bottom: 4px; }
  .creator-desc { font-size: 11px; color: var(--text-dim); line-height: 1.4; }

  .field-label {
    display: block;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-dim);
    margin-bottom: 4px;
  }

  .tag-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 11px;
    color: var(--text-secondary);
  }
  .tag-event { border-color: rgba(79,143,234,0.2); }
  .tag-cat {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }
  .tag-remove {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
  }
  .tag-remove:hover { color: var(--red); }

  .pin-type-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-weight: 500;
    text-transform: uppercase;
  }
</style>
