<script lang="ts">
  import { invoke } from '../mock';
  import { toast } from './Toast.svelte';
  import { onMount } from 'svelte';
  import BoardDiagram from './BoardDiagram.svelte';

  // ─── Device Status Polling ───
  let deviceStatuses: Record<number, { connected: boolean; last_heartbeat_ms: number | null; frames_per_sec: number; error: string | null }> = $state({});

  async function pollDeviceStatus() {
    try {
      const statuses: any[] = await invoke('get_device_status');
      const map: typeof deviceStatuses = {};
      for (const s of statuses) {
        map[s.device_id] = { connected: s.connected, last_heartbeat_ms: s.last_heartbeat_ms ?? null, frames_per_sec: s.frames_per_sec, error: s.error ?? null };
      }
      deviceStatuses = map;
    } catch {}
  }

  async function connectDevice(id: number) {
    try {
      await invoke('connect_device', { deviceId: id });
      toast(`Device ${id} connecting...`, 'info');
      pollDeviceStatus();
    } catch (e: any) {
      toast(`Connect failed: ${e}`, 'error');
    }
  }

  async function disconnectDevice(id: number) {
    try {
      await invoke('disconnect_device', { deviceId: id });
      toast(`Device ${id} disconnected`, 'info');
      pollDeviceStatus();
    } catch (e: any) {
      toast(`Disconnect failed: ${e}`, 'error');
    }
  }

  function formatHeartbeat(ms: number | null): string {
    if (ms == null) return 'never';
    if (ms < 1000) return 'just now';
    const secs = Math.round(ms / 1000);
    if (secs < 60) return `${secs}s ago`;
    return `${Math.round(secs / 60)}m ago`;
  }

  // Listen for live input events from Tauri
  let unlistenFn: (() => void) | null = null;

  onMount(() => {
    // Dynamic import for Tauri event API (won't exist in browser preview)
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/event').then(({ listen }) => {
        listen('input-event', (event: any) => {
          if (testRunning) {
            const e = event.payload;
            if (e.event_type !== 'heartbeat') {
              const pressed = e.event_type === 'press' || (e.value > 0 && e.event_type !== 'release');
              pinStates = {
                ...pinStates,
                [e.pin]: { pressed, ever: true, lastValue: e.value }
              };
            }
          }
        }).then(fn => { unlistenFn = fn; });
      });
    }

    // Poll device status every 2 seconds
    pollDeviceStatus();
    const statusInterval = setInterval(pollDeviceStatus, 2000);

    // Auto-connect on startup if setting enabled
    if (localStorage.getItem('scb-autoconnect') === 'true') {
      setTimeout(() => {
        for (const d of devices) {
          connectDevice(d.id);
        }
      }, 2000);
    }

    return () => {
      if (unlistenFn) unlistenFn();
      clearInterval(statusInterval);
    };
  });

  let { devices, ports, onrefresh }: {
    devices: any[];
    ports: any[];
    onrefresh: () => void;
  } = $props();

  // ─── Add Device Form ───
  let showAdd = $state(false);
  let newDevice = $state({
    id: 1, name: '', transportType: 'Serial' as 'Serial' | 'Udp',
    port: '', baudRate: 115200, address: '', listenPort: 49152, devicePort: 49153,
  });

  // ─── Flash Panel ───
  let flashPort = $state('');
  let flashBoard = $state('esp32c6');
  let flashStatus = $state<'idle' | 'detecting' | 'flashing' | 'configuring' | 'done' | 'error'>('idle');
  let flashLog: string[] = $state([]);
  let flashProgress = $state(0);
  let detectedChip = $state('');
  let showFlash = $state(false);

  // Device config during flash
  let wifiSSID = $state('');
  let wifiPassword = $state('');
  let enableWifi = $state(false);
  let customName = $state('');
  let deviceId = $state(1);

  const boardOptions = [
    { id: 'esp32c6', name: 'ESP32-C6', wifi: true },
    { id: 'esp32s3', name: 'ESP32-S3', wifi: true },
    { id: 'esp32', name: 'ESP32', wifi: true },
    { id: 'arduino-mega', name: 'Mega 2560', wifi: false },
    { id: 'arduino-uno', name: 'Uno', wifi: false },
    { id: 'arduino-nano', name: 'Nano', wifi: false },
  ];

  let flashablePorts = $derived(ports.filter((p: any) => p.flashable));
  let isEspBoard = $derived(flashBoard.startsWith('esp32'));

  function openFlash(port: any) {
    flashPort = port.name;
    showFlash = true;
    flashStatus = 'idle';
    flashLog = [];
    detectedChip = '';
    // Auto-select board
    if (port.device_kind === 'esp32') {
      if (port.product?.includes('C6')) flashBoard = 'esp32c6';
      else if (port.product?.includes('S3')) flashBoard = 'esp32s3';
      else flashBoard = 'esp32';
    } else if (port.device_kind === 'arduino') {
      if (port.product?.includes('Mega')) flashBoard = 'arduino-mega';
      else flashBoard = 'arduino-uno';
    }
  }

  async function detectBoard() {
    if (!flashPort) return;
    flashStatus = 'detecting';
    flashLog = ['Detecting board on ' + flashPort + '...'];
    try {
      const result = await invoke('detect_board', { port: flashPort });
      detectedChip = result?.chip || 'Unknown';
      flashLog = [...flashLog, 'Detected: ' + detectedChip];
      flashStatus = 'idle';
      toast('Board detected: ' + detectedChip, 'success');
    } catch (e: any) {
      flashLog = [...flashLog, 'Detection failed: ' + e];
      flashStatus = 'error';
      toast('Detection failed', 'error');
    }
  }

  let flashTimer: any = null;

  async function flashFirmware() {
    if (!flashPort || !flashBoard) return;
    flashStatus = 'flashing';
    flashProgress = 5;
    flashLog = [
      `Flashing ${flashBoard} firmware to ${flashPort}...`,
      'Compiling firmware (this takes 1-2 minutes on first run)...',
    ];

    // Animate progress while waiting
    flashTimer = setInterval(() => {
      if (flashProgress < 85) {
        flashProgress += Math.random() * 3;
        if (flashProgress > 30 && flashLog.length < 4) {
          flashLog = [...flashLog, 'Still compiling...'];
        }
        if (flashProgress > 60 && flashLog.length < 5) {
          flashLog = [...flashLog, 'Uploading to device...'];
        }
      }
    }, 1000);

    try {
      const result = await invoke('flash_firmware', { port: flashPort, board: flashBoard });
      clearInterval(flashTimer);
      flashProgress = 90;
      flashLog = [...flashLog, result || 'Upload complete.'];

      // Send config if set
      if (customName || enableWifi || deviceId !== 1) {
        flashProgress = 95;
        flashStatus = 'configuring';
        flashLog = [...flashLog, 'Configuring device...'];
        await new Promise(r => setTimeout(r, 2000));

        if (customName) {
          await invoke('configure_device', { port: flashPort, command: `N:${customName}` });
          flashLog = [...flashLog, `Name set: ${customName}`];
        }
        if (deviceId !== 1) {
          await invoke('configure_device', { port: flashPort, command: `I:${deviceId}` });
          flashLog = [...flashLog, `Device ID: ${deviceId}`];
        }
        if (enableWifi && wifiSSID) {
          await invoke('configure_device', { port: flashPort, command: `W:${wifiSSID}:${wifiPassword}` });
          await invoke('configure_device', { port: flashPort, command: 'E:1' });
          flashLog = [...flashLog, `WiFi: ${wifiSSID}`];
        }
      }

      flashProgress = 100;
      flashStatus = 'done';
      flashLog = [...flashLog, '', 'Firmware installed!'];
      toast('Firmware flashed successfully!', 'success');
    } catch (e: any) {
      clearInterval(flashTimer);
      flashStatus = 'error';
      flashLog = [...flashLog, 'Failed: ' + e];
      toast('Flash failed: ' + e, 'error');
    }
  }

  async function addDevice() {
    const transport = newDevice.transportType === 'Serial'
      ? { type: 'Serial', port: newDevice.port, baud_rate: newDevice.baudRate }
      : { type: 'Udp', address: newDevice.address, listen_port: newDevice.listenPort, device_port: newDevice.devicePort };

    await invoke('add_device', {
      device: { id: newDevice.id, name: newDevice.name, transport }
    });
    showAdd = false;
    toast(`Device "${newDevice.name}" added`, 'success');
    newDevice.name = '';
    onrefresh();
  }

  async function removeDevice(id: number) {
    await invoke('remove_device', { deviceId: id });
    toast('Device removed', 'info');
    onrefresh();
  }

  // Store probed info per port
  let probeInfo: Record<string, any> = $state({});

  // Live pin states: { [pin]: { pressed, ever, lastValue } }
  let pinStates: Record<number, { pressed: boolean; ever: boolean; lastValue: number }> = $state({});

  // All pins — native + multiplexer/expander virtual pins
  const allPins = [
    // Native digital
    { pin: 2, label: 'D2', type: 'digital' },
    { pin: 3, label: 'D3', type: 'digital' },
    { pin: 4, label: 'D4', type: 'digital' },
    { pin: 5, label: 'D5', type: 'digital' },
    { pin: 6, label: 'D6', type: 'digital' },
    { pin: 7, label: 'D7', type: 'digital' },
    { pin: 8, label: 'D8', type: 'digital' },
    { pin: 9, label: 'D9', type: 'digital' },
    { pin: 10, label: 'D10', type: 'digital' },
    { pin: 11, label: 'D11', type: 'digital' },
    { pin: 12, label: 'D12', type: 'digital' },
    { pin: 13, label: 'D13', type: 'digital' },
    // Native analog
    { pin: 14, label: 'A0', type: 'analog' },
    { pin: 15, label: 'A1', type: 'analog' },
    { pin: 16, label: 'A2', type: 'analog' },
    { pin: 17, label: 'A3', type: 'analog' },
    { pin: 18, label: 'A4', type: 'analog' },
    { pin: 19, label: 'A5', type: 'analog' },
    { pin: 20, label: 'A6', type: 'analog' },
    { pin: 21, label: 'A7', type: 'analog' },
    // 74HC165 shift register inputs (virtual pins 100+)
    ...Array.from({ length: 16 }, (_, i) => ({ pin: 100 + i, label: `SR${Math.floor(i/8)+1}.${i%8}`, type: 'shift-in' })),
    // MCP23017 I2C expander (virtual pins 200+)
    ...Array.from({ length: 16 }, (_, i) => ({ pin: 200 + i, label: `EX1.${i}`, type: 'i2c' })),
  ];

  // Input test mode
  let testPort = $state('');
  let testRunning = $state(false);
  let testEvents: any[] = $state([]);

  async function startInputTest(portName: string) {
    testPort = portName;
    testRunning = true;
    testEvents = [];
    pinStates = {};
    toast('Listening 30 sec — touch GND + D2/D3/D4!', 'info', 5000);
    try {
      console.log('[test] calling test_inputs...');
      const events = await invoke('test_inputs', { port: portName, durationSecs: 30 });
      const evts = (events || []) as any[];
      testEvents = evts;
      toast(`Test complete: ${evts.length} events captured`, evts.length > 0 ? 'success' : 'warning');
    } catch (e: any) {
      toast('Test failed: ' + e, 'error');
    }
    testRunning = false;
  }

  // Ignored ports
  let ignoredPorts: string[] = $state(JSON.parse(localStorage.getItem('scb-ignored-ports') || '[]'));

  function ignorePort(portName: string) {
    ignoredPorts = [...ignoredPorts, portName];
    localStorage.setItem('scb-ignored-ports', JSON.stringify(ignoredPorts));
    toast(`${portName} ignored`, 'info');
  }

  function unignorePort(portName: string) {
    ignoredPorts = ignoredPorts.filter(p => p !== portName);
    localStorage.setItem('scb-ignored-ports', JSON.stringify(ignoredPorts));
    toast(`${portName} restored`, 'info');
  }

  let visiblePorts = $derived(flashablePorts.filter((p: any) => !ignoredPorts.includes(p.name)));
  let hiddenPorts = $derived(ports.filter((p: any) => !p.flashable || ignoredPorts.includes(p.name)));

  function isAlreadyAdded(portName: string): boolean {
    return devices.some((d: any) => d.transport?.port === portName);
  }

  async function quickAddDevice(port: any) {
    if (isAlreadyAdded(port.name)) {
      toast(`${port.name} is already added as a device`, 'warning');
      return;
    }
    const name = probeInfo[port.name]?.name || port.product || port.name;
    const nextId = devices.length > 0 ? Math.max(...devices.map((d: any) => d.id)) + 1 : 1;
    await invoke('add_device', {
      device: { id: nextId, name, transport: { type: 'Serial', port: port.name, baud_rate: 115200 } }
    });
    toast(`Added "${name}" on ${port.name}`, 'success');
    onrefresh();
  }

  async function probePort(portName: string) {
    try {
      const result = await invoke('probe_device', { port: portName });
      probeInfo = { ...probeInfo, [portName]: result };
      if (result.name) {
        toast(`${portName}: ${result.name} (ID:${result.device_id})`, 'success');
      } else {
        toast(`${portName}: No firmware response`, 'warning');
      }
    } catch (e: any) {
      toast(`Probe failed: ${e}`, 'error');
    }
  }
</script>

<!-- ═══ Connected Hardware ═══ -->
<div class="card">
  <h3>
    Connected Hardware
    <span class="badge" style="margin-left: 8px">{visiblePorts.length} board{visiblePorts.length !== 1 ? 's' : ''}</span>
    {#if ignoredPorts.length > 0}
      <button class="btn btn-ghost btn-sm" onclick={() => { ignoredPorts = []; localStorage.removeItem('scb-ignored-ports'); }}>
        Show {ignoredPorts.length} hidden
      </button>
    {/if}
    <button class="btn btn-ghost btn-sm" style="margin-left: auto" onclick={onrefresh}>Refresh</button>
  </h3>
  <p style="font-size: 11px; color: var(--text-dim); margin: -4px 0 12px 0;">Detected USB devices ready for firmware flash or configuration.</p>

  {#if visiblePorts.length > 0}
    <div class="hw-grid">
      {#each visiblePorts as port}
        {@const info = probeInfo[port.name]}
        {@const configuredDevice = devices.find((d: any) => d.transport?.port === port.name)}
        <div class="hw-card">
          <div class="hw-icon">
            {#if port.device_kind === 'esp32'}&#x1F4E1;{:else}&#x1F527;{/if}
          </div>
          <div class="hw-info">
            <div class="hw-name">
              {#if configuredDevice}
                {configuredDevice.name}
              {:else if info?.name}
                {info.name}
              {:else}
                {port.product || 'Unknown Board'}
              {/if}
            </div>
            <div class="hw-meta">
              <span class="port-name-tag">{port.name}</span>
              <span style="color: var(--text-dim); font-size: 10px">
                {port.device_kind === 'esp32' ? 'ESP32' : port.device_kind === 'arduino' ? 'Arduino' : port.manufacturer || ''}
              </span>
              {#if configuredDevice}
                <span style="color: var(--pfd-green); font-size: 10px; font-weight: 600">ID:{configuredDevice.id}</span>
              {/if}
              {#if info?.wifi_connected}
                <span style="color: var(--green); font-size: 10px; font-weight: 600">WiFi: {info.wifi_ip}</span>
              {/if}
            </div>
          </div>
          <div class="hw-actions" style="display: flex; gap: 4px; flex-wrap: wrap">
            {#if isAlreadyAdded(port.name)}
              <span class="badge" style="background: var(--pfd-green-dim); color: var(--pfd-green)">Added</span>
            {:else}
              <button class="btn btn-primary btn-sm" onclick={() => quickAddDevice(port)} title="Add this board as a configured device">Add</button>
            {/if}
            <button class="btn btn-sm" onclick={() => openFlash(port)} title="Flash firmware and configure board settings">Setup</button>
            <button class="btn btn-sm" onclick={() => startInputTest(port.name)}
              disabled={testRunning} title="Test hardware inputs for 30 seconds">
              {testRunning && testPort === port.name ? 'Testing...' : 'Test'}
            </button>
            <button class="btn btn-danger btn-sm" onclick={() => ignorePort(port.name)}
              title="Hide this port from the list">Ignore</button>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty" style="padding: 32px; text-align: center;">
      <div style="font-size: 28px; opacity: 0.2; margin-bottom: 8px;">&#x1F50C;</div>
      <div style="font-size: 13px; font-weight: 600; color: var(--text-dim); margin-bottom: 4px;">No boards detected</div>
      <div style="font-size: 11px; color: var(--text-muted); margin-bottom: 12px;">Plug in an Arduino or ESP32 via USB to get started.</div>
      <button class="btn btn-sm" onclick={onrefresh}>Refresh Ports</button>
    </div>
  {/if}

  <!-- Hidden (ignored) ports -->
  {#if ignoredPorts.length > 0}
    <div style="margin-top: 12px; padding-top: 10px; border-top: 1px solid var(--border)">
      <div style="font-size: 10px; font-weight: 600; color: var(--text-dim); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 6px">
        Hidden Ports ({ignoredPorts.length})
      </div>
      <div style="display: flex; flex-wrap: wrap; gap: 6px">
        {#each ignoredPorts as portName}
          <div style="display: flex; align-items: center; gap: 6px; padding: 4px 8px; background: var(--bg-instrument, #0f1117); border: 1px solid var(--border); border-radius: 4px; font-size: 11px; color: var(--text-dim)">
            <span style="font-family: var(--mono)">{portName}</span>
            <button class="btn btn-ghost btn-sm" style="padding: 1px 4px; font-size: 10px" onclick={() => unignorePort(portName)}>Show</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- ═══ Live Board Visualizer ═══ -->
{#if testRunning || Object.keys(pinStates).length > 0}
  <div class="card">
    <h3>
      Live Hardware Monitor {testPort ? `(${testPort})` : ''}
      {#if testRunning}
        <span class="badge" style="background: var(--pfd-green-dim, rgba(0,255,136,0.15)); color: var(--pfd-green, #00ff88)">LIVE</span>
      {/if}
    </h3>

    <div class="pin-grid">
      {#each allPins as p}
        {@const state = pinStates[p.pin]}
        <div class="pin-cell" class:pin-active={state?.pressed} class:pin-touched={state?.ever}>
          <div class="pin-indicator" class:lit={state?.pressed}></div>
          <div class="pin-id">{p.label}</div>
          {#if state?.lastValue !== undefined}
            <div class="pin-val">{state.lastValue}</div>
          {/if}
        </div>
      {/each}
    </div>

    {#if !testRunning && Object.keys(pinStates).length > 0}
      <div style="margin-top: 12px; font-size: 11px; color: var(--text-dim); text-align: center">
        Test complete. Pins that responded are highlighted.
      </div>
    {/if}
  </div>
{/if}

<!-- ═══ Flash & Configure Panel ═══ -->
{#if showFlash}
  <div class="card">
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px">
      <h3 style="margin: 0">&#x26A1; Setup {flashPort}</h3>
      <button class="btn btn-sm" onclick={() => showFlash = false}>Close</button>
    </div>

    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px">
      <!-- Left: Config -->
      <div>
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 12px">
          <div>
            <span class="fl">Board Type</span>
            <select bind:value={flashBoard} style="width: 100%">
              {#each boardOptions as b}
                <option value={b.id}>{b.name}</option>
              {/each}
            </select>
          </div>
          <div>
            <span class="fl">Device ID</span>
            <input type="number" bind:value={deviceId} min="1" max="255" style="width: 100%" />
          </div>
        </div>

        <div style="margin-bottom: 12px">
          <span class="fl">Device Name</span>
          <input type="text" bind:value={customName} placeholder="e.g. Throttle Panel" style="width: 100%" />
        </div>

        {#if isEspBoard}
          <div style="padding: 10px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius-sm); margin-bottom: 12px">
            <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 12px; font-weight: 500">
              <input type="checkbox" bind:checked={enableWifi} style="accent-color: var(--accent)" />
              Enable WiFi
            </label>
            {#if enableWifi}
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-top: 8px">
                <div>
                  <span class="fl">SSID</span>
                  <input type="text" bind:value={wifiSSID} placeholder="WiFi name" style="width: 100%" />
                </div>
                <div>
                  <span class="fl">Password</span>
                  <input type="password" bind:value={wifiPassword} placeholder="Password" style="width: 100%" />
                </div>
              </div>
            {/if}
          </div>
        {/if}

        <div style="display: flex; gap: 8px">
          <button class="btn btn-sm" onclick={detectBoard}
            disabled={flashStatus === 'detecting'}>
            {flashStatus === 'detecting' ? 'Detecting...' : 'Detect'}
          </button>
          <button class="btn btn-primary" style="flex: 1" onclick={flashFirmware}
            disabled={flashStatus === 'flashing' || flashStatus === 'configuring'}>
            {#if flashStatus === 'flashing'}Flashing... {Math.round(flashProgress)}%
            {:else if flashStatus === 'configuring'}Configuring...
            {:else if flashStatus === 'done'}Flash Again
            {:else}Flash Firmware{/if}
          </button>
        </div>

        {#if flashStatus === 'flashing' || flashStatus === 'configuring'}
          <div class="progress-bar" style="margin-top: 8px">
            <div class="progress-bar-fill" style="width: {Math.round(flashProgress)}%"></div>
          </div>
        {/if}

        {#if detectedChip}
          <div style="margin-top: 8px; padding: 6px 10px; background: var(--green-soft); border-radius: var(--radius-sm); font-size: 11px; color: var(--green)">
            {detectedChip}
          </div>
        {/if}
      </div>

      <!-- Right: Log -->
      <div>
        <span class="fl">Flash Log</span>
        <div class="flash-log">
          {#each flashLog as line}
            <div class="log-line" class:log-ok={line.includes('installed') || line.includes('complete')} class:log-err={line.includes('Failed') || line.includes('failed')}>
              {line}
            </div>
          {:else}
            <div style="color: var(--text-dim)">Click "Flash Firmware" to begin...</div>
          {/each}
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- ═══ Configured Devices ═══ -->
<div class="card">
  <h3>
    Configured Devices
    <button class="btn btn-primary btn-sm" style="margin-left: auto" onclick={() => showAdd = !showAdd} title="Manually configure a device by ID and transport">
      {showAdd ? 'Cancel' : '+ Add Manually'}
    </button>
  </h3>
  <p style="font-size: 11px; color: var(--text-dim); margin: -4px 0 12px 0;">Devices registered for communication with the simulator.</p>

  {#if showAdd}
    <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; margin-bottom: 16px; padding: 12px; background: var(--bg-surface); border-radius: var(--radius);">
      <div>
        <span class="fl">Device ID</span>
        <input type="number" bind:value={newDevice.id} min="1" max="255" style="width: 100%" />
      </div>
      <div>
        <span class="fl">Name</span>
        <input type="text" bind:value={newDevice.name} placeholder="My Panel" style="width: 100%" />
      </div>
      <div>
        <span class="fl">Transport</span>
        <select bind:value={newDevice.transportType} style="width: 100%">
          <option value="Serial">Serial (USB)</option>
          <option value="Udp">UDP (WiFi)</option>
        </select>
      </div>
      {#if newDevice.transportType === 'Serial'}
        <div>
          <span class="fl">Port</span>
          <select bind:value={newDevice.port} style="width: 100%">
            {#each ports as p}
              <option value={p.name}>{p.name} — {p.product || p.port_type}</option>
            {/each}
          </select>
        </div>
        <div>
          <span class="fl">Baud Rate</span>
          <select bind:value={newDevice.baudRate} style="width: 100%">
            <option value={115200}>115200</option>
            <option value={9600}>9600</option>
            <option value={250000}>250000</option>
            <option value={500000}>500000</option>
          </select>
        </div>
      {:else}
        <div>
          <span class="fl">IP Address</span>
          <input type="text" bind:value={newDevice.address} placeholder="192.168.1.100" style="width: 100%" />
        </div>
        <div>
          <span class="fl">UDP Port</span>
          <input type="number" bind:value={newDevice.listenPort} style="width: 100%" />
        </div>
      {/if}
      <div style="display: flex; align-items: end">
        <button class="btn btn-primary btn-sm" onclick={addDevice}>Save</button>
      </div>
    </div>
  {/if}

  {#if devices.length > 0}
    <table class="table">
      <thead><tr><th>ID</th><th>Name</th><th>Transport</th><th>Connection</th><th>Status</th><th></th></tr></thead>
      <tbody>
        {#each devices as d}
          {@const status = deviceStatuses[d.id]}
          <tr>
            <td style="font-family: var(--mono); font-weight: 600">{d.id}</td>
            <td style="font-weight: 500">{d.name}</td>
            <td>
              <span class="tag" style="background: {d.transport.type === 'Serial' ? 'var(--green-soft)' : 'var(--orange-soft)'}; color: {d.transport.type === 'Serial' ? 'var(--green)' : 'var(--orange)'}">
                {d.transport.type}
              </span>
            </td>
            <td style="font-family: var(--mono); font-size: 11px; color: var(--text-secondary)">
              {#if d.transport.type === 'Serial'}
                {d.transport.port} @ {d.transport.baud_rate}
              {:else}
                {d.transport.address}:{d.transport.device_port}
              {/if}
            </td>
            <td style="font-size: 11px">
              <div style="display: flex; align-items: center; gap: 6px">
                <span class="status-dot" style="background: {status?.connected ? 'var(--pfd-green, #00ff88)' : 'var(--warning, #ff4444)'}"></span>
                <span style="color: {status?.connected ? 'var(--pfd-green, #00ff88)' : 'var(--text-dim)'}; font-weight: 600">
                  {status?.connected ? 'Connected' : 'Disconnected'}
                </span>
              </div>
              {#if status}
                <div style="font-size: 10px; color: var(--text-dim); margin-top: 2px; font-family: var(--mono)">
                  {formatHeartbeat(status.last_heartbeat_ms)} &middot; {status.frames_per_sec.toFixed(1)} fps
                </div>
                {#if status.error}
                  <div style="font-size: 10px; color: var(--warning); margin-top: 1px">{status.error}</div>
                {/if}
              {/if}
            </td>
            <td style="display: flex; gap: 4px">
              {#if status?.connected}
                <button class="btn btn-sm" style="background: var(--warning-soft, rgba(255,68,68,0.1)); color: var(--warning, #ff4444); border-color: var(--warning, #ff4444)" onclick={() => disconnectDevice(d.id)}>Disconnect</button>
              {:else}
                <button class="btn btn-primary btn-sm" onclick={() => connectDevice(d.id)}>Connect</button>
              {/if}
              <button class="btn btn-danger btn-sm" onclick={() => removeDevice(d.id)}>Remove</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else if !showAdd}
    <div class="empty" style="padding: 32px; text-align: center;">
      <div style="font-size: 28px; opacity: 0.2; margin-bottom: 8px;">&#x2699;</div>
      <div style="font-size: 13px; font-weight: 600; color: var(--text-dim); margin-bottom: 4px;">No devices configured</div>
      <div style="font-size: 11px; color: var(--text-muted); margin-bottom: 12px;">Flash a board above or click "+ Add Manually" to register a device.</div>
    </div>
  {/if}
</div>

<style>
  .status-dot {
    display: inline-block;
    width: 8px; height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    box-shadow: 0 0 6px currentColor;
  }

  .hw-grid { display: flex; flex-direction: column; gap: 8px; }
  .hw-card {
    display: flex; align-items: center; gap: 12px;
    padding: 12px 14px;
    background: var(--bg-surface); border: 1px solid var(--border);
    border-radius: var(--radius); transition: border-color 0.15s;
  }
  .hw-card:hover { border-color: var(--border-strong); }
  .hw-icon { font-size: 24px; flex-shrink: 0; }
  .hw-info { flex: 1; }
  .hw-name { font-size: 13px; font-weight: 600; }
  .hw-meta { display: flex; align-items: center; gap: 6px; margin-top: 2px; }
  .hw-actions { flex-shrink: 0; }

  .fl {
    display: block; font-size: 10px; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.5px;
    color: var(--text-dim); margin-bottom: 4px;
  }

  .flash-log {
    background: var(--bg-input); border: 1px solid var(--border);
    border-radius: var(--radius-sm); padding: 10px 12px;
    font-family: var(--mono); font-size: 11px;
    height: 200px; overflow-y: auto;
  }
  .log-line { padding: 1px 0; color: var(--text-secondary); }
  .log-ok { color: var(--green); font-weight: 600; }
  .log-err { color: var(--red); }

  :global(.progress-bar-fill) {
    transition: width 0.5s ease !important;
  }

  /* ═══ Pin Grid ═══ */
  .pin-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(70px, 1fr));
    gap: 6px;
  }

  .pin-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 6px;
    background: var(--bg-instrument, #0f1117);
    border: 1px solid var(--border-panel, rgba(255,255,255,0.08));
    border-radius: 6px;
    transition: all 0.1s;
    box-shadow: inset 0 1px 3px rgba(0,0,0,0.5);
  }

  .pin-cell.pin-active {
    background: rgba(0, 255, 136, 0.12);
    border-color: rgba(0, 255, 136, 0.4);
    box-shadow: 0 0 12px rgba(0, 255, 136, 0.2), inset 0 1px 3px rgba(0,0,0,0.3);
  }

  .pin-cell.pin-touched:not(.pin-active) {
    border-color: rgba(0, 170, 255, 0.2);
  }

  .pin-id {
    font-family: var(--mono, monospace);
    font-size: 13px;
    font-weight: 700;
    color: var(--text, #d4d8e0);
  }

  .pin-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    margin: 4px 0;
    background: rgba(255,255,255,0.06);
    transition: all 0.1s;
  }

  .pin-indicator.lit {
    background: #00ff88;
    box-shadow: 0 0 8px rgba(0, 255, 136, 0.6);
  }

</style>
