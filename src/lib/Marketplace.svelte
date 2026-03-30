<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Download, Search, Filter, RefreshCw, Check, ExternalLink } from 'lucide-svelte';

  interface MarketItem {
    id: string;
    category: string;
    name: string;
    author: string;
    version: string;
    aircraft?: string;
    simulator?: string;
    tags?: string[];
    file: string;
    description: string;
  }

  interface MarketIndex {
    version: number;
    updated: string;
    categories: Record<string, { label: string; description: string; path: string }>;
    items: MarketItem[];
  }

  const REPO_RAW = 'https://raw.githubusercontent.com/CockpitFlow/cockpitflow-data/main';

  let index = $state<MarketIndex | null>(null);
  let loading = $state(true);
  let error = $state('');
  let search = $state('');
  let filterCat = $state('all');
  let downloading = $state<Record<string, boolean>>({});
  let installed = $state<Record<string, boolean>>({});
  let msg = $state('');

  // Module ID mapping from category
  const categoryToModule: Record<string, string> = {
    checklists: 'checklist',
    cockpit: 'cockpit',
    hardware: 'hardware',
    scenarios: 'scenarios',
  };

  onMount(loadIndex);

  async function loadIndex() {
    loading = true;
    error = '';
    try {
      const resp = await fetch(`${REPO_RAW}/index.json`, { cache: 'no-cache' });
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      index = await resp.json();
      await checkInstalled();
    } catch (e: any) {
      error = `Could not load marketplace: ${e.message}`;
    }
    loading = false;
  }

  async function checkInstalled() {
    try {
      const modules = await invoke('list_modules') as any[];
      const inst: Record<string, boolean> = {};
      for (const mod of modules) {
        for (const preset of (mod.presets || [])) {
          inst[`${mod.id}/${preset}`] = true;
        }
      }
      installed = inst;
    } catch {}
  }

  function isInstalled(item: MarketItem): boolean {
    const modId = categoryToModule[item.category] || item.category;
    return !!installed[`${modId}/${item.id}`];
  }

  async function downloadItem(item: MarketItem) {
    downloading[item.id] = true;
    downloading = { ...downloading };
    try {
      // Fetch JSON from GitHub
      const resp = await fetch(`${REPO_RAW}/${item.file}`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = await resp.text();

      // Save via Tauri invoke
      const modId = categoryToModule[item.category] || item.category;
      await invoke('save_module_preset', {
        moduleId: modId,
        presetId: item.id,
        data: data,
      });

      installed[`${modId}/${item.id}`] = true;
      installed = { ...installed };
      msg = `Installed "${item.name}"`;
      setTimeout(() => msg = '', 3000);
    } catch (e: any) {
      msg = `Error: ${e.message}`;
    }
    downloading[item.id] = false;
    downloading = { ...downloading };
  }

  let filteredItems = $derived(
    (index?.items || []).filter(item => {
      if (filterCat !== 'all' && item.category !== filterCat) return false;
      if (search) {
        const q = search.toLowerCase();
        return item.name.toLowerCase().includes(q)
          || item.description.toLowerCase().includes(q)
          || item.author.toLowerCase().includes(q)
          || (item.aircraft || '').toLowerCase().includes(q)
          || (item.tags || []).some(t => t.includes(q));
      }
      return true;
    })
  );

  let categories = $derived(
    Object.entries(index?.categories || {}).map(([id, cat]) => ({ id, ...cat }))
  );
</script>

<div class="mp">
  <!-- Header -->
  <div class="mp-header">
    <div class="mp-title-row">
      <div>
        <div class="label">MARKETPLACE</div>
        <p class="desc">Community presets from <a href="https://github.com/CockpitFlow/cockpitflow-data" target="_blank" class="mp-link">cockpitflow-data <ExternalLink size={10} /></a></p>
      </div>
      <button class="mp-btn" onclick={loadIndex}>
        <RefreshCw size={12} /> Refresh
      </button>
    </div>

    <!-- Search + Filter -->
    <div class="mp-filters">
      <div class="mp-search-wrap">
        <Search size={13} />
        <input class="mp-search" placeholder="Search presets..." bind:value={search}>
      </div>
      <div class="mp-filter-btns">
        <button class="mp-filter" class:on={filterCat === 'all'} onclick={() => filterCat = 'all'}>All</button>
        {#each categories as cat}
          <button class="mp-filter" class:on={filterCat === cat.id} onclick={() => filterCat = cat.id}>{cat.label}</button>
        {/each}
      </div>
    </div>
  </div>

  <!-- Content -->
  {#if loading}
    <div class="mp-loading">Loading marketplace...</div>
  {:else if error}
    <div class="mp-error">{error}</div>
  {:else}
    <div class="mp-grid">
      {#each filteredItems as item}
        <div class="mp-card" class:installed={isInstalled(item)}>
          <div class="mp-card-top">
            <span class="mp-card-cat">{index?.categories[item.category]?.label || item.category}</span>
            {#if item.simulator}
              <span class="mp-card-sim">{item.simulator === 'both' ? 'X-Plane + MSFS' : item.simulator === 'xplane' ? 'X-Plane' : 'MSFS'}</span>
            {/if}
          </div>
          <div class="mp-card-name">{item.name}</div>
          <div class="mp-card-author">by {item.author} &middot; v{item.version}</div>
          <p class="mp-card-desc">{item.description}</p>
          {#if item.tags?.length}
            <div class="mp-card-tags">
              {#each item.tags as tag}
                <span class="mp-tag">{tag}</span>
              {/each}
            </div>
          {/if}
          <div class="mp-card-footer">
            {#if isInstalled(item)}
              <span class="mp-installed"><Check size={12} /> Installed</span>
            {:else if downloading[item.id]}
              <span class="mp-downloading">Downloading...</span>
            {:else}
              <button class="mp-btn primary" onclick={() => downloadItem(item)}>
                <Download size={12} /> Install
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    {#if filteredItems.length === 0}
      <div class="mp-empty">No presets found{search ? ` for "${search}"` : ''}.</div>
    {/if}

    <div class="mp-footer">
      <span class="mp-count">{filteredItems.length} of {index?.items.length || 0} items</span>
      {#if index?.updated}
        <span class="mp-updated">Updated: {index.updated}</span>
      {/if}
    </div>
  {/if}

  {#if msg}
    <div class="mp-toast">{msg}</div>
  {/if}
</div>

<style>
  .mp { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  .mp-header { padding: 12px 14px; border-bottom: 1px solid var(--color-border); flex-shrink: 0; }
  .mp-title-row { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 10px; }
  .mp-link { color: var(--color-accent); text-decoration: none; font-size: 11px; }
  .mp-link:hover { text-decoration: underline; }

  .mp-filters { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .mp-search-wrap {
    display: flex; align-items: center; gap: 6px;
    padding: 5px 10px; background: var(--color-surface-2);
    border: 1px solid var(--color-border); border-radius: 4px;
    color: var(--color-dim); flex: 1; min-width: 150px;
  }
  .mp-search {
    background: none; border: none; color: var(--color-fg);
    font-size: 12px; outline: none; flex: 1; font-family: inherit;
  }
  .mp-filter-btns { display: flex; gap: 3px; }
  .mp-filter {
    padding: 4px 10px; background: var(--color-surface-2);
    border: 1px solid var(--color-border); border-radius: 3px;
    color: var(--color-dim); font-size: 10px; font-weight: 600;
    cursor: pointer; font-family: inherit;
  }
  .mp-filter.on { border-color: var(--color-accent); color: var(--color-accent); background: rgba(74,158,255,.06); }

  .mp-btn {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 5px 12px; background: var(--color-surface-2);
    border: 1px solid var(--color-border); border-radius: 4px;
    color: var(--color-dim); font-size: 11px; font-weight: 600;
    cursor: pointer; font-family: inherit;
  }
  .mp-btn:hover { border-color: var(--color-dim); color: var(--color-fg); }
  .mp-btn.primary { background: rgba(74,158,255,.08); border-color: rgba(74,158,255,.3); color: var(--color-accent); }
  .mp-btn.primary:hover { background: rgba(74,158,255,.15); }

  /* Grid */
  .mp-grid {
    flex: 1; overflow-y: auto; padding: 12px;
    display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 10px; align-content: start;
  }

  .mp-card {
    padding: 14px; background: var(--color-surface);
    border: 1px solid var(--color-border); border-radius: 6px;
    display: flex; flex-direction: column; gap: 6px; transition: border-color .1s;
  }
  .mp-card:hover { border-color: rgba(74,158,255,.15); }
  .mp-card.installed { border-color: rgba(52,208,88,.15); }

  .mp-card-top { display: flex; justify-content: space-between; align-items: center; }
  .mp-card-cat {
    font-family: 'Cascadia Code', monospace; font-size: 9px; font-weight: 600;
    color: var(--color-accent); letter-spacing: .5px;
  }
  .mp-card-sim {
    font-size: 9px; color: var(--color-dim); padding: 1px 6px;
    background: var(--color-surface-2); border-radius: 2px;
  }
  .mp-card-name { font-size: 14px; font-weight: 700; color: var(--color-bright, #e2e8f0); }
  .mp-card-author { font-size: 10px; color: var(--color-dim); }
  .mp-card-desc { font-size: 11px; color: var(--color-dim); line-height: 1.4; flex: 1; }
  .mp-card-tags { display: flex; gap: 4px; flex-wrap: wrap; }
  .mp-tag {
    font-size: 9px; padding: 1px 6px; background: var(--color-surface-2);
    border: 1px solid var(--color-border); border-radius: 2px; color: var(--color-dim);
  }
  .mp-card-footer { display: flex; align-items: center; margin-top: 4px; }
  .mp-installed { font-size: 11px; color: var(--color-green); display: flex; align-items: center; gap: 4px; }
  .mp-downloading { font-size: 11px; color: var(--color-accent); }

  .mp-loading, .mp-error, .mp-empty {
    padding: 40px; text-align: center; color: var(--color-dim); font-size: 13px;
  }
  .mp-error { color: var(--color-red); }

  .mp-footer {
    padding: 8px 14px; border-top: 1px solid var(--color-border);
    display: flex; justify-content: space-between;
    font-size: 10px; color: var(--color-dim);
  }

  .mp-toast {
    position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%);
    padding: 8px 20px; background: var(--color-green); color: #fff;
    border-radius: 6px; font-size: 12px; font-weight: 600;
    z-index: 200; animation: fadeIn .2s;
  }
  @keyframes fadeIn { from { opacity: 0; transform: translateX(-50%) translateY(10px); } }
</style>
