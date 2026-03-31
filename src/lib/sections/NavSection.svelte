<script lang="ts">
  import { onMount } from 'svelte';
  import { toast } from '../gauges/Toast.svelte';

  let {
    onWheel = (_e: WheelEvent) => {},
    simData = {},
  }: {
    onWheel?: (e: WheelEvent) => void;
    simData?: any;
  } = $props();

  let navTab = $state('wb');
  let navPresets = $state<string[]>([]);
  let navActivePreset = $state('cessna-172');
  let navPresetNames = $state<Record<string, string>>({});
  let navData = $state<any>(null);

  // W&B
  let paxWeight = $state(170);
  let baggageWeight = $state(30);
  let fuelGallons = $state(40);
  let EW = $derived(navData?.weight?.empty_weight_lbs || 1680);
  let ECG = $derived(navData?.weight?.arm_front_seats || 40.5);
  let FA = $derived(navData?.weight?.arm_fuel || 48);
  let PA = $derived(navData?.weight?.arm_front_seats || 37);
  let BA = $derived(navData?.weight?.arm_baggage || 95);
  let maxTakeoff = $derived(navData?.weight?.max_takeoff_lbs || 2550);
  let cgFwd = $derived(navData?.weight?.cg_range_fwd || 35);
  let cgAft = $derived(navData?.weight?.cg_range_aft || 47.3);
  let fuelPerGal = $derived(navData?.fuel?.fuel_weight_lbs_per_gal || 6);
  let maxFuelGal = $derived(navData?.fuel?.usable_fuel_gal || navData?.fuel?.total_capacity_gal || 56);
  let tw = $derived(EW + paxWeight + baggageWeight + fuelGallons * fuelPerGal);
  let cg = $derived((EW * ECG + paxWeight * PA + baggageWeight * BA + fuelGallons * fuelPerGal * FA) / tw);
  let ok = $derived(tw <= maxTakeoff && cg >= cgFwd && cg <= cgAft);

  // Crosswind
  let rwyHeading = $state(360);
  let windDir = $state(270);
  let windSpd = $state(10);
  let windAngle = $derived(Math.abs(windDir - rwyHeading) * Math.PI / 180);
  let crosswind = $derived(Math.abs(Math.round(windSpd * Math.sin(windAngle))));
  let headwind = $derived(Math.round(windSpd * Math.cos(windAngle)));

  // Density altitude
  let fieldElev = $state(0);
  let oat = $state(15);
  let altSetting = $state(29.92);
  let pressAlt = $derived(Math.round(fieldElev + (29.92 - altSetting) * 1000));
  let densAlt = $derived(Math.round(pressAlt + 120 * (oat - (15 - 2 * pressAlt / 1000))));

  // Fuel planner
  let tripDist = $state(100);
  let cruiseSpeed = $state(110);
  let fuelBurn = $state(8.5);
  let reserveMin = $state(45);
  $effect(() => {
    if (navData) {
      cruiseSpeed = navData.performance?.cruise_speed_ktas || 110;
      fuelBurn = navData.fuel?.burn_rate_gph_cruise || 8.5;
    }
  });
  let tripTime = $derived(tripDist / cruiseSpeed * 60);
  let tripFuel = $derived(tripDist / cruiseSpeed * fuelBurn);
  let reserveFuel = $derived(reserveMin / 60 * fuelBurn);
  let totalFuelReq = $derived(tripFuel + reserveFuel);

  async function loadNavPresets() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        const modules = await invoke('list_modules') as any[];
        const nav = modules.find((m: any) => m.id === 'nav');
        navPresets = nav?.presets || ['cessna-172'];
        const names: Record<string, string> = {};
        for (const id of navPresets) {
          try {
            const json = await invoke('load_module_preset', { moduleId: 'nav', presetId: id }) as string;
            const data = JSON.parse(json);
            names[id] = `${data.name || id} (${data.author || 'Unknown'} v${data.version || '1.0'})`;
          } catch { names[id] = id; }
        }
        navPresetNames = names;
        loadNavData();
      }
    } catch {}
  }

  async function loadNavData(presetId?: string) {
    const id = presetId || navActivePreset;
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        const json = await invoke('load_module_preset', { moduleId: 'nav', presetId: id }) as string;
        navData = JSON.parse(json);
      }
    } catch (e) { console.error('loadNavData failed:', e); navData = null; }
  }

  async function setNavPreset(preset: string) {
    navActivePreset = preset;
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_module', { moduleId: 'nav', field: 'default_preset', value: preset });
        const json = await invoke('load_module_preset', { moduleId: 'nav', presetId: preset }) as string;
        navData = JSON.parse(json);
      }
      toast('Aircraft changed to ' + preset, 'success');
    } catch (e) { console.error('setNavPreset failed:', e); }
  }

  onMount(() => {
    loadNavPresets();
  });
</script>

<div class="w-full h-full overflow-hidden flex flex-col bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md">
  <!-- Aircraft selector -->
  <div class="flex items-center gap-2 px-3.5 py-2 border-b border-[var(--color-border)]">
    <span class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)]">AIRCRAFT</span>
    <select class="min-w-[200px] px-2.5 py-1.5 bg-[var(--color-bg)] border border-[var(--color-border)] rounded text-[var(--color-fg)] font-mono text-xs outline-none focus:border-[var(--color-accent)]" value={navActivePreset} onchange={(e) => setNavPreset((e.target as HTMLSelectElement).value)}>
      {#each navPresets as p}
        <option value={p}>{navPresetNames[p] || p}</option>
      {/each}
    </select>
    {#if navData}
      <span class="text-[10px] text-[var(--color-dim)]">{navData.category || ''} · {navData.engine?.type || ''}</span>
    {/if}
  </div>
  <!-- Sub-tabs -->
  <div class="flex border-b border-[var(--color-border)] shrink-0">
    {#each [['wb','W&B'],['wind','Wind'],['fuel','Fuel'],['perf','Performance'],['ref','Reference']] as [id, label]}
      <button class="px-4 py-1.5 border-none bg-transparent text-[var(--color-dim)] font-mono text-[10px] font-semibold tracking-wide cursor-pointer border-b-2 border-transparent transition-all hover:text-[var(--color-fg)] {navTab === id ? 'text-[var(--color-accent)]! border-b-[var(--color-accent)]!' : ''}" onclick={() => navTab=id}>{label}</button>
    {/each}
  </div>

  <div class="flex-1 overflow-y-auto p-3.5">
    {#if navTab === 'wb'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">LOADING</div>
          <table class="w-full border-collapse text-[13px]">
            <thead><tr><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">ITEM</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">WEIGHT</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">ARM</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">MOMENT</th></tr></thead>
            <tbody>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Empty Aircraft</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{EW}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{ECG}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{(EW*ECG).toFixed(0)}</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Pilot + Passengers</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50"><input type="number" onwheel={onWheel} class="w-16 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-accent)] px-1 rounded-sm font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" value={paxWeight} min="0" max="800" oninput={(e: Event) => paxWeight=+(e.target as HTMLInputElement).value} /></td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{PA}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{(paxWeight*PA).toFixed(0)}</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Baggage</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50"><input type="number" onwheel={onWheel} class="w-16 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-accent)] px-1 rounded-sm font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" value={baggageWeight} min="0" max="{navData?.weight?.max_baggage_lbs || 120}" oninput={(e: Event) => baggageWeight=+(e.target as HTMLInputElement).value} /></td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{BA}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{(baggageWeight*BA).toFixed(0)}</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Fuel ({fuelGallons} gal)</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50"><input type="number" onwheel={onWheel} class="w-16 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-accent)] px-1 rounded-sm font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" value={fuelGallons} min="0" max="{maxFuelGal}" oninput={(e: Event) => fuelGallons=+(e.target as HTMLInputElement).value} /></td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{FA}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{(fuelGallons*6*FA).toFixed(0)}</td></tr>
              <tr><td class="px-2 py-1.5 border-t-2 border-[var(--color-border)] font-bold text-[var(--color-bright)]">TOTAL</td><td class="px-2 py-1.5 border-t-2 border-[var(--color-border)] font-bold text-[var(--color-bright)] font-mono">{tw}</td><td class="px-2 py-1.5 border-t-2 border-[var(--color-border)] font-bold text-[var(--color-bright)] font-mono">{cg.toFixed(1)}</td><td class="px-2 py-1.5 border-t-2 border-[var(--color-border)] font-bold text-[var(--color-bright)] font-mono">{(tw*cg).toFixed(0)}</td></tr>
            </tbody>
          </table>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">RESULT</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">GROSS WEIGHT</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{tw} <small class="text-[10px] font-medium text-[var(--color-dim)]">lbs</small></span><span class="font-mono text-[9px] text-[var(--color-dim)]">max {maxTakeoff}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">CG POSITION</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{cg.toFixed(1)}<small class="text-[10px] font-medium text-[var(--color-dim)]">"</small></span><span class="font-mono text-[9px] text-[var(--color-dim)]">{cgFwd} — {cgAft}</span></div>
          </div>
          {#if ok}
            <div class="mt-2.5 px-3 py-1.5 rounded font-mono text-[11px] font-extrabold tracking-[2px] text-center bg-[var(--color-green)]/8 border border-[var(--color-green)]/20 text-[var(--color-green)]">WITHIN ENVELOPE</div>
          {:else}
            <div class="mt-2.5 px-3 py-1.5 rounded font-mono text-[11px] font-extrabold tracking-[2px] text-center bg-[var(--color-red)]/8 border border-[var(--color-red)]/20 text-[var(--color-red)]">OUT OF LIMITS</div>
          {/if}
        </div>
      </div>

    {:else if navTab === 'wind'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">CROSSWIND COMPONENT</div>
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Runway Heading</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={rwyHeading} min="1" max="360" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">°</span></div></div>
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Wind Direction</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={windDir} min="1" max="360" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">°</span></div></div>
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Wind Speed</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={windSpd} min="0" max="99" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">kt</span></div></div>
          </div>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">COMPONENTS</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">CROSSWIND</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{crosswind} <small class="text-[10px] font-medium text-[var(--color-dim)]">kt</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">{headwind >= 0 ? 'HEADWIND' : 'TAILWIND'}</span><span class="font-mono text-2xl font-extrabold leading-tight {headwind < 0 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{Math.abs(headwind)} <small class="text-[10px] font-medium text-[var(--color-dim)]">kt</small></span></div>
          </div>
          {#if headwind < 0}<div class="mt-2 px-2.5 py-1.5 rounded bg-[var(--color-yellow)]/5 border border-[var(--color-yellow)]/15 text-[var(--color-yellow)] text-[11px]">Tailwind component — check runway performance</div>{/if}
        </div>
      </div>

    {:else if navTab === 'fuel'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">TRIP PLANNER</div>
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Distance</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={tripDist} min="0" max="999" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">nm</span></div></div>
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Ground Speed</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={cruiseSpeed} min="1" max="200" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">kt</span></div></div>
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Fuel Burn</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={fuelBurn} min="1" max="20" step="0.5" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">GPH</span></div></div>
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">VFR Reserve</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={reserveMin} min="0" max="120" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">min</span></div></div>
          </div>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">CALCULATION</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">TRIP TIME</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{tripTime.toFixed(0)} <small class="text-[10px] font-medium text-[var(--color-dim)]">min</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">TRIP FUEL</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{tripFuel.toFixed(1)} <small class="text-[10px] font-medium text-[var(--color-dim)]">gal</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">RESERVE</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{reserveFuel.toFixed(1)} <small class="text-[10px] font-medium text-[var(--color-dim)]">gal</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-accent)]/25 bg-[var(--color-accent)]/5 rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">TOTAL REQUIRED</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{totalFuelReq.toFixed(1)} <small class="text-[10px] font-medium text-[var(--color-dim)]">gal</small></span></div>
          </div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)] mt-3">LIVE FROM SIM</div>
          <div class="grid grid-cols-3 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">FLOW</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{(simData?.fuel_flow??0).toFixed(1)} <small class="text-[10px] font-medium text-[var(--color-dim)]">GPH</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">QUANTITY</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{(simData?.fuel_qty??0).toFixed(1)} <small class="text-[10px] font-medium text-[var(--color-dim)]">gal</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">ENDURANCE</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{(simData?.fuel_flow>0)?((simData?.fuel_qty??0)/simData.fuel_flow*60).toFixed(0):'--'} <small class="text-[10px] font-medium text-[var(--color-dim)]">min</small></span></div>
          </div>
        </div>
      </div>

    {:else if navTab === 'perf'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">DENSITY ALTITUDE</div>
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Field Elevation</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={fieldElev} min="0" max="14000" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">ft</span></div></div>
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Outside Air Temp</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={oat} min="-40" max="50" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">°C</span></div></div>
            <div class="flex items-center justify-between gap-2.5"><label class="text-xs text-[var(--color-fg)]">Altimeter Setting</label><div class="flex"><input type="number" onwheel={onWheel} class="w-18 h-7 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[13px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={altSetting} min="28" max="31" step="0.01" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">inHg</span></div></div>
          </div>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">RESULT</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">PRESSURE ALT</span><span class="font-mono text-2xl font-extrabold text-[var(--color-bright)] leading-tight">{pressAlt} <small class="text-[10px] font-medium text-[var(--color-dim)]">ft</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">DENSITY ALT</span><span class="font-mono text-2xl font-extrabold leading-tight {densAlt > 5000 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{densAlt} <small class="text-[10px] font-medium text-[var(--color-dim)]">ft</small></span></div>
          </div>
          {#if densAlt > 5000}<div class="mt-2 px-2.5 py-1.5 rounded bg-[var(--color-yellow)]/5 border border-[var(--color-yellow)]/15 text-[var(--color-yellow)] text-[11px]">High density altitude — reduced performance</div>{/if}
          {#if densAlt > 8000}<div class="mt-2 px-2.5 py-1.5 rounded bg-[var(--color-yellow)]/5 border border-[var(--color-yellow)]/15 text-[var(--color-red)] text-[11px]">Very high DA — consider weight reduction</div>{/if}
          {#if navData?.performance}
            <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)] mt-3">AIRCRAFT DATA — {navData.name || ''}</div>
            <table class="w-full border-collapse text-[13px]">
              <tbody>
                {#if navData.performance.takeoff_ground_roll_ft}<tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Takeoff ground roll</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{navData.performance.takeoff_ground_roll_ft} ft</td></tr>{/if}
                {#if navData.performance.takeoff_over_50ft}<tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Takeoff over 50ft</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{navData.performance.takeoff_over_50ft} ft</td></tr>{/if}
                {#if navData.performance.landing_ground_roll_ft}<tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Landing ground roll</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{navData.performance.landing_ground_roll_ft} ft</td></tr>{/if}
                {#if navData.performance.rate_of_climb_fpm}<tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Rate of climb</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{navData.performance.rate_of_climb_fpm} fpm</td></tr>{/if}
                {#if navData.performance.service_ceiling_ft}<tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Service ceiling</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{navData.performance.service_ceiling_ft} ft</td></tr>{/if}
              </tbody>
            </table>
            <div class="mt-2.5 text-[10px] text-[var(--color-dim)] italic">Sea level ISA values. Actual performance varies with density altitude.</div>
          {/if}
        </div>
      </div>

    {:else if navTab === 'ref'}
      {#if navData}
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div>
            <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">V-SPEEDS — {navData.name || navActivePreset}</div>
            <table class="w-full border-collapse text-[13px]">
              <thead><tr><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">SPEED</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">VALUE</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">DESCRIPTION</th></tr></thead>
              <tbody>
                {#each Object.entries(navData.vspeeds || {}) as [name, info]}
                  <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 text-[var(--color-accent)] font-bold font-mono">{name}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono font-bold text-[var(--color-accent)] text-[13px]">{info.value} {info.unit || ''}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 text-[var(--color-dim)] text-[11px]">{info.desc || ''}</td></tr>
                {/each}
              </tbody>
            </table>
          </div>
          <div>
            <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">PERFORMANCE</div>
            <table class="w-full border-collapse text-[13px]">
              <tbody>
                {#each Object.entries(navData.performance || {}) as [key, val]}
                  <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">{key.replace(/_/g, ' ')}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{val}</td></tr>
                {/each}
              </tbody>
            </table>
            {#if navData.engine}
              <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)] mt-3">ENGINE</div>
              <table class="w-full border-collapse text-[13px]">
                <tbody>
                  {#each Object.entries(navData.engine || {}) as [key, val]}
                    <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">{key.replace(/_/g, ' ')}</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{val}</td></tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        </div>
      {:else}
        <div class="w-full h-full flex flex-col items-center justify-center gap-2 text-[var(--color-dim)] p-10"><p>Select an aircraft above to view reference data.</p></div>
      {/if}
    {/if}
  </div>
</div>
