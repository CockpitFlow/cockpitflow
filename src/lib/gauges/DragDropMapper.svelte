<script lang="ts">
  import { invoke } from '../mock';
  import { toast } from './Toast.svelte';
  import {
    ArrowRightLeft, RotateCw, RotateCcw, SlidersHorizontal,
    Monitor, Lightbulb, Gauge, Binary, ToggleLeft,
    FlipVertical, Save, X, ChevronDown
  } from 'lucide-svelte';

  let { aircraft, deviceProfile, mappingProfile, onSave }: {
    aircraft: any;
    deviceProfile?: any;
    mappingProfile?: any;
    onSave?: (mappings: any[]) => void;
  } = $props();

  // Current mappings being edited
  let mappings: any[] = $state(mappingProfile?.mappings ? [...mappingProfile.mappings] : []);
  let selectedPin: { pin: number | string; side: string } | null = $state(null);
  let filterCategory = $state('all');
  let searchQuery = $state('');
  let directionFilter = $state<'all' | 'input' | 'output'>('all');

  // Encoder dual-event state
  let encoderCwTarget: any = $state(null);
  let encoderCcwTarget: any = $state(null);
  let encoderStep: 'cw' | 'ccw' = $state('cw');

  // Analog calibration state (for the currently editing mapping)
  let calibrationPin: number | string | null = $state(null);
  let calMin = $state(0);
  let calMax = $state(100);
  let calDeadzone = $state(5);
  let calInvert = $state(false);
  let calCurve = $state<'linear' | 'logarithmic' | 'exponential'>('linear');

  // Output pin types — pins that receive data FROM the sim
  const OUTPUT_PIN_TYPES = ['led', 'servo', 'stepper', 'display', 'pwm', 'LED/SCK'];
  // Input pin types that could be encoders
  const ENCODER_CAPABLE_TYPES = ['digital', 'encoder', 'INT0', 'INT1'];
  // Analog pin types
  const ANALOG_PIN_TYPES = ['analog', 'potentiometer'];

  // Generate pin list from device profile or defaults
  let allPins = $derived.by(() => {
    if (deviceProfile?.pins?.length > 0) {
      return deviceProfile.pins.map((p: any) => p.pin ?? p.num ?? p.id);
    }
    // Default Arduino Nano pins
    const digital = Array.from({length: 14}, (_, i) => `D${i}`);
    const analog = Array.from({length: 8}, (_, i) => `A${i}`);
    return [...digital, ...analog];
  });

  function getPinInfo(pin: number | string): any {
    // Try to resolve pin info from device profile or board layout
    const pinStr = String(pin);
    const profile = deviceProfile?.pins?.find((p: any) => String(p.pin) === pinStr || String(p.num) === pinStr);
    if (profile) return profile;
    // Heuristic: A-prefixed pins are analog
    if (pinStr.startsWith('A') || pinStr.startsWith('a')) return { type: 'analog' };
    return { type: 'digital' };
  }

  function isOutputPin(pin: number | string): boolean {
    const info = getPinInfo(pin);
    const pinType = (info?.pin_type || info?.type || info?.special || '').toLowerCase();
    return OUTPUT_PIN_TYPES.some(t => pinType.includes(t.toLowerCase()));
  }

  function isEncoderCapable(pin: number | string): boolean {
    const info = getPinInfo(pin);
    const pinType = (info?.pin_type || info?.type || '').toLowerCase();
    // Only explicitly configured encoder pins, not all digital pins
    return pinType === 'encoder';
  }

  function isAnalogPin(pin: number | string): boolean {
    const info = getPinInfo(pin);
    const pinType = (info?.pin_type || info?.type || '').toLowerCase();
    return ANALOG_PIN_TYPES.some(t => pinType.includes(t.toLowerCase()));
  }

  // Get all vars + events combined for the target picker
  let allTargets = $derived([
    ...(aircraft?.variables || []).map((v: any) => ({
      ...v,
      targetType: 'variable',
      action: v.writable ? 'set_var' : 'read_var',
      direction: v.writable ? 'input' : 'output',
    })),
    ...(aircraft?.events || []).map((e: any) => ({
      ...e,
      targetType: 'event',
      action: 'event',
      direction: 'input',
    })),
  ]);

  let filteredTargets = $derived(
    allTargets.filter((t: any) => {
      if (filterCategory !== 'all' && t.category !== filterCategory) return false;
      if (searchQuery && !(t.label || t.name).toLowerCase().includes(searchQuery.toLowerCase())) return false;
      if (directionFilter === 'input' && t.direction === 'output') return false;
      if (directionFilter === 'output' && t.direction !== 'output') return false;
      // When in encoder CCW step, only show events
      if (selectedPin && isEncoderCapable(selectedPin.pin) && encoderStep === 'ccw') {
        if (t.targetType !== 'event') return false;
      }
      // When pin is an output pin, only show readable vars
      if (selectedPin && isOutputPin(selectedPin.pin)) {
        if (t.targetType !== 'variable') return false;
      }
      return true;
    })
  );

  let categories = $derived([
    'all',
    ...new Set(allTargets.map((t: any) => t.category).filter(Boolean))
  ]);

  // Is the current selected pin in encoder assignment mode?
  let isEncoderMode = $derived(
    selectedPin !== null && isEncoderCapable((selectedPin as any).pin) && encoderCwTarget !== null
  );

  // Pin states from current mappings
  let mappedPinStates = $derived(
    Object.fromEntries(mappings.map((m: any) => [
      String(m.pin),
      { pressed: false, mapped: m.target }
    ]))
  );

  function handlePinClick(pin: number | string) {
    selectedPin = { pin, side: '' };
    encoderCwTarget = null;
    encoderCcwTarget = null;
    encoderStep = 'cw';
  }

  function assignTarget(target: any) {
    if (!selectedPin) {
      toast('Select a pin on the board first', 'warning');
      return;
    }

    const pin = selectedPin.pin;

    // --- Encoder dual-event flow ---
    if (isEncoderCapable(pin) && target.targetType === 'event') {
      if (encoderStep === 'cw') {
        encoderCwTarget = target;
        encoderStep = 'ccw';
        toast(`CW: ${target.label || target.name} — now pick the CCW event`, 'info');
        return;
      } else {
        // CCW step — finalize encoder mapping
        encoderCcwTarget = target;
        const newMapping = {
          pin,
          action: 'encoder',
          target: encoderCwTarget.name,
          options: { event_dec: target.name },
        };
        upsertMapping(newMapping);
        toast(`Pin ${pin} encoder: ${encoderCwTarget.label || encoderCwTarget.name} / ${target.label || target.name}`, 'success');
        resetSelection();
        return;
      }
    }

    // --- Output pin flow (sim → device) ---
    if (isOutputPin(pin)) {
      const newMapping = {
        pin,
        action: 'read_var',
        target: target.name,
        options: {},
      };
      upsertMapping(newMapping);
      toast(`Pin ${pin} (output) ← ${target.label || target.name}`, 'success');
      resetSelection();
      return;
    }

    // --- Analog pin flow ---
    if (isAnalogPin(pin) && target.action === 'set_var') {
      const newMapping = {
        pin,
        action: target.action,
        target: target.name,
        options: { range: [0, 100], deadzone: 5, invert: false, curve: 'linear' as const },
      };
      upsertMapping(newMapping);
      toast(`Pin ${pin} → ${target.label || target.name} (open calibration to adjust)`, 'success');
      // Open calibration panel
      calibrationPin = pin;
      calMin = 0;
      calMax = 100;
      calDeadzone = 5;
      calInvert = false;
      calCurve = 'linear';
      selectedPin = null;
      return;
    }

    // --- Default flow ---
    const newMapping = {
      pin: selectedPin.pin,
      action: target.action,
      target: target.name,
      options: target.action === 'set_var' ? { range: [0, 100] } : {},
    };
    upsertMapping(newMapping);
    toast(`Pin ${selectedPin.pin} → ${target.label || target.name}`, 'success');
    resetSelection();
  }

  function upsertMapping(newMapping: any) {
    const existing = mappings.findIndex((m: any) => String(m.pin) === String(newMapping.pin));
    if (existing >= 0) {
      mappings = [...mappings.slice(0, existing), newMapping, ...mappings.slice(existing + 1)];
    } else {
      mappings = [...mappings, newMapping];
    }
  }

  function resetSelection() {
    selectedPin = null;
    encoderCwTarget = null;
    encoderCcwTarget = null;
    encoderStep = 'cw';
  }

  function removeMapping(pin: number | string) {
    mappings = mappings.filter((m: any) => String(m.pin) !== String(pin));
    if (calibrationPin !== null && String(calibrationPin) === String(pin)) {
      calibrationPin = null;
    }
    toast(`Pin ${pin} unmapped`, 'info');
  }

  function openCalibration(pin: number | string) {
    const m = mappings.find((m: any) => String(m.pin) === String(pin));
    if (!m) return;
    calibrationPin = pin;
    calMin = m.options?.range?.[0] ?? 0;
    calMax = m.options?.range?.[1] ?? 100;
    calDeadzone = m.options?.deadzone ?? 5;
    calInvert = m.options?.invert ?? false;
    calCurve = m.options?.curve ?? 'linear';
  }

  function applyCalibration() {
    if (calibrationPin === null) return;
    const idx = mappings.findIndex((m: any) => String(m.pin) === String(calibrationPin));
    if (idx < 0) return;
    const updated = {
      ...mappings[idx],
      options: {
        ...mappings[idx].options,
        range: [calMin, calMax],
        deadzone: calDeadzone,
        invert: calInvert,
        curve: calCurve,
      },
    };
    mappings = [...mappings.slice(0, idx), updated, ...mappings.slice(idx + 1)];
    toast(`Calibration applied to Pin ${calibrationPin}`, 'success');
    calibrationPin = null;
  }

  function saveAll() {
    if (onSave) onSave(mappings);
    toast(`Saved ${mappings.length} mappings`, 'success');
  }

  function categoryColor(cat: string): string {
    const colors: Record<string, string> = {
      flight: '#00aaff', engine: '#ff5555', autopilot: '#bb88ff',
      electrical: '#ffaa00', fuel: '#00ff88', radio: '#55ccff',
      lights: '#ffdd44', misc: '#888899',
    };
    return colors[cat] || '#888899';
  }

  // SVG curve preview path generator
  function curvePath(curve: string, invert: boolean): string {
    const points: [number, number][] = [];
    const steps = 40;
    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      let y: number;
      if (curve === 'logarithmic') {
        y = Math.log(1 + t * 9) / Math.log(10);
      } else if (curve === 'exponential') {
        y = (Math.pow(10, t) - 1) / 9;
      } else {
        y = t;
      }
      if (invert) y = 1 - y;
      // SVG coords: x goes right (0..100), y goes down (100..0)
      points.push([t * 100, (1 - y) * 100]);
    }
    return 'M ' + points.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(' L ');
  }

  function getMappingDirection(m: any): 'input' | 'output' {
    return m.action === 'read_var' ? 'output' : 'input';
  }
</script>

<div class="dnd-mapper">
  <!-- Top bar with save -->
  <div style="display: flex; justify-content: space-between; align-items: center; padding: 8px 0; margin-bottom: 8px; border-bottom: 1px solid var(--border-panel);">
    <div style="font-size: 12px; color: var(--text-dim);">
      {mappings.length} pin{mappings.length !== 1 ? 's' : ''} mapped
    </div>
    <button class="btn btn-primary btn-sm" onclick={saveAll} disabled={mappings.length === 0}>
      <Save size={12} />
      Save ({mappings.length})
    </button>
  </div>
  <div class="dnd-layout">
    <!-- Left: Pin List -->
    <div class="dnd-board">
      <div class="dnd-section-title">PINS</div>
      <div style="display: flex; gap: 12px; margin-bottom: 10px; font-size: 11px; color: var(--text-dim);">
        <span>{allPins.length} total</span>
        <span style="color: var(--pfd-green);">{mappings.length} mapped</span>
        <span style="color: var(--text-muted);">{allPins.length - mappings.length} available</span>
      </div>
      <div style="display: flex; flex-direction: column; gap: 2px; max-height: 400px; overflow-y: auto; padding: 4px;">
        {#each ['digital', 'analog', 'peripheral'] as section}
          {@const sectionPins = allPins.filter((pin: any) => {
            const info = getPinInfo(pin);
            const t = (info?.type || info?.pin_type || '').toLowerCase();
            if (section === 'digital') return t === 'digital' || t === 'gpio' || (!t.includes('analog') && !t.includes('shift') && !t.includes('i2c') && !t.includes('spi'));
            if (section === 'analog') return t.includes('analog') || t.includes('potentiometer');
            return t.includes('shift') || t.includes('i2c') || t.includes('spi') || t.includes('mux');
          })}
          {#if sectionPins.length > 0}
            <div style="font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 1.5px; color: var(--text-muted); padding: 8px 4px 4px; margin-top: 4px; border-top: 1px solid var(--border-panel);">
              {#if section === 'digital'}Digital Pins ({sectionPins.length}){:else if section === 'analog'}Analog Pins ({sectionPins.length}){:else}Peripherals ({sectionPins.length}){/if}
            </div>
          {/if}
          {#each sectionPins as pin}
            {@const info = getPinInfo(pin)}
            {@const mapped = mappings.find((m: any) => String(m.pin) === String(pin))}
            {@const isSelected = selectedPin?.pin === pin}
            <button
              style="display: flex; align-items: center; gap: 8px; padding: 6px 10px; border-radius: 6px; border: 1px solid {isSelected ? 'var(--accent)' : mapped ? 'var(--pfd-green)' : 'var(--border-panel)'}; background: {isSelected ? 'rgba(56,189,248,0.15)' : mapped ? 'rgba(34,197,94,0.08)' : 'var(--bg-instrument)'}; cursor: pointer; text-align: left; width: 100%; font-size: 12px; transition: all 0.15s;"
              onclick={() => handlePinClick(pin)}
            >
              {#if mapped}
                <span style="width: 6px; height: 6px; border-radius: 50%; background: var(--pfd-green); flex-shrink: 0; box-shadow: 0 0 4px rgba(34,197,94,0.5);"></span>
              {:else}
                <span style="width: 6px; height: 6px; border-radius: 50%; background: var(--border-panel); flex-shrink: 0;"></span>
              {/if}
              <span style="font-family: var(--mono); font-weight: 700; min-width: 36px; color: {isSelected ? 'var(--accent)' : 'var(--text-bright)'};">{pin}</span>
              <span style="font-size: 9px; padding: 1px 5px; border-radius: 3px; background: var(--bg-surface); color: var(--text-dim); text-transform: uppercase;">{info?.type || 'gpio'}</span>
              {#if mapped}
                <span style="flex: 1; font-size: 10px; color: var(--pfd-green); text-overflow: ellipsis; overflow: hidden; white-space: nowrap;">{mapped.target}</span>
              {:else}
                <span style="flex: 1; font-size: 10px; color: var(--text-muted);">—</span>
              {/if}
            </button>
          {/each}
        {/each}
      </div>

      {#if selectedPin}
        <div class="pin-selected-banner" class:pin-output={isOutputPin(selectedPin.pin)} class:pin-encoder={isEncoderCapable(selectedPin.pin) && encoderStep !== 'cw'}>
          <span class="psb-pin">Pin {selectedPin.pin}</span>

          {#if isOutputPin(selectedPin.pin)}
            <span class="psb-arrow psb-arrow-output">
              <ArrowRightLeft size={14} />
            </span>
            <span class="psb-text">OUTPUT pin — select a SimVar to read from the sim</span>
          {:else if isEncoderCapable(selectedPin.pin) && encoderStep === 'ccw'}
            <span class="psb-arrow psb-arrow-encoder">
              <RotateCcw size={14} />
            </span>
            <span class="psb-text">
              Now pick the <strong>counter-clockwise (CCW)</strong> event
              <span class="psb-cw-label">CW: {encoderCwTarget?.label || encoderCwTarget?.name}</span>
            </span>
          {:else}
            <span class="psb-arrow">&#x2190;</span>
            <span class="psb-text">Select a variable or event from the right panel</span>
          {/if}
          <button class="btn btn-ghost btn-sm" onclick={() => resetSelection()}>Cancel</button>
        </div>
      {/if}

      <!-- Analog Calibration Panel -->
      {#if calibrationPin !== null}
        {@const calMapping = mappings.find((m: any) => String(m.pin) === String(calibrationPin))}
        <div class="calibration-panel">
          <div class="cal-header">
            <SlidersHorizontal size={14} />
            <span class="cal-title">ANALOG CALIBRATION — Pin {calibrationPin}</span>
            <span class="cal-target">{calMapping?.target}</span>
            <button class="btn btn-ghost btn-sm" onclick={() => calibrationPin = null}>
              <X size={12} />
            </button>
          </div>

          <div class="cal-body">
            <div class="cal-fields">
              <label class="cal-field">
                <span class="cal-label">Min</span>
                <input type="number" bind:value={calMin} class="cal-input" />
              </label>
              <label class="cal-field">
                <span class="cal-label">Max</span>
                <input type="number" bind:value={calMax} class="cal-input" />
              </label>
              <label class="cal-field">
                <span class="cal-label">Deadzone</span>
                <input type="number" bind:value={calDeadzone} min="0" max="50" class="cal-input" />
              </label>
              <label class="cal-field cal-field-check">
                <input type="checkbox" bind:checked={calInvert} />
                <FlipVertical size={12} />
                <span class="cal-label">Invert</span>
              </label>
              <label class="cal-field">
                <span class="cal-label">Curve</span>
                <select bind:value={calCurve} class="cal-input">
                  <option value="linear">Linear</option>
                  <option value="logarithmic">Logarithmic</option>
                  <option value="exponential">Exponential</option>
                </select>
              </label>
            </div>

            <!-- Curve preview SVG -->
            <div class="cal-preview">
              <svg viewBox="-5 -5 110 110" class="cal-svg">
                <!-- Grid -->
                <line x1="0" y1="100" x2="100" y2="100" class="cal-axis" />
                <line x1="0" y1="0" x2="0" y2="100" class="cal-axis" />
                <line x1="0" y1="0" x2="100" y2="0" class="cal-grid" />
                <line x1="100" y1="0" x2="100" y2="100" class="cal-grid" />
                <!-- Deadzone indicator -->
                {#if calDeadzone > 0}
                  <rect x="0" y={100 - calDeadzone} width="100" height={calDeadzone}
                    class="cal-deadzone-rect" />
                  <rect x="0" y="0" width={calDeadzone} height="100"
                    class="cal-deadzone-rect" />
                {/if}
                <!-- Linear reference -->
                <line x1="0" y1="100" x2="100" y2="0" class="cal-ref-line" />
                <!-- Actual curve -->
                <path d={curvePath(calCurve, calInvert)} class="cal-curve" />
                <!-- Labels -->
                <text x="50" y="115" class="cal-svg-label" text-anchor="middle">Input</text>
                <text x="-10" y="50" class="cal-svg-label" text-anchor="middle" transform="rotate(-90, -10, 50)">Output</text>
              </svg>
            </div>
          </div>

          <div class="cal-actions">
            <button class="btn btn-primary btn-sm" onclick={applyCalibration}>
              <Save size={12} />
              Apply Calibration
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- Right: Variable/Event Picker -->
    <div class="dnd-targets">
      <div class="dnd-section-title">
        {aircraft?.name || 'AIRCRAFT'} — CONTROLS
      </div>

      <!-- Direction filter -->
      <div class="direction-pills">
        <button class="dir-pill" class:dir-active={directionFilter === 'all'} onclick={() => directionFilter = 'all'}>
          All
        </button>
        <button class="dir-pill dir-input" class:dir-active={directionFilter === 'input'} onclick={() => directionFilter = 'input'}>
          <ToggleLeft size={11} />
          Input
        </button>
        <button class="dir-pill dir-output" class:dir-active={directionFilter === 'output'} onclick={() => directionFilter = 'output'}>
          <Monitor size={11} />
          Output
        </button>
      </div>

      <!-- Search + Filter -->
      <div class="target-filters">
        <input type="text" bind:value={searchQuery} placeholder="Search variables..." style="flex: 1" />
        <div class="cat-pills">
          {#each categories as cat}
            <button
              class="cat-pill"
              class:cat-active={filterCategory === cat}
              style={cat !== 'all' ? `--cat-color: ${categoryColor(cat)}` : ''}
              onclick={() => filterCategory = cat}
            >
              {cat}
            </button>
          {/each}
        </div>
      </div>

      <!-- Target list -->
      <div class="target-list">
        {#each filteredTargets as target}
          {@const isMapped = mappings.some((m: any) => m.target === target.name)}
          <button
            class="target-item"
            class:target-mapped={isMapped}
            class:target-highlight={selectedPin !== null}
            class:target-input={target.direction === 'input'}
            class:target-output={target.direction === 'output'}
            onclick={() => assignTarget(target)}
          >
            <div class="target-cat-dot" style="background: {categoryColor(target.category)}"></div>
            <div class="target-dir-indicator" class:tdi-input={target.direction === 'input'} class:tdi-output={target.direction === 'output'}>
              {#if target.direction === 'output'}
                <Monitor size={10} />
              {:else if target.targetType === 'event'}
                <Binary size={10} />
              {:else}
                <Gauge size={10} />
              {/if}
            </div>
            <div class="target-info">
              <div class="target-label">{target.label || target.name}</div>
              <div class="target-name">{target.name}</div>
            </div>
            <div class="target-badges">
              {#if target.targetType === 'event'}
                <span class="target-badge target-badge-event">EVENT</span>
              {:else if target.writable}
                <span class="target-badge target-badge-write">R/W</span>
              {:else}
                <span class="target-badge target-badge-read">READ</span>
              {/if}
              {#if target.direction === 'output'}
                <span class="target-badge target-badge-output">OUTPUT</span>
              {/if}
              {#if target.var_type && target.var_type !== 'A'}
                <span class="target-badge target-badge-lvar">{target.var_type}:var</span>
              {/if}
              {#if target.unit}
                <span class="target-badge">{target.unit}</span>
              {/if}
            </div>
            {#if isMapped}
              <span class="target-check">&#x2713;</span>
            {/if}
          </button>
        {:else}
          <div style="padding: 20px; text-align: center; color: var(--text-dim); font-size: 12px">
            No matching variables
          </div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Bottom: Active Mappings Summary -->
  {#if mappings.length === 0}
    <div style="padding: 32px; text-align: center; background: var(--bg-instrument); border: 1px dashed var(--border-panel); border-radius: 10px;">
      <div style="font-size: 24px; opacity: 0.2; margin-bottom: 8px;">&#x1F50C;</div>
      <div style="font-size: 13px; font-weight: 600; color: var(--text-dim); margin-bottom: 4px;">No pins mapped yet</div>
      <div style="font-size: 11px; color: var(--text-muted);">Select a pin on the left, then pick a variable or event on the right to create a mapping.</div>
    </div>
  {:else}
    <div class="mapping-summary">
      <div class="ms-header">
        <span class="ms-title">ACTIVE MAPPINGS ({mappings.length})</span>
        <button class="btn btn-primary btn-sm" onclick={saveAll}>
          <Save size={12} />
          Save Profile
        </button>
      </div>
      <div class="ms-list">
        {#each mappings as m}
          <div class="ms-item" class:ms-item-input={getMappingDirection(m) === 'input'} class:ms-item-output={getMappingDirection(m) === 'output'}>
            <span class="ms-pin">Pin {m.pin}</span>

            {#if m.action === 'read_var'}
              <span class="ms-arrow ms-arrow-output">&#x2190;</span>
              <span class="ms-action ms-output">
                <Monitor size={10} />
                output
              </span>
            {:else if m.action === 'encoder'}
              <span class="ms-arrow">&#x2192;</span>
              <span class="ms-action ms-encoder">
                <RotateCw size={10} />
                encoder
              </span>
            {:else}
              <span class="ms-arrow">&#x2192;</span>
              <span class="ms-action" class:ms-event={m.action === 'event'}>{m.action}</span>
            {/if}

            <span class="ms-target">{m.target}</span>

            {#if m.action === 'encoder' && m.options?.event_dec}
              <span class="ms-encoder-details">
                <span class="ms-enc-dir">
                  <RotateCw size={9} />
                  {m.target}
                </span>
                <span class="ms-enc-dir">
                  <RotateCcw size={9} />
                  {m.options.event_dec}
                </span>
              </span>
            {/if}

            {#if m.options?.curve && m.options.curve !== 'linear'}
              <span class="ms-badge ms-badge-curve">{m.options.curve}</span>
            {/if}
            {#if m.options?.invert}
              <span class="ms-badge ms-badge-invert">
                <FlipVertical size={9} />
                inv
              </span>
            {/if}

            {#if isAnalogPin(m.pin) && m.action === 'set_var'}
              <button class="ms-cal-btn" onclick={() => openCalibration(m.pin)} title="Calibration">
                <SlidersHorizontal size={10} />
              </button>
            {/if}

            <button class="ms-remove" onclick={() => removeMapping(m.pin)}>&#x2715;</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .dnd-mapper {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .dnd-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    min-height: 400px;
  }

  .dnd-section-title {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 2px;
    color: var(--text-dim);
    margin-bottom: 12px;
    text-transform: uppercase;
  }

  /* ─── Board side ─── */
  .dnd-board { display: flex; flex-direction: column; gap: 12px; }

  .pin-selected-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: var(--accent-dim, rgba(0,170,255,0.1));
    border: 1px solid rgba(0, 170, 255, 0.2);
    border-radius: 6px;
    animation: pulse-border 1.5s ease-in-out infinite;
  }

  .pin-selected-banner.pin-output {
    background: rgba(255, 170, 0, 0.08);
    border-color: rgba(255, 170, 0, 0.3);
  }

  .pin-selected-banner.pin-encoder {
    background: rgba(187, 136, 255, 0.08);
    border-color: rgba(187, 136, 255, 0.3);
  }

  @keyframes pulse-border {
    0%, 100% { border-color: rgba(0, 170, 255, 0.2); }
    50% { border-color: rgba(0, 170, 255, 0.5); }
  }

  .psb-pin {
    font-family: var(--mono);
    font-size: 14px;
    font-weight: 700;
    color: var(--accent);
  }

  .psb-arrow { color: var(--accent); font-size: 16px; }
  .psb-arrow-output { color: #ffaa00; }
  .psb-arrow-encoder { color: #bb88ff; }
  .psb-text { font-size: 11px; color: var(--text-dim); flex: 1; }
  .psb-cw-label {
    display: inline-block;
    margin-left: 6px;
    padding: 1px 6px;
    background: rgba(187, 136, 255, 0.15);
    border-radius: 3px;
    font-size: 9px;
    font-family: var(--mono);
    color: #bb88ff;
  }

  /* ─── Direction filter pills ─── */
  .direction-pills {
    display: flex;
    gap: 4px;
    margin-bottom: 4px;
  }

  .dir-pill {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 10px;
    border: 1px solid var(--border-panel);
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
    font-family: var(--font);
    transition: all 0.1s;
  }

  .dir-pill:hover { border-color: var(--text-dim); }

  .dir-pill.dir-active {
    color: #000;
    border-color: var(--accent);
    background: var(--accent);
  }
  .dir-pill.dir-input.dir-active {
    background: #34d399;
    border-color: #34d399;
  }
  .dir-pill.dir-output.dir-active {
    background: #fbbf24;
    border-color: #fbbf24;
  }

  /* ─── Target picker ─── */
  .dnd-targets { display: flex; flex-direction: column; gap: 10px; }

  .target-filters {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .cat-pills {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .cat-pill {
    padding: 2px 8px;
    border: 1px solid var(--border-panel);
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    cursor: pointer;
    font-family: var(--font);
    transition: all 0.1s;
  }

  .cat-pill:hover { border-color: var(--text-dim); color: var(--text-dim); }
  .cat-pill.cat-active {
    background: var(--cat-color, var(--accent));
    border-color: var(--cat-color, var(--accent));
    color: #000;
  }

  .target-list {
    flex: 1;
    overflow-y: auto;
    max-height: 400px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .target-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.1s;
    text-align: left;
    font-family: var(--font);
    color: var(--text);
    width: 100%;
  }

  .target-item:hover { border-color: var(--accent); background: var(--accent-dim); }
  .target-item.target-highlight:hover { box-shadow: 0 0 0 2px var(--accent); }
  .target-item.target-mapped { opacity: 0.5; }

  .target-item.target-input { border-left: 3px solid #34d399; }
  .target-item.target-output { border-left: 3px solid #fbbf24; }

  .target-cat-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .target-dir-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 3px;
    flex-shrink: 0;
  }
  .target-dir-indicator.tdi-input { background: rgba(52, 211, 153, 0.15); color: #34d399; }
  .target-dir-indicator.tdi-output { background: rgba(251, 191, 36, 0.15); color: #fbbf24; }

  .target-info { flex: 1; min-width: 0; }
  .target-label { font-size: 11px; font-weight: 600; }
  .target-name { font-family: var(--mono); font-size: 9px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .target-badges { display: flex; gap: 3px; flex-shrink: 0; }
  .target-badge {
    font-size: 8px;
    font-weight: 700;
    padding: 1px 4px;
    border-radius: 2px;
    background: var(--bg-bezel);
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }
  .target-badge-event { background: rgba(0,170,255,0.15); color: var(--accent); }
  .target-badge-write { background: rgba(0,255,136,0.15); color: var(--pfd-green); }
  .target-badge-read { background: rgba(255,255,255,0.05); color: var(--text-muted); }
  .target-badge-lvar { background: rgba(187,136,255,0.15); color: #bb88ff; }
  .target-badge-output { background: rgba(251,191,36,0.15); color: #fbbf24; }

  .target-check { color: var(--pfd-green); font-size: 12px; font-weight: 700; }

  /* ─── Calibration panel ─── */
  .calibration-panel {
    background: var(--bg-panel);
    border: 1px solid rgba(251, 191, 36, 0.25);
    border-radius: 8px;
    padding: 14px 16px;
  }

  .cal-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
    color: #fbbf24;
  }

  .cal-title {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 1.5px;
    color: var(--text-dim);
  }

  .cal-target {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--text-muted);
    margin-left: auto;
    margin-right: 8px;
  }

  .cal-body {
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }

  .cal-fields {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
  }

  .cal-field {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .cal-field-check {
    cursor: pointer;
    color: var(--text-dim);
  }

  .cal-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-dim);
    min-width: 60px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .cal-input {
    width: 80px;
    padding: 3px 6px;
    border: 1px solid var(--border-panel);
    border-radius: 3px;
    background: var(--bg-instrument);
    color: var(--text);
    font-family: var(--mono);
    font-size: 11px;
  }

  select.cal-input {
    width: 120px;
    cursor: pointer;
  }

  .cal-preview {
    width: 120px;
    height: 120px;
    flex-shrink: 0;
    background: var(--bg-instrument);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 8px;
  }

  .cal-svg {
    width: 100%;
    height: 100%;
  }

  .cal-axis {
    stroke: var(--text-muted);
    stroke-width: 1;
    fill: none;
  }

  .cal-grid {
    stroke: var(--border);
    stroke-width: 0.5;
    fill: none;
    stroke-dasharray: 2,2;
  }

  .cal-ref-line {
    stroke: var(--text-muted);
    stroke-width: 0.5;
    stroke-dasharray: 3,3;
    fill: none;
    opacity: 0.4;
  }

  .cal-curve {
    stroke: #fbbf24;
    stroke-width: 2;
    fill: none;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .cal-deadzone-rect {
    fill: rgba(255, 80, 80, 0.08);
    stroke: rgba(255, 80, 80, 0.2);
    stroke-width: 0.5;
  }

  .cal-svg-label {
    font-size: 8px;
    fill: var(--text-muted);
    font-family: var(--font);
  }

  .cal-actions {
    margin-top: 10px;
    display: flex;
    justify-content: flex-end;
  }

  /* ─── Mapping summary ─── */
  .mapping-summary {
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    border-radius: 8px;
    padding: 14px 16px;
  }

  .ms-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  .ms-title {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 2px;
    color: var(--text-dim);
  }

  .ms-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .ms-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--bg-instrument);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 10px;
  }

  .ms-item.ms-item-input { border-left: 3px solid #34d399; }
  .ms-item.ms-item-output { border-left: 3px solid #fbbf24; }

  .ms-pin { font-family: var(--mono); font-weight: 700; color: var(--accent); }
  .ms-arrow { color: var(--text-muted); font-size: 10px; }
  .ms-arrow-output { color: #fbbf24; }
  .ms-action { font-weight: 600; color: var(--pfd-green); font-size: 9px; text-transform: uppercase; display: flex; align-items: center; gap: 3px; }
  .ms-action.ms-event { color: var(--accent); }
.ms-action.ms-output { color: #fbbf24; }
  .ms-action.ms-encoder { color: #bb88ff; }
  .ms-target { font-family: var(--mono); color: var(--text); max-width: 150px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .ms-encoder-details {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin-left: 4px;
  }

  .ms-enc-dir {
    display: flex;
    align-items: center;
    gap: 3px;
    font-family: var(--mono);
    font-size: 9px;
    color: var(--text-dim);
  }

  .ms-badge {
    font-size: 8px;
    font-weight: 700;
    padding: 1px 4px;
    border-radius: 2px;
    text-transform: uppercase;
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .ms-badge-curve {
    background: rgba(251, 191, 36, 0.12);
    color: #fbbf24;
  }

  .ms-badge-invert {
    background: rgba(255, 85, 85, 0.12);
    color: #ff5555;
  }

  .ms-cal-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px 4px;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }
  .ms-cal-btn:hover {
    color: #fbbf24;
    border-color: #fbbf24;
  }

  .ms-remove {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 10px;
    padding: 0 2px;
  }
  .ms-remove:hover { color: var(--warning); }

  @media (max-width: 800px) {
    .dnd-layout { grid-template-columns: 1fr; }
    .cal-body { flex-direction: column; }
    .cal-preview { width: 100%; height: 100px; }
  }
</style>
