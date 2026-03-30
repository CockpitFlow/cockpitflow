<script lang="ts">
  let { updateInfo = null }: { updateInfo?: { current: string } | null } = $props();

  let appTheme = $state(localStorage.getItem('cf-theme') || 'dark');

  function setTheme(t: string) {
    appTheme = t;
    document.documentElement.dataset.theme = t === 'dark' ? '' : t;
    localStorage.setItem('cf-theme', t);
  }

  if (appTheme !== 'dark') document.documentElement.dataset.theme = appTheme;
</script>

<div class="settings-page">
  <div class="cl-card">
    <div class="efb-heading">APPEARANCE</div>
    <div class="theme-row">
      <button class="theme-btn" class:on={appTheme==='dark'} onclick={() => setTheme('dark')}>
        <span class="theme-icon">&#9790;</span>Dark
      </button>
      <button class="theme-btn" class:on={appTheme==='light'} onclick={() => setTheme('light')}>
        <span class="theme-icon">&#9788;</span>Light
      </button>
      <button class="theme-btn" class:on={appTheme==='night'} onclick={() => setTheme('night')}>
        <span class="theme-icon">&#9733;</span>Night
      </button>
    </div>
    <p class="cl-card-desc">Night mode is extra dim for cockpit use in the dark.</p>
  </div>
  <div class="cl-card">
    <div class="efb-heading">ABOUT</div>
    <div class="cl-meta">
      <span class="cl-meta-name">CockpitFlow</span>
      <span class="cl-meta-detail">v{updateInfo?.current || '1.0.0'}</span>
      <span class="cl-meta-detail">Open source cockpit companion</span>
      <span class="cl-meta-detail" style="margin-top:4px">
        <a href="https://github.com/CockpitFlow/cockpitflow" target="_blank" style="color:var(--color-accent);text-decoration:none">GitHub</a>
        ·
        <a href="https://cockpitflow.github.io/cockpitflow-site/" target="_blank" style="color:var(--color-accent);text-decoration:none">Website</a>
      </span>
    </div>
  </div>
</div>

<style>
  .settings-page { padding: 14px; display: flex; flex-wrap: wrap; gap: 10px; width: 100%; align-content: flex-start; }
  .settings-page :global(.cl-card) { min-width: 250px; }
  .theme-row { display: flex; gap: 4px; }
  .theme-btn {
    flex: 1; padding: 10px; background: var(--color-bg);
    border: 1px solid var(--color-border); border-radius: 4px;
    color: var(--color-dim); font-size: 11px; font-weight: 600;
    cursor: pointer; text-align: center; font-family: inherit; transition: all .1s;
    display: flex; flex-direction: column; align-items: center; gap: 4px;
  }
  .theme-btn.on { border-color: var(--color-accent); color: var(--color-accent); background: rgba(74,158,255,.06); }
  .theme-icon { font-size: 18px; }
</style>
