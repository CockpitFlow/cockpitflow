<script lang="ts">
  import { toast } from '../gauges/Toast.svelte';

  let {
    failures = [],
    failureGroups = [],
    sendCmd = (_cmd: string, _val: number) => {},
    onWheel = (_e: WheelEvent) => {},
  }: {
    failures?: { id: string; label: string; group: string; desc: string; severity: string[] }[];
    failureGroups?: string[];
    sendCmd?: (cmd: string, val: number) => void;
    onWheel?: (e: WheelEvent) => void;
  } = $props();

  let tab = $state<'generator' | 'active'>('generator');

  interface ActiveFailure {
    id: string; label: string; severity: string;
    trigger: 'immediate' | 'delay' | 'random';
    delayMin: number; delayMax: number;
    armed: boolean; fired: boolean; fireTime: number;
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
    toast(f.label + ' armed (' + sev + ')', 'warning');
  }

  function armRandom() {
    const pool = failures.filter(f => !activeFailures.some(a => a.id === f.id));
    for (let i = 0; i < Math.min(randomCount, pool.length); i++) {
      const idx = Math.floor(Math.random() * pool.length);
      const f = pool.splice(idx, 1)[0];
      const sev = f.severity[Math.floor(Math.random() * f.severity.length)];
      armFailure(f, sev);
    }
    toast('Armed ' + Math.min(randomCount, pool.length) + ' random failures', 'warning');
  }

  function clearFailures() { activeFailures = []; toast('All failures cleared', 'info'); }
</script>

<div class="w-full h-full overflow-hidden flex flex-col bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md">
  <div class="flex border-b border-[var(--color-border)] shrink-0">
    {#each [['generator','Failure Generator'],['active','Active (' + activeFailures.length + ')']] as [id,label]}
      <button class="px-4 py-1.5 border-none bg-transparent text-[var(--color-dim)] font-mono text-[10px] font-semibold tracking-wide cursor-pointer border-b-2 border-transparent transition-all hover:text-[var(--color-fg)] {tab===id ? 'text-[var(--color-accent)]! border-b-[var(--color-accent)]!' : ''}" onclick={() => tab=id as any}>{label}</button>
    {/each}
  </div>
  <div class="flex-1 overflow-y-auto p-3.5">
    {#if tab === 'generator'}
      <div class="grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-4">
        <!-- LEFT: Timing + Random -->
        <div class="flex flex-col gap-3">
          <div>
            <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">TIMING</div>
            <div class="flex flex-col gap-1.5">
              {#each [['immediate','Immediate','Triggers now'],['delay','Delayed','After set minutes'],['random','Random','Random time in range']] as [id,label,desc]}
                <button class="flex flex-col gap-0.5 px-3 py-2.5 bg-[var(--color-bg)] border border-[var(--color-border)] rounded cursor-pointer text-left text-[var(--color-dim)] text-[11px] transition-all {failureTrigger===id ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/5' : ''}" onclick={() => failureTrigger=id as any}>
                  <strong class="text-xs {failureTrigger===id ? 'text-[var(--color-accent)]' : 'text-[var(--color-fg)]'}">{label}</strong><span>{desc}</span>
                </button>
              {/each}
            </div>
          </div>

          {#if failureTrigger === 'delay'}
            <div class="flex items-center gap-2 text-[var(--color-dim)] text-[11px]">
              <span>Delay</span>
              <div class="flex"><input type="number" onwheel={onWheel} class="w-16 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[11px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={failureDelayMin} min="1" max="120" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">min</span></div>
            </div>
          {:else if failureTrigger === 'random'}
            <div class="flex items-center gap-2 text-[var(--color-dim)] text-[11px]">
              <span>Between</span>
              <div class="flex"><input type="number" onwheel={onWheel} class="w-16 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[11px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={failureDelayMin} min="1" max="120" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">min</span></div>
              <span>and</span>
              <div class="flex"><input type="number" onwheel={onWheel} class="w-16 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[11px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={failureDelayMax} min="1" max="120" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">min</span></div>
            </div>
          {/if}

          <div>
            <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">RANDOM</div>
            <div class="flex items-center gap-2 text-[var(--color-dim)] text-[11px]">
              <span>Count</span>
              <div class="flex"><input type="number" onwheel={onWheel} class="w-16 h-6 bg-[var(--color-bg)] border border-[var(--color-border)] border-r-0 text-[var(--color-accent)] px-1.5 rounded-l font-mono text-[11px] text-right focus:outline-none focus:border-[var(--color-accent)]" bind:value={randomCount} min="1" max="5" /><span class="h-6 px-1.5 flex items-center bg-[var(--color-surface-2)] border border-[var(--color-border)] rounded-r font-mono text-[9px] text-[var(--color-dim)]">x</span></div>
            </div>
            <button class="px-4 py-2 bg-[var(--color-accent)] border-none text-white rounded font-mono text-[11px] font-bold tracking-wide cursor-pointer transition-opacity hover:opacity-85 mt-2 w-full" onclick={armRandom}>ARM RANDOM</button>
          </div>
        </div>

        <!-- RIGHT: Failure list -->
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">FAILURES</div>
          <div class="flex flex-col max-h-[calc(100vh-200px)] overflow-y-auto">
            {#each failureGroups as group}
              <div class="font-mono text-[9px] font-bold tracking-wide text-[var(--color-dim)] py-2 pb-1 border-b border-[var(--color-border)] mt-1">{group}</div>
              {#each failures.filter(f => f.group === group) as f}
                {@const isArmed = activeFailures.some(a => a.id === f.id)}
                <div class="flex items-center gap-1.5 px-1.5 py-1 border-b border-[var(--color-border)]/30 text-[11px] hover:bg-[var(--color-surface-2)] {isArmed ? 'opacity-50' : ''}">
                  <span class="text-[var(--color-fg)] font-semibold w-[140px] shrink-0">{f.label}</span>
                  <span class="text-[var(--color-dim)] text-[10px] flex-1">{f.desc}</span>
                  {#if isArmed}
                    <span class="font-mono text-[9px] font-bold text-[var(--color-yellow)] tracking-wide">ARMED</span>
                  {:else}
                    <div class="flex gap-1 shrink-0">
                      {#each f.severity as sev}
                        <button class="px-2 py-0.5 border border-[var(--color-border)] bg-transparent text-[var(--color-dim)] rounded-sm font-mono text-[9px] cursor-pointer transition-all hover:border-[var(--color-red)] hover:text-[var(--color-red)] hover:bg-[var(--color-red)]/5" onclick={() => armFailure(f, sev)}>{sev}</button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            {/each}
          </div>
        </div>
      </div>

    {:else if tab === 'active'}
      {#if activeFailures.length === 0}
        <div class="text-[var(--color-dim)] text-xs py-4">No failures armed. Use the generator to arm failures.</div>
      {:else}
        <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">ARMED FAILURES</div>
        <div class="flex flex-col gap-1.5">
          {#each activeFailures as af, i}
            {@const remaining = Math.max(0, Math.round((af.fireTime - Date.now()) / 60000))}
            {@const fired = Date.now() >= af.fireTime}
            <div class="flex items-center gap-2.5 px-2.5 py-2 bg-[var(--color-bg)] border rounded {fired ? 'border-[var(--color-red)]/40 bg-[var(--color-red)]/3' : 'border-[var(--color-border)]'}">
              <span class="w-2 h-2 rounded-full shrink-0 {fired ? 'bg-[var(--color-green)] shadow-[0_0_6px_var(--color-green)]' : 'bg-[var(--color-yellow)] shadow-[0_0_3px_var(--color-yellow)]'}"></span>
              <div class="flex-1 min-w-0">
                <span class="text-xs font-semibold text-[var(--color-fg)] block">{af.label}</span>
                <span class="text-[9px] text-[var(--color-dim)] font-mono">{af.severity} | {af.trigger === 'immediate' ? 'Now' : af.trigger === 'delay' ? af.delayMin + 'm delay' : af.delayMin + '-' + af.delayMax + 'm random'}</span>
              </div>
              <span class="font-mono text-[11px] font-bold shrink-0 {fired ? 'text-[var(--color-red)]' : 'text-[var(--color-yellow)]'}">{fired ? 'ACTIVE' : 'T-' + remaining + 'm'}</span>
              <button class="w-5 h-5 border border-[var(--color-border)] bg-transparent text-[var(--color-dim)] rounded text-[11px] cursor-pointer flex items-center justify-center shrink-0 hover:border-[var(--color-red)] hover:text-[var(--color-red)]" onclick={() => activeFailures = activeFailures.filter((_,j) => j !== i)}>x</button>
            </div>
          {/each}
        </div>
        <button class="mt-3 px-4 py-1.5 bg-transparent border border-[var(--color-border)] rounded font-mono text-[10px] tracking-wide text-[var(--color-dim)] cursor-pointer transition-all hover:border-[var(--color-red)] hover:text-[var(--color-red)]" onclick={clearFailures}>CLEAR ALL</button>
      {/if}
    {/if}
  </div>
</div>
