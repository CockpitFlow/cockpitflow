<script lang="ts">
  let { simData = {} }: { simData?: any } = $props();

  let mag = $state(0);
  let mAlt = $state(false);
  let mBat = $state(false);
  let fuel = $state(false);
  let bcn = $state(false);
  let land = $state(false);
  let taxi = $state(false);
  let nav = $state(false);
  let strb = $state(false);
  let pitot = $state(false);
  let av1 = $state(false);
  let av2 = $state(false);

  let userClickTime = $state(0);

  $effect(() => {
    if (simData?.connected && Date.now() - userClickTime > 1000) {
      mAlt = simData.master_alt > 0.5;
      mBat = simData.master_battery > 0.5;
      fuel = simData.fuel_pump > 0.5;
      bcn = simData.beacon > 0.5;
      land = simData.landing_light > 0.5;
      taxi = simData.taxi_light > 0.5;
      nav = simData.nav_light > 0.5;
      strb = simData.strobe > 0.5;
      pitot = simData.pitot_heat > 0.5;
      av1 = simData.avionics_master > 0.5;
      av2 = simData.avionics_master > 0.5;
      mag = Math.round(simData.magneto_key || 0);
      if (mag > 4) mag = 4;
    }
  });

  const ML = ['OFF','R','L','BOTH','START'];
  const MA = [-135,-90,-45,0,45];
  let ka = $derived(MA[mag] * Math.PI / 180);

  async function cmd(c: string, v: number) {
    userClickTime = Date.now();
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('send_command', { cmd: c, value: v });
      }
    } catch {}
  }

  function toggleSwitch(current: boolean, command: string, setter: (v: boolean) => void) {
    const next = !current;
    setter(next);
    cmd(command, next ? 1 : 0);
  }

  // Circuit breakers
  let cbs = $state<Record<string,boolean>>({
    flap:true, inst:true, av1:true, av2:true, turn:true, lts:true, afld:true, warn:true
  });
</script>

<svg viewBox="0 0 920 210" class="ep" xmlns="http://www.w3.org/2000/svg">
  <!-- Panel background - dark textured metal -->
  <defs>
    <linearGradient id="panelGrad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#2a2a2a"/>
      <stop offset="50%" stop-color="#1e1e1e"/>
      <stop offset="100%" stop-color="#222"/>
    </linearGradient>
    <filter id="innerShadow">
      <feOffset dx="0" dy="1"/>
      <feGaussianBlur stdDeviation="1"/>
      <feComposite operator="out" in="SourceGraphic"/>
      <feColorMatrix values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.4 0"/>
      <feBlend in="SourceGraphic"/>
    </filter>
  </defs>
  <rect x="0" y="0" width="920" height="210" rx="6" fill="url(#panelGrad)" stroke="#444" stroke-width="1.5"/>

  <!-- Panel screws -->
  {#each [[12,12],[908,12],[12,198],[908,198],[460,198]] as [sx,sy]}
    <circle cx={sx} cy={sy} r="5" fill="#252525" stroke="#3a3a3a" stroke-width="0.8"/>
    <line x1={sx-2.5} y1={sy-2.5} x2={sx+2.5} y2={sy+2.5} stroke="#333" stroke-width="0.6"/>
  {/each}

  <!-- ===== MAGNETO SWITCH (far left) ===== -->
  <text x="65" y="22" fill="#aaa" font-size="9" text-anchor="middle" font-weight="600" letter-spacing="1.5">MAGNETOS</text>

  <!-- Outer ring -->
  <circle cx="65" cy="100" r="38" fill="#141414" stroke="#444" stroke-width="2"/>
  <circle cx="65" cy="100" r="34" fill="#111" stroke="#333" stroke-width="1"/>

  <!-- Position marks + labels -->
  {#each MA as angle, i}
    {@const ax = 65 + 30 * Math.cos(angle * Math.PI/180)}
    {@const ay = 100 + 30 * Math.sin(angle * Math.PI/180)}
    {@const lx = 65 + 43 * Math.cos(angle * Math.PI/180)}
    {@const ly = 100 + 43 * Math.sin(angle * Math.PI/180)}
    <line x1={65 + 26 * Math.cos(angle * Math.PI/180)} y1={100 + 26 * Math.sin(angle * Math.PI/180)}
          x2={ax} y2={ay}
          stroke={i===mag ? '#4af' : '#555'} stroke-width="2"/>
    <text x={lx} y={ly} fill={i===mag ? '#4af' : '#666'} font-size="7" text-anchor="middle"
          dominant-baseline="middle" font-weight={i===mag ? '700' : '400'}>{ML[i]}</text>
  {/each}

  <!-- Key body -->
  <line x1="65" y1="100" x2={65+22*Math.cos(ka)} y2={100+22*Math.sin(ka)}
    stroke="#888" stroke-width="3.5" stroke-linecap="round"/>
  <circle cx="65" cy="100" r="5" fill="#333" stroke="#555" stroke-width="1"/>
  <!-- Key head -->
  <line x1={65+22*Math.cos(ka)} y1={100+22*Math.sin(ka)} x2={65+38*Math.cos(ka)} y2={100+38*Math.sin(ka)}
    stroke="#777" stroke-width="3" stroke-linecap="round"/>
  <circle cx={65+40*Math.cos(ka)} cy={100+40*Math.sin(ka)} r="5" fill="none" stroke="#555" stroke-width="1"/>

  <!-- Click zones -->
  <circle cx="65" cy="100" r="42" fill="transparent" cursor="pointer"
    onmousedown={() => { mag = Math.min(mag + 1, 4); cmd('MAGNETO', mag); }}
    onmouseup={() => { if (mag === 4) { mag = 3; cmd('MAGNETO', 3); } }}
    onmouseleave={() => { if (mag === 4) { mag = 3; cmd('MAGNETO', 3); } }}
    oncontextmenu={(e) => { e.preventDefault(); mag = Math.max(mag - 1, 0); cmd('MAGNETO', mag); }}
    role="button" tabindex="0" onkeydown={() => {}}
  />

  <!-- ===== MASTER SWITCH (red split rocker) ===== -->
  <text x="155" y="22" fill="#aaa" font-size="9" text-anchor="middle" font-weight="600" letter-spacing="1.5">MASTER</text>
  <text x="155" y="34" fill="#666" font-size="7" text-anchor="middle">ON</text>

  {#each [[140, mAlt, 'ALT', 'MASTER_ALT', (v: boolean) => mAlt=v],
          [170, mBat, 'BAT', 'MASTER_BATTERY', (v: boolean) => mBat=v]] as [x, on, label, c, setter]}
    <!-- Switch body -->
    <rect x={x-13} y="40" width="26" height="60" rx="3" fill="#0c0c0c" stroke="#444" stroke-width="1.2"/>
    <!-- Rocker -->
    <rect x={x-11} y={on ? 42 : 68} width="22" height="28" rx="2"
      fill={on ? '#b81c1c' : '#5a1a1a'} stroke={on ? '#d44' : '#3a1a1a'} stroke-width="1"/>
    {#if on}
      <rect x={x-8} y="44" width="16" height="6" rx="1.5" fill="rgba(255,120,120,0.25)"/>
    {/if}
    <text x={x} y="115" fill="#999" font-size="7" text-anchor="middle" font-weight="600">{label}</text>
    <!-- Click -->
    <rect x={x-13} y="40" width="26" height="60" fill="transparent" cursor="pointer"
      onclick={() => toggleSwitch(on, c, setter)}
      role="button" tabindex="0" onkeydown={() => {}}
    />
  {/each}

  <!-- ===== FUEL PUMP ===== -->
  <text x="220" y="22" fill="#aaa" font-size="8" text-anchor="middle" font-weight="600">FUEL</text>
  <text x="220" y="32" fill="#aaa" font-size="8" text-anchor="middle" font-weight="600">PUMP</text>

  <!-- Toggle switch -->
  <rect x="213" y="95" width="14" height="48" rx="7" fill="#111" stroke="#444" stroke-width="1"/>
  <circle cx="220" cy={fuel ? 104 : 134} r="7"
    fill={fuel ? '#e0e0e0' : '#666'} stroke={fuel ? '#fff' : '#555'} stroke-width="1" cursor="pointer"
    onclick={() => toggleSwitch(fuel, 'FUEL_PUMP', (v) => fuel=v)}
  />
  <text x="220" y="155" fill={fuel ? '#4af' : '#555'} font-size="6" text-anchor="middle">{fuel ? 'ON' : 'OFF'}</text>

  <!-- ===== CIRCUIT BREAKERS ROW ===== -->
  {#each [
    [270,'FLAP','flap',10], [320,'INST','inst',5], [370,'AVN\nBUS1','av1',15],
    [420,'AVN\nBUS2','av2',15], [470,'TURN\nCOORD','turn',5], [520,'INST\nLTS','lts',5]
  ] as [cx, label, key, amps]}
    <text x={cx} y="20" fill="#999" font-size="7" text-anchor="middle" font-weight="600">
      {#each label.split('\n') as line, li}
        <tspan x={cx} dy={li === 0 ? 0 : 10}>{line}</tspan>
      {/each}
    </text>
    <circle cx={cx} cy="52" r="12" fill={cbs[key] ? '#1a1a1a' : '#252525'}
      stroke={cbs[key] ? '#555' : '#4af'} stroke-width="1.5" cursor="pointer"
      onclick={() => cbs[key] = !cbs[key]}
    />
    <text x={cx} y="56" fill={cbs[key] ? '#888' : '#4af'} font-size="10" text-anchor="middle" font-weight="700">{amps}</text>
  {/each}

  <!-- LIGHTS label -->
  <text x="395" y="80" fill="#777" font-size="7" text-anchor="middle" letter-spacing="3" font-weight="600">LIGHTS</text>
  <line x1="280" y1="83" x2="350" y2="83" stroke="#444" stroke-width="0.5"/>
  <line x1="440" y1="83" x2="510" y2="83" stroke="#444" stroke-width="0.5"/>

  <!-- ===== LIGHT TOGGLE SWITCHES ===== -->
  {#each [
    [295, bcn, 'BCN', 'BEACON', (v: boolean) => bcn=v],
    [345, land, 'LAND', 'LANDING_LIGHT', (v: boolean) => land=v],
    [395, taxi, 'TAXI', 'TAXI_LIGHT', (v: boolean) => taxi=v],
    [445, nav, 'NAV', 'NAV_LIGHT', (v: boolean) => nav=v],
    [495, strb, 'STRB', 'STROBE', (v: boolean) => strb=v],
  ] as [cx, on, label, c, setter]}
    <rect x={cx-6} y="92" width="12" height="46" rx="6" fill="#111" stroke="#444" stroke-width="1"/>
    <circle cx={cx} cy={on ? 100 : 130} r="6.5"
      fill={on ? '#ddd' : '#666'} stroke={on ? '#fff' : '#555'} stroke-width="0.8" cursor="pointer"
      onclick={() => toggleSwitch(on, c, setter)}
    />
    {#if on}<circle cx={cx} cy="100" r="9" fill="none" stroke="rgba(74,158,255,0.15)" stroke-width="1.5"/>{/if}
    <text x={cx} y="152" fill={on ? '#4af' : '#666'} font-size="7" text-anchor="middle" font-weight="600">{label}</text>
    <text x={cx} y="163" fill="#444" font-size="5" text-anchor="middle">{on ? 'ON' : 'OFF'}</text>
  {/each}

  <!-- ===== PITOT HEAT ===== -->
  <text x="560" y="82" fill="#999" font-size="7" text-anchor="middle" font-weight="600">PITOT</text>
  <text x="560" y="92" fill="#999" font-size="7" text-anchor="middle" font-weight="600">HEAT</text>
  <rect x="554" y="98" width="12" height="46" rx="6" fill="#111" stroke="#444" stroke-width="1"/>
  <circle cx="560" cy={pitot ? 106 : 136} r="6.5"
    fill={pitot ? '#ddd' : '#666'} stroke={pitot ? '#fff' : '#555'} stroke-width="0.8" cursor="pointer"
    onclick={() => toggleSwitch(pitot, 'PITOT_HEAT', (v) => pitot=v)}
  />
  <text x="560" y="158" fill={pitot ? '#4af' : '#555'} font-size="6" text-anchor="middle">{pitot ? 'ON' : 'OFF'}</text>

  <!-- Separator -->
  <line x1="595" y1="15" x2="595" y2="195" stroke="#444" stroke-width="0.5" stroke-dasharray="2,3"/>

  <!-- ===== AVIONICS MASTER ===== -->
  <text x="655" y="22" fill="#aaa" font-size="9" text-anchor="middle" font-weight="600" letter-spacing="1.5">AVIONICS</text>
  <text x="655" y="38" fill="#666" font-size="7" text-anchor="middle">ON</text>

  <!-- Big toggle switch -->
  <rect x="643" y="48" width="24" height="56" rx="3" fill="#0c0c0c" stroke="#444" stroke-width="1.2"/>
  <rect x="646" y={av1 ? 50 : 76} width="18" height="24" rx="2"
    fill={av1 ? '#ddd' : '#666'} stroke={av1 ? '#fff' : '#444'} stroke-width="0.8"/>
  {#if av1}<rect x="648" y="52" width="14" height="5" rx="1.5" fill="rgba(255,255,255,0.2)"/>{/if}
  <rect x="643" y="48" width="24" height="56" fill="transparent" cursor="pointer"
    onclick={() => toggleSwitch(av1, 'AVIONICS_MASTER', (v) => { av1=v; av2=v; })}
    role="button" tabindex="0" onkeydown={() => {}}
  />

  <!-- Separator -->
  <line x1="700" y1="15" x2="700" y2="195" stroke="#444" stroke-width="0.5" stroke-dasharray="2,3"/>

  <!-- ===== RIGHT CBs ===== -->
  {#each [[740,'ALT\nFLD','afld',5], [790,'WARN','warn',5]] as [cx, label, key, amps]}
    <text x={cx} y="20" fill="#999" font-size="7" text-anchor="middle" font-weight="600">
      {#each label.split('\n') as line, li}
        <tspan x={cx} dy={li === 0 ? 0 : 10}>{line}</tspan>
      {/each}
    </text>
    <circle cx={cx} cy="52" r="12" fill={cbs[key] ? '#1a1a1a' : '#252525'}
      stroke={cbs[key] ? '#555' : '#4af'} stroke-width="1.5" cursor="pointer"
      onclick={() => cbs[key] = !cbs[key]}
    />
    <text x={cx} y="56" fill={cbs[key] ? '#888' : '#4af'} font-size="10" text-anchor="middle" font-weight="700">{amps}</text>
  {/each}

  <!-- ===== BUS 1/2 ROCKERS ===== -->
  {#each [[850, av1, 'BUS 1'], [890, av2, 'BUS 2']] as [x, on, label]}
    <rect x={x-13} y="40" width="26" height="60" rx="3" fill="#0c0c0c" stroke="#444" stroke-width="1.2"/>
    <rect x={x-11} y={on ? 42 : 68} width="22" height="28" rx="2"
      fill={on ? '#ddd' : '#555'} stroke={on ? '#fff' : '#444'} stroke-width="0.8"/>
    {#if on}<rect x={x-8} y="44" width="16" height="5" rx="1.5" fill="rgba(255,255,255,0.2)"/>{/if}
    <text x={x} y="115" fill="#888" font-size="7" text-anchor="middle" font-weight="600">{label}</text>
    <rect x={x-13} y="40" width="26" height="60" fill="transparent" cursor="pointer"
      onclick={() => { if(label==='BUS 1'){av1=!av1} else {av2=!av2}; cmd('AVIONICS_MASTER',(av1||av2)?1:0); }}
      role="button" tabindex="0" onkeydown={() => {}}
    />
  {/each}

  <!-- Branding -->
  <text x="905" y="200" fill="#333" font-size="6" text-anchor="end" font-style="italic">SkyFlow</text>
</svg>

<style>
  .ep {
    width: 100%;
    max-width: 920px;
    filter: drop-shadow(0 4px 16px rgba(0,0,0,0.5));
    user-select: none;
  }
  .ep text {
    font-family: 'Segoe UI', system-ui, sans-serif;
  }
</style>
