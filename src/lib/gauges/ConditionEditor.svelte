<script lang="ts">
  import { invoke } from '../mock';
  import { toast } from './Toast.svelte';
  import { Zap, Plus, Trash2, Play, Pause } from 'lucide-svelte';

  let { aircraft, devices }: { aircraft?: any; devices?: any[] } = $props();

  let rules: any[] = $state(JSON.parse(localStorage.getItem('scb-conditions') || '[]'));
  let showAdd = $state(false);

  // New rule form
  let newRule = $state({
    varName: '',
    operator: 'GreaterThan',
    value: 0,
    value2: 0,
    outputDeviceId: 1,
    outputPin: 13,
    outputValue: 1,
    elseValue: 0,
    label: '',
  });

  const operators = [
    { id: 'GreaterThan', label: '>', desc: 'greater than' },
    { id: 'LessThan', label: '<', desc: 'less than' },
    { id: 'GreaterThanOrEqual', label: '>=', desc: 'greater or equal' },
    { id: 'LessThanOrEqual', label: '<=', desc: 'less or equal' },
    { id: 'Equals', label: '=', desc: 'equals' },
    { id: 'NotEquals', label: '!=', desc: 'not equal' },
    { id: 'Between', label: 'BETWEEN', desc: 'between two values' },
  ];

  // Common presets
  const presets = [
    { label: 'Gear Down → LED ON', varName: 'GEAR HANDLE POSITION', operator: 'GreaterThan', value: 0.5, pin: 13, outputValue: 1, elseValue: 0 },
    { label: 'Battery ON → LED ON', varName: 'MASTER BATTERY SWITCH', operator: 'Equals', value: 1, pin: 40, outputValue: 1, elseValue: 0 },
    { label: 'AP Active → LED ON', varName: 'AUTOPILOT MASTER', operator: 'Equals', value: 1, pin: 43, outputValue: 1, elseValue: 0 },
    { label: 'Low Fuel Warning', varName: 'FUEL TANK LEFT LEVEL', operator: 'LessThan', value: 15, pin: 44, outputValue: 1, elseValue: 0 },
    { label: 'Overspeed Warning', varName: 'AIRSPEED INDICATED', operator: 'GreaterThan', value: 250, pin: 45, outputValue: 1, elseValue: 0 },
    { label: 'Altitude Display', varName: 'INDICATED ALTITUDE', operator: 'GreaterThanOrEqual', value: 0, pin: 200, outputValue: 1, elseValue: 0 },
  ];

  let allVars = $derived([
    ...(aircraft?.variables || []).map((v: any) => v.name),
    'GEAR HANDLE POSITION', 'MASTER BATTERY SWITCH', 'AUTOPILOT MASTER',
    'INDICATED ALTITUDE', 'AIRSPEED INDICATED', 'FUEL TANK LEFT LEVEL',
  ].filter((v: any, i: any, a: any) => a.indexOf(v) === i));

  function addRule() {
    if (!newRule.varName) { toast('Select a variable', 'warning'); return; }
    const rule = {
      label: newRule.label || `${newRule.varName} ${operators.find(o => o.id === newRule.operator)?.label} ${newRule.value}`,
      conditions: [{
        var_name: newRule.varName,
        operator: newRule.operator,
        value: newRule.value,
        value2: newRule.operator === 'Between' ? newRule.value2 : null,
      }],
      output_device_id: newRule.outputDeviceId,
      output_pin: newRule.outputPin,
      output_value: newRule.outputValue,
      else_value: newRule.elseValue,
      enabled: true,
    };
    rules = [...rules, rule];
    saveRules();
    showAdd = false;
    toast('Rule added', 'success');
  }

  function applyPreset(preset: any) {
    newRule.varName = preset.varName;
    newRule.operator = preset.operator;
    newRule.value = preset.value;
    newRule.outputPin = preset.pin;
    newRule.outputValue = preset.outputValue;
    newRule.elseValue = preset.elseValue;
    newRule.label = preset.label;
    showAdd = true;
  }

  function removeRule(i: number) {
    rules = rules.filter((_: any, j: number) => j !== i);
    saveRules();
    toast('Rule removed', 'info');
  }

  function toggleRule(i: number) {
    rules[i].enabled = !rules[i].enabled;
    rules = [...rules];
    saveRules();
  }

  function saveRules() {
    localStorage.setItem('scb-conditions', JSON.stringify(rules));
  }

  function operatorLabel(op: string): string {
    return operators.find(o => o.id === op)?.label || op;
  }
</script>

<div class="card">
  <h3>
    <Zap size={14} /> Conditional Logic
    <button class="btn btn-primary btn-sm" style="margin-left: auto" onclick={() => showAdd = !showAdd}>
      {showAdd ? 'Cancel' : '+ Add Rule'}
    </button>
  </h3>

  <!-- Presets -->
  {#if !showAdd && rules.length === 0}
    <div style="margin-bottom: 16px">
      <div style="font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 1px; color: var(--text-dim); margin-bottom: 8px">Quick Presets</div>
      <div style="display: flex; flex-wrap: wrap; gap: 6px">
        {#each presets as preset}
          <button class="btn btn-sm" onclick={() => applyPreset(preset)}>
            <Zap size={10} /> {preset.label}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Add Rule Form -->
  {#if showAdd}
    <div style="padding: 16px; background: var(--bg-instrument); border: 1px solid var(--border-panel); border-radius: var(--radius-lg); margin-bottom: 16px">
      <div style="font-size: 11px; font-weight: 600; margin-bottom: 12px; color: var(--text-bright)">IF</div>

      <div style="display: grid; grid-template-columns: 2fr 100px 80px 80px; gap: 8px; align-items: end">
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px">Variable</span>
          <select bind:value={newRule.varName} style="width: 100%">
            <option value="">Select...</option>
            {#each allVars as v}
              <option value={v}>{v}</option>
            {/each}
          </select>
        </div>
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px">Operator</span>
          <select bind:value={newRule.operator} style="width: 100%">
            {#each operators as op}
              <option value={op.id}>{op.label} {op.desc}</option>
            {/each}
          </select>
        </div>
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px">Value</span>
          <input type="number" bind:value={newRule.value} step="any" style="width: 100%" />
        </div>
        {#if newRule.operator === 'Between'}
          <div>
            <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px">AND</span>
            <input type="number" bind:value={newRule.value2} step="any" style="width: 100%" />
          </div>
        {/if}
      </div>

      <div style="font-size: 11px; font-weight: 600; margin: 12px 0 8px; color: var(--pfd-green)">THEN → Send to device</div>

      <div style="display: grid; grid-template-columns: 80px 80px 80px 80px 1fr; gap: 8px; align-items: end">
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px">Device ID</span>
          <input type="number" bind:value={newRule.outputDeviceId} min="1" max="255" style="width: 100%" />
        </div>
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px">Pin</span>
          <input type="number" bind:value={newRule.outputPin} min="0" max="255" style="width: 100%" />
        </div>
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--pfd-green); margin-bottom: 4px">TRUE</span>
          <input type="number" bind:value={newRule.outputValue} style="width: 100%" />
        </div>
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--warning); margin-bottom: 4px">FALSE</span>
          <input type="number" bind:value={newRule.elseValue} style="width: 100%" />
        </div>
        <div>
          <span style="display: block; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 4px">Label</span>
          <input type="text" bind:value={newRule.label} placeholder="e.g. Gear LED" style="width: 100%" />
        </div>
      </div>

      <div style="display: flex; gap: 8px; margin-top: 12px; justify-content: flex-end">
        <button class="btn btn-sm" onclick={() => showAdd = false}>Cancel</button>
        <button class="btn btn-primary btn-sm" onclick={addRule}>
          <Plus size={12} /> Add Rule
        </button>
      </div>
    </div>
  {/if}

  <!-- Active Rules -->
  {#if rules.length > 0}
    <div style="display: flex; flex-direction: column; gap: 6px">
      {#each rules as rule, i}
        <div style="display: flex; align-items: center; gap: 10px; padding: 10px 14px; background: var(--bg-instrument); border: 1px solid {rule.enabled ? 'var(--border-panel)' : 'var(--border)'}; border-radius: var(--radius); opacity: {rule.enabled ? 1 : 0.5}">
          <button style="background: none; border: none; cursor: pointer; color: {rule.enabled ? 'var(--pfd-green)' : 'var(--text-muted)'}; padding: 0" onclick={() => toggleRule(i)} title={rule.enabled ? 'Disable' : 'Enable'}>
            {#if rule.enabled}<Play size={14} />{:else}<Pause size={14} />{/if}
          </button>

          <div style="flex: 1">
            <div style="font-size: 12px; font-weight: 600; color: var(--text-bright)">{rule.label}</div>
            <div style="font-family: var(--mono); font-size: 10px; color: var(--text-dim); margin-top: 2px">
              IF {rule.conditions[0]?.var_name}
              {operatorLabel(rule.conditions[0]?.operator)}
              {rule.conditions[0]?.value}
              {#if rule.conditions[0]?.operator === 'Between'} AND {rule.conditions[0]?.value2}{/if}
              → Pin {rule.output_pin}
              = <span style="color: var(--pfd-green)">{rule.output_value}</span>
              / <span style="color: var(--warning)">{rule.else_value}</span>
            </div>
          </div>

          <span class="badge" style="background: {rule.enabled ? 'var(--pfd-green-dim)' : 'var(--bg-bezel)'}; color: {rule.enabled ? 'var(--pfd-green)' : 'var(--text-muted)'}">
            {rule.enabled ? 'ON' : 'OFF'}
          </span>

          <button class="btn btn-danger btn-sm" onclick={() => removeRule(i)}>
            <Trash2 size={12} />
          </button>
        </div>
      {/each}
    </div>
  {:else if !showAdd}
    <div style="font-size: 12px; color: var(--text-dim); text-align: center; padding: 16px">
      No rules configured. Use presets above or create custom rules.
    </div>
  {/if}
</div>
