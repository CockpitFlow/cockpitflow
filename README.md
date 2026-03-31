# CockpitFlow

Open-source cockpit companion for flight simulators. Custom cockpit layouts, checklists, hardware integration. X-Plane & MSFS.

## Features

- **Checklist** — Pre-flight to shutdown procedures. Strict/Smart modes, auto-detect from sim data
- **Controls** — Touch cockpit on phone/tablet. Throttle, mixture, flaps, yoke, switches
- **Nav** — Weight & balance, crosswind calculator, fuel planner, density altitude, V-speeds
- **Debrief** — Landing rate analysis, G-forces, speed envelope, performance scoring
- **Failures** — Arm engine, electrical, instrument failures with configurable timing
- **Hardware** — Map Arduino/ESP32 pins to sim functions (Beta)
- **Community** — Landing leaderboard, pilot profile, shared presets

## Stack

- **Desktop**: Tauri 2 + Svelte 5 + Rust + Tailwind CSS
- **Mobile**: Capacitor + Android (native gyro sensor)
- **Sim**: X-Plane UDP datarefs/commands
- **Data**: JSON presets for checklists, aircraft, cockpit layouts

## Getting Started

### Desktop App

```bash
cd cockpitflow
npm install
npm run tauri dev
```

### Mobile App

```bash
cd cockpitflow-mobile
npm install
npx cap sync android
# Open in Android Studio or:
cd android && ./gradlew assembleDebug
```

### Website

```bash
cd cockpitflow-site
# Static files — open index.html or push to GitHub Pages
```

## Project Structure

```
cockpitflow/
├── src/
│   ├── App.svelte              # Main app shell (sidebar, routing)
│   ├── lib/
│   │   ├── sections/           # Page sections (Tailwind, zero custom CSS)
│   │   │   ├── HomeSection.svelte
│   │   │   ├── ChecklistSection.svelte
│   │   │   ├── NavSection.svelte
│   │   │   ├── DebriefSection.svelte
│   │   │   ├── ScenariosSection.svelte
│   │   │   ├── HardwareSection.svelte
│   │   │   └── CommunitySection.svelte
│   │   ├── gauges/             # Reusable components (instruments, toast, etc)
│   │   ├── CockpitBuilder.svelte
│   │   ├── ModuleManager.svelte
│   │   └── Marketplace.svelte
│   ├── app.css                 # Tailwind + CSS variables (dark/light/night themes)
│   └── main.ts
├── public/
│   ├── cockpit.html            # Standalone mobile cockpit controls
│   ├── checklist.html          # Standalone mobile checklist
│   ├── cf-theme.css            # Shared theme (used by all pages)
│   ├── cf-widgets.js           # Web components for cockpit renderer
│   ├── data/
│   │   ├── failures.json       # Failure definitions
│   │   └── hardware-functions.json
│   └── modules/                # Module presets (checklist, cockpit, nav, etc)
├── src-tauri/
│   ├── src/lib.rs              # Rust backend (UDP, HTTP server, serial)
│   └── Cargo.toml
cockpitflow-mobile/
├── www/index.html              # Mobile app UI
├── android/                    # Android project
│   └── app/src/main/java/.../
│       ├── MainActivity.java
│       └── GyroPlugin.java     # Native gyro sensor → HTTP
└── capacitor.config.json
cockpitflow-site/               # GitHub Pages website
```

## Architecture

Desktop app runs a local HTTP server (port 8080) that:
- Receives sim data via X-Plane UDP
- Serves cockpit/checklist pages to mobile devices on LAN
- Provides REST API (`/api/sim-state`, `/api/command`, `/api/settings`)

Mobile app connects to desktop via LAN, loads cockpit controls, and sends commands back. Gyro data is read by a native Java plugin and sent directly to the server via HTTP POST.

## Themes

Three themes: Dark (default), Light, Night (extra dim for cockpit use). Toggle via sidebar button. All components use CSS variables from `app.css`.

## License

Open source. For flight simulation use only. Not for real-world aviation.
