<script lang="ts">
  import { onMount } from 'svelte';

  let { simState, mappings }: {
    simState: any;
    mappings: any[];
  } = $props();

  // Track last event per pin
  let lastEvents: Record<string, { value: string; time: number }> = $state({});

  function formatValue(val: number): string {
    if (Number.isInteger(val)) return val.toString();
    return val.toFixed(1);
  }

  onMount(() => {
    // Listen for sim events sent by hardware
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/event').then(({ listen }) => {
        listen('sim-event-sent', (event: any) => {
          const payload = event.payload;
          if (payload.type === 'event') {
            // Find which pin triggered this event
            const m = mappings.find((mp: any) => mp.target === payload.name);
            if (m) {
              lastEvents[String(m.pin)] = { value: 'TRIGGERED', time: Date.now() };
              lastEvents = { ...lastEvents };
            }
          } else if (payload.type === 'var') {
            const m = mappings.find((mp: any) => mp.target === payload.name);
            if (m) {
              lastEvents[String(m.pin)] = { value: String(payload.value), time: Date.now() };
              lastEvents = { ...lastEvents };
            }
          }
        });
      });
    }
  });
</script>

<div class="card">
  <h3>Live Sim Data</h3>

  {#if Object.keys(simState.values).length > 0}
    <div class="monitor-grid">
      {#each Object.entries(simState.values) as [name, value]}
        <div class="monitor-item">
          <div class="label">{name}</div>
          <div class="value">{formatValue(value as number)}</div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty">
      <div class="empty-icon">&#9881;</div>
      {#if simState.connected}
        Connected — waiting for sim data...
      {:else}
        Not connected to MSFS.<br/>
        Start Microsoft Flight Simulator to see live data.
      {/if}
    </div>
  {/if}
</div>

<div class="card" style="margin-top: 16px;">
  <h3>Active Mappings Status</h3>

  {#if mappings.length > 0}
    <table class="table">
      <thead>
        <tr>
          <th>Device</th>
          <th>Pin</th>
          <th>Type</th>
          <th>Target</th>
          <th>Last Value</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        {#each mappings as m}
          {@const target = m.target || m.sim_var || m.sim_event || '—'}
          {@const action = m.action || m.pin_type || '—'}
          {@const le = lastEvents[String(m.pin)]}
          {@const isRecent = le && (Date.now() - le.time) < 2000}
          <tr>
            <td>#{m.device_id || 1}</td>
            <td style="font-family: var(--mono);">{m.pin}</td>
            <td>{action}</td>
            <td style="font-family: var(--mono); font-size: 11px;">{target}</td>
            <td style="font-family: var(--mono); color: {isRecent ? 'var(--pfd-green)' : 'var(--text-dim)'};">
              {le ? le.value : '—'}
            </td>
            <td>
              {#if isRecent}
                <span style="color: var(--pfd-green); font-weight: 600;">FIRED</span>
              {:else if simState.connected}
                <span style="color: var(--green);">Active</span>
              {:else}
                <span style="color: var(--text-dim);">Idle</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <div class="empty">
      <div class="empty-icon">&#128279;</div>
      No active mappings. Configure pin mappings to see status here.
    </div>
  {/if}
</div>
