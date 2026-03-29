<script lang="ts">
  let { simData = {} }: { simData?: any } = $props();

  let throttle = $state(0);
  let mixture = $state(100);
  let carbHeat = $state(0);
  let flaps = $state(0);
  let trim = $state(50);
  let fuelSel = $state(2);

  // Sync from sim
  $effect(() => {
    if (simData?.connected) {
      throttle = Math.round((simData.throttle || 0) * 100);
      mixture = Math.round((simData.mixture || 0) * 100);
      carbHeat = Math.round((simData.carb_heat || 0) * 100);
      // Flap ratio 0-1 to notch 0-3
      const fr = simData.flap_ratio || 0;
      flaps = fr < 0.1 ? 0 : fr < 0.4 ? 1 : fr < 0.7 ? 2 : 3;
      // Trim: -1..+1 to 0..100
      trim = Math.round(((simData.elevator_trim || 0) + 1) * 50);
      // Fuel selector: 0=OFF,1=LEFT,3=RIGHT,4=BOTH -> our 0,1,3,2
      const fs = Math.round(simData.fuel_selector || 0);
      fuelSel = fs === 4 ? 2 : fs === 3 ? 3 : fs === 1 ? 1 : 0;
    }
  });

  const flapLabels = ['UP', '10°', '20°', 'FULL'];
  const fuelLabels = ['OFF', 'LEFT', 'BOTH', 'RIGHT'];

  // Derived positions (can't use @const in SVG)
  let ty = $derived(153 - throttle * 1.2);
  let my = $derived(153 - mixture * 1.2);
  let chy = $derived(153 - carbHeat * 1.2);
  let fy = $derived(40 + flaps * 33);
  let trimY = $derived(35 + trim * 1.1);
  let fa = $derived([-90, 180, 0, 90][fuelSel] * Math.PI / 180);

  let dragging: string | null = $state(null);

  async function cmd(c: string, v: number) {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('send_command', { cmd: c, value: v });
      }
    } catch {}
  }

  function onMouseDown(id: string) { dragging = id; }

  function onMouseMove(e: MouseEvent) {
    if (!dragging) return;
    const svg = (e.currentTarget as SVGSVGElement);
    const rect = svg.getBoundingClientRect();
    const y = (e.clientY - rect.top) / rect.height;

    if (dragging === 'throttle') {
      throttle = Math.round(Math.max(0, Math.min(100, (1 - y) * 130 - 15)));
      cmd('THROTTLE', throttle / 100);
    } else if (dragging === 'mixture') {
      mixture = Math.round(Math.max(0, Math.min(100, (1 - y) * 130 - 15)));
      cmd('MIXTURE', mixture / 100);
    } else if (dragging === 'carbheat') {
      carbHeat = Math.round(Math.max(0, Math.min(100, (1 - y) * 130 - 15)));
      cmd('CARB_HEAT', carbHeat / 100);
    } else if (dragging === 'trim') {
      trim = Math.round(Math.max(0, Math.min(100, y * 120 - 10)));
      cmd('TRIM', (trim - 50) / 50);
    }
  }

  function onMouseUp() { dragging = null; }
</script>

<svg viewBox="0 0 700 200" class="lc"
  onmousemove={onMouseMove}
  onmouseup={onMouseUp}
  onmouseleave={onMouseUp}
  role="application"
>
  <!-- Panel background -->
  <rect x="0" y="0" width="700" height="200" rx="4" fill="#111" stroke="#333" stroke-width="2"/>

  <!-- Screws -->
  {#each [[8,8],[692,8],[8,192],[692,192]] as [sx,sy]}
    <circle cx={sx} cy={sy} r="4" fill="#222" stroke="#444" stroke-width="0.5"/>
    <line x1={sx-2} y1={sy-2} x2={sx+2} y2={sy+2} stroke="#333" stroke-width="0.5"/>
    <line x1={sx+2} y1={sy-2} x2={sx-2} y2={sy+2} stroke="#333" stroke-width="0.5"/>
  {/each}

  <!-- ===== THROTTLE ===== -->
  <text x="60" y="18" fill="#888" font-size="8" text-anchor="middle" font-weight="bold" letter-spacing="1">THROTTLE</text>

  <!-- Slot -->
  <rect x="53" y="28" width="14" height="130" rx="7" fill="#0a0a0a" stroke="#333" stroke-width="1"/>
  <!-- Track marks -->
  {#each [0,25,50,75,100] as pct}
    <line x1="42" y1={153 - pct * 1.2} x2="50" y2={153 - pct * 1.2} stroke="#444" stroke-width="0.8"/>
    <text x="38" y={156 - pct * 1.2} fill="#555" font-size="5" text-anchor="end">{pct}</text>
  {/each}

  <!-- Knob -->
  <rect x="48" y={ty - 12} width="24" height="24" rx="3" fill="#111" stroke="#444" stroke-width="1.5"
    cursor="ns-resize" onmousedown={() => onMouseDown('throttle')}/>
  <rect x="50" y={ty - 10} width="20" height="20" rx="2"
    fill="url(#throttleGrad)" cursor="ns-resize" onmousedown={() => onMouseDown('throttle')}/>
  <!-- Grip lines -->
  {#each [-4,-1,2,5] as ly}
    <line x1="53" y1={ty + ly} x2="67" y2={ty + ly} stroke="rgba(0,0,0,0.3)" stroke-width="0.8"/>
  {/each}
  <text x="60" y="175" fill="#0af" font-size="7" text-anchor="middle" font-weight="bold">{throttle}%</text>

  <!-- ===== MIXTURE ===== -->
  <text x="140" y="18" fill="#888" font-size="8" text-anchor="middle" font-weight="bold" letter-spacing="1">MIXTURE</text>

  <rect x="133" y="28" width="14" height="130" rx="7" fill="#0a0a0a" stroke="#333" stroke-width="1"/>
  {#each [0,25,50,75,100] as pct}
    <line x1="150" y1={153 - pct * 1.2} x2="158" y2={153 - pct * 1.2} stroke="#444" stroke-width="0.8"/>
  {/each}
  <text x="162" y="38" fill="#555" font-size="5">LEAN</text>
  <text x="162" y="156" fill="#555" font-size="5">RICH</text>

  <rect x="128" y={my - 12} width="24" height="24" rx="3" fill="#111" stroke="#444" stroke-width="1.5"
    cursor="ns-resize" onmousedown={() => onMouseDown('mixture')}/>
  <rect x="130" y={my - 10} width="20" height="20" rx="2"
    fill="url(#mixtureGrad)" cursor="ns-resize" onmousedown={() => onMouseDown('mixture')}/>
  {#each [-4,-1,2,5] as ly}
    <line x1="133" y1={my + ly} x2="147" y2={my + ly} stroke="rgba(0,0,0,0.3)" stroke-width="0.8"/>
  {/each}
  <text x="140" y="175" fill="#ff4444" font-size="7" text-anchor="middle" font-weight="bold">{mixture}%</text>

  <!-- ===== CARB HEAT ===== -->
  <text x="220" y="18" fill="#888" font-size="8" text-anchor="middle" font-weight="bold" letter-spacing="1">CARB HEAT</text>

  <rect x="213" y="28" width="14" height="130" rx="7" fill="#0a0a0a" stroke="#333" stroke-width="1"/>
  <rect x="208" y={chy - 12} width="24" height="24" rx="3" fill="#111" stroke="#444" stroke-width="1.5"
    cursor="ns-resize" onmousedown={() => onMouseDown('carbheat')}/>
  <rect x="210" y={chy - 10} width="20" height="20" rx="2"
    fill="url(#carbGrad)" cursor="ns-resize" onmousedown={() => onMouseDown('carbheat')}/>
  {#each [-4,-1,2,5] as ly}
    <line x1="213" y1={chy + ly} x2="227" y2={chy + ly} stroke="rgba(0,0,0,0.3)" stroke-width="0.8"/>
  {/each}
  <text x="220" y="175" fill="#ffaa00" font-size="7" text-anchor="middle" font-weight="bold">{carbHeat}%</text>

  <!-- Separator -->
  <line x1="270" y1="15" x2="270" y2="185" stroke="#333" stroke-width="0.5"/>

  <!-- ===== FLAPS ===== -->
  <text x="330" y="18" fill="#888" font-size="8" text-anchor="middle" font-weight="bold" letter-spacing="1">FLAPS</text>

  <!-- Flap slot -->
  <rect x="323" y="30" width="14" height="120" rx="3" fill="#0a0a0a" stroke="#333" stroke-width="1"/>

  <!-- Flap notches + click areas combined -->
  {#each [0,1,2,3] as pos}
    <g cursor="pointer" onclick={() => { flaps = pos; cmd('FLAPS', pos / 3); }} role="button" tabindex="0" onkeydown={() => {}}>
      <rect x="310" y={28 + pos * 33} width="60" height="28" fill="transparent"/>
      <line x1="340" y1={40 + pos * 33} x2="352" y2={40 + pos * 33} stroke={flaps===pos ? '#0af' : '#444'} stroke-width={flaps===pos ? 1.5 : 0.8}/>
      <text x="356" y={40 + pos * 33 + 3} fill={flaps===pos ? '#0af' : '#666'} font-size="7" font-weight={flaps===pos ? 'bold' : 'normal'}>{flapLabels[pos]}</text>
    </g>
  {/each}

  <!-- Flap handle -->
  <rect x="319" y={fy-8} width="22" height="16" rx="2" fill="#222" stroke={flaps>0 ? '#0af' : '#444'} stroke-width="1.5"/>
  <rect x="321" y={fy-6} width="18" height="12" rx="1" fill="#1a1a1a"/>
  <line x1="325" y1={fy} x2="335" y2={fy} stroke="#555" stroke-width="1"/>

  <!-- Separator -->
  <line x1="390" y1="15" x2="390" y2="185" stroke="#333" stroke-width="0.5"/>

  <!-- ===== TRIM WHEEL ===== -->
  <text x="445" y="18" fill="#888" font-size="8" text-anchor="middle" font-weight="bold" letter-spacing="1">TRIM</text>

  <!-- Trim indicator -->
  <rect x="428" y="30" width="6" height="120" rx="3" fill="#0a0a0a" stroke="#333" stroke-width="0.8"/>
  <text x="424" y="38" fill="#555" font-size="5" text-anchor="end">ND</text>
  <text x="424" y="92" fill="#555" font-size="5" text-anchor="end">—</text>
  <text x="424" y="148" fill="#555" font-size="5" text-anchor="end">NU</text>

  <rect x="426" y={trimY-4} width="10" height="8" rx="1" fill="#0af" stroke="#088" stroke-width="0.5"/>

  <!-- Trim wheel -->
  <circle cx="460" cy="95" r="30" fill="#1a1a1a" stroke="#333" stroke-width="1.5"/>
  <circle cx="460" cy="95" r="26" fill="#111"/>
  <!-- Wheel ridges -->
  {#each Array(16) as _, i}
    <line
      x1={460 + 20 * Math.cos((i / 16) * Math.PI * 2 + (trim / 10))} y1={95 + 20 * Math.sin((i / 16) * Math.PI * 2 + (trim / 10))}
      x2={460 + 26 * Math.cos((i / 16) * Math.PI * 2 + (trim / 10))} y2={95 + 26 * Math.sin((i / 16) * Math.PI * 2 + (trim / 10))}
      stroke="#444" stroke-width="1.5" stroke-linecap="round"
    />
  {/each}
  <circle cx="460" cy="95" r="8" fill="#222" stroke="#444" stroke-width="1"/>

  <!-- Trim value -->
  <text x="460" y="140" fill="#0af" font-size="7" text-anchor="middle" font-weight="bold">
    {trim < 48 ? 'NOSE DN' : trim > 52 ? 'NOSE UP' : 'NEUTRAL'}
  </text>

  <!-- Trim drag area -->
  <rect x="438" y="28" width="10" height="125" fill="transparent" cursor="ns-resize"
    onmousedown={() => onMouseDown('trim')}
    role="application"
  />

  <!-- Separator -->
  <line x1="510" y1="15" x2="510" y2="185" stroke="#333" stroke-width="0.5"/>

  <!-- ===== FUEL SELECTOR ===== -->
  <text x="600" y="18" fill="#888" font-size="8" text-anchor="middle" font-weight="bold" letter-spacing="1">FUEL SELECTOR</text>

  <!-- Selector housing -->
  <circle cx="600" cy="100" r="38" fill="#1a1a1a" stroke="#444" stroke-width="2"/>
  <circle cx="600" cy="100" r="34" fill="#111"/>

  <!-- Position labels -->
  {#each [[-90,'OFF',0], [180,'LEFT',1], [0,'BOTH',2], [90,'RIGHT',3]] as [angle, label, pos]}
    <text
      x={600 + 45 * Math.cos((angle as number) * Math.PI / 180)} y={100 + 45 * Math.sin((angle as number) * Math.PI / 180) + 3}
      fill={(fuelSel === pos) ? '#0af' : '#555'} font-size="6" text-anchor="middle"
      font-weight={(fuelSel === pos) ? 'bold' : 'normal'}
    >{label}</text>
    <line
      x1={600 + 28 * Math.cos((angle as number) * Math.PI / 180)} y1={100 + 28 * Math.sin((angle as number) * Math.PI / 180)}
      x2={600 + 33 * Math.cos((angle as number) * Math.PI / 180)} y2={100 + 33 * Math.sin((angle as number) * Math.PI / 180)}
      stroke={(fuelSel === pos) ? '#0af' : '#444'} stroke-width="1.5"
    />
  {/each}

  <!-- Selector pointer -->
  <line x1="600" y1="100"
    x2={600 + 24 * Math.cos(fa)} y2={100 + 24 * Math.sin(fa)}
    stroke="#0af" stroke-width="3" stroke-linecap="round"/>
  <circle cx="600" cy="100" r="6" fill="#222" stroke="#0af" stroke-width="1.5"/>

  <!-- Fuel selector handle -->
  <rect
    x={600 + 15 * Math.cos(fa) - 6} y={100 + 15 * Math.sin(fa) - 4}
    width="12" height="8" rx="2"
    fill="#333" stroke="#555" stroke-width="0.8"
    transform="rotate({[-90,180,0,90][fuelSel]}, {600 + 15 * Math.cos(fa)}, {100 + 15 * Math.sin(fa)})"
  />

  <!-- Click areas for fuel positions -->
  {#each [[600,55,0],[555,100,1],[600,100,2],[645,100,3]] as [fx,fcy,pos]}
    <circle cx={fx} cy={fcy} r="25" fill="transparent" cursor="pointer"
      style="pointer-events: all"
      onclick={() => { fuelSel = pos as number; cmd('FUEL_SEL', [0, 1, 4, 3][pos as number]); }}
      role="button" tabindex="0" onkeydown={() => {}}
    />
  {/each}

  <!-- Gradients -->
  <defs>
    <linearGradient id="throttleGrad" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#222"/>
      <stop offset="50%" stop-color="#333"/>
      <stop offset="100%" stop-color="#222"/>
    </linearGradient>
    <linearGradient id="mixtureGrad" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#441111"/>
      <stop offset="50%" stop-color="#661111"/>
      <stop offset="100%" stop-color="#441111"/>
    </linearGradient>
    <linearGradient id="carbGrad" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#332200"/>
      <stop offset="50%" stop-color="#553300"/>
      <stop offset="100%" stop-color="#332200"/>
    </linearGradient>
  </defs>

  <!-- Branding -->
  <text x="690" y="192" fill="#333" font-size="6" text-anchor="end" font-style="italic">SkyFlow</text>
</svg>

<style>
  .lc {
    width: 100%;
    max-width: 900px;
    filter: drop-shadow(0 4px 16px rgba(0,0,0,0.6));
    user-select: none;
  }
  .lc text {
    font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
  }
</style>
