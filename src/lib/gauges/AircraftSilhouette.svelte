<script lang="ts">
  let { category = 'GA', aircraftId = '', active = false }: {
    category?: string;
    aircraftId?: string;
    active?: boolean;
  } = $props();

  // Specific aircraft silhouettes (top-down view, realistic proportions)
  const specificAircraft: Record<string, string> = {
    // ─── Boeing 737 ───
    'boeing-737-800': `<svg viewBox="0 0 200 80" fill="none">
      <!-- Fuselage -->
      <path d="M15 40 Q15 35 30 34 L160 32 Q185 32 190 38 L190 42 Q185 48 160 48 L30 46 Q15 45 15 40Z" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.06"/>
      <!-- Wings -->
      <path d="M70 34 L55 8 Q54 5 57 5 L95 5 Q97 5 96 8 L85 34" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <path d="M70 46 L55 72 Q54 75 57 75 L95 75 Q97 75 96 72 L85 46" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <!-- Engines -->
      <ellipse cx="65" cy="18" rx="4" ry="8" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.1"/>
      <ellipse cx="65" cy="62" rx="4" ry="8" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.1"/>
      <!-- Horizontal stabilizer -->
      <path d="M165 32 L158 18 Q157 16 160 16 L175 16 Q177 16 176 18 L172 32" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <path d="M165 48 L158 62 Q157 64 160 64 L175 64 Q177 64 176 62 L172 48" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <!-- Vertical stabilizer -->
      <line x1="178" y1="34" x2="188" y2="34" stroke="currentColor" stroke-width="1.5" opacity="0.5"/>
      <!-- Cockpit windows -->
      <path d="M18 38 L25 36 L25 44 L18 42Z" fill="currentColor" fill-opacity="0.15"/>
    </svg>`,
    'pmdg-737-800': '', // will fallback to boeing-737-800

    // ─── Airbus A320 ───
    'airbus-a320neo': `<svg viewBox="0 0 200 80" fill="none">
      <path d="M12 40 Q12 35 28 34 L162 32 Q188 32 192 38 L192 42 Q188 48 162 48 L28 46 Q12 45 12 40Z" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.06"/>
      <!-- Wings (slightly more swept than 737) -->
      <path d="M72 34 L52 6 Q51 3 54 3 L98 3 Q100 3 99 6 L88 34" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <path d="M72 46 L52 74 Q51 77 54 77 L98 77 Q100 77 99 74 L88 46" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <!-- Engines (larger CFM LEAP) -->
      <ellipse cx="66" cy="16" rx="5" ry="9" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.1"/>
      <ellipse cx="66" cy="64" rx="5" ry="9" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.1"/>
      <!-- H-stab -->
      <path d="M168 32 L160 20 Q159 18 162 18 L178 18 Q180 18 179 20 L174 32" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <path d="M168 48 L160 60 Q159 62 162 62 L178 62 Q180 62 179 60 L174 48" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <!-- Sharklets (winglets at wingtip) -->
      <line x1="54" y1="5" x2="50" y2="3" stroke="currentColor" stroke-width="1.2"/>
      <line x1="54" y1="75" x2="50" y2="77" stroke="currentColor" stroke-width="1.2"/>
      <path d="M15 38 L23 36 L23 44 L15 42Z" fill="currentColor" fill-opacity="0.15"/>
    </svg>`,
    'fbw-a320neo': '', // fallback to airbus-a320neo
    'fenix-a320': '', // fallback to airbus-a320neo

    // ─── Cessna 172 ───
    'cessna-172-skyhawk': `<svg viewBox="0 0 200 80" fill="none">
      <!-- Fuselage (smaller, GA proportions) -->
      <path d="M35 40 Q35 37 45 36 L150 36 Q170 36 175 39 L175 41 Q170 44 150 44 L45 44 Q35 43 35 40Z" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.06"/>
      <!-- High wing -->
      <path d="M70 36 L25 20 Q22 19 23 16 L25 14 L115 14 L117 16 Q118 19 115 20 L95 36" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.05"/>
      <path d="M70 44 L25 60 Q22 61 23 64 L25 66 L115 66 L117 64 Q118 61 115 60 L95 44" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.05"/>
      <!-- Wing struts -->
      <line x1="60" y1="36" x2="45" y2="18" stroke="currentColor" stroke-width="0.5" opacity="0.4"/>
      <line x1="60" y1="44" x2="45" y2="62" stroke="currentColor" stroke-width="0.5" opacity="0.4"/>
      <!-- H-stab -->
      <path d="M155 36 L148 26 L168 26 L164 36" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <path d="M155 44 L148 54 L168 54 L164 44" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <!-- Propeller -->
      <line x1="37" y1="30" x2="37" y2="50" stroke="currentColor" stroke-width="1.5" opacity="0.3"/>
      <circle cx="37" cy="40" r="1.5" fill="currentColor" fill-opacity="0.3"/>
      <!-- Cockpit -->
      <rect x="48" y="37" width="10" height="6" rx="2" fill="currentColor" fill-opacity="0.1"/>
    </svg>`,

    // ─── Piper PA-28 Cherokee ───
    'piper-pa28-cherokee': `<svg viewBox="0 0 200 80" fill="none">
      <path d="M38 40 Q38 37 48 36 L148 36 Q168 36 172 39 L172 41 Q168 44 148 44 L48 44 Q38 43 38 40Z" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.06"/>
      <!-- Low wing -->
      <path d="M72 44 L30 60 Q27 61 28 64 L30 65 L112 65 L114 64 Q115 61 112 60 L92 44" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.05"/>
      <path d="M72 36 L30 20 Q27 19 28 16 L30 15 L112 15 L114 16 Q115 19 112 20 L92 36" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.05"/>
      <!-- H-stab -->
      <path d="M152 36 L146 28 L166 28 L162 36" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <path d="M152 44 L146 52 L166 52 L162 44" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <!-- Propeller -->
      <line x1="40" y1="31" x2="40" y2="49" stroke="currentColor" stroke-width="1.5" opacity="0.3"/>
      <circle cx="40" cy="40" r="1.5" fill="currentColor" fill-opacity="0.3"/>
      <!-- Nosewheel -->
      <circle cx="55" cy="40" r="1" fill="currentColor" fill-opacity="0.2"/>
    </svg>`,

    // ─── TBM 930 (turboprop) ───
    'daher-tbm-930': `<svg viewBox="0 0 200 80" fill="none">
      <path d="M32 40 Q32 37 42 36 L155 35 Q175 35 180 39 L180 41 Q175 45 155 45 L42 44 Q32 43 32 40Z" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.06"/>
      <!-- Low wing (longer, thinner) -->
      <path d="M75 44 L28 62 Q25 63 26 66 L28 67 L118 67 L120 66 Q121 63 118 62 L98 44" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <path d="M75 36 L28 18 Q25 17 26 14 L28 13 L118 13 L120 14 Q121 17 118 18 L98 36" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <!-- T-tail -->
      <line x1="175" y1="35" x2="185" y2="35" stroke="currentColor" stroke-width="1.5" opacity="0.5"/>
      <path d="M182 35 L178 28 L192 28 L188 35" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <path d="M182 45 L178 52 L192 52 L188 45" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
      <!-- Propeller -->
      <line x1="34" y1="30" x2="34" y2="50" stroke="currentColor" stroke-width="1.5" opacity="0.3"/>
      <circle cx="34" cy="40" r="2" fill="currentColor" fill-opacity="0.3"/>
    </svg>`,

    // ─── King Air 350 (twin turboprop) ───
    'beechcraft-king-air-350': `<svg viewBox="0 0 200 80" fill="none">
      <path d="M28 40 Q28 36 40 35 L158 34 Q178 34 182 39 L182 41 Q178 46 158 46 L40 45 Q28 44 28 40Z" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.06"/>
      <!-- Low wing -->
      <path d="M68 45 L22 65 Q19 66 20 69 L22 70 L118 70 L120 69 Q121 66 118 65 L95 45" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <path d="M68 35 L22 15 Q19 14 20 11 L22 10 L118 10 L120 11 Q121 14 118 15 L95 35" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.04"/>
      <!-- Twin engines on wings -->
      <ellipse cx="52" cy="22" rx="3" ry="6" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.1"/>
      <ellipse cx="52" cy="58" rx="3" ry="6" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.1"/>
      <!-- Props -->
      <line x1="49" y1="14" x2="49" y2="30" stroke="currentColor" stroke-width="1" opacity="0.25"/>
      <line x1="49" y1="50" x2="49" y2="66" stroke="currentColor" stroke-width="1" opacity="0.25"/>
      <!-- T-tail -->
      <line x1="177" y1="35" x2="187" y2="35" stroke="currentColor" stroke-width="1.5" opacity="0.5"/>
      <path d="M184 35 L180 27 L194 27 L190 35" stroke="currentColor" stroke-width="0.7" fill="currentColor" fill-opacity="0.03"/>
      <path d="M184 45 L180 53 L194 53 L190 45" stroke="currentColor" stroke-width="0.7" fill="currentColor" fill-opacity="0.03"/>
    </svg>`,
  };

  // Generic fallbacks by category
  const categoryFallback: Record<string, string> = {
    GA: 'cessna-172-skyhawk',
    airliner: 'boeing-737-800',
    military: 'boeing-737-800',
    helicopter: '',
    glider: '',
  };

  // Helicopter (generic)
  const helicopterSvg = `<svg viewBox="0 0 200 80" fill="none">
    <!-- Body -->
    <ellipse cx="70" cy="40" rx="30" ry="12" stroke="currentColor" stroke-width="1.2" fill="currentColor" fill-opacity="0.06"/>
    <!-- Tail boom -->
    <path d="M100 39 L170 36 L170 44 L100 41Z" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.04"/>
    <!-- Tail rotor -->
    <line x1="170" y1="28" x2="170" y2="52" stroke="currentColor" stroke-width="1" opacity="0.4"/>
    <!-- Main rotor -->
    <line x1="20" y1="22" x2="120" y2="22" stroke="currentColor" stroke-width="0.8" opacity="0.3"/>
    <line x1="70" y1="22" x2="70" y2="28" stroke="currentColor" stroke-width="1"/>
    <!-- Skids -->
    <line x1="50" y1="52" x2="90" y2="52" stroke="currentColor" stroke-width="0.8" opacity="0.4"/>
    <line x1="55" y1="48" x2="55" y2="52" stroke="currentColor" stroke-width="0.6" opacity="0.3"/>
    <line x1="85" y1="48" x2="85" y2="52" stroke="currentColor" stroke-width="0.6" opacity="0.3"/>
  </svg>`;

  // Glider
  const gliderSvg = `<svg viewBox="0 0 200 80" fill="none">
    <path d="M45 40 Q45 38 55 37 L165 37 Q180 37 182 40 L182 40 Q180 43 165 43 L55 43 Q45 42 45 40Z" stroke="currentColor" stroke-width="1" fill="currentColor" fill-opacity="0.05"/>
    <!-- Very long wings -->
    <path d="M75 37 L5 18 Q2 17 3 14 L5 13 L130 13 L132 14 Q133 17 130 18 L100 37" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
    <path d="M75 43 L5 62 Q2 63 3 66 L5 67 L130 67 L132 66 Q133 63 130 62 L100 43" stroke="currentColor" stroke-width="0.8" fill="currentColor" fill-opacity="0.03"/>
    <!-- T-tail -->
    <line x1="178" y1="37" x2="186" y2="37" stroke="currentColor" stroke-width="1" opacity="0.4"/>
    <path d="M183 37 L180 31 L192 31 L189 37" stroke="currentColor" stroke-width="0.6" fill="currentColor" fill-opacity="0.02"/>
    <path d="M183 43 L180 49 L192 49 L189 43" stroke="currentColor" stroke-width="0.6" fill="currentColor" fill-opacity="0.02"/>
  </svg>`;

  function getSvg(): string {
    const id = aircraftId?.toLowerCase() || '';

    // Try specific aircraft first
    if (specificAircraft[id] && specificAircraft[id] !== '') return specificAircraft[id];

    // Check if it maps to another aircraft
    for (const [key, svg] of Object.entries(specificAircraft)) {
      if (id.includes(key.split('-')[0]) && svg !== '') return svg;
    }

    // Fallback by category
    const cat = category?.toLowerCase() || 'ga';
    if (cat === 'helicopter') return helicopterSvg;
    if (cat === 'glider') return gliderSvg;

    const fallbackId = categoryFallback[cat];
    if (fallbackId && specificAircraft[fallbackId]) return specificAircraft[fallbackId];

    return specificAircraft['cessna-172-skyhawk'];
  }
</script>

<div class="silhouette" class:silhouette-active={active}>
  {@html getSvg()}
</div>

<style>
  .silhouette {
    width: 100%;
    max-width: 180px;
    color: var(--text-dim, #4a5068);
    opacity: 0.35;
    transition: all 0.3s ease;
  }

  .silhouette-active {
    color: var(--accent, #00aaff);
    opacity: 0.85;
    filter: drop-shadow(0 0 10px rgba(0, 170, 255, 0.25));
  }

  .silhouette :global(svg) {
    width: 100%;
    height: auto;
  }
</style>
