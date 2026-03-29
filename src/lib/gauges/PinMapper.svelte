<script lang="ts">
  import { invoke } from '../mock';
  import { toast } from './Toast.svelte';
  import { onMount } from 'svelte';

  let { mappings, devices, onrefresh }: {
    mappings: any[];
    devices: any[];
    onrefresh: () => void;
  } = $props();

  let simvars: any[] = $state([]);
  let showAdd = $state(false);
  let newMapping = $state({
    device_id: 1,
    pin: 0,
    pin_type: 'DigitalInput',
    sim_var: null as string | null,
    sim_event: null as string | null,
    range_min: 0,
    range_max: 100,
  });

  const pinTypes = [
    { value: 'DigitalInput', label: 'Digital Input (Button/Switch)' },
    { value: 'DigitalOutput', label: 'Digital Output (LED)' },
    { value: 'AnalogInput', label: 'Analog Input (Potentiometer)' },
    { value: 'AnalogOutput', label: 'Analog Output (Servo/PWM)' },
    { value: 'Encoder', label: 'Rotary Encoder' },
  ];

  const commonEvents = [
    'TOGGLE_MASTER_BATTERY',
    'TOGGLE_MASTER_ALTERNATOR',
    'GEAR_TOGGLE',
    'FLAPS_UP',
    'FLAPS_DOWN',
    'FLAPS_INCR',
    'FLAPS_DECR',
    'AP_MASTER',
    'HEADING_BUG_INC',
    'HEADING_BUG_DEC',
    'AP_ALT_VAR_INC',
    'AP_ALT_VAR_DEC',
    'TOGGLE_NAV_LIGHTS',
    'TOGGLE_BEACON_LIGHTS',
    'TOGGLE_LANDING_LIGHTS',
    'TOGGLE_TAXI_LIGHTS',
    'TOGGLE_LOGO_LIGHTS',
    'PARKING_BRAKES',
    'SPOILERS_TOGGLE',
  ];

  onMount(async () => {
    try {
      simvars = await invoke('get_default_simvars');
    } catch (e) {
      console.error(e);
    }
  });

  async function addMapping() {
    await invoke('add_mapping', { mapping: newMapping });
    showAdd = false;
    toast('Mapping added', 'success');
    onrefresh();
  }

  async function removeMapping(deviceId: number, pin: number) {
    await invoke('remove_mapping', { deviceId, pin });
    toast('Mapping removed', 'info');
    onrefresh();
  }

  function pinTypeName(pt: string): string {
    return pinTypes.find(p => p.value === pt)?.label ?? pt;
  }
</script>

<div class="card">
  <h3>
    Pin Mappings
    <button class="btn btn-primary btn-sm" style="margin-left: auto" onclick={() => showAdd = !showAdd}>
      {showAdd ? 'Cancel' : '+ Add Mapping'}
    </button>
  </h3>

  {#if showAdd}
    <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; margin-bottom: 16px; padding: 12px; background: var(--bg); border-radius: var(--radius);">
      <label style="font-size: 11px; color: var(--text-dim);">
        Device
        <select bind:value={newMapping.device_id} style="width: 100%; margin-top: 4px;">
          {#each devices as d}
            <option value={d.id}>{d.name} (ID: {d.id})</option>
          {:else}
            <option value={1}>No devices — add one first</option>
          {/each}
        </select>
      </label>
      <label style="font-size: 11px; color: var(--text-dim);">
        Pin Number
        <input type="number" bind:value={newMapping.pin} min="0" max="255" style="width: 100%; margin-top: 4px;" />
      </label>
      <label style="font-size: 11px; color: var(--text-dim);">
        Pin Type
        <select bind:value={newMapping.pin_type} style="width: 100%; margin-top: 4px;">
          {#each pinTypes as pt}
            <option value={pt.value}>{pt.label}</option>
          {/each}
        </select>
      </label>

      {#if newMapping.pin_type === 'DigitalInput' || newMapping.pin_type === 'Encoder'}
        <label style="font-size: 11px; color: var(--text-dim); grid-column: 1 / -1;">
          Sim Event
          <select bind:value={newMapping.sim_event} style="width: 100%; margin-top: 4px;">
            <option value={null}>— Select Event —</option>
            {#each commonEvents as ev}
              <option value={ev}>{ev}</option>
            {/each}
          </select>
        </label>
      {/if}

      {#if newMapping.pin_type === 'AnalogInput' || newMapping.pin_type === 'AnalogOutput' || newMapping.pin_type === 'DigitalOutput'}
        <label style="font-size: 11px; color: var(--text-dim); grid-column: 1 / -1;">
          Sim Variable
          <select bind:value={newMapping.sim_var} style="width: 100%; margin-top: 4px;">
            <option value={null}>— Select Variable —</option>
            {#each simvars as sv}
              <option value={sv.name}>{sv.name} ({sv.unit})</option>
            {/each}
          </select>
        </label>
      {/if}

      {#if newMapping.pin_type === 'AnalogInput'}
        <label style="font-size: 11px; color: var(--text-dim);">
          Range Min
          <input type="number" bind:value={newMapping.range_min} style="width: 100%; margin-top: 4px;" />
        </label>
        <label style="font-size: 11px; color: var(--text-dim);">
          Range Max
          <input type="number" bind:value={newMapping.range_max} style="width: 100%; margin-top: 4px;" />
        </label>
      {/if}

      <div style="grid-column: 1 / -1; display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px;">
        <button class="btn btn-primary btn-sm" onclick={addMapping}>Save Mapping</button>
      </div>
    </div>
  {/if}

  {#if mappings.length > 0}
    <table class="table">
      <thead>
        <tr>
          <th>Device</th>
          <th>Pin</th>
          <th>Type</th>
          <th>Target</th>
          <th>Range</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each mappings as m}
          <tr>
            <td>{devices.find(d => d.id === m.device_id)?.name ?? `#${m.device_id}`}</td>
            <td style="font-family: var(--mono);">{m.pin}</td>
            <td>{pinTypeName(m.pin_type)}</td>
            <td style="font-family: var(--mono); font-size: 11px;">{m.sim_var ?? m.sim_event ?? '—'}</td>
            <td style="font-family: var(--mono); font-size: 11px;">
              {#if m.pin_type === 'AnalogInput'}
                {m.range_min} → {m.range_max}
              {:else}
                —
              {/if}
            </td>
            <td>
              <button class="btn btn-danger btn-sm" onclick={() => removeMapping(m.device_id, m.pin)}>Remove</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <div class="empty">
      <div class="empty-icon">&#128279;</div>
      No pin mappings yet.<br/>Map hardware pins to simulator variables above.
    </div>
  {/if}
</div>
