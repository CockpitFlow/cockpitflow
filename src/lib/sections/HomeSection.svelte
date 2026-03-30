<script lang="ts">
  import { ExternalLink } from 'lucide-svelte';
  import logo from '../../assets/logo.png';

  let { connected = false, arduinoStatus = '', flightActive = false, sidebarItems = [], onNavigate = (_id: string) => {} }:
    { connected?: boolean; arduinoStatus?: string; flightActive?: boolean; sidebarItems?: any[]; onNavigate?: (id: string) => void } = $props();
</script>

<div class="v-row">
  <div class="card" style="width:240px;flex-shrink:0">
    <div class="home-brand"><img src={logo} alt="" class="home-logo" /><div><div class="home-title">COCKPITFLOW</div><div class="home-sub">COCKPIT COMPANION</div></div></div>
    <div class="sep"></div>
    <div class="label">STATUS</div>
    <div class="status-list">
      <div class="status-row"><span class="dot" class:on={connected}></span><span>{connected ? 'Sim connected' : 'No simulator'}</span></div>
      <div class="status-row"><span class="dot" class:on={arduinoStatus==='connected'}></span><span>{arduinoStatus==='connected' ? 'Hardware online' : 'No hardware'}</span></div>
      <div class="status-row"><span class="dot" class:on={flightActive}></span><span>{flightActive ? 'In flight' : 'On ground'}</span></div>
    </div>
    <div class="sep"></div>
    <div class="label">LINKS</div>
    <a href="https://discord.gg/cockpitflow" target="_blank" class="link-btn discord">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M20.317 4.37a19.8 19.8 0 00-4.885-1.515.07.07 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.3 18.3 0 00-5.487 0 12.6 12.6 0 00-.617-1.25.08.08 0 00-.079-.037A19.7 19.7 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.08.08 0 00.031.057 19.9 19.9 0 005.993 3.03.08.08 0 00.084-.028 14.1 14.1 0 001.226-1.994.08.08 0 00-.041-.106 13.1 13.1 0 01-1.872-.892.08.08 0 01-.008-.128c.125-.094.25-.192.372-.292a.07.07 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.07.07 0 01.078.01c.12.1.246.198.373.292a.08.08 0 01-.006.127c-.598.35-1.22.645-1.873.892a.08.08 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.08.08 0 00.084.028 19.8 19.8 0 006.002-3.03.08.08 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.06.06 0 00-.031-.03z"/></svg>
      <span>Discord</span><ExternalLink size={11} />
    </a>
    <div class="spacer"></div>
    <div class="disclaimer">For flight simulation use only. Not for real-world aviation. Aircraft designations are trademarks of their respective owners.</div>
    <div class="version">v1.0.0</div>
  </div>
  <div class="card flex-1">
    <div class="label">MODULES</div>
    <div class="mod-grid">
      {#each sidebarItems.filter(m => m.id !== 'home') as m}
        <button class="mod-btn" onclick={() => onNavigate(m.id)}>
          <svelte:component this={m.icon} size={18} strokeWidth={1.5} />
          <span>{m.label}</span>
        </button>
      {/each}
    </div>
  </div>
</div>
