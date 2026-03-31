<script lang="ts">
  import { onMount } from 'svelte';
  import { toast } from '../gauges/Toast.svelte';
  import FirmwareFlasher from '../gauges/FirmwareFlasher.svelte';
  import LiveMonitor from '../gauges/LiveMonitor.svelte';
  import ProfileBrowser from '../gauges/ProfileBrowser.svelte';
  import FlowEditor from '../gauges/FlowEditor.svelte';

  interface Props {
    arduinoStatus: string;
    arduinoLastPin: string;
  }

  let { arduinoStatus, arduinoLastPin }: Props = $props();

  let hwTab = $state('map');
  let hwFilter = $state<'all'|'mapped'|'unmapped'>('all');
  let hwSearch = $state('');
  let hwGroup = $state('all');
  let hwHighlight = $state<string>(''); // pin or function id to highlight

  function hwScrollTo(id: string) {
    setTimeout(() => {
      const el = document.querySelector(`[data-hw-id="${id}"]`);
      el?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }, 50);
  }

  interface HwFunction { id: string; label: string; group: string; }

  let digitalFunctions = $state<HwFunction[]>([]);
  let analogFunctions = $state<HwFunction[]>([]);
  let digitalPins = $state<string[]>([]);
  let analogPins = $state<string[]>([]);

  onMount(async () => {
    try {
      const res = await fetch('/data/hardware-functions.json');
      const data = await res.json();
      digitalFunctions = data.digitalFunctions;
      analogFunctions = data.analogFunctions;
      digitalPins = data.digitalPins;
      analogPins = data.analogPins;
    } catch (e) {
      console.error('Failed to load hardware functions:', e);
    }
  });

  // Device management
  interface HwDevice { id: string; name: string; port: string; type: 'arduino' | 'esp32'; }
  function loadDevices(): HwDevice[] {
    try { const r = localStorage.getItem('sf-hw-devices'); return r ? JSON.parse(r) : [{ id: 'dev1', name: 'Arduino Nano', port: 'COM7', type: 'arduino' }]; } catch { return [{ id: 'dev1', name: 'Arduino Nano', port: 'COM7', type: 'arduino' }]; }
  }
  let hwDevices = $state<HwDevice[]>(loadDevices());
  $effect(() => { try { localStorage.setItem('sf-hw-devices', JSON.stringify(hwDevices)); } catch {} });

  // Pin-to-device mapping
  function loadPinDevices(): Record<string, string> {
    try { const r = localStorage.getItem('sf-hw-pin-dev'); return r ? JSON.parse(r) : {}; } catch { return {}; }
  }
  let pinDevice = $state<Record<string, string>>(loadPinDevices());
  $effect(() => { try { localStorage.setItem('sf-hw-pin-dev', JSON.stringify(pinDevice)); } catch {} });

  function getDeviceName(pin: string): string {
    const devId = pinDevice[pin];
    if (!devId) return hwDevices[0]?.name || '\u2014';
    return hwDevices.find(d => d.id === devId)?.name || '\u2014';
  }

  let addDevName = $state('');
  let addDevPort = $state('');
  let addDevType = $state<'arduino' | 'esp32'>('arduino');

  // Load saved mappings or defaults
  function loadMappings(): Record<string,string> {
    try { const r = localStorage.getItem('sf-hw-map'); return r ? JSON.parse(r) : {
      'D2':'MASTER_BATTERY','D3':'STARTER','D4':'FUEL_PUMP','D5':'LANDING_LIGHT',
      'D6':'TAXI_LIGHT','D7':'NAV_LIGHT','D8':'STROBE','D9':'PITOT_HEAT','D10':'AVIONICS_MASTER',
      'A0':'THROTTLE','A1':'MIXTURE','A2':'FLAPS'
    }; } catch { return {}; }
  }
  let hwMap = $state<Record<string,string>>(loadMappings());
  $effect(() => { try { localStorage.setItem('sf-hw-map', JSON.stringify(hwMap)); } catch {} });

  function getFunctionLabel(id: string): string {
    return digitalFunctions.find(f => f.id === id)?.label || analogFunctions.find(f => f.id === id)?.label || id || '\u2014';
  }
  function isUsed(fnId: string, currentPin: string): boolean {
    return Object.entries(hwMap).some(([pin, fn]) => fn === fnId && pin !== currentPin);
  }
  function usedOnPin(fnId: string, currentPin: string): string {
    const entry = Object.entries(hwMap).find(([pin, fn]) => fn === fnId && pin !== currentPin);
    return entry ? entry[0] : '';
  }
</script>

<div class="w-full h-full overflow-hidden flex flex-col bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md">
  <div class="flex border-b border-[var(--color-border)] shrink-0">
    {#each [['map','Mapping'],['info','Devices'],['flash','Firmware'],['monitor','Monitor'],['profiles','Profiles'],['flows','Flows']] as [id,label]}
      <button class="px-4 py-1.5 border-none bg-transparent text-[var(--color-dim)] font-mono text-[10px] font-semibold tracking-wide cursor-pointer border-b-2 border-transparent transition-all hover:text-[var(--color-fg)] {hwTab===id ? 'text-[var(--color-accent)]! border-b-[var(--color-accent)]!' : ''}" onclick={() => hwTab=id}>{label}</button>
    {/each}
    <div class="flex-1"></div>
    <div class="flex items-center gap-1.5 px-3 font-mono text-[9px] tracking-wide {arduinoStatus==='connected' ? 'text-[var(--color-green)]' : 'text-[var(--color-dim)]'}">
      <span class="w-1.5 h-1.5 rounded-full shrink-0 {arduinoStatus==='connected' ? 'bg-[var(--color-green)] shadow-[0_0_4px_var(--color-green)]' : 'bg-[var(--color-red)]'}"></span>
      {arduinoStatus === 'connected' ? 'HW ONLINE' : 'NO HW'}
    </div>
  </div>
  <div class="flex-1 overflow-y-auto p-3.5">
    {#if hwTab === 'map'}
      {@const usedIds = Object.values(hwMap).filter(Boolean)}
      {@const filterFn = (f: {id:string,label:string,group:string}) => {
        const pin = Object.entries(hwMap).find(([,v]) => v === f.id)?.[0];
        if (hwFilter === 'mapped' && !pin) return false;
        if (hwFilter === 'unmapped' && pin) return false;
        if (hwSearch && !f.label.toLowerCase().includes(hwSearch.toLowerCase())) return false;
        if (hwGroup !== 'all' && f.group !== hwGroup) return false;
        return true;
      }}
      <div class="flex flex-col lg:flex-row gap-2.5 h-full overflow-hidden">
        <!-- LEFT: Pin assignments per device -->
        <div class="lg:w-[340px] shrink-0 overflow-y-auto">
          {#each hwDevices as dev, di}
            <div class="flex items-center gap-2 px-2 py-1.5 bg-[var(--color-bg)] border border-[var(--color-border)] rounded mb-2">
              <span class="w-2 h-2 rounded-full shrink-0 transition-all {dev.port === 'COM7' && arduinoStatus === 'connected' ? 'bg-[var(--color-green)] border-[var(--color-green)] shadow-[0_0_6px_var(--color-green)]' : 'bg-[#2a3040] border border-[#3a4555]'}"></span>
              <span class="font-mono text-[11px] font-bold text-[var(--color-fg)]">{dev.name}</span>
              <span class="font-mono text-[8px] font-bold tracking-wide px-1.5 py-0.5 rounded bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-dim)]">{dev.type.toUpperCase()}</span>
              <span class="font-mono text-[10px] text-[var(--color-dim)] ml-auto">{dev.port}</span>
            </div>
            <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">DIGITAL</div>
            {#each digitalPins as pin}
              {#if hwDevices.length <= 1 || pinDevice[pin] === dev.id || (!pinDevice[pin] && di === 0)}
                {@const mapped = !!hwMap[pin]}
                {@const isActive = arduinoLastPin?.includes(pin)}
                <div class="flex items-center gap-2 py-1 px-0.5 border-b border-[var(--color-border)]/30 cursor-pointer transition-all hover:bg-[var(--color-bg)]/50 {mapped ? 'bg-[var(--color-bg)]/30' : ''} {isActive ? 'bg-[var(--color-green)]/10' : ''} {hwHighlight === pin || (hwMap[pin] && hwHighlight === hwMap[pin]) ? 'bg-[var(--color-accent)]/10 border-l-2 border-l-[var(--color-accent)]' : ''}" data-hw-id={pin}
                     onmouseenter={() => { hwHighlight = hwMap[pin] || pin; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}
                     onmouseleave={() => hwHighlight = ''}
                     onclick={() => { const id = hwMap[pin] || pin; hwHighlight = hwHighlight === id ? '' : id; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}>
                  <span class="w-2 h-2 rounded-full shrink-0 transition-all {isActive ? 'bg-[var(--color-green)] border-[var(--color-green)] shadow-[0_0_6px_var(--color-green)]' : mapped ? 'bg-[var(--color-yellow)] border-[var(--color-yellow)]' : 'bg-[#2a3040] border border-[#3a4555]'}"></span>
                  <span class="font-mono text-[11px] font-bold text-[var(--color-accent)] w-7 shrink-0">{pin}</span>
                  <select class="flex-1 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-1 rounded text-[11px] cursor-pointer focus:outline-none focus:border-[var(--color-accent)]" bind:value={hwMap[pin]} onchange={() => hwMap={...hwMap}} onclick={(e: Event) => e.stopPropagation()}>
                    <option value="">— None —</option>
                    {#each ['Electrical','Lights','Engine','Fuel','Controls','Autopilot','Avionics','Audio','Transponder','Environment','Misc'] as g}{@const items=digitalFunctions.filter(f=>f.group===g)}{#if items.length}<optgroup label={g}>{#each items as f}{@const onPin=usedOnPin(f.id,pin)}<option value={f.id}>{onPin ? '● ' : '○ '}{f.label}{onPin ? ` [${onPin}]` : ''}</option>{/each}</optgroup>{/if}{/each}
                  </select>
                  {#if isActive}<span class="font-mono text-[9px] font-extrabold text-[var(--color-green)] tracking-wide shrink-0">ON</span>{/if}
                </div>
              {/if}
            {/each}
            <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)] mt-1.5">ANALOG</div>
            {#each analogPins as pin}
              {#if hwDevices.length <= 1 || pinDevice[pin] === dev.id || (!pinDevice[pin] && di === 0)}
                {@const mapped = !!hwMap[pin]}
                <div class="flex items-center gap-2 py-1 px-0.5 border-b border-[var(--color-border)]/30 cursor-pointer transition-all hover:bg-[var(--color-bg)]/50 {mapped ? 'bg-[var(--color-bg)]/30' : ''} {hwHighlight === pin || (hwMap[pin] && hwHighlight === hwMap[pin]) ? 'bg-[var(--color-accent)]/10 border-l-2 border-l-[var(--color-accent)]' : ''}" data-hw-id={pin}
                     onmouseenter={() => { hwHighlight = hwMap[pin] || pin; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}
                     onmouseleave={() => hwHighlight = ''}
                     onclick={() => { const id = hwMap[pin] || pin; hwHighlight = hwHighlight === id ? '' : id; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}>
                  <span class="w-2 h-2 rounded-full shrink-0 transition-all {mapped ? 'bg-[var(--color-yellow)] border-[var(--color-yellow)]' : 'bg-[#2a3040] border border-[#3a4555]'}"></span>
                  <span class="font-mono text-[11px] font-bold text-[var(--color-accent)] w-7 shrink-0">{pin}</span>
                  <select class="flex-1 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-1 rounded text-[11px] cursor-pointer focus:outline-none focus:border-[var(--color-accent)]" bind:value={hwMap[pin]} onchange={() => hwMap={...hwMap}} onclick={(e: Event) => e.stopPropagation()}>
                    <option value="">— None —</option>
                    {#each ['Engine','Fuel','Controls','Avionics','Audio','Transponder','Lights','Environment'] as g}{@const items=analogFunctions.filter(f=>f.group===g)}{#if items.length}<optgroup label={g}>{#each items as f}{@const onPin=usedOnPin(f.id,pin)}<option value={f.id}>{onPin ? '● ' : '○ '}{f.label}{onPin ? ` [${onPin}]` : ''}</option>{/each}</optgroup>{/if}{/each}
                  </select>
                </div>
              {/if}
            {/each}
          {/each}
        </div>
        <!-- RIGHT: All functions with filters -->
        <div class="flex-1 min-w-0 flex flex-col gap-1.5 overflow-hidden">
          <div class="flex items-center gap-2 shrink-0">
            <input class="flex-1 h-[26px] bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-2 rounded text-[11px] focus:outline-none focus:border-[var(--color-accent)]" type="text" placeholder="Search..." bind:value={hwSearch} />
            <div class="flex items-center gap-0.5">
              <button class="px-3 py-1 border-none font-mono text-[10px] font-bold cursor-pointer transition-all {hwFilter==='all' ? 'bg-[var(--color-accent)] text-white' : 'bg-transparent text-[var(--color-dim)]'}" onclick={() => hwFilter='all'}>ALL</button>
              <button class="px-3 py-1 border-none font-mono text-[10px] font-bold cursor-pointer transition-all {hwFilter==='mapped' ? 'bg-[var(--color-accent)] text-white' : 'bg-transparent text-[var(--color-dim)]'}" onclick={() => hwFilter='mapped'}>IN USE</button>
              <button class="px-3 py-1 border-none font-mono text-[10px] font-bold cursor-pointer transition-all {hwFilter==='unmapped' ? 'bg-[var(--color-accent)] text-white' : 'bg-transparent text-[var(--color-dim)]'}" onclick={() => hwFilter='unmapped'}>FREE</button>
            </div>
          </div>
          <div class="flex flex-wrap gap-1 shrink-0">
            {#each ['all','Electrical','Lights','Engine','Fuel','Controls','Autopilot','Avionics','Audio','Transponder','Environment'] as g}
              <button class="px-2.5 py-0.5 border border-[var(--color-border)] rounded-full bg-transparent text-[var(--color-dim)] text-[10px] cursor-pointer transition-all hover:border-[var(--color-accent)] hover:text-[var(--color-fg)] {hwGroup===g ? 'bg-[var(--color-accent)] border-[var(--color-accent)] text-white' : ''}" onclick={() => hwGroup=g}>{g === 'all' ? 'All' : g}</button>
            {/each}
          </div>
          <div class="flex-1 overflow-y-auto">
            {#each ['Electrical','Lights','Engine','Fuel','Controls','Autopilot','Avionics','Audio','Transponder','Environment','Misc'] as group}
              {@const items = [...digitalFunctions, ...analogFunctions].filter(f => f.group === group).filter(filterFn)}
              {#if items.length}
                <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">{group}</div>
                {#each items as f}
                  {@const pin = Object.entries(hwMap).find(([,v]) => v === f.id)?.[0]}
                  <div class="flex items-center gap-2 py-1 px-0.5 border-b border-[var(--color-border)]/30 cursor-pointer transition-all hover:bg-[var(--color-bg)]/50 {!!pin ? 'bg-[var(--color-bg)]/30' : ''} {hwHighlight === f.id || (pin && hwHighlight === pin) ? 'bg-[var(--color-accent)]/10 border-l-2 border-l-[var(--color-accent)]' : ''}" data-hw-id={f.id}
                      onmouseenter={() => { hwHighlight = pin || f.id; if (pin) hwScrollTo(pin); }}
                      onmouseleave={() => hwHighlight = ''}
                      onclick={() => { hwHighlight = hwHighlight === f.id ? '' : (pin || f.id); if (pin) hwScrollTo(pin); }}>
                    <span class="w-2 h-2 rounded-full shrink-0 transition-all {!!pin ? 'bg-[var(--color-green)] border-[var(--color-green)] shadow-[0_0_6px_var(--color-green)]' : 'bg-[#2a3040] border border-[#3a4555]'}"></span>
                    <span class="text-[11px] text-[var(--color-fg)] flex-1">{f.label}</span>
                    {#if pin}<span class="font-mono text-[10px] font-bold text-[var(--color-accent)]">{pin}</span><span class="font-mono text-[9px] text-[var(--color-dim)]">{getDeviceName(pin)}</span>{:else}<span class="text-[var(--color-dim)] text-[10px]">—</span>{/if}
                  </div>
                {/each}
              {/if}
            {/each}
          </div>
          <div class="font-mono text-[9px] text-[var(--color-dim)] text-center py-1 shrink-0">{usedIds.length} mapped / {digitalFunctions.length + analogFunctions.length} total</div>
        </div>
      </div>

    {:else if hwTab === 'info'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">DEVICES</div>
          {#each hwDevices as dev, i}
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded p-2.5 mb-2">
              <div class="flex items-center gap-2 mb-2">
                <span class="w-2 h-2 rounded-full shrink-0 transition-all {dev.port === 'COM7' && arduinoStatus === 'connected' ? 'bg-[var(--color-green)] border-[var(--color-green)] shadow-[0_0_6px_var(--color-green)]' : 'bg-[#2a3040] border border-[#3a4555]'}"></span>
                <input class="flex-1 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-1 rounded text-[11px] font-mono font-bold focus:outline-none focus:border-[var(--color-accent)]" bind:value={hwDevices[i].name} onchange={() => hwDevices=[...hwDevices]} placeholder="Device name" />
                <span class="font-mono text-[8px] font-bold tracking-wide px-1.5 py-0.5 rounded bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-dim)]">{dev.type.toUpperCase()}</span>
                {#if hwDevices.length > 1}
                  <button class="w-5 h-5 flex items-center justify-center bg-transparent border border-[var(--color-border)] rounded text-[var(--color-dim)] text-[10px] cursor-pointer hover:border-[var(--color-red)] hover:text-[var(--color-red)]" onclick={() => { hwDevices = hwDevices.filter((_,j) => j !== i); toast('Device removed', 'info'); }}>x</button>
                {/if}
              </div>
              <div class="flex flex-col gap-1.5">
                <div class="flex items-center gap-2 text-[11px]"><span class="text-[var(--color-dim)] w-12">Port</span><input class="flex-1 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-1 rounded text-[11px] font-mono focus:outline-none focus:border-[var(--color-accent)]" bind:value={hwDevices[i].port} onchange={() => hwDevices=[...hwDevices]} placeholder="COM7" /></div>
                <div class="flex items-center gap-2 text-[11px]"><span class="text-[var(--color-dim)] w-12">Type</span>
                  <select class="flex-1 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-1 rounded text-[11px] cursor-pointer focus:outline-none focus:border-[var(--color-accent)]" bind:value={hwDevices[i].type} onchange={() => hwDevices=[...hwDevices]}>
                    <option value="arduino">Arduino</option>
                    <option value="esp32">ESP32</option>
                  </select>
                </div>
                <div class="flex items-center gap-2 text-[11px]"><span class="text-[var(--color-dim)] w-12">Pins used</span><span class="font-mono text-[var(--color-bright)]">{Object.entries(pinDevice).filter(([,d]) => d === dev.id).length + (i === 0 ? Object.entries(pinDevice).filter(([,d]) => !d).length : 0)}</span></div>
              </div>
            </div>
          {/each}
          <button class="w-full py-1.5 bg-transparent border border-dashed border-[var(--color-border)] rounded text-[var(--color-dim)] text-[11px] cursor-pointer transition-all hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]" onclick={() => { hwDevices = [...hwDevices, { id: `dev${Date.now()}`, name: `Device ${hwDevices.length+1}`, port: 'COM8', type: 'arduino' }]; toast('Device added', 'success'); }}>+ Add Device</button>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">PROTOCOL</div>
          <table class="w-full border-collapse text-[11px]"><tbody>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Baud Rate</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">115200</td></tr>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Frame</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">6B (HDR DEV PIN VL VH CK)</td></tr>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Digital</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">0xAA</td></tr>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Analog</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">0xCC</td></tr>
          </tbody></table>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)] mt-3.5">SUMMARY</div>
          <table class="w-full border-collapse text-[11px]"><tbody>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Devices</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{hwDevices.length}</td></tr>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Digital Mapped</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{digitalPins.filter(p=>hwMap[p]).length}/{digitalPins.length}</td></tr>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Analog Mapped</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{analogPins.filter(p=>hwMap[p]).length}/{analogPins.length}</td></tr>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Total Functions</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{Object.values(hwMap).filter(Boolean).length}</td></tr>
            <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Last Event</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{arduinoLastPin||'\u2014'}</td></tr>
          </tbody></table>
        </div>
      </div>
    {:else if hwTab === 'flash'}
      <FirmwareFlasher />
    {:else if hwTab === 'monitor'}
      <LiveMonitor />
    {:else if hwTab === 'profiles'}
      <ProfileBrowser />
    {:else if hwTab === 'flows'}
      <FlowEditor />
    {/if}
  </div>
</div>
