<script lang="ts">
  import { onMount } from 'svelte';
  import Toast from './lib/gauges/Toast.svelte';
  import {
    Home, Gauge, ClipboardList, Target, BarChart3, Navigation,
    Cpu, Terminal, BookOpen, Users,
    PanelLeftClose, PanelLeft, Route, Smartphone, Crosshair, Package, ShoppingBag, Sun, Moon
  } from 'lucide-svelte';
  import CockpitBuilder from './lib/CockpitBuilder.svelte';
  import ModuleManager from './lib/ModuleManager.svelte';
  import HomeSection from './lib/sections/HomeSection.svelte';
  import ChecklistSection from './lib/sections/ChecklistSection.svelte';
  import ScenariosSection from './lib/sections/ScenariosSection.svelte';
  import DebriefSection from './lib/sections/DebriefSection.svelte';
  import NavSection from './lib/sections/NavSection.svelte';
  import HardwareSection from './lib/sections/HardwareSection.svelte';
  import CommunitySection from './lib/sections/CommunitySection.svelte';

  // Icon name → Lucide component mapping
  const iconMap: Record<string, any> = {
    home: Home, gauge: Gauge, clipboard: ClipboardList, target: Target,
    chart: BarChart3, navigation: Navigation, cpu: Cpu, terminal: Terminal,
    book: BookOpen, route: Route, users: Users, smartphone: Smartphone,
    crosshair: Crosshair, package: Package, store: ShoppingBag,
  };

  interface ModuleInfo {
    id: string;
    name: string;
    icon: string;
    enabled: boolean;
    order: number;
    desktop_only?: boolean;
    renderer?: string;
    default_preset?: string;
    presets?: string[];
  }

  // Fallback modules if disk scan fails
  let diskModules = $state<ModuleInfo[]>([]);

  // Fallback modules if disk scan fails or is empty
  const fallbackModules = [
    { id: 'checklist', name: 'Checklist', icon: 'clipboard', enabled: true, order: 3 },
    { id: 'scenarios', name: 'Scenarios', icon: 'target', enabled: true, order: 4 },
    { id: 'debrief', name: 'Debrief', icon: 'chart', enabled: true, order: 5 },
    { id: 'nav', name: 'Nav', icon: 'navigation', enabled: true, order: 6 },
    { id: 'hardware', name: 'Hardware', icon: 'cpu', enabled: true, order: 7 },
    { id: 'community', name: 'Community', icon: 'users', enabled: true, order: 11 },
  ];

  // Combined sidebar: home + enabled modules (from disk or fallback) + module manager
  let activeModules = $derived(
    diskModules.length > 0 ? diskModules : fallbackModules as any[]
  );

  let sidebarItems = $derived([
    { id: 'home', label: 'Home', icon: Home, badge: '' },
    ...activeModules
      .filter(m => m.enabled)
      .sort((a, b) => (a.order || 99) - (b.order || 99))
      .map(m => ({ id: m.id, label: m.name, icon: iconMap[m.icon] || Package, badge: m.badge || '' })),
  ]);

  let active = $state('home');
  let collapsed = $state(false);

  // Theme toggle
  // svelte-ignore state_referenced_locally
  let isDark = $state(localStorage.getItem('cf-theme') !== 'light');
  function toggleTheme() {
    isDark = !isDark;
    const theme = isDark ? 'dark' : 'light';
    localStorage.setItem('cf-theme', theme);
    document.documentElement.dataset.theme = isDark ? '' : 'light';
  }
  // Apply on load
  $effect(() => {
    document.documentElement.dataset.theme = isDark ? '' : 'light';
  });

  // Update check

  let updateInfo = $state<{ update_available: boolean; current: string; latest: string; download_url: string; changelog: string } | null>(null);

  async function checkUpdates() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        updateInfo = await invoke('check_for_updates') as any;
      }
    } catch {}
  }

  // Load modules from disk via Tauri invoke
  async function loadDiskModules() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        diskModules = await invoke('list_modules') as ModuleInfo[];
      }
    } catch {
      diskModules = [];
    }
  }

  // G430 overlay buttons - positions in % relative to image
  // Panel overlay buttons (for future use)
  const g430Buttons = [
    // TOP LEFT: knobs COM (.C big + C small)
    { id: 'COM1_COARSE', label: '◀', x: 1, y: 2, w: 5.5, h: 16, style: 'knob' },
    { id: 'COM1_FINE', label: '▶', x: 7, y: 5, w: 4, h: 12, style: 'knob' },
    // COM flip (arrow between COM and VLOC)
    { id: 'COM1_FLIP', label: '⇅', x: 4, y: 20, w: 5, h: 6, style: 'btn-flip' },
    // BOTTOM LEFT: knobs VLOC (.V big + V small)
    { id: 'NAV1_COARSE', label: '◀', x: 1, y: 28, w: 5.5, h: 16, style: 'knob' },
    { id: 'NAV1_FINE', label: '▶', x: 7, y: 31, w: 4, h: 12, style: 'knob' },
    // VLOC label area
    { id: 'NAV1_FLIP', label: '⇅', x: 4, y: 46, w: 5, h: 6, style: 'btn-flip' },
    // BOTTOM LEFT: COM/VLOC toggle + PUSH C/V
    { id: 'GPS_CDI', label: 'COM\nVLOC', x: 3, y: 54, w: 8, h: 10, style: 'btn-flip' },
    { id: 'PUSH_CV', label: 'PUSH\nC/V', x: 1, y: 68, w: 9, h: 8, style: 'btn' },
    // BOTTOM ROW - left group
    { id: 'CDI_BTN', label: 'CDI', x: 14, y: 88, w: 8, h: 10, style: 'btn' },
    { id: 'GPS_OBS', label: 'OBS', x: 23, y: 88, w: 9, h: 10, style: 'btn' },
    { id: 'GPS_MSG', label: 'MSG', x: 33, y: 88, w: 9, h: 10, style: 'btn' },
    { id: 'GPS_FPL', label: 'FPL', x: 43, y: 88, w: 8, h: 10, style: 'btn' },
    { id: 'GPS_PROC', label: 'PROC', x: 52, y: 88, w: 9, h: 10, style: 'btn' },
    // BOTTOM ROW - center labels (VLOC, MSG, NAV on screen bottom)
    // These are screen elements, not buttons

    // TOP RIGHT: RNG knob
    { id: 'GPS_ZOOM_OUT', label: '−', x: 87, y: 2, w: 5, h: 12, style: 'knob' },
    { id: 'GPS_ZOOM_IN', label: '+', x: 93, y: 2, w: 5, h: 12, style: 'knob' },
    // RIGHT side buttons
    { id: 'GPS_DIRECT', label: 'D→', x: 87, y: 18, w: 11, h: 8, style: 'btn' },
    { id: 'GPS_MENU', label: 'MENU', x: 87, y: 28, w: 11, h: 8, style: 'btn' },
    { id: 'GPS_CLR', label: 'CLR', x: 87, y: 38, w: 11, h: 8, style: 'btn' },
    { id: 'GPS_ENT', label: 'ENT', x: 87, y: 48, w: 11, h: 8, style: 'btn-accent' },
    // DEFAULT NAV / GPS toggle
    { id: 'GPS_NAV', label: 'GPS', x: 88, y: 58, w: 10, h: 7, style: 'btn-flip' },
    // BOTTOM RIGHT: PUSH CRSR knob
    { id: 'GPS_OUTER', label: '◀', x: 86, y: 70, w: 6, h: 14, style: 'knob' },
    { id: 'GPS_INNER', label: '▶', x: 93, y: 73, w: 5, h: 10, style: 'knob' },
    { id: 'GPS_CURSOR', label: 'CRSR', x: 87, y: 86, w: 11, h: 8, style: 'btn' },
  ];
  let showOverlay = $state(true);
  let overlayOpacity = $state(0.6);
  let activeOverlay = $state('g430');
  let overlayOffsetX = $state(0);
  let overlayOffsetY = $state(0);
  let overlayScale = $state(100);

  // Overlay presets
  const electricalButtons = [
    // Far left: Magneto rotary (OFF R L BOTH START)
    { id: 'MAGNETO_OFF', label: 'OFF', x: 1, y: 65, w: 5, h: 18, style: 'btn' },
    { id: 'MAGNETO_R', label: 'R', x: 3, y: 20, w: 5, h: 14, style: 'btn' },
    { id: 'MAGNETO_L', label: 'L', x: 8, y: 10, w: 5, h: 14, style: 'btn' },
    { id: 'MAGNETO_BOTH', label: 'BOTH', x: 13, y: 10, w: 5, h: 14, style: 'btn' },
    { id: 'STARTER', label: 'START', x: 13, y: 55, w: 6, h: 18, style: 'btn-accent' },
    // Master switch (big red split: ALT + BAT)
    { id: 'MASTER_ALT', label: 'ALT', x: 20, y: 15, w: 5, h: 55, style: 'btn' },
    { id: 'MASTER_BATTERY', label: 'BAT', x: 25, y: 15, w: 5, h: 55, style: 'btn' },
    // Fuel pump toggle
    { id: 'FUEL_PUMP', label: 'FUEL\nPUMP', x: 31, y: 40, w: 5, h: 35, style: 'btn' },
    // Circuit breakers row (top)
    // FLAP 10, INST 5, AVN BUS1 15, AVN BUS2 15, TURN COORD 5, INST LTS 5
    // (not interactive usually, skip)
    // Lights row (bottom center): BCN, LAND, TAXI, NAV, STRB
    { id: 'BEACON', label: 'BCN', x: 38, y: 42, w: 5, h: 35, style: 'btn' },
    { id: 'LANDING_LIGHT', label: 'LAND', x: 44, y: 42, w: 5, h: 35, style: 'btn' },
    { id: 'TAXI_LIGHT', label: 'TAXI', x: 50, y: 42, w: 5, h: 35, style: 'btn' },
    { id: 'NAV_LIGHT', label: 'NAV', x: 56, y: 42, w: 5, h: 35, style: 'btn' },
    { id: 'STROBE', label: 'STRB', x: 62, y: 42, w: 5, h: 35, style: 'btn' },
    // Pitot heat
    { id: 'PITOT_HEAT', label: 'PITOT', x: 69, y: 42, w: 5, h: 35, style: 'btn' },
    // Avionics master (right area)
    { id: 'AVIONICS_MASTER', label: 'AV', x: 77, y: 15, w: 5, h: 55, style: 'btn' },
    // ALT FLD, WARN (breakers, right side)
    // BUS 1, BUS 2 (far right)
    { id: 'AVIONICS_MASTER_2', label: 'BUS1', x: 87, y: 15, w: 5, h: 55, style: 'btn' },
  ];

  const radioStackButtons = [
    { id: 'COM1_COARSE', label: '<<', x: 2, y: 5, w: 8, h: 12, style: 'knob' },
    { id: 'COM1_FINE', label: '>>', x: 90, y: 5, w: 8, h: 12, style: 'knob' },
    { id: 'COM1_FLIP', label: '<>', x: 45, y: 5, w: 10, h: 10, style: 'btn-flip' },
    { id: 'COM2_COARSE', label: '<<', x: 2, y: 25, w: 8, h: 12, style: 'knob' },
    { id: 'COM2_FINE', label: '>>', x: 90, y: 25, w: 8, h: 12, style: 'knob' },
    { id: 'COM2_FLIP', label: '<>', x: 45, y: 25, w: 10, h: 10, style: 'btn-flip' },
    { id: 'NAV1_COARSE', label: '<<', x: 2, y: 45, w: 8, h: 12, style: 'knob' },
    { id: 'NAV1_FINE', label: '>>', x: 90, y: 45, w: 8, h: 12, style: 'knob' },
    { id: 'NAV1_FLIP', label: '<>', x: 45, y: 45, w: 10, h: 10, style: 'btn-flip' },
    { id: 'XPDR_IDENT', label: 'IDENT', x: 40, y: 75, w: 12, h: 10, style: 'btn-accent' },
  ];

  const overlayPresets: Record<string, { label: string; buttons: typeof g430Buttons }> = {
    g430: { label: 'GNS 430', buttons: g430Buttons },
    electrical: { label: 'Electrical Panel', buttons: electricalButtons },
    radio: { label: 'Radio Stack', buttons: radioStackButtons },
    none: { label: 'No Overlay', buttons: [] },
  };

  function getActiveButtons() {
    return overlayPresets[activeOverlay]?.buttons || [];
  }

  // Sync overlay to backend for LAN
  $effect(() => {
    const btns = getActiveButtons();
    if ('__TAURI_INTERNALS__' in window && btns.length > 0) {
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke('store_overlay', { data: JSON.stringify(btns) }).catch(() => {});
      });
    }
  });

  // Persistent landing history
  function loadAllLandings(): { rate: number; speed: number; time: string; date: string }[] {
    try { return JSON.parse(localStorage.getItem('sf-landings') || '[]'); } catch { return []; }
  }
  let allLandings = $state(loadAllLandings());
  $effect(() => { try { localStorage.setItem('sf-landings', JSON.stringify(allLandings)); } catch {} });

  // Save touchdown to persistent history
  function saveTouchdown(td: any) {
    allLandings = [...allLandings, td];
  }
  let lanIp = $state('...');

  let connected = $state(false);
  let simData: any = $state({});
  let arduinoStatus = $state('disconnected');
  let arduinoLastPin = $state('');
  let flightActive = $state(false);
  let landingRate = $state(0);
  let maxG = $state(1.0);
  let flightStartTime = $state(0);
  let logEntries: any[] = $state([]);
  let currentFlight: any = $state(null);
  let utcTime = $state('--:--:--');

  // Debrief tracking
  let maxAlt = $state(0);
  let maxSpd = $state(0);
  let maxBank = $state(0);
  let maxVs = $state(0);
  let minVs = $state(0);
  let fuelUsed = $state(0);
  let fuelStart = $state(0);
  let overspeedEvents = $state(0);
  let stallWarnings = $state(0);
  let touchdowns: { rate: number; speed: number; time: string }[] = $state([]);
  let lastOnGround = $state(true);

  // Grading helpers
  function gradeFromRate(r: number) { const a = Math.abs(r); return a < 60 ? 'A+' : a < 120 ? 'A' : a < 200 ? 'B' : a < 300 ? 'C' : a < 500 ? 'D' : 'F'; }
  function gradeColor(g: string) { return g === 'A+' || g === 'A' ? 'var(--color-green)' : g === 'B' ? 'var(--color-accent)' : g === 'C' ? 'var(--color-yellow)' : 'var(--color-red)'; }

  function resetDebrief() {
    maxAlt = 0; maxSpd = 0; maxBank = 0; maxVs = 0; minVs = 0;
    fuelUsed = 0; fuelStart = 0; overspeedEvents = 0; stallWarnings = 0;
    touchdowns = []; landingRate = 0; maxG = 1.0;
  }


  // Failure system (used by ScenariosSection)
  // Loaded from /data/failures.json at startup
  let failures: { id: string; label: string; group: string; desc: string; severity: string[] }[] = $state([]);
  let failureGroups: string[] = $state([]);



  onMount(() => {
    loadDiskModules();
    checkUpdates();
    fetch('./data/failures.json').then(r => r.json()).then(d => { failures = d.failures; failureGroups = d.groups; }).catch(() => {});
    const tick = () => { utcTime = new Date().toISOString().slice(11, 19); }; tick();
    const ci = setInterval(tick, 1000);
    const poll = setInterval(async () => {
      try {
        if (!('__TAURI_INTERNALS__' in window)) return;
        const { invoke } = await import('@tauri-apps/api/core');
        const d = await invoke('get_gauge_data') as any;
        simData = d; connected = d.connected;
        const rpm = d?.rpm ?? 0, gs = d?.groundspeed ?? 0;
        const ias = d?.airspeed ?? 0, alt = d?.altitude ?? 0, vs = d?.vs ?? 0;
        const bank = Math.abs(d?.bank ?? 0), gforce = d?.g_force ?? 1.0;
        const onGround = (d?.on_ground ?? 0) > 0.5;

        if (rpm > 500 && !flightActive) {
          flightActive = true; flightStartTime = Date.now();
          currentFlight = { start: new Date().toISOString(), maxAlt: 0 };
          resetDebrief();
          fuelStart = d?.fuel_qty ?? 0;
        }
        if (flightActive && rpm < 100 && gs < 5) {
          flightActive = false;
          if (currentFlight) { currentFlight.end = new Date().toISOString(); currentFlight.duration = Math.round((Date.now() - flightStartTime) / 60000); currentFlight.maxAlt = maxAlt; logEntries = [...logEntries, currentFlight]; currentFlight = null; }
        }
        if (flightActive) {
          if (alt > maxAlt) maxAlt = alt;
          if (ias > maxSpd) maxSpd = ias;
          if (bank > maxBank) maxBank = bank;
          if (vs > maxVs) maxVs = vs;
          if (vs < minVs) minVs = vs;
          if (gforce > maxG) maxG = gforce;
          fuelUsed = Math.max(0, fuelStart - (d?.fuel_qty ?? fuelStart));
          if (ias > 163) overspeedEvents++;
          if (ias > 0 && ias < 45) stallWarnings++;
          // Touchdown detection
          if (lastOnGround === false && onGround === true && Math.abs(vs) > 10) {
            const td = {
              rate: vs,
              speed: ias,
              gforce: gforce,
              heading: d?.heading ?? 0,
              altitude: alt,
              bank: bank,
              wind: d?.wind_speed ?? 0,
              time: new Date().toISOString().slice(11, 19),
              date: new Date().toISOString().slice(0, 10),
              aircraft: 'unknown',
            };
            touchdowns = [...touchdowns, td];
            saveTouchdown(td);
            landingRate = vs;
          }
          lastOnGround = onGround;
        }
      } catch {}
    }, 100); // 10Hz - smooth enough for gauges
    if ('__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/event').then(({ listen }) => {
        listen('arduino-status', (e: any) => { arduinoStatus = e.payload; });
        listen('arduino-pin', (e: any) => { arduinoLastPin = e.payload; });
      });
    }
    // Detect LAN IP from Tauri or WebRTC
    if ('__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke('get_local_ip').then((ip: any) => { if (ip) lanIp = ip; }).catch(() => {
          detectIpWebRTC();
        });
      });
    } else {
      detectIpWebRTC();
    }
    function detectIpWebRTC() {
      try {
        const pc = new RTCPeerConnection({ iceServers: [] });
        pc.createDataChannel('');
        pc.createOffer().then(o => pc.setLocalDescription(o));
        pc.onicecandidate = (e) => {
          if (!e.candidate) return;
          const m = e.candidate.candidate.match(/(\d+\.\d+\.\d+\.\d+)/);
          if (m && !m[1].startsWith('127.')) { lanIp = m[1]; pc.close(); }
        };
        setTimeout(() => pc.close(), 3000);
      } catch { lanIp = '127.0.0.1'; }
    }

    return () => { clearInterval(poll); clearInterval(ci); };
  });

  async function sendCmd(cmd: string, val: number) {
    try { if ('__TAURI_INTERNALS__' in window) { const { invoke } = await import('@tauri-apps/api/core'); await invoke('send_command', { cmd, value: val }); } } catch {}
  }

  function onWheel(e: WheelEvent) {
    const input = e.currentTarget as HTMLInputElement;
    if (document.activeElement !== input) return;
    e.preventDefault();
    const step = parseFloat(input.step) || 1;
    const delta = e.deltaY < 0 ? step : -step;
    const min = parseFloat(input.min);
    const max = parseFloat(input.max);
    let val = parseFloat(input.value) + delta;
    if (!isNaN(min)) val = Math.max(min, val);
    if (!isNaN(max)) val = Math.min(max, val);
    input.value = String(Math.round(val * 100) / 100);
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }
</script>

<Toast />


<div class="flex w-full h-full overflow-hidden">
  <!-- SIDEBAR -->
  <nav class="flex flex-col bg-[var(--color-surface)] border-r border-[var(--color-border)] shrink-0 overflow-hidden transition-[width] duration-150 {collapsed ? 'w-12' : 'w-[200px]'}">
    {#if !collapsed}
      <div class="flex items-center justify-center px-3 py-3 shrink-0">
        <span class="font-mono text-sm font-black tracking-[3px] text-[var(--color-accent)] whitespace-nowrap">COCKPITFLOW</span>
      </div>
    {/if}
    <div class="flex-1 flex flex-col gap-0.5 p-2 overflow-y-auto overflow-x-hidden">
      {#each sidebarItems as m}
        <button
          class="flex items-center gap-2.5 px-2.5 py-2 border-none bg-transparent text-[var(--color-dim)] rounded-md cursor-pointer text-xs whitespace-nowrap transition-all hover:text-[var(--color-fg)] hover:bg-[var(--color-surface-2)] {active === m.id ? 'text-[var(--color-accent)]! bg-[var(--color-accent)]/8!' : ''}"
          onclick={() => active = m.id}
          title={collapsed ? m.label : ''}
        >
          <m.icon size={16} strokeWidth={1.8} />
          {#if !collapsed}<span>{m.label}</span>{#if m.badge}<span class="ml-auto text-[7px] font-bold tracking-wide px-1.5 py-0.5 rounded bg-[var(--color-accent)]/10 text-[var(--color-accent)] border border-[var(--color-accent)]/20">{m.badge}</span>{/if}{/if}
        </button>
      {/each}
    </div>
    <div class="p-1.5 border-t border-[var(--color-border)] shrink-0">
      <button
        class="flex items-center gap-2 px-2 py-1.5 w-full border-none bg-transparent text-[var(--color-dim)] rounded cursor-pointer text-xs whitespace-nowrap transition-all hover:text-[var(--color-fg)] hover:bg-[var(--color-surface-2)] {active === '_modules' ? 'text-[var(--color-accent)]! bg-[var(--color-accent)]/8!' : ''}"
        onclick={() => active = '_modules'}
      >
        <Package size={16} strokeWidth={1.8} />{#if !collapsed}<span>Modules</span>{/if}
      </button>
      <button
        class="flex items-center gap-2 px-2 py-1.5 w-full border-none bg-transparent text-[var(--color-dim)] rounded cursor-pointer text-xs whitespace-nowrap transition-all hover:text-[var(--color-fg)] hover:bg-[var(--color-surface-2)]"
        onclick={toggleTheme}
        title={collapsed ? (isDark ? 'Light mode' : 'Dark mode') : ''}
      >
        {#if isDark}<Sun size={16} strokeWidth={1.8} />{:else}<Moon size={16} strokeWidth={1.8} />{/if}
        {#if !collapsed}<span>{isDark ? 'Light' : 'Dark'}</span>{/if}

      </button>
    </div>
  </nav>

  <!-- MAIN -->
  <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
    <header class="flex items-center justify-between px-3 h-8 bg-[var(--color-surface)] border-b border-[var(--color-border)] shrink-0">
      <div class="flex items-center gap-2">
        <button class="bg-transparent border-none text-[var(--color-dim)] cursor-pointer p-0.5 flex hover:text-[var(--color-fg)]" onclick={() => collapsed = !collapsed}>
          {#if collapsed}<PanelLeft size={15} />{:else}<PanelLeftClose size={15} />{/if}
        </button>
        <span class="text-xs font-semibold text-[var(--color-bright)]">{sidebarItems.find(m => m.id === active)?.label ?? ''}</span>
      </div>
      <div class="flex items-center gap-2">
        {#if flightActive}<span class="font-mono text-[9px] font-bold tracking-wide px-2 py-0.5 rounded bg-[var(--color-green)]/10 text-[var(--color-green)] border border-[var(--color-green)]/20">IN FLIGHT</span>{/if}
      </div>
    </header>

    <main class="flex-1 overflow-hidden p-2 flex">
      {#if active === 'home'}
        <HomeSection {connected} {arduinoStatus} {flightActive} {sidebarItems} {updateInfo} onNavigate={(id) => active = id} />

      {:else if active === 'cockpit'}
        <CockpitBuilder {lanIp} />


      {:else if active === 'checklist'}
        <ChecklistSection {lanIp} {connected} />

      {:else if active === 'scenarios'}
        <ScenariosSection {failures} {failureGroups} {sendCmd} {onWheel} />

      {:else if active === 'debrief'}
        <DebriefSection {flightActive} {landingRate} {maxG} {maxAlt} {maxSpd} {maxBank} {maxVs} {minVs} {fuelUsed} {overspeedEvents} {stallWarnings} {touchdowns} {logEntries} {flightStartTime} {gradeFromRate} {gradeColor} {resetDebrief} />

      {:else if active === 'nav'}
        <NavSection {onWheel} {simData} />

      {:else if active === 'hardware'}
        <HardwareSection {arduinoStatus} {arduinoLastPin} />


      {:else if active === 'community'}
        <CommunitySection {allLandings} {logEntries} {gradeFromRate} {gradeColor} onClearLandings={() => allLandings = []} />

      {:else if active === '_modules'}
        <ModuleManager {lanIp} onRefresh={loadDiskModules} />

      {/if}
    </main>

    <footer class="flex items-center justify-between px-3 h-[22px] bg-[var(--color-surface)] border-t border-[var(--color-border)] shrink-0 font-mono text-[9px] text-[var(--color-dim)]">
      <div class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full shrink-0 {connected ? 'bg-[var(--color-green)] shadow-[0_0_4px_var(--color-green)]' : 'bg-[var(--color-red)]'}"></span>
        <span>{connected ? 'ONLINE' : 'OFFLINE'}</span>
      </div>
      <span>{utcTime} UTC</span>
    </footer>
  </div>
</div>
