<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from './gauges/Toast.svelte';

  interface Module {
    id: string;
    name: string;
    version: string;
    description: string;
    icon: string;
    enabled: boolean;
    order: number;
  }

  let { onRefresh = () => {} }: { onRefresh?: () => void } = $props();

  let modules = $state<Module[]>([]);

  onMount(load);

  async function load() {
    try { modules = await invoke('list_modules') as Module[]; } catch { modules = []; }
  }

  async function toggle(mod: Module) {
    try {
      await invoke('update_module', { moduleId: mod.id, field: 'enabled', value: !mod.enabled });
      mod.enabled = !mod.enabled;
      modules = [...modules];
      onRefresh();
      toast(`${mod.name} ${mod.enabled ? 'enabled' : 'disabled'}`, mod.enabled ? 'success' : 'info');
    } catch (e: any) {
      toast('Error: ' + e, 'error');
    }
  }
</script>

<div class="w-full h-full overflow-y-auto p-4">
  <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-1">MODULES</div>
  <p class="text-[11px] text-[var(--color-dim)] mb-4">Enable or disable modules. Disabled modules are hidden from the sidebar and mobile companion.</p>

  <div class="grid grid-cols-[repeat(auto-fill,minmax(250px,1fr))] gap-3">
    {#each [...modules].sort((a, b) => (a.order || 99) - (b.order || 99)) as mod}
      <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl p-4 flex flex-col gap-3 transition-opacity {mod.enabled ? '' : 'opacity-40'}">
        <div class="flex items-start justify-between gap-3">
          <div class="flex flex-col gap-1 flex-1 min-w-0">
            <span class="font-semibold text-sm text-[var(--color-fg)]">{mod.name}</span>
            <span class="text-[11px] text-[var(--color-dim)] leading-snug">{mod.description}</span>
          </div>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="w-10 h-5 rounded-full border cursor-pointer transition-all shrink-0 relative mt-0.5 {mod.enabled ? 'bg-[var(--color-green)]/12 border-[var(--color-green)]' : 'bg-[var(--color-surface-2)] border-[var(--color-border)]'}"
            role="switch"
            aria-checked={mod.enabled}
            onclick={() => toggle(mod)}
          >
            <div class="absolute w-4 h-4 rounded-full top-[1.5px] transition-all {mod.enabled ? 'left-[21px] bg-[var(--color-green)]' : 'left-[1.5px] bg-[var(--color-dim)]'}"></div>
          </div>
        </div>
        <div class="flex items-center gap-2 text-[9px] text-[var(--color-dim)] font-mono">
          <span>v{mod.version || '1.0.0'}</span>
          <span class="opacity-30">·</span>
          <span>{mod.enabled ? 'Enabled' : 'Disabled'}</span>
        </div>
      </div>
    {/each}
  </div>
</div>
