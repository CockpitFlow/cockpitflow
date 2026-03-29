<script lang="ts">
  import { onMount } from 'svelte';

  // ── Types ──────────────────────────────────────────────────────────
  interface FlowNode {
    id: string;
    type: 'source' | 'condition' | 'output' | 'math';
    x: number; y: number;
    config: {
      var_name?: string; operator?: string; value?: number; value2?: number;
      device_id?: number; pin?: number; output_value?: number;
      operation?: string; param?: number;
      range_in?: [number, number]; range_out?: [number, number];
    };
  }
  interface FlowWire { id: string; from_node: string; from_port: string; to_node: string; to_port: string; }
  interface Flow { nodes: FlowNode[]; wires: FlowWire[]; }

  // ── Props ──────────────────────────────────────────────────────────
  let { aircraft, devices, onSave }: {
    aircraft: any;
    devices: any[];
    onSave: (flow: Flow) => void;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────
  let nodes: FlowNode[] = $state([]);
  let wires: FlowWire[] = $state([]);

  // Canvas pan/zoom
  let panX = $state(0);
  let panY = $state(0);
  let zoom = $state(1);
  let isPanning = $state(false);
  let panStart = $state({ x: 0, y: 0 });

  // Node dragging
  let dragNode: string | null = $state(null);
  let dragOffset = $state({ x: 0, y: 0 });

  // Wire drawing
  let wiringFrom: { nodeId: string; port: string } | null = $state(null);
  let wirePreview = $state({ x: 0, y: 0 });

  // Config panel
  let configNodeId: string | null = $state(null);

  // Hover for delete
  let hoverNodeId: string | null = $state(null);

  let svgEl: SVGSVGElement | undefined = $state(undefined);

  // ── Helpers ────────────────────────────────────────────────────────
  const uid = () => crypto.randomUUID().slice(0, 8);

  const NODE_COLORS: Record<string, string> = {
    source: '#3b82f6', condition: '#eab308', output: '#22c55e', math: '#a855f7',
  };
  const NODE_LABELS: Record<string, string> = {
    source: 'Source', condition: 'Condition', output: 'Output', math: 'Math',
  };
  const NODE_W = 160;
  const NODE_H = 64;
  const PORT_R = 7;

  interface PortDef { id: string; side: 'in' | 'out'; label: string; yOff: number; }

  function getPorts(type: string): PortDef[] {
    switch (type) {
      case 'source':    return [{ id: 'out', side: 'out', label: 'Out', yOff: 0.5 }];
      case 'condition': return [
        { id: 'in',    side: 'in',  label: 'In',    yOff: 0.5 },
        { id: 'true',  side: 'out', label: 'T',     yOff: 0.33 },
        { id: 'false', side: 'out', label: 'F',     yOff: 0.67 },
      ];
      case 'output': return [{ id: 'in', side: 'in', label: 'In', yOff: 0.5 }];
      case 'math':   return [
        { id: 'in',  side: 'in',  label: 'In',  yOff: 0.5 },
        { id: 'out', side: 'out', label: 'Out', yOff: 0.5 },
      ];
      default: return [];
    }
  }

  function portPos(node: FlowNode, port: PortDef): { x: number; y: number } {
    return {
      x: node.x + (port.side === 'in' ? 0 : NODE_W),
      y: node.y + NODE_H * port.yOff,
    };
  }

  function findPortPos(nodeId: string, portId: string): { x: number; y: number } | null {
    const n = nodes.find(n => n.id === nodeId);
    if (!n) return null;
    const p = getPorts(n.type).find(p => p.id === portId);
    if (!p) return null;
    return portPos(n, p);
  }

  function bezierPath(x1: number, y1: number, x2: number, y2: number): string {
    const dx = Math.abs(x2 - x1) * 0.5;
    return `M${x1},${y1} C${x1 + dx},${y1} ${x2 - dx},${y2} ${x2},${y2}`;
  }

  function wireColor(wire: FlowWire): string {
    if (wire.from_port === 'true') return '#22c55e';
    if (wire.from_port === 'false') return '#ef4444';
    return '#3b82f6';
  }

  function svgPoint(e: MouseEvent): { x: number; y: number } {
    return {
      x: (e.clientX - (svgEl?.getBoundingClientRect().left ?? 0)) / zoom - panX / zoom,
      y: (e.clientY - (svgEl?.getBoundingClientRect().top ?? 0)) / zoom - panY / zoom,
    };
  }

  function nodeSubtitle(n: FlowNode): string {
    switch (n.type) {
      case 'source':    return n.config.var_name || 'No variable';
      case 'condition':  return `${n.config.operator || '>'} ${n.config.value ?? 0}`;
      case 'output': {
        const dev = devices?.find((d: any) => d.id === n.config.device_id || d.device_id === n.config.device_id);
        const devName = dev?.custom_name || dev?.name || `Dev ${n.config.device_id ?? '?'}`;
        return `${devName} → Pin ${n.config.pin ?? '?'}`;
      }
      case 'math':      return n.config.operation || 'scale';
      default:          return '';
    }
  }

  // ── Node CRUD ──────────────────────────────────────────────────────
  function addNode(type: FlowNode['type'], x = 200, y = 200): FlowNode {
    const defaults: FlowNode['config'] = {};
    if (type === 'source') defaults.var_name = '';
    if (type === 'condition') { defaults.operator = 'GreaterThan'; defaults.value = 0; }
    if (type === 'output') { defaults.device_id = 1; defaults.pin = 13; defaults.output_value = 1; }
    if (type === 'math') { defaults.operation = 'scale'; defaults.param = 1; defaults.range_in = [0, 16383]; defaults.range_out = [0, 180]; }
    const node: FlowNode = { id: uid(), type, x, y, config: defaults };
    nodes.push(node);
    return node;
  }

  function removeNode(id: string) {
    nodes = nodes.filter(n => n.id !== id);
    wires = wires.filter(w => w.from_node !== id && w.to_node !== id);
    if (configNodeId === id) configNodeId = null;
  }

  function addWire(from_node: string, from_port: string, to_node: string, to_port: string) {
    // prevent duplicates
    if (wires.some(w => w.from_node === from_node && w.from_port === from_port && w.to_node === to_node && w.to_port === to_port)) return;
    // remove existing wire into same input port
    wires = wires.filter(w => !(w.to_node === to_node && w.to_port === to_port));
    wires.push({ id: uid(), from_node, from_port, to_node, to_port });
  }

  // ── Event Handlers ─────────────────────────────────────────────────
  function onCanvasMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    // Close config panel on canvas click
    if (configNodeId && !(e.target as Element)?.closest('.config-panel')) {
      configNodeId = null;
    }
    // Cancel wiring on empty canvas click
    if (wiringFrom && !(e.target as Element)?.closest('.port')) {
      wiringFrom = null;
      return;
    }
    // Start panning
    isPanning = true;
    panStart = { x: e.clientX - panX, y: e.clientY - panY };
  }

  function onCanvasMouseMove(e: MouseEvent) {
    if (dragNode) {
      const pt = svgPoint(e);
      const n = nodes.find(n => n.id === dragNode);
      if (n) { n.x = pt.x - dragOffset.x; n.y = pt.y - dragOffset.y; }
    } else if (isPanning) {
      panX = e.clientX - panStart.x;
      panY = e.clientY - panStart.y;
    }
    if (wiringFrom) {
      const pt = svgPoint(e);
      wirePreview = { x: pt.x, y: pt.y };
    }
  }

  function onCanvasMouseUp() {
    dragNode = null;
    isPanning = false;
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 0.9;
    const newZoom = Math.max(0.2, Math.min(3, zoom * factor));
    // Zoom toward cursor
    const rect = svgEl?.getBoundingClientRect();
    if (rect) {
      const cx = e.clientX - rect.left;
      const cy = e.clientY - rect.top;
      panX = cx - (cx - panX) * (newZoom / zoom);
      panY = cy - (cy - panY) * (newZoom / zoom);
    }
    zoom = newZoom;
  }

  function onNodeMouseDown(e: MouseEvent, nodeId: string) {
    e.stopPropagation();
    const pt = svgPoint(e);
    const n = nodes.find(n => n.id === nodeId);
    if (!n) return;
    dragNode = nodeId;
    dragOffset = { x: pt.x - n.x, y: pt.y - n.y };
  }

  function onNodeDblClick(e: MouseEvent, nodeId: string) {
    e.stopPropagation();
    configNodeId = configNodeId === nodeId ? null : nodeId;
  }

  function onPortClick(e: MouseEvent, nodeId: string, port: PortDef) {
    e.stopPropagation();
    if (!wiringFrom && port.side === 'out') {
      wiringFrom = { nodeId, port: port.id };
      const pos = findPortPos(nodeId, port.id);
      if (pos) wirePreview = { x: pos.x, y: pos.y };
    } else if (wiringFrom && port.side === 'in') {
      if (wiringFrom.nodeId !== nodeId) {
        addWire(wiringFrom.nodeId, wiringFrom.port, nodeId, port.id);
      }
      wiringFrom = null;
    }
  }

  // ── Persistence ────────────────────────────────────────────────────
  function saveFlow() {
    const flow: Flow = { nodes: $state.snapshot(nodes), wires: $state.snapshot(wires) };
    localStorage.setItem('skyflow-flows', JSON.stringify(flow));
    onSave(flow);
  }

  function loadFlow() {
    try {
      const raw = localStorage.getItem('skyflow-flows');
      if (raw) {
        const flow: Flow = JSON.parse(raw);
        nodes = flow.nodes || [];
        wires = flow.wires || [];
      }
    } catch { /* ignore */ }
  }

  function clearFlow() {
    nodes = [];
    wires = [];
    configNodeId = null;
  }

  // ── Preset Templates ──────────────────────────────────────────────
  function presetGearLED() {
    clearFlow();
    const s = addNode('source', 80, 120);
    s.config.var_name = 'GEAR HANDLE POSITION';
    const c = addNode('condition', 320, 120);
    c.config.operator = 'Equals'; c.config.value = 1;
    const o = addNode('output', 560, 80);
    o.config.device_id = 1; o.config.pin = 13; o.config.output_value = 1;
    addWire(s.id, 'out', c.id, 'in');
    addWire(c.id, 'true', o.id, 'in');
  }

  function presetAPStatus() {
    clearFlow();
    const s = addNode('source', 80, 120);
    s.config.var_name = 'AUTOPILOT MASTER';
    const c = addNode('condition', 320, 120);
    c.config.operator = 'Equals'; c.config.value = 1;
    const oOn = addNode('output', 560, 60);
    oOn.config.device_id = 1; oOn.config.pin = 10; oOn.config.output_value = 1;
    const oOff = addNode('output', 560, 200);
    oOff.config.device_id = 1; oOff.config.pin = 10; oOff.config.output_value = 0;
    addWire(s.id, 'out', c.id, 'in');
    addWire(c.id, 'true', oOn.id, 'in');
    addWire(c.id, 'false', oOff.id, 'in');
  }

  function presetFuelWarning() {
    clearFlow();
    const s = addNode('source', 80, 120);
    s.config.var_name = 'FUEL TOTAL QUANTITY';
    const c = addNode('condition', 320, 120);
    c.config.operator = 'LessThan'; c.config.value = 20;
    const o = addNode('output', 560, 80);
    o.config.device_id = 1; o.config.pin = 8; o.config.output_value = 1;
    addWire(s.id, 'out', c.id, 'in');
    addWire(c.id, 'true', o.id, 'in');
  }

  function presetHeadingServo() {
    clearFlow();
    const s = addNode('source', 80, 140);
    s.config.var_name = 'HEADING INDICATOR';
    const m = addNode('math', 320, 140);
    m.config.operation = 'scale'; m.config.range_in = [0, 360]; m.config.range_out = [0, 180];
    const o = addNode('output', 560, 140);
    o.config.device_id = 1; o.config.pin = 9; o.config.output_value = 0;
    addWire(s.id, 'out', m.id, 'in');
    addWire(m.id, 'out', o.id, 'in');
  }

  function presetAltitudeDisplay() {
    clearFlow();
    const s = addNode('source', 80, 140);
    s.config.var_name = 'INDICATED ALTITUDE';
    const m = addNode('math', 320, 140);
    m.config.operation = 'divide'; m.config.param = 100;
    const o = addNode('output', 560, 140);
    o.config.device_id = 1; o.config.pin = 2; o.config.output_value = 0;
    addWire(s.id, 'out', m.id, 'in');
    addWire(m.id, 'out', o.id, 'in');
  }

  // ── Derived ────────────────────────────────────────────────────────
  let configNode = $derived(nodes.find(n => n.id === configNodeId) ?? null);
  let configPos = $derived.by(() => {
    if (!configNode) return { left: 0, top: 0 };
    const rect = svgEl?.getBoundingClientRect();
    if (!rect) return { left: 0, top: 0 };
    return {
      left: configNode.x * zoom + panX + NODE_W * zoom + 12,
      top: configNode.y * zoom + panY,
    };
  });

  const operators = [
    { id: 'GreaterThan', label: '>' },
    { id: 'LessThan', label: '<' },
    { id: 'GreaterThanOrEqual', label: '>=' },
    { id: 'LessThanOrEqual', label: '<=' },
    { id: 'Equals', label: '=' },
    { id: 'NotEquals', label: '!=' },
    { id: 'Between', label: 'between' },
  ];

  const mathOps = [
    { id: 'scale', label: 'Scale (range map)' },
    { id: 'multiply', label: 'Multiply' },
    { id: 'divide', label: 'Divide' },
    { id: 'add', label: 'Add' },
    { id: 'subtract', label: 'Subtract' },
    { id: 'clamp', label: 'Clamp' },
  ];

  onMount(() => { loadFlow(); });
</script>

<!-- ── Toolbar ──────────────────────────────────────────────────────── -->
<div class="flow-editor">
  <div class="toolbar">
    <div class="toolbar-group">
      <span class="toolbar-label">Add Node:</span>
      <button class="btn-node btn-source" onclick={() => addNode('source', 100 - panX / zoom, 100 - panY / zoom)}>+ Source</button>
      <button class="btn-node btn-condition" onclick={() => addNode('condition', 100 - panX / zoom, 100 - panY / zoom)}>+ Condition</button>
      <button class="btn-node btn-output" onclick={() => addNode('output', 100 - panX / zoom, 100 - panY / zoom)}>+ Output</button>
      <button class="btn-node btn-math" onclick={() => addNode('math', 100 - panX / zoom, 100 - panY / zoom)}>+ Math</button>
    </div>
    <div class="toolbar-group">
      <span class="toolbar-label">Presets:</span>
      <button class="btn-preset" onclick={presetGearLED}>&#x1F6A6; Gear LED</button>
      <button class="btn-preset" onclick={presetAPStatus}>&#x2708; AP Status</button>
      <button class="btn-preset" onclick={presetFuelWarning}>&#x26FD; Fuel Warn</button>
      <button class="btn-preset" onclick={presetHeadingServo}>&#x1F9ED; Heading Servo</button>
      <button class="btn-preset" onclick={presetAltitudeDisplay}>&#x1F4CF; Altitude Display</button>
    </div>
    <div class="toolbar-group toolbar-right">
      <button class="btn-clear" onclick={clearFlow}>Clear</button>
      <button class="btn-save" onclick={saveFlow}>Save Flow</button>
    </div>
  </div>

  <!-- ── Canvas ──────────────────────────────────────────────────────── -->
  <div class="canvas-wrap">
    <svg
      bind:this={svgEl}
      class="canvas"
      role="application"
      onmousedown={onCanvasMouseDown}
      onmousemove={onCanvasMouseMove}
      onmouseup={onCanvasMouseUp}
      onmouseleave={onCanvasMouseUp}
      onwheel={onWheel}
    >
      <!-- Background with grid -->
      <defs>
        <pattern id="grid-small" width="20" height="20" patternUnits="userSpaceOnUse"
          patternTransform="translate({panX},{panY}) scale({zoom})">
          <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#1e293b" stroke-width="0.5" />
        </pattern>
        <pattern id="grid-large" width="100" height="100" patternUnits="userSpaceOnUse"
          patternTransform="translate({panX},{panY}) scale({zoom})">
          <path d="M 100 0 L 0 0 0 100" fill="none" stroke="#1e293b" stroke-width="1" />
          <circle cx="0" cy="0" r="1.5" fill="#374151" />
        </pattern>
      </defs>
      <rect width="100%" height="100%" fill="#111827" />
      <rect width="100%" height="100%" fill="url(#grid-small)" />
      <rect width="100%" height="100%" fill="url(#grid-large)" />

      <!-- Empty state instruction -->
      {#if nodes.length === 0}
        <text x="50%" y="45%" fill="#4b5563" font-size="14" font-family="system-ui" text-anchor="middle" font-weight="500">
          Click a preset above or add nodes from the toolbar to get started
        </text>
        <text x="50%" y="50%" fill="#374151" font-size="11" font-family="system-ui" text-anchor="middle">
          Double-click a node to configure it  |  Drag between ports to wire them
        </text>
      {/if}

      <!-- Transformed layer -->
      <g transform="translate({panX},{panY}) scale({zoom})">

        <!-- Wires -->
        {#each wires as wire (wire.id)}
          {@const fromPos = findPortPos(wire.from_node, wire.from_port)}
          {@const toPos = findPortPos(wire.to_node, wire.to_port)}
          {#if fromPos && toPos}
            <path
              d={bezierPath(fromPos.x, fromPos.y, toPos.x, toPos.y)}
              stroke={wireColor(wire)}
              stroke-width="2.5"
              fill="none"
              opacity="0.85"
            />
          {/if}
        {/each}

        <!-- Wire preview while connecting -->
        {#if wiringFrom}
          {@const fromPos = findPortPos(wiringFrom.nodeId, wiringFrom.port)}
          {#if fromPos}
            <path
              d={bezierPath(fromPos.x, fromPos.y, wirePreview.x, wirePreview.y)}
              stroke="#9ca3af"
              stroke-width="2"
              stroke-dasharray="6 4"
              fill="none"
              opacity="0.7"
            />
          {/if}
        {/if}

        <!-- Nodes -->
        {#each nodes as node (node.id)}
          {@const color = NODE_COLORS[node.type]}
          {@const ports = getPorts(node.type)}
          <g
            class="flow-node"
            role="group"
            onmousedown={(e) => onNodeMouseDown(e, node.id)}
            ondblclick={(e) => onNodeDblClick(e, node.id)}
            onmouseenter={() => hoverNodeId = node.id}
            onmouseleave={() => hoverNodeId = null}
          >
            <!-- Node body -->
            <rect
              x={node.x} y={node.y}
              width={NODE_W} height={NODE_H}
              rx="8" ry="8"
              fill="#1f2937"
              stroke={configNodeId === node.id ? '#f59e0b' : color}
              stroke-width={configNodeId === node.id ? 2.5 : 1.5}
            />
            <!-- Color accent bar -->
            <rect
              x={node.x} y={node.y}
              width="6" height={NODE_H}
              rx="8" ry="0"
              fill={color}
            />
            <clipPath id="accent-clip-{node.id}">
              <rect x={node.x} y={node.y} width="8" height={NODE_H} rx="8" ry="8" />
            </clipPath>
            <rect
              x={node.x} y={node.y}
              width="6" height={NODE_H}
              fill={color}
              clip-path="url(#accent-clip-{node.id})"
            />

            <!-- Label -->
            <text
              x={node.x + 16} y={node.y + 22}
              fill="#f9fafb" font-size="12" font-weight="600" font-family="system-ui"
            >{NODE_LABELS[node.type]}</text>
            <!-- Subtitle -->
            <text
              x={node.x + 16} y={node.y + 40}
              fill="#9ca3af" font-size="10" font-family="system-ui"
            >{nodeSubtitle(node)}</text>

            <!-- Ports -->
            {#each ports as port}
              {@const pp = portPos(node, port)}
              <g
                class="port"
                role="button"
                tabindex="0"
                onmousedown={(e) => e.stopPropagation()}
                onclick={(e) => onPortClick(e, node.id, port)}
                onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onPortClick(e as any, node.id, port); }}
                style="cursor: crosshair"
              >
                <circle
                  cx={pp.x} cy={pp.y} r={PORT_R}
                  fill={port.side === 'in' ? '#374151' : color}
                  stroke="#d1d5db"
                  stroke-width="1.5"
                />
                <text
                  x={pp.x + (port.side === 'in' ? 12 : -12)}
                  y={pp.y + 4}
                  fill="#9ca3af"
                  font-size="9"
                  text-anchor={port.side === 'in' ? 'start' : 'end'}
                  font-family="system-ui"
                >{port.label}</text>
              </g>
            {/each}

            <!-- Delete button (on hover) -->
            {#if hoverNodeId === node.id}
              <g
                class="delete-btn"
                role="button"
                tabindex="0"
                onclick={(e) => { e.stopPropagation(); removeNode(node.id); }}
                onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.stopPropagation(); removeNode(node.id); } }}
                onmousedown={(e) => e.stopPropagation()}
                style="cursor: pointer"
              >
                <circle
                  cx={node.x + NODE_W - 4} cy={node.y - 4}
                  r="9" fill="#ef4444" stroke="#1f2937" stroke-width="2"
                />
                <text
                  x={node.x + NODE_W - 4} y={node.y}
                  fill="white" font-size="12" text-anchor="middle"
                  font-family="system-ui" font-weight="700"
                >x</text>
              </g>
            {/if}
          </g>
        {/each}
      </g>

      <!-- Zoom indicator -->
      <text x="12" y="98%" fill="#6b7280" font-size="11" font-family="system-ui">
        {Math.round(zoom * 100)}%  |  {nodes.length} nodes  |  {wires.length} wires
      </text>
    </svg>

    <!-- ── Config panel (HTML overlay) ───────────────────────────────── -->
    {#if configNode}
      <div
        class="config-panel"
        role="application"
        style="left: {configPos.left}px; top: {configPos.top}px;"
        onmousedown={(e) => e.stopPropagation()}
      >
        <div class="config-header">
          <span class="config-dot" style="background: {NODE_COLORS[configNode.type]}"></span>
          <strong>{NODE_LABELS[configNode.type]} Config</strong>
          <button class="config-close" onclick={() => configNodeId = null}>x</button>
        </div>

        {#if configNode.type === 'source'}
          <label class="cfg-label">
            SimVar Name
            <input type="text" bind:value={configNode.config.var_name} placeholder="e.g. INDICATED ALTITUDE" />
          </label>
        {/if}

        {#if configNode.type === 'condition'}
          <label class="cfg-label">
            Operator
            <select bind:value={configNode.config.operator}>
              {#each operators as op}
                <option value={op.id}>{op.label} ({op.id})</option>
              {/each}
            </select>
          </label>
          <label class="cfg-label">
            Value
            <input type="number" bind:value={configNode.config.value} />
          </label>
          {#if configNode.config.operator === 'Between'}
            <label class="cfg-label">
              Value 2 (upper)
              <input type="number" bind:value={configNode.config.value2} />
            </label>
          {/if}
        {/if}

        {#if configNode.type === 'output'}
          <label class="cfg-label">
            Device ID
            <select bind:value={configNode.config.device_id}>
              {#each (devices ?? [{ id: 1, name: 'Device 1' }]) as dev}
                <option value={dev.id}>{dev.name ?? `Device ${dev.id}`}</option>
              {/each}
            </select>
          </label>
          <label class="cfg-label">
            Pin
            <input type="number" bind:value={configNode.config.pin} min="0" max="54" />
          </label>
          <label class="cfg-label">
            Output Value
            <input type="number" bind:value={configNode.config.output_value} />
          </label>
        {/if}

        {#if configNode.type === 'math'}
          <label class="cfg-label">
            Operation
            <select bind:value={configNode.config.operation}>
              {#each mathOps as op}
                <option value={op.id}>{op.label}</option>
              {/each}
            </select>
          </label>
          {#if configNode.config.operation === 'scale'}
            <div class="cfg-range-row">
              <label class="cfg-label cfg-half">
                In Min
                <input type="number" bind:value={configNode.config.range_in![0]} />
              </label>
              <label class="cfg-label cfg-half">
                In Max
                <input type="number" bind:value={configNode.config.range_in![1]} />
              </label>
            </div>
            <div class="cfg-range-row">
              <label class="cfg-label cfg-half">
                Out Min
                <input type="number" bind:value={configNode.config.range_out![0]} />
              </label>
              <label class="cfg-label cfg-half">
                Out Max
                <input type="number" bind:value={configNode.config.range_out![1]} />
              </label>
            </div>
          {:else}
            <label class="cfg-label">
              Parameter
              <input type="number" bind:value={configNode.config.param} />
            </label>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
</div>

<!-- ── Styles ──────────────────────────────────────────────────────── -->
<style>
  .flow-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 500px;
    background: #0f172a;
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid #1e293b;
  }

  /* ── Toolbar ─────────────────────────────────────────────────────── */
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    padding: 8px 12px;
    background: #1e293b;
    border-bottom: 1px solid #334155;
  }
  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .toolbar-right { margin-left: auto; }
  .toolbar-label {
    color: #94a3b8;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-right: 2px;
  }

  .btn-node, .btn-preset, .btn-save, .btn-clear {
    padding: 4px 10px;
    border: 1px solid #475569;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    color: #e2e8f0;
    background: #334155;
  }
  .btn-node:hover, .btn-preset:hover { background: #475569; }
  .btn-source  { border-color: #3b82f6; color: #93c5fd; }
  .btn-condition { border-color: #eab308; color: #fde68a; }
  .btn-output  { border-color: #22c55e; color: #86efac; }
  .btn-math    { border-color: #a855f7; color: #d8b4fe; }
  .btn-save {
    background: #3b82f6;
    border-color: #3b82f6;
    color: white;
    font-weight: 600;
  }
  .btn-save:hover { background: #2563eb; }
  .btn-clear {
    background: transparent;
    border-color: #ef4444;
    color: #fca5a5;
  }
  .btn-clear:hover { background: #7f1d1d; }

  /* ── Canvas ──────────────────────────────────────────────────────── */
  .canvas-wrap {
    flex: 1;
    position: relative;
    overflow: hidden;
  }
  .canvas {
    width: 100%;
    height: 100%;
    display: block;
    cursor: grab;
  }
  .canvas:active { cursor: grabbing; }

  .flow-node { cursor: grab; }
  .flow-node:active { cursor: grabbing; }

  .port circle:hover {
    r: 9;
    stroke-width: 2.5;
    stroke: #f9fafb;
  }

  .delete-btn circle:hover {
    fill: #dc2626;
  }

  /* ── Config Panel ────────────────────────────────────────────────── */
  .config-panel {
    position: absolute;
    z-index: 50;
    width: 240px;
    background: #1e293b;
    border: 1px solid #475569;
    border-radius: 10px;
    padding: 12px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.5);
    font-family: system-ui, sans-serif;
  }
  .config-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 10px;
    color: #f1f5f9;
    font-size: 13px;
  }
  .config-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
  }
  .config-close {
    margin-left: auto;
    background: none;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    padding: 0 4px;
  }
  .config-close:hover { color: #ef4444; }

  .cfg-label {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-bottom: 8px;
    color: #94a3b8;
    font-size: 11px;
    font-weight: 500;
  }
  .cfg-label input,
  .cfg-label select {
    padding: 5px 8px;
    background: #0f172a;
    border: 1px solid #334155;
    border-radius: 6px;
    color: #e2e8f0;
    font-size: 12px;
    outline: none;
  }
  .cfg-label input:focus,
  .cfg-label select:focus {
    border-color: #3b82f6;
  }

  .cfg-range-row {
    display: flex;
    gap: 8px;
  }
  .cfg-half { flex: 1; }
</style>
