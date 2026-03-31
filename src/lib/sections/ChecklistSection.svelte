<script lang="ts">
  import { onMount } from 'svelte';
  import { Separator } from 'bits-ui';
  import { Smartphone, Copy, Check } from 'lucide-svelte';
  import { toast } from '../gauges/Toast.svelte';
  import QRCode from 'qrcode';

  let { lanIp = '...', connected = false }: { lanIp?: string; connected?: boolean } = $props();

  let lanCopied = $state<string>('');
  let clMode = $state<'strict' | 'smart'>('smart');
  let clAutoCheck = $state<'on' | 'off'>('on');
  let clFeedback = $state<'on' | 'off'>('on');
  let clHaptic = $state<'on' | 'off'>('on');
  let clSound = $state<'on' | 'off'>('on');
  let clCheckMethod = $state<'tap' | 'swipe'>('tap');
  let clAutoAdvance = $state<'on' | 'off'>('on');

  let qrDataUrl = $state('');
  let clPresets = $state<string[]>([]);
  let clActivePreset = $state('cessna-172');
  let clPresetNames = $state<Record<string, string>>({});
  let clInfo = $state<{ name: string; author: string; version: string; category: string; phases: number; items: number } | null>(null);

  function copyUrl(url: string, key: string) {
    navigator.clipboard.writeText(url);
    lanCopied = key;
    setTimeout(() => lanCopied = '', 2000);
  }

  async function generateQR(url: string) {
    try {
      const style = getComputedStyle(document.documentElement);
      const accent = style.getPropertyValue('--color-accent').trim() || '#4a9eff';
      const bg = style.getPropertyValue('--color-bg').trim() || '#0a0e14';
      qrDataUrl = await QRCode.toDataURL(url, { width: 140, margin: 1, color: { dark: accent, light: bg } });
    } catch {}
  }

  async function syncSettings() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('set_settings', { newSettings: {
          mode: clMode, auto_check: clAutoCheck, check_feedback: clFeedback,
          haptic: clHaptic, sound: clSound, check_method: clCheckMethod,
          auto_advance: clAutoAdvance, data_mode: 'live', web_theme: 'dark',
          active_checklist: clActivePreset,
        }});
        toast('Settings saved', 'success');
      }
    } catch {}
  }

  async function loadChecklistPresets() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        const mods = await invoke('list_modules') as any[];
        const cl = mods.find(m => m.id === 'checklist');
        if (cl) { clPresets = cl.presets || []; clActivePreset = cl.default_preset || 'cessna-172'; }
      }
    } catch {}
  }

  async function loadAllPresetNames() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        const names: Record<string, string> = {};
        for (const id of clPresets) {
          try {
            const json = await invoke('load_module_preset', { moduleId: 'checklist', presetId: id }) as string;
            const data = JSON.parse(json);
            names[id] = `${data.name || id} (${data.author || 'Unknown'} v${data.version || '1.0'})`;
          } catch { names[id] = id; }
        }
        clPresetNames = names;
      }
    } catch {}
  }

  async function loadChecklistInfo() {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        const json = await invoke('load_module_preset', { moduleId: 'checklist', presetId: clActivePreset }) as string;
        const data = JSON.parse(json);
        const items = (data.phases || []).reduce((s: number, p: any) => s + (p.items?.length || 0), 0);
        clInfo = { name: data.name || clActivePreset, author: data.author || 'Unknown', version: data.version || '1.0', category: data.category || '', phases: (data.phases || []).length, items };
      }
    } catch { clInfo = null; }
  }

  async function setChecklistPreset(preset: string) {
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_module', { moduleId: 'checklist', field: 'default_preset', value: preset });
        clActivePreset = preset;
        loadChecklistInfo();
        generateQR(`http://${lanIp}:8080/checklist`);
        toast('Aircraft changed to ' + preset, 'success');
      }
    } catch {}
  }

  async function deleteChecklistPreset() {
    if (clPresets.length <= 1) { toast('Cannot delete the only preset', 'warning'); return; }
    if (!confirm(`Delete checklist "${clActivePreset}"?`)) return;
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('delete_module_preset', { moduleId: 'checklist', presetId: clActivePreset });
        toast('Deleted: ' + clActivePreset, 'info');
        await loadChecklistPresets();
        await loadAllPresetNames();
        await loadChecklistInfo();
      }
    } catch {}
  }

  async function importChecklistPreset() {
    const input = document.createElement('input');
    input.type = 'file'; input.accept = '.json';
    input.onchange = async () => {
      const file = input.files?.[0]; if (!file) return;
      try {
        const text = await file.text(); JSON.parse(text);
        const name = file.name.replace('.json', '').toLowerCase().replace(/[^a-z0-9-]/g, '-');
        if ('__TAURI_INTERNALS__' in window) {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('save_module_preset', { moduleId: 'checklist', presetId: name, data: text });
          loadChecklistPresets();
          toast('Checklist imported: ' + name, 'success');
        }
      } catch {}
    };
    input.click();
  }

  onMount(() => {
    loadChecklistPresets().then(() => { loadAllPresetNames(); loadChecklistInfo(); });
    setTimeout(() => generateQR(`http://${lanIp}:8080/checklist`), 2000);

    // Regenerate QR when theme changes
    const obs = new MutationObserver(() => generateQR(`http://${lanIp}:8080/checklist`));
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
    return () => obs.disconnect();
  });

  // Toggle helper types
  type OnOff = 'on' | 'off';
  const toggles = $derived([
    { label: 'Auto Check', value: clAutoCheck, set: (v: OnOff) => { clAutoCheck = v; syncSettings(); } },
    { label: 'Feedback', value: clFeedback, set: (v: OnOff) => { clFeedback = v; syncSettings(); } },
    { label: 'Haptic', value: clHaptic, set: (v: OnOff) => { clHaptic = v; syncSettings(); } },
    { label: 'Sounds', value: clSound, set: (v: OnOff) => { clSound = v; syncSettings(); } },
  ]);
</script>

{#snippet toggleBtn(active: boolean, label: string, onclick: () => void)}
  <button
    class="px-3 py-1 border-none font-mono text-[10px] font-bold cursor-pointer transition-all {active ? 'bg-[var(--color-accent)] text-white' : 'bg-transparent text-[var(--color-dim)]'}"
    onclick={onclick}
  >{label}</button>
{/snippet}

<div class="w-full h-full overflow-y-auto p-4 flex flex-col gap-3">

  <!-- Row 1: Aircraft + Mode -->
  <div class="flex gap-3 flex-wrap">
    <!-- Aircraft -->
    <div class="flex-1 min-w-[220px] bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md p-4">
      <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5">AIRCRAFT</div>
      <select
        class="w-full px-2.5 py-1.5 bg-[var(--color-bg)] border border-[var(--color-border)] rounded text-[var(--color-fg)] font-mono text-xs outline-none focus:border-[var(--color-accent)] mb-2"
        value={clActivePreset}
        onchange={(e) => setChecklistPreset((e.target as HTMLSelectElement).value)}
      >
        {#each clPresets as p}
          <option value={p}>{clPresetNames[p] || p}</option>
        {/each}
      </select>
      {#if clInfo}
        <div class="flex flex-col gap-px mt-1.5">
          <span class="text-[13px] font-semibold text-[var(--color-fg)]">{clInfo.name}</span>
          <span class="text-[10px] text-[var(--color-dim)]">by {clInfo.author} · v{clInfo.version}{clInfo.category ? ` · ${clInfo.category}` : ''}</span>
          <span class="text-[10px] text-[var(--color-dim)]">{clInfo.phases} phases · {clInfo.items} items</span>
        </div>
      {/if}
      <button
        class="mt-2 py-1.5 w-full bg-transparent border border-dashed border-[var(--color-border)] rounded text-[var(--color-dim)] text-[10px] cursor-pointer transition-all hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
        onclick={importChecklistPreset}
      >Import checklist from file</button>
      {#if clPresets.length > 1}
        <button
          class="mt-1 py-1 w-full bg-transparent border border-[var(--color-border)] rounded text-[var(--color-dim)] text-[10px] cursor-pointer transition-all hover:border-[var(--color-red)] hover:text-[var(--color-red)]"
          onclick={deleteChecklistPreset}
        >Delete current preset</button>
      {/if}
    </div>

    <!-- Mode -->
    <div class="flex-1 min-w-[220px] bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md p-4">
      <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5">MODE</div>
      <div class="flex gap-1">
        {#each [['strict', 'Strict'], ['smart', 'Smart']] as [id, label]}
          <button
            class="flex-1 py-2 bg-[var(--color-bg)] border rounded text-[11px] font-semibold cursor-pointer text-center transition-all {clMode === id ? 'border-[var(--color-accent)] text-[var(--color-accent)] bg-[var(--color-accent)]/5' : 'border-[var(--color-border)] text-[var(--color-dim)]'}"
            onclick={() => { clMode = id as any; syncSettings(); }}
          >{label}</button>
        {/each}
      </div>
      <p class="text-[10px] text-[var(--color-dim)] leading-snug mt-2">{clMode === 'strict' ? 'Items must be checked in order. Current item highlighted, rest locked.' : 'Check items in any order. Flexible for experienced pilots.'}</p>
    </div>
  </div>

  <!-- Row 2: Auto-Detect + Interaction -->
  <div class="flex gap-3 flex-wrap">
    <!-- Auto-Detect -->
    <div class="flex-1 min-w-[220px] bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md p-4">
      <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5">AUTO-DETECT</div>
      <div class="flex flex-col gap-2">
        {#each toggles as t}
          <div class="flex items-center justify-between text-xs">
            <span class="text-[var(--color-fg)]">{t.label}</span>
            <div class="flex border border-[var(--color-border)] rounded overflow-hidden">
              {@render toggleBtn(t.value === 'on', 'ON', () => t.set('on'))}
              {@render toggleBtn(t.value === 'off', 'OFF', () => t.set('off'))}
            </div>
          </div>
        {/each}
      </div>
      <p class="text-[10px] text-[var(--color-dim)] leading-snug mt-2">Auto Check from sim. Feedback dots. Haptic vibration. Sounds on check/phase.</p>
    </div>

    <!-- Interaction -->
    <div class="flex-1 min-w-[220px] bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md p-4">
      <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5">INTERACTION</div>
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between text-xs">
          <span class="text-[var(--color-fg)]">Check method</span>
          <div class="flex border border-[var(--color-border)] rounded overflow-hidden">
            {@render toggleBtn(clCheckMethod === 'tap', 'TAP', () => { clCheckMethod = 'tap'; syncSettings(); })}
            {@render toggleBtn(clCheckMethod === 'swipe', 'SWIPE', () => { clCheckMethod = 'swipe'; syncSettings(); })}
          </div>
        </div>
        <div class="flex items-center justify-between text-xs">
          <span class="text-[var(--color-fg)]">Auto-advance</span>
          <div class="flex border border-[var(--color-border)] rounded overflow-hidden">
            {@render toggleBtn(clAutoAdvance === 'on', 'ON', () => { clAutoAdvance = 'on'; syncSettings(); })}
            {@render toggleBtn(clAutoAdvance === 'off', 'OFF', () => { clAutoAdvance = 'off'; syncSettings(); })}
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Row 3: Open on phone -->
  <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md p-4">
    <div class="font-mono text-[9px] font-bold tracking-[1.5px] text-[var(--color-dim)] mb-2.5 flex items-center gap-2">
      <Smartphone size={14} />
      OPEN ON PHONE / TABLET
    </div>
    <div class="flex items-start gap-4 flex-wrap">
      <!-- QR -->
      {#if qrDataUrl}
        <img class="w-[90px] h-[90px] rounded-md border border-[var(--color-border)] shrink-0" src={qrDataUrl} alt="QR">
      {:else}
        <div class="w-[90px] h-[90px] rounded-md border border-[var(--color-border)] shrink-0 flex items-center justify-center text-[var(--color-dim)] text-[9px]">QR</div>
      {/if}

      <!-- URLs -->
      <div class="flex flex-col gap-2 flex-1 min-w-[200px]">
        {#each [
          { label: 'Browser', url: `http://${lanIp}:8080/checklist`, key: 'browser' },
          { label: 'App', url: `http://${lanIp}:8080`, key: 'app' },
        ] as link}
          <div class="flex items-center gap-2">
            <span class="text-[9px] font-bold text-[var(--color-dim)] tracking-wide w-[50px] shrink-0">{link.label}</span>
            <span class="font-mono text-xs font-semibold text-[var(--color-accent)]">{link.url}</span>
            <button
              class="px-2.5 py-0.5 bg-transparent border border-[var(--color-border)] rounded font-mono text-[10px] font-bold tracking-wide text-[var(--color-dim)] cursor-pointer transition-all hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
              onclick={() => copyUrl(link.url, link.key)}
            >
              {#if lanCopied === link.key}
                <span class="flex items-center gap-1 text-[var(--color-green)]"><Check size={10} />COPIED</span>
              {:else}
                <span class="flex items-center gap-1"><Copy size={10} />COPY</span>
              {/if}
            </button>
          </div>
        {/each}
      </div>

      <!-- Status -->
      <div class="flex items-center gap-1.5 text-[10px] text-[var(--color-dim)] self-end">
        <span class="w-1.5 h-1.5 rounded-full shrink-0 {connected ? 'bg-[var(--color-green)] shadow-[0_0_4px_var(--color-green)]' : 'bg-[var(--color-red)]'}"></span>
        <span>{connected ? 'Sim connected' : 'Waiting for sim'}</span>
        <span class="opacity-30">|</span>
        <span>{clActivePreset}</span>
        <span class="opacity-30">|</span>
        <span>{clMode}</span>
      </div>
    </div>
  </div>
</div>
