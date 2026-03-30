<script lang="ts">
  import { onMount } from 'svelte';
  import ElectricalPanel from './lib/gauges/ElectricalPanel.svelte';
  import LowerControls from './lib/gauges/LowerControls.svelte';
  import ChecklistView from './lib/gauges/Checklist.svelte';
  import FirmwareFlasher from './lib/gauges/FirmwareFlasher.svelte';
  import DeviceManager from './lib/gauges/DeviceManager.svelte';
  import DragDropMapper from './lib/gauges/DragDropMapper.svelte';
  import ProfileBrowser from './lib/gauges/ProfileBrowser.svelte';
  import ProfileEditor from './lib/gauges/ProfileEditor.svelte';
  import FlowEditor from './lib/gauges/FlowEditor.svelte';
  import LiveMonitor from './lib/gauges/LiveMonitor.svelte';
  import AircraftSilhouette from './lib/gauges/AircraftSilhouette.svelte';
  import Toast from './lib/gauges/Toast.svelte';
  import Marketplace from './lib/Marketplace.svelte';
  import {
    Home, Gauge, ClipboardList, Target, BarChart3, Navigation,
    Cpu, Terminal, BookOpen, Settings, Users, ExternalLink, Menu,
    PanelLeftClose, PanelLeft, Route, Smartphone, Crosshair, Package, ShoppingBag
  } from 'lucide-svelte';
  import logo from './assets/logo.png';
  import CockpitBuilder from './lib/CockpitBuilder.svelte';
  import ModuleManager from './lib/ModuleManager.svelte';

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

  // Static home + settings (always present, not modules)
  let diskModules = $state<ModuleInfo[]>([]);

  // Fallback modules if disk scan fails or is empty
  const fallbackModules = [
    { id: 'panel', name: 'Panel', icon: 'gauge', enabled: true, order: 1 },
    { id: 'checklist', name: 'Checklist', icon: 'clipboard', enabled: true, order: 3 },
    { id: 'scenarios', name: 'Scenarios', icon: 'target', enabled: true, order: 4 },
    { id: 'debrief', name: 'Debrief', icon: 'chart', enabled: true, order: 5 },
    { id: 'nav', name: 'Nav', icon: 'navigation', enabled: true, order: 6 },
    { id: 'hardware', name: 'Hardware', icon: 'cpu', enabled: true, order: 7 },
    { id: 'console', name: 'Console', icon: 'terminal', enabled: true, order: 8 },
    { id: 'logbook', name: 'Logbook', icon: 'book', enabled: true, order: 9 },
    { id: 'flight', name: 'Flight', icon: 'route', enabled: true, order: 10 },
    { id: 'community', name: 'Community', icon: 'users', enabled: true, order: 11 },
    { id: 'marketplace', name: 'Marketplace', icon: 'store', enabled: true, order: 12 },
  ];

  // Combined sidebar: home + enabled modules (from disk or fallback) + module manager
  let activeModules = $derived(
    diskModules.length > 0 ? diskModules : fallbackModules as any[]
  );

  let sidebarItems = $derived([
    { id: 'home', label: 'Home', icon: Home },
    ...activeModules
      .filter(m => m.enabled)
      .sort((a, b) => (a.order || 99) - (b.order || 99))
      .map(m => ({ id: m.id, label: m.name, icon: iconMap[m.icon] || Package })),
  ]);

  let active = $state('home');
  let collapsed = $state(false);
  let panelTab = $state('gauges');

  // Update check
  let updateInfo = $state<{ update_available: boolean; current: string; latest: string; download_url: string; changelog: string } | null>(null);
  let updateDismissed = $state(false);

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

  let checklistMode = $state<'app' | 'lan' | 'settings'>('app');
  let communityTab = $state('leaderboard');
  let pilotName = $state(localStorage.getItem('sf-pilot') || 'Pilot');
  let pilotCallsign = $state(localStorage.getItem('sf-callsign') || 'N172SP');
  $effect(() => { localStorage.setItem('sf-pilot', pilotName); });
  $effect(() => { localStorage.setItem('sf-callsign', pilotCallsign); });

  // Persistent landing history
  function loadAllLandings(): { rate: number; speed: number; time: string; date: string }[] {
    try { return JSON.parse(localStorage.getItem('sf-landings') || '[]'); } catch { return []; }
  }
  let allLandings = $state(loadAllLandings());
  $effect(() => { try { localStorage.setItem('sf-landings', JSON.stringify(allLandings)); } catch {} });

  // Save touchdown to persistent history
  function saveTouchdown(td: { rate: number; speed: number; time: string }) {
    allLandings = [...allLandings, { ...td, date: new Date().toISOString().slice(0, 10) }];
  }
  let lanIp = $state('...');
  let lanCopied = $state(false);
  let clMode = $state<'strict' | 'smart'>('smart');
  let clAutoCheck = $state<'on' | 'off'>('on');
  let clFeedback = $state<'on' | 'off'>('on');

  async function syncSettings() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('set_settings', { newSettings: {
          mode: clMode,
          auto_check: clAutoCheck,
          check_feedback: clFeedback,
          data_mode: 'live',
          web_theme: 'dark',
          active_checklist: clActivePreset,
        }});
      }
    } catch {}
  }
  let clPresets = $state<string[]>([]);
  let clActivePreset = $state('cessna-172');

  async function loadChecklistPresets() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        const mods = await invoke('list_modules') as any[];
        const cl = mods.find(m => m.id === 'checklist');
        if (cl) {
          clPresets = cl.presets || [];
          clActivePreset = cl.default_preset || 'cessna-172';
        }
      }
    } catch {}
  }

  async function setChecklistPreset(preset: string) {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_module', { moduleId: 'checklist', field: 'default_preset', value: preset });
        clActivePreset = preset;
      }
    } catch {}
  }

  async function importChecklistPreset() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        JSON.parse(text);
        const name = file.name.replace('.json', '').toLowerCase().replace(/[^a-z0-9-]/g, '-');
        if ('__TAURI_INTERNALS__' in window) {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('save_module_preset', { moduleId: 'checklist', presetId: name, data: text });
          loadChecklistPresets();
        }
      } catch {}
    };
    input.click();
  }

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
  let debriefTab = $state('summary');
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
  function landingVerdict(r: number): string { const a = Math.abs(r); return a < 30 ? 'GREASER' : a < 60 ? 'BUTTER' : a < 120 ? 'SMOOTH' : a < 200 ? 'NORMAL' : a < 300 ? 'FIRM' : a < 500 ? 'HARD' : 'CRASHED'; }
  function arcPath(cx: number, cy: number, r: number, startAngle: number, endAngle: number): string {
    const s = startAngle * Math.PI / 180, e = endAngle * Math.PI / 180;
    const x1 = cx + r * Math.cos(s), y1 = cy + r * Math.sin(s);
    const x2 = cx + r * Math.cos(e), y2 = cy + r * Math.sin(e);
    const large = endAngle - startAngle > 180 ? 1 : 0;
    return `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2}`;
  }
  function needlePos(rate: number): number { return Math.min(180, Math.max(0, Math.abs(rate) / 500 * 180)); }
  function flightDuration() { return flightActive ? Math.round((Date.now() - flightStartTime) / 60000) : (logEntries.length > 0 ? logEntries[logEntries.length - 1].duration : 0); }

  function resetDebrief() {
    maxAlt = 0; maxSpd = 0; maxBank = 0; maxVs = 0; minVs = 0;
    fuelUsed = 0; fuelStart = 0; overspeedEvents = 0; stallWarnings = 0;
    touchdowns = []; landingRate = 0; maxG = 1.0;
  }

  let paxWeight = $state(170);
  let baggageWeight = $state(30);
  let fuelGallons = $state(40);
  const EW = 1680, ECG = 40.5, FA = 48, PA = 37, BA = 95;
  let tw = $derived(EW + paxWeight + baggageWeight + fuelGallons * 6);
  let cg = $derived((EW * ECG + paxWeight * PA + baggageWeight * BA + fuelGallons * 6 * FA) / tw);
  let ok = $derived(tw <= 2550 && cg >= 35 && cg <= 47.3);
  let navTab = $state('wb');

  // Crosswind calculator
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
  let tripTime = $derived(tripDist / cruiseSpeed * 60);
  let tripFuel = $derived(tripDist / cruiseSpeed * fuelBurn);
  let reserveFuel = $derived(reserveMin / 60 * fuelBurn);
  let totalFuelReq = $derived(tripFuel + reserveFuel);

  // V-speeds C172S
  const vspeeds = [
    ['Vr', '55', 'Rotation'],
    ['Vx', '62', 'Best angle climb'],
    ['Vy', '74', 'Best rate climb'],
    ['Va', '99', 'Maneuvering (max wt)'],
    ['Vfe', '85/110', 'Flaps extended'],
    ['Vno', '129', 'Max structural cruise'],
    ['Vne', '163', 'Never exceed'],
    ['Vso', '40', 'Stall (landing config)'],
    ['Vs1', '48', 'Stall (clean)'],
    ['Vglide', '65', 'Best glide'],
  ];

  let scenarioTab = $state('presets');

  const scenarios = [
    { name: 'Engine Failure 500ft', diff: 'HARD', desc: 'Pitch for 65 KIAS glide.' },
    { name: 'Crosswind Landing', diff: 'MED', desc: '15kt crosswind approach.' },
    { name: 'Electrical Failure', diff: 'MED', desc: 'Alternator + battery fail.' },
    { name: 'Stall Recovery', diff: 'EASY', desc: 'Power-off stall recovery.' },
    { name: 'Go-Around', diff: 'EASY', desc: 'Missed approach from 200ft.' },
    { name: 'Night VFR', diff: 'HARD', desc: 'Limited visual references.' },
    { name: 'Partial Panel', diff: 'HARD', desc: 'Vacuum pump failure.' },
    { name: 'Short Field', diff: 'MED', desc: 'Vref 61 KIAS short runway.' },
  ];

  // Failure system
  const failures = [
    { id: 'eng_fail', label: 'Engine Failure', group: 'Engine', desc: 'Complete power loss', severity: ['partial', 'full'] },
    { id: 'eng_rough', label: 'Engine Roughness', group: 'Engine', desc: 'Vibration, power fluctuation', severity: ['mild', 'severe'] },
    { id: 'mag_l', label: 'Left Magneto Fail', group: 'Engine', desc: 'RPM drop, roughness', severity: ['full'] },
    { id: 'mag_r', label: 'Right Magneto Fail', group: 'Engine', desc: 'RPM drop, roughness', severity: ['full'] },
    { id: 'carb_ice', label: 'Carburetor Icing', group: 'Engine', desc: 'Gradual power loss', severity: ['mild', 'severe'] },
    { id: 'oil_press', label: 'Oil Pressure Loss', group: 'Engine', desc: 'Low/no oil pressure', severity: ['partial', 'full'] },
    { id: 'fuel_starv', label: 'Fuel Starvation', group: 'Fuel', desc: 'Empty tank or blockage', severity: ['full'] },
    { id: 'fuel_leak', label: 'Fuel Leak', group: 'Fuel', desc: 'Gradual fuel loss', severity: ['slow', 'fast'] },
    { id: 'alt_fail', label: 'Alternator Failure', group: 'Electrical', desc: 'No charging, battery only', severity: ['full'] },
    { id: 'bat_fail', label: 'Battery Failure', group: 'Electrical', desc: 'No electrical power', severity: ['full'] },
    { id: 'bus_fail', label: 'Bus Failure', group: 'Electrical', desc: 'Partial electrical loss', severity: ['partial', 'full'] },
    { id: 'vac_pump', label: 'Vacuum Pump Fail', group: 'Instruments', desc: 'Attitude + heading lost', severity: ['full'] },
    { id: 'pitot_block', label: 'Pitot Blockage', group: 'Instruments', desc: 'Airspeed unreliable', severity: ['partial', 'full'] },
    { id: 'static_block', label: 'Static Port Block', group: 'Instruments', desc: 'Alt/VS/ASI affected', severity: ['full'] },
    { id: 'ai_fail', label: 'Attitude Indicator', group: 'Instruments', desc: 'AI tumbles/fails', severity: ['full'] },
    { id: 'hi_fail', label: 'Heading Indicator', group: 'Instruments', desc: 'HI precesses/fails', severity: ['full'] },
    { id: 'asi_fail', label: 'Airspeed Indicator', group: 'Instruments', desc: 'ASI stuck/wrong', severity: ['full'] },
    { id: 'alt_fail_inst', label: 'Altimeter Failure', group: 'Instruments', desc: 'Altimeter reads wrong', severity: ['full'] },
    { id: 'comm_fail', label: 'Comm Radio Fail', group: 'Avionics', desc: 'No radio communication', severity: ['com1', 'com2', 'all'] },
    { id: 'nav_fail', label: 'Nav Radio Fail', group: 'Avionics', desc: 'VOR/ILS unavailable', severity: ['nav1', 'nav2', 'all'] },
    { id: 'xpdr_fail', label: 'Transponder Fail', group: 'Avionics', desc: 'No transponder reply', severity: ['full'] },
    { id: 'gps_fail', label: 'GPS Failure', group: 'Avionics', desc: 'GPS signal lost', severity: ['full'] },
    { id: 'flap_fail', label: 'Flap Failure', group: 'Controls', desc: 'Flaps stuck/jammed', severity: ['up', 'down', 'asym'] },
    { id: 'trim_run', label: 'Trim Runaway', group: 'Controls', desc: 'Trim moves on its own', severity: ['nose_up', 'nose_down'] },
    { id: 'brake_fail', label: 'Brake Failure', group: 'Controls', desc: 'No braking on landing', severity: ['partial', 'full'] },
    { id: 'door_open', label: 'Door Pops Open', group: 'Misc', desc: 'Cabin door opens in flight', severity: ['full'] },
    { id: 'bird_strike', label: 'Bird Strike', group: 'Misc', desc: 'Windshield crack/engine', severity: ['windshield', 'engine'] },
    { id: 'fire_eng', label: 'Engine Fire', group: 'Misc', desc: 'Fire warning, smoke', severity: ['full'] },
  ];

  const failureGroups = ['Engine', 'Fuel', 'Electrical', 'Instruments', 'Avionics', 'Controls', 'Misc'];

  interface ActiveFailure {
    id: string;
    label: string;
    severity: string;
    trigger: 'immediate' | 'delay' | 'random';
    delayMin: number;
    delayMax: number;
    armed: boolean;
    fired: boolean;
    fireTime: number;
  }

  let activeFailures = $state<ActiveFailure[]>([]);
  let failureTrigger = $state<'immediate' | 'delay' | 'random'>('random');
  let failureDelayMin = $state(5);
  let failureDelayMax = $state(30);
  let randomCount = $state(1);

  function armFailure(f: typeof failures[0], sev: string) {
    const af: ActiveFailure = {
      id: f.id, label: f.label, severity: sev,
      trigger: failureTrigger,
      delayMin: failureDelayMin, delayMax: failureDelayMax,
      armed: true, fired: false,
      fireTime: failureTrigger === 'immediate' ? Date.now() :
        failureTrigger === 'delay' ? Date.now() + failureDelayMin * 60000 :
        Date.now() + (failureDelayMin + Math.random() * (failureDelayMax - failureDelayMin)) * 60000,
    };
    activeFailures = [...activeFailures, af];
  }

  function armRandom() {
    const pool = failures.filter(f => !activeFailures.some(a => a.id === f.id));
    for (let i = 0; i < Math.min(randomCount, pool.length); i++) {
      const idx = Math.floor(Math.random() * pool.length);
      const f = pool.splice(idx, 1)[0];
      const sev = f.severity[Math.floor(Math.random() * f.severity.length)];
      armFailure(f, sev);
    }
  }

  function clearFailures() { activeFailures = []; }

  // Flight module
  let flightTab = $state('plan');

  interface Waypoint {
    id: string;
    name: string;
    alt: number;
    freq: string;
    notes: string;
    failureId: string;
  }

  let flightPlan = $state<{ dep: string; arr: string; cruise: number; waypoints: Waypoint[] }>({
    dep: '', arr: '', cruise: 5500,
    waypoints: [],
  });

  function addWaypoint() {
    flightPlan.waypoints = [...flightPlan.waypoints, {
      id: 'wp' + Date.now(), name: '', alt: flightPlan.cruise, freq: '', notes: '', failureId: ''
    }];
  }
  function removeWaypoint(i: number) {
    flightPlan.waypoints = flightPlan.waypoints.filter((_, j) => j !== i);
  }

  // Flight events (failures during flight)
  interface FlightEvent {
    id: string;
    failureId: string;
    trigger: 'time' | 'altitude' | 'phase' | 'random';
    timeMin: number;        // minutes after takeoff
    altFt: number;          // trigger at altitude
    phase: string;          // trigger at phase
    randomMin: number;      // random range min
    randomMax: number;      // random range max
    fired: boolean;
  }

  let flightEvents = $state<FlightEvent[]>([]);

  function addFlightEvent() {
    flightEvents = [...flightEvents, {
      id: 'ev' + Date.now(), failureId: '', trigger: 'random',
      timeMin: 10, altFt: 3000, phase: 'CRUISE',
      randomMin: 5, randomMax: 30, fired: false,
    }];
  }
  function removeFlightEvent(i: number) {
    flightEvents = flightEvents.filter((_, j) => j !== i);
  }

  // Live flight tracking
  function getFlightPhase(): string {
    if (!connected) return 'PREFLIGHT';
    const gs = simData?.groundspeed ?? 0;
    const alt = simData?.altitude ?? 0;
    const vs = simData?.vs ?? 0;
    const rpm = simData?.rpm ?? 0;
    if (rpm < 100) return 'SHUTDOWN';
    if (gs < 5 && alt < 100) return 'GROUND';
    if (gs > 5 && gs < 30 && alt < 100) return 'TAXI';
    if (vs > 300 && alt < flightPlan.cruise * 0.9) return 'TAKEOFF/CLIMB';
    if (Math.abs(vs) < 200 && alt > flightPlan.cruise * 0.8) return 'CRUISE';
    if (vs < -200) return 'DESCENT';
    if (gs > 30 && alt < 1500 && vs < -100) return 'APPROACH';
    return 'EN ROUTE';
  }
  let flightPhase = $derived(getFlightPhase());

  // Altitude profile recording
  let altProfile = $state<{t: number; alt: number; gs: number}[]>([]);
  let profileInterval: ReturnType<typeof setInterval> | null = null;

  function startRecording() {
    altProfile = [];
    if (profileInterval) clearInterval(profileInterval);
    profileInterval = setInterval(() => {
      if (connected) {
        altProfile = [...altProfile, {
          t: Date.now(),
          alt: simData?.altitude ?? 0,
          gs: simData?.groundspeed ?? 0
        }];
        // Keep last 60min (1 sample/sec = 3600)
        if (altProfile.length > 3600) altProfile = altProfile.slice(-3600);
      }
    }, 1000);
  }
  function stopRecording() {
    if (profileInterval) { clearInterval(profileInterval); profileInterval = null; }
  }
  // Hardware mapping
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

  const digitalFunctions = [
    // Electrical
    { id: 'MASTER_BATTERY', label: 'Master Battery', group: 'Electrical' },
    { id: 'MASTER_ALT', label: 'Master Alternator', group: 'Electrical' },
    { id: 'AVIONICS_MASTER', label: 'Avionics Bus 1', group: 'Electrical' },
    { id: 'AVIONICS_MASTER_2', label: 'Avionics Bus 2', group: 'Electrical' },
    { id: 'FUEL_PUMP', label: 'Fuel Pump', group: 'Electrical' },
    { id: 'PITOT_HEAT', label: 'Pitot Heat', group: 'Electrical' },
    { id: 'STALL_HEAT', label: 'Stall Warning Heat', group: 'Electrical' },
    // Lights
    { id: 'BEACON', label: 'Beacon', group: 'Lights' },
    { id: 'LANDING_LIGHT', label: 'Landing Light', group: 'Lights' },
    { id: 'TAXI_LIGHT', label: 'Taxi Light', group: 'Lights' },
    { id: 'NAV_LIGHT', label: 'Nav Lights', group: 'Lights' },
    { id: 'STROBE', label: 'Strobe', group: 'Lights' },
    { id: 'INSTRUMENT_LIGHTS', label: 'Instrument Lights', group: 'Lights' },
    { id: 'DOME_LIGHT', label: 'Dome / Cabin Light', group: 'Lights' },
    // Engine
    { id: 'MAGNETO_OFF', label: 'Magneto OFF', group: 'Engine' },
    { id: 'MAGNETO_R', label: 'Magneto RIGHT', group: 'Engine' },
    { id: 'MAGNETO_L', label: 'Magneto LEFT', group: 'Engine' },
    { id: 'MAGNETO_BOTH', label: 'Magneto BOTH', group: 'Engine' },
    { id: 'STARTER', label: 'Starter Engage', group: 'Engine' },
    { id: 'PRIMER', label: 'Primer Pump', group: 'Engine' },
    { id: 'ALT_AIR', label: 'Alternate Air', group: 'Engine' },
    // Fuel
    { id: 'FUEL_SEL_OFF', label: 'Fuel Selector OFF', group: 'Fuel' },
    { id: 'FUEL_SEL_LEFT', label: 'Fuel Selector LEFT', group: 'Fuel' },
    { id: 'FUEL_SEL_RIGHT', label: 'Fuel Selector RIGHT', group: 'Fuel' },
    { id: 'FUEL_SEL_BOTH', label: 'Fuel Selector BOTH', group: 'Fuel' },
    { id: 'FUEL_SHUTOFF', label: 'Fuel Shutoff Valve', group: 'Fuel' },
    // Flight Controls
    { id: 'FLAPS_UP', label: 'Flaps UP', group: 'Controls' },
    { id: 'FLAPS_10', label: 'Flaps 10°', group: 'Controls' },
    { id: 'FLAPS_20', label: 'Flaps 20°', group: 'Controls' },
    { id: 'FLAPS_FULL', label: 'Flaps FULL', group: 'Controls' },
    { id: 'FLAPS_INC', label: 'Flaps Increment', group: 'Controls' },
    { id: 'FLAPS_DEC', label: 'Flaps Decrement', group: 'Controls' },
    { id: 'TRIM_UP', label: 'Trim Nose Up', group: 'Controls' },
    { id: 'TRIM_DOWN', label: 'Trim Nose Down', group: 'Controls' },
    { id: 'PARKING_BRAKE', label: 'Parking Brake', group: 'Controls' },
    { id: 'TOE_BRAKE_L', label: 'Left Toe Brake', group: 'Controls' },
    { id: 'TOE_BRAKE_R', label: 'Right Toe Brake', group: 'Controls' },
    // Autopilot
    { id: 'AP_MASTER', label: 'AP Master', group: 'Autopilot' },
    { id: 'AP_HDG', label: 'AP Heading', group: 'Autopilot' },
    { id: 'AP_NAV', label: 'AP Nav', group: 'Autopilot' },
    { id: 'AP_APR', label: 'AP Approach', group: 'Autopilot' },
    { id: 'AP_REV', label: 'AP Reverse', group: 'Autopilot' },
    { id: 'AP_ALT', label: 'AP Altitude Hold', group: 'Autopilot' },
    { id: 'AP_VS', label: 'AP Vertical Speed', group: 'Autopilot' },
    { id: 'AP_FD', label: 'Flight Director', group: 'Autopilot' },
    { id: 'AP_DISC', label: 'AP Disconnect', group: 'Autopilot' },
    // Avionics
    { id: 'PTT', label: 'Push to Talk', group: 'Avionics' },
    { id: 'COM1_FLIP', label: 'COM1 Flip', group: 'Avionics' },
    { id: 'COM2_FLIP', label: 'COM2 Flip', group: 'Avionics' },
    { id: 'NAV1_FLIP', label: 'NAV1 Flip', group: 'Avionics' },
    { id: 'NAV2_FLIP', label: 'NAV2 Flip', group: 'Avionics' },
    { id: 'ADF_FLIP', label: 'ADF Flip', group: 'Avionics' },
    { id: 'XPDR_IDENT', label: 'Transponder IDENT', group: 'Avionics' },
    { id: 'DME_TOGGLE', label: 'DME Toggle', group: 'Avionics' },
    { id: 'GPS_CDI', label: 'GPS/VLOC Toggle', group: 'Avionics' },
    { id: 'GPS_OBS', label: 'GPS OBS Mode', group: 'Avionics' },
    { id: 'GPS_MSG', label: 'GPS MSG', group: 'Avionics' },
    { id: 'GPS_FPL', label: 'GPS FPL', group: 'Avionics' },
    { id: 'GPS_VNAV', label: 'GPS VNAV', group: 'Avionics' },
    { id: 'GPS_PROC', label: 'GPS PROC', group: 'Avionics' },
    { id: 'GPS_ENT', label: 'GPS ENT', group: 'Avionics' },
    { id: 'GPS_CLR', label: 'GPS CLR', group: 'Avionics' },
    { id: 'GPS_DIRECT', label: 'GPS Direct-To', group: 'Avionics' },
    { id: 'GPS_MENU', label: 'GPS MENU', group: 'Avionics' },
    { id: 'GPS_ZOOM_IN', label: 'GPS Zoom In', group: 'Avionics' },
    { id: 'GPS_ZOOM_OUT', label: 'GPS Zoom Out', group: 'Avionics' },
    { id: 'GPS_CURSOR', label: 'GPS Cursor', group: 'Avionics' },
    // Environment
    { id: 'CABIN_HEAT', label: 'Cabin Heat', group: 'Environment' },
    { id: 'CABIN_AIR', label: 'Cabin Air Vent', group: 'Environment' },
    { id: 'DEFROST', label: 'Windshield Defrost', group: 'Environment' },
    // Audio Panel
    { id: 'AUDIO_COM1', label: 'Audio COM1 Select', group: 'Audio' },
    { id: 'AUDIO_COM2', label: 'Audio COM2 Select', group: 'Audio' },
    { id: 'AUDIO_NAV1', label: 'Audio NAV1 Select', group: 'Audio' },
    { id: 'AUDIO_NAV2', label: 'Audio NAV2 Select', group: 'Audio' },
    { id: 'AUDIO_DME', label: 'Audio DME Select', group: 'Audio' },
    { id: 'AUDIO_ADF', label: 'Audio ADF Select', group: 'Audio' },
    { id: 'AUDIO_MKR', label: 'Audio Marker Select', group: 'Audio' },
    { id: 'AUDIO_SPKR', label: 'Speaker Toggle', group: 'Audio' },
    { id: 'MKR_HILO', label: 'Marker Hi/Lo Sensitivity', group: 'Audio' },
    // Transponder
    { id: 'XPDR_OFF', label: 'Transponder OFF', group: 'Transponder' },
    { id: 'XPDR_STBY', label: 'Transponder STBY', group: 'Transponder' },
    { id: 'XPDR_ON', label: 'Transponder ON', group: 'Transponder' },
    { id: 'XPDR_ALT', label: 'Transponder ALT', group: 'Transponder' },
    { id: 'XPDR_TEST', label: 'Transponder TEST', group: 'Transponder' },
    // Annunciators & CB
    { id: 'ANNUN_TEST', label: 'Annunciator Test', group: 'Electrical' },
    { id: 'CB_RESET', label: 'Circuit Breaker Reset', group: 'Electrical' },
    { id: 'ALT_STATIC', label: 'Alternate Static Source', group: 'Electrical' },
    // Lights extra
    { id: 'MAP_LIGHT', label: 'Map Light', group: 'Lights' },
    { id: 'FLOOD_LIGHT', label: 'Flood Light', group: 'Lights' },
    // Controls extra
    { id: 'FLAP_OVERRIDE', label: 'Flap Detent Override', group: 'Controls' },
    // Environment extra
    { id: 'WINDOW_VENT', label: 'Window Vent', group: 'Environment' },
    { id: 'STORM_WINDOW', label: 'Storm Window', group: 'Environment' },
    // Views & misc
    { id: 'VIEW_FORWARD', label: 'View Forward', group: 'Misc' },
    { id: 'VIEW_LEFT', label: 'View Left', group: 'Misc' },
    { id: 'VIEW_RIGHT', label: 'View Right', group: 'Misc' },
    { id: 'VIEW_REAR', label: 'View Rear', group: 'Misc' },
    { id: 'VIEW_UP', label: 'View Up', group: 'Misc' },
    { id: 'VIEW_DOWN', label: 'View Down', group: 'Misc' },
    { id: 'VIEW_FREE', label: 'Free Look Toggle', group: 'Misc' },
    { id: 'PAUSE', label: 'Sim Pause', group: 'Misc' },
    { id: 'REPLAY', label: 'Instant Replay', group: 'Misc' },
    { id: 'SCREENSHOT', label: 'Screenshot', group: 'Misc' },
  ];

  const analogFunctions = [
    // Engine
    { id: 'THROTTLE', label: 'Throttle', group: 'Engine' },
    { id: 'MIXTURE', label: 'Mixture', group: 'Engine' },
    { id: 'CARB_HEAT', label: 'Carb Heat', group: 'Engine' },
    { id: 'PROP', label: 'Propeller RPM', group: 'Engine' },
    // Fuel
    { id: 'FUEL_SEL', label: 'Fuel Selector (multi-pos)', group: 'Fuel' },
    // Flight Controls
    { id: 'FLAPS', label: 'Flaps (analog)', group: 'Controls' },
    { id: 'TRIM', label: 'Elevator Trim', group: 'Controls' },
    { id: 'RUDDER_TRIM', label: 'Rudder Trim', group: 'Controls' },
    { id: 'AILERON_TRIM', label: 'Aileron Trim', group: 'Controls' },
    { id: 'BRAKE_LEFT', label: 'Left Brake Axis', group: 'Controls' },
    { id: 'BRAKE_RIGHT', label: 'Right Brake Axis', group: 'Controls' },
    { id: 'YOKE_PITCH', label: 'Yoke Pitch', group: 'Controls' },
    { id: 'YOKE_ROLL', label: 'Yoke Roll', group: 'Controls' },
    { id: 'RUDDER_PEDALS', label: 'Rudder Pedals', group: 'Controls' },
    // Avionics knobs
    { id: 'HDG_BUG', label: 'Heading Bug', group: 'Avionics' },
    { id: 'OBS1_KNOB', label: 'NAV1 OBS', group: 'Avionics' },
    { id: 'OBS2_KNOB', label: 'NAV2 OBS', group: 'Avionics' },
    { id: 'BARO_KNOB', label: 'Altimeter Baro', group: 'Avionics' },
    { id: 'COM1_COARSE', label: 'COM1 MHz', group: 'Avionics' },
    { id: 'COM1_FINE', label: 'COM1 kHz', group: 'Avionics' },
    { id: 'COM2_COARSE', label: 'COM2 MHz', group: 'Avionics' },
    { id: 'COM2_FINE', label: 'COM2 kHz', group: 'Avionics' },
    { id: 'NAV1_COARSE', label: 'NAV1 MHz', group: 'Avionics' },
    { id: 'NAV1_FINE', label: 'NAV1 kHz', group: 'Avionics' },
    { id: 'NAV2_COARSE', label: 'NAV2 MHz', group: 'Avionics' },
    { id: 'NAV2_FINE', label: 'NAV2 kHz', group: 'Avionics' },
    { id: 'ADF_COARSE', label: 'ADF Coarse', group: 'Avionics' },
    { id: 'ADF_FINE', label: 'ADF Fine', group: 'Avionics' },
    { id: 'XPDR_CODE', label: 'Transponder Code', group: 'Avionics' },
    { id: 'GPS_INNER', label: 'GPS Inner Knob', group: 'Avionics' },
    { id: 'GPS_OUTER', label: 'GPS Outer Knob', group: 'Avionics' },
    { id: 'AP_ALT_SEL', label: 'AP Altitude Select', group: 'Avionics' },
    { id: 'AP_VS_SEL', label: 'AP VS Select', group: 'Avionics' },
    { id: 'AP_HDG_SEL', label: 'AP Heading Select', group: 'Avionics' },
    // Lights
    { id: 'INSTRUMENT_DIM', label: 'Instrument Brightness', group: 'Lights' },
    { id: 'PANEL_DIM', label: 'Panel Brightness', group: 'Lights' },
    // Audio volumes
    { id: 'COM1_VOL', label: 'COM1 Volume', group: 'Audio' },
    { id: 'COM2_VOL', label: 'COM2 Volume', group: 'Audio' },
    { id: 'NAV1_VOL', label: 'NAV1 Volume', group: 'Audio' },
    { id: 'NAV2_VOL', label: 'NAV2 Volume', group: 'Audio' },
    { id: 'DME_VOL', label: 'DME Volume', group: 'Audio' },
    { id: 'ADF_VOL', label: 'ADF Volume', group: 'Audio' },
    { id: 'MKR_VOL', label: 'Marker Volume', group: 'Audio' },
    { id: 'INTERCOM_VOL', label: 'Intercom Volume', group: 'Audio' },
    // Transponder
    { id: 'XPDR_DIGIT_1', label: 'XPDR Digit 1', group: 'Transponder' },
    { id: 'XPDR_DIGIT_2', label: 'XPDR Digit 2', group: 'Transponder' },
    { id: 'XPDR_DIGIT_3', label: 'XPDR Digit 3', group: 'Transponder' },
    { id: 'XPDR_DIGIT_4', label: 'XPDR Digit 4', group: 'Transponder' },
    // Environment
    { id: 'CABIN_HEAT_KNOB', label: 'Cabin Heat Level', group: 'Environment' },
    { id: 'CABIN_AIR_KNOB', label: 'Cabin Air Level', group: 'Environment' },
    // Lights
    { id: 'MAP_LIGHT_DIM', label: 'Map Light Brightness', group: 'Lights' },
    { id: 'FLOOD_LIGHT_DIM', label: 'Flood Light Brightness', group: 'Lights' },
  ];

  const digitalPins = ['D2','D3','D4','D5','D6','D7','D8','D9','D10','D11','D12','D13'];
  const analogPins = ['A0','A1','A2','A3','A4','A5','A6','A7'];

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
    if (!devId) return hwDevices[0]?.name || '—';
    return hwDevices.find(d => d.id === devId)?.name || '—';
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
    return digitalFunctions.find(f => f.id === id)?.label || analogFunctions.find(f => f.id === id)?.label || id || '—';
  }
  function isUsed(fnId: string, currentPin: string): boolean {
    return Object.entries(hwMap).some(([pin, fn]) => fn === fnId && pin !== currentPin);
  }
  function usedOnPin(fnId: string, currentPin: string): string {
    const entry = Object.entries(hwMap).find(([pin, fn]) => fn === fnId && pin !== currentPin);
    return entry ? entry[0] : '';
  }

  onMount(() => {
    loadDiskModules();
    loadChecklistPresets();
    checkUpdates();
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
            const td = { rate: vs, speed: ias, time: new Date().toISOString().slice(11, 19) };
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

{#if updateInfo?.update_available && !updateDismissed}
  <div class="update-banner">
    <span>CockpitFlow <strong>v{updateInfo.latest}</strong> is available (you have v{updateInfo.current})</span>
    <a href={updateInfo.download_url} target="_blank" class="update-btn">Download</a>
    <button class="update-dismiss" onclick={() => updateDismissed = true}>&times;</button>
  </div>
{/if}

<div class="shell">
  <!-- SIDEBAR -->
  <nav class="sidebar" class:collapsed>
    <div class="sb-brand">
      <img src={logo} alt="CockpitFlow" class="sb-logo" />
    </div>
    <div class="sb-nav">
      {#each sidebarItems as m}
        <button class="sb-item" class:active={active === m.id} onclick={() => active = m.id} title={collapsed ? m.label : ''}>
          <svelte:component this={m.icon} size={16} strokeWidth={1.8} />
          {#if !collapsed}<span>{m.label}</span>{/if}
        </button>
      {/each}
    </div>
    <div class="sb-foot">
      <button class="sb-item" class:active={active === '_modules'} onclick={() => active = '_modules'}>
        <Package size={16} strokeWidth={1.8} />{#if !collapsed}<span>Modules</span>{/if}
      </button>
      <button class="sb-item" class:active={active === 'settings'} onclick={() => active = 'settings'}>
        <Settings size={16} strokeWidth={1.8} />{#if !collapsed}<span>Settings</span>{/if}
      </button>
      <div class="sb-status">
        <div class="sb-dot-row"><span class="dot" class:on={connected}></span>{#if !collapsed}<span>{connected ? 'SIM' : 'OFFLINE'}</span>{/if}</div>
        <div class="sb-dot-row"><span class="dot" class:on={arduinoStatus==='connected'}></span>{#if !collapsed}<span>{arduinoStatus==='connected' ? 'HW' : 'NO HW'}</span>{/if}</div>
      </div>
    </div>
  </nav>

  <!-- MAIN -->
  <div class="main">
    <header class="topbar">
      <div class="topbar-l">
        <button class="icon-btn" onclick={() => collapsed = !collapsed}>
          {#if collapsed}<PanelLeft size={15} />{:else}<PanelLeftClose size={15} />{/if}
        </button>
        <span class="topbar-title">{sidebarItems.find(m => m.id === active)?.label ?? 'Settings'}</span>
      </div>
      <div class="topbar-r">
        {#if flightActive}<span class="badge badge-green">IN FLIGHT</span>{/if}
        <span class="mono-sm">{utcTime} UTC</span>
      </div>
    </header>

    <main class="view">
      {#if active === 'home'}
        <div class="v-row">
          <div class="card" style="width:240px;flex-shrink:0">
            <div class="home-brand"><img src={logo} alt="" class="home-logo" /><div><div class="home-title">COCKPITFLOW</div><div class="home-sub">COCKPIT COMPANION</div></div></div>
            <div class="sep"></div>
            <div class="label">STATUS</div>
            <div class="status-list">
              <div class="status-row"><span class="dot" class:on={connected}></span><span>{connected ? 'Sim connected' : 'No simulator'}</span></div>
              <div class="status-row"><span class="dot" class:on={arduinoStatus==='connected'}></span><span>{arduinoStatus==='connected' ? 'Hardware online' : 'No hardware'}</span></div>
              <div class="status-row"><span class="dot" class:on={flightActive}></span><span>{flightActive ? 'In flight' : 'On ground'}</span></div>
            </div>
            <div class="sep"></div>
            <div class="label">LINKS</div>
            <a href="https://discord.gg/cockpitflow" target="_blank" class="link-btn discord">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M20.317 4.37a19.8 19.8 0 00-4.885-1.515.07.07 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.3 18.3 0 00-5.487 0 12.6 12.6 0 00-.617-1.25.08.08 0 00-.079-.037A19.7 19.7 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.08.08 0 00.031.057 19.9 19.9 0 005.993 3.03.08.08 0 00.084-.028 14.1 14.1 0 001.226-1.994.08.08 0 00-.041-.106 13.1 13.1 0 01-1.872-.892.08.08 0 01-.008-.128c.125-.094.25-.192.372-.292a.07.07 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.07.07 0 01.078.01c.12.1.246.198.373.292a.08.08 0 01-.006.127c-.598.35-1.22.645-1.873.892a.08.08 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.08.08 0 00.084.028 19.8 19.8 0 006.002-3.03.08.08 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.06.06 0 00-.031-.03z"/></svg>
              <span>Discord</span><ExternalLink size={11} />
            </a>
            <div class="spacer"></div>
            <div class="disclaimer">For flight simulation use only. Not for real-world aviation. Aircraft designations are used for reference only and are trademarks of their respective owners. Not affiliated with or endorsed by any aircraft manufacturer. Data provided is for educational purposes — verify all values against official documentation.</div>
            <div class="version">v0.1.0</div>
          </div>
          <div class="card flex-1">
            <div class="label">MODULES</div>
            <div class="mod-grid">
              {#each sidebarItems.filter(m => m.id !== 'home') as m}
                <button class="mod-btn" onclick={() => active = m.id}>
                  <svelte:component this={m.icon} size={18} strokeWidth={1.5} />
                  <span>{m.label}</span>
                </button>
              {/each}
            </div>
          </div>
        </div>

      {:else if active === 'panel'}
        <div class="v-row"><div class="flex-1 overflow-y-auto"><ElectricalPanel {simData} /></div><div class="flex-1 overflow-y-auto"><LowerControls {simData} /></div></div>

      {:else if active === 'checklist'}
        <div class="cl-settings" style="padding:14px;max-width:600px">
          <div class="efb-2col">
            <div class="efb-section">
              <div class="efb-heading">AIRCRAFT CHECKLIST</div>
              <div class="cl-preset-row">
                <select class="cl-preset-select" value={clActivePreset} onchange={(e) => setChecklistPreset((e.target as HTMLSelectElement).value)}>
                  {#each clPresets as p}
                    <option value={p}>{p}</option>
                  {/each}
                </select>
                <button class="btn-action" style="padding:6px 12px;font-size:10px" onclick={importChecklistPreset}>Import</button>
              </div>

              <div class="efb-heading" style="margin-top:16px">CHECKLIST MODE</div>
              <div class="cl-option-group">
                <button class="cl-opt" class:cl-opt-on={clMode==='strict'} onclick={() => { clMode='strict'; syncSettings(); }}>
                  <strong>Strict</strong>
                  <span>Items must be checked in order. Current item highlighted, rest locked.</span>
                </button>
                <button class="cl-opt" class:cl-opt-on={clMode==='smart'} onclick={() => { clMode='smart'; syncSettings(); }}>
                  <strong>Smart</strong>
                  <span>Check items in any order. More flexible for experienced pilots.</span>
                </button>
              </div>

              <div class="efb-heading" style="margin-top:16px">AUTO-DETECT</div>
              <div class="cl-toggle-row">
                <span>Auto Check</span>
                <div class="cl-toggle-group">
                  <button class="cl-tgl" class:cl-tgl-on={clAutoCheck==='on'} onclick={() => { clAutoCheck='on'; syncSettings(); }}>ON</button>
                  <button class="cl-tgl" class:cl-tgl-on={clAutoCheck==='off'} onclick={() => { clAutoCheck='off'; syncSettings(); }}>OFF</button>
                </div>
              </div>
              <p class="cl-hint">Automatically check items when sim detects correct state</p>

              <div class="cl-toggle-row">
                <span>Feedback Dots</span>
                <div class="cl-toggle-group">
                  <button class="cl-tgl" class:cl-tgl-on={clFeedback==='on'} onclick={() => { clFeedback='on'; syncSettings(); }}>ON</button>
                  <button class="cl-tgl" class:cl-tgl-on={clFeedback==='off'} onclick={() => { clFeedback='off'; syncSettings(); }}>OFF</button>
                </div>
              </div>
              <p class="cl-hint">Show green/red dots indicating sim variable state</p>
            </div>

            <div class="efb-section">
              <div class="efb-heading">INFO</div>
              <div class="cl-info-grid">
                <div class="cl-info-item"><span class="cl-info-label">ACTIVE</span><span>{clActivePreset}</span></div>
                <div class="cl-info-item"><span class="cl-info-label">PRESETS</span><span>{clPresets.length} available</span></div>
                <div class="cl-info-item"><span class="cl-info-label">AUTO-DETECT</span><span>{connected ? 'Active' : 'Waiting for sim'}</span></div>
                <div class="cl-info-item"><span class="cl-info-label">MODE</span><span>{clMode === 'strict' ? 'Strict (ordered)' : 'Smart (flexible)'}</span></div>
              </div>

              <div class="efb-heading" style="margin-top:16px">COMPANION APP</div>
              <p class="cl-hint" style="margin-bottom:6px">Use the CockpitFlow mobile app or open this URL to access the checklist on your phone/tablet.</p>
              <div class="lan-url-row">
                <span class="lan-url">http://{lanIp}:8080</span>
                <button class="lan-copy" onclick={() => { navigator.clipboard.writeText(`http://${lanIp}:8080`); lanCopied = true; setTimeout(() => lanCopied = false, 2000); }}>
                  {lanCopied ? 'COPIED' : 'COPY'}
                </button>
              </div>
            </div>
          </div>
        </div>

      {:else if active === 'scenarios'}
        <div class="efb-panel">
          <div class="efb-tabs">
            {#each [['presets','Presets'],['generator','Failure Generator'],['active','Active (' + activeFailures.length + ')']] as [id,label]}
              <button class="efb-tab" class:efb-tab-on={scenarioTab===id} onclick={() => scenarioTab=id}>{label}</button>
            {/each}
          </div>
          <div class="efb-body">
            {#if scenarioTab === 'presets'}
              <div class="grid-cards">{#each scenarios as s}
                <div class="card card-sm">
                  <div class="card-top"><span class="badge" class:badge-red={s.diff==='HARD'} class:badge-yellow={s.diff==='MED'} class:badge-green={s.diff==='EASY'}>{s.diff}</span></div>
                  <strong>{s.name}</strong><p class="desc">{s.desc}</p>
                  <button class="btn-accent" onclick={() => sendCmd('SCENARIO',0)}>START</button>
                </div>
              {/each}</div>

            {:else if scenarioTab === 'generator'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">TIMING</div>
                  <div class="gen-timing">
                    {#each [['immediate','Immediate','Triggers now'],['delay','Delayed','After set minutes'],['random','Random','Random time in range']] as [id,label,desc]}
                      <button class="cl-opt" class:cl-opt-on={failureTrigger===id} onclick={() => failureTrigger=id as any}>
                        <strong>{label}</strong><span>{desc}</span>
                      </button>
                    {/each}
                  </div>
                  {#if failureTrigger === 'delay'}
                    <div class="gen-delay-row">
                      <span>Delay</span>
                      <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={failureDelayMin} min="1" max="120" /><span class="input-unit">min</span></div>
                    </div>
                  {:else if failureTrigger === 'random'}
                    <div class="gen-delay-row">
                      <span>Between</span>
                      <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={failureDelayMin} min="1" max="120" /><span class="input-unit">min</span></div>
                      <span>and</span>
                      <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={failureDelayMax} min="1" max="120" /><span class="input-unit">min</span></div>
                    </div>
                  {/if}
                  <div class="efb-heading" style="margin-top:14px">RANDOM GENERATOR</div>
                  <div class="gen-delay-row">
                    <span>Count</span>
                    <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={randomCount} min="1" max="5" /><span class="input-unit">x</span></div>
                  </div>
                  <button class="btn-action" style="margin-top:8px;width:100%" onclick={armRandom}>ARM RANDOM FAILURES</button>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">PICK FAILURE</div>
                  <div class="gen-fail-list">
                    {#each failureGroups as group}
                      <div class="hw-group-label">{group}</div>
                      {#each failures.filter(f => f.group === group) as f}
                        {@const isArmed = activeFailures.some(a => a.id === f.id)}
                        <div class="gen-fail-row" class:gen-armed={isArmed}>
                          <span class="gen-fail-name">{f.label}</span>
                          <span class="gen-fail-desc">{f.desc}</span>
                          {#if isArmed}
                            <span class="gen-armed-tag">ARMED</span>
                          {:else}
                            <div class="gen-sev-btns">
                              {#each f.severity as sev}
                                <button class="gen-sev-btn" onclick={() => armFailure(f, sev)}>{sev}</button>
                              {/each}
                            </div>
                          {/if}
                        </div>
                      {/each}
                    {/each}
                  </div>
                </div>
              </div>

            {:else if scenarioTab === 'active'}
              {#if activeFailures.length === 0}
                <div class="efb-empty">No failures armed. Use the Generator tab.</div>
              {:else}
                <div class="efb-heading">ARMED FAILURES</div>
                <div class="af-list">
                  {#each activeFailures as af, i}
                    {@const remaining = Math.max(0, Math.round((af.fireTime - Date.now()) / 60000))}
                    {@const fired = Date.now() >= af.fireTime}
                    <div class="af-card" class:af-fired={fired}>
                      <span class="af-dot" class:hw-dot-active={fired} class:hw-dot-mapped={!fired}></span>
                      <div class="af-info">
                        <span class="af-name">{af.label}</span>
                        <span class="af-meta">{af.severity} | {af.trigger === 'immediate' ? 'Now' : af.trigger === 'delay' ? af.delayMin + 'm delay' : af.delayMin + '-' + af.delayMax + 'm random'}</span>
                      </div>
                      <span class="af-countdown" class:af-live={fired}>{fired ? 'ACTIVE' : 'T-' + remaining + 'm'}</span>
                      <button class="af-remove" onclick={() => activeFailures = activeFailures.filter((_,j) => j !== i)}>x</button>
                    </div>
                  {/each}
                </div>
                <button class="btn-reset" style="margin-top:12px" onclick={clearFailures}>CLEAR ALL</button>
              {/if}
            {/if}
          </div>
        </div>

      {:else if active === 'debrief'}
        <div class="efb-panel">
          <div class="efb-tabs">
            {#each [['summary','Summary'],['landing','Landing'],['envelope','Envelope'],['score','Score']] as [id,label]}
              <button class="efb-tab" class:efb-tab-on={debriefTab===id} onclick={() => debriefTab=id}>{label}</button>
            {/each}
            <div class="efb-tab-spacer"></div>
            <div class="efb-flight-indicator" class:efb-fi-active={flightActive}>
              <span class="dot" class:on={flightActive}></span>
              {flightActive ? 'RECORDING' : 'IDLE'}
            </div>
          </div>
          <div class="efb-body">
            {#if debriefTab === 'summary'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">FLIGHT DATA</div>
                  <table class="efb-tbl">
                    <tbody>
                      <tr><td>Status</td><td class="mono">{flightActive ? 'In Flight' : 'On Ground'}</td></tr>
                      <tr><td>Duration</td><td class="mono">{flightDuration()} min</td></tr>
                      <tr><td>Max Altitude</td><td class="mono">{maxAlt.toFixed(0)} ft</td></tr>
                      <tr><td>Max IAS</td><td class="mono" class:efb-ro-warn={maxSpd > 129}>{maxSpd.toFixed(0)} kt</td></tr>
                      <tr><td>Max Bank</td><td class="mono" class:efb-ro-warn={maxBank > 30}>{maxBank.toFixed(0)}°</td></tr>
                      <tr><td>Max VS Up</td><td class="mono">{maxVs.toFixed(0)} fpm</td></tr>
                      <tr><td>Max VS Down</td><td class="mono">{minVs.toFixed(0)} fpm</td></tr>
                      <tr><td>Fuel Used</td><td class="mono">{fuelUsed.toFixed(1)} gal</td></tr>
                    </tbody>
                  </table>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">KEY METRICS</div>
                  <div class="efb-readout-grid efb-rg-2x2">
                    <div class="efb-readout"><span class="efb-ro-label">LANDING RATE</span><span class="efb-ro-val" style="color:{gradeColor(gradeFromRate(landingRate))}">{landingRate.toFixed(0)} <small>fpm</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">MAX G-FORCE</span><span class="efb-ro-val" class:efb-ro-warn={maxG > 2.0}>{maxG.toFixed(2)} <small>G</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">OVERSPEED</span><span class="efb-ro-val" class:efb-ro-warn={overspeedEvents > 0}>{overspeedEvents}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">STALL WARN</span><span class="efb-ro-val" class:efb-ro-warn={stallWarnings > 0}>{stallWarnings}</span></div>
                  </div>
                </div>
              </div>

            {:else if debriefTab === 'landing'}
              {@const absRate = Math.abs(landingRate)}
              {@const grade = gradeFromRate(landingRate)}
              {@const color = gradeColor(grade)}
              {@const verdict = landingVerdict(landingRate)}
              {@const angle = needlePos(landingRate)}
              <div class="landing-layout">
                <!-- Left: big gauge -->
                <div class="landing-gauge-area">
                  <div class="landing-gauge-wrap">
                    <svg viewBox="0 0 200 120" class="landing-gauge-svg">
                      <!-- Arc background zones -->
                      <path d={arcPath(100, 100, 80, 180, 216)} stroke="#34d058" stroke-width="8" fill="none" stroke-linecap="round" opacity="0.3" />
                      <path d={arcPath(100, 100, 80, 216, 244)} stroke="#4a9eff" stroke-width="8" fill="none" opacity="0.3" />
                      <path d={arcPath(100, 100, 80, 244, 280)} stroke="#e3b341" stroke-width="8" fill="none" opacity="0.3" />
                      <path d={arcPath(100, 100, 80, 280, 324)} stroke="#e07020" stroke-width="8" fill="none" opacity="0.3" />
                      <path d={arcPath(100, 100, 80, 324, 360)} stroke="#ea4a5a" stroke-width="8" fill="none" stroke-linecap="round" opacity="0.3" />
                      <!-- Active arc up to needle -->
                      <path d={arcPath(100, 100, 80, 180, 180 + angle)} stroke={color} stroke-width="8" fill="none" stroke-linecap="round" />
                      <!-- Needle -->
                      <line x1="100" y1="100" x2={100 + 65 * Math.cos((180 + angle) * Math.PI / 180)} y2={100 + 65 * Math.sin((180 + angle) * Math.PI / 180)} stroke={color} stroke-width="2.5" stroke-linecap="round" />
                      <circle cx="100" cy="100" r="4" fill={color} />
                      <!-- Scale labels -->
                      <text x="18" y="105" fill="var(--color-dim)" font-size="7" font-family="monospace">0</text>
                      <text x="20" y="42" fill="var(--color-dim)" font-size="7" font-family="monospace">120</text>
                      <text x="95" y="18" fill="var(--color-dim)" font-size="7" font-family="monospace">250</text>
                      <text x="168" y="42" fill="var(--color-dim)" font-size="7" font-family="monospace">400</text>
                      <text x="174" y="105" fill="var(--color-dim)" font-size="7" font-family="monospace">500</text>
                    </svg>
                  </div>
                  <!-- Big readout below gauge -->
                  <div class="landing-readout">
                    <div class="landing-rate" style="color:{color}">{landingRate.toFixed(0)}</div>
                    <div class="landing-fpm">FPM</div>
                  </div>
                  <div class="landing-verdict" style="color:{color}">{verdict}</div>
                  <div class="landing-grade-big" style="color:{color}">{grade}</div>
                </div>

                <!-- Right: touchdown history -->
                <div class="landing-history">
                  <div class="efb-heading">TOUCHDOWNS</div>
                  {#if touchdowns.length === 0}
                    <div class="landing-empty">
                      <div class="landing-empty-icon">---</div>
                      <span>No touchdowns yet</span>
                      <span class="landing-empty-hint">Land in the simulator to record</span>
                    </div>
                  {:else}
                    <div class="landing-td-list">
                      {#each touchdowns.slice().reverse() as td, i}
                        {@const tGrade = gradeFromRate(td.rate)}
                        {@const tColor = gradeColor(tGrade)}
                        <div class="landing-td-card">
                          <div class="landing-td-num">#{touchdowns.length - i}</div>
                          <div class="landing-td-info">
                            <div class="landing-td-rate" style="color:{tColor}">{td.rate.toFixed(0)} fpm</div>
                            <div class="landing-td-meta">{td.speed.toFixed(0)} kt @ {td.time}</div>
                          </div>
                          <div class="landing-td-grade" style="color:{tColor}">{tGrade}</div>
                          <div class="landing-td-bar">
                            <div class="landing-td-fill" style="width:{Math.min(100, Math.abs(td.rate)/5)}%;background:{tColor}"></div>
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                  {#if touchdowns.length > 1}
                    <div class="landing-stats">
                      <div class="landing-stat"><span>AVG</span><strong>{(touchdowns.reduce((s,t) => s + Math.abs(t.rate), 0) / touchdowns.length).toFixed(0)} fpm</strong></div>
                      <div class="landing-stat"><span>BEST</span><strong>{Math.min(...touchdowns.map(t => Math.abs(t.rate))).toFixed(0)} fpm</strong></div>
                      <div class="landing-stat"><span>WORST</span><strong>{Math.max(...touchdowns.map(t => Math.abs(t.rate))).toFixed(0)} fpm</strong></div>
                    </div>
                  {/if}
                </div>
              </div>

            {:else if debriefTab === 'envelope'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">SPEED ENVELOPE</div>
                  <table class="efb-tbl">
                    <thead><tr><th>PARAMETER</th><th>RECORDED</th><th>LIMIT</th><th>STATUS</th></tr></thead>
                    <tbody>
                      <tr><td>Max IAS</td><td class="mono">{maxSpd.toFixed(0)} kt</td><td class="mono">163 kt (Vne)</td><td>{maxSpd > 163 ? '🔴 EXCEEDED' : maxSpd > 129 ? '🟡 CAUTION' : '🟢 OK'}</td></tr>
                      <tr><td>Max G-Force</td><td class="mono">{maxG.toFixed(2)} G</td><td class="mono">3.8 G</td><td>{maxG > 3.8 ? '🔴 EXCEEDED' : maxG > 2.5 ? '🟡 CAUTION' : '🟢 OK'}</td></tr>
                      <tr><td>Min Speed</td><td class="mono">--</td><td class="mono">48 kt (Vs1)</td><td>{stallWarnings > 0 ? '🟡 STALL WARN' : '🟢 OK'}</td></tr>
                      <tr><td>Max Bank</td><td class="mono">{maxBank.toFixed(0)}°</td><td class="mono">60° (std)</td><td>{maxBank > 60 ? '🔴 EXCEEDED' : maxBank > 45 ? '🟡 STEEP' : '🟢 OK'}</td></tr>
                    </tbody>
                  </table>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">EVENTS</div>
                  <div class="efb-readout-grid">
                    <div class="efb-readout"><span class="efb-ro-label">OVERSPEED EVENTS</span><span class="efb-ro-val" class:efb-ro-warn={overspeedEvents>0}>{overspeedEvents}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">STALL WARNINGS</span><span class="efb-ro-val" class:efb-ro-warn={stallWarnings>0}>{stallWarnings}</span></div>
                  </div>
                  {#if overspeedEvents === 0 && stallWarnings === 0}
                    <div class="efb-verdict efb-ok" style="margin-top:12px">CLEAN FLIGHT — NO EXCEEDANCES</div>
                  {:else}
                    <div class="efb-verdict efb-warn" style="margin-top:12px">EXCEEDANCES DETECTED</div>
                  {/if}
                </div>
              </div>

            {:else if debriefTab === 'score'}
              {@const lGrade = gradeFromRate(landingRate)}
              {@const gGrade = maxG < 1.5 ? 'A' : maxG < 2.0 ? 'B' : maxG < 3.0 ? 'C' : 'F'}
              {@const sGrade = overspeedEvents === 0 && stallWarnings === 0 ? 'A' : overspeedEvents + stallWarnings < 3 ? 'B' : 'C'}
              {@const bGrade = maxBank < 30 ? 'A' : maxBank < 45 ? 'B' : maxBank < 60 ? 'C' : 'F'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">PERFORMANCE GRADES</div>
                  <table class="efb-tbl">
                    <thead><tr><th>CATEGORY</th><th>GRADE</th><th>DETAIL</th></tr></thead>
                    <tbody>
                      <tr><td>Landing</td><td><span class="dbr-grade-sm" style="color:{gradeColor(lGrade)}">{lGrade}</span></td><td class="mono">{landingRate.toFixed(0)} fpm</td></tr>
                      <tr><td>G-Force Management</td><td><span class="dbr-grade-sm" style="color:{gradeColor(gGrade)}">{gGrade}</span></td><td class="mono">{maxG.toFixed(2)} G max</td></tr>
                      <tr><td>Speed Discipline</td><td><span class="dbr-grade-sm" style="color:{gradeColor(sGrade)}">{sGrade}</span></td><td class="mono">{overspeedEvents + stallWarnings} events</td></tr>
                      <tr><td>Bank Control</td><td><span class="dbr-grade-sm" style="color:{gradeColor(bGrade)}">{bGrade}</span></td><td class="mono">{maxBank.toFixed(0)}° max</td></tr>
                    </tbody>
                  </table>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">SESSION</div>
                  <div class="efb-readout-grid efb-rg-2x2">
                    <div class="efb-readout"><span class="efb-ro-label">FLIGHTS</span><span class="efb-ro-val">{logEntries.length}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">TOUCHDOWNS</span><span class="efb-ro-val">{touchdowns.length}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">TOTAL TIME</span><span class="efb-ro-val">{logEntries.reduce((s: number, e: any) => s + (e.duration || 0), 0)} <small>min</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">FUEL USED</span><span class="efb-ro-val">{fuelUsed.toFixed(1)} <small>gal</small></span></div>
                  </div>
                  <button class="btn-reset" onclick={resetDebrief}>RESET DEBRIEF DATA</button>
                </div>
              </div>
            {/if}
          </div>
        </div>

      {:else if active === 'nav'}
        <div class="efb-panel">
          <!-- Sub-tabs -->
          <div class="efb-tabs">
            {#each [['wb','W&B'],['wind','Wind'],['fuel','Fuel'],['perf','Performance'],['ref','Reference']] as [id, label]}
              <button class="efb-tab" class:efb-tab-on={navTab===id} onclick={() => navTab=id}>{label}</button>
            {/each}
          </div>

          <div class="efb-body">
            {#if navTab === 'wb'}
              <!-- WEIGHT & BALANCE — full width, 2 col layout -->
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">LOADING</div>
                  <table class="efb-tbl">
                    <thead><tr><th>ITEM</th><th>WEIGHT</th><th>ARM</th><th>MOMENT</th></tr></thead>
                    <tbody>
                      <tr><td>Empty Aircraft</td><td class="mono">{EW}</td><td class="mono">{ECG}</td><td class="mono">{(EW*ECG).toFixed(0)}</td></tr>
                      <tr><td>Pilot + Passengers</td><td><input type="number" onwheel={onWheel} class="efb-input" value={paxWeight} min="0" max="800" oninput={(e: Event) => paxWeight=+(e.target as HTMLInputElement).value} /></td><td class="mono">{PA}</td><td class="mono">{(paxWeight*PA).toFixed(0)}</td></tr>
                      <tr><td>Baggage</td><td><input type="number" onwheel={onWheel} class="efb-input" value={baggageWeight} min="0" max="120" oninput={(e: Event) => baggageWeight=+(e.target as HTMLInputElement).value} /></td><td class="mono">{BA}</td><td class="mono">{(baggageWeight*BA).toFixed(0)}</td></tr>
                      <tr><td>Fuel ({fuelGallons} gal)</td><td><input type="number" onwheel={onWheel} class="efb-input" value={fuelGallons} min="0" max="56" oninput={(e: Event) => fuelGallons=+(e.target as HTMLInputElement).value} /></td><td class="mono">{FA}</td><td class="mono">{(fuelGallons*6*FA).toFixed(0)}</td></tr>
                      <tr class="efb-total"><td>TOTAL</td><td class="mono">{tw}</td><td class="mono">{cg.toFixed(1)}</td><td class="mono">{(tw*cg).toFixed(0)}</td></tr>
                    </tbody>
                  </table>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">RESULT</div>
                  <div class="efb-readout-grid">
                    <div class="efb-readout"><span class="efb-ro-label">GROSS WEIGHT</span><span class="efb-ro-val">{tw} <small>lbs</small></span><span class="efb-ro-limit">max 2550</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">CG POSITION</span><span class="efb-ro-val">{cg.toFixed(1)}<small>"</small></span><span class="efb-ro-limit">35.0 — 47.3</span></div>
                  </div>
                  <div class="efb-verdict" class:efb-ok={ok} class:efb-warn={!ok}>{ok ? 'WITHIN ENVELOPE' : 'OUT OF LIMITS'}</div>
                </div>
              </div>

            {:else if navTab === 'wind'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">CROSSWIND COMPONENT</div>
                  <div class="efb-form">
                    <div class="efb-field"><label>Runway Heading</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={rwyHeading} min="1" max="360" /><span class="input-unit">°</span></div></div>
                    <div class="efb-field"><label>Wind Direction</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={windDir} min="1" max="360" /><span class="input-unit">°</span></div></div>
                    <div class="efb-field"><label>Wind Speed</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={windSpd} min="0" max="99" /><span class="input-unit">kt</span></div></div>
                  </div>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">COMPONENTS</div>
                  <div class="efb-readout-grid">
                    <div class="efb-readout"><span class="efb-ro-label">CROSSWIND</span><span class="efb-ro-val">{crosswind} <small>kt</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">{headwind >= 0 ? 'HEADWIND' : 'TAILWIND'}</span><span class="efb-ro-val" class:efb-ro-warn={headwind<0}>{Math.abs(headwind)} <small>kt</small></span></div>
                  </div>
                  {#if headwind < 0}<div class="efb-warn-msg">Tailwind component — check runway performance</div>{/if}
                </div>
              </div>

            {:else if navTab === 'fuel'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">TRIP PLANNER</div>
                  <div class="efb-form">
                    <div class="efb-field"><label>Distance</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={tripDist} min="0" max="999" /><span class="input-unit">nm</span></div></div>
                    <div class="efb-field"><label>Ground Speed</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={cruiseSpeed} min="1" max="200" /><span class="input-unit">kt</span></div></div>
                    <div class="efb-field"><label>Fuel Burn</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={fuelBurn} min="1" max="20" step="0.5" /><span class="input-unit">GPH</span></div></div>
                    <div class="efb-field"><label>VFR Reserve</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={reserveMin} min="0" max="120" /><span class="input-unit">min</span></div></div>
                  </div>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">CALCULATION</div>
                  <div class="efb-readout-grid efb-rg-2x2">
                    <div class="efb-readout"><span class="efb-ro-label">TRIP TIME</span><span class="efb-ro-val">{tripTime.toFixed(0)} <small>min</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">TRIP FUEL</span><span class="efb-ro-val">{tripFuel.toFixed(1)} <small>gal</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">RESERVE</span><span class="efb-ro-val">{reserveFuel.toFixed(1)} <small>gal</small></span></div>
                    <div class="efb-readout efb-readout-accent"><span class="efb-ro-label">TOTAL REQUIRED</span><span class="efb-ro-val">{totalFuelReq.toFixed(1)} <small>gal</small></span></div>
                  </div>
                  <div class="efb-heading" style="margin-top:12px">LIVE FROM SIM</div>
                  <div class="efb-readout-grid" style="grid-template-columns:repeat(3,1fr)">
                    <div class="efb-readout"><span class="efb-ro-label">FLOW</span><span class="efb-ro-val">{(simData?.fuel_flow??0).toFixed(1)} <small>GPH</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">QUANTITY</span><span class="efb-ro-val">{(simData?.fuel_qty??0).toFixed(1)} <small>gal</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">ENDURANCE</span><span class="efb-ro-val">{(simData?.fuel_flow>0)?((simData?.fuel_qty??0)/simData.fuel_flow*60).toFixed(0):'--'} <small>min</small></span></div>
                  </div>
                </div>
              </div>

            {:else if navTab === 'perf'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">DENSITY ALTITUDE</div>
                  <div class="efb-form">
                    <div class="efb-field"><label>Field Elevation</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={fieldElev} min="0" max="14000" /><span class="input-unit">ft</span></div></div>
                    <div class="efb-field"><label>Outside Air Temp</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={oat} min="-40" max="50" /><span class="input-unit">°C</span></div></div>
                    <div class="efb-field"><label>Altimeter Setting</label><div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={altSetting} min="28" max="31" step="0.01" /><span class="input-unit">inHg</span></div></div>
                  </div>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">RESULT</div>
                  <div class="efb-readout-grid">
                    <div class="efb-readout"><span class="efb-ro-label">PRESSURE ALT</span><span class="efb-ro-val">{pressAlt} <small>ft</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">DENSITY ALT</span><span class="efb-ro-val" class:efb-ro-warn={densAlt>5000}>{densAlt} <small>ft</small></span></div>
                  </div>
                  {#if densAlt > 5000}<div class="efb-warn-msg">High density altitude — reduced performance</div>{/if}
                  {#if densAlt > 8000}<div class="efb-warn-msg" style="color:var(--color-red)">Very high DA — consider weight reduction</div>{/if}
                </div>
              </div>

            {:else if navTab === 'ref'}
              <div class="efb-section" style="max-width:500px">
                <div class="efb-heading">V-SPEEDS — C172S</div>
                <table class="efb-tbl">
                  <thead><tr><th>SPEED</th><th>KIAS</th><th>DESCRIPTION</th></tr></thead>
                  <tbody>
                    {#each vspeeds as [name, val, desc]}
                      <tr><td class="td-accent mono">{name}</td><td class="mono efb-speed-val">{val}</td><td class="efb-speed-desc">{desc}</td></tr>
                    {/each}
                  </tbody>
                </table>
                <div class="efb-note">Values from C172S POH. Va varies with weight.</div>
              </div>
            {/if}
          </div>
        </div>

      {:else if active === 'hardware'}
        <div class="efb-panel">
          <div class="efb-tabs">
            {#each [['map','Mapping'],['info','Devices'],['flash','Firmware'],['monitor','Monitor'],['profiles','Profiles'],['flows','Flows']] as [id,label]}
              <button class="efb-tab" class:efb-tab-on={hwTab===id} onclick={() => hwTab=id}>{label}</button>
            {/each}
            <div class="efb-tab-spacer"></div>
            <div class="efb-flight-indicator" class:efb-fi-active={arduinoStatus==='connected'}>
              <span class="dot" class:on={arduinoStatus==='connected'}></span>
              {arduinoStatus === 'connected' ? 'HW ONLINE' : 'NO HW'}
            </div>
          </div>
          <div class="efb-body">
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
              <div class="hw-combined">
                <!-- LEFT: Pin assignments per device -->
                <div class="hw-pins-col">
                  {#each hwDevices as dev, di}
                    <div class="hw-dev-banner">
                      <span class="hw-fn-dot" class:hw-fn-dot-on={dev.port === 'COM7' && arduinoStatus === 'connected'}></span>
                      <span class="hw-dev-banner-name">{dev.name}</span>
                      <span class="hw-dev-type-badge">{dev.type.toUpperCase()}</span>
                      <span class="hw-dev-banner-port">{dev.port}</span>
                    </div>
                    <div class="hw-section-label">DIGITAL</div>
                    {#each digitalPins as pin}
                      {#if hwDevices.length <= 1 || pinDevice[pin] === dev.id || (!pinDevice[pin] && di === 0)}
                        {@const mapped = !!hwMap[pin]}
                        {@const isActive = arduinoLastPin?.includes(pin)}
                        <div class="hw-row" data-hw-id={pin} class:hw-row-mapped={mapped} class:hw-row-active={isActive} class:hw-row-hl={hwHighlight === pin || (hwMap[pin] && hwHighlight === hwMap[pin])}
                             onmouseenter={() => { hwHighlight = hwMap[pin] || pin; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}
                             onmouseleave={() => hwHighlight = ''}
                             onclick={() => { const id = hwMap[pin] || pin; hwHighlight = hwHighlight === id ? '' : id; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}>
                          <span class="hw-dot" class:hw-dot-mapped={mapped} class:hw-dot-active={isActive}></span>
                          <span class="hw-pin">{pin}</span>
                          <select class="hw-select" bind:value={hwMap[pin]} onchange={() => hwMap={...hwMap}} onclick={(e: Event) => e.stopPropagation()}>
                            <option value="">— None —</option>
                            {#each ['Electrical','Lights','Engine','Fuel','Controls','Autopilot','Avionics','Audio','Transponder','Environment','Misc'] as g}{@const items=digitalFunctions.filter(f=>f.group===g)}{#if items.length}<optgroup label={g}>{#each items as f}{@const onPin=usedOnPin(f.id,pin)}<option value={f.id}>{onPin ? '● ' : '○ '}{f.label}{onPin ? ` [${onPin}]` : ''}</option>{/each}</optgroup>{/if}{/each}
                          </select>
                          {#if isActive}<span class="hw-live-tag">ON</span>{/if}
                        </div>
                      {/if}
                    {/each}
                    <div class="hw-section-label" style="margin-top:6px">ANALOG</div>
                    {#each analogPins as pin}
                      {#if hwDevices.length <= 1 || pinDevice[pin] === dev.id || (!pinDevice[pin] && di === 0)}
                        {@const mapped = !!hwMap[pin]}
                        <div class="hw-row" data-hw-id={pin} class:hw-row-mapped={mapped} class:hw-row-hl={hwHighlight === pin || (hwMap[pin] && hwHighlight === hwMap[pin])}
                             onmouseenter={() => { hwHighlight = hwMap[pin] || pin; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}
                             onmouseleave={() => hwHighlight = ''}
                             onclick={() => { const id = hwMap[pin] || pin; hwHighlight = hwHighlight === id ? '' : id; if (hwMap[pin]) hwScrollTo(hwMap[pin]); }}>
                          <span class="hw-dot" class:hw-dot-mapped={mapped}></span>
                          <span class="hw-pin">{pin}</span>
                          <select class="hw-select" bind:value={hwMap[pin]} onchange={() => hwMap={...hwMap}} onclick={(e: Event) => e.stopPropagation()}>
                            <option value="">— None —</option>
                            {#each ['Engine','Fuel','Controls','Avionics','Audio','Transponder','Lights','Environment'] as g}{@const items=analogFunctions.filter(f=>f.group===g)}{#if items.length}<optgroup label={g}>{#each items as f}{@const onPin=usedOnPin(f.id,pin)}<option value={f.id}>{onPin ? '● ' : '○ '}{f.label}{onPin ? ` [${onPin}]` : ''}</option>{/each}</optgroup>{/if}{/each}
                          </select>
                        </div>
                      {/if}
                    {/each}
                  {/each}
                </div>
                <!-- RIGHT: All functions with filters -->
                <div class="hw-fns-col">
                  <div class="hw-filter-bar">
                    <input class="hw-search" type="text" placeholder="Search..." bind:value={hwSearch} />
                    <div class="hw-filter-btns">
                      <button class="cl-tgl" class:cl-tgl-on={hwFilter==='all'} onclick={() => hwFilter='all'}>ALL</button>
                      <button class="cl-tgl" class:cl-tgl-on={hwFilter==='mapped'} onclick={() => hwFilter='mapped'}>IN USE</button>
                      <button class="cl-tgl" class:cl-tgl-on={hwFilter==='unmapped'} onclick={() => hwFilter='unmapped'}>FREE</button>
                    </div>
                  </div>
                  <div class="hw-group-pills">
                    {#each ['all','Electrical','Lights','Engine','Fuel','Controls','Autopilot','Avionics','Audio','Transponder','Environment'] as g}
                      <button class="hw-pill" class:hw-pill-on={hwGroup===g} onclick={() => hwGroup=g}>{g === 'all' ? 'All' : g}</button>
                    {/each}
                  </div>
                  <div class="hw-fn-list">
                    {#each ['Electrical','Lights','Engine','Fuel','Controls','Autopilot','Avionics','Audio','Transponder','Environment','Misc'] as group}
                      {@const items = [...digitalFunctions, ...analogFunctions].filter(f => f.group === group).filter(filterFn)}
                      {#if items.length}
                        <div class="hw-group-label">{group}</div>
                        {#each items as f}
                          {@const pin = Object.entries(hwMap).find(([,v]) => v === f.id)?.[0]}
                          <div class="hw-fn-row" data-hw-id={f.id} class:hw-fn-mapped={!!pin} class:hw-row-hl={hwHighlight === f.id || (pin && hwHighlight === pin)}
                              onmouseenter={() => { hwHighlight = pin || f.id; if (pin) hwScrollTo(pin); }}
                              onmouseleave={() => hwHighlight = ''}
                              onclick={() => { hwHighlight = hwHighlight === f.id ? '' : (pin || f.id); if (pin) hwScrollTo(pin); }}>
                            <span class="hw-fn-dot" class:hw-fn-dot-on={!!pin}></span>
                            <span class="hw-fn-name">{f.label}</span>
                            {#if pin}<span class="hw-fn-pin">{pin}</span><span class="hw-fn-dev">{getDeviceName(pin)}</span>{:else}<span class="hw-fn-none">—</span>{/if}
                          </div>
                        {/each}
                      {/if}
                    {/each}
                  </div>
                  <div class="hw-fn-summary">{usedIds.length} mapped / {digitalFunctions.length + analogFunctions.length} total</div>
                </div>
              </div>

            {:else if hwTab === 'info'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">DEVICES</div>
                  {#each hwDevices as dev, i}
                    <div class="hw-device-card">
                      <div class="hw-dev-header">
                        <span class="hw-fn-dot" class:hw-fn-dot-on={dev.port === 'COM7' && arduinoStatus === 'connected'}></span>
                        <input class="hw-dev-name-input" bind:value={hwDevices[i].name} onchange={() => hwDevices=[...hwDevices]} placeholder="Device name" />
                        <span class="hw-dev-type-badge">{dev.type.toUpperCase()}</span>
                        {#if hwDevices.length > 1}
                          <button class="hw-dev-remove" onclick={() => { hwDevices = hwDevices.filter((_,j) => j !== i); }}>x</button>
                        {/if}
                      </div>
                      <div class="hw-dev-fields">
                        <div class="hw-dev-field"><span>Port</span><input class="hw-dev-port-input" bind:value={hwDevices[i].port} onchange={() => hwDevices=[...hwDevices]} placeholder="COM7" /></div>
                        <div class="hw-dev-field"><span>Type</span>
                          <select class="hw-dev-type-sel" bind:value={hwDevices[i].type} onchange={() => hwDevices=[...hwDevices]}>
                            <option value="arduino">Arduino</option>
                            <option value="esp32">ESP32</option>
                          </select>
                        </div>
                        <div class="hw-dev-field"><span>Pins used</span><span class="mono">{Object.entries(pinDevice).filter(([,d]) => d === dev.id).length + (i === 0 ? Object.entries(pinDevice).filter(([,d]) => !d).length : 0)}</span></div>
                      </div>
                    </div>
                  {/each}
                  <button class="hw-add-dev" onclick={() => { hwDevices = [...hwDevices, { id: `dev${Date.now()}`, name: `Device ${hwDevices.length+1}`, port: 'COM8', type: 'arduino' }]; }}>+ Add Device</button>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">PROTOCOL</div>
                  <table class="efb-tbl"><tbody>
                    <tr><td>Baud Rate</td><td class="mono">115200</td></tr>
                    <tr><td>Frame</td><td class="mono">6B (HDR DEV PIN VL VH CK)</td></tr>
                    <tr><td>Digital</td><td class="mono">0xAA</td></tr>
                    <tr><td>Analog</td><td class="mono">0xCC</td></tr>
                  </tbody></table>
                  <div class="efb-heading" style="margin-top:14px">SUMMARY</div>
                  <table class="efb-tbl"><tbody>
                    <tr><td>Devices</td><td class="mono">{hwDevices.length}</td></tr>
                    <tr><td>Digital Mapped</td><td class="mono">{digitalPins.filter(p=>hwMap[p]).length}/{digitalPins.length}</td></tr>
                    <tr><td>Analog Mapped</td><td class="mono">{analogPins.filter(p=>hwMap[p]).length}/{analogPins.length}</td></tr>
                    <tr><td>Total Functions</td><td class="mono">{Object.values(hwMap).filter(Boolean).length}</td></tr>
                    <tr><td>Last Event</td><td class="mono">{arduinoLastPin||'—'}</td></tr>
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

      {:else if active === 'console'}
        <div class="card console-card"><div class="console-grid">{#each Object.entries(simData||{}).filter(([k])=>k!=='connected') as [k,v]}<div class="con-row"><span class="con-key">{k}</span><span class="con-val" class:con-live={v!==0&&v!==''}>{typeof v==='number'?(v as number).toFixed(4):v}</span></div>{/each}
          {#if Object.keys(simData||{}).length<=1}<div class="v-center" style="grid-column:1/-1;padding:40px 0"><Terminal size={28} strokeWidth={1} /><p>Connect simulator for data</p></div>{/if}
        </div></div>

      {:else if active === 'logbook'}
        {#if logEntries.length===0}<div class="v-center"><BookOpen size={32} strokeWidth={1} /><p>No flights recorded yet</p></div>
        {:else}<div class="card flex-1"><table class="tbl"><thead><tr><th>#</th><th>DATE</th><th>DURATION</th><th>ALT</th></tr></thead><tbody>{#each logEntries as e,i}<tr><td class="td-accent">{i+1}</td><td>{new Date(e.start).toLocaleDateString()}</td><td>{e.duration}m</td><td>{e.maxAlt?.toFixed(0)??'--'}ft</td></tr>{/each}</tbody></table></div>{/if}

      {:else if active === 'flight'}
        <div class="efb-panel">
          <div class="efb-tabs">
            {#each [['plan','Plan'],['live','Live'],['profile','Profile'],['briefing','Briefing']] as [id,label]}
              <button class="efb-tab" class:efb-tab-on={flightTab===id} onclick={() => flightTab=id}>{label}</button>
            {/each}
            <div class="efb-tab-spacer"></div>
            <div class="efb-flight-indicator" class:efb-fi-active={flightActive}>
              <span class="dot" class:on={flightActive}></span>
              {flightPhase}
            </div>
          </div>
          <div class="efb-body">

            {#if flightTab === 'plan'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">ROUTE</div>
                  <div class="fl-route-row">
                    <div class="fl-airport">
                      <label>DEP</label>
                      <input class="fl-icao" bind:value={flightPlan.dep} placeholder="ICAO" />
                    </div>
                    <div class="fl-arrow">---</div>
                    <div class="fl-airport">
                      <label>ARR</label>
                      <input class="fl-icao" bind:value={flightPlan.arr} placeholder="ICAO" />
                    </div>
                    <div class="fl-airport">
                      <label>CRZ ALT</label>
                      <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={flightPlan.cruise} min="500" max="18000" step="500" /><span class="input-unit">ft</span></div>
                    </div>
                  </div>

                  <div class="efb-heading" style="margin-top:14px">WAYPOINTS</div>
                  {#each flightPlan.waypoints as wp, i}
                    <div class="fl-wp-row">
                      <span class="fl-wp-num">{i + 1}</span>
                      <input class="fl-wp-input" bind:value={flightPlan.waypoints[i].name} placeholder="Fix/VOR" />
                      <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={flightPlan.waypoints[i].alt} min="0" max="18000" step="100" /><span class="input-unit">ft</span></div>
                      <input class="fl-wp-freq" bind:value={flightPlan.waypoints[i].freq} placeholder="Freq" />
                      <select class="fl-wp-fail" bind:value={flightPlan.waypoints[i].failureId}>
                        <option value="">No failure</option>
                        {#each failures as f}<option value={f.id}>{f.label}</option>{/each}
                      </select>
                      <button class="af-remove" onclick={() => removeWaypoint(i)}>x</button>
                    </div>
                  {/each}
                  <button class="hw-add-dev" onclick={addWaypoint}>+ Add Waypoint</button>

                  <div class="efb-heading" style="margin-top:16px">EVENTS / FAILURES</div>
                  {#each flightEvents as ev, i}
                    <div class="fl-event-row">
                      <select class="fl-ev-fail" bind:value={flightEvents[i].failureId} onchange={() => flightEvents=[...flightEvents]}>
                        <option value="">— Select failure —</option>
                        {#each failureGroups as g}
                          <optgroup label={g}>{#each failures.filter(f=>f.group===g) as f}<option value={f.id}>{f.label}</option>{/each}</optgroup>
                        {/each}
                      </select>
                      <select class="fl-ev-trigger" bind:value={flightEvents[i].trigger} onchange={() => flightEvents=[...flightEvents]}>
                        <option value="time">After time</option>
                        <option value="altitude">At altitude</option>
                        <option value="phase">At phase</option>
                        <option value="random">Random</option>
                      </select>
                      {#if ev.trigger === 'time'}
                        <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={flightEvents[i].timeMin} min="1" max="120" /><span class="input-unit">min</span></div>
                      {:else if ev.trigger === 'altitude'}
                        <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={flightEvents[i].altFt} min="100" max="18000" step="100" /><span class="input-unit">ft</span></div>
                      {:else if ev.trigger === 'phase'}
                        <select class="fl-ev-trigger" bind:value={flightEvents[i].phase} onchange={() => flightEvents=[...flightEvents]}>
                          {#each ['TAXI','TAKEOFF/CLIMB','CRUISE','DESCENT','APPROACH'] as p}<option value={p}>{p}</option>{/each}
                        </select>
                      {:else if ev.trigger === 'random'}
                        <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={flightEvents[i].randomMin} min="1" max="120" /><span class="input-unit">-</span></div>
                        <div class="input-wrap"><input type="number" onwheel={onWheel} class="efb-input" bind:value={flightEvents[i].randomMax} min="1" max="120" /><span class="input-unit">min</span></div>
                      {/if}
                      <button class="af-remove" onclick={() => removeFlightEvent(i)}>x</button>
                    </div>
                  {/each}
                  <button class="hw-add-dev" onclick={addFlightEvent}>+ Add Event</button>
                </div>

                <div class="efb-section">
                  <div class="efb-heading">ROUTE PREVIEW</div>
                  <svg viewBox="0 0 400 160" class="fl-route-svg">
                    <!-- Background -->
                    <rect x="0" y="0" width="400" height="160" fill="var(--color-bg)" rx="4" />
                    <!-- Ground -->
                    <line x1="20" y1="140" x2="380" y2="140" stroke="var(--color-border)" stroke-width="1" />
                    <!-- Altitude profile line -->
                    {#if flightPlan.waypoints.length > 0}
                      {@const maxAlt = Math.max(flightPlan.cruise, ...flightPlan.waypoints.map(w => w.alt), 1000)}
                      {@const pts = [
                        { x: 30, y: 138 },
                        ...flightPlan.waypoints.map((w, i) => ({
                          x: 30 + ((i + 1) / (flightPlan.waypoints.length + 1)) * 340,
                          y: 140 - (w.alt / maxAlt) * 120
                        })),
                        { x: 370, y: 138 }
                      ]}
                      <polyline points={pts.map(p => `${p.x},${p.y}`).join(' ')} fill="none" stroke="var(--color-accent)" stroke-width="2" stroke-linejoin="round" />
                      <!-- Waypoint dots -->
                      {#each pts.slice(1, -1) as p, i}
                        <circle cx={p.x} cy={p.y} r="4" fill={flightPlan.waypoints[i].failureId ? 'var(--color-red)' : 'var(--color-accent)'} />
                        <text x={p.x} y={p.y - 8} fill="var(--color-fg)" font-size="8" text-anchor="middle" font-family="monospace">{flightPlan.waypoints[i].name || 'WP' + (i+1)}</text>
                        <text x={p.x} y={p.y + 14} fill="var(--color-dim)" font-size="7" text-anchor="middle" font-family="monospace">{flightPlan.waypoints[i].alt}ft</text>
                        {#if flightPlan.waypoints[i].failureId}
                          <text x={p.x} y={p.y + 22} fill="var(--color-red)" font-size="6" text-anchor="middle" font-family="monospace">FAIL</text>
                        {/if}
                      {/each}
                    {:else}
                      <text x="200" y="80" fill="var(--color-dim)" font-size="10" text-anchor="middle" font-family="sans-serif">Add waypoints to see route profile</text>
                    {/if}
                    <!-- DEP/ARR labels -->
                    <text x="30" y="152" fill="var(--color-accent)" font-size="9" font-family="monospace" font-weight="bold">{flightPlan.dep || 'DEP'}</text>
                    <text x="370" y="152" fill="var(--color-accent)" font-size="9" font-family="monospace" font-weight="bold" text-anchor="end">{flightPlan.arr || 'ARR'}</text>
                  </svg>

                  <div class="efb-heading" style="margin-top:12px">PLAN SUMMARY</div>
                  <div class="efb-readout-grid efb-rg-2x2">
                    <div class="efb-readout"><span class="efb-ro-label">WAYPOINTS</span><span class="efb-ro-val" style="font-size:16px">{flightPlan.waypoints.length}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">EVENTS</span><span class="efb-ro-val" style="font-size:16px;color:{(flightPlan.waypoints.filter(w=>w.failureId).length + flightEvents.filter(e=>e.failureId).length) ? 'var(--color-red)' : 'var(--color-dim)'}">{flightPlan.waypoints.filter(w=>w.failureId).length + flightEvents.filter(e=>e.failureId).length}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">CRUISE</span><span class="efb-ro-val" style="font-size:16px">{flightPlan.cruise}<small>ft</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">ROUTE</span><span class="efb-ro-val" style="font-size:14px">{flightPlan.dep || '?'} - {flightPlan.arr || '?'}</span></div>
                  </div>
                </div>
              </div>

            {:else if flightTab === 'live'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">CURRENT STATE</div>
                  <div class="efb-readout-grid efb-rg-2x2">
                    <div class="efb-readout"><span class="efb-ro-label">PHASE</span><span class="efb-ro-val" style="font-size:14px;color:var(--color-accent)">{flightPhase}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">ALTITUDE</span><span class="efb-ro-val" style="font-size:16px">{(simData?.altitude ?? 0).toFixed(0)}<small>ft</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">GROUND SPEED</span><span class="efb-ro-val" style="font-size:16px">{(simData?.groundspeed ?? 0).toFixed(0)}<small>kt</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">VS</span><span class="efb-ro-val" style="font-size:16px;color:{(simData?.vs ?? 0) > 100 ? 'var(--color-green)' : (simData?.vs ?? 0) < -100 ? 'var(--color-yellow)' : 'var(--color-fg)'}">{(simData?.vs ?? 0).toFixed(0)}<small>fpm</small></span></div>
                  </div>
                  <div class="efb-readout-grid efb-rg-2x2" style="margin-top:8px">
                    <div class="efb-readout"><span class="efb-ro-label">IAS</span><span class="efb-ro-val" style="font-size:16px">{(simData?.airspeed ?? 0).toFixed(0)}<small>kt</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">HEADING</span><span class="efb-ro-val" style="font-size:16px">{(simData?.heading ?? 0).toFixed(0)}<small>°</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">RPM</span><span class="efb-ro-val" style="font-size:16px">{(simData?.rpm ?? 0).toFixed(0)}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">FUEL</span><span class="efb-ro-val" style="font-size:16px">{(simData?.fuel_qty ?? 0).toFixed(1)}<small>gal</small></span></div>
                  </div>
                  <div class="fl-rec-btns">
                    <button class="btn-action" onclick={startRecording}>REC</button>
                    <button class="btn-action" style="background:var(--color-red)" onclick={stopRecording}>STOP</button>
                  </div>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">ALTITUDE PROFILE (LIVE)</div>
                  <svg viewBox="0 0 400 180" class="fl-route-svg">
                    <rect x="0" y="0" width="400" height="180" fill="var(--color-bg)" rx="4" />
                    <line x1="20" y1="160" x2="380" y2="160" stroke="var(--color-border)" stroke-width="1" />
                    {#if altProfile.length > 1}
                      {@const maxA = Math.max(...altProfile.map(p => p.alt), 500)}
                      {@const step = Math.max(1, Math.floor(altProfile.length / 380))}
                      <polyline points={altProfile.filter((_,i) => i % step === 0).map((p, i, arr) =>
                        `${20 + (i / arr.length) * 360},${158 - (p.alt / maxA) * 140}`
                      ).join(' ')} fill="none" stroke="var(--color-green)" stroke-width="1.5" />
                      <!-- Current position dot -->
                      {@const last = altProfile[altProfile.length - 1]}
                      <circle cx="380" cy={158 - (last.alt / maxA) * 140} r="3" fill="var(--color-green)" />
                      <!-- Scale -->
                      <text x="14" y="22" fill="var(--color-dim)" font-size="7" font-family="monospace" text-anchor="end">{maxA.toFixed(0)}</text>
                      <text x="14" y="160" fill="var(--color-dim)" font-size="7" font-family="monospace" text-anchor="end">0</text>
                    {:else}
                      <text x="200" y="90" fill="var(--color-dim)" font-size="10" text-anchor="middle" font-family="sans-serif">Press REC to start recording</text>
                    {/if}
                  </svg>
                </div>
              </div>

            {:else if flightTab === 'profile'}
              <div class="efb-section">
                <div class="efb-heading">FLIGHT PROFILE DATA ({altProfile.length} samples)</div>
                {#if altProfile.length === 0}
                  <div class="efb-empty">No data recorded. Use Live tab to start recording.</div>
                {:else}
                  <div class="efb-readout-grid" style="grid-template-columns:repeat(5,1fr)">
                    <div class="efb-readout"><span class="efb-ro-label">DURATION</span><span class="efb-ro-val" style="font-size:14px">{Math.round((altProfile[altProfile.length-1].t - altProfile[0].t) / 60000)}<small>min</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">MAX ALT</span><span class="efb-ro-val" style="font-size:14px">{Math.max(...altProfile.map(p=>p.alt)).toFixed(0)}<small>ft</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">MAX GS</span><span class="efb-ro-val" style="font-size:14px">{Math.max(...altProfile.map(p=>p.gs)).toFixed(0)}<small>kt</small></span></div>
                    <div class="efb-readout"><span class="efb-ro-label">SAMPLES</span><span class="efb-ro-val" style="font-size:14px">{altProfile.length}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">EXPORT</span><button class="btn-action" style="font-size:9px;padding:4px 8px;margin-top:2px" onclick={() => { const csv = 'Time,Altitude,GS\n' + altProfile.map(p => `${new Date(p.t).toISOString()},${p.alt.toFixed(0)},${p.gs.toFixed(0)}`).join('\n'); const b = new Blob([csv],{type:'text/csv'}); const a = document.createElement('a'); a.href = URL.createObjectURL(b); a.download = 'flight-profile.csv'; a.click(); }}>CSV</button></div>
                  </div>
                {/if}
              </div>

            {:else if flightTab === 'briefing'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">DEPARTURE BRIEFING</div>
                  <table class="efb-tbl">
                    <tbody>
                      <tr><td>Departure</td><td class="mono">{flightPlan.dep || '—'}</td></tr>
                      <tr><td>Arrival</td><td class="mono">{flightPlan.arr || '—'}</td></tr>
                      <tr><td>Cruise Altitude</td><td class="mono">{flightPlan.cruise} ft</td></tr>
                      <tr><td>Waypoints</td><td class="mono">{flightPlan.waypoints.map(w => w.name || '?').join(' → ') || 'Direct'}</td></tr>
                      <tr><td>Planned Failures</td><td class="mono" style="color:{flightPlan.waypoints.filter(w=>w.failureId).length ? 'var(--color-red)' : 'var(--color-dim)'}">{flightPlan.waypoints.filter(w=>w.failureId).length || 'None'}</td></tr>
                    </tbody>
                  </table>
                  <div class="efb-heading" style="margin-top:14px">FREQUENCIES</div>
                  <table class="efb-tbl">
                    <tbody>
                      {#each flightPlan.waypoints.filter(w => w.freq) as wp}
                        <tr><td>{wp.name || 'WP'}</td><td class="mono">{wp.freq}</td></tr>
                      {/each}
                      <tr><td>Emergency</td><td class="mono">121.500</td></tr>
                    </tbody>
                  </table>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">EMERGENCY</div>
                  <table class="efb-tbl">
                    <tbody>
                      <tr><td>Engine Failure</td><td class="mono">Pitch 65 KIAS, best field</td></tr>
                      <tr><td>Electrical Fire</td><td class="mono">Master OFF, land ASAP</td></tr>
                      <tr><td>Engine Fire</td><td class="mono">Mixture IDLE CUT, fuel OFF</td></tr>
                      <tr><td>Comm Failure</td><td class="mono">7600, last clearance</td></tr>
                    </tbody>
                  </table>
                  <div class="efb-heading" style="margin-top:14px">V-SPEEDS</div>
                  <div class="efb-readout-grid" style="grid-template-columns:repeat(4,1fr)">
                    <div class="efb-readout"><span class="efb-ro-label">Vr</span><span class="efb-ro-val" style="font-size:14px">55</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">Vx</span><span class="efb-ro-val" style="font-size:14px">62</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">Vy</span><span class="efb-ro-val" style="font-size:14px">74</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">Vglide</span><span class="efb-ro-val" style="font-size:14px">65</span></div>
                  </div>
                </div>
              </div>
            {/if}

          </div>
        </div>

      {:else if active === 'community'}
        <div class="efb-panel">
          <div class="efb-tabs">
            {#each [['leaderboard','Leaderboard'],['profile','Profile'],['connect','Connect'],['share','Share']] as [id,label]}
              <button class="efb-tab" class:efb-tab-on={communityTab===id} onclick={() => communityTab=id}>{label}</button>
            {/each}
          </div>
          <div class="efb-body">
            {#if communityTab === 'leaderboard'}
              {@const sorted = [...allLandings].sort((a,b) => Math.abs(a.rate) - Math.abs(b.rate))}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">PERSONAL BEST LANDINGS</div>
                  {#if sorted.length === 0}
                    <div class="efb-empty">No landings recorded yet. Fly and land to build your leaderboard.</div>
                  {:else}
                    <div class="lb-list">
                      {#each sorted.slice(0, 20) as ld, i}
                        {@const g = gradeFromRate(ld.rate)}
                        <div class="lb-row" class:lb-top3={i < 3}>
                          <span class="lb-rank" class:lb-gold={i===0} class:lb-silver={i===1} class:lb-bronze={i===2}>{i + 1}</span>
                          <span class="lb-rate" style="color:{gradeColor(g)}">{ld.rate.toFixed(0)} fpm</span>
                          <span class="lb-grade" style="color:{gradeColor(g)}">{g}</span>
                          <span class="lb-speed">{ld.speed.toFixed(0)} kt</span>
                          <span class="lb-date">{ld.date}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
                <div class="efb-section">
                  <div class="efb-heading">STATS</div>
                  <div class="efb-readout-grid efb-rg-2x2">
                    <div class="efb-readout"><span class="efb-ro-label">TOTAL LANDINGS</span><span class="efb-ro-val">{allLandings.length}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">BEST EVER</span><span class="efb-ro-val" style="color:var(--color-green)">{allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">AVERAGE</span><span class="efb-ro-val">{allLandings.length ? (allLandings.reduce((s,l) => s + Math.abs(l.rate), 0) / allLandings.length).toFixed(0) : '—'}</span></div>
                    <div class="efb-readout"><span class="efb-ro-label">BUTTER RATE</span><span class="efb-ro-val" style="color:var(--color-green)">{allLandings.length ? Math.round(allLandings.filter(l => Math.abs(l.rate) < 60).length / allLandings.length * 100) : 0}%</span></div>
                  </div>
                  <div class="efb-heading" style="margin-top:14px">GRADE DISTRIBUTION</div>
                  <div class="lb-dist">
                    {#each ['A+','A','B','C','D','F'] as g}
                      {@const count = allLandings.filter(l => gradeFromRate(l.rate) === g).length}
                      {@const pct = allLandings.length ? count / allLandings.length * 100 : 0}
                      <div class="lb-dist-row">
                        <span class="lb-dist-grade" style="color:{gradeColor(g)}">{g}</span>
                        <div class="lb-dist-bar-bg"><div class="lb-dist-bar-fill" style="width:{pct}%;background:{gradeColor(g)}"></div></div>
                        <span class="lb-dist-count">{count}</span>
                      </div>
                    {/each}
                  </div>
                  {#if allLandings.length > 0}
                    <button class="btn-reset" style="margin-top:12px" onclick={() => { if (confirm('Clear all landing history?')) allLandings = []; }}>CLEAR HISTORY</button>
                  {/if}
                </div>
              </div>

            {:else if communityTab === 'profile'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">PILOT PROFILE</div>
                  <div class="prof-form">
                    <div class="prof-field"><label>Pilot Name</label><input class="prof-input" bind:value={pilotName} placeholder="Your name" /></div>
                    <div class="prof-field"><label>Callsign</label><input class="prof-input" bind:value={pilotCallsign} placeholder="N172SP" /></div>
                  </div>
                  <div class="prof-card">
                    <div class="prof-card-header">
                      <div class="prof-avatar">{pilotName.charAt(0).toUpperCase()}</div>
                      <div><div class="prof-name">{pilotName}</div><div class="prof-cs">{pilotCallsign}</div></div>
                    </div>
                    <div class="prof-stats">
                      <div class="prof-stat"><span>{allLandings.length}</span><label>Landings</label></div>
                      <div class="prof-stat"><span>{allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'}</span><label>Best fpm</label></div>
                      <div class="prof-stat"><span>{logEntries.reduce((s: number, e: any) => s + (e.duration || 0), 0)}</span><label>Total min</label></div>
                    </div>
                  </div>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">DISCORD INTEGRATION</div>
                  <div class="connect-placeholder">
                    <svg width="28" height="28" viewBox="0 0 24 24" fill="#5865F2"><path d="M20.317 4.37a19.8 19.8 0 00-4.885-1.515.07.07 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.3 18.3 0 00-5.487 0 12.6 12.6 0 00-.617-1.25.08.08 0 00-.079-.037A19.7 19.7 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.08.08 0 00.031.057 19.9 19.9 0 005.993 3.03.08.08 0 00.084-.028 14.1 14.1 0 001.226-1.994.08.08 0 00-.041-.106 13.1 13.1 0 01-1.872-.892.08.08 0 01-.008-.128c.125-.094.25-.192.372-.292a.07.07 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.07.07 0 01.078.01c.12.1.246.198.373.292a.08.08 0 01-.006.127c-.598.35-1.22.645-1.873.892a.08.08 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.08.08 0 00.084.028 19.8 19.8 0 006.002-3.03.08.08 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.06.06 0 00-.031-.03z"/></svg>
                    <span class="connect-title">Connect Discord</span>
                    <span class="connect-desc">Auto-post landings, share stats, join leaderboards</span>
                    <button class="connect-btn" disabled>COMING SOON</button>
                  </div>
                  <div class="efb-heading" style="margin-top:14px">SETTINGS</div>
                  <div class="connect-options">
                    <label class="connect-opt"><input type="checkbox" checked disabled /><span>Auto-post butter landings to Discord</span></label>
                    <label class="connect-opt"><input type="checkbox" checked disabled /><span>Show Discord Rich Presence</span></label>
                    <label class="connect-opt"><input type="checkbox" disabled /><span>Share flight stats publicly</span></label>
                  </div>
                </div>
              </div>

            {:else if communityTab === 'connect'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">LINKS</div>
                  <a href="https://discord.gg/cockpitflow" target="_blank" class="link-btn discord"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M20.317 4.37a19.8 19.8 0 00-4.885-1.515.07.07 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.3 18.3 0 00-5.487 0 12.6 12.6 0 00-.617-1.25.08.08 0 00-.079-.037A19.7 19.7 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.08.08 0 00.031.057 19.9 19.9 0 005.993 3.03.08.08 0 00.084-.028 14.1 14.1 0 001.226-1.994.08.08 0 00-.041-.106 13.1 13.1 0 01-1.872-.892.08.08 0 01-.008-.128c.125-.094.25-.192.372-.292a.07.07 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.07.07 0 01.078.01c.12.1.246.198.373.292a.08.08 0 01-.006.127c-.598.35-1.22.645-1.873.892a.08.08 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.08.08 0 00.084.028 19.8 19.8 0 006.002-3.03.08.08 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.06.06 0 00-.031-.03z"/></svg><span>Discord Server</span><ExternalLink size={12} /></a>
                  <a href="https://github.com/cockpitflow" target="_blank" class="link-btn"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.6 9.6 0 0112 6.844a9.6 9.6 0 012.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg><span>GitHub</span><ExternalLink size={12} /></a>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">WEBHOOK</div>
                  <div class="prof-field"><label>Discord Webhook URL</label><input class="prof-input" placeholder="https://discord.com/api/webhooks/..." disabled /></div>
                  <p class="connect-hint">Paste a Discord channel webhook URL to auto-post your landings and achievements.</p>
                </div>
              </div>

            {:else if communityTab === 'share'}
              <div class="efb-2col">
                <div class="efb-section">
                  <div class="efb-heading">SHARE CARD PREVIEW</div>
                  <div class="share-card">
                    <div class="share-header">
                      <span class="share-logo">COCKPITFLOW</span>
                      <span class="share-cs">{pilotCallsign}</span>
                    </div>
                    <div class="share-body">
                      <div class="share-stat"><span class="share-stat-val">{allLandings.length}</span><span class="share-stat-lbl">LANDINGS</span></div>
                      <div class="share-stat"><span class="share-stat-val" style="color:var(--color-green)">{allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'}</span><span class="share-stat-lbl">BEST FPM</span></div>
                      <div class="share-stat"><span class="share-stat-val">{allLandings.length ? (allLandings.reduce((s,l) => s + Math.abs(l.rate), 0) / allLandings.length).toFixed(0) : '—'}</span><span class="share-stat-lbl">AVG FPM</span></div>
                      <div class="share-stat"><span class="share-stat-val" style="color:var(--color-green)">{allLandings.length ? Math.round(allLandings.filter(l => Math.abs(l.rate) < 60).length / allLandings.length * 100) : 0}%</span><span class="share-stat-lbl">BUTTER</span></div>
                    </div>
                    <div class="share-footer">{pilotName} - CockpitFlow</div>
                  </div>
                </div>
                <div class="efb-section">
                  <div class="efb-heading">EXPORT</div>
                  <button class="btn-action" onclick={() => { navigator.clipboard.writeText(`CockpitFlow Stats | ${pilotCallsign}\nLandings: ${allLandings.length}\nBest: ${allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'} fpm\nButter rate: ${allLandings.length ? Math.round(allLandings.filter(l => Math.abs(l.rate) < 60).length / allLandings.length * 100) : 0}%`); }}>COPY STATS TO CLIPBOARD</button>
                  <button class="btn-action" style="margin-top:6px;background:none;border:1px solid var(--color-border);color:var(--color-fg)" onclick={() => { const csv = 'Rate,Speed,Time,Date\n' + allLandings.map(l => `${l.rate},${l.speed},${l.time},${l.date}`).join('\n'); const blob = new Blob([csv],{type:'text/csv'}); const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'cockpitflow-landings.csv'; a.click(); }}>EXPORT LANDINGS CSV</button>
                </div>
              </div>
            {/if}
          </div>
        </div>

      {:else if active === 'marketplace'}
        <Marketplace />

      {:else if active === '_modules'}
        <ModuleManager {lanIp} onRefresh={loadDiskModules} />

      {:else if active === 'settings'}
        <div class="card" style="max-width:420px"><div class="label">SETTINGS</div><p class="desc">Global app settings — coming soon</p></div>
      {/if}
    </main>

    <footer class="statusbar">
      <div class="statusbar-l"><span class="dot" class:on={connected}></span><span>{connected ? 'ONLINE' : 'OFFLINE'}</span></div>
      <span>{utcTime} UTC</span>
    </footer>
  </div>
</div>

<style>
  /* LAYOUT */
  /* Update banner */
  .update-banner {
    position: fixed; top: 0; left: 0; right: 0; z-index: 999;
    display: flex; align-items: center; gap: 10px;
    padding: 8px 16px; background: rgba(74,158,255,.1);
    border-bottom: 1px solid rgba(74,158,255,.2);
    font-size: 12px; color: var(--color-fg);
  }
  .update-banner strong { color: var(--color-accent); }
  .update-btn {
    padding: 4px 12px; background: var(--color-accent); color: #fff;
    border-radius: 4px; font-size: 11px; font-weight: 600;
    text-decoration: none;
  }
  .update-dismiss {
    margin-left: auto; background: none; border: none;
    color: var(--color-dim); font-size: 16px; cursor: pointer;
    padding: 2px 6px;
  }
  .update-dismiss:hover { color: var(--color-fg); }

  .shell { display: flex; width: 100%; height: 100%; overflow: hidden; }
  .main { flex: 1; display: flex; flex-direction: column; min-width: 0; overflow: hidden; }
  .view { flex: 1; overflow: hidden; padding: 8px; display: flex; }
  .flex-1 { flex: 1; min-width: 0; }

  /* SIDEBAR */
  .sidebar { width: 200px; background: var(--color-surface); border-right: 1px solid var(--color-border); display: flex; flex-direction: column; transition: width .15s; overflow: hidden; flex-shrink: 0; }
  .sidebar.collapsed { width: 48px; }
  .sb-brand { display: flex; align-items: center; justify-content: center; padding: 4px 12px 2px; border-bottom: none; flex-shrink: 0; }
  .sb-logo { width: 130px; height: 130px; flex-shrink: 0; border-radius: 16px; }
  .sb-title { font-family: 'Cascadia Code', monospace; font-size: 11px; font-weight: 800; letter-spacing: 2.5px; color: var(--color-accent); white-space: nowrap; }
  .sb-nav { flex: 1; display: flex; flex-direction: column; gap: 1px; padding: 6px; overflow-y: auto; overflow-x: hidden; }
  .sb-item { display: flex; align-items: center; gap: 8px; padding: 6px 8px; border: none; background: none; color: var(--color-dim); border-radius: 4px; cursor: pointer; font-size: 12px; font-family: inherit; white-space: nowrap; transition: all .1s; }
  .sb-item:hover { color: var(--color-fg); background: var(--color-surface-2); }
  .sb-item.active { color: var(--color-accent); background: rgba(74,158,255,.08); }
  .sb-foot { padding: 6px; border-top: 1px solid var(--color-border); flex-shrink: 0; }
  .sb-status { display: flex; flex-direction: column; gap: 3px; padding: 6px 8px; }
  .sb-dot-row { display: flex; align-items: center; gap: 6px; font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }

  /* DOT */
  .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-red); flex-shrink: 0; }
  .dot.on { background: var(--color-green); box-shadow: 0 0 4px var(--color-green); }

  /* TOPBAR */
  .topbar { display: flex; align-items: center; justify-content: space-between; padding: 0 12px; height: 32px; background: var(--color-surface); border-bottom: 1px solid var(--color-border); flex-shrink: 0; }
  .topbar-l, .topbar-r { display: flex; align-items: center; gap: 8px; }
  .topbar-title { font-size: 12px; font-weight: 600; color: var(--color-bright); }
  .icon-btn { background: none; border: none; color: var(--color-dim); cursor: pointer; padding: 2px; display: flex; }
  .icon-btn:hover { color: var(--color-fg); }
  .mono-sm { font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-dim); }

  /* BADGES */
  .badge { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; letter-spacing: .5px; padding: 2px 8px; border-radius: 3px; }
  .badge-green { background: rgba(52,208,88,.1); color: var(--color-green); border: 1px solid rgba(52,208,88,.2); }
  .badge-red { background: rgba(234,74,90,.1); color: var(--color-red); }
  .badge-yellow { background: rgba(227,179,65,.1); color: var(--color-yellow); }

  /* STATUSBAR */
  .statusbar { display: flex; align-items: center; justify-content: space-between; padding: 0 12px; height: 22px; background: var(--color-surface); border-top: 1px solid var(--color-border); flex-shrink: 0; font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }
  .statusbar-l { display: flex; align-items: center; gap: 6px; }

  /* VIEW LAYOUTS */
  .v-row { display: flex; gap: 8px; width: 100%; height: 100%; overflow: hidden; }
  .v-scroll { width: 100%; height: 100%; overflow-y: auto; padding: 2px; }
  .v-center { width: 100%; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--color-dim); text-align: center; }

  /* CARD */
  .card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 5px; padding: 14px; overflow-y: auto; }
  .card-sm { padding: 12px; }
  .card-top { display: flex; justify-content: flex-end; margin-bottom: 4px; }

  /* LABEL */
  .label { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; letter-spacing: 1.5px; color: var(--color-dim); margin-bottom: 10px; }
  .sep { height: 1px; background: var(--color-border); margin: 8px 0; }
  .desc { font-size: 11px; color: var(--color-dim); line-height: 1.4; margin: 4px 0; }
  .spacer { flex: 1; }
  .disclaimer { font-size: 9px; color: var(--color-dim); opacity: .6; line-height: 1.3; }
  .version { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); margin-top: 4px; }

  /* HOME */
  .home-brand { display: flex; align-items: center; gap: 10px; }
  .home-logo { width: 40px; height: 40px; border-radius: 8px; }
  .home-title { font-family: 'Cascadia Code', monospace; font-size: 16px; font-weight: 900; letter-spacing: 3px; color: var(--color-accent); line-height: 1; }
  .home-sub { font-family: 'Cascadia Code', monospace; font-size: 8px; letter-spacing: 1px; color: var(--color-dim); margin-top: 2px; }
  .status-list { display: flex; flex-direction: column; gap: 5px; }
  .status-row { display: flex; align-items: center; gap: 6px; font-size: 11px; }

  /* MODULE GRID */
  .mod-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 6px; }
  .mod-btn { display: flex; align-items: center; gap: 10px; padding: 10px 14px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; color: var(--color-dim); cursor: pointer; font-size: 12px; font-family: inherit; font-weight: 500; transition: all .1s; text-align: left; }
  .mod-btn:hover { border-color: var(--color-accent); color: var(--color-accent); background: rgba(74,158,255,.05); }

  /* LINK BUTTON */
  .link-btn { display: flex; align-items: center; gap: 8px; padding: 8px 12px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; color: var(--color-fg); text-decoration: none; font-size: 12px; cursor: pointer; transition: border-color .1s; margin-bottom: 6px; }
  .link-btn span { flex: 1; }
  .link-btn:hover { border-color: var(--color-border); color: var(--color-bright); }
  .link-btn.discord:hover { border-color: #5865F2; color: #5865F2; }

  /* SCENARIOS GRID */
  .grid-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; }

  /* BUTTON ACCENT */
  .btn-accent { width: 100%; padding: 5px; margin-top: 8px; background: rgba(74,158,255,.08); border: 1px solid rgba(74,158,255,.2); color: var(--color-accent); border-radius: 3px; cursor: pointer; font-family: 'Cascadia Code', monospace; font-size: 10px; font-weight: 700; letter-spacing: 1px; transition: all .1s; }
  .btn-accent:hover { background: var(--color-accent); color: #fff; }

  /* METRIC */
  .metric { flex: 1; text-align: center; padding: 16px 10px; }
  .metric-label { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 600; letter-spacing: 1.5px; color: var(--color-dim); }
  .metric-val { font-family: 'Cascadia Code', monospace; font-size: 24px; font-weight: 800; color: var(--color-bright); line-height: 1.2; margin-top: 4px; }
  .metric-unit { font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-dim); }

  /* NAV FORM */
  .form-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 6px; font-size: 12px; }
  .input-wrap { display: flex; }
  .input-wrap input { width: 64px; height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); border-right: none; color: var(--color-accent); padding: 0 6px; border-radius: 3px 0 0 3px; font-family: 'Cascadia Code', monospace; font-size: 11px; text-align: right; }
  .input-wrap input:focus { outline: none; border-color: var(--color-accent); }
  .input-unit { height: 24px; padding: 0 6px; display: flex; align-items: center; background: var(--color-surface-2); border: 1px solid var(--color-border); border-radius: 0 3px 3px 0; font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }
  .wb-result { margin-top: 10px; padding: 8px; border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 11px; }
  .wb-ok { background: rgba(52,208,88,.06); border: 1px solid rgba(52,208,88,.2); color: var(--color-green); }
  .wb-fail { background: rgba(234,74,90,.06); border: 1px solid rgba(234,74,90,.2); color: var(--color-red); }
  .wb-line { display: flex; justify-content: space-between; margin-bottom: 2px; }
  .wb-verdict { text-align: center; font-weight: 800; letter-spacing: 2px; font-size: 10px; margin-top: 4px; }
  .fuel-row { display: grid; grid-template-columns: repeat(3,1fr); gap: 6px; }
  .fuel-cell { background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 3px; padding: 8px; text-align: center; }
  .fuel-lbl { font-family: 'Cascadia Code', monospace; font-size: 8px; letter-spacing: 1px; color: var(--color-dim); }
  .fuel-val { font-family: 'Cascadia Code', monospace; font-size: 18px; font-weight: 800; color: var(--color-accent); }
  .fuel-unit { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }

  /* EFB NAV PANEL */
  .efb-panel { width: 100%; height: 100%; display: flex; flex-direction: column; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 5px; overflow: hidden; }
  .efb-tabs { display: flex; border-bottom: 1px solid var(--color-border); flex-shrink: 0; }
  .efb-tab { padding: 7px 16px; border: none; background: none; color: var(--color-dim); font-family: 'Cascadia Code', monospace; font-size: 10px; font-weight: 600; letter-spacing: .5px; cursor: pointer; border-bottom: 2px solid transparent; transition: all .1s; }
  .efb-tab:hover { color: var(--color-fg); }
  .efb-tab-on { color: var(--color-accent); border-bottom-color: var(--color-accent); }
  .efb-body { flex: 1; overflow-y: auto; padding: 14px; }
  .efb-2col { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
  .efb-section { }
  .efb-heading { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; letter-spacing: 1.5px; color: var(--color-dim); margin-bottom: 10px; padding-bottom: 4px; border-bottom: 1px solid var(--color-border); }
  .efb-form { display: flex; flex-direction: column; gap: 6px; }
  .efb-field { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .efb-field label { font-size: 12px; color: var(--color-fg); }
  .efb-input { width: 64px; height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-accent); padding: 0 6px; border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 11px; text-align: right; }
  .efb-input:focus { outline: none; border-color: var(--color-accent); }
  .efb-tbl { width: 100%; border-collapse: collapse; font-size: 11px; }
  .efb-tbl th { text-align: left; font-family: 'Cascadia Code', monospace; font-size: 9px; letter-spacing: 1px; color: var(--color-dim); padding: 4px 8px; border-bottom: 1px solid var(--color-border); font-weight: 600; }
  .efb-tbl td { padding: 5px 8px; border-bottom: 1px solid rgba(30,39,54,.5); }
  .efb-tbl tr:hover td { background: var(--color-surface-2); }
  .efb-tbl .mono { font-family: 'Cascadia Code', monospace; color: var(--color-bright); }
  .efb-tbl input { width: 56px; height: 22px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-accent); padding: 0 4px; border-radius: 2px; font-family: 'Cascadia Code', monospace; font-size: 11px; text-align: right; }
  .efb-tbl input:focus { outline: none; border-color: var(--color-accent); }
  .efb-total td { border-top: 2px solid var(--color-border); font-weight: 700; color: var(--color-bright); }
  .efb-readout-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .efb-rg-2x2 { grid-template-columns: 1fr 1fr; }
  .efb-readout { background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 3px; padding: 8px 10px; display: flex; flex-direction: column; }
  .efb-readout-accent { border-color: rgba(74,158,255,.25); background: rgba(74,158,255,.04); }
  .efb-ro-label { font-family: 'Cascadia Code', monospace; font-size: 8px; letter-spacing: 1.5px; color: var(--color-dim); }
  .efb-ro-val { font-family: 'Cascadia Code', monospace; font-size: 18px; font-weight: 800; color: var(--color-bright); line-height: 1.2; }
  .efb-ro-val small { font-size: 10px; font-weight: 500; color: var(--color-dim); }
  .efb-ro-warn { color: var(--color-yellow); }
  .efb-ro-limit { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }
  .efb-verdict { margin-top: 10px; padding: 6px 12px; border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 11px; font-weight: 800; letter-spacing: 2px; text-align: center; }
  .efb-ok { background: rgba(52,208,88,.08); border: 1px solid rgba(52,208,88,.2); color: var(--color-green); }
  .efb-warn { background: rgba(234,74,90,.08); border: 1px solid rgba(234,74,90,.2); color: var(--color-red); }
  .efb-warn-msg { margin-top: 8px; padding: 5px 10px; border-radius: 3px; background: rgba(227,179,65,.06); border: 1px solid rgba(227,179,65,.15); color: var(--color-yellow); font-size: 11px; }
  .efb-speed-val { font-family: 'Cascadia Code', monospace; font-weight: 700; color: var(--color-accent); font-size: 13px; }
  .efb-speed-desc { color: var(--color-dim); font-size: 11px; }
  .efb-note { margin-top: 10px; font-size: 10px; color: var(--color-dim); font-style: italic; }
  .warn-val { color: var(--color-yellow); }

  /* DEBRIEF */
  .efb-tab-spacer { flex: 1; }
  .efb-flight-indicator { display: flex; align-items: center; gap: 5px; padding: 0 12px; font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); letter-spacing: .5px; }
  .efb-fi-active { color: var(--color-green); }
  .debrief-big-readout { display: flex; align-items: baseline; gap: 6px; padding: 12px 0; }
  .dbr-value { font-family: 'Cascadia Code', monospace; font-size: 48px; font-weight: 900; line-height: 1; }
  .dbr-unit { font-family: 'Cascadia Code', monospace; font-size: 14px; color: var(--color-dim); }
  .dbr-grade { font-family: 'Cascadia Code', monospace; font-size: 28px; font-weight: 900; margin-left: auto; }
  .dbr-scale { position: relative; margin-top: 8px; }
  .dbr-bar { display: flex; height: 6px; border-radius: 3px; overflow: hidden; }
  .dbr-zone { font-size: 0; }
  .dbr-green { background: var(--color-green); }
  .dbr-blue { background: var(--color-accent); }
  .dbr-yellow { background: var(--color-yellow); }
  .dbr-orange { background: #e07020; }
  .dbr-red { background: var(--color-red); }
  .dbr-marker { position: absolute; top: -2px; width: 2px; height: 10px; background: var(--color-bright); border-radius: 1px; transform: translateX(-50%); }
  .dbr-grade-sm { font-family: 'Cascadia Code', monospace; font-size: 13px; font-weight: 800; }
  .efb-empty { color: var(--color-dim); font-size: 12px; padding: 16px 0; }
  .btn-reset { margin-top: 12px; padding: 6px 16px; background: none; border: 1px solid var(--color-border); color: var(--color-dim); border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 10px; letter-spacing: 1px; cursor: pointer; transition: all .1s; }
  .btn-reset:hover { border-color: var(--color-red); color: var(--color-red); }

  /* LANDING */
  /* CAPTURE */
  .capture-layout { display: flex; height: 100%; overflow: hidden; }
  .capture-sidebar { width: 220px; flex-shrink: 0; padding: 10px; border-right: 1px solid var(--color-border); overflow-y: auto; display: flex; flex-direction: column; gap: 6px; }
  .capture-view { flex: 1; display: flex; align-items: center; justify-content: center; background: #000; overflow: hidden; position: relative; }
  .capture-img { max-width: 100%; max-height: 100%; object-fit: contain; }
  .capture-empty { display: flex; flex-direction: column; align-items: center; gap: 6px; color: var(--color-dim); }
  .cap-win-btn { display: flex; align-items: center; gap: 6px; padding: 6px 8px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 3px; color: var(--color-fg); cursor: pointer; font-size: 11px; text-align: left; width: 100%; transition: border-color .1s; }
  .cap-win-btn:hover { border-color: var(--color-accent); }
  .cap-win-active { border-color: var(--color-green); background: rgba(52,208,88,.04); }
  .cap-win-title { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 10px; }
  .cap-separator { height: 1px; background: var(--color-border); margin: 6px 0; }
  .cap-region-form { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  .cap-region-row { display: flex; align-items: center; gap: 4px; font-size: 10px; color: var(--color-dim); }
  .cap-region-row .efb-input { width: 100%; }
  .capture-container { position: relative; display: inline-block; max-width: 100%; max-height: 100%; }
  .capture-overlay { position: absolute; inset: 0; pointer-events: none; }
  .g430-btn { position: absolute; pointer-events: all; border-radius: 2px; color: #fff; cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all .1s; padding: 0; }
  .g430-btn span { font-family: 'Cascadia Code', monospace; font-size: 7px; font-weight: 700; letter-spacing: .5px; text-shadow: 0 0 4px rgba(0,0,0,.9); }
  /* Button styles */
  .g430-btn[data-style="btn"] { background: rgba(80,80,80,0.3); border: 1px solid rgba(150,150,150,0.3); }
  .g430-btn[data-style="btn"]:hover { background: rgba(120,120,120,0.5); }
  .g430-btn[data-style="btn"]:active { background: rgba(180,180,180,0.6); }
  .g430-btn[data-style="btn-accent"] { background: rgba(74,158,255,0.25); border: 1px solid rgba(74,158,255,0.5); }
  .g430-btn[data-style="btn-accent"]:hover { background: rgba(74,158,255,0.5); }
  .g430-btn[data-style="btn-accent"]:active { background: rgba(74,158,255,0.8); }
  .g430-btn[data-style="btn-flip"] { background: rgba(52,208,88,0.15); border: 1px solid rgba(52,208,88,0.3); }
  .g430-btn[data-style="btn-flip"]:hover { background: rgba(52,208,88,0.4); }
  .g430-btn[data-style="knob"] { background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.15); border-radius: 50%; }
  .g430-btn[data-style="knob"]:hover { background: rgba(255,255,255,0.2); }
  .g430-btn[data-style="knob"]:active { background: rgba(255,255,255,0.35); }
  .g430-btn[data-style="knob"] span { font-size: 10px; }
  .cap-controls { display: flex; flex-direction: column; gap: 6px; padding: 8px 0; border-top: 1px solid var(--color-border); margin-top: 8px; }
  .cap-ctrl-row { display: flex; align-items: center; justify-content: space-between; font-size: 11px; }
  .cap-range { width: 80px; accent-color: var(--color-accent); }
  .capture-overlay { transform-origin: top left; }
  .cap-adjust-grid { display: flex; flex-direction: column; gap: 3px; }
  .cap-adj-row { display: flex; align-items: center; gap: 4px; font-size: 10px; color: var(--color-dim); }
  .cap-adj-row .efb-input { width: 56px; }

  /* FLIGHT MODULE */
  .fl-route-row { display: flex; align-items: flex-end; gap: 12px; flex-wrap: wrap; }
  .fl-airport { display: flex; flex-direction: column; gap: 2px; }
  .fl-airport label { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; letter-spacing: 1px; color: var(--color-dim); }
  .fl-icao { width: 72px; height: 28px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-accent); padding: 0 6px; border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 14px; font-weight: 800; text-transform: uppercase; letter-spacing: 1px; }
  .fl-icao:focus { outline: none; border-color: var(--color-accent); }
  .fl-arrow { font-family: 'Cascadia Code', monospace; font-size: 12px; color: var(--color-dim); padding-bottom: 6px; }
  .fl-wp-row { display: flex; align-items: center; gap: 6px; padding: 4px 0; border-bottom: 1px solid rgba(30,39,54,.3); }
  .fl-wp-num { font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-dim); width: 18px; text-align: center; }
  .fl-wp-input { width: 80px; height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 6px; border-radius: 3px; font-size: 11px; font-family: 'Cascadia Code', monospace; text-transform: uppercase; }
  .fl-wp-input:focus { outline: none; border-color: var(--color-accent); }
  .fl-wp-freq { width: 64px; height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 4px; border-radius: 3px; font-size: 10px; font-family: 'Cascadia Code', monospace; }
  .fl-wp-freq:focus { outline: none; border-color: var(--color-accent); }
  .fl-wp-fail { height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-dim); padding: 0 3px; border-radius: 3px; font-size: 9px; flex: 1; min-width: 80px; }
  .fl-route-svg { width: 100%; border: 1px solid var(--color-border); border-radius: 4px; }
  .fl-rec-btns { display: flex; gap: 6px; margin-top: 10px; }
  .fl-rec-btns .btn-action { flex: 1; }
  .fl-event-row { display: flex; align-items: center; gap: 4px; padding: 4px 0; border-bottom: 1px solid rgba(30,39,54,.3); flex-wrap: wrap; }
  .fl-ev-fail { height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 3px; border-radius: 3px; font-size: 10px; min-width: 120px; }
  .fl-ev-trigger { height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-dim); padding: 0 3px; border-radius: 3px; font-size: 10px; min-width: 80px; }

  /* FAILURE GENERATOR */
  .gen-timing { display: flex; flex-direction: column; gap: 4px; margin-bottom: 10px; }
  .gen-delay-row { display: flex; align-items: center; gap: 8px; font-size: 12px; margin-top: 6px; }
  .gen-fail-list { overflow-y: auto; max-height: calc(100vh - 280px); }
  .gen-fail-row { display: flex; align-items: center; gap: 6px; padding: 4px 6px; border-bottom: 1px solid rgba(30,39,54,.3); font-size: 11px; }
  .gen-fail-row:hover { background: var(--color-surface-2); }
  .gen-armed { opacity: .5; }
  .gen-fail-name { font-weight: 600; color: var(--color-fg); width: 140px; flex-shrink: 0; }
  .gen-fail-desc { color: var(--color-dim); font-size: 10px; flex: 1; min-width: 0; }
  .gen-armed-tag { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; color: var(--color-yellow); letter-spacing: 1px; }
  .gen-sev-btns { display: flex; gap: 2px; flex-shrink: 0; }
  .gen-sev-btn { padding: 2px 8px; border: 1px solid var(--color-border); background: none; color: var(--color-dim); border-radius: 2px; font-size: 9px; font-family: 'Cascadia Code', monospace; cursor: pointer; transition: all .1s; }
  .gen-sev-btn:hover { border-color: var(--color-red); color: var(--color-red); background: rgba(234,74,90,.06); }

  /* ACTIVE FAILURES */
  .af-list { display: flex; flex-direction: column; gap: 4px; }
  .af-card { display: flex; align-items: center; gap: 10px; padding: 8px 10px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; }
  .af-fired { border-color: var(--color-red); background: rgba(234,74,90,.04); }
  .af-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .af-info { flex: 1; min-width: 0; }
  .af-name { font-size: 12px; font-weight: 600; color: var(--color-fg); display: block; }
  .af-meta { font-size: 9px; color: var(--color-dim); font-family: 'Cascadia Code', monospace; }
  .af-countdown { font-family: 'Cascadia Code', monospace; font-size: 11px; font-weight: 700; color: var(--color-yellow); flex-shrink: 0; }
  .af-live { color: var(--color-red); animation: pulse-red 1s infinite; }
  @keyframes pulse-red { 0%,100% { opacity: 1; } 50% { opacity: .5; } }
  .af-remove { width: 20px; height: 20px; border: 1px solid var(--color-border); background: none; color: var(--color-dim); border-radius: 3px; cursor: pointer; font-size: 11px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
  .af-remove:hover { border-color: var(--color-red); color: var(--color-red); }

  .landing-layout { display: flex; gap: 16px; height: 100%; overflow: hidden; }
  .landing-gauge-area { width: 280px; flex-shrink: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px; }
  .landing-gauge-wrap { width: 220px; }
  .landing-gauge-svg { width: 100%; height: auto; }
  .landing-readout { display: flex; align-items: baseline; gap: 4px; margin-top: -8px; }
  .landing-rate { font-family: 'Cascadia Code', monospace; font-size: 42px; font-weight: 900; line-height: 1; }
  .landing-fpm { font-family: 'Cascadia Code', monospace; font-size: 12px; color: var(--color-dim); }
  .landing-verdict { font-family: 'Cascadia Code', monospace; font-size: 14px; font-weight: 800; letter-spacing: 3px; margin-top: 2px; }
  .landing-grade-big { font-family: 'Cascadia Code', monospace; font-size: 36px; font-weight: 900; margin-top: 4px; }

  .landing-history { flex: 1; min-width: 0; display: flex; flex-direction: column; overflow: hidden; }
  .landing-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px; color: var(--color-dim); }
  .landing-empty-icon { font-family: 'Cascadia Code', monospace; font-size: 28px; color: var(--color-border); }
  .landing-empty-hint { font-size: 10px; color: var(--color-border); }

  .landing-td-list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 4px; padding: 2px 0; }
  .landing-td-card { display: flex; align-items: center; gap: 8px; padding: 8px 10px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; position: relative; overflow: hidden; }
  .landing-td-num { font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-dim); width: 20px; flex-shrink: 0; }
  .landing-td-info { flex: 1; min-width: 0; }
  .landing-td-rate { font-family: 'Cascadia Code', monospace; font-size: 14px; font-weight: 800; }
  .landing-td-meta { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }
  .landing-td-grade { font-family: 'Cascadia Code', monospace; font-size: 18px; font-weight: 900; width: 30px; text-align: center; flex-shrink: 0; }
  .landing-td-bar { position: absolute; bottom: 0; left: 0; right: 0; height: 2px; background: var(--color-border); }
  .landing-td-fill { height: 100%; border-radius: 1px; transition: width .3s; }

  .landing-stats { display: flex; gap: 8px; padding-top: 8px; border-top: 1px solid var(--color-border); flex-shrink: 0; margin-top: 4px; }
  .landing-stat { flex: 1; text-align: center; font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-dim); }
  .landing-stat strong { display: block; font-size: 13px; color: var(--color-fg); margin-top: 1px; }

  /* CHECKLIST */
  .checklist-wrap { width: 100%; height: 100%; overflow: hidden; }
  .cl-settings { padding: 14px; overflow-y: auto; height: 100%; }
  .cl-preset-row { display: flex; gap: 8px; align-items: center; margin-bottom: 4px; }
  .cl-preset-select {
    padding: 6px 10px; background: var(--color-bg); border: 1px solid var(--color-border);
    border-radius: 4px; color: var(--color-fg); font-family: 'Cascadia Code', monospace;
    font-size: 12px; outline: none; min-width: 180px;
  }
  .cl-preset-select:focus { border-color: var(--color-accent); }
  .cl-option-group { display: flex; flex-direction: column; gap: 6px; }
  .cl-opt { display: flex; flex-direction: column; gap: 2px; padding: 10px 12px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; cursor: pointer; text-align: left; color: var(--color-dim); font-family: inherit; font-size: 11px; transition: all .1s; }
  .cl-opt strong { font-size: 12px; color: var(--color-fg); }
  .cl-opt:hover { border-color: var(--color-accent); }
  .cl-opt-on { border-color: var(--color-accent); background: rgba(74,158,255,.06); }
  .cl-opt-on strong { color: var(--color-accent); }
  .cl-toggle-row { display: flex; align-items: center; justify-content: space-between; padding: 4px 0; font-size: 12px; }
  .cl-toggle-group { display: flex; border: 1px solid var(--color-border); border-radius: 3px; overflow: hidden; }
  .cl-tgl { padding: 3px 12px; border: none; background: none; color: var(--color-dim); font-family: 'Cascadia Code', monospace; font-size: 10px; font-weight: 700; cursor: pointer; transition: all .1s; }
  .cl-tgl-on { background: var(--color-accent); color: #fff; }
  .cl-hint { font-size: 10px; color: var(--color-dim); opacity: .6; margin: 0 0 8px; }
  .btn-action { padding: 8px 16px; background: var(--color-accent); border: none; color: #fff; border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 11px; font-weight: 700; letter-spacing: .5px; cursor: pointer; transition: opacity .1s; }
  .btn-action:hover { opacity: .85; }
  .cl-info-grid { display: flex; flex-direction: column; gap: 4px; }
  .cl-info-item { display: flex; justify-content: space-between; font-size: 11px; padding: 3px 0; border-bottom: 1px solid rgba(30,39,54,.4); }
  .cl-info-label { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); letter-spacing: 1px; }

  /* HARDWARE */
  .hw-row { display: flex; align-items: center; gap: 8px; padding: 4px 0; border-bottom: 1px solid rgba(30,39,54,.3); }
  .hw-row-active { background: rgba(52,208,88,.03); }
  .hw-row-hl { background: rgba(74,158,255,.1) !important; border-color: var(--color-accent) !important; }
  .hw-dot { width: 8px; height: 8px; border-radius: 50%; background: #2a3040; border: 1px solid #3a4555; flex-shrink: 0; transition: all .2s; }
  .hw-dot-mapped { background: var(--color-yellow); border-color: var(--color-yellow); box-shadow: 0 0 3px rgba(227,179,65,.3); }
  .hw-dot-active { background: var(--color-green); border-color: var(--color-green); box-shadow: 0 0 6px var(--color-green); }
  .hw-pin { font-family: 'Cascadia Code', monospace; font-size: 11px; font-weight: 700; color: var(--color-accent); width: 28px; flex-shrink: 0; }
  .hw-select { flex: 1; height: 24px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 4px; border-radius: 3px; font-size: 11px; font-family: inherit; cursor: pointer; }
  .hw-select:focus { outline: none; border-color: var(--color-accent); }
  .hw-live-tag { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 800; color: var(--color-green); letter-spacing: 1px; flex-shrink: 0; }
  .hw-map-scroll { overflow-y: auto; height: 100%; }
  .hw-device-section { margin-bottom: 16px; }
  .hw-dev-banner { display: flex; align-items: center; gap: 8px; padding: 6px 8px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; margin-bottom: 8px; }
  .hw-dev-banner-name { font-size: 12px; font-weight: 700; color: var(--color-bright); }
  .hw-dev-banner-port { font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-dim); margin-left: auto; }
  .hw-section-label { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; letter-spacing: 1px; color: var(--color-dim); padding: 4px 0 2px; }
  .hw-combined { display: flex; gap: 10px; height: 100%; overflow: hidden; }
  .hw-pins-col { width: 340px; flex-shrink: 0; overflow-y: auto; }
  .hw-fns-col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; overflow: hidden; }
  .hw-fn-list { flex: 1; overflow-y: auto; }
  .hw-fn-summary { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); padding: 4px 0; flex-shrink: 0; }

  .hw-monitor-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 6px; }
  .hw-mon-cell { display: flex; align-items: center; gap: 8px; padding: 8px 10px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; transition: all .15s; }
  .hw-mon-active { border-color: var(--color-green); background: rgba(52,208,88,.04); }
  .hw-mon-dot { width: 10px; height: 10px; border-radius: 50%; background: var(--color-dim); flex-shrink: 0; transition: all .2s; }
  .hw-mon-dot.hw-dot-active { background: var(--color-green); box-shadow: 0 0 8px var(--color-green); }
  .hw-mon-info { flex: 1; display: flex; flex-direction: column; }
  .hw-mon-fn { font-size: 12px; font-weight: 600; color: var(--color-fg); }
  .hw-mon-pin { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); }
  .hw-mon-val { font-family: 'Cascadia Code', monospace; font-size: 11px; font-weight: 800; color: var(--color-dim); }
  .hw-mon-active .hw-mon-val { color: var(--color-green); }
  .efb-empty { color: var(--color-dim); font-size: 12px; padding: 20px 0; }

  /* LEADERBOARD */
  .lb-list { display: flex; flex-direction: column; gap: 2px; }
  .lb-row { display: flex; align-items: center; gap: 8px; padding: 5px 8px; border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 11px; }
  .lb-row:hover { background: var(--color-surface-2); }
  .lb-top3 { background: rgba(74,158,255,.03); }
  .lb-rank { width: 22px; text-align: center; font-weight: 800; color: var(--color-dim); font-size: 12px; }
  .lb-gold { color: #ffd700; }
  .lb-silver { color: #c0c0c0; }
  .lb-bronze { color: #cd7f32; }
  .lb-rate { font-weight: 800; font-size: 13px; width: 70px; }
  .lb-grade { font-weight: 900; font-size: 14px; width: 28px; text-align: center; }
  .lb-speed { color: var(--color-dim); font-size: 10px; width: 40px; }
  .lb-date { color: var(--color-dim); font-size: 9px; margin-left: auto; }
  .lb-dist { display: flex; flex-direction: column; gap: 4px; }
  .lb-dist-row { display: flex; align-items: center; gap: 6px; }
  .lb-dist-grade { font-family: 'Cascadia Code', monospace; font-size: 12px; font-weight: 800; width: 22px; text-align: center; }
  .lb-dist-bar-bg { flex: 1; height: 6px; background: var(--color-border); border-radius: 3px; overflow: hidden; }
  .lb-dist-bar-fill { height: 100%; border-radius: 3px; transition: width .3s; }
  .lb-dist-count { font-family: 'Cascadia Code', monospace; font-size: 10px; color: var(--color-dim); width: 24px; text-align: right; }

  /* PROFILE */
  .prof-form { display: flex; flex-direction: column; gap: 8px; margin-bottom: 14px; }
  .prof-field { display: flex; flex-direction: column; gap: 3px; }
  .prof-field label { font-size: 10px; color: var(--color-dim); font-family: 'Cascadia Code', monospace; letter-spacing: .5px; }
  .prof-input { height: 28px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 8px; border-radius: 3px; font-size: 12px; }
  .prof-input:focus { outline: none; border-color: var(--color-accent); }
  .prof-input:disabled { opacity: .5; }
  .prof-card { background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 5px; padding: 14px; }
  .prof-card-header { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; }
  .prof-avatar { width: 36px; height: 36px; border-radius: 50%; background: var(--color-accent); color: #fff; display: flex; align-items: center; justify-content: center; font-family: 'Cascadia Code', monospace; font-size: 16px; font-weight: 800; flex-shrink: 0; }
  .prof-name { font-size: 14px; font-weight: 700; color: var(--color-bright); }
  .prof-cs { font-family: 'Cascadia Code', monospace; font-size: 11px; color: var(--color-accent); }
  .prof-stats { display: flex; gap: 12px; }
  .prof-stat { text-align: center; flex: 1; }
  .prof-stat span { display: block; font-family: 'Cascadia Code', monospace; font-size: 18px; font-weight: 800; color: var(--color-bright); }
  .prof-stat label { font-size: 9px; color: var(--color-dim); letter-spacing: .5px; }

  /* CONNECT */
  .connect-placeholder { display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 20px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 5px; text-align: center; }
  .connect-title { font-size: 14px; font-weight: 700; color: var(--color-bright); }
  .connect-desc { font-size: 11px; color: var(--color-dim); }
  .connect-btn { padding: 6px 20px; background: #5865F2; border: none; color: #fff; border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 10px; font-weight: 700; letter-spacing: 1px; cursor: not-allowed; opacity: .5; margin-top: 4px; }
  .connect-options { display: flex; flex-direction: column; gap: 4px; }
  .connect-opt { display: flex; align-items: center; gap: 8px; font-size: 11px; color: var(--color-dim); padding: 3px 0; cursor: default; }
  .connect-opt input { accent-color: var(--color-accent); }
  .connect-hint { font-size: 10px; color: var(--color-dim); margin-top: 6px; }

  /* SHARE CARD */
  .share-card { background: linear-gradient(135deg, #0f1520 0%, #141e2e 100%); border: 1px solid var(--color-border); border-radius: 8px; padding: 16px; }
  .share-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; }
  .share-logo { font-family: 'Cascadia Code', monospace; font-size: 14px; font-weight: 900; letter-spacing: 3px; color: var(--color-accent); }
  .share-cs { font-family: 'Cascadia Code', monospace; font-size: 12px; color: var(--color-dim); }
  .share-body { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-bottom: 12px; }
  .share-stat { text-align: center; }
  .share-stat-val { display: block; font-family: 'Cascadia Code', monospace; font-size: 20px; font-weight: 900; color: var(--color-bright); }
  .share-stat-lbl { font-family: 'Cascadia Code', monospace; font-size: 8px; letter-spacing: 1px; color: var(--color-dim); }
  .share-footer { font-size: 9px; color: var(--color-dim); text-align: center; border-top: 1px solid var(--color-border); padding-top: 8px; }

  /* HW FILTER */
  .hw-filter-bar { display: flex; align-items: center; gap: 8px; }
  .hw-search { flex: 1; height: 26px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 8px; border-radius: 3px; font-size: 11px; font-family: inherit; }
  .hw-search:focus { outline: none; border-color: var(--color-accent); }
  .hw-search::placeholder { color: var(--color-dim); }
  .hw-filter-btns { display: flex; border: 1px solid var(--color-border); border-radius: 3px; overflow: hidden; flex-shrink: 0; }
  .hw-group-pills { display: flex; gap: 4px; flex-wrap: wrap; margin-top: 6px; }
  .hw-pill { padding: 2px 10px; border: 1px solid var(--color-border); border-radius: 10px; background: none; color: var(--color-dim); font-size: 10px; cursor: pointer; transition: all .1s; font-family: inherit; }
  .hw-pill:hover { border-color: var(--color-accent); color: var(--color-fg); }
  .hw-pill-on { background: var(--color-accent); border-color: var(--color-accent); color: #fff; }

  /* HW ALL FUNCTIONS */
  .hw-group-label { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; letter-spacing: 1px; color: var(--color-dim); padding: 8px 0 3px; border-bottom: 1px solid var(--color-border); margin-top: 4px; }
  .hw-fn-row { display: flex; align-items: center; gap: 8px; padding: 3px 4px; border-bottom: 1px solid rgba(30,39,54,.25); font-size: 11px; color: var(--color-dim); }
  .hw-fn-mapped { color: var(--color-fg); }
  .hw-fn-dot { width: 7px; height: 7px; border-radius: 50%; background: #1e2736; border: 1px solid #2a3545; flex-shrink: 0; }
  .hw-fn-dot-on { background: var(--color-green); border-color: var(--color-green); box-shadow: 0 0 4px var(--color-green); }
  .hw-fn-name { flex: 1; }
  .hw-fn-pin { font-family: 'Cascadia Code', monospace; font-size: 10px; font-weight: 700; color: var(--color-accent); background: rgba(74,158,255,.08); padding: 1px 6px; border-radius: 2px; }
  .hw-fn-none { font-size: 10px; color: var(--color-border); }
  .hw-fn-dev { font-family: 'Cascadia Code', monospace; font-size: 9px; color: var(--color-dim); background: var(--color-surface-2); padding: 1px 5px; border-radius: 2px; }
  .hw-dev-sel { height: 22px; background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-dim); padding: 0 3px; border-radius: 2px; font-size: 10px; width: 80px; flex-shrink: 0; }

  /* Device cards */
  .hw-device-card { background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 4px; padding: 10px; margin-bottom: 8px; }
  .hw-dev-header { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
  .hw-dev-name-input { flex: 1; height: 24px; background: transparent; border: 1px solid var(--color-border); color: var(--color-bright); padding: 0 6px; border-radius: 3px; font-size: 12px; font-weight: 600; }
  .hw-dev-name-input:focus { outline: none; border-color: var(--color-accent); }
  .hw-dev-type-badge { font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 700; letter-spacing: .5px; padding: 2px 6px; border-radius: 2px; background: rgba(74,158,255,.08); color: var(--color-accent); flex-shrink: 0; }
  .hw-dev-remove { width: 20px; height: 20px; border: 1px solid var(--color-border); background: none; color: var(--color-dim); border-radius: 3px; cursor: pointer; font-size: 11px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
  .hw-dev-remove:hover { border-color: var(--color-red); color: var(--color-red); }
  .hw-dev-fields { display: flex; gap: 12px; }
  .hw-dev-field { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--color-dim); }
  .hw-dev-port-input { width: 56px; height: 22px; background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 4px; border-radius: 2px; font-family: 'Cascadia Code', monospace; font-size: 10px; }
  .hw-dev-port-input:focus { outline: none; border-color: var(--color-accent); }
  .hw-dev-type-sel { height: 22px; background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-fg); padding: 0 3px; border-radius: 2px; font-size: 10px; }
  .hw-add-dev { width: 100%; padding: 6px; background: none; border: 1px dashed var(--color-border); color: var(--color-dim); border-radius: 4px; font-size: 11px; cursor: pointer; transition: all .1s; }
  .hw-add-dev:hover { border-color: var(--color-accent); color: var(--color-accent); }

  /* LAN SETUP */
  .lan-setup { display: flex; gap: 24px; padding: 16px; height: 100%; }
  .lan-left { flex: 1; display: flex; flex-direction: column; gap: 12px; }
  .lan-right { flex-shrink: 0; display: flex; align-items: flex-start; }
  .lan-desc { font-size: 12px; color: var(--color-dim); line-height: 1.5; margin: 0; }
  .lan-url-row { display: flex; align-items: center; gap: 8px; }
  .lan-url { font-family: 'Cascadia Code', monospace; font-size: 16px; font-weight: 700; color: var(--color-accent); }
  .lan-copy { padding: 4px 12px; background: none; border: 1px solid var(--color-border); color: var(--color-dim); border-radius: 3px; font-family: 'Cascadia Code', monospace; font-size: 10px; font-weight: 700; letter-spacing: 1px; cursor: pointer; transition: all .1s; }
  .lan-copy:hover { border-color: var(--color-accent); color: var(--color-accent); }
  .lan-info { display: flex; flex-direction: column; gap: 4px; }
  .lan-info-row { display: flex; justify-content: space-between; font-size: 11px; padding: 3px 0; border-bottom: 1px solid rgba(30,39,54,.4); }
  .lan-info-row span { color: var(--color-dim); }
  .lan-info-row strong { color: var(--color-fg); font-weight: 600; }
  .lan-note { font-size: 10px; color: var(--color-dim); opacity: .7; line-height: 1.4; margin-top: auto; }
  .lan-qr-placeholder { width: 140px; height: 140px; border: 1px solid var(--color-border); border-radius: 5px; background: var(--color-bg); display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px; }
  .lan-qr-text { font-family: 'Cascadia Code', monospace; font-size: 24px; font-weight: 800; color: var(--color-dim); }
  .lan-qr-hint { font-size: 9px; color: var(--color-dim); }

  /* TABLE */
  .tbl { width: 100%; border-collapse: collapse; font-family: 'Cascadia Code', monospace; font-size: 11px; }
  .tbl th { text-align: left; font-size: 9px; letter-spacing: 1px; color: var(--color-dim); padding: 4px 8px; border-bottom: 1px solid var(--color-border); font-weight: 600; }
  .tbl td { padding: 4px 8px; border-bottom: 1px solid var(--color-border); }
  .tbl tr:hover td { background: var(--color-surface-2); }
  .td-accent { color: var(--color-accent); font-weight: 700; }
  .td-green { color: var(--color-green); font-weight: 600; }

  /* CONSOLE */
  .console-card { width: 100%; height: 100%; padding: 8px; }
  .console-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 0 10px; font-family: 'Cascadia Code', monospace; font-size: 11px; }
  .con-row { display: flex; justify-content: space-between; padding: 2px 6px; border-bottom: 1px solid rgba(30,39,54,.5); }
  .con-row:hover { background: var(--color-surface-2); }
  .con-key { color: var(--color-dim); }
  .con-val { color: var(--color-dim); }
  .con-live { color: var(--color-green); font-weight: 700; }
</style>
