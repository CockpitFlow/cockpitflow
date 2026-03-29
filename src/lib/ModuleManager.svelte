<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Power } from 'lucide-svelte';

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
  let msg = $state('');

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
      msg = `${mod.name} ${mod.enabled ? 'enabled' : 'disabled'}`;
      setTimeout(() => msg = '', 2000);
    } catch (e: any) {
      msg = 'Error: ' + e;
    }
  }
</script>

<div class="mm">
  <div class="mm-header">
    <div class="label">MODULES</div>
    <p class="desc">Enable or disable modules. Disabled modules are hidden from the sidebar and mobile companion.</p>
  </div>

  <div class="mm-list">
    {#each [...modules].sort((a, b) => (a.order || 99) - (b.order || 99)) as mod}
      <div class="mm-item" class:off={!mod.enabled}>
        <div class="mm-info">
          <span class="mm-name">{mod.name}</span>
          <span class="mm-desc">{mod.description}</span>
        </div>
        <div
          class="mm-switch" class:on={mod.enabled}
          onclick={() => toggle(mod)}
          role="switch"
        ></div>
      </div>
    {/each}
  </div>

  {#if msg}
    <div class="mm-msg">{msg}</div>
  {/if}
</div>

<style>
  .mm { padding: 4px; }
  .mm-header { margin-bottom: 14px; }
  .mm-list { display: flex; flex-direction: column; gap: 4px; }

  .mm-item {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 14px; background: var(--color-surface); border: 1px solid var(--color-border);
    border-radius: 5px; transition: opacity .15s;
  }
  .mm-item.off { opacity: .45; }

  .mm-info { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
  .mm-name { font-weight: 600; font-size: 12px; }
  .mm-desc { font-size: 10px; color: var(--color-dim); line-height: 1.3; }

  .mm-switch {
    width: 36px; height: 18px; border-radius: 9px;
    background: var(--color-surface-2); border: 1px solid var(--color-border);
    position: relative; cursor: pointer; transition: all .15s; flex-shrink: 0;
    margin-left: 12px;
  }
  .mm-switch::after {
    content: ''; position: absolute; width: 14px; height: 14px; border-radius: 50%;
    top: 1px; left: 1px; background: var(--color-dim); transition: all .15s;
  }
  .mm-switch.on { background: rgba(52,208,88,.12); border-color: var(--color-green); }
  .mm-switch.on::after { left: 19px; background: var(--color-green); }

  .mm-msg { font-size: 10px; color: var(--color-green); margin-top: 10px; text-align: center; }
</style>
