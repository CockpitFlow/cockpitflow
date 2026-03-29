/// Mock layer for browser preview (no Tauri runtime).
/// When running inside Tauri, the real invoke is used.

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

const mockPorts: any[] = [
  {
    name: 'COM3', port_type: 'USB', device_kind: 'esp32', flashable: true,
    vid: '303A', pid: '1001', manufacturer: 'Espressif', product: 'ESP32-C6',
  },
  {
    name: 'COM5', port_type: 'USB', device_kind: 'arduino', flashable: true,
    vid: '2341', pid: '0042', manufacturer: 'Arduino', product: 'Arduino Mega 2560',
  },
];

const mockDevices: any[] = [
  { id: 1, name: 'Throttle Panel', transport: { type: 'Serial', port: 'COM3', baud_rate: 115200 } },
  { id: 2, name: 'MCP Panel', transport: { type: 'Serial', port: 'COM5', baud_rate: 115200 } },
];

const mockMappings: any[] = [
  { device_id: 1, pin: 0, simvar: 'GENERAL ENG THROTTLE LEVER POSITION:1', direction: 'input' },
  { device_id: 1, pin: 1, simvar: 'GENERAL ENG MIXTURE LEVER POSITION:1', direction: 'input' },
  { device_id: 2, pin: 2, simvar: 'TOGGLE_MASTER_BATTERY', direction: 'input' },
  { device_id: 2, pin: 24, simvar: 'AUTOPILOT HEADING LOCK DIR', direction: 'input' },
];

const mockSimVars = [
  { name: 'INDICATED ALTITUDE', unit: 'feet', writable: false },
  { name: 'AIRSPEED INDICATED', unit: 'knots', writable: false },
  { name: 'HEADING INDICATOR', unit: 'degrees', writable: false },
  { name: 'VERTICAL SPEED', unit: 'feet per minute', writable: false },
  { name: 'GENERAL ENG THROTTLE LEVER POSITION:1', unit: 'percent', writable: true },
  { name: 'GENERAL ENG THROTTLE LEVER POSITION:2', unit: 'percent', writable: true },
  { name: 'GEAR HANDLE POSITION', unit: 'bool', writable: true },
  { name: 'FLAPS HANDLE INDEX', unit: 'number', writable: true },
  { name: 'AUTOPILOT MASTER', unit: 'bool', writable: false },
  { name: 'AUTOPILOT HEADING LOCK DIR', unit: 'degrees', writable: true },
  { name: 'AUTOPILOT ALTITUDE LOCK VAR', unit: 'feet', writable: true },
  { name: 'MASTER BATTERY SWITCH', unit: 'bool', writable: false },
  { name: 'NAV OBS:1', unit: 'degrees', writable: true },
  { name: 'COM ACTIVE FREQUENCY:1', unit: 'mhz', writable: false },
  { name: 'LIGHT LANDING', unit: 'bool', writable: false },
];

let simTime = 0;

function mockSimState() {
  simTime += 0.1;
  return {
    connected: true,
    paused: false,
    values: {
      'INDICATED ALTITUDE': 5280 + Math.sin(simTime * 0.3) * 200,
      'AIRSPEED INDICATED': 145 + Math.sin(simTime * 0.5) * 10,
      'HEADING INDICATOR': (270 + simTime * 2) % 360,
      'VERTICAL SPEED': Math.sin(simTime * 0.3) * 500,
      'GENERAL ENG THROTTLE LEVER POSITION:1': 72 + Math.sin(simTime * 0.2) * 5,
      'GENERAL ENG THROTTLE LEVER POSITION:2': 72 + Math.sin(simTime * 0.2) * 5,
      'GEAR HANDLE POSITION': 1,
      'FLAPS HANDLE INDEX': 2,
      'AUTOPILOT MASTER': 1,
      'AUTOPILOT HEADING LOCK DIR': 270,
      'AUTOPILOT ALTITUDE LOCK VAR': 5000,
      'MASTER BATTERY SWITCH': 1,
      'NAV OBS:1': 180,
      'COM ACTIVE FREQUENCY:1': 118.3,
      'LIGHT LANDING': 0,
    },
  };
}

// ─── Mock Profile Data ───

const mockDeviceProfiles = [
  {
    version: 1, id: 'arduino-mega-cockpit', name: 'Arduino Mega 2560 — Full Cockpit Panel',
    author: 'SkyFlow', description: '54 digital I/O + 16 analog inputs.',
    board: 'arduino-mega', transport: 'serial',
    pins: [
      { pin: 2, type: 'button', label: 'Master Battery' },
      { pin: 3, type: 'button', label: 'Alternator' },
      { pin: 7, type: 'toggle', label: 'Landing Gear' },
      { pin: 8, type: 'button', label: 'Landing Light' },
      { pin: 17, type: 'button', label: 'AP Master' },
      { pin: 24, type: 'encoder', label: 'Heading Bug', options: { pin_b: 25, steps_per_detent: 4 } },
      { pin: 26, type: 'encoder', label: 'AP Altitude', options: { pin_b: 27, steps_per_detent: 4 } },
      { pin: 40, type: 'led', label: 'Gear Nose Green' },
      { pin: 43, type: 'led', label: 'AP Engaged LED' },
      { pin: 'A0', type: 'potentiometer', label: 'Throttle 1', options: { deadzone: 3 } },
      { pin: 'A2', type: 'potentiometer', label: 'Mixture', options: { deadzone: 3 } },
      { pin: 'A5', type: 'potentiometer', label: 'Elevator Trim', options: { deadzone: 3 } },
    ],
  },
  {
    version: 1, id: 'esp32-radio-panel', name: 'ESP32 — Radio & AP Panel (WiFi)',
    author: 'SkyFlow', description: 'Encoders + 7-segment displays for radio tuning.',
    board: 'esp32', transport: 'udp',
    pins: [
      { pin: 13, type: 'encoder', label: 'COM1 MHz', options: { pin_b: 14, steps_per_detent: 4 } },
      { pin: 15, type: 'encoder', label: 'COM1 kHz', options: { pin_b: 16, steps_per_detent: 4 } },
      { pin: 17, type: 'button', label: 'COM1 Swap' },
      { pin: 2, type: 'display', label: 'COM1 Active Display', options: { digits: 6, protocol: 'tm1637' } },
    ],
  },
];

const mockAircraftProfiles = [
  {
    version: 1, id: 'cessna-172-skyhawk', name: 'Cessna 172 Skyhawk',
    author: 'SkyFlow', description: 'Standard Cessna 172 with steam gauges.',
    category: 'GA', simulator: 'both',
    variables: [
      { name: 'INDICATED ALTITUDE', unit: 'feet', writable: false, category: 'flight', label: 'Altitude' },
      { name: 'AIRSPEED INDICATED', unit: 'knots', writable: false, category: 'flight', label: 'Airspeed' },
      { name: 'HEADING INDICATOR', unit: 'degrees', writable: false, category: 'flight', label: 'Heading' },
      { name: 'VERTICAL SPEED', unit: 'feet per minute', writable: false, category: 'flight', label: 'VS' },
      { name: 'GENERAL ENG THROTTLE LEVER POSITION:1', unit: 'percent', writable: true, category: 'engine', label: 'Throttle' },
      { name: 'GENERAL ENG MIXTURE LEVER POSITION:1', unit: 'percent', writable: true, category: 'engine', label: 'Mixture' },
      { name: 'GENERAL ENG RPM:1', unit: 'rpm', writable: false, category: 'engine', label: 'RPM' },
      { name: 'MASTER BATTERY SWITCH', unit: 'bool', writable: false, category: 'electrical', label: 'Master Battery' },
      { name: 'AUTOPILOT MASTER', unit: 'bool', writable: false, category: 'autopilot', label: 'AP Master' },
      { name: 'COM ACTIVE FREQUENCY:1', unit: 'mhz', writable: false, category: 'radio', label: 'COM1' },
      { name: 'LIGHT LANDING', unit: 'bool', writable: false, category: 'lights', label: 'Landing Light' },
    ],
    events: [
      { name: 'TOGGLE_MASTER_BATTERY', category: 'electrical', label: 'Toggle Battery' },
      { name: 'MAGNETO_BOTH', category: 'engine', label: 'Magneto BOTH' },
      { name: 'FLAPS_INCR', category: 'flight', label: 'Flaps +' },
      { name: 'AP_MASTER', category: 'autopilot', label: 'AP Toggle' },
      { name: 'TOGGLE_LANDING_LIGHTS', category: 'lights', label: 'Landing Light' },
      { name: 'PARKING_BRAKES', category: 'misc', label: 'Parking Brake' },
    ],
  },
  {
    version: 1, id: 'boeing-737-800', name: 'Boeing 737-800',
    author: 'SkyFlow', description: 'Standard 737 cockpit with MCP and overhead.',
    category: 'airliner', simulator: 'both',
    variables: [
      { name: 'INDICATED ALTITUDE', unit: 'feet', writable: false, category: 'flight', label: 'Altitude' },
      { name: 'AIRSPEED INDICATED', unit: 'knots', writable: false, category: 'flight', label: 'IAS' },
      { name: 'GENERAL ENG N1:1', unit: 'percent', writable: false, category: 'engine', label: 'N1 Eng 1' },
      { name: 'GENERAL ENG N1:2', unit: 'percent', writable: false, category: 'engine', label: 'N1 Eng 2' },
      { name: 'AUTOPILOT AIRSPEED HOLD VAR', unit: 'knots', writable: true, category: 'autopilot', label: 'MCP Speed' },
      { name: 'AUTOPILOT HEADING LOCK DIR', unit: 'degrees', writable: true, category: 'autopilot', label: 'MCP HDG' },
      { name: 'AUTOPILOT ALTITUDE LOCK VAR', unit: 'feet', writable: true, category: 'autopilot', label: 'MCP ALT' },
    ],
    events: [
      { name: 'AP_MASTER', category: 'autopilot', label: 'AP Engage' },
      { name: 'HEADING_BUG_INC', category: 'autopilot', label: 'MCP HDG +1' },
      { name: 'AP_ALT_VAR_INC', category: 'autopilot', label: 'MCP ALT +100' },
      { name: 'GEAR_TOGGLE', category: 'flight', label: 'Gear Toggle' },
      { name: 'TOGGLE_LANDING_LIGHTS', category: 'lights', label: 'Landing Lights' },
    ],
  },
  {
    version: 1, id: 'airbus-a320neo', name: 'Airbus A320neo',
    author: 'SkyFlow', description: 'A320neo with FCU, overhead, and pedestal.',
    category: 'airliner', simulator: 'both',
    variables: [
      { name: 'INDICATED ALTITUDE', unit: 'feet', writable: false, category: 'flight', label: 'Altitude' },
      { name: 'AIRSPEED INDICATED', unit: 'knots', writable: false, category: 'flight', label: 'IAS' },
      { name: 'GENERAL ENG N1:1', unit: 'percent', writable: false, category: 'engine', label: 'N1 Eng 1' },
      { name: 'AUTOPILOT AIRSPEED HOLD VAR', unit: 'knots', writable: true, category: 'autopilot', label: 'FCU Speed' },
      { name: 'AUTOPILOT HEADING LOCK DIR', unit: 'degrees', writable: true, category: 'autopilot', label: 'FCU HDG' },
      { name: 'APU PCT RPM', unit: 'percent', writable: false, category: 'electrical', label: 'APU N%' },
    ],
    events: [
      { name: 'AP_MASTER', category: 'autopilot', label: 'AP1 Toggle' },
      { name: 'TOGGLE_FLIGHT_DIRECTOR', category: 'autopilot', label: 'FD Toggle' },
      { name: 'APU_STARTER', category: 'electrical', label: 'APU Start' },
      { name: 'FLAPS_INCR', category: 'flight', label: 'Config +1' },
    ],
  },
];

const mockMappingProfiles = [
  {
    version: 1, id: 'mega-cessna172-full', name: 'Arduino Mega → Cessna 172 (Full Setup)',
    author: 'SkyFlow', description: 'Complete mapping for a Cessna 172 cockpit.',
    device_profile: 'arduino-mega-cockpit', aircraft_profile: 'cessna-172-skyhawk',
    mappings: [
      { pin: 2, action: 'event', target: 'TOGGLE_MASTER_BATTERY' },
      { pin: 7, action: 'event', target: 'GEAR_TOGGLE' },
      { pin: 24, action: 'event', target: 'HEADING_BUG_INC', options: { event_dec: 'HEADING_BUG_DEC' } },
      { pin: 'A0', action: 'set_var', target: 'GENERAL ENG THROTTLE LEVER POSITION:1', options: { range: [0, 100] } },
      { pin: 40, action: 'read_var', target: 'GEAR CENTER POSITION', options: { threshold: 0.5 } },
      { pin: 43, action: 'read_var', target: 'AUTOPILOT MASTER', options: { threshold: 0.5 } },
    ],
  },
  {
    version: 1, id: 'esp32-radio-cessna172', name: 'ESP32 Radio Panel → Cessna 172',
    author: 'SkyFlow', description: 'WiFi radio panel mapping.',
    device_profile: 'esp32-radio-panel', aircraft_profile: 'cessna-172-skyhawk',
    mappings: [
      { pin: 13, action: 'event', target: 'COM_RADIO_WHOLE_INC', options: { event_dec: 'COM_RADIO_WHOLE_DEC' } },
      { pin: 17, action: 'event', target: 'COM_STBY_RADIO_SWAP' },
      { pin: 2, action: 'read_var', target: 'COM ACTIVE FREQUENCY:1' },
    ],
  },
];

const mockHandlers: Record<string, (...args: any[]) => any> = {
  list_serial_ports: () => mockPorts,
  get_devices: () => mockDevices,
  get_mappings: () => mockMappings,
  get_sim_state: () => mockSimState(),
  get_default_simvars: () => mockSimVars,
  add_device: () => {},
  remove_device: () => {},
  add_mapping: () => {},
  remove_mapping: () => {},
  save_config: () => {},
  get_device_profiles: () => mockDeviceProfiles,
  get_aircraft_profiles: () => mockAircraftProfiles,
  get_mapping_profiles: () => mockMappingProfiles,
  get_aircraft_profile: (args: any) => mockAircraftProfiles.find(a => a.id === args?.id) || null,
  get_compatible_mappings: (args: any) => mockMappingProfiles.filter(m => m.device_profile === args?.device_id && m.aircraft_profile === args?.aircraft_id),
  import_profile: (args: any) => {
    const data = JSON.parse(args?.json || '{}');
    return `${data.$schema || 'unknown'}:${data.id || 'imported'}`;
  },
  reload_profiles: () => {},
  detect_board: () => ({ chip: 'ESP32-C6 (USB-Serial/JTAG) + SimConnect Firmware OK', features: 'VID:303A PID:1001, Espressif, SN:58:8C:81:52:D4:BC' }),
  flash_firmware: () => 'Firmware flashed successfully!',
  configure_device: (args: any) => `OK:${args?.command?.split(':')[0] || 'CMD'}`,
  probe_device: () => ({ name: 'Throttle Panel', device_id: 1, wifi_connected: false, wifi_ip: null }),
  test_inputs: () => [
    { pin: 2, value: 1, event_type: 'press', timestamp_ms: 1200 },
    { pin: 2, value: 0, event_type: 'release', timestamp_ms: 1400 },
    { pin: 0, value: 1, event_type: 'press', timestamp_ms: 3100 },
    { pin: 0, value: 0, event_type: 'release', timestamp_ms: 3300 },
    { pin: 0, value: 0, event_type: 'heartbeat', timestamp_ms: 2000 },
    { pin: 0, value: 0, event_type: 'heartbeat', timestamp_ms: 4000 },
  ],
  toggle_profile: () => {},
  check_for_updates: () => ({ available: true, current_version: '0.1.0', latest_version: '0.2.0', new_profiles: 3, changelog: 'Added Boeing 777, Airbus A330, improved encoder support' }),
  list_checklists: () => [
    { id: 'cessna-172', name: 'Cessna 172 Skyhawk', category: 'GA', phases_count: 7, items_count: 42 },
    { id: 'boeing-737-800', name: 'Boeing 737-800', category: 'Airliner', phases_count: 9, items_count: 89 },
    { id: 'airbus-a320neo', name: 'Airbus A320neo', category: 'Airliner', phases_count: 8, items_count: 76 },
  ],
  load_checklist: (args: any) => {
    const mockChecklist: Record<string, any> = {
      'cessna-172': {
        id: 'cessna-172', name: 'Cessna 172 Skyhawk', category: 'GA', author: 'SkyFlow', version: '1.0',
        phases: [
          { id: 'preflight', name: 'Pre-Flight', items: [
            { id: 'documents', label: 'Documents (ARROW)', expected: 'CHECK', auto_detect: null, how_to: 'Verify ARROW documents are on board.', location: 'Glove compartment' },
            { id: 'fuel-quantity', label: 'Fuel Quantity', expected: 'CHECK', auto_detect: { var_name: 'FUEL TOTAL QUANTITY WEIGHT', var_type: 'A', condition: '> 0' }, how_to: 'Visually check fuel level.', location: 'Wing fuel tanks' },
            { id: 'master-switch', label: 'Master Switch', expected: 'ON', auto_detect: { var_name: 'ELECTRICAL MASTER BATTERY', var_type: 'A', condition: '> 0' }, how_to: 'Push master switch to ON.', location: 'Lower left instrument panel' },
            { id: 'fuel-gauges', label: 'Fuel Gauges', expected: 'CHECK', auto_detect: null, how_to: 'Compare gauge readings against visual inspection.', location: 'Instrument panel' },
            { id: 'flaps', label: 'Flaps', expected: 'UP', auto_detect: { var_name: 'TRAILING EDGE FLAPS LEFT ANGLE', var_type: 'A', condition: '== 0' }, how_to: 'Move flap lever fully up.', location: 'Center console' },
          ]},
          { id: 'engine-start', name: 'Engine Start', items: [
            { id: 'mixture', label: 'Mixture', expected: 'RICH', auto_detect: { var_name: 'GENERAL ENG MIXTURE LEVER POSITION:1', var_type: 'A', condition: '> 90' }, how_to: 'Push mixture knob fully in (rich).', location: 'Center console' },
            { id: 'throttle', label: 'Throttle', expected: '1/4 INCH', auto_detect: null, how_to: 'Advance throttle approximately 1/4 inch.', location: 'Center console' },
            { id: 'ignition', label: 'Ignition Switch', expected: 'START', auto_detect: null, how_to: 'Turn ignition key to START, release when engine catches.', location: 'Lower left panel' },
          ]},
          { id: 'taxi', name: 'Taxi', items: [
            { id: 'parking-brake', label: 'Parking Brake', expected: 'RELEASE', auto_detect: { var_name: 'BRAKE PARKING POSITION', var_type: 'A', condition: '== 0' }, how_to: 'Release parking brake.', location: 'Center floor' },
            { id: 'radios', label: 'Radios', expected: 'SET', auto_detect: null, how_to: 'Set COM/NAV frequencies as required.', location: 'Radio stack' },
          ]},
          { id: 'takeoff', name: 'Takeoff', items: [
            { id: 'flaps-takeoff', label: 'Flaps', expected: '0-10 DEG', auto_detect: null, how_to: 'Set flaps for takeoff (0-10 degrees).', location: 'Center console' },
            { id: 'trim', label: 'Elevator Trim', expected: 'TAKEOFF', auto_detect: null, how_to: 'Set trim to takeoff position.', location: 'Trim wheel' },
          ]},
        ],
      },
    };
    return mockChecklist[args?.id] || { id: args?.id, name: 'Unknown', category: '', phases: [] };
  },
  get_local_ip: () => '192.168.1.100',
};

let tauriInvoke: typeof import('@tauri-apps/api/core').invoke | null = null;
let tauriReady: Promise<void> | null = null;

if (isTauri) {
  tauriReady = import('@tauri-apps/api/core').then(m => { tauriInvoke = m.invoke; });
}

export async function invoke(cmd: string, args?: any): Promise<any> {
  // Wait for Tauri to be ready before falling back to mock
  if (tauriReady) {
    await tauriReady;
  }
  if (tauriInvoke) {
    return tauriInvoke(cmd, args);
  }
  const handler = mockHandlers[cmd];
  if (handler) return handler(args);
  console.warn(`[mock] Unknown command: ${cmd}`);
  return null;
}
