<script lang="ts">
  import { invoke } from '../mock';
  import ProfileEditor from './ProfileEditor.svelte';

  let aircraftProfiles: any[] = $state([]);
  let deviceProfiles: any[] = $state([]);
  let mappingProfiles: any[] = $state([]);
  let activeSection = $state<'aircraft' | 'devices' | 'mappings'>('aircraft');
  let selectedProfile = $state<any>(null);
  let importJson = $state('');
  let showImport = $state(false);
  let importStatus = $state('');
  let filterCategory = $state('all');

  async function loadProfiles() {
    try {
      [aircraftProfiles, deviceProfiles, mappingProfiles] = await Promise.all([
        invoke('get_aircraft_profiles'),
        invoke('get_device_profiles'),
        invoke('get_mapping_profiles'),
      ]);
    } catch (e) {
      console.error('Failed to load profiles:', e);
    }
  }

  async function importProfile() {
    if (!importJson.trim()) return;
    try {
      const result = await invoke('import_profile', { json: importJson });
      importStatus = `Imported: ${result}`;
      importJson = '';
      showImport = false;
      loadProfiles();
    } catch (e: any) {
      importStatus = `Error: ${e}`;
    }
  }

  function exportProfile(profile: any) {
    const json = JSON.stringify(profile, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${profile.id}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function categories(profiles: any[]): string[] {
    const cats = new Set(profiles.map((p: any) => p.category || 'other'));
    return ['all', ...Array.from(cats).sort()];
  }

  function filtered(profiles: any[]): any[] {
    if (filterCategory === 'all') return profiles;
    return profiles.filter((p: any) => (p.category || 'other') === filterCategory);
  }

  function pinTypeIcon(type: string): string {
    const icons: Record<string, string> = {
      button: '\u25CB', toggle: '\u25A0', encoder: '\u27F3',
      potentiometer: '\u25E4', led: '\u2B24', servo: '\u2699',
      display: '\u25A3', stepper: '\u2699',
    };
    return icons[type] || '\u25CF';
  }

  function categoryColor(cat: string): string {
    const colors: Record<string, string> = {
      flight: '#58a6ff', engine: '#f85149', autopilot: '#bc8cff',
      electrical: '#d29922', fuel: '#3fb950', radio: '#79c0ff',
      lights: '#e3b341', misc: '#8b949e',
    };
    return colors[cat] || '#8b949e';
  }

  let updateInfo: any = $state(null);

  async function toggleProfile(type: string, id: string, enabled: boolean) {
    try {
      await invoke('toggle_profile', { profileType: type, id, enabled });
      loadProfiles();
    } catch (e) { console.error(e); }
  }

  async function checkUpdates() {
    try {
      updateInfo = await invoke('check_for_updates');
    } catch (e) { console.error(e); }
  }

  $effect(() => {
    loadProfiles();
    checkUpdates();
  });
</script>

<div class="card">
  <div class="card-header" style="margin-bottom: 0">
    <h3>Community Profiles</h3>
    <p style="font-size: 11px; color: var(--text-dim); margin: 2px 0 0 0;">Browse, import, and manage aircraft, device, and mapping profiles shared by the community.</p>
    <div style="display: flex; gap: 8px;">
      <button class="btn btn-sm" onclick={async () => {
        try {
          const result = await invoke('sync_community_profiles');
          loadProfiles();
          importStatus = result;
        } catch (e) { importStatus = 'Sync failed: ' + e; }
      }}>Sync GitHub</button>
      <button class="btn btn-sm" onclick={() => loadProfiles()}>Reload</button>
      <button class="btn btn-primary btn-sm" onclick={() => { showImport = !showImport; importStatus = ''; }}>
        {showImport ? 'Cancel' : 'Import JSON'}
      </button>
    </div>
  </div>

  {#if importStatus}
    <div style="margin-top: 8px; padding: 6px 10px; border-radius: 4px; font-size: 11px;
      background: {importStatus.startsWith('Error') ? 'rgba(248,81,73,0.1)' : 'rgba(63,185,80,0.1)'};
      color: {importStatus.startsWith('Error') ? 'var(--red)' : 'var(--green)'};">
      {importStatus}
    </div>
  {/if}
</div>

{#if showImport}
  <div class="card" style="margin-top: 8px">
    <h3 style="font-size: 13px; font-weight: 600; margin-bottom: 8px">Import Profile</h3>
    <textarea
      bind:value={importJson}
      placeholder="Paste JSON profile here..."
      style="width: 100%; height: 120px; background: var(--bg); border: 1px solid var(--border);
        border-radius: 4px; color: var(--text); font-family: var(--mono); font-size: 11px;
        padding: 8px; resize: vertical;"
    ></textarea>
    <button class="btn btn-primary btn-sm" style="margin-top: 8px" onclick={importProfile}>
      Import
    </button>
  </div>
{/if}

<!-- Update Banner -->
{#if updateInfo?.available}
  <div class="card" style="margin-top: 8px; border-color: var(--accent); background: var(--accent-soft);">
    <div style="display: flex; align-items: center; justify-content: space-between;">
      <div>
        <div style="font-weight: 600; font-size: 13px; color: var(--accent);">Update Available: v{updateInfo.latest_version}</div>
        <div style="font-size: 11px; color: var(--text-secondary); margin-top: 2px;">
          {updateInfo.new_profiles} new profiles &middot; {updateInfo.changelog}
        </div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-sm" onclick={() => updateInfo = null}>Dismiss</button>
        <button class="btn btn-primary btn-sm">Download Update</button>
      </div>
    </div>
  </div>
{/if}

<!-- Section Tabs -->
<div style="display: flex; gap: 4px; margin-top: 12px; margin-bottom: 12px; background: var(--bg-elevated, rgba(255,255,255,0.03)); border-radius: 8px; padding: 3px;">
  <button class="btn btn-sm" class:active-tab={activeSection === 'aircraft'} onclick={() => { activeSection = 'aircraft'; selectedProfile = null; filterCategory = 'all'; }}
    style="border-radius: 6px; font-weight: 600;">
    &#x2708; Aircraft ({aircraftProfiles.length})
  </button>
  <button class="btn btn-sm" class:active-tab={activeSection === 'devices'} onclick={() => { activeSection = 'devices'; selectedProfile = null; }}
    style="border-radius: 6px; font-weight: 600;">
    &#x1F4BB; Devices ({deviceProfiles.length})
  </button>
  <button class="btn btn-sm" class:active-tab={activeSection === 'mappings'} onclick={() => { activeSection = 'mappings'; selectedProfile = null; }}
    style="border-radius: 6px; font-weight: 600;">
    &#x1F50C; Mappings ({mappingProfiles.length})
  </button>
  <button class="btn btn-primary btn-sm" class:active-tab={activeSection === 'create'} onclick={() => { activeSection = 'create' as any; selectedProfile = null; }}
    style="border-radius: 6px; font-weight: 600;">
    + Create
  </button>
</div>

{#if activeSection === 'create' as any}
  <ProfileEditor />
{/if}

<!-- Aircraft Profiles -->
{#if activeSection === 'aircraft'}
  <!-- Category filter tabs -->
  <div style="margin-bottom: 10px; display: flex; gap: 4px; flex-wrap: wrap; background: var(--bg-elevated, rgba(255,255,255,0.03)); border-radius: 8px; padding: 3px;">
    {#each ['all', 'ga', 'airliner', 'military', 'helicopter'] as cat}
      <button class="btn btn-sm" class:active-tab={filterCategory === cat} onclick={() => filterCategory = cat}
        style="text-transform: capitalize; font-size: 10px; padding: 4px 12px; border-radius: 6px; font-weight: 600;">
        {cat === 'all' ? 'All' : cat === 'ga' ? 'GA' : cat[0].toUpperCase() + cat.slice(1)}
      </button>
    {/each}
    {#if aircraftProfiles.length > 0}
      {#each categories(aircraftProfiles).filter(c => !['all', 'ga', 'airliner', 'military', 'helicopter'].includes(c)) as cat}
        <button class="btn btn-sm" class:active-tab={filterCategory === cat} onclick={() => filterCategory = cat}
          style="text-transform: capitalize; font-size: 10px; padding: 4px 12px; border-radius: 6px;">
          {cat}
        </button>
      {/each}
    {/if}
  </div>

  {#each filtered(aircraftProfiles) as profile}
    <div class="card" role="button" tabindex="0" style="margin-bottom: 8px; cursor: pointer; transition: all 0.15s; border-radius: 10px;"
      onclick={() => selectedProfile = selectedProfile?.id === profile.id ? null : profile}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectedProfile = selectedProfile?.id === profile.id ? null : profile; } }}>
      <div style="display: flex; justify-content: space-between; align-items: center">
        <div>
          <div style="display: flex; align-items: center; gap: 8px;">
            <span style="font-weight: 600; font-size: 13px">{profile.name}</span>
            {#if profile.category}
              <span style="font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px; padding: 2px 6px; border-radius: 4px; background: var(--accent-soft, rgba(56,189,248,0.1)); color: var(--accent);">{profile.category}</span>
            {/if}
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px">
            by {profile.author} &middot;
            {profile.variables?.length || 0} vars &middot;
            {profile.events?.length || 0} events
          </div>
        </div>
        <div style="display: flex; gap: 6px; align-items: center;">
          <span class="toggle-switch" role="button" tabindex="0" onclick={(e: MouseEvent) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
            <input type="checkbox" checked={profile.enabled !== false}
              onchange={(e: any) => toggleProfile('aircraft', profile.id, e.target.checked)} />
            <span class="toggle-slider"></span>
          </span>
          <button class="btn btn-sm" onclick={(e: MouseEvent) => { e.stopPropagation(); exportProfile(profile); }}>Export</button>
        </div>
      </div>

      {#if selectedProfile?.id === profile.id}
        <div style="margin-top: 12px; border-top: 1px solid var(--border); padding-top: 12px;">
          {#if profile.description}
            <p style="font-size: 12px; color: var(--text-dim); margin-bottom: 10px">{profile.description}</p>
          {/if}

          {#each ['flight', 'engine', 'autopilot', 'electrical', 'fuel', 'radio', 'lights', 'misc'] as cat}
            {@const vars = profile.variables?.filter((v: any) => v.category === cat) || []}
            {@const evts = profile.events?.filter((e: any) => e.category === cat) || []}
            {#if vars.length > 0 || evts.length > 0}
              <div style="margin-bottom: 8px">
                <div style="font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px;
                  color: {categoryColor(cat)}; margin-bottom: 4px">
                  {cat}
                </div>
                <div style="display: flex; flex-wrap: wrap; gap: 4px;">
                  {#each vars as v}
                    <span style="font-size: 10px; padding: 2px 6px; border-radius: 3px; background: var(--bg-hover);
                      color: var(--text-dim)" title="{v.name} ({v.unit}){v.writable ? ' [writable]' : ''}">
                      {v.label || v.name}
                    </span>
                  {/each}
                  {#each evts as e}
                    <span style="font-size: 10px; padding: 2px 6px; border-radius: 3px; background: var(--bg-hover);
                      color: var(--accent); font-style: italic" title="Event: {e.name}">
                      {e.label || e.name}
                    </span>
                  {/each}
                </div>
              </div>
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty" style="padding: 32px; text-align: center;">
      <div style="font-size: 28px; opacity: 0.2; margin-bottom: 8px;">&#x2708;</div>
      <div style="font-size: 13px; font-weight: 600; color: var(--text-dim); margin-bottom: 4px;">No aircraft profiles loaded</div>
      <div style="font-size: 11px; color: var(--text-muted); margin-bottom: 12px;">Click "Sync GitHub" to download community profiles, or import one manually.</div>
    </div>
  {/each}
{/if}

<!-- Device Profiles -->
{#if activeSection === 'devices'}
  {#each deviceProfiles as profile}
    <div class="card" role="button" tabindex="0" style="margin-bottom: 8px; cursor: pointer"
      onclick={() => selectedProfile = selectedProfile?.id === profile.id ? null : profile}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectedProfile = selectedProfile?.id === profile.id ? null : profile; } }}>
      <div style="display: flex; justify-content: space-between; align-items: center">
        <div>
          <div style="font-weight: 600; font-size: 13px">{profile.name}</div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px">
            by {profile.author} &middot;
            {profile.pins?.length || 0} pins &middot;
            {profile.board} &middot; {profile.transport}
          </div>
        </div>
        <div style="display: flex; gap: 6px; align-items: center;">
          <span class="toggle-switch" role="button" tabindex="0" onclick={(e: MouseEvent) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
            <input type="checkbox" checked={profile.enabled !== false}
              onchange={(e: any) => toggleProfile('device', profile.id, e.target.checked)} />
            <span class="toggle-slider"></span>
          </span>
          <button class="btn btn-sm" onclick={(e: MouseEvent) => { e.stopPropagation(); exportProfile(profile); }}>Export</button>
        </div>
      </div>

      {#if selectedProfile?.id === profile.id}
        <div style="margin-top: 12px; border-top: 1px solid var(--border); padding-top: 12px;">
          {#if profile.description}
            <p style="font-size: 12px; color: var(--text-dim); margin-bottom: 10px">{profile.description}</p>
          {/if}
          <table class="table">
            <thead>
              <tr><th>Pin</th><th>Type</th><th>Label</th><th>Options</th></tr>
            </thead>
            <tbody>
              {#each profile.pins || [] as pin}
                <tr>
                  <td style="font-family: var(--mono); font-weight: 600">{pin.pin}</td>
                  <td><span style="font-size: 11px">{pinTypeIcon(pin.type)} {pin.type}</span></td>
                  <td>{pin.label}</td>
                  <td style="font-size: 10px; color: var(--text-dim); font-family: var(--mono)">
                    {Object.keys(pin.options || {}).length > 0 ? JSON.stringify(pin.options) : '—'}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty" style="padding: 32px; text-align: center;">
      <div style="font-size: 28px; opacity: 0.2; margin-bottom: 8px;">&#x1F4BB;</div>
      <div style="font-size: 13px; font-weight: 600; color: var(--text-dim); margin-bottom: 4px;">No device profiles loaded</div>
      <div style="font-size: 11px; color: var(--text-muted);">Device profiles describe pin layouts for specific boards.</div>
    </div>
  {/each}
{/if}

<!-- Mapping Profiles -->
{#if activeSection === 'mappings'}
  {#each mappingProfiles as profile}
    <div class="card" role="button" tabindex="0" style="margin-bottom: 8px; cursor: pointer"
      onclick={() => selectedProfile = selectedProfile?.id === profile.id ? null : profile}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectedProfile = selectedProfile?.id === profile.id ? null : profile; } }}>
      <div style="display: flex; justify-content: space-between; align-items: center">
        <div>
          <div style="font-weight: 600; font-size: 13px">{profile.name}</div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px">
            by {profile.author} &middot;
            {profile.mappings?.length || 0} bindings &middot;
            {profile.device_profile} → {profile.aircraft_profile}
          </div>
        </div>
        <div style="display: flex; gap: 6px; align-items: center;">
          <span class="toggle-switch" role="button" tabindex="0" onclick={(e: MouseEvent) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
            <input type="checkbox" checked={profile.enabled !== false}
              onchange={(e: any) => toggleProfile('mapping', profile.id, e.target.checked)} />
            <span class="toggle-slider"></span>
          </span>
          <button class="btn btn-sm" onclick={(e: MouseEvent) => { e.stopPropagation(); exportProfile(profile); }}>Export</button>
        </div>
      </div>

      {#if selectedProfile?.id === profile.id}
        <div style="margin-top: 12px; border-top: 1px solid var(--border); padding-top: 12px;">
          {#if profile.description}
            <p style="font-size: 12px; color: var(--text-dim); margin-bottom: 10px">{profile.description}</p>
          {/if}
          <table class="table">
            <thead>
              <tr><th>Pin</th><th>Action</th><th>Target</th><th>Options</th></tr>
            </thead>
            <tbody>
              {#each profile.mappings || [] as m}
                <tr>
                  <td style="font-family: var(--mono); font-weight: 600">{m.pin}</td>
                  <td>
                    <span style="font-size: 10px; padding: 1px 5px; border-radius: 3px;
                      background: {m.action === 'event' ? '#1a2a3a' : m.action === 'set_var' ? '#2a1a3a' : '#1a3a2a'};
                      color: {m.action === 'event' ? '#58a6ff' : m.action === 'set_var' ? '#bc8cff' : '#3fb950'}">
                      {m.action}
                    </span>
                  </td>
                  <td style="font-family: var(--mono); font-size: 11px">{m.target}</td>
                  <td style="font-size: 10px; color: var(--text-dim); font-family: var(--mono)">
                    {Object.keys(m.options || {}).length > 0 ? JSON.stringify(m.options) : '—'}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty" style="padding: 32px; text-align: center;">
      <div style="font-size: 28px; opacity: 0.2; margin-bottom: 8px;">&#x1F50C;</div>
      <div style="font-size: 13px; font-weight: 600; color: var(--text-dim); margin-bottom: 4px;">No mapping profiles loaded</div>
      <div style="font-size: 11px; color: var(--text-muted);">Mapping profiles link device pins to aircraft variables and events.</div>
    </div>
  {/each}
{/if}

<style>
  .active-tab {
    background: var(--accent) !important;
    color: white !important;
    border-color: var(--accent) !important;
  }

  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 34px;
    height: 18px;
    cursor: pointer;
  }
  .toggle-switch input { opacity: 0; width: 0; height: 0; }
  .toggle-slider {
    position: absolute;
    inset: 0;
    background: var(--bg-elevated);
    border-radius: 18px;
    border: 1px solid var(--border-strong);
    transition: all 0.2s;
  }
  .toggle-slider::before {
    content: '';
    position: absolute;
    width: 12px;
    height: 12px;
    left: 2px;
    top: 2px;
    background: var(--text-dim);
    border-radius: 50%;
    transition: all 0.2s;
  }
  .toggle-switch input:checked + .toggle-slider {
    background: var(--accent-soft);
    border-color: var(--accent);
  }
  .toggle-switch input:checked + .toggle-slider::before {
    transform: translateX(16px);
    background: var(--accent);
  }
</style>
