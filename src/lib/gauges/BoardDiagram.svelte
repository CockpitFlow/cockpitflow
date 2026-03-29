<script lang="ts">
  import { toast } from './Toast.svelte';

  let { board = 'arduino-nano', pinStates = {}, onpinclick, activeMappings = [], selectedPin = null }: {
    board?: string;
    pinStates?: Record<number, { pressed?: boolean; value?: number; mapped?: string }>;
    onpinclick?: (pin: number | string, side: string) => void;
    activeMappings?: any[];
    selectedPin?: number | string | null;
  } = $props();

  // Drag & drop
  let dragTarget: { pin: number | string; el: HTMLElement } | null = $state(null);

  function getPinMapping(pin: number | string) {
    return activeMappings.find((m: any) => String(m.pin) === String(pin));
  }

  function handlePinClick(pin: number | string, side: string) {
    if (onpinclick) onpinclick(pin, side);
  }

  // ─── Arduino Nano Pin Layout ───
  const nanoPinsLeft = [
    { pin: 'D13', num: 13, type: 'digital', special: 'LED/SCK' },
    { pin: 'D12', num: 12, type: 'digital', special: 'MISO' },
    { pin: 'D11', num: 11, type: 'digital', special: 'MOSI/PWM' },
    { pin: 'D10', num: 10, type: 'digital', special: 'SS/PWM' },
    { pin: 'D9', num: 9, type: 'digital', special: 'PWM' },
    { pin: 'D8', num: 8, type: 'digital' },
    { pin: 'D7', num: 7, type: 'digital' },
    { pin: 'D6', num: 6, type: 'digital', special: 'PWM' },
    { pin: 'D5', num: 5, type: 'digital', special: 'PWM' },
    { pin: 'D4', num: 4, type: 'digital' },
    { pin: 'D3', num: 3, type: 'digital', special: 'PWM/INT1' },
    { pin: 'D2', num: 2, type: 'digital', special: 'INT0' },
    { pin: 'GND', num: -1, type: 'power' },
    { pin: 'RST', num: -2, type: 'power' },
    { pin: 'RX', num: 0, type: 'serial' },
    { pin: 'TX', num: 1, type: 'serial' },
  ];

  const nanoPinsRight = [
    { pin: 'D12', num: -3, type: 'power', special: 'VIN' },
    { pin: 'GND', num: -4, type: 'power' },
    { pin: 'RST', num: -5, type: 'power' },
    { pin: '5V', num: -6, type: 'power' },
    { pin: 'A7', num: 21, type: 'analog' },
    { pin: 'A6', num: 20, type: 'analog' },
    { pin: 'A5', num: 19, type: 'analog', special: 'SCL' },
    { pin: 'A4', num: 18, type: 'analog', special: 'SDA' },
    { pin: 'A3', num: 17, type: 'analog' },
    { pin: 'A2', num: 16, type: 'analog' },
    { pin: 'A1', num: 15, type: 'analog' },
    { pin: 'A0', num: 14, type: 'analog' },
    { pin: 'REF', num: -7, type: 'power' },
    { pin: '3V3', num: -8, type: 'power' },
    { pin: 'D13', num: -9, type: 'power' },
    { pin: 'GND', num: -10, type: 'power' },
  ];

  // ─── Arduino Mega Pin Layout (simplified) ───
  const megaPinsLeft = [
    ...Array.from({ length: 14 }, (_, i) => ({ pin: `D${53 - i}`, num: 53 - i, type: 'digital' as const })),
  ];
  const megaPinsRight = [
    ...Array.from({ length: 14 }, (_, i) => ({ pin: `A${i}`, num: 54 + i, type: 'analog' as const })),
  ];

  function getPins(side: 'left' | 'right') {
    if (board === 'arduino-nano') return side === 'left' ? nanoPinsLeft : nanoPinsRight;
    return side === 'left' ? megaPinsLeft : megaPinsRight;
  }

  function pinColor(p: any): string {
    if (p.type === 'power') return 'var(--warning, #ffaa00)';
    if (p.type === 'serial') return 'var(--magenta, #ff44ff)';
    const state = pinStates[p.num];
    if (state?.pressed) return 'var(--pfd-green, #00ff88)';
    const mapping = getPinMapping(p.num);
    if (mapping) return 'var(--accent, #00aaff)';
    if (p.type === 'analog') return '#8b5cf6';
    return 'var(--text-dim, #4a5068)';
  }

  function pinGlow(p: any): string {
    const state = pinStates[p.num];
    if (state?.pressed) return '0 0 8px rgba(0, 255, 136, 0.6)';
    return 'none';
  }
</script>

<div class="board-diagram">
  <div class="board-header">
    <span class="board-chip">{board === 'arduino-nano' ? 'ATmega328P' : board === 'arduino-mega' ? 'ATmega2560' : 'ESP32'}</span>
    <span class="board-label">{board === 'arduino-nano' ? 'ARDUINO NANO' : board === 'arduino-mega' ? 'ARDUINO MEGA' : 'ESP32-C6'}</span>
  </div>

  <div class="board-body">
    <!-- Left pins -->
    <div class="pin-col pin-col-left">
      {#each getPins('left') as p, i}
        {@const mapping = p.num >= 0 ? getPinMapping(p.num) : null}
        {@const state = p.num >= 0 ? pinStates[p.num] : null}
        <div class="pin-row">
          {#if mapping}
            <div class="pin-mapping-tag pin-mapping-left">{mapping.target?.split(' ')[0] || '?'}</div>
          {/if}
          <button
            class="pin-dot"
            class:pin-power={p.type === 'power'}
            class:pin-serial={p.type === 'serial'}
            class:pin-active={state?.pressed}
            class:pin-mapped={!!mapping}
            class:pin-selected={String(selectedPin) === String(p.num)}
            style="background: {pinColor(p)}; box-shadow: {pinGlow(p)}"
            onclick={() => p.num >= 0 && handlePinClick(p.num, 'left')}
            disabled={p.num < 0}
          ></button>
          <span class="pin-label" class:pin-label-active={state?.pressed}>
            {p.pin}
            {#if p.special}<span class="pin-special">{p.special}</span>{/if}
          </span>
        </div>
      {/each}
    </div>

    <!-- Board chip visual -->
    <div class="chip-visual">
      <div class="chip-body">
        <div class="chip-notch"></div>
        <div class="chip-text">
          {#if board === 'arduino-nano'}
            <div class="chip-line">NANO</div>
            <div class="chip-line chip-sub">328P</div>
          {:else if board === 'arduino-mega'}
            <div class="chip-line">MEGA</div>
            <div class="chip-line chip-sub">2560</div>
          {:else}
            <div class="chip-line">ESP32</div>
            <div class="chip-line chip-sub">C6</div>
          {/if}
        </div>
        <div class="chip-usb">USB</div>
      </div>
    </div>

    <!-- Right pins -->
    <div class="pin-col pin-col-right">
      {#each getPins('right') as p, i}
        {@const mapping = p.num >= 0 ? getPinMapping(p.num) : null}
        {@const state = p.num >= 0 ? pinStates[p.num] : null}
        <div class="pin-row pin-row-right">
          <span class="pin-label pin-label-right" class:pin-label-active={state?.pressed}>
            {#if p.special}<span class="pin-special">{p.special}</span>{/if}
            {p.pin}
          </span>
          <button
            class="pin-dot"
            class:pin-power={p.type === 'power'}
            class:pin-serial={p.type === 'serial'}
            class:pin-active={state?.pressed}
            class:pin-mapped={!!mapping}
            class:pin-selected={String(selectedPin) === String(p.num)}
            style="background: {pinColor(p)}; box-shadow: {pinGlow(p)}"
            onclick={() => p.num >= 0 && handlePinClick(p.num, 'right')}
            disabled={p.num < 0}
          ></button>
          {#if mapping}
            <div class="pin-mapping-tag pin-mapping-right">{mapping.target?.split(' ')[0] || '?'}</div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- Legend -->
  <div class="board-legend">
    <span class="legend-item"><span class="legend-dot" style="background: var(--pfd-green)"></span> Active</span>
    <span class="legend-item"><span class="legend-dot" style="background: var(--accent)"></span> Mapped</span>
    <span class="legend-item"><span class="legend-dot" style="background: #8b5cf6"></span> Analog</span>
    <span class="legend-item"><span class="legend-dot" style="background: var(--text-dim)"></span> Digital</span>
    <span class="legend-item"><span class="legend-dot" style="background: var(--warning)"></span> Power</span>
    <span class="legend-item"><span class="legend-dot" style="background: #ff6b6b"></span> Display</span>
    <span class="legend-item"><span class="legend-dot" style="background: #4ecdc4"></span> Motor</span>
  </div>
</div>

<style>
  .board-diagram {
    background: var(--bg-instrument, #0f1117);
    border: 1px solid var(--border-panel, rgba(255,255,255,0.08));
    border-radius: 12px;
    padding: 20px;
    box-shadow: inset 0 2px 6px rgba(0,0,0,0.4);
  }

  .board-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border, rgba(255,255,255,0.04));
  }

  .board-chip {
    font-family: var(--mono, monospace);
    font-size: 10px;
    font-weight: 700;
    color: var(--text-dim);
    letter-spacing: 1px;
  }

  .board-label {
    font-family: var(--display, monospace);
    font-size: 13px;
    font-weight: 700;
    color: var(--accent, #00aaff);
    letter-spacing: 3px;
    text-shadow: 0 0 10px rgba(0, 170, 255, 0.3);
  }

  .board-body {
    display: flex;
    gap: 0;
    align-items: stretch;
  }

  .pin-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
  }

  .pin-row {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    position: relative;
  }

  .pin-row-right {
    flex-direction: row;
    justify-content: flex-end;
  }

  .pin-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    transition: all 0.1s;
    flex-shrink: 0;
    padding: 0;
  }

  .pin-dot:hover:not(:disabled) {
    transform: scale(1.4);
    z-index: 1;
  }

  .pin-dot:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .pin-dot.pin-active {
    animation: pin-pulse 0.6s ease-in-out infinite;
  }

  .pin-dot.pin-selected {
    background: var(--accent, #00aaff) !important;
    box-shadow: 0 0 12px rgba(0, 170, 255, 0.8), 0 0 24px rgba(0, 170, 255, 0.4) !important;
    transform: scale(1.5);
    animation: pin-selected-pulse 0.8s ease-in-out infinite;
  }

  @keyframes pin-selected-pulse {
    0%, 100% { transform: scale(1.5); box-shadow: 0 0 12px rgba(0, 170, 255, 0.8); }
    50% { transform: scale(1.8); box-shadow: 0 0 20px rgba(0, 170, 255, 1); }
  }

  @keyframes pin-pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.3); }
  }

  .pin-label {
    font-family: var(--mono, monospace);
    font-size: 9px;
    font-weight: 600;
    color: var(--text-muted, #2d3348);
    white-space: nowrap;
    min-width: 28px;
    transition: color 0.15s;
  }

  .pin-label-right { text-align: right; }

  .pin-label-active {
    color: var(--pfd-green, #00ff88) !important;
    text-shadow: 0 0 6px rgba(0, 255, 136, 0.4);
  }

  .pin-special {
    font-size: 7px;
    color: var(--text-muted);
    opacity: 0.5;
    margin-left: 2px;
  }

  .pin-label-right .pin-special {
    margin-left: 0;
    margin-right: 2px;
  }

  /* Chip body */
  .chip-visual {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: stretch;
    min-width: 80px;
    padding: 0 4px;
  }

  .chip-body {
    width: 60px;
    background: #0a0a10;
    border: 2px solid #1a1d2e;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: relative;
    box-shadow: 0 0 20px rgba(0,0,0,0.5), inset 0 0 10px rgba(0,0,0,0.3);
  }

  .chip-notch {
    position: absolute;
    top: -1px;
    left: 50%;
    transform: translateX(-50%);
    width: 14px;
    height: 7px;
    border-radius: 0 0 7px 7px;
    background: #1a1d2e;
  }

  .chip-text {
    text-align: center;
  }

  .chip-line {
    font-family: var(--display, monospace);
    font-size: 11px;
    font-weight: 700;
    color: var(--text-dim);
    letter-spacing: 2px;
  }

  .chip-sub {
    font-size: 8px;
    color: var(--text-muted);
    letter-spacing: 1px;
  }

  .chip-usb {
    position: absolute;
    bottom: 6px;
    font-size: 7px;
    font-weight: 700;
    color: var(--text-muted);
    letter-spacing: 1px;
    padding: 1px 6px;
    border: 1px solid var(--border-panel);
    border-radius: 2px;
  }

  /* Mapping tags */
  .pin-mapping-tag {
    font-family: var(--mono, monospace);
    font-size: 7px;
    font-weight: 700;
    color: var(--accent);
    background: var(--accent-dim, rgba(0,170,255,0.1));
    padding: 1px 4px;
    border-radius: 2px;
    white-space: nowrap;
    max-width: 70px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pin-mapping-left { order: -1; }

  /* Legend */
  .board-legend {
    display: flex;
    gap: 14px;
    justify-content: center;
    margin-top: 14px;
    padding-top: 10px;
    border-top: 1px solid var(--border, rgba(255,255,255,0.04));
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 9px;
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .legend-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
</style>
