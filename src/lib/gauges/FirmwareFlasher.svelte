<script lang="ts">
  import { invoke } from '../mock';

  let ports: any[] = $state([]);
  let boards: any[] = $state([]);
  let selectedPort = $state('');
  let selectedBoard = $state('esp32c6');
  let flashStatus = $state<'idle' | 'detecting' | 'flashing' | 'configuring' | 'done' | 'error'>('idle');
  let flashLog: string[] = $state([]);
  let flashProgress = $state(0);
  let detectedChip = $state('');

  // WiFi + device config
  let wifiSSID = $state('');
  let wifiPassword = $state('');
  let enableWifi = $state(false);
  let customName = $state('');
  let deviceId = $state(1);

  const boardOptions = [
    { id: 'esp32c6', name: 'ESP32-C6', icon: '\u{1F4E1}', desc: 'WiFi 6, BT 5, USB-C native', wifi: true },
    { id: 'esp32s3', name: 'ESP32-S3', icon: '\u{1F4E1}', desc: 'Dual-core, USB OTG', wifi: true },
    { id: 'esp32', name: 'ESP32 (Classic)', icon: '\u{1F4E1}', desc: 'WiFi + BT Classic', wifi: true },
    { id: 'arduino-mega', name: 'Arduino Mega 2560', icon: '\u{1F527}', desc: '54 digital I/O, 16 analog', wifi: false },
    { id: 'arduino-uno', name: 'Arduino Uno', icon: '\u{1F527}', desc: '14 digital I/O, 6 analog', wifi: false },
    { id: 'arduino-nano', name: 'Arduino Nano', icon: '\u{1F527}', desc: 'Compact, breadboard-friendly', wifi: false },
  ];

  let selectedBoardInfo = $derived(boardOptions.find(b => b.id === selectedBoard));
  let isEsp = $derived(selectedBoard.startsWith('esp32'));

  async function refreshPorts() {
    try {
      const allPorts = await invoke('list_serial_ports');
      ports = allPorts.filter((p: any) => p.flashable);
      boards = allPorts.filter((p: any) => !p.flashable);
      if (ports.length > 0 && !selectedPort) {
        selectedPort = ports[0].name;
        const port = ports[0];
        if (port.device_kind === 'esp32') {
          if (port.product?.includes('C6')) selectedBoard = 'esp32c6';
          else if (port.product?.includes('S3')) selectedBoard = 'esp32s3';
          else selectedBoard = 'esp32';
        } else if (port.device_kind === 'arduino') {
          if (port.product?.includes('Mega')) selectedBoard = 'arduino-mega';
          else if (port.product?.includes('Nano')) selectedBoard = 'arduino-nano';
          else selectedBoard = 'arduino-uno';
        }
      }
    } catch {}
  }

  async function detectBoard() {
    if (!selectedPort) return;
    flashStatus = 'detecting';
    flashLog = ['Detecting board on ' + selectedPort + '...'];
    try {
      const result = await invoke('detect_board', { port: selectedPort });
      detectedChip = result?.chip || 'Unknown';
      flashLog = [...flashLog, 'Detected: ' + detectedChip, 'Features: ' + (result?.features || '')];
      if (detectedChip.includes('C6')) selectedBoard = 'esp32c6';
      else if (detectedChip.includes('S3')) selectedBoard = 'esp32s3';
      else if (detectedChip.includes('ESP32')) selectedBoard = 'esp32';
      else if (detectedChip.includes('Mega')) selectedBoard = 'arduino-mega';
      flashStatus = 'idle';
    } catch (e: any) {
      flashLog = [...flashLog, 'Detection failed: ' + e];
      flashStatus = 'error';
    }
  }

  async function flashFirmware() {
    if (!selectedPort || !selectedBoard) return;
    flashStatus = 'flashing';
    flashProgress = 0;
    flashLog = [`Flashing ${selectedBoard} firmware to ${selectedPort}...`];

    try {
      flashProgress = 10;
      flashLog = [...flashLog, 'Compiling and uploading firmware...'];

      const result = await invoke('flash_firmware', { port: selectedPort, board: selectedBoard });
      flashProgress = 90;
      flashLog = [...flashLog, result || 'Upload complete.'];

      // If WiFi or name configured, send config after flash
      if (enableWifi || customName) {
        flashProgress = 95;
        flashStatus = 'configuring';
        flashLog = [...flashLog, 'Configuring device...'];
        await new Promise(r => setTimeout(r, 2000)); // Wait for device reboot

        if (customName) {
          await invoke('configure_device', { port: selectedPort, command: `N:${customName}` });
          flashLog = [...flashLog, `Device name set: ${customName}`];
        }
        if (enableWifi && wifiSSID) {
          await invoke('configure_device', { port: selectedPort, command: `W:${wifiSSID}:${wifiPassword}` });
          flashLog = [...flashLog, `WiFi configured: ${wifiSSID}`];
          await invoke('configure_device', { port: selectedPort, command: 'E:1' });
          flashLog = [...flashLog, 'WiFi enabled. Device will connect on next boot.'];
        }
        if (deviceId !== 1) {
          await invoke('configure_device', { port: selectedPort, command: `I:${deviceId}` });
          flashLog = [...flashLog, `Device ID set: ${deviceId}`];
        }
      }

      flashProgress = 100;
      flashStatus = 'done';
      flashLog = [...flashLog, '', 'Firmware installed successfully!'];
      if (enableWifi && wifiSSID) {
        flashLog = [...flashLog, `Device will connect to "${wifiSSID}" via WiFi.`, 'Add it as UDP device in the Devices tab.'];
      }
    } catch (e: any) {
      flashStatus = 'error';
      flashLog = [...flashLog, 'Flash failed: ' + e];
    }
  }

  $effect(() => { refreshPorts(); });
</script>

<div class="card">
  <h3>Firmware Installer</h3>
  <p style="font-size: 12px; color: var(--text-secondary); margin-top: -8px; margin-bottom: 16px;">
    Flash SkyFlow firmware, configure WiFi, and set a custom device name.
  </p>

  <!-- Step 1: Port -->
  <div class="flash-step">
    <div class="step-num">1</div>
    <div class="step-content">
      <div class="step-title">Connect your board</div>
      <div class="step-body">
        <div style="display: flex; gap: 8px; align-items: end;">
          <div style="flex: 1">
            <span class="field-label">Detected Boards</span>
            <select bind:value={selectedPort} style="width: 100%">
              {#each ports as p}
                <option value={p.name}>{p.name} — {p.product || p.manufacturer || 'Unknown'}</option>
              {:else}
                <option value="">No flashable boards — plug in your Arduino or ESP32</option>
              {/each}
            </select>
          </div>
          <button class="btn btn-sm" onclick={refreshPorts}>Refresh</button>
          <button class="btn btn-primary btn-sm" onclick={detectBoard}
            disabled={!selectedPort || flashStatus === 'detecting'}>
            {flashStatus === 'detecting' ? 'Detecting...' : 'Auto-detect'}
          </button>
        </div>
        {#if detectedChip}
          <div class="chip-detected">
            <span class="chip-dot"></span>
            <strong>{detectedChip}</strong>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Step 2: Board -->
  <div class="flash-step">
    <div class="step-num">2</div>
    <div class="step-content">
      <div class="step-title">Select your board</div>
      <div class="step-body">
        <div class="board-grid">
          {#each boardOptions as board}
            <button class="board-card" class:selected={selectedBoard === board.id}
              onclick={() => selectedBoard = board.id}>
              <div class="board-icon">{board.icon}</div>
              <div class="board-info">
                <div class="board-name">{board.name}</div>
                <div class="board-desc">
                  {board.desc}
                  {#if board.wifi}
                    <span style="color: var(--accent)"> — WiFi</span>
                  {/if}
                </div>
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>
  </div>

  <!-- Step 3: Device Config -->
  <div class="flash-step">
    <div class="step-num">3</div>
    <div class="step-content">
      <div class="step-title">Configure device</div>
      <div class="step-body">
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
          <div>
            <span class="field-label">Device Name</span>
            <input type="text" bind:value={customName} placeholder="e.g. Throttle Panel" style="width: 100%" />
          </div>
          <div>
            <span class="field-label">Device ID (1-255)</span>
            <input type="number" bind:value={deviceId} min="1" max="255" style="width: 100%" />
          </div>
        </div>

        {#if isEsp}
          <div style="margin-top: 12px; padding: 12px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius);">
            <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px; font-weight: 500;">
              <input type="checkbox" bind:checked={enableWifi}
                style="width: 16px; height: 16px; accent-color: var(--accent);" />
              Enable WiFi (wireless mode)
            </label>

            {#if enableWifi}
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 10px;">
                <div>
                  <span class="field-label">WiFi SSID</span>
                  <input type="text" bind:value={wifiSSID} placeholder="Your WiFi network" style="width: 100%" />
                </div>
                <div>
                  <span class="field-label">WiFi Password</span>
                  <input type="password" bind:value={wifiPassword} placeholder="Password" style="width: 100%" />
                </div>
              </div>
              <div style="margin-top: 8px; font-size: 11px; color: var(--text-dim);">
                After flashing, the device will connect to this WiFi network and communicate via UDP.
                Add it as a UDP device in the Devices tab using its IP address.
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Step 4: Flash -->
  <div class="flash-step">
    <div class="step-num">4</div>
    <div class="step-content">
      <div class="step-title">Install firmware</div>
      <div class="step-body">
        <button class="btn btn-primary flash-btn" onclick={flashFirmware}
          disabled={!selectedPort || flashStatus === 'flashing' || flashStatus === 'configuring'}>
          {#if flashStatus === 'flashing'}
            Flashing... {flashProgress}%
          {:else if flashStatus === 'configuring'}
            Configuring device...
          {:else if flashStatus === 'done'}
            Flash Again
          {:else}
            Flash Firmware to {selectedPort || '...'}
          {/if}
        </button>

        {#if flashStatus === 'flashing' || flashStatus === 'configuring'}
          <div class="progress-bar" style="margin-top: 12px">
            <div class="progress-bar-fill" style="width: {flashProgress}%"></div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<!-- Flash Log -->
{#if flashLog.length > 0}
  <div class="card" style="margin-top: 12px">
    <h3 style="font-size: 12px">
      Flash Log
      {#if flashStatus === 'done'}
        <span style="margin-left: auto; color: var(--green); font-size: 11px; font-weight: 500">Success</span>
      {:else if flashStatus === 'error'}
        <span style="margin-left: auto; color: var(--red); font-size: 11px; font-weight: 500">Failed</span>
      {/if}
    </h3>
    <div class="flash-log">
      {#each flashLog as line}
        <div class="log-line" class:log-success={line.includes('successfully') || line.includes('OK:')}
          class:log-error={line.includes('failed') || line.includes('Failed') || line.includes('Error')}>
          {line}
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .flash-step { display: flex; gap: 14px; padding: 14px 0; border-bottom: 1px solid var(--border); }
  .flash-step:last-of-type { border-bottom: none; }
  .step-num { width: 28px; height: 28px; border-radius: 50%; background: var(--bg-elevated); color: var(--accent); display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 700; flex-shrink: 0; }
  .step-content { flex: 1; }
  .step-title { font-size: 13px; font-weight: 600; margin-bottom: 8px; }
  .step-body { width: 100%; }
  .field-label { display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px; }
  .chip-detected { margin-top: 8px; padding: 6px 10px; background: var(--green-soft); border-radius: var(--radius-sm); font-size: 12px; color: var(--green); display: flex; align-items: center; gap: 6px; }
  .chip-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--green); animation: pulse-green 2s ease-in-out infinite; }
  .board-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; }
  .board-card { display: flex; align-items: center; gap: 10px; padding: 10px 12px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius); cursor: pointer; transition: all 0.15s; text-align: left; font-family: var(--font); color: var(--text); width: 100%; }
  .board-card:hover { border-color: var(--border-strong); background: var(--bg-elevated); }
  .board-card.selected { border-color: var(--accent); background: var(--accent-soft); box-shadow: 0 0 0 3px rgba(79, 143, 234, 0.1); }
  .board-icon { font-size: 20px; }
  .board-name { font-size: 12px; font-weight: 600; }
  .board-desc { font-size: 10px; color: var(--text-dim); margin-top: 1px; }
  .flash-btn { width: 100%; padding: 12px !important; font-size: 14px !important; font-weight: 600 !important; }
  .flash-log { background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px; font-family: var(--mono); font-size: 11px; max-height: 200px; overflow-y: auto; }
  .log-line { padding: 1px 0; color: var(--text-secondary); }
  .log-line:empty { height: 8px; }
  .log-success { color: var(--green); font-weight: 600; }
  .log-error { color: var(--red); }
</style>
