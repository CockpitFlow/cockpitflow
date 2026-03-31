<script lang="ts">
  import Marketplace from '../Marketplace.svelte';
  import { ExternalLink } from 'lucide-svelte';
  import { toast } from '../gauges/Toast.svelte';

  let {
    allLandings = [],
    logEntries = [],
    gradeFromRate = (_r: number) => 'C',
    gradeColor = (_g: string) => 'var(--color-dim)',
    onClearLandings = () => {},
  }: {
    allLandings?: { rate: number; speed: number; time: string; date: string }[];
    logEntries?: any[];
    gradeFromRate?: (r: number) => string;
    gradeColor?: (g: string) => string;
    onClearLandings?: () => void;
  } = $props();

  let communityTab = $state('marketplace');
  let pilotName = $state(localStorage.getItem('sf-pilot') || 'Pilot');
  let pilotCallsign = $state(localStorage.getItem('sf-callsign') || 'N172SP');
  $effect(() => { localStorage.setItem('sf-pilot', pilotName); });
  $effect(() => { localStorage.setItem('sf-callsign', pilotCallsign); });
</script>

<div class="w-full h-full overflow-hidden flex flex-col bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md">
  <div class="flex border-b border-[var(--color-border)] shrink-0">
    {#each [['marketplace','Marketplace'],['leaderboard','Leaderboard'],['profile','Profile'],['connect','Connect'],['share','Share']] as [id,label]}
      <button class="px-4 py-1.5 border-none bg-transparent text-[var(--color-dim)] font-mono text-[10px] font-semibold tracking-wide cursor-pointer border-b-2 border-transparent transition-all hover:text-[var(--color-fg)] {communityTab===id ? 'text-[var(--color-accent)]! border-b-[var(--color-accent)]!' : ''}" onclick={() => communityTab=id}>{label}</button>
    {/each}
  </div>
  <div class="flex-1 overflow-y-auto p-3.5">
    {#if communityTab === 'marketplace'}
      <Marketplace />
    {:else if communityTab === 'leaderboard'}
      {@const sorted = [...allLandings].sort((a,b) => Math.abs(a.rate) - Math.abs(b.rate))}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">PERSONAL BEST LANDINGS</div>
          {#if sorted.length === 0}
            <div class="text-[var(--color-dim)] text-xs py-5">No landings recorded yet. Fly and land to build your leaderboard.</div>
          {:else}
            <div class="flex flex-col gap-0.5">
              {#each sorted.slice(0, 20) as ld, i}
                {@const g = gradeFromRate(ld.rate)}
                <div class="flex items-center gap-2 px-2 py-1.5 rounded font-mono text-[11px] hover:bg-[var(--color-surface-2)] {i < 3 ? 'bg-[var(--color-accent)]/3' : ''}">
                  <span class="w-[22px] text-center font-extrabold text-xs {i===0 ? 'text-[var(--color-gold)]' : i===1 ? 'text-[var(--color-silver)]' : i===2 ? 'text-[var(--color-bronze)]' : 'text-[var(--color-dim)]'}">{i + 1}</span>
                  <span class="font-extrabold text-[13px] w-[70px]" style="color:{gradeColor(g)}">{ld.rate.toFixed(0)} fpm</span>
                  <span class="font-black text-sm w-7 text-center" style="color:{gradeColor(g)}">{g}</span>
                  <span class="text-[var(--color-dim)] text-[10px] w-10">{ld.speed.toFixed(0)} kt</span>
                  <span class="text-[var(--color-dim)] text-[9px] ml-auto">{ld.date}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">STATS</div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 mb-3">
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded p-2 text-center"><span class="block font-mono text-[9px] tracking-wide text-[var(--color-dim)] mb-0.5">TOTAL LANDINGS</span><span class="block font-mono text-lg font-extrabold text-[var(--color-bright)]">{allLandings.length}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded p-2 text-center"><span class="block font-mono text-[9px] tracking-wide text-[var(--color-dim)] mb-0.5">BEST EVER</span><span class="block font-mono text-lg font-extrabold" style="color:var(--color-green)">{allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded p-2 text-center"><span class="block font-mono text-[9px] tracking-wide text-[var(--color-dim)] mb-0.5">AVERAGE</span><span class="block font-mono text-lg font-extrabold text-[var(--color-bright)]">{allLandings.length ? (allLandings.reduce((s,l) => s + Math.abs(l.rate), 0) / allLandings.length).toFixed(0) : '—'}</span></div>
            <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded p-2 text-center"><span class="block font-mono text-[9px] tracking-wide text-[var(--color-dim)] mb-0.5">BUTTER RATE</span><span class="block font-mono text-lg font-extrabold" style="color:var(--color-green)">{allLandings.length ? Math.round(allLandings.filter(l => Math.abs(l.rate) < 60).length / allLandings.length * 100) : 0}%</span></div>
          </div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)] mt-3.5">GRADE DISTRIBUTION</div>
          <div class="flex flex-col gap-1">
            {#each ['A+','A','B','C','D','F'] as g}
              {@const count = allLandings.filter(l => gradeFromRate(l.rate) === g).length}
              {@const pct = allLandings.length ? count / allLandings.length * 100 : 0}
              <div class="flex items-center gap-1.5">
                <span class="font-mono text-xs font-extrabold w-[22px] text-center" style="color:{gradeColor(g)}">{g}</span>
                <div class="flex-1 h-1.5 bg-[var(--color-border)] rounded-full overflow-hidden"><div class="h-full rounded-full transition-[width]" style="width:{pct}%;background:{gradeColor(g)}"></div></div>
                <span class="font-mono text-[10px] text-[var(--color-dim)] w-6 text-right">{count}</span>
              </div>
            {/each}
          </div>
          {#if allLandings.length > 0}
            <button class="mt-3 px-4 py-1.5 bg-transparent border border-[var(--color-border)] text-[var(--color-dim)] rounded font-mono text-[10px] font-bold tracking-wide cursor-pointer hover:border-[var(--color-red)] hover:text-[var(--color-red)] transition-colors" onclick={() => { if (confirm('Clear all landing history?')) { onClearLandings(); toast('Landing history cleared', 'info'); } }}>CLEAR HISTORY</button>
          {/if}
        </div>
      </div>

    {:else if communityTab === 'profile'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">PILOT PROFILE</div>
          <div class="flex flex-col gap-2 mb-3.5">
            <div class="flex flex-col gap-1"><label class="text-[10px] text-[var(--color-dim)] font-mono tracking-wide">Pilot Name</label><input class="h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-2 rounded text-xs focus:outline-none focus:border-[var(--color-accent)]" bind:value={pilotName} placeholder="Your name" /></div>
            <div class="flex flex-col gap-1"><label class="text-[10px] text-[var(--color-dim)] font-mono tracking-wide">Callsign</label><input class="h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-2 rounded text-xs focus:outline-none focus:border-[var(--color-accent)]" bind:value={pilotCallsign} placeholder="N172SP" /></div>
          </div>
          <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded-lg p-3.5">
            <div class="flex items-center gap-2.5 mb-3">
              <div class="w-9 h-9 rounded-full bg-[var(--color-accent)] text-white flex items-center justify-center font-mono text-base font-extrabold shrink-0">{pilotName.charAt(0).toUpperCase()}</div>
              <div><div class="text-sm font-bold text-[var(--color-bright)]">{pilotName}</div><div class="font-mono text-[11px] text-[var(--color-accent)]">{pilotCallsign}</div></div>
            </div>
            <div class="flex gap-3">
              <div class="text-center flex-1"><span class="block font-mono text-lg font-extrabold text-[var(--color-bright)]">{allLandings.length}</span><label class="text-[9px] text-[var(--color-dim)] tracking-wide">Landings</label></div>
              <div class="text-center flex-1"><span class="block font-mono text-lg font-extrabold text-[var(--color-bright)]">{allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'}</span><label class="text-[9px] text-[var(--color-dim)] tracking-wide">Best fpm</label></div>
              <div class="text-center flex-1"><span class="block font-mono text-lg font-extrabold text-[var(--color-bright)]">{logEntries.reduce((s: number, e: any) => s + (e.duration || 0), 0)}</span><label class="text-[9px] text-[var(--color-dim)] tracking-wide">Total min</label></div>
            </div>
          </div>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">DISCORD INTEGRATION</div>
          <div class="flex flex-col items-center gap-1.5 p-5 bg-[var(--color-bg)] border border-[var(--color-border)] rounded-lg text-center">
            <svg width="28" height="28" viewBox="0 0 24 24" style="fill: var(--color-discord)"><path d="M20.317 4.37a19.8 19.8 0 00-4.885-1.515.07.07 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.3 18.3 0 00-5.487 0 12.6 12.6 0 00-.617-1.25.08.08 0 00-.079-.037A19.7 19.7 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.08.08 0 00.031.057 19.9 19.9 0 005.993 3.03.08.08 0 00.084-.028 14.1 14.1 0 001.226-1.994.08.08 0 00-.041-.106 13.1 13.1 0 01-1.872-.892.08.08 0 01-.008-.128c.125-.094.25-.192.372-.292a.07.07 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.07.07 0 01.078.01c.12.1.246.198.373.292a.08.08 0 01-.006.127c-.598.35-1.22.645-1.873.892a.08.08 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.08.08 0 00.084.028 19.8 19.8 0 006.002-3.03.08.08 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.06.06 0 00-.031-.03z"/></svg>
            <span class="text-sm font-bold text-[var(--color-bright)]">Connect Discord</span>
            <span class="text-[11px] text-[var(--color-dim)]">Auto-post landings, share stats, join leaderboards</span>
            <button class="mt-1 px-5 py-1.5 bg-[var(--color-discord)] border-none text-white rounded font-mono text-[10px] font-bold tracking-wider cursor-not-allowed opacity-50" disabled>COMING SOON</button>
          </div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)] mt-3.5">SETTINGS</div>
          <div class="flex flex-col gap-1">
            <label class="flex items-center gap-2 text-[11px] text-[var(--color-dim)] py-0.5 cursor-default"><input type="checkbox" checked disabled class="accent-[var(--color-accent)]" /><span>Auto-post butter landings to Discord</span></label>
            <label class="flex items-center gap-2 text-[11px] text-[var(--color-dim)] py-0.5 cursor-default"><input type="checkbox" checked disabled class="accent-[var(--color-accent)]" /><span>Show Discord Rich Presence</span></label>
            <label class="flex items-center gap-2 text-[11px] text-[var(--color-dim)] py-0.5 cursor-default"><input type="checkbox" disabled class="accent-[var(--color-accent)]" /><span>Share flight stats publicly</span></label>
          </div>
        </div>
      </div>

    {:else if communityTab === 'connect'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">LINKS</div>
          <a href="https://discord.gg/cockpitflow" target="_blank" class="flex items-center gap-2 px-3 py-2 bg-[var(--color-bg)] border border-[var(--color-border)] rounded text-[var(--color-fg)] no-underline font-mono text-[11px] font-semibold mb-1.5 hover:border-[var(--color-accent)] transition-colors"><svg width="16" height="16" viewBox="0 0 24 24" style="fill: var(--color-discord)"><path d="M20.317 4.37a19.8 19.8 0 00-4.885-1.515.07.07 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.3 18.3 0 00-5.487 0 12.6 12.6 0 00-.617-1.25.08.08 0 00-.079-.037A19.7 19.7 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.08.08 0 00.031.057 19.9 19.9 0 005.993 3.03.08.08 0 00.084-.028 14.1 14.1 0 001.226-1.994.08.08 0 00-.041-.106 13.1 13.1 0 01-1.872-.892.08.08 0 01-.008-.128c.125-.094.25-.192.372-.292a.07.07 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.07.07 0 01.078.01c.12.1.246.198.373.292a.08.08 0 01-.006.127c-.598.35-1.22.645-1.873.892a.08.08 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.08.08 0 00.084.028 19.8 19.8 0 006.002-3.03.08.08 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.06.06 0 00-.031-.03z"/></svg><span>Discord Server</span><ExternalLink size={12} /></a>
          <a href="https://github.com/cockpitflow" target="_blank" class="flex items-center gap-2 px-3 py-2 bg-[var(--color-bg)] border border-[var(--color-border)] rounded text-[var(--color-fg)] no-underline font-mono text-[11px] font-semibold mb-1.5 hover:border-[var(--color-accent)] transition-colors"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.6 9.6 0 0112 6.844a9.6 9.6 0 012.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg><span>GitHub</span><ExternalLink size={12} /></a>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">WEBHOOK</div>
          <div class="flex flex-col gap-1"><label class="text-[10px] text-[var(--color-dim)] font-mono tracking-wide">Discord Webhook URL</label><input class="h-6 bg-[var(--color-bg)] border border-[var(--color-border)] text-[var(--color-fg)] px-2 rounded text-xs focus:outline-none focus:border-[var(--color-accent)] disabled:opacity-50" placeholder="https://discord.com/api/webhooks/..." disabled /></div>
          <p class="text-[10px] text-[var(--color-dim)] mt-1.5">Paste a Discord channel webhook URL to auto-post your landings and achievements.</p>
        </div>
      </div>

    {:else if communityTab === 'share'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">SHARE CARD PREVIEW</div>
          <div class="bg-gradient-to-br from-[var(--color-surface)] to-[var(--color-surface-2)] border border-[var(--color-border)] rounded-lg p-4">
            <div class="flex justify-between items-center mb-3.5">
              <span class="font-mono text-sm font-black tracking-[3px] text-[var(--color-accent)]">COCKPITFLOW</span>
              <span class="font-mono text-xs text-[var(--color-dim)]">{pilotCallsign}</span>
            </div>
            <div class="grid grid-cols-4 gap-2 mb-3">
              <div class="text-center"><span class="block font-mono text-xl font-black text-[var(--color-bright)]">{allLandings.length}</span><span class="font-mono text-[8px] tracking-wider text-[var(--color-dim)]">LANDINGS</span></div>
              <div class="text-center"><span class="block font-mono text-xl font-black" style="color:var(--color-green)">{allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'}</span><span class="font-mono text-[8px] tracking-wider text-[var(--color-dim)]">BEST FPM</span></div>
              <div class="text-center"><span class="block font-mono text-xl font-black text-[var(--color-bright)]">{allLandings.length ? (allLandings.reduce((s,l) => s + Math.abs(l.rate), 0) / allLandings.length).toFixed(0) : '—'}</span><span class="font-mono text-[8px] tracking-wider text-[var(--color-dim)]">AVG FPM</span></div>
              <div class="text-center"><span class="block font-mono text-xl font-black" style="color:var(--color-green)">{allLandings.length ? Math.round(allLandings.filter(l => Math.abs(l.rate) < 60).length / allLandings.length * 100) : 0}%</span><span class="font-mono text-[8px] tracking-wider text-[var(--color-dim)]">BUTTER</span></div>
            </div>
            <div class="text-[9px] text-[var(--color-dim)] text-center border-t border-[var(--color-border)] pt-2">{pilotName} - CockpitFlow</div>
          </div>
        </div>
        <div>
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 pb-1 border-b border-[var(--color-border)]">EXPORT</div>
          <button class="w-full px-4 py-2 bg-[var(--color-accent)] border-none text-white rounded font-mono text-[11px] font-bold tracking-wide cursor-pointer hover:opacity-85 transition-opacity" onclick={() => { navigator.clipboard.writeText(`CockpitFlow Stats | ${pilotCallsign}\nLandings: ${allLandings.length}\nBest: ${allLandings.length ? Math.min(...allLandings.map(l => Math.abs(l.rate))).toFixed(0) : '—'} fpm\nButter rate: ${allLandings.length ? Math.round(allLandings.filter(l => Math.abs(l.rate) < 60).length / allLandings.length * 100) : 0}%`); }}>COPY STATS TO CLIPBOARD</button>
          <button class="w-full mt-1.5 px-4 py-2 bg-transparent border border-[var(--color-border)] text-[var(--color-fg)] rounded font-mono text-[11px] font-bold tracking-wide cursor-pointer hover:opacity-85 transition-opacity" onclick={() => { const csv = 'Rate,Speed,Time,Date\n' + allLandings.map(l => `${l.rate},${l.speed},${l.time},${l.date}`).join('\n'); const blob = new Blob([csv],{type:'text/csv'}); const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'cockpitflow-landings.csv'; a.click(); }}>EXPORT LANDINGS CSV</button>
        </div>
      </div>
    {/if}
  </div>
</div>
