<script lang="ts">
  import { ExternalLink, ArrowUpCircle, CheckCircle2, Download, Loader2 } from 'lucide-svelte';

  let {
    connected = false,
    arduinoStatus = '',
    flightActive = false,
    sidebarItems = [],
    updateInfo = null,
    onNavigate = (_id: string) => {},
  }: {
    connected?: boolean;
    arduinoStatus?: string;
    flightActive?: boolean;
    sidebarItems?: any[];
    updateInfo?: { update_available: boolean; current: string; latest: string; download_url: string; installer_url?: string; changelog: string } | null;
    onNavigate?: (id: string) => void;
  } = $props();

  let version = $derived(updateInfo?.current || '1.0.0');
  let hasUpdate = $derived(updateInfo?.update_available ?? false);
  let hwEnabled = $derived(sidebarItems.some(m => m.id === 'hardware'));

  let showUpdateDialog = $state(false);
  let updateStatus = $state<'idle' | 'downloading' | 'done' | 'error'>('idle');
  let updateError = $state('');

  async function installUpdate() {
    if (!updateInfo?.installer_url) return;
    updateStatus = 'downloading';
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('download_and_install_update', { url: updateInfo.installer_url });
      updateStatus = 'done';
    } catch (e: any) {
      updateStatus = 'error';
      updateError = e?.message || String(e);
    }
  }

  const statuses = $derived([
    { on: connected, text: connected ? 'Sim Connected' : 'Sim Offline', show: true },
    { on: arduinoStatus === 'connected', text: arduinoStatus === 'connected' ? 'HW Online' : 'No Hardware', show: hwEnabled },
    { on: flightActive, text: flightActive ? 'In Flight' : 'On Ground', show: true },
  ].filter(s => s.show));
</script>

<div class="w-full h-full overflow-y-auto p-4 flex flex-col gap-3">

  <!-- Row 1: Status badges + version -->
  <div class="flex items-center gap-2 flex-wrap">
    {#each statuses as s}
      <div class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full border text-[11px] font-medium
        {s.on
          ? 'border-[var(--color-green)]/30 bg-[var(--color-green)]/5 text-[var(--color-green)]'
          : 'border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-dim)]'
        }">
        <span class="w-1.5 h-1.5 rounded-full shrink-0
          {s.on ? 'bg-[var(--color-green)] shadow-[0_0_4px_var(--color-green)]' : 'bg-[var(--color-dim)]/50'}"></span>
        {s.text}
      </div>
    {/each}

    <div class="ml-auto"></div>

    <!-- Version / Update badge -->
    {#if hasUpdate}
      <button
        class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full border border-[var(--color-accent)]/30 bg-[var(--color-accent)]/5 text-[var(--color-accent)] text-[11px] font-medium cursor-pointer transition-colors hover:bg-[var(--color-accent)]/10"
        onclick={() => showUpdateDialog = true}
      >
        <ArrowUpCircle size={13} />
        v{updateInfo?.latest} available
      </button>
    {:else}
      <div class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-dim)] text-[11px] font-medium">
        <CheckCircle2 size={13} />
        v{version} — up to date
      </div>
    {/if}
  </div>

  <!-- Row 2: Modules grid -->
  <div class="flex-1 min-h-0 overflow-y-auto">
    <div class="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-2">
      {#each sidebarItems.filter(m => m.id !== 'home') as m}
        <button
          class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2.5 flex items-center gap-2.5 text-left cursor-pointer transition-all hover:border-[var(--color-accent)] hover:bg-[var(--color-accent)]/5"
          onclick={() => onNavigate(m.id)}
        >
          <m.icon size={16} strokeWidth={1.5} class="text-[var(--color-accent)] shrink-0" />
          <span class="text-xs font-semibold text-[var(--color-fg)]">{m.label}</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Row 3: Community -->
  <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md p-4">
    <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-1">COMMUNITY</div>
    <p class="text-[11px] text-[var(--color-dim)] mb-3">Get help, share your setup, request features, or contribute to the project.</p>
    <div class="flex gap-2">
      <a
        href="https://discord.gg/cockpitflow"
        target="_blank"
        class="flex-1 flex items-center gap-3 px-4 py-3 bg-[var(--color-bg)] border border-[var(--color-border)] rounded-lg no-underline text-[13px] font-medium transition-all hover:border-[var(--color-discord)] hover:bg-[var(--color-discord)]/5"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" style="fill: var(--color-discord)"><path d="M20.317 4.37a19.8 19.8 0 00-4.885-1.515.07.07 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.3 18.3 0 00-5.487 0 12.6 12.6 0 00-.617-1.25.08.08 0 00-.079-.037A19.7 19.7 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.08.08 0 00.031.057 19.9 19.9 0 005.993 3.03.08.08 0 00.084-.028 14.1 14.1 0 001.226-1.994.08.08 0 00-.041-.106 13.1 13.1 0 01-1.872-.892.08.08 0 01-.008-.128c.125-.094.25-.192.372-.292a.07.07 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.07.07 0 01.078.01c.12.1.246.198.373.292a.08.08 0 01-.006.127c-.598.35-1.22.645-1.873.892a.08.08 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.08.08 0 00.084.028 19.8 19.8 0 006.002-3.03.08.08 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.06.06 0 00-.031-.03z"/></svg>
        <div class="flex flex-col">
          <span class="text-[var(--color-discord)]">Discord</span>
          <span class="text-[10px] text-[var(--color-dim)]">Join the community</span>
        </div>
        <ExternalLink size={12} class="ml-auto text-[var(--color-dim)]" />
      </a>
      <a
        href="https://github.com/CockpitFlow/cockpitflow"
        target="_blank"
        class="flex-1 flex items-center gap-3 px-4 py-3 bg-[var(--color-bg)] border border-[var(--color-border)] rounded-lg no-underline text-[13px] font-medium transition-all hover:border-[var(--color-bright)] hover:bg-[var(--color-bright)]/5"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="var(--color-bright)"><path d="M12 0C5.374 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0112 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/></svg>
        <div class="flex flex-col">
          <span class="text-[var(--color-bright)]">GitHub</span>
          <span class="text-[10px] text-[var(--color-dim)]">Source & issues</span>
        </div>
        <ExternalLink size={12} class="ml-auto text-[var(--color-dim)]" />
      </a>
    </div>
    <div class="text-[9px] text-[var(--color-dim)]/50 mt-2 flex flex-col gap-0.5">
      <span>CockpitFlow v{version} · Open source cockpit companion ·
        <a href="https://github.com/CockpitFlow/cockpitflow" target="_blank" class="text-[var(--color-accent)] no-underline">GitHub</a> ·
        <a href="https://cockpitflow.github.io/cockpitflow-site/" target="_blank" class="text-[var(--color-accent)] no-underline">Website</a>
      </span>
      <span>For flight simulation use only. Not for real-world aviation.</span>
    </div>
  </div>
</div>

<!-- Update dialog overlay -->
{#if showUpdateDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" role="dialog" onclick={() => { if (updateStatus === 'idle') showUpdateDialog = false; }}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="w-[420px] max-h-[80vh] bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl p-5 shadow-2xl flex flex-col gap-4" onclick={(e) => e.stopPropagation()}>
      <!-- Header -->
      <div class="flex items-center gap-3">
        <ArrowUpCircle size={24} class="text-[var(--color-accent)] shrink-0" />
        <div>
          <h3 class="text-sm font-bold text-[var(--color-bright)]">Update Available</h3>
          <p class="text-[11px] text-[var(--color-dim)]">v{updateInfo?.current} → v{updateInfo?.latest}</p>
        </div>
      </div>

      <!-- Changelog -->
      {#if updateInfo?.changelog}
        <div class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded-lg p-3 max-h-[200px] overflow-y-auto">
          <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2">CHANGELOG</div>
          <pre class="text-[11px] text-[var(--color-fg)] whitespace-pre-wrap leading-relaxed font-sans">{updateInfo.changelog}</pre>
        </div>
      {/if}

      <!-- Status -->
      {#if updateStatus === 'downloading'}
        <div class="flex items-center gap-2 text-[var(--color-accent)] text-xs">
          <Loader2 size={14} class="animate-spin" />
          Downloading update...
        </div>
      {:else if updateStatus === 'done'}
        <div class="flex items-center gap-2 text-[var(--color-green)] text-xs">
          <CheckCircle2 size={14} />
          Installing — app will restart shortly
        </div>
      {:else if updateStatus === 'error'}
        <div class="text-[var(--color-red)] text-xs">
          Update failed: {updateError}
          <a href={updateInfo?.download_url} target="_blank" class="text-[var(--color-accent)] no-underline ml-1">Download manually</a>
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-2 justify-end">
        {#if updateStatus === 'idle'}
          <button
            class="px-4 py-2 bg-transparent border border-[var(--color-border)] rounded-lg text-[var(--color-dim)] text-xs font-semibold cursor-pointer transition-all hover:border-[var(--color-fg)] hover:text-[var(--color-fg)]"
            onclick={() => showUpdateDialog = false}
          >Later</button>
          {#if updateInfo?.installer_url}
            <button
              class="px-4 py-2 bg-[var(--color-accent)] border-none rounded-lg text-white text-xs font-bold cursor-pointer transition-opacity hover:opacity-90 flex items-center gap-1.5"
              onclick={installUpdate}
            >
              <Download size={13} />
              Install Update
            </button>
          {:else}
            <a
              href={updateInfo?.download_url}
              target="_blank"
              class="px-4 py-2 bg-[var(--color-accent)] rounded-lg text-white text-xs font-bold no-underline flex items-center gap-1.5"
            >
              <ExternalLink size={13} />
              Download
            </a>
          {/if}
        {:else if updateStatus === 'done'}
          <button
            class="px-4 py-2 bg-[var(--color-green)] border-none rounded-lg text-white text-xs font-bold cursor-pointer"
            onclick={() => showUpdateDialog = false}
          >OK</button>
        {:else if updateStatus === 'error'}
          <button
            class="px-4 py-2 bg-transparent border border-[var(--color-border)] rounded-lg text-[var(--color-dim)] text-xs font-semibold cursor-pointer"
            onclick={() => { updateStatus = 'idle'; }}
          >Retry</button>
        {/if}
      </div>
    </div>
  </div>
{/if}
