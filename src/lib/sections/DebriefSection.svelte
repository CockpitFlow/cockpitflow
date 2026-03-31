<script lang="ts">
  import { toast } from '../gauges/Toast.svelte';

  let {
    flightActive = false,
    landingRate = 0,
    maxG = 1.0,
    maxAlt = 0,
    maxSpd = 0,
    maxBank = 0,
    maxVs = 0,
    minVs = 0,
    fuelUsed = 0,
    overspeedEvents = 0,
    stallWarnings = 0,
    touchdowns = [],
    logEntries = [],
    flightStartTime = 0,
    gradeFromRate = (_r: number) => 'C',
    gradeColor = (_g: string) => 'var(--color-dim)',
    resetDebrief = () => {},
  }: {
    flightActive?: boolean;
    landingRate?: number;
    maxG?: number;
    maxAlt?: number;
    maxSpd?: number;
    maxBank?: number;
    maxVs?: number;
    minVs?: number;
    fuelUsed?: number;
    overspeedEvents?: number;
    stallWarnings?: number;
    touchdowns?: { rate: number; speed: number; time: string }[];
    logEntries?: any[];
    flightStartTime?: number;
    gradeFromRate?: (r: number) => string;
    gradeColor?: (g: string) => string;
    resetDebrief?: () => void;
  } = $props();

  let debriefTab = $state('summary');

  function landingVerdict(r: number): string {
    const a = Math.abs(r);
    return a < 30 ? 'GREASER' : a < 60 ? 'BUTTER' : a < 120 ? 'SMOOTH' : a < 200 ? 'NORMAL' : a < 300 ? 'FIRM' : a < 500 ? 'HARD' : 'CRASHED';
  }

  function arcPath(cx: number, cy: number, r: number, startAngle: number, endAngle: number): string {
    const s = startAngle * Math.PI / 180, e = endAngle * Math.PI / 180;
    const x1 = cx + r * Math.cos(s), y1 = cy + r * Math.sin(s);
    const x2 = cx + r * Math.cos(e), y2 = cy + r * Math.sin(e);
    const large = endAngle - startAngle > 180 ? 1 : 0;
    return `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2}`;
  }

  function needlePos(rate: number): number {
    return Math.min(180, Math.max(0, Math.abs(rate) / 500 * 180));
  }

  function flightDuration() {
    return flightActive ? Math.round((Date.now() - flightStartTime) / 60000) : (logEntries.length > 0 ? logEntries[logEntries.length - 1].duration : 0);
  }
</script>

<div class="w-full h-full overflow-hidden flex flex-col bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md">
  <div class="flex border-b border-[var(--color-border)] shrink-0">
    {#each [['summary','Summary'],['landing','Landing'],['envelope','Envelope'],['score','Score'],['history','History']] as [id,label]}
      <button class="px-4 py-1.5 border-none bg-transparent text-[var(--color-dim)] font-mono text-[10px] font-semibold tracking-wide cursor-pointer border-b-2 border-transparent transition-all hover:text-[var(--color-fg)] {debriefTab === id ? 'text-[var(--color-accent)]! border-b-[var(--color-accent)]!' : ''}" onclick={() => debriefTab=id}>{label}</button>
    {/each}
    <div class="flex-1"></div>
    <div class="flex items-center gap-1.5 px-3 font-mono text-[9px] tracking-wide {flightActive ? 'text-[var(--color-green)]' : 'text-[var(--color-dim)]'}">
      <span class="w-1.5 h-1.5 rounded-full shrink-0 {flightActive ? 'bg-[var(--color-green)] shadow-[0_0_4px_var(--color-green)]' : 'bg-[var(--color-red)]'}"></span>
      {flightActive ? 'RECORDING' : 'IDLE'}
    </div>
  </div>
  <div class="flex-1 overflow-y-auto p-3.5">
    {#if debriefTab === 'summary'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">FLIGHT DATA</div>
          <table class="w-full border-collapse text-[11px]">
            <tbody>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Status</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{flightActive ? 'In Flight' : 'On Ground'}</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Duration</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{flightDuration()} min</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max Altitude</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{maxAlt.toFixed(0)} ft</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max IAS</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono {maxSpd > 129 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{maxSpd.toFixed(0)} kt</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max Bank</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono {maxBank > 30 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{maxBank.toFixed(0)}°</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max VS Up</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{maxVs.toFixed(0)} fpm</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max VS Down</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{minVs.toFixed(0)} fpm</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Fuel Used</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{fuelUsed.toFixed(1)} gal</td></tr>
            </tbody>
          </table>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">KEY METRICS</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">LANDING RATE</span><span class="font-mono text-lg font-extrabold leading-tight" style="color:{gradeColor(gradeFromRate(landingRate))}">{landingRate.toFixed(0)} <small class="text-[10px] font-medium text-[var(--color-dim)]">fpm</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">MAX G-FORCE</span><span class="font-mono text-lg font-extrabold leading-tight {maxG > 2.0 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{maxG.toFixed(2)} <small class="text-[10px] font-medium text-[var(--color-dim)]">G</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">OVERSPEED</span><span class="font-mono text-lg font-extrabold leading-tight {overspeedEvents > 0 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{overspeedEvents}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">STALL WARN</span><span class="font-mono text-lg font-extrabold leading-tight {stallWarnings > 0 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{stallWarnings}</span></div>
          </div>
        </div>
      </div>

    {:else if debriefTab === 'landing'}
      {@const grade = gradeFromRate(landingRate)}
      {@const color = gradeColor(grade)}
      {@const verdict = landingVerdict(landingRate)}
      {@const angle = needlePos(landingRate)}
      <div class="flex flex-col lg:flex-row gap-5 h-full">
        <!-- Left: big gauge -->
        <div class="lg:w-[280px] shrink-0 flex flex-col items-center justify-center gap-0.5">
          <div class="w-full max-w-[240px]">
            <svg viewBox="0 0 200 120" class="w-full h-auto block">
              <!-- Arc background zones -->
              <path d={arcPath(100, 100, 80, 180, 216)} style="stroke: var(--color-green)" stroke-width="8" fill="none" stroke-linecap="round" opacity="0.3" />
              <path d={arcPath(100, 100, 80, 216, 244)} style="stroke: var(--color-accent)" stroke-width="8" fill="none" opacity="0.3" />
              <path d={arcPath(100, 100, 80, 244, 280)} style="stroke: var(--color-yellow)" stroke-width="8" fill="none" opacity="0.3" />
              <path d={arcPath(100, 100, 80, 280, 324)} style="stroke: var(--color-orange)" stroke-width="8" fill="none" opacity="0.3" />
              <path d={arcPath(100, 100, 80, 324, 360)} style="stroke: var(--color-red)" stroke-width="8" fill="none" stroke-linecap="round" opacity="0.3" />
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
          <div class="flex items-baseline justify-center gap-1">
            <div class="font-mono text-[42px] font-black leading-none" style="color:{color}">{landingRate.toFixed(0)}</div>
            <div class="font-mono text-[11px] tracking-wide text-[var(--color-dim)]">FPM</div>
          </div>
          <div class="font-mono text-[13px] font-extrabold tracking-[2px]" style="color:{color}">{verdict}</div>
          <div class="font-mono text-[28px] font-black leading-none mt-1" style="color:{color}">{grade}</div>
        </div>

        <!-- Right: touchdown history -->
        <div class="flex-1 flex flex-col min-w-0">
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">TOUCHDOWNS</div>
          {#if touchdowns.length === 0}
            <div class="flex-1 flex flex-col items-center justify-center gap-1 text-[var(--color-dim)] text-[11px]">
              <div class="font-mono text-[20px] opacity-30">---</div>
              <span>No touchdowns yet</span>
              <span class="text-[9px] opacity-60">Land in the simulator to record</span>
            </div>
          {:else}
            <div class="flex flex-col gap-1.5 flex-1 overflow-y-auto">
              {#each touchdowns.slice().reverse() as td, i}
                {@const tGrade = gradeFromRate(td.rate)}
                {@const tColor = gradeColor(tGrade)}
                <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-1.5 flex items-center gap-2 relative overflow-hidden">
                  <div class="font-mono text-[9px] text-[var(--color-dim)] shrink-0">#{touchdowns.length - i}</div>
                  <div class="flex-1 min-w-0">
                    <div class="font-mono text-[12px] font-bold" style="color:{tColor}">{td.rate.toFixed(0)} fpm</div>
                    <div class="font-mono text-[9px] text-[var(--color-dim)]">{td.speed.toFixed(0)} kt @ {td.time}</div>
                  </div>
                  <div class="font-mono text-[13px] font-extrabold" style="color:{tColor}">{tGrade}</div>
                  <div class="absolute bottom-0 left-0 h-[2px] w-full">
                    <div class="h-full rounded-full" style="width:{Math.min(100, Math.abs(td.rate)/5)}%;background:{tColor}"></div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
          {#if touchdowns.length > 1}
            <div class="flex gap-3 mt-2 pt-2 border-t border-[var(--color-border)]">
              <div class="flex-1 text-center"><span class="font-mono text-[8px] tracking-wide text-[var(--color-dim)] block">AVG</span><strong class="font-mono text-[12px] text-[var(--color-bright)]">{(touchdowns.reduce((s,t) => s + Math.abs(t.rate), 0) / touchdowns.length).toFixed(0)} fpm</strong></div>
              <div class="flex-1 text-center"><span class="font-mono text-[8px] tracking-wide text-[var(--color-dim)] block">BEST</span><strong class="font-mono text-[12px] text-[var(--color-bright)]">{Math.min(...touchdowns.map(t => Math.abs(t.rate))).toFixed(0)} fpm</strong></div>
              <div class="flex-1 text-center"><span class="font-mono text-[8px] tracking-wide text-[var(--color-dim)] block">WORST</span><strong class="font-mono text-[12px] text-[var(--color-bright)]">{Math.max(...touchdowns.map(t => Math.abs(t.rate))).toFixed(0)} fpm</strong></div>
            </div>
          {/if}
        </div>
      </div>

    {:else if debriefTab === 'envelope'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">SPEED ENVELOPE</div>
          <table class="w-full border-collapse text-[11px]">
            <thead><tr><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">PARAMETER</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">RECORDED</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">LIMIT</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">STATUS</th></tr></thead>
            <tbody>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max IAS</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{maxSpd.toFixed(0)} kt</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">163 kt (Vne)</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">{maxSpd > 163 ? '🔴 EXCEEDED' : maxSpd > 129 ? '🟡 CAUTION' : '🟢 OK'}</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max G-Force</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{maxG.toFixed(2)} G</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">3.8 G</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">{maxG > 3.8 ? '🔴 EXCEEDED' : maxG > 2.5 ? '🟡 CAUTION' : '🟢 OK'}</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Min Speed</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">--</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">48 kt (Vs1)</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">{stallWarnings > 0 ? '🟡 STALL WARN' : '🟢 OK'}</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Max Bank</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{maxBank.toFixed(0)}°</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">60° (std)</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">{maxBank > 60 ? '🔴 EXCEEDED' : maxBank > 45 ? '🟡 STEEP' : '🟢 OK'}</td></tr>
            </tbody>
          </table>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">EVENTS</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">OVERSPEED EVENTS</span><span class="font-mono text-lg font-extrabold leading-tight {overspeedEvents > 0 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{overspeedEvents}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">STALL WARNINGS</span><span class="font-mono text-lg font-extrabold leading-tight {stallWarnings > 0 ? 'text-[var(--color-yellow)]' : 'text-[var(--color-bright)]'}">{stallWarnings}</span></div>
          </div>
          {#if overspeedEvents === 0 && stallWarnings === 0}
            <div class="mt-2.5 px-3 py-1.5 rounded font-mono text-[11px] font-extrabold tracking-[2px] text-center bg-[var(--color-green)]/8 border border-[var(--color-green)]/20 text-[var(--color-green)]">CLEAN FLIGHT — NO EXCEEDANCES</div>
          {:else}
            <div class="mt-2.5 px-3 py-1.5 rounded font-mono text-[11px] font-extrabold tracking-[2px] text-center bg-[var(--color-red)]/8 border border-[var(--color-red)]/20 text-[var(--color-red)]">EXCEEDANCES DETECTED</div>
          {/if}
        </div>
      </div>

    {:else if debriefTab === 'score'}
      {@const lGrade = gradeFromRate(landingRate)}
      {@const gGrade = maxG < 1.5 ? 'A' : maxG < 2.0 ? 'B' : maxG < 3.0 ? 'C' : 'F'}
      {@const sGrade = overspeedEvents === 0 && stallWarnings === 0 ? 'A' : overspeedEvents + stallWarnings < 3 ? 'B' : 'C'}
      {@const bGrade = maxBank < 30 ? 'A' : maxBank < 45 ? 'B' : maxBank < 60 ? 'C' : 'F'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">PERFORMANCE GRADES</div>
          <table class="w-full border-collapse text-[11px]">
            <thead><tr><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">CATEGORY</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">GRADE</th><th class="text-left font-mono text-[9px] tracking-wide text-[var(--color-dim)] px-2 py-1 border-b border-[var(--color-border)] font-semibold">DETAIL</th></tr></thead>
            <tbody>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Landing</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50"><span class="font-mono text-[13px] font-extrabold" style="color:{gradeColor(lGrade)}">{lGrade}</span></td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{landingRate.toFixed(0)} fpm</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">G-Force Management</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50"><span class="font-mono text-[13px] font-extrabold" style="color:{gradeColor(gGrade)}">{gGrade}</span></td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{maxG.toFixed(2)} G max</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Speed Discipline</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50"><span class="font-mono text-[13px] font-extrabold" style="color:{gradeColor(sGrade)}">{sGrade}</span></td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{overspeedEvents + stallWarnings} events</td></tr>
              <tr><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50">Bank Control</td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50"><span class="font-mono text-[13px] font-extrabold" style="color:{gradeColor(bGrade)}">{bGrade}</span></td><td class="px-2 py-1.5 border-b border-[var(--color-border)]/50 font-mono text-[var(--color-bright)]">{maxBank.toFixed(0)}° max</td></tr>
            </tbody>
          </table>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">SESSION</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">FLIGHTS</span><span class="font-mono text-lg font-extrabold text-[var(--color-bright)] leading-tight">{logEntries.length}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">TOUCHDOWNS</span><span class="font-mono text-lg font-extrabold text-[var(--color-bright)] leading-tight">{touchdowns.length}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">TOTAL TIME</span><span class="font-mono text-lg font-extrabold text-[var(--color-bright)] leading-tight">{logEntries.reduce((s: number, e: any) => s + (e.duration || 0), 0)} <small class="text-[10px] font-medium text-[var(--color-dim)]">min</small></span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded px-2.5 py-2 flex flex-col"><span class="font-mono text-[8px] tracking-[1.5px] text-[var(--color-dim)]">FUEL USED</span><span class="font-mono text-lg font-extrabold text-[var(--color-bright)] leading-tight">{fuelUsed.toFixed(1)} <small class="text-[10px] font-medium text-[var(--color-dim)]">gal</small></span></div>
          </div>
          <button class="mt-3 px-4 py-1.5 bg-transparent border border-[var(--color-border)] rounded font-mono text-[10px] tracking-wide text-[var(--color-dim)] cursor-pointer transition-all hover:border-[var(--color-red)] hover:text-[var(--color-red)]" onclick={() => { resetDebrief(); toast('Debrief data reset', 'info'); }}>RESET DEBRIEF DATA</button>
        </div>
      </div>
    {/if}
  </div>
</div>
